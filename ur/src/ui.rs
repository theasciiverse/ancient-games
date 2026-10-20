//! Drawing: the board after the one from the royal tombs of Ur (lapis
//! lazuli, shell and red limestone), the pieces with their five dots, the
//! throwing lots, the menu and the help.

use crate::app::{Game, Phase, Player, Setup};
use crate::rules::{self, COLS, HOME, Move, ROWS, Rules, START, Side};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const LAPIS: Color = Color::Rgb(28, 52, 120);
const LAPIS_DARK: Color = Color::Rgb(16, 30, 74);
const SHELL: Color = Color::Rgb(214, 198, 160);
const SHELL_SHADE: Color = Color::Rgb(176, 158, 120);
const CARNELIAN: Color = Color::Rgb(178, 52, 40);
const LIGHT_PIECE: Color = Color::Rgb(250, 246, 236);
const DARK_PIECE: Color = Color::Rgb(36, 30, 38);
const AMBER: Color = Color::Rgb(255, 176, 0);
const TEXT: Color = Color::Rgb(200, 196, 186);
const DIM: Color = Color::Rgb(120, 116, 108);
const BG: Color = Color::Rgb(10, 10, 14);

/// Where things were drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    /// The board square at a grid cell, in screen coordinates.
    pub squares: Vec<(Rect, usize, usize)>,
}

/// The size of one board square for an area: about twice as wide as tall,
/// so squares look square.
fn square_size(area: Rect) -> (u16, u16) {
    let by_height = (area.height.saturating_sub(12) / ROWS as u16).clamp(3, 7);
    let by_width = (area.width.saturating_sub(6) / COLS as u16 / 2).clamp(3, 7);
    let h = by_height.min(by_width);
    (h * 2, h)
}

fn fill(buf: &mut Buffer, area: Rect, bg: Color) {
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            buf[(x, y)].set_char(' ').set_bg(bg).set_fg(TEXT);
        }
    }
}

fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
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

fn centre(buf: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let len = text.chars().count() as u16;
    let x = area.x + area.width.saturating_sub(len) / 2;
    put(buf, x, y, text, style);
}

/// Draws the whole game screen.
pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.squares.clear();

    let (sw, sh) = square_size(area);
    let board_w = sw * COLS as u16 + 2;
    let board_h = sh * ROWS as u16 + 2;
    let bx = area.x + area.width.saturating_sub(board_w) / 2;
    let top = area.y + area.height.saturating_sub(board_h + 9) / 2;
    let by = top + 3;

    // Title and the far side's name.
    centre(
        buf,
        area,
        top,
        "THE ROYAL GAME OF UR",
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD),
    );
    centre(
        buf,
        area,
        top + 1,
        &format!("{} rules", game.rules.name),
        Style::new().fg(DIM).bg(BG),
    );
    side_label(buf, bx, by - 1, game, Side::Dark);

    // The board body: lapis with a dark rim.
    fill(buf, Rect::new(bx, by, board_w, board_h), LAPIS_DARK);

    let choosing = match &game.phase {
        Phase::Choosing { moves, selected, .. } => Some((moves.as_slice(), *selected)),
        _ => None,
    };

    for row in 0..ROWS {
        for col in 0..COLS {
            let x = bx + 1 + col as u16 * sw;
            let y = by + 1 + row as u16 * sh;
            let cell = Rect::new(x, y, sw, sh);
            if rules::on_board(row, col) {
                draw_square(buf, cell, row, col);
                hits.squares.push((cell, row, col));
            } else {
                draw_tray(buf, cell, row, col, game);
                hits.squares.push((cell, row, col));
            }
        }
    }

    // Pieces on the board.
    for side in [Side::Light, Side::Dark] {
        for &pos in game.position.of(side) {
            if pos != START && pos != HOME {
                let (row, col) = rules::square(side, pos);
                draw_piece(buf, cell_of(bx, by, sw, sh, row, col), side, false);
            }
        }
    }

    // The moves to choose from, and the chosen one's target.
    if let Some((moves, selected)) = choosing {
        let side = game.to_move();
        for (i, mv) in moves.iter().enumerate() {
            let chosen = i == selected;
            let from = place_cell(bx, by, sw, sh, side, mv.from, true);
            outline(buf, from, if chosen { AMBER } else { SHELL_SHADE }, chosen);
            if chosen {
                let to = place_cell(bx, by, sw, sh, side, mv.to, false);
                outline(buf, to, AMBER, false);
                mark_target(buf, to, mv);
            }
        }
    } else if let Some((side, mv)) = game.last {
        // Where the last move went.
        let to = place_cell(bx, by, sw, sh, side, mv.to, false);
        outline(buf, to, DIM, false);
    }

    side_label(buf, bx, by + board_h, game, Side::Light);

    // The lots and the message under the board.
    let ly = by + board_h + 2;
    draw_lots(buf, area, ly, game);
    centre(
        buf,
        area,
        ly + 4,
        &game.message,
        Style::new().fg(TEXT).bg(BG),
    );
    let keys = match game.phase {
        Phase::Over { .. } => "Enter: play again   M: menu   F1: help   Q: quit",
        Phase::Choosing { .. } => "Arrows or Tab: choose   Enter: move   F1: help   Q: quit",
        _ if game.human_to_move() => "Space: throw   F1: help   Q: quit",
        _ => "F1: help   Q: quit",
    };
    centre(buf, area, ly + 5, keys, Style::new().fg(DIM).bg(BG));
}

