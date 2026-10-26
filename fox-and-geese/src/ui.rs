//! Drawing: the cross of points and lines on dark wood, the white geese and
//! the fox, the cursor and the points a picked piece can go to, a side
//! panel, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{Point, Rules, SIZE, Side, on_board, points};
use ancient_core::ornament;
use ancient_core::draw::{AMBER, BG, DIM, TEXT, fill, put, wrap};
use ancient_core::shell::{self, Help, Menu};
use ancient_core::player::Player;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const WOOD: Color = Color::Rgb(44, 30, 18);
const LINE: Color = Color::Rgb(150, 118, 76);
const GOOSE: Color = Color::Rgb(246, 244, 236);
const FOX: Color = Color::Rgb(232, 112, 36);
const BLOOD: Color = Color::Rgb(200, 50, 40);

/// Where each point was drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub points: Vec<(Rect, Point)>,
}

/// Rows and columns between points: about twice as wide as tall.
fn spacing(area: Rect) -> (u16, u16) {
    let sy = (area.height.saturating_sub(8) / 6).clamp(2, 10);
    let sx = (area.width.saturating_sub(41) / 6).clamp(4, 2 * sy);
    (sx, sy)
}

/// The box-drawing junction for an empty point: lines only where the
/// board's lines run.
fn junction(p: Point) -> &'static str {
    let (r, c) = p;
    let up = r > 0 && on_board((r - 1, c));
    let down = r + 1 < SIZE && on_board((r + 1, c));
    let left = c > 0 && on_board((r, c - 1));
    let right = c + 1 < SIZE && on_board((r, c + 1));
    match (up, down, left, right) {
        (true, true, true, true) => "┼",
        (false, true, true, true) => "┬",
        (true, false, true, true) => "┴",
        (true, true, false, true) => "├",
        (true, true, true, false) => "┤",
        (false, true, false, true) => "┌",
        (false, true, true, false) => "┐",
        (true, false, false, true) => "└",
        (true, false, true, false) => "┘",
        _ => "·",
    }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.points.clear();
    let (sx, sy) = spacing(area);
    let bw = 6 * sx + 5;
    let bh = 6 * sy + 3;
    let panel_w = 34u16;
    let total = bw + 2 + panel_w;
    let bx = area.x + area.width.saturating_sub(total) / 2;
    let by = area.y + area.height.saturating_sub(bh) / 2;
    fill(buf, Rect::new(bx, by, bw, bh), WOOD);
    let at = |p: Point| (bx + 2 + p.1 as u16 * sx, by + 1 + p.0 as u16 * sy);

    // The lines along the rows and columns.
    let line = Style::new().fg(LINE).bg(WOOD);
    for p in points() {
        let (x, y) = at(p);
        let right = (p.0, p.1 + 1);
        if p.1 + 1 < SIZE && on_board(right) {
            for dx in 1..sx {
                put(buf, x + dx, y, "─", line);
            }
        }
        let down = (p.0 + 1, p.1);
        if p.0 + 1 < SIZE && on_board(down) {
            for dy in 1..sy {
                put(buf, x, y + dy, "│", line);
            }
        }
    }

    let targets = game.targets();
    // Geese the move under the cursor would kill.
    let doomed: Vec<Point> = targets
        .iter()
        .find(|m| m.to() == game.cursor)
        .map(|m| m.killed())
        .unwrap_or_default();

    for p in points() {
        let (x, y) = at(p);
        hits.points.push((Rect::new(x.saturating_sub(1), y, 3, 1), p));
        let (glyph, style) = if p == game.position.fox {
            ("◆", Style::new().fg(FOX).bg(WOOD).add_modifier(Modifier::BOLD))
        } else if game.position.is_goose(p) {
            let colour = if doomed.contains(&p) { BLOOD } else { GOOSE };
            ("●", Style::new().fg(colour).bg(WOOD).add_modifier(Modifier::BOLD))
        } else if targets.iter().any(|m| m.to() == p) {
            ("◇", Style::new().fg(AMBER).bg(WOOD).add_modifier(Modifier::BOLD))
        } else {
            (junction(p), line)
        };
        let style = if game.selected == Some(p) { style.bg(Color::Rgb(90, 64, 20)) } else { style };
        put(buf, x, y, glyph, style);
    }
    // Where the last move went.
    if let Some(mv) = &game.last
        && game.selected.is_none()
    {
        let (x, y) = at(mv.to());
        put(buf, x - 1, y, "·", Style::new().fg(DIM).bg(WOOD));
        put(buf, x + 1, y, "·", Style::new().fg(DIM).bg(WOOD));
    }
    if game.human_to_move() {
        let (x, y) = at(game.cursor);
        let style = Style::new().fg(AMBER).bg(WOOD).add_modifier(Modifier::BOLD);
        put(buf, x - 1, y, "[", style);
        put(buf, x + 1, y, "]", style);
    }

    draw_panel(buf, Rect::new(bx + bw + 2, by, panel_w, bh), game);

    // In the four corners outside the cross, the inlaid lozenges of the
    // fox-and-geese boards in a games box probably from Granada, 1525 to
    // 1575 (Victoria and Albert Museum 154-1900). Clear of the lines and
    // of the brackets beside the points.
    let (w, h) = (2 * sx - 1, 2 * sy);
    let right = bx + 2 + 4 * sx + 2;
    let bottom = by + 1 + 4 * sy + 1;
    for (x, y) in [(bx, by), (right, by), (bx, bottom), (right, bottom)] {
        draw_lozenge(buf, Rect::new(x, y, w, h));
    }
}

