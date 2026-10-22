//! The rules of the Royal Game of Ur: the boards, the paths, the throws and
//! the moves, for each rule set the game offers.
//!
//! Two boards. The third-millennium one from the royal tombs of Ur: a 3x8
//! grid without the fifth and sixth squares of the outer rows. And the later
//! Game of Twenty Squares of the second and first millennia BC, as Irving
//! Finkel draws it for the tablet's rules (his Fig. 3.4): a 3x4 block with
//! the middle row running on for eight squares more. Row 0 is the far side
//! (Dark's own row), row 1 the shared middle row, row 2 the near side
//! (Light's).

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

    /// The side's own row.
    pub fn row(self) -> usize {
        match self {
            Side::Light => 2,
            Side::Dark => 0,
        }
    }
}

/// Rows of every board.
pub const ROWS: usize = 3;

/// The position of a piece that has not entered.
pub const START: u8 = 0;

/// The board and the path a piece takes over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Board {
    /// The board from the royal tombs, with Bell's path: four squares on the
    /// own row leftwards from the fourth, the whole middle row rightwards,
    /// the last two squares of the own row leftwards, and off.
    Ur,
    /// The later Game of Twenty Squares with Finkel's path: the four squares
    /// of the own row leftwards (houses 1 to 4), then the twelve of the
    /// middle row (houses 5 to 16), and off the end.
    TwentySquares,
}

impl Board {
    pub fn cols(self) -> usize {
        match self {
            Board::Ur => 8,
            Board::TwentySquares => 12,
        }
    }

    /// Whether a grid square is part of the board.
    pub fn on_board(self, row: usize, col: usize) -> bool {
        if row >= ROWS || col >= self.cols() {
            return false;
        }
        match self {
            Board::Ur => row == 1 || !(col == 4 || col == 5),
            Board::TwentySquares => row == 1 || col < 4,
        }
    }

    /// The rosettes. Ur: first from the left and second from the right on
    /// the outer rows, fourth from the left in the middle. Twenty Squares:
    /// the corners of the outer rows and every fourth house of the middle
    /// row (8, 12 and 16).
    pub fn is_rosette(self, row: usize, col: usize) -> bool {
        if !self.on_board(row, col) {
            return false;
        }
        match (self, row) {
            (Board::Ur, 0 | 2) => col == 0 || col == 6,
            (Board::Ur, _) => col == 3,
            (Board::TwentySquares, 0 | 2) => col == 0,
            (Board::TwentySquares, _) => col % 4 == 3,
        }
    }

    /// Squares on the path, not counting the start and home.
    pub fn path_len(self) -> u8 {
        match self {
            Board::Ur => 14,
            Board::TwentySquares => 16,
        }
    }

    /// The position of a piece that has gone home.
    pub fn home(self) -> u8 {
        self.path_len() + 1
    }

    /// The grid square of path position `pos` for `side`.
    pub fn square(self, side: Side, pos: u8) -> (usize, usize) {
        let own = side.row();
        match (self, pos) {
            (_, 1..=4) => (own, 4 - pos as usize),
            (Board::Ur, 5..=12) => (1, pos as usize - 5),
            (Board::Ur, 13) => (own, 7),
            (Board::Ur, 14) => (own, 6),
            (Board::TwentySquares, 5..=16) => (1, pos as usize - 5),
            _ => panic!("position {pos} is not on the board"),
        }
    }

    /// The path position for `side` at a grid square, if it is on the path.
    pub fn pos_at(self, side: Side, row: usize, col: usize) -> Option<u8> {
        (1..=self.path_len()).find(|&p| self.square(side, p) == (row, col))
    }

    /// Whether path position `pos` is on the shared middle row.
    pub fn is_shared(self, pos: u8) -> bool {
        match self {
            Board::Ur => (5..=12).contains(&pos),
            Board::TwentySquares => (5..=16).contains(&pos),
        }
    }

    /// Whether path position `pos` is a rosette.
    pub fn is_rosette_pos(self, pos: u8) -> bool {
        (1..=self.path_len()).contains(&pos) && {
            let (row, col) = self.square(Side::Light, pos);
            self.is_rosette(row, col)
        }
    }
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
    /// The tablet's two knucklebones as Finkel reads them: a sheep astragal
    /// giving 1 to 4, and an ox astragal thrown after it if the player
    /// dares, whose "yes" turns 1, 2, 3, 4 into 5, 6, 7, 10 and whose "no"
    /// wastes the throw.
    Astragals,
}

