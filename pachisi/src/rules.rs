//! The rules of pachisi and chaupar, the cross-shaped race games of India,
//! as data, and one engine that plays them.
//!
//! The board is a cross of four arms, each three columns by eight rows,
//! round a large square in the middle. Four colours, one to an arm, each
//! with four men; partners sit opposite. A man leaves the middle down the
//! middle row of its own arm, goes round the outside of the whole cross
//! against the sun, and comes back up the same middle row into the
//! middle: 84 steps. Men move by throws of cowrie shells (pachisi) or of
//! long dice (chaupar). Where the sources differ or are silent, the rule
//! set chooses, and the rulebook says which and why.
//!
//! A man's place is a step on its own track: 0 in the middle, not yet
//! entered; 1 to 7 its own middle row, outward; 8 to 76 the loop round
//! the cross (76 is 8 again); 77 to 83 its own middle row, inward; 84 home.
//! `READY` marks a man entered but still in the middle (Falkener's
//! grace).

/// Steps on a man's way: 84 is home.
pub const HOME: u8 = 84;
/// A man entered but still standing in the middle (Falkener).
pub const READY: u8 = 85;
/// Squares in the loop round the cross.
pub const LOOP: u8 = 68;

/// The four colours, in the order they play, against the sun round the
/// board from the near arm; partners sit opposite (Falkener: "the yellow
/// and green should play against the red and black").
pub const COLOURS: [&str; 4] = ["Red", "Yellow", "Black", "Green"];

/// The two partnerships: colours 0 and 2, and 1 and 3.
pub fn team(colour: usize) -> usize {
    colour % 2
}

pub const TEAMS: [&str; 2] = ["Red and Black", "Yellow and Green"];

/// How a turn's throw is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Throws {
    /// Six cowries, counted by the mouths up: none 6, one 10, two 2, three
    /// 3, four 4, five 25, six 12 (Tylor, Herklots, Shway Yoe). 6, 10, 12
    /// and 25 throw again; 10 and 25 bring a starting 1 as well.
    TylorCowries,
    /// Six cowries: none up 25, one 10, two 2, three 3, four 4, five 5,
    /// six 6 (Falkener). 25, 10 and 6 throw again; 25 and 10 bring a grace
    /// of 1.
    FalkenerCowries,
    /// Three long dice, each showing 1, 2, 5 or 6 (Abu'l-Fazl).
    LongDice,
}

/// Where the castles (safe squares) stand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Castles {
    /// The outer end of each middle row, and the 4th square from the
    /// middle on each outer row (Tylor's figure).
    Tylor,
    /// The outer end of each middle row, and the 4th square from the outer
    /// end on each outer row (Falkener, Bell).
    Falkener,
    /// No castles (chaupar, Falkener's Chausar).
    None,
}

/// What two or more men of one colour on a square do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doubled {
    /// They hold the square: no enemy may land there (Tylor, the Ain).
    Hold,
    /// They are at their own peril: one enemy man takes them all
    /// (Falkener).
    Peril,
}

