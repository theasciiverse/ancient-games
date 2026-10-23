//! The rules of Seega, as Edward Lane saw it played in Egypt, as data, and
//! the engine that plays them.
//!
//! The board (the seega) is a square of holes ('eyns) in the ground, five,
//! seven or nine a side. Each player has a piece (kelb, a dog) for every
//! hole but the centre, half of them each. Two are set down by rule, then
//! the players place two at a time until only the centre is empty. Then a
//! move is one step along a row or a column into an empty hole; a kelb
//! that comes to stand so that an enemy kelb lies between it and another
//! of its own takes that enemy kelb, and as long as its owner can take
//! again at once, he moves again. A player who cannot move asks for a way,
//! and the other must open one by giving up a kelb of his own.
//!
//! Holes are numbered row by row from the top left.

/// The two sides. Lane's pieces were little bits of stone or dingy brick
/// for one player and of red brick or tile for the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Stone,
    Brick,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::Stone => 0,
            Side::Brick => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::Stone => Side::Brick,
            Side::Brick => Side::Stone,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Stone => "Stone",
            Side::Brick => "Brick",
        }
    }

    pub const BOTH: [Side; 2] = [Side::Stone, Side::Brick];
}

/// Who places first (on Lane's a, a); our choice.
pub const PLACES_FIRST: Side = Side::Stone;
/// Who makes the first move once every kelb is down; Lane does not say,
/// and our choice is the other player, who placed last.
pub const MOVES_FIRST: Side = Side::Brick;
/// Moves without a kelb taken or given after which the game ends, the side
/// with more kelbs winning; Lane says nothing of the end. Our choice.
pub const QUIET: u16 = 40;

/// A rule set: Lane names three boards and one game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    /// Where it comes from, in a sentence or two, for the menu.
    pub source: &'static str,
    /// The heading of this rule set in the rulebook chapter, where the
    /// help opens.
    pub section: &'static str,
    /// Holes a side; odd.
    pub size: usize,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::khamsawee(), Rules::sebawee(), Rules::tisawee()]
    }

    pub fn khamsawee() -> Rules {
        Rules {
            id: "five",
            name: "Seega of five (khamsáwee)",
            source: "Edward Lane (1836), as played by the fellahin of Egypt: five rows of five holes, \
                     twelve kelbs each, placed two at a time.",
            section: "Seega of five",
            size: 5,
        }
    }

    pub fn sebawee() -> Rules {
        Rules {
            id: "seven",
            name: "Seega of seven (seb'áwee)",
            source: "Lane's second board: seven rows of seven, twenty-four kelbs each.",
            section: "Seega of seven and of nine",
            size: 7,
        }
    }

    pub fn tisawee() -> Rules {
        Rules {
            id: "nine",
            name: "Seega of nine (tis'áwee)",
            source: "Lane's third board: nine rows of nine, forty kelbs each.",
            section: "Seega of seven and of nine",
            size: 9,
        }
    }

    pub fn len(&self) -> usize {
        self.size * self.size
    }

    pub fn centre(&self) -> usize {
        self.len() / 2
    }

    /// A hole's row and column.
    pub fn rc(&self, p: usize) -> (usize, usize) {
        (p / self.size, p % self.size)
    }

    /// The hole one step from `p` in direction (dr, dc), if on the board.
    pub fn step(&self, p: usize, (dr, dc): (isize, isize)) -> Option<usize> {
        let (r, c) = self.rc(p);
        let r = r as isize + dr;
        let c = c as isize + dc;
        let n = self.size as isize;
        (r >= 0 && c >= 0 && r < n && c < n).then(|| r as usize * self.size + c as usize)
    }

    /// A hole's name: the column as a letter from the left, the row as a
    /// number from the top.
    pub fn name(&self, p: usize) -> String {
        let (r, c) = self.rc(p);
        format!("{}{}", (b'a' + c as u8) as char, r + 1)
    }

    /// Lane's holes a, a (the middle of the top and bottom rows), where the
    /// first player sets two kelbs, and b, b (the middle of the outer
    /// columns), where the second does. His figure shows the board of five;
    /// on the larger boards we take the same places.
    pub fn marked(&self, side: Side) -> [usize; 2] {
        let m = self.size / 2;
        let n = self.size - 1;
        if side == PLACES_FIRST {
            [m, n * self.size + m]
        } else {
            [m * self.size, m * self.size + n]
        }
    }
}