fn cell_of(bx: u16, by: u16, sw: u16, sh: u16, row: usize, col: usize) -> Rect {
    Rect::new(bx + 1 + col as u16 * sw, by + 1 + row as u16 * sh, sw, sh)
}

/// The screen cell for a path position: a board square, or the side's tray
/// (start in the fifth column, home in the sixth).
fn place_cell(bx: u16, by: u16, sw: u16, sh: u16, side: Side, pos: u8, _from: bool) -> Rect {
    let own = match side {
        Side::Light => 2,
        Side::Dark => 0,
    };
    match pos {
        START => cell_of(bx, by, sw, sh, own, 4),
        HOME => cell_of(bx, by, sw, sh, own, 5),
        _ => {
            let (row, col) = rules::square(side, pos);
            cell_of(bx, by, sw, sh, row, col)
        }
    }
}

fn side_label(buf: &mut Buffer, bx: u16, y: u16, game: &Game, side: Side) {
    let who = match game.player(side) {
        Player::Human => "you".to_string(),
        Player::Computer(s) => format!("computer, {}", s.name()),
    };
    let to_move = game.to_move() == side && !matches!(game.phase, Phase::Over { .. });
    let marker = if to_move { "▶ " } else { "  " };
    let style = if to_move {
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(DIM).bg(BG)
    };
    let swatch = match side {
        Side::Light => LIGHT_PIECE,
        Side::Dark => DARK_PIECE,
    };
    put(buf, bx, y, marker, style);
    put(buf, bx + 2, y, "●", Style::new().fg(swatch).bg(SHELL_SHADE));
    put(buf, bx + 4, y, &format!("{} ({who})", side.name()), style);
}

/// A shell square with its inlay: rosettes in carnelian and lapis, the other
/// squares with dot and diamond motifs.
fn draw_square(buf: &mut Buffer, cell: Rect, row: usize, col: usize) {
    let inner = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    fill(buf, inner, SHELL);
    let cx = inner.x + inner.width / 2;
    let cy = inner.y + inner.height / 2;
    let red = Style::new().fg(CARNELIAN).bg(SHELL);
    let blue = Style::new().fg(LAPIS).bg(SHELL);
    if rules::is_rosette(row, col) {
        if inner.height >= 3 {
            put(buf, cx - 1, cy - 1, "╲│╱", blue);
            put(buf, cx - 1, cy, "─", blue);
            put(buf, cx, cy, "✿", red);
            put(buf, cx + 1, cy, "─", blue);
            put(buf, cx - 1, cy + 1, "╱│╲", blue);
        } else {
            put(buf, cx, cy, "✿", red);
        }
    } else if (row + col) % 2 == 0 {
        // Five dots, as on many of the board's squares.
        if inner.height >= 3 {
            put(buf, cx - 2, cy - 1, "•   •", blue);
            put(buf, cx, cy, "•", red);
            put(buf, cx - 2, cy + 1, "•   •", blue);
        } else {
            put(buf, cx - 1, cy, "•••", blue);
        }
    } else {
        put(buf, cx, cy, "◇", red);
    }
}

