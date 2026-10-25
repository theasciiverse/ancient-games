//! The game around the rules: who begins (each casts, the higher
//! begins), casting the beans, a stone moved by the whole throw, another
//! throw on a rounded end, forfeits, and the end.

use crate::ai;
use crate::rules::{HAND, Move, OFF, Outcome, Position, Rules, Side, value};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    ToThrow,
    Throwing { until: Instant },
    /// The person chooses which stone the throw moves.
    Choosing,
    /// The computer's move, shown until `until`.
    Showing { mv: Move, until: Instant },
    /// No stone can move: shown until `until`.
    Stuck { until: Instant },
    Over { outcome: Outcome },
}

pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// The beans as they fell: holes up.
    pub beans: [bool; 5],
    pub throw: u8,
    pub cursor: (u8, u8),
    pub last: Option<(Side, Move)>,
    pub message: String,
    pub rng: Rng,
}

const FALL: Duration = Duration::from_millis(550);
const SHOW: Duration = Duration::from_millis(500);
const PAUSE: Duration = Duration::from_millis(1100);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng) -> Game {
        // Each casts the beans; the higher begins (Bell); a tie casts
        // again.
        let mut cast = || value((0..5).filter(|_| rng.next() % 2 == 1).count() as u8);
        let (mut r, mut b) = (cast(), cast());
        while r == b {
            (r, b) = (cast(), cast());
        }
        let first = if r > b { Side::Red } else { Side::Blue };
        info!("new game: {} {:?}, {} begins", rules.id, players, first.name());
        Game {
            position: Position::new(&rules, first),
            rules,
            players,
            phase: Phase::ToThrow,
            beans: [false; 5],
            throw: 0,
            cursor: (8, 7),
            last: None,
            message: format!("To begin, Red cast {r} and Blue {b}: {} begins.", first.name()),
            rng,
        }
    }

    pub fn player(&self) -> Player {
        self.players[self.position.to_move.index()]
    }

    pub fn human(&self) -> bool {
        self.player() == Player::Human
    }

    /// The cells lit for the person: where the throw can take a stone.
    pub fn lit(&self) -> Vec<(u8, u8)> {
        if !(self.human() && self.phase == Phase::Choosing) {
            return Vec::new();
        }
        let me = self.position.to_move;
        self.position
            .moves(self.throw)
            .iter()
            .filter(|m| m.to != OFF)
            .map(|m| crate::rules::cell(self.position.square(me, m.to, m.dir)))
            .collect()
    }

    /// Whether a stone can be borne off with this throw.
    pub fn can_bear_off(&self) -> bool {
        self.human() && self.phase == Phase::Choosing && self.position.moves(self.throw).iter().any(|m| m.to == OFF)
    }

    pub fn move_cursor(&mut self, dr: i8, dc: i8) {
        let (r, c) = (self.cursor.0 as i8 + dr, self.cursor.1 as i8 + dc);
        let on = |r: i8, c: i8| (0..16).contains(&r) && (0..16).contains(&c) && ((7..=8).contains(&r) || (7..=8).contains(&c));
        if on(r, c) {
            self.cursor = (r as u8, c as u8);
        }
    }

    /// Enter or a click at `at` (`None`: the "bear off" tray).
    pub fn act(&mut self, at: Option<(u8, u8)>, now: Instant) {
        if !self.human() {
            return;
        }
        match self.phase {
            Phase::ToThrow => self.cast(now),
            Phase::Choosing => {
                let me = self.position.to_move;
                let moves = self.position.moves(self.throw);
                let pick = match at {
                    None => moves.iter().find(|m| m.to == OFF).copied(),
                    Some(at) => {
                        self.cursor = at;
                        // The compartment a stone goes to, or the stone itself.
                        moves
                            .iter()
                            .find(|m| m.to != OFF && crate::rules::cell(self.position.square(me, m.to, m.dir)) == at)
                            .or_else(|| {
                                moves.iter().find(|m| {
                                    m.from != HAND
                                        && crate::rules::cell(self.position.square(me, m.from, m.dir)) == at
                                })
                            })
                            .copied()
                    }
                };
                match pick {
                    Some(mv) => self.play(mv, now),
                    None => self.message = format!("Choose a lit compartment for the {}.", self.throw),
                }
            }
            _ => {}
        }
    }

    fn cast(&mut self, now: Instant) {
        for b in &mut self.beans {
            *b = self.rng.next() % 2 == 1;
        }
        self.throw = value(self.beans.iter().filter(|&&b| b).count() as u8);
        self.phase = Phase::Throwing { until: now + FALL };
        self.message = format!("{} casts the beans... Macuilxochitl!", self.position.to_move.name());
    }

    fn landed(&mut self, now: Instant) {
        let me = self.position.to_move;
        let n = self.throw;
        let moves = self.position.moves(n);
        info!("{} throws {n}", me.name());
        if moves.is_empty() {
            self.message = if n == 0 {
                format!("{} casts no holes: nothing moves.", me.name())
            } else {
                format!("{} casts {n}, but no stone can move: a forfeit into the pool.", me.name())
            };
            self.position.stuck(n);
            self.phase = Phase::Stuck { until: now + PAUSE };
            return;
        }
        match self.player() {
            Player::Human => {
                self.message = format!("You cast {n}. Choose where a stone goes.");
                self.phase = Phase::Choosing;
            }
            Player::Computer(s) => {
                let mv = ai::choose(&self.position, n, s, &mut self.rng).expect("a move");
                self.message = format!("{} casts {n}.", me.name());
                self.phase = Phase::Showing { mv, until: now + SHOW };
            }
        }
    }

    fn play(&mut self, mv: Move, now: Instant) {
        let me = self.position.to_move;
        let purse = self.position.purse;
        let first = self.position.dir[me.index()] == 0;
        let again = self.position.play(&mv);
        self.last = Some((me, mv));
        let way = if mv.dir > 0 { "against the sun (↺)" } else { "with the sun (↻)" };
        self.message = match (mv.from, mv.to) {
            (HAND, _) if first => format!("{} brings in a stone and sends it {} on, {way}: all its stones will go that way.", me.name(), mv.to),
            (HAND, _) => format!("{} brings in a stone.", me.name()),
            (_, OFF) => format!("{} bears a stone off; {} pays a forfeit.", me.name(), me.other().name()),
            (f, t) => format!("{} moves a stone {} on.", me.name(), t - f),
        };
        if self.position.purse[me.index()] + 2 <= purse[me.index()] {
            self.message.push_str(" It lands on a pinched compartment: two forfeits paid.");
        }
        if let Some(outcome) = self.position.outcome() {
            let Outcome::Home(w) = outcome;
            self.message.push_str(&format!(" {} has all six stones off and takes the pool!", w.name()));
            self.phase = Phase::Over { outcome };
            return;
        }
        if again {
            self.message.push_str(" A rounded end: another throw.");
        } else {
            self.position.pass();
        }
        self.phase = Phase::ToThrow;
        let _ = now;
    }

    pub fn tick(&mut self, now: Instant) {
        match self.phase.clone() {
            Phase::ToThrow if !self.human() => self.cast(now),
            Phase::Throwing { until } if now >= until => self.landed(now),
            Phase::Stuck { until } if now >= until => {
                self.position.pass();
                self.phase = Phase::ToThrow;
            }
            Phase::Showing { mv, until } if now >= until => self.play(mv, now),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computers_play_to_the_end() {
        let mut game = Game::new(
            Rules::bell(),
            [Player::Computer(ai::Strength::Normal), Player::Computer(ai::Strength::Hard)],
            Rng::new(2),
        );
        let t0 = Instant::now();
        let mut i = 0u64;
        while !matches!(game.phase, Phase::Over { .. }) {
            game.tick(t0 + Duration::from_millis(600 * i));
            i += 1;
            assert!(i < 200_000);
        }
    }
}
