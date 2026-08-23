//! UTF-8 decoding, grapheme clustering and the character-boundary walk.
//!
//! A port of `test/unit/mbyte_spec.lua`. The leaf arithmetic — the lead-byte
//! masking, the encode/decode round trip, the fold table — has in-crate tests
//! already. What the spec was the only oracle for is `mbyte/walk.rs` (520
//! lines, no tests) and the composing-character rules in
//! `utfc_ptr2schar`/`utfc_ptr2len`, which is where every sequence below is
//! aimed.
//!
//! `vim_iswordc` reads `curbuf` and `utf_head_off` reads `'arabicshape'`, so
//! every case here holds the editor lock.

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::charset::{vim_iswordc, vim_iswordp};
use neovim::grid::{MAX_SCHAR_SIZE, schar_get};
use neovim::main::p_arshape;
use neovim::mbyte::{
    utf_char2bytes, utf_cp_bounds_len, utf_fold, utf_head_off, utf_ptr2char, utfc_ptr2len,
    utfc_ptr2schar,
};

use crate::support::editor_lock;

/// `s` as a NUL-terminated C buffer. Every entry point below reads to the
/// terminator and no further, so the terminator is what bounds them.
fn cbuf(s: &[u8]) -> Vec<c_char> {
    s.iter()
        .map(|&b| b as c_char)
        .chain(std::iter::once(0))
        .collect()
}

/// `utfc_ptr2schar` unpacked: (the cluster's bytes, its first codepoint).
///
/// Known blind spot, inherited from the spec: nothing here reaches the byte
/// `utfc_ptr2schar` holds back for the space it synthesises in front of a
/// cluster that opens with a composing character. Removing that reservation
/// changes no answer below, because no fixture is anywhere near
/// `MAX_SCHAR_SIZE` long.
///
/// The `schar_T` it answers is an index into the glyph cache for anything too
/// long to sit inline, so reading it back goes through `schar_get`. That
/// makes this a statement about the pair, which is how the editor uses them.
fn cluster(seq: &[u8]) -> (Vec<u8>, c_int) {
    let src = cbuf(seq);
    let mut firstc: c_int = 0;
    let mut out = [0 as c_char; MAX_SCHAR_SIZE as usize];
    // SAFETY: `src` is NUL-terminated; `out` is `MAX_SCHAR_SIZE` bytes, which
    // is the most `schar_get` writes including the terminator.
    unsafe {
        let sc = utfc_ptr2schar(src.as_ptr(), &raw mut firstc);
        let len = schar_get(out.as_mut_ptr(), sc);
        let text = out[..len].iter().map(|&b| b as u8).collect();
        (text, firstc)
    }
}

/// What a one-byte ASCII cluster looks like: itself.
fn ascii(c: u8) -> (Vec<u8>, c_int) {
    (vec![c], c_int::from(c))
}

/// A cluster with no bytes at all — what an invalid lead byte produces,
/// alongside the byte itself as the "first codepoint".
fn nothing(firstc: c_int) -> (Vec<u8>, c_int) {
    (Vec::new(), firstc)
}

/// A cluster spelled by its bytes, with its first codepoint.
fn text(bytes: &[u8], firstc: c_int) -> (Vec<u8>, c_int) {
    (bytes.to_vec(), firstc)
}

#[test]
fn a_lone_byte_below_0x80_decodes_to_itself_and_so_does_every_other_lead() {
    let _editor = editor_lock();
    for c in 0..=255_u32 {
        let buf = cbuf(&[c as u8]);
        // SAFETY: `buf` is NUL-terminated.
        assert_eq!(
            unsafe { utf_ptr2char(buf.as_ptr()) },
            c as c_int,
            "a truncated sequence answers its first byte"
        );
    }
}

/// The encoder and the decoder are inverses over the whole Unicode range, and
/// the two spellings of "is this a word character" agree over it too. The
/// spec split this into sixteen cases of 0x1000 codepoints each purely
/// because `itp` forks; one loop is the same statement.
#[test]
fn every_codepoint_round_trips_and_both_word_tests_agree() {
    let _editor = editor_lock();
    let mut buf = [0 as c_char; 8];
    for c in 0..0x10000_i32 {
        // SAFETY: `buf` is eight bytes and `utf_char2bytes` writes at most
        // four; the read back stops at the sequence it just wrote.
        unsafe {
            utf_char2bytes(c, buf.as_mut_ptr());
            assert_eq!(utf_ptr2char(buf.as_ptr()), c, "round trip of {c:#x}");
            assert_eq!(
                vim_iswordc(c),
                vim_iswordp(buf.as_ptr()),
                "word test disagrees at {c:#x}"
            );
        }
    }
}

