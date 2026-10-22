//! The game around the rules: Lane's setting out of the pebbles, choosing
//! a pit, the computer's moves (thought in the background), the sowing
//! shown seed by seed, and the rounds or boards that make up a match.

use crate::ai;
use crate::rules::{Events, Match, Move, Position, Rules, Setup, Side, Slot, Step, apply};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::{debug, info};
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How the game stands.
pub enum Phase {
    /// Lane: `by` sets out the 72 pebbles (a person by hand or at random).
    Arrange { by: Side },
    /// Lane: `by` may turn the board round, giving up the first move.
    TurnBoard { by: Side },
    /// The side to move is choosing (a person) or about to think.
    Turn,
    /// The computer is thinking; its move is shown no sooner than `until`.
    Thinking { handle: JoinHandle<Option<Move>>, until: Instant },
    /// A move being shown: the steps not yet shown, and when the next is.
    Sowing { steps: Vec<Step>, next: usize, at: Instant },
    /// A round or board is over; Enter (or time, for the computer) starts
    /// the next.
    RoundOver { next: Position, starter: Side, by: Option<Side>, until: Option<Instant> },
    /// Over: the winner, or `None` for a draw.
    Over { winner: Option<Side> },
}

pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    /// The position as the rules have it (already after a move being shown).
    pub position: Position,
    /// The position drawn, the seeds in the hand and where the hand is.
    pub shown: Position,
    pub hand: u16,
    pub hand_at: Option<Slot>,
    pub phase: Phase,
    /// The pit the person's cursor is on.
    pub cursor: u8,
    /// Pits whose seeds were just taken, and until when they show it.
    pub flash: Vec<(u8, Instant)>,
    pub last: Option<Move>,
    /// Points won over the boards so far (points matches), or rounds won.
    pub totals: [u32; 2],
    pub round: u32,
    /// Who began the round being played.
    pub starter: Side,
    /// How often each position has occurred this round, for the endless
    /// cycle.
    pub seen: HashMap<Position, u32>,
    pub message: String,
    /// The moves before the last, newest first, for the panel.
    pub earlier: Vec<String>,
    /// Lane, setting out by hand: the pebbles still to place.
    pub to_place: u16,
    pub rng: Rng,
}

/// How long the computer's move waits at least, so it can be followed.
const SHOW: Duration = Duration::from_millis(450);

