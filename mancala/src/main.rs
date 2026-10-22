//! Mancala, part of Ancient Games: the sowing games of Africa, Asia and
//! Europe, each with its own rules, from Ghana's Oware and Lane's Cairo
//! mankal'ah to Toguz Kumalak of the steppe.
//!
//! Run: `cargo run --release -p mancala [-- --seed N] [--demo]
//! [--quit-after SECONDS]` (`RUST_LOG=debug ... 2>mancala.log` for a session
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
                let Some(game) = self.game.as_mut() else { return };
                match &game.phase {
                    Phase::Over { .. } => match code {
                        KeyCode::Enter | KeyCode::Char(' ') => self.start(),
                        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => self.screen = Screen::Menu { row: 3 },
                        _ => {}
                    },
                    Phase::RoundOver { .. } => match code {
                        KeyCode::Enter | KeyCode::Char(' ') => game.next_round(),
                        KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => self.screen = Screen::Menu { row: 3 },
                        _ => {}
                    },
                    Phase::Sowing { .. } => game.hurry(),
                    Phase::Arrange { by } if game.player(*by) == Player::Human => {
                        let side = game.position.owner(game.cursor);
                        let lower = ui::left_is_lower(game, side);
                        match code {
                            KeyCode::Left => game.step_cursor(!lower),
                            KeyCode::Right => game.step_cursor(lower),
                            KeyCode::Up => game.place(true),
                            KeyCode::Down => game.place(false),
                            KeyCode::Tab => game.cross_cursor(),
                            KeyCode::Char('r') | KeyCode::Char('R') => game.scatter(),
                            KeyCode::Enter | KeyCode::Char(' ') => game.arranged(),
                            KeyCode::Char('m') | KeyCode::Char('M') => self.screen = Screen::Menu { row: 3 },
                            _ => {}
                        }
                    }
                    Phase::TurnBoard { by } if game.player(*by) == Player::Human => match code {
                        KeyCode::Char('t') | KeyCode::Char('T') => game.turn_board(true),
                        KeyCode::Enter | KeyCode::Char(' ') => game.turn_board(false),
                        KeyCode::Char('m') | KeyCode::Char('M') => self.screen = Screen::Menu { row: 3 },
                        _ => {}
                    },
                    _ => {
                        let side = game.position.to_move;
                        let lower = ui::left_is_lower(game, side);
                        match code {
                            KeyCode::Left => game.step_cursor(!lower),
                            KeyCode::Right => game.step_cursor(lower),
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                let p = game.cursor;
                                game.choose(p);
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => self.screen = Screen::Menu { row: 3 },
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// A left click: sow a pit, finish a move being shown, or (setting out
    /// Lane's pebbles) put one more in a pit.
    fn click(&mut self, x: u16, y: u16, _now: Instant) {
        debug!("click {x},{y}");
        if self.help || !matches!(self.screen, Screen::Playing) {
            return;
        }
        let Some(game) = self.game.as_mut() else { return };
        if matches!(game.phase, Phase::Sowing { .. }) {
            return game.hurry();
        }
        let pit = self.hits.pits.iter().find(|(r, _)| r.contains((x, y).into())).map(|&(_, p)| p);
        let Some(p) = pit else { return };
        if let Phase::Arrange { by } = game.phase {
            if game.player(by) == Player::Human {
                game.cursor = p;
                game.place(true);
            }
            return;
        }
        game.choose(p);
    }

    fn tick(&mut self, now: Instant) {
        if self.help {
            return;
        }
        if let (Screen::Playing, Some(game)) = (&self.screen, self.game.as_mut()) {
            game.tick(now);
            if self.demo && matches!(game.phase, Phase::Over { .. }) {
                match self.demo_restart {
                    None => self.demo_restart = Some(now + Duration::from_secs(5)),
                    Some(at) if now >= at => {
                        self.demo_restart = None;
                        self.setup.rules = (self.setup.rules + 1) % self.rules.len();
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
        // Oware, the person as South against the computer.
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
        // The demo goes through the rule sets in turn, from a random one.
        app.setup.rules = (app.rng.next() % app.rules.len() as u64) as usize;
        app.start();
    }
    let result = shell::run(&mut app);
    info!("quit");
    result
}
