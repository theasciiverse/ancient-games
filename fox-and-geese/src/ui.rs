//! Drawing: the cross of points and lines on dark wood, the white geese and
//! the fox, the cursor and the points a picked piece can go to, a side
//! panel, the menu and the help.

use crate::app::{Game, Phase, Setup};
use crate::rules::{Point, Rules, SIZE, Side, on_board, points};
use ancient_core::draw::{AMBER, BG, DIM, TEXT, centre, fill, put, wrap};
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
    let sy = (area.height.saturating_sub(8) / 6).clamp(2, 5);
    let sx = (area.width.saturating_sub(40) / 6).clamp(4, 2 * sy);
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

pub fn draw_menu(buf: &mut Buffer, area: Rect, setup: &Setup, row: usize, rules: &[Rules]) {
    fill(buf, area, BG);
    let w = area.width.min(76);
    let x = area.x + (area.width - w) / 2;
    let mut y = area.y + area.height.saturating_sub(22) / 2;
    let panel = Rect::new(x, y, w, 1);
    centre(buf, panel, y, "FOX AND GEESE", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    centre(
        buf,
        panel,
        y,
        "A hunt game of medieval Europe, as R. C. Bell gives it.",
        Style::new().fg(DIM).bg(BG),
    );
    y += 3;
    let chosen = &rules[setup.rules];
    let side = match setup.side {
        Side::Fox => "the fox",
        Side::Geese => "the geese",
    };
    let items = [
        ("Rules", chosen.name.to_string()),
        ("Play against", Setup::OPPONENTS[setup.opponent].to_string()),
        ("You play", side.to_string()),
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
    let h = area.height.min(32);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    fill(buf, Rect::new(x, y, w, h), back);
    let title = Style::new().fg(AMBER).bg(back).add_modifier(Modifier::BOLD);
    let text = Style::new().fg(TEXT).bg(back);
    let dim = Style::new().fg(DIM).bg(back);
    let mut lines: Vec<(String, Style)> = vec![
        ("HOW TO PLAY".into(), title),
        (String::new(), text),
        (format!("{} RULES", rules.name.to_uppercase()), title),
    ];
    for line in rules.summary {
        lines.push((line.to_string(), text));
    }
    lines.extend([
        (
            "A position repeated a third time is a draw: Bell gives no end to endless \
             shuffling, so that rule is this game's own."
                .to_string(),
            text,
        ),
        (String::new(), text),
        ("KEYS".into(), title),
        (
            "Arrows: move the cursor   Enter: pick up a piece, then put it down on a lit \
             point   Esc: drop it   Mouse: click a piece, then a point   M: menu   Q: quit"
                .into(),
            text,
        ),
        (String::new(), text),
        ("WHAT WE KNOW".into(), title),
        (
            "Hunt games of one strong piece against many are old in northern Europe: the \
             Icelandic Grettis saga mentions Hala-tafl, the fox game, and Edward IV's \
             household bought two foxes and twenty-six hounds of silver gilt for two sets of \
             'Marelles' in the 1460s. The rules here are R. C. Bell's (1960)."
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
