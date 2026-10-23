//! Drawing: the board's points and lines on light wood, the white and
//! black men, the men still in hand, the cursor and the points lit for a
//! choice, a side panel, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Board, Outcome, Rules, Side};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const WOOD: Color = Color::Rgb(184, 142, 88);
const LINE: Color = Color::Rgb(74, 48, 24);
const WHITE: Color = Color::Rgb(250, 248, 240);
const BLACK: Color = Color::Rgb(24, 20, 18);
const MARK: Color = Color::Rgb(160, 26, 16);
const PICKED: Color = Color::Rgb(226, 190, 120);

/// Where each point was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, usize)>,
}

/// Columns and rows between neighbouring grid lines: twice as many
/// columns as rows, so the diagonals run at 45 degrees on the screen.
fn spacing(board: &Board, area: Rect) -> (u16, u16) {
    let gaps = board.size as u16 - 1;
    let sy = (area.height.saturating_sub(6) / gaps).clamp(2, 10);
    let sy = sy.min(area.width.saturating_sub(42) / (2 * gaps)).max(2);
    (2 * sy, sy)
}

fn man(side: Side) -> Style {
    let colour = match side {
        Side::White => WHITE,
        Side::Black => BLACK,
    };
    Style::new().fg(colour).bg(WOOD).add_modifier(Modifier::BOLD)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let board = &game.board;
    let gaps = board.size as u16 - 1;
    let (sx, sy) = spacing(board, area);
    let bw = gaps * sx + 7;
    let bh = gaps * sy + 3;
    let panel_w = 34u16;
    let total = bw + 3 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    fill(buf, Rect::new(bx, by, bw, bh), WOOD);
    let at = |p: usize| {
        let (r, c) = board.points[p];
        (bx + 3 + c as u16 * sx, by + 1 + r as u16 * sy)
    };

    // The lines, segment by segment between neighbours.
    let line = Style::new().fg(LINE).bg(WOOD);
    for l in &board.lines {
        for w in l.windows(2) {
            let ((x0, y0), (x1, y1)) = (at(w[0]), at(w[1]));
            if y0 == y1 {
                for x in x0.min(x1) + 1..x0.max(x1) {
                    put(buf, x, y0, "─", line);
                }
            } else if x0 == x1 {
                for y in y0.min(y1) + 1..y0.max(y1) {
                    put(buf, x0, y, "│", line);
                }
            } else {
                // A diagonal: two columns a row.
                let down_right = (x1 > x0) == (y1 > y0);
                let (x, y) = if y0 < y1 { (x0, y0) } else { (x1, y1) };
                let glyph = if down_right { "╲" } else { "╱" };
                for i in 1..y0.abs_diff(y1) {
                    let dx = 2 * i;
                    let cx = if down_right { x + dx } else { x - dx };
                    let cx = if down_right { cx - 1 } else { cx + 1 };
                    put(buf, cx, y + i, glyph, line);
                }
            }
        }
    }

    let lit = game.targets();
    let taking = game.pending.is_some();
    for p in 0..board.len() {
        let (x, y) = at(p);
        hits.points.push((Rect::new(x.saturating_sub(1), y, 3, 1), p));
        let (glyph, mut style) = match game.position.side_at(p) {
            Some(side) if taking && lit.contains(&p) => ("●", man(side).fg(MARK)),
            Some(side) => ("●", man(side)),
            None if lit.contains(&p) => ("◆", Style::new().fg(MARK).bg(WOOD).add_modifier(Modifier::BOLD)),
            None => ("•", line.add_modifier(Modifier::BOLD)),
        };
        if game.selected == Some(p) {
            style = style.bg(PICKED);
        }
        // A man entered or moved whose mill has still to choose what it
        // takes: shown where it went, gone from where it was.
        if let Some((from, to, _)) = &game.pending {
            if *to as usize == p {
                put(buf, x, y, "●", man(game.position.to_move).bg(PICKED));
                continue;
            }
            if *from == Some(p as u8) {
                put(buf, x, y, "•", line.add_modifier(Modifier::BOLD));
                continue;
            }
        }
        put(buf, x, y, glyph, style);
    }
    // Where the last move went, and the men it took.
    if let Some(mv) = &game.last
        && game.selected.is_none()
        && !taking
    {
        let (x, y) = at(mv.to as usize);
        put(buf, x - 1, y, "·", Style::new().fg(LINE).bg(WOOD));
        put(buf, x + 1, y, "·", Style::new().fg(LINE).bg(WOOD));
        for q in mv.taken() {
            let (x, y) = at(q);
            put(buf, x, y, "×", Style::new().fg(MARK).bg(WOOD).add_modifier(Modifier::BOLD));
        }
    }
    if game.human_to_move() {
        let (x, y) = at(game.cursor);
        let style = Style::new().fg(MARK).bg(WOOD).add_modifier(Modifier::BOLD);
        put(buf, x - 1, y, "[", style);
        put(buf, x + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 3, by, panel_w, bh.max(16)), game);
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "MORRIS", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    for line in wrap(game.rules.name, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 1;
    let to_move = game.position.to_move;
    let over = matches!(game.phase, Phase::Over { .. });
    for side in Side::BOTH {
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
        let colour = match side {
            Side::White => WHITE,
            Side::Black => Color::Rgb(110, 100, 92),
        };
        put(buf, x + 2, y, "●", Style::new().fg(colour).bg(BG));
        put(buf, x + 4, y, &format!("{} ({who})", side.name()), style);
        y += 1;
        let i = side.index();
        let hand = game.position.hand[i] as usize;
        let on = game.position.men[i].count_ones();
        let mut tray = format!("    {on} on the board");
        if hand > 0 {
            tray.push_str(&format!(", {hand} to enter"));
        }
        put(buf, x, y, &tray, Style::new().fg(DIM).bg(BG));
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
                Some(Outcome::Won(Side::White, _)) => "WHITE WINS",
                Some(Outcome::Won(Side::Black, _)) => "BLACK WINS",
                _ => "A DRAW",
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
            "Enter: enter, pick up, put down",
            "Esc: let go   Mouse: click",
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
        title: "MORRIS",
        subtitle: "Three, Six, Nine and Twelve Men's Morris: the mill games.",
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
        // The help is 84 columns wide with 3 on each side. Only a single
        // word, a web address, may be wider (the help runs it on).
        for line in HELP.lines() {
            let word = !line.trim().contains(' ');
            assert!(line.chars().count() <= 78 || word, "too wide: {line}");
        }
    }

    /// A person's turn drawn: a man picked up, where it may go lit.
    #[test]
    fn a_picked_man_lights_its_points() {
        use crate::app::{Game, Player, Rng};
        let mut game = Game::new(Rules::nine_modern(), [Player::Human, Player::Human], Rng::new(1));
        let b = game.board.clone();
        let at = |n: &str| (0..b.len()).find(|&p| b.name(p) == n).unwrap();
        game.position.hand = [0, 0];
        game.position.men = [
            1 << at("d2") | 1 << at("a1") | 1 << at("g7") | 1 << at("a4"),
            1 << at("d6") | 1 << at("a7") | 1 << at("g1"),
        ];
        game.pick(at("d2"));
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        draw_game(&mut buf, area, &game, &mut Hits::default());
        let text: String =
            (0..30).map(|y| (0..100).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect();
        assert_eq!(text.matches('◆').count(), 4, "d2 may go to d1, d3, b2, f2");
    }
}
