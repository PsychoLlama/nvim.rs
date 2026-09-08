//! Turning bytes into something displayable.
//!
//! The `msg_outtrans*` half renders unprintable bytes as `<xx>` and multibyte
//! sequences as themselves; the `str2special*` half renders key codes as
//! `<C-X>` notation, which is what mapping listings and `keytrans()` show.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::charset::CharDisplay;
use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::{Key, MAX_KEY_NAME_LEN, SpecialKeyName, termcap_key, termcap_name};
use crate::mbyte::{cells_at, char_at, cluster_len};
use crate::memory::handoff::owned_cstr;
use crate::types::MB_MAXCHAR;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

/// The `<xx>` form of an unprintable byte gets its own highlight so it can be
/// told apart from the same characters typed literally.
const SPECIAL_HL: c_int = HLF_8;

/// Show one character.
pub fn msg_putchar(c: c_int) {
    msg_putchar_hl(c, 0)
}

/// Show one character with a highlight id.
///
/// A special key is put back into the three-byte `K_SPECIAL` form it arrived
/// as, because that is what [`msg_outtrans_len`] and `str2special` downstream
/// know how to read.
pub fn msg_putchar_hl(c: c_int, hl_id: c_int) {
    let mut buf = [0 as c_char; MB_MAXCHAR + 1];
    if c < 0 {
        // `K_SECOND`/`K_THIRD`, less their `c == K_SPECIAL`/`c == NUL`
        // arms: both of those codes are positive, so neither is reachable
        // here.
        let name = termcap_name(c);
        buf[0] = K_SPECIAL as c_char;
        buf[1] = name[0] as c_char;
        buf[2] = name[1] as c_char;
    } else {
        let len = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) };
        buf[len as usize] = 0;
    }
    unsafe { msg_puts_hl(buf.as_ptr(), hl_id, false) }
}

