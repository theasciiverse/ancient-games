//! The rules of the morris games the program offers, as data, and one
//! engine that plays them all.
//!
//! Every morris game is played on points joined by lines. The players
//! enter their men one at a time on empty points; in most games they then
//! move them along the lines. Three men of one colour on one line make a
//! mill. In Three Men's Morris a mill wins the game; in the larger games
//! it lets its maker take one of the other side's men. Where a source is
//! silent, the rulebook chapter says what this game does ("Our choice").
//!
//! A board is a list of points (row and column on a square grid, row 0 at
//! the top) and a list of lines through them: neighbours on a line are
//! joined, and every line of three points is a mill. The men of each side
//! are a bit set over the points.

/// The two sides. White moves first.
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

    pub const BOTH: [Side; 2] = [Side::White, Side::Black];
}

/// The boards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Nine points in a square of three by three, joined along the rows
    /// and columns, and along both diagonals if `diagonals`.
    Three { diagonals: bool },
    /// Two squares, one inside the other, their sides' midpoints joined:
    /// 16 points.
    Six,
    /// Three squares, one inside the other, their sides' midpoints joined:
    /// 24 points.
    Nine,
    /// The board of Nine with the corners joined too, along the diagonals.
    Twelve,
}

/// A board: its points and lines, and what follows from them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    /// The grid the points sit on: rows and columns 0 to `size` - 1.
    pub size: u8,
    /// Each point's row (from the top) and column.
    pub points: Vec<(u8, u8)>,
    /// The lines, each a run of points in order along it.
    pub lines: Vec<Vec<usize>>,
    /// Each point's neighbours, as a bit set.
    pub adjacent: Vec<u32>,
    /// The mills, as bit sets, and for each point the mills through it.
    pub mills: Vec<u32>,
    pub mills_at: Vec<Vec<u32>>,
}

