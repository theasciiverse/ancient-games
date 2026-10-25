//! The rules of Alquerque and Fanorona, as data, and one engine that plays
//! them both.
//!
//! Both are played on a grid of points joined along the rows and columns,
//! and along the diagonals through every other point: the corners and
//! every point whose row and column add up to an even number. A man moves
//! one step along a line to an empty point. In Alquerque a man takes by
//! leaping over an enemy man next to it onto the empty point beyond, as
//! in draughts, and may leap on; in Fanorona a man takes by moving
//! towards an enemy line (approach) or away from one (withdrawal), and
//! the whole unbroken line of enemy men in that direction goes. Where a
//! source is silent, the rulebook chapter says what this game does ("Our
//! choice").
//!
//! Points are numbered row by row from the top left; a point's name is
//! its column as a letter from the left and its row as a number from the
//! bottom, as on the program's board.

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

/// The eight directions as (rows down, columns right).
pub const DIRECTIONS: [(i8, i8); 8] = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (-1, 1), (1, -1), (1, 1)];

/// How men take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capture {
    /// Leaping over an enemy man next to it onto the empty point beyond.
    Leap,
    /// Fanorona: moving onto the point next to an enemy line (approach),
    /// or away from one standing next to the man (withdrawal); the whole
    /// unbroken line of enemy men in that direction is taken.
    Line,
}

/// What limits a man that takes several times in one turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chain {
    /// Alquerque: it leaps on over any man it can, in any direction.
    Free,
    /// Montgomery's Fanorona: never straight back along the line it came.
    NoReturn,
    /// Today's Fanorona: never twice running in the same direction, and
    /// never to a point it has already stood on this turn.
    Modern,
}

