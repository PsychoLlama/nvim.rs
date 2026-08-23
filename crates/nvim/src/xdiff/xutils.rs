//! `xutils.c`: line hashing, line comparison and the unified-diff writer.
//!
//! Three of these are the whole of the `'diffopt'` whitespace family:
//! [`hash_record`] decides which lines land in the same equivalence class,
//! [`recmatch`] confirms it, and the two must agree or the classifier builds
//! classes whose members do not match each other.
//!
//! [`emit_hunk_hdr`] and [`emit_diffrec`] are reached only from `vim.diff()`
//! without an `on_hunk` callback: `:diffupdate` installs
//! `xdemitconf_t.hunk_func`, which routes around the whole writer.
//!
//! Ported from LibXDiff by Davide Libenzi (File Differential Library),
//! Copyright (C) 2003 Davide Libenzi. LibXDiff is LGPL-2.1-or-later, and
//! this port stays under that license (text: licenses/LGPL-2.1.txt).

#![forbid(unsafe_code)]

use crate::xdiff::ffi::{Emit, is_space};
use crate::xdiff::xdiffi::do_diff;
use crate::xdiff::xtypes::{
    Block, Env, Params, XDF_IGNORE_CR_AT_EOL, XDF_IGNORE_WHITESPACE, XDF_IGNORE_WHITESPACE_AT_EOL,
    XDF_IGNORE_WHITESPACE_CHANGE, XDF_WHITESPACE_FLAGS, XdResult,
};

/// What the writer appends when the last line of a file has no newline.
const NO_NEWLINE: &[u8] = b"\n\\ No newline at end of file\n";

/// A shift-only integer square root, used to size the Myers cost ceiling and
/// the "this line matches too many others" limit. Approximate on purpose:
/// both callers want an order of magnitude, not a root.
pub(crate) fn bogosqrt(mut n: i64) -> i64 {
    let mut i = 1i64;
    while n > 0 {
        i <<= 1;
        n >>= 2;
    }
    i
}

/// How many bits of hash a table of `size` entries should be indexed by.
pub(crate) fn hashbits(size: u32) -> u32 {
    let mut val: u32 = 1;
    let mut bits: u32 = 0;
    while val < size && bits < u32::BITS {
        // The last iteration shifts the top bit out. C's `unsigned int`
        // wraps there; no caller gets a `size` anywhere near it, but the
        // spelling has to be the wrapping one to mean the same thing.
        val = val.wrapping_shl(1);
        bits += 1;
    }
    if bits != 0 { bits } else { 1 }
}

/// Estimate `text`'s line count from its first `sample` lines.
///
/// Only a sizing hint: [`super::xprepare::prepare_ctx`] grows past it. The
/// histogram engine asks for a much smaller sample because it never fills a
/// hash table that would have to be grown.
pub(crate) fn guess_lines(text: &[u8], sample: i64) -> i64 {
    let mut nl = 0i64;
    let mut cur = 0usize;
    while nl < sample && cur < text.len() {
        nl += 1;
        cur = match text[cur..].iter().position(|&b| b == b'\n') {
            Some(off) => cur + off + 1,
            None => text.len(),
        };
    }
    let tsize = cur as i64;
    if nl != 0 && tsize != 0 {
        // `tsize / nl` is at least 1: every line counted consumed at least
        // its own byte, so this cannot divide by zero.
        nl = text.len() as i64 / (tsize / nl);
    }
    nl + 1
}

/// Is this line blank? Without a whitespace flag that means "empty or just a
/// newline"; with one, "nothing but whitespace".
pub(crate) fn blankline(line: &[u8], flags: u64) -> bool {
    if flags & XDF_WHITESPACE_FLAGS == 0 {
        return line.len() <= 1;
    }
    line.iter().all(|&b| is_space(b))
}

/// Have we eaten everything on the line, except for an optional CR at the
/// very end?
fn ends_with_optional_cr(line: &[u8], i: usize) -> bool {
    let complete = line.last() == Some(&b'\n');
    let end = if complete { line.len() - 1 } else { line.len() };
    if end == i {
        return true;
    }
    // Do not ignore CR at the end of an incomplete line.
    complete && end == i + 1 && line[i] == b'\r'
}

