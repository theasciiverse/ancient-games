//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `royal-ur --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Board, Move, Position, ROWS, Rules, START, Side};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    let finkel = Rules::finkel_2017();
    let museum = Rules::british_museum();
    let bell = Rules::bell();
    vec![
        ("capture", example(&finkel, &[6], &[9], 3)),
        ("middle-rosette", example(&finkel, &[5], &[8], 3)),
        ("middle-rosette-museum", example(&museum, &[5], &[8], 3)),
        ("bear-off", bear_off(&finkel)),
        ("bell-entry", bell_entry(&bell)),
    ]
}

/// A position with Light's and Dark's pieces on the board at the given
/// path positions, the rest of each side waiting at the start.
fn position(rules: &Rules, light: &[u8], dark: &[u8]) -> Position {
    let mut p = Position::new(rules, Side::Light);
    for (side, on) in [(Side::Light, light), (Side::Dark, dark)] {
        for (i, &pos) in on.iter().enumerate() {
            p.pieces[side.index()][i] = pos;
        }
    }
    p
}

/// The board as text: ○ Light, ● Dark, * an empty rosette, . an empty
/// square; then how many of each side wait at the start or are home.
fn board(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let b: Board = rules.board;
    let mut lines = vec![format!("{label}:")];
    for row in 0..ROWS {
        let mut line = String::from("  ");
        for col in 0..b.cols() {
            let cell = if !b.on_board(row, col) {
                " "
            } else if [Side::Light, Side::Dark].iter().any(|&s| on_square(p, b, s, row, col)) {
                if on_square(p, b, Side::Light, row, col) { "○" } else { "●" }
            } else if b.is_rosette(row, col) {
                "*"
            } else {
                "."
            };
            line.push_str(&format!(" {cell} "));
        }
        let side = ["Dark's row", "shared row", "Light's row"][row];
        lines.push(format!("{}   {side}", line.trim_end()));
    }
    let count = |s: Side, at: u8| p.of(s).iter().filter(|&&x| x == at).count();
    lines.push(format!(
        "  waiting: Light {}, Dark {}; home: Light {}, Dark {}",
        count(Side::Light, START),
        count(Side::Dark, START),
        count(Side::Light, rules.home()),
        count(Side::Dark, rules.home())
    ));
    lines
}

fn on_square(p: &Position, b: Board, side: Side, row: usize, col: usize) -> bool {
    b.pos_at(side, row, col).is_some_and(|pos| p.of(side).contains(&pos))
}

fn describe(rules: &Rules, m: &Move) -> String {
    match (m.from, m.to) {
        (START, to) => format!("enter to {to}"),
        (from, to) if to == rules.home() => format!("{from} off the board"),
        (from, to) => format!("{from} to {to}"),
    }
}

fn moves_line(rules: &Rules, moves: &[Move]) -> String {
    if moves.is_empty() {
        return "Moves allowed: none.".to_string();
    }
    let list: Vec<String> = moves.iter().map(|m| describe(rules, m)).collect();
    format!("Moves allowed: {}.", list.join(", "))
}

/// Light to move with `roll`: the board before, the moves allowed, and the
/// board after the move of Light's piece on the board (if it may move).
fn example(rules: &Rules, light: &[u8], dark: &[u8], roll: u8) -> String {
    let mut p = position(rules, light, dark);
    let moves = p.moves(rules, roll);
    let mut lines = vec![
        format!("{}: Light (○) to move, throw {roll}.", rules.name),
        "Squares are counted along Light's path.".to_string(),
    ];
    lines.extend(board(rules, &p, "Before"));
    lines.push(moves_line(rules, &moves));
    if let Some(&m) = moves.iter().find(|m| m.from == light[0]) {
        p.play(rules, m, false);
        lines.extend(board(rules, &p, &format!("After {}", describe(rules, &m))));
    }
    lines.join("\n")
}

/// Bearing off needs the exact throw.
fn bear_off(rules: &Rules) -> String {
    let p = position(rules, &[13], &[]);
    let mut lines = vec![format!("{}: Light (○) has a piece on square 13.", rules.name)];
    lines.extend(board(rules, &p, "Before"));
    for roll in 1..=4 {
        let moves: Vec<Move> = p.moves(rules, roll).into_iter().filter(|m| m.from == 13).collect();
        let what = match moves.first() {
            Some(m) => describe(rules, m),
            None => "the piece on 13 cannot move".to_string(),
        };
        lines.push(format!("Throw {roll}: {what}."));
    }
    lines.join("\n")
}

/// R. C. Bell: only a 5 brings a piece onto the board.
fn bell_entry(rules: &Rules) -> String {
    let p = position(rules, &[], &[]);
    let mut lines = vec![format!("{}: Light (○) has every piece waiting.", rules.name)];
    for roll in [1, 4, 5] {
        lines.push(format!("Throw {roll}: {}", moves_line(rules, &p.moves(rules, roll))));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_example_fits_the_help() {
        for (name, text) in all() {
            // The text indents examples by 4 and the help is 78 wide.
            for line in text.lines() {
                assert!(line.chars().count() <= 74, "{name}: {line}");
            }
        }
    }
}
