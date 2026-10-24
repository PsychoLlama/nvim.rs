//! UTF-8 decoding, grapheme clustering and the character-boundary walk.
//!
//! A port of `test/unit/mbyte_spec.lua`. The leaf arithmetic — the lead-byte
//! masking, the encode/decode round trip, the fold table — has in-crate tests
//! already. What the spec was the only oracle for is `mbyte/walk.rs` (520
//! lines, no tests) and the composing-character rules of upstream's
//! `utfc_ptr2schar`/`utfc_ptr2len` -- [`c_cluster_schar`]/[`c_cluster_len`]
//! here, the same readers over the bytes of a C string -- which is where
//! every sequence below is aimed.
//!
//! `vim_iswordc` reads `curbuf` and the cluster walk reads `'arabicshape'`,
//! so every case here holds the editor lock.

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::charset::{vim_iswordc, vim_iswordp};
use neovim::grid::MAX_SCHAR_SIZE;
use neovim::harness::schar_get;
use neovim::mbyte::{
    c_cells_at, c_char_info, c_cluster_len, c_cluster_schar, c_head_off, c_next_cluster,
    c_off_next, c_partial_cells_in, c_prev_start, c_string_cells, c_string_cells_len, cells_at,
    char_at, char_count, char_info_at, char_len, cluster_len, cp_bounds, encode_char,
    strict_char_at, string_cells, utf_fold, utf_ptr2char, utf_ptr2len,
};
use neovim::option::vars::P_ARSHAPE;

use crate::support::editor_lock;

/// `s` as a NUL-terminated C buffer. Every entry point below reads to the
/// terminator and no further, so the terminator is what bounds them.
fn cbuf(s: &[u8]) -> Vec<c_char> {
    s.iter()
        .map(|&b| b as c_char)
        .chain(std::iter::once(0))
        .collect()
}

