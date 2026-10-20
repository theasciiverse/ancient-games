//! The computer opponent: a beginner, a careful judge of the move at hand,
//! and an expectimax that looks at the throws of the next turns.

pub use ancient_core::player::Strength;

use crate::rules::{Action, BEAUTY, OFF, Position, Rules, Side, THREE_TRUTHS, Variant, WATER};

/// Throws again that do not use up a turn of lookahead, in a row.
const FREE_AGAINS: u32 = 1;

fn depth(strength: Strength) -> u32 {
    match strength {
        Strength::Easy | Strength::Normal => 0,
        Strength::Hard => 2,
    }
}

/// The best action for the side to move with throw `t`, or `None`.
pub fn choose(rules: &Rules, position: &Position, t: u8, strength: Strength) -> Option<Action> {
    let moves = position.moves(rules, t);
    if strength == Strength::Easy {
        return naive(position, &moves);
    }
    let me = position.to_move;
    let mut best: Option<(f64, Action)> = None;
    for action in moves {
        let mut next = position.clone();
        next.play(rules, action, t);
        let value = value_after(rules, &next, me, depth(strength), FREE_AGAINS);
        if best.is_none_or(|(v, _)| value > v) {
            best = Some((value, action));
        }
    }
    best.map(|(_, a)| a)
}

/// Kendall, seven pieces: whether to take the drowned piece to the House of
/// Rebirth instead of throwing. Walker: throwing for an exact four is only
/// worth it when that piece is the last one on the board.
pub fn wants_rescue(position: &Position) -> bool {
    position.on_board(position.to_move) > 1
}

/// A beginner: bump or capture if possible, else bear off, else move the
/// piece furthest along.
fn naive(position: &Position, moves: &[Action]) -> Option<Action> {
    let me = position.to_move;
    moves.iter().copied().max_by_key(|a| match *a {
        Action::Move { from, to } => {
            let hits = to <= 30 && position.at(to) == Some(me.other());
            let score = if hits {
                3
            } else if to == OFF {
                2
            } else {
                1
            };
            (score, from)
        }
        Action::Rescue => (0, 0),
    })
}

fn value_after(rules: &Rules, position: &Position, me: Side, depth: u32, agains: u32) -> f64 {
    if let Some(w) = position.winner(rules) {
        return if w == me { 10_000.0 } else { -10_000.0 };
    }
    if position.finished(rules) {
        return match position.bell_winner() {
            Some(w) if w == me => 10_000.0,
            Some(_) => -10_000.0,
            None => 0.0,
        };
    }
    if depth == 0 {
        return judge(rules, position, me);
    }
    let throws = rules.throws();
    let total: u32 = throws.iter().map(|&(_, n)| n).sum();
    let mover = position.to_move;
    let mut expected = 0.0;
    for (t, n) in throws {
        let moves = position.moves(rules, t);
        let value = if moves.is_empty() {
            let mut next = position.clone();
            next.pass();
            value_after(rules, &next, me, depth - 1, FREE_AGAINS)
        } else {
            let values = moves.into_iter().map(|a| {
                let mut next = position.clone();
                let out = next.play(rules, a, t);
                if out.again && agains > 0 {
                    value_after(rules, &next, me, depth, agains - 1)
                } else {
                    value_after(rules, &next, me, depth - 1, FREE_AGAINS)
                }
            });
            if mover == me {
                values.fold(f64::NEG_INFINITY, f64::max)
            } else {
                values.fold(f64::INFINITY, f64::min)
            }
        };
        expected += n as f64 / total as f64 * value;
    }
    expected
}

fn judge(rules: &Rules, position: &Position, me: Side) -> f64 {
    side_value(rules, position, me) - side_value(rules, position, me.other())
}

