//! Drawing: the board of pits on wood, the seeds in them, the stores at the
//! ends, the hand going round, and a panel below with the players, what
//! happened and the keys; the menu and the help.
//!
//! South's row runs along the bottom in the order of sowing and North's
//! along the top the other way, so pits facing each other stand one above
//! the other; South's store is at the right, North's at the left. A game
//! that sows clockwise is drawn mirrored.

use crate::app::{Game, Phase};
use crate::rules::{Position, Rules, Setup, Side, Slot};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const WOOD: Color = Color::Rgb(58, 38, 22);
const PIT: Color = Color::Rgb(34, 22, 12);
const RIM: Color = Color::Rgb(120, 86, 52);
const SEED: Color = Color::Rgb(222, 204, 160);
const BLOOD: Color = Color::Rgb(210, 70, 50);
const CLOSED: Color = Color::Rgb(70, 52, 36);

/// Where each pit was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub pits: Vec<(Rect, u8)>,
}

/// The sizes of the board for `pits` a side in `area`: pit width, pit
/// height, store width.
fn sizes(area: Rect, pits: usize) -> (u16, u16, u16) {
    let store = 8;
    let w = ((area.width.saturating_sub(2 * store + 6)) / pits as u16).clamp(6, 12);
    let h = if area.height >= 34 { 6 } else { 5 };
    (w, h, store)
}

/// The column (0 at the left) where pit `p` is drawn.
fn column(pos: &Position, p: u8, mirror: bool) -> usize {
    let n = pos.len() / 2;
    let p = p as usize;
    let c = if p < n { p } else { 2 * n - 1 - p };
    if mirror { n - 1 - c } else { c }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.pits.clear();
    let pos = &game.shown;
    let n = pos.len() / 2;
    let mirror = game.rules.clockwise;
    let (pw, ph, sw) = sizes(area, n);
    let bw = n as u16 * pw + 2 * sw + 4;
    let bh = 2 * ph + 4;
    let bx = area.x + area.width.saturating_sub(bw) / 2;
    let by = area.y + 2;
    let title = Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD);
    put(buf, bx, area.y, "MANCALA", title);
    put(buf, bx + 9, area.y, game.rules.name, Style::new().fg(DIM).bg(BG));
    fill(buf, Rect::new(bx, by, bw, bh), WOOD);

    let legal = game.legal();
    let arranging = matches!(game.phase, Phase::Arrange { .. });
    let cursor_on = game.human_to_move()
        || (arranging && matches!(game.phase, Phase::Arrange { by } if game.player(by) == Player::Human));
    let wise = matches!(game.rules.setup, Setup::Lane { wise: true });
    let x0 = bx + 2 + sw;
    for p in 0..pos.len() as u8 {
        let col = column(pos, p, mirror) as u16;
        let north = pos.owner(p) == Side::North;
        let x = x0 + col * pw;
        let y = if north { by + 1 } else { by + 3 + ph };
        let cell = Rect::new(x, y, pw - 1, ph);
        hits.pits.push((cell, p));
        let flash = game.flash.iter().any(|&(f, _)| f == p);
        let under_cursor = cursor_on
            && (game.cursor == p || (arranging && wise && game.cursor as usize % n == p as usize % n));
        let rim = if flash {
            BLOOD
        } else if under_cursor {
            AMBER
        } else if legal.contains(&p) {
            Color::Rgb(200, 160, 90)
        } else {
            RIM
        };
        draw_pit(buf, cell, pos, p, rim, under_cursor);
        // The name under South's pits and over North's.
        let label_y = if north { by } else { by + 3 + 2 * ph };
        let name = pos.name(p).to_string();
        put(buf, x + (pw - 1) / 2, label_y, &name, Style::new().fg(Color::Rgb(160, 130, 90)).bg(WOOD));
        if game.hand > 0 && game.hand_at == Some(Slot::Pit(p)) {
            let text = format!("✋{}", game.hand);
            put(buf, x + (pw - 1) / 2 - 1, label_y, &text, Style::new().fg(AMBER).bg(WOOD).add_modifier(Modifier::BOLD));
        }
    }
    // The stores: South's after its last pit, North's after its last.
    for side in Side::BOTH {
        let right = (side == Side::South) != mirror;
        let x = if right { x0 + n as u16 * pw } else { bx + 1 };
        let cell = Rect::new(x, by + 1, sw, 2 * ph + 2);
        draw_store(buf, cell, pos.stores[side.index()], side);
        if game.hand > 0 && game.hand_at == Some(Slot::Store(side)) {
            put(buf, x + 1, by, &format!("✋{}", game.hand), Style::new().fg(AMBER).bg(WOOD).add_modifier(Modifier::BOLD));
        }
    }

    draw_panel(buf, Rect::new(bx, by + bh + 1, bw.max(60).min(area.width), area.bottom().saturating_sub(by + bh + 1)), game);
}