impl Lots {
    /// How many lots are shown (for the knucklebones, the two bones).
    pub fn count(self) -> u32 {
        match self {
            Lots::FourBinary => 4,
            Lots::BellThree => 3,
            Lots::Astragals => 2,
        }
    }

    /// The throw for `marked` tips up (for the knucklebones, the sheep
    /// bone's face, 1 to 4, is passed as `marked`).
    pub fn throw(self, marked: u32) -> Throw {
        match self {
            Lots::FourBinary => Throw { moves: marked as u8, again: false },
            Lots::BellThree => match marked {
                3 => Throw { moves: 5, again: true },
                0 => Throw { moves: 4, again: true },
                2 => Throw { moves: 1, again: true },
                _ => Throw { moves: 0, again: false },
            },
            Lots::Astragals => Throw { moves: marked as u8, again: false },
        }
    }

    /// The sheep bone's score after the ox bone says yes.
    pub fn doubled(sheep: u8) -> u8 {
        match sheep {
            1 => 5,
            2 => 6,
            3 => 7,
            _ => 10,
        }
    }

    /// Every possible throw with its chance out of the total of the chances.
    /// For the knucklebones, the sheep bone alone, each face alike.
    pub fn distribution(self) -> Vec<(Throw, u32)> {
        match self {
            Lots::Astragals => (1..=4).map(|k| (self.throw(k), 1)).collect(),
            _ => {
                let n = self.count();
                (0..=n).map(|k| (self.throw(k), binomial(n, k))).collect()
            }
        }
    }
}

fn binomial(n: u32, k: u32) -> u32 {
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
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
    /// The tablet: each of the five pieces has its own throw. The Swallow
    /// comes on with 2 onto the first rosette (house 4); the Storm-bird,
    /// Raven, Rooster and Eagle with 5, 6, 7 and 10 onto that house.
    /// Finkel offers a second reading for the Swallow, the square before any
    /// rosette; in a race that lets it skip nearly the whole board, so the
    /// first is used.
    ByPiece,
}

/// The throws that bring the tablet's five pieces on.
pub const TABLET_ENTRY: [u8; 5] = [2, 5, 6, 7, 10];

/// The tablet's five pieces, and short tags that tell them apart.
pub const TABLET_PIECES: [&str; 5] = ["Swallow", "Storm-bird", "Raven", "Rooster", "Eagle"];
pub const TABLET_TAGS: [&str; 5] = ["Sw", "St", "Ra", "Ro", "Ea"];

/// What landing on a rosette does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rosettes {
    /// The side throws again.
    ThrowAgain,
    /// The opponent pays a fine into the pool (R. C. Bell).
    OpponentPays,
    /// The tablet as Finkel reads it: a piece landing on a rosette takes
    /// tokens from the pool, a piece passing one pays them in; each piece
    /// its own value (Finkel's Table 3.1: 3, 4, 4, 4, 5).
    Tokens,
}

/// Tokens a tablet piece wins on a rosette or pays for passing one.
pub const TABLET_TOKENS: [u32; 5] = [3, 4, 4, 4, 5];

/// A betting game: each side's purse at the start of a session, what each
/// side puts into the pool at the start of a game, and the fine for a
/// rosette where the opponent pays. The winner takes the pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stakes {
    pub purse: u32,
    pub stake: u32,
    pub fine: u32,
}

