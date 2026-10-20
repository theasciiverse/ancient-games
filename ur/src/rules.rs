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

/// The throwing lots, and what each count of marked tips means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lots {
    /// Four tetrahedra, each with two of four tips marked: the count of
    /// marked tips up, 0 to 4, is the move; 0 loses the turn.
    FourBinary,
}

impl Lots {
    /// How many lots are thrown.
    pub fn count(self) -> u32 {
        match self {
            Lots::FourBinary => 4,
        }
    }

    /// The move for `marked` tips up.
    pub fn moves(self, marked: u32) -> u8 {
        match self {
            Lots::FourBinary => marked as u8,
        }
    }

    /// Every possible move with its chance out of `2^count`.
    pub fn distribution(self) -> Vec<(u8, u32)> {
        let n = self.count();
        (0..=n).map(|k| (self.moves(k), binomial(n, k))).collect()
    }
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
    /// Landing on any rosette gives another throw.
    pub rosette_throws_again: bool,
    /// A piece on the middle rosette cannot be taken.
    pub middle_rosette_safe: bool,
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
            rosette_throws_again: true,
            middle_rosette_safe: true,
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
            rosette_throws_again: true,
            middle_rosette_safe: false,
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![Rules::finkel_2017(), Rules::british_museum()]
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
            let to = from + roll;
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
    /// throws again.
    pub fn play(&mut self, rules: &Rules, mv: Move) -> Outcome {
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

        outcome.again =
            !outcome.won && rules.rosette_throws_again && rules.is_rosette_pos(mv.to);
        if !outcome.again {
            self.to_move = side.other();
        }
        outcome
    }

    /// Passes the turn (a throw of no use).
    pub fn pass(&mut self) {
        self.to_move = self.to_move.other();
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
        assert_eq!(
            Lots::FourBinary.distribution(),
            vec![(0, 1), (1, 4), (2, 6), (3, 4), (4, 1)]
        );
    }

    #[test]
    fn exact_throw_to_bear_off() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = vec![13, 15, 15, 15, 15, 15, 15];
        assert_eq!(p.moves(&rules, 2), vec![Move { from: 13, to: 15 }]);
        assert!(p.moves(&rules, 3).is_empty());
        let out = p.play(&rules, Move { from: 13, to: 15 });
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
        let out = p.play(&rules, Move { from: 0, to: 4 });
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
        let out = q.play(&bm, Move { from: 6, to: 8 });
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
        let out = p.play(&rules, Move { from: 1, to: 3 });
        assert!(!out.took);
        assert_eq!(p.count_at(Side::Dark, 3), 1);
        // On the shared row a piece is taken.
        let mut q = Position::new(&rules, Side::Light);
        q.pieces[0] = vec![0, 0, 0, 0, 0, 0, 5];
        q.pieces[1] = vec![0, 0, 0, 0, 0, 0, 7];
        let out = q.play(&rules, Move { from: 5, to: 7 });
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