/// [`c_cluster_schar`] unpacked: (the cluster's bytes, its first codepoint).
///
/// Known blind spot, inherited from the spec: nothing here reaches the byte
/// the packer holds back for the space it synthesises in front of a
/// cluster that opens with a composing character. Removing that reservation
/// changes no answer below, because no fixture is anywhere near
/// `MAX_SCHAR_SIZE` long.
///
/// The `ScreenChar` it answers is an index into the glyph cache for anything too
/// long to sit inline, so reading it back goes through `schar_get`. That
/// makes this a statement about the pair, which is how the editor uses them.
fn cluster(seq: &[u8]) -> (Vec<u8>, c_int) {
    let mut out = [0 as c_char; MAX_SCHAR_SIZE as usize + 1];
    let (sc, firstc) = c_cluster_schar(seq);
    // SAFETY: `out` has room for any glyph and the terminator.
    let len = unsafe { schar_get(out.as_mut_ptr(), sc) };
    let text = out[..len].iter().map(|&b| b as u8).collect();
    (text, firstc)
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
    let mut buf = [0u8; 8];
    for c in 0..0x10000_i32 {
        // Eight bytes, and the encoder writes at most four here.
        encode_char(c, &mut buf);
        let buf: [c_char; 8] = buf.map(|b| b as c_char);
        // SAFETY: the read back stops at the sequence just written.
        unsafe {
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

/// `cp_bounds` answers, for every byte of a string, how far back the
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
        let (mut begin, mut end) = (Vec::new(), Vec::new());
        for i in 0..raw.len() {
            let bounds = cp_bounds(raw, i);
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
    let whole = "\u{10000}".as_bytes();
    for base_at in [1, 0] {
        // Three bytes from the base: the walk is given no more.
        let base = &whole[base_at..base_at + 3];
        let (mut begin, mut end) = (Vec::new(), Vec::new());
        for i in 0..3 {
            let bounds = cp_bounds(base, i);
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

/// [`c_head_off`] answers how far back the *grapheme cluster* containing a
/// byte starts — a different question from the one above, because a cluster
/// can span several codepoints. The fixtures are the awkward ones: ZWJ
/// sequences, regional indicators, tag sequences, invalid bytes wedged
/// between clusters, embedded NULs, and Arabic, whose clustering an option
/// decides. All byte literals, for the same reason as above.
#[test]
fn every_byte_points_back_at_the_start_of_its_grapheme_cluster() {
    let _editor = editor_lock();

    /// Walk `raw` with [`c_cluster_len`], collect the multi-byte clusters,
    /// then check that every byte reports its offset from its cluster's
    /// first.
    #[track_caller]
    fn check(raw: &[u8], want_clusters: &[&[u8]]) {
        let mut breaks = vec![0_usize];
        let mut clusters: Vec<&[u8]> = Vec::new();
        let mut pos = 0;
        while pos < raw.len() {
            let mut len = c_cluster_len(&raw[pos..]);
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
                assert_eq!(c_head_off(raw, p), p - start, "at byte {p} of {raw:x?}");
            }
        }
        assert_eq!(
            c_head_off(raw, raw.len()),
            0,
            "the NUL past the end is its own cluster"
        );
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
    // `ScreenChar` holds inline.
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
    let saved = P_ARSHAPE.get();
    P_ARSHAPE.set(true);
    check(
        b"\xd8\xb3\xd9\x84\xd8\xa7\xd9\x85",
        &[b"\xd8\xb3", b"\xd9\x84\xd8\xa7", b"\xd9\x85"],
    );
    P_ARSHAPE.set(false);
    check(
        b"\xd8\xb3\xd9\x84\xd8\xa7\xd9\x85",
        &[b"\xd8\xb3", b"\xd9\x84", b"\xd8\xa7", b"\xd9\x85"],
    );
    P_ARSHAPE.set(saved);
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

/// Byte strings covering every shape the decoders distinguish: ASCII, each
/// sequence length, an overlong form, a truncated one, a bad continuation
/// byte, a bare continuation byte, `0xFE`/`0xFF`, the five- and six-byte
/// sequences only the original UTF-8 had, and a composing pair.
const SLICE_CASES: &[&[u8]] = &[
    b"",
    b"a",
    b"\x7f",
    b"\xc3\xa9",
    b"\xe2\x82\xac",
    b"\xf0\x9f\x92\xa9",
    b"\xf8\x88\x80\x80\x80",
    b"\xfc\x84\x80\x80\x80\x80",
    b"\xc0\x80",
    b"\xc3",
    b"\xe2\x82",
    b"\xc3z",
    b"\x80",
    b"\xfe",
    b"\xff",
    b"\xe4\xb8\xad\xe6\x96\x87",
    b"e\xcc\x81",
    b"\xe0\xae\xb9\xe0\xaf\x8di",
];

/// The `&[u8]` codec answers what the C-string codec (the pointer readers'
/// spelling, and the two the harness still resolves) answers for the same
/// bytes. This is the whole contract phase 26's rewrites lean on: a walk
/// converted from `*const c_char` to a slice and an index decodes text the
/// same way, so the conversion cannot change what the editor displays.
///
/// The C-string forms stop at a NUL and the slice forms only at their
/// length, so the fixtures below carry no embedded NUL; that one deliberate
/// difference is stated in `mbyte/utf8/slice.rs`'s own tests.
#[test]
fn the_slice_codec_answers_what_the_pointer_codec_answers() {
    let _editor = editor_lock();
    for case in SLICE_CASES {
        let buf = cbuf(case);
        // SAFETY: `buf` is NUL-terminated, which is all these forms ask for.
        let (want_len, want_char) =
            unsafe { (utf_ptr2len(buf.as_ptr()), utf_ptr2char(buf.as_ptr())) };
        let want_cluster = c_cluster_len(case);
        assert_eq!(char_len(case), want_len as usize, "char_len {case:x?}");
        assert_eq!(char_at(case), want_char, "char_at {case:x?}");
        assert_eq!(cluster_len(case), want_cluster, "cluster_len {case:x?}");
    }
}

/// How wide one character is drawn, for every shape the width rules answer
/// differently: plain ASCII, a control character, an ambiguous-width
/// character under the default `'ambiwidth'`, an East Asian wide one, a
/// combining mark (which is *one* cell on its own -- it only vanishes when it
/// is part of a cluster), an emoji, an emoji requested with VS-16, and the
/// three ways a byte can fail to be a character.
///
/// Bounded widths come with it: [`c_partial_cells_in`] is asked for every prefix
/// length, which is where its truncation rule shows. That rule is not the one
/// its comment claims -- see [`a_sequence_cut_short_is_measured_past_the_cut`].
#[test]
fn the_cells_a_character_occupies() {
    let _editor = editor_lock();
    #[track_caller]
    fn check(raw: &[u8], want: c_int) {
        assert_eq!(c_cells_at(raw), want, "c_cells_at {raw:x?}");
        let bounded = c_partial_cells_in(raw, 0, raw.len());
        assert_eq!(bounded, want, "c_partial_cells_in {raw:x?}");
    }

    check(b"a", 1);
    check(b"\x01", 1); // the escape's width is `char2cells`', not this one's
    check(b"\xc2\xa9", 1); // (c) U+00A9, ambiguous: 'ambiwidth' is "single"
    check(b"\xe4\xb8\xad", 2); // U+4E2D, East Asian wide
    check(b"\xef\xbc\xa1", 2); // U+FF21, fullwidth
    check(b"\xcc\x81", 1); // a lone combining acute is still a character
    check(b"\xf0\x9f\x92\xa9", 2); // U+1F4A9, emoji
    check(b"\xe2\x98\x80", 1); // U+2600, ambiguous rather than emoji-wide
    check(b"\xe2\x9d\xa4\xef\xb8\x8f", 2); // U+2764 + VS-16 asks for emoji
    check(b"\x80", 4); // a continuation byte on its own: <80>
    check(b"\xfe", 4); // never a lead byte: <fe>
    check(b"\xc0\x80", 4); // overlong NUL: not a character at all
    check(b"\xe0\x80\x80", 4); // overlong, decoding to NUL again
    check(b"\xc1\xbf", 2); // overlong U+007F, drawn as the escape ^?
}

/// The one place the bounded width parts company with the terminated one: a
/// sequence the size cuts short.
///
/// Upstream's `utf_ptr2cells_len` comment says such a sequence answers 1, and
/// it cannot: `utf_ptr2len_len` reports a truncated sequence's *full* length, so
/// the `len < utf8len_tab[*p]` test never fires for one, and the decode that
/// follows reads past the cut. Upstream has the same body. What the size
/// really bounds is therefore only the VS-16 lookahead; the character itself
/// is measured out of whatever bytes are there.
#[test]
fn a_sequence_cut_short_is_measured_past_the_cut() {
    let _editor = editor_lock();
    // The bytes are all present, so cutting the size does not change the
    // answer -- the decode reads them anyway.
    let whole = b"\xe4\xb8\xad";
    for size in 1..=3 {
        assert_eq!(c_partial_cells_in(whole, 0, size), 2, "{size}");
    }
    // Cut in the buffer rather than by the size: now the byte the decode
    // reaches is the terminator, and the sequence is illegal.
    let cut = b"\xe4\xb8";
    for size in 1..=2 {
        assert_eq!(c_partial_cells_in(cut, 0, size), 4, "{size}");
    }
    // The VS-16 lookahead is bounded only by whether it *starts* inside the
    // size: `size > len` is the whole test, and the same full-length report
    // then lets the decode read the rest of it.
    let heart = b"\xe2\x9d\xa4\xef\xb8\x8f";
    for size in 1..=3 {
        assert_eq!(c_partial_cells_in(heart, 0, size), 1, "{size}");
    }
    for size in 4..=6 {
        assert_eq!(c_partial_cells_in(heart, 0, size), 2, "{size}");
    }
}

/// Characters and cells over a whole string, in both the NUL-terminated and
/// the bounded spelling.
///
/// The counters walk in *clusters*: a combining mark, a VS-16 and a ZWJ
/// sequence each count once and contribute their base character's width.
/// Bytes that are not characters count one each and are drawn as `<xx>`.
#[test]
fn counting_characters_and_cells_across_a_string() {
    let _editor = editor_lock();
    #[track_caller]
    fn check(raw: &[u8], want_chars: usize, want_cells: usize) {
        assert_eq!(char_count(raw), want_chars, "char_count {raw:x?}");
        assert_eq!(c_string_cells(raw), want_cells, "c_string_cells {raw:x?}");
        assert_eq!(
            c_string_cells_len(raw, raw.len()),
            want_cells,
            "c_string_cells_len {raw:x?}"
        );
    }

    check(b"", 0, 0);
    check(b"abc", 3, 3);
    check(b"a\xe4\xb8\xadb", 3, 4);
    check(b"e\xcc\x81x", 2, 2); // e + combining acute is one cluster
    check(b"\xe2\x9d\xa4\xef\xb8\x8fz", 2, 3); // the VS-16 joins the heart
    check(b"\x80\xfe\xc3", 3, 12); // three <xx> escapes
    check(b"\xf0\x9f\x91\xa8\xe2\x80\x8d\xf0\x9f\x91\xa9!", 2, 3); // ZWJ
}

/// The character walk: [`c_char_info`] opens it and [`c_next_cluster`] steps
/// it, one step per *cluster* but carrying the *base* character's codepoint
/// and length. [`c_off_next`] and [`c_prev_start`] are the same geometry
/// asked from a byte in the middle.
///
/// The last fixture is the one that says the two lengths are different
/// questions: the step from the heart to the `z` is six bytes, and the
/// `CharInfo` it carries says three.
#[test]
fn the_character_walk_steps_over_whole_clusters() {
    let _editor = editor_lock();
    #[track_caller]
    fn check(raw: &[u8], want_walk: &[(usize, i32, c_int)], want_prev: &[usize]) {
        let mut walk = Vec::new();
        let (mut at, mut chr) = (0, c_char_info(raw, 0));
        while at < raw.len() && raw[at] != 0 {
            walk.push((at, chr.value, chr.len));
            (at, chr) = c_next_cluster(raw, at, chr);
        }
        assert_eq!(walk, want_walk, "the walk over {raw:x?}");

        // `c_prev_start` from every byte, the terminator included, and
        // `c_off_next` from every byte: how far the *next* character start
        // is, which is zero at a character start.
        let starts: Vec<usize> = walk.iter().map(|&(at, _, _)| at).collect();
        let prev: Vec<usize> = (0..=raw.len()).map(|i| c_prev_start(raw, i)).collect();
        assert_eq!(prev, want_prev, "c_prev_start over {raw:x?}");

        for i in 0..raw.len() {
            let next = starts.iter().copied().find(|&s| s >= i);
            let want = next.map_or(0, |s| s - i);
            assert_eq!(c_off_next(raw, i), want, "c_off_next at {i} of {raw:x?}");
        }
    }

    check(b"abc", &[(0, 97, 1), (1, 98, 1), (2, 99, 1)], &[0, 0, 1, 2]);
    check(
        b"a\xe4\xb8\xadb",
        &[(0, 97, 1), (1, 0x4e2d, 3), (4, 98, 1)],
        &[0, 0, 1, 1, 1, 4],
    );
    check(b"e\xcc\x81x", &[(0, 101, 1), (3, 120, 1)], &[0, 0, 0, 0, 3]);
    // Not a character anywhere: every byte is its own, and the strict
    // decoder reports each as negative.
    check(
        b"\x80\xfe\xc3",
        &[(0, -1, 1), (1, -1, 1), (2, -1, 1)],
        &[0, 0, 1, 2],
    );
    check(
        b"\xe2\x9d\xa4\xef\xb8\x8fz",
        &[(0, 0x2764, 3), (6, 122, 1)],
        &[0, 0, 0, 0, 0, 0, 0, 6],
    );
}

/// The rest of the `&[u8]` API answers what its C-string twin answers, over
/// the same fixtures the codec equivalence uses.
///
/// Everything here is measured against the form that stops at a NUL,
/// because "the end of the slice is the end of the string" is the slice
/// convention. [`c_partial_cells_in`] is deliberately absent: it is not the
/// twin of [`cells_at`] but a different function, as
/// [`a_sequence_cut_short_is_measured_past_the_cut`] shows.
#[test]
fn the_slice_api_answers_what_the_pointer_api_answers() {
    let _editor = editor_lock();
    for case in SLICE_CASES {
        assert_eq!(cells_at(case), c_cells_at(case), "cells {case:x?}");
        assert_eq!(
            string_cells(case),
            c_string_cells(case),
            "string_cells {case:x?}"
        );
        if !case.is_empty() {
            let info = c_char_info(case, 0);
            assert_eq!(char_info_at(case).value, info.value, "value {case:x?}");
            assert_eq!(char_info_at(case).len, info.len, "info len {case:x?}");
        }
    }
}

/// `strict_char_at` is the decoder that says "not a character": negative for
/// an incomplete sequence, a bad continuation byte, or a byte that leads
/// nothing, and the codepoint otherwise. `char_at` answers the lead byte's
/// own value for all three, which is the pair the module documents.
#[test]
fn the_strict_decoder_rejects_what_the_forgiving_one_passes_through() {
    for (case, want) in [
        (b"a".as_slice(), 0x61),
        (b"\x7f", 0x7f),
        (b"\xc3\xa9", 0xe9),
        (b"\xe2\x82\xac", 0x20ac),
        (b"\xf0\x9f\x92\xa9", 0x1f4a9),
        (b"\xc0\x80", 0), // an overlong NUL decodes, to zero
    ] {
        assert_eq!(strict_char_at(case), want, "{case:x?}");
    }
    for case in [
        b"".as_slice(),
        b"\x80", // a continuation byte leads nothing
        b"\xfe", // never a lead byte
        b"\xc3", // incomplete
        b"\xe2\x82",
        b"\xc3z", // a bad continuation byte
    ] {
        assert!(strict_char_at(case) < 0, "{case:x?}");
        assert_eq!(char_at(case), i32::from(case.first().copied().unwrap_or(0)));
    }
}
