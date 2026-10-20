//! The game around the rules: a turn is choosing a pharaoh, fighting the
//! lion on it if there is one, hiding marbles and guessing, the hand
//! opening, and the guessers' choices after a right guess. With two people
//! at one keyboard, one looks away while the other hides.

use crate::ai::{self, Habits};
use crate::rules::{Rules, State, Team};
pub use ancient_core::player::Player;
pub use ancient_core::rng::Rng;
use log::info;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// The team to move picks which pharaoh moves.
    Choose { options: Vec<usize>, selected: usize },
    /// `team` is about to hide; the other team looks away.
    LookAway { team: Team, then: Box<Phase> },
    /// The lion's team hides 1 to 3 marbles to fight pharaoh `p`'s team.
    FightHide { p: usize },
    FightGuess { p: usize, hidden: u8 },
    /// Pharaoh `p`'s team hides 1 to 5 marbles.
    Hide { p: usize },
    Guess { p: usize, hidden: u8 },
    /// The hand is open.
    Reveal { p: usize, hidden: u8, guess: u8, fight: bool, until: Instant },
    /// After a right guess of `steps`: the guessers pick their pharaoh.
    Pick { p: usize, steps: u8, options: Vec<usize>, selected: usize },
    /// ... and whether to set a lion on pharaoh `p`.
    Unleash { p: usize, steps: u8, mover: Option<usize> },
    Over { winner: Team },
}

pub struct Game {
    pub rules: Rules,
    /// Who plays the lions and who the lionesses.
    pub players: [Player; 2],
    pub state: State,
    pub phase: Phase,
    /// The number being chosen with the arrows.
    pub number: u8,
    pub habits: [Habits; 2],
    pub message: String,
    /// The last hand, to keep showing it.
    pub last_hand: Option<(Team, u8, u8)>,
    /// The computer acts no sooner than this.
    wait: Instant,
    pub rng: Rng,
}

const THINK: Duration = Duration::from_millis(700);
const SHOW: Duration = Duration::from_millis(1600);

impl Game {
    pub fn new(rules: Rules, players: [Player; 2], mut rng: Rng, now: Instant) -> Game {
        let first = if rng.coin() { Team::Lions } else { Team::Lionesses };
        let state = State::new(&rules, first);
        info!("new game: {} {:?}, {} first", rules.id, players, first.name());
        let mut game = Game {
            rules,
            players,
            state,
            phase: Phase::Over { winner: first },
            number: 3,
            habits: Default::default(),
            message: format!("{} move first.", first.title()),
            last_hand: None,
            wait: now,
            rng,
        };
        game.start_turn(now);
        game
    }

    pub fn player(&self, team: Team) -> Player {
        self.players[team.index()]
    }

    /// The team that acts in the present phase.
    pub fn acting(&self) -> Option<Team> {
        let team_of = |p: usize| self.state.team_of(p);
        match &self.phase {
            Phase::Choose { .. } => Some(self.state.to_move),
            Phase::LookAway { team, .. } => Some(*team),
            Phase::FightHide { p } => Some(team_of(*p).other()),
            Phase::FightGuess { p, .. } => Some(team_of(*p)),
            Phase::Hide { p } => Some(team_of(*p)),
            Phase::Guess { p, .. } => Some(team_of(*p).other()),
            Phase::Pick { p, .. } | Phase::Unleash { p, .. } => Some(team_of(*p).other()),
            Phase::Reveal { .. } | Phase::Over { .. } => None,
        }
    }

    pub fn human_acting(&self) -> bool {
        self.acting().is_some_and(|t| self.player(t) == Player::Human)
    }

    fn both_human(&self) -> bool {
        self.players.iter().all(|p| *p == Player::Human)
    }

    fn start_turn(&mut self, now: Instant) {
        let team = self.state.to_move;
        let options = self.state.travelling(team);
        if options.len() == 1 {
            self.begin_move(options[0], now);
        } else {
            self.phase = Phase::Choose { options, selected: 0 };
            self.wait = now + THINK;
        }
    }

    fn begin_move(&mut self, p: usize, now: Instant) {
        let phase = if self.state.attacker(p).is_some() { Phase::FightHide { p } } else { Phase::Hide { p } };
        self.enter(phase, now);
    }

