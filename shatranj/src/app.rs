//! The game around the rules: moves made with the cursor or the mouse,
//! the computer's moves (thought in the background), the moves written
//! down, the end, and the draw by threefold repetition.

use crate::ai;
use crate::rules::{Kind, Move, Outcome, Position, Rules, Side, Why, name};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub enum Phase {
    /// The side to move is choosing (a person) or about to think.
    Turn,
    /// The computer is thinking; its move is shown no sooner than `until`.
    Thinking { handle: JoinHandle<Option<Move>>, until: Instant },
    Over { outcome: Outcome },
}

pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    pub seen: HashMap<Position, u32>,
    pub cursor: usize,
    pub selected: Option<usize>,
    pub last: Option<Move>,
    /// The moves so far, written as the panel shows them.
    pub record: Vec<String>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let mut seen = HashMap::new();
        seen.insert(position.clone(), 1);
        Game {
            cursor: 12,
            position,
            rules,
            players,
            phase: Phase::Turn,
            seen,
            selected: None,
            last: None,
            record: Vec::new(),
            message: "Red begins.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The squares lit for the person: where the man picked up may go.
    pub fn targets(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        match self.selected {
            Some(s) => self.position.moves().iter().filter(|m| m.from as usize == s).map(|m| m.to as usize).collect(),
            None => Vec::new(),
        }
    }

    pub fn move_cursor(&mut self, df: i8, dr: i8) {
        let (f, r) = crate::rules::fr(self.cursor);
        if let Some(s) = crate::rules::square(f + df, r + dr) {
            self.cursor = s;
        }
    }

    pub fn pick(&mut self, sq: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = sq;
        let moves = self.position.moves();
        if let Some(s) = self.selected
            && let Some(mv) = moves.iter().find(|m| m.from as usize == s && m.to as usize == sq)
        {
            self.play(*mv);
            return;
        }
        let me = self.position.to_move;
        if self.position.board[sq].is_some_and(|m| m.side == me) {
            self.selected = Some(sq);
            self.message = if moves.iter().any(|m| m.from as usize == sq) {
                "Now choose where it goes.".to_string()
            } else if self.position.in_check(me) {
                "Your shah is attacked: that man cannot save him.".to_string()
            } else {
                "That man cannot move.".to_string()
            };
        } else {
            self.selected = None;
        }
    }

    pub fn cancel(&mut self) {
        self.selected = None;
    }

    /// A move written as the panel shows it: the man's letter, from, to,
    /// x for a capture, =F for a baidaq that becomes a firzan, + check.
    pub fn notation(&self, mv: &Move) -> String {
        let p = &self.position;
        let man = p.board[mv.from as usize].expect("a man");
        let capture = p.board[mv.to as usize].is_some();
        let mut next = p.clone();
        next.play(&self.rules, mv);
        let check = if next.in_check(next.to_move) { "+" } else { "" };
        let letter = if man.kind == Kind::Baidaq { String::new() } else { man.kind.letter().to_string() };
        let promote = if mv.promotes { "=F" } else { "" };
        format!("{letter}{}{}{}{promote}{check}", name(mv.from as usize), if capture { "x" } else { "-" }, name(mv.to as usize))
    }

    fn play(&mut self, mv: Move) {
        let side = self.position.to_move;
        let man = self.position.board[mv.from as usize].expect("a man");
        let taken = self.position.board[mv.to as usize];
        info!("{} plays {:?}", side.name(), mv);
        self.record.push(self.notation(&mv));
        self.message = match taken {
            Some(t) => format!(
                "{}'s {} takes the {} on {}.",
                side.name(),
                man.kind.name(),
                t.kind.name(),
                name(mv.to as usize)
            ),
            None => format!("{}'s {} to {}.", side.name(), man.kind.name(), name(mv.to as usize)),
        };
        if mv.promotes {
            self.message.push_str(" It becomes a firzan.");
        }
        self.position.play(&self.rules, &mv);
        self.selected = None;
        self.last = Some(mv);
        let moves = self.position.moves();
        let count = self.seen.entry(self.position.clone()).or_insert(0);
        *count += 1;
        let repeated = *count >= 3;
        let outcome = self.position.outcome(&self.rules, &moves).or(repeated.then_some(Outcome::Draw(Why::Repetition)));
        match outcome {
            Some(outcome) => {
                let loser = self.position.to_move;
                let end = match outcome {
                    Outcome::Win(w, Why::Mate) => format!(" Shah mat: {} wins!", w.name()),
                    Outcome::Win(w, Why::Stalemate) => {
                        format!(" {} has no move, and a shah shut in loses: {} wins!", loser.name(), w.name())
                    }
                    Outcome::Win(w, Why::Bare) => {
                        format!(" {} has only his shah left: {} wins!", w.other().name(), w.name())
                    }
                    Outcome::Draw(Why::BothBare) => " Both shahs are bare: a draw.".to_string(),
                    Outcome::Draw(Why::Quiet) => {
                        format!(" {} moves each with nothing taken: a draw.", self.rules.quiet)
                    }
                    Outcome::Draw(Why::Repetition) => " The same position a third time: a draw.".to_string(),
                    Outcome::Draw(_) => " A draw.".to_string(),
                    Outcome::Win(w, _) => format!(" {} wins!", w.name()),
                };
                self.message.push_str(&end);
                self.phase = Phase::Over { outcome };
            }
            None => {
                if self.position.in_check(self.position.to_move) {
                    self.message.push_str(" Shah!");
                }
                self.phase = Phase::Turn;
            }
        }
    }

    pub fn thinking(&self) -> bool {
        matches!(&self.phase, Phase::Thinking { handle, .. } if !handle.is_finished())
    }

    pub fn tick(&mut self, now: Instant) {
        match &self.phase {
            Phase::Turn => {
                if let Player::Computer(strength) = self.player(self.position.to_move) {
                    let rules = self.rules.clone();
                    let position = self.position.clone();
                    let seen = self.seen.clone();
                    let mut rng = Rng::new(self.rng.next());
                    let handle =
                        std::thread::spawn(move || ai::choose(&rules, &position, strength, &seen, &mut rng));
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
    use crate::rules::parse;

    #[test]
    fn a_person_moves() {
        let mut game = Game::new(Rules::shatranj(), [Player::Human, Player::Human], Rng::new(1));
        game.pick(parse("b1").unwrap());
        let mut t = game.targets();
        t.sort();
        assert_eq!(t, [parse("a3").unwrap(), parse("c3").unwrap()]);
        game.pick(parse("c3").unwrap());
        assert_eq!(game.position.to_move, Side::Black);
        assert_eq!(game.record, ["Nb1-c3"]);
    }
}