/// How men enter from the middle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// Only by the starting 1 of a 10 or a 25, onto the first square
    /// (Tylor).
    Pau,
    /// The grace of a 10 or a 25 lets a man out into the middle, from
    /// where any throw moves it; one man a colour starts out (Falkener).
    Grace,
    /// The men start on the board; a man taken comes back on a die of 1,
    /// onto the first square (our choice for chaupar).
    Ace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub section: &'static str,
    pub throws: Throws,
    pub castles: Castles,
    pub doubled: Doubled,
    pub entry: Entry,
    /// Whether taking a man brings another throw.
    pub capture_throws: bool,
    /// Tylor's "rotten" throws: three high throws running are struck out
    /// unless the next is a 10 or a 25.
    pub rotten: bool,
    /// Whether a colour with all its men home throws for its partner.
    pub partner_throws: bool,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![Rules::tylor(), Rules::falkener(), Rules::chaupar()]
    }

    pub fn tylor() -> Rules {
        Rules {
            id: "tylor",
            name: "Pachisi, after Tylor (1879)",
            source: "The rules Rajendralala Mitra of Calcutta gave E. B. Tylor (1879): six cowries, men \\
                     entering only with the starting 1 of a 10 or a 25, two men holding a square, forts \\
                     that block the enemy.",
            section: "Pachisi, after Tylor",
            throws: Throws::TylorCowries,
            castles: Castles::Tylor,
            doubled: Doubled::Hold,
            entry: Entry::Pau,
            capture_throws: true,
            rotten: true,
            partner_throws: true,
        }
    }

    pub fn falkener() -> Rules {
        Rules {
            id: "falkener",
            name: "Pachisi, after Falkener (1892) and Bell",
            source: "Edward Falkener's rules (1892), which R. C. Bell (1969) follows: twenty-five on all \\
                     six cowries mouth down, graces, castles four from the outer end, and doubled men at \\
                     their own peril.",
            section: "Pachisi, after Falkener",
            throws: Throws::FalkenerCowries,
            castles: Castles::Falkener,
            doubled: Doubled::Peril,
            entry: Entry::Grace,
            capture_throws: true,
            rotten: false,
            partner_throws: false,
        }
    }

    pub fn chaupar() -> Rules {
        Rules {
            id: "chaupar",
            name: "Chaupar, after Abu'l-Fazl (c. 1590)",
            source: "Akbar's game as the Ain-i Akbari describes it: three long dice of 1, 2, 5 and 6, the \\
                     men starting on the board, two men together safe, and a player who is home \\
                     throwing for his partner.",
            section: "Chaupar, after Abu'l-Fazl",
            throws: Throws::LongDice,
            castles: Castles::None,
            doubled: Doubled::Hold,
            entry: Entry::Ace,
            capture_throws: false,
            rotten: false,
            partner_throws: true,
        }
    }

    /// The steps of a colour's own track that are castles.
    pub fn castle_steps(&self) -> &'static [u8] {
        match self.castles {
            Castles::Tylor => &[8, 13, 20, 25, 30, 37, 42, 47, 54, 59, 64, 71, 76],
            Castles::Falkener => &[8, 12, 21, 25, 29, 38, 42, 46, 55, 59, 63, 72, 76],
            Castles::None => &[],
        }
    }

    /// Whether loop square `sq` is a castle (the same for every colour).
    pub fn castle(&self, sq: u8) -> bool {
        self.castle_steps().iter().any(|&s| loop_square(0, s) == Some(sq))
    }

    /// The value of a throw of cowries with `up` mouths up, whether it
    /// throws again, and whether it brings a starting 1 (pau, grace).
    pub fn cowries(&self, up: u8) -> (u8, bool, bool) {
        match self.throws {
            Throws::TylorCowries => {
                let v = [6, 10, 2, 3, 4, 25, 12][up as usize];
                (v, matches!(v, 6 | 10 | 12 | 25), matches!(v, 10 | 25))
            }
            _ => {
                let v = [25, 10, 2, 3, 4, 5, 6][up as usize];
                (v, matches!(v, 6 | 10 | 25), matches!(v, 10 | 25))
            }
        }
    }
}

/// The loop square (0 to 67) of `colour`'s step `t`, if `t` is on the
/// loop. Colour 0's step 9 is loop square 0; each colour's track starts
/// 17 squares further round.
pub fn loop_square(colour: usize, t: u8) -> Option<u8> {
    (8..=76).contains(&t).then(|| ((t as u32 + 59 + 17 * colour as u32) % LOOP as u32) as u8)
}

/// Where a step of `colour`'s track is drawn: (row, column) on the 19 by
/// 19 grid of the cross, row 0 at the top; the middle is rows and columns
/// 8 to 10. Red's arm is the lower one.
pub fn cell(colour: usize, t: u8) -> (u8, u8) {
    let (r, c): (i32, i32) = match t {
        0 | HOME | READY => (9, 9),
        1..=8 => (10 + t as i32, 9),
        9..=16 => (18 - (t as i32 - 9), 10),
        17..=24 => (10, 11 + (t as i32 - 17)),
        25 => (9, 18),
        26..=33 => (8, 18 - (t as i32 - 26)),
        34..=41 => (7 - (t as i32 - 34), 10),
        42 => (0, 9),
        43..=50 => (t as i32 - 43, 8),
        51..=58 => (8, 7 - (t as i32 - 51)),
        59 => (9, 0),
        60..=67 => (10, t as i32 - 60),
        68..=75 => (11 + (t as i32 - 68), 8),
        76 => (18, 9),
        _ => (18 - (t as i32 - 76), 9),
    };
    // Turn the near arm round to the colour's own, against the sun.
    let (mut r, mut c) = (r - 9, c - 9);
    for _ in 0..colour {
        (r, c) = (-c, r);
    }
    ((r + 9) as u8, (c + 9) as u8)
}

