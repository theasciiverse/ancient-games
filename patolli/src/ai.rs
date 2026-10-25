//! The computer. Easy moves the stone furthest on, and now and then any
//! stone. Normal judges the position each move leaves: the race, the
//! purses (a forfeit is worth a few steps), another throw on a rounded
//! end, and the stones of the other side shut in behind its own. Hard
//! also weighs every throw the other side could make next and its best
//! answer.

pub use ancient_core::player::Strength;

use crate::rules::{Move, ODDS, Outcome, Position, Side};
use ancient_core::rng::Rng;

const BEGINNER_SLIPS: u64 = 4;

/// How `p` looks to `me`.
pub fn judge(p: &Position, me: Side) -> i32 {
    if let Some(Outcome::Home(w)) = p.outcome() {
        return if w == me { 100_000 } else { -100_000 };
    }
    let them = me.other();
    let race = p.to_go(them) as i32 - p.to_go(me) as i32;
    let money = p.purse[me.index()] as i32 - p.purse[them.index()] as i32;
    race * 10 + money * 6
}

/// The value of playing `mv` for the side to move: with another throw
/// to come on a rounded end, worth an average throw.
fn after(p: &Position, mv: &Move) -> (Position, bool) {
    let mut q = p.clone();
    let again = q.play(mv);
    (q, again)
}

pub fn choose(p: &Position, n: u8, strength: Strength, rng: &mut Rng) -> Option<Move> {
    let moves = p.moves(n);
    if moves.len() <= 1 {
        return moves.into_iter().next();
    }
    let me = p.to_move;
    match strength {
        Strength::Easy => {
            if rng.next() % 16 < BEGINNER_SLIPS {
                return Some(moves[(rng.next() % moves.len() as u64) as usize]);
            }
            moves.into_iter().max_by_key(|m| (m.to as i32, (rng.next() % 3) as i32))
        }
        Strength::Normal => moves.into_iter().max_by_key(|m| {
            let (q, again) = after(p, m);
            judge(&q, me) + if again { 50 } else { 0 } + (rng.next() % 3) as i32
        }),
        Strength::Hard => moves.into_iter().max_by_key(|m| {
            let (q, again) = after(p, m);
            if again || q.outcome().is_some() {
                return judge(&q, me) * 32 + 50 * 32;
            }
            // The other side's next throw, and its best answer by normal's
            // judgement.
            let mut r = q.clone();
            r.pass();
            let mut total = 0i32;
            for &(v, w) in &ODDS {
                let replies = r.moves(v);
                let worst = if replies.is_empty() {
                    let mut s = r.clone();
                    s.stuck(v);
                    judge(&s, me)
                } else {
                    replies
                        .iter()
                        .map(|mv| {
                            let (s, again) = after(&r, mv);
                            judge(&s, me) - if again { 50 } else { 0 }
                        })
                        .min()
                        .unwrap()
                };
                total += worst * w as i32;
            }
            total + (rng.next() % 3) as i32
        }),
    }
}

/// A whole game between two strengths; the outcome and the throws.
#[cfg(test)]
pub fn play_out(players: [Strength; 2], seed: u64) -> (Outcome, u32, Position) {
    let rules = crate::rules::Rules::bell();
    let mut rng = Rng::new(seed);
    let mut p = Position::new(&rules, if seed % 2 == 0 { Side::Red } else { Side::Blue });
    let mut throws = 0;
    loop {
        if let Some(o) = p.outcome() {
            return (o, throws, p);
        }
        let holes = (0..5).filter(|_| rng.next() % 2 == 1).count() as u8;
        let n = crate::rules::value(holes);
        throws += 1;
        match choose(&p, n, players[p.to_move.index()], &mut rng) {
            Some(mv) => {
                if !p.play(&mv) {
                    p.pass();
                }
            }
            None => {
                p.stuck(n);
                p.pass();
            }
        }
        assert!(throws < 20_000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn games_end() {
        let (_, throws, _) = play_out([Strength::Normal, Strength::Easy], 1);
        assert!(throws > 50);
    }

    /// `PATOLLI_GAMES=100 cargo test --release -p patolli strengths -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn strengths() {
        let games: u64 = std::env::var("PATOLLI_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(100);
        for (strong, weak) in
            [(Strength::Normal, Strength::Easy), (Strength::Hard, Strength::Easy), (Strength::Hard, Strength::Normal)]
        {
            let (mut won, mut purse) = (0, 0i64);
            for g in 0..games {
                let s = if g % 4 < 2 { Side::Red } else { Side::Blue };
                let mut players = [weak; 2];
                players[s.index()] = strong;
                let (o, _, p) = play_out(players, g + 1);
                let Outcome::Home(w) = o;
                won += (w == s) as u32;
                purse += p.purse[s.index()] as i64 - p.purse[s.other().index()] as i64;
            }
            eprintln!("{strong:?} beats {weak:?} {won}/{games}, ahead in the purse by {} a game", purse / games as i64);
        }
    }
}
