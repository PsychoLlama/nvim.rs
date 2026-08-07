//! The two byte-length tables, indexed by a leading byte.
//!
//! Both answer how many bytes the UTF-8 sequence introduced by a byte
//! occupies. They differ only in what they say about a byte that introduces
//! nothing — a continuation byte (`0x80`–`0xBF`) or `0xFE`/`0xFF`, which are
//! not valid anywhere in UTF-8. [`utf8len_tab`] answers 1 there, so a caller
//! stepping through a string always advances and never loops; [`utf8len_tab_zero`]
//! answers 0, for the callers whose job is to *detect* the invalid sequence.
//!
//! Neither table restricts itself to the four sequence lengths Unicode
//! actually uses: `0xF8`–`0xFD` still report 5 and 6, the lengths the original
//! (pre-2003) UTF-8 definition gave them. Upstream keeps those so a walk over
//! text carrying such a sequence steps over the whole thing rather than
//! stopping inside it; the decoders reject the value separately.
//!
//! `utf8len_tab` is exported by name: `test/unit/fixtures/shim.h` indexes it
//! directly from C, so it must stay a 256-byte array under that symbol.

#![deny(unsafe_op_in_unsafe_fn)]

/// One of the two tables, given the answer for a byte that introduces nothing.
const fn lead_byte_lengths(introduces_nothing: u8) -> [u8; 256] {
    let mut tab = [0u8; 256];
    let mut b = 0;
    while b < 256 {
        tab[b] = match b {
            0x00..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            0xf8..=0xfb => 5,
            0xfc..=0xfd => 6,
            _ => introduces_nothing,
        };
        b += 1;
    }
    tab
}

/// Sequence length per lead byte, answering **1** for a byte that introduces
/// nothing — so a walk over invalid text always makes progress.
#[unsafe(no_mangle)]
pub static utf8len_tab: [u8; 256] = lead_byte_lengths(1);

/// Sequence length per lead byte, answering **0** for a byte that introduces
/// nothing — so a caller can tell a valid lead byte from any other.
pub static utf8len_tab_zero: [u8; 256] = lead_byte_lengths(0);

#[cfg(test)]
mod tests {
    use super::*;

    /// The tables as `mbyte.c` spells them out, row by row, so a fold that
    /// drifted from upstream's literal is a failing test and not a silent
    /// change of what a lead byte means.
    #[rustfmt::skip]
    const UPSTREAM: [[u8; 16]; 16] = [
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 0?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 1?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 2?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 3?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 4?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 5?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 6?
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], // 7?
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 8?
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 9?
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // A?
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // B?
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], // C?
        [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], // D?
        [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3], // E?
        [4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 0, 0], // F?
    ];

    #[test]
    fn matches_upstream() {
        for b in 0..256 {
            let zero = UPSTREAM[b >> 4][b & 0xf];
            assert_eq!(utf8len_tab_zero[b], zero, "utf8len_tab_zero[{b:#04x}]");
            let one = if zero == 0 { 1 } else { zero };
            assert_eq!(utf8len_tab[b], one, "utf8len_tab[{b:#04x}]");
        }
    }

    /// The one difference between the tables, stated as the rule it is.
    #[test]
    fn tables_differ_only_where_nothing_is_introduced() {
        for b in 0..256 {
            if utf8len_tab_zero[b] == 0 {
                assert_eq!(utf8len_tab[b], 1);
                assert!(matches!(b, 0x80..=0xbf | 0xfe | 0xff));
            } else {
                assert_eq!(utf8len_tab[b], utf8len_tab_zero[b]);
            }
        }
    }
}
