//! The pointer-form multibyte readers, folded into one number each over a
//! corpus of hostile text (`support::corpus`), under every option setting
//! that changes an answer.
//!
//! These are the oracle for rewriting the readers onto slices: each digest
//! below was recorded from the pointer forms as upstream ported them, and a
//! rewrite -- a pointer form turned into a shim over its slice twin, or a
//! caller moved onto the twin and the pointer form deleted -- has to
//! reproduce it. A digest that changes is a behaviour change, to be
//! explained or fixed, never re-recorded to make the test pass.
//!
//! The pointer forms are gone now except the two the C harness resolves by
//! name (`utf_ptr2char`, `utf_ptr2len`): every other reader below is its
//! `c_*` twin over the same bytes -- the bytes of a C string, ending at the
//! first NUL -- and reproduced the digest recorded from the pointer form.

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::ex_docmd::do_cmdline_cmd;
use neovim::grid::MAX_SCHAR_SIZE;
use neovim::harness::schar_get;
use neovim::mbyte::{
    c_ambiguous_width, c_cells_at, c_char_info, c_cluster_len, c_cluster_schar, c_head_off,
    c_next_cluster, c_off_next, c_partial_cells_in, c_prev_start, c_string_cells,
    c_string_cells_len, c_utf_index_to_byte, c_utf_lengths, char_count, class_in, cluster_len,
    cp_bounds, encode_char, promised_char_len, strcmp_ic_in, strnicmp_in, utf_ptr2char,
    utf_ptr2len, valid_utf8,
};
use neovim::memory::XString;
use neovim::option::vars::{P_AMBW, P_ARSHAPE, P_EMOJI};
use neovim::winlayer::Buf;

use crate::support::corpus::{Digest, corpus};
use crate::support::editor_lock;

/// `s` as a NUL-terminated C buffer.
fn cbuf(s: &[u8]) -> Vec<c_char> {
    s.iter()
        .map(|&b| b as c_char)
        .chain(std::iter::once(0))
        .collect()
}

/// The option settings an answer here can depend on: `'ambiwidth'`,
/// `'emoji'` and `'arabicshape'`, in four combinations that between them
/// flip each one against the others.
const SETTINGS: [(&str, bool, bool); 4] = [
    ("single", true, false),
    ("double", false, true),
    ("single", false, true),
    ("double", true, false),
];

/// Run `f` once per [`SETTINGS`] row over the whole corpus, folding what it
/// answers into one digest, and put the options back.
fn digest(f: impl Fn(&mut Digest, &[u8], &[c_char])) -> u64 {
    let _editor = editor_lock();
    let saved = (P_AMBW.swap(None), P_EMOJI.get(), P_ARSHAPE.get());
    // The character tables a width or class answer reads, at their
    // defaults whatever an earlier case left behind.
    for cmd in [
        c"set isprint=@,161-255 display=lastline,msgsep",
        c"setlocal iskeyword=@,48-57,_,192-255",
    ] {
        do_cmdline_cmd(cmd).expect("the option values are valid");
    }
    let mut digest = Digest::new();
    for (ambw, emoji, arshape) in SETTINGS {
        P_AMBW.swap(Some(XString::from(ambw)));
        P_EMOJI.set(emoji);
        P_ARSHAPE.set(arshape);
        for text in corpus() {
            let buf = cbuf(&text);
            f(&mut digest, &text, &buf);
        }
    }
    P_AMBW.swap(saved.0);
    P_EMOJI.set(saved.1);
    P_ARSHAPE.set(saved.2);
    digest.value()
}

/// `buf`'s bytes, its terminator included.
fn cstr_bytes(buf: &[c_char]) -> Vec<u8> {
    buf.iter().map(|&b| b as u8).collect()
}

/// The bytes of `text` up to its first NUL, as a C string would have them.
fn until_nul(text: &[u8]) -> &[u8] {
    text.split(|&b| b == 0).next().unwrap_or(text)
}

/// The drawing walk's digest. It was `0x8245ccc2ae2e9065` with the
/// length-taking packer (`utfc_ptrlen2schar`) folded in too; that packer's
/// last caller went, and with it the packer. This value was recorded from
/// the pointer forms with only that packer's lines taken out, and the twins
/// reproduce it.
const DRAWING_WALK: u64 = 0xad76c89878a519dd;

