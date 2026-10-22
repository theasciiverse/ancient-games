//! The rules of the mancala games the program offers, as data, and one
//! engine that plays them all.
//!
//! Mancala is not one game but a family: seeds are lifted from a pit and
//! sown one by one into the pits that follow, and the regions differ in
//! almost everything after that: how many pits and seeds, whether the
//! stores are sown into, whether the last seed lifts its pit and goes on,
//! what it captures, and what happens when a side runs dry. Each rule set
//! here follows its source; where the source is silent, the rulebook
//! chapter says what this game does ("Our choice").
//!
//! The board is two rows of `pits` pits, numbered in the direction of
//! sowing: South's pits are 0 to n-1 from South's left, North's are n to
//! 2n-1 from North's left, so pit i is followed by pit i+1 and the last
//! by the first. The pits facing each other are i and 2n-1-i. South's
//! store sits after South's last pit, North's after North's last. A game
//! that sows clockwise is the same game seen in a mirror, so it is played
//! the same way here and only drawn mirrored.
//!
//! Every change to the board goes through [`act`], which also records it
//! as a [`Step`]: the animation, the rulebook's examples and the tests
//! replay those steps, so they can never show something the rules did not
//! do.

use std::collections::HashSet;

/// The two sides. South moves first unless a rule set says otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    South,
    North,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::South => 0,
            Side::North => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::South => Side::North,
            Side::North => Side::South,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::South => "South",
            Side::North => "North",
        }
    }

    pub const BOTH: [Side; 2] = [Side::South, Side::North];
}

/// How the seeds are set out at the start of a game or a round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    /// The same number in every pit.
    Uniform(u8),
    /// Lane's Egyptian mankal'ah: 72 pebbles distributed by one player
    /// without counting (the "tebweez"), after which the other may turn
    /// the board round. `wise`: the "game of the wise", where they go in
    /// matching pits on both sides.
    Lane { wise: bool },
}

/// What happens to the pit a sowing starts from when the sowing comes
/// round to it again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// It is sown into like any other.
    Sown,
    /// It is passed over (Oware, Wari: a lap of twelve or more).
    Skipped,
    /// The first seed goes back into it, unless it held only one (Toguz
    /// Kumalak).
    FirstBack,
}

/// What the last seed of a lap does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laps {
    /// The sowing ends with the last seed.
    Single,
    /// A last seed in a loaded pit lifts that pit and sows on (Congkak).
    Relay,
    /// Lane: a last seed making three, five or more lifts the pit and
    /// sows on; one ends the turn; two or four capture.
    Lane,
    /// The Indian lap (Pallanguli): the pit after the last seed is lifted
    /// and sown; an empty pit there captures the pit beyond it and ends the
    /// move; two empty pits end it with nothing.
    Indian,
}

/// What the last seed captures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capture {
    /// The last seed makes a pit one of `counts` (on the opponent's side
    /// only, if `opponent_only`): it is taken, and so is each pit before
    /// it holding one of `counts`, back to the first that does not.
    /// `with_opposite`: each such pit is taken with the pit facing it
    /// (Lane).
    Counts { counts: &'static [u8], opponent_only: bool, with_opposite: bool },
    /// The last seed makes an opponent's pit even: it is taken (Toguz
    /// Kumalak).
    Even,
    /// The last seed falls in an empty pit of the mover's own: it is taken
    /// with the pit facing it. `with_seed`: the capturing seed goes to the
    /// store too (Kalah); otherwise only the facing pit is taken (Hellier's
    /// Congkak). `alone`: the seed is taken even when the
    /// facing pit is empty (Congkak); otherwise nothing happens then.
    EmptyOwn { with_seed: bool, alone: bool },
    /// The Indian lap does its own capturing.
    InLap,
}

/// What a capture of every seed on the opponent's side does (Oware's
/// "grand slam").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrandSlam {
    Allowed,
    /// The move is played but captures nothing (Oware Abapa).
    NoCapture,
    /// The move may not be played (Bell's Wari), unless every move the
    /// player has would do it (our choice; as Romein and Bal play Awari).
    Illegal,
}

/// What happens when a side has no seeds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    /// A player must give seeds to an opponent who has none; if no move
    /// does, the player takes what is left and the game ends (Oware, Wari).
    Feed,
    /// When either side is empty after a move, each side's seeds go to its
    /// owner and the game ends (Kalah).
    Sweep,
    /// A player with no move ends the round; the seeds left go to the
    /// opponent.
    OpponentTakes,
    /// Lane: a player with seeds when the other has none puts one into the
    /// other's first pit; a single seed left belongs to its side.
    LaneGive,
    /// Hellier's Congkak: a player whose row is empty fills it again from
    /// her own store, `per_pit` to a pit as far as the seeds go; the pits
    /// left empty are closed. A player who cannot fill one pit has lost.
    Restock { per_pit: u8 },
}

