//! Senet, part of Ancient Games: the Egyptian game of passing, in the
//! terminal, with every honest reconstruction of its lost rules.
//!
//! Run: `cargo run --release -p senet [-- --seed N] [--demo]
//! [--quit-after SECONDS]` (`RUST_LOG=debug ... 2>senet.log` for a session
//! log).

mod ai;
mod app;
mod rules;
mod ui;

use app::{Game, Phase, Player, Rng, Setup, Strength};
use log::{debug, info};
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use rules::{Rules, Side};
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
        self.game = Some(Game::new(self.rules[self.setup.rules].clone(), players, rng));
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
                        KeyCode::Enter | KeyCode::Char(' ') => game.confirm(),
                        KeyCode::Char('m') | KeyCode::Char('M') => {
                            self.screen = Screen::Menu { row: 3 }
                        }
                        _ => {}
                    },
                    _ => match code {
                        KeyCode::Enter | KeyCode::Char(' ') if game.human_to_move() => {
                            game.throw(now)
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') if game.human_to_move() => {
                            game.rescue()
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
                let house = self
                    .hits
                    .squares
                    .iter()
                    .find(|(r, _)| r.contains((x, y).into()))
                    .map(|&(_, h)| h);
                if let Some(house) = house
                    && game.select_from(house)
                {
                    game.confirm();
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
        let area = frame.area();
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
    let seed = arg("--seed").and_then(|s| s.parse().ok()).unwrap_or_else(|| {
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
        setup: Setup { rules: 0, opponent: 1, side: Side::White },
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
