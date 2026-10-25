//! The game around the rules: a turn made step by step with the cursor
//! (a man picked up, then each point it goes to; where one step could
//! take either of two lines, the person points at the line), the
//! computer's moves (thought in the background), the end, and the draw by
//! threefold repetition.

use crate::ai;
use crate::rules::{Move, Outcome, Position, Rules, Side, Step};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// How the game stands.
pub enum Phase {
    /// The side to move is choosing (a person) or about to think.
    Turn,
    /// The computer is thinking; its move is shown no sooner than `until`.
    Thinking { handle: JoinHandle<Option<Move>>, until: Instant },
    /// A computer's turn of several steps, shown one step at a time.
    Showing { mv: Move, shown: usize, next: Instant },
    Over { outcome: Outcome },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays White, and who Black.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// How often each position has occurred, for the draw.
    pub seen: HashMap<Position, u32>,
    pub cursor: usize,
    /// The man picked up by the person to move.
    pub selected: Option<usize>,
    /// The steps of the person's turn made so far.
    pub partial: Vec<Step>,
    /// A step whose point has been chosen, but which could take either
    /// of two lines: the person points at one of them.
    pub choosing: Vec<Step>,
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

const SHOW: Duration = Duration::from_millis(450);
const STEP: Duration = Duration::from_millis(500);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        let mut seen = HashMap::new();
        seen.insert(position.clone(), 1);
        Game {
            cursor: rules.centre(),
            position,
            rules,
            players,
            phase: Phase::Turn,
            seen,
            selected: None,
            partial: Vec::new(),
            choosing: Vec::new(),
            last: None,
            message: "White begins.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        matches!(self.phase, Phase::Turn) && self.player(self.position.to_move) == Player::Human
    }

    /// The board as it is to be drawn: with the steps made so far of a
    /// turn in progress, the person's or the computer's.
    pub fn shown(&self) -> Position {
        let mut p = self.position.clone();
        let steps: &[Step] = match &self.phase {
            Phase::Showing { mv, shown, .. } => &mv.steps[..*shown],
            _ => &self.partial,
        };
        for s in steps {
            p.at[s.to as usize] = p.at[s.from as usize].take();
            for &q in &s.taken {
                p.at[q as usize] = None;
            }
        }
        p
    }

    /// The legal turns that begin with the steps made so far.
    fn matching(&self) -> Vec<Move> {
        let moves = self.position.moves(&self.rules);
        moves
            .into_iter()
            .filter(|m| {
                m.steps.len() >= self.partial.len() && m.steps[..self.partial.len()] == self.partial[..]
            })
            .filter(|m| !self.partial.is_empty() || self.selected.is_none_or(|s| m.from() == s))
            .collect()
    }

    /// The next steps the person could make.
    fn next_steps(&self) -> Vec<Step> {
        let n = self.partial.len();
        let mut steps: Vec<Step> = self.matching().into_iter().filter_map(|m| m.steps.get(n).cloned()).collect();
        steps.dedup();
        steps
    }

    /// Whether the turn may end with the steps made so far.
    pub fn may_stop(&self) -> bool {
        !self.partial.is_empty() && self.matching().iter().any(|m| m.steps.len() == self.partial.len())
    }

    /// The points lit for the person: the men that may move, where the
    /// man picked up may go, or the men of each line one step could take.
    pub fn lit(&self) -> Vec<usize> {
        if !self.human_to_move() {
            return Vec::new();
        }
        let mut lit: Vec<usize> = if !self.choosing.is_empty() {
            self.choosing.iter().flat_map(|s| s.taken.iter().map(|&q| q as usize)).collect()
        } else if self.selected.is_some() {
            self.next_steps().iter().map(|s| s.to as usize).collect()
        } else {
            self.matching().iter().map(|m| m.from()).collect()
        };
        lit.sort_unstable();
        lit.dedup();
        lit
    }

    /// The men a step to the cursor would take, to be marked.
    pub fn doomed(&self) -> Vec<usize> {
        if !self.human_to_move() || self.selected.is_none() || !self.choosing.is_empty() {
            return Vec::new();
        }
        let steps: Vec<Step> = self.next_steps().into_iter().filter(|s| s.to as usize == self.cursor).collect();
        if steps.len() == 1 { steps[0].taken.iter().map(|&q| q as usize).collect() } else { Vec::new() }
    }

