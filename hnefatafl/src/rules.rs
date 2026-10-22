//! The rules of Hnefatafl and its kin, for each rule set the game offers.
//!
//! Tafl games are known from sagas, finds of boards and pieces, and one
//! eyewitness: Linnaeus, who watched the Saami play Tablut in 1732 and wrote
//! the rules in his diary. Today's rule sets are reconstructions; each here
//! follows its source (the World Tafl Federation and Aage Nielsen's pages
//! for Copenhagen, Fetlar, Brandubh and Tablut, R. C. Bell for his Tablut).
//!
//! Squares are (row, column) from the top left. The attackers move first.

/// The two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// The besiegers, dark: Muscovites in Linnaeus, Vikings in Brandubh.
    Attackers,
    /// The king's side, light: Swedes in Linnaeus, the Irish princes.
    Defenders,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Attackers => Side::Defenders,
            Side::Defenders => Side::Attackers,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Attackers => "the attackers",
            Side::Defenders => "the defenders",
        }
    }
}

/// What stands on a square.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    Attacker,
    Defender,
    King,
}

impl Piece {
    pub fn side(self) -> Side {
        match self {
            Piece::Attacker => Side::Attackers,
            Piece::Defender | Piece::King => Side::Defenders,
        }
    }
}

pub type Square = (usize, usize);

/// Where the king escapes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    Corners,
    Edge,
}

/// How the king is captured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KingCapture {
    /// Surrounded on all four sides by attackers; next to the throne, on the
    /// other three. Never on the edge (`edge_safe`).
    FourSides { edge_safe: bool },
    /// Like any other piece, between two attackers or an attacker and a
    /// hostile square; on the throne four attackers, next to it three
    /// (`three_by_throne`; Brandubh only asks four on the throne).
    TwoSides { three_by_throne: bool },
}

/// What a third repetition of a position means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repetition {
    /// A draw.
    Draw,
    /// The defenders lose ("any perpetual repetition results in a loss for
    /// White").
    DefendersLose,
    /// The side that made the move repeating it loses (Brandubh).
    MoverLoses,
}

/// A rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    pub size: usize,
    pub attackers: &'static [Square],
    pub defenders: &'static [Square],
    pub escape: Escape,
    pub king_capture: KingCapture,
    /// Captures of a whole row along the edge (Copenhagen).
    pub shield_wall: bool,
    /// The king wins by an unbreakable fort on the edge (Copenhagen).
    pub exit_fort: bool,
    /// The attackers win by enclosing the king and all his defenders.
    pub encirclement: bool,
    pub repetition: Repetition,
    /// The heading of this rule set in the rulebook chapter (RULES.txt),
    /// where the help opens.
    pub section: &'static str,
}