/// Do these two lines match under `flags`?
///
/// `-w` matches everything `-b` matches, `-b` everything
/// `--ignore-space-at-eol` matches, and that in turn everything
/// `--ignore-cr-at-eol` matches — but each needs its own way of skipping
/// whitespace while both sides are still in hand, so they are four loops
/// rather than one.
pub(crate) fn recmatch(l1: &[u8], l2: &[u8], flags: u64) -> bool {
    if l1 == l2 {
        return true;
    }
    if flags & XDF_WHITESPACE_FLAGS == 0 {
        return false;
    }

    let (s1, s2) = (l1.len(), l2.len());
    let (mut i1, mut i2) = (0usize, 0usize);

    if flags & XDF_IGNORE_WHITESPACE != 0 {
        loop {
            while i1 < s1 && is_space(l1[i1]) {
                i1 += 1;
            }
            while i2 < s2 && is_space(l2[i2]) {
                i2 += 1;
            }
            if i1 >= s1 || i2 >= s2 {
                break;
            }
            if l1[i1] != l2[i2] {
                return false;
            }
            i1 += 1;
            i2 += 1;
        }
    } else if flags & XDF_IGNORE_WHITESPACE_CHANGE != 0 {
        while i1 < s1 && i2 < s2 {
            if is_space(l1[i1]) && is_space(l2[i2]) {
                // Skip matching spaces and try again.
                while i1 < s1 && is_space(l1[i1]) {
                    i1 += 1;
                }
                while i2 < s2 && is_space(l2[i2]) {
                    i2 += 1;
                }
                continue;
            }
            if l1[i1] != l2[i2] {
                return false;
            }
            i1 += 1;
            i2 += 1;
        }
    } else if flags & (XDF_IGNORE_WHITESPACE_AT_EOL | XDF_IGNORE_CR_AT_EOL) != 0 {
        // Find the first difference; where it falls is the whole answer for
        // the CR flavour, and the tail scan below settles the other.
        while i1 < s1 && i2 < s2 && l1[i1] == l2[i2] {
            i1 += 1;
            i2 += 1;
        }
        if flags & XDF_IGNORE_WHITESPACE_AT_EOL == 0 {
            return ends_with_optional_cr(l1, i1) && ends_with_optional_cr(l2, i2);
        }
    }

    // After running out of one side, the remaining side must have nothing
    // but whitespace for the lines to match. Note that the
    // ignore-whitespace-at-eol case may break out of its loop while there
    // still are characters remaining on both lines.
    if i1 < s1 {
        while i1 < s1 && is_space(l1[i1]) {
            i1 += 1;
        }
        if s1 != i1 {
            return false;
        }
    }
    if i2 < s2 {
        while i2 < s2 && is_space(l2[i2]) {
            i2 += 1;
        }
        return s2 == i2;
    }
    true
}

/// Hash the line `text` starts with, and say how many bytes it took —
/// its newline included, so the answer is where the next line begins.
///
/// `text` is the rest of the *file*, not one line: the whitespace flavours
/// need to see the byte after a whitespace run to know whether the run is at
/// end of line, and that byte may be the newline itself.
pub(crate) fn hash_record(text: &[u8], flags: u64) -> (u64, usize) {
    // Where this line ends, and where the next one starts. Splitting the
    // line off first is what keeps the fold loop below index-free: it is the
    // one place in the engine that runs per *byte* of both files.
    let end = text.iter().position(|&b| b == b'\n').unwrap_or(text.len());
    let used = if end < text.len() { end + 1 } else { end };
    if flags & XDF_WHITESPACE_FLAGS != 0 {
        let complete = end < text.len();
        return (
            hash_record_with_whitespace(&text[..end], complete, flags),
            used,
        );
    }
    let mut ha = 5381u64;
    for &byte in &text[..end] {
        ha = fold(ha, byte);
    }
    (ha, used)
}

/// One djb2-ish step. The byte enters as a *signed* char, as it does in the
/// C, so anything over 0x7F sign-extends to a 64-bit value with the top
/// fifty-seven bits set — which is load-bearing, because it is what a
/// multibyte line's hash is built out of.
fn fold(ha: u64, byte: u8) -> u64 {
    ha.wrapping_add(ha << 5) ^ (byte as i8 as u64)
}

