//! The rules of the Royal Game of Ur: the board, the path, the throws and
//! the moves, for each rule set the game offers.
//!
//! The board is the third-millennium one from the royal tombs of Ur: a 3x8
//! grid without the fifth and sixth squares of the outer rows, so a block of
//! 4x3, a bridge of two and a block of 2x3. Row 0 is the far side (Dark's own
//! row), row 1 the shared middle row, row 2 the near side (Light's own row).

/// The two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Light,
    Dark,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::Light => 0,
            Side::Dark => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::Light => Side::Dark,
            Side::Dark => Side::Light,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Light => "Light",
            Side::Dark => "Dark",
        }
    }
}

/// Columns and rows of the grid.
pub const COLS: usize = 8;
pub const ROWS: usize = 3;

/// Whether a grid square is part of the board (the outer rows have no fifth
/// and sixth squares).
pub fn on_board(row: usize, col: usize) -> bool {
    row < ROWS && col < COLS && !(row != 1 && (col == 4 || col == 5))
}

/// The five rosettes: first from the left and second from the right on both
/// outer rows, fourth from the left on the middle row.
pub fn is_rosette(row: usize, col: usize) -> bool {
    match row {
        0 | 2 => col == 0 || col == 6,
        1 => col == 3,
        _ => false,
    }
}

/// Squares on the path, not counting the start (0) and home (`PATH + 1`).
/// Bell's path, used by Finkel: four squares on the own row leftwards from
/// the fourth, the whole middle row rightwards, then the last two squares of
/// the own row leftwards, and off.
pub const PATH: u8 = 14;

/// The position of a piece that has not entered.
pub const START: u8 = 0;

/// The position of a piece that has gone home.
pub const HOME: u8 = PATH + 1;

/// The grid square of path position `pos` (1 to `PATH`) for `side`.
pub fn square(side: Side, pos: u8) -> (usize, usize) {
    let own = match side {
        Side::Light => 2,
        Side::Dark => 0,
    };
    match pos {
        1..=4 => (own, 4 - pos as usize),
        5..=12 => (1, pos as usize - 5),
        13 => (own, 7),
        14 => (own, 6),
        _ => panic!("position {pos} is not on the board"),
    }
}

/// Whether path position `pos` is on the shared middle row, where the sides
/// meet.
pub fn is_shared(pos: u8) -> bool {
    (5..=12).contains(&pos)
}

/// What a throw of the lots gives: squares to move, and whether the side
/// throws again after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Throw {
    pub moves: u8,
    pub again: bool,
}

/// The throwing lots, and what each count of marked tips means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lots {
    /// Four tetrahedra, each with two of four tips marked: the count of
    /// marked tips up, 0 to 4, is the move; 0 loses the turn.
    FourBinary,
    /// R. C. Bell's reading of three: all three jewelled tips up, 5 and
    /// throw again; none, 4 and throw again; two, 1 and throw again; one,
    /// nothing, and the turn is over.
    BellThree,
}

impl Lots {
    /// How many lots are thrown.
    pub fn count(self) -> u32 {
        match self {
            Lots::FourBinary => 4,
            Lots::BellThree => 3,
        }
    }

    /// The throw for `marked` tips up.
    pub fn throw(self, marked: u32) -> Throw {
        match self {
            Lots::FourBinary => Throw { moves: marked as u8, again: false },
            Lots::BellThree => match marked {
                3 => Throw { moves: 5, again: true },
                0 => Throw { moves: 4, again: true },
                2 => Throw { moves: 1, again: true },
                _ => Throw { moves: 0, again: false },
            },
        }
    }

    /// Every possible throw with its chance out of `2^count`.
    pub fn distribution(self) -> Vec<(Throw, u32)> {
        let n = self.count();
        (0..=n).map(|k| (self.throw(k), binomial(n, k))).collect()
    }
}

/// How pieces come onto the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// Any throw brings a piece on, that many squares along.
    ByThrow,
    /// Only a throw of five brings a piece on, onto its first square
    /// (R. C. Bell: "it moves forward by the indicated numbers on the
    /// additional throws").
    OnFive,
}

/// What landing on a rosette does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rosettes {
    /// The side throws again.
    ThrowAgain,
    /// The opponent pays a fine into the pool (R. C. Bell).
    OpponentPays,
}

/// A betting game: what each side puts into the pool at the start, and the
/// fine for a rosette. The winner takes the pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stakes {
    pub stake: u32,
    pub fine: u32,
}

fn binomial(n: u32, k: u32) -> u32 {
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

/// A rule set, as plain data, so the engine serves every version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where the rules come from, in a sentence.
    pub source: &'static str,
    pub pieces: u8,
    pub lots: Lots,
    pub entry: Entry,
    pub rosettes: Rosettes,
    /// A piece on the middle rosette cannot be taken.
    pub middle_rosette_safe: bool,
    pub stakes: Option<Stakes>,
    /// How to play, a line each, for the help.
    pub summary: &'static [&'static str],
}