impl Board {
    pub fn new(layout: Layout) -> Board {
        let (size, lines): (u8, Vec<Vec<(u8, u8)>>) = match layout {
            Layout::Three { diagonals } => {
                let mut lines = Vec::new();
                for i in 0..3 {
                    lines.push((0..3).map(|c| (i, c)).collect());
                    lines.push((0..3).map(|r| (r, i)).collect());
                }
                if diagonals {
                    lines.push(vec![(0, 0), (1, 1), (2, 2)]);
                    lines.push(vec![(0, 2), (1, 1), (2, 0)]);
                }
                (3, lines)
            }
            Layout::Six => {
                let mut lines = squares(4, 2);
                lines.push(vec![(0, 2), (1, 2)]);
                lines.push(vec![(2, 0), (2, 1)]);
                lines.push(vec![(2, 3), (2, 4)]);
                lines.push(vec![(3, 2), (4, 2)]);
                (5, lines)
            }
            Layout::Nine | Layout::Twelve => {
                let mut lines = squares(6, 3);
                lines.push(vec![(0, 3), (1, 3), (2, 3)]);
                lines.push(vec![(3, 0), (3, 1), (3, 2)]);
                lines.push(vec![(3, 4), (3, 5), (3, 6)]);
                lines.push(vec![(4, 3), (5, 3), (6, 3)]);
                if layout == Layout::Twelve {
                    lines.push(vec![(0, 0), (1, 1), (2, 2)]);
                    lines.push(vec![(0, 6), (1, 5), (2, 4)]);
                    lines.push(vec![(4, 2), (5, 1), (6, 0)]);
                    lines.push(vec![(4, 4), (5, 5), (6, 6)]);
                }
                (7, lines)
            }
        };
        // The points, row by row from the top.
        let mut points: Vec<(u8, u8)> = lines.iter().flatten().copied().collect();
        points.sort_unstable();
        points.dedup();
        let at = |p: (u8, u8)| points.iter().position(|&q| q == p).expect("a point");
        let lines: Vec<Vec<usize>> = lines.iter().map(|l| l.iter().map(|&p| at(p)).collect()).collect();
        let mut adjacent = vec![0u32; points.len()];
        for line in &lines {
            for w in line.windows(2) {
                adjacent[w[0]] |= 1 << w[1];
                adjacent[w[1]] |= 1 << w[0];
            }
        }
        let mills: Vec<u32> =
            lines.iter().filter(|l| l.len() == 3).map(|l| l.iter().fold(0, |m, &p| m | 1 << p)).collect();
        let mills_at = (0..points.len()).map(|p| mills.iter().copied().filter(|m| m & 1 << p != 0).collect()).collect();
        Board { size, points, lines, adjacent, mills, mills_at }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Every point, as a bit set.
    pub fn all(&self) -> u32 {
        (1u32 << self.len()) - 1
    }

    /// The point at a row and column, if there is one.
    pub fn point_at(&self, row: u8, col: u8) -> Option<usize> {
        self.points.iter().position(|&q| q == (row, col))
    }

    /// A point's name: the column as a letter from the left, the row as a
    /// number from the bottom, as players of Nine Men's Morris write them
    /// (a1 is the bottom left corner).
    pub fn name(&self, p: usize) -> String {
        let (r, c) = self.points[p];
        format!("{}{}", (b'a' + c) as char, self.size - r)
    }

    /// The points in some mill of `men` (all of them men of one side).
    pub fn in_mills(&self, men: u32) -> u32 {
        self.mills.iter().filter(|&&m| men & m == m).fold(0, |a, &m| a | m)
    }
}

/// The sides of `count` squares, the outermost `size` across, each next
/// one inside by one point on every side; each side a line of three.
fn squares(size: u8, count: u8) -> Vec<Vec<(u8, u8)>> {
    let mut lines = Vec::new();
    for i in 0..count {
        let (lo, hi) = (i, size - i);
        let mid = size / 2;
        lines.push(vec![(lo, lo), (lo, mid), (lo, hi)]);
        lines.push(vec![(hi, lo), (hi, mid), (hi, hi)]);
        lines.push(vec![(lo, lo), (mid, lo), (hi, lo)]);
        lines.push(vec![(lo, hi), (mid, hi), (hi, hi)]);
    }
    lines
}

/// Which of the other side's men a mill may take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taking {
    /// Any man at all, in a mill or not.
    Any,
    /// Never one in a mill; if every one is, nothing is taken.
    NeverFromMills,
    /// Not one in a mill while there is any other; if all are in mills,
    /// any.
    FromMillsLast,
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where it comes from, in a sentence or two, for the menu.
    pub source: &'static str,
    /// The heading of this rule set in the rulebook chapter (in HELP.txt
    /// and RULES.txt), where the help opens.
    pub section: &'static str,
    pub layout: Layout,
    /// Men each side enters.
    pub men: u8,
    /// Whether the men move along the lines once all are entered; if not,
    /// the game ends when they are.
    pub moving: bool,
    /// Whether a mill wins the game (Three Men's Morris) rather than
    /// taking a man.
    pub mill_wins: bool,
    /// A side down to this many men may move a man to any empty point.
    pub flying: Option<u8>,
    pub taking: Taking,
    /// Men taken when one move makes two mills at once.
    pub double_mill_takes: u8,
    /// Whether a board filled while entering ends the game drawn (twelve
    /// men each, 24 men for 24 points).
    pub full_board_draws: bool,
    /// Whether a man that left a mill to make a new one may not go
    /// straight back on its side's next move to make a mill again
    /// (Morabaraba).
    pub no_return: bool,
    /// Moves (of both sides together) without a man taken, once a side is
    /// down to three, that end the game drawn.
    pub quiet_draw: Option<u8>,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![
            Rules::nine_modern(),
            Rules::nine_bell(),
            Rules::morabaraba(),
            Rules::six_bell(),
            Rules::three_moving(),
            Rules::three_placing(),
        ]
    }

    /// Nine Men's Morris as played in clubs and tournaments today.
    pub fn nine_modern() -> Rules {
        Rules {
            id: "nine",
            name: "Nine Men's Morris, today's rules",
            source: "The World Mill Association's rules (2008), and Ralph Gasser's, who \
                     solved the game (1996): down to three men, a side may fly.",
            section: "Nine Men's Morris, today's rules",
            layout: Layout::Nine,
            men: 9,
            moving: true,
            mill_wins: false,
            flying: Some(3),
            taking: Taking::FromMillsLast,
            double_mill_takes: 1,
            full_board_draws: false,
            no_return: false,
            quiet_draw: None,
        }
    }

    /// Nine Men's Morris as R. C. Bell gives it: no flying.
    pub fn nine_bell() -> Rules {
        Rules {
            id: "nine-bell",
            name: "Nine Men's Morris after Bell",
            source: "R. C. Bell, Board and Table Games from Many Civilizations (1960): nine \
                     men each and no flying; a side blocked, or down to two men, loses.",
            section: "Nine Men's Morris after Bell",
            flying: None,
            taking: Taking::NeverFromMills,
            ..Rules::nine_modern()
        }
    }

    /// Morabaraba, twelve men each on the board with diagonals, as Mind
    /// Sports South Africa gives its rules.
    pub fn morabaraba() -> Rules {
        Rules {
            id: "morabaraba",
            name: "Morabaraba, Mind Sports South Africa",
            source: "The Generally Accepted Rules of Mind Sports South Africa (2007): twelve \
                     cows each on the board with diagonals, mills along them too.",
            section: "Morabaraba, Mind Sports South Africa",
            layout: Layout::Twelve,
            men: 12,
            full_board_draws: true,
            no_return: true,
            quiet_draw: Some(20),
            ..Rules::nine_modern()
        }
    }

    /// Six Men's Morris as R. C. Bell gives it.
    pub fn six_bell() -> Rules {
        Rules {
            id: "six",
            name: "Six Men's Morris after Bell",
            source: "R. C. Bell (1960): the medieval game of two squares, six men each, \
                     mills only along the squares' sides; down to two men, a side loses.",
            section: "Six Men's Morris after Bell",
            layout: Layout::Six,
            men: 6,
            flying: None,
            taking: Taking::Any,
            ..Rules::nine_modern()
        }
    }

    /// Three Men's Morris, Bell's second form: three men each, entered and
    /// then moved, on the board with diagonals; a line of three wins.
    pub fn three_moving() -> Rules {
        Rules {
            id: "three",
            name: "Three Men's Morris after Bell",
            source: "R. C. Bell (1960), and Alfonso X's alquerque de tres (1283): three men \
                     each on nine points with diagonals, entered, then moved; a line wins.",
            section: "Three Men's Morris after Bell",
            layout: Layout::Three { diagonals: true },
            men: 3,
            moving: true,
            mill_wins: true,
            flying: None,
            taking: Taking::FromMillsLast,
            double_mill_takes: 1,
            full_board_draws: false,
            no_return: false,
            quiet_draw: None,
        }
    }

    /// Three Men's Morris, Bell's first form: four counters each, placed
    /// only, as in noughts and crosses.
    pub fn three_placing() -> Rules {
        Rules {
            id: "three-placing",
            name: "Three Men's Morris, placing only",
            source: "R. C. Bell (1960), his first form: nine points without diagonals and \
                     four counters each, only placed; three in a line wins.",
            section: "Three Men's Morris, placing only",
            layout: Layout::Three { diagonals: false },
            men: 4,
            moving: false,
            ..Rules::three_moving()
        }
    }

    pub fn board(&self) -> Board {
        Board::new(self.layout)
    }
}

