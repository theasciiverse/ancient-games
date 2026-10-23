//! The rules of Kōnane, as data, and the engine that plays them.
//!
//! The board (papamū) is a grid of hollows, every one filled at the start
//! with black and white stones in turn, so that each diagonal is of one
//! colour. Two stones are lifted out to open the game; after that a move
//! is a jump: a stone leaps an enemy stone next to it, along a row or a
//! column, into the empty hollow beyond, and the enemy stone is taken. In
//! most rule sets it may go on jumping in the same direction. Who cannot
//! move has lost.
//!
//! Hollows are numbered row by row from the top left; (0, 0) is black.

/// The two sides. Black, the dark lava stones, begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Black,
    White,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::Black => 0,
            Side::White => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Black => "Black",
            Side::White => "White",
        }
    }

    pub const BOTH: [Side; 2] = [Side::Black, Side::White];
}

/// How the first two stones are lifted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    /// Black lifts a black stone in the middle (the centre, or one next to
    /// it) or at a corner; White then lifts a white stone next to that
    /// hollow, the centre stone if Black's was next to the centre (Ernst,
    /// after Emory and Buck).
    MiddleOrCorner,
    /// Each lifts a stone of his own colour from anywhere (the leaflet of
    /// Puʻuhonua o Hōnaunau).
    Anywhere,
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where it comes from, in a sentence or two, for the menu.
    pub source: &'static str,
    /// The heading of this rule set in the rulebook chapter, where the
    /// help opens.
    pub section: &'static str,
    pub rows: usize,
    pub cols: usize,
    pub opening: Opening,
    /// Whether a stone may jump on in the same direction in one move.
    pub multiple: bool,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::ernst(), Rules::modern(), Rules::honaunau(), Rules::king()]
    }

    pub fn ernst() -> Rules {
        Rules {
            id: "ernst",
            name: "Kōnane after Ernst, 8 by 8",
            source: "Michael Ernst (1995), after Emory (1924) and Buck (1957): the first stones lifted \
                     in the middle or at a corner; jumps may go on in a straight line.",
            section: "Kōnane after Ernst",
            rows: 8,
            cols: 8,
            opening: Opening::MiddleOrCorner,
            multiple: true,
        }
    }

    pub fn modern() -> Rules {
        Rules {
            id: "modern",
            name: "Modern Kōnane, 8 by 8",
            source: "What Ernst (1995) calls Modern Konane, after Sizer (1991): one jump a move.",
            section: "Modern Kōnane",
            multiple: false,
            ..Rules::ernst()
        }
    }

    pub fn honaunau() -> Rules {
        Rules {
            id: "honaunau",
            name: "Puʻuhonua o Hōnaunau, 9 by 11",
            source: "The rules of the National Park at Puʻuhonua o Hōnaunau, on its old stone papamū \
                     of 9 rows by 11: the first stones lifted from anywhere.",
            section: "Puʻuhonua o Hōnaunau",
            rows: 9,
            cols: 11,
            opening: Opening::Anywhere,
            multiple: true,
        }
    }

    pub fn king() -> Rules {
        Rules {
            id: "king",
            name: "King's papamū, 14 by 17",
            source: "The board James King saw in 1779, 238 squares, 14 in a row, with Ernst's rules.",
            section: "King's papamū",
            rows: 14,
            cols: 17,
            ..Rules::ernst()
        }
    }

    pub fn len(&self) -> usize {
        self.rows * self.cols
    }

    /// A hollow's row and column.
    pub fn rc(&self, p: usize) -> (usize, usize) {
        (p / self.cols, p % self.cols)
    }

    /// The colour a hollow's stone has at the start.
    pub fn colour(&self, p: usize) -> Side {
        let (r, c) = self.rc(p);
        if (r + c) % 2 == 0 { Side::Black } else { Side::White }
    }

    /// The hollow `n` steps from `p` in direction (dr, dc), if on the board.
    pub fn step(&self, p: usize, (dr, dc): (isize, isize), n: isize) -> Option<usize> {
        let (r, c) = self.rc(p);
        let r = r as isize + dr * n;
        let c = c as isize + dc * n;
        (r >= 0 && c >= 0 && (r as usize) < self.rows && (c as usize) < self.cols)
            .then(|| r as usize * self.cols + c as usize)
    }

    /// A hollow's name: the column as a letter from the left, the row as a
    /// number from the top.
    pub fn name(&self, p: usize) -> String {
        let (r, c) = self.rc(p);
        format!("{}{}", (b'a' + c as u8) as char, r + 1)
    }

    /// The middle hollows: the centre and those next to it on an odd
    /// board, the four central ones on an even side.
    fn middle(&self) -> Vec<usize> {
        let rows: Vec<usize> = if self.rows % 2 == 1 {
            vec![self.rows / 2]
        } else {
            vec![self.rows / 2 - 1, self.rows / 2]
        };
        let cols: Vec<usize> = if self.cols % 2 == 1 {
            vec![self.cols / 2]
        } else {
            vec![self.cols / 2 - 1, self.cols / 2]
        };
        let mut out = Vec::new();
        for &r in &rows {
            for &c in &cols {
                out.push(r * self.cols + c);
            }
        }
        // On an odd board, the hollows next to the single centre too.
        if out.len() == 1 {
            let centre = out[0];
            out.extend(DIRECTIONS.iter().filter_map(|&d| self.step(centre, d, 1)));
        }
        out
    }

    /// The single centre hollow, on a board odd both ways.
    fn centre(&self) -> Option<usize> {
        (self.rows % 2 == 1 && self.cols % 2 == 1).then(|| (self.rows / 2) * self.cols + self.cols / 2)
    }

    fn corners(&self) -> [usize; 4] {
        [0, self.cols - 1, self.len() - self.cols, self.len() - 1]
    }
}

