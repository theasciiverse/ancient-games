//! The rules of Senet, for each rule set the game offers.
//!
//! No rules of Senet survive. What is played today are reconstructions:
//! Timothy Kendall's (1978), in its seven-piece original and the five-piece
//! "modern" form most sets use, and R. C. Bell's (1960). Every rule here
//! follows its source; where a source is silent, the help says what this
//! game does.
//!
//! The board is 3x10 squares, the "houses", numbered along an S-shaped
//! track: 1 to 10 left to right on the top row, 11 to 20 right to left on
//! the middle row, 21 to 30 left to right on the bottom row. Both sides run
//! the same way. A piece that leaves the board is borne off (31).

/// The two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::White => 0,
            Side::Black => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::White => "White",
            Side::Black => "Black",
        }
    }
}

/// Houses on the board.
pub const HOUSES: u8 = 30;

/// Where a borne-off piece goes.
pub const OFF: u8 = 31;

/// The houses with names and what they do.
pub const REBIRTH: u8 = 15;
pub const BEAUTY: u8 = 26;
pub const WATER: u8 = 27;
pub const THREE_TRUTHS: u8 = 28;
pub const RE_ATOUM: u8 = 29;
pub const HORUS: u8 = 30;

/// The grid square (row, column) of house `house` (1 to 30).
pub fn square(house: u8) -> (usize, usize) {
    let i = house as usize - 1;
    let row = i / 10;
    let col = if row == 1 { 9 - i % 10 } else { i % 10 };
    (row, col)
}

/// The house at a grid square.
pub fn house_at(row: usize, col: usize) -> u8 {
    let col = if row == 1 { 9 - col } else { col };
    (row * 10 + col + 1) as u8
}

/// The rule sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Kendall's original with seven pieces, as Damian Walker gives it.
    Kendall7,
    /// The five-piece "modern Kendall" rules most sets use.
    Kendall5,
    /// R. C. Bell's Game of Thirty Squares (1960).
    Bell,
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub variant: Variant,
    pub pieces: u8,
    /// How to play, a line each, for the help.
    pub summary: &'static [&'static str],
}

impl Rules {
    pub fn kendall5() -> Rules {
        Rules {
            id: "kendall-5",
            name: "Kendall, five pieces",
            source: "Timothy Kendall's reconstruction (1978) in the five-piece \
                     form most modern sets and online games use, as written out \
                     exactly by Devin O'Keefe for his solution of the game (2026).",
            variant: Variant::Kendall5,
            pieces: 5,
            summary: &[
                "Throw four sticks: the light sides up are how far you move, 1 to 4; none up is 5. After moving on 1, 4 or 5 you throw again.",
                "Land on a lone enemy piece and you swap places. A piece next to a friend is protected; three or more in a row form a blockade nobody passes.",
                "If no piece can move forward, one must move backward. A throw with no move at all ends your turn.",
                "Every piece must stop on the House of Beauty (26). The House of Water (27) sends a piece back to the House of Rebirth (15). From 28, 29 and 30 a piece leaves only on exactly 3, 2 or 1; from 26 on a 5.",
                "First to bear off all five pieces wins.",
            ],
        }
    }

    pub fn kendall7() -> Rules {
        Rules {
            id: "kendall-7",
            name: "Kendall, seven pieces",
            source: "Timothy Kendall's original reconstruction in Passing \
                     Through the Netherworld (1978), with seven pieces each, as \
                     Damian Walker sets it out.",
            variant: Variant::Kendall7,
            pieces: 7,
            summary: &[
                "Throw four sticks: the light sides up are how far you move, 1 to 4; none up is 5. After moving on 1, 4 or 5 you throw again.",
                "Land on a lone enemy piece and it goes back to where you came from. A piece next to a friend is protected. If no piece can move forward, one must move backward.",
                "Every piece must stop on the House of Beauty (26). A piece falling into the House of Humiliation, the water (27), ends your turn and freezes your other pieces until it is rescued: move it to the House of Rebirth (15) instead of throwing (R), or bear it off on an exact 4.",
                "Pieces bear off from the last five houses on an exact throw; from 28 only on 3, from 29 only on 2, from 30 on any throw.",
                "First to bear off all seven pieces wins.",
            ],
        }
    }

