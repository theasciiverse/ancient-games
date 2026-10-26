//! Drawing: the snake coiled on a limestone disc, tail outside and head in
//! the middle, the pharaohs' marbles and the lions on them, the hand that
//! hides the marbles, a side panel, the menu and the help.

use crate::app::{Game, Phase, lion_word};
use crate::rules::{GRID, HOME, LIONS, Lion, Rules, SQUARES, Team, coil};
use ancient_core::ornament;
use ancient_core::draw::{AMBER, BG, DIM, TEXT, centre, fill, put, wrap};
use ancient_core::player::Player;
use ancient_core::shell::{self, Help, Menu};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

const DISC: Color = Color::Rgb(58, 50, 40);
const STONE_A: Color = Color::Rgb(206, 190, 158);
const STONE_B: Color = Color::Rgb(190, 172, 138);
const INCISED: Color = Color::Rgb(120, 104, 80);
const HEAD: Color = Color::Rgb(24, 116, 138);
const PICKED: Color = Color::Rgb(238, 200, 120);
const LION_GOLD: Color = Color::Rgb(214, 150, 30);
const LIONESS_SAND: Color = Color::Rgb(150, 96, 60);

/// The players' marbles: warm for the lions, cool for the lionesses.
const MARBLES: [[Color; 3]; 2] = [
    [Color::Rgb(196, 48, 36), Color::Rgb(232, 132, 30), Color::Rgb(150, 30, 110)],
    [Color::Rgb(40, 96, 210), Color::Rgb(30, 150, 120), Color::Rgb(110, 80, 200)],
];

pub fn marble(game: &Game, p: usize) -> Color {
    let team = game.state.team_of(p);
    let first = if team == Team::Lions { 0 } else { game.rules.pharaohs };
    MARBLES[team.index()][p - first]
}

fn cat(team: Team) -> Color {
    match team {
        Team::Lions => LION_GOLD,
        Team::Lionesses => LIONESS_SAND,
    }
}

/// Where things were drawn, for mouse clicks.
#[derive(Default, Clone)]
pub struct Hits {
    /// Pharaohs, by number.
    pub pharaohs: Vec<(Rect, usize)>,
    /// Number buttons.
    pub numbers: Vec<(Rect, u8)>,
    /// Yes and no.
    pub answers: Vec<(Rect, bool)>,
}

const PANEL: u16 = 36;

fn cell_size(area: Rect) -> (u16, u16) {
    let (w, h) = (GRID.0 as u16, GRID.1 as u16);
    let by_height = area.height.saturating_sub(12) / h;
    let by_width = area.width.saturating_sub(PANEL + 4) / w / 2;
    let ch = by_height.min(by_width).clamp(1, 4);
    (2 * ch, ch)
}