/// How a game is won: one game, or rounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Match {
    /// One game: the most seeds wins; reaching `majority` ends it at once.
    Single { majority: Option<u16> },
    /// Games are scored and added up until a player reaches `target`.
    /// `excess`: the score is the difference between the two stores
    /// (Lane); otherwise it is the seeds won beyond those a side began
    /// with (Champion's Kalah).
    Points { target: u16, excess: bool },
    /// Rounds: each player refills their pits from their store with
    /// `per_pit` seeds each, as many as they can; the rest stay closed for
    /// the round. A player who cannot fill one pit has lost. `loser_starts`:
    /// the loser of a round starts the next; otherwise the first move
    /// alternates.
    Refill { per_pit: u8, loser_starts: bool },
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where it comes from, for the menu.
    pub source: &'static str,
    /// The heading of this rule set in the rulebook chapter (RULES.txt),
    /// where the help opens.
    pub section: &'static str,
    /// Pits on each side.
    pub pits: usize,
    pub setup: Setup,
    /// Whether the sowing drops seeds into the mover's own store (never the
    /// opponent's).
    pub stores_sown: bool,
    /// Whether a last seed in the mover's own store gives another move.
    pub store_again: bool,
    pub origin: Origin,
    pub laps: Laps,
    pub capture: Capture,
    /// A pit that a seed brings to this count goes at once to its owner
    /// (Pallanguli's "cow").
    pub cow: Option<u8>,
    /// Toguz Kumalak's tuzdyk.
    pub tuzdyk: bool,
    pub grand_slam: GrandSlam,
    pub ending: Ending,
    /// Lane's game of the ignorant: a player always plays from his last pit,
    /// or the loaded pit nearest it.
    pub from_last: bool,
    /// Whether the game sows clockwise (drawn mirrored).
    pub clockwise: bool,
    pub rounds: Match,
}

impl Rules {
    fn base(id: &'static str, name: &'static str, section: &'static str, pits: usize, seeds: u8) -> Rules {
        Rules {
            id,
            name,
            source: "",
            section,
            pits,
            setup: Setup::Uniform(seeds),
            stores_sown: false,
            store_again: false,
            origin: Origin::Sown,
            laps: Laps::Single,
            capture: Capture::Counts { counts: &[2, 3], opponent_only: true, with_opposite: false },
            cow: None,
            tuzdyk: false,
            grand_slam: GrandSlam::Allowed,
            ending: Ending::OpponentTakes,
            from_last: false,
            clockwise: false,
            rounds: Match::Single { majority: None },
        }
    }

    /// Oware as played in tournaments: the Abapa rules.
    pub fn oware() -> Rules {
        Rules {
            source: "Oware Abapa, the Ghanaian game as played in tournaments \
                     today: a capture that would take all the opponent's seeds \
                     takes none, and a player must feed an opponent who has \
                     nothing. 25 seeds win.",
            origin: Origin::Skipped,
            grand_slam: GrandSlam::NoCapture,
            ending: Ending::Feed,
            rounds: Match::Single { majority: Some(25) },
            ..Rules::base("oware", "Oware (Abapa)", "Oware, Abapa", 6, 4)
        }
    }

    /// Wari as R. C. Bell gives it (his board drawn from Rattray's Religion
    /// and Art in Ashanti, 1927, whose Wari chapter is G. T. Bennett's).
    pub fn wari() -> Rules {
        Rules {
            source: "Wari, the Ashanti game, as R. C. Bell gives it in Board and \
                     Table Games from Many Civilizations (1960): a move that would \
                     take every seed on the other side may not be played, and seeds \
                     that only go round are shared.",
            origin: Origin::Skipped,
            grand_slam: GrandSlam::Illegal,
            ending: Ending::Feed,
            ..Rules::base("wari", "Wari (Bell 1960)", "Wari, after Bell", 6, 4)
        }
    }

    /// Lane's Egyptian mankal'ah, "the game of the ignorant".
    pub fn lane_ignorant() -> Rules {
        Rules {
            source: "Mankal'ah as E. W. Lane saw it in the coffee shops of Cairo \
                     (Manners and Customs of the Modern Egyptians, 1836), in the \
                     simple form he calls the game of the ignorant: 72 pebbles \
                     set out without counting, and play always from the last pit.",
            setup: Setup::Lane { wise: false },
            laps: Laps::Lane,
            capture: Capture::Counts { counts: &[2, 4], opponent_only: false, with_opposite: true },
            ending: Ending::LaneGive,
            from_last: true,
            rounds: Match::Points { target: 60, excess: true },
            ..Rules::base("mankalah-ignorant", "Mankal'ah, the game of the ignorant (Lane)", "Mankal'ah, the game of the ignorant", 6, 0)
        }
    }

    /// Lane's Egyptian mankal'ah, "the game of the wise".
    pub fn lane_wise() -> Rules {
        Rules {
            source: "Mankal'ah as E. W. Lane saw it in Cairo (1836), in the form \
                     he calls the game of the wise: the pebbles set out in \
                     matching pits on both sides, and play from any pit.",
            setup: Setup::Lane { wise: true },
            from_last: false,
            ..Rules::lane_ignorant()
        }
        .renamed("mankalah-wise", "Mankal'ah, the game of the wise (Lane)", "Mankal'ah, the game of the wise")
    }

    /// Kalah, W. J. Champion's game.
    pub fn kalah() -> Rules {
        Rules {
            source: "Kalah, the American game W. J. Champion patented and sold \
                     from the 1950s (US patent 2,720,362): the stores are sown \
                     into, the last seed in your own store plays again, and games \
                     are scored until one player has 40 points. A modern game, but \
                     the one most people know as Mancala.",
            stores_sown: true,
            store_again: true,
            capture: Capture::EmptyOwn { with_seed: true, alone: false },
            ending: Ending::Sweep,
            rounds: Match::Points { target: 40, excess: false },
            ..Rules::base("kalah", "Kalah (Champion 1955)", "Kalah", 6, 4)
        }
    }

