//! Patolli, the race and betting game of the Aztecs, as R. C. Bell
//! suggested it might be played (1969), on the equipment Diego Durán
//! describes: six stones a side, five black beans with one white hole
//! each, and a cross of compartments painted on a mat.
//!
//! The track is the outline of the cross, 60 compartments in a ring: a
//! square of the middle, out along one row of an arm to its rounded end,
//! back along the other row, the next square of the middle, and so on.
//! Each player enters on the square of the middle nearest him and goes
//! once round, either way (his first throw chooses), and off beside
//! where he came in. The pinched (hourglass) compartments cost two
//! forfeits; the rounded ends give another throw. Nobody is ever taken:
//! a compartment holds one stone. Where Bell is silent, the rulebook
//! says what this game does ("Our choice").

/// Compartments round the ring.
pub const RING: u8 = 60;
/// Stones a side.
pub const STONES: usize = 6;
/// A stone not yet entered.
pub const HAND: u8 = 255;
/// A stone borne off.
pub const OFF: u8 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Red,
    Blue,
}

impl Side {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn other(self) -> Side {
        match self {
            Side::Red => Side::Blue,
            Side::Blue => Side::Red,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Red => "Red",
            Side::Blue => "Blue",
        }
    }

    pub const BOTH: [Side; 2] = [Side::Red, Side::Blue];

    /// The ring compartment where the side enters: the square of the
    /// middle nearest it. The two players sit opposite.
    pub fn entry(self) -> u8 {
        match self {
            Side::Red => 0,
            Side::Blue => 30,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub section: &'static str,
    /// What each side puts in the pool at the start, and keeps in its
    /// purse: our choice.
    pub stake: u32,
    pub purse: u32,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::bell()]
    }

    pub fn bell() -> Rules {
        Rules {
            id: "bell",
            name: "Patolli, after Bell's suggested rules",
            source: "Durán's mat, stones and beans (c. 1579), with the rules R. C. Bell (1969) suggested to \\
                     make a game of them: once round the cross, either way, forfeits on the pinched squares \\
                     and another throw on the rounded ends. A reconstruction.",
            section: "Bell's suggested rules",
            stake: 10,
            purse: 20,
        }
    }
}

/// What a compartment of the ring is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// One of the four squares of the middle.
    Middle,
    Plain,
    /// Pinched by the black triangles: landing there pays two forfeits to
    /// the other player.
    Pinched,
    /// The rounded end of an arm: landing there gives another throw.
    End,
}

pub fn kind(sq: u8) -> Kind {
    match sq % 15 {
        0 => Kind::Middle,
        5 | 6 | 9 | 10 => Kind::Pinched,
        7 | 8 => Kind::End,
        _ => Kind::Plain,
    }
}

/// Where ring compartment `sq` is drawn: (row, column) on a 16 by 16 grid
/// of the cross drawn upright, the middle at rows and columns 7 and 8.
/// Red's square of the middle is the lower left one; Red's arm the lower.
pub fn cell(sq: u8) -> (u8, u8) {
    let limb = sq / 15;
    let k = sq % 15;
    // The lower arm, then turned against the sun for the others.
    let (r, c): (i32, i32) = match k {
        0 => (8, 7),
        1..=7 => (8 + k as i32, 7),
        _ => (15 - (k as i32 - 8), 8),
    };
    // Turn about the middle (7.5, 7.5) a quarter at a time.
    let (mut y, mut x) = (2 * r - 15, 2 * c - 15);
    for _ in 0..limb {
        (y, x) = (-x, y);
    }
    (((y + 15) / 2) as u8, ((x + 15) / 2) as u8)
}

/// The value of a throw with `holes` white holes up: 1 to 4 as shown, all
/// five 10 (Durán); none up moves nothing (our choice).
pub fn value(holes: u8) -> u8 {
    match holes {
        5 => 10,
        h => h,
    }
}