impl Rules {
    /// Finkel's rules as he played them against Tom Scott for the British
    /// Museum in 2017: the version most people play today.
    pub fn finkel_2017() -> Rules {
        Rules {
            id: "finkel-2017",
            name: "Finkel 2017",
            source: "Irving Finkel's rules as he played them against Tom Scott \
                     for the British Museum in 2017, from the tablet of 177 BC \
                     and the shape of the board.",
            pieces: 7,
            lots: Lots::FourBinary,
            entry: Entry::ByThrow,
            rosettes: Rosettes::ThrowAgain,
            middle_rosette_safe: true,
            stakes: None,
            summary: &[
                "Throw four lots: the marked tips up, 0 to 4, are how far you move. A throw of 0 loses the turn.",
                "Bring pieces on with any throw. Seven pieces each.",
                "A rosette gives you another throw. A piece on the middle rosette cannot be taken.",
            ],
        }
    }

    /// The rules sold with the British Museum's replica board in the 1990s.
    pub fn british_museum() -> Rules {
        Rules {
            id: "british-museum",
            name: "British Museum 1990s",
            source: "The rules Irving Finkel published with the British \
                     Museum's replica board in the late 1990s: five pieces, \
                     and no rosette is safe.",
            pieces: 5,
            lots: Lots::FourBinary,
            entry: Entry::ByThrow,
            rosettes: Rosettes::ThrowAgain,
            middle_rosette_safe: false,
            stakes: None,
            summary: &[
                "Throw four lots: the marked tips up, 0 to 4, are how far you move. A throw of 0 loses the turn.",
                "Bring pieces on with any throw. Five pieces each.",
                "A rosette gives you another throw. No square in the middle row is safe.",
            ],
        }
    }

    /// R. C. Bell's suggested rules, a betting game (Board and Table Games
    /// from Many Civilizations, 1960, second edition 1969, pp. 24-25).
    pub fn bell() -> Rules {
        Rules {
            id: "bell",
            name: "R. C. Bell",
            source: "R. C. Bell's suggested rules from Board and Table Games \
                     from Many Civilizations (1960): a betting game played with \
                     three lots, where you throw until a dead throw ends your \
                     turn.",
            pieces: 7,
            lots: Lots::BellThree,
            entry: Entry::OnFive,
            rosettes: Rosettes::OpponentPays,
            middle_rosette_safe: false,
            stakes: Some(Stakes { stake: 5, fine: 1 }),
            summary: &[
                "Throw three lots. All three jewelled tips up: 5 and throw again. None: 4 and throw again. Two: 1 and throw again. One: nothing, and your turn is over.",
                "A piece comes on only with a 5, onto its first square. A piece that is taken needs a 5 again.",
                "Both sides put 5 into the pool. Land on a rosette and your opponent pays 1 into it. First with every piece home takes the pool; your purse carries on to the next game.",
                "Bell does not say where a piece entered on a 5 stands, nor whether an unusable throw still throws again: here it stands on its first square, and the throw again stands.",
            ],
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![Rules::finkel_2017(), Rules::british_museum(), Rules::bell()]
    }

    /// Whether `pos` on the path is a rosette.
    pub fn is_rosette_pos(&self, pos: u8) -> bool {
        (1..=PATH).contains(&pos) && {
            let (row, col) = square(Side::Light, pos);
            is_rosette(row, col)
        }
    }

    /// Whether a piece at `pos` can be taken there.
    pub fn is_safe_pos(&self, pos: u8) -> bool {
        !is_shared(pos) || (self.middle_rosette_safe && self.is_rosette_pos(pos))
    }
}

/// A move: a piece of the side to move goes from `from` to `to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
}

/// What a move did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Outcome {
    /// An opposing piece was sent back to the start.
    pub took: bool,
    /// The side throws again.
    pub again: bool,
    /// The move landed on a rosette where the opponent pays a fine.
    pub fine: bool,
    /// A piece went home.
    pub home: bool,
    /// The side has brought every piece home.
    pub won: bool,
}

/// The position: where every piece is and whose turn it is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Path positions of each side's pieces, sorted.
    pub pieces: [Vec<u8>; 2],
    pub to_move: Side,
}

impl Position {
    pub fn new(rules: &Rules, first: Side) -> Position {
        let start = vec![START; rules.pieces as usize];
        Position {
            pieces: [start.clone(), start],
            to_move: first,
        }
    }

    pub fn of(&self, side: Side) -> &[u8] {
        &self.pieces[side.index()]
    }

