//! What every game's program shares: the options on the command line, the
//! menu before a game, the help over it, and the loop that runs the
//! terminal.

use crate::draw::{AMBER, BG, DIM, TEXT, centre, fill, put, wrap};
use crate::player::{Player, Strength};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The value after `name` on the command line.
pub fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

/// Whether `name` is on the command line.
pub fn flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

/// `--seed N`, or the clock.
pub fn seed() -> u64 {
    arg("--seed").and_then(|s| s.parse().ok()).unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    })
}

/// `--quit-after SECONDS`, for recordings.
pub fn quit_after() -> Option<Duration> {
    arg("--quit-after").and_then(|s| s.parse::<f64>().ok()).map(Duration::from_secs_f64)
}

/// The choices on the menu: the rule set, the opponent, and which of the
/// two sides the (first) person plays.
#[derive(Clone, Debug)]
pub struct Setup {
    pub rules: usize,
    /// 0 to 2: computer easy, normal, hard; 3: two players.
    pub opponent: usize,
    /// 0 or 1: the side the person plays, in the game's own order.
    pub side: usize,
}

impl Setup {
    pub const OPPONENTS: [&'static str; 4] = [
        "the computer, easy",
        "the computer, normal",
        "the computer, hard",
        "a friend at this keyboard",
    ];

    /// Who plays each side, in the game's order.
    pub fn players(&self) -> [Player; 2] {
        let computer = match self.opponent {
            0 => Some(Strength::Easy),
            1 => Some(Strength::Normal),
            2 => Some(Strength::Hard),
            _ => None,
        };
        let them = computer.map_or(Player::Human, Player::Computer);
        if self.side == 0 { [Player::Human, them] } else { [them, Player::Human] }
    }
}

/// What a game's menu shows.
pub struct Menu<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    /// The names of the rule sets, and where the chosen one comes from.
    pub rules: Vec<&'a str>,
    pub source: &'a str,
    /// The names of the two sides, in the game's order.
    pub sides: [&'a str; 2],
}

/// The menu's rows: rules, opponent, side, and play.
pub const MENU_ROWS: usize = 4;

