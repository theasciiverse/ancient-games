//! Pictures of the games for the rulebook: a game played forward on a
//! simulated clock, drawn without a terminal, and written out as SVG.
//! Nothing here depends on the real clock, so a picture is the same on
//! every build.

use crate::shell::Program;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};
use std::fmt::Write;
use std::time::{Duration, Instant};

/// The size every game is made for, and the size of the pictures.
pub const WIDTH: u16 = 100;
pub const HEIGHT: u16 = 30;

/// `--rulebook-screen RULES:SECONDS`: the rule set (its place in the
/// menu) and how long to play the demo before the picture.
pub fn screen_arg() -> Option<(usize, f64)> {
    let spec = crate::shell::arg("--rulebook-screen")?;
    let (rules, seconds) = spec.split_once(':')?;
    Some((rules.parse().ok()?, seconds.parse().ok()?))
}

/// Prints worked examples for `rulebook/build.sh`, each after a line
/// `=== name`.
pub fn print_examples(examples: &[(&str, String)]) {
    for (name, text) in examples {
        println!("=== {name}\n{text}");
    }
}

/// Plays `program` for `seconds` of simulated time, in frames of 33 ms
/// as the terminal loop does, then draws it.
pub fn play_and_draw(program: &mut impl Program, seconds: f64) -> Buffer {
    let start = Instant::now();
    let frames = (seconds * 1000.0 / 33.0) as u32;
    for i in 0..=frames {
        program.tick(start + Duration::from_millis(33 * i as u64));
    }
    draw(program)
}

/// Draws `program` into a buffer of the standard size.
pub fn draw(program: &mut impl Program) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("test backend");
    terminal.draw(|frame| program.draw(frame)).expect("draw");
    terminal.backend().buffer().clone()
}