/// How long a computer-only round end stays on the screen.
const BETWEEN: Duration = Duration::from_secs(4);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let lane = matches!(rules.setup, Setup::Lane { .. });
        // Lane: who sets out the pebbles first is decided by lot (our
        // choice); the other then starts, or turns the board.
        let phase = if lane { Phase::Arrange { by: if rng.coin() { Side::South } else { Side::North } } } else { Phase::Turn };
        let human = Side::BOTH.into_iter().find(|s| players[s.index()] == Player::Human).unwrap_or(Side::South);
        let mut game = Game {
            cursor: position.row(human).start,
            shown: position.clone(),
            position,
            rules,
            players,
            hand: 0,
            hand_at: None,
            phase,
            flash: Vec::new(),
            last: None,
            totals: [0, 0],
            round: 1,
            starter: Side::South,
            seen: HashMap::new(),
            message: String::new(),
            earlier: Vec::new(),
            to_place: 0,
            rng,
        };
        game.begin_round();
        game
    }

    /// The message and set-up at the start of a round.
    fn begin_round(&mut self) {
        self.earlier.clear();
        self.seen.clear();
        self.seen.insert(self.position.clone(), 1);
        self.shown = self.position.clone();
        match self.phase {
            Phase::Arrange { by } => {
                self.position.pits.iter_mut().for_each(|p| *p = 0);
                self.shown = self.position.clone();
                self.to_place = self.rules.seeds();
                self.message = format!("{} sets out the {} pebbles.", by.name(), self.to_place);
                self.cursor = self.position.row(by).start;
            }
            _ => {
                let me = self.position.to_move;
                self.message = format!("{} begins.", me.name());
                self.cursor = self.first_pit(me);
            }
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The pits the side to move may lift.
    pub fn legal(&self) -> Vec<u8> {
        if !matches!(self.phase, Phase::Turn) {
            return Vec::new();
        }
        self.position.moves(&self.rules).iter().map(|m| m.pit).collect()
    }

    fn first_pit(&self, side: Side) -> u8 {
        let legal = self.position.moves(&self.rules);
        legal.first().map_or(self.position.row(side).start, |m| m.pit)
    }

    /// Moves the cursor one pit along its row, towards the higher pit
    /// numbers or the lower (the drawing decides which is Left).
    pub fn step_cursor(&mut self, towards_higher: bool) {
        let side = match self.phase {
            Phase::Arrange { .. } => self.position.owner(self.cursor),
            _ => self.position.to_move,
        };
        let row = self.position.row(side);
        if !row.contains(&self.cursor) {
            self.cursor = row.start;
            return;
        }
        if towards_higher && self.cursor + 1 < row.end {
            self.cursor += 1;
        } else if !towards_higher && self.cursor > row.start {
            self.cursor -= 1;
        }
    }

    /// Lane, setting out by hand in the game of the ignorant: the cursor to
    /// the pit facing it, on the other row.
    pub fn cross_cursor(&mut self) {
        if matches!(self.phase, Phase::Arrange { .. }) && matches!(self.rules.setup, Setup::Lane { wise: false }) {
            self.cursor = self.position.opposite(self.cursor);
        }
    }

    /// The person chooses pit `p`: sows it if it is a legal move.
    pub fn choose(&mut self, p: u8) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        if self.legal().contains(&p) {
            self.play(Move { pit: p });
        } else if self.position.owner(p) != self.position.to_move {
            self.message = "That pit is not yours.".to_string();
        } else if self.position.pits[p as usize] == 0 {
            self.message = "That pit is empty.".to_string();
        } else if self.rules.from_last {
            self.message = "In this game you always play from your last loaded pit.".to_string();
        } else {
            self.message = "The other side has nothing: you must give it seeds.".to_string();
        }
    }

    /// Plays a move and starts showing it.
    fn play(&mut self, mv: Move) {
        let side = self.position.to_move;
        let before = self.position.clone();
        let trace = self.position.play(&self.rules, mv);
        info!("{} sows {} ({} seeds)", side.name(), before.name(mv.pit), before.pits[mv.pit as usize]);
        debug!("trace {:?}", trace.steps);
        self.last = Some(mv);
        if !self.message.is_empty() {
            self.earlier.insert(0, std::mem::take(&mut self.message));
            self.earlier.truncate(2);
        }
        self.message = describe(&self.rules, &before, side, mv, &trace.events);
        let n = self.seen.entry(self.position.clone()).or_insert(0);
        *n += 1;
        let mut steps = trace.steps;
        if *n >= 3 && !self.position.over {
            // Seeds only going round: each side takes its own.
            let share = self.position.share();
            steps.extend(share.steps);
            self.message.push_str(" The same position a third time: each side takes its own seeds.");
        }
        self.show(before, steps);
    }

    /// Starts showing `steps` from `before`.
    fn show(&mut self, before: Position, steps: Vec<Step>) {
        self.shown = before;
        self.hand = 0;
        self.hand_at = None;
        self.phase = Phase::Sowing { steps, next: 0, at: Instant::now() };
    }

    /// Shows the rest of a move at once (a key or a click).
    pub fn hurry(&mut self) {
        if let Phase::Sowing { steps, next, .. } = &mut self.phase {
            for s in &steps[*next..] {
                apply(&mut self.shown, &mut self.hand, *s);
            }
            *next = steps.len();
        }
    }

}

/// How long a step stays before the next, in a move of `seeds` drops.
fn pause(step: &Step, seeds: usize) -> Duration {
        let drop = if seeds > 24 { 35 } else if seeds > 12 { 60 } else { 110 };
        Duration::from_millis(match step {
            Step::Lift { .. } => 260,
            Step::Drop { .. } => drop,
            Step::Take { .. } | Step::Give { .. } | Step::Tuzdyk { .. } => 380,
            Step::Pass => 0,
            Step::Restock { .. } => 900,
        })
}

