//! The rules of Fox and Geese as R. C. Bell gives them (Board and Table
//! Games from Many Civilizations, 1960, 2nd ed. 1969, pp. 76-77, with his
//! figs. 62 and 63).
//!
//! The board is a cross of 33 points, joined by lines along the rows and
//! columns (Bell's figures have no diagonals). Rows 0-1 and 5-6 have points
//! in columns 2 to 4; rows 2 to 4 have all seven. The geese start at the
//! bottom, the fox above them.

/// Rows and columns of the grid the cross sits in.
pub const SIZE: usize = 7;

/// A point on the grid (row, column).
pub type Point = (usize, usize);

/// Whether a grid square is one of the 33 points.
pub fn on_board(p: Point) -> bool {
    let (r, c) = p;
    r < SIZE && c < SIZE && ((2..=4).contains(&r) || (2..=4).contains(&c))
}

/// Every point, row by row.
pub fn points() -> Vec<Point> {
    (0..SIZE)
        .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
        .filter(|&p| on_board(p))
        .collect()
}

/// The four directions along the lines: up, down, left, right.
pub const DIRECTIONS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// The point one step from `p` in direction `d`, if on the board.
pub fn step(p: Point, d: (isize, isize)) -> Option<Point> {
    let r = p.0 as isize + d.0;
    let c = p.1 as isize + d.1;
    if r < 0 || c < 0 {
        return None;
    }
    let q = (r as usize, c as usize);
    on_board(q).then_some(q)
}

/// The sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Fox,
    Geese,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Fox => Side::Geese,
            Side::Geese => Side::Fox,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Fox => "the fox",
            Side::Geese => "the geese",
        }
    }
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    /// Where the geese start.
    pub geese: &'static [Point],
    /// Whether geese may move back, away from the fox's end (row 0).
    pub geese_back: bool,
    /// The heading of this rule set in the rulebook chapter (RULES.txt),
    /// where the help opens.
    pub section: &'static str,
}

/// Bell's fig. 62: the row below the middle and the lower arm.
const THIRTEEN: [Point; 13] = [
    (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6),
    (5, 2), (5, 3), (5, 4),
    (6, 2), (6, 3), (6, 4),
];

/// Bell's fig. 63: the same and two more down each side.
const SEVENTEEN: [Point; 17] = [
    (2, 0), (2, 6),
    (3, 0), (3, 6),
    (4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6),
    (5, 2), (5, 3), (5, 4),
    (6, 2), (6, 3), (6, 4),
];

/// Where the fox starts in both of Bell's figures.
pub const FOX_START: Point = (2, 3);

/// Geese this few can never trap the fox: every trap on this board needs at
/// least four (two on its open sides, each with a goose or an edge behind,
/// as the fox jumps any goose with an empty point beyond).
pub const FOX_WINS_AT: usize = 3;

impl Rules {
    pub fn thirteen() -> Rules {
        Rules {
            id: "thirteen",
            name: "Bell, thirteen geese",
            source: "R. C. Bell's early form of the game, from his Board and \
                     Table Games from Many Civilizations (1960): thirteen geese \
                     that may move any way along a line.",
            geese: &THIRTEEN,
            geese_back: true,
            section: "Bell, thirteen geese",
        }
    }

    pub fn seventeen() -> Rules {
        Rules {
            id: "seventeen",
            name: "Bell, seventeen geese",
            source: "R. C. Bell's later form (1960): seventeen geese, which \
                     have lost the power of moving backwards.",
            geese: &SEVENTEEN,
            geese_back: false,
            section: "Bell, seventeen geese",
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![Rules::thirteen(), Rules::seventeen()]
    }
}

/// A move: the path of the piece, from its point through every landing.
/// A goose or a fox step has two points; a fox's jumps one more per goose.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub path: Vec<Point>,
}

impl Move {
    pub fn from(&self) -> Point {
        self.path[0]
    }

    pub fn to(&self) -> Point {
        *self.path.last().unwrap()
    }

    /// The geese a fox's jumps kill: the points between landings.
    pub fn killed(&self) -> Vec<Point> {
        self.path
            .windows(2)
            .filter(|w| w[0].0.abs_diff(w[1].0) + w[0].1.abs_diff(w[1].1) == 2)
            .map(|w| ((w[0].0 + w[1].0) / 2, (w[0].1 + w[1].1) / 2))
            .collect()
    }
}

