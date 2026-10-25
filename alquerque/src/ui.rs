//! Drawing: the board's points and lines cut in stone, the white and
//! black men, the cursor and the points lit for a choice, a side panel,
//! the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Outcome, Rules, Side};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const STONE: Color = Color::Rgb(176, 164, 140);
const LINE: Color = Color::Rgb(70, 60, 48);
const WHITE: Color = Color::Rgb(250, 248, 240);
const BLACK: Color = Color::Rgb(24, 20, 18);
const MARK: Color = Color::Rgb(160, 26, 16);
const PICKED: Color = Color::Rgb(214, 196, 150);

/// Where each point was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, usize)>,
}

/// Columns and rows between neighbouring points: twice as many columns as
/// rows, so the diagonals run at 45 degrees on the screen.
fn spacing(rules: &Rules, area: Rect) -> (u16, u16) {
    let (gx, gy) = (rules.cols as u16 - 1, rules.rows as u16 - 1);
    let sy = (area.height.saturating_sub(7) / gy).clamp(2, 6);
    let sy = sy.min(area.width.saturating_sub(44) / (2 * gx)).max(2);
    (2 * sy, sy)
}

fn man(side: Side) -> Style {
    let colour = match side {
        Side::White => WHITE,
        Side::Black => BLACK,
    };
    Style::new().fg(colour).bg(STONE).add_modifier(Modifier::BOLD)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let rules = &game.rules;
    let (sx, sy) = spacing(rules, area);
    let bw = (rules.cols as u16 - 1) * sx + 7;
    let bh = (rules.rows as u16 - 1) * sy + 3;
    let panel_w = 36u16;
    let total = bw + 3 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2 + 1;
    let by = area.y + area.height.saturating_sub(bh + 1) / 2 + 1;
    fill(buf, Rect::new(bx, by, bw, bh), STONE);
    let at = |p: usize| {
        let (r, c) = rules.rc(p);
        (bx + 3 + c as u16 * sx, by + 1 + r as u16 * sy)
    };
    // The names of the columns and rows, as the messages use them.
    let label = Style::new().fg(DIM).bg(BG);
    for c in 0..rules.cols {
        let (x, _) = at(c);
        put(buf, x, by + bh, &((b'a' + c as u8) as char).to_string(), label);
    }
    for r in 0..rules.rows {
        let (_, y) = at(r * rules.cols);
        put(buf, bx.saturating_sub(2), y, &format!("{}", rules.rows - r), label);
    }

    // The lines, from each point to its neighbours right and below.
    let line = Style::new().fg(LINE).bg(STONE);
    for p in 0..rules.len() {
        let (x0, y0) = at(p);
        for d in [(0, 1), (1, 0), (1, 1), (1, -1)] {
            let Some(q) = rules.step(p, d) else {
                continue;
            };
            let (x1, y1) = at(q);
            match d {
                (0, 1) => (x0 + 1..x1).for_each(|x| put(buf, x, y0, "─", line)),
                (1, 0) => (y0 + 1..y1).for_each(|y| put(buf, x0, y, "│", line)),
                (1, dc) => {
                    let glyph = if dc > 0 { "╲" } else { "╱" };
                    for i in 1..sy {
                        let x = if dc > 0 { x0 + 2 * i - 1 } else { x0 - 2 * i + 1 };
                        put(buf, x, y0 + i, glyph, line);
                    }
                }
                _ => {}
            }
        }
    }

    let shown = game.shown();
    let lit = game.lit();
    let doomed = game.doomed();
    let choosing = !game.choosing.is_empty();
    for p in 0..rules.len() {
        let (x, y) = at(p);
        hits.points.push((Rect::new(x.saturating_sub(1), y, 3, 1), p));
        let (glyph, mut style) = match shown.at[p] {
            Some(side) if doomed.contains(&p) || (choosing && lit.contains(&p)) => ("✕", man(side).fg(MARK)),
            Some(side) => ("●", man(side)),
            None if lit.contains(&p) => ("◆", Style::new().fg(MARK).bg(STONE).add_modifier(Modifier::BOLD)),
            None => ("•", line.add_modifier(Modifier::BOLD)),
        };
        if game.selected == Some(p) {
            style = style.bg(PICKED);
        } else if lit.contains(&p) && shown.at[p].is_some() && !choosing {
            style = style.bg(PICKED);
        }
        put(buf, x, y, glyph, style);
    }
    // Where the last move went, and the men it took.
    if let Some(mv) = &game.last
        && game.selected.is_none()
        && matches!(game.phase, Phase::Turn | Phase::Thinking { .. } | Phase::Over { .. })
    {
        let (x, y) = at(mv.to());
        put(buf, x - 1, y, "·", Style::new().fg(LINE).bg(STONE));
        put(buf, x + 1, y, "·", Style::new().fg(LINE).bg(STONE));
        for q in mv.taken() {
            if shown.at[q].is_none() {
                let (x, y) = at(q);
                put(buf, x, y, "×", Style::new().fg(MARK).bg(STONE).add_modifier(Modifier::BOLD));
            }
        }
    }
    if game.human_to_move() {
        let (x, y) = at(game.cursor);
        let style = Style::new().fg(MARK).bg(STONE).add_modifier(Modifier::BOLD);
        put(buf, x - 1, y, "[", style);
        put(buf, x + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 3, by, panel_w, bh.max(18)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    let title = if game.rules.id == "fanorona" { "FANORONA" } else { "ALQUERQUE" };
    put(buf, x, y, title, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
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
        let colour = match side {
            Side::White => WHITE,
            Side::Black => Color::Rgb(110, 100, 92),
        };
        put(buf, x + 2, y, "●", Style::new().fg(colour).bg(BG));
        let n = shown.count(side);
        let men = if n == 1 { "man" } else { "men" };
        put(buf, x + 4, y, &format!("{} ({who}), {n} {men}", side.name()), style);
        y += 1;
    }
    let quiet = game.position.quiet;
    if quiet > 0 && !over {
        put(buf, x, y, &format!("Turns with nothing taken: {quiet} of {}", game.rules.quiet), Style::new().fg(DIM).bg(BG));
    }
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { outcome } => {
            let result = match outcome {
                Outcome::Win(Side::White) => "WHITE WINS",
                Outcome::Win(Side::Black) => "BLACK WINS",
                Outcome::Draw => "A DRAW",
            };
            put(buf, x, y + 1, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
        _ => {}
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_to_move() {
        &["Arrows: move the cursor", "Enter: pick up, put down", "Esc: let go, or stop taking", "Mouse: click   F1: help   Q: quit"]
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
        title: "ALQUERQUE AND FANORONA",
        subtitle: "The leaping game of medieval Spain, and the line-taking game of Madagascar.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["White, who begins", "Black"],
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

    #[test]
    fn both_boards_fit_the_screen() {
        use crate::app::{Game, Player, Rng};
        for rules in Rules::all() {
            let game = Game::new(rules.clone(), [Player::Human, Player::Human], Rng::new(1));
            let area = Rect::new(0, 0, 100, 30);
            let mut buf = Buffer::empty(area);
            let mut hits = Hits::default();
            draw_game(&mut buf, area, &game, &mut hits);
            assert_eq!(hits.points.len(), rules.len());
            assert!(hits.points.iter().all(|(r, _)| r.right() <= 64 && r.bottom() <= 30), "{}", rules.id);
        }
    }
}
