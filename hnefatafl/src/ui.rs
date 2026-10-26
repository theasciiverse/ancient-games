//! Drawing: the board of squares in two woods with the throne and the
//! corners marked, the dark attackers, the light defenders and the gold
//! king, the cursor and the squares a picked piece can go to, a side panel,
//! the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Escape, Move, Piece, Rules, Side, Square};
use ancient_core::ornament;
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const FRAME: Color = Color::Rgb(40, 26, 16);
const WOOD_A: Color = Color::Rgb(150, 112, 70);
const WOOD_B: Color = Color::Rgb(138, 102, 62);
const KONAKIS: Color = Color::Rgb(96, 50, 34);
const PICKED: Color = Color::Rgb(186, 140, 60);
const ATTACKER: Color = Color::Rgb(28, 22, 20);
const DEFENDER: Color = Color::Rgb(240, 232, 214);
const KING: Color = Color::Rgb(232, 184, 48);
const BLOOD: Color = Color::Rgb(200, 50, 40);

/// Where each square was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub squares: Vec<(Rect, Square)>,
}

const PANEL: u16 = 34;

/// A square's width and height: twice as wide as tall, which looks square.
fn cell_size(area: Rect, size: usize) -> (u16, u16) {
    let n = size as u16;
    let by_height = area.height.saturating_sub(4) / n;
    let by_width = area.width.saturating_sub(PANEL + 4) / n / 2;
    let h = by_height.min(by_width).clamp(1, 5);
    (2 * h, h)
}

fn piece_colour(p: Piece) -> Color {
    match p {
        Piece::Attacker => ATTACKER,
        Piece::Defender => DEFENDER,
        Piece::King => KING,
    }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.squares.clear();
    let rules = &game.rules;
    let n = rules.size;
    let (cw, ch) = cell_size(area, n);
    let bw = n as u16 * cw + 2;
    let bh = n as u16 * ch + 2;
    let total = bw + 2 + PANEL;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    fill(buf, Rect::new(bx, by, bw, bh), FRAME);
    let cell = |s: Square| Rect::new(bx + 1 + s.1 as u16 * cw, by + 1 + s.0 as u16 * ch, cw, ch);

    let targets = game.targets();
    // What the move under the cursor would take.
    let doomed: Vec<Square> = match game.selected {
        Some(from) if targets.contains(&game.cursor) => {
            let mut next = game.position.clone();
            next.play(rules, Move { from, to: game.cursor }).captured
        }
        _ => Vec::new(),
    };

    for r in 0..n {
        for c in 0..n {
            let s = (r, c);
            let rect = cell(s);
            hits.squares.push((rect, s));
            let wood = if game.selected == Some(s) {
                PICKED
            } else if rules.restricted(s) {
                KONAKIS
            } else if (r + c) % 2 == 0 {
                WOOD_A
            } else {
                WOOD_B
            };
            fill(buf, rect, wood);
            let mid_x = rect.x + cw / 2 - 1;
            let mid_y = rect.y + (ch - 1) / 2;
            match game.position.at(s) {
                Some(p) => {
                    let colour = if doomed.contains(&s) { BLOOD } else { piece_colour(p) };
                    draw_piece(buf, rect, colour, wood, p == Piece::King);
                }
                None if targets.contains(&s) => {
                    put(buf, mid_x, mid_y, "◆◆", Style::new().fg(AMBER).bg(wood));
                }
                None if game.taken.contains(&s) => {
                    put(buf, mid_x, mid_y, "××", Style::new().fg(BLOOD).bg(wood).add_modifier(Modifier::BOLD));
                }
                None if rules.restricted(s) => {
                    put(buf, mid_x, mid_y, "╳╳", Style::new().fg(WOOD_B).bg(wood));
                }
                None => {}
            }
        }
    }

    // Where the last move came from and went.
    if let Some(mv) = game.last
        && game.selected.is_none()
    {
        let from = cell(mv.from);
        if game.position.at(mv.from).is_none() {
            put(buf, from.x + cw / 2 - 1, from.y + (ch - 1) / 2, "··", Style::new().fg(FRAME).bg(wood_at(rules, mv.from)));
        }
        frame(buf, cell(mv.to), Style::new().fg(FRAME));
    }
    if game.human_to_move() {
        frame(buf, cell(game.cursor), Style::new().fg(AMBER).add_modifier(Modifier::BOLD));
    }

    // The files and ranks, a to k and 1 to 11 from the bottom, as tafl
    // players write moves.
    for c in 0..n {
        let x = bx + 1 + c as u16 * cw + cw / 2 - 1;
        put(buf, x, by + bh - 1, &((b'a' + c as u8) as char).to_string(), Style::new().fg(DIM).bg(FRAME));
    }

    draw_panel(buf, Rect::new(bx + bw + 2, by, PANEL, bh.max(20)), game);
    draw_ornament(buf, area, Rect::new(bx, by, bw, bh), bx + bw + 2 + PANEL);
}

