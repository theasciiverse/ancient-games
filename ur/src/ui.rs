//! Drawing: the board after the one from the royal tombs of Ur (lapis
//! lazuli, shell and red limestone), the pieces with their five dots, the
//! throwing lots, the menu and the help.

use crate::app::{Game, Phase};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, centre, fill, outline, put};
use ancient_core::ornament;
use ancient_core::shell::{self, Help, Menu};
use ancient_core::player::Player;
use crate::rules::{Board, Lots, Move, ROWS, Rules, START, Side, TABLET_TAGS};
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

/// Where things were drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    /// The board square at a grid cell, in screen coordinates.
    pub squares: Vec<(Rect, usize, usize)>,
}

/// The size of one board square for an area: about twice as wide as tall,
/// so squares look square.
fn square_size(area: Rect, board: Board) -> (u16, u16) {
    let cols = board.cols() as u16;
    // The board plus eleven rows: title, names, lots, message and keys.
    let by_height = (area.height.saturating_sub(13) / ROWS as u16).clamp(3, 7);
    let width = (area.width.saturating_sub(2) / cols).clamp(6, 14);
    let h = by_height.min(width.div_ceil(2));
    (width.min(h * 2), h)
}

/// The tray for a side's waiting pieces (`home` false) or finished ones:
/// the gap squares of the Ur board, or four cells each beside the short
/// rows of Twenty Squares.
fn tray_cols(board: Board, home: bool) -> (usize, usize) {
    match (board, home) {
        (Board::Ur, false) => (4, 1),
        (Board::Ur, true) => (5, 1),
        (Board::TwentySquares, false) => (4, 4),
        (Board::TwentySquares, true) => (8, 4),
    }
}




/// Draws the whole game screen.
pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.squares.clear();

    let board = game.rules.board;
    let home = game.rules.home();
    let (sw, sh) = square_size(area, board);
    let cols = board.cols();
    let board_w = sw * cols as u16 + 2;
    let board_h = sh * ROWS as u16 + 2;
    let bx = area.x + area.width.saturating_sub(board_w) / 2;
    // Three rows above the board, eight below: names, lots, message, keys.
    let top = area.y + area.height.saturating_sub(board_h + 11) / 2;
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
        for col in 0..cols {
            let x = bx + 1 + col as u16 * sw;
            let y = by + 1 + row as u16 * sh;
            let cell = Rect::new(x, y, sw, sh);
            if board.on_board(row, col) {
                draw_square(buf, cell, row, col, board);
                hits.squares.push((cell, row, col));
            } else {
                for home in [false, true] {
                    let (first, span) = tray_cols(board, home);
                    if col == first {
                        let tray = Rect::new(x, y, sw * span as u16, sh);
                        draw_tray(buf, tray, row, home, game);
                    }
                    if (first..first + span).contains(&col) {
                        hits.squares.push((cell, row, first));
                    }
                }
            }
        }
    }

    // Pieces on the board.
    for side in [Side::Light, Side::Dark] {
        for (piece, &pos) in game.position.of(side).iter().enumerate() {
            if pos != START && pos != home {
                let (row, col) = board.square(side, pos);
                let name = game.rules.piece_name(piece).map(|n| (n, TABLET_TAGS[piece]));
                draw_piece(buf, cell_of(bx, by, sw, sh, row, col), side, name);
            }
        }
    }

    // The moves to choose from, and the chosen one's target.
    if let Some((moves, selected)) = choosing {
        let side = game.to_move();
        for (i, mv) in moves.iter().enumerate() {
            let chosen = i == selected;
            let from = place_cell(bx, by, sw, sh, board, side, mv.from);
            outline(buf, from, if chosen { AMBER } else { SHELL_SHADE }, chosen);
            if chosen {
                let to = place_cell(bx, by, sw, sh, board, side, mv.to);
                outline(buf, to, AMBER, false);
                mark_target(buf, to, mv, home);
            }
        }
    } else if let Some((side, mv)) = game.last {
        // Where the last move went.
        let to = place_cell(bx, by, sw, sh, board, side, mv.to);
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
        Phase::Choosing { can_dare: true, .. } => {
            "D: dare the ox bone   Arrows: choose   Enter: move   F1: help   Q: quit"
        }
        Phase::Choosing { .. } => "Arrows or Tab: choose   Enter: move   F1: help   Q: quit",
        _ if game.human_to_move() => "Space: throw   F1: help   Q: quit",
        _ => "F1: help   Q: quit",
    };
    centre(buf, area, ly + 5, keys, Style::new().fg(DIM).bg(BG));

    // Last, so it only goes where nothing else is.
    let board_area = Rect::new(bx, by, board_w, board_h);
    draw_rim(buf, board_area);
    draw_margins(buf, area, board_area);
}

