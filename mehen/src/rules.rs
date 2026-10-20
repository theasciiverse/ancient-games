//! The rules of Mehen, the Egyptian game of the coiled snake, in James F. R.
//! Masters' reconstruction (2024): pharaohs race from the tail to the head,
//! moved by guessing how many marbles are hidden in a fist, and lions are
//! set on the pharaohs of the other team.
//!
//! Squares are numbered from 1 at the tail to `SQUARES` at the head; 0 is
//! the start beside the tail, and `HOME` the afterlife beyond the head.

/// The two teams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    Lions,
    Lionesses,
}

impl Team {
    pub fn other(self) -> Team {
        match self {
            Team::Lions => Team::Lionesses,
            Team::Lionesses => Team::Lions,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Team::Lions => 0,
            Team::Lionesses => 1,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Team::Lions => "the lions",
            Team::Lionesses => "the lionesses",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Team::Lions => "The lions",
            Team::Lionesses => "The lionesses",
        }
    }
}

/// The squares of the snake: 70, as Masters recommends for play.
pub const SQUARES: u8 = 70;
/// Reborn into the afterlife.
pub const HOME: u8 = SQUARES + 1;
/// Each team's big cats.
pub const LIONS: usize = 3;

/// A rule set: the reconstruction for two, four or six players.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub id: &'static str,
    pub name: &'static str,
    pub source: &'static str,
    /// Pharaohs (players) in each team.
    pub pharaohs: usize,
}

const SOURCE: &str = "James F. R. Masters' reconstruction (2024): the marble guessing his \
                      reading of the evidence, the lions his own suggestion. No ancient rules \
                      survive.";

impl Rules {
    pub fn all() -> Vec<Rules> {
        vec![
            Rules { id: "two", name: "Masters 2024, two players", source: SOURCE, pharaohs: 1 },
            Rules { id: "four", name: "Masters 2024, four players", source: SOURCE, pharaohs: 2 },
            Rules { id: "six", name: "Masters 2024, six players", source: SOURCE, pharaohs: 3 },
        ]
    }
}

/// A pharaoh: a marble on the snake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pharaoh {
    pub team: Team,
    pub pos: u8,
}

/// Where a lion is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lion {
    Reserve,
    /// Attacking that pharaoh.
    Attacking(usize),
    Dead,
}

/// What a pharaoh's move did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Landing {
    /// The opposing pharaoh landed on, sent back to where the mover came from.
    pub swapped: Option<usize>,
    /// Teammates hopped over.
    pub hops: u8,
    pub home: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The lions' pharaohs first, then the lionesses'.
    pub pharaohs: Vec<Pharaoh>,
    pub lions: [[Lion; LIONS]; 2],
    /// Right guesses in a row for a move, per team.
    pub streak: [u8; 2],
    pub to_move: Team,
}

impl State {
    pub fn new(rules: &Rules, first: Team) -> State {
        let mut pharaohs = Vec::new();
        for team in [Team::Lions, Team::Lionesses] {
            for _ in 0..rules.pharaohs {
                pharaohs.push(Pharaoh { team, pos: 0 });
            }
        }
        State { pharaohs, lions: [[Lion::Reserve; LIONS]; 2], streak: [0; 2], to_move: first }
    }

    pub fn team_of(&self, p: usize) -> Team {
        self.pharaohs[p].team
    }

    /// The pharaohs of a team still on their way.
    pub fn travelling(&self, team: Team) -> Vec<usize> {
        (0..self.pharaohs.len())
            .filter(|&p| self.pharaohs[p].team == team && self.pharaohs[p].pos != HOME)
            .collect()
    }

    /// The lion attacking pharaoh `p`: its team and number.
    pub fn attacker(&self, p: usize) -> Option<(Team, usize)> {
        for team in [Team::Lions, Team::Lionesses] {
            for (i, lion) in self.lions[team.index()].iter().enumerate() {
                if *lion == Lion::Attacking(p) {
                    return Some((team, i));
                }
            }
        }
        None
    }

