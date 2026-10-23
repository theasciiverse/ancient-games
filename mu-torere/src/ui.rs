//! Drawing: the eight-pointed star on dark wood (the kewai on a ring, each
//! joined to its neighbours and by a spoke to the pūtahi), the white and
//! black men, the cursor and the men that may move, a side panel, the menu
//! and the help.

use crate::app::{Game, Phase, name};
use crate::rules::{CENTRE, POINTS, Rules, Side};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const WOOD: Color = Color::Rgb(58, 36, 22);
const LINE: Color = Color::Rgb(196, 150, 96);
const WHITE: Color = Color::Rgb(246, 242, 230);
const BLACK: Color = Color::Rgb(16, 14, 12);
const RING: Color = Color::Rgb(120, 92, 60);

/// Where each point was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, usize)>,
    /// Where each point was drawn, for the cursor keys.
    pub at: [(i32, i32); POINTS],
}

/// Where each point stands on the screen: the kewai on an ellipse twice
/// as wide as tall (so the diagonal spokes look diagonal), the pūtahi in
/// the middle.
pub fn layout(cx: u16, cy: u16, r: u16) -> [(i32, i32); POINTS] {
    let mut at = [(cx as i32, cy as i32); POINTS];
    for (k, point) in at.iter_mut().take(8).enumerate() {
        let a = (k as f64 * 45.0 - 90.0).to_radians();
        *point = (
            cx as i32 + (2.0 * r as f64 * a.cos()).round() as i32,
            cy as i32 + (r as f64 * a.sin()).round() as i32,
        );
    }
    at
}

/// A line from `a` to `b`, in the glyph nearest its slope on the screen
/// (a row is about twice as tall as a column is wide).
fn line(buf: &mut Buffer, a: (i32, i32), b: (i32, i32), style: Style) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let angle = (2.0 * dy as f64).atan2(dx as f64).to_degrees().rem_euclid(180.0);
    let glyph = match angle {
        a if !(28.0..=152.0).contains(&a) => "─",
        a if (62.0..=118.0).contains(&a) => "│",
        a if a < 90.0 => "╲",
        _ => "╱",
    };
    // One stroke a row for anything steeper than flat, one a column for
    // the flat ones.
    let steps = if glyph == "─" { dx.abs() } else { dy.abs() }.max(1);
    for i in 1..steps {
        let t = i as f64 / steps as f64;
        let x = a.0 + (dx as f64 * t).round() as i32;
        let y = a.1 + (dy as f64 * t).round() as i32;
        if x >= 0 && y >= 0 {
            put(buf, x as u16, y as u16, glyph, style);
        }
    }
}

/// The ring through the kewai, as a dotted oval round `c`.
fn ring(buf: &mut Buffer, c: (i32, i32), r: u16, style: Style) {
    let steps = 32 * r as i32;
    for i in 0..steps {
        let a = (i as f64 / steps as f64) * std::f64::consts::TAU;
        let x = c.0 + (2.0 * r as f64 * a.cos()).round() as i32;
        let y = c.1 + (r as f64 * a.sin()).round() as i32;
        if x >= 0 && y >= 0 {
            put(buf, x as u16, y as u16, "·", style);
        }
    }
}

fn man(side: Side) -> Style {
    let colour = match side {
        Side::White => WHITE,
        Side::Black => BLACK,
    };
    Style::new().fg(colour).bg(WOOD).add_modifier(Modifier::BOLD)
}

/// The board's size for `area`: the ellipse's height, in rows from the
/// middle.
fn radius(area: Rect) -> u16 {
    (area.height.saturating_sub(8) / 2).clamp(4, 11).min(area.width.saturating_sub(46) / 4)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let r = radius(area);
    let bw = 4 * r + 9;
    let bh = 2 * r + 3;
    let panel_w = 34u16;
    let total = bw + 3 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    fill(buf, Rect::new(bx, by, bw, bh), WOOD);
    let at = layout(bx + bw / 2, by + bh / 2, r);
    hits.at = at;

    ring(buf, at[CENTRE], r, Style::new().fg(RING).bg(WOOD).add_modifier(Modifier::BOLD));
    let spoke = Style::new().fg(LINE).bg(WOOD);
    for &point in at.iter().take(8) {
        line(buf, at[CENTRE], point, spoke);
    }

    let movable = game.movable();
    for p in 0..POINTS {
        let (x, y) = (at[p].0 as u16, at[p].1 as u16);
        hits.points.push((Rect::new(x.saturating_sub(1), y, 3, 1), p));
        let (glyph, style) = match game.position.at[p] {
            Some(side) if movable.contains(&p) => ("●", man(side).bg(Color::Rgb(110, 76, 30))),
            Some(side) => ("●", man(side)),
            None => ("◯", Style::new().fg(LINE).bg(WOOD)),
        };
        put(buf, x, y, glyph, style);
    }
    if let Some(mv) = &game.last {
        let (x, y) = (at[mv.to as usize].0 as u16, at[mv.to as usize].1 as u16);
        put(buf, x - 1, y, "·", Style::new().fg(DIM).bg(WOOD));
        put(buf, x + 1, y, "·", Style::new().fg(DIM).bg(WOOD));
    }
    if game.human_to_move() {
        let (x, y) = (at[game.cursor].0 as u16, at[game.cursor].1 as u16);
        let style = Style::new().fg(AMBER).bg(WOOD).add_modifier(Modifier::BOLD);
        put(buf, x - 1, y, "[", style);
        put(buf, x + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 3, by, panel_w, bh.max(16)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "MŪ TŌRERE", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
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
        let colour = match side {
            Side::White => WHITE,
            Side::Black => Color::Rgb(110, 100, 92),
        };
        put(buf, x + 2, y, "●", Style::new().fg(colour).bg(BG));
        put(buf, x + 4, y, &format!("{} ({who})", side.name()), style);
        y += 1;
    }
    y += 1;
    let empty = format!("Empty: {}", name(game.position.empty()));
    put(buf, x, y, &empty, Style::new().fg(DIM).bg(BG));
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if let Phase::Over { winner } = &game.phase {
        let result = match winner {
            Some(Side::White) => "WHITE WINS",
            Some(Side::Black) => "BLACK WINS",
            None => "A DRAW",
        };
        put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &["Arrows: move the cursor", "Enter: move that man", "Mouse: click a man", "F1: help   Q: quit"]
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
        title: "MŪ TŌRERE",
        subtitle: "The Māori game of the eight-pointed star.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["White", "Black"],
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
