//! The game around the rules: throwing the dice, a turn made hop by hop
//! (a man picked up, then where it goes, one die at a time), the
//! computer's turns, and the end.

use crate::ai;
use crate::rules::{Hop, Move, Outcome, Position, Rules, Side};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Waiting for the side to move to throw.
    ToThrow,
    /// The dice are tumbling; they land at `until`.
    Throwing { until: Instant },
    /// The side to move is choosing its hops.
    Choosing,
    /// No die can be used: shown until `until`, then the turn passes.
    NoMove { until: Instant },
    /// The computer's turn, shown a hop at a time.
    Showing { mv: Move, shown: usize, next: Instant },
    Over { winner: Side },
}

pub struct Game {
    pub rules: Rules,
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    /// The dice thrown this turn.
    pub dice: Vec<u8>,
    /// The hops of the person's turn made so far.
    pub partial: Vec<Hop>,
    /// The step (of the side to move) of the man picked up.
    pub selected: Option<u8>,
    /// The spot under the cursor: 0 the hand, 1 to `len` the board's
    /// points, `len + 1` off.
    pub cursor: u8,
    pub last: Option<(Side, Move)>,
    pub message: String,
    pub rng: Rng,
}

const TUMBLE: Duration = Duration::from_millis(600);
const PAUSE: Duration = Duration::from_millis(1100);
const HOP: Duration = Duration::from_millis(420);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        Game {
            dice: vec![1; rules.dice as usize],
            rules,
            players,
            position,
            phase: Phase::ToThrow,
            partial: Vec::new(),
            selected: None,
            cursor: 0,
            last: None,
            message: "White throws first.".to_string(),
            rng,
        }
    }

    pub fn player(&self, side: Side) -> Player {
        self.players[side.index()]
    }

    pub fn human_to_move(&self) -> bool {
        self.player(self.position.to_move) == Player::Human
    }

    pub fn choosing(&self) -> bool {
        matches!(self.phase, Phase::Choosing) && self.human_to_move()
    }

    /// The position as drawn: with the hops made so far.
    pub fn shown(&self) -> Position {
        let mut p = self.position.clone();
        let hops: &[Hop] = match &self.phase {
            Phase::Showing { mv, shown, .. } => &mv.hops[..*shown],
            _ => &self.partial,
        };
        for h in hops {
            p.play_hop(&self.rules, h);
        }
        p
    }

    /// The dice still to use.
    pub fn dice_left(&self) -> Vec<u8> {
        let mut left = self.dice.clone();
        let used: Vec<u8> = match &self.phase {
            Phase::Showing { mv, shown, .. } => mv.hops[..*shown].iter().map(|h| h.die).collect(),
            _ => self.partial.iter().map(|h| h.die).collect(),
        };
        for d in used {
            if let Some(i) = left.iter().position(|&x| x == d) {
                left.remove(i);
            }
        }
        left
    }

    fn matching(&self) -> Vec<Move> {
        self.position
            .moves(&self.rules, &self.dice)
            .into_iter()
            .filter(|m| {
                m.hops.len() >= self.partial.len()
                    && m.hops[..self.partial.len()].iter().zip(&self.partial).all(|(a, b)| a.from == b.from && a.to == b.to)
            })
            .collect()
    }

    fn next_hops(&self) -> Vec<Hop> {
        let n = self.partial.len();
        let mut hops: Vec<Hop> = self.matching().iter().filter_map(|m| m.hops.get(n).copied()).collect();
        hops.sort_by_key(|h| (h.from, h.to, h.die));
        hops.dedup_by_key(|h| (h.from, h.to));
        hops
    }

    /// The steps lit for the person: men that may move, or where the one
    /// picked up may go.
    pub fn lit(&self) -> Vec<u8> {
        if !self.choosing() {
            return Vec::new();
        }
        let hops = self.next_hops();
        let mut lit: Vec<u8> = match self.selected {
            Some(s) => hops.iter().filter(|h| h.from == s).map(|h| h.to).collect(),
            None => hops.iter().map(|h| h.from).collect(),
        };
        lit.sort_unstable();
        lit.dedup();
        lit
    }

    /// The spot shown for a step of the side to move: points are drawn by
    /// board number, the hand and off as themselves.
    pub fn spot(&self, step: u8) -> u8 {
        if step == 0 || step == self.rules.off() { step } else { self.rules.point(self.position.to_move, step) }
    }

    pub fn step_at(&self, spot: u8) -> u8 {
        if spot == 0 || spot == self.rules.off() { spot } else { self.rules.step_of(self.position.to_move, spot) }
    }

    /// Enter or Space, or a click: throw, or choose at the spot.
    pub fn act(&mut self, spot: u8, now: Instant) {
        match self.phase {
            Phase::ToThrow if self.human_to_move() => self.throw(now),
            Phase::Choosing if self.human_to_move() => self.pick(spot),
            _ => {}
        }
    }

    fn pick(&mut self, spot: u8) {
        self.cursor = spot;
        let step = self.step_at(spot);
        let hops = self.next_hops();
        if let Some(s) = self.selected
            && let Some(hop) = hops.iter().find(|h| h.from == s && h.to == step).copied()
        {
            self.partial.push(hop);
            self.selected = None;
            if self.next_hops().is_empty() {
                let mv = Move { hops: std::mem::take(&mut self.partial) };
                self.finish(mv);
            } else {
                self.message = format!("Dice left: {}.", list(&self.dice_left()));
            }
            return;
        }
        if hops.iter().any(|h| h.from == step) {
            self.selected = Some(step);
            self.message = "Now choose where it goes.".to_string();
        } else {
            self.selected = None;
        }
    }

    /// Esc: lets go of the man picked up, or takes back the last hop.
    pub fn cancel(&mut self) {
        if self.selected.is_some() {
            self.selected = None;
        } else if self.partial.pop().is_some() {
            self.message = format!("Taken back. Dice left: {}.", list(&self.dice_left()));
        }
    }

    fn throw(&mut self, now: Instant) {
        self.dice = (0..self.rules.dice).map(|_| (self.rng.next() % 6) as u8 + 1).collect();
        self.phase = Phase::Throwing { until: now + TUMBLE };
        self.message = format!("{} throws...", self.position.to_move.name());
    }

    fn thrown(&mut self, now: Instant) {
        let side = self.position.to_move;
        let moves = self.position.moves(&self.rules, &self.dice);
        info!("{} throws {:?}", side.name(), self.dice);
        if moves.iter().all(|m| m.hops.is_empty()) {
            self.message = format!("{} throws {}: no man can move.", side.name(), list(&self.dice));
            self.phase = Phase::NoMove { until: now + PAUSE };
            return;
        }
        match self.player(side) {
            Player::Human => {
                let n = moves[0].hops.len();
                let some = if n < self.dice.len() { format!(" (only {n} can be used)") } else { String::new() };
                self.message = format!("You throw {}{some}. Choose a man.", list(&self.dice));
                self.partial.clear();
                self.selected = None;
                self.phase = Phase::Choosing;
            }
            Player::Computer(strength) => {
                let mv = ai::choose(&self.rules, &self.position, &self.dice, strength, &mut self.rng).expect("a move");
                self.message = format!("{} throws {}.", side.name(), list(&self.dice));
                self.phase = Phase::Showing { mv, shown: 0, next: now + HOP };
            }
        }
    }

    fn finish(&mut self, mv: Move) {
        let side = self.position.to_move;
        info!("{} plays {:?}", side.name(), mv);
        let hits = mv.hops.iter().filter(|h| h.hits).count();
        self.message = format!("{} throws {}", side.name(), list(&self.dice));
        self.message.push_str(match hits {
            0 => ".",
            1 => " and hits a blot.",
            _ => " and hits blots.",
        });
        self.position.play(&self.rules, &mv);
        self.last = Some((side, mv));
        self.partial.clear();
        self.selected = None;
        self.phase = match self.position.outcome(&self.rules) {
            Some(Outcome::Win(w)) => {
                self.message.push_str(&format!(" {} has borne every man off and wins!", w.name()));
                Phase::Over { winner: w }
            }
            None => Phase::ToThrow,
        };
    }

    pub fn tick(&mut self, now: Instant) {
        match &mut self.phase {
            Phase::ToThrow => {
                if !self.human_to_move() {
                    self.throw(now);
                }
            }
            Phase::Throwing { until } => {
                if now >= *until {
                    self.thrown(now);
                }
            }
            Phase::NoMove { until } => {
                if now >= *until {
                    self.position.to_move = self.position.to_move.other();
                    self.phase = Phase::ToThrow;
                }
            }
            Phase::Showing { mv, shown, next } => {
                if now >= *next {
                    if *shown < mv.hops.len() {
                        *shown += 1;
                        *next = now + HOP;
                    } else {
                        let mv = mv.clone();
                        self.finish(mv);
                    }
                }
            }
            Phase::Choosing | Phase::Over { .. } => {}
        }
    }
}

fn list(dice: &[u8]) -> String {
    dice.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")
}
