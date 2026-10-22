//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `mehen --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::rules::{HOME, Lion, Rules, State, Team};

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    // Four players: two pharaohs a team, so teammates meet on the snake.
    let rules = Rules::all().into_iter().find(|r| r.pharaohs == 2).expect("four players");
    vec![
        ("swap", advance(&rules, &[10, 30], &[13, 40], 3, 8, 16)),
        ("teammate", advance(&rules, &[10, 13], &[30, 40], 3, 8, 16)),
        ("entering", advance(&rules, &[0, 30], &[2, 40], 2, 0, 6)),
        ("home", advance(&rules, &[67, 30], &[20, 40], 5, 63, 70)),
        ("lion-back", lion_back(&rules)),
    ]
}

/// A state with the lions' pharaohs and the lionesses' on the squares
/// given (0 waiting at the tail), the lions to move.
fn state(rules: &Rules, lions: &[u8], lionesses: &[u8]) -> State {
    let mut s = State::new(rules, Team::Lions);
    let mut on = lions.iter().chain(lionesses.iter());
    for p in s.pharaohs.iter_mut() {
        p.pos = *on.next().expect("a square for every pharaoh");
    }
    s
}

/// Squares `lo` to `hi` (0 the tail, before the first square): L a lions'
/// pharaoh, S a lionesses', . empty.
fn strip(s: &State, label: &str, lo: u8, hi: u8) -> String {
    let cells: String = (lo..=hi)
        .map(|sq| {
            let here: Vec<&str> = s
                .pharaohs
                .iter()
                .filter(|p| p.pos == sq)
                .map(|p| if p.team == Team::Lions { "L" } else { "S" })
                .collect();
            format!("{:>4}", if here.is_empty() { ".".to_string() } else { here.concat() })
        })
        .collect();
    format!("{label:<8}{cells}")
}

fn squares(lo: u8, hi: u8) -> String {
    let cells: String = (lo..=hi).map(|sq| format!("{sq:>4}")).collect();
    format!("{:<8}{cells}", "square")
}

/// The lions' first pharaoh moves `steps` along the snake.
fn advance(rules: &Rules, lions: &[u8], lionesses: &[u8], steps: u8, lo: u8, hi: u8) -> String {
    let mut s = state(rules, lions, lionesses);
    let from = s.pharaohs[0].pos;
    let mut lines = vec![
        format!("{}: a lions' pharaoh (L) moves {steps} from {from}.", rules.name),
        "S is a pharaoh of the lionesses; square 0 is off the board.".to_string(),
        squares(lo, hi),
        strip(&s, "before", lo, hi),
    ];
    let landing = s.advance(0, steps);
    lines.push(strip(&s, "after", lo, hi));
    let to = s.pharaohs[0].pos;
    let mut what = if landing.home {
        "It reaches the head and is reborn".to_string()
    } else {
        format!("It lands on {to}")
    };
    if landing.hops > 0 {
        what.push_str(&format!(", one further for each teammate ({})", landing.hops));
    }
    if let Some(q) = landing.swapped {
        let back = s.pharaohs[q].pos;
        what.push_str(&format!("; the lionesses' pharaoh goes back to {back}"));
    }
    lines.push(format!("{what}."));
    if to == HOME {
        lines.push("(No exact count is needed.)".to_string());
    }
    lines.join("\n")
}

/// Two right guesses in a row bring back a dead cat.
fn lion_back(rules: &Rules) -> String {
    let mut s = state(rules, &[10, 30], &[20, 40]);
    s.lions[Team::Lions.index()][0] = Lion::Dead;
    let count = |s: &State| {
        format!(
            "in reserve {}, dead {}",
            s.count(Team::Lions, Lion::Reserve),
            s.count(Team::Lions, Lion::Dead)
        )
    };
    let mut lines = vec![
        format!("{}: the lions' team has lost a cat.", rules.name),
        format!("The lions' cats: {}.", count(&s)),
    ];
    for n in 1..=2 {
        let back = s.record_guess(Team::Lions, true);
        lines.push(format!(
            "Right guess {n}: cats {}{}.",
            count(&s),
            if back { " (one is back)" } else { "" }
        ));
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
