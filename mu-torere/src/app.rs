//! The game around the rules: choosing a man with the cursor (with one
//! empty point, a man that can move has one place to go, so choosing it
//! moves it), the computer's moves, the end, and the draw by threefold
//! repetition.

use crate::ai;
use crate::rules::solve::{self, Value};
use crate::rules::{CENTRE, Move, POINTS, Position, Rules, Side};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How the game stands.
pub enum Phase {
    /// The side to move is choosing (a person) or the computer is about to
    /// move, no sooner than `until`.
    Turn { until: Instant },
    /// Over: the winner, or `None` for a draw by repetition.
    Over { winner: Option<Side> },
}

pub struct Game {
    pub rules: Rules,
    /// Every position's value, for the computer.
    pub table: HashMap<Position, Value>,
    /// Who plays White, and who Black.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// How often each position has occurred, for the draw.
    pub seen: HashMap<Position, u32>,
    pub cursor: usize,
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

/// How long the computer's move waits, so it can be followed.
const SHOW: Duration = Duration::from_millis(700);

/// A point's name in messages.
pub fn name(p: usize) -> String {
    if p == CENTRE { "the pūtahi".to_string() } else { format!("kewai {}", p + 1) }
}

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let table = solve::table(&rules);
        let seen = HashMap::from([(position, 1)]);
        let message = format!("{} begins: move a man into the empty pūtahi.", rules.first.name());
        Game {
            cursor: position.moves(&rules).first().map_or(0, |m| m.from as usize),
            table,
            position,
            rules,
            players,
            phase: Phase::Turn { until: Instant::now() },
            seen,
            last: None,
            message,
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn { .. }) && self.player(self.position.to_move) == Player::Human
    }

    /// The men the person to move may move, lit on the board.
    pub fn movable(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        self.position.moves(&self.rules).iter().map(|m| m.from as usize).collect()
    }

    /// The cursor moves to the nearest point in a direction, by where the
    /// points stand on the star (`at`: column and row of each).
    pub fn move_cursor(&mut self, dx: i32, dy: i32, at: &[(i32, i32); POINTS]) {
        let (x0, y0) = at[self.cursor];
        let best = (0..POINTS)
            .filter_map(|p| {
                let (x, y) = (at[p].0 - x0, at[p].1 - y0);
                let along = x * dx + y * dy;
                let off = (x * dy - y * dx).abs();
                (along > 0).then_some((along + 2 * off, p))
            })
            .min();
        if let Some((_, p)) = best {
            self.cursor = p;
        }
    }

    /// The person chooses the man on `p`: it moves to the empty point.
    pub fn pick(&mut self, p: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        match self.position.moves(&self.rules).into_iter().find(|m| m.from as usize == p) {
            Some(mv) => self.play(mv, Instant::now()),
            None if self.position.at[p] == Some(self.position.to_move) => {
                self.message = if self.position.empty() == CENTRE {
                    "That man may not go into the pūtahi: it has no enemy beside it.".to_string()
                } else {
                    "That man is not next to the empty point.".to_string()
                }
            }
            None => self.message = format!("Choose one of {}'s men.", self.position.to_move.name()),
        }
    }

    fn play(&mut self, mv: Move, now: Instant) {
        let side = self.position.to_move;
        info!("{} plays {:?}", side.name(), mv);
        self.position.play(&self.rules, &mv);
        self.last = Some(mv);
        let n = self.seen.entry(self.position).or_insert(0);
        *n += 1;
        let repeated = *n >= 3;
        self.message = format!("{} moves from {} to {}.", side.name(), name(mv.from as usize), name(mv.to as usize));
        if let Some(winner) = self.position.winner(&self.rules) {
            self.message.push_str(&format!(" {} cannot move: {} wins!", winner.other().name(), winner.name()));
            self.phase = Phase::Over { winner: Some(winner) };
        } else if repeated {
            self.message.push_str(" The same position a third time: a draw.");
            self.phase = Phase::Over { winner: None };
        } else {
            self.phase = Phase::Turn { until: now + SHOW };
            // The cursor goes to a man the next person may move.
            if let Some(m) = self.position.moves(&self.rules).first() {
                self.cursor = m.from as usize;
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        let Phase::Turn { until } = self.phase else { return };
        let side = self.position.to_move;
        if let Player::Computer(strength) = self.player(side)
            && now >= until
        {
            let mv = ai::choose(&self.rules, &self.table, &self.position, strength, &self.seen, &mut self.rng);
            if let Some(mv) = mv {
                self.play(mv, now);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ancient_core::player::Strength;

    #[test]
    fn choosing_a_man_moves_it() {
        let mut game = Game::new(Rules::beside_enemy(), [Player::Human, Player::Human], Rng::new(1));
        assert_eq!(game.movable(), [0, 3]);
        game.pick(1);
        assert!(game.message.contains("may not go into"));
        game.pick(3);
        assert_eq!(game.position.at[CENTRE], Some(Side::White));
        assert_eq!(game.position.to_move, Side::Black);
    }

    #[test]
    fn computer_games_end() {
        let players = [Player::Computer(Strength::Hard), Player::Computer(Strength::Easy)];
        let mut game = Game::new(Rules::beside_enemy(), players, Rng::new(3));
        let mut now = Instant::now();
        for _ in 0..2000 {
            if matches!(game.phase, Phase::Over { .. }) {
                break;
            }
            now += Duration::from_secs(1);
            game.tick(now);
        }
        assert!(matches!(game.phase, Phase::Over { .. }));
    }
}
