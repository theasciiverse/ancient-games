//! Drawing: the board of thirty houses in acacia and ebony with its marked
//! houses, the pieces in ivory and ebony, the throwing sticks or the long
//! die, the menu and the help.

use crate::app::{Game, Phase};
use crate::rules::{
    self, Action, BEAUTY, HORUS, OFF, RE_ATOUM, REBIRTH, Rules, Side, THREE_TRUTHS, Variant, WATER,
};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, centre, fill, outline, put};
use ancient_core::ornament;
use ancient_core::shell::{self, Help, Menu};
use ancient_core::player::Player;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const EBONY: Color = Color::Rgb(46, 30, 20);
const ACACIA: Color = Color::Rgb(196, 158, 104);
const ACACIA_DARK: Color = Color::Rgb(170, 132, 82);
const FAIENCE: Color = Color::Rgb(24, 116, 138);
const IVORY: Color = Color::Rgb(240, 232, 212);
const EBONY_PIECE: Color = Color::Rgb(30, 22, 18);
const WATER_BLUE: Color = Color::Rgb(40, 96, 170);

/// Where things were drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    pub squares: Vec<(Rect, u8)>,
}

fn square_size(area: Rect) -> (u16, u16) {
    let by_height = (area.height.saturating_sub(13) / 3).clamp(3, 7);
    let width = (area.width.saturating_sub(2) / 10).clamp(6, 14);
    let h = by_height.min(width.div_ceil(2));
    (width.min(h * 2), h)
}