/// A lozenge after the Granada box: an ivory outline, a ring of
/// sawtooth, an eight-pointed star of two squares, and a rosette of eight
/// petals on green in the middle.
fn draw_lozenge(buf: &mut Buffer, area: Rect) {
    let ivory = ornament::dim(GOOSE, WOOD, 0.7);
    let green = ornament::dim(Color::Rgb(110, 160, 100), WOOD, 0.45);
    let (cx, cy) = (area.width as i32, area.height as i32 * 2);
    let r = cx.min(cy) - 2;
    if r < 7 {
        return;
    }
    ornament::paint_in(buf, area, |x, y| {
        let (x, y) = (x - cx, y - cy);
        let d = x.abs() + y.abs();
        let along = x.abs() - y.abs();
        let outline = d == r || d == r - 4;
        // The sawtooth between the two lines, as a zigzag.
        let tooth = d > r - 4 && d < r && along.rem_euclid(6).min(6 - along.rem_euclid(6)) == r - 1 - d;
        let s = (r - 5) * 5 / 9;
        let square = x.abs().max(y.abs()) == s;
        let diamond = d == s * 7 / 5;
        let star = (square && d <= s * 7 / 5) || (diamond && x.abs().max(y.abs()) <= s);
        if outline || tooth || star {
            return Some(ivory);
        }
        // A large rosette in outline, or it fills in.
        let rosette = |x, y| ornament::rosette(8, (s - 1) as f32, x, y);
        let k = rosette(x, y)?;
        (s < 7 || k == 0 || ornament::edge(x, y, rosette)).then_some(green)
    });
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "FOX AND GEESE", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    put(buf, x, y, game.rules.name, Style::new().fg(DIM).bg(BG));
    y += 2;
    let to_move = game.position.to_move;
    let over = matches!(game.phase, Phase::Over { .. });
    for side in [Side::Fox, Side::Geese] {
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
        let (glyph, colour) = match side {
            Side::Fox => ("◆", FOX),
            Side::Geese => ("●", GOOSE),
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        put(buf, x + 2, y, glyph, Style::new().fg(colour).bg(BG));
        let name = match side {
            Side::Fox => "Fox",
            Side::Geese => "Geese",
        };
        put(buf, x + 4, y, &format!("{name} ({who})"), style);
        y += 1;
    }
    y += 1;
    let left = game.position.geese.len();
    let killed = game.rules.geese.len() - left;
    put(buf, x, y, &format!("{left} geese left, {killed} killed"), Style::new().fg(TEXT).bg(BG));
    y += 2;
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    match &game.phase {
        Phase::Thinking { .. } => put(buf, x, y, "thinking...", Style::new().fg(DIM).bg(BG)),
        Phase::Over { winner } => {
            let result = match winner {
                Some(Side::Fox) => "THE FOX WINS",
                Some(Side::Geese) => "THE GEESE WIN",
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
        title: "FOX AND GEESE",
        subtitle: "A hunt game of medieval Europe, as R. C. Bell gives it.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["the fox", "the geese"],
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