const COPENHAGEN_ATTACKERS: [Square; 24] = [
    (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), (1, 5),
    (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (5, 1),
    (10, 3), (10, 4), (10, 5), (10, 6), (10, 7), (9, 5),
    (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (5, 9),
];

/// The king first, then the defenders.
const COPENHAGEN_DEFENDERS: [Square; 13] = [
    (5, 5),
    (5, 6), (6, 5), (5, 4), (4, 5), (4, 6), (6, 4), (6, 6), (4, 4),
    (5, 7), (7, 5), (5, 3), (3, 5),
];

const TABLUT_ATTACKERS: [Square; 16] = [
    (0, 3), (0, 4), (0, 5), (1, 4),
    (8, 3), (8, 4), (8, 5), (7, 4),
    (3, 0), (4, 0), (5, 0), (4, 1),
    (3, 8), (4, 8), (5, 8), (4, 7),
];

const TABLUT_DEFENDERS: [Square; 9] = [
    (4, 4),
    (2, 4), (3, 4), (5, 4), (6, 4),
    (4, 2), (4, 3), (4, 5), (4, 6),
];

const BRANDUBH_ATTACKERS: [Square; 8] = [
    (0, 3), (1, 3), (5, 3), (6, 3), (3, 0), (3, 1), (3, 5), (3, 6),
];

const BRANDUBH_DEFENDERS: [Square; 5] = [(3, 3), (2, 3), (4, 3), (3, 2), (3, 4)];

impl Rules {
    pub fn copenhagen() -> Rules {
        Rules {
            id: "copenhagen",
            name: "Copenhagen 11x11",
            source: "The Copenhagen rules of the World Tafl Federation, as Aage \
                     Nielsen gives them: the tournament game, with the shield \
                     wall and the exit fort.",
            size: 11,
            attackers: &COPENHAGEN_ATTACKERS,
            defenders: &COPENHAGEN_DEFENDERS,
            escape: Escape::Corners,
            king_capture: KingCapture::FourSides { edge_safe: true },
            shield_wall: true,
            exit_fort: true,
            encirclement: true,
            repetition: Repetition::DefendersLose,
            section: "Copenhagen 11x11",
        }
    }

    pub fn fetlar() -> Rules {
        Rules {
            id: "fetlar",
            name: "Fetlar 11x11",
            source: "The Fetlar rules (the Fetlar Hnefatafl Panel, 2007) as Aage \
                     Nielsen gives them: the 11x11 game without the shield wall or the fort.",
            size: 11,
            attackers: &COPENHAGEN_ATTACKERS,
            defenders: &COPENHAGEN_DEFENDERS,
            escape: Escape::Corners,
            king_capture: KingCapture::FourSides { edge_safe: true },
            shield_wall: false,
            exit_fort: false,
            encirclement: true,
            repetition: Repetition::Draw,
            section: "Fetlar 11x11",
        }
    }

    pub fn brandubh() -> Rules {
        Rules {
            id: "brandubh",
            name: "Brandubh 7x7",
            source: "Brandubh, the Irish game, in the World Tafl Federation's \
                     official rules: a king and four princes against eight \
                     Viking invaders on seven by seven.",
            size: 7,
            attackers: &BRANDUBH_ATTACKERS,
            defenders: &BRANDUBH_DEFENDERS,
            escape: Escape::Corners,
            king_capture: KingCapture::TwoSides { three_by_throne: false },
            shield_wall: false,
            exit_fort: false,
            encirclement: true,
            repetition: Repetition::MoverLoses,
            section: "Brandubh 7x7",
        }
    }

    pub fn tablut() -> Rules {
        Rules {
            id: "tablut",
            name: "Tablut 9x9 (Linnaeus)",
            source: "Tablut as Linnaeus saw the Saami play it in 1732, in the \
                     reading of his diary by Cartier and Loof, confirmed by Olli \
                     Salmi's translation: the king is captured like any piece and \
                     escapes to the edge.",
            size: 9,
            attackers: &TABLUT_ATTACKERS,
            defenders: &TABLUT_DEFENDERS,
            escape: Escape::Edge,
            king_capture: KingCapture::TwoSides { three_by_throne: true },
            shield_wall: false,
            exit_fort: false,
            encirclement: true,
            repetition: Repetition::DefendersLose,
            section: "Tablut 9x9 (Linnaeus)",
        }
    }

    pub fn bell_tablut() -> Rules {
        Rules {
            id: "bell-tablut",
            name: "Tablut 9x9 (Bell 1960)",
            source: "R. C. Bell's Tablut (1960), from the English translation of \
                     1811, which read Linnaeus' \"likewise the king\" as \"except \
                     the king\": the king is captured only by four.",
            size: 9,
            attackers: &TABLUT_ATTACKERS,
            defenders: &TABLUT_DEFENDERS,
            escape: Escape::Edge,
            king_capture: KingCapture::FourSides { edge_safe: false },
            shield_wall: false,
            exit_fort: false,
            encirclement: false,
            repetition: Repetition::Draw,
            section: "Tablut 9x9 (Bell 1960)",
        }
    }

    pub fn all() -> Vec<Rules> {
        vec![
            Rules::copenhagen(),
            Rules::fetlar(),
            Rules::brandubh(),
            Rules::tablut(),
            Rules::bell_tablut(),
        ]
    }

    pub fn throne(&self) -> Square {
        (self.size / 2, self.size / 2)
    }

    pub fn is_corner(&self, s: Square) -> bool {
        let e = self.size - 1;
        (s.0 == 0 || s.0 == e) && (s.1 == 0 || s.1 == e)
    }

    /// Squares only the king may stop on: the throne, and the corners where
    /// the king escapes to them.
    pub fn restricted(&self, s: Square) -> bool {
        s == self.throne() || (self.escape == Escape::Corners && self.is_corner(s))
    }

    pub fn on_edge(&self, s: Square) -> bool {
        let e = self.size - 1;
        s.0 == 0 || s.1 == 0 || s.0 == e || s.1 == e
    }
}

/// The four directions along rows and columns.
pub const DIRECTIONS: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// A move: from one square to another along a row or column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

/// What a move did.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Outcome {
    pub captured: Vec<Square>,
    pub winner: Option<Side>,
}

/// The position.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub size: usize,
    pub board: Vec<Option<Piece>>,
    pub to_move: Side,
}