    pub fn bell() -> Rules {
        Rules {
            id: "bell",
            name: "R. C. Bell 1960",
            source: "R. C. Bell's suggested rules for the Game of Thirty Squares \
                     in Board and Table Games from Many Civilizations (1960): ten \
                     men each, a long die, and pieces that are taken leave the \
                     game.",
            variant: Variant::Bell,
            pieces: 10,
            summary: &[
                "Throw the long die, 1 to 4, and move one man forward that far.",
                "Land on an enemy man and it is captured and leaves the game. You cannot land on your own.",
                "The door (26) and the marked houses X, III, II, I (27 to 30) are safe. A man on a marked house leaves the board on the exact throw (4, 3, 2, 1) and scores; a man must leave a marked house, or the door, at the first throw that allows it.",
                "When the board is empty, the player who brought more men off wins.",
                "Bell does not give the long die's faces beyond 1 to 4; here each is as likely.",
            ],
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![Rules::kendall5(), Rules::kendall7(), Rules::bell()]
    }

    fn kendall(&self) -> bool {
        self.variant != Variant::Bell
    }

    /// Every throw with its chance out of the total of the chances.
    pub fn throws(&self) -> Vec<(u8, u32)> {
        if self.kendall() {
            // Four sticks: light sides up 1 to 4, none up counts 5.
            vec![(1, 4), (2, 6), (3, 4), (4, 1), (5, 1)]
        } else {
            vec![(1, 1), (2, 1), (3, 1), (4, 1)]
        }
    }

    /// Whether moving on `throw` gives another throw.
    pub fn throws_again(&self, throw: u8) -> bool {
        self.kendall() && matches!(throw, 1 | 4 | 5)
    }
}

/// What a side can do with a throw (or, for a piece in the water under
/// Kendall's seven-piece rules, instead of throwing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Move the piece on `from` to `to` (`OFF` to bear it off).
    Move { from: u8, to: u8 },
    /// Kendall, seven pieces: take the piece out of the water to the House
    /// of Rebirth instead of throwing.
    Rescue,
}

/// What an action did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Outcome {
    /// An enemy piece was bumped (swapped or sent back).
    pub bumped: bool,
    /// An enemy piece was captured and left the game (Bell).
    pub captured: bool,
    /// The piece fell into the water.
    pub water: bool,
    /// A piece was borne off.
    pub off: bool,
    /// The side throws again.
    pub again: bool,
    /// The game is over.
    pub over: bool,
}

/// The position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Who is on each house, 1 to 30 (index 0 unused).
    pub board: [Option<Side>; 31],
    /// Pieces borne off by each side.
    pub off: [u8; 2],
    pub to_move: Side,
}

impl Position {
    /// The starting position. Kendall: pieces alternate along the first
    /// houses, White on 1. Kendall's seven-piece rules let Black move first,
    /// the five-piece rules White. Bell: ten men each on the first two rows,
    /// the first player's on the second houses of each pair.
    pub fn new(rules: &Rules, first_bell: Side) -> Position {
        let mut board = [None; 31];
        let n = rules.pieces as usize * 2;
        let (on_odd, to_move) = match rules.variant {
            Variant::Kendall7 => (Side::White, Side::Black),
            Variant::Kendall5 => (Side::White, Side::White),
            Variant::Bell => (first_bell.other(), first_bell),
        };
        for (i, house) in board.iter_mut().enumerate().skip(1).take(n) {
            *house = Some(if i % 2 == 1 { on_odd } else { on_odd.other() });
        }
        Position { board, off: [0, 0], to_move }
    }

    pub fn at(&self, house: u8) -> Option<Side> {
        if (1..=HOUSES).contains(&house) {
            self.board[house as usize]
        } else {
            None
        }
    }

    /// The houses of `side`'s pieces, lowest first.
    pub fn pieces(&self, side: Side) -> Vec<u8> {
        (1..=HOUSES).filter(|&h| self.at(h) == Some(side)).collect()
    }

    pub fn on_board(&self, side: Side) -> usize {
        self.pieces(side).len()
    }

