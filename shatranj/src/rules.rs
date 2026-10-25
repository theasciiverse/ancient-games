//! The rules of shatranj, the chess of Persia and the caliphate, as data,
//! and the engine that plays them.
//!
//! Six kinds of men on a board of eight by eight, as in chess today, but
//! three of them weaker: the firzan (counsellor) steps one square
//! diagonally, the fil (elephant) leaps exactly two squares diagonally
//! over whatever stands between, and the baidaq (foot soldier) steps one
//! square forward, takes one diagonally forward, and on the far row
//! becomes a firzan. The shah (king), the faras (horse) and the rukh
//! (chariot) move as the king, the knight and the rook do now. There is
//! no castling and no double step. A game is won by mate, by stalemate in
//! the older rule, or by taking all the other side's men but his king
//! ("baring"). Where the sources differ or are silent, the rule set
//! chooses, and the rulebook says which and why.
//!
//! Squares are numbered 0 (a1) to 63 (h8), rank by rank from Red's
//! side.

/// The two sides. Red, as the books number the squares, moves first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Red,
    Black,
}

impl Side {
    pub fn index(self) -> usize {
        match self {
            Side::Red => 0,
            Side::Black => 1,
        }
    }

    pub fn other(self) -> Side {
        match self {
            Side::Red => Side::Black,
            Side::Black => Side::Red,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Red => "Red",
            Side::Black => "Black",
        }
    }

    pub const BOTH: [Side; 2] = [Side::Red, Side::Black];

    /// The rank a pawn of this side moves towards, one step at a time.
    pub fn forward(self) -> i8 {
        match self {
            Side::Red => 1,
            Side::Black => -1,
        }
    }
}

/// The kinds of men.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The king: one step any way.
    Shah,
    /// The counsellor: one step diagonally.
    Firzan,
    /// The elephant: a leap of exactly two squares diagonally.
    Fil,
    /// The horse: the knight's leap.
    Faras,
    /// The chariot: any distance along a rank or file.
    Rukh,
    /// The foot soldier.
    Baidaq,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Shah => "shah",
            Kind::Firzan => "firzan",
            Kind::Fil => "fil",
            Kind::Faras => "faras",
            Kind::Rukh => "rukh",
            Kind::Baidaq => "baidaq",
        }
    }

    /// The letter used in the move list and the book's diagrams: the
    /// English letters of today's chess for the men that move the same
    /// way, F and E for the firzan and the elephant.
    pub fn letter(self) -> char {
        match self {
            Kind::Shah => 'K',
            Kind::Firzan => 'F',
            Kind::Fil => 'E',
            Kind::Faras => 'N',
            Kind::Rukh => 'R',
            Kind::Baidaq => 'P',
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Man {
    pub side: Side,
    pub kind: Kind,
}

/// What a stalemate does: a side to move with no legal move, his shah
/// not attacked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stalemate {
    /// The stalemated side loses (shatranj, from the solutions of the
    /// Arabic problems).
    Loses,
    /// The stalemated side wins (al-'Adli's Indian rule).
    Wins,
}

/// How the elephant moves: India knew three moves for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Elephant {
    /// A leap of exactly two squares diagonally (the fil of shatranj).
    Diagonal,
    /// A leap of exactly two squares along a rank or file (al-'Adli's
    /// Indian elephant, which stands in the corner).
    Straight,
    /// One step diagonally or one straight forward: the trunk and the
    /// four feet (al-Biruni).
    Five,
}

