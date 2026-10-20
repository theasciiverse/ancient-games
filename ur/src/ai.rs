//! The computer opponent: expectimax over the throws, looking a number of
//! turns ahead, with a judgement of positions at the end.

use crate::rules::{HOME, Move, PATH, Position, Rules, START, Side, is_shared};

/// How hard the computer plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strength {
    /// A beginner: advances and takes pieces, never thinks about danger.
    Easy,
    /// Weighs progress, safety and danger for the move at hand.
    Normal,
    /// Also looks at the throws and replies of the next two turns.
    Hard,
}

impl Strength {
    pub fn name(self) -> &'static str {
        match self {
            Strength::Easy => "easy",
            Strength::Normal => "normal",
            Strength::Hard => "hard",
        }
    }

    /// Turns of throws looked ahead after the move.
    fn depth(self) -> u32 {
        match self {
            Strength::Easy | Strength::Normal => 0,
            Strength::Hard => 2,
        }
    }
}

/// The best move for the side to move with throw `roll`, or `None` if it
/// has none.
pub fn choose(rules: &Rules, position: &Position, roll: u8, strength: Strength) -> Option<Move> {
    let side = position.to_move;
    let moves = position.moves(rules, roll);
    if strength == Strength::Easy {
        return naive(rules, position, &moves);
    }
    let mut best: Option<(f64, Move)> = None;
    for mv in moves {
        let mut next = position.clone();
        next.play(rules, mv);
        let value = value_after(rules, &next, side, strength.depth(), FREE_AGAINS);
        if best.is_none_or(|(v, _)| value > v) {
            best = Some((value, mv));
        }
    }
    best.map(|(_, mv)| mv)
}

/// A beginner's move: take a piece if possible, else land on a rosette,
/// else bring the furthest piece on.
fn naive(rules: &Rules, position: &Position, moves: &[Move]) -> Option<Move> {
    let side = position.to_move;
    let takes = |mv: &Move| is_shared(mv.to) && position.of(side.other()).contains(&mv.to);
    moves
        .iter()
        .copied()
        .max_by_key(|mv| {
            let score = if takes(mv) {
                3
            } else if mv.to != HOME && rules.is_rosette_pos(mv.to) {
                2
            } else {
                1
            };
            (score, mv.from)
        })
}

/// Throws again that do not use up a turn of lookahead, in a row. Without a
/// limit, chains of rosettes multiply the search beyond any time.
const FREE_AGAINS: u32 = 2;