    /// Whether the piece on `house` is protected by a friend next to it.
    /// Kendall's seven-piece rules never protect a piece in the water.
    fn protected(&self, rules: &Rules, house: u8) -> bool {
        let Some(owner) = self.at(house) else { return false };
        if rules.variant == Variant::Kendall7 && house == WATER {
            return false;
        }
        self.at(house - 1) == Some(owner) || self.at(house + 1) == Some(owner)
    }

    /// Whether `house` is in a run of three or more pieces of `side`
    /// (Kendall, five pieces).
    fn in_blockade(&self, side: Side, house: u8) -> bool {
        if self.at(house) != Some(side) {
            return false;
        }
        let mut lo = house;
        while lo > 1 && self.at(lo - 1) == Some(side) {
            lo -= 1;
        }
        let mut hi = house;
        while hi < HOUSES && self.at(hi + 1) == Some(side) {
            hi += 1;
        }
        hi - lo + 1 >= 3
    }

    /// The side's piece in the water, under Kendall's seven-piece rules.
    fn drowning(&self, rules: &Rules) -> bool {
        rules.variant == Variant::Kendall7 && self.at(WATER) == Some(self.to_move)
    }

    /// Whether the side may rescue a piece from the water instead of
    /// throwing (Kendall, seven pieces).
    pub fn can_rescue(&self, rules: &Rules) -> bool {
        self.drowning(rules) && self.at(REBIRTH).is_none()
    }

    /// The legal moves for the side to move with throw `t`.
    pub fn moves(&self, rules: &Rules, t: u8) -> Vec<Action> {
        match rules.variant {
            Variant::Kendall5 => self.moves_kendall5(rules, t),
            Variant::Kendall7 => self.moves_kendall7(rules, t),
            Variant::Bell => self.moves_bell(t),
        }
    }

    fn moves_kendall5(&self, rules: &Rules, t: u8) -> Vec<Action> {
        let me = self.to_move;
        let enemy = me.other();
        let blocked = |a: u8, d: u8| {
            let (lo, hi) = if a < d { (a, d) } else { (d, a) };
            (lo + 1..hi).any(|h| h <= HOUSES && self.in_blockade(enemy, h))
        };
        let landing = |d: u8| match self.at(d) {
            None => true,
            Some(s) if s == me => false,
            Some(_) => !self.protected(rules, d),
        };
        let mut forward = Vec::new();
        for a in self.pieces(me) {
            if (THREE_TRUTHS..=HORUS).contains(&a) {
                if a + t == OFF {
                    forward.push(Action::Move { from: a, to: OFF });
                }
                continue;
            }
            let d = a + t;
            if a < BEAUTY && d > BEAUTY {
                continue;
            }
            if blocked(a, d) {
                continue;
            }
            if d == OFF {
                forward.push(Action::Move { from: a, to: OFF });
            } else if d == WATER || (d <= HOUSES && landing(d)) {
                forward.push(Action::Move { from: a, to: d });
            }
        }
        if !forward.is_empty() {
            return forward;
        }
        let mut back = Vec::new();
        for a in self.pieces(me) {
            if a > BEAUTY || a <= t {
                continue;
            }
            let d = a - t;
            if blocked(d, a) || !landing(d) {
                continue;
            }
            back.push(Action::Move { from: a, to: d });
        }
        back
    }