/// The screen's text, one line per row, without colours.
pub fn text(buf: &Buffer) -> String {
    let area = buf.area;
    let mut out = String::new();
    for y in area.top()..area.bottom() {
        let line: String = (area.left()..area.right()).map(|x| buf[(x, y)].symbol()).collect();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// A colour as #rrggbb; `fg` says which default a reset colour takes.
fn hex(color: Color, fg: bool) -> String {
    let (r, g, b) = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Reset => if fg { (0xd0, 0xd0, 0xd0) } else { (0x10, 0x10, 0x14) },
        Color::Black => (0, 0, 0),
        Color::Red => (0xcd, 0, 0),
        Color::Green => (0, 0xcd, 0),
        Color::Yellow => (0xcd, 0xcd, 0),
        Color::Blue => (0, 0, 0xee),
        Color::Magenta => (0xcd, 0, 0xcd),
        Color::Cyan => (0, 0xcd, 0xcd),
        Color::Gray => (0xe5, 0xe5, 0xe5),
        Color::DarkGray => (0x7f, 0x7f, 0x7f),
        Color::LightRed => (0xff, 0, 0),
        Color::LightGreen => (0, 0xff, 0),
        Color::LightYellow => (0xff, 0xff, 0),
        Color::LightBlue => (0x5c, 0x5c, 0xff),
        Color::LightMagenta => (0xff, 0, 0xff),
        Color::LightCyan => (0, 0xff, 0xff),
        Color::White => (0xff, 0xff, 0xff),
        Color::Indexed(i) => indexed(i),
    };
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// The xterm 256-colour palette.
fn indexed(i: u8) -> (u8, u8, u8) {
    const BASE: [(u8, u8, u8); 16] = [
        (0, 0, 0), (0xcd, 0, 0), (0, 0xcd, 0), (0xcd, 0xcd, 0),
        (0, 0, 0xee), (0xcd, 0, 0xcd), (0, 0xcd, 0xcd), (0xe5, 0xe5, 0xe5),
        (0x7f, 0x7f, 0x7f), (0xff, 0, 0), (0, 0xff, 0), (0xff, 0xff, 0),
        (0x5c, 0x5c, 0xff), (0xff, 0, 0xff), (0, 0xff, 0xff), (0xff, 0xff, 0xff),
    ];
    match i {
        0..=15 => BASE[i as usize],
        16..=231 => {
            let i = i - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            (level(i / 36), level(i / 6 % 6), level(i % 6))
        }
        _ => {
            let v = 8 + (i - 232) * 10;
            (v, v, v)
        }
    }
}

/// The part of a cell a block character fills, as (x, y, w, h) in eighths
/// of the cell, so blocks join up whatever the font.
fn block(symbol: &str) -> Option<(u8, u8, u8, u8)> {
    Some(match symbol {
        "█" => (0, 0, 8, 8),
        "▀" => (0, 0, 8, 4),
        "▄" => (0, 4, 8, 4),
        "▌" => (0, 0, 4, 8),
        "▐" => (4, 0, 4, 8),
        "▁" => (0, 7, 8, 1),
        "▔" => (0, 0, 8, 1),
        _ => return None,
    })
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The screen as an SVG picture: every cell at its place on a grid of
/// 10 by 20 units, backgrounds as rectangles, block characters drawn as
/// rectangles, text in a monospaced font.
pub fn svg(buf: &Buffer) -> String {
    const CW: u32 = 10;
    const CH: u32 = 20;
    let area = buf.area;
    let (w, h) = (area.width as u32 * CW, area.height as u32 * CH);
    let mut out = String::new();
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" shape-rendering="crispEdges">"#
    );
    let _ = writeln!(out, r#"<rect width="{w}" height="{h}" fill="{}"/>"#, hex(Color::Reset, false));

    let colours = |x: u16, y: u16| {
        let cell = &buf[(x, y)];
        let (mut fg, mut bg) = (hex(cell.fg, true), hex(cell.bg, false));
        if cell.modifier.contains(Modifier::REVERSED) {
            std::mem::swap(&mut fg, &mut bg);
        }
        (fg, bg, cell.modifier)
    };

    // Backgrounds, in runs along each row.
    for y in area.top()..area.bottom() {
        let mut x = area.left();
        while x < area.right() {
            let (_, bg, _) = colours(x, y);
            let mut end = x + 1;
            while end < area.right() && colours(end, y).1 == bg {
                end += 1;
            }
            if bg != hex(Color::Reset, false) {
                let _ = writeln!(
                    out,
                    r#"<rect x="{}" y="{}" width="{}" height="{}.5" fill="{bg}"/>"#,
                    (x - area.left()) as u32 * CW,
                    (y - area.top()) as u32 * CH,
                    (end - x) as u32 * CW,
                    CH
                );
            }
            x = end;
        }
    }

    // Block characters as rectangles, other characters as text in runs of
    // one style, each character placed on its own cell.
    let _ = writeln!(
        out,
        r#"<g font-family="DejaVu Sans Mono, Menlo, Consolas, monospace" font-size="16" xml:space="preserve">"#
    );
    for y in area.top()..area.bottom() {
        let top = (y - area.top()) as u32 * CH;
        let mut run: Vec<(u32, String)> = Vec::new();
        let mut run_style: Option<(String, Modifier)> = None;
        let flush = |out: &mut String, run: &mut Vec<(u32, String)>, style: &Option<(String, Modifier)>| {
            if run.is_empty() {
                return;
            }
            let (fg, modifier) = style.clone().unwrap();
            let xs: Vec<String> = run.iter().map(|(x, _)| x.to_string()).collect();
            let text: String = run.iter().map(|(_, s)| escape(s)).collect();
            let weight = if modifier.contains(Modifier::BOLD) { r#" font-weight="bold""# } else { "" };
            let _ = writeln!(
                out,
                r#"<text x="{}" y="{}" fill="{fg}"{weight}>{text}</text>"#,
                xs.join(" "),
                top + 15
            );
            run.clear();
        };
        for x in area.left()..area.right() {
            let symbol = buf[(x, y)].symbol();
            let left = (x - area.left()) as u32 * CW;
            let (fg, _, modifier) = colours(x, y);
            if let Some((bx, by, bw, bh)) = block(symbol) {
                let _ = writeln!(
                    out,
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{fg}"/>"#,
                    left + bx as u32 * CW / 8,
                    top + by as u32 * CH / 8,
                    bw as u32 * CW / 8,
                    bh as u32 * CH / 8
                );
                continue;
            }
            if symbol.trim().is_empty() {
                continue;
            }
            let style = Some((fg, modifier));
            if style != run_style {
                flush(&mut out, &mut run, &run_style);
                run_style = style;
            }
            run.push((left, symbol.to_string()));
        }
        flush(&mut out, &mut run, &run_style);
    }
    out.push_str("</g>\n</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    #[test]
    fn svg_places_text_and_blocks() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
        buf[(0, 0)].set_char('A').set_style(Style::new().fg(Color::Rgb(255, 0, 0)));
        buf[(1, 0)].set_char('<');
        buf[(2, 1)].set_char('█').set_style(Style::new().fg(Color::Rgb(0, 0, 255)));
        let svg = svg(&buf);
        assert!(svg.contains(r##"fill="#ff0000">A</text>"##), "{svg}");
        assert!(svg.contains("&lt;"));
        assert!(svg.contains(r##"<rect x="20" y="20" width="10" height="20" fill="#0000ff"/>"##), "{svg}");
        assert_eq!(text(&buf), "A<\n  █\n");
    }
}
