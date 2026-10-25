//! Drawing: the cross of cloth, its castles, the men of the four colours,
//! the middle with the men still to enter and those home, the cowries or
//! dice as they fell, the throws still to use, a side panel, the menu and
//! the help.

use crate::app::{Game, Phase};
use crate::rules::{COLOURS, HOME, READY, Rules, TEAMS, Throws, cell, loop_square, team};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const CLOTH: Color = Color::Rgb(206, 186, 146);
const CASTLE: Color = Color::Rgb(176, 120, 84);
const MIDDLE: Color = Color::Rgb(120, 42, 34);
const SEAM: Color = Color::Rgb(150, 128, 92);
const LIT: Color = Color::Rgb(236, 220, 170);
const PICKED: Color = Color::Rgb(255, 200, 90);
const MARK: Color = Color::Rgb(30, 90, 160);

pub fn colour(c: usize) -> Color {
    [Color::Rgb(196, 32, 24), Color::Rgb(196, 140, 0), Color::Rgb(18, 16, 14), Color::Rgb(24, 128, 52)][c]
}

/// Where each cell was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub cells: Vec<(Rect, (u8, u8))>,
}

const W: u16 = 3;
const PANEL: u16 = 37;

fn on_cross(r: u8, c: u8) -> bool {
    (8..=10).contains(&r) || (8..=10).contains(&c)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.cells.clear();
    let rules = &game.rules;
    let p = &game.position;
    let bw = 19 * W;
    let total = bw + 3 + PANEL;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(20) / 2;
    let lit = game.lit();
    let moving = game.moving();
    let selected = game.selected.map(|s| cell(moving, s));
    let last_to = game.last.as_ref().map(|m| cell(m.colour as usize, m.to));

    // What stands on each cell: (colour, count) for each colour there.
    let mut men = std::collections::HashMap::<(u8, u8), Vec<(usize, u8)>>::new();
    for c in 0..4 {
        for &s in &p.men[c] {
            if s == 0 || s == HOME || s == READY {
                continue;
            }
            let at = cell(c, s);
            let list = men.entry(at).or_default();
            match list.iter_mut().find(|(k, _)| *k == c) {
                Some((_, n)) => *n += 1,
                None => list.push((c, 1)),
            }
        }
    }
    for r in 0..19u8 {
        for c in 0..19u8 {
            if !on_cross(r, c) {
                continue;
            }
            let x = bx + c as u16 * W;
            let y = by + r as u16;
            let rect = Rect::new(x, y, W, 1);
            hits.cells.push((rect, (r, c)));
            let middle = (8..=10).contains(&r) && (8..=10).contains(&c);
            let castle = !middle && (0..68).any(|sq| rules.castle(sq) && cell(0, loop_step(sq)) == (r, c));
            let mut bg = if middle {
                MIDDLE
            } else if castle {
                CASTLE
            } else if (r as u16 + c as u16) % 2 == 0 {
                CLOTH
            } else {
                Color::Rgb(198, 178, 138)
            };
            if lit.contains(&(r, c)) {
                bg = LIT;
            }
            if selected == Some((r, c)) {
                bg = PICKED;
            }
            fill(buf, rect, bg);
            if middle {
                continue;
            }
            match men.get(&(r, c)) {
                Some(list) if list.len() == 1 => {
                    let (k, n) = list[0];
                    let text = if n > 1 { format!("●{n}") } else { "●".to_string() };
                    put(buf, x + 1 - (n > 1) as u16, y, &text, Style::new().fg(colour(k)).bg(bg).add_modifier(Modifier::BOLD));
                }
                Some(list) => {
                    for (i, &(k, _)) in list.iter().take(3).enumerate() {
                        put(buf, x + i as u16, y, "●", Style::new().fg(colour(k)).bg(bg).add_modifier(Modifier::BOLD));
                    }
                }
                None if lit.contains(&(r, c)) => put(buf, x + 1, y, "◆", Style::new().fg(MARK).bg(bg)),
                None if castle => put(buf, x + 1, y, "✕", Style::new().fg(SEAM).bg(bg)),
                None => {}
            }
            if last_to == Some((r, c)) && game.selected.is_none() {
                put(buf, x, y, "·", Style::new().fg(MARK).bg(bg));
            }
        }
    }
    // The middle: each colour's men home on its own side; on the very
    // middle the men of the colour moving that still wait to come out.
    for c in 0..4 {
        let home = p.men[c].iter().filter(|&&s| s == HOME).count();
        let (r, k) = cell(c, 1);
        let toward = |v: u8| (9 + (v as i8 - 9).signum()) as u8;
        let (r, k) = (toward(r), toward(k));
        if home > 0 {
            put(buf, bx + k as u16 * W, by + r as u16, &format!("⌂{home}"), Style::new().fg(colour(c)).bg(MIDDLE).add_modifier(Modifier::BOLD));
        }
    }
    let waiting = p.men[moving].iter().filter(|&&s| s == 0 || s == READY).count();
    let mid_bg = if lit.contains(&(9, 9)) { LIT } else if selected == Some((9, 9)) { PICKED } else { MIDDLE };
    if waiting > 0 {
        fill(buf, Rect::new(bx + 9 * W, by + 9, W, 1), mid_bg);
        put(buf, bx + 9 * W, by + 9, &format!("○{waiting}"), Style::new().fg(colour(moving)).bg(mid_bg).add_modifier(Modifier::BOLD));
    }
    if game.human() && game.phase == Phase::Playing {
        let (r, c) = game.cursor;
        let x = bx + c as u16 * W;
        let style = Style::new().fg(MARK).add_modifier(Modifier::BOLD);
        put(buf, x.saturating_sub(1), by + r as u16, "▶", style);
    }
    draw_panel(buf, Rect::new(bx + bw + 3, by, PANEL, 20.min(area.height)), game);
}

