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

#![cfg(not(miri))]

use std::ffi::{c_char, c_int};

use neovim::ex_docmd::do_cmdline_cmd;
use neovim::grid::{MAX_SCHAR_SIZE, schar_get};
use neovim::mbyte::{
    mb_charlen, mb_charlen_len, mb_cptr2char_adv, mb_get_class, mb_off_next, mb_prevptr,
    mb_ptr2char_adv, mb_stricmp, mb_string2cells, mb_string2cells_len, mb_strnicmp,
    mb_utf_index_to_bytes, mb_utflen, utf_ambiguous_width, utf_char2bytes, utf_cp_bounds,
    utf_cp_bounds_len, utf_head_off, utf_ptr2cells, utf_ptr2cells_len, utf_ptr2char,
    utf_ptr2char_info, utf_ptr2len, utf_ptr2len_len, utf_ptr2str_char_info, utf_strnicmp,
    utf_valid_string, utfc_next, utfc_ptr2len, utfc_ptr2len_len, utfc_ptr2schar, utfc_ptrlen2schar,
};
use neovim::memory::XString;
use neovim::option::vars::{P_AMBW, P_ARSHAPE, P_EMOJI};

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
                let info = utf_ptr2char_info(p.add(i));
                d.add(i64::from(info.value));
                d.add(i64::from(info.len));
                for n in bounds(text, i).filter(|&n| n > 0) {
                    d.add(i64::from(utf_ptr2len_len(p.add(i), n as c_int)));
                }
            }
        }
    });
    assert_eq!(got, 0xb5ecfb633bb01b25, "codepoint readers");
}

#[test]
fn the_cluster_readers() {
    let got = digest(|d, text, buf| {
        let p = buf.as_ptr();
        for i in 0..=text.len() {
            // SAFETY: as above; every bound is inside the string.
            unsafe {
                d.add(i64::from(utfc_ptr2len(p.add(i))));
                d.add(i64::from(utf_head_off(p, p.add(i))));
                d.add(i64::from(mb_off_next(p, p.add(i))));
                let prev = mb_prevptr(p.cast_mut(), p.add(i).cast_mut());
                d.add(prev.offset_from(p) as i64);
                for n in bounds(text, i) {
                    d.add(i64::from(utfc_ptr2len_len(p.add(i), n as c_int)));
                    d.add(i64::from(mb_charlen_len(p.add(i), n as c_int)));
                }
            }
        }
        // SAFETY: as above.
        unsafe { d.add(i64::from(mb_charlen(p))) };
    });
    assert_eq!(got, 0xc865c1fa01f61725, "cluster readers");
}

/// The drawing path's walk: `utf_ptr2str_char_info` then `utfc_next` to the
/// NUL, and the two cluster-to-`ScreenChar` packers at every offset.
#[test]
fn the_drawing_walk() {
    let got = digest(|d, text, buf| {
        let p = buf.as_ptr();
        // SAFETY: as above; each step lands inside `buf`.
        unsafe {
            for i in 0..text.len() {
                let mut cur = utf_ptr2str_char_info(p.add(i).cast_mut());
                while *cur.ptr != 0 {
                    d.add(cur.ptr.offset_from(p) as i64);
                    d.add(i64::from(cur.chr.value));
                    d.add(i64::from(cur.chr.len));
                    cur = utfc_next(cur);
                }
                d.add(cur.ptr.offset_from(p) as i64);
            }
            let mut out = [0 as c_char; MAX_SCHAR_SIZE as usize];
            for i in 0..=text.len() {
                let mut firstc: c_int = 0;
                let sc = utfc_ptr2schar(p.add(i), &raw mut firstc);
                let n = schar_get(out.as_mut_ptr(), sc);
                d.add(i64::from(firstc));
                d.add(n as i64);
                for &b in &out[..n] {
                    d.add(i64::from(b as u8));
                }
                // As `grid_line_puts` calls it: with the cluster's length.
                for left in bounds(text, i) {
                    let len = utfc_ptr2len_len(p.add(i), left as c_int);
                    let sc = utfc_ptrlen2schar(p.add(i), len, &raw mut firstc);
                    let n = schar_get(out.as_mut_ptr(), sc);
                    d.add(i64::from(firstc));
                    for &b in &out[..n] {
                        d.add(i64::from(b as u8));
                    }
                }
            }
        }
    });
    assert_eq!(got, 0x8245ccc2ae2e9065, "drawing walk");
}

