//! The computer opponent: alpha-beta search with iterative deepening, a
//! table of positions already searched, and captures searched on to quiet
//! positions. A position is judged by its men (a rukh is worth most, then
//! the faras; the firzan and the fil are weak), the baidaqs' advance, and
//! how freely the men move. With few men left it also drives the other
//! shah to the edge and takes men, since a bare shah loses.

pub use ancient_core::player::Strength;

use crate::rules::{Kind, Move, Outcome, Position, Rules, Side, fr};
use ancient_core::rng::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 3;

const WIN: i32 = 1_000_000;

/// What returning to a position already seen costs, in the mover's eyes.
const REPEAT: i32 = 30;

fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(80), 1),
        // Even depths: an odd depth ends on one's own move and is too
        // hopeful. Measured: deeper than 4 did not play better.
        Strength::Normal => (Duration::from_millis(400), 2),
        Strength::Hard => (Duration::from_millis(1500), 4),
    };
    if ancient_core::capture::untimed() { (Duration::MAX, depth.min(2)) } else { (time, depth) }
}

pub fn value(kind: Kind) -> i32 {
    match kind {
        Kind::Shah => 0,
        Kind::Firzan => 180,
        Kind::Fil => 130,
        Kind::Faras => 330,
        Kind::Rukh => 520,
        Kind::Baidaq => 100,
    }
}

struct Search<'a> {
    rules: &'a Rules,
    start: Instant,
    time: Duration,
    table: HashMap<Position, (u32, i32, Option<Move>)>,
    seen: &'a HashMap<Position, u32>,
    nodes: u64,
}

impl Search<'_> {
    fn out(&mut self) -> bool {
        self.nodes += 1;
        self.nodes.is_multiple_of(512) && self.start.elapsed() > self.time
    }
}

/// The best move for the side to move. `seen` counts the positions of the
/// game so far: coming back to one is a little worse.
pub fn choose(
    rules: &Rules,
    position: &Position,
    strength: Strength,
    seen: &HashMap<Position, u32>,
    rng: &mut Rng,
) -> Option<Move> {
    let mut moves = position.moves();
    if moves.len() <= 1 {
        return moves.into_iter().next();
    }
    order(position, &mut moves, None);
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        let i = (rng.next() % moves.len() as u64) as usize;
        return Some(moves[i]);
    }
    let (time, max_depth) = budget(strength);
    let mut search = Search { rules, start: Instant::now(), time, table: HashMap::new(), seen, nodes: 0 };
    let mut best = moves[0];
    // Hard keeps the move of its last finished even depth: an odd depth
    // ends on its own move and is too hopeful.
    let mut kept = None;
    for depth in 1..=max_depth {
        if let Some(i) = moves.iter().position(|m| *m == best) {
            moves[..=i].rotate_right(1);
        }
        let mut alpha = -WIN * 2;
        let mut found: Option<(i32, Move)> = None;
        let mut out_of_time = false;
        for mv in &moves {
            let mut next = position.clone();
            next.play(rules, mv);
            let mut v = -negamax(&mut search, &next, depth - 1, 1, -WIN * 2, -(alpha - 1));
            if seen.get(&next).is_some_and(|&n| n > 0) {
                v -= REPEAT;
            }
            if search.start.elapsed() > time && depth > 1 {
                out_of_time = true;
                break;
            }
            let v = v * 4 + (rng.next() % 3) as i32;
            if found.is_none_or(|(b, _)| v > b) {
                found = Some((v, *mv));
                alpha = alpha.max(v / 4);
            }
        }
        if out_of_time {
            break;
        }
        if let Some((v, mv)) = found {
            best = mv;
            if depth % 2 == 0 {
                kept = Some(mv);
            }
            if (v / 4).abs() >= WIN - 1000 {
                kept = Some(mv);
                break;
            }
        }
        if search.start.elapsed() > time {
            break;
        }
    }
    if strength == Strength::Hard { Some(kept.unwrap_or(best)) } else { Some(best) }
}

