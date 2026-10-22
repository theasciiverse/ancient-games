//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `senet --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Action, OFF, Position, Rules, Side};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    let k5 = Rules::kendall5();
    let k7 = Rules::kendall7();
    let bell = Rules::bell();
    vec![
        ("swap", example(&k5, &[12], &[15], 3, 11, 17)),
        ("protected", example(&k5, &[12], &[15, 16], 3, 8, 17)),
        ("blockade", example(&k5, &[12], &[14, 15, 16], 5, 6, 18)),
        ("beauty", example(&k5, &[24], &[], 3, 20, 28)),
        ("water", example(&k5, &[26], &[15], 1, 13, 27)),
        ("drowning", drowning(&k7)),
        ("bell-capture", example(&bell, &[12], &[15, 16], 3, 11, 17)),
        ("bell-forced", example(&bell, &[22, 26], &[], 2, 21, 30)),
    ]
}

fn position(rules: &Rules, white: &[u8], black: &[u8]) -> Position {
    let mut p = Position::new(rules, Side::White);
    p.board = [None; 31];
    for &h in white {
        p.board[h as usize] = Some(Side::White);
    }
    for &h in black {
        p.board[h as usize] = Some(Side::Black);
    }
    p.off = [0, 0];
    p.to_move = Side::White;
    p
}

/// The houses `lo` to `hi` with what stands on them.
fn strip(p: &Position, label: &str, lo: u8, hi: u8) -> String {
    let cells: String = (lo..=hi)
        .map(|h| match p.at(h) {
            Some(Side::White) => "   ▲",
            Some(Side::Black) => "   ■",
            None => "   .",
        })
        .collect();
    format!("{label:<8}{cells}")
}

fn houses(lo: u8, hi: u8) -> String {
    let cells: String = (lo..=hi).map(|h| format!("{h:>4}")).collect();
    format!("{:<8}{cells}", "house")
}

fn describe(action: &Action) -> String {
    match *action {
        Action::Move { from, to } if to == OFF => format!("{from} off the board"),
        Action::Move { from, to } if to < from => format!("{from} back to {to}"),
        Action::Move { from, to } => format!("{from} to {to}"),
        Action::Rescue => "out of the water to 15".to_string(),
    }
}

fn moves_line(moves: &[Action]) -> String {
    if moves.is_empty() {
        "Moves allowed: none.".to_string()
    } else {
        let list: Vec<String> = moves.iter().map(describe).collect();
        format!("Moves allowed: {}.", list.join(", "))
    }
}

/// White to move with `throw`: the strip of houses before, the moves the
/// rules allow, and the strip after the first of them.
fn example(rules: &Rules, white: &[u8], black: &[u8], throw: u8, lo: u8, hi: u8) -> String {
    let mut p = position(rules, white, black);
    let moves = p.moves(rules, throw);
    let mut lines = vec![
        format!("{}: White (▲) to move, throw {throw}.", rules.name),
        houses(lo, hi),
        strip(&p, "before", lo, hi),
    ];
    if let Some(&first) = moves.first() {
        p.play(rules, first, throw);
        lines.push(strip(&p, "after", lo, hi));
    }
    lines.push(moves_line(&moves));
    if let Some(first) = moves.first()
        && moves.len() > 1
    {
        lines.push(format!("After: {}.", describe(first)));
    }
    lines.join("\n")
}

/// Kendall's seven-piece rules: a piece in the water freezes the others.
fn drowning(rules: &Rules) -> String {
    let p = position(rules, &[24, 27], &[]);
    let mut lines = vec![
        format!("{}: White (▲) has a piece in the water (27).", rules.name),
        houses(13, 28),
        strip(&p, "before", 13, 28),
    ];
    for throw in [2, 4] {
        lines.push(format!("Throw {throw}. {}", moves_line(&p.moves(rules, throw))));
    }
    let rescue = if p.can_rescue(rules) { "allowed" } else { "not allowed" };
    lines.push(format!("Instead of throwing, rescue to 15: {rescue}."));
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
