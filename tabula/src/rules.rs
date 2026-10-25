//! The rules of the Roman tables games and their medieval kin, as data,
//! and one engine that plays them.
//!
//! Two players race fifteen men each round a board of 24 points (in
//! duodecim scripta, three rows of twelve) and off it, moving by the
//! throw of dice. A point held by two or more men of one side is closed
//! to the other; a single man (a blot) is hit by an enemy landing on it
//! and must enter again from the start. Where the sources differ or are
//! silent, the rule set chooses, and the rulebook says which and why.
//!
//! Each side's men run along its own track, steps 1 to `len` (0 is the
//! hand, not yet entered; `len + 1` is off the board). The board's points
//! are numbered 1 to `len` along White's track; Black's track runs the
//! same way or the other, as the rule set says.

/// The two sides. White throws first.
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

/// How the board is laid out on the screen and in the book.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Two rows of twelve points, as a backgammon board (tabula).
    Tables,
    /// Three rows of twelve (duodecim scripta).
    ThreeRows,
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
    pub layout: Layout,
    /// Points on each side's track.
    pub len: u8,
    pub men: u8,
    pub dice: u8,
    /// Whether Black's track runs the same way as White's (both entering
    /// at point 1), or the other way (Black entering at point `len`).
    pub same_way: bool,
    /// The last steps of the track, from which men may bear off once all
    /// of a side's men are there (or off).
    pub home: u8,
    /// Whether a man may bear off with a throw higher than it needs, when
    /// no man of its side stands further back in the home.
    pub over_throw: bool,
    /// Whether a hit man must enter again before any other man moves.
    pub enter_first: bool,
    /// Duodecim scripta: no man goes beyond this step until every man is
    /// entered.
    pub hold: Option<u8>,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::tabula(), Rules::tabula_parallel(), Rules::duodecim()]
    }

    pub fn tabula() -> Rules {
        Rules {
            id: "tabula",
            name: "Tabula, after Becq de Fouquieres and Austin",
            source: "The game of the emperor Zeno (c. AD 480), as Becq de Fouquieres (1869) and \
                     R. G. Austin (1934) worked it out from Agathias' epigram: fifteen men, three dice, \
                     the two colours running opposite ways as in backgammon.",
            section: "Tabula, the colours opposite",
            layout: Layout::Tables,
            len: 24,
            men: 15,
            dice: 3,
            same_way: false,
            home: 6,
            over_throw: true,
            enter_first: true,
            hold: None,
        }
    }

    pub fn tabula_parallel() -> Rules {
        Rules {
            id: "tabula-bell",
            name: "Tabula, after Bell",
            source: "R. C. Bell (1969): the same game with both colours entering on the same quarter \
                     and running the same way round the board.",
            section: "Tabula, the colours the same way",
            same_way: true,
            ..Rules::tabula()
        }
    }

    pub fn duodecim() -> Rules {
        Rules {
            id: "duodecim",
            name: "Ludus duodecim scriptorum, after Murray",
            source: "The older Roman game on three rows of twelve, as H. J. R. Murray reconstructed it \
                     (printed by Austin, 1934) from a lettered board at Ostia: enter on the middle row, \
                     then round the top and the bottom row.",
            section: "Ludus duodecim scriptorum",
            layout: Layout::ThreeRows,
            len: 36,
            same_way: true,
            hold: Some(12),
            ..Rules::tabula()
        }
    }

    /// The board point (1 to `len`) of `side`'s step `s` (1 to `len`).
    pub fn point(&self, side: Side, s: u8) -> u8 {
        if side == Side::White || self.same_way { s } else { self.len + 1 - s }
    }

    /// `side`'s step on board point `p`.
    pub fn step_of(&self, side: Side, p: u8) -> u8 {
        self.point(side, p)
    }

    pub fn off(&self) -> u8 {
        self.len + 1
    }
}

/// One man moved by one die: from a step (0, the hand) to a step
/// (`len + 1`, off).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Hop {
    pub from: u8,
    pub to: u8,
    pub die: u8,
    /// Whether it hits a blot.
    pub hits: bool,
}

