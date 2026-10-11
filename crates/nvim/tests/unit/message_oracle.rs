//! The message layer's text shapers -- the middle cut of `trunc_string`,
//! the head cut of `msg_may_trunc`, and the `<>` rendering of
//! `str2special` -- folded into one number each over `support::corpus`,
//! recorded from the pointer forms upstream ported. The rewrite onto slices
//! and owned answers has to reproduce every digest (see `mbyte_oracle.rs`
//! for the rule); only the adapters at the top of this file may change.

#![cfg(not(miri))]

use std::ffi::{CString, c_int};

use neovim::ex_docmd::do_cmdline_cmd;
use neovim::keycodes::escape_ks_bytes;

use crate::support::corpus::{Digest, corpus};
use crate::support::editor_lock;

/// `text` cut to `room` cells with a `...` in the middle, into a buffer of
/// `capacity` bytes.
fn middle_cut(text: &[u8], room: c_int, capacity: usize) -> Vec<u8> {
    neovim::message::cut_middle(text, room, capacity)
}

/// `text` with its head dropped to fit the message area, marked with `<`.
fn head_cut(text: &[u8], force: bool) -> Vec<u8> {
    neovim::message::fit_head(text, force).into_owned()
}

/// The `<>` rendering of every key in `text`.
fn keys_shown(text: &[u8], replace_spaces: bool, replace_lt: bool) -> Vec<u8> {
    neovim::message::special_keys_text(text, replace_spaces, replace_lt)
}

/// The corpus, some long lines, and key-code text: the corpus with its
/// `K_SPECIAL` bytes escaped, and the escapes for a few special keys.
fn texts() -> Vec<Vec<u8>> {
    let mut out = corpus();
    out.push(b"a text that nott fits".to_vec());
    out.push("一二三四五六七八九十".repeat(4).into_bytes());
    out.push(b"abcdefghijklmnopqrstuvwxyz".repeat(5));
    out.push("e\u{301}abcdefghijklmnop".repeat(3).into_bytes());
    let escaped: Vec<Vec<u8>> = corpus().iter().map(|text| escape_ks_bytes(text)).collect();
    out.extend(escaped);
    for keys in [
        &b"\x80\xfc\x04a"[..],
        b"\x80\xfc\x02\x80ku",
        b"\x80\xfc\x04\xc3\xa9",
        b"\x80\xfc\x08\x80\xfe\x58",
        b"\x80\xfe\x58",
        b"\x80\xfd\x04",
        b"\x80ku\x80kd",
        b" lead and trail ",
        b"<lt>",
        b"\x01\x1f\x7f",
        b"\x80",
        b"\x80\xfc",
        b"\x80\xfc\x04",
    ] {
        out.push(keys.to_vec());
    }
    out
}

#[test]
fn the_middle_cut() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        for room in [-1, 0, 1, 2, 3, 4, 5, 7, 10, 11, 20, 33, 79, 200] {
            // Only a buffer the whole text fits in: the pointer form wrote
            // past a smaller one when a character straddled its end, and
            // left the copy unterminated when the text fitted the room but
            // not the buffer. `message.rs` holds the fixed answers there.
            for capacity in [1, 2, 3, 4, 6, 8, 12, 40, 200, 1025] {
                if capacity > text.len() {
                    d.bytes(&middle_cut(&text, room, capacity));
                }
            }
        }
    }
    assert_eq!(d.value(), 0x52e4_56f1_a8b8_b063, "trunc_string");
}

#[test]
fn the_head_cut() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    // The room is the screen's, less the command line's: move both, so the
    // values every case starts from are set here rather than left by an
    // earlier case in the lane.
    for setting in [
        c"set lines=25 cmdheight=2",
        c"set lines=24 cmdheight=1",
        c"silent! new",
        c"silent! close",
    ] {
        do_cmdline_cmd(setting).expect("valid");
    }
    // And put the command line where the window layout says it is.
    neovim::ex_getln::compute_cmdrow();
    for (columns, shm) in [(80, "+=t"), (40, "+=t"), (20, "+=t"), (40, "-=t")] {
        do_cmdline_cmd(
            &CString::new(format!(
                "set lines=24 cmdheight=1 noruler noshowcmd columns={columns} shortmess{shm}"
            ))
            .unwrap(),
        )
        .expect("valid");
        for text in texts() {
            for force in [false, true] {
                d.bytes(&head_cut(&text, force));
            }
        }
    }
    do_cmdline_cmd(c"set columns=80 shortmess& ruler& showcmd&").expect("valid");
    assert_eq!(d.value(), 0x702e_1265_a86d_8e80, "msg_may_trunc");
}

#[test]
fn keys_rendered() {
    let _editor = editor_lock();
    let mut d = Digest::new();
    for text in texts() {
        for (spaces, lt) in [(false, false), (true, false), (false, true), (true, true)] {
            d.bytes(&keys_shown(&text, spaces, lt));
        }
    }
    assert_eq!(d.value(), 0x8414_0814_338f_b649, "str2special");
}
