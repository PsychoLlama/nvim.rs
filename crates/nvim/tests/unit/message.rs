//! Fitting a message into the columns there are: `cut_middle`, upstream's
//! `trunc_string`.
//!
//! Every `:` command that reports a long file name goes through this, via
//! `fit_middle`, and the shape it produces — head, `...`, tail — is what
//! the user reads. `test/unit/message_spec.lua` covered five ASCII strings
//! against one room and one buffer size, in place and by copy; the copy is
//! the only form now.
//!
//! The truncation reads the display width of each character out of the
//! global `chartab`, so every case takes the editor lock rather than
//! standing up its own table.

#![cfg(not(miri))]

use std::ffi::c_int;

use neovim::message::cut_middle;

use crate::support::{Editor, editor_lock};

/// The Lua spec's buffer size. Big enough that nothing here hits the
/// "cannot even fit the `...`" arm unless a case asks for it.
const BUFLEN: usize = 40;

/// Truncate `s` into a buffer of `buflen` bytes.
fn by_copy(_editor: &Editor, s: &str, room: c_int, buflen: usize) -> Vec<u8> {
    cut_middle(s.as_bytes(), room, buflen)
}

/// Every case through the one form, so a case never has to say which.
#[track_caller]
fn both(editor: &Editor, s: &str, room: c_int, expected: &str) {
    assert_eq!(
        String::from_utf8_lossy(&by_copy(editor, s, room, BUFLEN)),
        expected,
        "{s:?} in {room} cells"
    );
}

#[test]
fn a_string_that_fits_is_copied_whole() {
    let editor = editor_lock();
    both(&editor, "text", 20, "text");
    both(&editor, "a short text", 20, "a short text");
    both(&editor, "a text that fits", 34, "a text that fits");
    // Exactly `room` cells is a fit.
    both(&editor, "a text tha just fits", 20, "a text tha just fits");
}

#[test]
fn one_cell_too_many_puts_an_ellipsis_in_the_middle() {
    let editor = editor_lock();
    both(&editor, "a text that nott fits", 20, "a text t...nott fits");
    // The result is `room` cells wide, head and tail split around it.
    let out = by_copy(&editor, "abcdefghijklmnopqrstuvwxyz", 10, BUFLEN);
    assert_eq!(out, b"abc...wxyz");
}

#[test]
fn an_empty_string_stays_empty() {
    let editor = editor_lock();
    both(&editor, "", 20, "");
    // …and it is the one input that does not even look at `room`.
    both(&editor, "", 0, "");
}

/// With no room for the `...` there is nothing to keep: `room_in < 3` zeroes
/// the half the head would have taken, and the tail walk then rejects every
/// character too, so the whole answer is the ellipsis.
#[test]
fn a_room_too_small_for_the_ellipsis_keeps_nothing_but_the_ellipsis() {
    let editor = editor_lock();
    for room in [3, 2, 1, 0, -1] {
        assert_eq!(by_copy(&editor, "abcdefgh", room, BUFLEN), b"...", "{room}");
    }
    // Four cells is the first room that keeps anything, and what it keeps is
    // one cell of tail: the head half is `(4 - 3) / 2`, which is zero.
    assert_eq!(by_copy(&editor, "abcdefgh", 4, BUFLEN), b"...h");
}

/// The split is on character boundaries, not byte boundaries: a multibyte
/// character is copied whole or not at all, and its *cell* width is what
/// counts against `room`.
#[test]
fn a_multibyte_string_is_split_between_characters() {
    let editor = editor_lock();
    // Ten double-width characters: twenty cells, thirty bytes.
    let cjk = "一二三四五六七八九十";
    assert_eq!(cjk.len(), 30);
    both(&editor, cjk, 22, cjk);

    let out = by_copy(&editor, cjk, 10, BUFLEN);
    let text = String::from_utf8(out).expect("whole characters only");
    assert_eq!(text, "一...九十");
}

/// Upstream quirk, asserted so a change to it is deliberate: a string that
/// is *exactly* `room` cells wide survives whole only when its characters
/// are one cell each.
///
/// The decision is `i <= e + 3`, comparing the head's end and the tail's
/// start as **byte** offsets after both were chosen by **cell** width. With
/// one byte per cell the two halves meet and the fit is seen; with three
/// bytes per two cells the tail's byte offset overshoots the head's by more
/// than the three bytes the `...` stands for, so a twenty-cell line in
/// twenty columns is truncated to nineteen.
#[test]
fn an_exact_fit_is_only_recognised_for_single_width_characters() {
    let editor = editor_lock();
    both(&editor, "a text tha just fits", 20, "a text tha just fits");
    let cjk = "一二三四五六七八九十";
    assert_eq!(
        String::from_utf8(by_copy(&editor, cjk, 20, BUFLEN)).unwrap(),
        "一二三四...七八九十"
    );
}

/// A composing character rides along with the base it belongs to, because
/// the head copies a whole cluster at a time.
#[test]
fn a_composing_character_is_not_separated_from_its_base() {
    let editor = editor_lock();
    // "éa…" with the accent as a combining mark, then plain ASCII.
    let s = "e\u{0301}abcdefghijklmnop";
    let out = by_copy(&editor, s, 10, BUFLEN);
    let text = String::from_utf8(out).expect("whole characters only");
    assert!(text.starts_with("e\u{0301}"), "got {text:?}");
    assert!(text.contains("..."), "got {text:?}");
    assert!(text.ends_with("mnop"), "got {text:?}");
}