/// One use of a throw: men of `colour` moved `by` steps each, all from
/// `from` (a pair or three together in chaupar), using `tokens`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub colour: u8,
    /// Which of the colour's men (0 to 3).
    pub men: Vec<u8>,
    pub from: u8,
    pub to: u8,
    /// The throws to hand it uses.
    pub tokens: Vec<Token>,
    /// Men taken: (colour, man).
    pub takes: Vec<(u8, u8)>,
}

/// A throw to hand: its value, and whether it is a starting 1 (pau or
/// grace), which enters men.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub value: u8,
    pub enters: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    /// Each colour's men, as steps on its track.
    pub men: [[u8; 4]; 4],
    /// The colour to throw and move.
    pub to_move: usize,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let start = match rules.entry {
            Entry::Pau => [0; 4],
            Entry::Grace => [READY, 0, 0, 0],
            Entry::Ace => [6, 7, 15, 16],
        };
        Position { men: [start; 4], to_move: 0 }
    }

    /// The colour whose men `colour` moves: its own, or, where the rules
    /// allow, its partner's once its own are all home.
    pub fn moving(&self, rules: &Rules, colour: usize) -> usize {
        let done = self.men[colour].iter().all(|&s| s == HOME);
        if done && rules.partner_throws { (colour + 2) % 4 } else { colour }
    }

    pub fn team_home(&self, t: usize) -> bool {
        [t, t + 2].iter().all(|&c| self.men[c].iter().all(|&s| s == HOME))
    }

    pub fn winner(&self) -> Option<usize> {
        (0..2).find(|&t| self.team_home(t))
    }

    /// The men of each colour on loop square `sq`.
    pub fn on_loop(&self, sq: u8) -> Vec<(u8, u8)> {
        let mut out = Vec::new();
        for c in 0..4 {
            for (m, &s) in self.men[c].iter().enumerate() {
                if loop_square(c, s) == Some(sq) {
                    out.push((c as u8, m as u8));
                }
            }
        }
        out
    }

    /// Where `n` men of `colour` from step `from` would land moving `by`,
    /// and the men they would take; `None` if they may not.
    fn landing(&self, rules: &Rules, colour: usize, from: u8, by: u8, n: usize) -> Option<(u8, Vec<(u8, u8)>)> {
        let base = if from == READY { 0 } else { from };
        let to = base + by;
        if to > HOME {
            return None;
        }
        let Some(sq) = loop_square(colour, to) else {
            return Some((to, Vec::new()));
        };
        let here = self.on_loop(sq);
        let enemies: Vec<(u8, u8)> = here.iter().copied().filter(|&(c, _)| team(c as usize) != team(colour)).collect();
        if enemies.is_empty() {
            return Some((to, Vec::new()));
        }
        if rules.castle(sq) {
            return None;
        }
        let most = (0..4).map(|c| enemies.iter().filter(|&&(k, _)| k == c).count()).max().unwrap_or(0);
        if most >= 2 {
            return match rules.doubled {
                Doubled::Hold if n >= most => Some((to, enemies)),
                Doubled::Hold => None,
                Doubled::Peril => Some((to, enemies)),
            };
        }
        Some((to, enemies))
    }

    /// Every legal use of one or more of `tokens` by the colour to move.
    pub fn moves(&self, rules: &Rules, tokens: &[Token]) -> Vec<Move> {
        let mover = self.to_move;
        let colour = self.moving(rules, mover);
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for tok in tokens {
            if !seen.insert(*tok) {
                continue;
            }
            let mut done = std::collections::HashSet::new();
            for m in 0..4u8 {
                let from = self.men[colour][m as usize];
                if from == HOME || !done.insert(from) {
                    continue;
                }
                let mv = |to: u8, takes: Vec<(u8, u8)>| Move {
                    colour: colour as u8,
                    men: vec![m],
                    from,
                    to,
                    tokens: vec![*tok],
                    takes,
                };
                if from == 0 {
                    match rules.entry {
                        Entry::Pau if tok.enters => {
                            if let Some((to, takes)) = self.landing(rules, colour, 0, 1, 1) {
                                out.push(mv(to, takes));
                            }
                        }
                        Entry::Grace if tok.enters => out.push(mv(READY, Vec::new())),
                        Entry::Ace if tok.value == 1 => {
                            if let Some((to, takes)) = self.landing(rules, colour, 0, 1, 1) {
                                out.push(mv(to, takes));
                            }
                        }
                        _ => {}
                    }
                    continue;
                }
                if let Some((to, takes)) = self.landing(rules, colour, from, tok.value, 1) {
                    out.push(mv(to, takes));
                }
            }
        }
        // Chaupar: a doublet moves two men standing together twice its
        // value each, a triplet three (Abu'l-Fazl).
        if rules.throws == Throws::LongDice {
            for v in [1u8, 2, 5, 6] {
                let idx: Vec<Token> = tokens.iter().copied().filter(|t| t.value == v).collect();
                for k in 2..=idx.len().min(3) {
                    let mut done = std::collections::HashSet::new();
                    for m in 0..4u8 {
                        let from = self.men[colour][m as usize];
                        if from == HOME || from == 0 || !done.insert(from) {
                            continue;
                        }
                        let together: Vec<u8> = (0..4u8).filter(|&q| self.men[colour][q as usize] == from).collect();
                        if together.len() < k {
                            continue;
                        }
                        if let Some((to, takes)) = self.landing(rules, colour, from, 2 * v, k) {
                            out.push(Move {
                                colour: colour as u8,
                                men: together[..k].to_vec(),
                                from,
                                to,
                                tokens: idx[..k].to_vec(),
                                takes,
                            });
                        }
                    }
                }
            }
        }
        out
    }

    /// Plays a move: the men go, the men taken go back to the middle.
    pub fn apply(&mut self, mv: &Move) {
        for &m in &mv.men {
            self.men[mv.colour as usize][m as usize] = mv.to;
        }
        for &(c, m) in &mv.takes {
            self.men[c as usize][m as usize] = 0;
        }
    }

    /// The turn passes to the next colour.
    pub fn pass(&mut self) {
        self.to_move = (self.to_move + 1) % 4;
    }

    /// Steps still to go for a colour's men, the race.
    pub fn to_go(&self, colour: usize) -> u32 {
        self.men[colour]
            .iter()
            .map(|&s| match s {
                READY => HOME as u32,
                0 => HOME as u32 + 6,
                s => (HOME - s) as u32,
            })
            .sum()
    }
}