impl Position {
    pub fn new(rules: &Rules) -> Position {
        let n = rules.size;
        let mut board = vec![None; n * n];
        for &s in rules.attackers {
            board[s.0 * n + s.1] = Some(Piece::Attacker);
        }
        for (i, &s) in rules.defenders.iter().enumerate() {
            board[s.0 * n + s.1] = Some(if i == 0 { Piece::King } else { Piece::Defender });
        }
        Position { size: n, board, to_move: Side::Attackers }
    }

    pub fn at(&self, s: Square) -> Option<Piece> {
        self.board[s.0 * self.size + s.1]
    }

    fn set(&mut self, s: Square, p: Option<Piece>) {
        self.board[s.0 * self.size + s.1] = p;
    }

    pub fn step(&self, s: Square, d: (isize, isize)) -> Option<Square> {
        let r = s.0 as isize + d.0;
        let c = s.1 as isize + d.1;
        (r >= 0 && c >= 0 && (r as usize) < self.size && (c as usize) < self.size)
            .then_some((r as usize, c as usize))
    }

    pub fn king(&self) -> Option<Square> {
        (0..self.board.len())
            .find(|&i| self.board[i] == Some(Piece::King))
            .map(|i| (i / self.size, i % self.size))
    }

    /// The squares of `side`'s pieces.
    pub fn pieces(&self, side: Side) -> Vec<Square> {
        (0..self.board.len())
            .filter(|&i| self.board[i].is_some_and(|p| p.side() == side))
            .map(|i| (i / self.size, i % self.size))
            .collect()
    }

    /// The legal moves of the piece on `from`.
    pub fn moves_from(&self, rules: &Rules, from: Square) -> Vec<Move> {
        let Some(piece) = self.at(from) else { return Vec::new() };
        let mut moves = Vec::new();
        for d in DIRECTIONS {
            let mut s = from;
            while let Some(next) = self.step(s, d) {
                if self.at(next).is_some() {
                    break;
                }
                s = next;
                // Others may pass the empty throne but not stop there.
                if piece != Piece::King && rules.restricted(s) {
                    continue;
                }
                moves.push(Move { from, to: s });
            }
        }
        moves
    }

    /// All legal moves of the side to move.
    pub fn moves(&self, rules: &Rules) -> Vec<Move> {
        self.pieces(self.to_move)
            .into_iter()
            .flat_map(|s| self.moves_from(rules, s))
            .collect()
    }

    /// Whether square `s` helps capture a piece of `victim`'s side: an enemy
    /// piece (the king counts for the defenders), or a hostile square.
    fn hostile_to(&self, rules: &Rules, s: Square, victim: Piece) -> bool {
        match self.at(s) {
            Some(p) => p.side() != victim.side(),
            None => {
                if rules.escape == Escape::Corners && rules.is_corner(s) {
                    // Brandubh: hostile to all, the king too; elsewhere the
                    // king is not taken in this way.
                    return victim != Piece::King || rules.id == "brandubh";
                }
                if s == rules.throne() {
                    return match victim {
                        Piece::Attacker => true,
                        Piece::Defender => true, // empty here
                        Piece::King => false,
                    };
                }
                false
            }
        }
    }

    /// Plays `mv` for the side to move: captures, the winner if any.
    pub fn play(&mut self, rules: &Rules, mv: Move) -> Outcome {
        let mover = self.to_move;
        let piece = self.at(mv.from).expect("no piece there");
        self.set(mv.from, None);
        self.set(mv.to, Some(piece));
        let mut out = Outcome::default();

        // Custodian captures around the square moved to.
        for d in DIRECTIONS {
            let Some(next) = self.step(mv.to, d) else { continue };
            let Some(victim) = self.at(next) else { continue };
            if victim.side() == mover {
                continue;
            }
            if victim == Piece::King {
                if self.king_captured(rules, next) {
                    self.set(next, None);
                    out.captured.push(next);
                    out.winner = Some(Side::Attackers);
                }
                continue;
            }
            let Some(beyond) = self.step(next, d) else { continue };
            // The king takes part in captures; the throne counts as hostile
            // to defenders only when empty, which it is if `beyond` is empty.
            if self.hostile_to(rules, beyond, victim) {
                self.set(next, None);
                out.captured.push(next);
            }
        }
        if rules.shield_wall {
            out.captured.extend(self.shield_wall(rules, mv.to));
        }

        self.to_move = mover.other();
        if out.winner.is_none() {
            out.winner = self.decided(rules, piece, mv);
        }
        out
    }