/// The expected value of `position` for `me`, with `depth` more turns of
/// throws to look at; a throw again is free up to `agains` times in a row.
fn value_after(rules: &Rules, position: &Position, me: Side, depth: u32, agains: u32) -> f64 {
    if let Some(winner) = position.winner() {
        return if winner == me { 10_000.0 } else { -10_000.0 };
    }
    if depth == 0 {
        return judge(rules, position, me);
    }

    let total: u32 = rules.lots.distribution().iter().map(|&(_, n)| n).sum();
    let mut expected = 0.0;
    for (roll, chances) in rules.lots.distribution() {
        let p = chances as f64 / total as f64;
        let mover = position.to_move;
        let moves = position.moves(rules, roll);
        let value = if moves.is_empty() {
            let mut next = position.clone();
            next.pass();
            value_after(rules, &next, me, depth - 1, FREE_AGAINS)
        } else {
            let values = moves.into_iter().map(|mv| {
                let mut next = position.clone();
                let out = next.play(rules, mv);
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
        expected += p * value;
    }
    expected
}

/// How good `position` looks for `me`: progress along the path, pieces home,
/// safety, and the danger pieces on the shared row are in.
fn judge(rules: &Rules, position: &Position, me: Side) -> f64 {
    side_value(rules, position, me) - side_value(rules, position, me.other())
}

fn side_value(rules: &Rules, position: &Position, side: Side) -> f64 {
    let dist = rules.lots.distribution();
    let total: u32 = dist.iter().map(|&(_, n)| n).sum();
    let chance_of = |steps: u8| -> f64 {
        dist.iter()
            .filter(|&&(m, _)| m == steps)
            .map(|&(_, n)| n as f64 / total as f64)
            .sum()
    };

    let theirs = position.of(side.other());
    let mut value = 0.0;
    for &pos in position.of(side) {
        if pos == HOME {
            value += PATH as f64 + 6.0;
            continue;
        }
        if pos == START {
            continue;
        }
        value += pos as f64;
        if rules.is_rosette_pos(pos) {
            value += 1.5;
        }
        // On the shared row and not safe: weigh the chance of being taken
        // by an opposing piece behind it (on its own row or the shared row).
        if is_shared(pos) && !rules.is_safe_pos(pos) {
            let mut risk = 0.0;
            for &q in theirs {
                if q != START && q != HOME && q < pos && pos - q <= 4 {
                    risk += chance_of(pos - q);
                }
            }
            value -= risk.min(1.0) * pos as f64 * 0.8;
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rules;

    #[test]
    fn takes_a_piece_when_it_can() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = vec![0, 0, 0, 0, 0, 1, 7];
        p.pieces[1] = vec![0, 0, 0, 0, 0, 0, 9];
        for strength in [Strength::Easy, Strength::Normal, Strength::Hard] {
            assert_eq!(
                choose(&rules, &p, 2, strength),
                Some(Move { from: 7, to: 9 }),
                "{strength:?}"
            );
        }
    }

    #[test]
    fn bears_off_rather_than_wander() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = vec![0, 13, 15, 15, 15, 15, 15];
        p.pieces[1] = vec![0, 0, 0, 0, 0, 0, 0];
        // A two takes the last-but-one piece home.
        assert_eq!(
            choose(&rules, &p, 2, Strength::Normal),
            Some(Move { from: 13, to: 15 })
        );
    }

    /// The hard computer beats the easy one clearly over many games.
    #[test]
    fn hard_beats_easy() {
        let rules = Rules::finkel_2017();
        let mut seed = 12345u64;
        let mut roll = || {
            let mut marked = 0;
            for _ in 0..4 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                marked += (seed & 1) as u32;
            }
            rules.lots.moves(marked)
        };
        let mut hard_wins = 0;
        let games = 40;
        for g in 0..games {
            let first = if g % 2 == 0 { Side::Light } else { Side::Dark };
            let mut p = Position::new(&rules, first);
            while p.winner().is_none() {
                let strength = if p.to_move == Side::Light {
                    Strength::Hard
                } else {
                    Strength::Easy
                };
                let r = roll();
                match choose(&rules, &p, r, strength) {
                    Some(mv) => {
                        p.play(&rules, mv);
                    }
                    None => p.pass(),
                }
            }
            if p.winner() == Some(Side::Light) {
                hard_wins += 1;
            }
        }
        assert!(hard_wins > games * 6 / 10, "hard won only {hard_wins} of {games}");
    }
}

#[cfg(test)]
mod measure {
    use super::*;
    use crate::rules::Rules;

    fn rng(seed: &mut u64) -> u64 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        *seed
    }

    /// `a` plays Light against `b`; `None` is a random mover.
    fn match_up(a: Option<Strength>, b: Option<Strength>, games: u32, seed: u64) -> u32 {
        let rules = Rules::finkel_2017();
        let mut seed = seed;
        let mut wins = 0;
        for g in 0..games {
            let first = if g % 2 == 0 { Side::Light } else { Side::Dark };
            let mut p = Position::new(&rules, first);
            while p.winner().is_none() {
                let marked = (0..4).map(|_| (rng(&mut seed) >> 33) as u32 & 1).sum();
                let r = rules.lots.moves(marked);
                let who = if p.to_move == Side::Light { a } else { b };
                let mv = match who {
                    Some(s) => choose(&rules, &p, r, s),
                    None => {
                        let ms = p.moves(&rules, r);
                        if ms.is_empty() { None } else { Some(ms[(rng(&mut seed) >> 40) as usize % ms.len()]) }
                    }
                };
                match mv {
                    Some(mv) => {
                        p.play(&rules, mv);
                    }
                    None => p.pass(),
                }
            }
            if p.winner() == Some(Side::Light) {
                wins += 1;
            }
        }
        wins
    }

    #[test]
    #[ignore]
    fn strengths() {
        println!("easy vs random {}/200", match_up(Some(Strength::Easy), None, 200, 1));
        println!("normal vs random {}/200", match_up(Some(Strength::Normal), None, 200, 2));
        println!("normal vs easy {}/200", match_up(Some(Strength::Normal), Some(Strength::Easy), 200, 3));
        println!("hard vs easy {}/100", match_up(Some(Strength::Hard), Some(Strength::Easy), 100, 4));
        println!("hard vs normal {}/100", match_up(Some(Strength::Hard), Some(Strength::Normal), 100, 5));
    }
}

#[cfg(test)]
mod timing {
    use super::*;
    use crate::rules::Rules;
    use std::time::Instant;

    #[test]
    fn hard_move_time() {
        let rules = Rules::finkel_2017();
        let mut p = Position::new(&rules, Side::Light);
        p.pieces[0] = vec![0, 0, 2, 5, 7, 10, 13];
        p.pieces[1] = vec![0, 1, 3, 6, 9, 11, 14];
        // Many pieces able to hop between rosettes.
        let mut q = Position::new(&rules, Side::Light);
        q.pieces[0] = vec![0, 0, 0, 4, 6, 10, 12];
        q.pieces[1] = vec![0, 0, 0, 0, 1, 2, 3];
        for roll in 1..=4 {
            let t = Instant::now();
            choose(&rules, &q, roll, Strength::Hard);
            println!("rosettes roll {roll}: {:?}", t.elapsed());
        }
        for strength in [Strength::Normal, Strength::Hard] {
            for roll in 1..=4 {
                let t = Instant::now();
                let mv = choose(&rules, &p, roll, strength);
                println!("{strength:?} roll {roll}: {:?} in {:?}", mv, t.elapsed());
            }
        }
    }
}
