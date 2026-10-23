//! The computer opponent. Mū tōrere has only a few hundred positions, so
//! the game is solved once at the start (`rules::solve`) and the computer
//! knows the value of every move. Hard plays perfectly: the quickest win,
//! else a draw, else the longest defence. Normal does too, but now and then
//! plays any move that does not lose at once. Easy plays at random, but
//! takes a win in one and avoids losing in one when it can.

pub use ancient_core::player::Strength;

use crate::rules::solve::Value;
use crate::rules::{Move, Position, Rules};
use ancient_core::rng::Rng;
use std::collections::HashMap;

/// How often normal strays from the best move, in sixteenths.
const NORMAL_SLIPS: u64 = 5;

/// The best move for the side to move. `seen` counts the positions of
/// the game so far: among equal moves, one to a position seen less often
/// is preferred, so a drawn game does not go round the same circle.
pub fn choose(
    rules: &Rules,
    table: &HashMap<Position, Value>,
    position: &Position,
    strength: Strength,
    seen: &HashMap<Position, u32>,
    rng: &mut Rng,
) -> Option<Move> {
    let moves = position.moves(rules);
    if moves.len() <= 1 {
        return moves.first().copied();
    }
    let after = |mv: &Move| {
        let mut next = *position;
        next.play(rules, mv);
        next
    };
    // A move's worth to the mover: higher is better.
    let worth = |mv: &Move| match table.get(&after(mv)).copied().unwrap_or(Value::Draw) {
        Value::Loss(n) => 1000 - n as i32,
        Value::Draw => 0,
        Value::Win(n) => -1000 + n as i32,
    };
    let random = |moves: &[Move], rng: &mut Rng| moves[(rng.next() % moves.len() as u64) as usize];
    let wins_now: Vec<Move> = moves.iter().copied().filter(|m| after(m).moves(rules).is_empty()).collect();
    if !wins_now.is_empty() {
        return Some(wins_now[0]);
    }
    // Moves after which the other side cannot win at once.
    let safe: Vec<Move> = moves
        .iter()
        .copied()
        .filter(|m| {
            let next = after(m);
            !next.moves(rules).iter().any(|r| {
                let mut q = next;
                q.play(rules, r);
                q.moves(rules).is_empty()
            })
        })
        .collect();
    let pool = if safe.is_empty() { &moves } else { &safe };
    match strength {
        Strength::Easy => return Some(random(pool, rng)),
        Strength::Normal if rng.next() % 16 < NORMAL_SLIPS => return Some(random(pool, rng)),
        _ => {}
    }
    let best = moves.iter().map(worth).max()?;
    let equal: Vec<Move> = moves.iter().copied().filter(|m| worth(m) == best).collect();
    let fewest = equal.iter().map(|m| *seen.get(&after(m)).unwrap_or(&0)).min()?;
    let fresh: Vec<Move> = equal.into_iter().filter(|m| *seen.get(&after(m)).unwrap_or(&0) == fewest).collect();
    Some(random(&fresh, rng))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::solve;

    #[test]
    fn hard_never_loses_from_the_start() {
        let rules = Rules::beside_enemy();
        let table = solve::table(&rules);
        for seed in 1..20 {
            let mut rng = Rng::new(seed);
            let mut p = Position::new(&rules);
            let mut seen = HashMap::new();
            for ply in 0..200 {
                if p.winner(&rules).is_some() {
                    break;
                }
                *seen.entry(p).or_insert(0) += 1;
                // Hard plays White, easy Black.
                let s = if ply % 2 == 0 { Strength::Hard } else { Strength::Easy };
                let mv = choose(&rules, &table, &p, s, &seen, &mut rng).unwrap();
                p.play(&rules, &mv);
            }
            assert_ne!(p.winner(&rules), Some(crate::rules::Side::Black), "seed {seed}");
        }
    }
}
