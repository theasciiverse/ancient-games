//! Drawing: the board of thirty houses in acacia and ebony with its marked
//! houses, the pieces in ivory and ebony, the throwing sticks or the long
//! die, the menu and the help.

use crate::app::{Game, Phase, Setup};
use crate::rules::{
    self, Action, BEAUTY, HORUS, OFF, RE_ATOUM, REBIRTH, Rules, Side, THREE_TRUTHS, Variant, WATER,
};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, centre, fill, outline, put, wrap};
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
    let top = area.y + area.height.saturating_sub(board_h + 9) / 2;
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

pub fn draw_menu(buf: &mut Buffer, area: Rect, setup: &Setup, row: usize, rules: &[Rules]) {
    fill(buf, area, BG);
    let w = area.width.min(76);
    let x = area.x + (area.width - w) / 2;
    let mut y = area.y + area.height.saturating_sub(22) / 2;
    let panel = Rect::new(x, y, w, 1);
    centre(buf, panel, y, "SENET", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    centre(
        buf,
        panel,
        y,
        "The game of passing. Egypt, from about 3000 BC. Its rules are lost.",
        Style::new().fg(DIM).bg(BG),
    );
    y += 3;
    let chosen = &rules[setup.rules];
    let items = [
        ("Rules", chosen.name.to_string()),
        ("Play against", Setup::OPPONENTS[setup.opponent].to_string()),
        ("You play", setup.side.name().to_string()),
    ];
    for (i, (label, value)) in items.iter().enumerate() {
        let active = i == row;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x + 4, y, if active { "▶" } else { " " }, style);
        put(buf, x + 6, y, label, style);
        put(buf, x + 22, y, &format!("◀ {value} ▶"), style);
        y += 2;
    }
    y += 1;
    for line in wrap(chosen.source, (w - 8) as usize) {
        put(buf, x + 6, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 2;
    let start = if row == 3 {
        Style::new().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(AMBER).bg(BG)
    };
    centre(buf, panel, y, "  Enter: play  ", start);
    y += 2;
    centre(
        buf,
        panel,
        y,
        "Up and down: choose   Left and right: change   F1: how to play   Q: quit",
        Style::new().fg(DIM).bg(BG),
    );
}

pub fn draw_help(buf: &mut Buffer, area: Rect, rules: &Rules) {
    let back = Color::Rgb(18, 18, 26);
    let w = area.width.min(84);
    let h = area.height.min(34);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    fill(buf, Rect::new(x, y, w, h), back);
    let title = Style::new().fg(AMBER).bg(back).add_modifier(Modifier::BOLD);
    let text = Style::new().fg(TEXT).bg(back);
    let dim = Style::new().fg(DIM).bg(back);
    let mut lines: Vec<(String, Style)> = vec![
        ("HOW TO PLAY".into(), title),
        (String::new(), text),
        (
            "Race your pieces along the S: the top row left to right, the middle row \
             right to left, the bottom row left to right, and off the board."
                .into(),
            text,
        ),
        (String::new(), text),
        (format!("{} RULES", rules.name.to_uppercase()), title),
    ];
    for line in rules.summary {
        lines.push((line.to_string(), text));
    }
    lines.extend([
        (String::new(), text),
        ("KEYS".into(), title),
        (
            "Space or Enter: throw   Arrows or Tab: choose   Enter: move   R: rescue from the \
             water   Mouse: click a lit piece   M: menu   Q: quit"
                .into(),
            text,
        ),
        (String::new(), text),
        ("WHAT WE KNOW".into(), title),
        (
            "Senet was played in Egypt for some three thousand years and became a game of \
             the soul's passage through the netherworld; boards were buried with the dead, \
             Tutankhamun among them. No rules survive. What you play here are modern \
             reconstructions from boards, pieces and tomb paintings: Timothy Kendall's \
             (1978) and R. C. Bell's (1960)."
                .into(),
            dim,
        ),
        (String::new(), text),
        ("Esc or F1: back".into(), dim),
    ]);
    let mut yy = y + 1;
    for (line, style) in lines {
        for part in wrap(&line, (w - 6) as usize) {
            if yy >= y + h - 1 {
                break;
            }
            put(buf, x + 3, yy, &part, style);
            yy += 1;
        }
    }
}