impl Game {
    /// The end of a move: the next turn, or the end of a round.
    fn moved(&mut self, now: Instant) {
        self.shown = self.position.clone();
        self.hand = 0;
        self.hand_at = None;
        if self.position.over {
            self.round_over(now);
            return;
        }
        self.phase = Phase::Turn;
        let me = self.position.to_move;
        if self.player(me) == Player::Human {
            self.cursor = self.first_pit(me);
        }
    }

    /// Scores a finished round and sets up the next, or ends the match.
    fn round_over(&mut self, now: Instant) {
        let pos = &self.position;
        let leader = pos.leader();
        let [s, n] = pos.stores;
        let score = format!("South {s}, North {n}.");
        let all_computer = self.players.iter().all(|p| *p != Player::Human);
        let until = all_computer.then(|| now + BETWEEN);
        match self.rules.rounds {
            Match::Single { .. } => {
                self.message.push_str(&format!(" Game over: {score} {}", winner_text(leader)));
                self.phase = Phase::Over { winner: leader };
            }
            Match::Points { target, excess } => {
                let half = self.rules.seeds() / 2;
                if let Some(w) = leader {
                    let mine = pos.stores[w.index()];
                    let theirs = pos.stores[w.other().index()];
                    let points = if excess { mine - theirs } else { mine - half };
                    self.totals[w.index()] += points as u32;
                }
                let [ts, tn] = self.totals;
                let board = format!("Board {} over: {score} Points: South {ts}, North {tn} (to {target}).", self.round);
                self.message = board;
                if self.totals.iter().any(|&t| t >= target as u32) {
                    let w = if ts > tn { Side::South } else { Side::North };
                    self.message.push_str(&format!(" {} wins the match!", w.name()));
                    self.phase = Phase::Over { winner: Some(w) };
                    return;
                }
                // Lane: the gainer begins the next board, the other having set
                // it out; Champion: the first move alternates.
                let starter = match (self.rules.setup, leader) {
                    (Setup::Lane { .. }, Some(w)) => w,
                    _ => self.starter.other(),
                };
                let by = matches!(self.rules.setup, Setup::Lane { .. }).then(|| starter.other());
                let mut next = Position::new(&self.rules);
                next.to_move = starter;
                self.phase = Phase::RoundOver { next, starter, by, until };
            }
            Match::Refill { loser_starts, .. } => {
                if let Some(w) = leader {
                    self.totals[w.index()] += 1;
                }
                let starter = match (loser_starts, leader) {
                    (true, Some(w)) => w.other(),
                    _ => self.starter.other(),
                };
                match pos.next_round(&self.rules, starter) {
                    Some(next) => {
                        let closed = |side: Side| next.row(side).filter(|&p| next.closed[p as usize]).count();
                        self.message = format!(
                            "Round {} over: {score} Next round: South plays with {} pits, North with {}.",
                            self.round,
                            self.rules.pits - closed(Side::South),
                            self.rules.pits - closed(Side::North)
                        );
                        self.phase = Phase::RoundOver { next, starter, by: None, until };
                    }
                    None => {
                        let w = if s >= n { Side::South } else { Side::North };
                        self.message = format!(
                            "Round {} over: {score} {} cannot fill a single pit: {} wins the game!",
                            self.round,
                            w.other().name(),
                            w.name()
                        );
                        self.phase = Phase::Over { winner: Some(w) };
                    }
                }
            }
        }
    }

    /// Starts the next round (Enter, or time for the computer).
    pub fn next_round(&mut self) {
        let Phase::RoundOver { next, starter, by, .. } = std::mem::replace(&mut self.phase, Phase::Turn) else {
            return;
        };
        self.round += 1;
        self.position = next;
        self.starter = starter;
        if let Some(by) = by {
            self.phase = Phase::Arrange { by };
        }
        self.begin_round();
    }

    /// Lane, by hand: one pebble more (or fewer) in the pit under the
    /// cursor; in the game of the wise, in the matching pit on the other
    /// side too.
    pub fn place(&mut self, more: bool) {
        let Phase::Arrange { .. } = self.phase else { return };
        let wise = matches!(self.rules.setup, Setup::Lane { wise: true });
        let n = self.position.len() / 2;
        let pits: Vec<usize> = if wise {
            let k = self.cursor as usize % n;
            vec![k, k + n]
        } else {
            vec![self.cursor as usize]
        };
        for p in pits {
            if more && self.to_place > 0 {
                self.position.pits[p] += 1;
                self.to_place -= 1;
            } else if !more && self.position.pits[p] > 0 {
                self.position.pits[p] -= 1;
                self.to_place += 1;
            }
        }
        self.shown = self.position.clone();
    }