/// Captures first, the most valuable man taken by the least valuable;
/// the move the table remembers before all.
fn order(position: &Position, moves: &mut [Move], first: Option<Move>) {
    moves.sort_by_key(|m| {
        if Some(*m) == first {
            return i32::MIN;
        }
        let taken = position.board[m.to as usize].map_or(0, |t| value(t.kind) * 10);
        let by = position.board[m.from as usize].map_or(0, |t| value(t.kind));
        let promote = if m.promotes { 500 } else { 0 };
        -(taken - if taken > 0 { by } else { 0 } + promote)
    });
}

fn terminal(rules: &Rules, position: &Position, moves: &[Move], ply: u32) -> Option<i32> {
    position.outcome(rules, moves).map(|o| match o {
        Outcome::Win(side, _) => {
            let v = WIN - 100 - ply as i32;
            if side == position.to_move { v } else { -v }
        }
        Outcome::Draw(_) => 0,
    })
}

fn negamax(s: &mut Search, position: &Position, depth: u32, ply: u32, mut alpha: i32, beta: i32) -> i32 {
    let mut moves = position.moves();
    if let Some(v) = terminal(s.rules, position, &moves, ply) {
        return v;
    }
    if s.seen.get(position).is_some_and(|&n| n >= 2) {
        return 0;
    }
    if depth == 0 || s.out() {
        return quiesce(s, position, ply, alpha, beta, 6);
    }
    let first = match s.table.get(position) {
        Some(&(d, v, mv)) => {
            if d >= depth && v.abs() < WIN - 1000 {
                // Only exact values are kept (below), so a deep enough
                // entry answers outright.
                return v;
            }
            mv
        }
        None => None,
    };
    order(position, &mut moves, first);
    let mut best = -WIN * 2;
    let mut best_move = None;
    let alpha0 = alpha;
    for mv in &moves {
        let mut next = position.clone();
        next.play(s.rules, mv);
        let v = -negamax(s, &next, depth - 1, ply + 1, -beta, -alpha);
        if v > best {
            best = v;
            best_move = Some(*mv);
        }
        alpha = alpha.max(v);
        if alpha >= beta {
            break;
        }
    }
    if best > alpha0 && best < beta {
        s.table.insert(position.clone(), (depth, best, best_move));
    } else if let Some(e) = s.table.get_mut(position) {
        e.2 = best_move;
    } else {
        s.table.insert(position.clone(), (0, 0, best_move));
    }
    best
}

/// Captures only, until the position is quiet; a shah in check tries
/// every way out instead.
fn quiesce(s: &mut Search, position: &Position, ply: u32, mut alpha: i32, beta: i32, left: u32) -> i32 {
    let check = position.in_check(position.to_move);
    let stand = judge(s.rules, position);
    if !check && (stand >= beta || left == 0) {
        return stand;
    }
    if !check {
        alpha = alpha.max(stand);
    }
    let mut moves: Vec<Move> = position
        .moves()
        .into_iter()
        .filter(|m| check || position.board[m.to as usize].is_some() || m.promotes)
        .collect();
    if check && left == 0 {
        return stand;
    }
    order(position, &mut moves, None);
    for mv in &moves {
        let mut next = position.clone();
        next.play(s.rules, mv);
        let after = next.moves();
        let v = match terminal(s.rules, &next, &after, ply + 1) {
            Some(v) => -v,
            None => -quiesce(s, &next, ply + 1, -beta, -alpha, left - 1),
        };
        if v >= beta {
            return v;
        }
        alpha = alpha.max(v);
    }
    if check && alpha == -WIN * 2 { stand } else { alpha }
}

/// Squares the faras or rukh on `sq` could move to.
fn mobility(position: &Position, sq: usize) -> i32 {
    let Some(m) = position.board[sq] else { return 0 };
    let (f, r) = fr(sq);
    let free = |s: usize| position.board[s].is_none_or(|o| o.side != m.side);
    match m.kind {
        Kind::Faras => [(1, 2), (2, 1), (-1, 2), (-2, 1), (1, -2), (2, -1), (-1, -2), (-2, -1)]
            .iter()
            .filter(|&&(df, dr)| crate::rules::square(f + df, r + dr).is_some_and(free))
            .count() as i32,
        Kind::Rukh => {
            let mut n = 0;
            for (df, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (mut x, mut y) = (f + df, r + dr);
                while let Some(s) = crate::rules::square(x, y) {
                    if position.board[s].is_some() {
                        n += free(s) as i32;
                        break;
                    }
                    n += 1;
                    x += df;
                    y += dr;
                }
            }
            n
        }
        _ => 0,
    }
}