    fn moves_kendall7(&self, rules: &Rules, t: u8) -> Vec<Action> {
        let me = self.to_move;
        // A piece in the water freezes the others: only bearing it off on an
        // exact 4 is possible with a throw.
        if self.drowning(rules) {
            return if t == 4 {
                vec![Action::Move { from: WATER, to: OFF }]
            } else {
                Vec::new()
            };
        }
        let landing = |d: u8| match self.at(d) {
            None => true,
            Some(s) if s == me => false,
            Some(_) => !self.protected(rules, d),
        };
        let mut forward = Vec::new();
        for a in self.pieces(me) {
            match a {
                HORUS => {
                    forward.push(Action::Move { from: a, to: OFF });
                    continue;
                }
                THREE_TRUTHS | RE_ATOUM => {
                    if a + t == OFF {
                        forward.push(Action::Move { from: a, to: OFF });
                    }
                    continue;
                }
                _ => {}
            }
            let d = a + t;
            if a < BEAUTY && d > BEAUTY {
                continue;
            }
            if d == OFF {
                // From the last five houses on an exact throw.
                forward.push(Action::Move { from: a, to: OFF });
            } else if d <= HOUSES && landing(d) {
                forward.push(Action::Move { from: a, to: d });
            }
        }
        if !forward.is_empty() {
            return forward;
        }
        let mut back = Vec::new();
        for a in self.pieces(me) {
            if a > BEAUTY || a <= t {
                continue;
            }
            let d = a - t;
            if landing(d) {
                back.push(Action::Move { from: a, to: d });
            }
        }
        if back.is_empty() {
            // With no other piece to move, a piece stuck on 28 or 29 falls
            // back into the water.
            let mine = self.pieces(me);
            if mine.len() == 1
                && matches!(mine[0], THREE_TRUTHS | RE_ATOUM)
                && self.at(WATER).is_none()
            {
                back.push(Action::Move { from: mine[0], to: WATER });
            }
        }
        back
    }

    fn moves_bell(&self, t: u8) -> Vec<Action> {
        let me = self.to_move;
        let marked = |h: u8| h >= BEAUTY;
        let mut moves = Vec::new();
        for a in self.pieces(me) {
            let d = a + t;
            if a >= WATER {
                // On a marked house: off on the exact throw.
                if d == OFF {
                    moves.push(Action::Move { from: a, to: OFF });
                }
                continue;
            }
            if d > HOUSES {
                continue;
            }
            match self.at(d) {
                Some(s) if s == me => continue,
                Some(_) if marked(d) => continue,
                _ => moves.push(Action::Move { from: a, to: d }),
            }
        }
        // A man on a marked house, or the door, must leave it at the first
        // throw that allows it (the door: onto a marked house).
        let forced: Vec<Action> = moves
            .iter()
            .copied()
            .filter(|m| matches!(m, Action::Move { from, .. } if marked(*from)))
            .collect();
        if forced.is_empty() { moves } else { forced }
    }

    /// Plays `action` with throw `t` (ignored for a rescue).
    pub fn play(&mut self, rules: &Rules, action: Action, t: u8) -> Outcome {
        let me = self.to_move;
        let mut out = Outcome::default();
        match action {
            Action::Rescue => {
                self.board[WATER as usize] = None;
                self.board[REBIRTH as usize] = Some(me);
            }
            Action::Move { from, to } => {
                self.board[from as usize] = None;
                if to == OFF {
                    self.off[me.index()] += 1;
                    out.off = true;
                } else {
                    self.land(rules, from, to, &mut out);
                }
            }
        }
        out.over = self.winner(rules).is_some() || self.finished(rules);
        out.again = !out.over
            && !out.water
            && matches!(action, Action::Move { .. })
            && rules.throws_again(t);
        if !out.again {
            self.to_move = me.other();
        }
        out
    }

    fn land(&mut self, rules: &Rules, from: u8, to: u8, out: &mut Outcome) {
        let me = self.to_move;
        let enemy = me.other();
        match rules.variant {
            Variant::Kendall5 => {
                if to == WATER {
                    // To the House of Rebirth, or the highest empty house
                    // below it.
                    let h = (1..=REBIRTH).rev().find(|&h| self.at(h).is_none()).unwrap_or(1);
                    self.board[h as usize] = Some(me);
                    out.water = true;
                    return;
                }
                if self.at(to) == Some(enemy) {
                    self.board[from as usize] = Some(enemy);
                    out.bumped = true;
                }
                self.board[to as usize] = Some(me);
            }
            Variant::Kendall7 => {
                if self.at(to) == Some(enemy) {
                    out.bumped = true;
                    // Bumped from 28 to 30 (only from the House of Beauty):
                    // into the water, or to Beauty if the water is taken.
                    let back = if to > WATER && self.at(WATER).is_none() {
                        WATER
                    } else {
                        from
                    };
                    self.board[back as usize] = Some(enemy);
                }
                self.board[to as usize] = Some(me);
                if to == WATER {
                    out.water = true;
                }
            }
            Variant::Bell => {
                if self.at(to) == Some(enemy) {
                    out.captured = true;
                }
                self.board[to as usize] = Some(me);
            }
        }
    }

