//! Drawing: the papamū, a grid of hollows in grey lava, the black lava
//! and white coral stones, the cursor and the hollows lit for a choice, a
//! side panel, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Move, Rules, Side};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const LAVA: Color = Color::Rgb(70, 68, 66);
const HOLLOW: Color = Color::Rgb(40, 38, 37);
const BLACK: Color = Color::Rgb(14, 12, 12);
const CORAL: Color = Color::Rgb(244, 236, 220);
const MARK: Color = Color::Rgb(220, 70, 50);
const PICKED: Color = Color::Rgb(120, 96, 50);

/// Where each hollow was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, usize)>,
}

/// Columns and rows from one hollow to the next.
fn spacing(rules: &Rules, area: Rect) -> (u16, u16) {
    let tall = rules.rows as u16 * 2 + 3 <= area.height;
    let wide = rules.cols as u16 * 4 + 44 <= area.width;
    if tall && wide { (4, 2) } else { (3, 1) }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let rules = &game.rules;
    let (sx, sy) = spacing(rules, area);
    let bw = rules.cols as u16 * sx + 3;
    let bh = rules.rows as u16 * sy + 3 - sy;
    let panel_w = 34u16;
    let total = bw + 3 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2 + 2;
    let by = area.y + area.height.saturating_sub(bh + 1) / 2 + 1;
    fill(buf, Rect::new(bx, by, bw, bh), LAVA);
    let at = |p: usize| {
        let (r, c) = rules.rc(p);
        (bx + 2 + c as u16 * sx, by + 1 + r as u16 * sy)
    };
    // The names of the columns and rows, as the messages use them.
    let label = Style::new().fg(DIM).bg(BG);
    for c in 0..rules.cols {
        let (x, _) = at(c);
        put(buf, x, by - 1, &((b'a' + c as u8) as char).to_string(), label);
    }
    for r in 0..rules.rows {
        let (_, y) = at(r * rules.cols);
        put(buf, bx.saturating_sub(3), y, &format!("{:>2}", r + 1), label);
    }

    let lit = game.lit();
    // The stones a jump would take, when the cursor is on its landing.
    let doomed: Vec<usize> = match game.selected {
        Some(s) => game
            .position
            .moves(rules)
            .into_iter()
            .find(|m| matches!(m, Move::Jump { from, path } if *from == s && path.last() == Some(&game.cursor)))
            .map(|m| m.taken(rules))
            .unwrap_or_default(),
        None => Vec::new(),
    };
    for p in 0..rules.len() {
        let (x, y) = at(p);
        hits.points.push((Rect::new(x.saturating_sub(1), y, 3, 1), p));
        let bg = if game.selected == Some(p) {
            PICKED
        } else if lit.contains(&p) && game.position.at[p].is_some() {
            Color::Rgb(100, 90, 70)
        } else {
            LAVA
        };
        let (glyph, style) = match game.position.at[p] {
            Some(_) if doomed.contains(&p) => ("●", Style::new().fg(MARK).bg(bg)),
            Some(Side::Black) => ("●", Style::new().fg(BLACK).bg(bg)),
            Some(Side::White) => ("●", Style::new().fg(CORAL).bg(bg)),
            None if lit.contains(&p) => ("◆", Style::new().fg(MARK).bg(bg).add_modifier(Modifier::BOLD)),
            None => ("•", Style::new().fg(HOLLOW).bg(bg)),
        };
        put(buf, x, y, glyph, style.add_modifier(Modifier::BOLD));
    }
    if let Some(Move::Jump { path, .. }) = &game.last
        && game.selected.is_none()
    {
        let (x, y) = at(*path.last().unwrap());
        put(buf, x - 1, y, "·", Style::new().fg(DIM).bg(LAVA));
        put(buf, x + 1, y, "·", Style::new().fg(DIM).bg(LAVA));
    }
    if game.human_to_move() {
        let (x, y) = at(game.cursor);
        let style = Style::new().fg(AMBER).bg(LAVA).add_modifier(Modifier::BOLD);
        put(buf, x - 1, y, "[", style);
        put(buf, x + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 3, by, panel_w, bh.max(16)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "KŌNANE", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
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
        let colour = if side == Side::Black { Color::Rgb(110, 100, 92) } else { CORAL };
        put(buf, x + 2, y, "●", Style::new().fg(colour).bg(BG));
        let stones = game.position.at.iter().filter(|s| **s == Some(side)).count();
        put(buf, x + 4, y, &format!("{} ({who}), {stones}", side.name()), style);
        y += 1;
    }
    y += 1;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { winner } => {
            let result = if *winner == Side::Black { "BLACK WINS" } else { "WHITE WINS" };
            put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
        Phase::Turn => {}
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &["Arrows: move the cursor", "Enter: lift, pick up, land", "Esc: let go   Mouse: click", "F1: help   Q: quit"]
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
        title: "KŌNANE",
        subtitle: "The Hawaiian game of jumping stones.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Black, who begins", "White"],
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
        for line in HELP.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }
}