/// Handles a key on the menu. Returns true when the game should start.
pub fn menu_key(setup: &mut Setup, row: &mut usize, rule_sets: usize, code: KeyCode) -> bool {
    match code {
        KeyCode::Up => *row = (*row + MENU_ROWS - 1) % MENU_ROWS,
        KeyCode::Down | KeyCode::Tab => *row = (*row + 1) % MENU_ROWS,
        KeyCode::Left | KeyCode::Right => {
            let step: isize = if code == KeyCode::Left { -1 } else { 1 };
            match *row {
                0 => setup.rules = (setup.rules as isize + step).rem_euclid(rule_sets as isize) as usize,
                1 => setup.opponent = (setup.opponent as isize + step).rem_euclid(4) as usize,
                2 => setup.side = 1 - setup.side,
                _ => {}
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => return true,
        _ => {}
    }
    false
}

pub fn draw_menu(buf: &mut Buffer, area: Rect, menu: &Menu, setup: &Setup, row: usize) {
    fill(buf, area, BG);
    let w = area.width.min(76);
    let x = area.x + (area.width - w) / 2;
    let mut y = area.y + area.height.saturating_sub(22) / 2;
    let panel = Rect::new(x, y, w, 1);
    centre(buf, panel, y, menu.title, Style::new().fg(AMBER).bg(BG).add_modifier(Modifier::BOLD));
    y += 1;
    centre(buf, panel, y, menu.subtitle, Style::new().fg(DIM).bg(BG));
    y += 3;
    let items = [
        ("Rules", menu.rules[setup.rules].to_string()),
        ("Play against", Setup::OPPONENTS[setup.opponent].to_string()),
        ("You play", menu.sides[setup.side].to_string()),
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
    for line in wrap(menu.source, (w - 8) as usize) {
        put(buf, x + 6, y, &line, Style::new().fg(DIM).bg(BG));
        y += 1;
    }
    y += 2;
    let start = if row == MENU_ROWS - 1 {
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

/// A line of the help: a heading, text, quieter text, a line kept as it is
/// (not wrapped), or a blank.
pub enum Help {
    Heading(String),
    Text(String),
    Quiet(String),
    Pre(String),
    Blank,
}

/// The help from a game's rulebook chapter (its HELP.txt, written by
/// `rulebook/build.sh` already wrapped to fit the help): the first line
/// (the title) and every underlined line are headings, the underlines go,
/// and every other line is kept as it is.
pub fn rules_help(text: &str) -> Vec<Help> {
    let underline = |l: &str| l.len() >= 3 && (l.bytes().all(|b| b == b'=') || l.bytes().all(|b| b == b'-'));
    let lines: Vec<&str> = text.lines().collect();
    let mut help = Vec::new();
    for (i, &line) in lines.iter().enumerate() {
        if underline(line) && i > 0 {
            continue;
        }
        let heading = i == 0 || lines.get(i + 1).is_some_and(|next| underline(next));
        help.push(if line.trim().is_empty() {
            Help::Blank
        } else if heading {
            Help::Heading(line.to_string())
        } else {
            Help::Pre(line.to_string())
        });
    }
    help
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_help_finds_headings_and_drops_underlines() {
        let text = "Title\nAsciiverse\n\nSection\n-------\n\n  ---- ----\n  a    b\nRefs\n====\n";
        let help = rules_help(text);
        let headings: Vec<&str> = help
            .iter()
            .filter_map(|l| if let Help::Heading(h) = l { Some(h.as_str()) } else { None })
            .collect();
        assert_eq!(headings, ["Title", "Section", "Refs"]);
        // The table rule is indented, so it stays.
        assert!(help.iter().any(|l| matches!(l, Help::Pre(p) if p == "  ---- ----")));
        assert_eq!(help_find(&help, "section"), Some(3));
    }
}

/// The line of the heading `name` in `lines`, compared in capitals and with
/// typographic apostrophes made plain.
pub fn help_find(lines: &[Help], name: &str) -> Option<usize> {
    let plain = |s: &str| s.replace('\u{2019}', "'").to_uppercase();
    let name = plain(name);
    lines.iter().position(|l| matches!(l, Help::Heading(h) if plain(h) == name))
}

/// Scroll keys in the help. Returns whether the key was one. The draw
/// keeps `scroll` within the text.
pub fn help_key(code: KeyCode, scroll: &mut usize) -> bool {
    const PAGE: usize = 24;
    match code {
        KeyCode::Up | KeyCode::Char('k') => *scroll = scroll.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => *scroll += 1,
        KeyCode::PageUp => *scroll = scroll.saturating_sub(PAGE),
        KeyCode::PageDown | KeyCode::Char(' ') => *scroll += PAGE,
        KeyCode::Home => *scroll = 0,
        KeyCode::End => *scroll = usize::MAX,
        _ => return false,
    }
    true
}

/// The help, over the screen.
pub fn draw_help(buf: &mut Buffer, area: Rect, lines: &[Help]) {
    draw_help_at(buf, area, lines, &mut 0);
}

/// The help, over the screen, from row `scroll` of the text on.
pub fn draw_help_at(buf: &mut Buffer, area: Rect, lines: &[Help], scroll: &mut usize) {
    let back = Color::Rgb(18, 18, 26);
    let w = area.width.min(84);
    let h = area.height.min(34);
    let x = area.x + (area.width - w) / 2;
    let y = area.y + (area.height - h) / 2;
    fill(buf, Rect::new(x, y, w, h), back);

    let heading = Style::new().fg(AMBER).bg(back).add_modifier(Modifier::BOLD);
    let text = Style::new().fg(TEXT).bg(back);
    let quiet = Style::new().fg(DIM).bg(back);
    let mut rows: Vec<(String, Style)> = Vec::new();
    for line in lines {
        let (t, style) = match line {
            Help::Heading(t) => (t, heading),
            Help::Text(t) => (t, text),
            Help::Quiet(t) => (t, quiet),
            Help::Pre(t) => {
                // Kept as it is, but a line too wide (a long web address)
                // goes on over the next rows rather than being cut off.
                let chars: Vec<char> = t.chars().collect();
                for part in chars.chunks((w - 6).max(1) as usize) {
                    rows.push((part.iter().collect(), text));
                }
                if chars.is_empty() {
                    rows.push((String::new(), text));
                }
                continue;
            }
            Help::Blank => {
                rows.push((String::new(), text));
                continue;
            }
        };
        rows.extend(wrap(t, (w - 6) as usize).into_iter().map(|part| (part, style)));
    }

    let visible = h.saturating_sub(4) as usize;
    let last = rows.len().saturating_sub(visible);
    *scroll = (*scroll).min(last);
    for (i, (t, style)) in rows.iter().skip(*scroll).take(visible).enumerate() {
        put(buf, x + 3, y + 1 + i as u16, t, *style);
    }
    let foot = if last > 0 {
        let percent = *scroll * 100 / last;
        format!("Esc or F1: back   Up, Down, PgUp, PgDn: scroll   {percent}%")
    } else {
        "Esc or F1: back".to_string()
    };
    put(buf, x + 3, y + h - 2, &foot, quiet);
}

/// A game's program, as the loop drives it.
pub trait Program {
    /// Time passes.
    fn tick(&mut self, now: Instant);
    /// A key press (Ctrl+C is handled by the loop).
    fn key(&mut self, code: KeyCode, now: Instant);
    /// A left click.
    fn click(&mut self, x: u16, y: u16, now: Instant);
    fn draw(&mut self, frame: &mut ratatui::Frame);
    fn quit(&self) -> bool;
    /// Whether the computer is still thinking on another thread (the
    /// rulebook's pictures wait for it).
    fn busy(&self) -> bool {
        false
    }
}

/// Runs the terminal: raw mode, the mouse, a frame about every 33 ms,
/// `--quit-after`, and everything put back at the end.
pub fn run(program: &mut impl Program) -> io::Result<()> {
    const FRAME: Duration = Duration::from_millis(33);
    let quit_after = quit_after();
    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let started = Instant::now();
    let result = (|| -> io::Result<()> {
        while !program.quit() {
            let now = Instant::now();
            if quit_after.is_some_and(|q| now - started >= q) {
                break;
            }
            program.tick(now);
            terminal.draw(|frame| program.draw(frame))?;
            if event::poll(FRAME)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            break;
                        }
                        program.key(key.code, Instant::now());
                    }
                    Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                        program.click(mouse.column, mouse.row, Instant::now());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}