#[test]
fn the_width_readers() {
    let got = digest(|d, text, buf| {
        let p = buf.as_ptr();
        for i in 0..=text.len() {
            // SAFETY: as above.
            unsafe {
                d.add(i64::from(utf_ptr2cells(p.add(i))));
                d.add(i64::from(utf_ambiguous_width(p.add(i))));
                for n in bounds(text, i) {
                    d.add(i64::from(utf_ptr2cells_len(p.add(i), n as c_int)));
                    d.add(mb_string2cells_len(p.add(i), n) as i64);
                }
            }
        }
        // SAFETY: as above.
        unsafe { d.add(mb_string2cells(p) as i64) };
    });
    assert_eq!(got, 0x30bc2fa857ad5ffb, "width readers");
}

#[test]
fn the_bounds_and_class_readers() {
    let got = digest(|d, text, buf| {
        let p = buf.as_ptr();
        for i in 0..text.len() {
            // SAFETY: as above; `i` is a byte of the string.
            unsafe {
                let b = utf_cp_bounds(p, p.add(i));
                d.add(i64::from(b.begin_off));
                d.add(i64::from(b.end_off));
                for n in bounds(text, i).filter(|&n| n > 0) {
                    let b = utf_cp_bounds_len(p, p.add(i), n as c_int);
                    d.add(i64::from(b.begin_off));
                    d.add(i64::from(b.end_off));
                }
                d.add(i64::from(mb_get_class(p.add(i))));
            }
        }
        for i in 0..=text.len() {
            for n in bounds(text, i) {
                let (mut points, mut units) = (0, 0);
                // SAFETY: `n` bytes from `i` are inside the string, and a
                // sequence they cut short is completed by the bytes after.
                unsafe {
                    mb_utflen(p.add(i), n, &raw mut points, &raw mut units);
                    d.add(points as i64);
                    d.add(units as i64);
                    for index in 0..4 {
                        for utf16 in [false, true] {
                            d.add(mb_utf_index_to_bytes(p.add(i), n, index, utf16) as i64);
                        }
                    }
                }
            }
        }
        // SAFETY: as above.
        unsafe {
            d.add(i64::from(utf_valid_string(p, std::ptr::null())));
            for k in 0..=text.len() {
                d.add(i64::from(utf_valid_string(p, p.add(k))));
            }
        }
    });
    assert_eq!(got, 0xf1feeec46fdfa4ad, "bounds and class readers");
}

#[test]
fn the_advancing_readers() {
    let got = digest(|d, _text, buf| {
        for advance in [mb_ptr2char_adv, mb_cptr2char_adv] {
            let mut cursor = buf.as_ptr();
            // SAFETY: each step stays inside the NUL-terminated `buf`.
            unsafe {
                while *cursor != 0 {
                    d.add(i64::from(advance(&raw mut cursor)));
                    d.add(cursor.offset_from(buf.as_ptr()) as i64);
                }
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
            let (bx, by) = (cbuf(x), cbuf(y));
            for n1 in [0, 1, 3, x.len(), x.len() + 1] {
                for n2 in [0, 2, y.len(), y.len() + 1] {
                    // SAFETY: `n1`/`n2` reach at most the terminator.
                    let c = unsafe { utf_strnicmp(bx.as_ptr(), by.as_ptr(), n1, n2) };
                    d.add(i64::from(c.signum()));
                    d.add(i64::from(c));
                }
                // SAFETY: as above.
                d.add(i64::from(unsafe {
                    mb_strnicmp(bx.as_ptr(), by.as_ptr(), n1)
                }));
            }
            // SAFETY: both are NUL-terminated.
            d.add(i64::from(unsafe { mb_stricmp(bx.as_ptr(), by.as_ptr()) }));
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
        let mut buf = [0 as c_char; 8];
        // SAFETY: eight bytes hold the longest sequence.
        let len = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) };
        d.add(i64::from(len));
        for &b in &buf[..len as usize] {
            d.add(i64::from(b as u8));
        }
    }
    assert_eq!(d.value(), 0xa19c0fe513221d76, "encoder");
}
