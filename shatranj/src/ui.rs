//! Drawing: the board, a cloth of one colour ruled into squares as the
//! old boards were, unchequered; the men as letters in red and black, the
//! colours of the manuscripts; the cursor and the squares a man picked up
//! can go to; a side panel with the moves; the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Outcome, Rules, Side, fr};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const CLOTH: Color = Color::Rgb(214, 196, 158);
const CLOTH_LIT: Color = Color::Rgb(236, 220, 180);
const RULE: Color = Color::Rgb(150, 126, 88);
const PICKED: Color = Color::Rgb(240, 196, 100);
const RED: Color = Color::Rgb(176, 30, 22);
const INK: Color = Color::Rgb(22, 18, 16);
const MARK: Color = Color::Rgb(30, 110, 60);

#[derive(Default, Clone)]
pub struct Hits {
    pub squares: Vec<(Rect, usize)>,
}

const PANEL: u16 = 36;

/// A square: seven columns by three rows, or less on a small screen.
fn cell_size(area: Rect) -> (u16, u16) {
    let h = (area.height.saturating_sub(3) / 8).clamp(1, 3);
    let w = (area.width.saturating_sub(PANEL + 6) / 8).clamp(3, 7);
    (w, h)
}

fn colour(side: Side) -> Color {
    match side {
        Side::Red => RED,
        Side::Black => INK,
    }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.squares.clear();
    let (cw, ch) = cell_size(area);
    let bw = 8 * cw + 1;
    let bh = 8 * ch + 1;
    let total = bw + 3 + PANEL;
    let bx = area.x + area.width.saturating_sub(total) / 2 + 2;
    let by = area.y + area.height.saturating_sub(bh + 1) / 2;
    let targets = game.targets();
    let p = &game.position;
    // Rank 8 at the top: Red plays up the screen.
    let cell = |sq: usize| {
        let (f, r) = fr(sq);
        Rect::new(bx + f as u16 * cw, by + (7 - r) as u16 * ch, cw + 1, ch + 1)
    };
    fill(buf, Rect::new(bx, by, bw, bh), CLOTH);
    for sq in 0..64 {
        let rect = cell(sq);
        hits.squares.push((Rect::new(rect.x + 1, rect.y + 1, cw - 1, ch.max(1)), sq));
        let lit = targets.contains(&sq);
        let back = if game.selected == Some(sq) {
            PICKED
        } else if lit {
            CLOTH_LIT
        } else {
            CLOTH
        };
        fill(buf, Rect::new(rect.x + 1, rect.y + 1, cw - 1, ch.saturating_sub(1).max(1)), back);
        let mid_x = rect.x + cw / 2;
        let mid_y = rect.y + ch.div_ceil(2);
        match p.board[sq] {
            Some(m) => {
                let fg = if lit { MARK } else { colour(m.side) };
                put(buf, mid_x, mid_y, &m.kind.letter().to_string(), Style::new().fg(fg).bg(back).add_modifier(Modifier::BOLD));
            }
            None if lit => put(buf, mid_x, mid_y, "◆", Style::new().fg(MARK).bg(back)),
            None => {}
        }
    }
    // The rules of the cloth.
    let rule = Style::new().fg(RULE).bg(CLOTH);
    if ch > 1 {
        for i in 0..=8u16 {
            let y = by + i * ch;
            for x in bx..bx + bw {
                put(buf, x, y, "─", rule);
            }
            let x = bx + i * cw;
            for y in by..by + bh {
                put(buf, x, y, "│", rule);
            }
        }
        for i in 0..=8u16 {
            for j in 0..=8u16 {
                let glyph = match (i, j) {
                    (0, 0) => "┌",
                    (8, 0) => "┐",
                    (0, 8) => "└",
                    (8, 8) => "┘",
                    (_, 0) => "┬",
                    (_, 8) => "┴",
                    (0, _) => "├",
                    (8, _) => "┤",
                    _ => "┼",
                };
                put(buf, bx + i * cw, by + j * ch, glyph, rule);
            }
        }
    }
    // Where the last move went.
    if let Some(mv) = game.last
        && game.selected.is_none()
    {
        let r = cell(mv.to as usize);
        let style = Style::new().fg(MARK).bg(CLOTH);
        put(buf, r.x + 1, r.y + ch.div_ceil(2), "·", style);
        put(buf, r.x + cw - 1, r.y + ch.div_ceil(2), "·", style);
        let f = cell(mv.from as usize);
        put(buf, f.x + cw / 2, f.y + ch.div_ceil(2), "·", style);
    }
    if game.human_to_move() {
        let r = cell(game.cursor);
        let style = Style::new().fg(Color::Rgb(24, 64, 170)).bg(CLOTH).add_modifier(Modifier::BOLD);
        put(buf, r.x + 1, r.y + ch.div_ceil(2), "[", style);
        put(buf, r.x + cw - 1, r.y + ch.div_ceil(2), "]", style);
    }
    let label = Style::new().fg(DIM).bg(BG);
    for i in 0..8u16 {
        put(buf, bx + i * cw + cw / 2, by + bh, &((b'a' + i as u8) as char).to_string(), label);
        put(buf, bx.saturating_sub(2), by + (7 - i) * ch + ch.div_ceil(2), &format!("{}", i + 1), label);
    }
    draw_panel(buf, Rect::new(bx + bw + 3, by, PANEL, bh.max(20)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "SHATRANJ", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    let over = matches!(game.phase, Phase::Over { .. });
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
        let shown = if side == Side::Black { Color::Rgb(150, 140, 130) } else { colour(side) };
        put(buf, x + 2, y, "K", Style::new().fg(shown).bg(BG).add_modifier(Modifier::BOLD));
        put(buf, x + 4, y, &format!("{} ({who})", side.name()), style);
        y += 1;
    }
    put(buf, x, y, "K shah  F firzan  E fil  N faras", Style::new().fg(DIM).bg(BG));
    put(buf, x, y + 1, "R rukh  P baidaq", Style::new().fg(DIM).bg(BG));
    y += 3;
    // The last few moves, numbered in pairs.
    let rec = &game.record;
    let pairs: Vec<String> = rec
        .chunks(2)
        .enumerate()
        .map(|(i, c)| format!("{:>3}. {:<10} {}", i + 1, c[0], c.get(1).map_or("", |s| s.as_str())))
        .collect();
    for line in pairs.iter().skip(pairs.len().saturating_sub(4)) {
        put(buf, x, y, line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { outcome } => {
            let result = match outcome {
                Outcome::Win(Side::Red, _) => "RED WINS",
                Outcome::Win(Side::Black, _) => "BLACK WINS",
                Outcome::Draw(_) => "A DRAW",
            };
            put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
        Phase::Turn => {}
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &["Arrows: move the cursor", "Enter: pick up, put down", "Esc: let go   Mouse: click", "F1: help   Q: quit"]
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
        title: "SHATRANJ",
        subtitle: "The chess of Persia and the caliphate, before the queen and the bishop.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Red, who begins", "Black"],
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
