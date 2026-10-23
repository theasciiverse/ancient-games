//! Seega, part of Ancient Games: the Egyptian game of placing and
//! taking, as Edward Lane saw it played.
//!
//! Run: `cargo run --release -p seega [-- --seed N] [--demo]
//! [--quit-after SECONDS]` (`RUST_LOG=debug ... 2>seega.log` for a session
//! log).

mod ai;
mod app;
mod examples;
mod rules;
mod ui;

use ancient_core::capture;
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
    help_lines: Vec<shell::Help>,
    help_scroll: usize,
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
        self.game = Some(Game::new(self.rules[self.setup.rules].clone(), players, rng));
        self.screen = Screen::Playing;
    }
}

impl Program for App {
    fn key(&mut self, code: KeyCode, _now: Instant) {
        debug!("key {code:?}");
        if self.help {
            match code {
                KeyCode::Esc | KeyCode::F(1) | KeyCode::Char('?') => self.help = false,
                KeyCode::Char('q') | KeyCode::Char('Q') => self.quit = true,
                _ => {
                    shell::help_key(code, &mut self.help_scroll);
                }
            }
            return;
        }
        match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return self.quit = true,
            KeyCode::F(1) | KeyCode::Char('?') => {
                // The rulebook chapter, opened at the rule set in play.
                (self.help_lines, self.help_scroll) = ui::help(&self.rules[self.setup.rules]);
                return self.help = true;
            }
            _ => {}
        }
        match &mut self.screen {
            Screen::Menu { row } => {
                if shell::menu_key(&mut self.setup, row, self.rules.len(), code) {
                    self.start();
                }
            }
            Screen::Playing => {
                let Some(game) = self.game.as_mut() else {
                    return;
                };
                if matches!(game.phase, Phase::Over { .. }) {
                    match code {
                        KeyCode::Enter | KeyCode::Char(' ') => self.start(),
                        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => self.screen = Screen::Menu { row: 3 },
                        _ => {}
                    }
                    return;
                }
                match code {
                    KeyCode::Up => game.move_cursor(-1, 0),
                    KeyCode::Down => game.move_cursor(1, 0),
                    KeyCode::Left => game.move_cursor(0, -1),
                    KeyCode::Right => game.move_cursor(0, 1),
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let at = game.cursor;
                        game.pick(at);
                    }
                    KeyCode::Esc => game.cancel(),
                    KeyCode::Char('m') | KeyCode::Char('M') => self.screen = Screen::Menu { row: 3 },
                    _ => {}
                }
            }
        }
    }

    /// A left click: place a kelb, give one up, pick one up, or move it.
    fn click(&mut self, x: u16, y: u16, _now: Instant) {
        debug!("click {x},{y}");
        if self.help || !matches!(self.screen, Screen::Playing) {
            return;
        }
        let Some(game) = self.game.as_mut() else {
            return;
        };
        let point = self.hits.points.iter().find(|(r, _)| r.contains((x, y).into())).map(|&(_, p)| p);
        if let Some(p) = point {
            game.pick(p);
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
            shell::draw_help_at(buf, area, &self.help_lines, &mut self.help_scroll);
        }
    }

    fn quit(&self) -> bool {
        self.quit
    }

    fn busy(&self) -> bool {
        self.game.as_ref().is_some_and(|g| g.thinking())
    }
}

fn main() -> io::Result<()> {
    env_logger::init();
    let seed = shell::seed();
    let demo = shell::flag("--demo");
    info!("start: seed {seed}, demo {demo}");
    let mut app = App {
        rules: Rules::all(),
        // The board of five, the person as Stone, who places first.
        setup: Setup { rules: 0, opponent: 1, side: 0 },
        screen: Screen::Menu { row: 3 },
        game: None,
        help: false,
        help_lines: Vec::new(),
        help_scroll: 0,
        demo,
        hits: ui::Hits::default(),
        rng: Rng::new(seed),
        quit: false,
        demo_restart: None,
    };
    // Pictures for the rulebook (rulebook/build.sh): the worked examples,
    // or a demo of one rule set drawn after some simulated seconds.
    if shell::flag("--rulebook-examples") {
        capture::print_examples(&examples::all());
        return Ok(());
    }
    if let Some((rules, seconds)) = capture::screen_arg() {
        app.setup.rules = rules.min(app.rules.len() - 1);
        app.demo = true;
        app.start();
        print!("{}", capture::svg(&capture::play_and_draw(&mut app, seconds)));
        return Ok(());
    }
    if demo {
        app.start();
    }
    let result = shell::run(&mut app);
    info!("quit");
    result
}
