//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `morris --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Board, Move, Outcome, Position, Rules, Side, Why};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("mill", mill()),
        ("all-in-mills", all_in_mills()),
        ("blocked", blocked()),
        ("flying", flying()),
        ("three", three()),
    ]
}

fn at(board: &Board, name: &str) -> usize {
    (0..board.len()).find(|&p| board.name(p) == name).unwrap_or_else(|| panic!("no point {name}"))
}

fn position(board: &Board, white: &[&str], black: &[&str], hand: [u8; 2], to_move: Side) -> Position {
    let set = |names: &[&str]| names.iter().fold(0u32, |m, n| m | 1 << at(board, n));
    Position { men: [set(white), set(black)], hand, to_move, barred: [None, None], quiet: 0 }
}

/// The board as text: W a white man, B a black one, + an empty point,
/// four columns and two rows between grid lines.
fn draw(board: &Board, p: &Position, label: &str) -> Vec<String> {
    let n = board.size as usize;
    let (w, h) = (4 * (n - 1) + 1, 2 * (n - 1) + 1);
    let mut grid = vec![vec![' '; w]; h];
    for line in &board.lines {
        for pair in line.windows(2) {
            let (r0, c0) = board.points[pair[0]];
            let (r1, c1) = board.points[pair[1]];
            let (y0, x0, y1, x1) = (2 * r0 as i32, 4 * c0 as i32, 2 * r1 as i32, 4 * c1 as i32);
            let (dy, dx) = ((y1 - y0).signum(), (x1 - x0).signum());
            let glyph = match (dy, dx) {
                (0, _) => '-',
                (_, 0) => '|',
                (a, b) if a == b => '\\',
                _ => '/',
            };
            if dy != 0 && dx != 0 {
                // A diagonal: one stroke in the middle of each grid step.
                for j in 0..(y1 - y0).abs() / 2 {
                    grid[(y0 + dy * (2 * j + 1)) as usize][(x0 + dx * (4 * j + 2)) as usize] = glyph;
                }
            } else {
                for i in 1..(y1 - y0).abs().max((x1 - x0).abs()) {
                    grid[(y0 + dy * i) as usize][(x0 + dx * i) as usize] = glyph;
                }
            }
        }
    }
    for q in 0..board.len() {
        let (r, c) = board.points[q];
        grid[2 * r as usize][4 * c as usize] = match p.side_at(q) {
            Some(Side::White) => 'W',
            Some(Side::Black) => 'B',
            None => '+',
        };
    }
    let mut lines = vec![format!("{label}:")];
    for (y, row) in grid.iter().enumerate() {
        let number = if y % 2 == 0 { format!("{}", n - y / 2) } else { String::new() };
        let text: String = row.iter().collect();
        lines.push(format!("  {number:>1}  {text}").trim_end().to_string());
    }
    let letters: Vec<String> = (0..n).map(|c| ((b'a' + c as u8) as char).to_string()).collect();
    lines.push(format!("     {}", letters.join("   ")));
    lines
}

fn describe(board: &Board, m: &Move) -> String {
    let to = board.name(m.to as usize);
    let mut s = match m.from {
        Some(f) => format!("{} to {to}", board.name(f as usize)),
        None => format!("enter on {to}"),
    };
    let taken: Vec<String> = m.taken().map(|q| board.name(q)).collect();
    if !taken.is_empty() {
        s.push_str(&format!(", taking {}", taken.join(" and ")));
    }
    s
}

fn hand(p: &Position) -> String {
    format!("In hand: White {}, Black {}.", p.hand[0], p.hand[1])
}

/// A mill takes a man, never one standing in a mill.
fn mill() -> String {
    let rules = Rules::nine_modern();
    let b = rules.board();
    let mut p = position(&b, &["a1", "d1", "d2", "f4"], &["a7", "d7", "g7", "b4", "e3"], [5, 4], Side::White);
    let mut lines = vec![format!("{}: White to enter a man.", rules.name)];
    lines.extend(draw(&b, &p, "Before"));
    lines.push(hand(&p));
    let g1 = at(&b, "g1") as u8;
    let mut options: Vec<Move> = p.moves(&rules, &b).into_iter().filter(|m| m.to == g1).collect();
    options.sort_by_key(|m| m.take[0]);
    let taken: Vec<String> = options.iter().flat_map(|m| m.taken().map(|q| b.name(q)).collect::<Vec<_>>()).collect();
    lines.push(format!("White on g1 makes the mill a1-d1-g1, and may take: {}.", taken.join(", ")));
    lines.push("Not a7, d7 or g7: they stand in a mill.".to_string());
    let mv = options.into_iter().find(|m| m.take[0] == Some(at(&b, "b4") as u8)).expect("b4 can go");
    p.play(&rules, &b, &mv);
    lines.extend(draw(&b, &p, &format!("After White {}", describe(&b, &mv))));
    lines.join("\n")
}