pub fn draw_game(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    fill(buf, area, BG);
    hits.pharaohs.clear();
    hits.numbers.clear();
    hits.answers.clear();
    let (cw, ch) = cell_size(area);
    let bw = GRID.0 as u16 * cw + 2;
    let bh = GRID.1 as u16 * ch + 2;
    let total_w = bw + 2 + PANEL;
    let total_h = bh + 8;
    let bx = area.x + area.width.saturating_sub(total_w) / 2;
    let top = area.y + area.height.saturating_sub(total_h) / 2;
    let by = top + 2;

    if let Phase::LookAway { team, .. } = &game.phase {
        draw_look_away(buf, area, *team);
        return;
    }

    let chosen = match &game.phase {
        Phase::Choose { options, selected } | Phase::Pick { options, selected, .. } => Some(options[*selected]),
        _ => None,
    };
    let choosable: Vec<usize> = match &game.phase {
        Phase::Choose { options, .. } | Phase::Pick { options, .. } if game.human_acting() => options.clone(),
        _ => Vec::new(),
    };

    // The start, beside the tail.
    put(buf, bx, top, "start", Style::new().fg(DIM).bg(BG));
    let mut sx = bx + 7;
    for p in 0..game.state.pharaohs.len() {
        if game.state.pharaohs[p].pos == 0 {
            let rect = Rect::new(sx, top, 3, 1);
            hits.pharaohs.push((rect, p));
            let style = Style::new().fg(marble(game, p)).bg(if chosen == Some(p) { PICKED } else { BG });
            put(buf, sx, top, "●", style);
            if let Some((team, _)) = game.state.attacker(p) {
                put(buf, sx + 1, top, "L", Style::new().fg(cat(team)).bg(BG).add_modifier(Modifier::BOLD));
            }
            sx += 3;
        }
    }

    fill(buf, Rect::new(bx, by, bw, bh), DISC);
    let path = coil(GRID.0, GRID.1);
    for (i, &(r, c)) in path.iter().enumerate() {
        let square = i as u8 + 1;
        let rect = Rect::new(bx + 1 + c as u16 * cw, by + 1 + r as u16 * ch, cw, ch);
        let occupant = (0..game.state.pharaohs.len()).find(|&p| game.state.pharaohs[p].pos == square);
        let stone = if occupant.is_some() && occupant == chosen {
            PICKED
        } else if square == SQUARES {
            HEAD
        } else if i % 2 == 0 {
            STONE_A
        } else {
            STONE_B
        };
        fill(buf, rect, stone);
        let mid_x = rect.x + cw / 2 - 1;
        let mid_y = rect.y + (ch - 1) / 2;
        match occupant {
            Some(p) => {
                hits.pharaohs.push((rect, p));
                draw_marble(buf, rect, marble(game, p), stone);
                if let Some((team, _)) = game.state.attacker(p) {
                    put(buf, rect.right() - 1, rect.y, "L", Style::new().fg(cat(team)).bg(DISC).add_modifier(Modifier::BOLD));
                }
                if choosable.contains(&p) {
                    frame(buf, rect, if chosen == Some(p) { AMBER } else { INCISED });
                }
            }
            None if square == SQUARES => {
                put(buf, mid_x, mid_y, "☥", Style::new().fg(PICKED).bg(stone).add_modifier(Modifier::BOLD));
            }
            None if square == 1 => put(buf, mid_x, mid_y, "≈≈", Style::new().fg(INCISED).bg(stone)),
            None if square % 10 == 0 => {
                put(buf, mid_x, mid_y, &format!("{square:>2}"), Style::new().fg(INCISED).bg(stone));
            }
            None => {}
        }
    }

    draw_hand(buf, Rect::new(bx, by + bh + 1, bw, 6), game, hits);
    draw_panel(buf, Rect::new(bx + bw + 2, top, PANEL, total_h.max(22)), game);

    // The scales of the serpent's tail round the rim, after the alabaster
    // board in Chicago (ISAC E16950): a lattice of small lozenges cut in
    // two sets of diagonals. Last, so it only goes where nothing else is.
    let scales = ornament::dim(INCISED, DISC, 0.35);
    let lattice = |x: i32, y: i32| (x + y) % 4 == 0 || (x - y).rem_euclid(4) == 0;
    ornament::rim(buf, Rect::new(bx, by, bw, bh), scales, 4, lattice, lattice);
}

/// Thin bars down both sides of a square.
fn frame(buf: &mut Buffer, rect: Rect, colour: Color) {
    for y in rect.y..rect.bottom() {
        if let Some(cell) = buf.cell_mut((rect.x, y)) {
            cell.set_char('▏').set_fg(colour);
        }
        if let Some(cell) = buf.cell_mut((rect.right() - 1, y)) {
            cell.set_char('▕').set_fg(colour);
        }
    }
}

/// A marble, round, in the middle of its square.
fn draw_marble(buf: &mut Buffer, rect: Rect, colour: Color, stone: Color) {
    let (cw, ch) = (rect.width, rect.height);
    if ch == 1 {
        put(buf, rect.x + cw / 2 - 1, rect.y, "●", Style::new().fg(colour).bg(stone));
        return;
    }
    let w = cw - 2;
    let x = rect.x + 1;
    for row in 0..ch {
        let top = 2 * row;
        let upper = top >= 1 && top <= 2 * ch - 2;
        let lower = top + 1 <= 2 * ch - 2;
        let glyph = match (upper, lower) {
            (true, true) => "█",
            (false, true) => "▄",
            (true, false) => "▀",
            _ => continue,
        };
        for dx in 0..w {
            put(buf, x + dx, rect.y + row, glyph, Style::new().fg(colour).bg(stone));
        }
    }
}