/// Where the men stand at the start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    /// Alquerque: each side fills the two rows nearest it and the two
    /// points on its right of the middle row; only the centre is empty.
    Alquerque,
    /// Fanorona: each side fills the two rows nearest it; the middle row
    /// alternates, the centre empty (on the board of five as on the board
    /// of nine).
    Fanorona,
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
    pub cols: usize,
    pub rows: usize,
    pub setup: Setup,
    pub capture: Capture,
    /// Whether a side able to take must take.
    pub must_take: bool,
    /// Whether a man that has taken and can take again must go on.
    pub must_go_on: bool,
    pub chain: Chain,
    /// Whether a man may step (not take) back towards its own side.
    pub backwards: bool,
    /// Bell: no man may come back to a point it has stood on.
    pub no_return: bool,
    /// Bell: a man on the other side's back row moves only to take.
    pub back_row_takes_only: bool,
    /// Fanorona: the first turn of the game takes once only.
    pub first_takes_once: bool,
    /// The vela: Black, who won the last game, may not take until he has
    /// given up 17 men; White's captures take one man and end the turn.
    pub vela: bool,
    /// Moves with nothing taken after which the game is drawn.
    pub quiet: u16,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![
            Rules::alquerque(),
            Rules::alquerque_bell(),
            Rules::fanorona(),
            Rules::fanorona_montgomery(),
            Rules::vela(),
            Rules::fanoron_dimy(),
        ]
    }

    pub fn alquerque() -> Rules {
        Rules {
            id: "alquerque",
            name: "Alquerque de doze, after Alfonso X",
            source: "The Book of Games of Alfonso X of Castile (1283): twelve men each on a board of \
                     five by five points, taking by leaping, several at once. Alfonso does not make \
                     taking compulsory.",
            section: "Alquerque de doze, after Alfonso X",
            cols: 5,
            rows: 5,
            setup: Setup::Alquerque,
            capture: Capture::Leap,
            must_take: false,
            must_go_on: false,
            chain: Chain::Free,
            backwards: true,
            no_return: false,
            back_row_takes_only: false,
            first_takes_once: false,
            vela: false,
            quiet: 50,
        }
    }

    pub fn alquerque_bell() -> Rules {
        Rules {
            id: "alquerque-bell",
            name: "Alquerque, after Bell",
            source: "R. C. Bell (1969): Alfonso's game with Bell's own rules added: taking compulsory, no \
                     step backwards, no man back to a point it has stood on.",
            section: "Alquerque, after Bell",
            must_take: true,
            must_go_on: true,
            backwards: false,
            no_return: true,
            back_row_takes_only: true,
            ..Rules::alquerque()
        }
    }

    pub fn fanorona() -> Rules {
        Rules {
            id: "fanorona",
            name: "Fanorona, today's rules",
            source: "The national game of Madagascar as it is played today, after Bell and Chauvicourt as \
                     Schadd and others state it (2008), who proved that best play draws.",
            section: "Fanorona, today's rules",
            cols: 9,
            rows: 5,
            setup: Setup::Fanorona,
            capture: Capture::Line,
            must_take: true,
            must_go_on: false,
            chain: Chain::Modern,
            backwards: true,
            no_return: false,
            back_row_takes_only: false,
            first_takes_once: false,
            vela: false,
            quiet: 50,
        }
    }

    pub fn fanorona_montgomery() -> Rules {
        Rules {
            id: "fanorona-1886",
            name: "Fanorona, after Montgomery (1886)",
            source: "The earliest account of the rules, W. Montgomery's in the Antananarivo Annual (1886): \
                     the first turn takes once only, and a man taking again must not go back along \
                     the line it came.",
            section: "Fanorona, after Montgomery",
            chain: Chain::NoReturn,
            first_takes_once: true,
            ..Rules::fanorona()
        }
    }

    pub fn vela() -> Rules {
        Rules {
            id: "vela",
            name: "Vela, the game after a defeat",
            source: "Montgomery (1886): the winner of the last game (Black) gives up seventeen men, \
                     taken one at a time, before he may take any himself; then his five play on \
                     against the loser's whole side.",
            section: "Vela",
            vela: true,
            ..Rules::fanorona_montgomery()
        }
    }

    pub fn fanoron_dimy() -> Rules {
        Rules {
            id: "fanoron-dimy",
            name: "Fanoron-dimy, the board of five",
            source: "The small, quick fanorona on five lines that Malagasy sources name; its setup and \
                     rules here are today's, by analogy.",
            section: "Fanoron-dimy",
            cols: 5,
            ..Rules::fanorona()
        }
    }

    pub fn len(&self) -> usize {
        self.cols * self.rows
    }

    pub fn centre(&self) -> usize {
        self.len() / 2
    }

    /// A point's row (from the top) and column.
    pub fn rc(&self, p: usize) -> (usize, usize) {
        (p / self.cols, p % self.cols)
    }

    /// A point's name: its column as a letter from the left, its row as a
    /// number from the bottom.
    pub fn name(&self, p: usize) -> String {
        let (r, c) = self.rc(p);
        format!("{}{}", (b'a' + c as u8) as char, self.rows - r)
    }

    /// The point named `name`, if any.
    pub fn point(&self, name: &str) -> Option<usize> {
        (0..self.len()).find(|&p| self.name(p) == name)
    }

    /// Whether diagonals run through `p`.
    pub fn diagonal(&self, p: usize) -> bool {
        let (r, c) = self.rc(p);
        (r + c) % 2 == 0
    }

    /// The point one step from `p` in direction `d`, if a line joins them.
    pub fn step(&self, p: usize, (dr, dc): (i8, i8)) -> Option<usize> {
        if dr != 0 && dc != 0 && !self.diagonal(p) {
            return None;
        }
        self.towards(p, (dr, dc))
    }

    /// The point one step from `p` in direction `d` on the grid, joined
    /// or not (the cursor uses it).
    pub fn towards(&self, p: usize, (dr, dc): (i8, i8)) -> Option<usize> {
        let (r, c) = self.rc(p);
        let r = r as isize + dr as isize;
        let c = c as isize + dc as isize;
        (r >= 0 && c >= 0 && r < self.rows as isize && c < self.cols as isize)
            .then(|| r as usize * self.cols + c as usize)
    }

    /// Which way is forward for `side`: White starts at the bottom and
    /// moves up the board.
    pub fn forward(side: Side) -> i8 {
        match side {
            Side::White => -1,
            Side::Black => 1,
        }
    }
}

