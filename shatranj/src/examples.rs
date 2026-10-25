//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `shatranj --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Elephant, Kind, Man, Move, Outcome, Position, Rules, Side, Why, name, parse};

pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("start", start(&Rules::shatranj())),
        ("start-corner", start(&Rules::chaturanga(Elephant::Straight))),
        ("dilaram", dilaram()),
        ("bare", bare()),
        ("stalemate", stalemate()),
    ]
}

/// The board as text: Red's men in capitals, Black's in small letters.
pub fn draw(p: &Position, label: &str) -> Vec<String> {
    let mut out = vec![format!("{label}:")];
    for r in (0..8).rev() {
        let mut row = format!("  {} ", r + 1);
        for f in 0..8 {
            let c = match p.board[r * 8 + f] {
                Some(m) if m.side == Side::Red => m.kind.letter(),
                Some(m) => m.kind.letter().to_ascii_lowercase(),
                None => '.',
            };
            row.push(' ');
            row.push(c);
        }
        out.push(row);
    }
    out.push("     a b c d e f g h".to_string());
    out
}

fn setup(red: &[(&str, Kind)], black: &[(&str, Kind)], to_move: Side) -> Position {
    let mut p = Position::empty();
    for (side, men) in [(Side::Red, red), (Side::Black, black)] {
        for &(sq, kind) in men {
            p.board[parse(sq).expect("a square")] = Some(Man { side, kind });
        }
    }
    p.to_move = to_move;
    p
}

/// Plays "h4-h8" (or "h4xh8") if it is legal, and writes it down.
fn play(rules: &Rules, p: &mut Position, mv: &str) -> String {
    let (from, to) = (parse(&mv[..2]).unwrap(), parse(&mv[3..5]).unwrap());
    let m: Move = p
        .moves()
        .into_iter()
        .find(|m| m.from as usize == from && m.to as usize == to)
        .unwrap_or_else(|| panic!("{mv} is not legal here"));
    let man = p.board[from].unwrap();
    let take = if p.board[to].is_some() { "x" } else { "-" };
    let letter = if man.kind == Kind::Baidaq { String::new() } else { man.kind.letter().to_string() };
    p.play(rules, &m);
    let check = if p.in_check(p.to_move) { "+" } else { "" };
    format!("{letter}{}{take}{}{check}", name(from), name(to))
}

fn line(rules: &Rules, p: &mut Position, moves: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, pair) in moves.chunks(2).enumerate() {
        let a = play(rules, p, pair[0]);
        let b = pair.get(1).map(|m| play(rules, p, m)).unwrap_or_default();
        out.push(format!("  {}. {a:<10} {b}", i + 1));
    }
    out
}

fn ending(rules: &Rules, p: &Position) -> String {
    match p.outcome(rules, &p.moves()) {
        Some(Outcome::Win(w, Why::Mate)) => format!("Mate: {} wins.", w.name()),
        Some(Outcome::Win(w, Why::Bare)) => format!("{} is bare: {} wins.", w.other().name(), w.name()),
        Some(Outcome::Win(w, Why::Stalemate)) => format!("{} has no move: {} wins.", p.to_move.name(), w.name()),
        Some(Outcome::Draw(Why::BothBare)) => "Both shahs are bare: a draw.".to_string(),
        Some(o) => format!("{o:?}"),
        None => "The game goes on.".to_string(),
    }
}

fn start(rules: &Rules) -> String {
    let p = Position::new(rules);
    let mut lines = draw(&p, "The start");
    lines.push("K shah, F firzan, E elephant (fil), N horse (faras),".to_string());
    lines.push("R rukh, P foot soldier (baidaq). Red in capitals.".to_string());
    lines.join("\n")
}

/// Dilaram's problem, from as-Suli (Murray's No. 83).
fn dilaram() -> String {
    use Kind::*;
    let rules = Rules::shatranj();
    let start = setup(
        &[("a4", Shah), ("h4", Rukh), ("h1", Rukh), ("h3", Fil), ("g4", Faras), ("f6", Baidaq), ("g6", Baidaq)],
        &[("g8", Shah), ("b8", Rukh), ("b2", Rukh), ("c4", Faras)],
        Side::Red,
    );
    let mut lines = draw(&start, "Red to play and win");
    let mut p = start.clone();
    lines.push("The solution in the manuscripts:".to_string());
    lines.extend(line(&rules, &mut p, &["h4-h8", "g8-h8", "h3-f5", "b2-h2", "h1-h2", "h8-g8", "h2-h8", "g8-h8", "g6-g7", "h8-g8", "g4-h6"]));
    lines.push(ending(&rules, &p));
    let mut q = start;
    lines.push("If Black declines the block, mate comes a move sooner:".to_string());
    lines.extend(line(&rules, &mut q, &["h4-h8", "g8-h8", "h3-f5", "h8-g8", "h1-h8", "g8-h8", "g6-g7", "h8-g8", "g4-h6"]));
    lines.push(ending(&rules, &q));
    lines.join("\n")
}

/// Murray's illustration of the bare king, under both rules.
fn bare() -> String {
    use Kind::*;
    let suli = Rules::shatranj();
    let medina = Rules::medinese();
    let start = setup(&[("d6", Shah), ("h3", Rukh)], &[("c8", Shah), ("a8", Rukh)], Side::Red);
    let mut lines = draw(&start, "Red to play");
    let mut p = start.clone();
    lines.extend(line(&suli, &mut p, &["h3-h8", "c8-b7", "h8-a8"]));
    assert!(p.outcome(&suli, &p.moves()).is_none());
    lines.push(format!("Black is bare. In Medina: {}", ending(&medina, &p)));
    let back = play(&suli, &mut p, "b7-a8");
    lines.push(format!("Elsewhere he may take back, {back}: {}", ending(&suli, &p).to_lowercase()));
    let start = setup(&[("e6", Shah), ("h3", Rukh)], &[("d8", Shah), ("a8", Rukh)], Side::Red);
    lines.extend(draw(&start, "With the shahs on e6 and d8"));
    let mut q = start;
    lines.extend(line(&suli, &mut q, &["h3-h8", "d8-c7", "h8-a8"]));
    lines.push(format!("Black cannot take back. {}", ending(&suli, &q)));
    lines.join("\n")
}

/// A stalemate: lost in shatranj, won under the Indian rule.
fn stalemate() -> String {
    use Kind::*;
    let start = setup(&[("c2", Shah), ("b3", Firzan), ("h4", Baidaq)], &[("a1", Shah), ("h5", Baidaq)], Side::Black);
    let mut lines = draw(&start, "Black to move");
    lines.push("The black shah is not attacked, but every square".to_string());
    lines.push("he could go to is, and his foot soldier is blocked.".to_string());
    lines.push(format!("Shatranj: {}", ending(&Rules::shatranj(), &start)));
    lines.push(format!("Chaturanga, the Indian rule: {}", ending(&Rules::chaturanga(Elephant::Diagonal), &start)));
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

    #[test]
    fn dilaram_ends_in_mate() {
        let text = dilaram();
        assert_eq!(text.matches("Mate: Red wins.").count(), 2, "{text}");
    }
}