/// What baring the king does: taking every man of the other side but his
/// shah.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bare {
    /// The bare side loses at once (the Medinese victory, and the Indian
    /// rule).
    Loses,
    /// The bare side loses, unless he can bare the other in his reply,
    /// which draws.
    LosesUnlessReplied,
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
    /// The back rank from a to h, Red's (Black's mirrors it).
    pub back: [Kind; 8],
    pub elephant: Elephant,
    pub stalemate: Stalemate,
    pub bare: Bare,
    /// Moves (each side's) with no capture and no pawn move after which
    /// the game is drawn; our choice where the sources say nothing.
    pub quiet: u16,
}

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![
            Rules::shatranj(),
            Rules::medinese(),
            Rules::chaturanga(Elephant::Diagonal),
            Rules::chaturanga(Elephant::Straight),
            Rules::chaturanga(Elephant::Five),
        ]
    }

    pub fn shatranj() -> Rules {
        use Kind::*;
        Rules {
            id: "shatranj",
            name: "Shatranj, after as-Suli",
            source: "The chess of Baghdad in the 9th and 10th centuries, as H. J. R. Murray (1913) \
                     gathers it from the Arabic masters: mate, a bare shah, or a stalemated shah loses. \
                     A shah bared may bare back at once, for a draw.",
            section: "Shatranj",
            back: [Rukh, Faras, Fil, Firzan, Shah, Fil, Faras, Rukh],
            elephant: Elephant::Diagonal,
            stalemate: Stalemate::Loses,
            bare: Bare::LosesUnlessReplied,
            quiet: 70,
        }
    }

    pub fn medinese() -> Rules {
        Rules {
            id: "medinese",
            name: "Shatranj, the Medinese victory",
            source: "Shatranj as played in Medina and the Hijaz, after al-'Adli (c. 840, in Murray 1913): \
                     whoever first bares the other's shah wins, even if he could bare back.",
            section: "Shatranj, the Medinese victory",
            bare: Bare::Loses,
            ..Rules::shatranj()
        }
    }

    pub fn chaturanga(elephant: Elephant) -> Rules {
        use Kind::*;
        let (id, name, source, back) = match elephant {
            Elephant::Diagonal => (
                "chaturanga",
                "Chaturanga, the Indian rules after al-'Adli",
                "The Indian rules al-'Adli recorded (c. 840, in Murray 1913): a stalemated shah wins, \
                 and the first to bare the other wins. The elephant leaps two diagonally, the move \
                 Murray thinks oldest.",
                [Rukh, Faras, Fil, Firzan, Shah, Fil, Faras, Rukh],
            ),
            Elephant::Straight => (
                "chaturanga-corner",
                "Chaturanga, al-'Adli's elephant in the corner",
                "Al-'Adli's Indian rules with his Indian elephant: in the corner, leaping two squares \
                 straight; the chariot (rukh) beside the counsellor's wing on c1 and f1.",
                [Fil, Faras, Rukh, Firzan, Shah, Rukh, Faras, Fil],
            ),
            Elephant::Five => (
                "chaturanga-biruni",
                "Chaturanga, al-Biruni's elephant",
                "Al-'Adli's Indian rules with the elephant al-Biruni saw in India (c. 1030): one step \
                 diagonally or straight forward, \"the trunk and the four feet\".",
                [Rukh, Faras, Fil, Firzan, Shah, Fil, Faras, Rukh],
            ),
        };
        Rules {
            id,
            name,
            source,
            section: match elephant {
                Elephant::Diagonal => "Chaturanga, the Indian rules",
                Elephant::Straight => "The elephant in the corner",
                Elephant::Five => "Al-Biruni's elephant",
            },
            back,
            elephant,
            stalemate: Stalemate::Wins,
            bare: Bare::Loses,
            quiet: 70,
        }
    }
}

/// A square's file (0 = a) and rank (0 = 1).
pub fn fr(sq: usize) -> (i8, i8) {
    ((sq % 8) as i8, (sq / 8) as i8)
}

pub fn square(file: i8, rank: i8) -> Option<usize> {
    ((0..8).contains(&file) && (0..8).contains(&rank)).then(|| rank as usize * 8 + file as usize)
}

pub fn name(sq: usize) -> String {
    let (f, r) = fr(sq);
    format!("{}{}", (b'a' + f as u8) as char, r + 1)
}

pub fn parse(name: &str) -> Option<usize> {
    let b = name.as_bytes();
    if b.len() != 2 {
        return None;
    }
    square(b[0] as i8 - b'a' as i8, b[1] as i8 - b'1' as i8)
}

const KING: [(i8, i8); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
const FIRZAN: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const FIL: [(i8, i8); 4] = [(2, 2), (2, -2), (-2, 2), (-2, -2)];
const STRAIGHT: [(i8, i8); 4] = [(2, 0), (-2, 0), (0, 2), (0, -2)];

const FIVE_UP: [(i8, i8); 5] = [(1, 1), (-1, 1), (1, -1), (-1, -1), (0, 1)];
const FIVE_DOWN: [(i8, i8); 5] = [(1, 1), (-1, 1), (1, -1), (-1, -1), (0, -1)];

/// The elephant's moves for `side`.
fn elephant(e: Elephant, side: Side) -> &'static [(i8, i8)] {
    match (e, side) {
        (Elephant::Diagonal, _) => &FIL,
        (Elephant::Straight, _) => &STRAIGHT,
        (Elephant::Five, Side::Red) => &FIVE_UP,
        (Elephant::Five, Side::Black) => &FIVE_DOWN,
    }
}

