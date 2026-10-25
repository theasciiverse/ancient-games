//! The computer opponent. Easy races: it moves the man furthest on and
//! hits when it can, and now and then plays any move at all. Normal
//! judges the position each move leaves: the race, the blots it leaves
//! and how many of the other side's throws hit them, the points it holds
//! ahead of the other side's men, and men taken. Hard looks one throw
//! further: for each move it weighs every throw the other side could make
//! and the best reply to each, as normal judges them.

pub use ancient_core::player::Strength;

use crate::rules::{Move, Position, Rules, Side};
use ancient_core::rng::Rng;

/// How often a beginner plays any legal move, in sixteenths.
const BEGINNER_SLIPS: u64 = 3;

/// Every throw of `n` dice, as sorted dice with how many ways it comes.
pub fn throws(n: u8) -> Vec<(Vec<u8>, u32)> {
    let mut all: std::collections::BTreeMap<Vec<u8>, u32> = Default::default();
    let total = 6u32.pow(n as u32);
    for k in 0..total {
        let mut d: Vec<u8> = (0..n).map(|i| (k / 6u32.pow(i as u32) % 6) as u8 + 1).collect();
        d.sort_unstable();
        *all.entry(d).or_default() += 1;
    }
    all.into_iter().collect()
}

pub fn choose(rules: &Rules, position: &Position, dice: &[u8], strength: Strength, rng: &mut Rng) -> Option<Move> {
    let moves = position.moves(rules, dice);
    if moves.len() <= 1 {
        return moves.into_iter().next();
    }
    let me = position.to_move;
    let after = |m: &Move| {
        let mut p = position.clone();
        p.play(rules, m);
        p
    };
    match strength {
        Strength::Easy => {
            if rng.next() % 16 < BEGINNER_SLIPS {
                let i = (rng.next() % moves.len() as u64) as usize;
                return Some(moves[i].clone());
            }
            moves.into_iter().max_by_key(|m| {
                let hits = m.hops.iter().filter(|h| h.hits).count() as i32;
                let far: i32 = m.hops.iter().map(|h| h.from as i32).sum();
                hits * 1000 + far * 10 + (rng.next() % 7) as i32
            })
        }
        Strength::Normal => {
            moves.into_iter().max_by_key(|m| judge(rules, &after(m), me) * 8 + (rng.next() % 5) as i32)
        }
        Strength::Hard => {
            // The best few by normal's judgement, then each weighed over
            // the other side's throws and best replies.
            let mut scored: Vec<(i32, Move)> = moves.into_iter().map(|m| (judge(rules, &after(&m), me), m)).collect();
            scored.sort_by_key(|(v, _)| -v);
            scored.truncate(if ancient_core::capture::untimed() { 3 } else { 6 });
            let replies = throws(rules.dice);
            let total: u32 = replies.iter().map(|(_, w)| w).sum();
            scored
                .into_iter()
                .map(|(_, m)| {
                    let p = after(&m);
                    let mut sum = 0i64;
                    for (d, w) in &replies {
                        let theirs = p.moves(rules, d);
                        let worst = theirs
                            .iter()
                            .map(|r| {
                                let mut q = p.clone();
                                q.play(rules, r);
                                judge(rules, &q, me)
                            })
                            .min()
                            .unwrap_or_else(|| judge(rules, &p, me));
                        sum += worst as i64 * *w as i64;
                    }
                    ((sum / total as i64) as i32 * 8 + (rng.next() % 5) as i32, m)
                })
                .max_by_key(|(v, _)| *v)
                .map(|(_, m)| m)
        }
    }
}

/// How `position` looks to `me`.
pub fn judge(rules: &Rules, position: &Position, me: Side) -> i32 {
    if let Some(crate::rules::Outcome::Win(w)) = position.outcome(rules) {
        return if w == me { 100_000 } else { -100_000 };
    }
    let them = me.other();
    let race = position.pips(rules, them) as i32 - position.pips(rules, me) as i32;
    race * 4 + safety(rules, position, me) - safety(rules, position, them)
        + (position.borne_off(rules, me) as i32 - position.borne_off(rules, them) as i32) * 6
}

/// Points held, blots in danger, men sent back.
fn safety(rules: &Rules, position: &Position, side: Side) -> i32 {
    let mine = &position.men[side.index()];
    let mut v = 0;
    // Men in hand after the start are men that were hit: a cost beyond
    // the race.
    let entered = mine.iter().skip(1).map(|&n| n as i32).sum::<i32>();
    if entered > 0 {
        v -= mine[0] as i32 * 12;
    }
    let enemy = side.other();
    for s in 1..=rules.len {
        let n = mine[s as usize];
        if n >= 2 {
            v += 14;
        } else if n == 1 {
            // A blot: worse the more enemy men stand behind it, within a
            // throw's reach, along the enemy's way.
            let p = rules.point(side, s);
            let es = rules.step_of(enemy, p);
            let reach = 6 * rules.dice;
            let shooters: i32 = (es.saturating_sub(reach)..es)
                .map(|t| position.men[enemy.index()][t as usize] as i32)
                .sum();
            v -= 4 + shooters * 3 * s as i32 / rules.len as i32;
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_dice_throws() {
        let t = throws(3);
        assert_eq!(t.len(), 56);
        assert_eq!(t.iter().map(|(_, w)| w).sum::<u32>(), 216);
    }

    #[test]
    fn games_end() {
        let r = Rules::tabula();
        for (a, b) in [(Strength::Easy, Strength::Normal), (Strength::Hard, Strength::Normal)] {
            let mut p = Position::new(&r);
            let mut rng = Rng::new(9);
            let mut n = 0;
            while p.outcome(&r).is_none() {
                let dice: Vec<u8> = (0..r.dice).map(|_| (rng.next() % 6) as u8 + 1).collect();
                let s = if p.to_move == Side::White { a } else { b };
                match choose(&r, &p, &dice, s, &mut rng) {
                    Some(m) => p.play(&r, &m),
                    None => p.to_move = p.to_move.other(),
                }
                n += 1;
                assert!(n < 3000);
            }
        }
    }
}