/// Takes the tokens a move used out of `toks`.
pub fn remove_tokens(toks: &mut Vec<Token>, used: &[Token]) {
    for u in used {
        if let Some(i) = toks.iter().position(|t| t == u) {
            toks.remove(i);
        }
    }
}

/// The tokens a list of throws gives: each throw's value, and a starting 1
/// for each that brings one.
pub fn tokens(rules: &Rules, throws: &[Throw]) -> Vec<Token> {
    let mut out = Vec::new();
    for t in throws {
        out.push(Token { value: t.value, enters: false });
        if t.pau {
            out.push(Token { value: 1, enters: true });
        }
    }
    if rules.throws == Throws::LongDice {
        for t in &mut out {
            t.enters = t.value == 1;
        }
    }
    out
}

/// One throw of the cowries or the dice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Throw {
    pub value: u8,
    /// Throw again.
    pub again: bool,
    /// Brings a starting 1.
    pub pau: bool,
    /// Cowries mouth up, or the die's face.
    pub shows: u8,
}

/// Throws once: six cowries (each mouth up one time in two), or three
/// long dice (each face one time in four), as `r` gives random bits.
pub fn throw(rules: &Rules, mut bits: impl FnMut() -> u64) -> Vec<Throw> {
    match rules.throws {
        Throws::LongDice => (0..3)
            .map(|_| {
                let v = [1, 2, 5, 6][(bits() % 4) as usize];
                Throw { value: v, again: false, pau: false, shows: v }
            })
            .collect(),
        _ => {
            let up = (0..6).filter(|_| bits() % 2 == 1).count() as u8;
            let (value, again, pau) = rules.cowries(up);
            vec![Throw { value, again, pau, shows: up }]
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn the_track() {
        // 84 steps, the loop shared: Red's 25 is Green's 8.
        assert_eq!(loop_square(0, 25), loop_square(1, 8));
        assert_eq!(loop_square(0, 8), loop_square(0, 76));
        assert_eq!(loop_square(0, 7), None);
        // Every loop square drawn in one place whichever colour counts it.
        for c in 0..4 {
            for t in 8..=76 {
                let sq = loop_square(c, t).unwrap();
                let t0 = (8..=76).find(|&u| loop_square(0, u) == Some(sq)).unwrap();
                assert_eq!(cell(c, t), cell(0, t0), "colour {c} step {t}");
            }
        }
        // The cross has 96 squares: 68 on the loop, 7 private each way per
        // arm... 68 + 4 * 7 = 96.
        let mut cells = std::collections::HashSet::new();
        for c in 0..4 {
            for t in 1..HOME {
                cells.insert(cell(c, t));
            }
        }
        assert_eq!(cells.len(), 96);
    }

    #[test]
    fn falkeners_castles_are_25_apart() {
        let r = Rules::falkener();
        for (a, b) in [(0, 25), (21, 46), (59, 84)] {
            assert_eq!(b - a, 25);
            assert!(a == 0 || r.castle_steps().contains(&a));
        }
        // Twelve castles round the cross.
        let n = (0..LOOP).filter(|&s| r.castle(s)).count();
        assert_eq!(n, 12);
        assert_eq!((0..LOOP).filter(|&s| Rules::tylor().castle(s)).count(), 12);
    }

    #[test]
    fn entering_by_the_starting_one() {
        let r = Rules::tylor();
        let p = Position::new(&r);
        let ten = tokens(&r, &[Throw { value: 10, again: true, pau: true, shows: 1 }]);
        let moves = p.moves(&r, &ten);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].to, 1);
        let two = tokens(&r, &[Throw { value: 2, again: false, pau: false, shows: 2 }]);
        assert!(p.moves(&r, &two).is_empty());
    }

    #[test]
    fn taking_and_holding() {
        let r = Rules::tylor();
        let mut p = Position::new(&r);
        // Red on 20 (a fort), Green single on Red's 23, Green two on 30.
        p.men[0] = [17, 0, 0, 0];
        let green = |t: u8| (8..=76).find(|&u| loop_square(1, u) == loop_square(0, t)).unwrap();
        p.men[1] = [green(23), green(28), green(28), 0];
        let tok = |v| vec![Token { value: v, enters: false }];
        // 17 + 6 = 23: takes the single man.
        let m = p.moves(&r, &tok(6));
        assert_eq!(m[0].takes.len(), 1);
        // 17 + 11 = 28: two Green men hold it.
        assert!(p.moves(&r, &tok(11)).is_empty());
        // Falkener: one man takes the pair.
        let f = Rules::falkener();
        assert_eq!(p.moves(&f, &tok(11))[0].takes.len(), 2);
    }

    #[test]
    fn chaupar_doublets() {
        let r = Rules::chaupar();
        let mut p = Position::new(&r);
        p.men[0] = [20, 20, 40, 50];
        let toks: Vec<Token> = [6, 6, 2].iter().map(|&v| Token { value: v, enters: false }).collect();
        let pair = p.moves(&r, &toks).into_iter().find(|m| m.men.len() == 2).unwrap();
        assert_eq!(pair.to, 32);
        assert_eq!(pair.tokens.len(), 2);
    }

    #[test]
    fn exact_home() {
        let r = Rules::tylor();
        let mut p = Position::new(&r);
        p.men[0] = [80, HOME, HOME, HOME];
        let tok = |v| vec![Token { value: v, enters: false }];
        assert!(p.moves(&r, &tok(6)).is_empty());
        assert_eq!(p.moves(&r, &tok(4))[0].to, HOME);
    }
}

#[cfg(test)]
mod stuck {
    use super::*;

    #[test]
    fn home_from_83_with_the_starting_one() {
        let r = Rules::tylor();
        let mut p = Position::new(&r);
        p.men = [[83, 83, 84, 83], [83, 84, 83, 83], [83, 83, 84, 84], [83, 83, 84, 84]];
        let toks = tokens(&r, &[Throw { value: 10, again: true, pau: true, shows: 1 }, Throw { value: 3, again: false, pau: false, shows: 3 }]);
        let moves = p.moves(&r, &toks);
        assert!(moves.iter().any(|m| m.to == HOME), "{moves:?}");
        let mut rng = ancient_core::rng::Rng::new(1);
        let plan = crate::ai::plan(&r, &p, &toks, ancient_core::player::Strength::Normal, &mut rng);
        assert!(!plan.is_empty());
    }
}