const FARAS: [(i8, i8); 8] = [(1, 2), (2, 1), (-1, 2), (-2, 1), (1, -2), (2, -1), (-1, -2), (-2, -1)];
const RUKH: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    /// A baidaq reaching the far rank becomes a firzan.
    pub promotes: bool,
}

/// How a game ended, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Win(Side, Why),
    Draw(Why),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Mate,
    Stalemate,
    /// The loser has nothing but his shah.
    Bare,
    /// Both bare: the second baring answered the first.
    BothBare,
    Quiet,
    Repetition,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub board: [Option<Man>; 64],
    pub to_move: Side,
    /// Moves since a capture or a baidaq move.
    pub quiet: u16,
    /// Whether the last move took the last man but the shah of the side
    /// now to move, who may then answer in kind.
    pub just_bared: bool,
    /// How elephants move in this game.
    pub elephant: Elephant,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let mut board = [None; 64];
        for f in 0..8 {
            board[f] = Some(Man { side: Side::Red, kind: rules.back[f] });
            board[8 + f] = Some(Man { side: Side::Red, kind: Kind::Baidaq });
            board[48 + f] = Some(Man { side: Side::Black, kind: Kind::Baidaq });
            board[56 + f] = Some(Man { side: Side::Black, kind: rules.back[f] });
        }
        Position { board, to_move: Side::Red, quiet: 0, just_bared: false, elephant: rules.elephant }
    }

    pub fn empty() -> Position {
        Position { board: [None; 64], to_move: Side::Red, quiet: 0, just_bared: false, elephant: Elephant::Diagonal }
    }

    pub fn shah(&self, side: Side) -> Option<usize> {
        (0..64).find(|&s| self.board[s] == Some(Man { side, kind: Kind::Shah }))
    }

    /// Men other than the shah.
    pub fn others(&self, side: Side) -> usize {
        self.board.iter().flatten().filter(|m| m.side == side && m.kind != Kind::Shah).count()
    }

    /// Every move of the man on `from`, whether or not it leaves the shah
    /// attacked.
    fn pseudo(&self, from: usize, out: &mut Vec<Move>) {
        let Some(man) = self.board[from] else {
            return;
        };
        let (f, r) = fr(from);
        let add = |to: usize, out: &mut Vec<Move>| {
            if self.board[to].is_none_or(|m| m.side != man.side) {
                out.push(Move { from: from as u8, to: to as u8, promotes: false });
            }
        };
        let leaps: &[(i8, i8)] = match man.kind {
            Kind::Shah => &KING,
            Kind::Firzan => &FIRZAN,
            Kind::Fil => elephant(self.elephant, man.side),
            Kind::Faras => &FARAS,
            Kind::Rukh | Kind::Baidaq => &[],
        };
        for &(df, dr) in leaps {
            if let Some(to) = square(f + df, r + dr) {
                add(to, out);
            }
        }
        match man.kind {
            Kind::Rukh => {
                for (df, dr) in RUKH {
                    let (mut nf, mut nr) = (f + df, r + dr);
                    while let Some(to) = square(nf, nr) {
                        add(to, out);
                        if self.board[to].is_some() {
                            break;
                        }
                        nf += df;
                        nr += dr;
                    }
                }
            }
            Kind::Baidaq => {
                let dr = man.side.forward();
                let last = if man.side == Side::Red { 7 } else { 0 };
                if let Some(to) = square(f, r + dr)
                    && self.board[to].is_none()
                {
                    out.push(Move { from: from as u8, to: to as u8, promotes: r + dr == last });
                }
                for df in [-1, 1] {
                    if let Some(to) = square(f + df, r + dr)
                        && self.board[to].is_some_and(|m| m.side != man.side)
                    {
                        out.push(Move { from: from as u8, to: to as u8, promotes: r + dr == last });
                    }
                }
            }
            _ => {}
        }
    }

    /// Whether a man of `by` attacks `sq`.
    pub fn attacked(&self, sq: usize, by: Side) -> bool {
        let (f, r) = fr(sq);
        let is = |s: Option<usize>, kinds: &[Kind]| {
            s.is_some_and(|s| self.board[s].is_some_and(|m| m.side == by && kinds.contains(&m.kind)))
        };
        for (df, dr) in KING {
            if is(square(f + df, r + dr), &[Kind::Shah]) {
                return true;
            }
        }
        for (df, dr) in FIRZAN {
            if is(square(f + df, r + dr), &[Kind::Firzan]) {
                return true;
            }
        }
        for &(df, dr) in elephant(self.elephant, by) {
            if is(square(f - df, r - dr), &[Kind::Fil]) {
                return true;
            }
        }
        for (df, dr) in FARAS {
            if is(square(f + df, r + dr), &[Kind::Faras]) {
                return true;
            }
        }
        // A baidaq of `by` attacks diagonally forward, so from one rank
        // behind `sq` as it moves.
        for df in [-1, 1] {
            if is(square(f + df, r - by.forward()), &[Kind::Baidaq]) {
                return true;
            }
        }
        for (df, dr) in RUKH {
            let (mut nf, mut nr) = (f + df, r + dr);
            while let Some(s) = square(nf, nr) {
                if let Some(m) = self.board[s] {
                    if m.side == by && m.kind == Kind::Rukh {
                        return true;
                    }
                    break;
                }
                nf += df;
                nr += dr;
            }
        }
        false
    }

    pub fn in_check(&self, side: Side) -> bool {
        self.shah(side).is_some_and(|k| self.attacked(k, side.other()))
    }

    /// The legal moves for the side to move.
    pub fn moves(&self) -> Vec<Move> {
        let me = self.to_move;
        let mut pseudo = Vec::with_capacity(48);
        for s in 0..64 {
            if self.board[s].is_some_and(|m| m.side == me) {
                self.pseudo(s, &mut pseudo);
            }
        }
        pseudo
            .into_iter()
            .filter(|mv| {
                let mut next = self.clone();
                next.apply(mv);
                !next.in_check(me)
            })
            .collect()
    }

    fn apply(&mut self, mv: &Move) {
        let mut man = self.board[mv.from as usize].take().expect("a man to move");
        if mv.promotes {
            man.kind = Kind::Firzan;
        }
        self.board[mv.to as usize] = Some(man);
    }

    pub fn captures(&self, mv: &Move) -> Option<Man> {
        self.board[mv.to as usize]
    }

    pub fn play(&mut self, rules: &Rules, mv: &Move) {
        let me = self.to_move;
        let pawn = self.board[mv.from as usize].is_some_and(|m| m.kind == Kind::Baidaq);
        let capture = self.board[mv.to as usize].is_some();
        self.apply(mv);
        self.quiet = if pawn || capture { 0 } else { self.quiet + 1 };
        self.just_bared = capture && self.others(me.other()) == 0;
        let _ = rules;
        self.to_move = me.other();
    }

    /// How the game stands after the last move, `moves` being the side to
    /// move's legal moves.
    pub fn outcome(&self, rules: &Rules, moves: &[Move]) -> Option<Outcome> {
        let me = self.to_move;
        if moves.is_empty() {
            return Some(if self.in_check(me) {
                Outcome::Win(me.other(), Why::Mate)
            } else {
                match rules.stalemate {
                    Stalemate::Loses => Outcome::Win(me.other(), Why::Stalemate),
                    Stalemate::Wins => Outcome::Win(me, Why::Stalemate),
                }
            });
        }
        let (mine, theirs) = (self.others(me), self.others(me.other()));
        if mine == 0 && theirs == 0 {
            return Some(Outcome::Draw(Why::BothBare));
        }
        match rules.bare {
            // The side not to move is bare: he was bared and had his
            // chance to answer, and did not.
            _ if theirs == 0 => return Some(Outcome::Win(me, Why::Bare)),
            Bare::Loses if mine == 0 => return Some(Outcome::Win(me.other(), Why::Bare)),
            Bare::LosesUnlessReplied if mine == 0 => {
                // Bared by the last move: he may bare back now, if he can
                // take the other's last man. If not, he has lost.
                let can_reply =
                    self.just_bared && theirs == 1 && moves.iter().any(|m| self.board[m.to as usize].is_some());
                if !can_reply {
                    return Some(Outcome::Win(me.other(), Why::Bare));
                }
            }
            _ => {}
        }
        if self.quiet >= rules.quiet * 2 {
            return Some(Outcome::Draw(Why::Quiet));
        }
        None
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn place(p: &mut Position, side: Side, men: &[(&str, Kind)]) {
        for &(sq, kind) in men {
            p.board[parse(sq).unwrap()] = Some(Man { side, kind });
        }
    }

    #[test]
    fn the_start() {
        let r = Rules::shatranj();
        let p = Position::new(&r);
        assert_eq!(p.board[parse("e1").unwrap()].unwrap().kind, Kind::Shah);
        assert_eq!(p.board[parse("e8").unwrap()].unwrap().kind, Kind::Shah);
        assert_eq!(p.board[parse("d8").unwrap()].unwrap().kind, Kind::Firzan);
        // Eight baidaqs a single step each, two fils, four faras moves.
        assert_eq!(p.moves().len(), 8 + 4 + 4);
    }

    #[test]
    fn the_fil_leaps_two() {
        let mut p = Position::empty();
        place(&mut p, Side::Red, &[("c1", Kind::Fil), ("d2", Kind::Baidaq), ("a1", Kind::Shah)]);
        place(&mut p, Side::Black, &[("h8", Kind::Shah)]);
        let to: Vec<String> = p.moves().iter().filter(|m| name(m.from as usize) == "c1").map(|m| name(m.to as usize)).collect();
        assert_eq!(to.len(), 2);
        assert!(to.contains(&"e3".to_string()) && to.contains(&"a3".to_string()));
    }

    #[test]
    fn a_baidaq_becomes_a_firzan() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        place(&mut p, Side::Red, &[("b7", Kind::Baidaq), ("a1", Kind::Shah)]);
        place(&mut p, Side::Black, &[("h8", Kind::Shah), ("h2", Kind::Rukh)]);
        let mv = p.moves().into_iter().find(|m| name(m.from as usize) == "b7").unwrap();
        assert!(mv.promotes);
        p.play(&r, &mv);
        assert_eq!(p.board[parse("b8").unwrap()].unwrap().kind, Kind::Firzan);
    }

    #[test]
    fn stalemate_loses() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        // Black's shah on a1; Red's shah on c2 and firzan on b3 guard
        // b1, b2 and a2. Black has a baidaq too, blocked, so he is not bare.
        place(&mut p, Side::Red, &[("c2", Kind::Shah), ("b3", Kind::Firzan)]);
        place(&mut p, Side::Black, &[("a1", Kind::Shah), ("h5", Kind::Baidaq)]);
        place(&mut p, Side::Red, &[("h4", Kind::Baidaq)]);
        p.to_move = Side::Black;
        let moves = p.moves();
        assert!(moves.is_empty());
        assert!(!p.in_check(Side::Black));
        assert_eq!(p.outcome(&r, &moves), Some(Outcome::Win(Side::Red, Why::Stalemate)));
    }

    #[test]
    fn baring() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        // Red takes Black's last man; Black cannot take back: bare.
        place(&mut p, Side::Red, &[("a1", Kind::Shah), ("d1", Kind::Rukh)]);
        place(&mut p, Side::Black, &[("h8", Kind::Shah), ("d5", Kind::Faras)]);
        let mv = Move { from: parse("d1").unwrap() as u8, to: parse("d5").unwrap() as u8, promotes: false };
        p.play(&r, &mv);
        let moves = p.moves();
        assert_eq!(p.outcome(&r, &moves), Some(Outcome::Win(Side::Red, Why::Bare)));
        // But if Black can bare Red in reply, he may: then both are bare,
        // a draw.
        let mut q = Position::empty();
        place(&mut q, Side::Red, &[("a1", Kind::Shah), ("d1", Kind::Rukh)]);
        place(&mut q, Side::Black, &[("e6", Kind::Shah), ("d5", Kind::Faras)]);
        q.play(&r, &mv);
        let moves = q.moves();
        assert_eq!(q.outcome(&r, &moves), None);
        let reply = moves.into_iter().find(|m| name(m.to as usize) == "d5").unwrap();
        q.play(&r, &reply);
        let moves = q.moves();
        assert_eq!(q.outcome(&r, &moves), Some(Outcome::Draw(Why::BothBare)));
    }

    #[test]
    fn games_end_at_random() {
        let r = Rules::shatranj();
        let mut rng = ancient_core::rng::Rng::new(5);
        for _ in 0..10 {
            let mut p = Position::new(&r);
            loop {
                let moves = p.moves();
                if p.outcome(&r, &moves).is_some() {
                    break;
                }
                let mv = moves[(rng.next() % moves.len() as u64) as usize];
                p.play(&r, &mv);
            }
        }
    }
}
