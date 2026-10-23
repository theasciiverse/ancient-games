//! The computer opponent: alpha-beta search with iterative deepening and a
//! table of positions searched, deeper as it gets harder. Morris has no
//! dice, so the search sees exactly what the rules will do.
//!
//! The judgement of a position: men (on the board and in hand) above all,
//! then mills one move from closing, mills already standing, and room to
//! move. In Three Men's Morris, where a line wins, only the lines count.

pub use ancient_core::player::Strength;

use crate::rules::{Board, Move, Outcome, Position, Rules, Side, bits};
use ancient_core::rng::Rng;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// How often a beginner plays any legal move instead of its best, in
/// sixteenths.
const BEGINNER_SLIPS: u64 = 4;

/// What returning to a position already seen costs, in the mover's eyes: a
/// man is worth 1000, so this only breaks ties and endless shuffling.
const REPEAT: i32 = 120;

const WIN: i32 = 1_000_000;

/// How long the computer may think, and how deep at most, in moves.
fn budget(strength: Strength) -> (Duration, u32) {
    let (time, depth) = match strength {
        Strength::Easy => (Duration::from_millis(60), 2),
        Strength::Normal => (Duration::from_millis(400), 4),
        Strength::Hard => (Duration::from_millis(1500), 40),
    };
    // The rulebook's pictures search to a fixed depth, the same on every
    // machine, and no deeper than normal so they are made in good time.
    if ancient_core::capture::untimed() { (Duration::MAX, depth.min(4)) } else { (time, depth) }
}

/// The best move for the side to move, searching deeper while time allows.
/// `seen` counts the positions of the game so far, to keep away from
/// repeating them.
pub fn choose(
    rules: &Rules,
    board: &Board,
    position: &Position,
    strength: Strength,
    seen: &HashMap<Position, u32>,
    rng: &mut Rng,
) -> Option<Move> {
    let moves = position.moves(rules, board);
    if moves.len() <= 1 {
        return moves.first().copied();
    }
    if strength == Strength::Easy && rng.next() % 16 < BEGINNER_SLIPS {
        return Some(moves[(rng.next() % moves.len() as u64) as usize]);
    }
    let (time, max_depth) = budget(strength);
    let mut search = Search { rules, board, start: Instant::now(), time, table: HashMap::new(), out_of_time: false };
    let mut order = moves.clone();
    order.sort_by_key(|m| std::cmp::Reverse(m.taken().count()));
    let mut best = order[0];
    for depth in 1..=max_depth {
        if let Some(i) = order.iter().position(|m| *m == best) {
            order[..=i].rotate_right(1);
        }
        let mut alpha = -WIN * 2;
        let mut found: Option<(i32, Move)> = None;
        for &mv in &order {
            let mut next = position.clone();
            next.play(rules, board, &mv);
            // Searched against the best so far less one: a worse move then
            // only proves itself worse, but a move as good gets its true
            // value, so equals stay equal for the noise below to choose.
            let value = -search.negamax(&next, depth - 1, -WIN * 2, -(alpha - 1));
            if search.out_of_time && depth > 1 {
                break;
            }
            let value = value - REPEAT * *seen.get(&next).unwrap_or(&0) as i32;
            // A little noise among equals, so the computer does not always
            // open the same way.
            let value = value * 4 + (rng.next() % 3) as i32;
            if found.is_none_or(|(v, _)| value > v) {
                found = Some((value, mv));
                alpha = alpha.max(value / 4);
            }
        }
        if search.out_of_time && depth > 1 {
            break;
        }
        if let Some((value, mv)) = found {
            best = mv;
            log::debug!("depth {depth}: {mv:?} at {}, {:?}", value / 4, search.start.elapsed());
            if (value / 4).abs() >= WIN - 1000 {
                break;
            }
        }
        if search.start.elapsed() > time {
            break;
        }
    }
    Some(best)
}

/// What a stored value is: exact, or only a bound because the search of
/// that position was cut short.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Bound {
    Exact,
    AtLeast,
    AtMost,
}

struct Search<'a> {
    rules: &'a Rules,
    board: &'a Board,
    start: Instant,
    time: Duration,
    /// Positions already searched: the depth, the value for the side to
    /// move there, whether that value is exact or a bound, and the best
    /// move found, tried first next time.
    table: HashMap<Position, (u32, i32, Bound, Option<Move>)>,
    out_of_time: bool,
}

