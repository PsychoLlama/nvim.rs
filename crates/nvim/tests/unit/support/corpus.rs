//! A fixed corpus of awkward text, and a digest of what a function answers
//! over it, for the tests that hold a rewritten text reader to the answers
//! of the one it replaced.
//!
//! The corpus is UTF-8 at its most hostile: valid sequences of every length,
//! composing characters with and without a base, ZWJ and variation-selector
//! emoji, regional indicators, Hangul jamo, Arabic letters that the
//! `'arabicshape'` rule joins, a Prepend character, ambiguous-width and
//! unprintable codepoints, overlong encodings, stray continuation bytes,
//! sequences cut short, five- and six-byte forms, and the `K_SPECIAL` escape.
//! No string holds a NUL: the pointer forms stop there and the slice forms do
//! not, which is the one difference they document.
//!
//! A [`Digest`] folds every answer into one number. A test records the
//! number the old reader produced and asserts the new one produces it too,
//! so the oracle survives the old reader being deleted.

/// The pieces the corpus strings are built from.
const TOKENS: &[&[u8]] = &[
    b"a",
    b"Z",
    b"0",
    b" ",
    b"\t",
    b"(",
    b")",
    b"~",
    b"_",
    b"-",
    b"\\",
    b"<",
    b">",
    b"\r",
    b"\x16",
    // Latin-1 and friends.
    b"\xc3\xa9",
    b"\xc3\x84",
    b"\xc3\x9f",
    b"\xc2\xa0",
    b"\xc2\xa1",
    b"\xc2\xb1",
    b"\xc2\x85",
    b"\xc3\x83",
    b"\xc4\xb0",
    // Composing characters.
    b"\xcc\x80",
    b"\xcc\x81",
    b"\xcc\x88",
    b"\xe2\x83\x90",
    b"\xe1\xaa\xb0",
    b"\xd9\x8b",
    // ZWJ, VS-16, VS-15, ZWSP, BOM, line separator.
    b"\xe2\x80\x8d",
    b"\xef\xb8\x8f",
    b"\xef\xb8\x8e",
    b"\xe2\x80\x8b",
    b"\xef\xbb\xbf",
    b"\xe2\x80\xa8",
    // Emoji: a face, a person, a sheaf, a heart, a flag pair, a white flag,
    // the transgender symbol, a skin tone.
    b"\xf0\x9f\x98\x80",
    b"\xf0\x9f\xa7\x91",
    b"\xf0\x9f\x8c\xbe",
    b"\xe2\x9d\xa4",
    b"\xf0\x9f\x87\xab",
    b"\xf0\x9f\x87\xb7",
    b"\xf0\x9f\x8f\xb3",
    b"\xe2\x9a\xa7",
    b"\xf0\x9f\x8f\xbb",
    b"\xe2\x98\xba",
    // Wide: CJK, Hangul syllable, Hangul jamo L/V/T, fullwidth A.
    b"\xe4\xb8\xad",
    b"\xea\xb0\x80",
    b"\xe1\x84\x80",
    b"\xe1\x85\xa1",
    b"\xe1\x86\xa8",
    b"\xef\xbc\xa1",
    // Arabic lam, alef, alef with hamza; a Prepend mark.
    b"\xd9\x84",
    b"\xd8\xa7",
    b"\xd8\xa3",
    b"\xd8\x80",
    // Ambiguous width.
    b"\xe2\x80\xa6",
    b"\xe2\x91\xa0",
    b"\xe2\x97\x8b",
    // An encoded surrogate, a private-use character, U+10FFFF.
    b"\xed\xa0\x80",
    b"\xee\x80\x80",
    b"\xf4\x8f\xbf\xbf",
    // Overlong encodings.
    b"\xc0\x80",
    b"\xc1\xbf",
    b"\xe0\x80\x80",
    b"\xf0\x80\x80\x80",
    b"\xc0\xa9",
    // Not characters: stray continuation bytes, bytes that lead nothing,
    // lone leads, sequences cut short.
    b"\x80",
    b"\xbf",
    b"\xfe",
    b"\xff",
    b"\xc3",
    b"\xe2\x82",
    b"\xf0\x9f\x98",
    b"\xe2",
    // Five- and six-byte forms.
    b"\xf8\x88\x80\x80\x80",
    b"\xfc\x84\x80\x80\x80\x80",
    b"\xfd\xbf\xbf\xbf\xbf\xbf",
    // The K_SPECIAL escapes: a literal 0x80, and a special key.
    b"\x80\xfe\x58",
    b"\x80\xfd\x04",
    b"\x80\x6b\x75",
];

/// A small deterministic generator, so the corpus is the same on every run
/// and under Miri.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n).expect("small")).expect("small")
    }
}

/// The pieces themselves, for an oracle that builds its own combinations.
#[allow(dead_code, reason = "not every oracle combines tokens")]
pub(crate) fn tokens() -> &'static [&'static [u8]] {
    TOKENS
}

/// The corpus: every token on its own, every token after an ASCII letter and
/// before one, then strings of up to eight random tokens.
pub(crate) fn corpus() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![Vec::new()];
    for token in TOKENS {
        out.push(token.to_vec());
        let mut framed = b"x".to_vec();
        framed.extend_from_slice(token);
        framed.push(b'y');
        out.push(framed);
    }
    let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..160 {
        let mut s = Vec::new();
        for _ in 0..=rng.below(8) {
            s.extend_from_slice(TOKENS[rng.below(TOKENS.len())]);
        }
        out.push(s);
    }
    out
}

/// FNV-1a over every answer a test feeds it.
pub(crate) struct Digest(u64);

impl Digest {
    pub(crate) fn new() -> Digest {
        Digest(0xcbf2_9ce4_8422_2325)
    }

    /// Fold one answer in.
    pub(crate) fn add(&mut self, value: i64) {
        for byte in value.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }

    /// Fold a run of bytes in, length first.
    #[allow(dead_code, reason = "not every oracle answers in bytes")]
    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        self.add(i64::try_from(bytes.len()).expect("short"));
        for &byte in bytes {
            self.add(i64::from(byte));
        }
    }

    pub(crate) fn value(&self) -> u64 {
        self.0
    }
}