/// A move: a man entered (`from` is `None`) or moved, and the men of the
/// other side its mill takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Option<u8>,
    pub to: u8,
    /// Up to two men taken; `None` for none.
    pub take: [Option<u8>; 2],
}

impl Move {
    pub fn taken(&self) -> impl Iterator<Item = usize> + '_ {
        self.take.iter().flatten().map(|&p| p as usize)
    }
}

/// How a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won(Side, Why),
    Drawn(Why),
}

/// Why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// Three in a line, where that wins.
    Line,
    /// The loser is down to two men.
    TwoLeft,
    /// The loser cannot move.
    Blocked,
    /// Every man is entered, and placing is all there is.
    AllPlaced,
    /// The board filled up while entering.
    BoardFull,
    /// Too many moves without a man taken, a side down to three.
    Quiet,
}

/// The position: the men of each side on the board, those still in hand,
/// and the side to move.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub men: [u32; 2],
    pub hand: [u8; 2],
    pub to_move: Side,
    /// For each side, a move (from, to) it may not play next if it makes
    /// a mill (`Rules::no_return`).
    pub barred: [Option<(u8, u8)>; 2],
    /// Moves without a man taken since a side came down to three
    /// (`Rules::quiet_draw`).
    pub quiet: u8,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        Position { men: [0, 0], hand: [rules.men, rules.men], to_move: Side::White, barred: [None, None], quiet: 0 }
    }

    pub fn empty(&self, board: &Board) -> u32 {
        board.all() & !(self.men[0] | self.men[1])
    }

    pub fn side_at(&self, p: usize) -> Option<Side> {
        Side::BOTH.into_iter().find(|s| self.men[s.index()] & 1 << p != 0)
    }

    /// Men a side has on the board and in hand.
    pub fn count(&self, side: Side) -> u32 {
        self.men[side.index()].count_ones() + self.hand[side.index()] as u32
    }

    /// Whether `side` may fly: every man entered and down to the rule
    /// set's number.
    pub fn flies(&self, rules: &Rules, side: Side) -> bool {
        self.hand[side.index()] == 0 && rules.flying.is_some_and(|n| self.count(side) <= n as u32)
    }

    /// The legal moves for the side to move; none once the game is over.
    pub fn moves(&self, rules: &Rules, board: &Board) -> Vec<Move> {
        self.status(rules, board).1
    }

    /// The end of the game if it has come, else the legal moves: one
    /// generation of the moves for both (the search asks at every node).
    pub fn status(&self, rules: &Rules, board: &Board) -> (Option<Outcome>, Vec<Move>) {
        if let Some(end) = self.ended(rules, board) {
            return (Some(end), Vec::new());
        }
        let steps = self.steps(rules, board);
        if steps.is_empty() {
            return (Some(Outcome::Won(self.to_move.other(), Why::Blocked)), steps);
        }
        (None, steps)
    }

    /// The moves, without asking first whether the game is over.
    fn steps(&self, rules: &Rules, board: &Board) -> Vec<Move> {
        let me = self.to_move.index();
        let empty = self.empty(board);
        let mut moves = Vec::new();
        let mut add = |from: Option<u8>, to: usize| {
            let mut mine = self.men[me];
            if let Some(f) = from {
                mine &= !(1 << f);
            }
            mine |= 1 << to;
            let made = board.mills_at[to].iter().filter(|&&m| mine & m == m).count() as u8;
            if made > 0 && from.is_some_and(|f| self.barred[me] == Some((f, to as u8))) {
                return;
            }
            if made == 0 || rules.mill_wins {
                moves.push(Move { from, to: to as u8, take: [None, None] });
                return;
            }
            let takes = made.min(rules.double_mill_takes);
            let first = self.takeable(rules, board, self.men[1 - me]);
            if first == 0 {
                moves.push(Move { from, to: to as u8, take: [None, None] });
                return;
            }
            for a in bits(first) {
                if takes < 2 {
                    moves.push(Move { from, to: to as u8, take: [Some(a as u8), None] });
                    continue;
                }
                let rest = self.men[1 - me] & !(1 << a);
                let second = self.takeable(rules, board, rest);
                if second == 0 {
                    moves.push(Move { from, to: to as u8, take: [Some(a as u8), None] });
                }
                for b in bits(second).filter(|&b| b > a || first & 1 << b == 0) {
                    moves.push(Move { from, to: to as u8, take: [Some(a as u8), Some(b as u8)] });
                }
            }
        };
        if self.hand[me] > 0 {
            for to in bits(empty) {
                add(None, to);
            }
        } else if rules.moving {
            let fly = self.flies(rules, self.to_move);
            for from in bits(self.men[me]) {
                let reach = if fly { empty } else { board.adjacent[from] & empty };
                for to in bits(reach) {
                    add(Some(from as u8), to);
                }
            }
        }
        moves
    }

    /// The men of `theirs` that a mill may take.
    fn takeable(&self, rules: &Rules, board: &Board, theirs: u32) -> u32 {
        let free = theirs & !board.in_mills(theirs);
        match rules.taking {
            Taking::Any => theirs,
            Taking::NeverFromMills => free,
            Taking::FromMillsLast if free == 0 => theirs,
            Taking::FromMillsLast => free,
        }
    }

    /// Plays `mv` for the side to move.
    pub fn play(&mut self, rules: &Rules, board: &Board, mv: &Move) {
        let me = self.to_move.index();
        // A man leaving a mill to make a new one may not go straight back.
        self.barred[me] = None;
        if rules.no_return
            && let Some(f) = mv.from
            && board.in_mills(self.men[me]) & 1 << f != 0
            && board.mills_at[mv.to as usize].iter().any(|&m| (self.men[me] & !(1 << f) | 1 << mv.to) & m == m)
        {
            self.barred[me] = Some((mv.to, f));
        }
        match mv.from {
            Some(f) => self.men[me] &= !(1 << f),
            None => self.hand[me] -= 1,
        }
        self.men[me] |= 1 << mv.to;
        for p in mv.taken() {
            self.men[1 - me] &= !(1 << p);
        }
        if mv.take[0].is_some() {
            self.quiet = 0;
        } else if Side::BOTH.iter().any(|&s| self.count(s) <= 3) {
            self.quiet = self.quiet.saturating_add(1);
        }
        self.to_move = self.to_move.other();
    }

    /// The end of the game, if it has come.
    pub fn outcome(&self, rules: &Rules, board: &Board) -> Option<Outcome> {
        self.status(rules, board).0
    }

    /// Every end but a side with no move.
    fn ended(&self, rules: &Rules, board: &Board) -> Option<Outcome> {
        let me = self.to_move;
        if rules.mill_wins {
            // Only the side that just moved can have made a line.
            let last = me.other();
            if board.in_mills(self.men[last.index()]) != 0 {
                return Some(Outcome::Won(last, Why::Line));
            }
        }
        for side in Side::BOTH {
            if !rules.mill_wins && self.count(side) < 3 {
                return Some(Outcome::Won(side.other(), Why::TwoLeft));
            }
        }
        let entered = self.hand == [0, 0];
        if entered && !rules.moving {
            return Some(Outcome::Drawn(Why::AllPlaced));
        }
        if entered && rules.full_board_draws && self.empty(board) == 0 {
            return Some(Outcome::Drawn(Why::BoardFull));
        }
        if rules.quiet_draw.is_some_and(|n| self.quiet >= n) {
            return Some(Outcome::Drawn(Why::Quiet));
        }
        None
    }
}