/// When even the head plus `...` will not fit in the *buffer*, the answer is
/// whatever fits, at most `buflen - 1` bytes — never a write past the end.
#[test]
fn a_buffer_too_small_for_the_ellipsis_is_merely_cut() {
    let editor = editor_lock();
    let long = "abcdefghijklmnopqrstuvwxyz";
    for buflen in 1..=12 {
        let out = by_copy(&editor, long, 20, buflen);
        assert!(
            out.len() < buflen,
            "buflen {buflen} produced {} bytes",
            out.len()
        );
    }
    // Room for a five-cell head but only eight bytes of buffer: the `...`
    // and the tail have nowhere to go.
    assert_eq!(by_copy(&editor, long, 20, 8), b"abcdefg");
}

/// The history as `:messages` prints it, one entry a line.
fn history(probe: &crate::support::Probe<'_>) -> String {
    probe.output("messages")
}

/// `:echomsg` and errors land in the history and `:echo` does not; a count
/// keeps the newest, `:messages clear` empties it, and `'messagesopt'`'s
/// `history:` caps it, dropping the oldest.
#[test]
fn the_history_keeps_what_was_said_and_drops_the_oldest() {
    let editor = editor_lock();
    let probe = crate::support::Probe::new(&editor);
    probe.run("messages clear");
    probe.run("echomsg 'one'");
    probe.run("echo 'not kept'");
    probe.run("echomsg 'two'");
    // Under `:silent!` an error sets `v:errmsg` and is not kept.
    probe.run("silent! call nosuchfunction()");
    probe.run("echomsg \"\\nthree\\n\"");
    probe.check_output("messages", "\none\ntwo\nthree");
    probe.check_output("2messages", "\ntwo\nthree");
    probe.run("set messagesopt=hit-enter,history:2");
    probe.check_output("messages", "\ntwo\nthree");
    probe.run("echomsg 'four'");
    probe.check_output("messages", "\nthree\nfour");
    probe.run("set messagesopt&");
    probe.run("messages clear");
    probe.check_output("messages", "");
}

/// A message longer than a line is kept whole, a multibyte and a control
/// byte as themselves: the history holds the text, not its rendering.
#[test]
fn the_history_holds_the_text_not_its_rendering() {
    let editor = editor_lock();
    let probe = crate::support::Probe::new(&editor);
    probe.run("messages clear");
    let long = "x".repeat(300);
    probe.run(&format!("echomsg '{long}'"));
    probe.run("echomsg \"caf\\u00e9 \\x01\"");
    let text = history(&probe);
    assert_eq!(text, format!("\n{long}\ncafé ^A"));
    probe.run("messages clear");
}

/// `'messagesopt'` refuses what it cannot use and leaves the old value.
#[test]
fn messagesopt_refuses_what_it_cannot_use() {
    let editor = editor_lock();
    let probe = crate::support::Probe::new(&editor);
    for bad in [
        "history:500",
        "hit-enter",
        "wait:10001,history:5",
        "hit-enter,history:10001",
        "hit-enter,history:",
        "hit-enter,history:5,bogus",
    ] {
        probe.check_error(
            &format!("set messagesopt={bad}"),
            &format!("E474: Invalid argument: messagesopt={bad}"),
        );
    }
    probe.check_error("set messagesopt=wait:0,history:0,progress:c", "none");
    probe.check_error("set messagesopt=history:7,hit-enter", "none");
    // An empty item between two commas is no item at all.
    probe.check_error("set messagesopt=hit-enter,,history:5", "none");
    probe.check("&messagesopt", "'hit-enter,,history:5'");
    probe.run("set messagesopt=history:7,hit-enter");
    probe.check("&messagesopt", "'history:7,hit-enter'");
    probe.run("set messagesopt&");
}

/// Two cases upstream got wrong, fixed: a text that fits the room but not the
/// buffer was copied without a terminator (so the reader ran on past the
/// buffer), and a character straddling the end of the buffer made the head
/// loop step one past it and write the terminator there. Both answer the
/// first `buflen - 1` bytes of what the buffer would hold.
#[test]
fn a_buffer_smaller_than_the_text_is_never_overrun() {
    let editor = editor_lock();
    // Fits twenty cells, not five bytes.
    assert_eq!(by_copy(&editor, "abcdefgh", 20, 5), b"abcd");
    // Three-byte characters, one cell each: the head wants ten cells and the
    // buffer has room for four bytes, so the second character straddles it.
    let wide = "\u{2026}".repeat(30);
    for buflen in 1..=12 {
        let out = by_copy(&editor, &wide, 23, buflen);
        assert!(
            out.len() < buflen,
            "buflen {buflen} produced {} bytes",
            out.len()
        );
    }
    // Every room and size: nothing past the buffer, ever.
    for room in -1..40 {
        for buflen in 1..40 {
            let out = by_copy(&editor, &wide, room, buflen);
            assert!(out.len() < buflen, "{room}/{buflen}: {} bytes", out.len());
        }
    }
}
