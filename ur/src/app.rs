//! The game around the rules: the menu, whose turn it is, throwing, choosing
//! a move, the computer's turns, and the end.

use crate::ai;
pub use ancient_core::player::{Player, Strength};
pub use ancient_core::rng::Rng;
use crate::rules::{Lots, Move, Outcome, Position, Rosettes, Rules, Side, TABLET_TOKENS, Throw};
use log::{debug, info};
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
    /// The throw is shown; the side picks one of `moves` (`selected`). With
    /// the knucklebones the side may first dare the ox bone (`can_dare`);
    /// then `moves` may be empty, and confirming lets the throw pass.
    Choosing { throw: Throw, moves: Vec<Move>, selected: usize, can_dare: bool },
    /// The ox bone is tumbling after the sheep bone showed `sheep`.
    Daring { sheep: u8, until: Instant },
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
    /// The lots as they lie: marked tip up or not.
    pub lots: Vec<bool>,
    /// The knucklebones as they lie: the sheep bone's face, and the ox
    /// bone's answer once thrown.
    pub sheep: u8,
    pub ox: Option<bool>,
    /// The throw being played.
    pub shown: Throw,
    /// The last thing that happened, for the message line.
    pub message: String,
    /// The last move, to show where it went.
    pub last: Option<(Side, Move)>,
    pub rng: Rng,
    /// In a betting game: each side's purse and the pool.
    pub purses: [u32; 2],
    pub pool: u32,
}

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
            sheep: 1,
            ox: None,
            shown: Throw { moves: 0, again: false },
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

    fn knucklebones(&self) -> bool {
        self.rules.lots == Lots::Astragals
    }

    /// Throws the lots, if it is time to.
    pub fn throw(&mut self, now: Instant) {
        if self.phase == Phase::ToThrow {
            debug!("{} throws", self.to_move().name());
            self.ox = None;
            self.phase = Phase::Throwing { until: now + TUMBLE };
        }
    }

    /// Dares the ox bone, if the throw allows it.
    pub fn dare(&mut self, now: Instant) {
        if let Phase::Choosing { throw, can_dare: true, .. } = self.phase {
            debug!("{} dares the ox bone on {}", self.to_move().name(), throw.moves);
            self.message = format!("{} dares the ox bone...", self.to_move().name());
            self.phase = Phase::Daring { sheep: throw.moves, until: now + TUMBLE };
        }
    }

    /// Moves the selection among the legal moves.
    pub fn select(&mut self, step: isize) {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase
            && !moves.is_empty()
        {
            let n = moves.len() as isize;
            *selected = ((*selected as isize + step).rem_euclid(n)) as usize;
        }
    }

    /// Picks the move starting from path position `from`, if legal; a
    /// second click on a piece with several targets (the Swallow) cycles.
    pub fn select_from(&mut self, from: u8) -> bool {
        if let Phase::Choosing { moves, selected, .. } = &mut self.phase {
            let n = moves.len();
            for k in 1..=n {
                let i = (*selected + k) % n;
                if moves[i].from == from {
                    *selected = i;
                    return true;
                }
            }
        }
        false
    }

    /// Plays the selected move, or lets a useless throw pass.
    pub fn confirm(&mut self, now: Instant) {
        if let Phase::Choosing { throw, moves, selected, .. } = &self.phase {
            let throw = *throw;
            if moves.is_empty() {
                self.message = format!("{} lets the throw pass.", self.to_move().name());
                self.phase = Phase::NoMove { throw, until: now };
            } else {
                let mv = moves[*selected];
                self.play(mv, throw);
            }
        }
    }

    fn piece_label(&self, side: Side, mv: Move) -> String {
        match self.rules.piece_name(mv.piece) {
            Some(name) => format!("{}'s {name}", side.name()),
            None => side.name().to_string(),
        }
    }

    fn play(&mut self, mv: Move, throw: Throw) {
        let side = self.to_move();
        let out = self.position.play(&self.rules, mv, throw.again);
        info!("{} plays {} {}->{} {:?}", side.name(), mv.piece, mv.from, mv.to, out);
        self.last = Some((side, mv));
        self.message = describe(&self.piece_label(side, mv), mv, out, throw.again);
        let me = side.index();
        let them = side.other().index();
        match self.rules.rosettes {
            Rosettes::OpponentPays if out.rosette => {
                let fine = self.rules.stakes.map_or(0, |s| s.fine);
                let paid = fine.min(self.purses[them]);
                self.purses[them] -= paid;
                self.pool += paid;
                self.message
                    .push_str(&format!(" {} pays {paid} into the pool.", side.other().name()));
            }
            Rosettes::Tokens => {
                let value = TABLET_TOKENS[mv.piece];
                if out.passed > 0 {
                    let due = value * out.passed as u32;
                    let paid = due.min(self.purses[me]);
                    self.purses[me] -= paid;
                    self.pool += paid;
                    self.message.push_str(&format!(" Passing a rosette costs {paid}."));
                }
                if out.rosette {
                    let won = value.min(self.pool);
                    self.pool -= won;
                    self.purses[me] += won;
                    self.message.push_str(&format!(" {} wins {won} from the pool.", side.name()));
                }
            }
            _ => {}
        }
        if out.won && self.rules.stakes.is_some() {
            self.purses[me] += self.pool;
            self.message
                .push_str(&format!(" {} takes the pool of {}.", side.name(), self.pool));
            self.pool = 0;
        }
        self.phase = match out {
            Outcome { won: true, .. } => Phase::Over { winner: side },
            _ => Phase::ToThrow,
        };
    }

    /// The throw is known: offer it to the side, the computer or a person.
    fn offer(&mut self, throw: Throw, can_dare: bool, now: Instant) {
        self.shown = throw;
        let roll = throw.moves;
        let side = self.to_move();
        let moves = self.position.moves(&self.rules, roll);
        debug!("{} has {throw:?}, {} moves", side.name(), moves.len());
        let again = if throw.again { " and throws again" } else { "" };
        let who = side.name();

        if let Player::Computer(strength) = self.player(side) {
            if can_dare && ai::wants_ox(&self.rules, &self.position, roll, strength) {
                self.message = format!("{who} threw {roll} and dares the ox bone...");
                self.phase = Phase::Daring { sheep: roll, until: now + TUMBLE };
            } else if moves.is_empty() {
                self.message = no_move(who, roll, throw.again);
                self.phase = Phase::NoMove { throw, until: now + PAUSE };
            } else {
                self.message = format!("{who} threw {roll}{again}.");
                self.phase = Phase::Thinking { throw, until: now + THINK };
            }
            return;
        }

        if moves.is_empty() && !can_dare {
            self.message = no_move(who, roll, throw.again);
            self.phase = Phase::NoMove { throw, until: now + PAUSE };
            return;
        }
        self.message = match (moves.is_empty(), can_dare) {
            (true, _) => format!("{who} threw {roll} and cannot move: D dares the ox bone, Enter lets it pass."),
            (false, true) => format!("{who} threw {roll}: choose a piece, or D to dare the ox bone."),
            (false, false) => format!(
                "{who} threw {roll}{again}. Choose a piece{}.",
                if moves.len() > 1 { " (arrows, Enter)" } else { " (Enter)" }
            ),
        };
        self.phase = Phase::Choosing { throw, moves, selected: 0, can_dare };
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
                // Tumbling: the tips flicker, the bone turns.
                for lot in &mut self.lots {
                    *lot = self.rng.coin();
                }
                self.sheep = 1 + (self.rng.next() % 4) as u8;
                if now < until {
                    return;
                }
                if self.knucklebones() {
                    let throw = self.rules.lots.throw(self.sheep as u32);
                    self.offer(throw, true, now);
                } else {
                    let marked = self.lots.iter().filter(|&&m| m).count() as u32;
                    let throw = self.rules.lots.throw(marked);
                    self.offer(throw, false, now);
                }
            }
            Phase::Daring { sheep, until } => {
                self.ox = Some(self.rng.coin());
                if now < until {
                    return;
                }
                let who = self.to_move().name();
                if self.ox == Some(true) {
                    let doubled = Lots::doubled(sheep);
                    self.message = format!("The ox bone says yes: {sheep} becomes {doubled}.");
                    let throw = Throw { moves: doubled, again: false };
                    self.offer(throw, false, now);
                    if !self.human_to_move() || matches!(self.phase, Phase::NoMove { .. }) {
                        self.message = format!("The ox bone says yes: {who} moves {doubled}.");
                    }
                } else {
                    self.message = format!("The ox bone says no: {who}'s throw is wasted.");
                    self.shown = Throw { moves: 0, again: false };
                    self.phase = Phase::NoMove { throw: self.shown, until: now + PAUSE };
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

fn no_move(who: &str, roll: u8, again: bool) -> String {
    match (roll, again) {
        (0, false) => format!("{who} threw nothing. The turn passes."),
        (_, false) => format!("{who} threw {roll} but cannot move."),
        (_, true) => format!("{who} threw {roll} but cannot move, and throws again."),
    }
}

fn describe(who: &str, mv: Move, out: Outcome, throw_again: bool) -> String {
    if out.won {
        return format!("{who} comes home, and that is the last: the game is won!");
    }
    let mut text = if out.home {
        format!("{who} comes home.")
    } else if mv.from == 0 {
        format!("{who} comes on.")
    } else {
        let n = mv.to - mv.from;
        format!("{who} moves {n} square{}.", if n == 1 { "" } else { "s" })
    };
    if out.took {
        text.push_str(" A piece is taken and sent back!");
    }
    if out.again {
        text.push_str(" A rosette: throw again.");
    } else if out.rosette {
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
                let mut game = Game::new(rules.clone(), players, Rng::new(seed * 7 + i as u64), [40; 2]);
                let mut now = Instant::now();
                let mut steps = 0;
                while !matches!(game.phase, Phase::Over { .. }) {
                    now += Duration::from_millis(100);
                    game.tick(now);
                    steps += 1;
                    assert!(steps < 200_000, "{} seed {seed}: no end", rules.id);
                }
                let Phase::Over { winner } = game.phase else { unreachable!() };
                assert_eq!(game.position.winner(&game.rules), Some(winner));
                assert!(game.message.contains("won"));
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
                let mut game = Game::new(rules.clone(), players, Rng::new(seed + 11), [40; 2]);
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