    /// A throw with no move: the turn passes.
    pub fn pass(&mut self) {
        self.to_move = self.to_move.other();
    }

    /// The side that has borne off every piece (Kendall), if any.
    pub fn winner(&self, rules: &Rules) -> Option<Side> {
        if rules.variant == Variant::Bell {
            return None;
        }
        [Side::White, Side::Black]
            .into_iter()
            .find(|&s| self.off[s.index()] == rules.pieces)
    }

    /// Bell: the game is over when the board is empty.
    pub fn finished(&self, rules: &Rules) -> bool {
        rules.variant == Variant::Bell && self.board.iter().all(Option::is_none)
    }

    /// Bell: the result when the board is empty (`None` for a draw).
    pub fn bell_winner(&self) -> Option<Side> {
        match self.off[0].cmp(&self.off[1]) {
            std::cmp::Ordering::Greater => Some(Side::White),
            std::cmp::Ordering::Less => Some(Side::Black),
            std::cmp::Ordering::Equal => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(rules: &Rules, white: &[u8], black: &[u8], to_move: Side) -> Position {
        let mut p = Position::new(rules, Side::White);
        p.board = [None; 31];
        for &h in white {
            p.board[h as usize] = Some(Side::White);
        }
        for &h in black {
            p.board[h as usize] = Some(Side::Black);
        }
        p.to_move = to_move;
        p
    }

    fn mv(from: u8, to: u8) -> Action {
        Action::Move { from, to }
    }

    #[test]
    fn the_track_is_an_s() {
        assert_eq!(square(1), (0, 0));
        assert_eq!(square(10), (0, 9));
        assert_eq!(square(11), (1, 9));
        assert_eq!(square(20), (1, 0));
        assert_eq!(square(21), (2, 0));
        assert_eq!(square(30), (2, 9));
        for h in 1..=HOUSES {
            let (r, c) = square(h);
            assert_eq!(house_at(r, c), h);
        }
    }

    #[test]
    fn sticks_give_four_six_four_one_one() {
        let rules = Rules::kendall5();
        assert_eq!(rules.throws(), vec![(1, 4), (2, 6), (3, 4), (4, 1), (5, 1)]);
        assert!(rules.throws_again(1) && rules.throws_again(4) && rules.throws_again(5));
        assert!(!rules.throws_again(2) && !rules.throws_again(3));
    }

    #[test]
    fn starting_positions() {
        let k5 = Rules::kendall5();
        let p = Position::new(&k5, Side::White);
        assert_eq!(p.pieces(Side::White), vec![1, 3, 5, 7, 9]);
        assert_eq!(p.pieces(Side::Black), vec![2, 4, 6, 8, 10]);
        assert_eq!(p.to_move, Side::White);
        let k7 = Rules::kendall7();
        let p = Position::new(&k7, Side::White);
        assert_eq!(p.pieces(Side::White), vec![1, 3, 5, 7, 9, 11, 13]);
        assert_eq!(p.to_move, Side::Black);
        let bell = Rules::bell();
        let p = Position::new(&bell, Side::Black);
        assert_eq!(p.on_board(Side::White), 10);
        assert_eq!(p.at(2), Some(Side::Black), "the first player on the second houses");
        assert_eq!(p.to_move, Side::Black);
    }

    #[test]
    fn kendall5_swaps_with_lone_pieces_but_not_protected_ones() {
        let rules = Rules::kendall5();
        let mut p = with(&rules, &[3], &[5, 9, 10], Side::White);
        // 5 is alone: swap. 9 and 10 protect each other.
        assert!(p.moves(&rules, 2).contains(&mv(3, 5)));
        let out = p.play(&rules, mv(3, 5), 2);
        assert!(out.bumped && !out.again);
        assert_eq!(p.at(3), Some(Side::Black));
        assert_eq!(p.at(5), Some(Side::White));
        let q = with(&rules, &[7], &[9, 10], Side::White);
        assert!(!q.moves(&rules, 2).contains(&mv(7, 9)));
    }

    #[test]
    fn kendall5_blockade_and_backwards() {
        let rules = Rules::kendall5();
        // Black's 6-7-8 blockade: White on 5 cannot pass with any throw.
        let p = with(&rules, &[5], &[6, 7, 8], Side::White);
        for t in 1..=5 {
            assert!(
                !p.moves(&rules, t).iter().any(|a| matches!(a, Action::Move { to, .. } if *to > 5)),
                "throw {t}"
            );
        }
        // No forward move: backwards it is.
        assert_eq!(p.moves(&rules, 2), vec![mv(5, 3)]);
    }

    #[test]
    fn kendall5_beauty_water_and_the_last_houses() {
        let rules = Rules::kendall5();
        let p = with(&rules, &[24], &[1], Side::White);
        // Cannot jump past 26.
        assert!(p.moves(&rules, 3).iter().all(|a| *a != mv(24, 27)));
        assert!(p.moves(&rules, 2).contains(&mv(24, 26)));
        // From 26: 1 into the water, 5 off.
        let mut q = with(&rules, &[26], &[1], Side::White);
        assert!(q.moves(&rules, 5).contains(&mv(26, OFF)));
        let out = q.play(&rules, mv(26, WATER), 1);
        assert!(out.water && !out.again);
        assert_eq!(q.at(REBIRTH), Some(Side::White));
        // 28 leaves only on 3, 29 on 2, 30 on 1.
        let r = with(&rules, &[28, 29, 30], &[1], Side::White);
        assert_eq!(r.moves(&rules, 3), vec![mv(28, OFF)]);
        assert_eq!(r.moves(&rules, 2), vec![mv(29, OFF)]);
        assert_eq!(r.moves(&rules, 1), vec![mv(30, OFF)]);
    }

    #[test]
    fn kendall7_water_freezes_and_rescue() {
        let rules = Rules::kendall7();
        let mut p = with(&rules, &[26, 10], &[1], Side::White);
        let out = p.play(&rules, mv(26, WATER), 1);
        assert!(out.water && !out.again, "the water ends the turn");
        p.to_move = Side::White;
        // The other piece is frozen: only an exact 4 bears the drowned one off.
        assert!(p.moves(&rules, 2).is_empty());
        assert_eq!(p.moves(&rules, 4), vec![mv(WATER, OFF)]);
        assert!(p.can_rescue(&rules));
        p.play(&rules, Action::Rescue, 0);
        assert_eq!(p.at(REBIRTH), Some(Side::White));
        assert_eq!(p.to_move, Side::Black, "a rescue ends the turn");
    }

    #[test]
    fn kendall7_bumps_back_and_horus_leaves_on_anything() {
        let rules = Rules::kendall7();
        let mut p = with(&rules, &[4], &[6], Side::White);
        p.play(&rules, mv(4, 6), 2);
        assert_eq!(p.at(4), Some(Side::Black));
        let q = with(&rules, &[30], &[1], Side::White);
        for t in 1..=5 {
            assert_eq!(q.moves(&rules, t), vec![mv(30, OFF)]);
        }
    }

    #[test]
    fn bell_captures_and_forces_marked_houses() {
        let rules = Rules::bell();
        let mut p = with(&rules, &[3], &[5], Side::White);
        let out = p.play(&rules, mv(3, 5), 2);
        assert!(out.captured);
        assert_eq!(p.on_board(Side::Black), 0);
        // A man on X (27) must leave on an exact 4, before anything else.
        let q = with(&rules, &[27, 10], &[1], Side::White);
        assert_eq!(q.moves(&rules, 4), vec![mv(27, OFF)]);
        assert_eq!(q.moves(&rules, 2), vec![mv(10, 12)]);
        // Marked houses are safe.
        let r = with(&rules, &[25], &[28], Side::White);
        assert!(r.moves(&rules, 3).is_empty());
        // The board empty: most men off wins.
        let mut s = with(&rules, &[30], &[], Side::White);
        s.off = [2, 3];
        let out = s.play(&rules, mv(30, OFF), 1);
        assert!(out.over);
        assert_eq!(s.bell_winner(), None, "three each: a draw");
    }
}
