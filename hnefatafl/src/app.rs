//! The game around the rules: picking pieces with the cursor, the
//! computer's moves (thought in the background), repetitions, and the end.

use crate::ai;
use crate::rules::{Move, Outcome, Position, Repetition, Rules, Side, Square};
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
    /// Over: the winner, or `None` for a draw.
    Over { winner: Option<Side> },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays the attackers, and who the defenders.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// How often each position has occurred.
    pub seen: HashMap<Position, u32>,
    pub cursor: Square,
    /// The piece picked up by the person to move.
    pub selected: Option<Square>,
    pub last: Option<Move>,
    /// Where pieces were just taken, to show it.
    pub taken: Vec<Square>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);

fn index(side: Side) -> usize {
    match side {
        Side::Attackers => 0,
        Side::Defenders => 1,
    }
}

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let mut seen = HashMap::new();
        seen.insert(position.clone(), 1);
        // The cursor starts on the person's own side.
        let cursor = if players[1] == Player::Human && players[0] != Player::Human {
            rules.throne()
        } else {
            rules.attackers[0]
        };
        Game {
            cursor,
            position,
            rules,
            players,
            phase: Phase::Turn,
            seen,
            selected: None,
            last: None,
            taken: Vec::new(),
            message: "The attackers move first.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[index(side)]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// Where the picked-up piece can go.
    pub fn targets(&self) -> Vec<Square> {
        let Some(from) = self.selected else { return Vec::new() };
        self.position.moves_from(&self.rules, from).iter().map(|m| m.to).collect()
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let n = self.rules.size as isize;
        let r = (self.cursor.0 as isize + dr).clamp(0, n - 1);
        let c = (self.cursor.1 as isize + dc).clamp(0, n - 1);
        self.cursor = (r as usize, c as usize);
    }

    /// Picks up a piece, or puts the picked one down, at `s`.
    pub fn pick(&mut self, s: Square) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = s;
        if let Some(from) = self.selected
            && self.targets().contains(&s)
        {
            self.play(Move { from, to: s });
            return;
        }
        if self.position.at(s).is_some_and(|p| p.side() == self.position.to_move) {
            self.selected = Some(s);
            self.message = if self.targets().is_empty() {
                "That piece cannot move.".to_string()
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
        let out = self.position.play(&self.rules, mv);
        info!("{} plays {:?}: {:?}", side.name(), mv, out);
        self.selected = None;
        self.last = Some(mv);
        self.taken = out.captured.clone();
        let n = self.seen.entry(self.position.clone()).or_insert(0);
        *n += 1;
        let repeated = *n >= 3;
        self.message = self.describe(side, &out);
        if let Some(winner) = out.winner {
            self.message.push_str(&self.why(winner));
            self.phase = Phase::Over { winner: Some(winner) };
        } else if repeated {
            let winner = match self.rules.repetition {
                Repetition::Draw => None,
                Repetition::DefendersLose => Some(Side::Attackers),
                Repetition::MoverLoses => Some(side.other()),
            };
            self.message.push_str(match self.rules.repetition {
                Repetition::Draw => " The same position a third time: a draw.",
                Repetition::DefendersLose => {
                    " The same position a third time: the defenders lose by repetition."
                }
                Repetition::MoverLoses => " The same position a third time: whoever repeats it loses.",
            });
            self.phase = Phase::Over { winner };
        } else {
            self.phase = Phase::Turn;
            // Bell's Tablut: the king calls out his ways out.
            if self.rules.id == "bell-tablut" {
                match self.position.clear_ways(&self.rules) {
                    1 => self.message.push_str(" The king calls Raichi!"),
                    n if n >= 2 => self.message.push_str(" The king calls Tuichi!"),
                    _ => {}
                }
            }
        }
    }

    fn describe(&self, side: Side, out: &Outcome) -> String {
        let king_taken = out.winner == Some(Side::Attackers) && self.position.king().is_none();
        let taken = out.captured.len() - king_taken as usize;
        let who = match side {
            Side::Attackers => "The attackers",
            Side::Defenders => "The defenders",
        };
        match taken {
            0 => format!("{who} move."),
            1 => format!("{who} take a piece!"),
            n => format!("{who} take {n} pieces!"),
        }
    }

    fn why(&self, winner: Side) -> String {
        match winner {
            Side::Defenders => {
                let king = self.position.king();
                if king.is_some_and(|k| self.rules.is_corner(k)) {
                    " The king reaches the corner: the defenders win!".to_string()
                } else if king.is_some_and(|k| self.rules.on_edge(k))
                    && self.rules.escape == crate::rules::Escape::Edge
                {
                    " The king reaches the edge: the defenders win!".to_string()
                } else if self.rules.exit_fort && king.is_some_and(|k| self.position.exit_fort(&self.rules, k)) {
                    " The king is safe in his fort on the edge: the defenders win!".to_string()
                } else {
                    " The attackers cannot move: the defenders win!".to_string()
                }
            }
            Side::Attackers => {
                if self.position.king().is_none() {
                    " The king is captured: the attackers win!".to_string()
                } else if self.rules.encirclement && self.position.encircled() {
                    " The defenders are surrounded: the attackers win!".to_string()
                } else {
                    " The defenders cannot move: the attackers win!".to_string()
                }
            }
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
                    let position = self.position.clone();
                    let seen = self.seen.clone();
                    let mut rng = Rng::new(self.rng.next());
                    let handle = std::thread::spawn(move || {
                        ai::choose(&rules, &position, strength, &seen, &mut rng)
                    });
                    self.phase = Phase::Thinking { handle, until: now + SHOW };
                }
            }
            Phase::Thinking { handle, until } => {
                if handle.is_finished() && now >= *until {
                    let Phase::Thinking { handle, .. } = std::mem::replace(&mut self.phase, Phase::Turn)
                    else {
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

    #[test]
    fn pick_and_put_down() {
        let rules = Rules::brandubh();
        let mut game = Game::new(rules, [Player::Human, Player::Human], Rng::new(1));
        game.pick((0, 3));
        assert_eq!(game.selected, Some((0, 3)));
        assert!(game.targets().contains(&(0, 1)));
        assert!(!game.targets().contains(&(0, 0)), "a corner");
        game.pick((0, 1));
        assert_eq!(game.position.to_move, Side::Defenders);
    }

    #[test]
    fn computer_games_end() {
        for rules in Rules::all() {
            let players = [Player::Computer(Strength::Easy), Player::Computer(Strength::Easy)];
            let mut game = Game::new(rules.clone(), players, Rng::new(3));
            let mut now = Instant::now();
            for _ in 0..40_000 {
                if matches!(game.phase, Phase::Over { .. }) {
                    break;
                }
                now += Duration::from_millis(500);
                game.tick(now);
                std::thread::sleep(Duration::from_millis(1));
            }
            assert!(matches!(game.phase, Phase::Over { .. }), "{}", rules.id);
        }
    }
}
