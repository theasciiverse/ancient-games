//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `alquerque --rulebook-examples` and the chapter shows each block, so
//! the book can never show a move the program would not make.

use crate::rules::{Move, Position, Rules, Side};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("alquerque-start", start(&Rules::alquerque())),
        ("alquerque-leaps", leaps()),
        ("fanorona-start", start(&Rules::fanorona())),
        ("fanorona-lines", lines()),
        ("fanorona-chain", chain()),
        ("bell", bell()),
        ("vela", vela()),
    ]
}

/// The board as text, lines and all: W a white man, B a black one, o an
/// empty point.
pub fn draw(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let mut out = vec![format!("{label}:")];
    for r in 0..rules.rows {
        let mut row = format!("  {} ", rules.rows - r);
        for c in 0..rules.cols {
            let q = r * rules.cols + c;
            row.push(match p.at[q] {
                Some(Side::White) => 'W',
                Some(Side::Black) => 'B',
                None => 'o',
            });
            if c + 1 < rules.cols {
                row.push_str("---");
            }
        }
        out.push(row);
        if r + 1 < rules.rows {
            let mut links = "    ".to_string();
            for c in 0..rules.cols {
                links.push('|');
                if c + 1 < rules.cols {
                    // Between columns c and c+1 the diagonal runs from
                    // whichever of the two upper points is a diagonal one.
                    let q = r * rules.cols + c;
                    links.push_str(if rules.diagonal(q) { " \\ " } else { " / " });
                }
            }
            out.push(links);
        }
    }
    let letters: Vec<String> = (0..rules.cols).map(|c| ((b'a' + c as u8) as char).to_string()).collect();
    out.push(format!("    {}", letters.join("   ")));
    out
}

pub fn board(rules: &Rules, to_move: Side, men: &[(&str, Side)]) -> Position {
    let mut p = Position::new(rules);
    p.at = vec![None; rules.len()];
    for &(name, side) in men {
        p.at[rules.point(name).unwrap_or_else(|| panic!("no point {name}"))] = Some(side);
    }
    p.been = (0..rules.len()).map(|q| if p.at[q].is_some() { 1u64 << q } else { 0 }).collect();
    p.to_move = to_move;
    p.turns = 10;
    p
}

fn describe(rules: &Rules, mv: &Move) -> String {
    let mut route = vec![rules.name(mv.from())];
    route.extend(mv.steps.iter().map(|s| rules.name(s.to as usize)));
    let taken: Vec<String> = mv.taken().map(|q| rules.name(q)).collect();
    if taken.is_empty() {
        route.join(" to ")
    } else {
        format!("{}, taking {}", route.join(" to "), taken.join(" "))
    }
}

fn start(rules: &Rules) -> String {
    let p = Position::new(rules);
    let mut lines = draw(rules, &p, "The start");
    lines.push(format!("{} men each; White begins.", p.count(Side::White)));
    lines.join("\n")
}

/// A leap, and leaping on.
fn leaps() -> String {
    let rules = Rules::alquerque();
    use Side::*;
    let mut p = board(&rules, White, &[("a1", White), ("b2", Black), ("d4", Black), ("c1", White), ("d2", Black), ("e5", Black)]);
    let mut lines = draw(&rules, &p, "White to move");
    let moves: Vec<Move> = p.moves(&rules).into_iter().filter(|m| m.captures() > 0).collect();
    lines.push("The turns that take (Alfonso lets White stop after".to_string());
    lines.push("the first leap, or not take at all):".to_string());
    for m in moves.iter().filter(|m| p.moves(&Rules::alquerque_bell()).contains(m)) {
        lines.push(format!("  {}", describe(&rules, m)));
    }
    let best = moves.iter().max_by_key(|m| m.captures()).unwrap().clone();
    p.play(&best);
    lines.extend(draw(&rules, &p, &format!("After {}", describe(&rules, &best))));
    lines.join("\n")
}

/// Approach and withdrawal.
fn lines() -> String {
    let rules = Rules::fanorona();
    use Side::*;
    let p = board(
        &rules,
        White,
        &[("e3", White), ("g3", Black), ("h3", Black), ("i3", White), ("c3", Black), ("d3", Black), ("e5", Black)],
    );
    let mut lines = draw(&rules, &p, "White to move");
    lines.push("The captures White can begin with:".to_string());
    for m in p.moves(&rules).iter().filter(|m| m.steps.len() == 1) {
        let kind = if m.steps[0].approach(&rules) { "approach" } else { "withdrawal" };
        lines.push(format!("  {} ({kind})", describe(&rules, m)));
    }
    lines.join("\n")
}

/// A turn of several captures, turning at each.
fn chain() -> String {
    let rules = Rules::fanorona();
    use Side::*;
    let mut p = board(&rules, White, &[("c1", White), ("e1", Black), ("f1", Black), ("d3", Black), ("b4", Black), ("i5", Black)]);
    let mut lines = draw(&rules, &p, "White to move");
    let moves = p.moves(&rules);
    let best = moves.iter().max_by_key(|m| (m.captures(), m.steps.len())).unwrap().clone();
    lines.push(format!("The longest turn: {}.", describe(&rules, &best)));
    p.play(&best);
    lines.extend(draw(&rules, &p, "After it"));
    lines.join("\n")
}

/// Bell's added rules: forward and sideways only, never back to a point.
fn bell() -> String {
    let rules = Rules::alquerque_bell();
    use Side::*;
    let mut p = board(&rules, White, &[("c2", White), ("e5", Black), ("a5", Black)]);
    let mut lines = draw(&rules, &p, "White to move");
    let to = |p: &Position| -> Vec<String> { p.moves(&rules).iter().map(|m| rules.name(m.to())).collect() };
    lines.push(format!("c2 may go to {}, not back to c1.", to(&p).join(", ")));
    let mv = p.moves(&rules).into_iter().find(|m| rules.name(m.to()) == "b2").unwrap();
    p.play(&mv);
    p.to_move = White;
    lines.push(format!("After c2 to b2 it may go to {}: never c2 again.", to(&p).join(", ")));
    lines.join("\n")
}

/// The vela: the loser takes one man at a time; the winner gives.
fn vela() -> String {
    let rules = Rules::vela();
    let mut p = Position::new(&rules);
    let mut lines = vec!["The vela opens: White, who lost, moves first.".to_string()];
    let moves = p.moves(&rules);
    lines.push(format!(
        "White's {} turns all take, one man each: d2 to e3, approaching",
        moves.len()
    ));
    let mv = moves.iter().find(|m| rules.name(m.from()) == "d2" && rules.name(m.to()) == "e3").unwrap().clone();
    lines.push(format!("f4 and g5, takes only {}.", mv.taken().map(|q| rules.name(q)).collect::<Vec<_>>().join(" ")));
    p.play(&mv);
    let takes = p.moves(&rules).iter().filter(|m| m.captures() > 0).count();
    lines.push(format!("Black now has {} moves and may take with none of them.", p.moves(&rules).len()));
    assert_eq!(takes, 0);
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
