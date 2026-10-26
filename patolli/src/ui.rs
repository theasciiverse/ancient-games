//! Drawing: the mat with the cross painted on it (drawn upright), its
//! pinched compartments and rounded ends, the red and blue stones, the
//! beans as they fell, the purses and the pool, a side panel, the menu
//! and the help.

use crate::app::{Game, Phase};
use crate::rules::{HAND, Kind, OFF, Outcome, RING, Rules, STONES, Side, cell, kind};
use ancient_core::ornament;
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const MAT: Color = Color::Rgb(196, 168, 108);
const SQUARE: Color = Color::Rgb(220, 196, 140);
const MIDDLE: Color = Color::Rgb(150, 104, 62);
const PINCH: Color = Color::Rgb(40, 32, 26);
const END: Color = Color::Rgb(236, 214, 162);
const LIT: Color = Color::Rgb(250, 236, 190);
const MARK: Color = Color::Rgb(20, 90, 60);

pub fn colour(side: Side) -> Color {
    match side {
        Side::Red => Color::Rgb(186, 30, 22),
        Side::Blue => Color::Rgb(30, 70, 170),
    }
}

#[derive(Default, Clone)]
pub struct Hits {
    pub cells: Vec<(Rect, (u8, u8))>,
    /// The "bear off" tray.
    pub off: Option<Rect>,
}

const PANEL: u16 = 38;

