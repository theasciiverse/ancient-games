//! The rules of Mū tōrere, as data, and the engine that plays them.
//!
//! The board is an eight-pointed star: eight points (kewai) round a
//! centre (pūtahi). Each kewai is joined to the two beside it and to the
//! centre. Each side has four men, which start on four neighbouring kewai,
//! so that only the centre is empty. A move takes a man to the one empty
//! point, if it is joined to it. Who cannot move has lost.
//!
//! The kewai are numbered 0 to 7 clockwise from the top; the centre is 8.
//! Where the sources differ, chiefly on when a man may enter the centre,
//! each version is a rule set, and the rulebook chapter says which source
//! gives it.

/// The centre's number.
pub const CENTRE: usize = 8;
/// Every point: the eight kewai and the centre.
pub const POINTS: usize = 9;

/// The two sides. Each rule set says which begins.
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

/// When a man may move from a kewai into the centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Centre {
    /// Only if one of the kewai beside it holds an enemy man.
    BesideEnemy,
    /// Only beside an enemy man in each side's first `n` moves; any man
    /// after that.
    BesideEnemyFirst(u8),
    /// Any man, at any time (in the tests only: the first player wins at
    /// once).
    #[cfg_attr(not(test), allow(dead_code))]
    Free,
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
    pub centre: Centre,
    pub first: Side,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::beside_enemy(), Rules::best()]
    }

    /// As Elsdon Best recorded it from Tuta Nihoniho of Ngāti Porou (1925):
    /// only the opening move is restricted.
    pub fn best() -> Rules {
        Rules {
            id: "best",
            name: "Mū tōrere after Best",
            source: "Elsdon Best, Games and Pastimes of the Maori (1925), from Tuta Nihoniho of \
                     Ngāti Porou: a man may enter the empty pūtahi, but the opening move must be \
                     made by a man beside an enemy.",
            section: "Mū tōrere after Best",
            centre: Centre::BesideEnemyFirst(1),
            first: Side::White,
        }
    }

    /// As most accounts give it today: every move into the centre is
    /// restricted.
    pub fn beside_enemy() -> Rules {
        Rules {
            id: "beside-enemy",
            name: "The pūtahi beside an enemy",
            source: "As most accounts give it today: a man may enter the pūtahi only from beside \
                     an enemy man, at any time in the game.",
            section: "The pūtahi beside an enemy",
            centre: Centre::BesideEnemy,
            first: Side::White,
        }
    }

    /// Without any restriction: the first player wins at once (a test, not
    /// in the menu).
    #[cfg(test)]
    pub fn free() -> Rules {
        Rules { id: "free", name: "free", source: "", section: "", centre: Centre::Free, first: Side::White }
    }
}

/// A move: the man on `from` goes to the empty point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
}

/// The kewai beside kewai `k`.
pub fn beside(k: usize) -> [usize; 2] {
    [(k + 7) % 8, (k + 1) % 8]
}

/// Whether points `a` and `b` are joined.
pub fn joined(a: usize, b: usize) -> bool {
    a == CENTRE || b == CENTRE || beside(a).contains(&b)
}

/// The position: who stands where, the side to move, and how many moves
/// each side has made (counted only as far as the rules need).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub at: [Option<Side>; POINTS],
    pub to_move: Side,
    pub moves_made: [u8; 2],
}

impl Position {
    /// White on kewai 0 to 3 (the top and the right), Black on 4 to 7, the
    /// centre empty.
    pub fn new(rules: &Rules) -> Position {
        let mut at = [None; POINTS];
        for k in 0..8 {
            at[k] = Some(if k < 4 { Side::White } else { Side::Black });
        }
        Position { at, to_move: rules.first, moves_made: [0, 0] }
    }

    pub fn empty(&self) -> usize {
        (0..POINTS).find(|&p| self.at[p].is_none()).expect("one point is empty")
    }

    /// The legal moves for the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        let me = self.to_move;
        let to = self.empty();
        let mut moves = Vec::new();
        for from in 0..POINTS {
            if self.at[from] != Some(me) || !joined(from, to) {
                continue;
            }
            if to == CENTRE {
                let restricted = match rules.centre {
                    Centre::BesideEnemy => true,
                    Centre::BesideEnemyFirst(n) => self.moves_made[me.index()] < n,
                    Centre::Free => false,
                };
                if restricted && !beside(from).iter().any(|&k| self.at[k] == Some(me.other())) {
                    continue;
                }
            }
            moves.push(Move { from: from as u8, to: to as u8 });
        }
        moves
    }

    pub fn play(&mut self, rules: &Rules, mv: &Move) {
        let me = self.to_move;
        self.at[mv.to as usize] = self.at[mv.from as usize].take();
        // The count matters only for the first moves.
        if let Centre::BesideEnemyFirst(n) = rules.centre {
            let made = &mut self.moves_made[me.index()];
            *made = (*made + 1).min(n);
        }
        self.to_move = me.other();
    }

    /// The winner, if the side to move cannot move.
    pub fn winner(&self, rules: &Rules) -> Option<Side> {
        self.moves(rules).is_empty().then(|| self.to_move.other())
    }
}