/// The board's border after the original in the British Museum: a row of
/// small shell lozenges along the edge (a bead of shell on the short ends).
fn draw_rim(buf: &mut Buffer, board: Rect) {
    let shell = ornament::dim(SHELL, LAPIS_DARK, 0.45);
    // Lozenges three dots wide, three apart.
    let lozenge = |x: i32, y: i32| match y % 4 {
        0 | 3 => x == 2,
        _ => (1..=3).contains(&x),
    };
    ornament::rim(buf, board, shell, 6, lozenge, |_, y| matches!(y % 4, 1 | 2));
}

/// On a terminal larger than the game is made for, the board's rosettes
/// in the margins beside it: eight petals, red and blue in turn, a red
/// centre, as inlaid on the five rosette squares.
fn draw_margins(buf: &mut Buffer, area: Rect, board: Rect) {
    if !ornament::roomy(area) {
        return;
    }
    let red = ornament::dim(CARNELIAN, BG, 0.5);
    let blue = ornament::dim(Color::Rgb(70, 100, 190), BG, 0.5);
    for column in ornament::beside(area, board) {
        // Petals of up to fourteen dots, with two cells free on each side.
        let r = (column.width as i32 - 4).min(14);
        let petal = |x, y| ornament::rosette(8, r as f32, x, y);
        // The petals in outline, as incised in the shell; the centre whole.
        ornament::stack(buf, column, 2 * r + 1, 2 * r + 1, |_, x, y| match petal(x, y)? {
            0 => Some(red),
            _ if !ornament::edge(x, y, petal) => None,
            k if k % 2 == 0 => Some(blue),
            _ => Some(red),
        });
    }
}

fn cell_of(bx: u16, by: u16, sw: u16, sh: u16, row: usize, col: usize) -> Rect {
    Rect::new(bx + 1 + col as u16 * sw, by + 1 + row as u16 * sh, sw, sh)
}

