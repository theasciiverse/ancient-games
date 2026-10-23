//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `seega --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Move, Position, Rules, Side, Stage};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![("start", start()), ("capture", capture()), ("way", way())]
}

fn at(rules: &Rules, name: &str) -> usize {
    (0..rules.len()).find(|&p| rules.name(p) == name).unwrap_or_else(|| panic!("no hole {name}"))
}

/// The board as text: S a stone kelb, B a brick one, . an empty hole.
fn draw(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let letters: Vec<String> = (0..rules.size).map(|c| ((b'a' + c as u8) as char).to_string()).collect();
    let mut lines = vec![format!("{label}:"), format!("     {}", letters.join(" "))];
    for r in 0..rules.size {
        let cells: Vec<&str> = (0..rules.size)
            .map(|c| match p.at[r * rules.size + c] {
                Some(Side::Stone) => "S",
                Some(Side::Brick) => "B",
                None => ".",
            })
            .collect();
        lines.push(format!("  {:>2} {}", r + 1, cells.join(" ")));
    }
    lines
}

fn board(rules: &Rules, to_move: Side, kelbs: &[(&str, Side)]) -> Position {
    let mut p = Position::new(rules);
    p.at = vec![None; rules.len()];
    for &(name, side) in kelbs {
        p.at[at(rules, name)] = Some(side);
    }
    p.stage = Stage::Moving;
    p.to_move = to_move;
    p
}

fn step(rules: &Rules, p: &mut Position, from: &str, to: &str) -> String {
    let (from, to) = (at(rules, from), at(rules, to));
    let taken: Vec<String> = p.takes(rules, from, to).iter().map(|&q| rules.name(q)).collect();
    let who = p.to_move.name();
    p.play(rules, &Move::Step { from, to });
    let what = if taken.is_empty() { "taking nothing".to_string() } else { format!("taking {}", taken.join(" and ")) };
    format!("{who} moves {} to {}, {what}.", rules.name(from), rules.name(to))
}

/// The board of five as Lane sets it out: his a, a and b, b.
fn start() -> String {
    let rules = Rules::khamsawee();
    let p = Position::new(&rules);
    let mut lines = draw(&rules, &p, "Before the placing");
    lines.push("Stone has set kelbs on c1 and c5 (Lane's a, a),".to_string());
    lines.push("Brick on a3 and e3 (b, b). Then each in turn sets".to_string());
    lines.push(format!("down two, on any of the {} empty holes but c3.", p.moves(&rules).len()));
    lines.join("\n")
}

/// Taking, and taking again at once.
fn capture() -> String {
    let rules = Rules::khamsawee();
    use Side::*;
    let mut p = board(
        &rules,
        Stone,
        &[("b1", Stone), ("c3", Stone), ("e4", Stone), ("c2", Brick), ("d3", Brick), ("d4", Brick), ("a5", Brick)],
    );
    let mut lines = vec!["The board of five, Stone to move.".to_string()];
    lines.extend(draw(&rules, &p, "Position"));
    lines.push(step(&rules, &mut p, "b1", "c1"));
    let next: Vec<String> = p
        .moves(&rules)
        .iter()
        .map(|m| match m {
            Move::Step { from, to } => format!("{} to {}", rules.name(*from), rules.name(*to)),
            _ => String::new(),
        })
        .collect();
    lines.push("Stone takes again at once, and may only take:".to_string());
    lines.push(format!("{}.", next.join("; ")));
    lines.push(step(&rules, &mut p, "e4", "e3"));
    lines.push(format!("Nothing more to take: Brick moves, {} kelbs to {}.", p.count(Brick), p.count(Stone)));
    lines.join("\n")
}

/// No way to move: the other must open one.
fn way() -> String {
    let rules = Rules::khamsawee();
    use Side::*;
    let mut p = board(&rules, Stone, &[("a1", Brick), ("a2", Stone), ("b1", Stone), ("c1", Stone), ("e5", Stone)]);
    let mut lines = vec!["Stone to move.".to_string()];
    lines.extend(draw(&rules, &p, "Position"));
    lines.push(step(&rules, &mut p, "e5", "e4"));
    let gives: Vec<String> = p
        .moves(&rules)
        .iter()
        .map(|m| match m {
            Move::Give(q) => rules.name(*q),
            _ => String::new(),
        })
        .collect();
    lines.push(format!("Brick's a1 is shut in. Stone must give up {}.", gives.join(" or ")));
    p.play(&rules, &Move::Give(at(&rules, "a2")));
    lines.push("Stone gives a2, and Brick moves a1 into it.".to_string());
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
