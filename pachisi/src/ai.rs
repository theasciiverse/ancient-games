//! The computer: it plans the use of all its throws together, trying
//! every order and every man (the same position reached twice is tried
//! once), and keeps the plan that leaves the best position. Easy moves
//! its men forward and takes when it can, and now and then plays any
//! move; normal judges a position by the race, men in safety and men
//! within reach of an enemy; hard reckons the chance that each exposed
//! man is hit by the next throws.

pub use ancient_core::player::Strength;

use crate::rules::{HOME, Move, Position, READY, Rules, Throws, Token, loop_square, team};
use ancient_core::rng::Rng;
use std::collections::HashMap;

const BEGINNER_SLIPS: u64 = 3;

/// Plans already searched: the position and the throws left, the value
/// and the plan.
type Memo = HashMap<(Position, Vec<Token>), (i32, Vec<Move>)>;

/// The chance, in parts of 4096, that one man of the side to move can be
/// brought exactly `d` steps (1 to 60) by some of the throws of one whole
/// turn: worked out once per kind of throw by playing many turns.
fn reach(rules: &Rules) -> &'static [u32; 61] {
    use std::sync::OnceLock;
    static TABLES: [OnceLock<[u32; 61]>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let k = match rules.throws {
        Throws::TylorCowries => 0,
        Throws::FalkenerCowries => 1,
        Throws::LongDice => 2,
    };
    TABLES[k].get_or_init(|| {
        const TURNS: u32 = 20_000;
        let mut rng = Rng::new(7);
        let mut hits = [0u32; 61];
        for _ in 0..TURNS {
            let mut turn = crate::turn::Turn::new();
            while turn.add(rules, &crate::rules::throw(rules, || rng.next())) {}
            let values: Vec<u8> = crate::rules::tokens(rules, &turn.throws).iter().map(|t| t.value).collect();
            // The distances a subset of the values adds up to.
            let mut can = [false; 61];
            can[0] = true;
            for v in values {
                for d in (v as usize..=60).rev() {
                    can[d] |= can[d - v as usize];
                }
            }
            for d in 1..=60 {
                hits[d] += can[d] as u32;
            }
        }
        hits.map(|h| (h as u64 * 4096 / TURNS as u64) as u32)
    })
}

/// How good `p` is for `team` (higher better).
pub fn judge(rules: &Rules, p: &Position, us: usize, strength: Strength) -> i32 {
    if let Some(w) = p.winner() {
        return if w == us { 1_000_000 } else { -1_000_000 };
    }
    let odds = reach(rules);
    let _ = &odds;
    let mut score = 0i32;
    for c in 0..4 {
        let sign = if team(c) == us { 1 } else { -1 };
        let mut v = -(p.to_go(c) as i32) * 10;
        for (m, &s) in p.men[c].iter().enumerate() {
            if s == HOME {
                // Safe for good, and worth more than safe on the way.
                v += 25;
                continue;
            }
            if s == 0 || s == READY {
                continue;
            }
            let Some(sq) = loop_square(c, s) else {
                // On its own middle row: safe from everyone.
                v += 15;
                continue;
            };
            if rules.castle(sq) {
                v += 20;
                continue;
            }
            let together = p.men[c].iter().filter(|&&q| q == s).count();
            if together >= 2 && rules.doubled == crate::rules::Doubled::Hold {
                v += 12;
                continue;
            }
            let _ = m;
            // Enemies behind it within a throw's reach: for hard, the
            // chance that none of them is brought onto it.
            let mut danger = 0u32;
            let mut safe = 4096u64;
            for e in 0..4 {
                if team(e) == team(c) {
                    continue;
                }
                for &es in &p.men[e] {
                    let Some(esq) = loop_square(e, es) else { continue };
                    let d = ((sq as i32 - esq as i32).rem_euclid(68)) as u8;
                    if d == 0 || d > 37 {
                        continue;
                    }
                    match strength {
                        Strength::Hard if (d as usize) < odds.len() => {
                            safe = safe * (4096 - odds[d as usize] as u64) / 4096;
                        }
                        Strength::Hard => {}
                        _ => danger += if d <= 25 { 400 } else { 0 },
                    }
                }
            }
            if strength == Strength::Hard {
                danger = 4096 - safe as u32;
            }
            // The steps it would lose, times the chance.
            v -= ((s as u32 * 10 + 60) * danger.min(4096) / 4096) as i32;
        }
        score += sign * v;
    }
    score
}