    /// Enters a hiding phase, with someone looking away first if both
    /// teams are people.
    fn enter(&mut self, phase: Phase, now: Instant) {
        self.number = if matches!(phase, Phase::FightHide { .. }) { 2 } else { 3 };
        self.wait = now + THINK;
        self.phase = phase.clone();
        if let Some(team) = self.acting()
            && self.both_human()
            && matches!(phase, Phase::Hide { .. } | Phase::FightHide { .. })
        {
            self.phase = Phase::LookAway { team, then: Box::new(phase) };
        }
    }

    /// The person picks a choice with the arrows: a pharaoh or a number.
    pub fn step(&mut self, delta: isize) {
        match &mut self.phase {
            Phase::Choose { options, selected } | Phase::Pick { options, selected, .. } => {
                *selected = (*selected as isize + delta).rem_euclid(options.len() as isize) as usize;
            }
            Phase::FightHide { .. } | Phase::FightGuess { .. } => {
                self.number = (self.number as isize + delta).clamp(1, 3) as u8;
            }
            Phase::Hide { .. } | Phase::Guess { .. } => {
                self.number = (self.number as isize + delta).clamp(1, 5) as u8;
            }
            _ => {}
        }
    }

    /// Picks pharaoh `p` in a choice, if it is one.
    pub fn select_pharaoh(&mut self, p: usize) -> bool {
        if let Phase::Choose { options, selected } | Phase::Pick { options, selected, .. } = &mut self.phase
            && let Some(i) = options.iter().position(|&o| o == p)
        {
            *selected = i;
            return true;
        }
        false
    }

    /// Enter: confirms the choice on screen.
    pub fn confirm(&mut self, now: Instant) {
        if !self.human_acting() {
            return;
        }
        match self.phase.clone() {
            Phase::Choose { options, selected } => self.begin_move(options[selected], now),
            Phase::LookAway { then, .. } => self.phase = *then,
            Phase::Pick { p, steps, options, selected } => self.picked(p, steps, Some(options[selected]), now),
            Phase::Unleash { .. } => self.answer(true, now),
            Phase::FightHide { .. } | Phase::FightGuess { .. } | Phase::Hide { .. } | Phase::Guess { .. } => {
                self.number_chosen(self.number, now)
            }
            _ => {}
        }
    }

    /// A number key, or Enter on a number.
    pub fn number_chosen(&mut self, n: u8, now: Instant) {
        match self.phase.clone() {
            Phase::FightHide { p } if (1..=3).contains(&n) => {
                self.phase = Phase::FightGuess { p, hidden: n };
                self.number = 2;
                self.wait = now + THINK;
            }
            Phase::FightGuess { p, hidden } if (1..=3).contains(&n) => {
                self.reveal(p, hidden, n, true, now);
            }
            Phase::Hide { p } if (1..=5).contains(&n) => {
                self.habits[self.state.team_of(p).index()].hides[n as usize - 1] += 1;
                self.phase = Phase::Guess { p, hidden: n };
                self.number = 3;
                self.wait = now + THINK;
            }
            Phase::Guess { p, hidden } if (1..=5).contains(&n) => {
                self.habits[self.state.team_of(p).other().index()].guesses[n as usize - 1] += 1;
                self.reveal(p, hidden, n, false, now);
            }
            _ => {}
        }
    }

    fn reveal(&mut self, p: usize, hidden: u8, guess: u8, fight: bool, now: Instant) {
        let hider = if fight { self.state.team_of(p).other() } else { self.state.team_of(p) };
        info!("{} hid {hidden}, guessed {guess}, fight {fight}", hider.name());
        self.last_hand = Some((hider, hidden, guess));
        let right = hidden == guess;
        self.message = format!(
            "{} hid {hidden}{}. {} guessed {guess}: {}",
            hider.title(),
            if hidden == 1 { " marble" } else { " marbles" },
            hider.other().title(),
            if right { "right!" } else { "wrong." }
        );
        self.phase = Phase::Reveal { p, hidden, guess, fight, until: now + SHOW };
    }

