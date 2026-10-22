//! The computer opponent: alpha-beta search, deeper as it gets harder, with
//! a judgement of positions from the defenders' side: how close the king is
//! to getting out, how closely he is pressed, and the pieces left.

pub use ancient_core::player::Strength;

use crate::rules::{DIRECTIONS, Escape, Move, Piece, Position, Repetition, Rules, Side, Square};
use ancient_core::rng::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

/// What returning to a position already seen costs, in the mover's eyes.
const REPEAT: i32 = 60;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most.
fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(80), 1),
        Strength::Normal => (Duration::from_millis(400), 2),
        Strength::Hard => (Duration::from_millis(1500), 8),
    };
    // The rulebook's pictures search to a fixed depth, the same on every
    // machine, and no deeper than normal so they are made in good time.
    if ancient_core::capture::untimed() {
        (Duration::MAX, depth.min(2))
    } else {
        (time, depth)
    }
}

/// The best move for the side to move, searching deeper while time allows.
/// `seen` counts the positions of the game so far: a third repetition ends
/// the game as the rules say, and coming back at all is a little worse.
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
    order(position, &mut moves);
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        let i = (rng.next() % moves.len() as u64) as usize;
        return Some(moves[i]);
    }
    let mover = position.to_move;
    let (time, max_depth) = budget(strength);
    let start = Instant::now();
    let mut best = moves[0];
    for depth in 1..=max_depth {
        if let Some(i) = moves.iter().position(|m| *m == best) {
            moves[..=i].rotate_right(1);
        }
        let mut alpha = -WIN - 1;
        let mut found: Option<(i32, Move)> = None;
        let mut out_of_time = false;
        for &mv in &moves {
            let mut next = position.clone();
            let out = next.play(rules, mv);
            let value = match out.winner {
                Some(w) if w == mover => WIN,
                Some(_) => -WIN,
                None => match seen.get(&next).copied().unwrap_or(0) {
                    n if n >= 2 => repeated(rules, mover),
                    n => {
                        -negamax(rules, &next, depth - 1, -WIN - 1, -alpha, start, time)
                            - REPEAT * n as i32
                    }
                },
            };
            if start.elapsed() > time && depth > 1 {
                out_of_time = true;
                break;
            }
            if found.is_none_or(|(v, _)| value > v) {
                found = Some((value, mv));
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

/// The value for `mover` of making a position appear the third time.
fn repeated(rules: &Rules, mover: Side) -> i32 {
    match rules.repetition {
        Repetition::Draw => 0,
        Repetition::MoverLoses => -WIN,
        Repetition::DefendersLose if mover == Side::Defenders => -WIN,
        Repetition::DefendersLose => WIN,
    }
}

/// King moves first, then moves landing next to an enemy piece: they
/// decide the most.
fn order(position: &Position, moves: &mut [Move]) {
    let side = position.to_move;
    moves.sort_by_key(|m| {
        if position.at(m.from) == Some(Piece::King) {
            return 0;
        }
        let touches = DIRECTIONS.iter().any(|&d| {
            position
                .step(m.to, d)
                .and_then(|s| position.at(s))
                .is_some_and(|p| p.side() != side)
        });
        if touches { 1 } else { 2 }
    });
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
    if depth == 0 || start.elapsed() > time {
        let defenders = judge(rules, position);
        return if position.to_move == Side::Defenders { defenders } else { -defenders };
    }
    let mover = position.to_move;
    let mut moves = position.moves(rules);
    if moves.is_empty() {
        return -WIN + 100;
    }
    order(position, &mut moves);
    let mut best = -WIN - 1;
    for mv in moves {
        let mut next = position.clone();
        let out = next.play(rules, mv);
        // Sooner wins are better: the depth left counts.
        let value = match out.winner {
            Some(w) if w == mover => WIN - 100 + depth as i32,
            Some(_) => -(WIN - 100 + depth as i32),
            None => -negamax(rules, &next, depth - 1, -beta, -alpha, start, time),
        };
        best = best.max(value);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// How many king moves at least to an escape square, up to 4.
fn escape_distance(rules: &Rules, position: &Position, king: Square) -> u32 {
    let n = position.size;
    let mut seen = vec![false; n * n];
    seen[king.0 * n + king.1] = true;
    let mut frontier = vec![king];
    for distance in 1..=3 {
        let mut next = Vec::new();
        for &s in &frontier {
            for d in DIRECTIONS {
                let mut t = s;
                while let Some(u) = position.step(t, d) {
                    if position.at(u).is_some() {
                        break;
                    }
                    t = u;
                    let escaped = match rules.escape {
                        Escape::Corners => rules.is_corner(t),
                        Escape::Edge => rules.on_edge(t),
                    };
                    if escaped {
                        return distance;
                    }
                    if !seen[t.0 * n + t.1] {
                        seen[t.0 * n + t.1] = true;
                        next.push(t);
                    }
                }
            }
        }
        frontier = next;
    }
    4
}

/// How good the position is for the defenders.
fn judge(rules: &Rules, position: &Position) -> i32 {
    let Some(king) = position.king() else { return -WIN + 200 };
    let mut value = 0;
    let mut attackers = 0;
    let mut defenders = 0;
    for p in position.board.iter().flatten() {
        match p {
            Piece::Attacker => attackers += 1,
            Piece::Defender => defenders += 1,
            Piece::King => {}
        }
    }
    // A defender is worth about two attackers: there are half as many.
    value += defenders * 200 - attackers * 100;

    let ways = position.clear_ways(rules);
    let to_move = position.to_move;
    if ways > 0 && to_move == Side::Defenders {
        value += 50_000;
    } else if ways >= 2 {
        // Two ways out and one move to block them.
        value += 20_000;
    } else {
        value += match escape_distance(rules, position, king) {
            1 => 600,
            2 => 250,
            3 => 80,
            _ => 0,
        };
    }

    // Attackers beside the king, and the room he has.
    let mut pressed = 0;
    let mut room = 0;
    for d in DIRECTIONS {
        match position.step(king, d).map(|s| position.at(s)) {
            Some(Some(Piece::Attacker)) => pressed += 1,
            Some(None) => room += 1,
            _ => {}
        }
    }
    let press = match rules.king_capture {
        crate::rules::KingCapture::TwoSides { .. } => 220,
        crate::rules::KingCapture::FourSides { .. } => 120,
    };
    value -= pressed * press;
    value += room * 30;

    // Attackers guarding the squares that lead to the corners.
    if rules.escape == Escape::Corners {
        let e = rules.size - 1;
        for s in [(0, 1), (1, 0), (0, e - 1), (1, e), (e, 1), (e - 1, 0), (e, e - 1), (e - 1, e)] {
            if position.at(s) == Some(Piece::Attacker) {
                value -= 40;
            }
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(rules: &Rules) -> Position {
        let mut p = Position::new(rules);
        p.board = vec![None; rules.size * rules.size];
        p
    }

    #[test]
    fn the_king_takes_the_way_out() {
        let rules = Rules::copenhagen();
        let mut p = empty(&rules);
        p.board[3 * 11] = Some(Piece::King); // (3,0): a straight run to (0,0)
        p.board[5 * 11 + 5] = Some(Piece::Attacker);
        p.board[7 * 11 + 7] = Some(Piece::Defender);
        p.to_move = Side::Defenders;
        let mv = choose(&rules, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert!(rules.is_corner(mv.to), "{mv:?}");
    }

    #[test]
    fn the_attackers_block_one_way_out() {
        let rules = Rules::tablut();
        let mut p = empty(&rules);
        // The king on (2,2) sees the top edge up column 2; every other way
        // is shut by attackers that cannot be taken.
        p.board[2 * 9 + 2] = Some(Piece::King);
        for s in [(2, 1), (2, 6), (6, 2), (8, 8), (7, 5)] {
            p.board[s.0 * 9 + s.1] = Some(Piece::Attacker);
        }
        p.board[5 * 9 + 7] = Some(Piece::Defender);
        p.to_move = Side::Attackers;
        assert_eq!(p.clear_ways(&rules), 1);
        let mv = choose(&rules, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        let mut next = p.clone();
        next.play(&rules, mv);
        assert_eq!(next.clear_ways(&rules), 0, "left the way open: {mv:?}");
    }
}

#[cfg(test)]
mod measure {
    use super::*;

    /// Plays a game: `None` is a draw (by the rules, or 300 moves).
    fn play(rules: &Rules, attackers: Strength, defenders: Strength) -> Option<Side> {
        let mut p = Position::new(rules);
        let mut seen: HashMap<Position, u32> = HashMap::new();
        let mut rng = Rng::new(9);
        for _ in 0..300 {
            let s = if p.to_move == Side::Attackers { attackers } else { defenders };
            let mover = p.to_move;
            let mv = choose(rules, &p, s, &seen, &mut rng)?;
            let out = p.play(rules, mv);
            if out.winner.is_some() {
                return out.winner;
            }
            let n = seen.entry(p.clone()).or_insert(0);
            *n += 1;
            if *n >= 3 {
                return match rules.repetition {
                    Repetition::Draw => None,
                    Repetition::DefendersLose => Some(Side::Attackers),
                    Repetition::MoverLoses => Some(mover.other()),
                };
            }
        }
        None
    }

    #[test]
    #[ignore]
    fn matches() {
        use Strength::*;
        for rules in Rules::all() {
            for (a, d) in [(Normal, Normal), (Hard, Easy), (Easy, Hard), (Hard, Normal), (Normal, Hard)] {
                println!("{}: attackers {} v defenders {}: {:?}", rules.id, a.name(), d.name(), play(&rules, a, d));
            }
        }
    }
}
