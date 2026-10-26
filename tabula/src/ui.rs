//! Drawing: the board of two rows of twelve points, the men stacked on
//! them, the men still in hand and those borne off, the dice, the cursor
//! and the spots lit for a choice, a side panel, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Layout, Rules, Side};
use ancient_core::ornament;
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const FRAME: Color = Color::Rgb(70, 44, 26);
const FIELD: Color = Color::Rgb(196, 168, 120);
const POINT_A: Color = Color::Rgb(150, 66, 40);
const POINT_B: Color = Color::Rgb(70, 82, 60);
const WHITE: Color = Color::Rgb(250, 246, 236);
const BLACK: Color = Color::Rgb(30, 24, 22);
const PICKED: Color = Color::Rgb(232, 200, 110);
const MARK: Color = Color::Rgb(255, 214, 90);

/// Where each spot was drawn, for mouse clicks and the cursor: 0 the
/// hand, 1 to 24 the points, 25 off.
#[derive(Default, Clone)]
pub struct Hits {
    pub spots: Vec<(Rect, u8)>,
}

impl Hits {
    /// The spot next to `from` in direction (dx, dy) on the screen.
    pub fn neighbour(&self, from: u8, (dx, dy): (i32, i32)) -> Option<u8> {
        let (r, _) = self.spots.iter().find(|(_, s)| *s == from)?;
        let (cx, cy) = (r.x as i32 + r.width as i32 / 2, r.y as i32 + r.height as i32 / 2);
        self.spots
            .iter()
            .filter(|(q, s)| {
                *s != from && {
                    let (qx, qy) = (q.x as i32 + q.width as i32 / 2, q.y as i32 + q.height as i32 / 2);
                    (qx - cx) * dx > 0 || (qy - cy) * dy > 0
                }
            })
            .min_by_key(|(q, _)| {
                let (qx, qy) = (q.x as i32 + q.width as i32 / 2, q.y as i32 + q.height as i32 / 2);
                let along = (qx - cx) * dx + (qy - cy) * dy * 3;
                let across = ((qx - cx) * dy).abs() + ((qy - cy) * dx).abs() * 3;
                along + across * 2
            })
            .map(|(_, s)| *s)
    }
}

const PANEL: u16 = 34;

/// The scale of the board: 1 on the screen the game is made for, larger
/// on a larger one. A point is `2s + 2` columns wide, and a row of points
/// `5s` rows deep (`3s` on the three-row board).
fn scale(area: Rect, three: bool) -> u16 {
    let by_height = if three { area.height.saturating_sub(12) / 9 } else { area.height.saturating_sub(13) / 10 };
    let by_width = area.width.saturating_sub(66) / 24;
    by_height.min(by_width).clamp(1, 4)
}

