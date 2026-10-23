//! The computer opponent: alpha-beta search with iterative deepening,
//! deeper as it gets harder. In Kōnane the side left without a move loses,
//! so a position is judged by the moves each side has: its own, less the
//! other side's.

pub use ancient_core::player::Strength;

use crate::rules::{Move, Position, Rules};
use ancient_core::rng::Rng;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most, in moves.
fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(60), 1),
        Strength::Normal => (Duration::from_millis(400), 3),
        Strength::Hard => (Duration::from_millis(1500), 30),
    };
    // The rulebook's pictures search to a fixed depth, the same on every
    // machine.
    if ancient_core::capture::untimed() { (Duration::MAX, depth.min(3)) } else { (time, depth) }
}

/// The best move for the side to move, searching deeper while time allows.
pub fn choose(rules: &Rules, position: &Position, strength: Strength, rng: &mut Rng) -> Option<Move> {
    let moves = position.moves(rules);
    if moves.len() <= 1 {
        return moves.into_iter().next();
    }
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        let i = (rng.next() % moves.len() as u64) as usize;
        return Some(moves[i].clone());
    }
    let (time, max_depth) = budget(strength);
    let start = Instant::now();
    let mut order = moves;
    let mut best = order[0].clone();
    for depth in 1..=max_depth {
        if let Some(i) = order.iter().position(|m| *m == best) {
            order[..=i].rotate_right(1);
        }
        let mut alpha = -WIN * 2;
        let mut found: Option<(i32, Move)> = None;
        let mut out_of_time = false;
        for mv in &order {
            let mut next = position.clone();
            next.play(rules, mv);
            let value = -negamax(rules, &next, depth - 1, -WIN * 2, -(alpha - 1), start, time);
            if start.elapsed() > time && depth > 1 {
                out_of_time = true;
                break;
            }
            // A little noise among equals.
            let value = value * 4 + (rng.next() % 3) as i32;
            if found.as_ref().is_none_or(|(v, _)| value > *v) {
                found = Some((value, mv.clone()));
                alpha = alpha.max(value / 4);
            }
        }
        if out_of_time {
            break;
        }
        if let Some((value, mv)) = found {
            best = mv;
            if (value / 4).abs() >= WIN - 1000 {
                break;
            }
        }
        if start.elapsed() > time {
            break;
        }
    }
    Some(best)
}

/// The value of `position` for the side to move.
fn negamax(
    rules: &Rules,
    position: &Position,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    start: Instant,
    time: Duration,
) -> i32 {
    let moves = position.moves(rules);
    if moves.is_empty() {
        // Lost; later losses are less bad.
        return -(WIN - 100 + depth as i32);
    }
    if depth == 0 || start.elapsed() > time {
        let mut other = position.clone();
        other.to_move = other.to_move.other();
        return moves.len() as i32 * 10 - other.moves(rules).len() as i32 * 12;
    }
    let mut best = -WIN * 2;
    for mv in moves {
        let mut next = position.clone();
        next.play(rules, &mv);
        let value = -negamax(rules, &next, depth - 1, -beta, -alpha, start, time);
        best = best.max(value);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Side;

    #[test]
    fn games_end() {
        for rules in Rules::all() {
            let mut p = Position::new(&rules);
            let mut rng = Rng::new(2);
            for _ in 0..rules.len() {
                if p.winner(&rules).is_some() {
                    break;
                }
                let s = if p.to_move == Side::Black { Strength::Normal } else { Strength::Easy };
                let mv = choose(&rules, &p, s, &mut rng).unwrap();
                p.play(&rules, &mv);
            }
            assert!(p.winner(&rules).is_some(), "{}", rules.id);
        }
    }
}
