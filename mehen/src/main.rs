//! Mehen, part of Ancient Games: the Egyptian game of the coiled snake, in
//! James F. R. Masters' reconstruction, for two, four or six players.
//!
//! Run: `cargo run --release -p mehen [-- --seed N] [--demo]
//! [--quit-after SECONDS]` (`RUST_LOG=debug ... 2>mehen.log` for a session
//! log).

mod ai;
mod app;
mod rules;
mod ui;

use ancient_core::player::{Player, Strength};
use ancient_core::rng::Rng;
use ancient_core::shell::{self, Program, Setup};
use app::{Game, Phase};
use log::{debug, info};
use ratatui::crossterm::event::KeyCode;
use rules::Rules;
use std::io;
use std::time::{Duration, Instant};

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
            [Player::Computer(Strength::Normal), Player::Computer(Strength::Hard)]
        } else {
            self.setup.players()
        };
        let rng = Rng::new(self.rng.next());
        self.game = Some(Game::new(self.rules[self.setup.rules].clone(), players, rng, Instant::now()));
        self.screen = Screen::Playing;
    }
}

impl Program for App {
    fn key(&mut self, code: KeyCode, now: Instant) {
        debug!("key {code:?}");
        if self.help {
            match code {
                KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') => self.help = false,
                KeyCode::Char('q') | KeyCode::Char('Q') => self.quit = true,
                _ => {}
            }
            return;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return self.quit = true,
            KeyCode::F(1) | KeyCode::Char('?') => return self.help = true,
            _ => {}
        }
        match &mut self.screen {
            Screen::Menu { row } => {
                if shell::menu_key(&mut self.setup, row, self.rules.len(), code) {
                    self.start();
                }
            }
            Screen::Playing => {
                let Some(game) = self.game.as_mut() else { return };
                if matches!(game.phase, Phase::Over { .. }) {
                    match code {
                        KeyCode::Enter | KeyCode::Char(' ') => self.start(),
                        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => {
                            self.screen = Screen::Menu { row: 3 }
                        }
                        _ => {}
                    }
                    return;
                }
                match code {
                    KeyCode::Left | KeyCode::Up | KeyCode::BackTab => game.step(-1),
                    KeyCode::Right | KeyCode::Down | KeyCode::Tab => game.step(1),
                    KeyCode::Enter | KeyCode::Char(' ') => game.confirm(now),
                    KeyCode::Char(c @ '1'..='5') if game.human_acting() => {
                        game.number_chosen(c as u8 - b'0', now)
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') if game.human_acting() => game.answer(true, now),
                    KeyCode::Char('n') | KeyCode::Char('N') if game.human_acting() => game.answer(false, now),
                    KeyCode::Char('m') | KeyCode::Char('M') => self.screen = Screen::Menu { row: 3 },
                    _ => {}
                }
            }
        }
    }

    /// A left click: a pharaoh, a number or an answer.
    fn click(&mut self, x: u16, y: u16, now: Instant) {
        debug!("click {x},{y}");
        if self.help || !matches!(self.screen, Screen::Playing) {
            return;
        }
        let Some(game) = self.game.as_mut() else { return };
        if !game.human_acting() {
            return;
        }
        let at = |r: &ratatui::layout::Rect| r.contains((x, y).into());
        if let Some(&(_, n)) = self.hits.numbers.iter().find(|(r, _)| at(r)) {
            game.number_chosen(n, now);
        } else if let Some(&(_, yes)) = self.hits.answers.iter().find(|(r, _)| at(r)) {
            game.answer(yes, now);
        } else if let Some(&(_, p)) = self.hits.pharaohs.iter().find(|(r, _)| at(r))
            && game.select_pharaoh(p)
        {
            game.confirm(now);
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
            Screen::Menu { row } => {
                shell::draw_menu(buf, area, &ui::menu(&self.rules, self.setup.rules), &self.setup, *row)
            }
            Screen::Playing => {
                if let Some(game) = &self.game {
                    ui::draw_game(buf, area, game, &mut self.hits);
                }
            }
        }
        if self.help {
            shell::draw_help(buf, area, &ui::help(&self.rules[self.setup.rules]));
        }
    }

    fn quit(&self) -> bool {
        self.quit
    }
}

fn main() -> io::Result<()> {
    env_logger::init();
    let seed = shell::seed();
    let demo = shell::flag("--demo");
    info!("start: seed {seed}, demo {demo}");
    let mut app = App {
        rules: Rules::all(),
        // Seventeen geese, the person as the fox.
        setup: Setup { rules: 0, opponent: 1, side: 0 },
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
    let result = shell::run(&mut app);
    info!("quit");
    result
}
