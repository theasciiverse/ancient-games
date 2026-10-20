//! The computer opponent: alpha-beta search, deeper as it gets harder, with
//! a judgement of positions from the geese's side.

pub use ancient_core::player::Strength;

use crate::rules::{DIRECTIONS, Move, Position, Rules, Side, step};
use ancient_core::rng::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 5;

/// What returning to a position already seen costs, in the mover's eyes: a
/// goose is worth 1000, so this only breaks ties and endless shuffling.
const REPEAT: i32 = 150;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most.
fn budget(strength: Strength) -> (Duration, u32) {
    match strength {
        Strength::Easy => (Duration::from_millis(50), 1),
        Strength::Normal => (Duration::from_millis(250), 4),
        Strength::Hard => (Duration::from_millis(900), 12),
    }
}

/// The best move for the side to move, searching deeper while time allows.
/// `seen` counts the positions of the game so far, to keep away from
/// repeating them.
pub fn choose(
    rules: &Rules,
    position: &Position,
    strength: Strength,
    seen: &HashMap<Position, u32>,
    rng: &mut Rng,
) -> Option<Move> {
    let mut moves = position.moves(rules);
    if moves.is_empty() {
        return None;
    }
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        let i = (rng.next() % moves.len() as u64) as usize;
        return Some(moves[i].clone());
    }
    let (time, max_depth) = budget(strength);
    let start = Instant::now();
    let mut best = moves[0].clone();
    for depth in 1..=max_depth {
        let mut alpha = -WIN - 1;
        let mut found: Option<(i32, Move)> = None;
        // Search last round's best first.
        if let Some(i) = moves.iter().position(|m| *m == best) {
            moves.swap(0, i);
        }
        let mut out_of_time = false;
        for mv in &moves {
            let mut next = position.clone();
            next.play(mv);
            let mut value = -negamax(rules, &next, depth - 1, -WIN - 1, -alpha, start, time);
            value -= REPEAT * *seen.get(&next).unwrap_or(&0) as i32;
            if start.elapsed() > time && depth > 1 {
                out_of_time = true;
                break;
            }
            if found.as_ref().is_none_or(|(v, _)| value > *v) {
                found = Some((value, mv.clone()));
                alpha = alpha.max(value);
            }
        }
        if out_of_time {
            break;
        }
        if let Some((value, mv)) = found {
            best = mv;
            if value.abs() >= WIN - 100 {
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
    if let Some(winner) = position.winner(rules) {
        // Sooner wins are better: the depth left counts.
        let value = WIN - 100 + depth as i32;
        return if winner == position.to_move { value } else { -value };
    }
    if depth == 0 || start.elapsed() > time {
        let geese = judge(position);
        return if position.to_move == Side::Geese { geese } else { -geese };
    }
    let mut moves = position.moves(rules);
    // Jumps first: they decide the most.
    moves.sort_by_key(|m| std::cmp::Reverse(m.path.len()));
    let mut best = -WIN - 1;
    for mv in moves {
        let mut next = position.clone();
        next.play(&mv);
        let value = -negamax(rules, &next, depth - 1, -beta, -alpha, start, time);
        best = best.max(value);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// How good the position is for the geese: geese alive, the fox hemmed in,
/// no goose open to a jump, and the gaggle pressing up the board.
fn judge(position: &Position) -> i32 {
    let mut value = position.geese.len() as i32 * 1000;
    // The fox's freedom.
    let mut fox_room = 0;
    let mut open = 0;
    for d in DIRECTIONS {
        if let Some(q) = step(position.fox, d) {
            if position.is_empty(q) {
                fox_room += 1;
            } else if let Some(beyond) = step(q, d)
                && position.is_empty(beyond)
            {
                open += 1;
            }
        }
    }
    value -= fox_room * 60;
    value -= open * 400;
    // Further up the board is better for the geese, and so is staying close
    // together.
    for &(r, c) in &position.geese {
        value += (6 - r as i32) * 8;
        let friends = DIRECTIONS
            .iter()
            .filter_map(|&d| step((r, c), d))
            .filter(|&q| position.is_goose(q))
            .count() as i32;
        value += friends * 6;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Point, Rules};

    #[test]
    fn the_fox_takes_a_free_goose() {
        let rules = Rules::thirteen();
        let mut p = Position::new(&rules);
        p.to_move = Side::Fox;
        p.fox = (2, 3);
        p.geese = vec![(3, 3), (6, 2), (6, 3), (6, 4), (5, 2)];
        p.geese.sort();
        let mv = choose(&rules, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert_eq!(mv.killed(), vec![(3, 3)]);
    }

    #[test]
    fn a_goose_steps_out_of_danger() {
        let rules = Rules::thirteen();
        let mut p = Position::new(&rules);
        // The goose on (3,3) can be jumped (4,3 is empty); the geese move.
        p.fox = (2, 3);
        p.geese = vec![(3, 3), (5, 2), (5, 3), (5, 4), (6, 2), (6, 3), (6, 4)];
        p.geese.sort();
        let mv = choose(&rules, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        let mut next = p.clone();
        next.play(&mv);
        let jumps: Vec<Point> = next.moves(&rules).iter().flat_map(|m| m.killed()).collect();
        assert!(jumps.is_empty(), "left a goose open: {mv:?}");
    }
}

#[cfg(test)]
mod measure {
    use super::*;
    use crate::rules::Rules;

    /// Plays a game: `None` is a draw by threefold repetition or 400 moves.
    fn play(rules: &Rules, fox: Strength, geese: Strength) -> Option<Side> {
        let mut p = Position::new(rules);
        let mut seen: HashMap<Position, u32> = HashMap::new();
        let mut rng = Rng::new(9);
        for _ in 0..400 {
            if let Some(w) = p.winner(rules) {
                return Some(w);
            }
            let n = seen.entry(p.clone()).or_insert(0);
            *n += 1;
            if *n >= 3 {
                return None;
            }
            let s = if p.to_move == Side::Fox { fox } else { geese };
            let mv = choose(rules, &p, s, &seen, &mut rng).unwrap();
            p.play(&mv);
        }
        None
    }

    #[test]
    #[ignore]
    fn matches() {
        use Strength::*;
        for rules in Rules::all() {
            for (fox, geese) in [(Normal, Normal), (Hard, Hard), (Hard, Easy), (Easy, Hard), (Hard, Normal), (Normal, Hard)] {
                println!("{}: fox {} v geese {}: {:?}", rules.id, fox.name(), geese.name(), play(&rules, fox, geese));
            }
        }
    }
}