/// The best plan for the colour to move with `tokens` to hand: moves in
/// order. Captures bring more throws, worth something of their own.
pub fn plan(rules: &Rules, p: &Position, tokens: &[Token], strength: Strength, rng: &mut Rng) -> Vec<Move> {
    let us = team(p.to_move);
    if strength == Strength::Easy {
        let mut out = Vec::new();
        let mut q = p.clone();
        let mut toks = tokens.to_vec();
        loop {
            let moves = q.moves(rules, &toks);
            if moves.is_empty() {
                break;
            }
            let mv = if rng.next() % 16 < BEGINNER_SLIPS {
                moves[(rng.next() % moves.len() as u64) as usize].clone()
            } else {
                moves
                    .iter()
                    .max_by_key(|m| (m.takes.len() * 1000) as i64 + m.to as i64 * (m.men.len() as i64) + (rng.next() % 5) as i64)
                    .unwrap()
                    .clone()
            };
            q.apply(&mv);
            crate::rules::remove_tokens(&mut toks, &mv.tokens);
            out.push(mv);
        }
        return out;
    }
    let mut memo: Memo = HashMap::new();
    if strength == Strength::Normal {
        let (_, best) = search(rules, p, tokens.to_vec(), us, strength, &mut memo, 0);
        return best;
    }
    // Hard: every place the throws can bring the men to, the best few
    // by normal's judgement, each weighed over the next colour's turn
    // played out (the same throws for every candidate).
    let mut ends: HashMap<Position, (usize, Vec<Move>)> = HashMap::new();
    leaves(rules, p, tokens.to_vec(), Vec::new(), &mut ends);
    // Ties ranked in the order the positions were found, not the map's
    // order, which changes from run to run.
    let mut ranked: Vec<(i32, usize, Position, Vec<Move>)> = ends
        .into_iter()
        .map(|(q, (found, plan))| {
            let taken: i32 = plan.iter().map(|m| (!m.takes.is_empty()) as i32 * 60).sum();
            (judge(rules, &q, us, Strength::Normal) + taken, found, q, plan)
        })
        .collect();
    ranked.sort_by_key(|(v, found, _, _)| (-v, *found));
    ranked.truncate(if ancient_core::capture::untimed() { 3 } else { 6 });
    if ranked.len() == 1 {
        return ranked.pop().map(|(_, _, _, plan)| plan).unwrap_or_default();
    }
    let samples = if ancient_core::capture::untimed() { 6 } else { 24 };
    let seed = rng.next();
    let mut best = (i64::MIN, Vec::new());
    for (_, _, q, plan) in ranked {
        let mut total = 0i64;
        let mut sim = Rng::new(seed);
        for _ in 0..samples {
            let mut r = q.clone();
            r.pass();
            let mut turn = crate::turn::Turn::new();
            while turn.add(rules, &crate::rules::throw(rules, || sim.next())) {}
            let (toks, _) = turn.left(rules);
            let mut m2 = HashMap::new();
            let (_, reply) = search(rules, &r, toks, team(r.to_move), Strength::Normal, &mut m2, 0);
            for mv in &reply {
                r.apply(mv);
            }
            total += judge(rules, &r, us, Strength::Normal) as i64;
        }
        if total > best.0 {
            best = (total, plan);
        }
    }
    best.1
}

/// Every position the throws can bring the colour to move to, with one
/// plan that gets there.
fn leaves(rules: &Rules, p: &Position, toks: Vec<Token>, done: Vec<Move>, out: &mut HashMap<Position, (usize, Vec<Move>)>) {
    if out.contains_key(p) || out.len() > 20_000 {
        return;
    }
    out.insert(p.clone(), (out.len(), done.clone()));
    for mv in p.moves(rules, &toks) {
        let mut q = p.clone();
        q.apply(&mv);
        let mut rest = toks.clone();
        crate::rules::remove_tokens(&mut rest, &mv.tokens);
        let mut plan = done.clone();
        plan.push(mv);
        leaves(rules, &q, rest, plan, out);
    }
}