/// A rule set, as plain data, so the engine serves every version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where the rules come from, in a sentence.
    pub source: &'static str,
    pub board: Board,
    pub pieces: u8,
    pub lots: Lots,
    pub entry: Entry,
    pub rosettes: Rosettes,
    /// A piece on a rosette of the middle row cannot be taken.
    pub rosettes_safe: bool,
    pub stakes: Option<Stakes>,
    /// The heading of this rule set in the rulebook chapter (in HELP.txt and RULES.txt),
    /// where the help opens.
    pub section: &'static str,
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
            board: Board::Ur,
            pieces: 7,
            lots: Lots::FourBinary,
            entry: Entry::ByThrow,
            rosettes: Rosettes::ThrowAgain,
            rosettes_safe: true,
            stakes: None,
            section: "Finkel 2017",
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
            board: Board::Ur,
            pieces: 5,
            lots: Lots::FourBinary,
            entry: Entry::ByThrow,
            rosettes: Rosettes::ThrowAgain,
            rosettes_safe: false,
            stakes: None,
            section: "British Museum 1990s",
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
            board: Board::Ur,
            pieces: 7,
            lots: Lots::BellThree,
            entry: Entry::OnFive,
            rosettes: Rosettes::OpponentPays,
            rosettes_safe: false,
            stakes: Some(Stakes { purse: 20, stake: 5, fine: 1 }),
            section: "R. C. Bell's betting game",
        }
    }

    /// The Babylonian game of the tablet BM 33333B (177-176 BC), as Irving
    /// Finkel reconstructs it in "On the Rules for the Royal Game of Ur"
    /// (Ancient Board Games in Perspective, 2007).
    pub fn tablet() -> Rules {
        Rules {
            id: "tablet",
            name: "The Babylonian tablet",
            source: "The game on the clay tablet the scribe Itti-Marduk-balatu \
                     wrote in Babylon in 177 BC, as Irving Finkel reconstructs \
                     it: five birds each, two knucklebones, the later board of \
                     twenty squares, and tokens won and lost on the rosettes.",
            board: Board::TwentySquares,
            pieces: 5,
            lots: Lots::Astragals,
            entry: Entry::ByPiece,
            rosettes: Rosettes::Tokens,
            rosettes_safe: true,
            stakes: Some(Stakes { purse: 40, stake: 5, fine: 0 }),
            section: "The Babylonian tablet",
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![
            Rules::finkel_2017(),
            Rules::british_museum(),
            Rules::bell(),
            Rules::tablet(),
        ]
    }

    /// The position of a piece that has gone home.
    pub fn home(&self) -> u8 {
        self.board.home()
    }

    /// Whether `pos` on the path is a rosette.
    pub fn is_rosette_pos(&self, pos: u8) -> bool {
        self.board.is_rosette_pos(pos)
    }

    /// Whether a piece at `pos` can be taken there.
    pub fn is_safe_pos(&self, pos: u8) -> bool {
        !self.board.is_shared(pos) || (self.rosettes_safe && self.is_rosette_pos(pos))
    }

    /// Whether the pieces are told apart (the tablet's birds).
    pub fn distinct(&self) -> bool {
        self.entry == Entry::ByPiece
    }

    /// The name of piece `piece`, for rule sets with distinct pieces.
    pub fn piece_name(&self, piece: usize) -> Option<&'static str> {
        self.distinct().then(|| TABLET_PIECES[piece])
    }
}

/// A move: piece `piece` of the side to move goes from `from` to `to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub piece: usize,
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
    /// The move landed on a rosette.
    pub rosette: bool,
    /// Rosettes the piece passed over without landing on them.
    pub passed: u8,
    /// A piece went home.
    pub home: bool,
    /// The side has brought every piece home.
    pub won: bool,
}