/// The gaps in the outer rows hold a side's waiting pieces (fifth column)
/// and the pieces that have gone home (sixth).
fn draw_tray(buf: &mut Buffer, cell: Rect, row: usize, col: usize, game: &Game) {
    let side = if row == 0 { Side::Dark } else { Side::Light };
    let (pos, label) = if col == 4 { (START, "to go") } else { (HOME, "home") };
    let count = game.position.count_at(side, pos);
    let inner = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    let colour = match side {
        Side::Light => LIGHT_PIECE,
        Side::Dark => DARK_PIECE,
    };
    let cy = inner.y + inner.height / 2;
    // Each piece a small chip in its own colour with a dot, a gap between.
    let dot = match side {
        Side::Light => DARK_PIECE,
        Side::Dark => LIGHT_PIECE,
    };
    // As many as fit inside the frame; the label gives the count.
    let shown = count.min((inner.width as usize).saturating_sub(2).div_ceil(2));
    let len = (shown * 2).saturating_sub(1) as u16;
    let x = inner.x + inner.width.saturating_sub(len) / 2;
    for i in 0..shown as u16 {
        put(buf, x + i * 2, cy.saturating_sub(1), "•", Style::new().fg(dot).bg(colour));
    }
    let text = format!("{count} {label}");
    let tx = inner.x + inner.width.saturating_sub(text.len() as u16) / 2;
    put(buf, tx, cy + 1, &text, Style::new().fg(SHELL_SHADE).bg(LAPIS_DARK));
}

/// A piece: a disc of shell or black stone with five dots.
fn draw_piece(buf: &mut Buffer, cell: Rect, side: Side, _ghost: bool) {
    let inner = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    let (bg, dot) = match side {
        Side::Light => (LIGHT_PIECE, DARK_PIECE),
        Side::Dark => (DARK_PIECE, LIGHT_PIECE),
    };
    // The piece covers the square's inlay.
    fill(buf, inner, SHELL);
    // A row free above and below, where the square's frame goes.
    let w = (inner.width - 2).min(7).max(3);
    let h = inner.height.saturating_sub(2).clamp(1, 3);
    let x = inner.x + (inner.width - w) / 2;
    let y = inner.y + (inner.height - h) / 2;
    fill(buf, Rect::new(x, y, w, h), bg);
    let style = Style::new().fg(dot).bg(bg);
    let cx = x + w / 2;
    match h {
        3 => {
            put(buf, cx - 2, y, "•   •", style);
            put(buf, cx, y + 1, "•", style);
            put(buf, cx - 2, y + 2, "•   •", style);
        }
        2 => {
            put(buf, cx - 2, y, "• • •", style);
            put(buf, cx - 1, y + 1, "• •", style);
        }
        _ => put(buf, cx - 1, y, "•••", style),
    }
}

/// A frame around a square, to show it can be chosen or where a piece goes.
fn outline(buf: &mut Buffer, cell: Rect, colour: Color, bold: bool) {
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

fn mark_target(buf: &mut Buffer, cell: Rect, mv: &Move) {
    let r = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    let label = if mv.to == HOME { "home" } else { "here" };
    let x = r.x + r.width.saturating_sub(label.len() as u16) / 2;
    put(buf, x, r.bottom() - 1, label, Style::new().fg(AMBER).bg(LAPIS_DARK));
}

/// The throwing lots: little pyramids, the marked tip white when up.
fn draw_lots(buf: &mut Buffer, area: Rect, y: u16, game: &Game) {
    let n = game.lots.len() as u16;
    let width = n * 6 + 8;
    let x0 = area.x + area.width.saturating_sub(width) / 2;
    let tumbling = matches!(game.phase, Phase::Throwing { .. });
    let thrown = !matches!(game.phase, Phase::ToThrow) || tumbling;
    for (i, &marked) in game.lots.iter().enumerate() {
        let x = x0 + i as u16 * 6;
        let tip = if thrown && marked {
            Style::new().fg(LIGHT_PIECE).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(DIM).bg(BG)
        };
        put(buf, x + 2, y, if thrown && marked { "●" } else { "·" }, tip);
        put(buf, x + 1, y + 1, "◢█◣", Style::new().fg(SHELL_SHADE).bg(BG));
        put(buf, x, y + 2, "◢███◣", Style::new().fg(SHELL_SHADE).bg(BG));
    }
    let result = match &game.phase {
        Phase::Throwing { .. } => "...".to_string(),
        Phase::ToThrow => "".to_string(),
        _ => format!("= {}", game.roll()),
    };
    put(
        buf,
        x0 + n * 6 + 1,
        y + 1,
        &result,
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD),
    );
}

