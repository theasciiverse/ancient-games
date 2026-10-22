//! Worked examples for the rulebook: positions set up by hand, played by
//! the rules engine, and drawn as text. `rulebook/build.sh` runs
//! `mancala --rulebook-examples` and the chapter shows each block, so the
//! book can never show a move the program would not make.

use crate::app::describe;
use crate::rules::{Move, Position, Rules, Side, Slot, Step, Why};
use ancient_core::draw::wrap;

/// Text lines in the examples fit the help (its RULES.txt is 76 wide and
/// indents code blocks by four).
const WIDTH: usize = 68;

/// Every example, as (name, text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("wari-opening", wari_opening()),
        ("oware-capture", oware_capture()),
        ("oware-grand-slam", oware_grand_slam()),
        ("oware-twelve", oware_twelve()),
        ("lane-relay", lane_relay()),
        ("kalah-store", kalah_store()),
        ("kalah-capture", kalah_capture()),
        ("toguz-tuzdyk", toguz_tuzdyk()),
        ("pallanguli-lap", pallanguli_lap()),
        ("congkak-relay", congkak_relay()),
        ("bohnenspiel-chain", bohnenspiel_chain()),
    ]
}

fn position(rules: &Rules, pits: &[u8], to_move: Side) -> Position {
    let mut p = Position::new(rules);
    p.pits = pits.to_vec();
    p.to_move = to_move;
    p
}

/// The board as text: North's pits along the top (a on the right),
/// South's along the bottom (A on the left), the stores at the ends.
fn board(rules: &Rules, p: &Position, label: &str) -> Vec<String> {
    let n = rules.pits;
    let cell = |pit: usize| match p.tuz_owner(pit as u8) {
        Some(_) => "  X".to_string(),
        None => format!("{:>3}", p.pits[pit]),
    };
    // Columns left to right; a clockwise game is drawn mirrored, its
    // South row running from the right.
    let cols: Vec<usize> = if rules.clockwise { (0..n).rev().collect() } else { (0..n).collect() };
    let names_top: String = cols.iter().map(|&c| format!("{:>3}", p.name((2 * n - 1 - c) as u8))).collect();
    let names_bottom: String = cols.iter().map(|&c| format!("{:>3}", p.name(c as u8))).collect();
    let top: String = cols.iter().map(|&c| cell(2 * n - 1 - c)).collect();
    let bottom: String = cols.iter().map(|&c| cell(c)).collect();
    let stores = rules.stores_sown || p.stores != [0, 0];
    // The store on the left, then the one on the right.
    let (left, right) = if rules.clockwise { (p.stores[0], p.stores[1]) } else { (p.stores[1], p.stores[0]) };
    let (ns, ss) = if stores {
        (format!("{left:>3} "), format!("  {right}"))
    } else {
        ("    ".to_string(), String::new())
    };
    vec![
        format!("{label}:"),
        format!("    {names_top}"),
        format!("{ns}{top}"),
        format!("    {bottom}{ss}"),
        format!("    {names_bottom}"),
    ]
    .into_iter()
    .map(|l| l.trim_end().to_string())
    .collect()
}

/// Plays `mv` from `before` and writes the board before and after, with
/// what happened.
fn worked(rules: &Rules, before: &Position, mv: Move, intro: &str) -> String {
    let side = before.to_move;
    let mut after = before.clone();
    let trace = after.play(rules, mv);
    let mut lines = wrap(intro, WIDTH);
    lines.push(String::new());
    lines.extend(board(rules, before, &format!("Before, {} to move", side.name())));
    lines.push(String::new());
    lines.extend(wrap(&describe(rules, before, side, mv, &trace.events), WIDTH));
    lines.extend(wrap(&format!("The seeds go: {}.", path(before, &trace.steps)), WIDTH));
    lines.push(String::new());
    lines.extend(board(rules, &after, "After"));
    lines.join("\n")
}

/// Where the seeds went, pit by pit: lifts, drops and takings.
fn path(p: &Position, steps: &[Step]) -> String {
    let mut out: Vec<String> = Vec::new();
    for s in steps {
        match *s {
            Step::Lift { pit } => out.push(format!("lift {}", p.name(pit))),
            Step::Drop { slot: Slot::Pit(q) } => out.push(p.name(q).to_string()),
            Step::Drop { slot: Slot::Store(side) } => out.push(format!("{}'s store", side.name())),
            Step::Take { pit, why: Why::Capture, .. } => out.push(format!("take {}", p.name(pit))),
            Step::Take { pit, why: Why::Cow, to } => out.push(format!("cow in {} to {}", p.name(pit), to.name())),
            Step::Take { .. } | Step::Pass | Step::Give { .. } => {}
            Step::Restock { side, .. } => out.push(format!("{} fills her row again", side.name())),
            Step::Tuzdyk { pit, .. } => out.push(format!("{} becomes a tuzdyk", p.name(pit))),
        }
    }
    out.join(", ")
}