/// When every man is in a mill: the two Nine Men's Morris rule sets
/// differ.
fn all_in_mills() -> String {
    let b = Board::new(crate::rules::Layout::Nine);
    let p = position(&b, &["a1", "d1"], &["b6", "d6", "f6"], [6, 6], Side::White);
    let mut lines = vec!["Every black man stands in a mill; White to enter.".to_string()];
    lines.extend(draw(&b, &p, "Position"));
    let g1 = at(&b, "g1") as u8;
    for rules in [Rules::nine_modern(), Rules::nine_bell()] {
        let mut taken: Vec<String> = p
            .moves(&rules, &b)
            .into_iter()
            .filter(|m| m.to == g1)
            .flat_map(|m| m.taken().map(|q| b.name(q)).collect::<Vec<_>>())
            .collect();
        taken.sort();
        let what = if taken.is_empty() { "nothing".to_string() } else { format!("one of {}", taken.join(", ")) };
        lines.push(format!("{}: the mill on g1 takes {what}.", rules.name));
    }
    lines.join("\n")
}

/// A side that cannot move has lost.
fn blocked() -> String {
    let rules = Rules::nine_bell();
    let b = rules.board();
    let p = position(
        &b,
        &["d1", "a4", "b2", "f2", "c3", "e3", "c4", "g4", "g7"],
        &["a1", "d2", "d3", "g1"],
        [0, 0],
        Side::Black,
    );
    let mut lines = vec![format!("{}: Black to move.", rules.name)];
    lines.extend(draw(&b, &p, "Position"));
    lines.push(format!("Black's moves: {}.", p.moves(&rules, &b).len()));
    let result = match p.outcome(&rules, &b) {
        Some(Outcome::Won(Side::White, Why::Blocked)) => "White wins: Black cannot move.",
        _ => "not over",
    };
    lines.push(result.to_string());
    lines.join("\n")
}

/// Down to three men: today's rules let them fly, Bell's do not.
fn flying() -> String {
    let b = Board::new(crate::rules::Layout::Nine);
    let p = position(&b, &["a1", "d2", "g7", "c5", "e4", "b4"], &["a4", "g1", "d7"], [0, 0], Side::Black);
    let mut lines = vec!["Black, down to three men, to move.".to_string()];
    lines.extend(draw(&b, &p, "Position"));
    let a4 = at(&b, "a4") as u8;
    for rules in [Rules::nine_modern(), Rules::nine_bell()] {
        let mut to: Vec<String> =
            p.moves(&rules, &b).iter().filter(|m| m.from == Some(a4)).map(|m| b.name(m.to as usize)).collect();
        to.sort();
        to.dedup();
        let list = if to.len() > 6 { format!("any of the {} empty points", to.len()) } else { to.join(", ") };
        lines.push(format!("{}: a4 may go to {list}.", rules.name));
    }
    lines.join("\n")
}

/// Three Men's Morris: a line of three wins.
fn three() -> String {
    let rules = Rules::three_moving();
    let b = rules.board();
    let mut p = position(&b, &["a1", "b2", "c2"], &["a3", "c3", "c1"], [0, 0], Side::White);
    let mut lines = vec![format!("{}: every man entered, White to move.", rules.name)];
    lines.extend(draw(&b, &p, "Before"));
    let mut moves: Vec<String> = p.moves(&rules, &b).iter().map(|m| describe(&b, m)).collect();
    moves.sort();
    lines.push(format!("White's moves: {}.", moves.join("; ")));
    let mv = p
        .moves(&rules, &b)
        .into_iter()
        .find(|m| m.from == Some(at(&b, "a1") as u8) && m.to == at(&b, "a2") as u8)
        .expect("a1 to a2");
    p.play(&rules, &b, &mv);
    lines.extend(draw(&b, &p, &format!("After White {}", describe(&b, &mv))));
    if let Some(Outcome::Won(side, Why::Line)) = p.outcome(&rules, &b) {
        lines.push(format!("Three in a line, a2-b2-c2: {} wins.", side.name()));
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

    #[test]
    fn the_examples_show_what_they_say() {
        let all = all();
        let text = |n: &str| all.iter().find(|(name, _)| *name == n).unwrap().1.clone();
        assert!(text("mill").contains("may take: b4, e3, f4") || text("mill").contains("may take:"));
        assert!(text("blocked").contains("White wins"));
        assert!(text("three").contains("White wins"));
        print!("{}", all.iter().map(|(n, t)| format!("=== {n}\n{t}\n")).collect::<String>());
    }
}