/// The value of `position` for the side to move.
pub fn judge(_rules: &Rules, position: &Position) -> i32 {
    let me = position.to_move;
    let mut score = [0i32; 2];
    let mut men = [0i32; 2];
    for sq in 0..64 {
        let Some(m) = position.board[sq] else {
            continue;
        };
        let i = m.side.index();
        let (f, r) = fr(sq);
        score[i] += value(m.kind);
        if m.kind != Kind::Shah {
            men[i] += 1;
        }
        // Baidaqs gain as they go on; men like the middle of the board,
        // and the rukh and faras like room to move.
        let advance = if m.side == Side::Red { r } else { 7 - r } as i32;
        let central = 3 - ((2 * f as i32 - 7).abs().max((2 * r as i32 - 7).abs()) - 1) / 2;
        score[i] += match m.kind {
            // Up to 60 on the seventh rank: still less than the firzan it
            // is about to become.
            Kind::Baidaq => advance * 10,
            Kind::Faras => central * 10 + mobility(position, sq) * 4,
            Kind::Rukh => mobility(position, sq) * 3,
            Kind::Firzan | Kind::Fil => central * 6,
            Kind::Shah => 0,
        };
    }
    let (a, b) = (me.index(), me.other().index());
    let mut v = score[a] - score[b];
    // The ending: with more men, drive the other shah to the edge and
    // keep one's own near it, and take men, for a bare shah loses.
    if men[a] + men[b] <= 6 && men[a] != men[b] {
        let ahead = if men[a] > men[b] { me } else { me.other() };
        if let (Some(k), Some(theirs)) = (position.shah(ahead), position.shah(ahead.other())) {
            let (tf, tr) = fr(theirs);
            let edge = (2 * tf as i32 - 7).abs().max((2 * tr as i32 - 7).abs());
            let (kf, kr) = fr(k);
            let near = (kf - tf).abs().max((kr - tr).abs()) as i32;
            let push = edge * 6 - near * 4 + (6 - men[ahead.other().index()]) * 40;
            v += if ahead == me { push } else { -push };
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::tests::place;
    use crate::rules::{name, parse};

    #[test]
    fn takes_a_rukh_left_hanging() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        place(&mut p, Side::Red, &[("a1", Kind::Shah), ("c3", Kind::Faras)]);
        place(&mut p, Side::Black, &[("h8", Kind::Shah), ("d5", Kind::Rukh), ("h7", Kind::Baidaq)]);
        let mv = choose(&r, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert_eq!(name(mv.to as usize), "d5");
    }

    #[test]
    fn mates_in_one() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        // Rukh to a8 mates: the black shah on g8 is shut in by its baidaqs.
        place(&mut p, Side::Red, &[("g1", Kind::Shah), ("a1", Kind::Rukh)]);
        place(&mut p, Side::Black, &[("g8", Kind::Shah), ("f7", Kind::Baidaq), ("g7", Kind::Baidaq), ("h7", Kind::Baidaq)]);
        let mv = choose(&r, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert_eq!((mv.from as usize, mv.to as usize), (parse("a1").unwrap(), parse("a8").unwrap()));
    }
}

/// How a game between two strengths ends, for the tests.
#[cfg(test)]
pub fn play_out(rules: &Rules, players: [Strength; 2], seed: u64) -> (Option<Outcome>, usize) {
    let mut p = Position::new(rules);
    let mut rng = Rng::new(seed);
    let mut seen: HashMap<Position, u32> = HashMap::new();
    let mut plies = 0;
    loop {
        let moves = p.moves();
        if let Some(o) = p.outcome(rules, &moves) {
            return (Some(o), plies);
        }
        let n = seen.entry(p.clone()).or_insert(0);
        *n += 1;
        if *n >= 3 {
            return (Some(Outcome::Draw(crate::rules::Why::Repetition)), plies);
        }
        // Four random moves open each test game, so that games differ.
        let mv = if plies < 4 {
            moves[(rng.next() % moves.len() as u64) as usize]
        } else {
            choose(rules, &p, players[p.to_move.index()], &seen, &mut rng).unwrap()
        };
        p.play(rules, &mv);
        plies += 1;
    }
}

/// Strength against strength, sides alternating. Slow:
/// `SHATRANJ_GAMES=10 cargo test --release -p shatranj strengths -- --ignored --nocapture`
/// (`SHATRANJ_ONLY=<id>` for one rule set, `SHATRANJ_HARD=1` only hard
/// against normal).
#[cfg(test)]
mod strength_tests {
    use super::*;

    #[test]
    #[ignore]
    fn strengths() {
        let games: u64 = std::env::var("SHATRANJ_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        let only = std::env::var("SHATRANJ_ONLY").ok();
        for rules in Rules::all().into_iter().filter(|r| only.as_deref().is_none_or(|o| o == r.id)) {
            let pairs = if std::env::var("SHATRANJ_HARD").is_ok() {
                vec![(Strength::Hard, Strength::Normal)]
            } else {
                vec![(Strength::Normal, Strength::Easy), (Strength::Hard, Strength::Easy), (Strength::Hard, Strength::Normal)]
            };
            for (strong, weak) in pairs {
                let (mut won, mut drawn) = (0, 0);
                for g in 0..games {
                    let strong_side = if g % 2 == 0 { Side::Red } else { Side::Black };
                    let mut players = [weak; 2];
                    players[strong_side.index()] = strong;
                    let (outcome, plies) = play_out(&rules, players, g + 1);
                    match outcome {
                        Some(Outcome::Win(s, _)) if s == strong_side => won += 1,
                        Some(Outcome::Draw(_)) => drawn += 1,
                        _ => {}
                    }
                    eprintln!("  {} {:?} as {:?}: {:?} in {plies} plies", rules.id, strong, strong_side, outcome);
                }
                eprintln!("{}: {:?} beats {:?} {won}/{games}, {drawn} drawn", rules.id, strong, weak);
            }
        }
    }
}

#[cfg(test)]
mod hard_probe {
    use super::*;
    use crate::rules::tests::place;
    use crate::rules::{name, parse};

    #[test]
    fn hard_mates_in_one_and_sees_the_threat() {
        let r = Rules::shatranj();
        let mut p = Position::empty();
        place(&mut p, Side::Red, &[("g1", Kind::Shah), ("a1", Kind::Rukh)]);
        place(&mut p, Side::Black, &[("g8", Kind::Shah), ("f7", Kind::Baidaq), ("g7", Kind::Baidaq), ("h7", Kind::Baidaq)]);
        for s in [Strength::Normal, Strength::Hard] {
            let mv = choose(&r, &p, s, &HashMap::new(), &mut Rng::new(1)).unwrap();
            assert_eq!(name(mv.to as usize), "a8", "{s:?} mates");
        }
        // Black to move must stop Ra1-a8 mate: his rukh on b2 goes to b8,
        // or a baidaq makes room.
        let mut q = p.clone();
        place(&mut q, Side::Black, &[("b2", Kind::Rukh)]);
        q.to_move = Side::Black;
        for s in [Strength::Normal, Strength::Hard] {
            let mv = choose(&r, &q, s, &HashMap::new(), &mut Rng::new(1)).unwrap();
            let mut after = q.clone();
            after.play(&r, &mv);
            let mate = after.moves().into_iter().any(|m| {
                let mut x = after.clone();
                x.play(&r, &m);
                let ms = x.moves();
                ms.is_empty() && x.in_check(x.to_move)
            });
            assert!(!mate, "{s:?} played {}-{} and allows mate", name(mv.from as usize), name(mv.to as usize));
        }
        let _ = parse("a1");
    }
}