/// Ornament after two Viking-age boards. Round the frame (not the bottom,
/// where the files are), the side panels of the Ballinderry board
/// (National Museum of Ireland 1932:6533): chevrons between two rows of
/// pellets. On a terminal larger than the game is made for, in the
/// margins, the knotted squares of the Gokstad board as Nicolaysen drew
/// them in 1882: a square with an X knot and a pellet in each corner.
fn draw_ornament(buf: &mut Buffer, area: Rect, board: Rect, panel_end: u16) {
    let carved = ornament::dim(WOOD_A, FRAME, 0.45);
    // Pellets on the top and bottom dot rows, a chevron between.
    let chevrons = |x: i32, y: i32| match y {
        0 | 3 => x % 2 == 0,
        _ => (x % 4 < 2) == (y == 1),
    };
    ornament::band(buf, Rect::new(board.x + 1, board.y, board.width - 2, 1), carved, 4, chevrons);
    for x in [board.left(), board.right() - 1] {
        let side = Rect::new(x, board.y + 1, 1, board.height - 2);
        ornament::paint(buf, side, carved, |x, y| x == (y / 2) % 2);
    }
    if !ornament::roomy(area) {
        return;
    }
    let wood = ornament::dim(WOOD_A, BG, 0.55);
    let left = Rect::new(area.x, board.y, board.x - area.x, board.height);
    let right = Rect::new(panel_end, board.y, area.right().saturating_sub(panel_end), board.height);
    for column in [left, right] {
        ornament::stack(buf, column, 21, 21, |_, x, y| knot(x, y).then_some(wood));
    }
}

/// The Gokstad board's X knot, 21 dots square about its centre: a frame,
/// a pellet in each corner, two crossing bands drawn as double lines, and
/// the loops they make on each side as triangles pointing in.
fn knot(x: i32, y: i32) -> bool {
    let (ax, ay) = (x.abs(), y.abs());
    let frame = ax.max(ay) == 10;
    let pellet = (ax - 8).pow(2) + (ay - 8).pow(2) <= 2;
    let bands = (ax - ay).abs() == 2 && ax.max(ay) <= 6;
    // The loops: a triangle on each side, its point toward the centre.
    let (along, from) = if ax > ay { (ay, 8 - ax) } else { (ax, 8 - ay) };
    let triangle = (from == 0 && along <= 4) || (from >= 0 && from <= 4 && along == 4 - from);
    frame || pellet || bands || triangle
}

fn wood_at(rules: &Rules, s: Square) -> Color {
    if rules.restricted(s) {
        KONAKIS
    } else if (s.0 + s.1) % 2 == 0 {
        WOOD_A
    } else {
        WOOD_B
    }
}

/// Thin bars down both sides of a square.
fn frame(buf: &mut Buffer, rect: Rect, style: Style) {
    for y in rect.y..rect.bottom() {
        if let Some(cell) = buf.cell_mut((rect.x, y)) {
            cell.set_char('▏').set_fg(style.fg.unwrap_or(AMBER));
        }
        if let Some(cell) = buf.cell_mut((rect.right() - 1, y)) {
            cell.set_char('▕').set_fg(style.fg.unwrap_or(AMBER));
        }
    }
}

