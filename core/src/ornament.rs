//! Ornament drawn in braille: each braille character (U+2800 to U+28FF) is
//! a grid of 2 by 4 dots in one cell, and in most fonts the dots are close
//! to square, so a pattern is worked out in dot space, as a function from
//! a dot to on or off, and drawn cell by cell in one colour per cell.
//!
//! Ornament only goes on blank cells, so it never covers a piece, a
//! highlight or a number drawn before it. `--plain` or the O key turns it
//! off (some fonts draw braille thin or spaced apart).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::sync::atomic::{AtomicU8, Ordering};

/// 0 not yet read from the command line, 1 on, 2 off.
static STATE: AtomicU8 = AtomicU8::new(0);

/// Whether ornament is drawn: on unless `--plain` or turned off with O.
pub fn on() -> bool {
    match STATE.load(Ordering::Relaxed) {
        0 => {
            let on = !crate::shell::flag("--plain");
            STATE.store(if on { 1 } else { 2 }, Ordering::Relaxed);
            on
        }
        s => s == 1,
    }
}

/// Turns ornament on or off.
pub fn toggle() {
    STATE.store(if on() { 2 } else { 1 }, Ordering::Relaxed);
}

/// The bit of each dot in a braille character, by column and row.
const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];

/// The braille character for the cell whose top left dot is (`x`, `y`) in
/// dot space, or none if no dot is on.
pub fn cell(x: i32, y: i32, dot: &impl Fn(i32, i32) -> bool) -> Option<char> {
    let mut bits = 0;
    for (dx, column) in BITS.iter().enumerate() {
        for (dy, bit) in column.iter().enumerate() {
            if dot(x + dx as i32, y + dy as i32) {
                bits |= bit;
            }
        }
    }
    (bits != 0).then(|| char::from_u32(0x2800 + bits as u32).unwrap())
}

/// Draws the pattern `dot` over `area` in `colour`, dot (0, 0) at the
/// area's top left, on the blank cells only; each cell keeps its
/// background. Does nothing when ornament is off.
pub fn paint(buf: &mut Buffer, area: Rect, colour: Color, dot: impl Fn(i32, i32) -> bool) {
    paint_in(buf, area, |x, y| dot(x, y).then_some(colour));
}

/// As `paint`, each dot in its own colour; a cell takes the colour most of
/// its dots have.
pub fn paint_in(buf: &mut Buffer, area: Rect, dot: impl Fn(i32, i32) -> Option<Color>) {
    if !on() {
        return;
    }
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let at = &mut buf[(x, y)];
            if at.symbol() != " " {
                continue;
            }
            let (dx, dy) = ((x - area.x) as i32 * 2, (y - area.y) as i32 * 4);
            let mut bits = 0;
            let mut colours: Vec<(Color, usize)> = Vec::new();
            for (ox, column) in BITS.iter().enumerate() {
                for (oy, bit) in column.iter().enumerate() {
                    let Some(c) = dot(dx + ox as i32, dy + oy as i32) else { continue };
                    bits |= bit;
                    match colours.iter_mut().find(|(k, _)| *k == c) {
                        Some((_, n)) => *n += 1,
                        None => colours.push((c, 1)),
                    }
                }
            }
            if bits != 0 {
                // The first colour wins a tie.
                let best = colours.iter().fold(colours[0], |a, &b| if b.1 > a.1 { b } else { a });
                let ch = char::from_u32(0x2800 + bits as u32).unwrap();
                at.set_char(ch).set_fg(best.0);
            }
        }
    }
}

/// `colour` brought toward `bg` by `t` (0 the colour, 1 the background):
/// ornament in dim tones of a game's own colours.
pub fn dim(colour: Color, bg: Color, t: f32) -> Color {
    match (colour, bg) {
        (Color::Rgb(r, g, b), Color::Rgb(br, bgg, bb)) => {
            let mix = |a: u8, z: u8| (a as f32 + (z as f32 - a as f32) * t).round() as u8;
            Color::Rgb(mix(r, br), mix(g, bgg), mix(b, bb))
        }
        _ => colour,
    }
}

/// A rosette of `petals` petals of length `r` dots around its centre, the
/// first pointing up: for the dot (`x`, `y`) from the centre, `Some(0)` in
/// the centre's disc, `Some(1 + k)` in petal k, or none.
pub fn rosette(petals: usize, r: f32, x: i32, y: i32) -> Option<usize> {
    let (x, y) = (x as f32, y as f32);
    let d = (x * x + y * y).sqrt();
    if d <= r * 0.16 {
        return Some(0);
    }
    if d > r {
        return None;
    }
    // Each petal a pointed lens from the centre's disc to its tip.
    let step = std::f32::consts::TAU / petals as f32;
    let angle = x.atan2(-y).rem_euclid(std::f32::consts::TAU);
    let k = ((angle + step / 2.0) / step) as usize % petals;
    let off = angle - k as f32 * step;
    let (along, across) = (d * off.cos(), d * off.sin());
    let (mid, half) = (r * 0.55, r * 0.45);
    let width = r * 0.26;
    let u = (along - mid) / half;
    (u * u + (across / width).powi(2) <= 1.0).then_some(1 + k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn dots_make_braille() {
        assert_eq!(cell(0, 0, &|_, _| false), None);
        assert_eq!(cell(0, 0, &|_, _| true), Some('⣿'));
        assert_eq!(cell(0, 0, &|x, y| x == 0 && y == 0), Some('⠁'));
        assert_eq!(cell(0, 0, &|x, y| x == 1 && y == 3), Some('⢀'));
        // The second cell starts at dot column 2.
        assert_eq!(cell(2, 0, &|x, _| x == 2), Some('⡇'));
    }

    #[test]
    fn a_rosette_has_its_petals_round_a_centre() {
        assert_eq!(rosette(8, 9.0, 0, 0), Some(0));
        assert_eq!(rosette(8, 9.0, 0, -6), Some(1));
        assert_eq!(rosette(8, 9.0, 6, 0), Some(3));
        assert_eq!(rosette(8, 9.0, 0, 6), Some(5));
        assert_eq!(rosette(8, 9.0, 0, -10), None);
        // Between two petals.
        assert_eq!(rosette(8, 9.0, 3, -7), None);
    }

    #[test]
    fn paint_leaves_what_is_drawn() {
        STATE.store(1, Ordering::Relaxed);
        let mut buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        buf[(1, 0)].set_char('A').set_style(Style::new().fg(Color::Red));
        paint(&mut buf, Rect::new(0, 0, 3, 1), Color::Blue, |_, _| true);
        assert_eq!(buf[(0, 0)].symbol(), "⣿");
        assert_eq!(buf[(1, 0)].symbol(), "A");
        assert_eq!(buf[(2, 0)].fg, Color::Blue);
    }
}
