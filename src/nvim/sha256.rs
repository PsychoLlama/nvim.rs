//! SHA-256 (FIPS 180-2), used for undo-file checksums and the vimscript
//! `sha256()` function. The undo-file format stores these digests on disk,
//! so the output must stay bit-identical.

/// Size of a SHA-256 digest in bytes.
pub const SHA256_SUM_SIZE: usize = 32;

const BLOCK_SIZE: usize = 64;

/// FIPS 180-2 initial hash value.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// FIPS 180-2 round constants.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Streaming SHA-256 context: feed bytes with [`update`](Self::update), then
/// take the digest with [`finish`](Self::finish).
pub struct Sha256 {
    /// Total message length in bytes.
    total: u64,
    state: [u32; 8],
    buffer: [u8; BLOCK_SIZE],
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            total: 0,
            state: H0,
            buffer: [0; BLOCK_SIZE],
        }
    }

    pub fn update(&mut self, mut input: &[u8]) {
        let left = self.total as usize % BLOCK_SIZE;
        self.total = self.total.wrapping_add(input.len() as u64);
        if left != 0 {
            let fill = BLOCK_SIZE - left;
            if input.len() < fill {
                self.buffer[left..left + input.len()].copy_from_slice(input);
                return;
            }
            self.buffer[left..].copy_from_slice(&input[..fill]);
            let block = self.buffer;
            self.process(&block);
            input = &input[fill..];
        }
        let mut blocks = input.chunks_exact(BLOCK_SIZE);
        for block in &mut blocks {
            self.process(block.try_into().expect("exact chunk"));
        }
        let rest = blocks.remainder();
        self.buffer[..rest.len()].copy_from_slice(rest);
    }

    pub fn finish(mut self) -> [u8; SHA256_SUM_SIZE] {
        const PADDING: [u8; BLOCK_SIZE] = {
            let mut p = [0; BLOCK_SIZE];
            p[0] = 0x80;
            p
        };
        let bit_len = self.total.wrapping_mul(8);
        let last = self.total as usize % BLOCK_SIZE;
        let padn = if last < 56 { 56 - last } else { 120 - last };
        self.update(&PADDING[..padn]);
        self.update(&bit_len.to_be_bytes());
        let mut digest = [0; SHA256_SUM_SIZE];
        for (bytes, word) in digest.chunks_exact_mut(4).zip(self.state) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    fn process(&mut self, block: &[u8; BLOCK_SIZE]) {
        let mut w = [0u32; 64];
        for (word, bytes) in w.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().expect("exact chunk"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (state, val) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *state = state.wrapping_add(val);
        }
    }
}

/// Lowercase-hex SHA-256 digest of `input`, as the vimscript `sha256()`
/// function returns it.
pub fn hex_digest(input: &[u8]) -> String {
    use core::fmt::Write;

    let mut ctx = Sha256::new();
    ctx.update(input);
    let mut hex = String::with_capacity(SHA256_SUM_SIZE * 2);
    for byte in ctx.finish() {
        write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    hex
}