/// A move: a stone entered or moved, by the whole throw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub stone: u8,
    /// Its place on its way before and after (HAND, 0 to 59, OFF).
    pub from: u8,
    pub to: u8,
    /// The way chosen, for a side's very first stone.
    pub dir: i8,
}

/// How a game ended: a side has all six stones off first, and takes the
/// pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Home(Side),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Each stone's place on its owner's way: HAND, 0 to 59, or OFF.
    pub stones: [[u8; STONES]; 2],
    /// The way each side goes round, once chosen: +1 or -1.
    pub dir: [i8; 2],
    pub to_move: Side,
    pub purse: [u32; 2],
    pub pool: u32,
}

impl Position {
    pub fn new(rules: &Rules, first: Side) -> Position {
        Position {
            stones: [[HAND; STONES]; 2],
            dir: [0, 0],
            to_move: first,
            purse: [rules.purse; 2],
            pool: 2 * rules.stake,
        }
    }

    /// The ring compartment of `side`'s way at `p`.
    pub fn square(&self, side: Side, p: u8, dir: i8) -> u8 {
        ((side.entry() as i32 + dir as i32 * p as i32).rem_euclid(RING as i32)) as u8
    }

    /// Whether ring compartment `sq` holds a stone.
    pub fn taken(&self, sq: u8) -> bool {
        Side::BOTH.iter().any(|&s| {
            let d = self.dir[s.index()];
            self.stones[s.index()].iter().any(|&p| p < OFF && self.square(s, p, d) == sq)
        })
    }

    /// The legal moves for the side to move with a throw of `n`.
    pub fn moves(&self, n: u8) -> Vec<Move> {
        let me = self.to_move;
        let i = me.index();
        let mut out = Vec::new();
        if n == 0 {
            return out;
        }
        let dir = self.dir[i];
        let first_in_hand = self.stones[i].iter().position(|&p| p == HAND);
        if dir == 0 {
            // The opening throw: a stone enters and goes the whole throw,
            // either way.
            if let Some(s) = first_in_hand {
                for d in [1i8, -1] {
                    if !self.taken(self.square(me, n, d)) {
                        out.push(Move { stone: s as u8, from: HAND, to: n, dir: d });
                    }
                }
            }
            return out;
        }
        // A later stone enters on a throw of 1, onto the entry square.
        if n == 1
            && let Some(s) = first_in_hand
            && !self.taken(me.entry())
        {
            out.push(Move { stone: s as u8, from: HAND, to: 0, dir });
        }
        let mut seen = Vec::new();
        for (s, &p) in self.stones[i].iter().enumerate() {
            if p == HAND || p == OFF || seen.contains(&p) {
                continue;
            }
            seen.push(p);
            let to = p + n;
            if to == OFF || (to < OFF && !self.taken(self.square(me, to, dir))) {
                out.push(Move { stone: s as u8, from: p, to, dir });
            }
        }
        out
    }

    /// Pays `n` forfeits from `side`'s purse to the pool (`to: None`) or
    /// to the other side; a side that cannot pay in full pays what it has
    /// (our choice).
    fn pay(&mut self, side: Side, n: u32, to: Option<Side>) {
        let paid = n.min(self.purse[side.index()]);
        self.purse[side.index()] -= paid;
        match to {
            Some(o) => self.purse[o.index()] += paid,
            None => self.pool += paid,
        }
    }

    /// Plays a move. Returns whether the side throws again (a rounded
    /// end).
    pub fn play(&mut self, mv: &Move) -> bool {
        let me = self.to_move;
        let i = me.index();
        if self.dir[i] == 0 {
            self.dir[i] = mv.dir;
        }
        self.stones[i][mv.stone as usize] = mv.to;
        if mv.to == OFF {
            // The other pays one forfeit for every stone borne off.
            self.pay(me.other(), 1, Some(me));
            if self.stones[i].iter().all(|&p| p == OFF) {
                // The winner takes the pool.
                self.purse[i] += self.pool;
                self.pool = 0;
            }
            return false;
        }
        match kind(self.square(me, mv.to, self.dir[i])) {
            Kind::Pinched => {
                self.pay(me, 2, Some(me.other()));
                false
            }
            Kind::End => true,
            _ => false,
        }
    }