/// The screen cell for a path position: a board square, or the side's tray
/// (start in the fifth column, home in the sixth, off the outer rows on
/// both boards).
fn place_cell(bx: u16, by: u16, sw: u16, sh: u16, board: Board, side: Side, pos: u8) -> Rect {
    let own = side.row();
    if pos == START || pos == board.home() {
        let (first, span) = tray_cols(board, pos != START);
        let cell = cell_of(bx, by, sw, sh, own, first);
        Rect::new(cell.x, cell.y, sw * span as u16, sh)
    } else {
        let (row, col) = board.square(side, pos);
        cell_of(bx, by, sw, sh, row, col)
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
    let name = format!("{} ({who})", side.name());
    put(buf, bx + 4, y, &name, style);
    // A betting game: the purse, and the pool by the far side's name.
    if game.rules.stakes.is_some() {
        let purse = format!("   purse {}", game.purses[side.index()]);
        put(buf, bx + 4 + name.chars().count() as u16, y, &purse, Style::new().fg(AMBER).bg(BG));
        if side == Side::Dark {
            let pool = format!("pool {}", game.pool);
            let board = game.rules.board;
            let width = (board.cols() as u16) * square_size(buf.area, board).0 + 2;
            let x = bx + width.saturating_sub(pool.len() as u16);
            put(buf, x, y, &pool, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
    }
}

/// A shell square with its inlay: rosettes in carnelian and lapis, the other
/// squares with dot and diamond motifs.
fn draw_square(buf: &mut Buffer, cell: Rect, row: usize, col: usize, board: Board) {
    let inner = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    fill(buf, inner, SHELL);
    let cx = inner.x + inner.width / 2;
    let cy = inner.y + inner.height / 2;
    let red = Style::new().fg(CARNELIAN).bg(SHELL);
    let blue = Style::new().fg(LAPIS).bg(SHELL);
    if board.is_rosette(row, col) {
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
fn draw_tray(buf: &mut Buffer, cell: Rect, row: usize, home: bool, game: &Game) {
    let side = if row == 0 { Side::Dark } else { Side::Light };
    let (pos, label) = if home { (game.rules.home(), "home") } else { (START, "to go") };
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
    // Distinct pieces (the tablet's birds) show their tags.
    if game.rules.distinct() {
        let tags: Vec<&str> = game
            .position
            .of(side)
            .iter()
            .enumerate()
            .filter(|&(_, &p)| p == pos)
            .map(|(i, _)| TABLET_TAGS[i])
            .collect();
        let len = (tags.len() * 3).saturating_sub(1) as u16;
        let tx = inner.x + inner.width.saturating_sub(len) / 2;
        for (i, tag) in tags.iter().enumerate() {
            put(buf, tx + i as u16 * 3, cy.saturating_sub(1), tag, Style::new().fg(dot).bg(colour));
        }
    } else {
        for i in 0..shown as u16 {
            put(buf, x + i * 2, cy.saturating_sub(1), "•", Style::new().fg(dot).bg(colour));
        }
    }
    let text = format!("{count} {label}");
    let tx = inner.x + inner.width.saturating_sub(text.len() as u16) / 2;
    put(buf, tx, cy + 1, &text, Style::new().fg(SHELL_SHADE).bg(LAPIS_DARK));
}

/// A piece: a disc of shell or black stone with five dots, or a bird of the
/// tablet with its name.
fn draw_piece(buf: &mut Buffer, cell: Rect, side: Side, name: Option<(&str, &str)>) {
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
    if let Some((name, tag)) = name {
        // The full name where it fits, else the tag.
        let label = if name.len() <= w as usize { name } else { tag };
        let lx = x + w.saturating_sub(label.chars().count() as u16) / 2;
        put(buf, lx, y + (h - 1) / 2, label, style.add_modifier(Modifier::BOLD));
        return;
    }
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


fn mark_target(buf: &mut Buffer, cell: Rect, mv: &Move, home: u8) {
    let r = Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1);
    let label = if mv.to == home { "home" } else { "here" };
    let x = r.x + r.width.saturating_sub(label.len() as u16) / 2;
    put(buf, x, r.bottom() - 1, label, Style::new().fg(AMBER).bg(LAPIS_DARK));
}

/// The throwing lots: little pyramids, the marked tip white when up; or the
/// tablet's two knucklebones.
fn draw_lots(buf: &mut Buffer, area: Rect, y: u16, game: &Game) {
    if game.rules.lots == Lots::Astragals {
        draw_bones(buf, area, y, game);
        return;
    }
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
        _ => {
            let throw = game.shown;
            if throw.again {
                format!("= {}, again", throw.moves)
            } else {
                format!("= {}", throw.moves)
            }
        }
    };
    put(
        buf,
        x0 + n * 6 + 1,
        y + 1,
        &result,
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD),
    );
}

/// The sheep and ox knucklebones: the sheep bone's face, and the ox bone's
/// answer when it has been dared.
fn draw_bones(buf: &mut Buffer, area: Rect, y: u16, game: &Game) {
    let x0 = area.x + area.width.saturating_sub(44) / 2;
    let bone = Style::new().fg(BG).bg(SHELL);
    let label = Style::new().fg(DIM).bg(BG);
    let tumbling = matches!(game.phase, Phase::Throwing { .. });
    let sheep = if matches!(game.phase, Phase::ToThrow) {
        "  ".to_string()
    } else {
        format!("{}", game.sheep)
    };
    put(buf, x0, y, "sheep", label);
    put(buf, x0, y + 1, &format!(" ({sheep:^3}) "), bone);
    let daring = matches!(game.phase, Phase::Daring { .. });
    let ox = match game.ox {
        None => "   ".to_string(),
        Some(true) => "yes".to_string(),
        Some(false) => "no".to_string(),
    };
    let ox_style = if game.ox.is_some() || daring {
        Style::new().fg(BG).bg(SHELL_SHADE).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(DIM).bg(LAPIS_DARK)
    };
    put(buf, x0 + 12, y, "ox", label);
    put(buf, x0 + 12, y + 1, &format!(" ( {ox:^3} ) "), ox_style);
    let result = match &game.phase {
        _ if tumbling || daring => "...".to_string(),
        Phase::ToThrow => String::new(),
        _ => format!("= {}", game.shown.moves),
    };
    put(
        buf,
        x0 + 26,
        y + 1,
        &result,
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD),
    );
}

/// The menu's text.
pub fn menu(rules: &[Rules], chosen: usize) -> Menu<'_> {
    Menu {
        title: "THE ROYAL GAME OF UR",
        subtitle: "Ur, Mesopotamia, about 2600 BC. Rules from a Babylonian tablet of 177 BC.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Light (near side)", "Dark (far side)"],
    }
}

/// The short form of the game's rulebook chapter (its "brief" blocks), as
/// plain text: the F1 help. The full chapter is RULES.txt in the release
/// archives. Both are written by `rulebook/build.sh`.
pub const HELP: &str = include_str!("../HELP.txt");

/// How to play: the rules in short, and the line where the rule set
/// `rules` begins.
pub fn help(rules: &Rules) -> (Vec<Help>, usize) {
    let lines = shell::rules_help(HELP);
    let at = shell::help_find(&lines, rules.section).unwrap_or(0);
    (lines, at)
}


#[cfg(test)]
mod tests {
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
        // The help is 84 columns wide with 3 on each side. Only a single
        // word, a web address, may be wider (the help runs it on).
        for line in HELP.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }
}
