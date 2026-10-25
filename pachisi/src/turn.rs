//! A turn: the throws collected (a high throw throws again), Tylor's
//! rotten throws struck out, and then the throws used one by one; a man
//! taken brings another throw.

use crate::rules::{Rules, Throw, Token, tokens};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    /// The throws to be used.
    pub throws: Vec<Throw>,
    /// Which of the tokens from `throws` are used already.
    pub used: Vec<bool>,
    /// Throws still owed for men taken.
    pub bonus: u8,
    /// High throws running (Tylor's rotten rule).
    run: u8,
    /// The last three throws are struck out unless the next throw is a 10
    /// or a 25.
    pending: bool,
    /// Throws struck out, for the message.
    pub struck: u8,
}

impl Turn {
    pub fn new() -> Turn {
        Turn { throws: Vec::new(), used: Vec::new(), bonus: 0, run: 0, pending: false, struck: 0 }
    }

    /// Adds a throw. Returns whether the side throws again.
    pub fn add(&mut self, rules: &Rules, new: &[Throw]) -> bool {
        let mut again = false;
        for &t in new {
            if rules.rotten {
                let restores = matches!(t.value, 10 | 25);
                if self.pending {
                    self.pending = false;
                    if !restores {
                        // The three high throws before this are rotten.
                        let n = self.throws.len();
                        self.throws.truncate(n - 3);
                        self.struck += 3;
                        self.run = 0;
                    }
                }
                if matches!(t.value, 10 | 12 | 25) {
                    self.run += 1;
                    if self.run % 3 == 0 {
                        self.pending = true;
                    }
                } else {
                    self.run = 0;
                }
            }
            self.throws.push(t);
            again |= t.again;
        }
        // Three rotten throws still pending when the throwing ends are
        // lost: nothing came to restore them.
        if !again && self.pending {
            let n = self.throws.len();
            self.throws.truncate(n - 3);
            self.struck += 3;
            self.pending = false;
        }
        self.used = vec![false; tokens(rules, &self.throws).len()];
        again
    }

    /// The tokens still to use, with their places in the full list.
    pub fn left(&self, rules: &Rules) -> (Vec<Token>, Vec<usize>) {
        let all = tokens(rules, &self.throws);
        let mut toks = Vec::new();
        let mut at = Vec::new();
        for (i, t) in all.into_iter().enumerate() {
            if !self.used.get(i).copied().unwrap_or(false) {
                toks.push(t);
                at.push(i);
            }
        }
        (toks, at)
    }

    /// Marks the tokens a move used.
    pub fn spend(&mut self, rules: &Rules, used: &[Token]) {
        let all = tokens(rules, &self.throws);
        for u in used {
            if let Some(i) = (0..all.len()).find(|&i| !self.used[i] && all[i] == *u) {
                self.used[i] = true;
            }
        }
    }

    /// Starts collecting again (for a bonus throw).
    pub fn clear(&mut self) {
        self.throws.clear();
        self.used.clear();
        self.run = 0;
        self.pending = false;
    }
}

impl Default for Turn {
    fn default() -> Self {
        Turn::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(v: u8) -> Throw {
        Throw { value: v, again: matches!(v, 6 | 10 | 12 | 25), pau: matches!(v, 10 | 25), shows: 0 }
    }

    #[test]
    fn rotten_throws() {
        let r = Rules::tylor();
        let mut turn = Turn::new();
        turn.add(&r, &[t(25)]);
        turn.add(&r, &[t(12)]);
        turn.add(&r, &[t(10)]);
        // A fourth throw of 3: the three are rotten.
        turn.add(&r, &[t(3)]);
        assert_eq!(turn.throws.iter().map(|x| x.value).collect::<Vec<_>>(), [3]);
        // But a 25 restores them.
        let mut turn = Turn::new();
        for v in [25, 12, 10, 25, 2] {
            turn.add(&r, &[t(v)]);
        }
        assert_eq!(turn.throws.len(), 5);
    }
}