/// The value of every position reachable from the start, for the side to
/// move there: won or lost in so many moves, or drawn (the game can only
/// go round in circles). Worked out from the ends back; there are a few
/// hundred positions.
pub mod solve {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Value {
        /// Won, the side to move ending it in this many moves of its own
        /// at most.
        Win(u32),
        /// Lost in this many moves at most, however well it plays.
        Loss(u32),
        Draw,
    }

    pub fn table(rules: &Rules) -> HashMap<Position, Value> {
        let start = Position::new(rules);
        let mut all = vec![start];
        let mut index: HashMap<Position, usize> = HashMap::from([(start, 0)]);
        let mut next: Vec<Vec<usize>> = Vec::new();
        let mut i = 0;
        while i < all.len() {
            let p = all[i];
            let mut out = Vec::new();
            for mv in p.moves(rules) {
                let mut q = p;
                q.play(rules, &mv);
                let n = all.len();
                let j = *index.entry(q).or_insert(n);
                if j == n {
                    all.push(q);
                }
                out.push(j);
            }
            next.push(out);
            i += 1;
        }
        let mut value: Vec<Option<Value>> = next.iter().map(|n| n.is_empty().then_some(Value::Loss(0))).collect();
        loop {
            let mut changed = false;
            for k in 0..all.len() {
                if value[k].is_some() {
                    continue;
                }
                let replies: Vec<Option<Value>> = next[k].iter().map(|&j| value[j]).collect();
                // A move to a lost position wins; the quickest is taken.
                let win = replies.iter().filter_map(|v| if let Some(Value::Loss(n)) = v { Some(*n) } else { None }).min();
                if let Some(n) = win {
                    value[k] = Some(Value::Win(n + 1));
                    changed = true;
                } else if replies.iter().all(|v| matches!(v, Some(Value::Win(_)))) {
                    let n = replies.iter().map(|v| if let Some(Value::Win(n)) = v { *n } else { 0 }).max().unwrap_or(0);
                    value[k] = Some(Value::Loss(n));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        all.into_iter().zip(value).map(|(p, v)| (p, v.unwrap_or(Value::Draw))).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(white: &[usize], black: &[usize], to_move: Side) -> Position {
        let mut at = [None; POINTS];
        for &p in white {
            at[p] = Some(Side::White);
        }
        for &p in black {
            at[p] = Some(Side::Black);
        }
        Position { at, to_move, moves_made: [5, 5] }
    }

    #[test]
    fn the_start() {
        let rules = Rules::beside_enemy();
        let p = Position::new(&rules);
        assert_eq!(p.empty(), CENTRE);
        // Only the two men beside an enemy may go in: kewai 0 and 3.
        let mut from: Vec<u8> = p.moves(&rules).iter().map(|m| m.from).collect();
        from.sort();
        assert_eq!(from, [0, 3]);
        // With the centre open to all, any of the four.
        assert_eq!(p.moves(&Rules::free()).len(), 4);
    }

    #[test]
    fn moves_along_the_ring_and_out_of_the_centre() {
        let rules = Rules::beside_enemy();
        // Kewai 2 is empty; White on 1 and 3 beside it, and in the centre.
        let p = set(&[1, 3, 8, 0], &[4, 5, 6, 7], Side::White);
        let mut from: Vec<u8> = p.moves(&rules).iter().map(|m| m.from).collect();
        from.sort();
        assert_eq!(from, [1, 3, 8]);
    }

    #[test]
    fn blocked_loses() {
        let rules = Rules::beside_enemy();
        // Kewai 2 empty, Black's men far from it, the centre White's.
        let p = set(&[1, 3, 8, 0], &[4, 5, 6, 7], Side::Black);
        assert!(p.moves(&rules).is_empty());
        assert_eq!(p.winner(&rules), Some(Side::White));
    }

    #[test]
    fn first_moves_rule_lapses() {
        let rules = Rules { centre: Centre::BesideEnemyFirst(2), ..Rules::beside_enemy() };
        let mut p = Position::new(&rules);
        assert_eq!(p.moves(&rules).len(), 2);
        p.moves_made = [2, 2];
        assert_eq!(p.moves(&rules).len(), 4);
    }

    #[test]
    fn solved() {
        use solve::Value;
        let best = solve::table(&Rules::best());
        println!("Best: {} positions, start {:?}", best.len(), best[&Position::new(&Rules::best())]);
        let beside = solve::table(&Rules::beside_enemy());
        let free = solve::table(&Rules::free());
        println!("beside an enemy: {} positions, start {:?}", beside.len(), beside[&Position::new(&Rules::beside_enemy())]);
        println!("open to all: {} positions, start {:?}", free.len(), free[&Position::new(&Rules::free())]);
        assert!(beside.len() < 2000 && free.len() < 2000);
        // With the centre open to all, the first player wins at once: a
        // man from kewai 1 or 2 into the centre leaves its place to be
        // filled by... nobody of the other side, so the other side is
        // blocked.
        assert!(matches!(free[&Position::new(&Rules::free())], Value::Win(1)));
    }
}
