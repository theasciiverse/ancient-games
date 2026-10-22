//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `fox-and-geese --rulebook-examples` and the chapter shows each block,
//! so the book can never show a move the program would not make.

use crate::rules::{Move, Point, Position, Rules, SIZE, Side, on_board};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    let thirteen = Rules::thirteen();
    let seventeen = Rules::seventeen();
    vec![
        ("jumps", jumps(&thirteen)),
        ("trapped", trapped(&thirteen)),
        ("geese-forward", geese_forward(&thirteen, &seventeen)),
    ]
}

/// A point as a column letter and a row number, A to G and 1 to 7 from
/// the top left, as Bell names them.
fn name(p: Point) -> String {
    format!("{}{}", (b'A' + p.1 as u8) as char, p.0 + 1)
}

fn position(fox: Point, geese: &[Point], to_move: Side) -> Position {
    let mut geese = geese.to_vec();
    geese.sort_unstable();
    Position { fox, geese, to_move }
}

/// The cross as text: F the fox, G a goose, . an empty point.
fn board(p: &Position, label: &str) -> Vec<String> {
    let mut lines = vec![format!("{label}:"), "     A B C D E F G".to_string()];
    for r in 0..SIZE {
        let cells: Vec<&str> = (0..SIZE)
            .map(|c| {
                let q = (r, c);
                if !on_board(q) {
                    " "
                } else if q == p.fox {
                    "F"
                } else if p.is_goose(q) {
                    "G"
                } else {
                    "."
                }
            })
            .collect();
        lines.push(format!("  {}  {}", r + 1, cells.join(" ")).trim_end().to_string());
    }
    lines
}

fn path(m: &Move) -> String {
    m.path.iter().map(|&p| name(p)).collect::<Vec<_>>().join(" to ")
}

/// The fox may jump goose after goose in one move.
fn jumps(rules: &Rules) -> String {
    let mut p = position((2, 3), &[(3, 3), (5, 3), (3, 2), (4, 4)], Side::Fox);
    let moves = p.moves(rules);
    let longest = moves.iter().max_by_key(|m| m.path.len()).cloned().expect("a move");
    let mut lines = vec![format!("{}: the fox (F) to move.", rules.name)];
    lines.extend(board(&p, "Before"));
    let mut jumps: Vec<String> = moves.iter().filter(|m| !m.killed().is_empty()).map(path).collect();
    jumps.sort();
    lines.push(format!("Jumps allowed: {}.", jumps.join("; ")));
    p.play(&longest);
    lines.extend(board(&p, &format!("After {}", path(&longest))));
    lines.join("\n")
}

/// The geese win when the fox cannot move.
fn trapped(rules: &Rules) -> String {
    let p = position((0, 2), &[(0, 3), (0, 4), (1, 2), (2, 2), (1, 3), (5, 3), (6, 3)], Side::Fox);
    let mut lines = vec![format!("{}: the fox (F) to move.", rules.name)];
    lines.extend(board(&p, "Position"));
    lines.push(format!("Fox moves allowed: {}.", p.moves(rules).len()));
    let winner = match p.winner(rules) {
        Some(Side::Geese) => "the geese",
        Some(Side::Fox) => "the fox",
        None => "nobody yet",
    };
    lines.push(format!("Winner: {winner}."));
    lines.join("\n")
}

/// The thirteen geese may step back; the seventeen may not.
fn geese_forward(thirteen: &Rules, seventeen: &Rules) -> String {
    let p = position((0, 3), &[(3, 3)], Side::Geese);
    let mut lines = vec!["The goose on D4, the geese to move.".to_string()];
    lines.extend(board(&p, "Position"));
    for rules in [thirteen, seventeen] {
        let mut to: Vec<String> = p.moves(rules).iter().map(|m| name(m.to())).collect();
        to.sort();
        lines.push(format!("{}: D4 to {}.", rules.name, to.join(", ")));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_example_fits_the_help() {
        // The text indents examples by 4 and the help is 78 wide.
        for (name, text) in all() {
            for line in text.lines() {
                assert!(line.chars().count() <= 74, "{name}: {line}");
            }
        }
    }
}