    pub fn move_cursor(&mut self, dr: i8, dc: i8) {
        if let Some(p) = self.rules.towards(self.cursor, (dr, dc)) {
            self.cursor = p;
        }
    }

    /// The person points at `p`: picks a man up, sends the man picked up
    /// there, chooses the line it takes, or, pointing at the man itself
    /// in the middle of a turn, ends the turn there.
    pub fn pick(&mut self, p: usize) {
        if !self.human_to_move() {
            return;
        }
        self.cursor = p;
        if !self.choosing.is_empty() {
            if let Some(step) = self.choosing.iter().find(|s| s.taken.contains(&(p as u8))).cloned() {
                self.choosing.clear();
                self.take_step(step);
            } else {
                self.message = "Point at one of the men lit: the line to take.".to_string();
            }
            return;
        }
        if let Some(s) = self.selected {
            let steps: Vec<Step> = self.next_steps().into_iter().filter(|st| st.to as usize == p).collect();
            match steps.len() {
                0 if p == s && self.may_stop() => self.finish(),
                0 if self.partial.is_empty() && self.position.at[p] == Some(self.position.to_move) => {
                    self.selected = None;
                    self.pick(p);
                }
                0 if self.partial.is_empty() => self.selected = None,
                0 => {
                    self.message = if self.may_stop() {
                        "Take again on a lit point, or point at the man (or press Esc) to stop.".to_string()
                    } else {
                        "It must take again: choose a lit point.".to_string()
                    }
                }
                1 => self.take_step(steps.into_iter().next().unwrap()),
                _ => {
                    self.choosing = steps;
                    self.message = "That step can take either line: point at a man of the one to take.".to_string();
                }
            }
            return;
        }
        if self.matching().iter().any(|m| m.from() == p) {
            self.selected = Some(p);
            self.message = "Now choose where it goes.".to_string();
        } else if self.position.at[p] == Some(self.position.to_move) {
            self.message = if self.position.moves(&self.rules).iter().any(|m| m.captures() > 0) {
                "A capture must be made: choose a lit man.".to_string()
            } else {
                "That man cannot move.".to_string()
            };
        }
    }

    fn take_step(&mut self, step: Step) {
        let to = step.to as usize;
        self.partial.push(step);
        self.selected = Some(to);
        self.cursor = to;
        let more = !self.next_steps().is_empty();
        if !more {
            self.finish();
        } else if self.may_stop() {
            self.message = "It may take again: a lit point, or Esc to stop here.".to_string();
        } else {
            self.message = "It must take again: choose a lit point.".to_string();
        }
    }

    /// Ends the person's turn with the steps made.
    fn finish(&mut self) {
        let mv = Move { steps: std::mem::take(&mut self.partial) };
        self.play(mv);
    }

    /// Esc: drops a line half chosen, ends a turn that may end here, or
    /// lets go of the man picked up.
    pub fn cancel(&mut self) {
        if !self.choosing.is_empty() {
            self.choosing.clear();
        } else if self.may_stop() {
            self.finish();
        } else if self.partial.is_empty() {
            self.selected = None;
        }
    }

    fn describe(&self, mv: &Move) -> String {
        let r = &self.rules;
        let side = self.position.to_move;
        let mut route = vec![r.name(mv.from())];
        route.extend(mv.steps.iter().map(|s| r.name(s.to as usize)));
        let n = mv.captures();
        let what = match n {
            0 => String::new(),
            1 => format!(", taking {}", r.name(mv.taken().next().unwrap())),
            n => format!(", taking {n} men"),
        };
        format!("{} moves {}{what}.", side.name(), route.join(" to "))
    }