/// The lengths a bounded reader is asked about at offset `at` of `text`:
/// nothing, the first few bytes, and everything left.
fn bounds(text: &[u8], at: usize) -> impl Iterator<Item = usize> {
    let left = text.len() - at;
    [0, 1, 2, 3, 4, 6, left]
        .into_iter()
        .filter(move |&n| n <= left)
}

#[test]
fn the_codepoint_readers() {
    let got = digest(|d, text, buf| {
        let p = buf.as_ptr();
        for i in 0..=text.len() {
            // SAFETY: `buf` is NUL-terminated and `i` is inside it.
            unsafe {
                d.add(i64::from(utf_ptr2char(p.add(i))));
                d.add(i64::from(utf_ptr2len(p.add(i))));
            }
            let info = c_char_info(text, i);
            d.add(i64::from(info.value));
            d.add(i64::from(info.len));
            for n in bounds(text, i).filter(|&n| n > 0) {
                d.add(promised_char_len(&text[i..i + n]) as i64);
            }
        }
    });
    assert_eq!(got, 0xb5ecfb633bb01b25, "codepoint readers");
}

#[test]
fn the_cluster_readers() {
    let got = digest(|d, text, buf| {
        let _ = buf;
        for i in 0..=text.len() {
            d.add(c_cluster_len(&text[i..]) as i64);
            d.add(c_head_off(text, i) as i64);
            d.add(c_off_next(text, i) as i64);
            d.add(c_prev_start(text, i) as i64);
            for n in bounds(text, i) {
                d.add(cluster_len(&text[i..i + n]) as i64);
                d.add(char_count(until_nul(&text[i..i + n])) as i64);
            }
        }
        d.add(char_count(until_nul(text)) as i64);
    });
    assert_eq!(got, 0xc865c1fa01f61725, "cluster readers");
}

/// The drawing path's walk: the first character, then the next cluster to
/// the NUL, and the cluster-to-`ScreenChar` packer at every offset.
#[test]
fn the_drawing_walk() {
    let got = digest(|d, text, _buf| {
        let byte = |at: usize| text.get(at).copied().unwrap_or(0);
        for i in 0..text.len() {
            let (mut at, mut chr) = (i, c_char_info(text, i));
            while byte(at) != 0 {
                d.add(at as i64);
                d.add(i64::from(chr.value));
                d.add(i64::from(chr.len));
                (at, chr) = c_next_cluster(text, at, chr);
            }
            d.add(at as i64);
        }
        let mut out = [0 as c_char; MAX_SCHAR_SIZE as usize + 1];
        for i in 0..=text.len() {
            let (sc, firstc) = c_cluster_schar(&text[i..]);
            // SAFETY: `out` has room for any glyph and its NUL.
            let n = unsafe { schar_get(out.as_mut_ptr(), sc) };
            d.add(i64::from(firstc));
            d.add(n as i64);
            for &b in &out[..n] {
                d.add(i64::from(b as u8));
            }
        }
    });
    assert_eq!(got, DRAWING_WALK, "drawing walk");
}

#[test]
fn the_width_readers() {
    let got = digest(|d, text, buf| {
        let _ = buf;
        for i in 0..=text.len() {
            d.add(i64::from(c_cells_at(&text[i..])));
            d.add(i64::from(c_ambiguous_width(&text[i..])));
            for n in bounds(text, i) {
                d.add(i64::from(c_partial_cells_in(&text[i..], 0, n)));
                d.add(c_string_cells_len(&text[i..], n) as i64);
            }
        }
        d.add(c_string_cells(text) as i64);
    });
    assert_eq!(got, 0x30bc2fa857ad5ffb, "width readers");
}

