//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `hnefatafl --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Move, Piece, Position, Rules, Side, Square};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    let copenhagen = Rules::copenhagen();
    let brandubh = Rules::brandubh();
    let linnaeus = Rules::tablut();
    let bell = Rules::bell_tablut();
    vec![
        ("capture", example(&copenhagen, &[(2, 1), (4, 3)], &[(2, 2)], (5, 5), Side::Attackers, mv((4, 3), (2, 3)))),
        ("corner-hostile", example(&brandubh, &[(2, 2)], &[(0, 1)], (3, 3), Side::Attackers, mv((2, 2), (0, 2)))),
        ("king-escapes-corner", example(&copenhagen, &[(3, 3)], &[], (0, 4), Side::Defenders, mv((0, 4), (0, 0)))),
        ("king-two-linnaeus", example(&linnaeus, &[(2, 1), (5, 3)], &[], (2, 2), Side::Attackers, mv((5, 3), (2, 3)))),
        ("king-two-bell", example(&bell, &[(2, 1), (5, 3)], &[], (2, 2), Side::Attackers, mv((5, 3), (2, 3)))),
        ("king-escapes-edge", example(&linnaeus, &[(6, 6)], &[], (2, 3), Side::Defenders, mv((2, 3), (0, 3)))),
    ]
}

fn mv(from: Square, to: Square) -> Move {
    Move { from, to }
}

/// Squares as tafl players write them: files a, b, c from the left, ranks
/// 1, 2, 3 from the bottom.
fn name(rules: &Rules, s: Square) -> String {
    format!("{}{}", (b'a' + s.1 as u8) as char, rules.size - s.0)
}

fn position(rules: &Rules, attackers: &[Square], defenders: &[Square], king: Square, to_move: Side) -> Position {
    let mut p = Position::new(rules);
    p.board = vec![None; rules.size * rules.size];
    for &s in attackers {
        p.board[s.0 * rules.size + s.1] = Some(Piece::Attacker);
    }
    for &s in defenders {
        p.board[s.0 * rules.size + s.1] = Some(Piece::Defender);
    }
    p.board[king.0 * rules.size + king.1] = Some(Piece::King);
    p.to_move = to_move;
    p
}

/// The board as text: A an attacker, D a defender, K the king, + an empty
/// throne or corner, . an empty square.
fn board(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let n = rules.size;
    let files: String = (0..n).map(|c| format!(" {}", (b'a' + c as u8) as char)).collect();
    let mut lines = vec![format!("{label}:"), format!("    {files}")];
    for r in 0..n {
        let cells: String = (0..n)
            .map(|c| {
                let s = (r, c);
                let ch = match p.at(s) {
                    Some(Piece::Attacker) => 'A',
                    Some(Piece::Defender) => 'D',
                    Some(Piece::King) => 'K',
                    None if rules.restricted(s) || rules.is_corner(s) => '+',
                    None => '.',
                };
                format!(" {ch}")
            })
            .collect();
        lines.push(format!("  {:>2}{cells}", n - r));
    }
    lines
}

/// The side to move plays `m`: the board before, whether the move is
/// allowed, what it captured, and the board after.
fn example(
    rules: &Rules,
    attackers: &[Square],
    defenders: &[Square],
    king: Square,
    to_move: Side,
    m: Move,
) -> String {
    let mut p = position(rules, attackers, defenders, king, to_move);
    let mut lines = vec![format!("{}: {} to move.", rules.name, to_move.name())];
    lines.extend(board(rules, &p, "Before"));
    let what = format!("{} to {}", name(rules, m.from), name(rules, m.to));
    if !p.moves_from(rules, m.from).contains(&m) {
        lines.push(format!("{what}: not allowed."));
        return lines.join("\n");
    }
    let out = p.play(rules, m);
    let captured: Vec<String> = out.captured.iter().map(|&s| name(rules, s)).collect();
    lines.push(format!(
        "{what}. Captured: {}.",
        if captured.is_empty() { "nothing".to_string() } else { captured.join(", ") }
    ));
    if let Some(w) = out.winner {
        lines.push(format!("The game is over: {} win.", w.name()));
    }
    lines.extend(board(rules, &p, "After"));
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