impl Search<'_> {
    /// The value of `position` for the side to move there.
    fn negamax(&mut self, position: &Position, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        let me = position.to_move;
        let (end, mut moves) = position.status(self.rules, self.board);
        if let Some(outcome) = end {
            // Sooner wins are better: the depth left counts.
            return match outcome {
                Outcome::Won(side, _) if side == me => WIN - 100 + depth as i32,
                Outcome::Won(..) => -(WIN - 100 + depth as i32),
                Outcome::Drawn(_) => 0,
            };
        }
        if depth == 0 || self.out_of_time {
            return self.quiet(position, moves, alpha, beta, QUIET_DEPTH);
        }
        if self.start.elapsed() > self.time {
            self.out_of_time = true;
            return judge(self.rules, self.board, position, me);
        }
        let alpha_in = alpha;
        let mut first = None;
        if let Some(&(d, v, bound, mv)) = self.table.get(position) {
            if d >= depth {
                match bound {
                    Bound::Exact => return v,
                    Bound::AtLeast if v >= beta => return v,
                    Bound::AtMost if v <= alpha => return v,
                    _ => {}
                }
            }
            first = mv;
        }
        // Last time's best first, then the moves that take a man.
        moves.sort_by_key(|m| (Some(*m) != first, std::cmp::Reverse(m.taken().count())));
        let mut best = -WIN * 2;
        let mut best_move = None;
        for mv in moves {
            let mut next = position.clone();
            next.play(self.rules, self.board, &mv);
            let value = -self.negamax(&next, depth - 1, -beta, -alpha);
            if value > best {
                best = value;
                best_move = Some(mv);
            }
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        if !self.out_of_time {
            let bound = if best >= beta {
                Bound::AtLeast
            } else if best <= alpha_in {
                Bound::AtMost
            } else {
                Bound::Exact
            };
            self.table.insert(position.clone(), (depth, best, bound, best_move));
        }
        best
    }
}

impl Search<'_> {
    /// The value of a position at the end of the search: as judged, unless
    /// the side to move can take a man, when the takes are searched on (a
    /// few moves at most) so a mill about to close is not overlooked.
    fn quiet(&mut self, position: &Position, moves: Vec<Move>, mut alpha: i32, beta: i32, left: u32) -> i32 {
        let me = position.to_move;
        let stand = judge(self.rules, self.board, position, me);
        if left == 0 || stand >= beta {
            return stand;
        }
        alpha = alpha.max(stand);
        let mut best = stand;
        for mv in moves.into_iter().filter(|m| m.take[0].is_some()) {
            let mut next = position.clone();
            next.play(self.rules, self.board, &mv);
            let (end, replies) = next.status(self.rules, self.board);
            let value = match end {
                Some(Outcome::Won(side, _)) if side == me => WIN - 100,
                Some(Outcome::Won(..)) => -(WIN - 100),
                Some(Outcome::Drawn(_)) => 0,
                None => -self.quiet(&next, replies, -beta, -alpha, left - 1),
            };
            best = best.max(value);
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        best
    }
}

/// How many takes in a row the end of a search follows.
const QUIET_DEPTH: u32 = 4;

/// How good `position` is for `side`.
fn judge(rules: &Rules, board: &Board, position: &Position, side: Side) -> i32 {
    let ours = for_side(rules, board, position, side);
    let theirs = for_side(rules, board, position, side.other());
    ours - theirs
}