    /// Lane: the rest of the pebbles set out at random, uncounted.
    pub fn scatter(&mut self) {
        let Phase::Arrange { .. } = self.phase else { return };
        let wise = matches!(self.rules.setup, Setup::Lane { wise: true });
        let placed = self.rules.seeds() - self.to_place;
        if placed == 0 {
            self.position.pits = lane_setout(&mut self.rng, wise, self.rules.pits);
        } else {
            let n = self.position.len();
            while self.to_place > 0 {
                let p = (self.rng.next() % n as u64) as usize;
                self.position.pits[p] += 1;
                self.to_place -= 1;
            }
        }
        self.to_place = 0;
        self.shown = self.position.clone();
    }

    /// Lane: the pebbles are set out; the other side may now turn the board.
    pub fn arranged(&mut self) {
        let Phase::Arrange { by } = self.phase else { return };
        if self.to_place > 0 {
            self.message = format!("{} pebbles are still in the hand (R sets them out at random).", self.to_place);
            return;
        }
        let other = by.other();
        self.position.to_move = other;
        self.starter = other;
        self.seen.clear();
        self.shown = self.position.clone();
        self.message = format!("{} may turn the board round, and give up the first move.", other.name());
        self.phase = Phase::TurnBoard { by: other };
    }

    /// Lane: the board turned round (or not); the game begins.
    pub fn turn_board(&mut self, turn: bool) {
        let Phase::TurnBoard { by } = self.phase else { return };
        if turn {
            self.position = self.position.turned();
            self.position.to_move = by.other();
            self.starter = by.other();
            self.message = format!("{} turns the board round: {} begins.", by.name(), by.other().name());
        } else {
            self.message = format!("{} keeps the board as it is, and begins.", by.name());
        }
        info!("lane set-out {:?}, turned {turn}", self.position.pits);
        self.seen.clear();
        self.seen.insert(self.position.clone(), 1);
        self.shown = self.position.clone();
        self.phase = Phase::Turn;
        let me = self.position.to_move;
        self.cursor = self.first_pit(me);
    }

    /// Whether the computer is still working out its move.
    pub fn thinking(&self) -> bool {
        matches!(&self.phase, Phase::Thinking { handle, .. } if !handle.is_finished())
    }

    pub fn tick(&mut self, now: Instant) {
        self.flash.retain(|&(_, until)| until > now);
        match &mut self.phase {
            Phase::Arrange { by } => {
                let by = *by;
                if let Player::Computer(_) = self.player(by) {
                    self.scatter();
                    self.arranged();
                }
            }
            Phase::TurnBoard { by } => {
                let by = *by;
                if let Player::Computer(strength) = self.player(by) {
                    // Turn the board if starting second on it looks better
                    // than starting first on this one.
                    let depth = if strength == ai::Strength::Easy { 1 } else { 3 };
                    let keep = ai::value(&self.rules, &self.position, depth);
                    let mut turned = self.position.turned();
                    turned.to_move = by.other();
                    let turn = -ai::value(&self.rules, &turned, depth) > keep;
                    self.turn_board(turn);
                }
            }
            Phase::Turn => {
                let side = self.position.to_move;
                if let Player::Computer(strength) = self.player(side) {
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
                        if let Phase::Sowing { at, .. } = &mut self.phase {
                            *at = now;
                        }
                    }
                }
            }
            Phase::Sowing { steps, next, at } => {
                let seeds = steps.iter().filter(|s| matches!(s, Step::Drop { .. })).count();
                while *next < steps.len() && now >= *at {
                    let step = steps[*next];
                    apply(&mut self.shown, &mut self.hand, step);
                    match step {
                        Step::Lift { pit } => self.hand_at = Some(Slot::Pit(pit)),
                        Step::Drop { slot } => self.hand_at = Some(slot),
                        Step::Take { pit, .. } | Step::Tuzdyk { pit, .. } => {
                            self.flash.push((pit, now + Duration::from_millis(900)))
                        }
                        _ => {}
                    }
                    *at += pause(&step, seeds);
                    *next += 1;
                }
                if *next >= steps.len() && now >= *at {
                    self.moved(now);
                }
            }
            Phase::RoundOver { until, .. } => {
                if until.is_some_and(|u| now >= u) {
                    self.next_round();
                }
            }
            Phase::Over { .. } => {}
        }
    }
}