fn search(
    rules: &Rules,
    p: &Position,
    toks: Vec<Token>,
    us: usize,
    strength: Strength,
    memo: &mut Memo,
    bonus: i32,
) -> (i32, Vec<Move>) {
    let mut sorted = toks.clone();
    sorted.sort_by_key(|t| (t.value, t.enters));
    let key = (p.clone(), sorted);
    if let Some((v, plan)) = memo.get(&key) {
        return (*v + bonus, plan.clone());
    }
    // Stopping here is always allowed (a throw may be left unused).
    let mut best = (judge(rules, p, us, strength), Vec::new());
    if memo.len() < 40_000 {
        for mv in p.moves(rules, &toks) {
            let mut q = p.clone();
            q.apply(&mv);
            let mut rest = toks.clone();
            crate::rules::remove_tokens(&mut rest, &mv.tokens);
            let extra = if rules.capture_throws && !mv.takes.is_empty() { 60 } else { 0 };
            let (v, mut rest_plan) = search(rules, &q, rest, us, strength, memo, extra);
            if v > best.0 {
                rest_plan.insert(0, mv);
                best = (v, rest_plan);
            }
        }
    }
    memo.insert(key, best.clone());
    (best.0 + bonus, best.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Throw, throw, tokens};
    use crate::turn::Turn;

    /// Plays a whole game between two strengths; returns the winning team
    /// and the turns it took.
    pub fn play_out(rules: &Rules, players: [Strength; 2], seed: u64) -> (usize, u32) {
        let mut p = Position::new(rules);
        let mut rng = Rng::new(seed);
        let mut turns = 0;
        loop {
            let mut turn = Turn::new();
            loop {
                let t: Vec<Throw> = throw(rules, || rng.next());
                if !turn.add(rules, &t) {
                    break;
                }
            }
            loop {
                let (toks, _) = turn.left(rules);
                let s = players[team(p.to_move)];
                let plan = plan(rules, &p, &toks, s, &mut rng);
                let mut taken = 0;
                for mv in &plan {
                    p.apply(mv);
                    taken += !mv.takes.is_empty() as u8;
                }
                if let Some(w) = p.winner() {
                    return (w, turns);
                }
                if !rules.capture_throws || taken == 0 {
                    break;
                }
                turn.clear();
                for _ in 0..taken {
                    loop {
                        let t = throw(rules, || rng.next());
                        if !turn.add(rules, &t) {
                            break;
                        }
                    }
                }
                let _ = tokens(rules, &turn.throws);
            }
            p.pass();
            turns += 1;
            assert!(turns < 5000, "{} does not end: {:?}", rules.id, p.men);
        }
    }

    #[test]
    fn games_end() {
        for rules in Rules::all() {
            let (_, turns) = play_out(&rules, [Strength::Easy, Strength::Normal], 2);
            assert!(turns > 10);
        }
    }

    /// `PACHISI_GAMES=20 cargo test --release -p pachisi strengths -- --ignored --nocapture`
    /// (`PACHISI_ONLY=<id>` one rule set, `PACHISI_HARD=1` only hard against normal)
    #[test]
    #[ignore]
    fn strengths() {
        let games: u64 = std::env::var("PACHISI_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(20);
        let only = std::env::var("PACHISI_ONLY").ok();
        let hard = std::env::var("PACHISI_HARD").is_ok();
        for rules in Rules::all().into_iter().filter(|r| only.as_deref().is_none_or(|o| o == r.id)) {
            let pairs = if hard {
                vec![(Strength::Hard, Strength::Normal)]
            } else {
                vec![(Strength::Normal, Strength::Easy), (Strength::Hard, Strength::Easy), (Strength::Hard, Strength::Normal)]
            };
            for (strong, weak) in pairs {
                let mut won = 0;
                for g in 0..games {
                    let s = (g % 2) as usize;
                    let mut players = [weak; 2];
                    players[s] = strong;
                    let (w, _) = play_out(&rules, players, g + 1);
                    won += (w == s) as u32;
                }
                eprintln!("{}: {:?} beats {:?} {won}/{games}", rules.id, strong, weak);
            }
        }
    }
}

#[cfg(test)]
mod debug {
    use super::*;
    use crate::rules::throw;
    use crate::turn::Turn;

    #[test]
    #[ignore]
    fn watch() {
        let rules = Rules::tylor();
        for seed in 1..6 {
            for pair in [[Strength::Normal, Strength::Easy], [Strength::Easy, Strength::Normal]] {
                let mut p = Position::new(&rules);
                let mut rng = Rng::new(seed);
                let mut ended = false;
                for _ in 0..5000 {
                    let mut turn = Turn::new();
                    while turn.add(&rules, &throw(&rules, || rng.next())) {}
                    let (toks, _) = turn.left(&rules);
                    let plan = plan(&rules, &p, &toks, pair[team(p.to_move)], &mut rng);
                    for mv in &plan {
                        p.apply(mv);
                    }
                    if p.winner().is_some() {
                        ended = true;
                        break;
                    }
                    p.pass();
                }
                if !ended {
                    eprintln!("seed {seed} {pair:?}: stuck at {:?}", p.men);
                }
            }
        }
    }
}