    /// The hand has been seen: what it does.
    fn settle(&mut self, p: usize, hidden: u8, guess: u8, fight: bool, now: Instant) {
        let team = self.state.team_of(p);
        if fight {
            if hidden == guess {
                self.state.kill_lion(p);
                self.message = format!("{} kill the lion! Now the move.", team.title());
                self.enter(Phase::Hide { p }, now);
            } else {
                self.message = format!("The lion holds {}'s pharaoh fast.", team.name());
                self.pass(now);
            }
            return;
        }
        let guessers = team.other();
        if hidden != guess {
            self.state.record_guess(guessers, false);
            let landing = self.state.advance(p, hidden);
            self.message = format!("{} move {hidden}.{}", team.title(), self.landing_text(landing, team));
            self.after_move(now);
            return;
        }
        if self.state.record_guess(guessers, true) {
            self.message = format!("Two right in a row: one of {}' dead lions comes back.", guessers.name());
        } else {
            self.message = format!("{} guessed right and move {hidden}.", guessers.title());
        }
        let options = self.state.free(guessers);
        match self.player(guessers) {
            Player::Human if options.len() > 1 => {
                self.phase = Phase::Pick { p, steps: hidden, options, selected: 0 };
            }
            Player::Human => self.picked(p, hidden, options.first().copied(), now),
            Player::Computer(strength) => {
                let mover = ai::guesser_piece(&self.state, guessers, hidden);
                let unleash = self.state.can_unleash(guessers, p) && ai::unleash(strength, &mut self.rng);
                self.finish_right_guess(p, hidden, mover, unleash, now);
            }
        }
    }

    fn picked(&mut self, p: usize, steps: u8, mover: Option<usize>, now: Instant) {
        let guessers = self.state.team_of(p).other();
        if self.state.can_unleash(guessers, p) {
            self.phase = Phase::Unleash { p, steps, mover };
            self.wait = now;
        } else {
            self.finish_right_guess(p, steps, mover, false, now);
        }
    }

    /// Y or N to setting a lion loose.
    pub fn answer(&mut self, yes: bool, now: Instant) {
        if let Phase::Unleash { p, steps, mover } = self.phase {
            self.finish_right_guess(p, steps, mover, yes, now);
        }
    }

    fn finish_right_guess(&mut self, p: usize, steps: u8, mover: Option<usize>, unleash: bool, now: Instant) {
        let guessers = self.state.team_of(p).other();
        let mut text = String::new();
        match mover {
            Some(q) => {
                let landing = self.state.advance(q, steps);
                text.push_str(&format!(" {} move {steps}.{}", guessers.title(), self.landing_text(landing, guessers)));
            }
            None => text.push_str(&format!(" Every pharaoh of {} is held by a lion; none moves.", guessers.name())),
        }
        if unleash && self.state.unleash(guessers, p) {
            text.push_str(&format!(" A {} pounces on {}' pharaoh!", lion_word(guessers), guessers.other().name()));
        }
        self.message = text.trim_start().to_string();
        self.after_move(now);
    }

    fn landing_text(&self, landing: crate::rules::Landing, team: Team) -> String {
        if landing.home {
            return " A pharaoh reaches the head and is reborn!".to_string();
        }
        match landing.swapped {
            Some(q) if self.state.pharaohs[q].pos == 0 => {
                format!(" It lands on {}' pharaoh, which must start again.", team.other().name())
            }
            Some(_) => format!(" It lands on {}' pharaoh: they change places.", team.other().name()),
            None => String::new(),
        }
    }

    fn after_move(&mut self, now: Instant) {
        if let Some(winner) = self.state.winner() {
            info!("{} win", winner.name());
            self.message.push_str(&format!(" {} are all reborn: they win!", winner.title()));
            self.phase = Phase::Over { winner };
            return;
        }
        self.pass(now);
    }

    fn pass(&mut self, now: Instant) {
        self.state.to_move = self.state.to_move.other();
        self.start_turn(now);
    }

