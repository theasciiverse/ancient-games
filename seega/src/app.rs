//! The game around the rules: placing kelbs and moving them with the
//! cursor, giving one up to open a way, the computer's moves (thought in
//! the background), and the end.

use crate::ai;
use crate::rules::{Move, Outcome, Position, Rules, Side, Stage};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How the game stands.
pub enum Phase {
    /// The side to move is choosing (a person) or about to think.
    Turn,
    /// The computer is thinking; its move is shown no sooner than `until`.
    Thinking { handle: JoinHandle<Option<Move>>, until: Instant },
    Over { outcome: Outcome },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays Stone, and who Brick.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    pub cursor: usize,
    /// The kelb picked up by the person to move.
    pub selected: Option<usize>,
    pub last: Option<Move>,
    /// The holes emptied by the last move: kelbs taken or given.
    pub emptied: Vec<usize>,
    pub message: String,
    pub rng: Rng,
}

/// How long a computer's move stays in view at least; placing is quicker.
const SHOW: Duration = Duration::from_millis(450);
const SHOW_PLACING: Duration = Duration::from_millis(150);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let cursor = rules.centre();
        Game {
            cursor,
            position,
            rules,
            players,
            phase: Phase::Turn,
            selected: None,
            last: None,
            emptied: Vec::new(),
            message: "Each sets down two kelbs a turn, anywhere but the centre. Stone begins.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The holes lit for the person: where a kelb may be set down, the
    /// kelbs that may move or be given, or where the one picked up may go.
    pub fn lit(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        let moves = self.position.moves(&self.rules);
        let mut lit: Vec<usize> = moves
            .iter()
            .filter_map(|m| match (*m, self.selected) {
                (Move::Place(p), _) | (Move::Give(p), _) => Some(p),
                (Move::Step { from, to }, Some(s)) if from == s => Some(to),
                (Move::Step { from, .. }, None) => Some(from),
                _ => None,
            })
            .collect();
        lit.sort_unstable();
        lit.dedup();
        lit
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        if let Some(p) = self.rules.step(self.cursor, (dr, dc)) {
            self.cursor = p;
        }
    }

    /// Sets a kelb down at `p`, gives it up, picks it up, or moves the
    /// picked one there.
    pub fn pick(&mut self, p: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        let moves = self.position.moves(&self.rules);
        match self.position.stage {
            Stage::Placing | Stage::Opening => {
                match moves.into_iter().find(|m| matches!(*m, Move::Place(q) | Move::Give(q) if q == p)) {
                    Some(mv) => self.play(mv),
                    None if self.position.stage == Stage::Placing => {
                        self.message = "Not there: any empty hole but the centre.".to_string()
                    }
                    None => self.message = "Give up one of the lit kelbs, next to his.".to_string(),
                }
                return;
            }
            Stage::Moving => {}
        }
        if let Some(s) = self.selected
            && let Some(mv) = moves.iter().find(|m| **m == Move::Step { from: s, to: p })
        {
            self.play(*mv);
            return;
        }
        if self.position.at[p] == Some(self.position.to_move) {
            self.selected = Some(p);
            self.message = if self.lit().is_empty() {
                if self.position.chain {
                    "That kelb cannot take again.".to_string()
                } else {
                    "That kelb cannot move.".to_string()
                }
            } else {
                "Now choose where it goes.".to_string()
            };
        } else {
            self.selected = None;
        }
    }

    pub fn cancel(&mut self) {
        self.selected = None;
    }

    fn play(&mut self, mv: Move) {
        let side = self.position.to_move;
        info!("{} plays {:?}", side.name(), mv);
        let r = &self.rules;
        self.emptied = match mv {
            Move::Step { from, to } => self.position.takes(r, from, to),
            Move::Give(p) => vec![p],
            Move::Place(_) => Vec::new(),
        };
        self.message = match mv {
            Move::Place(p) => format!("{} sets a kelb down on {}.", side.name(), r.name(p)),
            Move::Give(p) => {
                format!("{} has no move: {} gives up the kelb on {} to open a way.", side.other().name(), side.name(), r.name(p))
            }
            Move::Step { from, to } => {
                let taken: Vec<String> = self.emptied.iter().map(|&q| r.name(q)).collect();
                let what = if taken.is_empty() { String::new() } else { format!(", taking {}", taken.join(" and ")) };
                format!("{} moves {} to {}{what}.", side.name(), r.name(from), r.name(to))
            }
        };
        let was_placing = self.position.stage == Stage::Placing;
        self.position.play(&self.rules, &mv);
        self.selected = None;
        self.last = Some(mv);
        let pos = &self.position;
        if pos.chain {
            self.message.push_str(&format!(" {} takes again.", side.name()));
        } else if was_placing && pos.stage != Stage::Placing {
            self.message.push_str(&format!(" All are down: {} moves first, into the centre.", rules_first(pos)));
        }
        if pos.stage == Stage::Opening {
            self.message.push_str(&format!(
                " {} cannot move: {} must give up a kelb next to his.",
                pos.to_move.other().name(),
                pos.to_move.name()
            ));
        }
        if let Some(outcome) = pos.outcome() {
            let end = match outcome {
                Outcome::Win(w) if pos.count(w.other()) == 0 => format!(" {} has no kelbs left: {} wins!", w.other().name(), w.name()),
                Outcome::Win(w) => format!(" {} moves without a capture: {} has more kelbs and wins!", crate::rules::QUIET, w.name()),
                Outcome::Draw => format!(" {} moves without a capture, kelbs even: a draw.", crate::rules::QUIET),
            };
            self.message.push_str(&end);
            self.phase = Phase::Over { outcome };
        } else {
            self.phase = Phase::Turn;
        }
    }

    /// Whether the computer is still working out its move.
    pub fn thinking(&self) -> bool {
        matches!(&self.phase, Phase::Thinking { handle, .. } if !handle.is_finished())
    }

    pub fn tick(&mut self, now: Instant) {
        match &self.phase {
            Phase::Turn => {
                if let Player::Computer(strength) = self.player(self.position.to_move) {
                    let rules = self.rules.clone();
                    let position = self.position.clone();
                    let mut rng = Rng::new(self.rng.next());
                    let show = if position.stage == Stage::Placing { SHOW_PLACING } else { SHOW };
                    let handle = std::thread::spawn(move || ai::choose(&rules, &position, strength, &mut rng));
                    self.phase = Phase::Thinking { handle, until: now + show };
                }
            }
            Phase::Thinking { handle, until } => {
                if handle.is_finished() && now >= *until {
                    let Phase::Thinking { handle, .. } = std::mem::replace(&mut self.phase, Phase::Turn) else {
                        unreachable!()
                    };
                    if let Some(mv) = handle.join().ok().flatten() {
                        self.play(mv);
                    }
                }
            }
            Phase::Over { .. } => {}
        }
    }
}

/// Who makes the first move, as a name.
fn rules_first(pos: &Position) -> &'static str {
    match pos.stage {
        Stage::Opening => pos.to_move.other().name(),
        _ => pos.to_move.name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_then_move() {
        let rules = Rules::khamsawee();
        let mut game = Game::new(rules.clone(), [Player::Human, Player::Human], Rng::new(1));
        // The centre is refused; then fill the board in reading order.
        game.pick(rules.centre());
        assert_eq!(game.position.count(Side::Stone), 2);
        while game.position.stage == Stage::Placing {
            let p = game.lit()[0];
            game.pick(p);
        }
        assert_eq!(game.position.count(Side::Stone), 12);
        // Brick is to move, or Stone to give him a way.
        let lit = game.lit();
        assert!(!lit.is_empty());
        if game.position.stage == Stage::Moving {
            game.pick(lit[0]);
            assert_eq!(game.lit(), [rules.centre()]);
            game.pick(rules.centre());
            assert!(game.position.at[rules.centre()].is_some());
        }
    }
}
