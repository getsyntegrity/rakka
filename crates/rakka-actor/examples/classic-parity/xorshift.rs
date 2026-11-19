//! The generators the Java oracle uses to build its sweeps; the harness must draw the same sequences.

/// One step of xorshift64*: updates `state` and returns the scrambled output.
pub fn xorshift64_star_next(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

/// xorshift64* generator seeded like the oracle (`0` is replaced by the golden-ratio constant).
pub struct XorshiftStar(u64);

impl XorshiftStar {
    pub fn new(seed: i64) -> Self {
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed as u64
        })
    }

    pub fn next(&mut self) -> u64 {
        xorshift64_star_next(&mut self.0)
    }

    pub fn below(&mut self, bound: usize) -> usize {
        ((self.next() >> 33) % bound as u64) as usize
    }
}

/// Plain xorshift64 (shifts 13, 7, 17), as `PlatformOracle.Gen`.
pub struct Xorshift64(pub u64);

impl Xorshift64 {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