    /// Travelling pharaohs of `team` with no lion on them.
    pub fn free(&self, team: Team) -> Vec<usize> {
        self.travelling(team).into_iter().filter(|&p| self.attacker(p).is_none()).collect()
    }

    pub fn count(&self, team: Team, state: Lion) -> usize {
        self.lions[team.index()]
            .iter()
            .filter(|l| match state {
                Lion::Attacking(_) => matches!(l, Lion::Attacking(_)),
                s => **l == s,
            })
            .count()
    }

    /// Whether `team` can set a lion on pharaoh `target`.
    pub fn can_unleash(&self, team: Team, target: usize) -> bool {
        self.count(team, Lion::Reserve) > 0
            && self.attacker(target).is_none()
            && self.pharaohs[target].pos != HOME
    }

    pub fn unleash(&mut self, team: Team, target: usize) -> bool {
        if !self.can_unleash(team, target) {
            return false;
        }
        let lions = &mut self.lions[team.index()];
        let i = lions.iter().position(|l| *l == Lion::Reserve).expect("a lion in reserve");
        lions[i] = Lion::Attacking(target);
        true
    }

    /// The lion on `p` is killed and goes to the graveyard.
    pub fn kill_lion(&mut self, p: usize) {
        if let Some((team, i)) = self.attacker(p) {
            self.lions[team.index()][i] = Lion::Dead;
        }
    }

    /// Records a guess for a move by `team`; two right in a row bring back
    /// one of its dead lions. Returns whether one came back.
    pub fn record_guess(&mut self, team: Team, right: bool) -> bool {
        let streak = &mut self.streak[team.index()];
        if !right {
            *streak = 0;
            return false;
        }
        *streak += 1;
        if *streak < 2 {
            return false;
        }
        *streak = 0;
        let lions = &mut self.lions[team.index()];
        match lions.iter().position(|l| *l == Lion::Dead) {
            Some(i) => {
                lions[i] = Lion::Reserve;
                true
            }
            None => false,
        }
    }

    /// Moves pharaoh `p` `steps` squares towards the head: past teammates,
    /// changing places with an opponent landed on, and home on reaching the
    /// head (no exact throw needed).
    pub fn advance(&mut self, p: usize, steps: u8) -> Landing {
        let from = self.pharaohs[p].pos;
        let team = self.pharaohs[p].team;
        let mut to = from + steps;
        let mut landing = Landing::default();
        loop {
            if to >= SQUARES {
                self.pharaohs[p].pos = HOME;
                landing.home = true;
                return landing;
            }
            match (0..self.pharaohs.len()).find(|&q| q != p && self.pharaohs[q].pos == to) {
                Some(q) if self.pharaohs[q].team == team => {
                    to += 1;
                    landing.hops += 1;
                }
                Some(q) => {
                    // The other goes back to where the mover came from: off
                    // the board again if it was entering. A lion on it
                    // follows it.
                    self.pharaohs[q].pos = from;
                    self.pharaohs[p].pos = to;
                    landing.swapped = Some(q);
                    return landing;
                }
                None => {
                    self.pharaohs[p].pos = to;
                    return landing;
                }
            }
        }
    }

    /// The team with all its pharaohs reborn.
    pub fn winner(&self) -> Option<Team> {
        [Team::Lions, Team::Lionesses].into_iter().find(|&t| self.travelling(t).is_empty())
    }
}

/// The squares' places on a grid, from the tail at the top left coiling
/// inwards to the head, one empty row or column between the coils.
pub fn coil(width: usize, height: usize) -> Vec<(usize, usize)> {
    const DIRS: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let mut seen = vec![false; width * height];
    let (mut r, mut c, mut d) = (0isize, 0isize, 0usize);
    seen[0] = true;
    let mut path = vec![(0, 0)];
    let free = |seen: &[bool], r: isize, c: isize| {
        r >= 0 && c >= 0 && (r as usize) < height && (c as usize) < width && !seen[r as usize * width + c as usize]
    };
    let mut turns = 0;
    while turns < 2 {
        let (nr, nc) = (r + DIRS[d].0, c + DIRS[d].1);
        let (ar, ac) = (nr + DIRS[d].0, nc + DIRS[d].1);
        let beyond_taken = ar >= 0
            && ac >= 0
            && (ar as usize) < height
            && (ac as usize) < width
            && seen[ar as usize * width + ac as usize];
        if free(&seen, nr, nc) && !beyond_taken {
            r = nr;
            c = nc;
            seen[r as usize * width + c as usize] = true;
            path.push((r as usize, c as usize));
            turns = 0;
        } else {
            d = (d + 1) % 4;
            turns += 1;
        }
    }
    path
}

