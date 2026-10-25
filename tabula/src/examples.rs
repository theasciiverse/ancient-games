//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `tabula --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{Layout, Move, Position, Rules, Side};

pub fn all() -> Vec<(&'static str, String)> {
    vec![("zeno", zeno()), ("entering", entering()), ("hit", hit()), ("duodecim", duodecim())]
}

/// The board as text, points numbered along White's way: W3 three white
/// men, B2 two black, . an empty point.
pub fn draw(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let cell = |pt: u8| {
        let (w, b) = (p.on_point(rules, Side::White, pt), p.on_point(rules, Side::Black, pt));
        match (w, b) {
            (0, 0) => "  .".to_string(),
            (w, 0) => format!(" W{w}"),
            (_, b) => format!(" B{b}"),
        }
    };
    let row = |pts: Vec<u8>| {
        let cells: Vec<String> = pts.iter().map(|&q| cell(q)).collect();
        format!("  {} | {}", cells[..6].join(" "), cells[6..].join(" "))
    };
    let numbers = |pts: Vec<u8>| {
        let n: Vec<String> = pts.iter().map(|q| format!("{q:>3}")).collect();
        format!("  {} | {}", n[..6].join(" "), n[6..].join(" "))
    };
    let rows: Vec<Vec<u8>> = match rules.layout {
        Layout::Tables => vec![(13..=24).collect(), (1..=12).rev().collect()],
        Layout::ThreeRows => vec![(13..=24).rev().collect(), (1..=12).collect(), (25..=36).collect()],
    };
    let mut out = vec![format!("{label}:")];
    for r in rows {
        out.push(numbers(r.clone()));
        out.push(row(r));
    }
    let hand = |s: Side| p.men[s.index()][0];
    let off = |s: Side| p.borne_off(rules, s);
    out.push(format!(
        "  In hand: White {}, Black {}. Borne off: White {}, Black {}.",
        hand(Side::White),
        hand(Side::Black),
        off(Side::White),
        off(Side::Black)
    ));
    out
}

fn describe(rules: &Rules, side: Side, mv: &Move) -> String {
    let at = |s: u8| {
        if s == 0 {
            "hand".to_string()
        } else if s == rules.off() {
            "off".to_string()
        } else {
            rules.point(side, s).to_string()
        }
    };
    let hops: Vec<String> = mv
        .hops
        .iter()
        .map(|h| format!("{} to {} ({}){}", at(h.from), at(h.to), h.die, if h.hits { " hitting" } else { "" }))
        .collect();
    hops.join(", ")
}

fn zeno() -> String {
    let rules = Rules::tabula();
    let p = crate::rules::zeno();
    let mut lines = draw(&rules, &p, "White (Zeno) to play, having thrown 2, 5, 6");
    let moves = p.moves(&rules, &[2, 5, 6]);
    lines.push(format!("Legal plays: {}.", moves.len()));
    let mut q = p.clone();
    q.play(&rules, &moves[0]);
    lines.push(format!("  {}", describe(&rules, Side::White, &moves[0])));
    lines.extend(draw(&rules, &q, "After it"));
    let blots: Vec<String> = (1..=24).filter(|&s| q.men[0][s as usize] == 1).map(|s| s.to_string()).collect();
    lines.push(format!("White's single men: {} ({}).", blots.len(), blots.join(", ")));
    lines.join("\n")
}

fn entering() -> String {
    let rules = Rules::tabula();
    let mut p = Position::new(&rules);
    let mut lines = vec!["The start: every man in hand. White throws 6, 4, 1.".to_string()];
    let moves = p.moves(&rules, &[6, 4, 1]);
    lines.push(format!("White has {} different plays, for instance:", moves.len()));
    let pick = moves.iter().find(|m| m.hops.iter().all(|h| h.from == 0)).unwrap().clone();
    lines.push(format!("  {}", describe(&rules, Side::White, &pick)));
    let pick2 = moves.iter().find(|m| m.hops.iter().filter(|h| h.from == 0).count() == 1).unwrap().clone();
    lines.push(format!("  {}", describe(&rules, Side::White, &pick2)));
    p.play(&rules, &pick);
    lines.extend(draw(&rules, &p, "After the first"));
    lines.join("\n")
}

fn hit() -> String {
    let rules = Rules::tabula();
    let mut p = Position::new(&rules);
    p.men = [vec![0; 26], vec![0; 26]];
    p.men[0][0] = 10;
    p.men[0][5] = 2;
    p.men[0][9] = 1;
    p.men[0][11] = 2;
    p.men[1][0] = 12;
    p.men[1][rules.step_of(Side::Black, 14) as usize] = 2;
    p.men[1][rules.step_of(Side::Black, 22) as usize] = 1;
    p.to_move = Side::Black;
    let mut lines = draw(&rules, &p, "Black to play 5, 3, 2");
    let moves = p.moves(&rules, &[5, 3, 2]);
    let mv = moves
        .iter()
        .filter(|m| m.hops.iter().any(|h| h.hits))
        .max_by_key(|m| m.hops.iter().filter(|h| h.from != 0).count())
        .unwrap()
        .clone();
    lines.push(format!("Black: {}.", describe(&rules, Side::Black, &mv)));
    p.play(&rules, &mv);
    lines.extend(draw(&rules, &p, "After it"));
    let entering_only = p.moves(&rules, &[6, 6, 2]).iter().all(|m| m.hops.first().is_none_or(|h| h.from == 0));
    lines.push(format!(
        "White's hit man must enter again first{}.",
        if entering_only { ": every play begins by entering it" } else { "" }
    ));
    lines.join("\n")
}

fn duodecim() -> String {
    let rules = Rules::duodecim();
    let mut p = Position::new(&rules);
    p.men[0][0] = 4;
    p.men[0][3] = 4;
    p.men[0][7] = 4;
    p.men[0][12] = 3;
    p.men[1][0] = 9;
    p.men[1][5] = 2;
    p.men[1][9] = 2;
    p.men[1][10] = 2;
    let mut lines = draw(&rules, &p, "Duodecim scripta, White to play 6, 3, 2");
    let moves = p.moves(&rules, &[6, 3, 2]);
    let past = moves.iter().any(|m| m.hops.iter().any(|h| h.to > 12));
    assert!(!past, "a man passed point 12 with men in hand");
    lines.push("White still has men in hand, so no man may pass point 12.".to_string());
    lines.push(format!("Plays: {}, for instance {}.", moves.len(), describe(&rules, Side::White, &moves[0])));
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