/// A compartment's width and height: three columns by one row on the
/// screen the game is made for, larger on a larger one (`2s + 1` by `s`).
fn cell_size(area: Rect) -> (u16, u16) {
    let by_height = area.height.saturating_sub(4) / 16;
    let by_width = (area.width.saturating_sub(PANEL + 5) / 16).saturating_sub(1) / 2;
    let s = by_height.min(by_width).clamp(1, 4);
    (2 * s + 1, s)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.cells.clear();
    let p = &game.position;
    let (cw, ch) = cell_size(area);
    let bw = 16 * cw + 2;
    let bh = 16 * ch + 2;
    let total = bw + 3 + PANEL;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    fill(buf, Rect::new(bx, by, bw, bh), MAT);
    let lit = game.lit();
    let last = game.last;
    for sq in 0..RING {
        let (r, c) = cell(sq);
        let x = bx + 1 + c as u16 * cw;
        let y = by + 1 + r as u16 * ch;
        let rect = Rect::new(x, y, cw, ch);
        hits.cells.push((rect, (r, c)));
        // Larger compartments are parted by a line of the mat, as the
        // cross is ruled in the pictures.
        let face = if ch > 1 { Rect::new(x, y, cw - 1, ch - 1) } else { rect };
        let (mx, my) = (face.x + face.width / 2, face.y + face.height / 2);
        let k = kind(sq);
        let mut bg = match k {
            Kind::Middle => MIDDLE,
            Kind::Pinched => PINCH,
            Kind::End => END,
            Kind::Plain => SQUARE,
        };
        if lit.contains(&(r, c)) {
            bg = LIT;
        }
        fill(buf, face, bg);
        let stone = Side::BOTH.into_iter().find(|&s| {
            let d = p.dir[s.index()];
            p.stones[s.index()].iter().any(|&q| q < OFF && p.square(s, q, d) == sq)
        });
        match stone {
            Some(s) => put(buf, mx, my, "●", Style::new().fg(colour(s)).bg(bg).add_modifier(Modifier::BOLD)),
            None if lit.contains(&(r, c)) => put(buf, mx, my, "◆", Style::new().fg(MARK).bg(bg)),
            None => match k {
                Kind::Pinched => put(buf, mx - 1, my, "▸◂", Style::new().fg(Color::Rgb(120, 100, 80)).bg(bg)),
                Kind::End => put(buf, mx, my, "◠", Style::new().fg(Color::Rgb(160, 130, 90)).bg(bg)),
                _ => {}
            },
        }
        if let Some((s, m)) = last
            && m.to < OFF
            && p.square(s, m.to, m.dir) == sq
        {
            put(buf, face.x, my, "·", Style::new().fg(MARK).bg(bg));
        }
    }
    if game.human() && game.phase == crate::app::Phase::Choosing {
        let (r, c) = game.cursor;
        let y = by + 1 + r as u16 * ch + ch.saturating_sub(1) / 2;
        put(buf, bx + c as u16 * cw, y, "▶", Style::new().fg(MARK).bg(MAT).add_modifier(Modifier::BOLD));
    }
    // Stones in hand and borne off, under the mat.
    let mut y = by + bh;
    for s in Side::BOTH {
        let hand = p.stones[s.index()].iter().filter(|&&q| q == HAND).count();
        let off = p.off(s);
        let mut x = bx;
        put(buf, x, y, &format!("{:<5}", s.name()), Style::new().fg(TEXT).bg(BG));
        x += 6;
        put(buf, x, y, &"●".repeat(hand), Style::new().fg(colour(s)).bg(BG));
        put(buf, x + STONES as u16 + 1, y, "in hand", Style::new().fg(DIM).bg(BG));
        x += STONES as u16 + 10;
        put(buf, x, y, &"●".repeat(off), Style::new().fg(colour(s)).bg(BG));
        put(buf, x + STONES as u16 + 1, y, "off", Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    hits.off = None;
    if game.can_bear_off() {
        let r = Rect::new(bx + bw - 13, by + bh, 13, 1);
        put(buf, r.x, r.y, "[ bear off ]", Style::new().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD));
        hits.off = Some(r);
    }
    let panel_h = (bh + 2).max(20).min(area.bottom().saturating_sub(by));
    draw_panel(buf, Rect::new(bx + bw + 3, by, PANEL, panel_h), game);
    draw_ornament(buf, area, Rect::new(bx, by, bw, bh), (cw, ch));
}

/// Ornament after the patolli scene of the Codex Magliabechiano (fol. 60r).
/// Durán and Sahagún say the board was painted on a mat; in the four
/// quarters beside the cross, the weave of the mats the players sit on, a
/// herringbone of short strokes. On a terminal larger than the game is
/// made for, above the mat, the five gold discs drawn over the board.
fn draw_ornament(buf: &mut Buffer, area: Rect, mat: Rect, (cw, ch): (u16, u16)) {
    let reed = ornament::dim(MIDDLE, MAT, 0.55);
    let weave = |x: i32, y: i32| {
        if x % 16 < 8 { (x + y) % 4 == 0 } else { (x - y).rem_euclid(4) == 0 }
    };
    // Grid rows and columns 0 to 6 and 9 to 15, clear of the arms (7 and
    // 8) and of the cursor beside a compartment.
    let (near, far) = (7 * cw, 1 + 9 * cw);
    let left = Rect::new(mat.x, mat.y, near, 1 + 7 * ch);
    let right = Rect::new(mat.x + far, mat.y, mat.width - far, 1 + 7 * ch);
    let down = 1 + 9 * ch;
    for quarter in [left, right, Rect { y: mat.y + down, ..left }, Rect { y: mat.y + down, ..right }] {
        ornament::paint(buf, quarter, reed, weave);
    }
    if !ornament::roomy(area) || mat.y < area.y + 3 {
        return;
    }
    // Five discs in a row over the middle of the mat, as in the codex: a
    // ring seven dots across round a dot, ten dots apart.
    let gold = ornament::dim(Color::Rgb(226, 180, 50), BG, 0.45);
    let row = Rect::new((mat.x + mat.width / 2).saturating_sub(13), mat.y - 3, 26, 2);
    ornament::paint(buf, row, gold, |x, y| {
        let (dx, dy) = ((x - 1) % 10 - 4, y - 4);
        let d = dx * dx + dy * dy;
        (1..50).contains(&x) && ((5..=10).contains(&d) || d == 0)
    });
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "PATOLLI", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    let p = &game.position;
    let over = matches!(game.phase, Phase::Over { .. });
    for s in Side::BOTH {
        let who = match game.players[s.index()] {
            Player::Human => "you".to_string(),
            Player::Computer(st) => format!("computer, {}", st.name()),
        };
        let active = s == p.to_move && !over;
        let style = if active { Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD) } else { Style::new().fg(TEXT).bg(BG) };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        put(buf, x + 2, y, "●", Style::new().fg(colour(s)).bg(BG).add_modifier(Modifier::BOLD));
        let way = match p.dir[s.index()] {
            1 => ", goes ↺",
            -1 => ", goes ↻",
            _ => "",
        };
        put(buf, x + 4, y, &format!("{} ({who}){way}", s.name()), style);
        y += 1;
        put(buf, x + 4, y, &format!("purse {}", p.purse[s.index()]), Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    put(buf, x, y, &format!("Pool: {}", p.pool), Style::new().fg(DIM).bg(BG));
    y += 2;
    if !matches!(game.phase, Phase::ToThrow) || game.throw > 0 {
        let falling = matches!(game.phase, Phase::Throwing { .. });
        let beans: String = if falling { "  . . . . .".to_string() } else { game.beans.iter().map(|&b| if b { " ◉" } else { " ●" }).collect() };
        put(buf, x, y, &format!("Beans:{beans}"), Style::new().fg(TEXT).bg(BG));
        if !falling {
            put(buf, x + 18, y, &format!("= {}", game.throw), Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        }
    }
    y += 2;
    for line in wrap(&game.message, panel.width as usize).into_iter().take(5) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if let Phase::Over { outcome: Outcome::Home(w) } = game.phase {
        put(buf, x, y, &format!("{} WINS", w.name().to_uppercase()), Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human() && game.phase == Phase::ToThrow {
        &["Enter or Space: cast the beans", "M: menu   F1: help   Q: quit"]
    } else if game.human() && game.phase == Phase::Choosing {
        &["Arrows, Enter: a lit compartment", "B: bear off   Mouse: click", "F1: help   Q: quit"]
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
        title: "PATOLLI",
        subtitle: "The Aztec game of the beans and the painted cross.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["Red", "Blue"],
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