/// What one side has: men, mills nearly made and made, and room.
fn for_side(rules: &Rules, board: &Board, position: &Position, side: Side) -> i32 {
    let mine = position.men[side.index()];
    let empty = position.empty(board);
    let entering = position.hand[side.index()] > 0;
    let flies = position.flies(rules, side);
    // Lines of two men and an empty point that a man can reach: any, while
    // entering or flying; else a man of ours next to it, off that line.
    let threat = |m: u32| {
        let gap = m & empty;
        if (mine & m).count_ones() != 2 || gap == 0 {
            return false;
        }
        let gap = gap.trailing_zeros() as usize;
        entering || flies || board.adjacent[gap] & mine & !m != 0
    };
    let open = board.mills.iter().filter(|&&m| threat(m)).count() as i32;
    if rules.mill_wins {
        // The middle point is on the most lines.
        let middle = board.point_at(1, 1).map_or(0, |c| (mine >> c) as i32 & 1);
        return open * 50 + middle * 20;
    }
    let mut value = position.count(side) as i32 * 1000;
    let made = board.mills.iter().filter(|&&m| mine & m == m).count() as i32;
    value += made * 40;
    value += open * if entering { 90 } else { 120 };
    if !flies && !entering {
        let room: i32 = bits(mine).map(|p| (board.adjacent[p] & empty).count_ones() as i32).sum();
        let stuck = bits(mine).filter(|&p| board.adjacent[p] & empty == 0).count() as i32;
        value += room * 12 - stuck * 25;
        // A man that swings between two mills, taking every move (a
        // Zwickmühle): out of a standing mill onto a point that closes
        // another.
        let standing = board.in_mills(mine);
        let swings = bits(standing)
            .filter(|&p| {
                bits(board.adjacent[p] & empty).any(|q| {
                    let moved = mine & !(1 << p) | 1 << q;
                    board.mills_at[q].iter().any(|&m| moved & m == m)
                })
            })
            .count() as i32;
        value += swings * 250;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays one game from the start: the outcome, or `None` for a third
    /// repetition or 600 moves.
    fn game(rules: &Rules, strengths: [Strength; 2], seed: u64) -> Option<Outcome> {
        let board = rules.board();
        let mut rng = Rng::new(seed);
        let mut position = Position::new(rules);
        let mut seen: HashMap<Position, u32> = HashMap::new();
        for _ in 0..600 {
            if let Some(o) = position.outcome(rules, &board) {
                return Some(o);
            }
            let n = seen.entry(position.clone()).or_insert(0);
            *n += 1;
            if *n >= 3 {
                return None;
            }
            let side = position.to_move;
            let mv = choose(rules, &board, &position, strengths[side.index()], &seen, &mut rng).expect("a move");
            position.play(rules, &board, &mv);
        }
        None
    }

    /// How deep each strength gets from a few positions:
    /// `cargo test --release -p morris depths -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn depths() {
        struct Print;
        impl log::Log for Print {
            fn enabled(&self, _: &log::Metadata) -> bool {
                true
            }
            fn log(&self, record: &log::Record) {
                println!("  {}", record.args());
            }
            fn flush(&self) {}
        }
        let _ = log::set_logger(&Print).map(|()| log::set_max_level(log::LevelFilter::Debug));
        let rules = Rules::nine_modern();
        let b = rules.board();
        let mut p = Position::new(&rules);
        let mut rng = Rng::new(1);
        for ply in 0..30 {
            let mv = choose(&rules, &b, &p, Strength::Normal, &HashMap::new(), &mut rng).unwrap();
            if ply % 10 == 0 {
                println!("ply {ply}, hard:");
                choose(&rules, &b, &p, Strength::Hard, &HashMap::new(), &mut rng);
            }
            p.play(&rules, &b, &mv);
        }
    }

    #[test]
    fn closes_a_mill_and_blocks_one() {
        let rules = Rules::nine_modern();
        let b = rules.board();
        let at = |n: &str| (0..b.len()).find(|&p| b.name(p) == n).unwrap();
        let mut p = Position::new(&rules);
        p.men = [1 << at("a1") | 1 << at("d1"), 1 << at("a7") | 1 << at("d7")];
        p.hand = [7, 7];
        let mv = choose(&rules, &b, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert_eq!(b.name(mv.to as usize), "g1", "closes its mill");
        assert!(mv.take[0].is_some());
        // Black, with no mill to close, stops White's.
        p.men = [1 << at("a1") | 1 << at("d1"), 1 << at("a7")];
        p.hand = [7, 8];
        p.to_move = Side::Black;
        let mv = choose(&rules, &b, &p, Strength::Normal, &HashMap::new(), &mut Rng::new(1)).unwrap();
        assert_eq!(b.name(mv.to as usize), "g1", "blocks");
    }

    #[test]
    fn three_mens_first_player_takes_the_middle() {
        let rules = Rules::three_moving();
        let b = rules.board();
        let p = Position::new(&rules);
        let mv = choose(&rules, &b, &p, Strength::Hard, &HashMap::new(), &mut Rng::new(2)).unwrap();
        assert_eq!(b.name(mv.to as usize), "b2");
    }

    #[test]
    fn games_end() {
        for rules in Rules::all() {
            let o = game(&rules, [Strength::Easy, Strength::Easy], 5);
            println!("{}: {o:?}", rules.id);
        }
    }

    /// Measures the strengths against each other, each side in turn:
    /// `cargo test --release -p morris strengths -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn strengths() {
        use Strength::*;
        let n: u64 = std::env::var("MORRIS_GAMES").ok().and_then(|s| s.parse().ok()).unwrap_or(10);
        // MORRIS_ONLY=<id> measures one rule set, MORRIS_HARD=1 only hard
        // against normal.
        let only = std::env::var("MORRIS_ONLY").ok();
        let pairs: &[(Strength, Strength)] = if std::env::var("MORRIS_HARD").is_ok() {
            &[(Hard, Normal)]
        } else {
            &[(Normal, Easy), (Hard, Easy), (Hard, Normal)]
        };
        let all = [Rules::nine_modern(), Rules::nine_bell(), Rules::morabaraba(), Rules::six_bell(), Rules::three_moving()];
        for rules in all.into_iter().filter(|r| only.as_deref().is_none_or(|o| o == r.id)) {
            for &(a, b) in pairs {
                let (mut wins, mut draws) = (0, 0);
                for seed in 0..n {
                    let (strengths, me) = if seed % 2 == 0 { ([a, b], Side::White) } else { ([b, a], Side::Black) };
                    let result = game(&rules, strengths, seed + 1);
                    if std::env::var("MORRIS_VERBOSE").is_ok() {
                        println!("  {} as White, {} as Black: {result:?}", strengths[0].name(), strengths[1].name());
                    }
                    match result {
                        Some(Outcome::Won(w, _)) if w == me => wins += 1,
                        Some(Outcome::Won(..)) => {}
                        _ => draws += 1,
                    }
                }
                println!("{}: {} beats {} {wins}/{n} ({draws} drawn)", rules.id, a.name(), b.name());
            }
        }
    }
}
