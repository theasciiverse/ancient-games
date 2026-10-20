//! The game around the rules: whose turn it is, throwing, choosing a move,
//! the computer's turns, and the end.

use crate::ai;
use crate::rules::{Action, OFF, Outcome, Position, Rules, Side, Variant};
pub use ancient_core::player::{Player, Strength};
pub use ancient_core::rng::Rng;
use log::{debug, info};
use std::time::{Duration, Instant};

/// The choices on the menu.
#[derive(Clone, Debug)]
pub struct Setup {
    pub rules: usize,
    /// 0 to 2: computer easy, normal, hard; 3: two players.
    pub opponent: usize,
    /// The side the (first) human plays.
    pub side: Side,
}

impl Setup {
    pub const OPPONENTS: [&'static str; 4] = [
        "the computer, easy",
        "the computer, normal",
        "the computer, hard",
        "a friend at this keyboard",
    ];

    pub fn players(&self) -> [Player; 2] {
        let computer = match self.opponent {
            0 => Some(Strength::Easy),
            1 => Some(Strength::Normal),
            2 => Some(Strength::Hard),
            _ => None,
        };
        let them = computer.map_or(Player::Human, Player::Computer);
        match self.side {
            Side::White => [Player::Human, them],
            Side::Black => [them, Player::Human],
        }
    }
}

/// What is happening in a game.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Waiting for the side to move to throw.
    ToThrow,
    /// The sticks are tumbling; they land at `until`.
    Throwing { until: Instant },
    /// The side picks one of `moves` (`selected`).
    Choosing { throw: u8, moves: Vec<Action>, selected: usize },
    /// A throw with no move, shown until `until`.
    NoMove { until: Instant },
    /// The computer is thinking about its move.
    Thinking { throw: u8, until: Instant },
    /// The game is over: `None` is a draw (Bell, equal men off).
    Over { winner: Option<Side> },
}

/// A game in progress.
pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// The four sticks as they lie, light side up or not; or the long die's
    /// face (Bell).
    pub sticks: [bool; 4],
    pub die: u8,
    /// The throw being played.
    pub shown: u8,
    pub message: String,
    /// The last move, to show where it went.
    pub last: Option<(Side, Action)>,
    pub rng: Rng,
}

