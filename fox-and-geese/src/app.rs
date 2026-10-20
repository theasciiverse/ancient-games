//! The game around the rules: picking pieces with the cursor, the
//! computer's moves (thought in the background), the end, and the draw by
//! threefold repetition.

use crate::ai;
use crate::rules::{Move, Point, Position, Rules, Side, on_board};
pub use ancient_core::player::{Player, Strength};
pub use ancient_core::rng::Rng;
use log::info;
use std::collections::HashMap;
use std::thread::JoinHandle;
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

    /// Who plays the fox and who the geese.
    pub fn players(&self) -> [Player; 2] {
        let computer = match self.opponent {
            0 => Some(Strength::Easy),
            1 => Some(Strength::Normal),
            2 => Some(Strength::Hard),
            _ => None,
        };
        let them = computer.map_or(Player::Human, Player::Computer);
        match self.side {
            Side::Fox => [Player::Human, them],
            Side::Geese => [them, Player::Human],
        }
    }
}

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
    /// Who plays the fox, and who the geese.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// How often each position has occurred, for the draw.
    pub seen: HashMap<Position, u32>,
    pub cursor: Point,
    /// The piece picked up by the person to move.
    pub selected: Option<Point>,
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);

fn index(side: Side) -> usize {
    match side {
        Side::Fox => 0,
        Side::Geese => 1,
    }
}

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let mut seen = HashMap::new();
        seen.insert(position.clone(), 1);
        // The cursor starts on the person's own pieces.
        let cursor = if players[0] == Player::Human && players[1] != Player::Human {
            position.fox
        } else {
            position.geese[position.geese.len() / 2]
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
            message: "The geese move first.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[index(side)]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The moves of the piece picked up.
    pub fn targets(&self) -> Vec<Move> {
        let Some(from) = self.selected else { return Vec::new() };
        let mut moves: Vec<Move> = self
            .position
            .moves(&self.rules)
            .into_iter()
            .filter(|m| m.from() == from)
            .collect();
        // One move per landing point: the one that kills the most.
        moves.sort_by_key(|m| std::cmp::Reverse(m.killed().len()));
        let mut seen = Vec::new();
        moves.retain(|m| {
            let fresh = !seen.contains(&m.to());
            seen.push(m.to());
            fresh
        });
        moves
    }

    /// The cursor moves to the nearest point in a direction.
    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let (mut r, mut c) = (self.cursor.0 as isize, self.cursor.1 as isize);
        for _ in 0..7 {
            r += dr;
            c += dc;
            if !(0..7).contains(&r) || !(0..7).contains(&c) {
                return;
            }
            let p = (r as usize, c as usize);
            if on_board(p) {
                self.cursor = p;
                return;
            }
        }
    }

    /// Picks up a piece, or puts the picked one down on a target, at `p`.
    pub fn pick(&mut self, p: Point) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        if let Some(mv) = self.targets().into_iter().find(|m| m.to() == p) {
            self.play(mv);
            return;
        }
        let mine = match self.position.to_move {
            Side::Fox => p == self.position.fox,
            Side::Geese => self.position.is_goose(p),
        };
        if mine {
            self.selected = Some(p);
            if self.targets().is_empty() {
                self.message = "That piece cannot move.".to_string();
            } else {
                self.message = "Now choose where it goes.".to_string();
            }
        } else {
            self.selected = None;
        }
    }

    pub fn cancel(&mut self) {
        self.selected = None;
    }

    fn play(&mut self, mv: Move) {
        let side = self.position.to_move;
        info!("{} plays {:?}", side.name(), mv.path);
        let killed = mv.killed().len();
        self.position.play(&mv);
        self.selected = None;
        self.last = Some(mv);
        let n = self.seen.entry(self.position.clone()).or_insert(0);
        *n += 1;
        let repeated = *n >= 3;
        self.message = match (side, killed) {
            (Side::Fox, 0) => "The fox moves.".to_string(),
            (Side::Fox, 1) => "The fox kills a goose!".to_string(),
            (Side::Fox, k) => format!("The fox kills {k} geese in one move!"),
            (Side::Geese, _) => "A goose moves.".to_string(),
        };
        if let Some(winner) = self.position.winner(&self.rules) {
            self.message.push_str(match winner {
                Side::Geese => " The fox is trapped: the geese win!",
                Side::Fox if self.position.geese.len() <= 3 => {
                    " Too few geese are left to trap him: the fox wins!"
                }
                Side::Fox => " The geese cannot move: the fox wins!",
            });
            self.phase = Phase::Over { winner: Some(winner) };
        } else if repeated {
            self.message.push_str(" The same position a third time: a draw.");
            self.phase = Phase::Over { winner: None };
        } else {
            self.phase = Phase::Turn;
        }
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

    /// The person's picking: a goose, then where it goes.
    #[test]
    fn pick_and_put_down() {
        let rules = Rules::thirteen();
        let mut game = Game::new(rules, [Player::Human, Player::Human], Rng::new(1));
        game.pick((4, 3));
        assert_eq!(game.selected, Some((4, 3)));
        assert!(game.targets().iter().any(|m| m.to() == (3, 3)));
        game.pick((3, 3));
        assert_eq!(game.position.to_move, Side::Fox);
        assert!(game.position.is_goose((3, 3)));
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
            assert!(matches!(game.phase, Phase::Over { .. }));
        }
    }
}