/// A pit: a rim, the count, and the seeds as dots while they fit.
fn draw_pit(buf: &mut Buffer, cell: Rect, pos: &Position, p: u8, rim: Color, bold: bool) {
    let closed = pos.closed[p as usize];
    let tuz = pos.tuz_owner(p);
    let inner = Rect::new(cell.x + 1, cell.y + 1, cell.width - 2, cell.height - 2);
    fill(buf, inner, if closed { CLOSED } else { PIT });
    let style = if bold {
        Style::new().fg(rim).bg(WOOD).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(rim).bg(WOOD)
    };
    let (r, b) = (cell.right() - 1, cell.bottom() - 1);
    for x in cell.x..=r {
        let (top, bottom) = match x {
            x if x == cell.x => ("╭", "╰"),
            x if x == r => ("╮", "╯"),
            _ => ("─", "─"),
        };
        put(buf, x, cell.y, top, style);
        put(buf, x, b, bottom, style);
    }
    for y in cell.y + 1..b {
        put(buf, cell.x, y, "│", style);
        put(buf, r, y, "│", style);
    }
    if closed {
        let mid = inner.y + inner.height / 2;
        let x = inner.x + inner.width.saturating_sub(1) / 2;
        put(buf, x, mid, "╳", Style::new().fg(RIM).bg(CLOSED));
        return;
    }
    let seeds = pos.pits[p as usize];
    if let Some(owner) = tuz {
        let text = if owner == Side::South { "tuz S" } else { "tuz N" };
        put(buf, inner.x + inner.width.saturating_sub(5) / 2, inner.y, text, Style::new().fg(AMBER).bg(PIT));
        return;
    }
    let count = seeds.to_string();
    let cx = inner.x + inner.width.saturating_sub(count.len() as u16) / 2;
    put(buf, cx, inner.y, &count, Style::new().fg(TEXT).bg(PIT).add_modifier(Modifier::BOLD));
    // The seeds themselves, while they fit in the rows below the count.
    let rows = inner.height.saturating_sub(1);
    let room = (inner.width as usize) * rows as usize;
    let shown = (seeds as usize).min(room);
    let per_row = inner.width as usize;
    for i in 0..shown {
        let row = (i / per_row) as u16;
        // Each row of seeds centred in the pit.
        let in_row = (shown - row as usize * per_row).min(per_row) as u16;
        let colm = (i % per_row) as u16 + (inner.width - in_row) / 2;
        let glyph = if seeds as usize > room && i == shown - 1 { "+" } else { "•" };
        put(buf, inner.x + colm, inner.y + 1 + row, glyph, Style::new().fg(SEED).bg(PIT));
    }
}