#[test]
fn the_bounds_and_class_readers() {
    let got = digest(|d, text, buf| {
        // The string and its terminator: the unbounded form looked a
        // sequence's worth past `i`, and the NUL is what it found there.
        let with_nul = cstr_bytes(buf);
        for i in 0..text.len() {
            let b = cp_bounds(&with_nul[..(i + 6).min(with_nul.len())], i);
            d.add(i64::from(b.begin_off));
            d.add(i64::from(b.end_off));
            for n in bounds(text, i).filter(|&n| n > 0) {
                let b = cp_bounds(&text[..i + n.min(6)], i);
                d.add(i64::from(b.begin_off));
                d.add(i64::from(b.end_off));
            }
            d.add(i64::from(class_in(&text[i..], Buf::current())));
        }
        for i in 0..=text.len() {
            for n in bounds(text, i) {
                let (points, units) = c_utf_lengths(&text[i..], n);
                d.add(points as i64);
                d.add(units as i64);
                for index in 0..4 {
                    for utf16 in [false, true] {
                        let at = c_utf_index_to_byte(&text[i..], n, index, utf16);
                        d.add(at.map_or(-1, |at| at as i64));
                    }
                }
            }
        }
        d.add(i64::from(valid_utf8(text)));
        for k in 0..=text.len() {
            d.add(i64::from(valid_utf8(&text[..k])));
        }
    });
    assert_eq!(got, 0xf1feeec46fdfa4ad, "bounds and class readers");
}

#[test]
fn the_advancing_readers() {
    let got = digest(|d, text, buf| {
        // What `mb_ptr2char_adv` and `mb_cptr2char_adv` did, as their
        // callers now spell it: decode, then step a cluster or a character.
        let p = buf.as_ptr();
        // SAFETY (the second): `at` is inside the NUL-terminated `buf`.
        let steps: [&dyn Fn(usize) -> usize; 2] = [
            &|at| c_cluster_len(&text[at..]),
            &|at| unsafe { utf_ptr2len(p.add(at)) } as usize,
        ];
        for step in steps {
            let mut at = 0;
            while at < text.len() && text[at] != 0 {
                // SAFETY: as above.
                d.add(i64::from(unsafe { utf_ptr2char(p.add(at)) }));
                at += step(at);
                d.add(at as i64);
            }
        }
    });
    assert_eq!(got, 0x5ca496d6218b19e5, "advancing readers");
}

/// `utf_strnicmp` over every pair of neighbouring corpus strings, each
/// against the other and against an upper-cased copy of itself, at a few
/// lengths -- the bytewise fallback after a bad sequence included.
#[test]
fn case_insensitive_comparison() {
    let _editor = editor_lock();
    let texts = corpus();
    let mut d = Digest::new();
    let upper = |s: &[u8]| -> Vec<u8> {
        let mut out = s.to_vec();
        out.make_ascii_uppercase();
        // é upper-cased by hand, and Ä lower-cased: both fold to the same
        // character as the original, through a different byte.
        for i in 1..out.len() {
            if out[i - 1] == 0xc3 && (out[i] == 0xa9 || out[i] == 0x84) {
                out[i] ^= 0x20;
            }
        }
        out
    };
    for pair in texts.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let a_up = upper(a);
        for (x, y) in [(a, b), (b, a), (a, &a_up), (&a_up, a)] {
            // At most `n` bytes, and never past a NUL: a C string's prefix.
            let prefix =
                |s: &'_ [u8], n: usize| -> Vec<u8> { until_nul(&s[..n.min(s.len())]).to_vec() };
            for n1 in [0, 1, 3, x.len(), x.len() + 1] {
                for n2 in [0, 2, y.len(), y.len() + 1] {
                    let c = strnicmp_in(&prefix(x, n1), &prefix(y, n2));
                    d.add(i64::from(c.signum()));
                    d.add(i64::from(c));
                }
                d.add(i64::from(strnicmp_in(&prefix(x, n1), &prefix(y, n1))));
            }
            d.add(i64::from(strcmp_ic_in(true, until_nul(x), until_nul(y))));
        }
    }
    assert_eq!(d.value(), 0x3d00abbaa93d7f04, "case-insensitive comparison");
}

/// The encoder over every length's boundaries and a sweep of the range.
#[test]
fn the_encoder() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    let mut codes: Vec<c_int> = (0..0x900).collect();
    for boundary in [0x800, 0x10000, 0x200000, 0x4000000] {
        codes.extend(boundary - 3..boundary + 3);
    }
    codes.extend((0x10000..0x7fff_ffff).step_by(0x3_1d31));
    codes.push(c_int::MAX);
    for c in codes {
        let mut buf = [0u8; 8];
        // Eight bytes hold the longest sequence.
        let len = encode_char(c, &mut buf);
        d.add(len as i64);
        for &b in &buf[..len] {
            d.add(i64::from(b));
        }
    }
    assert_eq!(d.value(), 0xa19c0fe513221d76, "encoder");
}