    /// Whether the king on `k` is captured, after an attacker's move.
    fn king_captured(&self, rules: &Rules, k: Square) -> bool {
        let throne = rules.throne();
        let attacker = |s: Option<Square>| s.is_some_and(|s| self.at(s) == Some(Piece::Attacker));
        let sides: Vec<Option<Square>> = DIRECTIONS.iter().map(|&d| self.step(k, d)).collect();
        let next_to_throne = sides.contains(&Some(throne));
        // Around him: attackers, and the throne counting for one beside it.
        let surrounded = sides
            .iter()
            .all(|&s| attacker(s) || (s == Some(throne) && self.at(throne).is_none()));
        match rules.king_capture {
            KingCapture::FourSides { edge_safe } => {
                if rules.on_edge(k) {
                    if !edge_safe {
                        return false;
                    }
                    // Fetlar: the last defender, completely surrounded.
                    return rules.id == "fetlar"
                        && self.pieces(Side::Defenders).len() == 1
                        && sides.iter().flatten().all(|&s| self.at(s) == Some(Piece::Attacker));
                }
                surrounded
            }
            KingCapture::TwoSides { three_by_throne } => {
                if k == throne || (three_by_throne && next_to_throne) {
                    return surrounded;
                }
                // Like any piece: between two attackers, or an attacker and a
                // hostile square, along a row or a column.
                let pinch = |s: Option<Square>| {
                    s.is_some_and(|s| match self.at(s) {
                        Some(p) => p == Piece::Attacker,
                        None => self.hostile_to(rules, s, Piece::King),
                    })
                };
                // `sides` follows DIRECTIONS: up, down, left, right.
                (pinch(sides[0]) && pinch(sides[1])) || (pinch(sides[2]) && pinch(sides[3]))
            }
        }
    }

    /// Copenhagen's shield wall: the moved piece brackets a row of enemies
    /// along the edge, each with one of the mover's pieces in front of it,
    /// with the mover's piece or a corner at the far end. The king is not
    /// taken this way.
    fn shield_wall(&mut self, rules: &Rules, to: Square) -> Vec<Square> {
        let mover = self.to_move;
        if !rules.on_edge(to) {
            return Vec::new();
        }
        let n = self.size;
        let mut taken = Vec::new();
        // The edge lines through `to`, and which way is "in front".
        let mut lines: Vec<((isize, isize), (isize, isize))> = Vec::new();
        if to.0 == 0 || to.0 == n - 1 {
            let inward = if to.0 == 0 { (1, 0) } else { (-1, 0) };
            lines.push(((0, 1), inward));
            lines.push(((0, -1), inward));
        }
        if to.1 == 0 || to.1 == n - 1 {
            let inward = if to.1 == 0 { (0, 1) } else { (0, -1) };
            lines.push(((1, 0), inward));
            lines.push(((-1, 0), inward));
        }
        for (along, inward) in lines {
            let mut row = Vec::new();
            let mut s = to;
            let mut closed = false;
            while let Some(next) = self.step(s, along) {
                match self.at(next) {
                    Some(p) if p.side() != mover => {
                        let front = self.step(next, inward).and_then(|f| self.at(f));
                        if !front.is_some_and(|f| f.side() == mover) {
                            break;
                        }
                        row.push((next, p));
                        s = next;
                    }
                    Some(_) => {
                        closed = true;
                        break;
                    }
                    None => {
                        closed = rules.is_corner(next);
                        break;
                    }
                }
            }
            if closed && row.len() >= 2 {
                for (sq, p) in row {
                    if p != Piece::King {
                        self.set(sq, None);
                        taken.push(sq);
                    }
                }
            }
        }
        taken
    }