/// One byte in: ASCII is itself, NUL is nothing, and a continuation or
/// invalid lead byte produces an empty cluster whose first codepoint is the
/// raw byte.
#[test]
fn one_byte_clusters() {
    let _editor = editor_lock();
    assert_eq!(cluster(&[0]), nothing(0));
    for c in 1..=127_u8 {
        assert_eq!(cluster(&[c]), ascii(c), "{c:#x}");
    }
    for c in 128..=255_u8 {
        assert_eq!(cluster(&[c]), nothing(c_int::from(c)), "{c:#x}");
    }
}

/// Two bytes in: the second byte joins only if it completes a sequence, and
/// an incomplete sequence throws its lead byte away.
#[test]
fn two_byte_clusters() {
    let _editor = editor_lock();
    assert_eq!(cluster(&[0x7f, 0x7f]), ascii(0x7f), "nothing to combine");
    assert_eq!(cluster(&[0x7f, 0x80]), ascii(0x7f), "still nothing");
    assert_eq!(cluster(&[0xc2, 0x7f]), nothing(0xc2), "not a sequence");
    assert_eq!(cluster(&[0xc2, 0x80]), text(&[0xc2, 0x80], 0x80));
    assert_eq!(cluster(&[0xc2, 0xc0]), nothing(0xc2), "not a sequence");
}

/// Three bytes in: this is where composing characters start, and where the
/// rule that a composing character needs a valid base shows up.
#[test]
fn three_byte_clusters() {
    let _editor = editor_lock();
    assert_eq!(cluster(&[0x7f, 0x80, 0x80]), ascii(0x7f));
    assert_eq!(cluster(&[0x7f, 0xc2, 0x80]), ascii(0x7f), "not composing");

    // U+0300 combines with `)`.
    assert_eq!(
        cluster(&[0x29, 0xcc, 0x80]),
        text(&[0x29, 0xcc, 0x80], 0x29)
    );
    // ...but not with DEL, which is not a valid base.
    assert_eq!(cluster(&[0x7f, 0xcc, 0x80]), ascii(0x7f));

    assert_eq!(cluster(&[0xc2, 0x7f, 0xcc]), nothing(0xc2));
    assert_eq!(
        cluster(&[0xc2, 0x80, 0xcc]),
        text(&[0xc2, 0x80], 0x80),
        "the incomplete composing character is dropped"
    );

    // A composing character with nothing before it gets a space to sit on.
    assert_eq!(
        cluster(&[0xe2, 0x83, 0x90]),
        text(&[0x20, 0xe2, 0x83, 0x90], 0x20d0)
    );
}

#[test]
fn four_byte_clusters() {
    let _editor = editor_lock();
    assert_eq!(cluster(&[0x7f, 0x7f, 0xcc, 0x80]), ascii(0x7f));
    assert_eq!(cluster(&[0x29, 0x29, 0xcc, 0x80]), ascii(0x29));
    assert_eq!(cluster(&[0x7f, 0xc2, 0xcc, 0x80]), ascii(0x7f));

    assert_eq!(
        cluster(&[0x29, 0xcc, 0x80, 0xcc]),
        text(&[0x29, 0xcc, 0x80], 0x29)
    );
    assert_eq!(cluster(&[0xc2, 0x7f, 0xcc, 0x80]), nothing(0xc2));
    assert_eq!(
        cluster(&[0xc2, 0x80, 0xcc, 0xcc]),
        text(&[0xc2, 0x80], 0x80)
    );
    // U+0301 combines with U+00BC.
    assert_eq!(
        cluster(&[0xc2, 0xbc, 0xcc, 0x81]),
        text(&[0xc2, 0xbc, 0xcc, 0x81], 0xbc)
    );
    // U+0080 is not a valid base, so the composing character is dropped.
    assert_eq!(
        cluster(&[0xc2, 0x80, 0xcc, 0x81]),
        text(&[0xc2, 0x80], 0x80)
    );
    // A four-byte sequence on its own.
    assert_eq!(
        cluster(&[0xf4, 0x80, 0x80, 0x80]),
        text(&[0xf4, 0x80, 0x80, 0x80], 0x100000)
    );
}

