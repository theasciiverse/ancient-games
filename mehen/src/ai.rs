//! The computer opponent. Mehen is mostly a guessing game, so the heart of
//! it is how many marbles to hide and what to guess: a beginner picks at
//! random, a normal player mixes its choices as game theory says is safest,
//! and a hard one also watches what you tend to do and leans against it.

pub use ancient_core::player::Strength;

use crate::rules::{SQUARES, State, Team};
use ancient_core::rng::Rng;
use std::sync::OnceLock;

/// What a lion on a pharaoh is worth, in squares: it loses turns until a
/// one-in-three guess kills it, about two turns of three squares.
const LION: f64 = 6.0;

/// The hider's payoff when it hides `h` and the guesser says `g`: the hider
/// moves `h` on a wrong guess; on a right one the guesser moves `h` and may
/// set a lion on the hider.
fn payoff(h: usize, g: usize, lion: bool) -> f64 {
    let h_squares = (h + 1) as f64;
    if h == g { -(h_squares + if lion { LION } else { 0.0 }) } else { h_squares }
}

/// The safest mixes for hiding 1 to 5 and for guessing, found by fictitious
/// play (each side answering the other's history, which settles on the
/// equilibrium of a zero-sum game).
pub fn equilibrium(lion: bool) -> ([f64; 5], [f64; 5]) {
    static MIXES: OnceLock<[([f64; 5], [f64; 5]); 2]> = OnceLock::new();
    MIXES.get_or_init(|| [fictitious_play(false), fictitious_play(true)])[lion as usize]
}

fn fictitious_play(lion: bool) -> ([f64; 5], [f64; 5]) {
    let mut hides = [0u32; 5];
    let mut guesses = [0u32; 5];
    // What each hide has earned against the guesses so far, and what each
    // guess has given away against the hides.
    let mut hide_value = [0.0f64; 5];
    let mut guess_cost = [0.0f64; 5];
    for _ in 0..20_000 {
        let h = (0..5).max_by(|&a, &b| hide_value[a].total_cmp(&hide_value[b])).unwrap();
        let g = (0..5).min_by(|&a, &b| guess_cost[a].total_cmp(&guess_cost[b])).unwrap();
        hides[h] += 1;
        guesses[g] += 1;
        for i in 0..5 {
            hide_value[i] += payoff(i, g, lion);
            guess_cost[i] += payoff(h, i, lion);
        }
    }
    let total = hides.iter().sum::<u32>() as f64;
    (hides.map(|n| n as f64 / total), guesses.map(|n| n as f64 / total))
}

fn sample(weights: &[f64], rng: &mut Rng) -> usize {
    let total: f64 = weights.iter().sum();
    let mut x = (rng.next() % 1_000_000) as f64 / 1_000_000.0 * total;
    for (i, &w) in weights.iter().enumerate() {
        if x < w {
            return i;
        }
        x -= w;
    }
    weights.len() - 1
}

/// What one side has done so far: how often it hid or guessed each number.
#[derive(Clone, Debug, Default)]
pub struct Habits {
    pub hides: [u32; 5],
    pub guesses: [u32; 5],
}

/// Counts as a mix, with a few made-up even observations so that a short
/// history does not count for too much.
fn smoothed(counts: &[u32; 5]) -> [f64; 5] {
    let total = counts.iter().sum::<u32>() as f64 + 10.0;
    counts.map(|n| (n as f64 + 2.0) / total)
}