/// The mark of a house, as on the boards the sources describe.
fn mark(rules: &Rules, house: u8) -> Option<(&'static str, Color)> {
    let bell = rules.variant == Variant::Bell;
    match house {
        REBIRTH if !bell => Some(("☥", FAIENCE)),
        BEAUTY if bell => Some(("⊓", FAIENCE)),
        BEAUTY => Some(("◯", FAIENCE)),
        WATER if bell => Some(("X", FAIENCE)),
        WATER => Some(("≈≈≈", WATER_BLUE)),
        THREE_TRUTHS => Some(("III", FAIENCE)),
        RE_ATOUM => Some(("II", FAIENCE)),
        HORUS => Some(("I", FAIENCE)),
        _ => None,
    }
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.squares.clear();
    let (sw, sh) = square_size(area);
    let board_w = sw * 10 + 2;
    let board_h = sh * 3 + 2;
    let bx = area.x + area.width.saturating_sub(board_w) / 2;
    // Three rows above the board, eight below: names, throw, message, keys.
    let top = area.y + area.height.saturating_sub(board_h + 11) / 2;
    let by = top + 3;

    centre(buf, area, top, "SENET", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    centre(buf, area, top + 1, &format!("{} rules", game.rules.name), Style::new().fg(DIM).bg(BG));
    side_label(buf, bx, board_w, by - 1, game, Side::Black);

    fill(buf, Rect::new(bx, by, board_w, board_h), EBONY);
    for house in 1..=rules::HOUSES {
        let cell = cell_of(bx, by, sw, sh, house);
        draw_house(buf, cell, house, &game.rules);
        hits.squares.push((cell, house));
        if let Some(side) = game.position.at(house) {
            draw_piece(buf, cell, side);
        }
    }

    match &game.phase {
        Phase::Choosing { moves, selected, .. } => {
            for (i, action) in moves.iter().enumerate() {
                let Action::Move { from, to } = *action else { continue };
                let chosen = i == *selected;
                outline(buf, inner(cell_of(bx, by, sw, sh, from)), if chosen { AMBER } else { ACACIA_DARK }, chosen);
                if chosen {
                    if to == OFF {
                        put(buf, bx + board_w - 6, by + board_h - 1, " off ", Style::new().fg(BG).bg(AMBER));
                    } else {
                        let cell = inner(cell_of(bx, by, sw, sh, to));
                        outline(buf, cell, AMBER, false);
                        let x = cell.x + cell.width.saturating_sub(4) / 2;
                        put(buf, x, cell.bottom() - 1, "here", Style::new().fg(AMBER).bg(EBONY));
                    }
                }
            }
        }
        _ => {
            if let Some((_, Action::Move { to, .. })) = game.last
                && to != OFF
            {
                outline(buf, inner(cell_of(bx, by, sw, sh, to)), DIM, false);
            }
        }
    }

    side_label(buf, bx, board_w, by + board_h, game, Side::White);

    let ly = by + board_h + 2;
    draw_throw(buf, area, ly, game);
    centre(buf, area, ly + 4, &game.message, Style::new().fg(TEXT).bg(BG));
    let keys = match game.phase {
        Phase::Over { .. } => "Enter: play again   M: menu   F1: help   Q: quit",
        Phase::Choosing { .. } => "Arrows or Tab: choose   Enter: move   F1: help   Q: quit",
        _ if game.human_to_move() && game.can_rescue() => {
            "Space: throw   R: rescue from the water   F1: help   Q: quit"
        }
        _ if game.human_to_move() => "Space: throw   F1: help   Q: quit",
        _ => "F1: help   Q: quit",
    };
    centre(buf, area, ly + 5, keys, Style::new().fg(DIM).bg(BG));

    // Last, so it only goes where nothing else is.
    let board = Rect::new(bx, by, board_w, board_h);
    draw_ornament(buf, area, board);
}

/// Ornament after the faience senet box inscribed for Amenhotep III
/// (Brooklyn Museum 49.56a-b): the checker of light and dark blue on its
/// drawer's ends round the rim, and on a terminal larger than the game is
/// made for, the panels of its long sides beside the board, ankh and djed
/// in turn, each between double rules.
fn draw_ornament(buf: &mut Buffer, area: Rect, board: Rect) {
    let glaze = ornament::dim(Color::Rgb(70, 150, 230), EBONY, 0.5);
    // Squares of two by two dots, two rows of them in a cell.
    let checker = |x: i32, y: i32| (x / 2 + y / 2) % 2 == 0;
    ornament::rim(buf, board, glaze, 4, checker, |_, y| (y / 2) % 2 == 0);
    if !ornament::roomy(area) {
        return;
    }
    let faience = ornament::dim(Color::Rgb(70, 150, 230), BG, 0.5);
    let line = ornament::dim(FAIENCE, BG, 0.3);
    for column in ornament::beside(area, board) {
        ornament::stack(buf, column, 17, 24, |i, x, y| {
            // The panel: double rules at the sides, a rule above and below.
            let side = matches!(x.abs(), 7 | 8) && y.abs() <= 11;
            let end = y.abs() == 11 && x.abs() <= 8;
            if side || end {
                return Some(line);
            }
            let sign = if i % 2 == 0 { ankh(x, y) } else { djed(x, y) };
            sign.then_some(faience)
        });
    }
}

/// The ankh, 11 by 19 dots about its centre: a loop, the arms, the stem
/// widening downward.
fn ankh(x: i32, y: i32) -> bool {
    let (fx, fy) = (x as f32 / 3.4, (y + 5) as f32 / 4.4);
    let ring = fx * fx + fy * fy;
    let arms = (0..=1).contains(&y) && x.abs() <= 5;
    let stem = (2..=9).contains(&y) && x.abs() <= 1 + (y - 2) / 5;
    (0.45..=1.3).contains(&ring) || arms || stem
}

/// The djed: four bars at the top of a pillar on a base.
fn djed(x: i32, y: i32) -> bool {
    let bar = matches!(y, -9 | -7 | -5 | -3) && x.abs() <= 5;
    let pillar = (-9..=8).contains(&y) && x.abs() <= 1;
    let base = (8..=9).contains(&y) && x.abs() <= 3;
    bar || pillar || base
}

fn cell_of(bx: u16, by: u16, sw: u16, sh: u16, house: u8) -> Rect {
    let (row, col) = rules::square(house);
    Rect::new(bx + 1 + col as u16 * sw, by + 1 + row as u16 * sh, sw, sh)
}

/// A square without the gap on its right and bottom.
fn inner(cell: Rect) -> Rect {
    Rect::new(cell.x, cell.y, cell.width - 1, cell.height - 1)
}

fn draw_house(buf: &mut Buffer, cell: Rect, house: u8, rules: &Rules) {
    let inner = inner(cell);
    fill(buf, inner, ACACIA);
    // The house's number, small, in the corner: it helps to learn the S.
    put(buf, inner.x, inner.y, &house.to_string(), Style::new().fg(ACACIA_DARK).bg(ACACIA));
    if let Some((text, colour)) = mark(rules, house) {
        let x = inner.x + inner.width.saturating_sub(text.chars().count() as u16) / 2;
        let y = inner.y + inner.height / 2;
        put(buf, x, y, text, Style::new().fg(colour).bg(ACACIA).add_modifier(Modifier::BOLD));
    }
}

/// A piece: ivory or ebony.
fn draw_piece(buf: &mut Buffer, cell: Rect, side: Side) {
    let inner = inner(cell);
    let (bg, fg) = match side {
        Side::White => (IVORY, EBONY),
        Side::Black => (EBONY_PIECE, IVORY),
    };
    let w = inner.width.saturating_sub(4).clamp(3, 6);
    let h = inner.height.saturating_sub(2).clamp(1, 2);
    let x = inner.x + (inner.width - w) / 2;
    let y = inner.y + (inner.height - h).div_ceil(2);
    fill(buf, Rect::new(x, y, w, h), bg);
    let glyph = match side {
        Side::White => "▲",
        Side::Black => "■",
    };
    put(buf, x + w / 2, y + (h - 1) / 2, glyph, Style::new().fg(fg).bg(bg));
}

fn side_label(buf: &mut Buffer, bx: u16, board_w: u16, y: u16, game: &Game, side: Side) {
    let who = match game.player(side) {
        Player::Human => "you".to_string(),
        Player::Computer(s) => format!("computer, {}", s.name()),
    };
    let to_move = game.to_move() == side && !matches!(game.phase, Phase::Over { .. });
    let style = if to_move {
        Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(DIM).bg(BG)
    };
    put(buf, bx, y, if to_move { "▶ " } else { "  " }, style);
    let (swatch_bg, swatch) = match side {
        Side::White => (IVORY, "▲"),
        Side::Black => (EBONY_PIECE, "■"),
    };
    put(buf, bx + 2, y, swatch, Style::new().fg(if side == Side::White { EBONY } else { IVORY }).bg(swatch_bg));
    put(buf, bx + 4, y, &format!("{} ({who})", side.name()), style);
    let off = game.position.off[side.index()];
    let mut tally = format!("{off} borne off");
    if game.rules.variant == Variant::Bell {
        let left = game.position.on_board(side) as u8 + off;
        tally.push_str(&format!(", {} taken", game.rules.pieces - left));
    }
    put(
        buf,
        bx + board_w.saturating_sub(tally.len() as u16),
        y,
        &tally,
        Style::new().fg(AMBER).bg(BG),
    );
}

/// The four throwing sticks, light side or dark side up; or Bell's long die.
fn draw_throw(buf: &mut Buffer, area: Rect, y: u16, game: &Game) {
    let thrown = !matches!(game.phase, Phase::ToThrow);
    let tumbling = matches!(game.phase, Phase::Throwing { .. });
    let result = if tumbling {
        "...".to_string()
    } else if thrown {
        format!("= {}", game.shown)
    } else {
        String::new()
    };
    let result_style = Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD);
    if game.rules.variant == Variant::Bell {
        let x = area.x + area.width.saturating_sub(24) / 2;
        let face = if thrown { game.die.to_string() } else { " ".to_string() };
        put(buf, x, y, "long die", Style::new().fg(DIM).bg(BG));
        put(buf, x, y + 1, &format!("▕ {face} ▏▕   ▏"), Style::new().fg(EBONY).bg(IVORY));
        put(buf, x + 14, y + 1, &result, result_style);
        return;
    }
    let x0 = area.x + area.width.saturating_sub(30) / 2;
    for (i, &light) in game.sticks.iter().enumerate() {
        let x = x0 + i as u16 * 4;
        let colour = if thrown && light { IVORY } else { EBONY };
        for dy in 0..3 {
            put(buf, x, y + dy, "▐▌", Style::new().fg(colour).bg(BG));
        }
    }
    put(buf, x0 + 18, y + 1, &result, result_style);
}

/// The menu's text.
pub fn menu(rules: &[Rules], chosen: usize) -> Menu<'_> {
    Menu {
        title: "SENET",
        subtitle: "The game of passing. Egypt, from about 3000 BC. Its rules are lost.",
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
mod tests {
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