/// A round piece, drawn with half blocks in the middle of its square; the
/// king a little wider, with a cross when there is room.
fn draw_piece(buf: &mut Buffer, rect: Rect, colour: Color, wood: Color, king: bool) {
    let (cw, ch) = (rect.width, rect.height);
    if ch == 1 {
        put(buf, rect.x + cw / 2 - 1, rect.y, if king { "◆◆" } else { "●●" }, Style::new().fg(colour).bg(wood));
        return;
    }
    let w = cw - 2;
    let x = rect.x + (cw - w) / 2;
    // In half rows: from 1 to 2ch - 2, centred.
    for row in 0..ch {
        let top = 2 * row; // the upper half of this row
        let upper = top >= 1 && top <= 2 * ch - 2;
        let lower = top + 1 >= 1 && top + 1 <= 2 * ch - 2;
        let glyph = match (upper, lower) {
            (true, true) => "█",
            (false, true) => "▄",
            (true, false) => "▀",
            _ => continue,
        };
        for dx in 0..w {
            put(buf, x + dx, rect.y + row, glyph, Style::new().fg(colour).bg(wood));
        }
    }
    if king && ch >= 3 {
        let style = Style::new().fg(FRAME).bg(colour).add_modifier(Modifier::BOLD);
        put(buf, rect.x + cw / 2 - 1, rect.y + ch / 2, "✚", style);
    }
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    let title = match game.rules.id {
        "brandubh" => "BRANDUBH",
        "tablut" | "bell-tablut" => "TABLUT",
        _ => "HNEFATAFL",
    };
    put(buf, x, y, title, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    put(buf, x, y, game.rules.name, Style::new().fg(DIM).bg(BG));
    y += 2;
    let to_move = game.position.to_move;
    let over = matches!(game.phase, Phase::Over { .. });
    for side in [Side::Attackers, Side::Defenders] {
        let who = match game.player(side) {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        let active = side == to_move && !over;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        let (colour, name, start) = match side {
            Side::Attackers => (ATTACKER, "Attackers", game.rules.attackers.len()),
            Side::Defenders => (DEFENDER, "Defenders", game.rules.defenders.len() - 1),
        };
        put(buf, x + 2, y, "●", Style::new().fg(colour).bg(Color::Rgb(150, 112, 70)));
        put(buf, x + 4, y, &format!("{name} ({who})"), style);
        y += 1;
        let left = game
            .position
            .board
            .iter()
            .flatten()
            .filter(|p| p.side() == side && **p != Piece::King)
            .count();
        let mut tally = format!("    {left} of {start} left");
        if side == Side::Defenders {
            tally.push_str(if game.position.king().is_some() { ", and the king" } else { ", the king taken" });
        }
        put(buf, x, y, &tally, Style::new().fg(DIM).bg(BG));
        y += 2;
    }
    let goal = match game.rules.escape {
        Escape::Corners => "The king escapes to a corner.",
        Escape::Edge => "The king escapes to the edge.",
    };
    put(buf, x, y, goal, Style::new().fg(DIM).bg(BG));
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { winner } => {
            let result = match winner {
                Some(Side::Attackers) => "THE ATTACKERS WIN",
                Some(Side::Defenders) => "THE DEFENDERS WIN",
                None => "A DRAW",
            };
            put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
        Phase::Turn => {}
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &[
            "Arrows: move the cursor",
            "Enter: pick up, put down",
            "Esc: drop   Mouse: click",
            "F1: help   Q: quit",
        ]
    } else {
        &["F1: help   Q: quit"]
    };
    let mut ky = panel.bottom().saturating_sub(keys.len() as u16);
    for k in keys {
        put(buf, x, ky, k, Style::new().fg(DIM).bg(BG));
        ky += 1;
    }
}

/// The menu's text.
pub fn menu(rules: &[Rules], chosen: usize) -> Menu<'_> {
    Menu {
        title: "HNEFATAFL",
        subtitle: "The Norse game of the king, and its kin from Ireland to Lapland.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["the attackers", "the king's side"],
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
        // The help is 84 columns wide with 3 on each side. Only a single
        // word, a web address, may be wider (the help runs it on).
        for line in HELP.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }
}
