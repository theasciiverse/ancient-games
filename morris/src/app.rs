//! The game around the rules: entering and moving men with the cursor,
//! choosing which man a mill takes, the computer's moves (thought in the
//! background), the end, and the draw by threefold repetition.

use crate::ai;
use crate::rules::{Board, Move, Outcome, Position, Rules, Side, Why};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How the game stands.
pub enum Phase {
    /// The side to move is choosing (a person) or about to think.
    Turn,
    /// The computer is thinking; its move is shown no sooner than `until`.
    Thinking { handle: JoinHandle<Option<Move>>, until: Instant },
    /// Over: the outcome, or `None` for a draw by repetition.
    Over { outcome: Option<Outcome> },
}

pub struct Game {
    pub rules: Rules,
    pub board: Board,
    /// Who plays White, and who Black.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// How often each position has occurred, for the draw.
    pub seen: HashMap<Position, u32>,
    pub cursor: usize,
    /// The man picked up by the person to move.
    pub selected: Option<usize>,
    /// A move that has made a mill, waiting for the person to choose the
    /// men it takes, and those chosen so far.
    pub pending: Option<(Option<u8>, u8, Vec<u8>)>,
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let board = rules.board();
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let mut seen = HashMap::new();
        seen.insert(position.clone(), 1);
        // The cursor starts in the middle of the board, or near it.
        let mid = board.size / 2;
        let cursor = (0..board.len())
            .min_by_key(|&p| {
                let (r, c) = board.points[p];
                r.abs_diff(mid) + c.abs_diff(mid)
            })
            .unwrap_or(0);
        Game {
            cursor,
            board,
            position,
            rules,
            players,
            phase: Phase::Turn,
            seen,
            selected: None,
            pending: None,
            last: None,
            message: "White begins: enter a man on any empty point.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    pub fn entering(&self) -> bool {
        self.position.hand[self.position.to_move.index()] > 0
    }

    /// The legal moves matching what the person has chosen so far.
    fn matching(&self) -> Vec<Move> {
        let moves = self.position.moves(&self.rules, &self.board);
        match &self.pending {
            Some((from, to, chosen)) => moves
                .into_iter()
                .filter(|m| m.from == *from && m.to == *to)
                .filter(|m| chosen.iter().enumerate().all(|(i, &q)| m.take[i] == Some(q)))
                .collect(),
            None => match self.selected {
                Some(s) => moves.into_iter().filter(|m| m.from == Some(s as u8)).collect(),
                None => moves,
            },
        }
    }

    /// The points lit for the person: where the man picked up may go, or
    /// the men a mill may take.
    pub fn targets(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        let mut lit: Vec<usize> = match &self.pending {
            Some((_, _, chosen)) => {
                self.matching().iter().filter_map(|m| m.take[chosen.len()]).map(|q| q as usize).collect()
            }
            None if self.selected.is_some() => self.matching().iter().map(|m| m.to as usize).collect(),
            None => Vec::new(),
        };
        lit.sort_unstable();
        lit.dedup();
        lit
    }

    /// The cursor moves to the nearest point in a direction.
    pub fn move_cursor(&mut self, dr: i32, dc: i32) {
        let (r0, c0) = self.board.points[self.cursor];
        let (r0, c0) = (r0 as i32, c0 as i32);
        let best = (0..self.board.len())
            .filter_map(|p| {
                let (r, c) = self.board.points[p];
                let (r, c) = (r as i32 - r0, c as i32 - c0);
                // How far along the direction, and how far off it.
                let along = r * dr + c * dc;
                let off = (r * dc - c * dr).abs();
                (along > 0).then_some((along + 2 * off, p))
            })
            .min();
        if let Some((_, p)) = best {
            self.cursor = p;
        }
    }

    /// Enters a man, picks one up, puts it down, or chooses a man to take,
    /// at point `p`.
    pub fn pick(&mut self, p: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        let me = self.position.to_move;
        if self.pending.is_some() {
            if self.targets().contains(&p)
                && let Some((_, _, chosen)) = self.pending.as_mut()
            {
                chosen.push(p as u8);
            }
            self.settle();
            return;
        }
        if self.entering() {
            if self.position.side_at(p).is_none() {
                self.commit(None, p as u8);
            } else {
                self.message = "Enter a man on an empty point.".to_string();
            }
            return;
        }
        if self.selected.is_some() && self.targets().contains(&p) {
            let from = self.selected.map(|s| s as u8);
            self.commit(from, p as u8);
            return;
        }
        if self.position.side_at(p) == Some(me) {
            self.selected = Some(p);
            self.message = if self.targets().is_empty() {
                "That man cannot move.".to_string()
            } else if self.position.flies(&self.rules, me) {
                "Down to three: it may fly to any empty point.".to_string()
            } else {
                "Now choose where it goes.".to_string()
            };
        } else {
            self.selected = None;
        }
    }

    /// The person has chosen where a man goes: play it, or first ask which
    /// men its mill takes.
    fn commit(&mut self, from: Option<u8>, to: u8) {
        self.pending = Some((from, to, Vec::new()));
        if self.matching().is_empty() {
            self.pending = None;
            return;
        }
        self.settle();
    }

    /// Plays the pending move once nothing is left to choose.
    fn settle(&mut self) {
        let moves = self.matching();
        let Some((_, _, chosen)) = &self.pending else {
            return;
        };
        let more = moves.iter().any(|m| m.take.get(chosen.len()).copied().flatten().is_some());
        if moves.len() == 1 || !more {
            let mv = moves[0];
            self.pending = None;
            self.play(mv);
        } else {
            let other = self.position.to_move.other().name();
            self.message = if chosen.is_empty() {
                format!("A mill! Choose a man of {other}'s to take.")
            } else {
                format!("Two mills! Choose a second man of {other}'s.")
            };
        }
    }