/// Longer input: composing characters accumulate until the cluster would not
/// fit, and anything that is not one ends it.
#[test]
fn clusters_of_five_bytes_and_more() {
    let _editor = editor_lock();
    assert_eq!(cluster(&[0x7f, 0x7f, 0xcc, 0x80, 0x80]), ascii(0x7f));
    assert_eq!(cluster(&[0x7f, 0xc2, 0xcc, 0x80, 0x80]), ascii(0x7f));
    assert_eq!(
        cluster(&[0x29, 0xcc, 0x80, 0xcc, 0x00]),
        text(&[0x29, 0xcc, 0x80], 0x29)
    );

    // U+0300 through U+0306 all join `)`, one after another.
    let mut want = vec![0x29];
    let mut seq = vec![0x29];
    for low in 0x80..=0x86_u8 {
        seq.extend_from_slice(&[0xcc, low]);
        want.extend_from_slice(&[0xcc, low]);
        assert_eq!(cluster(&seq), text(&want, 0x29), "{} composing", want.len());
    }

    // A base that is not composing ends the cluster, and what follows it is
    // not read at all.
    assert_eq!(
        cluster(&[
            0x29, 0xcc, 0x80, 0xcc, 0x81, 0xcc, 0x82, 0xc2, 0x80, 0xcc, 0x84, 0xcc, 0x85
        ]),
        text(&[0x29, 0xcc, 0x80, 0xcc, 0x81, 0xcc, 0x82], 0x29)
    );

    assert_eq!(cluster(&[0xc2, 0x7f, 0xcc, 0x80, 0x80]), nothing(0xc2));
    assert_eq!(
        cluster(&[0xc2, 0xbc, 0xcc, 0xcc, 0x80]),
        text(&[0xc2, 0xbc], 0xbc)
    );
    for tail in [0x7f, 0xcc] {
        assert_eq!(
            cluster(&[0xc2, 0xbc, 0xcc, 0x81, tail]),
            text(&[0xc2, 0xbc, 0xcc, 0x81], 0xbc)
        );
    }
    for tail in [0x7f, 0x80, 0xcc] {
        assert_eq!(
            cluster(&[0xf4, 0x80, 0x80, 0x80, tail]),
            text(&[0xf4, 0x80, 0x80, 0x80], 0x100000)
        );
    }
    // U+1AB0 and U+0301 both join a plane-16 base.
    assert_eq!(
        cluster(&[0xf4, 0x80, 0x80, 0x80, 0xe1, 0xaa, 0xb0, 0xcc, 0x81]),
        text(
            &[0xf4, 0x80, 0x80, 0x80, 0xe1, 0xaa, 0xb0, 0xcc, 0x81],
            0x100000
        )
    );
}

