//! The game around the rules: lifting the first stones and jumping with
//! the cursor, the computer's moves (thought in the background), and the
//! end. Every jump takes a stone, so a game cannot go round in circles.

use crate::ai;
use crate::rules::{Move, Position, Rules, Side};
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
    Over { winner: Side },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays Black, and who White.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    pub cursor: usize,
    /// The stone picked up by the person to move.
    pub selected: Option<usize>,
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let cursor = (rules.rows / 2) * rules.cols + rules.cols / 2;
        Game {
            cursor,
            position,
            rules,
            players,
            phase: Phase::Turn,
            selected: None,
            last: None,
            message: "Black begins by lifting one of his stones from the board.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The hollows lit for the person: the stones that may be lifted, or
    /// that may jump, or where the stone picked up may land.
    pub fn lit(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        let moves = self.position.moves(&self.rules);
        let mut lit: Vec<usize> = moves
            .iter()
            .filter_map(|m| match (m, self.selected) {
                (Move::Lift(p), _) => Some(*p),
                (Move::Jump { from, path }, Some(s)) if *from == s => path.last().copied(),
                (Move::Jump { from, .. }, None) => Some(*from),
                _ => None,
            })
            .collect();
        lit.sort_unstable();
        lit.dedup();
        lit
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        if let Some(p) = self.rules.step(self.cursor, (dr, dc), 1) {
            self.cursor = p;
        }
    }

    /// Lifts the stone at `p`, picks it up, or puts the picked one down.
    pub fn pick(&mut self, p: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        let moves = self.position.moves(&self.rules);
        if self.position.opening() {
            match moves.into_iter().find(|m| *m == Move::Lift(p)) {
                Some(mv) => self.play(mv),
                None => self.message = "That stone may not be lifted; the lit ones may.".to_string(),
            }
            return;
        }
        if let Some(s) = self.selected
            && let Some(mv) =
                moves.iter().find(|m| matches!(m, Move::Jump { from, path } if *from == s && path.last() == Some(&p)))
        {
            self.play(mv.clone());
            return;
        }
        if self.position.at[p] == Some(self.position.to_move) {
            self.selected = Some(p);
            self.message = if self.lit().is_empty() {
                "That stone cannot jump.".to_string()
            } else {
                "Now choose where it lands.".to_string()
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
        self.position.play(&self.rules, &mv);
        self.selected = None;
        self.message = match &mv {
            Move::Lift(p) => format!("{} lifts the stone on {}.", side.name(), self.rules.name(*p)),
            Move::Jump { from, path } => {
                let taken = path.len();
                let to = self.rules.name(*path.last().unwrap());
                let what = if taken == 1 { "a stone".to_string() } else { format!("{taken} stones") };
                format!("{} jumps {} to {to}, taking {what}.", side.name(), self.rules.name(*from))
            }
        };
        self.last = Some(mv);
        if let Some(winner) = self.position.winner(&self.rules) {
            self.message.push_str(&format!(" {} cannot jump: {} wins!", winner.other().name(), winner.name()));
            self.phase = Phase::Over { winner };
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
                    let handle = std::thread::spawn(move || ai::choose(&rules, &position, strength, &mut rng));
                    self.phase = Phase::Thinking { handle, until: now + SHOW };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_then_jump() {
        let rules = Rules::ernst();
        let mut game = Game::new(rules.clone(), [Player::Human, Player::Human], Rng::new(1));
        let at = |n: &str| (0..rules.len()).find(|&p| rules.name(p) == n).unwrap();
        game.pick(at("d4"));
        game.pick(at("e4"));
        assert!(!game.position.opening());
        game.pick(at("d2"));
        assert_eq!(game.lit(), [at("d4")]);
        game.pick(at("d4"));
        assert_eq!(game.position.at[at("d4")], Some(Side::Black));
        assert_eq!(game.position.at[at("d3")], None);
    }
}