/// Progress, pieces home, safety, and the danger lone pieces are in. A
/// piece borne off is always worth more than any piece on the board.
fn side_value(rules: &Rules, position: &Position, side: Side) -> f64 {
    let enemy = side.other();
    let bell = rules.variant == Variant::Bell;
    // Bell: a man still in the game is worth a lot, since taken men are lost.
    let alive = if bell { 20.0 } else { 0.0 };
    let mut value = position.off[side.index()] as f64 * (alive + 45.0);
    let theirs = position.pieces(enemy);
    let throws = rules.throws();
    let total: u32 = throws.iter().map(|&(_, n)| n).sum();
    let chance = |d: u8| -> f64 {
        throws.iter().filter(|&&(t, _)| t == d).map(|&(_, n)| n as f64 / total as f64).sum()
    };
    for h in position.pieces(side) {
        value += h as f64 + alive;
        if bell && h >= BEAUTY {
            value += 6.0;
            continue;
        }
        if h == WATER {
            value -= 12.0;
            continue;
        }
        if h >= THREE_TRUTHS {
            value += 4.0;
            continue;
        }
        let friend = position.at(h - 1) == Some(side) || position.at(h + 1) == Some(side);
        if friend && rules.variant != Variant::Bell {
            value += 2.0;
            continue;
        }
        // The chance an enemy piece behind lands here next throw, against
        // what this piece would lose: its way so far, and in Bell its life.
        let risk: f64 = theirs.iter().filter(|&&q| q < h).map(|&q| chance(h - q)).sum();
        let loss = if bell { alive + h as f64 } else { (h as f64) * 0.5 };
        value -= risk.min(1.0) * loss;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use ancient_core::rng::Rng;

    fn play_out(rules: &Rules, a: Strength, b: Strength, seed: u64) -> Option<Side> {
        let mut rng = Rng::new(seed);
        let mut p = Position::new(rules, Side::White);
        let throws = rules.throws();
        let total: u32 = throws.iter().map(|&(_, n)| n).sum();
        let mut turns = 0;
        loop {
            turns += 1;
            assert!(turns < 20_000, "no end");
            if let Some(w) = p.winner(rules) {
                return Some(w);
            }
            if p.finished(rules) {
                return p.bell_winner();
            }
            let strength = if p.to_move == Side::White { a } else { b };
            if p.can_rescue(rules) && wants_rescue(&p) {
                p.play(rules, Action::Rescue, 0);
                continue;
            }
            let mut r = (rng.next() % total as u64) as u32;
            let t = throws
                .iter()
                .find(|&&(_, n)| {
                    if r < n {
                        true
                    } else {
                        r -= n;
                        false
                    }
                })
                .unwrap()
                .0;
            match choose(rules, &p, t, strength) {
                Some(action) => {
                    p.play(rules, action, t);
                }
                None => p.pass(),
            }
        }
    }

    #[test]
    fn games_end_under_every_rule_set() {
        for rules in Rules::all() {
            for seed in 1..=4 {
                play_out(&rules, Strength::Normal, Strength::Easy, seed);
            }
        }
    }

    #[test]
    #[ignore]
    fn strengths() {
        for rules in Rules::all() {
            let games = 100;
            let mut normal = 0;
            let mut hard = 0;
            for seed in 0..games {
                // Swap colours every game.
                let (w, b) = if seed % 2 == 0 {
                    (Strength::Normal, Strength::Easy)
                } else {
                    (Strength::Easy, Strength::Normal)
                };
                let winner = play_out(&rules, w, b, seed + 1);
                let normal_side = if seed % 2 == 0 { Side::White } else { Side::Black };
                if winner == Some(normal_side) {
                    normal += 1;
                }
                let (w, b) = if seed % 2 == 0 {
                    (Strength::Hard, Strength::Normal)
                } else {
                    (Strength::Normal, Strength::Hard)
                };
                let winner = play_out(&rules, w, b, seed + 1000);
                let hard_side = if seed % 2 == 0 { Side::White } else { Side::Black };
                if winner == Some(hard_side) {
                    hard += 1;
                }
            }
            println!("{}: normal beats easy {normal}/{games}, hard beats normal {hard}/{games}", rules.id);
        }
    }
}

#[cfg(test)]
mod timing {
    use super::*;
    use ancient_core::rng::Rng;
    use std::time::{Duration, Instant};

    /// The slowest move the hard computer makes over a few games.
    #[test]
    #[ignore]
    fn slowest_hard_move() {
        for rules in Rules::all() {
            let mut worst = Duration::ZERO;
            let mut rng = Rng::new(7);
            for _ in 0..3 {
                let mut p = Position::new(&rules, Side::White);
                let throws = rules.throws();
                let total: u32 = throws.iter().map(|&(_, n)| n).sum();
                for _ in 0..400 {
                    if p.winner(&rules).is_some() || p.finished(&rules) {
                        break;
                    }
                    if p.can_rescue(&rules) && wants_rescue(&p) {
                        p.play(&rules, Action::Rescue, 0);
                        continue;
                    }
                    let mut r = (rng.next() % total as u64) as u32;
                    let t = throws.iter().find(|&&(_, n)| if r < n { true } else { r -= n; false }).unwrap().0;
                    let start = Instant::now();
                    let a = choose(&rules, &p, t, Strength::Hard);
                    worst = worst.max(start.elapsed());
                    match a {
                        Some(a) => {
                            p.play(&rules, a, t);
                        }
                        None => p.pass(),
                    }
                }
            }
            println!("{}: slowest hard move {worst:?}", rules.id);
        }
    }
}