/// The points in a bit set, lowest first.
pub fn bits(mut set: u32) -> impl Iterator<Item = usize> {
    std::iter::from_fn(move || {
        (set != 0).then(|| {
            let p = set.trailing_zeros() as usize;
            set &= set - 1;
            p
        })
    })
}

/// A small game solved: every position reachable from the start, worked
/// back from every end. Three Men's Morris has a few thousand, so the
/// hard computer plays it perfectly from this.
pub struct Solved {
    /// For each position, for the side to move: 1 won, -1 lost, 0 drawn
    /// (or only able to go round in circles), and the round of the working
    /// back at which that was found, the moves to the end for a win or a
    /// loss.
    table: std::collections::HashMap<Position, (i8, u16)>,
}

impl Solved {
    pub fn new(rules: &Rules) -> Solved {
        use std::collections::HashMap;
        let b = rules.board();
        let start = Position::new(rules);
        let mut index: HashMap<Position, usize> = HashMap::new();
        let mut all = vec![start.clone()];
        index.insert(start, 0);
        let mut next: Vec<Vec<usize>> = Vec::new();
        let mut i = 0;
        while i < all.len() {
            let p = all[i].clone();
            let mut out = Vec::new();
            if p.outcome(rules, &b).is_none() {
                for mv in p.moves(rules, &b) {
                    let mut q = p.clone();
                    q.play(rules, &b, &mv);
                    let n = all.len();
                    let j = *index.entry(q.clone()).or_insert(n);
                    if j == n {
                        all.push(q);
                    }
                    out.push(j);
                }
            }
            next.push(out);
            i += 1;
        }
        let mut value: Vec<(i8, u16)> = all
            .iter()
            .map(|p| match p.outcome(rules, &b) {
                Some(Outcome::Won(s, _)) if s == p.to_move => (1, 0),
                Some(Outcome::Won(..)) => (-1, 0),
                _ => (0, 0),
            })
            .collect();
        // Round by round, so that a win found in round n is n moves from
        // its end, at most.
        let mut round = 0;
        loop {
            round += 1;
            let mut found = Vec::new();
            for k in 0..all.len() {
                if value[k].0 != 0 || next[k].is_empty() {
                    continue;
                }
                if next[k].iter().any(|&j| value[j].0 == -1) {
                    found.push((k, 1));
                } else if next[k].iter().all(|&j| value[j].0 == 1) {
                    found.push((k, -1));
                }
            }
            if found.is_empty() {
                break;
            }
            for (k, v) in found {
                value[k] = (v, round);
            }
        }
        Solved { table: all.into_iter().zip(value).collect() }
    }

