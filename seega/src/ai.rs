//! The computer opponent: alpha-beta search with iterative deepening,
//! deeper as it gets harder. A position is judged by the kelbs each side
//! has, and then by the captures each could make next. A move that lets
//! the mover go on (a capture with another to follow) or hands the turn
//! to the other side is searched from the right side either way.

pub use ancient_core::player::Strength;

use crate::rules::{Move, Outcome, Position, Rules, Side, Stage};
use ancient_core::rng::Rng;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most, in moves.
/// Placing gets a quarter of the time: Lane saw most kelbs "placed at
/// random", and a board of nine has 76 of them to place.
fn budget(strength: Strength, placing: bool) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(60), 1),
        Strength::Normal => (Duration::from_millis(400), 3),
        Strength::Hard => (Duration::from_millis(1500), 30),
    };
    let time = if placing { time / 4 } else { time };
    // The rulebook's pictures search to a fixed depth, the same on every
    // machine.
    if ancient_core::capture::untimed() { (Duration::MAX, depth.min(2)) } else { (time, depth) }
}

struct Clock {
    start: Instant,
    time: Duration,
}

impl Clock {
    fn out(&self) -> bool {
        self.start.elapsed() > self.time
    }
}

/// The best move for the side to move, searching deeper while time allows.
pub fn choose(rules: &Rules, position: &Position, strength: Strength, rng: &mut Rng) -> Option<Move> {
    let mut order = ordered(rules, position);
    if order.len() <= 1 {
        return order.into_iter().next();
    }
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        let i = (rng.next() % order.len() as u64) as usize;
        return Some(order[i]);
    }
    let (time, max_depth) = budget(strength, position.stage == Stage::Placing);
    let clock = Clock { start: Instant::now(), time };
    let me = position.to_move;
    let mut best = order[0];
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
            // Searched against the best less one, so that the noise below
            // only ever chooses among equals.
            let value = child(rules, me, &next, depth - 1, alpha - 1, WIN * 2, &clock);
            if clock.out() && depth > 1 {
                out_of_time = true;
                break;
            }
            let value = value * 4 + (rng.next() % 3) as i32;
            if found.as_ref().is_none_or(|(v, _)| value > *v) {
                found = Some((value, *mv));
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
        if clock.out() {
            break;
        }
    }
    Some(best)
}

/// The moves, captures first.
fn ordered(rules: &Rules, position: &Position) -> Vec<Move> {
    let mut moves = position.moves(rules);
    moves.sort_by_key(|m| std::cmp::Reverse(position.taken(rules, m)));
    moves
}

/// The value of `next` for `me`, who moved into it.
fn child(rules: &Rules, me: Side, next: &Position, depth: u32, alpha: i32, beta: i32, clock: &Clock) -> i32 {
    if next.to_move == me {
        negamax(rules, next, depth, alpha, beta, clock)
    } else {
        -negamax(rules, next, depth, -beta, -alpha, clock)
    }
}

/// The value of `position` for the side to move.
fn negamax(rules: &Rules, position: &Position, depth: u32, mut alpha: i32, beta: i32, clock: &Clock) -> i32 {
    let me = position.to_move;
    match position.outcome() {
        // Sooner wins are better, later losses less bad.
        Some(Outcome::Win(side)) => {
            let v = WIN - 100 + depth as i32;
            return if side == me { v } else { -v };
        }
        Some(Outcome::Draw) => return 0,
        None => {}
    }
    if depth == 0 || clock.out() {
        return judge(rules, position);
    }
    let mut best = -WIN * 2;
    for mv in ordered(rules, position) {
        let mut next = position.clone();
        next.play(rules, &mv);
        let value = child(rules, me, &next, depth - 1, alpha, beta, clock);
        best = best.max(value);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Kelbs, then the captures each side has to hand.
fn judge(rules: &Rules, position: &Position) -> i32 {
    let me = position.to_move;
    let kelbs = position.count(me) as i32 - position.count(me.other()) as i32;
    let threats = |side: Side| {
        let mut p = position.clone();
        p.to_move = side;
        p.stage = Stage::Moving;
        p.chain = false;
        p.moves(rules).iter().filter(|m| p.taken(rules, m) > 0).count() as i32
    };
    if position.stage == Stage::Placing {
        return kelbs * 100;
    }
    kelbs * 100 + threats(me) * 12 - threats(me.other()) * 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn games_end() {
        for rules in Rules::all() {
            let mut p = Position::new(&rules);
            let mut rng = Rng::new(2);
            for _ in 0..5000 {
                if p.outcome().is_some() {
                    break;
                }
                let s = if p.to_move == Side::Stone { Strength::Normal } else { Strength::Easy };
                let mv = choose(&rules, &p, s, &mut rng).unwrap();
                p.play(&rules, &mv);
            }
            assert!(p.outcome().is_some(), "{}", rules.id);
        }
    }

    #[test]
    fn takes_what_is_there() {
        let rules = Rules::khamsawee();
        let at = |n: &str| (0..rules.len()).find(|&p| rules.name(p) == n).unwrap();
        let mut p = Position::new(&rules);
        p.at = vec![None; rules.len()];
        for (n, s) in [("c2", Side::Stone), ("c4", Side::Brick), ("c5", Side::Stone), ("a1", Side::Brick), ("e1", Side::Brick)] {
            p.at[at(n)] = Some(s);
        }
        p.stage = Stage::Moving;
        p.to_move = Side::Stone;
        let mv = choose(&rules, &p, Strength::Normal, &mut Rng::new(1)).unwrap();
        assert_eq!(mv, Move::Step { from: at("c2"), to: at("c3") });
    }
}

/// Strength against strength, sides alternating. Slow:
/// `SEEGA_GAMES=10 cargo test --release -p seega strengths -- --ignored --nocapture`.
#[cfg(test)]
mod strength_tests {
    use super::*;

    fn play(rules: &Rules, players: [Strength; 2], seed: u64) -> (Option<Outcome>, usize, u16) {
        let mut p = Position::new(rules);
        let mut rng = Rng::new(seed);
        let mut moves = 0;
        while p.outcome().is_none() {
            let mv = choose(rules, &p, players[p.to_move.index()], &mut rng).unwrap();
            p.play(rules, &mv);
            moves += 1;
        }
        (p.outcome(), moves, p.quiet)
    }

    #[test]
    #[ignore]
    fn strengths() {
        let games: u64 = std::env::var("SEEGA_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        for rules in Rules::all() {
            for (strong, weak) in [(Strength::Normal, Strength::Easy), (Strength::Hard, Strength::Easy), (Strength::Hard, Strength::Normal)] {
                let (mut won, mut drawn, mut quiet_ends) = (0, 0, 0);
                for g in 0..games {
                    let strong_side = if g % 2 == 0 { Side::Stone } else { Side::Brick };
                    let mut players = [weak; 2];
                    players[strong_side.index()] = strong;
                    let (outcome, moves, quiet) = play(&rules, players, g + 1);
                    if quiet >= crate::rules::QUIET {
                        quiet_ends += 1;
                    }
                    match outcome {
                        Some(Outcome::Win(s)) if s == strong_side => won += 1,
                        Some(Outcome::Draw) => drawn += 1,
                        _ => {}
                    }
                    eprintln!("  {} {:?} as {:?}: {:?} in {moves} moves", rules.id, strong, strong_side, outcome);
                }
                eprintln!("{}: {:?} beats {:?} {won}/{games}, {drawn} drawn, {quiet_ends} ended quiet", rules.id, strong, weak);
            }
        }
    }
}
