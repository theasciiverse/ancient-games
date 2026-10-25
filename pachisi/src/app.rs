//! The game around the rules: four colours in turn, two partnerships, each
//! played by a person or the computer; throwing (and throwing again on a
//! high throw), then using the throws one by one, a man picked up and put
//! down where a throw takes it; a man taken brings another throw; the end.

use crate::ai;
use crate::rules::{COLOURS, HOME, Move, Position, READY, Rules, TEAMS, Throw, Token, cell, team, throw};
use crate::turn::Turn;
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Waiting for the colour to move to throw.
    ToThrow,
    /// The throw is in the air; it lands at `until`.
    Throwing { until: Instant },
    /// Using the throws.
    Playing,
    /// The computer's moves, shown one at a time.
    Showing { plan: Vec<Move>, next: Instant },
    /// Nothing to be done with the throws: shown until `until`.
    Stuck { until: Instant },
    Over { winner: usize },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays each partnership.
    pub players: [Player; 2],
    pub position: Position,
    pub phase: Phase,
    pub turn: Turn,
    /// The last throw, as it fell.
    pub fallen: Vec<Throw>,
    /// The man picked up: the step it stands on, of the colour moving.
    pub selected: Option<u8>,
    /// The cursor, as (row, column) on the cross.
    pub cursor: (u8, u8),
    pub last: Option<Move>,
    pub message: String,
    pub rng: Rng,
}