/// How many marbles to hide, 1 to 5. `lion`: whether a right guess lets the
/// other side set a lion on us; `them`: the guesser's habits.
pub fn hide(strength: Strength, lion: bool, them: &Habits, rng: &mut Rng) -> u8 {
    let (mix, _) = equilibrium(lion);
    let i = match strength {
        Strength::Easy => (rng.next() % 5) as usize,
        Strength::Normal => sample(&mix, rng),
        Strength::Hard => {
            if rng.next() % 2 == 0 || them.guesses.iter().sum::<u32>() < 4 {
                sample(&mix, rng)
            } else {
                let q = smoothed(&them.guesses);
                (0..5)
                    .max_by(|&a, &b| {
                        let va: f64 = (0..5).map(|g| q[g] * payoff(a, g, lion)).sum();
                        let vb: f64 = (0..5).map(|g| q[g] * payoff(b, g, lion)).sum();
                        va.total_cmp(&vb)
                    })
                    .unwrap()
            }
        }
    };
    i as u8 + 1
}

/// What to guess, 1 to 5; `them`: the hider's habits.
pub fn guess(strength: Strength, lion: bool, them: &Habits, rng: &mut Rng) -> u8 {
    let (_, mix) = equilibrium(lion);
    let i = match strength {
        Strength::Easy => (rng.next() % 5) as usize,
        Strength::Normal => sample(&mix, rng),
        Strength::Hard => {
            if rng.next() % 2 == 0 || them.hides.iter().sum::<u32>() < 4 {
                sample(&mix, rng)
            } else {
                let p = smoothed(&them.hides);
                (0..5)
                    .min_by(|&a, &b| {
                        let va: f64 = (0..5).map(|h| p[h] * payoff(h, a, lion)).sum();
                        let vb: f64 = (0..5).map(|h| p[h] * payoff(h, b, lion)).sum();
                        va.total_cmp(&vb)
                    })
                    .unwrap()
            }
        }
    };
    i as u8 + 1
}

/// The fight with a lion, 1 to 3 marbles: no number is better than another.
pub fn fight(rng: &mut Rng) -> u8 {
    1 + (rng.next() % 3) as u8
}

/// Which pharaoh to move: the furthest behind without a lion, since the
/// whole team must get home; if every one has a lion, the furthest behind.
pub fn mover(state: &State, team: Team) -> Option<usize> {
    let behind = |ps: Vec<usize>| ps.into_iter().min_by_key(|&p| state.pharaohs[p].pos);
    behind(state.free(team)).or_else(|| behind(state.travelling(team)))
}

/// After a right guess of `steps`: which of our free pharaohs moves. Going
/// home or landing on an opponent first, then the furthest behind.
pub fn guesser_piece(state: &State, team: Team, steps: u8) -> Option<usize> {
    state.free(team).into_iter().max_by_key(|&p| {
        let mut next = state.clone();
        let landing = next.advance(p, steps);
        let behind = (SQUARES - state.pharaohs[p].pos.min(SQUARES)) as i32;
        landing.home as i32 * 1000 + landing.swapped.is_some() as i32 * 500 + behind
    })
}

/// Whether to set a lion on the pharaoh that was guessed: always, but a
/// beginner sometimes forgets.
pub fn unleash(strength: Strength, rng: &mut Rng) -> bool {
    strength != Strength::Easy || rng.next() % 3 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mixes_are_sensible() {
        for lion in [false, true] {
            let (h, g) = equilibrium(lion);
            assert!((h.iter().sum::<f64>() - 1.0).abs() < 1e-6);
            // Big numbers are guessed more than small ones.
            assert!(g[4] > g[0], "{g:?}");
            // With a lion at stake every number gets hidden sometimes;
            // without one, hiding a single marble never pays.
            if lion {
                assert!(h.iter().all(|&x| x > 0.05), "{h:?}");
            } else {
                assert!(h[0] < 0.01, "{h:?}");
            }
        }
    }

    #[test]
    fn a_hard_guesser_catches_a_habit() {
        let mut rng = Rng::new(4);
        let habits = Habits { hides: [0, 0, 0, 0, 40], guesses: [0; 5] };
        let fives = (0..200).filter(|_| guess(Strength::Hard, true, &habits, &mut rng) == 5).count();
        assert!(fives > 110, "{fives}");
    }
}