    /// How many of `side`'s pieces are at `pos`.
    pub fn count_at(&self, side: Side, pos: u8) -> usize {
        self.of(side).iter().filter(|&&p| p == pos).count()
    }

    /// The opposing piece on the shared square at `pos`, if any. Shared
    /// squares are the same squares for both sides at the same position.
    fn opponent_on(&self, side: Side, pos: u8) -> bool {
        is_shared(pos) && self.of(side.other()).contains(&pos)
    }

    /// The legal moves for the side to move with a throw of `roll`, one per
    /// distinct starting square (pieces waiting at the start are alike).
    pub fn moves(&self, rules: &Rules, roll: u8) -> Vec<Move> {
        if roll == 0 {
            return Vec::new();
        }
        let side = self.to_move;
        let mut moves: Vec<Move> = Vec::new();
        for &from in self.of(side) {
            if from == HOME || moves.iter().any(|m| m.from == from) {
                continue;
            }
            let to = match (from, rules.entry) {
                (START, Entry::OnFive) if roll == 5 => 1,
                (START, Entry::OnFive) => continue,
                _ => from + roll,
            };
            // Off the board by an exact throw only.
            if to > HOME {
                continue;
            }
            if to < HOME {
                // One piece per square: not onto your own piece, and not
                // onto an opposing piece that is safe where it stands.
                if self.of(side).contains(&to) {
                    continue;
                }
                if self.opponent_on(side, to) && rules.is_safe_pos(to) {
                    continue;
                }
            }
            moves.push(Move { from, to });
        }
        moves
    }

    /// Plays `mv` for the side to move, and passes the turn unless the side
    /// throws again, by a rosette or because the throw (`keep`) says so.
    pub fn play(&mut self, rules: &Rules, mv: Move, keep: bool) -> Outcome {
        let side = self.to_move;
        let mut outcome = Outcome::default();

        let own = &mut self.pieces[side.index()];
        let i = own.iter().position(|&p| p == mv.from).expect("no piece there");
        own[i] = mv.to;
        own.sort_unstable();

        if mv.to == HOME {
            outcome.home = true;
            outcome.won = self.of(side).iter().all(|&p| p == HOME);
        } else if self.opponent_on(side, mv.to) {
            let theirs = &mut self.pieces[side.other().index()];
            let j = theirs.iter().position(|&p| p == mv.to).unwrap();
            theirs[j] = START;
            theirs.sort_unstable();
            outcome.took = true;
        }

        let rosette = !outcome.won && mv.to != HOME && rules.is_rosette_pos(mv.to);
        outcome.again = rosette && rules.rosettes == Rosettes::ThrowAgain;
        outcome.fine = rosette && rules.rosettes == Rosettes::OpponentPays;
        if !outcome.won && !outcome.again && !keep {
            self.to_move = side.other();
        }
        outcome
    }

    /// A throw with no move: the turn passes, unless the throw says to
    /// throw again (`keep`).
    pub fn pass(&mut self, keep: bool) {
        if !keep {
            self.to_move = self.to_move.other();
        }
    }

