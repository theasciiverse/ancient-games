//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `patolli --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{HAND, Kind, ODDS, OFF, Position, Rules, Side, kind};

pub fn all() -> Vec<(&'static str, String)> {
    vec![("beans", beans()), ("opening", opening()), ("squares", squares())]
}

fn beans() -> String {
    let mut lines = vec!["Holes up  Counts  Chance".to_string()];
    for (holes, &(v, w)) in ODDS.iter().enumerate() {
        let note = if holes == 0 { "  nothing moves (our choice)" } else { "" };
        lines.push(format!("{holes:>8}  {v:>6}  {w:>2} in 32{note}"));
    }
    lines.join("\n")
}

fn opening() -> String {
    let rules = Rules::bell();
    let mut p = Position::new(&rules, Side::Red);
    let mut lines = vec!["Red begins, and casts 3.".to_string()];
    let moves = p.moves(3);
    lines.push(format!(
        "{} moves: a stone in on Red's square of the middle and 3 on, either way.",
        moves.len()
    ));
    p.play(&moves[0]);
    lines.push("Red takes it down its own arm first (against the sun, as the program".to_string());
    lines.push("draws it): every Red stone will go that way round.".to_string());
    let two = p.moves(2);
    let enters = |n: u8, p: &Position| p.moves(n).iter().any(|m| m.from == HAND);
    lines.push(format!("Next, a 2 can only move that stone ({} move).", two.len()));
    lines.push(format!("Only a 1 brings in another stone: {}.", if enters(1, &p) { "yes" } else { "no" }));
    lines.join("\n")
}

fn squares() -> String {
    let rules = Rules::bell();
    let mut p = Position::new(&rules, Side::Red);
    p.dir = [1, 1];
    p.stones[0][0] = 3;
    p.stones[0][1] = 56;
    let mut lines = vec!["Red has stones 3 and 56 along its way; purses 20 and 20.".to_string()];
    let mv = p.moves(2).into_iter().find(|m| m.from == 3).unwrap();
    let again = p.play(&mv);
    assert!(!again && kind(p.square(Side::Red, 5, 1)) == Kind::Pinched);
    lines.push(format!("A 2: 3 to 5, a pinched compartment. Red pays Blue 2: purses {} and {}.", p.purse[0], p.purse[1]));
    let mv = p.moves(2).into_iter().find(|m| m.from == 5).unwrap();
    let again = p.play(&mv);
    lines.push(format!("A 2 again: 5 to 7, a rounded end{}.", if again { ": Red throws again" } else { "" }));
    let off = p.moves(4).into_iter().find(|m| m.to == OFF);
    lines.push(format!("A 4 bears 56 off exactly{}; a 10 could not.", if off.is_some() { "" } else { " (not!)" }));
    p.play(&off.unwrap());
    lines.push(format!("Blue pays one for it: purses {} and {}.", p.purse[0], p.purse[1]));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_example_fits_the_help() {
        for (name, text) in all() {
            for line in text.lines() {
                assert!(line.chars().count() <= 74, "{name}: {line}");
            }
        }
    }
}