fn man_colour(side: Side) -> Color {
    match side {
        Side::White => WHITE,
        Side::Black => BLACK,
    }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.spots.clear();
    let rules = &game.rules;
    let three = rules.layout == Layout::ThreeRows;
    // Rows of twelve points, each split six and six by the bar. Two rows
    // (tabula) or three (duodecim scripta); between rows a gap, which in
    // the three-row board carries the middle row's numbers.
    let s = scale(area, three);
    let col = 2 * s + 2;
    let (rows, stack, sep) = if three { (3u16, 3 * s, 1u16) } else { (2, 5 * s, 3) };
    let bw = 12 * col + 3 + 2;
    let bh = rows * stack + (rows - 1) * sep + 2;
    let total = bw + 3 + PANEL;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh + 8) / 2 + 3;
    fill(buf, Rect::new(bx, by, bw, bh), FRAME);
    fill(buf, Rect::new(bx + 1, by + 1, bw - 2, bh - 2), FIELD);
    let shown = game.shown();
    let lit = game.lit();
    let lit_spots: Vec<u8> = lit.iter().map(|&s| game.spot(s)).collect();
    let selected = game.selected.map(|s| game.spot(s));
    let to_move = game.position.to_move;

    // Point p's column, its row from the top, and whether its men stack
    // down from the row's top. Tabula: the top row runs 13 to 24 from the
    // left, the bottom row 12 to 1. Duodecim scripta: 1 to 12 along the
    // middle row from the left, 13 to 24 back along the top, 25 to 36
    // along the bottom.
    let place = |p: u8| -> (u16, u16, bool) {
        let (i, row) = if three {
            match p {
                1..=12 => (p - 1, 1),
                13..=24 => (24 - p, 0),
                _ => (p - 25, 2),
            }
        } else if p > 12 {
            (p - 13, 0)
        } else {
            (12 - p, 1)
        };
        let gap = if i >= 6 { 3 } else { 0 };
        let x = bx + 1 + i as u16 * col + gap;
        let y0 = by + 1 + row * (stack + sep);
        (x, y0, row + 1 < rows)
    };
    // The bar between the halves.
    let bar_x = bx + 1 + 6 * col;
    for y in by + 1..by + bh - 1 {
        put(buf, bar_x, y, "   ", Style::new().bg(FRAME));
    }
    for p in 1..=rules.len {
        let (x, y0, down) = place(p);
        let colour = if p % 2 == 0 { POINT_A } else { POINT_B };
        // The row of position k from the base of the point.
        let at = |k: u16| if down { y0 + k } else { y0 + stack - 1 - k };
        // The middle column of the point.
        let cx = x + s;
        // The point itself, a narrowing wedge.
        for k in 0..stack {
            let tip = k + 1 == stack;
            let style = Style::new().fg(colour).bg(FIELD);
            match (tip, down) {
                (false, _) if s == 1 => put(buf, x, at(k), "▐█▌ ", style),
                (false, _) => {
                    // A triangle, in half columns: as wide as the point at
                    // the base, narrowing evenly to the tip, about the
                    // middle of column `cx`.
                    let wide = 2 * s + 1;
                    let half = (wide * (stack - k)).div_ceil(stack);
                    let (from, to) = (2 * cx + 1 - half, 2 * cx + 1 + half);
                    for c in from / 2..to.div_ceil(2) {
                        let glyph = match (2 * c >= from, 2 * c + 1 < to) {
                            (true, true) => "█",
                            (true, false) => "▌",
                            (false, true) => "▐",
                            (false, false) => " ",
                        };
                        put(buf, c, at(k), glyph, style);
                    }
                }
                (true, true) => put(buf, cx, at(k), "▾", style),
                (true, false) => put(buf, cx, at(k), "▴", style),
            }
        }
        let men = [shown.on_point(rules, Side::White, p), shown.on_point(rules, Side::Black, p)];
        let side = if men[0] > 0 { Some(Side::White) } else if men[1] > 0 { Some(Side::Black) } else { None };
        let count = men[0].max(men[1]) as u16;
        let picked = selected == Some(p);
        for k in 0..count.min(stack) {
            let last = k + 1 == stack.min(count);
            let fg = man_colour(side.unwrap());
            let bg = if picked && last { PICKED } else { colour };
            let style = Style::new().fg(fg).bg(bg).add_modifier(Modifier::BOLD);
            if k + 1 == stack && count > stack {
                put(buf, cx, at(k), &format!("{:>2}", count), style);
            } else if s == 1 {
                put(buf, cx, at(k), "●", style);
            } else {
                // A man seen from the side, a row of a stack.
                let man = "▀".repeat(2 * s as usize - 1);
                let fg = if picked && last { PICKED } else { fg };
                put(buf, cx + 1 - s, at(k), &man, style.fg(fg));
            }
        }
        if lit_spots.contains(&p) {
            let style = Style::new().fg(MARK).bg(FIELD).add_modifier(Modifier::BOLD);
            if count == 0 || game.selected.is_some() {
                put(buf, cx, at(count.min(stack - 1)), "◆", style);
            } else {
                put(buf, x, at(0), "▸", style);
            }
        }
        let label_y = if down { y0 - 1 } else { y0 + stack };
        let label_bg = if label_y == by || label_y == by + bh - 1 { FRAME } else { FIELD };
        put(buf, cx, label_y, &format!("{p:<2}"), Style::new().fg(DIM).bg(label_bg));
        hits.spots.push((Rect::new(x, y0, col, stack), p));
    }

    // The hand, above the board, and the men off, below it.
    let tray = |buf: &mut Buffer, y: u16, label: &str, step: u8, spot: u8, hits: &mut Hits| {
        put(buf, bx, y, label, Style::new().fg(DIM).bg(BG));
        let mut x = bx + label.chars().count() as u16 + 1;
        for side in Side::BOTH {
            let k = shown.men[side.index()][step as usize];
            let style = Style::new().fg(man_colour(side)).bg(Color::Rgb(90, 80, 70)).add_modifier(Modifier::BOLD);
            put(buf, x, y, &format!(" ● {k:>2} "), style);
            if side == to_move {
                hits.spots.push((Rect::new(x, y, 7, 1), spot));
                if lit_spots.contains(&spot) {
                    put(buf, x + 7, y, "◆", Style::new().fg(MARK).bg(BG));
                }
                if selected == Some(spot) {
                    put(buf, x, y, " ●", style.bg(PICKED));
                }
            }
            x += 10;
        }
    };
    tray(buf, by - 2, "In hand:", 0, 0, hits);
    tray(buf, by + bh + 1, "Borne off:", rules.off(), rules.off(), hits);

    if game.choosing()
        && let Some((r, _)) = hits.spots.iter().find(|(_, s)| *s == game.cursor)
    {
        {
            let style = Style::new().fg(AMBER).add_modifier(Modifier::BOLD);
            put(buf, r.x.saturating_sub(1), r.y + r.height / 2, "▶", style);
        }
    }
    draw_panel(buf, Rect::new(bx + bw + 3, by - 2, PANEL, bh + 4), game);
    draw_margins(buf, area, by + bh / 2, bx.saturating_sub(1), bx + bw + 3 + PANEL);
}