    fn play(&mut self, mv: Move) {
        info!("{} plays {:?}", self.position.to_move.name(), mv);
        self.message = self.describe(&mv);
        self.position.play(&mv);
        self.selected = None;
        self.partial.clear();
        self.choosing.clear();
        self.last = Some(mv);
        let count = self.seen.entry(self.position.clone()).or_insert(0);
        *count += 1;
        if *count >= 3 {
            self.message.push_str(" The same position a third time: a draw.");
            self.phase = Phase::Over { outcome: Outcome::Draw };
            return;
        }
        if let Some(outcome) = self.position.outcome(&self.rules) {
            let pos = &self.position;
            let end = match outcome {
                Outcome::Win(w) if pos.count(w.other()) == 0 => {
                    format!(" {} has no men left: {} wins!", w.other().name(), w.name())
                }
                Outcome::Win(w) => format!(" {} cannot move: {} wins!", w.other().name(), w.name()),
                Outcome::Draw => format!(" {} turns with nothing taken: a draw.", self.rules.quiet),
            };
            self.message.push_str(&end);
            self.phase = Phase::Over { outcome };
        } else {
            self.phase = Phase::Turn;
        }
    }

    /// Whether the computer is still working out its move.
    pub fn thinking(&self) -> bool {
        matches!(&self.phase, Phase::Thinking { handle, .. } if !handle.is_finished())
    }

    pub fn tick(&mut self, now: Instant) {
        match &mut self.phase {
            Phase::Turn => {
                if let Player::Computer(strength) = self.player(self.position.to_move) {
                    let rules = self.rules.clone();
                    let position = self.position.clone();
                    let mut rng = Rng::new(self.rng.next());
                    let handle = std::thread::spawn(move || ai::choose(&rules, &position, strength, &mut rng));
                    self.phase = Phase::Thinking { handle, until: now + SHOW };
                }
            }
            Phase::Thinking { handle, until } => {
                if handle.is_finished() && now >= *until {
                    let Phase::Thinking { handle, .. } = std::mem::replace(&mut self.phase, Phase::Turn) else {
                        unreachable!()
                    };
                    if let Some(mv) = handle.join().ok().flatten() {
                        if mv.steps.len() > 1 {
                            self.message = format!("{} takes again and again...", self.position.to_move.name());
                            self.phase = Phase::Showing { mv, shown: 1, next: now + STEP };
                        } else {
                            self.play(mv);
                        }
                    }
                }
            }
            Phase::Showing { mv, shown, next } => {
                if now >= *next {
                    if *shown < mv.steps.len() {
                        *shown += 1;
                        *next = now + STEP;
                    } else {
                        let mv = mv.clone();
                        self.phase = Phase::Turn;
                        self.play(mv);
                    }
                }
            }
            Phase::Over { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::tests::board;

    #[test]
    fn a_person_takes_step_by_step() {
        let a = Rules::alquerque();
        let mut game = Game::new(a.clone(), [Player::Human, Player::Human], Rng::new(1));
        use Side::*;
        game.position = board(&a, White, &[("a1", White), ("b2", Black), ("d4", Black), ("e1", Black)]);
        let at = |n: &str| a.point(n).unwrap();
        assert_eq!(game.lit(), [at("a1")]);
        game.pick(at("a1"));
        assert_eq!(game.lit(), [at("c3")]);
        game.pick(at("c3"));
        // Must go on: d4 to e5.
        assert_eq!(game.lit(), [at("e5")]);
        game.pick(at("e5"));
        assert_eq!(game.position.count(Black), 1);
        assert_eq!(game.position.to_move, Black);
    }

    #[test]
    fn a_person_chooses_the_line() {
        let f = Rules::fanorona();
        let mut game = Game::new(f.clone(), [Player::Human, Player::Human], Rng::new(1));
        use Side::*;
        game.position = board(&f, White, &[("e3", White), ("g3", Black), ("d3", Black), ("a5", Black)]);
        let at = |n: &str| f.point(n).unwrap();
        game.pick(at("e3"));
        game.pick(at("f3"));
        // Approach g3 or withdraw from d3.
        assert_eq!(game.lit(), [at("d3"), at("g3")]);
        game.pick(at("d3"));
        // Done, or going on? f3 can go on only if it takes again; here not.
        assert_eq!(game.position.at[at("d3")], None);
        assert_eq!(game.position.at[at("g3")], Some(Black));
    }
}