/// Up, down, left, right.
pub const DIRECTIONS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// A move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    /// A kelb set down in an empty hole, two to a turn.
    Place(usize),
    /// A kelb moved one step into an empty hole.
    Step { from: usize, to: usize },
    /// A kelb given up to open a way for the other player, who has none.
    Give(usize),
}

/// What the side to move is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stage {
    Placing,
    Moving,
    /// The other side cannot move: the side to move must open a way.
    Opening,
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
    pub stage: Stage,
    /// Kelbs the side to move has set down this turn, while placing.
    pub placed: u8,
    /// After a capture the mover goes on, but only to capture again.
    pub chain: bool,
    /// Moves since a kelb was last taken or given.
    pub quiet: u16,
}

impl Position {
    /// The board with Lane's four kelbs set down, the first player to
    /// place.
    pub fn new(rules: &Rules) -> Position {
        let mut at = vec![None; rules.len()];
        for side in Side::BOTH {
            for p in rules.marked(side) {
                at[p] = Some(side);
            }
        }
        Position { at, to_move: PLACES_FIRST, stage: Stage::Placing, placed: 0, chain: false, quiet: 0 }
    }

    pub fn count(&self, side: Side) -> usize {
        self.at.iter().filter(|s| **s == Some(side)).count()
    }

    /// The enemy kelbs a step from `from` to `to` would take: each one
    /// next to `to` with a kelb of the mover's straight beyond it.
    pub fn takes(&self, rules: &Rules, from: usize, to: usize) -> Vec<usize> {
        let Some(me) = self.at[from] else {
            return Vec::new();
        };
        DIRECTIONS
            .iter()
            .filter_map(|&d| {
                let over = rules.step(to, d)?;
                let beyond = rules.step(over, d)?;
                (self.at[over] == Some(me.other()) && self.at[beyond] == Some(me)).then_some(over)
            })
            .collect()
    }

    /// The steps the side to move could make, ignoring a chain.
    fn steps(&self, rules: &Rules, side: Side) -> Vec<Move> {
        let mut out = Vec::new();
        for from in 0..rules.len() {
            if self.at[from] != Some(side) {
                continue;
            }
            for d in DIRECTIONS {
                if let Some(to) = rules.step(from, d)
                    && self.at[to].is_none()
                {
                    out.push(Move::Step { from, to });
                }
            }
        }
        out
    }