    pub fn cancel(&mut self) {
        if self.pending.take().is_some() {
            // A man entered or moved stays where it went only once its
            // mill has taken; until then, the choice can be undone.
            self.message = "Choose again.".to_string();
        }
        self.selected = None;
    }

    fn play(&mut self, mv: Move) {
        let side = self.position.to_move;
        info!("{} plays {:?}", side.name(), mv);
        self.position.play(&self.rules, &self.board, &mv);
        self.selected = None;
        self.last = Some(mv);
        let n = self.seen.entry(self.position.clone()).or_insert(0);
        *n += 1;
        let repeated = *n >= 3;
        let to = self.board.name(mv.to as usize);
        let taken: Vec<String> = mv.taken().map(|p| self.board.name(p)).collect();
        self.message = match (mv.from, taken.is_empty()) {
            (None, true) => format!("{} enters a man on {to}.", side.name()),
            (Some(f), true) => format!("{} moves {} to {to}.", side.name(), self.board.name(f as usize)),
            (None, false) => format!("{} enters on {to}: a mill, and takes {}.", side.name(), taken.join(" and ")),
            (Some(f), false) => format!(
                "{} moves {} to {to}: a mill, and takes {}.",
                side.name(),
                self.board.name(f as usize),
                taken.join(" and ")
            ),
        };
        if let Some(outcome) = self.position.outcome(&self.rules, &self.board) {
            self.message.push(' ');
            self.message.push_str(&match outcome {
                Outcome::Won(w, Why::Line) => format!("Three in a line: {} wins!", w.name()),
                Outcome::Won(w, Why::TwoLeft) => format!("{} is down to two men: {} wins!", w.other().name(), w.name()),
                Outcome::Won(w, _) => {
                    format!("{} cannot move: {} wins!", w.other().name(), w.name())
                }
                Outcome::Drawn(Why::BoardFull) => "The board is full: a draw.".to_string(),
                Outcome::Drawn(Why::Quiet) => {
                    "Ten moves each with a side down to three and no man taken: a draw.".to_string()
                }
                Outcome::Drawn(_) => "Every man is placed and nobody has a line: a draw.".to_string(),
            });
            self.phase = Phase::Over { outcome: Some(outcome) };
        } else if repeated {
            self.message.push_str(" The same position a third time: a draw.");
            self.phase = Phase::Over { outcome: None };
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
                let side = self.position.to_move;
                if let Player::Computer(strength) = self.player(side) {
                    let rules = self.rules.clone();
                    let board = self.board.clone();
                    let position = self.position.clone();
                    let seen = self.seen.clone();
                    let mut rng = Rng::new(self.rng.next());
                    let handle =
                        std::thread::spawn(move || ai::choose(&rules, &board, &position, strength, &seen, &mut rng));
                    self.phase = Phase::Thinking { handle, until: now + SHOW };
                }
            }
            Phase::Thinking { handle, until } => {
                if handle.is_finished() && now >= *until {
                    let Phase::Thinking { handle, .. } = std::mem::replace(&mut self.phase, Phase::Turn) else {
                        unreachable!()
                    };
                    match handle.join().ok().flatten() {
                        Some(mv) => self.play(mv),
                        None => self.phase = Phase::Turn,
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
    use ancient_core::player::Strength;

    fn at(game: &Game, name: &str) -> usize {
        (0..game.board.len()).find(|&p| game.board.name(p) == name).unwrap()
    }

    /// The person's choices: enter, make a mill, choose the man it takes.
    #[test]
    fn enter_make_a_mill_and_take() {
        let mut game = Game::new(Rules::nine_modern(), [Player::Human, Player::Human], Rng::new(1));
        for name in ["a1", "a7", "d1", "d7"] {
            let p = at(&game, name);
            game.pick(p);
        }
        assert_eq!(game.position.men[0].count_ones(), 2);
        let g1 = at(&game, "g1");
        game.pick(g1);
        assert!(game.pending.is_some(), "waits for the man to take");
        assert_eq!(game.targets().len(), 2);
        let a7 = at(&game, "a7");
        game.pick(a7);
        assert!(game.pending.is_none());
        assert_eq!(game.position.men[1].count_ones(), 1);
        assert_eq!(game.position.to_move, Side::Black);
    }

    #[test]
    fn the_cursor_walks_the_board() {
        let mut game = Game::new(Rules::nine_modern(), [Player::Human, Player::Human], Rng::new(1));
        game.cursor = at(&game, "a1");
        game.move_cursor(-1, 0);
        assert_eq!(game.board.name(game.cursor), "a4");
        game.move_cursor(0, 1);
        assert_eq!(game.board.name(game.cursor), "b4");
        game.move_cursor(-1, 0);
        assert_eq!(game.board.name(game.cursor), "b6");
    }

    #[test]
    fn computer_games_end() {
        for rules in Rules::all() {
            let players = [Player::Computer(Strength::Easy), Player::Computer(Strength::Easy)];
            let mut game = Game::new(rules, players, Rng::new(3));
            let mut now = Instant::now();
            for _ in 0..20_000 {
                if matches!(game.phase, Phase::Over { .. }) {
                    break;
                }
                now += Duration::from_millis(500);
                game.tick(now);
                std::thread::sleep(Duration::from_millis(1));
            }
            assert!(matches!(game.phase, Phase::Over { .. }), "{}", game.rules.id);
        }
    }
}