const TUMBLE: Duration = Duration::from_millis(650);
const PAUSE: Duration = Duration::from_millis(900);
const THINK: Duration = Duration::from_millis(550);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng) -> Game {
        // Bell: the higher throw moves first; a coin stands in for it.
        let first = if rng.coin() { Side::White } else { Side::Black };
        let position = Position::new(&rules, first);
        info!("new game: {} {:?}, {} starts", rules.id, players, position.to_move.name());
        let message = format!("{} throws first.", position.to_move.name());
        Game {
            position,
            rules,
            players,
            phase: Phase::ToThrow,
            sticks: [false; 4],
            die: 1,
            shown: 0,
            message,
            last: None,
            rng,
        }
    }

    pub fn to_move(&self) -> Side {
        self.position.to_move
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        self.player(self.to_move()) == Player::Human
    }

    /// Whether a person may rescue a piece from the water now.
    pub fn can_rescue(&self) -> bool {
        self.phase == Phase::ToThrow && self.position.can_rescue(&self.rules)
    }

    pub fn throw(&mut self, now: Instant) {
        if self.phase == Phase::ToThrow {
            debug!("{} throws", self.to_move().name());
            self.phase = Phase::Throwing { until: now + TUMBLE };
        }
    }

    /// Kendall, seven pieces: the drowned piece to the House of Rebirth
    /// instead of throwing.
    pub fn rescue(&mut self) {
        if self.can_rescue() {
            let side = self.to_move();
            let out = self.position.play(&self.rules, Action::Rescue, 0);
            info!("{} rescues a piece", side.name());
            self.last = Some((side, Action::Rescue));
            self.message = format!(
                "{} takes the piece out of the water to the House of Rebirth.",
                side.name()
            );
            self.finish(out, side);
        }
    }

    pub fn select(&mut self, step: isize) {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase {
            let n = moves.len() as isize;
            *selected = ((*selected as isize + step).rem_euclid(n)) as usize;
        }
    }

    /// Picks the move from house `from`, if legal.
    pub fn select_from(&mut self, from: u8) -> bool {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase
            && let Some(i) = moves
                .iter()
                .position(|a| matches!(a, Action::Move { from: f, .. } if *f == from))
        {
            *selected = i;
            return true;
        }
        false
    }

    pub fn confirm(&mut self) {
        if let Phase::Choosing { throw, moves, selected } = &self.phase {
            let (t, action) = (*throw, moves[*selected]);
            self.play(action, t);
        }
    }

    fn play(&mut self, action: Action, t: u8) {
        let side = self.to_move();
        let out = self.position.play(&self.rules, action, t);
        info!("{} plays {action:?} with {t}: {out:?}", side.name());
        self.last = Some((side, action));
        self.message = describe(side, action, out);
        self.finish(out, side);
    }

    fn finish(&mut self, out: Outcome, _side: Side) {
        if out.over {
            let winner = match self.rules.variant {
                Variant::Bell => self.position.bell_winner(),
                _ => self.position.winner(&self.rules),
            };
            self.message.push_str(&match winner {
                Some(w) => format!(" {} wins!", w.name()),
                None => " The board is empty, and it is a draw.".to_string(),
            });
            self.phase = Phase::Over { winner };
        } else {
            self.phase = Phase::ToThrow;
        }
    }

    /// The throw shown on the sticks or the die.
    fn read_throw(&self) -> u8 {
        if self.rules.variant == Variant::Bell {
            self.die
        } else {
            match self.sticks.iter().filter(|&&s| s).count() {
                0 => 5,
                n => n as u8,
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match self.phase.clone() {
            Phase::ToThrow => {
                if let Player::Computer(_) = self.player(self.to_move()) {
                    if self.position.can_rescue(&self.rules) && ai::wants_rescue(&self.position) {
                        self.rescue();
                    } else {
                        self.throw(now);
                    }
                }
            }
            Phase::Throwing { until } => {
                for s in &mut self.sticks {
                    *s = self.rng.coin();
                }
                self.die = 1 + (self.rng.next() % 4) as u8;
                if now < until {
                    return;
                }
                let t = self.read_throw();
                self.shown = t;
                let side = self.to_move();
                let moves = self.position.moves(&self.rules, t);
                debug!("{} threw {t}, {} moves", side.name(), moves.len());
                let who = side.name();
                if moves.is_empty() {
                    self.message = format!("{who} threw {t} but cannot move.");
                    self.phase = Phase::NoMove { until: now + PAUSE };
                } else if self.human_to_move() {
                    self.message = format!(
                        "{who} threw {t}. Choose a piece{}.",
                        if moves.len() > 1 { " (arrows, Enter)" } else { " (Enter)" }
                    );
                    self.phase = Phase::Choosing { throw: t, moves, selected: 0 };
                } else {
                    self.message = format!("{who} threw {t}.");
                    self.phase = Phase::Thinking { throw: t, until: now + THINK };
                }
            }
            Phase::NoMove { until } => {
                if now >= until {
                    self.position.pass();
                    self.phase = Phase::ToThrow;
                    self.message = format!("{} to throw.", self.to_move().name());
                }
            }
            Phase::Thinking { throw, until } => {
                if now >= until {
                    let Player::Computer(strength) = self.player(self.to_move()) else {
                        return;
                    };
                    match ai::choose(&self.rules, &self.position, throw, strength) {
                        Some(action) => self.play(action, throw),
                        None => {
                            self.position.pass();
                            self.phase = Phase::ToThrow;
                        }
                    }
                }
            }
            Phase::Choosing { .. } | Phase::Over { .. } => {}
        }
    }
}

fn describe(side: Side, action: Action, out: Outcome) -> String {
    let who = side.name();
    let mut text = match action {
        Action::Rescue => format!("{who} rescues a piece."),
        Action::Move { to: OFF, .. } => format!("{who} bears a piece off."),
        Action::Move { from, to } if to < from => format!("{who} must go back, from {from} to {to}."),
        Action::Move { from, to } => format!("{who} moves from {from} to {to}."),
    };
    if out.bumped {
        text.push_str(" An enemy piece is bumped back!");
    }
    if out.captured {
        text.push_str(" An enemy man is captured!");
    }
    if out.water {
        text.push_str(" Into the water!");
    }
    if out.again {
        text.push_str(" Throw again.");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computer_games_run_to_the_end() {
        for rules in Rules::all() {
            for seed in 1..=3u64 {
                let players = [Player::Computer(Strength::Normal), Player::Computer(Strength::Easy)];
                let mut game = Game::new(rules.clone(), players, Rng::new(seed));
                let mut now = Instant::now();
                let mut steps = 0;
                while !matches!(game.phase, Phase::Over { .. }) {
                    now += Duration::from_millis(100);
                    game.tick(now);
                    steps += 1;
                    assert!(steps < 400_000, "{} seed {seed}: no end", rules.id);
                }
            }
        }
    }
}