/// One step of a turn: a man from one point to the next, and the men it
/// takes there.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    pub from: u8,
    pub to: u8,
    pub taken: Vec<u8>,
}

impl Step {
    /// Fanorona: whether the step takes by approach (the line lies ahead
    /// of where it lands), not by withdrawal.
    pub fn approach(&self, rules: &Rules) -> bool {
        let d = direction(rules, self.from as usize, self.to as usize);
        self.taken.first().is_some_and(|&q| rules.step(self.to as usize, d) == Some(q as usize))
    }
}

/// A whole turn: one man's steps, each after the first a capture.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub steps: Vec<Step>,
}

impl Move {
    pub fn from(&self) -> usize {
        self.steps[0].from as usize
    }

    pub fn to(&self) -> usize {
        self.steps[self.steps.len() - 1].to as usize
    }

    pub fn taken(&self) -> impl Iterator<Item = usize> + '_ {
        self.steps.iter().flat_map(|s| s.taken.iter().map(|&q| q as usize))
    }

    pub fn captures(&self) -> usize {
        self.steps.iter().map(|s| s.taken.len()).sum()
    }
}

/// How a game ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win(Side),
    Draw,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub at: Vec<Option<Side>>,
    pub to_move: Side,
    /// Turns played so far.
    pub turns: u16,
    /// Turns since a man was last taken.
    pub quiet: u16,
    /// Bell's rule: the points each man has stood on, kept on the point
    /// where it stands (empty points hold nothing).
    pub been: Vec<u64>,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let mut at = vec![None; rules.len()];
        let mid = rules.rows / 2;
        for p in 0..rules.len() {
            let (r, c) = rules.rc(p);
            at[p] = if r < mid {
                Some(Side::Black)
            } else if r > mid {
                Some(Side::White)
            } else if p == rules.centre() {
                None
            } else {
                match rules.setup {
                    // Each side's two on its right, as it faces the board:
                    // White's on the right of the screen, Black's on the
                    // left.
                    Setup::Alquerque => Some(if c > rules.cols / 2 { Side::White } else { Side::Black }),
                    // Black, White, Black, White, empty, Black, White, ...
                    // from the left, as the board is drawn.
                    Setup::Fanorona => {
                        let k = if c < rules.cols / 2 { c } else { c - 1 };
                        Some(if k % 2 == 0 { Side::Black } else { Side::White })
                    }
                }
            };
        }
        let been = (0..rules.len()).map(|p| if at[p].is_some() { 1u64 << p } else { 0 }).collect();
        Position { at, to_move: Side::White, turns: 0, quiet: 0, been }
    }

    pub fn count(&self, side: Side) -> usize {
        self.at.iter().filter(|s| **s == Some(side)).count()
    }

    /// The captures a man of `me` could make by stepping from `from` in
    /// direction `d`, on this board, each with the men it takes; empty if
    /// none. (For Fanorona one step may offer both an approach and a
    /// withdrawal: two entries.)
    fn captures_from(&self, rules: &Rules, me: Side, from: usize, d: (i8, i8)) -> Vec<(usize, Vec<u8>)> {
        let mut out = Vec::new();
        match rules.capture {
            Capture::Leap => {
                if let Some(over) = rules.step(from, d)
                    && self.at[over] == Some(me.other())
                    && let Some(to) = rules.step(over, d)
                    && self.at[to].is_none()
                {
                    out.push((to, vec![over as u8]));
                }
            }
            Capture::Line => {
                let Some(to) = rules.step(from, d) else {
                    return out;
                };
                if self.at[to].is_some() {
                    return out;
                }
                // Approach: the line beyond where it lands.
                let line = |start: Option<usize>, dir: (i8, i8)| {
                    let mut taken = Vec::new();
                    let mut q = start;
                    while let Some(p) = q {
                        if self.at[p] != Some(me.other()) {
                            break;
                        }
                        taken.push(p as u8);
                        q = rules.step(p, dir);
                    }
                    taken
                };
                let ahead = line(rules.step(to, d), d);
                if !ahead.is_empty() {
                    out.push((to, ahead));
                }
                // Withdrawal: the line behind where it left.
                let back = (-d.0, -d.1);
                let behind = line(rules.step(from, back), back);
                if !behind.is_empty() {
                    out.push((to, behind));
                }
            }
        }
        out
    }

    /// Whether the vela's opening part is still on: the winner of the
    /// last game (Black) has not yet given up seventeen men.
    pub fn vela_on(&self, rules: &Rules) -> bool {
        rules.vela && self.count(Side::Black) > 5
    }

    /// Whether the man on `from` may come to `to` (Bell's rule).
    fn may_land(&self, rules: &Rules, from: usize, to: usize) -> bool {
        !rules.no_return || self.been[from] & 1 << to == 0
    }

    /// Whether `p` is on the back row of the side that is not `me`.
    fn their_back_row(&self, rules: &Rules, me: Side, p: usize) -> bool {
        let (r, _) = rules.rc(p);
        match me {
            Side::White => r == 0,
            Side::Black => r == rules.rows - 1,
        }
    }

    /// The captures the man of `me` on `from` could make now, as steps.
    fn captures_at(&self, rules: &Rules, me: Side, from: usize) -> Vec<((i8, i8), Step)> {
        let mut out = Vec::new();
        for d in DIRECTIONS {
            for (to, mut taken) in self.captures_from(rules, me, from, d) {
                if !self.may_land(rules, from, to) {
                    continue;
                }
                // In the vela the loser's captures take one man only, the
                // nearest.
                if self.vela_on(rules) {
                    taken.truncate(1);
                }
                out.push((d, Step { from: from as u8, to: to as u8, taken }));
            }
        }
        out
    }

    /// The legal turns for the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        let me = self.to_move;
        let vela = self.vela_on(rules);
        // In the vela the winner of the last game may not take.
        let can_take = !(vela && me == Side::Black);
        let mut out = Vec::new();
        let mut took = false;
        if can_take {
            for from in 0..rules.len() {
                if self.at[from] != Some(me) {
                    continue;
                }
                for (d, step) in self.captures_at(rules, me, from) {
                    took = true;
                    let mut board = self.clone();
                    board.apply(&step);
                    let once = (rules.first_takes_once && self.turns == 0) || vela;
                    let mut visited = vec![step.from as usize, step.to as usize];
                    board.chains(rules, me, vec![step], &mut visited, d, once, &mut out);
                }
            }
        }
        if took && rules.must_take {
            return out;
        }
        for from in 0..rules.len() {
            if self.at[from] != Some(me) {
                continue;
            }
            if rules.back_row_takes_only && self.their_back_row(rules, me, from) {
                continue;
            }
            for d in DIRECTIONS {
                if !rules.backwards && d.0 == -Rules::forward(me) {
                    continue;
                }
                if let Some(to) = rules.step(from, d)
                    && self.at[to].is_none()
                    && self.may_land(rules, from, to)
                {
                    out.push(Move { steps: vec![Step { from: from as u8, to: to as u8, taken: Vec::new() }] });
                }
            }
        }
        out
    }

    /// Goes on from the turn so far, `done`: every way the man that has
    /// just taken may take again, and stopping, where stopping is allowed.
    #[allow(clippy::too_many_arguments)]
    fn chains(
        &self,
        rules: &Rules,
        me: Side,
        done: Vec<Step>,
        visited: &mut Vec<usize>,
        last_dir: (i8, i8),
        once: bool,
        out: &mut Vec<Move>,
    ) {
        let at = done[done.len() - 1].to as usize;
        let mut next = Vec::new();
        if !once {
            for (d, step) in self.captures_at(rules, me, at) {
                let allowed = match rules.chain {
                    Chain::Free => true,
                    // Not straight back along the line it came.
                    Chain::NoReturn => d != (-last_dir.0, -last_dir.1),
                    Chain::Modern => d != last_dir && !visited.contains(&(step.to as usize)),
                };
                if allowed {
                    next.push((d, step));
                }
            }
        }
        if next.is_empty() || !rules.must_go_on {
            out.push(Move { steps: done.clone() });
        }
        for (d, step) in next {
            let mut board = self.clone();
            board.apply(&step);
            visited.push(step.to as usize);
            let mut steps = done.clone();
            steps.push(step);
            board.chains(rules, me, steps, visited, d, once, out);
            visited.pop();
        }
    }

    fn apply(&mut self, step: &Step) {
        let (from, to) = (step.from as usize, step.to as usize);
        self.at[to] = self.at[from].take();
        self.been[to] = self.been[from] | 1 << to;
        self.been[from] = 0;
        for &q in &step.taken {
            self.at[q as usize] = None;
            self.been[q as usize] = 0;
        }
    }

    pub fn play(&mut self, mv: &Move) {
        for step in &mv.steps {
            self.apply(step);
        }
        self.quiet = if mv.captures() > 0 { 0 } else { self.quiet + 1 };
        self.turns += 1;
        self.to_move = self.to_move.other();
    }

    /// How the game ended, if it has: a side with no men, or no move,
    /// has lost; after `rules.quiet` turns with nothing taken, a draw.
    pub fn outcome(&self, rules: &Rules) -> Option<Outcome> {
        for side in Side::BOTH {
            if self.count(side) == 0 {
                return Some(Outcome::Win(side.other()));
            }
        }
        if self.moves(rules).is_empty() {
            return Some(Outcome::Win(self.to_move.other()));
        }
        if self.quiet >= rules.quiet {
            return Some(Outcome::Draw);
        }
        None
    }
}