    /// The legal moves for the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        let me = self.to_move;
        match self.stage {
            Stage::Placing => {
                (0..rules.len()).filter(|&p| p != rules.centre() && self.at[p].is_none()).map(Move::Place).collect()
            }
            Stage::Opening => (0..rules.len())
                .filter(|&p| {
                    self.at[p] == Some(me)
                        && DIRECTIONS.iter().any(|&d| rules.step(p, d).is_some_and(|q| self.at[q] == Some(me.other())))
                })
                .map(Move::Give)
                .collect(),
            Stage::Moving => {
                let steps = self.steps(rules, me);
                if self.chain {
                    steps.into_iter().filter(|m| self.taken(rules, m) > 0).collect()
                } else {
                    steps
                }
            }
        }
    }

    /// How many kelbs a move takes.
    pub fn taken(&self, rules: &Rules, mv: &Move) -> usize {
        match *mv {
            Move::Step { from, to } => self.takes(rules, from, to).len(),
            _ => 0,
        }
    }

    pub fn play(&mut self, rules: &Rules, mv: &Move) {
        let me = self.to_move;
        match *mv {
            Move::Place(p) => {
                self.at[p] = Some(me);
                self.placed += 1;
                if self.placed == 2 {
                    self.placed = 0;
                    self.to_move = me.other();
                }
                if self.at.iter().filter(|s| s.is_none()).count() == 1 {
                    self.stage = Stage::Moving;
                    self.to_move = MOVES_FIRST;
                    self.placed = 0;
                    self.settle(rules);
                }
            }
            Move::Step { from, to } => {
                let taken = self.takes(rules, from, to);
                self.at[to] = self.at[from].take();
                for &q in &taken {
                    self.at[q] = None;
                }
                if taken.is_empty() {
                    self.quiet += 1;
                } else {
                    self.quiet = 0;
                    // He takes again at once, if he can, before the other
                    // moves.
                    let again = self.count(me.other()) > 0
                        && self.steps(rules, me).iter().any(|m| self.taken(rules, m) > 0);
                    if again {
                        self.chain = true;
                        return;
                    }
                }
                self.chain = false;
                self.to_move = me.other();
                self.settle(rules);
            }
            Move::Give(p) => {
                self.at[p] = None;
                self.quiet = 0;
                self.stage = Stage::Moving;
                self.to_move = me.other();
            }
        }
    }

    /// If the side to move has kelbs but no step, the other must open a
    /// way.
    fn settle(&mut self, rules: &Rules) {
        let me = self.to_move;
        if self.stage == Stage::Moving && self.count(me) > 0 && self.steps(rules, me).is_empty() {
            self.stage = Stage::Opening;
            self.to_move = me.other();
        }
    }

    /// How the game ended, if it has: a side with no kelbs left has lost;
    /// after `QUIET` moves with nothing taken, more kelbs win.
    pub fn outcome(&self) -> Option<Outcome> {
        if self.stage == Stage::Placing {
            return None;
        }
        for side in Side::BOTH {
            if self.count(side) == 0 {
                return Some(Outcome::Win(side.other()));
            }
        }
        if self.quiet >= QUIET {
            let (s, b) = (self.count(Side::Stone), self.count(Side::Brick));
            return Some(match s.cmp(&b) {
                std::cmp::Ordering::Greater => Outcome::Win(Side::Stone),
                std::cmp::Ordering::Less => Outcome::Win(Side::Brick),
                std::cmp::Ordering::Equal => Outcome::Draw,
            });
        }
        None
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn at(rules: &Rules, name: &str) -> usize {
        (0..rules.len()).find(|&p| rules.name(p) == name).unwrap()
    }

    /// A position in the moving stage with the named kelbs only.
    pub fn board(rules: &Rules, to_move: Side, kelbs: &[(&str, Side)]) -> Position {
        let mut p = Position::new(rules);
        p.at = vec![None; rules.len()];
        for &(name, side) in kelbs {
            p.at[at(rules, name)] = Some(side);
        }
        p.stage = Stage::Moving;
        p.to_move = to_move;
        p
    }

    #[test]
    fn placing_two_at_a_time_then_brick_moves_into_the_centre() {
        let rules = Rules::khamsawee();
        let mut p = Position::new(&rules);
        assert_eq!(p.count(Side::Stone), 2);
        assert_eq!(p.at[at(&rules, "c1")], Some(Side::Stone));
        assert_eq!(p.at[at(&rules, "a3")], Some(Side::Brick));
        // 20 holes left, the centre not among them.
        assert_eq!(p.moves(&rules).len(), 20);
        let mut turns = Vec::new();
        while p.stage == Stage::Placing {
            turns.push(p.to_move);
            let mv = p.moves(&rules)[0];
            p.play(&rules, &mv);
        }
        assert_eq!(turns.len(), 20);
        assert_eq!(&turns[..4], [Side::Stone, Side::Stone, Side::Brick, Side::Brick]);
        assert_eq!((p.count(Side::Stone), p.count(Side::Brick)), (12, 12));
        assert_eq!(p.at[rules.centre()], None);
        // Brick moves first, into the centre, if he has a kelb beside it.
        if p.stage == Stage::Moving {
            assert_eq!(p.to_move, MOVES_FIRST);
            assert!(p.moves(&rules).iter().all(|m| matches!(m, Move::Step { to, .. } if *to == rules.centre())));
        } else {
            assert_eq!(p.to_move, MOVES_FIRST.other());
        }
    }

    #[test]
    fn a_kelb_between_two_enemies_takes_and_the_mover_goes_on() {
        let rules = Rules::khamsawee();
        use Side::*;
        // Stone c2 steps to c3: takes d3 (e3 beyond) and b3 (a3 beyond).
        let mut p = board(&rules, Stone, &[("c2", Stone), ("d3", Brick), ("e3", Stone), ("b3", Brick), ("a3", Stone), ("e5", Brick)]);
        let mv = Move::Step { from: at(&rules, "c2"), to: at(&rules, "c3") };
        assert_eq!(p.taken(&rules, &mv), 2);
        p.play(&rules, &mv);
        assert_eq!(p.count(Brick), 1);
        // Nothing more to take: Brick's turn.
        assert_eq!((p.to_move, p.chain), (Brick, false));
    }

    #[test]
    fn a_kelb_may_stand_between_two_enemies() {
        let rules = Rules::khamsawee();
        use Side::*;
        let mut p = board(&rules, Brick, &[("c2", Brick), ("b3", Stone), ("d3", Stone), ("a1", Brick), ("e1", Stone)]);
        let mv = Move::Step { from: at(&rules, "c2"), to: at(&rules, "c3") };
        assert_eq!(p.taken(&rules, &mv), 0);
        p.play(&rules, &mv);
        assert_eq!(p.at[at(&rules, "c3")], Some(Brick));
        assert_eq!(p.to_move, Stone);
    }

    #[test]
    fn the_chain_goes_on_only_with_a_capture() {
        let rules = Rules::khamsawee();
        use Side::*;
        // Stone b1 to c1 takes c2 (c3 beyond); then only a4 to a3, taking
        // b3 (c3 beyond), may follow.
        let mut p = board(&rules, Stone, &[("b1", Stone), ("c2", Brick), ("c3", Stone), ("b3", Brick), ("a4", Stone), ("e5", Brick)]);
        p.play(&rules, &Move::Step { from: at(&rules, "b1"), to: at(&rules, "c1") });
        assert_eq!(p.to_move, Stone);
        assert!(p.chain);
        assert_eq!(p.moves(&rules), [Move::Step { from: at(&rules, "a4"), to: at(&rules, "a3") }]);
        p.play(&rules, &Move::Step { from: at(&rules, "a4"), to: at(&rules, "a3") });
        assert_eq!(p.to_move, Brick);
        assert!(!p.chain);
        assert_eq!(p.count(Brick), 1);
    }

    #[test]
    fn a_player_without_a_move_is_given_a_way() {
        let rules = Rules::khamsawee();
        use Side::*;
        // Brick a1 is shut in by Stone a2 and b1; Stone moves e5 to e4.
        let mut p = board(&rules, Stone, &[("a1", Brick), ("a2", Stone), ("b1", Stone), ("e5", Stone)]);
        p.play(&rules, &Move::Step { from: at(&rules, "e5"), to: at(&rules, "e4") });
        assert_eq!(p.stage, Stage::Opening);
        assert_eq!(p.to_move, Stone);
        let gives = p.moves(&rules);
        assert_eq!(gives, [Move::Give(at(&rules, "b1")), Move::Give(at(&rules, "a2"))]);
        p.play(&rules, &Move::Give(at(&rules, "a2")));
        assert_eq!((p.stage, p.to_move), (Stage::Moving, Brick));
        assert_eq!(p.moves(&rules), [Move::Step { from: at(&rules, "a1"), to: at(&rules, "a2") }]);
    }

    #[test]
    fn the_end() {
        let rules = Rules::khamsawee();
        use Side::*;
        let mut p = board(&rules, Stone, &[("c2", Stone), ("c4", Brick), ("c5", Stone)]);
        p.play(&rules, &Move::Step { from: at(&rules, "c2"), to: at(&rules, "c3") });
        assert_eq!(p.outcome(), Some(Outcome::Win(Stone)));
        let mut q = board(&rules, Stone, &[("a1", Stone), ("e5", Brick)]);
        q.quiet = QUIET - 1;
        assert_eq!(q.outcome(), None);
        q.play(&rules, &Move::Step { from: at(&rules, "a1"), to: at(&rules, "a2") });
        assert_eq!(q.outcome(), Some(Outcome::Draw));
    }
}
