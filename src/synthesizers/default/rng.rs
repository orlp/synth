#[derive(Debug, Copy, Clone)]
pub struct Xoroshiro {
    s0: u64,
    s1: u64,
}

impl Xoroshiro {
    pub fn new(seed: u64) -> Self {
        Self {
            s0: 0xdeadbeef,
            s1: seed ^ 0xfeeddddd,
        }
    }

    pub fn next(&mut self) -> u64 {
        let s0 = self.s0;
        let mut s1 = self.s1;
        let r = s0 + s1;

        s1 ^= s0;
        self.s0 = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
        self.s1 = s1.rotate_left(37);
        r
    }

    pub fn next_float(&mut self) -> f32 {
        let m = 1u64 << 24;
        ((self.next() % m) as f64 / m as f64) as f32
    }
}