/// On a terminal larger than the game is made for, the dividing marks of
/// the marble board given by Flavius Photius at Aphrodisias (Roueche,
/// InsAph 69): six-petalled compass rosettes in circles, a half rosette
/// above and below a whole one, down each margin.
fn draw_margins(buf: &mut Buffer, area: Rect, middle: u16, left_end: u16, right_start: u16) {
    if !ornament::roomy(area) {
        return;
    }
    let marble = ornament::dim(FIELD, BG, 0.55);
    // Three marks, as on the stone, each in eight rows, about the board's
    // middle.
    let h = 24.min(area.height);
    let top = middle.saturating_sub(h / 2).max(area.y);
    let left = Rect::new(area.x, top, left_end.saturating_sub(area.x), h);
    let right = Rect::new(right_start, top, area.right().saturating_sub(right_start), h);
    for column in [left, right] {
        ornament::stack(buf, column, 25, 25, |i, x, y| {
            // A half (the lower half, petals up, as on the stone) above and
            // below the whole.
            let mark = if i % 2 == 0 { y >= -6 && compass(x, y + 6) } else { compass(x, y) };
            mark.then_some(marble)
        });
    }
}

/// A compass rosette of radius 12 dots: the circle, and inside it the
/// six arcs of the same radius centred on it, which make six petals.
fn compass(x: i32, y: i32) -> bool {
    const R: f32 = 12.0;
    let (x, y) = (x as f32, y as f32);
    let on = |cx: f32, cy: f32| ((((x - cx).powi(2) + (y - cy).powi(2)).sqrt()) - R).abs() < 0.6;
    let inside = x * x + y * y <= R * R + 0.5;
    on(0.0, 0.0)
        || inside
            && (0..6).any(|k| {
                let a = std::f32::consts::FRAC_PI_3 * k as f32;
                on(R * a.sin(), -R * a.cos())
            })
}

fn die_face(d: u8) -> &'static str {
    ["⚀", "⚁", "⚂", "⚃", "⚄", "⚅"][(d.clamp(1, 6) - 1) as usize]
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "TABULA", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    let over = matches!(game.phase, Phase::Over { .. });
    let shown = game.shown();
    for side in Side::BOTH {
        let who = match game.player(side) {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        let active = side == game.position.to_move && !over;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        put(buf, x + 2, y, "●", Style::new().fg(if side == Side::White { WHITE } else { Color::Rgb(120, 110, 100) }).bg(BG));
        put(buf, x + 4, y, &format!("{} ({who})", side.name()), style);
        y += 1;
        put(buf, x + 4, y, &format!("{} steps to go", shown.pips(&game.rules, side)), Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    // The dice: used ones dimmed.
    let showing = !matches!(game.phase, Phase::ToThrow | Phase::Over { .. });
    if showing {
        let left = game.dice_left();
        let mut pool = left.clone();
        let mut dx = x;
        for &d in &game.dice {
            let unused = pool.iter().position(|&q| q == d).map(|i| pool.remove(i)).is_some();
            let style = if matches!(game.phase, Phase::Throwing { .. }) {
                Style::new().fg(DIM).bg(BG)
            } else if unused {
                Style::new().fg(TEXT).bg(BG).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(DIM).bg(BG)
            };
            let face = if matches!(game.phase, Phase::Throwing { .. }) { "?".to_string() } else { format!("{} {d}", die_face(d)) };
            put(buf, dx, y, &face, style);
            dx += 5;
        }
    }
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if let Phase::Over { winner } = game.phase {
        let result = if winner == Side::White { "WHITE WINS" } else { "BLACK WINS" };
        put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() && game.phase == Phase::ToThrow {
        &["Enter or Space: throw", "M: menu   F1: help   Q: quit"]
    } else if game.choosing() {
        &["Arrows: move   Enter: choose", "Esc: let go, take back a hop", "Mouse: click   F1: help   Q: quit"]
    } else {
        &["F1: help   Q: quit"]
    };
    let mut ky = panel.bottom().saturating_sub(keys.len() as u16);
    for k in keys {
        put(buf, x, ky, k, Style::new().fg(DIM).bg(BG));
        ky += 1;
    }
}

pub fn menu(rules: &[Rules], chosen: usize) -> Menu<'_> {
    Menu {
        title: "TABULA",
        subtitle: "The Roman game of tables, ancestor of backgammon.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["White, who throws first", "Black"],
    }
}

/// The short form of the game's rulebook chapter (its "brief" blocks), as
/// plain text: the F1 help. The full chapter is RULES.txt in the release
/// archives. Both are written by `rulebook/build.sh`.
pub const HELP: &str = include_str!("../HELP.txt");

pub fn help(rules: &Rules) -> (Vec<Help>, usize) {
    let lines = shell::rules_help(HELP);
    let at = shell::help_find(&lines, rules.section).unwrap_or(0);
    (lines, at)
}

#[cfg(test)]
mod help_tests {
    use super::*;

    #[test]
    fn the_help_opens_at_every_rule_set() {
        let lines = shell::rules_help(HELP);
        for rules in Rules::all() {
            assert!(shell::help_find(&lines, rules.section).is_some(), "{}", rules.section);
        }
    }

    #[test]
    fn the_rulebook_text_fits_the_help() {
        for line in HELP.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }
}