/// `utf_cp_bounds_len` answers, for every byte of a string, how far back the
/// codepoint it belongs to starts and how far forward it ends. The three
/// fixtures are a well-formed string, one that ends mid-sequence, and one
/// with stray continuation bytes.
///
/// They are spelled as bytes rather than as Rust `str`s on purpose: two of
/// the three are not valid UTF-8, which is the whole point of them, and the
/// Lua spec wrote them as a mix of literal text and `\xNN` escapes.
#[test]
fn the_codepoint_bounds_of_every_byte() {
    let _editor = editor_lock();
    #[track_caller]
    fn check(raw: &[u8], want_begin: &[i8], want_end: &[i8]) {
        assert_eq!(raw.len(), want_begin.len(), "one answer per byte");
        let buf = cbuf(raw);
        let (mut begin, mut end) = (Vec::new(), Vec::new());
        for i in 0..raw.len() {
            let left = c_int::try_from(raw.len() - i).expect("the fixtures are short");
            // SAFETY: `buf` holds `raw.len()` bytes plus a terminator, `i` is
            // inside it, and `left` is exactly what remains.
            let bounds = unsafe { utf_cp_bounds_len(buf.as_ptr(), buf.as_ptr().add(i), left) };
            begin.push(bounds.begin_off);
            end.push(bounds.end_off);
        }
        assert_eq!((&begin[..], &end[..]), (want_begin, want_end), "{raw:x?}");
    }

    check(
        b"i\xc3\x80ii\xe2\xb1\xa0i\xe2\xb1\xa0\xe2\xb1\xa0\xf0\x90\x80\x80i",
        &[0, 0, 1, 0, 0, 0, 1, 2, 0, 0, 1, 2, 0, 1, 2, 0, 1, 2, 3, 0],
        &[1, 2, 1, 1, 1, 3, 2, 1, 1, 3, 2, 1, 3, 2, 1, 4, 3, 2, 1, 1],
    );
    // Ends mid-sequence, twice over.
    check(
        b"i\xc3i\xc3\x80\xe2\xb1\xa0i\xc3\x80\xe2\xb1\xe2\xb1\xa0\xf0\x90\x80",
        &[0, 0, 0, 0, 1, 0, 1, 2, 0, 0, 1, 0, 0, 0, 1, 2, 0, 0, 0],
        &[1, 1, 1, 2, 1, 3, 2, 1, 1, 2, 1, 1, 1, 3, 2, 1, 1, 1, 1],
    );
    // Stray continuation bytes between well-formed characters.
    check(
        b"i\xc3\x80\xa0\xe2\xb1\xa0\xa0\xe2\xb1\xa0\xf0\x90\x80\x80\xa0i",
        &[0, 0, 1, 0, 0, 1, 2, 0, 0, 1, 2, 0, 1, 2, 3, 0, 0],
        &[1, 2, 1, 1, 3, 2, 1, 1, 3, 2, 1, 4, 3, 2, 1, 1, 1],
    );
}

/// The walk is bounded on both sides: it never reads before the base it was
/// given, and never past the length it was given. Both fixtures hand it the
/// interior of a four-byte sequence and expect it to give up rather than
/// reconstruct it.
#[test]
fn the_bounds_walk_stays_inside_what_it_was_given() {
    let _editor = editor_lock();
    let whole = cbuf("\u{10000}".as_bytes());
    for base_at in [1, 0] {
        let base = &whole[base_at..];
        let (mut begin, mut end) = (Vec::new(), Vec::new());
        for i in 0..3 {
            let left = 3 - c_int::try_from(i).expect("three");
            // SAFETY: `base` has at least three bytes plus a terminator.
            let bounds = unsafe { utf_cp_bounds_len(base.as_ptr(), base.as_ptr().add(i), left) };
            begin.push(bounds.begin_off);
            end.push(bounds.end_off);
        }
        assert_eq!(
            (&begin[..], &end[..]),
            (&[0, 0, 0][..], &[1, 1, 1][..]),
            "starting at byte {base_at}"
        );
    }
}