    /// Congkak, the Malay game, as M. Hellier described it in 1907.
    pub fn congkak() -> Rules {
        Rules {
            source: "Congkak (Jongkak), the Malay game, as M. Hellier described it \
                     in 1907: seven holes of seven seeds, the stores sown into, \
                     sowing that goes on from every loaded hole, and a player whose \
                     row runs dry fills it again from her store, leaving ruined \
                     wells, until she cannot.",
            pits: 7,
            stores_sown: true,
            store_again: true,
            laps: Laps::Relay,
            capture: Capture::EmptyOwn { with_seed: false, alone: false },
            ending: Ending::Restock { per_pit: 7 },
            clockwise: true,
            ..Rules::base("congkak", "Congkak (Hellier 1907)", "Congkak, after Hellier", 7, 7)
        }
    }

    /// Pallanguli as Mrs H. G. Durai described it in 1928.
    pub fn pallanguli() -> Rules {
        Rules {
            source: "Pallanguli, the Tamil game, as Mrs H. G. Durai described it \
                     in Man (1928), the source of R. C. Bell's account: the Indian \
                     lap, a capture beyond an empty hole, cows of four, and rounds \
                     with rubbish holes.",
            laps: Laps::Indian,
            capture: Capture::InLap,
            cow: Some(4),
            rounds: Match::Refill { per_pit: 6, loser_starts: false },
            ..Rules::base("pallanguli", "Pallanguli (Durai 1928)", "Pallanguli, after Durai", 7, 6)
        }
    }

    /// Toguz Kumalak, the Kazakh and Kyrgyz game.
    pub fn toguz() -> Rules {
        Rules {
            source: "Toguz Kumalak (Kazakh) or Toguz Korgool (Kyrgyz), nine \
                     holes of nine stones, as played in competition: captures on \
                     even numbers, and one tuzdyk each, a hole of the opponent's \
                     that becomes yours. 82 stones win.",
            origin: Origin::FirstBack,
            capture: Capture::Even,
            tuzdyk: true,
            rounds: Match::Single { majority: Some(82) },
            ..Rules::base("toguz", "Toguz Kumalak", "Toguz Kumalak", 9, 9)
        }
    }

    /// Bohnenspiel, the German bean game.
    pub fn bohnenspiel() -> Rules {
        Rules {
            source: "Bohnenspiel, the bean game Fritz Jahn found in the Baltic \
                     provinces in 1908 and described in Old German Games (1917): six \
                     beans a pit, and captures of two, four or six on either side, \
                     back along the row.",
            capture: Capture::Counts { counts: &[2, 4, 6], opponent_only: false, with_opposite: false },
            ..Rules::base("bohnenspiel", "Bohnenspiel", "Bohnenspiel", 6, 6)
        }
    }

    fn renamed(mut self, id: &'static str, name: &'static str, section: &'static str) -> Rules {
        self.id = id;
        self.name = name;
        self.section = section;
        self
    }

    pub fn all() -> Vec<Rules> {
        vec![
            Rules::oware(),
            Rules::wari(),
            Rules::lane_ignorant(),
            Rules::lane_wise(),
            Rules::kalah(),
            Rules::congkak(),
            Rules::pallanguli(),
            Rules::toguz(),
            Rules::bohnenspiel(),
        ]
    }

    /// Seeds in the game.
    pub fn seeds(&self) -> u16 {
        match self.setup {
            Setup::Uniform(n) => n as u16 * 2 * self.pits as u16,
            Setup::Lane { .. } => 72,
        }
    }
}

/// A place a seed can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    Pit(u8),
    Store(Side),
}

/// Why seeds went to a store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Capture,
    /// Pallanguli's cow, to the pit's owner whoever sowed.
    Cow,
    /// The end of a game or round: what is left goes to a store.
    Left,
}

/// One change to the board. Replaying the steps of a move with [`apply`]
/// gives the position after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// The seeds of a pit go into the hand.
    Lift { pit: u8 },
    /// One seed from the hand into a slot (into the owner's store if the
    /// pit is a tuzdyk).
    Drop { slot: Slot },
    /// The seeds of a pit go to a store.
    Take { pit: u8, to: Side, why: Why },
    /// One seed moves from one pit to another (Lane's rule for a side with
    /// none).
    Give { from: u8, to: u8 },
    /// A pit becomes the tuzdyk of `owner`.
    Tuzdyk { pit: u8, owner: Side },
    /// The turn passes to the other side.
    Pass,
    /// Hellier's Congkak: `side` fills her empty row from her store,
    /// `per_pit` to a pit from her first pit on, and closes the rest.
    Restock { side: Side, per_pit: u8 },
}

/// What happened in a move, for messages.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Events {
    /// Seeds captured by the mover.
    pub captured: u16,
    /// Seeds that went to their owners as cows.
    pub cows: u16,
    /// A capture that would have emptied the other side and so took nothing.
    pub grand_slam: bool,
    /// A capture that took every seed on the other side.
    pub emptied: bool,
    pub tuzdyk: Option<u8>,
    /// The mover plays again.
    pub again: bool,
    /// Lane: a seed was given to a side that had none.
    pub gave: bool,
    /// Hellier's Congkak: the side to move filled its empty row again.
    pub restocked: bool,
    /// The game or round ended after this move.
    pub over: bool,
}