/// The position: where every piece is and whose turn it is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Path positions of each side's pieces, by piece.
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

    /// Whether an opposing piece stands on the shared square at `pos`.
    /// Shared squares are the same squares for both sides at the same
    /// position.
    fn opponent_on(&self, rules: &Rules, side: Side, pos: u8) -> bool {
        rules.board.is_shared(pos) && self.of(side.other()).contains(&pos)
    }

    /// Where piece `piece` at the start can come on with `roll`, if at all.
    fn entries(rules: &Rules, piece: usize, roll: u8) -> Vec<u8> {
        match rules.entry {
            Entry::ByThrow => vec![roll],
            Entry::OnFive if roll == 5 => vec![1],
            Entry::OnFive => Vec::new(),
            Entry::ByPiece if roll != TABLET_ENTRY[piece] => Vec::new(),
            // The Swallow: onto the first rosette.
            Entry::ByPiece if piece == 0 => vec![4],
            Entry::ByPiece => vec![roll],
        }
    }

    /// The legal moves for the side to move with a throw of `roll`. Pieces
    /// that are alike give one move per starting square.
    pub fn moves(&self, rules: &Rules, roll: u8) -> Vec<Move> {
        if roll == 0 {
            return Vec::new();
        }
        let side = self.to_move;
        let home = rules.home();
        let mut moves: Vec<Move> = Vec::new();
        for (piece, &from) in self.of(side).iter().enumerate() {
            if from == home {
                continue;
            }
            if !rules.distinct() && moves.iter().any(|m| m.from == from) {
                continue;
            }
            let targets = if from == START {
                Position::entries(rules, piece, roll)
            } else {
                vec![from + roll]
            };
            for to in targets {
                // Off the board by an exact throw only.
                if to > home {
                    continue;
                }
                if to < home {
                    // One piece per square: not onto your own piece, and not
                    // onto an opposing piece that is safe where it stands.
                    if self.of(side).contains(&to) {
                        continue;
                    }
                    if self.opponent_on(rules, side, to) && rules.is_safe_pos(to) {
                        continue;
                    }
                }
                moves.push(Move { piece, from, to });
            }
        }
        moves
    }

    /// Plays `mv` for the side to move, and passes the turn unless the side
    /// throws again, by a rosette or because the throw (`keep`) says so.
    pub fn play(&mut self, rules: &Rules, mv: Move, keep: bool) -> Outcome {
        let side = self.to_move;
        let home = rules.home();
        let mut outcome = Outcome::default();

        self.pieces[side.index()][mv.piece] = mv.to;

        if mv.to == home {
            outcome.home = true;
            outcome.won = self.of(side).iter().all(|&p| p == home);
        } else if self.opponent_on(rules, side, mv.to) {
            let theirs = &mut self.pieces[side.other().index()];
            let j = theirs.iter().position(|&p| p == mv.to).unwrap();
            theirs[j] = START;
            outcome.took = true;
        }

        outcome.rosette = mv.to != home && rules.is_rosette_pos(mv.to);
        if mv.from != START {
            outcome.passed = (mv.from + 1..mv.to.min(home))
                .filter(|&p| rules.is_rosette_pos(p))
                .count() as u8;
        }
        outcome.again =
            !outcome.won && outcome.rosette && rules.rosettes == Rosettes::ThrowAgain;
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
    pub fn winner(&self, rules: &Rules) -> Option<Side> {
        let home = rules.home();
        [Side::Light, Side::Dark]
            .into_iter()
            .find(|&s| self.of(s).iter().all(|&p| p == home))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squares(board: Board) -> usize {
        (0..ROWS)
            .flat_map(|r| (0..board.cols()).map(move |c| (r, c)))
            .filter(|&(r, c)| board.on_board(r, c))
            .count()
    }

    fn rosettes(board: Board) -> usize {
        (0..ROWS)
            .flat_map(|r| (0..board.cols()).map(move |c| (r, c)))
            .filter(|&(r, c)| board.is_rosette(r, c))
            .count()
    }

    #[test]
    fn both_boards_have_twenty_squares_and_five_rosettes() {
        for board in [Board::Ur, Board::TwentySquares] {
            assert_eq!(squares(board), 20, "{board:?}");
            assert_eq!(rosettes(board), 5, "{board:?}");
        }
    }

    #[test]
    fn paths_visit_distinct_squares_on_the_board() {
        for board in [Board::Ur, Board::TwentySquares] {
            for side in [Side::Light, Side::Dark] {
                let squares: Vec<_> = (1..=board.path_len()).map(|p| board.square(side, p)).collect();
                for &(r, c) in &squares {
                    assert!(board.on_board(r, c), "{board:?} {side:?} {r},{c}");
                }
                let mut unique = squares.clone();
                unique.sort();
                unique.dedup();
                assert_eq!(unique.len(), board.path_len() as usize);
            }
        }
        let rosettes: Vec<u8> = (1..=14).filter(|&p| Board::Ur.is_rosette_pos(p)).collect();
        assert_eq!(rosettes, vec![4, 8, 14]);
        // Finkel's Fig. 3.4: every fourth house.
        let rosettes: Vec<u8> = (1..=16).filter(|&p| Board::TwentySquares.is_rosette_pos(p)).collect();
        assert_eq!(rosettes, vec![4, 8, 12, 16]);
    }

    #[test]
    fn sides_share_the_middle_row_only() {
        for board in [Board::Ur, Board::TwentySquares] {
            for pos in 1..=board.path_len() {
                let same = board.square(Side::Light, pos) == board.square(Side::Dark, pos);
                assert_eq!(same, board.is_shared(pos), "{board:?} position {pos}");
            }
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
        let d = Lots::BellThree.distribution();
        assert_eq!(d[0], (Throw { moves: 4, again: true }, 1));
        assert_eq!(d[1], (Throw { moves: 0, again: false }, 3));
        assert_eq!(d[2], (Throw { moves: 1, again: true }, 3));
        assert_eq!(d[3], (Throw { moves: 5, again: true }, 1));
    }

    fn at(pieces: &[u8]) -> Vec<u8> {
        pieces.to_vec()
    }

    #[test]
    fn exact_throw_to_bear_off() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = at(&[13, 15, 15, 15, 15, 15, 15]);
        assert_eq!(p.moves(&rules, 2), vec![Move { piece: 0, from: 13, to: 15 }]);
        assert!(p.moves(&rules, 3).is_empty());
        let out = p.play(&rules, Move { piece: 0, from: 13, to: 15 }, false);
        assert!(out.won && out.home);
        assert_eq!(p.winner(&rules), Some(Side::Light));
    }

    #[test]
    fn rosette_throws_again_and_middle_one_is_safe() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[1] = at(&[8, 0, 0, 0, 0, 0, 0]);
        p.pieces[0] = at(&[6, 0, 0, 0, 0, 0, 0]);
        assert!(!p.moves(&rules, 2).iter().any(|m| m.to == 8));
        let out = p.play(&rules, Move { piece: 1, from: 0, to: 4 }, false);
        assert!(out.again && out.rosette);
        assert_eq!(p.to_move, Side::Light);

        let bm = Rules { pieces: 7, ..Rules::british_museum() };
        let mut q = Position::new(&bm, Side::Light);
        q.pieces[1] = at(&[0, 0, 0, 0, 0, 0, 8]);
        q.pieces[0] = at(&[0, 0, 0, 0, 0, 0, 6]);
        let out = q.play(&bm, Move { piece: 6, from: 6, to: 8 }, false);
        assert!(out.took && out.again);
        assert_eq!(q.count_at(Side::Dark, START), 7);
    }

    #[test]
    fn taking_only_on_the_shared_row() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = at(&[0, 0, 0, 0, 0, 0, 1]);
        p.pieces[1] = at(&[0, 0, 0, 0, 0, 0, 3]);
        let out = p.play(&rules, Move { piece: 6, from: 1, to: 3 }, false);
        assert!(!out.took);
        assert_eq!(p.count_at(Side::Dark, 3), 1);
        let mut q = Position::new(&rules, Side::Light);
        q.pieces[0] = at(&[0, 0, 0, 0, 0, 0, 5]);
        q.pieces[1] = at(&[0, 0, 0, 0, 0, 0, 7]);
        let out = q.play(&rules, Move { piece: 6, from: 5, to: 7 }, false);
        assert!(out.took && !out.again);
        assert_eq!(q.to_move, Side::Dark);
    }

    #[test]
    fn waiting_pieces_give_one_move() {
        let rules = Rules::finkel_2017();
        let p = Position::new(&rules, Side::Dark);
        assert_eq!(p.moves(&rules, 3), vec![Move { piece: 0, from: 0, to: 3 }]);
        assert!(p.moves(&rules, 0).is_empty());
    }

    #[test]
    fn bell_enters_on_five_and_fines_on_rosettes() {
        let rules = Rules::bell();
        let mut p = Position::new(&rules, Side::Light);
        assert!(p.moves(&rules, 4).is_empty());
        assert!(p.moves(&rules, 1).is_empty());
        assert_eq!(p.moves(&rules, 5), vec![Move { piece: 0, from: 0, to: 1 }]);
        let out = p.play(&rules, Move { piece: 0, from: 0, to: 1 }, true);
        assert!(!out.again && !out.rosette);
        assert_eq!(p.to_move, Side::Light, "the throw said throw again");
        let out = p.play(&rules, Move { piece: 0, from: 1, to: 4 }, false);
        assert!(out.rosette && !out.again);
        assert_eq!(p.to_move, Side::Dark);
        assert!(!rules.is_safe_pos(8));
    }

    #[test]
    fn tablet_birds_enter_on_their_own_throws() {
        let rules = Rules::tablet();
        let p = Position::new(&rules, Side::Light);
        // The Swallow with 2: onto the first rosette.
        let swallow: Vec<u8> = p.moves(&rules, 2).iter().map(|m| m.to).collect();
        assert_eq!(swallow, vec![4]);
        assert!(p.moves(&rules, 2).iter().all(|m| m.piece == 0));
        // The Storm-bird on 5, the Raven on 6, the Rooster on 7, the Eagle on 10.
        for (piece, roll) in [(1, 5), (2, 6), (3, 7), (4, 10)] {
            assert_eq!(
                p.moves(&rules, roll),
                vec![Move { piece, from: 0, to: roll }],
                "roll {roll}"
            );
        }
        // Nothing comes on with 1, 3 or 4.
        for roll in [1, 3, 4] {
            assert!(p.moves(&rules, roll).is_empty());
        }
    }

    #[test]
    fn tablet_counts_rosettes_passed_and_keeps_rosettes_safe() {
        let rules = Rules::tablet();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = at(&[0, 5, 0, 0, 0]);
        p.pieces[1] = at(&[0, 0, 8, 0, 0]);
        // The Raven on house 8 is safe: no landing there.
        assert!(!p.moves(&rules, 3).iter().any(|m| m.to == 8));
        // From 5 by 7 to 12: passes the rosette on 8, lands on 12.
        let out = p.play(&rules, Move { piece: 1, from: 5, to: 12 }, false);
        assert_eq!(out.passed, 1);
        assert!(out.rosette && !out.again);
        assert_eq!(Lots::doubled(1), 5);
        assert_eq!(Lots::doubled(4), 10);
    }
}
