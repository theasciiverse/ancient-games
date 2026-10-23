//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `mu-torere --rulebook-examples` and the chapter shows each block, so
//! the book can never show a move the program would not make.

use crate::app::name;
use crate::rules::{CENTRE, POINTS, Position, Rules, Side};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![("start", start()), ("blocked", blocked()), ("best-game", best_game())]
}

/// Where each point goes in the square drawing: the ring round the
/// border, kewai 1 at the top middle and on clockwise, the pūtahi in the
/// middle. The joins are those of the star.
const SQUARE: [(usize, usize); POINTS] = [(0, 1), (0, 2), (1, 2), (2, 2), (2, 1), (2, 0), (1, 0), (0, 0), (1, 1)];

/// The board drawn square, with the points' numbers beside it: W a white
/// man, B a black one, . the empty point.
fn draw(p: &Position, label: &str) -> Vec<String> {
    let mark = |q: usize| match p.at[q] {
        Some(Side::White) => 'W',
        Some(Side::Black) => 'B',
        None => '.',
    };
    let number = |q: usize| if q == CENTRE { 'P' } else { char::from(b'1' + q as u8) };
    let row = |r: usize, f: &dyn Fn(usize) -> char| {
        let at = |c: usize| (0..POINTS).find(|&q| SQUARE[q] == (r, c)).map(f).unwrap_or(' ');
        format!("{} --- {} --- {}", at(0), at(1), at(2))
    };
    let mut lines = vec![format!("{label}:")];
    for r in 0..3 {
        lines.push(format!("  {}        {}", row(r, &mark), row(r, &number)));
        if r < 2 {
            let between = if r == 0 { "| \\   |   / |" } else { "| /   |   \\ |" };
            lines.push(format!("  {between}        {between}"));
        }
    }
    lines
}

fn moves(rules: &Rules, p: &Position) -> String {
    let mut list: Vec<String> =
        p.moves(rules).iter().map(|m| format!("{} to {}", name(m.from as usize), name(m.to as usize))).collect();
    list.sort();
    if list.is_empty() { "none".to_string() } else { list.join("; ") }
}

/// At the start only the men beside an enemy may enter the pūtahi.
fn start() -> String {
    let rules = Rules::beside_enemy();
    let p = Position::new(&rules);
    let mut lines = vec![format!("{}: the start, White to move.", rules.name)];
    lines.extend(draw(&p, "Position"));
    lines.push(format!("White's moves: {}.", moves(&rules, &p)));
    lines.join("\n")
}

/// A side that cannot move has lost.
fn blocked() -> String {
    let rules = Rules::beside_enemy();
    let mut at = [None; POINTS];
    for q in [0, 1, 3, CENTRE] {
        at[q] = Some(Side::White);
    }
    for q in [4, 5, 6, 7] {
        at[q] = Some(Side::Black);
    }
    let p = Position { at, to_move: Side::Black, moves_made: [5, 5] };
    let mut lines = vec![format!("{}: Black to move.", rules.name)];
    lines.extend(draw(&p, "Position"));
    lines.push(format!("Black's moves: {}.", moves(&rules, &p)));
    if let Some(w) = p.winner(&rules) {
        lines.push(format!("{} wins: Black's men are nowhere near kewai 3.", w.name()));
    }
    lines.join("\n")
}

/// Tuta Nihoniho's sample game, as Best gives it (1925, p. 111), replayed:
/// B (Black, on 5 to 8) opens. Each move is checked against both rule
/// sets, and every move into the pūtahi is marked with whether the man
/// came from beside an enemy.
fn best_game() -> String {
    let best = Rules::best();
    let beside = Rules::beside_enemy();
    // Best's numbers: kewai 1 to 8; 0 for the pūtahi.
    let game: [(usize, usize); 11] =
        [(5, 0), (4, 5), (0, 4), (3, 0), (4, 3), (0, 4), (3, 0), (2, 3), (0, 2), (4, 0), (0, 0)];
    let point = |n: usize| if n == 0 { CENTRE } else { n - 1 };
    let mut p = Position::new(&best);
    p.to_move = Side::Black;
    let mut lines = vec!["Tuta Nihoniho's game (Best 1925, p. 111), B opening:".to_string()];
    for &(from, to) in game.iter().take(10) {
        let (from, to) = (point(from), point(to));
        let side = p.to_move;
        let mv = crate::rules::Move { from: from as u8, to: to as u8 };
        let legal = p.moves(&best).contains(&mv);
        let strict = p.moves(&beside).contains(&mv);
        let who = if side == Side::Black { "B" } else { "A" };
        let mut line = format!("  {who}: {} to {}", name(from), name(to));
        if to == CENTRE {
            line.push_str(if strict { " (beside an enemy)" } else { " (not beside an enemy)" });
        }
        if !legal {
            line.push_str(" ILLEGAL");
        }
        lines.push(line);
        p.play(&best, &mv);
    }
    lines.extend(draw(&p, "The end"));
    let blocked = p.moves(&best).is_empty() && p.moves(&beside).is_empty();
    lines.push(format!("B to move: {}.", if blocked { "blocked, A has won" } else { "not blocked" }));
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