/// A whole turn: the dice used, one hop each, in order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub hops: Vec<Hop>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win(Side),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Men of each side on each of its steps: index 0 the hand, 1 to
    /// `len` the board, `len + 1` off.
    pub men: [Vec<u8>; 2],
    /// Of the men in hand, how many were hit (and must enter first).
    pub hit: [u8; 2],
    pub to_move: Side,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let mut track = vec![0u8; rules.len as usize + 2];
        track[0] = rules.men;
        Position { men: [track.clone(), track], hit: [0, 0], to_move: Side::White }
    }

    /// How many men of `side` stand on board point `p`.
    pub fn on_point(&self, rules: &Rules, side: Side, p: u8) -> u8 {
        self.men[side.index()][rules.step_of(side, p) as usize]
    }

    pub fn borne_off(&self, rules: &Rules, side: Side) -> u8 {
        self.men[side.index()][rules.off() as usize]
    }

    /// Whether every man of `side` is home or off.
    fn all_home(&self, rules: &Rules, side: Side) -> bool {
        let first_home = rules.len - rules.home + 1;
        self.men[side.index()][..first_home as usize].iter().all(|&n| n == 0)
    }

    /// The hop of a man of the side to move from step `from` by `die`, if
    /// it is allowed.
    fn hop(&self, rules: &Rules, from: u8, die: u8) -> Option<Hop> {
        let me = self.to_move;
        let mine = &self.men[me.index()];
        if mine[from as usize] == 0 || from == rules.off() {
            return None;
        }
        if rules.enter_first && from != 0 && self.hit[me.index()] > 0 {
            return None;
        }
        let to = from + die;
        if let Some(hold) = rules.hold
            && mine[0] > 0
            && to > hold
        {
            return None;
        }
        if to > rules.len {
            if !self.all_home(rules, me) {
                return None;
            }
            if to > rules.off() {
                // A higher throw than needed: only from the man furthest
                // back.
                let behind = (rules.len - rules.home + 1..from).any(|s| mine[s as usize] > 0);
                if !rules.over_throw || behind {
                    return None;
                }
            }
            return Some(Hop { from, to: rules.off(), die, hits: false });
        }
        let theirs = self.on_point(rules, me.other(), rules.point(me, to));
        if theirs >= 2 {
            return None;
        }
        Some(Hop { from, to, die, hits: theirs == 1 })
    }

    /// One hop, the turn not passing.
    pub fn play_hop(&mut self, rules: &Rules, hop: &Hop) {
        let me = self.to_move;
        self.men[me.index()][hop.from as usize] -= 1;
        self.men[me.index()][hop.to as usize] += 1;
        if hop.from == 0 && self.hit[me.index()] > 0 {
            self.hit[me.index()] -= 1;
        }
        if hop.hits {
            let s = rules.step_of(me.other(), rules.point(me, hop.to));
            self.men[me.other().index()][s as usize] -= 1;
            self.men[me.other().index()][0] += 1;
            self.hit[me.other().index()] += 1;
        }
    }

    /// The legal turns for the side to move with `dice` thrown: as many
    /// dice used as can be, and of those the highest numbers (our choice,
    /// after Alfonso X's "they use the largest number they can"), each
    /// order of them, without repeats.
    pub fn moves(&self, rules: &Rules, dice: &[u8]) -> Vec<Move> {
        let mut out = Vec::new();
        let mut best = 0;
        self.extend(rules, dice, &mut Vec::new(), &mut out, &mut best);
        let mut out: Vec<Move> = out.into_iter().filter(|m| m.hops.len() == best).collect();
        let pips = |m: &Move| m.hops.iter().map(|h| h.die as u32).sum::<u32>();
        let most = out.iter().map(pips).max().unwrap_or(0);
        out.retain(|m| pips(m) == most);
        // The same turn reached in two orders is offered once, judged by
        // where the men end.
        let mut seen = std::collections::HashSet::new();
        out.retain(|m| {
            let mut p = self.clone();
            for h in &m.hops {
                p.play_hop(rules, h);
            }
            seen.insert(p.men)
        });
        out
    }

    fn extend(&self, rules: &Rules, dice: &[u8], done: &mut Vec<Hop>, out: &mut Vec<Move>, best: &mut usize) {
        let mut any = false;
        let mut tried = Vec::new();
        for (i, &d) in dice.iter().enumerate() {
            if tried.contains(&d) {
                continue;
            }
            tried.push(d);
            for from in 0..=rules.len {
                if let Some(hop) = self.hop(rules, from, d) {
                    any = true;
                    let mut next = self.clone();
                    next.play_hop(rules, &hop);
                    let mut rest = dice.to_vec();
                    rest.remove(i);
                    done.push(hop);
                    next.extend(rules, &rest, done, out, best);
                    done.pop();
                }
            }
        }
        if !any {
            *best = (*best).max(done.len());
            out.push(Move { hops: done.clone() });
        }
    }

    pub fn play(&mut self, rules: &Rules, mv: &Move) {
        for h in &mv.hops {
            self.play_hop(rules, h);
        }
        self.to_move = self.to_move.other();
    }

    pub fn outcome(&self, rules: &Rules) -> Option<Outcome> {
        Side::BOTH.into_iter().find(|&s| self.borne_off(rules, s) == rules.men).map(Outcome::Win)
    }

    /// The steps each side still has to go: the race.
    pub fn pips(&self, rules: &Rules, side: Side) -> u32 {
        self.men[side.index()].iter().enumerate().map(|(s, &n)| n as u32 * (rules.off() as u32 - s as u32)).sum()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn entering_and_hitting() {
        let r = Rules::tabula_parallel();
        let mut p = Position::new(&r);
        let moves = p.moves(&r, &[3, 4, 5]);
        assert!(moves.iter().all(|m| m.hops.len() == 3));
        // Enter one man on 3, 4, 5: one of the turns.
        let spread = moves.iter().find(|m| m.hops.iter().all(|h| h.from == 0)).unwrap().clone();
        p.play(&r, &spread);
        // Black, on the same track, enters on 4 and hits White's blot.
        let hit = p.moves(&r, &[4, 1, 1]).into_iter().find(|m| m.hops.iter().any(|h| h.hits)).unwrap();
        p.play(&r, &hit);
        assert_eq!(p.men[0][0], 13);
    }

    #[test]
    fn a_closed_point() {
        let r = Rules::tabula_parallel();
        let mut p = Position::new(&r);
        p.men[1][0] = 13;
        p.men[1][6] = 2;
        // White cannot enter on 6.
        assert!(p.moves(&r, &[6, 6, 6]).iter().all(|m| m.hops.is_empty()));
    }

    /// Zeno's position, after Becq and Austin (Agathias, Anth. Pal.
    /// IX.482): White to play 2, 5, 6.
    pub fn zeno() -> Position {
        let r = Rules::tabula();
        let mut p = Position::new(&r);
        p.men = [vec![0; 26], vec![0; 26]];
        for (pt, n) in [(6, 7), (9, 1), (10, 2), (19, 2), (20, 2), (23, 1)] {
            p.men[0][pt] = n;
        }
        for (pt, n) in [(8, 2), (11, 2), (12, 2), (13, 1), (14, 2), (15, 2), (18, 2), (21, 2)] {
            p.men[1][r.step_of(Side::Black, pt) as usize] = n;
        }
        p
    }

    #[test]
    fn zenos_throw() {
        let r = Rules::tabula();
        let p = zeno();
        let moves = p.moves(&r, &[2, 5, 6]);
        assert_eq!(moves.len(), 1);
        let mut hops: Vec<(u8, u8)> = moves[0].hops.iter().map(|h| (h.from, h.to)).collect();
        hops.sort();
        assert_eq!(hops, [(10, 16), (19, 24), (20, 22)]);
        let mut q = p.clone();
        q.play(&r, &moves[0]);
        let blots = (1..=24).filter(|&s| q.men[0][s] == 1).count();
        assert_eq!(blots, 8);
    }

    #[test]
    fn bearing_off() {
        let r = Rules::tabula();
        let mut p = Position::new(&r);
        p.men[0] = vec![0; 26];
        p.men[0][24] = 1;
        p.men[0][20] = 1;
        p.men[0][25] = 13;
        // 1 bears the man on 24 off; 6 the man on 20 (none behind it).
        let moves = p.moves(&r, &[1, 6, 2]);
        assert!(moves.iter().any(|m| m.hops.iter().filter(|h| h.to == 25).count() == 2));
        let mut q = p.clone();
        let mv = moves.iter().find(|m| m.hops.iter().filter(|h| h.to == 25).count() == 2).unwrap();
        q.play(&r, mv);
        assert_eq!(q.outcome(&r), Some(Outcome::Win(Side::White)));
    }
}