/// The whitespace flavours, over one line with its newline already removed.
///
/// Upstream works over the whole remaining file and tests `ptr[1] == '\n'`
/// to decide a whitespace run is at end of line; with the newline stripped
/// that test is just "is this the last byte of the line", which is the same
/// answer and needs no lookahead past the slice. `complete` says whether the
/// line *had* a newline, which the CR rule below still has to know.
fn hash_record_with_whitespace(line: &[u8], complete: bool, flags: u64) -> u64 {
    let mut ha = 5381u64;
    let cr_at_eol_only = flags & XDF_WHITESPACE_FLAGS == XDF_IGNORE_CR_AT_EOL;
    let top = line.len();
    let mut p = 0usize;

    while p < top {
        if cr_at_eol_only {
            // Do not ignore CR at the end of an incomplete line. The
            // newline was stripped above, so "immediately before the
            // newline" is "the last byte of a line that had one".
            if line[p] == b'\r' && p + 1 == top && complete {
                p += 1;
                continue;
            }
        } else if is_space(line[p]) {
            let run_start = p;
            while p + 1 < top && is_space(line[p + 1]) {
                p += 1;
            }
            let at_eol = p + 1 >= top;
            if flags & XDF_IGNORE_WHITESPACE != 0 {
                // Already handled: the whole run contributes nothing.
            } else if flags & XDF_IGNORE_WHITESPACE_CHANGE != 0 && !at_eol {
                ha = fold(ha, b' ');
            } else if flags & XDF_IGNORE_WHITESPACE_AT_EOL != 0 && !at_eol {
                for &byte in &line[run_start..=p] {
                    ha = fold(ha, byte);
                }
            }
            p += 1;
            continue;
        }
        ha = fold(ha, line[p]);
        p += 1;
    }

    ha
}