/// The whole of a move: the steps and what they meant.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Trace {
    pub steps: Vec<Step>,
    pub events: Events,
}

/// A move: the pit to lift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub pit: u8,
}

/// A position within a game or round.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub pits: Vec<u8>,
    pub stores: [u16; 2],
    /// Pits out of play this round (Pallanguli's rubbish holes, Congkak's
    /// burnt pits).
    pub closed: Vec<bool>,
    /// The pit that is each side's tuzdyk (Toguz Kumalak).
    pub tuz: [Option<u8>; 2],
    pub to_move: Side,
    /// The game or round is over.
    pub over: bool,
}

/// Longest a single move may go on, in laps: Indian and relay sowing can in
/// principle go round for ever.
const MAX_LAPS: usize = 500;

/// Records a step and applies it.
fn act(pos: &mut Position, trace: &mut Trace, hand: &mut u16, step: Step) {
    apply(pos, hand, step);
    trace.steps.push(step);
}

/// Applies one step to a position, with the seeds in the hand.
pub fn apply(pos: &mut Position, hand: &mut u16, step: Step) {
    match step {
        Step::Lift { pit } => {
            *hand += pos.pits[pit as usize] as u16;
            pos.pits[pit as usize] = 0;
        }
        Step::Drop { slot } => {
            *hand -= 1;
            match slot {
                Slot::Store(s) => pos.stores[s.index()] += 1,
                Slot::Pit(p) => match pos.tuz_owner(p) {
                    Some(owner) => pos.stores[owner.index()] += 1,
                    None => pos.pits[p as usize] += 1,
                },
            }
        }
        Step::Take { pit, to, .. } => {
            pos.stores[to.index()] += pos.pits[pit as usize] as u16;
            pos.pits[pit as usize] = 0;
        }
        Step::Give { from, to } => {
            pos.pits[from as usize] -= 1;
            pos.pits[to as usize] += 1;
        }
        Step::Tuzdyk { pit, owner } => {
            pos.tuz[owner.index()] = Some(pit);
            pos.stores[owner.index()] += pos.pits[pit as usize] as u16;
            pos.pits[pit as usize] = 0;
        }
        Step::Pass => pos.to_move = pos.to_move.other(),
        Step::Restock { side, per_pit } => {
            for p in pos.row(side) {
                if pos.stores[side.index()] >= per_pit as u16 {
                    pos.stores[side.index()] -= per_pit as u16;
                    pos.pits[p as usize] = per_pit;
                    pos.closed[p as usize] = false;
                } else {
                    pos.closed[p as usize] = true;
                }
            }
        }
    }
}

impl Position {
    /// The position at the start of a game, for a uniform setup (Lane's
    /// tebweez sets the pits afterwards).
    pub fn new(rules: &Rules) -> Position {
        let n = rules.pits * 2;
        let seeds = match rules.setup {
            Setup::Uniform(k) => k,
            Setup::Lane { .. } => 6,
        };
        Position {
            pits: vec![seeds; n],
            stores: [0, 0],
            closed: vec![false; n],
            tuz: [None, None],
            to_move: Side::South,
            over: false,
        }
    }

    pub fn len(&self) -> usize {
        self.pits.len()
    }

    /// Whose row pit `p` is on.
    pub fn owner(&self, p: u8) -> Side {
        if (p as usize) < self.len() / 2 { Side::South } else { Side::North }
    }

    /// The pit facing `p`.
    pub fn opposite(&self, p: u8) -> u8 {
        (self.len() - 1 - p as usize) as u8
    }

    /// The pits of `side`, in the order of sowing.
    pub fn row(&self, side: Side) -> std::ops::Range<u8> {
        let n = (self.len() / 2) as u8;
        match side {
            Side::South => 0..n,
            Side::North => n..2 * n,
        }
    }

    /// The side whose tuzdyk pit `p` is, if any.
    pub fn tuz_owner(&self, p: u8) -> Option<Side> {
        Side::BOTH.into_iter().find(|s| self.tuz[s.index()] == Some(p))
    }

    /// Seeds in the pits of `side`.
    pub fn on_side(&self, side: Side) -> u16 {
        self.row(side).map(|p| self.pits[p as usize] as u16).sum()
    }

    pub fn on_board(&self) -> u16 {
        self.pits.iter().map(|&s| s as u16).sum()
    }

    /// The pit after `p`, passing over closed pits.
    fn next_pit(&self, p: u8) -> u8 {
        let n = self.len() as u8;
        let mut q = p;
        for _ in 0..n {
            q = (q + 1) % n;
            if !self.closed[q as usize] {
                return q;
            }
        }
        p
    }