/// A store: its owner and the seeds in it.
fn draw_store(buf: &mut Buffer, cell: Rect, seeds: u16, side: Side) {
    let inner = Rect::new(cell.x + 1, cell.y + 1, cell.width - 2, cell.height - 2);
    fill(buf, inner, PIT);
    let style = Style::new().fg(RIM).bg(WOOD);
    let (r, b) = (cell.right() - 1, cell.bottom() - 1);
    for x in cell.x..=r {
        let (top, bottom) = match x {
            x if x == cell.x => ("╭", "╰"),
            x if x == r => ("╮", "╯"),
            _ => ("─", "─"),
        };
        put(buf, x, cell.y, top, style);
        put(buf, x, b, bottom, style);
    }
    for y in cell.y + 1..b {
        put(buf, cell.x, y, "│", style);
        put(buf, r, y, "│", style);
    }
    let name = side.name();
    let mid = inner.y + inner.height / 2;
    put(buf, inner.x + inner.width.saturating_sub(name.len() as u16) / 2, mid - 1, name, Style::new().fg(DIM).bg(PIT));
    let count = seeds.to_string();
    put(
        buf,
        inner.x + inner.width.saturating_sub(count.len() as u16) / 2,
        mid,
        &count,
        Style::new().fg(SEED).bg(PIT).add_modifier(Modifier::BOLD),
    );
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let x = panel.x;
    let mut y = panel.y;
    let to_move = game.position.to_move;
    let playing = matches!(game.phase, Phase::Turn | Phase::Thinking { .. } | Phase::Sowing { .. });
    for side in [Side::South, Side::North] {
        let who = match game.player(side) {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        let active = side == to_move && playing;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        let mut line = format!("{} ({who}): {} in store", side.name(), game.shown.stores[side.index()]);
        match game.rules.rounds {
            crate::rules::Match::Points { .. } => line.push_str(&format!(", {} points", game.totals[side.index()])),
            crate::rules::Match::Refill { .. } => line.push_str(&format!(", {} rounds won", game.totals[side.index()])),
            _ => {}
        }
        put(buf, x + 2, y, &line, style);
        y += 1;
    }
    y += 1;
    if game.to_place > 0 && matches!(game.phase, Phase::Arrange { .. }) {
        put(buf, x, y, &format!("Pebbles in the hand: {}", game.to_place), Style::new().fg(AMBER).bg(BG));
        y += 1;
    }
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if matches!(game.phase, Phase::Thinking { .. }) {
        put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    if let Phase::Over { winner } = game.phase {
        let result = match winner {
            Some(Side::South) => "SOUTH WINS",
            Some(Side::North) => "NORTH WINS",
            None => "A DRAW",
        };
        put(buf, x, y, result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
        y += 1;
    }
    // The moves before, quieter, while there is room above the keys.
    for earlier in &game.earlier {
        for line in wrap(earlier, panel.width as usize) {
            if y + 3 >= panel.bottom() {
                break;
            }
            put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
            y += 1;
        }
    }
    let human = |s: Side| game.player(s) == Player::Human;
    let keys: Vec<&str> = match &game.phase {
        Phase::Over { .. } => vec!["Enter: play again   M: menu   F1: help   Q: quit"],
        Phase::RoundOver { .. } => vec!["Enter: next round   M: menu   F1: help   Q: quit"],
        Phase::Arrange { by } if human(*by) => vec![
            "Left, Right: choose a pit   Up, Down: one pebble more or less   R: the rest at random",
            "Enter: done   F1: help   Q: quit",
        ],
        Phase::TurnBoard { by } if human(*by) => {
            vec!["T: turn the board round (the other side begins)   Enter: keep it and begin", "F1: help   Q: quit"]
        }
        Phase::Sowing { .. } => vec!["Any key: finish the move   F1: help   Q: quit"],
        _ if game.human_to_move() => {
            vec!["Left, Right: choose a pit   Enter: sow it   Mouse: click a pit", "M: menu   F1: help   Q: quit"]
        }
        _ => vec!["M: menu   F1: help   Q: quit"],
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
        title: "MANCALA",
        subtitle: "The sowing games of Africa, Asia and Europe, each with its own rules.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["South", "North"],
    }
}

/// Whether Left on the screen means towards lower pit numbers for the side
/// to move (South's pits run left to right, North's right to left, and a
/// clockwise game is mirrored).
pub fn left_is_lower(game: &Game, side: Side) -> bool {
    (side == Side::South) != game.rules.clockwise
}

/// The short form of the game's rulebook chapter (its "brief" blocks), as
/// plain text: the F1 help. The full chapter is RULES.txt in the release
/// archives. Both are written by `rulebook/build.sh`.
pub const RULES: &str = include_str!("../HELP.txt");

/// How to play: the rules in short, and the line where the rule set
/// `rules` begins.
pub fn help(rules: &Rules) -> (Vec<Help>, usize) {
    let lines = shell::rules_help(RULES);
    let at = shell::help_find(&lines, rules.section).unwrap_or(0);
    (lines, at)
}

#[cfg(test)]
mod help_tests {
    use super::*;

    #[test]
    fn the_help_opens_at_every_rule_set() {
        let lines = shell::rules_help(RULES);
        for rules in Rules::all() {
            assert!(shell::help_find(&lines, rules.section).is_some(), "{}", rules.section);
        }
    }

    #[test]
    fn the_rulebook_text_fits_the_help() {
        // The help is 84 columns wide with 3 on each side. Only a single
        // word, a web address, may be wider (the help runs it on).
        for line in RULES.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }
}