    /// The winner after a move by the piece now on `mv.to`, if decided:
    /// escape, a fort, encirclement, or the side to move without moves.
    fn decided(&self, rules: &Rules, piece: Piece, mv: Move) -> Option<Side> {
        let Some(king) = self.king() else { return Some(Side::Attackers) };
        if piece == Piece::King {
            let escaped = match rules.escape {
                Escape::Corners => rules.is_corner(mv.to),
                Escape::Edge => rules.on_edge(mv.to),
            };
            if escaped {
                return Some(Side::Defenders);
            }
        }
        if rules.exit_fort && self.exit_fort(rules, king) {
            return Some(Side::Defenders);
        }
        if rules.encirclement && self.encircled() {
            return Some(Side::Attackers);
        }
        if !self.can_move(rules) {
            return Some(self.to_move.other());
        }
        None
    }

    /// Whether the side to move has any move at all.
    pub fn can_move(&self, rules: &Rules) -> bool {
        self.pieces(self.to_move).into_iter().any(|s| {
            let king = self.at(s) == Some(Piece::King);
            DIRECTIONS.iter().any(|&d| {
                self.step(s, d).is_some_and(|t| {
                    self.at(t).is_none()
                        && (king
                            || !rules.restricted(t)
                            // Past the empty throne.
                            || self.step(t, d).is_some_and(|u| self.at(u).is_none() && !rules.restricted(u)))
                })
            })
        })
    }

    /// Whether every defender is shut in by attackers: no way from any of
    /// them to the edge through empty squares or other defenders.
    pub fn encircled(&self) -> bool {
        let n = self.size;
        let mut seen = vec![false; n * n];
        let mut stack: Vec<Square> = self.pieces(Side::Defenders);
        for &s in &stack {
            seen[s.0 * n + s.1] = true;
        }
        while let Some(s) = stack.pop() {
            if s.0 == 0 || s.1 == 0 || s.0 == n - 1 || s.1 == n - 1 {
                return false;
            }
            for d in DIRECTIONS {
                if let Some(t) = self.step(s, d)
                    && !seen[t.0 * n + t.1]
                    && self.at(t) != Some(Piece::Attacker)
                {
                    seen[t.0 * n + t.1] = true;
                    stack.push(t);
                }
            }
        }
        true
    }

    /// Copenhagen's exit fort: the king on the edge, able to move, in a
    /// region of empty squares and defenders that no attacker is in, walled
    /// by defenders none of which can ever be captured (each, along each
    /// line, has the edge, the region or another wall piece on one side).
    pub fn exit_fort(&self, rules: &Rules, king: Square) -> bool {
        if !rules.on_edge(king) || self.moves_from(rules, king).is_empty() {
            return false;
        }
        let n = self.size;
        // The king's region: empty squares he could reach step by step.
        let mut region = vec![false; n * n];
        region[king.0 * n + king.1] = true;
        let mut stack = vec![king];
        let mut wall: Vec<Square> = Vec::new();
        while let Some(s) = stack.pop() {
            for d in DIRECTIONS {
                let Some(t) = self.step(s, d) else { continue };
                if region[t.0 * n + t.1] {
                    continue;
                }
                match self.at(t) {
                    None => {
                        region[t.0 * n + t.1] = true;
                        stack.push(t);
                    }
                    Some(Piece::Defender) => {
                        if !wall.contains(&t) {
                            wall.push(t);
                        }
                    }
                    Some(_) => return false,
                }
            }
        }
        let safe_side = |s: Option<Square>| match s {
            None => true,
            Some(s) => region[s.0 * n + s.1] || wall.contains(&s),
        };
        wall.iter().all(|&w| {
            (safe_side(self.step(w, (-1, 0))) || safe_side(self.step(w, (1, 0))))
                && (safe_side(self.step(w, (0, -1))) || safe_side(self.step(w, (0, 1))))
        })
    }