/// Lane's ways of setting out the pebbles, uncounted. The game of the
/// ignorant: unevenly in every pit, "generally putting at least four into
/// each". The game of the wise: in matching pits on both sides, commonly
/// the four middle pits of each side, or about half in each side's first
/// pit.
pub fn lane_setout(rng: &mut Rng, wise: bool, pits: usize) -> Vec<u8> {
    let n = pits;
    let mut out = vec![0u8; 2 * n];
    let drop = |out: &mut Vec<u8>, targets: &[usize], seeds: u16, rng: &mut Rng| {
        for _ in 0..seeds {
            let p = targets[(rng.next() % targets.len() as u64) as usize];
            out[p] += 1;
        }
    };
    if !wise {
        for p in out.iter_mut() {
            *p = 4;
        }
        let all: Vec<usize> = (0..2 * n).collect();
        drop(&mut out, &all, 72 - 8 * n as u16, rng);
        return out;
    }
    if rng.coin() {
        // Four middle pits on each side.
        let south: Vec<usize> = (1..n - 1).collect();
        let north: Vec<usize> = (n + 1..2 * n - 1).collect();
        let half = 32 + (rng.next() % 9) as u16;
        drop(&mut out, &south, half, rng);
        drop(&mut out, &north, 72 - half, rng);
    } else {
        // About half in A and the rest in a.
        let half = 32 + (rng.next() % 9) as u8;
        out[0] = half;
        out[n] = 72 - half;
    }
    out
}

fn winner_text(leader: Option<Side>) -> String {
    match leader {
        Some(w) => format!("{} wins!", w.name()),
        None => "A draw.".to_string(),
    }
}

/// What a move did, in words.
pub fn describe(rules: &Rules, before: &Position, side: Side, mv: Move, ev: &Events) -> String {
    let seeds = before.pits[mv.pit as usize];
    let mut s = format!(
        "{} sows {} from {}.",
        side.name(),
        plural(seeds as u16, "seed"),
        before.name(mv.pit)
    );
    if ev.grand_slam {
        s.push_str(" That would take every seed on the other side, so it takes none.");
    }
    if let Some(t) = ev.tuzdyk {
        s.push_str(&format!(" {} becomes {}'s tuzdyk!", before.name(t), side.name()));
    } else if ev.captured > 0 {
        s.push_str(&format!(" Captures {}!", plural(ev.captured, "seed")));
    }
    if ev.cows > 0 {
        s.push_str(&format!(" {} go to their owners as cows.", plural(ev.cows, "seed")));
    }
    if ev.gave {
        s.push_str(" The other side had none, so it is given one.");
    }
    if ev.restocked {
        s.push_str(" The other side's row was empty: it is filled again from its store.");
    }
    if ev.again {
        s.push_str(" The last seed fell in the store: another move.");
    }
    let _ = rules;
    s
}

fn plural(n: u16, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ancient_core::player::Strength;

    #[test]
    fn computer_games_end() {
        for rules in Rules::all() {
            let players = [Player::Computer(Strength::Easy), Player::Computer(Strength::Easy)];
            let mut game = Game::new(rules.clone(), players, Rng::new(3));
            let mut now = Instant::now();
            for _ in 0..400_000 {
                if matches!(game.phase, Phase::Over { .. }) {
                    break;
                }
                now += Duration::from_millis(200);
                game.tick(now);
                if game.thinking() {
                    std::thread::sleep(Duration::from_micros(200));
                }
            }
            assert!(matches!(game.phase, Phase::Over { .. }), "{} did not end: {}", rules.id, game.message);
        }
    }
}