/// Up, down, left, right.
pub const DIRECTIONS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// A move: a stone lifted in the opening, or a jump along a line,
/// landing on each of `path` in turn.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Lift(usize),
    Jump { from: usize, path: Vec<usize> },
}

impl Move {
    /// The stones a jump takes: the hollows between landings.
    pub fn taken(&self, rules: &Rules) -> Vec<usize> {
        match self {
            Move::Lift(_) => Vec::new(),
            Move::Jump { from, path } => {
                let mut at = *from;
                path.iter()
                    .map(|&to| {
                        let (r0, c0) = rules.rc(at);
                        let (r1, c1) = rules.rc(to);
                        at = to;
                        ((r0 + r1) / 2) * rules.cols + (c0 + c1) / 2
                    })
                    .collect()
            }
        }
    }
}

/// The position: the stones on the board, the side to move, and how many
/// stones have been lifted (the game proper begins after two).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub at: Vec<Option<Side>>,
    pub to_move: Side,
    pub lifted: u8,
    /// Where the first stone was lifted.
    pub first_hole: Option<usize>,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let at = (0..rules.len()).map(|p| Some(rules.colour(p))).collect();
        Position { at, to_move: Side::Black, lifted: 0, first_hole: None }
    }

    pub fn opening(&self) -> bool {
        self.lifted < 2
    }

    /// The legal moves for the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        let me = self.to_move;
        if self.opening() {
            return self.lifts(rules).into_iter().map(Move::Lift).collect();
        }
        let mut moves = Vec::new();
        for from in 0..rules.len() {
            if self.at[from] != Some(me) {
                continue;
            }
            for d in DIRECTIONS {
                let mut path = Vec::new();
                let mut n = 0;
                loop {
                    let (Some(over), Some(land)) = (rules.step(from, d, n + 1), rules.step(from, d, n + 2)) else {
                        break;
                    };
                    if self.at[over] != Some(me.other()) || self.at[land].is_some() {
                        break;
                    }
                    path.push(land);
                    moves.push(Move::Jump { from, path: path.clone() });
                    if !rules.multiple {
                        break;
                    }
                    n += 2;
                }
            }
        }
        moves
    }

    /// The stones that may be lifted now.
    fn lifts(&self, rules: &Rules) -> Vec<usize> {
        let me = self.to_move;
        let mine = |p: &usize| self.at[*p] == Some(me);
        match (rules.opening, self.first_hole) {
            (Opening::Anywhere, _) => (0..rules.len()).filter(mine).collect(),
            (Opening::MiddleOrCorner, None) => {
                let mut out: Vec<usize> = rules.middle().into_iter().chain(rules.corners()).filter(mine).collect();
                out.sort_unstable();
                out.dedup();
                out
            }
            (Opening::MiddleOrCorner, Some(hole)) => {
                // Next to the centre: the centre itself.
                if let Some(c) = rules.centre()
                    && DIRECTIONS.iter().any(|&d| rules.step(hole, d, 1) == Some(c))
                    && mine(&c)
                {
                    return vec![c];
                }
                DIRECTIONS.iter().filter_map(|&d| rules.step(hole, d, 1)).filter(mine).collect()
            }
        }
    }

    pub fn play(&mut self, rules: &Rules, mv: &Move) {
        match mv {
            Move::Lift(p) => {
                self.at[*p] = None;
                self.lifted += 1;
                self.first_hole.get_or_insert(*p);
            }
            Move::Jump { from, path } => {
                for q in mv.taken(rules) {
                    self.at[q] = None;
                }
                let to = *path.last().expect("a landing");
                self.at[to] = self.at[*from].take();
            }
        }
        self.to_move = self.to_move.other();
    }

    /// The winner, if the side to move cannot move.
    pub fn winner(&self, rules: &Rules) -> Option<Side> {
        self.moves(rules).is_empty().then(|| self.to_move.other())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(rules: &Rules, name: &str) -> usize {
        (0..rules.len()).find(|&p| rules.name(p) == name).unwrap()
    }

    #[test]
    fn the_board_and_the_opening() {
        let rules = Rules::ernst();
        let mut p = Position::new(&rules);
        assert_eq!(rules.colour(0), Side::Black);
        // Black lifts: the two black stones of the central four, and the
        // two black corners.
        let lifts: Vec<String> = p.moves(&rules).iter().map(|m| if let Move::Lift(q) = m { rules.name(*q) } else { String::new() }).collect();
        assert_eq!(lifts, ["a1", "d4", "e5", "h8"]);
        p.play(&rules, &Move::Lift(at(&rules, "d4")));
        // White: a white stone next to d4.
        assert_eq!(p.moves(&rules).len(), 4);
        p.play(&rules, &Move::Lift(at(&rules, "e4")));
        // Black jumps into e4 or d4? d4 is a black hollow: Black lands there.
        assert!(p.moves(&rules).iter().all(|m| matches!(m, Move::Jump { path, .. } if path[0] == at(&rules, "d4"))));
        // Into d4 from d2, d6, b4 (over d3, d5, c4); f4 cannot, e4 is empty.
        assert_eq!(p.moves(&rules).len(), 3);
    }

    #[test]
    fn the_centre_after_a_stone_next_to_it() {
        // 9 by 11, Ernst's opening: the centre f5 is white, so Black lifts
        // one next to it, and White must then lift the centre.
        let rules = Rules { opening: Opening::MiddleOrCorner, ..Rules::honaunau() };
        let mut p = Position::new(&rules);
        let centre = at(&rules, "f5");
        assert_eq!(rules.colour(centre), Side::White);
        p.play(&rules, &Move::Lift(at(&rules, "f4")));
        assert_eq!(p.moves(&rules), [Move::Lift(centre)]);
    }

    #[test]
    fn multiple_jumps_in_a_line() {
        let rules = Rules::ernst();
        let mut p = Position::new(&rules);
        p.lifted = 2;
        p.at = vec![None; rules.len()];
        let a1 = at(&rules, "a1");
        p.at[a1] = Some(Side::Black);
        p.at[at(&rules, "b1")] = Some(Side::White);
        p.at[at(&rules, "d1")] = Some(Side::White);
        p.at[at(&rules, "a2")] = Some(Side::White);
        let moves = p.moves(&rules);
        // c1, then e1; and a3 down. The modern rules stop at c1.
        assert_eq!(moves.len(), 3);
        assert_eq!(p.moves(&Rules::modern()).len(), 2);
        let long = moves.iter().find(|m| matches!(m, Move::Jump { path, .. } if path.len() == 2)).unwrap().clone();
        assert_eq!(long.taken(&rules).len(), 2);
        p.play(&rules, &long);
        assert_eq!(p.at[at(&rules, "e1")], Some(Side::Black));
        assert_eq!(p.winner(&rules), Some(Side::Black));
    }
}
