//! The computer opponent: alpha-beta search with iterative deepening,
//! deeper as it gets harder. A position is judged by the men each side
//! has, then by how freely each can move, and in Alquerque by the men
//! left standing where they can be leapt.

pub use ancient_core::player::Strength;

use crate::rules::{Move, Position, Rules};
#[cfg(test)]
use crate::rules::Outcome;
use ancient_core::rng::Rng;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most, in turns.
fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(60), 1),
        Strength::Normal => (Duration::from_millis(400), 3),
        Strength::Hard => (Duration::from_millis(1500), 30),
    };
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
        return Some(order.swap_remove(i));
    }
    let (time, max_depth) = budget(strength);
    let clock = Clock { start: Instant::now(), time };
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
            next.play(mv);
            // Searched against the best less one, so that the noise below
            // only ever chooses among equals.
            let value = -negamax(rules, &next, depth - 1, -WIN * 2, -(alpha - 1), &clock);
            if clock.out() && depth > 1 {
                out_of_time = true;
                break;
            }
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
        if clock.out() {
            break;
        }
    }
    Some(best)
}

/// The moves, the biggest captures first.
fn ordered(rules: &Rules, position: &Position) -> Vec<Move> {
    let mut moves = position.moves(rules);
    moves.sort_by_key(|m| std::cmp::Reverse(m.captures()));
    moves
}

/// The value of `position` for the side to move.
fn negamax(rules: &Rules, position: &Position, depth: u32, mut alpha: i32, beta: i32, clock: &Clock) -> i32 {
    let me = position.to_move;
    let moves = ordered(rules, position);
    if position.count(me) == 0 || moves.is_empty() {
        return -(WIN - 100 + depth as i32);
    }
    if position.quiet >= rules.quiet {
        return 0;
    }
    if depth == 0 || clock.out() {
        return judge(rules, position, &moves);
    }
    let mut best = -WIN * 2;
    for mv in moves {
        let mut next = position.clone();
        next.play(&mv);
        let value = -negamax(rules, &next, depth - 1, -beta, -alpha, clock);
        best = best.max(value);
        alpha = alpha.max(value);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Men, then room to move. The side to move with a capture in hand is
/// worth that capture; the side waiting is not yet hurt by it.
fn judge(rules: &Rules, position: &Position, moves: &[Move]) -> i32 {
    let me = position.to_move;
    let men = position.count(me) as i32 - position.count(me.other()) as i32;
    let in_hand = moves.iter().map(|m| m.captures()).max().unwrap_or(0) as i32;
    let mut theirs = position.clone();
    theirs.to_move = me.other();
    let their_room = theirs.moves(rules).len() as i32;
    let room = moves.len() as i32 - their_room;
    men * 100 + in_hand * 60 + room
}

/// How a game between two strengths ends, for the tests.
#[cfg(test)]
pub fn play_out(rules: &Rules, players: [Strength; 2], seed: u64) -> (Option<Outcome>, usize) {
    let mut p = Position::new(rules);
    let mut rng = Rng::new(seed);
    let mut turns = 0;
    while p.outcome(rules).is_none() {
        let mv = choose(rules, &p, players[p.to_move.index()], &mut rng).unwrap();
        p.play(&mv);
        turns += 1;
    }
    (p.outcome(rules), turns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Side;
    use crate::rules::tests::board;

    #[test]
    fn games_end() {
        for rules in Rules::all() {
            let (outcome, _) = play_out(&rules, [Strength::Easy, Strength::Easy], 3);
            assert!(outcome.is_some(), "{}", rules.id);
        }
    }

    #[test]
    fn takes_the_most() {
        let f = Rules::fanorona();
        use Side::*;
        // e3 to f3 approaches three; e3 to d3 approaches one; e3 to e4
        // or d4 approaches nothing.
        let p = board(&f, White, &[("e3", White), ("g3", Black), ("h3", Black), ("i3", Black), ("c3", Black), ("a5", Black)]);
        let mv = choose(&f, &p, Strength::Normal, &mut Rng::new(1)).unwrap();
        assert!(mv.captures() >= 3, "{mv:?}");
    }
}

/// Strength against strength, sides alternating. Slow:
/// `ALQUERQUE_GAMES=10 cargo test --release -p alquerque strengths -- --ignored --nocapture`
/// (`ALQUERQUE_ONLY=<id>` for one rule set).
#[cfg(test)]
mod strength_tests {
    use super::*;
    use crate::rules::Side;

    #[test]
    #[ignore]
    fn strengths() {
        let games: u64 = std::env::var("ALQUERQUE_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        let only = std::env::var("ALQUERQUE_ONLY").ok();
        for rules in Rules::all().into_iter().filter(|r| only.as_deref().is_none_or(|o| o == r.id)) {
            for (strong, weak) in
                [(Strength::Normal, Strength::Easy), (Strength::Hard, Strength::Easy), (Strength::Hard, Strength::Normal)]
            {
                let (mut won, mut drawn) = (0, 0);
                for g in 0..games {
                    let strong_side = if g % 2 == 0 { Side::White } else { Side::Black };
                    let mut players = [weak; 2];
                    players[strong_side.index()] = strong;
                    let (outcome, turns) = play_out(&rules, players, g + 1);
                    match outcome {
                        Some(Outcome::Win(s)) if s == strong_side => won += 1,
                        Some(Outcome::Draw) => drawn += 1,
                        _ => {}
                    }
                    eprintln!("  {} {:?} as {:?}: {:?} in {turns} turns", rules.id, strong, strong_side, outcome);
                }
                eprintln!("{}: {:?} beats {:?} {won}/{games}, {drawn} drawn", rules.id, strong, weak);
            }
        }
    }
}
