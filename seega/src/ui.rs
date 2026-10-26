//! Drawing: the seega, a square of holes in the beaten earth, the kelbs
//! of grey stone and of red brick, the cursor and the holes lit for a
//! choice, a side panel, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Move, Outcome, QUIET, Rules, Side, Stage};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, disc, fill, put, wrap};
use ancient_core::ornament;
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const EARTH: Color = Color::Rgb(150, 118, 78);
const HOLE: Color = Color::Rgb(92, 68, 42);
const STONE: Color = Color::Rgb(205, 200, 188);
const BRICK: Color = Color::Rgb(178, 52, 34);
const MARK: Color = Color::Rgb(255, 214, 90);
const PICKED: Color = Color::Rgb(112, 86, 50);
const LIT: Color = Color::Rgb(176, 144, 96);

/// Where each hole was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, usize)>,
}

fn colour(side: Side) -> Color {
    match side {
        Side::Stone => STONE,
        Side::Brick => BRICK,
    }
}

const PANEL: u16 = 36;

/// The scale of the board: 1 on the screen the game is made for (holes a
/// glyph each, four columns and two rows apart), on a larger one holes
/// drawn as discs `2s + 1` wide, `4s` columns and `2s` rows apart.
fn scale(area: Rect, n: u16) -> u16 {
    if !ornament::roomy(area) {
        return 1;
    }
    let by_height = area.height.saturating_sub(3) / (2 * n);
    let by_width = area.width.saturating_sub(PANEL + 8) / (4 * n);
    by_height.min(by_width).clamp(1, 4)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let rules = &game.rules;
    let n = rules.size as u16;
    let s = scale(area, n);
    let (sx, sy) = (4 * s, 2 * s);
    // A hole's reach from its middle: columns and rows.
    // A disc's reach: most of the room between holes (4s by 2s), with a
    // column free on each side for the brackets.
    let (hw, hh) = if s > 1 { ((8 * s + 2) / 5, 3 * s / 4) } else { (0, 0) };
    let bw = (n - 1) * sx + 2 * hw + 7;
    let bh = (n - 1) * sy + 2 * hh + 3;
    let panel_w = PANEL;
    let total = bw + 3 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2 + 2;
    let by = area.y + area.height.saturating_sub(bh + 1) / 2 + 1;
    fill(buf, Rect::new(bx, by, bw, bh), EARTH);
    let at = |p: usize| {
        let (r, c) = rules.rc(p);
        (bx + 3 + hw + c as u16 * sx, by + 1 + hh + r as u16 * sy)
    };
    // The names of the columns and rows, as the messages use them.
    let label = Style::new().fg(DIM).bg(BG);
    for c in 0..rules.size {
        let (x, _) = at(c);
        put(buf, x, by - 1, &((b'a' + c as u8) as char).to_string(), label);
    }
    for r in 0..rules.size {
        let (_, y) = at(r * rules.size);
        put(buf, bx.saturating_sub(3), y, &format!("{:>2}", r + 1), label);
    }

    let lit = game.lit();
    // The kelbs a step would take, when the cursor is on where it goes.
    let doomed: Vec<usize> = match game.selected {
        Some(s) if lit.contains(&game.cursor) => game.position.takes(rules, s, game.cursor),
        _ => Vec::new(),
    };
    for p in 0..rules.len() {
        let (x, y) = at(p);
        hits.points.push((Rect::new(x - hw - 1, y - hh, 2 * hw + 3, 2 * hh + 1), p));
        let bg = if game.selected == Some(p) {
            PICKED
        } else if lit.contains(&p) && game.position.at[p].is_some() {
            LIT
        } else {
            EARTH
        };
        let (glyph, style) = match game.position.at[p] {
            Some(_) if doomed.contains(&p) => ("✕", Style::new().fg(MARK).bg(bg)),
            Some(side) => ("●", Style::new().fg(colour(side)).bg(bg)),
            None if lit.contains(&p) => ("◆", Style::new().fg(MARK).bg(bg)),
            None if game.emptied.contains(&p) => ("·", Style::new().fg(MARK).bg(bg)),
            None if p == rules.centre() => ("◯", Style::new().fg(HOLE).bg(bg)),
            None => ("●", Style::new().fg(HOLE).bg(bg)),
        };
        if s > 1 {
            // A hole in the earth, a disc: the kelb in it fills it.
            let fill_colour = match game.position.at[p] {
                Some(side) if !doomed.contains(&p) => colour(side),
                _ => HOLE,
            };
            fill(buf, Rect::new(x - hw - 1, y - hh, 2 * hw + 3, 2 * hh + 1), bg);
            disc(buf, x, y, hw as f32 + 0.4, hh as f32 + 0.4, fill_colour);
            let mark = match game.position.at[p] {
                Some(_) if doomed.contains(&p) => Some(("✕", MARK)),
                None if lit.contains(&p) => Some(("◆", MARK)),
                None if game.emptied.contains(&p) => Some(("·", MARK)),
                None if p == rules.centre() => Some(("◯", EARTH)),
                _ => None,
            };
            if let Some((glyph, fg)) = mark {
                put(buf, x, y, glyph, Style::new().fg(fg).bg(fill_colour).add_modifier(Modifier::BOLD));
            }
        } else {
            put(buf, x, y, glyph, style.add_modifier(Modifier::BOLD));
        }
    }
    // Where the last kelb went.
    let last_to = match game.last {
        Some(Move::Step { to, .. }) | Some(Move::Place(to)) => Some(to),
        _ => None,
    };
    if let Some(to) = last_to
        && game.selected.is_none()
    {
        let (x, y) = at(to);
        put(buf, x - hw - 1, y, "·", Style::new().fg(TEXT).bg(EARTH));
        put(buf, x + hw + 1, y, "·", Style::new().fg(TEXT).bg(EARTH));
    }
    if game.human_to_move() {
        let (x, y) = at(game.cursor);
        let style = Style::new().fg(AMBER).bg(EARTH).add_modifier(Modifier::BOLD);
        put(buf, x - hw - 1, y, "[", style);
        put(buf, x + hw + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 3, by, panel_w, bh.max(17)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "SEEGA", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    let over = matches!(game.phase, Phase::Over { .. });
    let pos = &game.position;
    for side in Side::BOTH {
        let who = match game.player(side) {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        let active = side == pos.to_move && !over;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        put(buf, x + 2, y, "●", Style::new().fg(colour(side)).bg(BG));
        put(buf, x + 4, y, &format!("{} ({who}), {}", side.name(), pos.count(side)), style);
        y += 1;
    }
    let stage = match pos.stage {
        Stage::Placing => format!("Placing: {} more", pos.at.iter().filter(|s| s.is_none()).count() - 1),
        Stage::Opening => "Opening a way".to_string(),
        Stage::Moving if pos.chain => "Taking again".to_string(),
        Stage::Moving => format!("Moves without a capture: {} of {QUIET}", pos.quiet),
    };
    put(buf, x, y, &stage, Style::new().fg(DIM).bg(BG));
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { outcome } => {
            let result = match outcome {
                Outcome::Win(Side::Stone) => "STONE WINS",
                Outcome::Win(Side::Brick) => "BRICK WINS",
                Outcome::Draw => "A DRAW",
            };
            put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
        Phase::Turn => {}
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &["Arrows: move the cursor", "Enter: place, pick up, put down", "Esc: let go   Mouse: click", "F1: help   Q: quit"]
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
        title: "SEEGA",
        subtitle: "The Egyptian game of holes in the ground, after Lane.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Stone, who places first", "Brick, who moves first"],
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