    /// Where the next seed of a sowing by `mover` goes after `slot`.
    fn next_slot(&self, rules: &Rules, slot: Slot, mover: Side, origin: u8) -> Slot {
        let n = rules.pits as u8;
        let last = |s: Side| match s {
            Side::South => n - 1,
            Side::North => 2 * n - 1,
        };
        let mut s = slot;
        loop {
            s = match s {
                Slot::Pit(p) if rules.stores_sown && p == last(mover) => Slot::Store(mover),
                Slot::Pit(p) => Slot::Pit((p + 1) % (2 * n)),
                Slot::Store(side) => Slot::Pit((last(side) + 1) % (2 * n)),
            };
            match s {
                Slot::Pit(p) if self.closed[p as usize] => continue,
                Slot::Pit(p) if p == origin && rules.origin == Origin::Skipped => continue,
                _ => return s,
            }
        }
    }

    /// The legal moves of the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        if self.over {
            return Vec::new();
        }
        let me = self.to_move;
        let mut moves: Vec<Move> = self
            .row(me)
            .filter(|&p| self.pits[p as usize] > 0 && self.tuz_owner(p).is_none())
            .map(|pit| Move { pit })
            .collect();
        if rules.from_last {
            moves = moves.last().copied().into_iter().collect();
        }
        if rules.grand_slam == GrandSlam::Illegal {
            let clean: Vec<Move> = moves
                .iter()
                .copied()
                .filter(|&m| {
                    let mut trace = Trace::default();
                    self.clone().sow(rules, m, &mut trace);
                    !trace.events.emptied
                })
                .collect();
            if !clean.is_empty() {
                moves = clean;
            }
        }
        if rules.ending == Ending::Feed && self.on_side(me.other()) == 0 {
            moves.retain(|&m| {
                let mut next = self.clone();
                next.sow(rules, m, &mut Trace::default());
                next.on_side(me.other()) > 0
            });
        }
        moves
    }

    /// Plays a move: the sowing, then what the rules do at the end of a
    /// turn, then the other side's turn begins (or the mover's again).
    pub fn play(&mut self, rules: &Rules, mv: Move) -> Trace {
        let mut trace = Trace::default();
        let mover = self.to_move;
        let again = self.sow(rules, mv, &mut trace);
        let mut hand = 0;
        if self.reached_majority(rules) {
            self.over = true;
        } else if rules.ending == Ending::Sweep
            && (self.on_side(Side::South) == 0 || self.on_side(Side::North) == 0)
        {
            self.sweep_own(&mut trace);
            self.over = true;
        } else {
            if again && self.on_side(mover) > 0 {
                trace.events.again = true;
            } else {
                act(self, &mut trace, &mut hand, Step::Pass);
            }
            self.begin_turn(rules, &mut trace);
        }
        if self.on_board() == 0 {
            self.over = true;
        }
        trace.events.over = self.over;
        trace
    }

    /// Whether a side has the majority that ends the game.
    fn reached_majority(&self, rules: &Rules) -> bool {
        match rules.rounds {
            Match::Single { majority: Some(m) } => self.stores.iter().any(|&s| s >= m),
            _ => false,
        }
    }

    /// Each side's seeds go to its owner.
    fn sweep_own(&mut self, trace: &mut Trace) {
        let mut hand = 0;
        for p in 0..self.len() as u8 {
            if self.pits[p as usize] > 0 {
                let to = self.owner(p);
                act(self, trace, &mut hand, Step::Take { pit: p, to, why: Why::Left });
            }
        }
    }

    /// All seeds left go to `side`.
    fn sweep_to(&mut self, side: Side, trace: &mut Trace) {
        let mut hand = 0;
        for p in 0..self.len() as u8 {
            if self.pits[p as usize] > 0 {
                act(self, trace, &mut hand, Step::Take { pit: p, to: side, why: Why::Left });
            }
        }
    }

    /// What happens before the side to move chooses: Lane's gift of a seed,
    /// or the end when the side cannot move.
    fn begin_turn(&mut self, rules: &Rules, trace: &mut Trace) {
        let me = self.to_move;
        let mut hand = 0;
        if rules.ending == Ending::LaneGive {
            let (mine, theirs) = (self.on_side(me), self.on_side(me.other()));
            if mine == 0 && theirs == 1 {
                // "If only one hasweh remain on one side, and none on the
                // other, that one is the property of the person on whose
                // side it is."
                self.sweep_to(me.other(), trace);
            } else if mine == 0 && theirs > 1 {
                // The other side gives one into this side's first pit, from
                // its loaded pit nearest the crossing (our choice).
                let from = self
                    .row(me.other())
                    .rev()
                    .find(|&p| self.pits[p as usize] > 0)
                    .expect("seeds on the other side");
                let to = self.row(me).start;
                act(self, trace, &mut hand, Step::Give { from, to });
                trace.events.gave = true;
            }
            if self.on_board() == 0 {
                self.over = true;
            }
            return;
        }
        if let Ending::Restock { per_pit } = rules.ending
            && self.on_side(me) == 0
            && !self.restock(me, per_pit, trace)
        {
            // She cannot fill a single pit: the game is over, and what is
            // left on the board is the other's.
            self.sweep_to(me.other(), trace);
            self.over = true;
            return;
        }
        if self.moves(rules).is_empty() {
            match rules.ending {
                Ending::Feed if self.on_side(me) > 0 => {
                    // The mover cannot feed the other side: the seeds left
                    // are the mover's.
                    self.sweep_to(me, trace);
                }
                Ending::Sweep => self.sweep_own(trace),
                _ => self.sweep_to(me.other(), trace),
            }
            self.over = true;
        }
    }

    /// Hellier's Congkak: `side` fills her row again from her store,
    /// `per_pit` to a pit from her first pit on; the rest are closed.
    /// Returns false if she cannot fill a single pit.
    fn restock(&mut self, side: Side, per_pit: u8, trace: &mut Trace) -> bool {
        if self.stores[side.index()] < per_pit as u16 {
            return false;
        }
        let mut hand = 0;
        act(self, trace, &mut hand, Step::Restock { side, per_pit });
        trace.events.restocked = true;
        true
    }

    /// Sows `mv`, with every lap and capture it brings. Returns whether the
    /// mover plays again (a last seed in the mover's own store).
    pub fn sow(&mut self, rules: &Rules, mv: Move, trace: &mut Trace) -> bool {
        let mover = self.to_move;
        let mut hand: u16 = 0;
        let mut origin = mv.pit;
        let mut at = Slot::Pit(origin);
        let mut laps = 0;
        loop {
            laps += 1;
            let lift = self.pits[origin as usize];
            act(self, trace, &mut hand, Step::Lift { pit: origin });
            if rules.origin == Origin::FirstBack && lift > 1 {
                act(self, trace, &mut hand, Step::Drop { slot: Slot::Pit(origin) });
            }
            while hand > 0 {
                at = self.next_slot(rules, at, mover, origin);
                act(self, trace, &mut hand, Step::Drop { slot: at });
                if let (Some(cow), Slot::Pit(p)) = (rules.cow, at)
                    && self.pits[p as usize] == cow
                {
                    let to = self.owner(p);
                    act(self, trace, &mut hand, Step::Take { pit: p, to, why: Why::Cow });
                    trace.events.cows += cow as u16;
                }
            }
            let Slot::Pit(last) = at else {
                // The last seed in the mover's own store.
                return rules.store_again;
            };
            if laps >= MAX_LAPS {
                return false;
            }
            let count = self.pits[last as usize];
            match rules.laps {
                Laps::Single => break,
                Laps::Relay => {
                    if count > 1 {
                        origin = last;
                        continue;
                    }
                    break;
                }
                Laps::Lane => {
                    if count == 3 || count >= 5 {
                        origin = last;
                        continue;
                    }
                    break;
                }
                Laps::Indian => {
                    // The pit after the last seed is lifted and sown; if it
                    // is empty, the pit beyond it is captured and the move
                    // ends (Durai); if that is empty too, the move ends with
                    // nothing.
                    let next = self.next_pit(last);
                    if self.pits[next as usize] > 0 {
                        origin = next;
                        at = Slot::Pit(next);
                        continue;
                    }
                    let beyond = self.next_pit(next);
                    if self.pits[beyond as usize] == 0 {
                        return false;
                    }
                    let seeds = self.pits[beyond as usize] as u16;
                    act(self, trace, &mut hand, Step::Take { pit: beyond, to: mover, why: Why::Capture });
                    trace.events.captured += seeds;
                    return false;
                }
            }
        }
        let Slot::Pit(last) = at else { return false };
        self.capture(rules, last, trace);
        false
    }

    /// The captures after the last seed fell in `last`.
    fn capture(&mut self, rules: &Rules, last: u8, trace: &mut Trace) {
        let mover = self.to_move;
        let theirs = |p: u8, pos: &Position| pos.owner(p) != mover;
        let mut hand = 0;
        let mut take: Vec<u8> = Vec::new();
        match rules.capture {
            Capture::Counts { counts, opponent_only, with_opposite } => {
                let mut p = last;
                let mut seen = HashSet::new();
                while counts.contains(&self.pits[p as usize])
                    && (!opponent_only || theirs(p, self))
                    && self.tuz_owner(p).is_none()
                    && seen.insert(p)
                {
                    take.push(p);
                    if with_opposite {
                        take.push(self.opposite(p));
                    }
                    p = self.prev_pit(p);
                }
            }
            Capture::Even => {
                let count = self.pits[last as usize];
                if theirs(last, self) && self.tuz_owner(last).is_none() {
                    if count.is_multiple_of(2) && count > 0 {
                        take.push(last);
                    } else if count == 3 && rules.tuzdyk && self.may_make_tuzdyk(rules, last) {
                        let seeds = count as u16;
                        act(self, trace, &mut hand, Step::Tuzdyk { pit: last, owner: mover });
                        trace.events.tuzdyk = Some(last);
                        trace.events.captured += seeds;
                        return;
                    }
                }
            }
            Capture::EmptyOwn { with_seed, alone } => {
                if !theirs(last, self) && self.pits[last as usize] == 1 {
                    let facing = self.opposite(last);
                    if self.pits[facing as usize] > 0 || alone {
                        if with_seed {
                            take.push(last);
                        }
                        take.push(facing);
                    }
                }
            }
            Capture::InLap => {}
        }
        let mut done = HashSet::new();
        take.retain(|&p| self.pits[p as usize] > 0 && done.insert(p));
        if take.is_empty() {
            return;
        }
        let left: u16 = self
            .row(mover.other())
            .filter(|p| !take.contains(p))
            .map(|p| self.pits[p as usize] as u16)
            .sum();
        if left == 0 {
            if rules.grand_slam == GrandSlam::NoCapture {
                trace.events.grand_slam = true;
                return;
            }
            trace.events.emptied = true;
        }
        for p in take {
            trace.events.captured += self.pits[p as usize] as u16;
            act(self, trace, &mut hand, Step::Take { pit: p, to: mover, why: Why::Capture });
        }
    }

    /// The pit before `p` in the order of sowing.
    fn prev_pit(&self, p: u8) -> u8 {
        let n = self.len() as u8;
        (p + n - 1) % n
    }

    /// Toguz Kumalak: a side may make one tuzdyk, never in the opponent's
    /// last (ninth) hole, nor in the hole matching the opponent's tuzdyk.
    fn may_make_tuzdyk(&self, rules: &Rules, pit: u8) -> bool {
        let me = self.to_move;
        let n = rules.pits as u8;
        let place = |p: u8| p % n;
        self.tuz[me.index()].is_none()
            && place(pit) != n - 1
            && self.tuz[me.other().index()].is_none_or(|t| place(t) != place(pit))
    }

    /// Ends a game or round that only goes round without captures (a
    /// position seen for the third time): each side takes the seeds on its
    /// own side, as Bell gives for Wari.
    pub fn share(&mut self) -> Trace {
        let mut trace = Trace::default();
        self.sweep_own(&mut trace);
        self.over = true;
        trace.events.over = true;
        trace
    }

    /// Lane: the board turned round, each row now the other side's.
    pub fn turned(&self) -> Position {
        let n = self.len() / 2;
        let mut next = self.clone();
        for p in 0..self.len() {
            next.pits[(p + n) % self.len()] = self.pits[p];
        }
        next
    }

    /// The name of a pit: A to F (or further) for South's pits in the order
    /// of sowing, a to f for North's, as Lane and Bell name them.
    pub fn name(&self, p: u8) -> char {
        let n = (self.len() / 2) as u8;
        if p < n { (b'A' + p) as char } else { (b'a' + p - n) as char }
    }

    /// The winner of a finished game or round by the stores, or `None` for
    /// a tie.
    pub fn leader(&self) -> Option<Side> {
        use std::cmp::Ordering::*;
        match self.stores[0].cmp(&self.stores[1]) {
            Greater => Some(Side::South),
            Less => Some(Side::North),
            Equal => None,
        }
    }

    /// The position at the start of the next round of a refill match:
    /// each player fills their pits from their own store, `per_pit` each,
    /// from their first pit on, as far as the seeds go; the rest of the
    /// store stays in it and the unfilled pits are closed. Returns `None`
    /// if a player cannot fill a single pit: the match is over.
    pub fn next_round(&self, rules: &Rules, first: Side) -> Option<Position> {
        let Match::Refill { per_pit, .. } = rules.rounds else { return None };
        let mut next = Position::new(rules);
        next.to_move = first;
        for side in Side::BOTH {
            let mut store = self.stores[side.index()];
            if store < per_pit as u16 {
                return None;
            }
            for p in next.row(side) {
                if store >= per_pit as u16 {
                    next.pits[p as usize] = per_pit;
                    store -= per_pit as u16;
                } else {
                    next.pits[p as usize] = 0;
                    next.closed[p as usize] = true;
                }
            }
            next.stores[side.index()] = store;
        }
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(rules: &Rules, pits: &[u8], to_move: Side) -> Position {
        let mut p = Position::new(rules);
        p.pits = pits.to_vec();
        p.to_move = to_move;
        p
    }

    /// Replaying a trace from the position before the move gives the
    /// position after it.
    fn replays(rules: &Rules, before: &Position, mv: Move) -> (Position, Trace) {
        let mut after = before.clone();
        let trace = after.play(rules, mv);
        let mut again = before.clone();
        let mut hand = 0;
        for &s in &trace.steps {
            apply(&mut again, &mut hand, s);
        }
        assert_eq!(hand, 0);
        again.over = after.over;
        assert_eq!(again, after, "{:?}", trace.steps);
        (after, trace)
    }

    /// Bell's Wari example: the position after P empties C at the start.
    #[test]
    fn wari_opening_sowing() {
        let rules = Rules::wari();
        let before = Position::new(&rules);
        let (after, _) = replays(&rules, &before, Move { pit: 2 });
        assert_eq!(after.pits, vec![4, 4, 0, 5, 5, 5, 5, 4, 4, 4, 4, 4]);
        assert_eq!(after.to_move, Side::North);
    }

    /// A capture runs back through the twos and threes on the other side.
    #[test]
    fn oware_chain_capture() {
        let rules = Rules::oware();
        // South's last pit holds 3: it reaches pits 6, 7 and 8, making
        // them 2, 3 and 3 (with 1 left on pit 9 so it is no grand slam).
        let before = at(&rules, &[0, 0, 0, 0, 0, 3, 1, 2, 2, 1, 0, 0], Side::South);
        let (after, trace) = replays(&rules, &before, Move { pit: 5 });
        assert_eq!(trace.events.captured, 8);
        assert_eq!(after.pits[6..9], [0, 0, 0]);
        assert_eq!(after.stores[0], 8);
    }

    /// Bell's Wari: a move that would take every seed on the other side may
    /// not be played.
    #[test]
    fn wari_grand_slam_is_not_allowed() {
        let rules = Rules::wari();
        let before = at(&rules, &[1, 0, 0, 0, 0, 2, 1, 1, 0, 0, 0, 0], Side::South);
        assert_eq!(before.moves(&rules), vec![Move { pit: 0 }]);
    }

    /// Taking every seed on the other side takes none.
    #[test]
    fn oware_grand_slam_takes_nothing() {
        let rules = Rules::oware();
        // South keeps a seed in pit 0, so North need not feed afterwards.
        let before = at(&rules, &[1, 0, 0, 0, 0, 2, 1, 1, 0, 0, 0, 0], Side::South);
        let (after, trace) = replays(&rules, &before, Move { pit: 5 });
        assert!(trace.events.grand_slam);
        assert_eq!(trace.events.captured, 0);
        assert_eq!(after.pits[6..8], [2, 2]);
    }

    /// Twelve or more seeds pass over the pit they came from.
    #[test]
    fn oware_skips_the_origin() {
        let rules = Rules::oware();
        let mut before = at(&rules, &[0; 12], Side::South);
        before.pits[0] = 12;
        before.pits[6] = 4;
        let (after, _) = replays(&rules, &before, Move { pit: 0 });
        assert_eq!(after.pits[0], 0);
        assert_eq!(after.pits[1], 2);
    }

    /// A player must feed an opponent who has nothing.
    #[test]
    fn oware_must_feed() {
        let rules = Rules::oware();
        let before = at(&rules, &[1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], Side::South);
        assert_eq!(before.moves(&rules), vec![Move { pit: 5 }]);
    }

    /// A last seed in your own store plays again; one in an empty pit of
    /// your own takes the pit facing it.
    #[test]
    fn kalah_again_and_capture() {
        let rules = Rules::kalah();
        let before = Position::new(&rules);
        let (after, trace) = replays(&rules, &before, Move { pit: 2 });
        assert!(trace.events.again);
        assert_eq!(after.to_move, Side::South);
        assert_eq!(after.stores[0], 1);
        let before = at(&rules, &[1, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4], Side::South);
        let (after, trace) = replays(&rules, &before, Move { pit: 0 });
        assert_eq!(trace.events.captured, 5);
        assert_eq!(after.pits[1], 0);
        assert_eq!(after.pits[10], 0);
    }

    /// Lane: a last pebble making three lifts the pit and sows on; one
    /// making two takes it with the pit facing it.
    #[test]
    fn lane_relay_then_capture() {
        let rules = Rules::lane_wise();
        // From pit 0 (2 seeds): pit 1 gets one, pit 2 becomes 3 and is
        // lifted: pits 3, 4 and 5; pit 5 becomes 2 and is taken with
        // pit 6 facing it.
        let before = at(&rules, &[2, 0, 2, 0, 0, 1, 5, 3, 0, 0, 0, 7], Side::South);
        let (after, trace) = replays(&rules, &before, Move { pit: 0 });
        assert_eq!(trace.events.captured, 7);
        assert_eq!(after.pits[5], 0);
        assert_eq!(after.pits[6], 0);
    }

    /// Toguz Kumalak: the first stone goes back into its hole, an even count
    /// on the other side is taken, and three makes a tuzdyk.
    #[test]
    fn toguz_first_back_even_and_tuzdyk() {
        let rules = Rules::toguz();
        let before = Position::new(&rules);
        let (after, trace) = replays(&rules, &before, Move { pit: 8 });
        // Nine stones from the last hole: one back, eight into North's
        // first eight holes; the last makes North's eighth hole 10.
        assert_eq!(after.pits[8], 1);
        assert_eq!(trace.events.captured, 10);
        let mut before = at(&rules, &[0; 18], Side::South);
        before.pits[0] = 3;
        before.pits[2] = 2;
        before.pits[9] = 5;
        let (after, trace) = replays(&rules, &before, Move { pit: 0 });
        // 3 stones: back into 0, then 1, 2: hole 2 is South's own: nothing.
        assert_eq!(trace.events.captured, 0);
        assert_eq!(after.pits[2], 3);
        let mut before = at(&rules, &[0; 18], Side::South);
        before.pits[7] = 3;
        before.pits[9] = 2;
        before.pits[12] = 1;
        let (after, trace) = replays(&rules, &before, Move { pit: 7 });
        assert_eq!(trace.events.tuzdyk, Some(9));
        assert_eq!(after.tuz[0], Some(9));
    }

    /// Pallanguli: a pit reaching four is a cow and goes to its owner.
    #[test]
    fn pallanguli_cow() {
        let rules = Rules::pallanguli();
        let mut before = at(&rules, &[0; 14], Side::South);
        before.pits[0] = 2;
        before.pits[1] = 3;
        before.pits[5] = 1;
        let (_, trace) = replays(&rules, &before, Move { pit: 0 });
        assert_eq!(trace.events.cows, 4);
    }

    /// The next round fills as many pits as the store allows and closes
    /// the rest.
    #[test]
    fn refill_closes_pits() {
        let rules = Rules::pallanguli();
        let mut end = at(&rules, &[0; 14], Side::South);
        end.stores = [50, 34];
        let next = end.next_round(&rules, Side::North).expect("both can play");
        assert_eq!(next.closed.iter().filter(|&&c| c).count(), 2);
        assert_eq!(next.stores, [8, 4]);
        end.stores = [80, 4];
        assert!(end.next_round(&rules, Side::North).is_none());
    }
}