/// The position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub fox: Point,
    /// The geese, sorted.
    pub geese: Vec<Point>,
    pub to_move: Side,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let mut geese = rules.geese.to_vec();
        geese.sort_unstable();
        Position { fox: FOX_START, geese, to_move: Side::Geese }
    }

    pub fn is_goose(&self, p: Point) -> bool {
        self.geese.binary_search(&p).is_ok()
    }

    pub fn is_empty(&self, p: Point) -> bool {
        p != self.fox && !self.is_goose(p)
    }

    /// The legal moves for the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        match self.to_move {
            Side::Fox => self.fox_moves(),
            Side::Geese => self.goose_moves(rules),
        }
    }

    fn goose_moves(&self, rules: &Rules) -> Vec<Move> {
        let mut moves = Vec::new();
        for &g in &self.geese {
            for d in DIRECTIONS {
                // Back is down the board, away from the fox's end.
                if !rules.geese_back && d == (1, 0) {
                    continue;
                }
                if let Some(q) = step(g, d)
                    && self.is_empty(q)
                {
                    moves.push(Move { path: vec![g, q] });
                }
            }
        }
        moves
    }

    fn fox_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for d in DIRECTIONS {
            if let Some(q) = step(self.fox, d)
                && self.is_empty(q)
            {
                moves.push(Move { path: vec![self.fox, q] });
            }
        }
        // Jumps, and every chain of further jumps; the fox may stop after
        // any of them.
        let mut stack = vec![(vec![self.fox], Vec::<Point>::new())];
        while let Some((path, killed)) = stack.pop() {
            let at = *path.last().unwrap();
            for d in DIRECTIONS {
                let (Some(over), Some(land)) = (step(at, d), step(at, d).and_then(|o| step(o, d)))
                else {
                    continue;
                };
                // The fox's own starting point is empty once it has left.
                let land_free = land == self.fox || self.is_empty(land);
                if self.is_goose(over) && !killed.contains(&over) && land_free && !path.contains(&land) {
                    let mut p = path.clone();
                    p.push(land);
                    let mut k = killed.clone();
                    k.push(over);
                    moves.push(Move { path: p.clone() });
                    stack.push((p, k));
                }
            }
        }
        moves
    }

    /// Plays `mv` for the side to move.
    pub fn play(&mut self, mv: &Move) {
        match self.to_move {
            Side::Fox => {
                for k in mv.killed() {
                    self.geese.retain(|&g| g != k);
                }
                self.fox = mv.to();
            }
            Side::Geese => {
                let i = self.geese.binary_search(&mv.from()).expect("no goose there");
                self.geese[i] = mv.to();
                self.geese.sort_unstable();
            }
        }
        self.to_move = self.to_move.other();
    }

    /// The winner, if the game is over: the fox trapped, or too few geese
    /// left to trap him, or the side to move unable to move.
    pub fn winner(&self, rules: &Rules) -> Option<Side> {
        if self.geese.len() <= FOX_WINS_AT {
            return Some(Side::Fox);
        }
        if self.moves(rules).is_empty() {
            return Some(self.to_move.other());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cross_has_thirty_three_points() {
        assert_eq!(points().len(), 33);
        assert!(on_board((0, 2)) && !on_board((0, 1)) && on_board((3, 0)));
    }

    #[test]
    fn bells_figures() {
        assert_eq!(Rules::thirteen().geese.len(), 13);
        assert_eq!(Rules::seventeen().geese.len(), 17);
        for rules in Rules::all() {
            assert!(rules.geese.iter().all(|&p| on_board(p) && p != FOX_START));
        }
    }

    #[test]
    fn geese_without_backward_moves() {
        let rules = Rules::seventeen();
        let mut p = Position::new(&rules);
        p.geese = vec![(3, 3)];
        p.fox = (0, 3);
        let targets: Vec<Point> = p.moves(&rules).iter().map(|m| m.to()).collect();
        assert!(targets.contains(&(2, 3)) && targets.contains(&(3, 2)) && targets.contains(&(3, 4)));
        assert!(!targets.contains(&(4, 3)), "no goose moves back");
        let rules = Rules::thirteen();
        let targets: Vec<Point> = p.moves(&rules).iter().map(|m| m.to()).collect();
        assert!(targets.contains(&(4, 3)));
    }

    #[test]
    fn the_fox_jumps_and_chains_jumps() {
        let rules = Rules::thirteen();
        let mut p = Position::new(&rules);
        p.to_move = Side::Fox;
        p.fox = (2, 2);
        p.geese = vec![(3, 2), (4, 3)];
        p.geese.sort();
        // Over (3,2) to (4,2), then over (4,3) to (4,4).
        let chain = p.moves(&rules).into_iter().find(|m| m.to() == (4, 4)).expect("a double jump");
        assert_eq!(chain.killed(), vec![(3, 2), (4, 3)]);
        // Stopping after the first jump is allowed too.
        assert!(p.moves(&rules).iter().any(|m| m.to() == (4, 2) && m.killed().len() == 1));
        p.play(&chain);
        assert!(p.geese.is_empty());
        assert_eq!(p.winner(&rules), Some(Side::Fox));
    }

    #[test]
    fn no_jump_without_an_empty_point_beyond() {
        let rules = Rules::thirteen();
        let mut p = Position::new(&rules);
        p.to_move = Side::Fox;
        p.fox = (0, 3);
        // A goose on (0,2) with the edge behind it cannot be jumped.
        p.geese = vec![(0, 2), (0, 4), (1, 3), (2, 3), (5, 5)];
        p.geese.retain(|&g| on_board(g));
        p.geese.sort();
        assert!(p.moves(&rules).is_empty(), "trapped: {:?}", p.moves(&rules));
        assert_eq!(p.winner(&rules), Some(Side::Geese));
    }

    /// Checked by brute force: no placement of three or fewer geese traps
    /// the fox on any point, so with three left the fox has won.
    #[test]
    fn no_trap_with_three_geese_anywhere() {
        let rules = Rules::thirteen();
        let all = points();
        for &fox in &all {
            let others: Vec<Point> = all.iter().copied().filter(|&q| q != fox).collect();
            for i in 0..others.len() {
                for j in i..others.len() {
                    for k in j..others.len() {
                        let mut geese = vec![others[i], others[j], others[k]];
                        geese.sort();
                        geese.dedup();
                        let p = Position { fox, geese, to_move: Side::Fox };
                        assert!(!p.moves(&rules).is_empty(), "trapped at {fox:?} by {:?}", p.geese);
                    }
                }
            }
        }
    }

    #[test]
    fn four_geese_can_trap_but_three_never() {
        // The trap in the test above takes four geese; with three, the fox has
        // won.
        let rules = Rules::thirteen();
        let mut p = Position::new(&rules);
        p.geese = vec![(4, 0), (4, 1), (4, 2)];
        assert_eq!(p.winner(&rules), Some(Side::Fox));
    }
}