/// Show a number in decimal.
pub fn msg_outnum(n: c_int) {
    // Filled from the right so the digits come out in order; the last byte
    // stays zero and terminates it.
    let mut buf = [0u8; 16];
    let mut at = buf.len() - 1;
    let mut rest = n.unsigned_abs();
    loop {
        at -= 1;
        buf[at] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    if n < 0 {
        at -= 1;
        buf[at] = b'-';
    }
    unsafe { msg_puts(buf[at..].as_ptr().cast()) }
}

/// Show a file name with `$HOME` folded back to `~`.
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string.
pub unsafe fn msg_home_replace(fname: *const c_char) {
    unsafe { msg_home_replace_hl(fname, 0) }
}

/// # Safety
///
/// `fname` must point at a NUL-terminated string.
pub(crate) unsafe fn msg_home_replace_hl(fname: *const c_char, hl_id: c_int) {
    let name = unsafe { home_replace_save(None, fname) };
    unsafe { msg_outtrans(name, hl_id, false) };
    unsafe { xfree(name.cast()) };
}

/// Show a NUL-terminated string, translating what cannot be displayed.
///
/// Answers how many screen cells it took.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string.
pub unsafe fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int {
    unsafe { msg_outtrans_len(str, cstr::bytes_at(str).len() as c_int, hl_id, hist) }
}

/// Show the one character at `p`, answering a pointer to the next one.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
pub unsafe fn msg_outtrans_one(p: *const c_char, hl_id: c_int, hist: bool) -> *const c_char {
    let len = unsafe { utfc_ptr2len(p) };
    if len > 1 {
        unsafe { msg_outtrans_len(p, len, hl_id, hist) };
        return unsafe { p.add(len as usize) };
    }
    let display = unsafe { transchar_byte_buf(None, *p as u8 as c_int) };
    unsafe { msg_puts_hl(display.as_ptr(), hl_id, hist) };
    unsafe { p.add(1) }
}

/// Show `len` bytes of `msgstr`, NULs included, translating what cannot be
/// displayed.
///
/// Answers how many screen cells it took.
///
/// # Safety
/// `msgstr` must point at `len` readable bytes.
pub unsafe fn msg_outtrans_len(
    msgstr: *const c_char,
    len: c_int,
    hl_id: c_int,
    hist: bool,
) -> c_int {
    debug_assert!(len >= 0, "a negative length has no pointer form left");
    // SAFETY: the caller's contract.
    let bytes = unsafe { slice::from_raw_parts(msgstr.cast::<u8>(), len.cast_unsigned() as usize) };
    msg_display_bytes(bytes, hl_id, hist)
}

/// Show `bytes`, NULs included, translating what cannot be displayed.
///
/// Printable runs are handed to [`msg_puts_len`] whole; only the characters
/// that need a `^X` or `<xx>` rendering are emitted one at a time, in the
/// `SPECIAL_HL` highlight.
///
/// Answers how many screen cells it took.
pub fn msg_display_bytes(bytes: &[u8], hl_id: c_int, hist: bool) -> c_int {
    // Only quit when got_int was set in here.
    let save_got_int = got_int.get();
    got_int.set(false);

    if hist {
        // SAFETY: `bytes` is readable for its own length.
        unsafe { msg_hist_add(bytes.as_ptr().cast(), bytes.len() as c_int, hl_id) };
    }

    // When drawing over the command line there is no need to clear it
    // later or to remove the mode message.
    if msg_silent.get() == 0
        && !bytes.is_empty()
        && msg_row.get() >= cmdline_row.get()
        && msg_col.get() == 0
    {
        clear_cmdline.set(false);
        mode_displayed.set(false);
    }

    let cells = walk_display(bytes, &mut |shown| match shown {
        Shown::Plain(run) => {
            // SAFETY: `run` is a subslice of `bytes`, so it is readable for
            // its own length.
            unsafe { msg_puts_len(run.as_ptr().cast(), run.len() as ptrdiff_t, hl_id, hist) };
        }
        Shown::Instead(text) => {
            // SAFETY: a `CharDisplay` is NUL-terminated.
            unsafe { msg_puts_hl(text.as_ptr(), special_hl(hl_id), false) };
        }
    });

    got_int.set(got_int.get() | save_got_int);
    cells
}

/// One run of a message on its way to the screen.
enum Shown<'a> {
    /// Bytes that display as themselves.
    Plain(&'a [u8]),
    /// The `^X` / `<xx>` rendering standing in for one character that does
    /// not.
    Instead(CharDisplay),
}

/// Hand `emit` each run of `bytes` in the order it is shown, and answer how
/// many screen cells they take.
///
/// Printable characters accumulate into a run that is emitted whole; one
/// that cannot be displayed ends the run, comes out on its own as its
/// rendering, and starts the next. An *empty* message emits one empty run,
/// which is what clears the message line, and is why the emptiness test
/// below asks whether anything was ever replaced rather than whether the run
/// is empty.
///
/// The walk stops at an interrupt, and then so does the message: a run left
/// pending when `got_int` arrives is dropped rather than shown.
fn walk_display(bytes: &[u8], emit: &mut impl FnMut(Shown<'_>)) -> c_int {
    let mut cells = 0;
    // Start of the run of printable bytes not yet emitted.
    let mut run = 0;
    let mut at = 0;
    while at < bytes.len() && !got_int.get() {
        let rest = &bytes[at..];
        // A composing character only partly inside `bytes` is left out.
        let len = cluster_len(rest);
        if len > 1 {
            let c = char_at(rest);
            if vim_isprintc(c) {
                // SAFETY: a message is shown long after options exist.
                cells += unsafe { cells_at(rest) };
            } else {
                if at > run {
                    emit(Shown::Plain(&bytes[run..at]));
                }
                emit(Shown::Instead(transchar_buf(None, c)));
                // SAFETY: as above.
                cells += unsafe { char2cells(c) };
                run = at + len;
            }
            at += len;
        } else {
            let rendered = transchar_byte_buf(None, c_int::from(rest[0]));
            if rendered[1] != 0 {
                if at > run {
                    emit(Shown::Plain(&bytes[run..at]));
                }
                cells += cstr::in_chars(&rendered).count_bytes() as c_int;
                emit(Shown::Instead(rendered));
                run = at + 1;
            } else {
                cells += 1;
            }
            at += 1;
        }
    }

    // The printable characters at the end -- or, for a message that reached
    // here with nothing replaced, the empty run the callers rely on.
    if (at > run || run == 0) && !got_int.get() {
        emit(Shown::Plain(&bytes[run..at]));
    }
    cells
}

/// Unprintable characters take `SPECIAL_HL` unless the caller asked for a
/// highlight of its own.
fn special_hl(hl_id: c_int) -> c_int {
    if hl_id == 0 { SPECIAL_HL } else { hl_id }
}

/// `:smile`.
///
/// # Safety
///
/// `arg` must point at a NUL-terminated string.
pub unsafe fn msg_make(arg: *const c_char) {
    // The command name backwards, and the answer with every byte shifted up
    // by three -- both so that neither reads as itself in the binary.
    const REVERSED: &[u8] = b"eeffoc";
    const SHIFTED: &[u8] = b"Plon#dqg#vxjduB";

    let mut arg = unsafe { skipwhite(arg) };
    let mut at = REVERSED.len() as isize - 1;
    while unsafe { *arg } != 0 && at >= 0 {
        let byte = unsafe { *arg as u8 };
        arg = unsafe { arg.add(1) };
        if byte != REVERSED[at as usize] {
            break;
        }
        at -= 1;
    }
    if at < 0 {
        msg_putchar(NL);
        for &byte in SHIFTED {
            msg_putchar((byte - 3) as c_int);
        }
    }
}

/// Show a string with key codes rendered as `<C-X>` notation, the way a
/// mapping is listed.
///
/// A leading or trailing space is shown as `<Space>` so it cannot be missed.
/// Stops before exceeding `maxlen` screen columns; 0 means unlimited.
///
/// @param from  true for the left-hand side of a mapping
///
/// # Safety
///
/// `strstart` must point at a NUL-terminated string.
pub unsafe fn msg_outtrans_special(strstart: *const c_char, from: bool, maxlen: c_int) -> c_int {
    if strstart.is_null() {
        return 0;
    }
    let mut piece: SpecialKeyName = [0; MAX_KEY_NAME_LEN as usize + 1];
    let mut display: CharDisplay;
    let mut str = strstart;
    let mut cells = 0;
    while unsafe { *str } != 0 {
        let mut text = if unsafe { *str } == b' ' as c_char
            && (str == strstart || unsafe { *str.add(1) } == 0)
        {
            str = unsafe { str.add(1) };
            c"<Space>".as_ptr()
        } else {
            unsafe { str2special(&raw mut str, from, false, &mut piece) }
        };
        if unsafe { *text } != 0 && unsafe { *text.add(1) } == 0 {
            // Single-byte character, or an illegal byte.
            display = unsafe { transchar_byte_buf(None, *text as u8 as c_int) };
            text = display.as_ptr();
        }
        let len = unsafe { vim_strsize(text) };
        if maxlen > 0 && cells + len >= maxlen {
            break;
        }
        // Highlight the ones that came out as a `<>` name.
        let hl_id = if len > 1 && unsafe { utfc_ptr2len(text) } <= 1 {
            SPECIAL_HL
        } else {
            0
        };
        unsafe { msg_puts_hl(text, hl_id, false) };
        cells += len;
    }
    cells
}

/// [`str2special`] over a whole string, into a freshly allocated one.
///
/// The caller owns the result and frees it with `xfree`.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string.
pub unsafe fn str2special_save(
    str: *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
) -> *mut c_char {
    let mut piece: SpecialKeyName = [0; MAX_KEY_NAME_LEN as usize + 1];
    let mut out = Vec::<u8>::new();
    let mut p = str;
    while unsafe { *p } != 0 {
        let text = unsafe { str2special(&raw mut p, replace_spaces, replace_lt, &mut piece) };
        // SAFETY: `str2special` answers a NUL-terminated rendering.
        out.extend_from_slice(unsafe { cstr::bytes_at(text) });
    }
    owned_cstr(out)
}

/// [`str2special`] over a whole string, into `arena`.
///
/// Measures first and copies second, so that the arena is asked for the
/// exact size once.
///
/// # Safety
///
/// `str` must point at a NUL-terminated string. `arena` must point at a live
/// arena, which the memory this answers with is taken from and must outlive.
pub unsafe fn str2special_arena(
    str: *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
    arena: *mut Arena,
) -> *mut c_char {
    let mut piece: SpecialKeyName = [0; MAX_KEY_NAME_LEN as usize + 1];
    let mut len: size_t = 0;
    let mut p = str;
    while unsafe { *p } != 0 {
        let text = unsafe { str2special(&raw mut p, replace_spaces, replace_lt, &mut piece) };
        len += unsafe { cstr::bytes_at(text) }.len();
    }

    let buf: *mut c_char = unsafe { arena_alloc(arena, len + 1, false) }.cast();
    let mut at: size_t = 0;
    p = str;
    while unsafe { *p } != 0 {
        let text = unsafe { str2special(&raw mut p, replace_spaces, replace_lt, &mut piece) };
        let piece_len = unsafe { cstr::bytes_at(text) }.len();
        unsafe { ptr::copy_nonoverlapping(text, buf.add(at), piece_len) };
        at += piece_len;
    }
    unsafe { *buf.add(at) = 0 };
    buf
}

/// Render one key code as printable text, advancing `*sp` past it.
///
/// Special keys and C0 control characters come out in `<>` form;
/// `replace_spaces` and `replace_lt` extend that to `<Space>` and `<lt>`,
/// which is what a mapping's left-hand side and `keytrans()` want and its
/// right-hand side does not.
///
/// The answer is written into `out` and answered as a pointer to it; an
/// illegal byte comes back as itself. Upstream answers one shared static
/// buffer, which is why `str2special_arena` cannot hold two answers.
///
/// # Safety
/// `cursor` must point at a readable pointer into a NUL-terminated string.
pub(crate) unsafe fn str2special(
    cursor: *mut *const c_char,
    replace_spaces: bool,
    replace_lt: bool,
    out: &mut SpecialKeyName,
) -> *const c_char {
    let mut ch = [0 as c_char; MB_MAXCHAR];
    // A multi-byte character escaped into the stream comes back whole.
    if !unsafe { mb_unescape(cursor, &mut ch) }.is_null() {
        out[..MB_MAXCHAR].copy_from_slice(&ch);
        return out.as_ptr();
    }

    let mut str = unsafe { *cursor };
    let mut c = unsafe { *str as u8 as c_int };
    let mut modifiers = ModMask::NONE;
    let mut special = false;
    if c == K_SPECIAL && unsafe { *str.add(1) } != 0 && unsafe { *str.add(2) } != 0 {
        if unsafe { *str.add(1) as u8 as c_int } == KS_MODIFIER {
            modifiers = ModMask::from_bits(unsafe { *str.add(2) as u8 as c_int });
            str = unsafe { str.add(3) };
            c = unsafe { *str as u8 as c_int };
        }
        if c == K_SPECIAL && unsafe { *str.add(1) } != 0 && unsafe { *str.add(2) } != 0 {
            c = to_special(unsafe { *str.add(1) as u8 }, unsafe { *str.add(2) as u8 });
            str = unsafe { str.add(2) };
        }
        if c < 0 || !modifiers.is_empty() {
            special = true;
        }
    }

    if c >= 0 && utf8len_tab[c as usize] > 1 {
        unsafe { *cursor = str };
        // Try to un-escape a multi-byte character after the modifiers.
        let unescaped = unsafe { mb_unescape(cursor, &mut ch) };
        if unescaped.is_null() {
            // Illegal byte.
            unsafe { *cursor = str.add(1) };
        } else {
            // `special` is set, so get_special_key_name() renders it.
            c = unsafe { utf_ptr2char(unescaped) };
        }
    } else {
        // Single-byte character, NUL or illegal byte.
        unsafe { *cursor = str.add(usize::from(*str != 0)) };
    }

    if special
        || c < b' ' as c_int
        || (replace_spaces && c == b' ' as c_int)
        || (replace_lt && c == b'<' as c_int)
    {
        *out = get_special_key_name(c, modifiers);
        return out.as_ptr();
    }
    out[0] = c as c_char;
    out[1] = 0;
    out.as_ptr()
}

/// The key code a two-byte termcap name stands for (C's `TO_SPECIAL`).
fn to_special(second: u8, third: u8) -> c_int {
    match second as c_int {
        KS_SPECIAL => K_SPECIAL,
        KS_ZERO => Key::Zero.code(),
        _ => termcap_key([second, third]),
    }
}

/// Show a string, cutting the middle out with `...` if it would not fit on the
/// rest of the line.
///
/// Does not handle multi-byte characters.
///
/// # Safety
///
/// `longstr` must point at a NUL-terminated string.
pub unsafe fn msg_outtrans_long(longstr: *const c_char, hl_id: c_int) {
    let len = unsafe { cstr::bytes_at(longstr).len() as c_int };
    let mut tail = len;
    let room = Columns.get() - msg_col.get();
    if !ui_has(kUIMessages) && len > room && room >= 20 {
        tail = (room - 3) / 2;
        unsafe { msg_outtrans_len(longstr, tail, hl_id, false) };
        unsafe { msg_puts_hl(c"...".as_ptr(), SPECIAL_HL, false) };
    }
    unsafe { msg_outtrans_len(longstr.offset((len - tail) as isize), tail, hl_id, false) };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs [`walk_display`] emits, as text, and the cells it counted.
    ///
    /// The lib harness has no editor behind it, so `chartab` is all zeros.
    /// That is not a hole: `transchar`'s own fallback trusts printable ASCII
    /// before the table exists and treats everything else as unprintable,
    /// which for a *byte* is the answer a live table gives too — a byte is
    /// never taken for a printable Latin-1 character. The cases below stay
    /// away from the one thing the table really decides, the width of an
    /// ambiguous-width character, by using one that is unambiguously wide.
    fn shown(bytes: &[u8]) -> (Vec<String>, c_int) {
        let mut runs = Vec::new();
        let cells = walk_display(bytes, &mut |shown| {
            let text = match shown {
                Shown::Plain(run) => run.to_vec(),
                Shown::Instead(rendered) => cstr::in_chars(&rendered).to_bytes().to_vec(),
            };
            runs.push(String::from_utf8_lossy(&text).into_owned());
        });
        (runs, cells)
    }

    #[track_caller]
    fn assert_shown(bytes: &[u8], runs: &[&str], cells: c_int) {
        let (got_runs, got_cells) = shown(bytes);
        assert_eq!(got_runs, runs, "runs of {bytes:?}");
        assert_eq!(got_cells, cells, "cells of {bytes:?}");
    }

    #[test]
    fn a_printable_run_comes_out_whole() {
        assert_shown(b"hello", &["hello"], 5);
    }

    #[test]
    fn an_empty_message_still_emits_one_empty_run() {
        // What clears the message line, and what every caller that shows an
        // empty string relies on.
        assert_shown(b"", &[""], 0);
    }

    #[test]
    fn a_control_character_interrupts_the_run() {
        assert_shown(b"a\tb", &["a", "^I", "b"], 4);
        assert_shown(b"\x1bc", &["^[", "c"], 3);
        assert_shown(b"c\x1b", &["c", "^["], 3);
    }

    #[test]
    fn a_newline_and_a_nul_share_the_caret_form() {
        // The memline stores a NUL as a newline, so the rendering folds the
        // two together and neither reaches the screen as itself.
        assert_shown(b"\n", &["^@"], 2);
        assert_shown(b"\0", &["^@"], 2);
    }

    #[test]
    fn an_interior_nul_is_an_ordinary_byte() {
        // The length is the slice's, so a NUL in the middle is shown rather
        // than ending the message.
        assert_shown(b"a\0b", &["a", "^@", "b"], 4);
    }

    #[test]
    fn del_and_the_high_bytes_take_their_own_forms() {
        assert_shown(b"\x7f", &["^?"], 2);
        assert_shown(b"\xff", &["<ff>"], 4);
    }

    #[test]
    fn an_incomplete_sequence_is_shown_byte_by_byte() {
        // Two thirds of a three-byte character: each byte is illegal on its
        // own and gets its own `<xx>`.
        assert_shown(b"\xe4\xb8", &["<e4>", "<b8>"], 8);
    }

    #[test]
    fn a_wide_character_stays_in_the_run_and_counts_two_cells() {
        assert_shown("一".as_bytes(), &["一"], 2);
        assert_shown("a一b".as_bytes(), &["a一b"], 4);
    }

    #[test]
    fn an_unprintable_character_is_replaced_by_its_hex_form() {
        // U+200B ZERO WIDTH SPACE: printable to Unicode, not to Vim.
        assert_shown("\u{200b}".as_bytes(), &["<200b>"], 6);
        assert_shown("a\u{200b}b".as_bytes(), &["a", "<200b>", "b"], 8);
    }

    #[test]
    fn the_slice_is_the_string() {
        // No `-1` and no terminator: the walk stops where the slice does,
        // even though the bytes after it are readable and printable.
        assert_shown(&b"abcdef"[..3], &["abc"], 3);
    }
}