const FALL: Duration = Duration::from_millis(500);
const PAUSE: Duration = Duration::from_millis(1100);
const STEP: Duration = Duration::from_millis(450);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], rng: Rng) -> Game {
        let position = Position::new(&rules);
        info!("new game: {} {:?}", rules.id, players);
        Game {
            rules,
            players,
            position,
            phase: Phase::ToThrow,
            turn: Turn::new(),
            fallen: Vec::new(),
            selected: None,
            cursor: (12, 9),
            last: None,
            message: "Red throws first.".to_string(),
            rng,
        }
    }

    pub fn player(&self) -> Player {
        self.players[team(self.position.to_move)]
    }

    pub fn human(&self) -> bool {
        self.player() == Player::Human
    }

    pub fn tokens(&self) -> Vec<Token> {
        self.turn.left(&self.rules).0
    }

    fn moves(&self) -> Vec<Move> {
        self.position.moves(&self.rules, &self.tokens())
    }

    /// The colour whose men are being moved.
    pub fn moving(&self) -> usize {
        self.position.moving(&self.rules, self.position.to_move)
    }

    /// The cells lit for the person: men that may move, or where the one
    /// picked up may go.
    pub fn lit(&self) -> Vec<(u8, u8)> {
        if !(self.human() && self.phase == Phase::Playing) {
            return Vec::new();
        }
        let c = self.moving();
        let mut out: Vec<(u8, u8)> = self
            .moves()
            .iter()
            .filter(|m| self.selected.is_none_or(|s| m.from == s))
            .map(|m| cell(c, if self.selected.is_some() { m.to } else { m.from }))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn move_cursor(&mut self, dr: i8, dc: i8) {
        let (r, c) = (self.cursor.0 as i8 + dr, self.cursor.1 as i8 + dc);
        let on = |r: i8, c: i8| (0..19).contains(&r) && (0..19).contains(&c) && ((8..=10).contains(&r) || (8..=10).contains(&c));
        if on(r, c) {
            self.cursor = (r as u8, c as u8);
        }
    }

    /// Enter or Space, or a click on the cell `at`.
    pub fn act(&mut self, at: (u8, u8), now: Instant) {
        if !self.human() {
            return;
        }
        match self.phase {
            Phase::ToThrow => self.throw_now(now),
            Phase::Playing => self.pick(at),
            _ => {}
        }
    }

    /// The step of the moving colour's track drawn at `at`, where one of
    /// its men stands, if any.
    fn man_at(&self, at: (u8, u8)) -> Option<u8> {
        let c = self.moving();
        self.position.men[c].iter().copied().find(|&s| s != HOME && cell(c, s) == at)
    }

    fn pick(&mut self, at: (u8, u8)) {
        self.cursor = at;
        let c = self.moving();
        let moves = self.moves();
        if let Some(s) = self.selected {
            // Prefer the move of one man; a pair or three move only when
            // nothing else lands there.
            let mut fits: Vec<&Move> = moves.iter().filter(|m| m.from == s && cell(c, m.to) == at).collect();
            fits.sort_by_key(|m| m.men.len());
            if let Some(mv) = fits.first().map(|m| (*m).clone()) {
                self.play(mv);
                return;
            }
        }
        match self.man_at(at) {
            Some(s) if moves.iter().any(|m| m.from == s) => {
                self.selected = Some(s);
                self.message = "Now choose where it goes.".to_string();
            }
            Some(_) => {
                self.selected = None;
                self.message = "That man cannot use these throws.".to_string();
            }
            None => self.selected = None,
        }
    }

    /// Esc: lets go of the man picked up.
    pub fn cancel(&mut self) {
        self.selected = None;
    }

    /// P: leaves the rest of the throws unused.
    pub fn pass_rest(&mut self, now: Instant) {
        if self.human() && self.phase == Phase::Playing {
            self.message = format!("{} leaves the rest unused.", COLOURS[self.position.to_move]);
            self.end_play(now);
        }
    }

    fn describe(&self, mv: &Move) -> String {
        let who = COLOURS[mv.colour as usize];
        let men = if mv.men.len() == 1 { "a man".to_string() } else { format!("{} men together", mv.men.len()) };
        let place = |s: u8| match s {
            0 => "the middle".to_string(),
            READY => "out into the middle".to_string(),
            HOME => "home".to_string(),
            s => format!("square {s}"),
        };
        let mut text = if mv.to == READY {
            format!("{who} brings {men} out with a starting 1.")
        } else {
            format!("{who} moves {men} from {} to {}.", place(mv.from), place(mv.to))
        };
        if !mv.takes.is_empty() {
            let mut names: Vec<&str> = mv.takes.iter().map(|&(c, _)| COLOURS[c as usize]).collect();
            names.dedup();
            text.push_str(&format!(" It takes {} {} back to the middle!", mv.takes.len(), names.join(" and ")));
        }
        text
    }

    fn play(&mut self, mv: Move) {
        info!("{:?}", mv);
        self.message = self.describe(&mv);
        self.turn.spend(&self.rules, &mv.tokens);
        if self.rules.capture_throws && !mv.takes.is_empty() {
            self.turn.bonus += 1;
        }
        self.position.apply(&mv);
        self.last = Some(mv);
        self.selected = None;
        if let Some(w) = self.position.winner() {
            self.message.push_str(&format!(" {} have all their men home and win!", TEAMS[w]));
            self.phase = Phase::Over { winner: w };
            return;
        }
        if self.human() && self.moves().is_empty() && self.phase == Phase::Playing {
            self.end_play(Instant::now());
        }
    }

    fn end_play(&mut self, now: Instant) {
        self.selected = None;
        if self.turn.bonus > 0 {
            self.turn.bonus -= 1;
            self.turn.clear();
            self.message.push_str(" A man taken: another throw.");
            self.phase = Phase::ToThrow;
            return;
        }
        let _ = now;
        self.position.pass();
        self.turn = Turn::new();
        self.phase = Phase::ToThrow;
    }

    fn throw_now(&mut self, now: Instant) {
        self.fallen = throw(&self.rules, || self.rng.next());
        self.phase = Phase::Throwing { until: now + FALL };
    }

    fn landed(&mut self, now: Instant) {
        let colour = COLOURS[self.position.to_move];
        let struck_before = self.turn.struck;
        let again = self.turn.add(&self.rules, &self.fallen.clone());
        let shown: Vec<String> = self.fallen.iter().map(|t| t.value.to_string()).collect();
        self.message = format!("{colour} throws {}", shown.join(", "));
        if self.turn.struck > struck_before {
            self.message.push_str(": three high throws running are rotten, and struck out");
        }
        if again {
            self.message.push_str(", and throws again.");
            self.phase = Phase::ToThrow;
            return;
        }
        self.message.push('.');
        if self.moves().is_empty() {
            self.message.push_str(" Nothing can be moved.");
            self.phase = Phase::Stuck { until: now + PAUSE };
            return;
        }
        match self.player() {
            Player::Human => self.phase = Phase::Playing,
            Player::Computer(s) => {
                let plan = ai::plan(&self.rules, &self.position, &self.tokens(), s, &mut self.rng);
                self.phase = Phase::Showing { plan, next: now + STEP };
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        match &mut self.phase {
            Phase::ToThrow => {
                if !self.human() {
                    self.throw_now(now);
                }
            }
            Phase::Throwing { until } => {
                if now >= *until {
                    self.landed(now);
                }
            }
            Phase::Stuck { until } => {
                if now >= *until {
                    self.end_play(now);
                }
            }
            Phase::Showing { next, .. } => {
                if now >= *next {
                    let Phase::Showing { mut plan, .. } = std::mem::replace(&mut self.phase, Phase::Playing) else {
                        unreachable!()
                    };
                    if plan.is_empty() {
                        self.end_play(now);
                    } else {
                        let mv = plan.remove(0);
                        self.play(mv);
                        if !matches!(self.phase, Phase::Over { .. }) {
                            self.phase = Phase::Showing { plan, next: now + STEP };
                        }
                    }
                }
            }
            Phase::Playing | Phase::Over { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computers_play_to_the_end() {
        for rules in Rules::all() {
            let mut game = Game::new(
                rules.clone(),
                [Player::Computer(ai::Strength::Normal), Player::Computer(ai::Strength::Easy)],
                Rng::new(5),
            );
            let t0 = Instant::now();
            let mut i = 0u64;
            while !matches!(game.phase, Phase::Over { .. }) {
                game.tick(t0 + Duration::from_millis(500 * i));
                i += 1;
                assert!(i < 400_000, "{}", rules.id);
            }
        }
    }

    #[test]
    fn a_person_enters_a_man() {
        let mut game = Game::new(Rules::chaupar(), [Player::Human, Player::Human], Rng::new(3));
        let t0 = Instant::now();
        game.act((0, 0), t0);
        game.tick(t0 + Duration::from_secs(1));
        if game.phase == Phase::Playing {
            let lit = game.lit();
            assert!(!lit.is_empty());
            game.act(lit[0], t0);
            let to = game.lit();
            assert!(!to.is_empty());
            game.act(to[0], t0);
            assert!(game.last.is_some());
        }
    }
}