/// The board: 14 by 8 coils to exactly 70 squares.
pub const GRID: (usize, usize) = (14, 8);

#[cfg(test)]
mod tests {
    use super::*;

    fn two() -> Rules {
        Rules::all()[0].clone()
    }

    #[test]
    fn the_coil_has_seventy_squares_and_ends_inside() {
        let path = coil(GRID.0, GRID.1);
        assert_eq!(path.len(), SQUARES as usize);
        let (r, c) = *path.last().unwrap();
        assert!(r > 0 && r < GRID.1 - 1 && c > 0 && c < GRID.0 - 1);
        // Each square touches the next.
        for w in path.windows(2) {
            let d = w[0].0.abs_diff(w[1].0) + w[0].1.abs_diff(w[1].1);
            assert_eq!(d, 1);
        }
    }

    #[test]
    fn landing_on_an_opponent_changes_places() {
        let mut s = State::new(&two(), Team::Lions);
        s.pharaohs[0].pos = 10;
        s.pharaohs[1].pos = 13;
        let l = s.advance(0, 3);
        assert_eq!(l.swapped, Some(1));
        assert_eq!((s.pharaohs[0].pos, s.pharaohs[1].pos), (13, 10));
        // Entering onto an opponent sends it off the board.
        let mut s = State::new(&two(), Team::Lions);
        s.pharaohs[1].pos = 4;
        s.advance(0, 4);
        assert_eq!((s.pharaohs[0].pos, s.pharaohs[1].pos), (4, 0));
    }

    #[test]
    fn teammates_are_hopped_and_the_head_needs_no_exact_throw() {
        let rules = Rules::all()[1].clone();
        let mut s = State::new(&rules, Team::Lions);
        s.pharaohs[0].pos = 5;
        s.pharaohs[1].pos = 8;
        s.pharaohs[2].pos = 9; // a lioness
        let l = s.advance(0, 3);
        assert_eq!(l.hops, 1);
        assert_eq!(l.swapped, Some(2));
        assert_eq!((s.pharaohs[0].pos, s.pharaohs[2].pos), (9, 5));
        s.pharaohs[0].pos = 67;
        assert!(s.advance(0, 5).home);
    }

    #[test]
    fn lions_attack_die_and_come_back() {
        let mut s = State::new(&two(), Team::Lions);
        assert!(s.unleash(Team::Lionesses, 0));
        assert!(!s.can_unleash(Team::Lionesses, 0), "one lion at a time");
        assert!(s.free(Team::Lions).is_empty());
        // The lion follows its pharaoh when it is sent back.
        s.pharaohs[0].pos = 6;
        s.pharaohs[1].pos = 3;
        s.advance(1, 3);
        assert_eq!(s.pharaohs[0].pos, 3);
        assert!(s.attacker(0).is_some());
        s.kill_lion(0);
        assert_eq!(s.count(Team::Lionesses, Lion::Dead), 1);
        assert!(!s.record_guess(Team::Lionesses, true));
        assert!(!s.record_guess(Team::Lionesses, false));
        assert!(!s.record_guess(Team::Lionesses, true));
        assert!(s.record_guess(Team::Lionesses, true));
        assert_eq!(s.count(Team::Lionesses, Lion::Reserve), LIONS);
    }

    #[test]
    fn the_winner_has_every_pharaoh_home() {
        let rules = Rules::all()[1].clone();
        let mut s = State::new(&rules, Team::Lions);
        s.pharaohs[0].pos = HOME;
        assert_eq!(s.winner(), None);
        s.pharaohs[1].pos = HOME;
        assert_eq!(s.winner(), Some(Team::Lions));
    }
}
