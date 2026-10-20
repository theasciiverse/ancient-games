//! A small, seedable random number generator, so a game can be repeated
//! with `--seed`.

/// xorshift64*: small, fast and good enough for dice.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    pub fn coin(&mut self) -> bool {
        self.next() >> 63 == 1
    }
}