    /// The side that has brought every piece home, if any.
    pub fn winner(&self) -> Option<Side> {
        [Side::Light, Side::Dark]
            .into_iter()
            .find(|&s| self.of(s).iter().all(|&p| p == HOME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_has_twenty_squares_and_five_rosettes() {
        let squares = (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (r, c)))
            .filter(|&(r, c)| on_board(r, c))
            .count();
        assert_eq!(squares, 20);
        let rosettes = (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (r, c)))
            .filter(|&(r, c)| on_board(r, c) && is_rosette(r, c))
            .count();
        assert_eq!(rosettes, 5);
    }

    #[test]
    fn path_visits_fourteen_distinct_squares_on_the_board() {
        for side in [Side::Light, Side::Dark] {
            let squares: Vec<_> = (1..=PATH).map(|p| square(side, p)).collect();
            for &(r, c) in &squares {
                assert!(on_board(r, c), "{side:?} {r},{c}");
            }
            let mut unique = squares.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), PATH as usize);
        }
        // Rosettes on the path: 4 (corner), 8 (middle), 14 (near the end).
        let rules = Rules::finkel_2017();
        let rosettes: Vec<u8> = (1..=PATH).filter(|&p| rules.is_rosette_pos(p)).collect();
        assert_eq!(rosettes, vec![4, 8, 14]);
    }

    #[test]
    fn sides_share_the_middle_row_only() {
        for pos in 1..=PATH {
            let same = square(Side::Light, pos) == square(Side::Dark, pos);
            assert_eq!(same, is_shared(pos), "position {pos}");
        }
    }

    #[test]
    fn four_lots_give_one_four_six_four_one() {
        let moves: Vec<(u8, u32)> = Lots::FourBinary
            .distribution()
            .into_iter()
            .map(|(t, n)| (t.moves, n))
            .collect();
        assert_eq!(moves, vec![(0, 1), (1, 4), (2, 6), (3, 4), (4, 1)]);
        assert!(Lots::FourBinary.distribution().iter().all(|(t, _)| !t.again));
    }

    #[test]
    fn bells_three_lots() {
        // Out of 8: none up 1 (4, again), one up 3 (nothing, turn over),
        // two up 3 (1, again), three up 1 (5, again).
        let d = Lots::BellThree.distribution();
        assert_eq!(d[0], (Throw { moves: 4, again: true }, 1));
        assert_eq!(d[1], (Throw { moves: 0, again: false }, 3));
        assert_eq!(d[2], (Throw { moves: 1, again: true }, 3));
        assert_eq!(d[3], (Throw { moves: 5, again: true }, 1));
    }

    #[test]
    fn bell_enters_on_five_and_fines_on_rosettes() {
        let rules = Rules::bell();
        let mut p = Position::new(&rules, Side::Light);
        // Only a five brings a piece on, onto the first square.
        assert!(p.moves(&rules, 4).is_empty());
        assert!(p.moves(&rules, 1).is_empty());
        assert_eq!(p.moves(&rules, 5), vec![Move { from: 0, to: 1 }]);
        let out = p.play(&rules, Move { from: 0, to: 1 }, true);
        assert!(!out.again && !out.fine);
        assert_eq!(p.to_move, Side::Light, "the throw said throw again");
        // A rosette: no throw again from it, but a fine.
        let out = p.play(&rules, Move { from: 1, to: 4 }, false);
        assert!(out.fine && !out.again);
        assert_eq!(p.to_move, Side::Dark);
        // The middle rosette is not safe.
        assert!(!rules.is_safe_pos(8));
    }

    #[test]
    fn exact_throw_to_bear_off() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = vec![13, 15, 15, 15, 15, 15, 15];
        assert_eq!(p.moves(&rules, 2), vec![Move { from: 13, to: 15 }]);
        assert!(p.moves(&rules, 3).is_empty());
        let out = p.play(&rules, Move { from: 13, to: 15 }, false);
        assert!(out.won && out.home);
        assert_eq!(p.winner(), Some(Side::Light));
    }

    #[test]
    fn rosette_throws_again_and_middle_one_is_safe() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[1] = vec![8, 0, 0, 0, 0, 0, 0];
        p.pieces[1].sort();
        p.pieces[0] = vec![6, 0, 0, 0, 0, 0, 0];
        p.pieces[0].sort();
        // Dark sits on the middle rosette: Light cannot land there.
        assert!(!p.moves(&rules, 2).iter().any(|m| m.to == 8));
        // Landing on the corner rosette throws again.
        let out = p.play(&rules, Move { from: 0, to: 4 }, false);
        assert!(out.again);
        assert_eq!(p.to_move, Side::Light);

        // Without the safe rosette, Light takes Dark there.
        let bm = Rules {
            pieces: 7,
            ..Rules::british_museum()
        };
        let mut q = Position::new(&bm, Side::Light);
        q.pieces[1] = vec![0, 0, 0, 0, 0, 0, 8];
        q.pieces[0] = vec![0, 0, 0, 0, 0, 0, 6];
        let out = q.play(&bm, Move { from: 6, to: 8 }, false);
        assert!(out.took && out.again);
        assert_eq!(q.count_at(Side::Dark, START), 7);
    }

    #[test]
    fn taking_only_on_the_shared_row() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        // Both at their own position 3: different squares, nothing taken.
        p.pieces[0] = vec![0, 0, 0, 0, 0, 0, 1];
        p.pieces[1] = vec![0, 0, 0, 0, 0, 0, 3];
        let out = p.play(&rules, Move { from: 1, to: 3 }, false);
        assert!(!out.took);
        assert_eq!(p.count_at(Side::Dark, 3), 1);
        // On the shared row a piece is taken.
        let mut q = Position::new(&rules, Side::Light);
        q.pieces[0] = vec![0, 0, 0, 0, 0, 0, 5];
        q.pieces[1] = vec![0, 0, 0, 0, 0, 0, 7];
        let out = q.play(&rules, Move { from: 5, to: 7 }, false);
        assert!(out.took && !out.again);
        assert_eq!(q.to_move, Side::Dark);
    }

    #[test]
    fn waiting_pieces_give_one_move() {
        let rules = Rules::finkel_2017();
        let p = Position::new(&rules, Side::Dark);
        assert_eq!(p.moves(&rules, 3), vec![Move { from: 0, to: 3 }]);
        assert!(p.moves(&rules, 0).is_empty());
    }
}
