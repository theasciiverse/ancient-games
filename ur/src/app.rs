//! The game around the rules: the menu, whose turn it is, throwing, choosing
//! a move, the computer's turns, and the end.

use crate::ai::{self, Strength};
use crate::rules::{Move, Outcome, Position, Rules, Side};
use log::{debug, info};
use std::time::{Duration, Instant};

/// A small, seedable random number generator (xorshift64*), so a game can
/// be repeated with `--seed`.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    pub fn coin(&mut self) -> bool {
        self.next() >> 63 == 1
    }
}

/// Who plays a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Human,
    Computer(Strength),
}

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
            Side::Light => [Player::Human, them],
            Side::Dark => [them, Player::Human],
        }
    }
}

/// What is happening in a game.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Waiting for the side to move to throw.
    ToThrow,
    /// The lots are tumbling; they land at `until`.
    Throwing { until: Instant },
    /// The throw is shown; the side picks one of `moves` (`selected`).
    Choosing { roll: u8, moves: Vec<Move>, selected: usize },
    /// A throw with no move: shown until `until`, then the turn passes.
    NoMove { roll: u8, until: Instant },
    /// The computer is thinking about its move (shown a moment).
    Thinking { roll: u8, until: Instant },
    /// Somebody won.
    Over { winner: Side },
}

/// A game in progress.
pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// The four lots as they lie: marked tip up or not.
    pub lots: Vec<bool>,
    /// The last thing that happened, for the message line.
    pub message: String,
    /// The last move, to show where it went.
    pub last: Option<(Side, Move)>,
    pub rng: Rng,
}

/// How long things take on screen.
const TUMBLE: Duration = Duration::from_millis(650);
const PAUSE: Duration = Duration::from_millis(900);
const THINK: Duration = Duration::from_millis(550);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng) -> Game {
        let first = if rng.coin() { Side::Light } else { Side::Dark };
        let lots = vec![false; rules.lots.count() as usize];
        info!("new game: {} {:?}, {} starts", rules.id, players, first.name());
        Game {
            position: Position::new(&rules, first),
            rules,
            players,
            phase: Phase::ToThrow,
            lots,
            message: format!("{} throws first.", first.name()),
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

    /// Whether the side to move is a person at the keyboard.
    pub fn human_to_move(&self) -> bool {
        self.player(self.to_move()) == Player::Human
    }

    /// The throw shown on the lots.
    pub fn roll(&self) -> u8 {
        let marked = self.lots.iter().filter(|&&m| m).count() as u32;
        self.rules.lots.moves(marked)
    }

    /// Throws the lots, if it is time to.
    pub fn throw(&mut self, now: Instant) {
        if self.phase == Phase::ToThrow {
            debug!("{} throws", self.to_move().name());
            self.phase = Phase::Throwing { until: now + TUMBLE };
        }
    }

    /// Moves the selection among the legal moves.
    pub fn select(&mut self, step: isize) {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase {
            let n = moves.len() as isize;
            *selected = ((*selected as isize + step).rem_euclid(n)) as usize;
        }
    }

    /// Picks the move starting from path position `from`, if legal.
    pub fn select_from(&mut self, from: u8) -> bool {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase
            && let Some(i) = moves.iter().position(|m| m.from == from)
        {
            *selected = i;
            return true;
        }
        false
    }

    /// Plays the selected move.
    pub fn confirm(&mut self) {
        if let Phase::Choosing { moves, selected, .. } = &self.phase {
            let mv = moves[*selected];
            self.play(mv);
        }
    }

    fn play(&mut self, mv: Move) {
        let side = self.to_move();
        let out = self.position.play(&self.rules, mv);
        info!("{} plays {}->{} {:?}", side.name(), mv.from, mv.to, out);
        self.last = Some((side, mv));
        self.message = describe(side, mv, out);
        self.phase = match out {
            Outcome { won: true, .. } => Phase::Over { winner: side },
            _ => Phase::ToThrow,
        };
    }

    /// Advances timed phases and the computer's turns.
    pub fn tick(&mut self, now: Instant) {
        match self.phase.clone() {
            Phase::ToThrow => {
                if !self.human_to_move() {
                    self.throw(now);
                }
            }
            Phase::Throwing { until } => {
                if now < until {
                    // Tumbling: the tips flicker.
                    for lot in &mut self.lots {
                        *lot = self.rng.coin();
                    }
                    return;
                }
                for lot in &mut self.lots {
                    *lot = self.rng.coin();
                }
                let roll = self.roll();
                let side = self.to_move();
                let moves = self.position.moves(&self.rules, roll);
                debug!("{} threw {roll}, {} moves", side.name(), moves.len());
                if moves.is_empty() {
                    self.message = if roll == 0 {
                        format!("{} threw nothing. The turn passes.", side.name())
                    } else {
                        format!("{} threw {roll} but cannot move.", side.name())
                    };
                    self.phase = Phase::NoMove { roll, until: now + PAUSE };
                } else if self.human_to_move() {
                    self.message = format!(
                        "{} threw {roll}. Choose a piece{}.",
                        side.name(),
                        if moves.len() > 1 { " (arrows, Enter)" } else { " (Enter)" }
                    );
                    self.phase = Phase::Choosing { roll, moves, selected: 0 };
                } else {
                    self.message = format!("{} threw {roll}.", side.name());
                    self.phase = Phase::Thinking { roll, until: now + THINK };
                }
            }
            Phase::NoMove { until, .. } => {
                if now >= until {
                    self.position.pass();
                    self.phase = Phase::ToThrow;
                    self.message = format!("{} to throw.", self.to_move().name());
                }
            }
            Phase::Thinking { roll, until } => {
                if now >= until {
                    let Player::Computer(strength) = self.player(self.to_move()) else {
                        return;
                    };
                    match ai::choose(&self.rules, &self.position, roll, strength) {
                        Some(mv) => self.play(mv),
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

fn describe(side: Side, mv: Move, out: Outcome) -> String {
    let who = side.name();
    if out.won {
        return format!("{who} brings the last piece home and wins!");
    }
    let mut text = if out.home {
        format!("{who} brings a piece home.")
    } else if mv.from == 0 {
        format!("{who} enters a piece.")
    } else {
        format!("{who} moves {} squares.", mv.to - mv.from)
    };
    if out.took {
        text.push_str(" A piece is taken and sent back!");
    }
    if out.again {
        text.push_str(" A rosette: throw again.");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rules;

    /// Two computers play whole games through the same phases the screen
    /// shows, with time running fast, to the end.
    #[test]
    fn computer_games_run_to_the_end() {
        for (i, rules) in Rules::all().into_iter().enumerate() {
            for seed in 1..=3u64 {
                let players = [
                    Player::Computer(Strength::Normal),
                    Player::Computer(Strength::Hard),
                ];
                let mut game = Game::new(rules.clone(), players, Rng::new(seed * 7 + i as u64));
                let mut now = Instant::now();
                let mut steps = 0;
                while !matches!(game.phase, Phase::Over { .. }) {
                    now += Duration::from_millis(100);
                    game.tick(now);
                    steps += 1;
                    assert!(steps < 200_000, "{} seed {seed}: no end", rules.id);
                }
                let Phase::Over { winner } = game.phase else { unreachable!() };
                assert_eq!(game.position.winner(), Some(winner));
                assert!(game.message.contains("wins"));
            }
        }
    }
}