/// Red's step for loop square `sq`.
fn loop_step(sq: u8) -> u8 {
    (8..=75).find(|&t| loop_square(0, t) == Some(sq)).unwrap_or(8)
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    let title = if game.rules.throws == Throws::LongDice { "CHAUPAR" } else { "PACHISI" };
    put(buf, x, y, title, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    let over = matches!(game.phase, Phase::Over { .. });
    let p = &game.position;
    for t in 0..2 {
        let who = match game.players[t] {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        put(buf, x, y, &format!("{} ({who})", TEAMS[t]), Style::new().fg(TEXT).bg(BG));
        y += 1;
        for c in [t, t + 2] {
            let active = c == p.to_move && !over;
            let style = if active { Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD) } else { Style::new().fg(DIM).bg(BG) };
            let waiting = p.men[c].iter().filter(|&&s| s == 0 || s == READY).count();
            let home = p.men[c].iter().filter(|&&s| s == HOME).count();
            put(buf, x, y, if active { "▶" } else { " " }, style);
            put(buf, x + 2, y, "●", Style::new().fg(colour(c)).bg(BG).add_modifier(Modifier::BOLD));
            put(buf, x + 4, y, &format!("{:<7}{waiting} to enter, {home} home", COLOURS[c]), style);
            y += 1;
        }
    }
    y += 1;
    // The throw as it fell, and the throws to use.
    if !game.fallen.is_empty() && !matches!(game.phase, Phase::ToThrow) || !game.turn.throws.is_empty() {
        let falling = matches!(game.phase, Phase::Throwing { .. });
        let fell = if falling {
            "...".to_string()
        } else if game.rules.throws == Throws::LongDice {
            game.fallen.iter().map(|t| format!("[{}]", t.shows)).collect::<Vec<_>>().join(" ")
        } else {
            let up = game.fallen.first().map_or(0, |t| t.shows);
            (0..6).map(|i| if i < up { "◉" } else { "◯" }).collect::<Vec<_>>().join("")
        };
        put(buf, x, y, &format!("Thrown: {fell}"), Style::new().fg(TEXT).bg(BG));
        y += 1;
        let toks = game.tokens();
        if !toks.is_empty() && !falling {
            let list: Vec<String> = toks.iter().map(|t| if t.enters && game.rules.throws != Throws::LongDice { "1*".to_string() } else { t.value.to_string() }).collect();
            put(buf, x, y, &format!("To use: {}", list.join(" ")), Style::new().fg(AMBER).bg(BG));
        }
        y += 1;
    }
    y += 1;
    for line in wrap(&game.message, panel.width as usize).into_iter().take(4) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if let Phase::Over { winner } = game.phase {
        put(buf, x, y, &format!("{} WIN", TEAMS[winner].to_uppercase()), Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human() && game.phase == Phase::ToThrow {
        &["Enter or Space: throw", "M: menu   F1: help   Q: quit"]
    } else if game.human() && game.phase == Phase::Playing {
        &["Arrows, Enter: pick up, put down", "Esc: let go   P: leave the rest", "Mouse: click   F1: help   Q: quit"]
    } else {
        &["F1: help   Q: quit"]
    };
    let mut ky = panel.bottom().saturating_sub(keys.len() as u16);
    for k in keys {
        put(buf, x, ky, k, Style::new().fg(DIM).bg(BG));
        ky += 1;
    }
    let _ = team;
}

pub fn menu(rules: &[Rules], chosen: usize) -> Menu<'_> {
    Menu {
        title: "PACHISI AND CHAUPAR",
        subtitle: "The cross-shaped race games of India, for two partnerships.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Red and Black, who begin", "Yellow and Green"],
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