/// Below the board: what is asked now, the hand, and buttons.
fn draw_hand(buf: &mut Buffer, area: Rect, game: &Game, hits: &mut Hits) {
    let human = game.human_acting();
    let acting = game.acting();
    let text = Style::new().fg(TEXT).bg(BG);
    let x = area.x;
    let y = area.y;
    // Names the team when two people share the keyboard.
    let lead = |t: Team| {
        if game.players.iter().all(|p| *p == Player::Human) { format!("{}: ", t.title()) } else { String::new() }
    };
    let prompt = match (&game.phase, acting) {
        (Phase::Choose { .. }, Some(t)) if human => format!("{}Choose which pharaoh moves.", lead(t)),
        (Phase::Choose { .. }, Some(t)) => format!("{} choose a pharaoh...", t.title()),
        (Phase::FightHide { p }, Some(t)) if human => format!(
            "{}Your {} holds {}' pharaoh. Hide 1 to 3 marbles.",
            lead(t),
            lion_word(t),
            game.state.team_of(*p).name()
        ),
        (Phase::FightHide { .. }, Some(t)) => format!("{} hide marbles for their {}...", t.title(), lion_word(t)),
        (Phase::FightGuess { .. }, Some(t)) if human => {
            format!("{}Guess the marbles, 1 to 3, to kill the {}.", lead(t), lion_word(t.other()))
        }
        (Phase::FightGuess { .. }, Some(t)) => format!("{} guess, to kill the {}...", t.title(), lion_word(t.other())),
        (Phase::Hide { .. }, Some(t)) if human => format!("{}Hide 1 to 5 marbles in your fist.", lead(t)),
        (Phase::Hide { .. }, Some(t)) => format!("{} hide marbles...", t.title()),
        (Phase::Guess { .. }, Some(t)) if human => format!("{}How many marbles are in the fist?", lead(t)),
        (Phase::Guess { .. }, Some(t)) => format!("{} guess...", t.title()),
        (Phase::Pick { steps, .. }, Some(t)) => format!("{}Choose which of your pharaohs moves {steps}.", lead(t)),
        (Phase::Unleash { .. }, Some(t)) => {
            format!("{}Set a {} on {}' pharaoh?", lead(t), lion_word(t), t.other().name())
        }
        (Phase::Over { winner }, _) => format!("{} win!", winner.title()),
        _ => String::new(),
    };
    put(buf, x, y, &prompt, text);

    // The fist, closed or open.
    let (closed, shown) = match &game.phase {
        Phase::Guess { .. } | Phase::FightGuess { .. } => (true, None),
        Phase::Reveal { hidden, guess, .. } => (false, Some((*hidden, *guess))),
        _ => (false, None),
    };
    let palm = Color::Rgb(176, 124, 84);
    if closed {
        put(buf, x, y + 2, " ▄▄▄▄▄▄ ", Style::new().fg(palm).bg(BG));
        put(buf, x, y + 3, "▐██??██▌", Style::new().fg(palm).bg(BG));
        put(buf, x + 3, y + 3, "??", Style::new().fg(BG).bg(palm).add_modifier(Modifier::BOLD));
        put(buf, x, y + 4, " ▀▀▀▀▀▀ ", Style::new().fg(palm).bg(BG));
    } else if let Some((hidden, guess)) = shown {
        let colour = Color::Rgb(232, 228, 216);
        let w = 2 * hidden as u16 + 3;
        for row in 2..5 {
            put(buf, x, y + row, &" ".repeat(w as usize), Style::new().bg(palm));
        }
        let mut mx = x + 2;
        for _ in 0..hidden {
            put(buf, mx, y + 3, "●", Style::new().fg(colour).bg(palm));
            mx += 2;
        }
        let verdict = if hidden == guess { "right!" } else { "wrong" };
        let style = Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD);
        put(buf, x + w + 2, y + 3, &format!("{hidden} hidden, {guess} guessed: {verdict}"), style);
    }

    // Buttons for the person.
    let bx = x + 12;
    if human {
        match &game.phase {
            Phase::FightHide { .. } | Phase::FightGuess { .. } | Phase::Hide { .. } | Phase::Guess { .. } => {
                let max = if matches!(game.phase, Phase::Hide { .. } | Phase::Guess { .. }) { 5 } else { 3 };
                for n in 1..=max {
                    let rect = Rect::new(bx + (n as u16 - 1) * 6, y + 3, 5, 1);
                    hits.numbers.push((rect, n));
                    let style = if n == game.number {
                        Style::new().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(TEXT).bg(Color::Rgb(40, 40, 48))
                    };
                    put(buf, rect.x, rect.y, &format!("  {n}  "), style);
                }
            }
            Phase::Unleash { .. } => {
                for (i, (label, yes)) in [("  Yes  ", true), ("  No  ", false)].into_iter().enumerate() {
                    let rect = Rect::new(bx + i as u16 * 9, y + 3, label.len() as u16, 1);
                    hits.answers.push((rect, yes));
                    let style = if yes {
                        Style::new().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(TEXT).bg(Color::Rgb(40, 40, 48))
                    };
                    put(buf, rect.x, rect.y, label, style);
                }
            }
            _ => {}
        }
    }
}

fn draw_look_away(buf: &mut Buffer, area: Rect, team: Team) {
    let y = area.y + area.height / 2 - 3;
    let bold = Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD);
    centre(buf, area, y, &format!("{}, look away!", team.other().title()), bold);
    centre(buf, area, y + 2, &format!("{} hide their marbles now.", team.title()), Style::new().fg(TEXT).bg(BG));
    centre(
        buf,
        area,
        y + 4,
        &format!("{}: press Enter when they cannot see.", team.title()),
        Style::new().fg(DIM).bg(BG),
    );
}