/// `xdl_num_out`: append `val` in decimal.
///
/// Upstream writes the sign into its scratch buffer *first* and then prepends
/// the digits in front of it, so a negative value comes out with the minus on
/// the wrong end: `-199` renders as `199-`. Spelled correctly here (O-B15-1).
/// Nothing reaches it negative any longer either — [`super::ffi::xdl_diff`]
/// clamps `ctxlen`, which was the only way to compute a negative count.
fn push_num(out: &mut Vec<u8>, val: i64) {
    let mut digits = [0u8; 21];
    // `unsigned_abs`, so `i64::MIN` does not overflow the negation.
    let mut n = val.unsigned_abs();
    let mut i = digits.len();
    loop {
        i -= 1;
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if val < 0 {
        i -= 1;
        digits[i] = b'-';
    }
    out.extend_from_slice(&digits[i..]);
}

/// Write `@@ -s1,c1 +s2,c2 @@` as a line, the way `diff -u` does: an empty
/// side names the line *before* the insertion point, and a one-line side
/// drops the count.
fn format_hunk_hdr(s1: i64, c1: i64, s2: i64, c2: i64, emit: &mut Emit) -> XdResult {
    let mut buf = Vec::with_capacity(32);
    buf.extend_from_slice(b"@@ -");
    push_num(&mut buf, if c1 != 0 { s1 } else { s1 - 1 });
    if c1 != 1 {
        buf.push(b',');
        push_num(&mut buf, c1);
    }
    buf.extend_from_slice(b" +");
    push_num(&mut buf, if c2 != 0 { s2 } else { s2 - 1 });
    if c2 != 1 {
        buf.push(b',');
        push_num(&mut buf, c2);
    }
    buf.extend_from_slice(b" @@\n");
    emit.line(&[&buf])
}

/// Report a hunk's extent, through `xdemitcb_t.out_hunk` if the caller
/// installed one and as a formatted `@@` line otherwise.
pub(crate) fn emit_hunk_hdr(s1: i64, c1: i64, s2: i64, c2: i64, emit: &mut Emit) -> XdResult {
    if !emit.has_out_hunk() {
        return format_hunk_hdr(s1, c1, s2, c2, emit);
    }
    emit.out_hunk(
        if c1 != 0 { s1 } else { s1 - 1 },
        c1,
        if c2 != 0 { s2 } else { s2 - 1 },
        c2,
    )
}

/// Write one body line: its ` `/`-`/`+` marker, the line, and the
/// no-final-newline marker when the line has no newline of its own.
pub(crate) fn emit_diffrec(rec: &[u8], pre: &[u8], emit: &mut Emit) -> XdResult {
    if rec.last().is_some_and(|&b| b != b'\n') {
        emit.line(&[pre, rec, NO_NEWLINE])
    } else {
        emit.line(&[pre, rec])
    }
}

/// Diff lines `line1 ..` of file 1 against `line2 ..` of file 2 with the
/// classic algorithm and fold the answer back into `env`.
///
/// Both of the other engines end here when they run out of anchors. The
/// sub-files are re-derived from `env`'s records rather than from the
/// original `mmfile_t`s — upstream's comment says it would rather reuse the
/// prepared environment, but the library has no way to diff a range.
pub(crate) fn fall_back_diff(env: &mut Env<'_>, xpp: &Params<'_>, blk: Block) -> XdResult {
    let sub1 = env.xdf1.span(blk.line1 - 1, blk.end1() - 1);
    let sub2 = env.xdf2.span(blk.line2 - 1, blk.end2() - 1);
    let sub = do_diff(sub1, sub2, xpp)?;
    env.xdf1
        .rchg
        .write(blk.line1 - 1, sub.xdf1.rchg.slice(0, blk.count1));
    env.xdf2
        .rchg
        .write(blk.line2 - 1, sub.xdf2.rchg.slice(0, blk.count2));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bogosqrt_is_a_power_of_two_bracketing_the_root() {
        assert_eq!(bogosqrt(0), 1);
        assert_eq!(bogosqrt(1), 2);
        assert_eq!(bogosqrt(3), 2);
        assert_eq!(bogosqrt(4), 4);
        assert_eq!(bogosqrt(16), 8);
        assert_eq!(bogosqrt(1_000_000), 1024);
    }

    #[test]
    fn hashbits_never_answers_zero() {
        assert_eq!(hashbits(0), 1);
        assert_eq!(hashbits(1), 1);
        assert_eq!(hashbits(2), 1);
        assert_eq!(hashbits(3), 2);
        assert_eq!(hashbits(256), 8);
        assert_eq!(hashbits(257), 9);
        assert_eq!(hashbits(u32::MAX), 32);
    }

    #[test]
    fn guess_lines_counts_and_extrapolates() {
        assert_eq!(guess_lines(b"", 256), 1);
        assert_eq!(guess_lines(b"a\nb\nc\n", 256), 4);
        // Sampled: two of six lines seen, so the estimate is scaled up.
        assert_eq!(guess_lines(b"a\nb\nc\nd\ne\nf\n", 2), 7);
    }

    #[test]
    fn hash_record_consumes_the_newline() {
        let (_, used) = hash_record(b"abc\ndef\n", 0);
        assert_eq!(used, 4);
        let (_, used) = hash_record(b"abc", 0);
        assert_eq!(used, 3);
    }

    #[test]
    fn hash_record_sign_extends_high_bytes() {
        // 0xC3 enters as -61, so the fold sets the top bits; an unsigned
        // byte would leave them clear.
        let (ha, _) = hash_record(&[0xC3, b'\n'], 0);
        assert_eq!(ha, fold(5381, 0xC3));
        assert!(ha > u64::from(u32::MAX));
    }

    #[test]
    fn whitespace_flavours_agree_between_hash_and_match() {
        let cases: &[(&[u8], &[u8], u64, bool)] = &[
            (b"a b\n", b"a  b\n", XDF_IGNORE_WHITESPACE_CHANGE, true),
            (b"a b\n", b"ab\n", XDF_IGNORE_WHITESPACE_CHANGE, false),
            (b"a b\n", b"ab\n", XDF_IGNORE_WHITESPACE, true),
            (b"ab \n", b"ab\n", XDF_IGNORE_WHITESPACE_AT_EOL, true),
            (b"a b\n", b"ab\n", XDF_IGNORE_WHITESPACE_AT_EOL, false),
            (b"ab\r\n", b"ab\n", XDF_IGNORE_CR_AT_EOL, true),
            (b"ab\r", b"ab", XDF_IGNORE_CR_AT_EOL, false),
        ];
        for &(l1, l2, flags, matches) in cases {
            assert_eq!(recmatch(l1, l2, flags), matches, "{l1:?} vs {l2:?}");
            if matches {
                assert_eq!(
                    hash_record(l1, flags).0,
                    hash_record(l2, flags).0,
                    "hash disagrees with recmatch for {l1:?} vs {l2:?}"
                );
            }
        }
    }

    #[test]
    fn blankline_needs_a_flag_to_see_spaces() {
        assert!(blankline(b"\n", 0));
        assert!(!blankline(b"  \n", 0));
        assert!(blankline(b"  \n", XDF_IGNORE_WHITESPACE));
        assert!(!blankline(b" x\n", XDF_IGNORE_WHITESPACE));
    }

    #[test]
    fn push_num_puts_the_sign_first() {
        let mut out = Vec::new();
        push_num(&mut out, 0);
        push_num(&mut out, 199);
        assert_eq!(out, b"0199".to_vec());

        // O-B15-1: upstream would have written `199-` and `...808-`.
        let mut out = Vec::new();
        push_num(&mut out, -199);
        push_num(&mut out, i64::MIN);
        assert_eq!(out, b"-199-9223372036854775808".to_vec());
    }
}