    /// The king's clear ways out to an escape square in one move, for
    /// Raichi (one) and Tuichi (two).
    pub fn clear_ways(&self, rules: &Rules) -> usize {
        let Some(king) = self.king() else { return 0 };
        self.moves_from(rules, king)
            .iter()
            .filter(|m| match rules.escape {
                Escape::Corners => rules.is_corner(m.to),
                Escape::Edge => rules.on_edge(m.to),
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(rules: &Rules) -> Position {
        let mut p = Position::new(rules);
        p.board = vec![None; rules.size * rules.size];
        p
    }

    fn put(p: &mut Position, s: Square, piece: Piece) {
        p.set(s, Some(piece));
    }

    #[test]
    fn starting_positions() {
        for rules in Rules::all() {
            let p = Position::new(&rules);
            let attackers = p.pieces(Side::Attackers).len();
            let defenders = p.pieces(Side::Defenders).len();
            assert_eq!(attackers, rules.attackers.len());
            assert_eq!(p.king(), Some(rules.throne()), "{}", rules.id);
            // Twice as many attackers as defenders, king aside.
            assert_eq!(attackers, 2 * (defenders - 1), "{}", rules.id);
        }
    }

    #[test]
    fn rook_moves_and_restricted_squares() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        put(&mut p, (5, 2), Piece::Attacker);
        put(&mut p, (5, 9), Piece::Defender);
        p.to_move = Side::Attackers;
        let tos: Vec<Square> = p.moves_from(&rules, (5, 2)).iter().map(|m| m.to).collect();
        // Passes the throne (5,5) but cannot stop on it; stops at (5,8).
        assert!(tos.contains(&(5, 6)) && tos.contains(&(5, 8)));
        assert!(!tos.contains(&(5, 5)) && !tos.contains(&(5, 9)));
        // Cannot stop on a corner either.
        put(&mut p, (0, 4), Piece::Attacker);
        assert!(!p.moves_from(&rules, (0, 4)).iter().any(|m| m.to == (0, 0)));
    }

    #[test]
    fn custodian_capture_and_safe_entry() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        put(&mut p, (2, 2), Piece::Attacker);
        put(&mut p, (2, 3), Piece::Defender);
        put(&mut p, (2, 6), Piece::Attacker);
        put(&mut p, (8, 8), Piece::King);
        p.to_move = Side::Attackers;
        let out = p.play(&rules, Move { from: (2, 6), to: (2, 4) });
        assert_eq!(out.captured, vec![(2, 3)]);
        // Moving between two enemies is safe.
        let mut q = empty(&rules);
        put(&mut q, (3, 2), Piece::Attacker);
        put(&mut q, (3, 4), Piece::Attacker);
        put(&mut q, (6, 3), Piece::Defender);
        put(&mut q, (8, 8), Piece::King);
        q.to_move = Side::Defenders;
        let out = q.play(&rules, Move { from: (6, 3), to: (3, 3) });
        assert!(out.captured.is_empty());
        assert_eq!(q.at((3, 3)), Some(Piece::Defender));
    }

    #[test]
    fn corners_and_throne_are_hostile() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        put(&mut p, (0, 1), Piece::Defender);
        put(&mut p, (0, 5), Piece::Attacker);
        put(&mut p, (8, 8), Piece::King);
        p.to_move = Side::Attackers;
        let out = p.play(&rules, Move { from: (0, 5), to: (0, 2) });
        assert_eq!(out.captured, vec![(0, 1)], "against the corner");
        // An attacker beside the throne is taken against it.
        let mut q = empty(&rules);
        put(&mut q, (5, 4), Piece::Attacker);
        put(&mut q, (5, 1), Piece::Defender);
        put(&mut q, (8, 8), Piece::King);
        q.to_move = Side::Defenders;
        let out = q.play(&rules, Move { from: (5, 1), to: (5, 3) });
        assert_eq!(out.captured, vec![(5, 4)]);
    }