fn draw_panel(buf: &mut Buffer, panel: Rect, game: &Game) {
    let mut y = panel.y;
    let x = panel.x;
    put(buf, x, y, "MEHEN", Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    put(buf, x, y, game.rules.name, Style::new().fg(DIM).bg(BG));
    y += 2;
    let over = matches!(game.phase, Phase::Over { .. });
    for team in [Team::Lions, Team::Lionesses] {
        let who = match game.player(team) {
            Player::Human => "you".to_string(),
            Player::Computer(s) => format!("computer, {}", s.name()),
        };
        let active = team == game.state.to_move && !over;
        let style = if active {
            Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(TEXT).bg(BG)
        };
        put(buf, x, y, if active { "▶" } else { " " }, style);
        put(buf, x + 2, y, &format!("{} ({who})", team.title()), style);
        y += 1;
        // Each pharaoh: where it is.
        let mut px = x + 4;
        for p in 0..game.state.pharaohs.len() {
            if game.state.team_of(p) != team {
                continue;
            }
            let pos = game.state.pharaohs[p].pos;
            let place = match pos {
                0 => "start".to_string(),
                HOME => "reborn".to_string(),
                n => n.to_string(),
            };
            put(buf, px, y, "●", Style::new().fg(marble(game, p)).bg(BG));
            put(buf, px + 2, y, &place, Style::new().fg(DIM).bg(BG));
            px += place.len() as u16 + 4;
        }
        y += 1;
        // The big cats: in reserve, hunting, dead.
        let lions = &game.state.lions[team.index()];
        let mut lx = x + 4;
        for lion in lions.iter().take(LIONS) {
            let (glyph, colour) = match lion {
                Lion::Reserve => ("L", cat(team)),
                Lion::Attacking(_) => ("L", Color::Rgb(220, 60, 40)),
                Lion::Dead => ("†", DIM),
            };
            put(buf, lx, y, glyph, Style::new().fg(colour).bg(BG).add_modifier(Modifier::BOLD));
            lx += 2;
        }
        let reserve = game.state.count(team, Lion::Reserve);
        let hunting = game.state.count(team, Lion::Attacking(0));
        let dead = game.state.count(team, Lion::Dead);
        let mut parts = Vec::new();
        if reserve > 0 {
            parts.push(format!("{reserve} ready"));
        }
        if hunting > 0 {
            parts.push(format!("{hunting} hunting"));
        }
        if dead > 0 {
            parts.push(format!("{dead} dead"));
        }
        put(buf, lx + 1, y, &parts.join(", "), Style::new().fg(DIM).bg(BG));
        y += 2;
    }
    for line in wrap(&game.message, panel.width as usize) {
        put(buf, x, y, &line, Style::new().fg(TEXT).bg(BG));
        y += 1;
    }
    if over && let Phase::Over { winner } = game.phase {
        let result = format!("{} WIN", winner.title().to_uppercase());
        put(buf, x, y + 1, &result, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    }
    let keys: &[&str] = if over {
        &["Enter: play again", "M: menu   F1: help   Q: quit"]
    } else if game.human_acting() {
        match game.phase {
            Phase::Unleash { .. } => &["Y or Enter: yes   N: no", "Mouse: click", "F1: help   Q: quit"],
            Phase::Choose { .. } | Phase::Pick { .. } => {
                &["Arrows: choose   Enter: go", "Mouse: click a pharaoh", "F1: help   Q: quit"]
            }
            Phase::FightHide { .. } | Phase::FightGuess { .. } => {
                &["1 to 3, or arrows and Enter", "Mouse: click a number", "F1: help   Q: quit"]
            }
            Phase::LookAway { .. } => &["Enter: ready", "F1: help   Q: quit"],
            _ => &["1 to 5, or arrows and Enter", "Mouse: click a number", "F1: help   Q: quit"],
        }
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
        title: "MEHEN",
        subtitle: "The game of the coiled snake. Egypt, from about 3000 BC. Its rules are lost.",
        rules: rules.iter().map(|r| r.name).collect(),
        source: rules[chosen].source,
        sides: ["the lions", "the lionesses"],
    }
}

/// The short form of the game's rulebook chapter (its "brief" blocks), as
/// plain text: the F1 help. The full chapter is RULES.txt in the release
/// archives. Both are written by `rulebook/build.sh`.
pub const HELP: &str = include_str!("../HELP.txt");

/// How to play: the rules in short, and the line where the rule set
/// `rules` begins (every Mehen rule set is "Masters 2024").
pub fn help(_rules: &Rules) -> (Vec<Help>, usize) {
    let lines = shell::rules_help(HELP);
    let at = shell::help_find(&lines, "Masters 2024").unwrap_or(0);
    (lines, at)
}

#[cfg(test)]
mod help_tests {
    use super::*;

    #[test]
    fn the_help_opens_at_every_rule_set() {
        let lines = shell::rules_help(HELP);
        for _rules in Rules::all() {
            assert!(shell::help_find(&lines, "Masters 2024").is_some(), "Masters 2024");
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
