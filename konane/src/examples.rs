//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `konane --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Move, Position, Rules, Side};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![("opening", opening()), ("jumps", jumps())]
}

fn at(rules: &Rules, name: &str) -> usize {
    (0..rules.len()).find(|&p| rules.name(p) == name).unwrap_or_else(|| panic!("no hollow {name}"))
}

/// The board as text: X a black stone, O a white one, . an empty hollow.
fn draw(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let letters: Vec<String> = (0..rules.cols).map(|c| ((b'a' + c as u8) as char).to_string()).collect();
    let mut lines = vec![format!("{label}:"), format!("     {}", letters.join(" "))];
    for r in 0..rules.rows {
        let cells: Vec<&str> = (0..rules.cols)
            .map(|c| match p.at[r * rules.cols + c] {
                Some(Side::Black) => "X",
                Some(Side::White) => "O",
                None => ".",
            })
            .collect();
        lines.push(format!("  {:>2} {}", r + 1, cells.join(" ")));
    }
    lines
}

fn describe(rules: &Rules, m: &Move) -> String {
    match m {
        Move::Lift(p) => format!("lift {}", rules.name(*p)),
        Move::Jump { from, path } => {
            let mut s = rules.name(*from);
            for &q in path {
                s.push_str(&format!(" to {}", rules.name(q)));
            }
            s
        }
    }
}

fn list(rules: &Rules, p: &Position) -> String {
    let mut all: Vec<String> = p.moves(rules).iter().map(|m| describe(rules, m)).collect();
    all.sort();
    all.join("; ")
}

/// The opening after Ernst: what each side may lift.
fn opening() -> String {
    let rules = Rules::ernst();
    let mut p = Position::new(&rules);
    let mut lines = vec![format!("{}: the start.", rules.name)];
    lines.extend(draw(&rules, &p, "Position"));
    lines.push(format!("Black may: {}.", list(&rules, &p)));
    p.play(&rules, &Move::Lift(at(&rules, "d4")));
    lines.push(format!("After Black lifts d4, White may: {}.", list(&rules, &p)));
    p.play(&rules, &Move::Lift(at(&rules, "e4")));
    lines.push(format!("After White lifts e4, Black may: {}.", list(&rules, &p)));
    lines.join("\n")
}

/// A jump may go on in a straight line, not round a corner.
fn jumps() -> String {
    let rules = Rules::ernst();
    let mut p = Position::new(&rules);
    p.lifted = 2;
    p.at = vec![None; rules.len()];
    for (name, side) in [("b2", Side::Black), ("c2", Side::White), ("e2", Side::White), ("b3", Side::White), ("f3", Side::White)] {
        p.at[at(&rules, name)] = Some(side);
    }
    let mut lines = vec![format!("{}: Black to move.", rules.name)];
    lines.extend(draw(&rules, &p, "Position"));
    lines.push(format!("After Ernst, Black may: {}.", list(&rules, &p)));
    let modern = Rules::modern();
    lines.push(format!("{}: {}.", modern.name, list(&modern, &p)));
    lines.push("Landed on f2, the stone may not turn to jump f3: a move keeps to one line.".to_string());
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
