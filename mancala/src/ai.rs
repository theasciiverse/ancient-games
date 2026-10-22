//! The computer opponent: alpha-beta search over the sowings, deeper as it
//! gets harder. Every game here is one of perfect information with no
//! dice, so the search sees exactly what the rules will do. A move that
//! gives the same side another move (a last seed in its own store) is
//! searched as that side moving again.
//!
//! The judgement of a position is the difference between the stores, the
//! seeds each side has made safe; a game that is over counts as won or
//! lost by its margin.

pub use ancient_core::player::Strength;

use crate::rules::{Move, Position, Rules, Side};
use ancient_core::rng::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most, in moves.
fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(60), 1),
        Strength::Normal => (Duration::from_millis(400), 4),
        Strength::Hard => (Duration::from_millis(1500), 40),
    };
    // The rulebook's pictures search to a fixed depth, the same on every
    // machine, and no deeper than normal so they are made in good time.
    if ancient_core::capture::untimed() {
        (Duration::MAX, depth.min(4))
    } else {
        (time, depth)
    }
}

/// The best move for the side to move, searching deeper while time allows.
pub fn choose(rules: &Rules, position: &Position, strength: Strength, rng: &mut Rng) -> Option<Move> {
    let moves = position.moves(rules);
    if moves.len() <= 1 {
        return moves.first().copied();
    }
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        return Some(moves[(rng.next() % moves.len() as u64) as usize]);
    }
    let (time, max_depth) = budget(strength);
    let mut search = Search { rules, start: Instant::now(), time, table: HashMap::new(), out_of_time: false };
    let mut order = moves.clone();
    let mut best = moves[0];
    for depth in 1..=max_depth {
        if let Some(i) = order.iter().position(|m| *m == best) {
            order[..=i].rotate_right(1);
        }
        let mut alpha = -WIN * 2;
        let mut found: Option<(i32, Move)> = None;
        for &mv in &order {
            let mut next = position.clone();
            next.play(rules, mv);
            let value = search.value_for(position.to_move, &next, depth - 1, alpha, WIN * 2);
            if search.out_of_time && depth > 1 {
                break;
            }
            // A little noise among equals, so the computer does not always
            // open the same way.
            let value = value * 4 + (rng.next() % 3) as i32;
            if found.is_none_or(|(v, _)| value > v) {
                found = Some((value, mv));
                alpha = alpha.max(value / 4);
            }
        }
        if search.out_of_time && depth > 1 {
            break;
        }
        if let Some((value, mv)) = found {
            best = mv;
            if (value / 4).abs() >= WIN {
                break;
            }
        }
        if search.start.elapsed() > time {
            break;
        }
    }
    Some(best)
}

/// What a stored value is: exact, or only a bound because the search of
/// that position was cut short.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    Exact,
    AtLeast,
    AtMost,
}

/// The value of `position` for the side to move, searched `depth` moves
/// deep (for choices outside the moves, such as Lane's turning of the
/// board).
pub fn value(rules: &Rules, position: &Position, depth: u32) -> i32 {
    let mut search =
        Search { rules, start: Instant::now(), time: Duration::MAX, table: HashMap::new(), out_of_time: false };
    search.negamax(position, depth, -WIN * 2, WIN * 2)
}

struct Search<'a> {
    rules: &'a Rules,
    start: Instant,
    time: Duration,
    /// Positions already searched: the depth, the value for the side to
    /// move there, and whether that value is exact or a bound.
    table: HashMap<Position, (u32, i32, Bound)>,
    out_of_time: bool,
}

impl Search<'_> {
    /// The value of `position` for `side`.
    fn value_for(&mut self, side: Side, position: &Position, depth: u32, alpha: i32, beta: i32) -> i32 {
        if position.to_move == side {
            self.negamax(position, depth, alpha, beta)
        } else {
            -self.negamax(position, depth, -beta, -alpha)
        }
    }

    /// The value of `position` for the side to move there.
    fn negamax(&mut self, position: &Position, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        let me = position.to_move;
        if position.over {
            return outcome(position, me);
        }
        if depth == 0 || self.out_of_time {
            return judge(position, me);
        }
        if self.start.elapsed() > self.time {
            self.out_of_time = true;
            return judge(position, me);
        }
        let alpha_in = alpha;
        if let Some(&(d, v, bound)) = self.table.get(position)
            && d >= depth
        {
            match bound {
                Bound::Exact => return v,
                Bound::AtLeast if v >= beta => return v,
                Bound::AtMost if v <= alpha => return v,
                _ => {}
            }
        }
        let moves = position.moves(self.rules);
        if moves.is_empty() {
            return judge(position, me);
        }
        let mut best = -WIN * 2;
        for mv in moves {
            let mut next = position.clone();
            next.play(self.rules, mv);
            let value = self.value_for(me, &next, depth - 1, alpha, beta);
            best = best.max(value);
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        if !self.out_of_time {
            let bound = if best >= beta {
                Bound::AtLeast
            } else if best <= alpha_in {
                Bound::AtMost
            } else {
                Bound::Exact
            };
            self.table.insert(position.clone(), (depth, best, bound));
        }
        best
    }
}

/// The stores' difference for `side`.
fn judge(position: &Position, side: Side) -> i32 {
    position.stores[side.index()] as i32 - position.stores[side.other().index()] as i32
}

/// A finished game or round: won or lost by its margin.
fn outcome(position: &Position, side: Side) -> i32 {
    let margin = judge(position, side);
    match margin.signum() {
        1 => WIN + margin,
        -1 => -WIN + margin,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays one game from the start; returns the winner, if any.
    fn game(rules: &Rules, strengths: [Strength; 2], seed: u64) -> Option<Side> {
        let mut rng = Rng::new(seed);
        let mut position = Position::new(rules);
        for _ in 0..2000 {
            if position.over {
                break;
            }
            let side = position.to_move;
            let mv = choose(rules, &position, strengths[side.index()], &mut rng).expect("a move");
            position.play(rules, mv);
        }
        position.leader()
    }

    #[test]
    fn games_end_and_harder_wins_more() {
        let rules = Rules::oware();
        let mut normal = 0;
        for seed in 0..6 {
            // Each side in turn.
            let w = game(&rules, [Strength::Normal, Strength::Easy], seed);
            normal += (w == Some(Side::South)) as u32;
            let w = game(&rules, [Strength::Easy, Strength::Normal], seed + 100);
            normal += (w == Some(Side::North)) as u32;
        }
        assert!(normal >= 8, "normal won {normal} of 12");
    }
}