/// The menu before a game.
pub fn draw_menu(buf: &mut Buffer, area: Rect, setup: &Setup, row: usize, rules: &[Rules]) {
    fill(buf, area, BG);
    let w = area.width.min(76);
    let x = area.x + (area.width - w) / 2;
    let mut y = area.y + area.height.saturating_sub(22) / 2;
    let panel = Rect::new(x, y, w, 1);

    centre(
        buf,
        panel,
        y,
        "THE ROYAL GAME OF UR",
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD),
    );
    y += 1;
    centre(
        buf,
        panel,
        y,
        "Ur, Mesopotamia, about 2600 BC. Rules from a Babylonian tablet of 177 BC.",
        Style::new().fg(DIM).bg(BG),
    );
    y += 3;

    let chosen = &rules[setup.rules];
    let items = [
        ("Rules", chosen.name.to_string()),
        ("Play against", Setup::OPPONENTS[setup.opponent].to_string()),
        (
            "You play",
            format!(
                "{} ({})",
                setup.side.name(),
                if setup.side == Side::Light { "near side" } else { "far side" }
            ),
        ),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let active = i == row;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x + 4, y, if active { "▶" } else { " " }, style);
        put(buf, x + 6, y, label, style);
        put(buf, x + 22, y, &format!("◀ {value} ▶"), style);
        y += 2;
    }

    // Where the chosen rules come from.
    y += 1;
    for line in wrap(chosen.source, (w - 8) as usize) {
        put(buf, x + 6, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }

    y += 2;
    let start = if row == 3 {
        Style::new().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(AMBER).bg(BG)
    };
    centre(buf, panel, y, "  Enter: play  ", start);
    y += 2;
    centre(
        buf,
        panel,
        y,
        "Up and down: choose   Left and right: change   F1: how to play   Q: quit",
        Style::new().fg(DIM).bg(BG),
    );
}

/// How to play, and what is known of the game, over the screen.
pub fn draw_help(buf: &mut Buffer, area: Rect, rules: &Rules) {
    let w = area.width.min(84);
    let h = area.height.min(32);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    let panel = Rect::new(x, y, w, h);
    fill(buf, panel, Color::Rgb(18, 18, 26));
    let title = Style::new().fg(AMBER).bg(Color::Rgb(18, 18, 26)).add_modifier(Modifier::BOLD);
    let text = Style::new().fg(TEXT).bg(Color::Rgb(18, 18, 26));
    let dim = Style::new().fg(DIM).bg(Color::Rgb(18, 18, 26));

    let mut lines: Vec<(String, Style)> = vec![
        ("HOW TO PLAY".into(), title),
        (String::new(), text),
        (
            "Race your pieces from the start, along your own row, down the shared middle \
             row and back onto your own row, then off the board. First with every piece \
             home wins."
                .into(),
            text,
        ),
        (
            format!(
                "Throw the lots: the number of marked tips up is how far you move, 0 to 4. \
                 {} pieces each. One piece per square.",
                rules.pieces
            ),
            text,
        ),
        (
            "Land on an enemy piece in the middle row and it goes back to the start. \
             To bear a piece off you need the exact throw."
                .into(),
            text,
        ),
        (
            format!(
                "Rosettes ✿ give you another throw.{}",
                if rules.middle_rosette_safe {
                    " A piece on the middle rosette cannot be taken."
                } else {
                    " No square is safe in the middle row."
                }
            ),
            text,
        ),
        (String::new(), text),
        ("KEYS".into(), title),
        (
            "Space or Enter: throw   Arrows or Tab: choose a piece   Enter: move   \
             Mouse: click a lit piece to move it   M: menu   Q: quit"
                .into(),
            text,
        ),
        (String::new(), text),
        ("WHAT WE KNOW".into(), title),
        (
            "The game was played across the Near East for three thousand years. Its rules \
             were lost until Irving Finkel of the British Museum read a clay tablet written \
             by the Babylonian scribe Itti-Marduk-balatu in 177 BC, catalogued for decades \
             as bird omens. The rules here are reconstructions from that tablet and the \
             board; where scholars differ, the menu lets you choose."
                .into(),
            dim,
        ),
        (String::new(), text),
        ("Esc or F1: back".into(), dim),
    ];
    let mut yy = y + 1;
    for (line, style) in lines.drain(..) {
        for part in wrap(&line, (w - 6) as usize) {
            if yy >= y + h - 1 {
                break;
            }
            put(buf, x + 3, yy, &part, style);
            yy += 1;
        }
        if line.is_empty() {
            yy += 0;
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
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