/// Bell's first Wari example: P (South here) empties C at the start.
fn wari_opening() -> String {
    let rules = Rules::wari();
    let before = Position::new(&rules);
    worked(&rules, &before, Move { pit: 2 }, "The opening move of Bell's example: South lifts the four seeds of C.")
}

/// A capture that runs back through the twos and threes.
fn oware_capture() -> String {
    let rules = Rules::oware();
    let before = position(&rules, &[2, 1, 0, 3, 0, 3, 1, 2, 2, 1, 3, 0], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 5 },
        "The last seed makes c three; b before it is three and a two, so all three are taken.",
    )
}

/// A grand slam takes nothing.
fn oware_grand_slam() -> String {
    let rules = Rules::oware();
    let before = position(&rules, &[1, 0, 2, 0, 0, 3, 1, 2, 1, 0, 0, 0], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 5 },
        "Taking c, b and a would leave North with nothing at all, so under the Abapa rules nothing is taken.",
    )
}

/// Twelve seeds or more pass over the pit they came from.
fn oware_twelve() -> String {
    let rules = Rules::oware();
    let before = position(&rules, &[0, 0, 13, 1, 0, 2, 1, 1, 2, 0, 1, 2], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 2 },
        "Thirteen seeds go once round the board; C itself is passed over, so the last seed lands in E.",
    )
}

/// Lane: three makes the sowing go on; two captures with the pit opposite.
fn lane_relay() -> String {
    let rules = Rules::lane_wise();
    let before = position(&rules, &[2, 0, 2, 0, 0, 1, 5, 3, 0, 0, 0, 7], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 0 },
        "The last pebble makes C three, so C is lifted and sown on; its last pebble makes F two, \
         which is taken with a, facing it.",
    )
}

/// Kalah: the last seed in your own store plays again.
fn kalah_store() -> String {
    let rules = Rules::kalah();
    let before = Position::new(&rules);
    worked(&rules, &before, Move { pit: 2 }, "From C, four seeds reach South's store exactly: South moves again.")
}

/// Kalah: the last seed in an empty pit of your own takes the pit facing it.
fn kalah_capture() -> String {
    let rules = Rules::kalah();
    let before = position(&rules, &[0, 2, 0, 0, 5, 5, 6, 6, 5, 1, 5, 5], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 1 },
        "The last seed falls in D, which was empty: it is taken with the five seeds of c, facing it.",
    )
}

/// Toguz Kumalak: three in an opponent's hole makes a tuzdyk.
fn toguz_tuzdyk() -> String {
    let rules = Rules::toguz();
    let before = position(&rules, &[9, 9, 9, 9, 9, 9, 9, 4, 9, 9, 2, 9, 9, 9, 9, 9, 9, 9], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 7 },
        "Four stones from H: the first goes back into H, then I, a and b. North's b now holds three: \
         it becomes South's tuzdyk, and its stones go to South's kazan.",
    )
}

/// Pallanguli: the Indian lap, a capture beyond an empty pit, and a cow.
fn pallanguli_lap() -> String {
    let rules = Rules::pallanguli();
    let before = position(&rules, &[2, 3, 0, 2, 0, 5, 0, 3, 0, 0, 3, 0, 0, 0], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 0 },
        "South lifts A's two seeds. The first makes B four, a cow, which goes to South as B's owner. \
         The last falls in C, and D after it is lifted and sown: E, F. After F comes the empty G, so \
         a beyond it is captured, and the move ends.",
    )
}

/// Congkak: the sowing goes on from every loaded hole, and an empty hole
/// of your own takes the hole facing it.
fn congkak_relay() -> String {
    let rules = Rules::congkak();
    let before = position(&rules, &[0, 2, 0, 1, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 1 },
        "Congkak sows clockwise, so South's holes run from A on the right to G on the left, and \
         South's store is on the left. The last seed from B falls in D, which holds a seed: D is \
         lifted and sown on, into E and F. F was empty: the seven seeds of the hole facing it are \
         taken, and South is \"dead\".",
    )
}

/// Bohnenspiel: two, four or six, back along the row.
fn bohnenspiel_chain() -> String {
    let rules = Rules::bohnenspiel();
    let before = position(&rules, &[6, 6, 6, 6, 6, 3, 1, 3, 5, 6, 6, 6], Side::South);
    worked(
        &rules,
        &before,
        Move { pit: 5 },
        "The last bean makes c six; b before it is four and a two, so all three are taken.",
    )
}