    #[test]
    fn copenhagen_king_needs_four_and_is_safe_on_the_edge() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        put(&mut p, (3, 3), Piece::King);
        put(&mut p, (2, 3), Piece::Attacker);
        put(&mut p, (4, 3), Piece::Attacker);
        put(&mut p, (3, 2), Piece::Attacker);
        put(&mut p, (3, 6), Piece::Attacker);
        put(&mut p, (8, 8), Piece::Defender);
        p.to_move = Side::Attackers;
        let out = p.play(&rules, Move { from: (3, 6), to: (3, 4) });
        assert_eq!(out.winner, Some(Side::Attackers));
        // Two are not enough.
        let mut q = empty(&rules);
        put(&mut q, (3, 3), Piece::King);
        put(&mut q, (3, 2), Piece::Attacker);
        put(&mut q, (3, 6), Piece::Attacker);
        put(&mut q, (8, 8), Piece::Defender);
        q.to_move = Side::Attackers;
        let out = q.play(&rules, Move { from: (3, 6), to: (3, 4) });
        assert_eq!(out.winner, None);
        assert!(q.king().is_some());
    }

    #[test]
    fn tablut_king_is_taken_by_two_but_by_four_on_the_throne() {
        let rules = Rules::tablut();
        let mut p = empty(&rules);
        put(&mut p, (2, 2), Piece::King);
        put(&mut p, (2, 1), Piece::Attacker);
        put(&mut p, (2, 6), Piece::Attacker);
        put(&mut p, (7, 7), Piece::Defender);
        p.to_move = Side::Attackers;
        let out = p.play(&rules, Move { from: (2, 6), to: (2, 3) });
        assert_eq!(out.winner, Some(Side::Attackers));
        let mut q = empty(&rules);
        put(&mut q, (4, 4), Piece::King);
        put(&mut q, (4, 3), Piece::Attacker);
        put(&mut q, (4, 7), Piece::Attacker);
        put(&mut q, (7, 7), Piece::Defender);
        q.to_move = Side::Attackers;
        let out = q.play(&rules, Move { from: (4, 7), to: (4, 5) });
        assert_eq!(out.winner, None, "on the throne two are not enough");
    }

    #[test]
    fn escapes() {
        let rules = Rules::tablut();
        let mut p = empty(&rules);
        put(&mut p, (2, 4), Piece::King);
        put(&mut p, (2, 1), Piece::Attacker);
        put(&mut p, (2, 7), Piece::Attacker);
        put(&mut p, (7, 4), Piece::Attacker);
        p.to_move = Side::Defenders;
        assert_eq!(p.clear_ways(&rules), 1, "Raichi");
        let out = p.play(&rules, Move { from: (2, 4), to: (0, 4) });
        assert_eq!(out.winner, Some(Side::Defenders), "the edge");
        let rules = Rules::copenhagen();
        let mut q = empty(&rules);
        put(&mut q, (0, 4), Piece::King);
        put(&mut q, (6, 6), Piece::Attacker);
        q.to_move = Side::Defenders;
        let out = q.play(&rules, Move { from: (0, 4), to: (0, 0) });
        assert_eq!(out.winner, Some(Side::Defenders), "a corner");
    }

    #[test]
    fn shield_wall() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        // Defenders on the top edge at (0,3),(0,4); attackers in front and
        // on (0,2); the attacker moving to (0,5) closes the wall.
        put(&mut p, (0, 3), Piece::Defender);
        put(&mut p, (0, 4), Piece::Defender);
        put(&mut p, (1, 3), Piece::Attacker);
        put(&mut p, (1, 4), Piece::Attacker);
        put(&mut p, (0, 2), Piece::Attacker);
        put(&mut p, (3, 5), Piece::Attacker);
        put(&mut p, (8, 8), Piece::King);
        p.to_move = Side::Attackers;
        let mut out = p.play(&rules, Move { from: (3, 5), to: (0, 5) });
        out.captured.sort();
        assert_eq!(out.captured, vec![(0, 3), (0, 4)]);
    }

    #[test]
    fn encirclement() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        put(&mut p, (5, 5), Piece::King);
        for s in [(4, 5), (6, 5), (5, 4), (5, 6), (4, 4), (4, 6), (6, 4), (6, 6)] {
            put(&mut p, s, Piece::Attacker);
        }
        assert!(p.encircled());
        // Corners of the ring do not matter: pieces move along lines.
        p.set((4, 4), None);
        assert!(p.encircled());
        p.set((4, 5), None);
        assert!(!p.encircled());
    }

    #[test]
    fn exit_fort() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        // The king on the bottom edge with room to move, walled by defenders
        // that each have the edge or another wall piece beside them.
        put(&mut p, (10, 5), Piece::King);
        for s in [(10, 3), (9, 3), (9, 4), (9, 5), (9, 6), (9, 7), (10, 7)] {
            put(&mut p, s, Piece::Defender);
        }
        put(&mut p, (2, 2), Piece::Attacker);
        assert!(p.exit_fort(&rules, (10, 5)));
        // Open the wall and it is no fort.
        p.set((9, 5), None);
        assert!(!p.exit_fort(&rules, (10, 5)));
    }
}