    pub fn tick(&mut self, now: Instant) {
        if let Phase::Reveal { p, hidden, guess, fight, until } = self.phase {
            if now >= until {
                self.settle(p, hidden, guess, fight, now);
            }
            return;
        }
        let Some(team) = self.acting() else { return };
        let Player::Computer(strength) = self.player(team) else { return };
        if now < self.wait {
            return;
        }
        let them = self.habits[team.other().index()].clone();
        match self.phase.clone() {
            Phase::Choose { .. } => {
                if let Some(p) = ai::mover(&self.state, team) {
                    self.begin_move(p, now);
                }
            }
            Phase::FightHide { .. } | Phase::FightGuess { .. } => {
                let n = ai::fight(&mut self.rng);
                self.number_chosen(n, now);
            }
            Phase::Hide { p } => {
                let lion = self.state.can_unleash(team.other(), p);
                let n = ai::hide(strength, lion, &them, &mut self.rng);
                self.number_chosen(n, now);
            }
            Phase::Guess { p, .. } => {
                // The guess is made without looking at the hidden number.
                let lion = self.state.can_unleash(team, p);
                let n = ai::guess(strength, lion, &them, &mut self.rng);
                self.number_chosen(n, now);
            }
            _ => {}
        }
    }
}

/// A lion or a lioness, by team.
pub fn lion_word(team: Team) -> &'static str {
    match team {
        Team::Lions => "lion",
        Team::Lionesses => "lioness",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ancient_core::player::Strength;

    #[test]
    fn computer_games_end() {
        for rules in Rules::all() {
            for seed in 1..=4 {
                let players = [Player::Computer(Strength::Hard), Player::Computer(Strength::Easy)];
                let mut now = Instant::now();
                let mut game = Game::new(rules.clone(), players, Rng::new(seed), now);
                for _ in 0..200_000 {
                    if matches!(game.phase, Phase::Over { .. }) {
                        break;
                    }
                    now += Duration::from_millis(250);
                    game.tick(now);
                }
                assert!(matches!(game.phase, Phase::Over { .. }), "{} seed {seed}", rules.id);
            }
        }
    }

    #[test]
    fn a_person_hides_and_the_computer_guesses() {
        let rules = Rules::all()[0].clone();
        let mut now = Instant::now();
        let players = [Player::Human, Player::Computer(Strength::Normal)];
        let mut game = Game::new(rules, players, Rng::new(2), now);
        // Play until it is the lions' (the person's) turn to hide.
        for _ in 0..1000 {
            if matches!(game.phase, Phase::Hide { p } if game.state.team_of(p) == Team::Lions) {
                break;
            }
            match game.phase {
                Phase::Guess { .. } | Phase::FightGuess { .. } | Phase::FightHide { .. } if game.human_acting() => {
                    game.number_chosen(1, now)
                }
                Phase::Unleash { .. } => game.answer(true, now),
                _ => {}
            }
            now += Duration::from_millis(500);
            game.tick(now);
        }
        game.number_chosen(4, now);
        assert!(matches!(game.phase, Phase::Guess { hidden: 4, .. }));
        now += Duration::from_secs(1);
        game.tick(now);
        assert!(matches!(game.phase, Phase::Reveal { hidden: 4, .. }));
    }
}

#[cfg(test)]
mod measure {
    use super::*;
    use ancient_core::player::Strength;

    /// Wins for the first strength (playing either team) out of `games`,
    /// and the average number of turns.
    fn duel(rules: &Rules, a: Strength, b: Strength, games: u64) -> (u64, u64) {
        let mut wins = 0;
        let mut turns = 0;
        for seed in 0..games {
            let a_lions = seed % 2 == 0;
            let players = if a_lions {
                [Player::Computer(a), Player::Computer(b)]
            } else {
                [Player::Computer(b), Player::Computer(a)]
            };
            let mut now = Instant::now();
            let mut game = Game::new(rules.clone(), players, Rng::new(seed + 1), now);
            let mut moves = 0;
            while !matches!(game.phase, Phase::Over { .. }) {
                if matches!(game.phase, Phase::Reveal { .. }) {
                    moves += 1;
                }
                now += Duration::from_secs(2);
                game.tick(now);
            }
            let Phase::Over { winner } = game.phase else { unreachable!() };
            if (winner == Team::Lions) == a_lions {
                wins += 1;
            }
            turns += moves;
        }
        (wins, turns / games)
    }

    #[test]
    #[ignore]
    fn strengths() {
        use Strength::*;
        for rules in Rules::all() {
            for (a, b) in [(Normal, Easy), (Hard, Easy), (Hard, Normal)] {
                let (w, t) = duel(&rules, a, b, 400);
                println!("{}: {} beats {} {w}/400, about {t} hands a game", rules.id, a.name(), b.name());
            }
        }
    }
}