/// `utf_head_off` answers how far back the *grapheme cluster* containing a
/// byte starts — a different question from the one above, because a cluster
/// can span several codepoints. The fixtures are the awkward ones: ZWJ
/// sequences, regional indicators, tag sequences, invalid bytes wedged
/// between clusters, embedded NULs, and Arabic, whose clustering an option
/// decides. All byte literals, for the same reason as above.
#[test]
fn every_byte_points_back_at_the_start_of_its_grapheme_cluster() {
    let _editor = editor_lock();

    /// Walk `raw` with `utfc_ptr2len`, collect the multi-byte clusters, then
    /// check that every byte reports its offset from its cluster's first.
    #[track_caller]
    fn check(raw: &[u8], want_clusters: &[&[u8]]) {
        let buf = cbuf(raw);
        let mut breaks = vec![0_usize];
        let mut clusters: Vec<&[u8]> = Vec::new();
        let mut pos = 0;
        while pos < raw.len() {
            // SAFETY: `buf` is NUL-terminated and `pos` is inside it.
            let answered = unsafe { utfc_ptr2len(buf.as_ptr().add(pos)) };
            let mut len = usize::try_from(answered).expect("never negative");
            if len == 0 {
                assert_eq!(raw[pos], 0, "only a NUL has length zero");
                len = 1; // ...but step over it, or the walk never ends.
            }
            if len > 1 {
                clusters.push(&raw[pos..pos + len]);
            }
            pos += len;
            breaks.push(pos);
        }
        assert_eq!(pos, raw.len(), "the walk lands exactly on the end");
        assert_eq!(clusters, want_clusters, "clusters of {raw:x?}");

        for pair in breaks.windows(2) {
            let (start, next) = (pair[0], pair[1]);
            for p in start..next {
                // SAFETY: both pointers are inside `buf`.
                let back = unsafe { utf_head_off(buf.as_ptr(), buf.as_ptr().add(p)) };
                assert_eq!(
                    usize::try_from(back).expect("never negative"),
                    p - start,
                    "at byte {p} of {raw:x?}"
                );
            }
        }
        // SAFETY: the terminator is inside `buf`.
        let at_nul = unsafe { utf_head_off(buf.as_ptr(), buf.as_ptr().add(raw.len())) };
        assert_eq!(at_nul, 0, "the NUL past the end is its own cluster");
    }

    // "hej och hå 🧑‍🌾!"
    check(
        b"hej och h\xc3\xa5 \xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe!",
        &[b"\xc3\xa5", b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe"],
    );
    // Five ZWJ/variation-selector emoji, back to back.
    check(
        b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe\xe2\x9d\xa4\xef\xb8\x8f\xf0\x9f\x98\x82\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f",
        &[
            b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe",
            b"\xe2\x9d\xa4\xef\xb8\x8f",
            b"\xf0\x9f\x98\x82",
            b"\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f",
        ],
    );
    // The same, separated by ASCII and a carriage return.
    check(
        b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8fxy\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe\x0d\xe2\x9d\xa4\xef\xb8\x8f\xf0\x9f\x98\x82\xc3\xa5\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f\xc2\x80",
        &[
            b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe",
            b"\xe2\x9d\xa4\xef\xb8\x8f",
            b"\xf0\x9f\x98\x82",
            b"\xc3\xa5",
            b"\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f",
            b"\xc2\x80",
        ],
    );
    // ...and separated by NULs, which have length zero and end a cluster.
    check(
        b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f\x00\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe\x00\xe2\x9d\xa4\xef\xb8\x8f\x00\xf0\x9f\x98\x82\x00\xc3\xa5\x00\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f\x00\xc2\x80",
        &[
            b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe",
            b"\xe2\x9d\xa4\xef\xb8\x8f",
            b"\xf0\x9f\x98\x82",
            b"\xc3\xa5",
            b"\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f",
            b"\xc2\x80",
        ],
    );
    // ...and separated by bytes that begin no sequence at all.
    check(
        b"\xc3\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f\xc6\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe\xa5\xe2\x9d\xa4\xef\xb8\x8f\xa8\xc3\xf0\x9f\x98\x82\xff\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f\x81\xc2\x80\xa5",
        &[
            b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xe2\x9a\xa7\xef\xb8\x8f",
            b"\xf0\x9f\xa7\x91\xe2\x80\x8d\xf0\x9f\x8c\xbe",
            b"\xe2\x9d\xa4\xef\xb8\x8f",
            b"\xf0\x9f\x98\x82",
            b"\xf0\x9f\x8f\xb4\xe2\x80\x8d\xe2\x98\xa0\xef\xb8\x8f",
            b"\xc2\x80",
        ],
    );
    // Regional indicators pair up left to right; an odd one stands alone.
    check(
        b"\xf0\x9f\x87\xa6\xf0\x9f\x85\xb1\xef\xb8\x8f \xf0\x9f\x87\xa6\xf0\x9f\x87\xbd \xf0\x9f\x87\xa6\xf0\x9f\x87\xa8\xf0\x9f\x87\xa6 \xf0\x9f\x87\xb2\xf0\x9f\x87\xbd\xf0\x9f\x87\xb9\xf0\x9f\x87\xb1",
        &[
            b"\xf0\x9f\x87\xa6",
            b"\xf0\x9f\x85\xb1\xef\xb8\x8f",
            b"\xf0\x9f\x87\xa6\xf0\x9f\x87\xbd",
            b"\xf0\x9f\x87\xa6\xf0\x9f\x87\xa8",
            b"\xf0\x9f\x87\xa6",
            b"\xf0\x9f\x87\xb2\xf0\x9f\x87\xbd",
            b"\xf0\x9f\x87\xb9\xf0\x9f\x87\xb1",
        ],
    );
    // Tag sequences: the Scottish and Welsh flags.
    check(
        b"\xf0\x9f\x8f\xb4\xf3\xa0\x81\xa7\xf3\xa0\x81\xa2\xf3\xa0\x81\xb3\xf3\xa0\x81\xa3\xf3\xa0\x81\xb4\xf3\xa0\x81\xbf\xf0\x9f\x8f\xb4\xf3\xa0\x81\xa7\xf3\xa0\x81\xa2\xf3\xa0\x81\xb7\xf3\xa0\x81\xac\xf3\xa0\x81\xb3\xf3\xa0\x81\xbf",
        &[
            b"\xf0\x9f\x8f\xb4\xf3\xa0\x81\xa7\xf3\xa0\x81\xa2\xf3\xa0\x81\xb3\xf3\xa0\x81\xa3\xf3\xa0\x81\xb4\xf3\xa0\x81\xbf",
            b"\xf0\x9f\x8f\xb4\xf3\xa0\x81\xa7\xf3\xa0\x81\xa2\xf3\xa0\x81\xb7\xf3\xa0\x81\xac\xf3\xa0\x81\xb3\xf3\xa0\x81\xbf",
        ],
    );
    // Two-byte characters with junk bytes and a NUL between them.
    check(
        b"\xc3\xa5\xa5\xc3\xbc\xc3a\xc3\xabq\xa8\xce\xb2\x00\xa9\xe6\x9c\xac\xff",
        &[
            b"\xc3\xa5",
            b"\xc3\xbc",
            b"\xc3\xab",
            b"\xce\xb2",
            b"\xe6\x9c\xac",
        ],
    );
    // Six composing marks on each of five bases, which is more than a
    // `schar_T` holds inline.
    check(
        b"L\xcc\x93\xcc\x89\xcc\x91\xcc\x92\xcc\x8c\xcc\x9ao\xcc\x8c\xcc\x92\xcc\x97\xcc\x84\xcc\x9b\xcc\x80r\xcc\x81\xcc\x88\xcc\x95\xcc\x88\xcc\x8e\xcc\x90e\xcc\x80\xcc\x87\xcc\x85\xcc\x84\xcc\x84\xcc\x90m\xcc\x85\xcc\x96\xcc\x9f\xcc\x84\xcc\x9f\xcc\x9a",
        &[
            b"L\xcc\x93\xcc\x89\xcc\x91\xcc\x92\xcc\x8c\xcc\x9a",
            b"o\xcc\x8c\xcc\x92\xcc\x97\xcc\x84\xcc\x9b\xcc\x80",
            b"r\xcc\x81\xcc\x88\xcc\x95\xcc\x88\xcc\x8e\xcc\x90",
            b"e\xcc\x80\xcc\x87\xcc\x85\xcc\x84\xcc\x84\xcc\x90",
            b"m\xcc\x85\xcc\x96\xcc\x9f\xcc\x84\xcc\x9f\xcc\x9a",
        ],
    );

    // `'arabicshape'` joins lam-alef into one cluster; with it off they stay
    // two. This is the only fixture whose clustering an option decides, and
    // the only reason this file needs the editor lock for more than `curbuf`.
    let saved = p_arshape.get();
    p_arshape.set(1);
    check(
        b"\xd8\xb3\xd9\x84\xd8\xa7\xd9\x85",
        &[b"\xd8\xb3", b"\xd9\x84\xd8\xa7", b"\xd9\x85"],
    );
    p_arshape.set(0);
    check(
        b"\xd8\xb3\xd9\x84\xd8\xa7\xd9\x85",
        &[b"\xd8\xb3", b"\xd9\x84", b"\xd8\xa7", b"\xd9\x85"],
    );
    p_arshape.set(saved);
}

/// `#30527`: folding is a table lookup, and the table does not cover the
/// surrogate range or anything past the last plane. Every one of those has to
/// come back unchanged rather than index out of bounds.
#[test]
fn folding_leaves_everything_it_has_no_entry_for_alone() {
    let _editor = editor_lock();
    for c in [0xddfb, 0xd800, 9_000_000, 0, -1, c_int::MAX] {
        assert_eq!(utf_fold(c), c, "{c:#x}");
    }
    // ...and still folds what it does have an entry for.
    assert_eq!(utf_fold(i32::from(b'A')), i32::from(b'a'));
    assert_eq!(utf_fold(0x00c0), 0x00e0, "À folds to à");
}
