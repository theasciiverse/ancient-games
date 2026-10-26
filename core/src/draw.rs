//! Drawing helpers shared by the games: text, fills and frames straight
//! into a Ratatui buffer, and the colours every game shares.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

/// Highlights, titles and the side to move.
pub const AMBER: Color = Color::Rgb(255, 176, 0);
/// Ordinary text.
pub const TEXT: Color = Color::Rgb(200, 196, 186);
/// Hints, keys and quieter text.
pub const DIM: Color = Color::Rgb(120, 116, 108);
/// The screen behind everything.
pub const BG: Color = Color::Rgb(10, 10, 14);

/// Fills `area` with blanks on `bg`.
pub fn fill(buf: &mut Buffer, area: Rect, bg: Color) {
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_char(' ').set_bg(bg).set_fg(TEXT);
        }
    }
}

/// Writes `text` from (`x`, `y`), clipped to the buffer.
pub fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    if y >= buf.area.bottom() {
        return;
    }
    let mut x = x;
    for ch in text.chars() {
        if x >= buf.area.right() {
            break;
        }
        if x >= buf.area.left() {
            buf[(x, y)].set_char(ch).set_style(style);
        }
        x += 1;
    }
}

/// Writes `text` centred in `area`'s width on row `y`.
pub fn centre(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let len = text.chars().count() as u16;
    let x = area.x + area.width.saturating_sub(len) / 2;
    put(buf, x, y, text, style);
}

/// A frame around a square, to show it can be chosen or where a piece goes.
pub fn outline(buf: &mut Buffer, cell: Rect, colour: Color, bold: bool) {
    let r = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    let style = if bold {
        Style::new().fg(colour).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(colour)
    };
    let (tl, tr, bl, br, hz, vt) = if bold {
        ("┏", "┓", "┗", "┛", "━", "┃")
    } else {
        ("┌", "┐", "└", "┘", "─", "│")
    };
    let right = r.right() - 1;
    let bottom = r.bottom() - 1;
    for x in r.left()..=right {
        for y in [r.top(), bottom] {
            let ch = match (x, y) {
                (x, y) if x == r.left() && y == r.top() => tl,
                (x, y) if x == right && y == r.top() => tr,
                (x, y) if x == r.left() && y == bottom => bl,
                (x, _) if x == right => br,
                _ => hz,
            };
            buf[(x, y)].set_symbol(ch).set_style(style);
        }
    }
    for y in r.top() + 1..bottom {
        buf[(r.left(), y)].set_symbol(vt).set_style(style);
        buf[(right, y)].set_symbol(vt).set_style(style);
    }
}

/// A round disc in `colour` about the cell (`x`, `y`), `rx` columns and `ry`
/// rows from the middle to the edge (rx about twice ry looks round). Its
/// edge is worked in half blocks, so it is drawn at twice the rows; cells
/// keep their background where the disc does not cover them.
pub fn disc(buf: &mut Buffer, x: u16, y: u16, rx: f32, ry: f32, colour: Color) {
    let inside = |dx: f32, dy: f32| (dx / rx).powi(2) + (dy / ry).powi(2) <= 1.0;
    let (w, h) = (rx.ceil() as i32, ry.ceil() as i32);
    for dy in -h..=h {
        for dx in -w..=w {
            let (cx, cy) = (x as i32 + dx, y as i32 + dy);
            if cx < 0 || cy < 0 || !buf.area.contains((cx as u16, cy as u16).into()) {
                continue;
            }
            let (top, bottom) = (inside(dx as f32, dy as f32 - 0.25), inside(dx as f32, dy as f32 + 0.25));
            let cell = &mut buf[(cx as u16, cy as u16)];
            match (top, bottom) {
                (true, true) => {
                    cell.set_char(' ').set_bg(colour);
                }
                (true, false) => {
                    cell.set_char('▀').set_fg(colour);
                }
                (false, true) => {
                    cell.set_char('▄').set_fg(colour);
                }
                _ => {}
            }
        }
    }
}

/// Breaks `text` into lines of at most `width` characters.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    // Runs of spaces inside a line are kept (they set keys apart); at a
    // break they go.
    let mut gap = 0;
    for word in text.split(' ') {
        if word.is_empty() {
            gap += 1;
            continue;
        }
        let sep = gap + 1;
        gap = 0;
        if !line.is_empty() && line.chars().count() + sep + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push_str(&" ".repeat(sep));
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
