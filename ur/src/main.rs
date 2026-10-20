//! The Royal Game of Ur, part of Ancient Games: the oldest board game with
//! rules that survive, in the terminal, with every honest rule set.
//!
//! Run: `cargo run --release -p royal-ur [-- --seed N] [--demo]
//! [--quit-after SECONDS]` (`RUST_LOG=debug ... 2>ur.log` for a session log).

mod ai;
mod app;
mod rules;
mod ui;

use crate::ai::Strength;
use app::{Game, Phase, Player, Rng, Setup};
use log::{debug, info};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::Rect;
use rules::{Rules, START, Side};
use std::io;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FRAME: Duration = Duration::from_millis(33);

fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

enum Screen {
    Menu { row: usize },
    Playing,
}

struct App {
    rules: Vec<Rules>,
    setup: Setup,
    screen: Screen,
    game: Option<Game>,
    help: bool,
    demo: bool,
    hits: ui::Hits,
    rng: Rng,
    quit: bool,
    /// When a finished demo game starts over.
    demo_restart: Option<Instant>,
}

impl App {
    fn start(&mut self) {
        let players = if self.demo {
            [Player::Computer(Strength::Hard), Player::Computer(Strength::Normal)]
        } else {
            self.setup.players()
        };
        let rng = Rng::new(self.rng.next());
        // Purses carry on from game to game under the same betting rules.
        let rules = self.rules[self.setup.rules].clone();
        let fresh = [rules.stakes.map_or(0, |s| s.purse); 2];
        let purses = match &self.game {
            Some(g) if g.rules.id == rules.id => g.purses,
            _ => fresh,
        };
        self.game = Some(Game::new(rules, players, rng, purses));
        self.screen = Screen::Playing;
    }

    fn key(&mut self, code: KeyCode, now: Instant) {
        debug!("key {code:?}");
        if self.help {
            if matches!(code, KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?')) {
                self.help = false;
            } else if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                self.quit = true;
            }
            return;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.quit = true;
                return;
            }
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.help = true;
                return;
            }
            _ => {}
        }
        match &mut self.screen {
            Screen::Menu { row } => match code {
                KeyCode::Up => *row = (*row + 3) % 4,
                KeyCode::Down | KeyCode::Tab => *row = (*row + 1) % 4,
                KeyCode::Left | KeyCode::Right => {
                    let step: isize = if code == KeyCode::Left { -1 } else { 1 };
                    match *row {
                        0 => {
                            let n = self.rules.len() as isize;
                            self.setup.rules =
                                (self.setup.rules as isize + step).rem_euclid(n) as usize;
                        }
                        1 => {
                            self.setup.opponent =
                                (self.setup.opponent as isize + step).rem_euclid(4) as usize;
                        }
                        2 => self.setup.side = self.setup.side.other(),
                        _ => {}
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => self.start(),
                _ => {}
            },
            Screen::Playing => {
                let Some(game) = self.game.as_mut() else { return };
                match &game.phase {
                    Phase::Over { .. } => match code {
                        KeyCode::Enter | KeyCode::Char(' ') => self.start(),
                        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => {
                            self.screen = Screen::Menu { row: 3 };
                        }
                        _ => {}
                    },
                    Phase::Choosing { .. } => match code {
                        KeyCode::Left | KeyCode::Up | KeyCode::BackTab => game.select(-1),
                        KeyCode::Right | KeyCode::Down | KeyCode::Tab => game.select(1),
                        KeyCode::Enter | KeyCode::Char(' ') => game.confirm(now),
                        KeyCode::Char('d') | KeyCode::Char('D') => game.dare(now),
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            self.screen = Screen::Menu { row: 3 }
                        }
                        _ => {}
                    },
                    _ => match code {
                        KeyCode::Enter | KeyCode::Char(' ') if game.human_to_move() => {
                            game.throw(now)
                        }
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            self.screen = Screen::Menu { row: 3 }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    /// A left click: throw, or play the lit piece clicked.
    fn click(&mut self, x: u16, y: u16, now: Instant) {
        debug!("click {x},{y}");
        if self.help || !matches!(self.screen, Screen::Playing) {
            return;
        }
        let Some(game) = self.game.as_mut() else { return };
        match &game.phase {
            Phase::ToThrow if game.human_to_move() => game.throw(now),
            Phase::Choosing { .. } => {
                let side = game.to_move();
                let own = if side == Side::Light { 2 } else { 0 };
                let hit = self
                    .hits
                    .squares
                    .iter()
                    .find(|(r, _, _)| r.contains((x, y).into()))
                    .map(|&(_, row, col)| (row, col));
                let Some((row, col)) = hit else { return };
                let from = if row == own && col == 4 {
                    Some(START)
                } else {
                    game.rules.board.pos_at(side, row, col)
                };
                if let Some(from) = from
                    && game.select_from(from)
                {
                    game.confirm(now);
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, now: Instant) {
        if self.help {
            return;
        }
        if let (Screen::Playing, Some(game)) = (&self.screen, self.game.as_mut()) {
            game.tick(now);
            if self.demo && matches!(game.phase, Phase::Over { .. }) {
                match self.demo_restart {
                    None => self.demo_restart = Some(now + Duration::from_secs(4)),
                    Some(at) if now >= at => {
                        self.demo_restart = None;
                        self.start();
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let area: Rect = frame.area();
        let buf = frame.buffer_mut();
        match &self.screen {
            Screen::Menu { row } => ui::draw_menu(buf, area, &self.setup, *row, &self.rules),
            Screen::Playing => {
                if let Some(game) = &self.game {
                    ui::draw_game(buf, area, game, &mut self.hits);
                }
            }
        }
        if self.help {
            ui::draw_help(buf, area, &self.rules[self.setup.rules]);
        }
    }
}

fn main() -> io::Result<()> {
    env_logger::init();

    let seed = arg("--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1)
        });
    let quit_after = arg("--quit-after")
        .and_then(|s| s.parse::<f64>().ok())
        .map(Duration::from_secs_f64);
    let demo = flag("--demo");
    info!("start: seed {seed}, demo {demo}");

    let mut app = App {
        rules: Rules::all(),
        setup: Setup {
            rules: 0,
            opponent: 1,
            side: Side::Light,
        },
        screen: Screen::Menu { row: 3 },
        game: None,
        help: false,
        demo,
        hits: ui::Hits::default(),
        rng: Rng::new(seed),
        quit: false,
        demo_restart: None,
    };
    if demo {
        app.start();
    }

    let mut terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let started = Instant::now();
    let result = (|| -> io::Result<()> {
        while !app.quit {
            let now = Instant::now();
            if quit_after.is_some_and(|q| now - started >= q) {
                break;
            }
            app.tick(now);
            terminal.draw(|frame| app.draw(frame))?;
            if event::poll(FRAME)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(event::KeyModifiers::CONTROL)
                        {
                            app.quit = true;
                        } else {
                            app.key(key.code, Instant::now());
                        }
                    }
                    Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                        app.click(mouse.column, mouse.row, Instant::now());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    info!("quit");
    result
}