    /// No move with the throw: a forfeit into the pool (a throw of
    /// nothing costs nothing; our choice).
    pub fn stuck(&mut self, n: u8) {
        if n > 0 {
            self.pay(self.to_move, 1, None);
        }
    }

    pub fn pass(&mut self) {
        self.to_move = self.to_move.other();
    }

    pub fn off(&self, side: Side) -> usize {
        self.stones[side.index()].iter().filter(|&&p| p == OFF).count()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        for s in Side::BOTH {
            if self.off(s) == STONES {
                return Some(Outcome::Home(s));
            }
        }
        None
    }

    /// Steps still to go.
    pub fn to_go(&self, side: Side) -> u32 {
        self.stones[side.index()]
            .iter()
            .map(|&p| match p {
                HAND => RING as u32 + 5,
                OFF => 0,
                p => (OFF - p) as u32,
            })
            .sum()
    }
}

/// The chance of each count, in 32nds: five beans, each hole up one time
/// in two.
pub const ODDS: [(u8, u32); 6] = [(0, 1), (1, 5), (2, 10), (3, 10), (4, 5), (10, 1)];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring() {
        // 60 compartments, each drawn in its own cell, neighbours next to
        // each other.
        let mut cells = std::collections::HashSet::new();
        for sq in 0..RING {
            assert!(cells.insert(cell(sq)), "{sq}");
            let (a, b) = (cell(sq), cell((sq + 1) % RING));
            let d = (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs();
            assert_eq!(d, 1, "{sq} {a:?} {b:?}");
        }
        assert_eq!((0..RING).filter(|&s| kind(s) == Kind::Pinched).count(), 16);
        assert_eq!((0..RING).filter(|&s| kind(s) == Kind::End).count(), 8);
    }

    #[test]
    fn opening_entering_and_bearing_off() {
        let r = Rules::bell();
        let mut p = Position::new(&r, Side::Red);
        // The opening throw: two ways.
        let m = p.moves(3);
        assert_eq!(m.len(), 2);
        p.play(&m[0]);
        assert_eq!(p.dir[0], 1);
        // Later stones enter only on a 1.
        let two: Vec<Move> = p.moves(2);
        assert!(two.iter().all(|m| m.from != HAND));
        assert!(p.moves(1).iter().any(|m| m.from == HAND));
        // Exact throw off: from 56, a 4.
        p.stones[0][0] = 56;
        assert!(p.moves(4).iter().any(|m| m.to == OFF));
        assert!(p.moves(10).iter().all(|m| m.stone != 0));
    }

    #[test]
    fn no_landing_on_a_stone() {
        let r = Rules::bell();
        let mut p = Position::new(&r, Side::Red);
        p.dir = [1, 1];
        p.stones[0][0] = 3;
        p.stones[1][0] = 33;
        // Blue's way runs from ring 30: its 33rd compartment is ring 3.
        let sq = p.square(Side::Blue, 33, 1);
        assert_eq!(sq, 3);
        // Two stones cannot share: Red's stone at 3 is at ring 3 too.
        p.stones[0][1] = 1;
        assert!(p.moves(2).iter().all(|m| !(m.stone == 1 && m.to == 3)));
    }

    #[test]
    fn forfeits() {
        let r = Rules::bell();
        let mut p = Position::new(&r, Side::Red);
        p.dir = [1, 1];
        p.stones[0][0] = 3;
        // 3 + 2 = 5: pinched, Red pays Blue two.
        let mv = p.moves(2).into_iter().find(|m| m.stone == 0).unwrap();
        assert!(!p.play(&mv));
        assert_eq!(p.purse, [18, 22]);
        // 5 + 2 = 7: a rounded end, another throw.
        let mv = p.moves(2).into_iter().find(|m| m.stone == 0).unwrap();
        assert!(p.play(&mv));
    }
}