/// The direction of a step from `a` to `b`.
pub fn direction(rules: &Rules, a: usize, b: usize) -> (i8, i8) {
    let (ra, ca) = rules.rc(a);
    let (rb, cb) = rules.rc(b);
    let sign = |x: isize| x.signum() as i8;
    (sign(rb as isize - ra as isize), sign(cb as isize - ca as isize))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn board(rules: &Rules, to_move: Side, men: &[(&str, Side)]) -> Position {
        let mut p = Position::new(rules);
        p.at = vec![None; rules.len()];
        for &(name, side) in men {
            p.at[rules.point(name).unwrap()] = Some(side);
        }
        p.been = (0..rules.len()).map(|q| if p.at[q].is_some() { 1u64 << q } else { 0 }).collect();
        p.to_move = to_move;
        p.turns = 10;
        p
    }

    fn picture(rules: &Rules, p: &Position) -> String {
        (0..rules.rows)
            .map(|r| {
                (0..rules.cols)
                    .map(|c| match p.at[r * rules.cols + c] {
                        Some(Side::White) => 'W',
                        Some(Side::Black) => 'B',
                        None => '.',
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    #[test]
    fn the_starts() {
        let a = Rules::alquerque();
        assert_eq!(picture(&a, &Position::new(&a)), "BBBBB/BBBBB/BB.WW/WWWWW/WWWWW");
        let f = Rules::fanorona();
        assert_eq!(picture(&f, &Position::new(&f)), "BBBBBBBBB/BBBBBBBBB/BWBW.BWBW/WWWWWWWWW/WWWWWWWWW");
        for rules in Rules::all() {
            let p = Position::new(&rules);
            assert_eq!(p.count(Side::White), p.count(Side::Black));
        }
    }

    #[test]
    fn diagonals_only_through_even_points() {
        let f = Rules::fanorona();
        // b1 is off the diagonals; b2 is on the long one from a1.
        let b1 = f.point("b1").unwrap();
        assert!(f.step(b1, (-1, 1)).is_none());
        let b2 = f.point("b2").unwrap();
        assert_eq!(f.step(b2, (1, -1)).map(|p| f.name(p)), Some("a1".to_string()));
        let c3 = f.point("c3").unwrap();
        assert_eq!(f.step(c3, (-1, 1)).map(|p| f.name(p)), Some("d4".to_string()));
    }

    #[test]
    fn leaping_and_leaping_on() {
        let a = Rules::alquerque_bell();
        use Side::*;
        // White a1 leaps b2 to c3, then d4 to e5.
        let p = board(&a, White, &[("a1", White), ("b2", Black), ("d4", Black), ("e1", Black)]);
        let moves = p.moves(&a);
        assert_eq!(moves.len(), 1);
        let names: Vec<String> = moves[0].taken().map(|q| a.name(q)).collect();
        assert_eq!(names, ["b2", "d4"]);
        assert_eq!(a.name(moves[0].to()), "e5");
    }

    #[test]
    fn alfonso_lets_a_man_stop_or_not_take() {
        let a = Rules::alquerque();
        use Side::*;
        let p = board(&a, White, &[("a1", White), ("b2", Black), ("d4", Black), ("e1", Black)]);
        let moves = p.moves(&a);
        // A step, one leap, or both leaps.
        assert!(moves.iter().any(|m| m.captures() == 0));
        assert!(moves.iter().any(|m| m.captures() == 1));
        assert!(moves.iter().any(|m| m.captures() == 2));
    }

    #[test]
    fn bell_forbids_backwards_and_returning() {
        let a = Rules::alquerque_bell();
        use Side::*;
        let mut p = board(&a, White, &[("c3", White), ("a5", Black)]);
        let to: Vec<String> = p.moves(&a).iter().map(|m| a.name(m.to())).collect();
        // Forward, diagonally forward, sideways: not c2, b2, d2.
        assert_eq!(to.len(), 5, "{to:?}");
        assert!(!to.contains(&"c2".to_string()));
        // c3 to b3, then b3 may not come back to c3.
        let mv = p.moves(&a).into_iter().find(|m| a.name(m.to()) == "b3").unwrap();
        p.play(&mv);
        p.to_move = White;
        assert!(p.moves(&a).iter().all(|m| a.name(m.to()) != "c3"));
        // On Black's back row a man moves only to take.
        let q = board(&a, White, &[("c5", White), ("a1", Black)]);
        assert!(q.moves(&a).is_empty());
    }

    #[test]
    fn montgomery_only_forbids_going_back() {
        let f = Rules::fanorona_montgomery();
        use Side::*;
        // c1 to d1 withdraws from b1; d1 to e1 (the same way again)
        // approaches f1: allowed by Montgomery, not today.
        let p = board(&f, White, &[("c1", White), ("b1", Black), ("f1", Black), ("i5", Black)]);
        let two = |r: &Rules| p.moves(r).iter().any(|m| m.steps.len() == 2);
        assert!(two(&f));
        assert!(!two(&Rules::fanorona()));
    }

    #[test]
    fn the_vela() {
        use Side::*;
        let v = Rules::vela();
        let mut p = Position::new(&v);
        // White, the loser, moves first and takes one man only.
        let moves = p.moves(&v);
        assert!(moves.iter().all(|m| m.captures() == 1 && m.steps.len() == 1), "{:?}", moves[0]);
        p.play(&moves[0]);
        // Black may not take, though he could.
        assert!(p.moves(&v).iter().all(|m| m.captures() == 0));
        // With five men left Black plays as usual.
        let mut q = board(&v, Black, &[("e3", Black), ("a1", Black), ("a2", Black), ("a3", Black), ("a4", Black), ("g3", White), ("h3", White)]);
        q.turns = 40;
        assert!(q.moves(&v).iter().any(|m| m.captures() == 2));
    }

    #[test]
    fn approach_and_withdrawal() {
        let f = Rules::fanorona();
        use Side::*;
        // White on e3 moving to f3 approaches g3 h3 (i3 white stops the
        // line); moving to d3 withdraws from f3 g3 h3? No: f3 is empty
        // here, so moving west from e3 withdraws from nothing.
        let p = board(&f, White, &[("e3", White), ("g3", Black), ("h3", Black), ("i3", White), ("a1", Black)]);
        let moves = p.moves(&f);
        let east: Vec<&Move> = moves.iter().filter(|m| f.name(m.to()) == "f3").collect();
        assert_eq!(east.len(), 1);
        assert_eq!(east[0].taken().map(|q| f.name(q)).collect::<Vec<_>>(), ["g3", "h3"]);
        // Captures are compulsory: every move takes.
        assert!(moves.iter().all(|m| m.captures() > 0));
        // Withdrawal: Black on d3 next to White e3 steps away to c3.
        let q = board(&f, Black, &[("e3", White), ("f3", White), ("d3", Black), ("i5", White)]);
        let w: Vec<Move> = q.moves(&f).into_iter().filter(|m| f.name(m.to()) == "c3").collect();
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].captures(), 2);
    }

    #[test]
    fn a_step_may_offer_both() {
        let f = Rules::fanorona();
        use Side::*;
        // e3 to f3: approach g3, or withdraw from d3.
        let p = board(&f, White, &[("e3", White), ("g3", Black), ("d3", Black)]);
        let to_f3: Vec<Move> = p.moves(&f).into_iter().filter(|m| f.name(m.to()) == "f3" && m.steps.len() == 1).collect();
        assert_eq!(to_f3.len(), 2);
    }

    #[test]
    fn fanorona_chains_turn_and_never_revisit() {
        let f = Rules::fanorona();
        use Side::*;
        // White c1 to d1 approaching e1, then d1 to d2 approaching d3 (a
        // new direction), or stopping after the first.
        let p = board(&f, White, &[("c1", White), ("e1", Black), ("d3", Black), ("i5", Black)]);
        let moves = p.moves(&f);
        let lens: Vec<usize> = moves.iter().filter(|m| m.from() == f.point("c1").unwrap()).map(|m| m.steps.len()).collect();
        assert!(lens.contains(&1) && lens.contains(&2), "{lens:?}");
        // Not twice the same way: c1 to d1 (approach e1) cannot go on to
        // e1 (east again) even if it then approached something.
        let q = board(&f, White, &[("b1", White), ("d1", Black), ("f1", Black), ("i5", Black)]);
        for m in q.moves(&f) {
            for w in m.steps.windows(2) {
                assert_ne!(direction(&f, w[0].from as usize, w[0].to as usize), direction(&f, w[1].from as usize, w[1].to as usize));
            }
        }
    }

    #[test]
    fn the_first_turn_takes_once() {
        let f = Rules::fanorona();
        let p = Position::new(&f);
        let moves = p.moves(&f);
        assert!(!moves.is_empty());
        assert!(moves.iter().all(|m| m.steps.len() == 1 && m.captures() > 0));
    }

    #[test]
    fn games_end_at_random() {
        for rules in Rules::all() {
            let mut rng = ancient_core::rng::Rng::new(7);
            for _ in 0..20 {
                let mut p = Position::new(&rules);
                let mut n = 0;
                while p.outcome(&rules).is_none() {
                    let moves = p.moves(&rules);
                    let mv = &moves[(rng.next() % moves.len() as u64) as usize];
                    let before = p.count(p.to_move.other());
                    p.play(mv);
                    assert_eq!(p.count(p.to_move), before - mv.captures());
                    n += 1;
                    assert!(n < 2000);
                }
            }
        }
    }
}
