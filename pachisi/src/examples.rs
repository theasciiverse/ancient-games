//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `pachisi --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{HOME, Position, Rules, Throw, Throws, loop_square, tokens};
use crate::turn::Turn;

pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("throws-tylor", throws(&Rules::tylor())),
        ("throws-falkener", throws(&Rules::falkener())),
        ("tylor-turn", tylor_turn()),
        ("rotten", rotten()),
        ("doublet", doublet()),
    ]
}

/// The throws of six cowries, with how often each falls (each shell
/// mouth up one time in two).
fn throws(rules: &Rules) -> String {
    let ways = [1, 6, 15, 20, 15, 6, 1];
    let mut lines = vec!["Mouths up  Counts   Chance     Then".to_string()];
    for up in 0..=6u8 {
        let (v, again, pau) = rules.cowries(up);
        let extra = match (again, pau) {
            (true, true) if rules.throws == Throws::TylorCowries => "throw again; a starting 1",
            (true, true) => "throw again; a grace of 1",
            (true, false) => "throw again",
            _ => "",
        };
        lines.push(format!("{up:>9}  {v:>6}   {:>2} in 64   {extra}", ways[up as usize]).trim_end().to_string());
    }
    lines.join("\n")
}

fn t(rules: &Rules, v: u8) -> Throw {
    let up = (0..=6).find(|&u| rules.cowries(u).0 == v).unwrap();
    let (value, again, pau) = rules.cowries(up);
    Throw { value, again, pau, shows: up }
}

fn place(s: u8) -> String {
    match s {
        0 => "the middle".to_string(),
        HOME => "home".to_string(),
        s => format!("square {s}"),
    }
}

/// A turn of Tylor's game from the start: a 10, then a 2.
fn tylor_turn() -> String {
    let rules = Rules::tylor();
    let mut p = Position::new(&rules);
    let mut turn = Turn::new();
    turn.add(&rules, &[t(&rules, 10)]);
    turn.add(&rules, &[t(&rules, 2)]);
    let toks = tokens(&rules, &turn.throws);
    let shown: Vec<String> = toks.iter().map(|k| if k.enters { "1 (starting)".to_string() } else { k.value.to_string() }).collect();
    let mut lines = vec![
        "Red throws 10 (one mouth up): throw again; then 2.".to_string(),
        format!("To use: {}.", shown.join(", ")),
    ];
    let mut left = toks.clone();
    for want in [1u8, 10, 2] {
        let mv = p.moves(&rules, &left).into_iter().find(|m| m.tokens[0].value == want).expect("a move");
        lines.push(format!("  {}: a man from {} to {}.", want, place(mv.from), place(mv.to)));
        crate::rules::remove_tokens(&mut left, &mv.tokens);
        p.apply(&mv);
    }
    let safe = rules.castle(loop_square(0, 13).unwrap());
    lines.push(format!("Square 13 is a fort: {}.", if safe { "the man is safe there" } else { "not safe" }));
    lines.join("\n")
}

/// Tylor's rotten throws.
fn rotten() -> String {
    let rules = Rules::tylor();
    let mut lines = Vec::new();
    for seq in [[25, 12, 10, 3], [25, 12, 10, 25]] {
        let mut turn = Turn::new();
        let mut again = true;
        for &v in &seq {
            if !again {
                break;
            }
            again = turn.add(&rules, &[t(&rules, v)]);
        }
        let kept: Vec<String> = turn.throws.iter().map(|x| x.value.to_string()).collect();
        let seq: Vec<String> = seq.iter().map(|v| v.to_string()).collect();
        lines.push(format!("Thrown {}: kept {}{}", seq.join(", "), kept.join(", "), if again { ", and throws on" } else { "" }));
    }
    lines.join("\n")
}

/// Chaupar: a doublet moves two men together twice its value.
fn doublet() -> String {
    let rules = Rules::chaupar();
    let mut p = Position::new(&rules);
    let mut lines = vec![format!(
        "Red's men start on squares {} (Abu'l-Fazl).",
        p.men[0].iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")
    )];
    p.men[0] = [6, 7, 20, 20];
    lines.push("Red has men on 6, 7 and two on 20, and throws 5, 5, 2.".to_string());
    let toks = tokens(&rules, &[5, 5, 2].map(|v| Throw { value: v, again: false, pau: false, shows: v }));
    let moves = p.moves(&rules, &toks);
    let pair = moves.iter().find(|m| m.men.len() == 2).unwrap();
    lines.push(format!(
        "The two fives may move the pair on 20 ten squares each, to {}.",
        pair.to
    ));
    lines.push(format!("Or each die moves a man by itself: {} plays in all.", moves.len()));
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
