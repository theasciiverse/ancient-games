//! The game around the rules: the menu, whose turn it is, throwing, choosing
//! a move, the computer's turns, and the end.

use crate::ai::{self, Strength};
use crate::rules::{Move, Outcome, Position, Rules, Side, Throw};
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
    Choosing { throw: Throw, moves: Vec<Move>, selected: usize },
    /// A throw with no move: shown until `until`, then the turn passes,
    /// unless the throw says to throw again.
    NoMove { throw: Throw, until: Instant },
    /// The computer is thinking about its move (shown a moment).
    Thinking { throw: Throw, until: Instant },
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
    /// In a betting game: each side's purse and the pool.
    pub purses: [u32; 2],
    pub pool: u32,
}

/// What each side has in its purse when a session starts, for betting games.
pub const PURSE: u32 = 20;

/// How long things take on screen.
const TUMBLE: Duration = Duration::from_millis(650);
const PAUSE: Duration = Duration::from_millis(900);
const THINK: Duration = Duration::from_millis(550);

impl Game {
    /// A new game; `purses` carry over from earlier betting games.
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng, purses: [u32; 2]) -> Game {
        let first = if rng.coin() { Side::Light } else { Side::Dark };
        let lots = vec![false; rules.lots.count() as usize];
        info!("new game: {} {:?}, {} starts", rules.id, players, first.name());
        let mut purses = purses;
        let mut pool = 0;
        if let Some(stakes) = rules.stakes {
            for purse in &mut purses {
                let paid = stakes.stake.min(*purse);
                *purse -= paid;
                pool += paid;
            }
        }
        Game {
            purses,
            pool,
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
    pub fn throw_shown(&self) -> Throw {
        let marked = self.lots.iter().filter(|&&m| m).count() as u32;
        self.rules.lots.throw(marked)
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
        if let Phase::Choosing { throw, moves, selected } = &self.phase {
            let (throw, mv) = (*throw, moves[*selected]);
            self.play(mv, throw);
        }
    }

    fn play(&mut self, mv: Move, throw: Throw) {
        let side = self.to_move();
        let out = self.position.play(&self.rules, mv, throw.again);
        info!("{} plays {}->{} {:?}", side.name(), mv.from, mv.to, out);
        self.last = Some((side, mv));
        self.message = describe(side, mv, out, throw.again);
        if let (true, Some(stakes)) = (out.fine, self.rules.stakes) {
            let payer = side.other().index();
            let paid = stakes.fine.min(self.purses[payer]);
            self.purses[payer] -= paid;
            self.pool += paid;
            self.message
                .push_str(&format!(" {} pays {paid} into the pool.", side.other().name()));
        }
        if out.won && self.rules.stakes.is_some() {
            self.purses[side.index()] += self.pool;
            self.message
                .push_str(&format!(" {} takes the pool of {}.", side.name(), self.pool));
            self.pool = 0;
        }
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
                let throw = self.throw_shown();
                let roll = throw.moves;
                let side = self.to_move();
                let moves = self.position.moves(&self.rules, roll);
                debug!("{} threw {throw:?}, {} moves", side.name(), moves.len());
                let again = if throw.again { " and throws again" } else { "" };
                if moves.is_empty() {
                    self.message = match (roll, throw.again) {
                        (0, false) => format!("{} threw nothing. The turn passes.", side.name()),
                        (_, false) => format!("{} threw {roll} but cannot move.", side.name()),
                        (_, true) => format!(
                            "{} threw {roll} but cannot move, and throws again.",
                            side.name()
                        ),
                    };
                    self.phase = Phase::NoMove { throw, until: now + PAUSE };
                } else if self.human_to_move() {
                    self.message = format!(
                        "{} threw {roll}{again}. Choose a piece{}.",
                        side.name(),
                        if moves.len() > 1 { " (arrows, Enter)" } else { " (Enter)" }
                    );
                    self.phase = Phase::Choosing { throw, moves, selected: 0 };
                } else {
                    self.message = format!("{} threw {roll}{again}.", side.name());
                    self.phase = Phase::Thinking { throw, until: now + THINK };
                }
            }
            Phase::NoMove { throw, until } => {
                if now >= until {
                    self.position.pass(throw.again);
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
                        Some(mv) => self.play(mv, throw),
                        None => {
                            self.position.pass(throw.again);
                            self.phase = Phase::ToThrow;
                        }
                    }
                }
            }
            Phase::Choosing { .. } | Phase::Over { .. } => {}
        }
    }
}

fn describe(side: Side, mv: Move, out: Outcome, throw_again: bool) -> String {
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
    } else if out.fine {
        text.push_str(" A rosette!");
    }
    if throw_again && !out.again {
        text.push_str(" Throw again.");
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
                let mut game = Game::new(rules.clone(), players, Rng::new(seed * 7 + i as u64), [PURSE; 2]);
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

#[cfg(test)]
mod length {
    use super::*;
    use crate::rules::Rules;

    #[test]
    #[ignore]
    fn game_lengths() {
        for rules in Rules::all() {
            let mut throws_total = 0;
            let games = 30;
            for seed in 0..games {
                let players = [Player::Computer(Strength::Normal); 2];
                let mut game = Game::new(rules.clone(), players, Rng::new(seed + 11), [PURSE; 2]);
                let mut now = Instant::now();
                let mut throws = 0;
                while !matches!(game.phase, Phase::Over { .. }) {
                    if game.phase == Phase::ToThrow {
                        throws += 1;
                    }
                    now += Duration::from_millis(100);
                    game.tick(now);
                }
                throws_total += throws;
            }
            println!("{}: {} throws a game on average", rules.id, throws_total / games);
        }
    }
}