    /// The value of a position for the side to move, and its distance.
    pub fn value(&self, p: &Position) -> Option<(i8, u16)> {
        self.table.get(p).copied()
    }

    /// The best move: the quickest win, else a draw, else the slowest
    /// loss.
    pub fn best(&self, rules: &Rules, board: &Board, p: &Position) -> Option<Move> {
        p.moves(rules, board)
            .into_iter()
            .filter_map(|mv| {
                let mut q = p.clone();
                q.play(rules, board, &mv);
                let (v, d) = self.value(&q)?;
                // The value for the mover: the other side's, turned round.
                let key = match -v {
                    1 => 3_000_000 - d as i32,
                    0 => 0,
                    _ => -3_000_000 + d as i32,
                };
                Some((key, mv))
            })
            .max_by_key(|&(key, _)| key)
            .map(|(_, mv)| mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(board: &Board, name: &str) -> usize {
        (0..board.len()).find(|&p| board.name(p) == name).unwrap_or_else(|| panic!("no point {name}"))
    }

    fn set(board: &Board, names: &[&str]) -> u32 {
        names.iter().fold(0, |m, n| m | 1 << at(board, n))
    }

    #[test]
    fn the_boards() {
        let counts = [(Layout::Three { diagonals: false }, 9, 6, 12), (Layout::Three { diagonals: true }, 9, 8, 16)];
        for (layout, points, mills, links) in counts {
            let b = Board::new(layout);
            assert_eq!((b.len(), b.mills.len()), (points, mills));
            let joined: u32 = b.adjacent.iter().map(|a| a.count_ones()).sum();
            assert_eq!(joined / 2, links);
        }
        let six = Board::new(Layout::Six);
        assert_eq!((six.len(), six.mills.len()), (16, 8));
        let nine = Board::new(Layout::Nine);
        assert_eq!((nine.len(), nine.mills.len()), (24, 16));
        let twelve = Board::new(Layout::Twelve);
        assert_eq!((twelve.len(), twelve.mills.len()), (24, 20));
        // The corners of nine have two neighbours, of twelve three or four.
        assert_eq!(nine.adjacent[at(&nine, "a1")].count_ones(), 2);
        assert_eq!(twelve.adjacent[at(&twelve, "a1")].count_ones(), 3);
        assert_eq!(twelve.adjacent[at(&twelve, "b2")].count_ones(), 4);
        // Names: a1 bottom left, g7 top right, d4 not a point of nine.
        assert_eq!(nine.points[at(&nine, "g7")], (0, 6));
        assert!(nine.point_at(3, 3).is_none());
    }

    #[test]
    fn a_mill_takes_a_man_not_in_a_mill() {
        let rules = Rules::nine_bell();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.men = [set(&b, &["a1", "d1"]), set(&b, &["a4", "a7", "b4", "b6", "d6", "f6"])];
        p.hand = [3, 3];
        // g1 closes a1-d1-g1; b6-d6-f6 is a mill, so only a4, a7, b4 can go.
        let taken: Vec<String> = p
            .moves(&rules, &b)
            .iter()
            .filter(|m| m.to as usize == at(&b, "g1"))
            .flat_map(|m| m.taken().map(|q| b.name(q)).collect::<Vec<_>>())
            .collect();
        let mut names = taken.clone();
        names.sort();
        assert_eq!(names, ["a4", "a7", "b4"]);
    }

    #[test]
    fn all_in_mills() {
        let b = Board::new(Layout::Nine);
        let theirs = set(&b, &["b6", "d6", "f6"]);
        let mut p = Position::new(&Rules::nine_bell());
        p.men = [set(&b, &["a1", "d1"]), theirs];
        p.hand = [3, 3];
        let g1 = at(&b, "g1");
        // Bell: none from a mill, so nothing is taken.
        let bell = Rules::nine_bell();
        let m: Vec<Move> = p.moves(&bell, &b).into_iter().filter(|m| m.to as usize == g1).collect();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].take, [None, None]);
        // Today's: then one from the mill.
        let m: Vec<Move> = p.moves(&Rules::nine_modern(), &b).into_iter().filter(|m| m.to as usize == g1).collect();
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn moving_along_lines_and_flying() {
        let rules = Rules::nine_modern();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.hand = [0, 0];
        p.men = [set(&b, &["a1", "d2", "g7", "c5"]), set(&b, &["a4", "g1", "d7"])];
        let moves = p.moves(&rules, &b);
        let from_a1: Vec<String> =
            moves.iter().filter(|m| m.from == Some(at(&b, "a1") as u8)).map(|m| b.name(m.to as usize)).collect();
        assert_eq!(from_a1, ["d1"]);
        // Black is down to three: it flies.
        p.to_move = Side::Black;
        let empty = p.empty(&b).count_ones() as usize;
        assert_eq!(p.moves(&rules, &b).len(), 3 * empty);
        // Not after Bell.
        let bell = Rules::nine_bell();
        assert!(p.moves(&bell, &b).len() < 3 * empty);
    }

    #[test]
    fn two_left_and_blocked() {
        let rules = Rules::nine_bell();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.hand = [0, 0];
        p.men = [set(&b, &["a1", "d1", "g1"]), set(&b, &["a7", "d7"])];
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Won(Side::White, Why::TwoLeft)));
        // Black on a1 hemmed in by d1 and a4.
        p.men = [set(&b, &["d1", "a4", "g7"]), set(&b, &["a1", "d2", "d3"])];
        p.men[0] |= set(&b, &["c3", "c4", "e3"]);
        p.to_move = Side::Black;
        // d2 can still go to b2, f2; d3 to e3? taken; so not blocked yet.
        assert!(p.outcome(&rules, &b).is_none());
        p.men[0] |= set(&b, &["b2", "f2"]);
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Won(Side::White, Why::Blocked)));
    }

    #[test]
    fn double_mill() {
        let b = Board::new(Layout::Nine);
        let mut p = Position::new(&Rules::nine_modern());
        p.men = [set(&b, &["a1", "d1", "g4", "g7"]), set(&b, &["a7", "b6", "c5"])];
        p.hand = [3, 3];
        let g1 = at(&b, "g1") as u8;
        let one = Rules::nine_modern();
        assert!(p.moves(&one, &b).iter().filter(|m| m.to == g1).all(|m| m.take[1].is_none()));
        let two = Rules { double_mill_takes: 2, ..Rules::nine_modern() };
        let pairs: Vec<Move> = p.moves(&two, &b).into_iter().filter(|m| m.to == g1).collect();
        assert_eq!(pairs.len(), 3, "{pairs:?}");
        assert!(pairs.iter().all(|m| m.take[1].is_some()));
    }

    #[test]
    fn three_mens_morris() {
        let rules = Rules::three_moving();
        let b = rules.board();
        let mut p = Position::new(&rules);
        for name in ["b2", "a1", "c1", "a3", "c3"] {
            let mv = p.moves(&rules, &b).into_iter().find(|m| m.to as usize == at(&b, name)).unwrap();
            p.play(&rules, &b, &mv);
        }
        // White b2, c1, c3; c2 is free, so not a line yet. Black a1, a3.
        assert!(p.outcome(&rules, &b).is_none());
        let mv = p.moves(&rules, &b).into_iter().find(|m| m.to as usize == at(&b, "a2")).unwrap();
        p.play(&rules, &b, &mv);
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Won(Side::Black, Why::Line)));
        // Placing only: eight counters and no line is a draw.
        let rules = Rules::three_placing();
        let b = rules.board();
        let mut p = Position::new(&rules);
        for name in ["b2", "a1", "c3", "a3", "a2", "c2", "b3", "b1"] {
            let mv = p.moves(&rules, &b).into_iter().find(|m| m.to as usize == at(&b, name)).unwrap();
            p.play(&rules, &b, &mv);
        }
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Drawn(Why::AllPlaced)));
    }

    #[test]
    fn twelve_fills_the_board() {
        let rules = Rules::morabaraba();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.hand = [0, 0];
        p.men = [0, 0];
        // Alternate colours so no mill: rows of points shared out in turn.
        for q in 0..b.len() {
            p.men[q % 2] |= 1 << q;
        }
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Drawn(Why::BoardFull)));
    }

    #[test]
    fn morabaraba_no_straight_return() {
        let rules = Rules::morabaraba();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.hand = [0, 0];
        // White's a1-d1-g1 stands; a1 up to a4 makes a4-b4-c4.
        p.men = [set(&b, &["a1", "d1", "g1", "b4", "c4"]), set(&b, &["d7", "e5", "f6", "g7"])];
        let mv = *p
            .moves(&rules, &b)
            .iter()
            .find(|m| m.from == Some(at(&b, "a1") as u8) && m.to as usize == at(&b, "a4"))
            .expect("a1 to a4");
        p.play(&rules, &b, &mv);
        assert_eq!(p.barred[0], Some((at(&b, "a4") as u8, at(&b, "a1") as u8)));
        // Black passes the time; White may not go straight back to a1.
        let black = p.moves(&rules, &b)[0];
        p.play(&rules, &b, &black);
        let back = |p: &Position| {
            p.moves(&rules, &b).iter().any(|m| m.from == Some(at(&b, "a4") as u8) && m.to as usize == at(&b, "a1"))
        };
        assert!(!back(&p), "barred");
        // In Nine Men's Morris today it may.
        let nine = Rules { layout: Layout::Twelve, ..Rules::nine_modern() };
        let mut q = p.clone();
        q.barred = [None, None];
        assert!(q.moves(&nine, &b).iter().any(|m| m.from == Some(at(&b, "a4") as u8) && m.to as usize == at(&b, "a1")));
    }

    #[test]
    fn quiet_draw_with_three_left() {
        let rules = Rules::morabaraba();
        let b = rules.board();
        let mut p = Position::new(&rules);
        p.hand = [0, 0];
        p.men = [set(&b, &["a1", "d2", "g7", "c5"]), set(&b, &["a4", "g1", "d6"])];
        for _ in 0..20 {
            assert!(p.outcome(&rules, &b).is_none());
            let quiet = p.moves(&rules, &b).into_iter().find(|m| m.take[0].is_none()).expect("a quiet move");
            p.play(&rules, &b, &quiet);
        }
        assert_eq!(p.outcome(&rules, &b), Some(Outcome::Drawn(Why::Quiet)));
    }

    /// The value of the start for the first player: 1 a win, 0 a draw,
    /// -1 a loss.
    fn solve(rules: &Rules) -> i32 {
        Solved::new(rules).value(&Position::new(rules)).map_or(0, |(v, _)| v as i32)
    }

    /// Bell: in his second form "the first player could always force a
    /// win by correct play"; Alfonso X says the same of his alquerque de
    /// tres. Placing only, three each, it is a draw.
    #[test]
    fn three_mens_morris_solved() {
        assert_eq!(solve(&Rules::three_moving()), 1, "moving: the first player wins");
        let three_each = Rules { men: 3, layout: Layout::Three { diagonals: true }, ..Rules::three_placing() };
        assert_eq!(solve(&three_each), 0, "placing three each, with diagonals: a draw");
        assert_eq!(solve(&Rules::three_placing()), 0, "placing four each: a draw");
    }
}
