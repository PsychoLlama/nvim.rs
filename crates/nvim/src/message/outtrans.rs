//! Turning bytes into something displayable.
//!
//! The `msg_display*` half renders unprintable bytes as `<xx>` and multibyte
//! sequences as themselves; the `str2special*` half renders key codes as
//! `<C-X>` notation, which is what mapping listings and `keytrans()` show.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::{CharDisplay, skip};
use crate::cstr;
use crate::keycodes::ModMask;
use crate::keycodes::{Key, termcap_key, termcap_name};
use crate::mbyte::{cells_at, char_at, cluster_len, encode_char, unescape_at};
use crate::os::env::home_replace_in;
use crate::types::MB_MAXCHAR;
use core::ffi::{CStr, c_int};

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
/// as, because that is what [`msg_display_bytes`] and `str2special` downstream
/// know how to read.
pub fn msg_putchar_hl(c: c_int, hl_id: c_int) {
    let mut buf = [0u8; MB_MAXCHAR + 1];
    if c < 0 {
        // `K_SECOND`/`K_THIRD`, less their `c == K_SPECIAL`/`c == NUL`
        // arms: both of those codes are positive, so neither is reachable
        // here.
        let name = termcap_name(c);
        buf[0] = K_SPECIAL as u8;
        buf[1] = name[0];
        buf[2] = name[1];
    } else {
        encode_char(c, &mut buf);
    }
    msg_str_hl(cstr::in_bytes(&buf), hl_id, false)
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
    msg_str(cstr::in_bytes(&buf[at..]))
}

/// Show a file name with `$HOME` folded back to `~`.
pub fn msg_home_replace(fname: &CStr) {
    msg_home_replace_hl(fname, 0)
}

/// [`msg_home_replace`] with a highlight id.
pub(crate) fn msg_home_replace_hl(fname: &CStr, hl_id: c_int) {
    // Room for the name, a `~/` and the terminator, as upstream sized it.
    let name = home_replace_in(None, fname, fname.count_bytes() + 3, true);
    msg_display(name.as_cstr(), hl_id, false);
}

/// Show a NUL-terminated string, translating what cannot be displayed.
///
/// Answers how many screen cells it took.
pub fn msg_display(text: &CStr, hl_id: c_int, hist: bool) -> c_int {
    msg_display_bytes(text.to_bytes(), hl_id, hist)
}

/// Show the character at the start of `bytes`, answering how many bytes of
/// it were shown.
pub fn msg_display_char(bytes: &[u8], hl_id: c_int, hist: bool) -> usize {
    let len = cluster_len(bytes);
    if len > 1 {
        msg_display_bytes(&bytes[..len], hl_id, hist);
        return len;
    }
    // Past the end of the slice this reads a NUL, which renders as `^@` --
    // what the pointer form showed at the terminator, and what every caller
    // that walks up to a bound already expects.
    let rendered = transchar_byte_buf(None, c_int::from(cstr::byte_at(bytes, 0)));
    msg_str_hl(cstr::in_chars(&rendered), hl_id, hist);
    1
}

/// Show `bytes`, NULs included, translating what cannot be displayed.
///
/// Printable runs are handed to [`msg_bytes`] whole; only the characters
/// that need a `^X` or `<xx>` rendering are emitted one at a time, in the
/// `SPECIAL_HL` highlight.
///
/// Answers how many screen cells it took.
pub fn msg_display_bytes(bytes: &[u8], hl_id: c_int, hist: bool) -> c_int {
    display(bytes, hl_id, hist, true)
}

/// [`msg_display_bytes`] for one piece of a message already being built:
/// an empty `bytes` adds nothing rather than being an empty message.
/// [`msg_part`] says why the two differ. The cells are not answered because
/// the one caller that splits a message is not counting them.
pub(crate) fn msg_display_part(bytes: &[u8], hl_id: c_int, hist: bool) {
    display(bytes, hl_id, hist, false);
}

fn display(bytes: &[u8], hl_id: c_int, hist: bool, whole_message: bool) -> c_int {
    // Only quit when got_int was set in here.
    let save_got_int = got_int.get();
    got_int.set(false);

    if hist {
        msg_hist_add(bytes, hl_id);
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
            if whole_message {
                msg_bytes(run, hl_id, hist);
            } else {
                msg_part(run, hl_id, hist);
            }
        }
        Shown::Instead(text) => msg_str_hl(cstr::in_chars(&text), special_hl(hl_id), false),
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
                cells += cells_at(rest);
            } else {
                if at > run {
                    emit(Shown::Plain(&bytes[run..at]));
                }
                emit(Shown::Instead(transchar_buf(None, c)));
                cells += char2cells(c);
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
pub fn msg_make(arg: &CStr) {
    // The command name backwards, and the answer with every byte shifted up
    // by three -- both so that neither reads as itself in the binary.
    const REVERSED: &[u8] = b"eeffoc";
    const SHIFTED: &[u8] = b"Plon#dqg#vxjduB";

    let bytes = arg.to_bytes();
    let mut at = skip::white(bytes);
    let mut left = REVERSED.len();
    while at < bytes.len() && left > 0 {
        let byte = bytes[at];
        at += 1;
        left -= 1;
        if byte != REVERSED[left] {
            return;
        }
    }
    if left == 0 {
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
pub fn msg_display_keys(text: &CStr, from: bool, maxlen: c_int) -> c_int {
    let bytes = text.to_bytes();
    // One key's rendering at a time, the buffer reused across them.
    let mut piece = Vec::new();
    let mut at = 0;
    let mut cells = 0;
    while at < bytes.len() {
        piece.clear();
        if bytes[at] == b' ' && (at == 0 || at + 1 == bytes.len()) {
            at += 1;
            piece.extend_from_slice(b"<Space>");
        } else {
            special_key_into(bytes, &mut at, from, false, &mut piece);
        }
        // Single-byte character, or an illegal byte.
        let display: CharDisplay;
        let shown: &[u8] = if piece.len() == 1 {
            display = transchar_byte_buf(None, c_int::from(piece[0]));
            cstr::in_chars(&display).to_bytes()
        } else {
            &piece
        };
        let len = strsize(shown);
        if maxlen > 0 && cells + len >= maxlen {
            break;
        }
        // Highlight the ones that came out as a `<>` name.
        let hl_id = if len > 1 && c_cluster_len(shown) <= 1 {
            SPECIAL_HL
        } else {
            0
        };
        msg_bytes(shown, hl_id, false);
        cells += len;
    }
    cells
}

/// The readable spelling of every key in `text`: [`special_key_into`] over
/// the whole of it. A NUL ends it, as it ended the C string.
pub fn special_keys_text(text: &[u8], replace_spaces: bool, replace_lt: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() && text[at] != 0 {
        special_key_into(text, &mut at, replace_spaces, replace_lt, &mut out);
    }
    out
}

/// Render the key at `text[*at..]` as printable text onto `out`, advancing
/// `*at` past it. Upstream's `str2special`.
///
/// Special keys and C0 control characters come out in `<>` form;
/// `replace_spaces` and `replace_lt` extend that to `<Space>` and `<lt>`,
/// which is what a mapping's left-hand side and `keytrans()` want and its
/// right-hand side does not. An illegal byte comes out as itself. The end of
/// `text` reads as the NUL the C string ended at.
pub(crate) fn special_key_into(
    text: &[u8],
    at: &mut usize,
    replace_spaces: bool,
    replace_lt: bool,
    out: &mut Vec<u8>,
) {
    let byte = |i: usize| cstr::byte_at(text, i);
    // A multi-byte character escaped into the stream comes back whole.
    if byte(*at) >= 0x80
        && let Some((ch, used)) = unescape_at(&text[*at..])
    {
        out.extend_from_slice(cstr::in_bytes(&ch).to_bytes());
        *at += used;
        return;
    }

    let mut str = *at;
    let mut c = c_int::from(byte(str));
    let mut modifiers = ModMask::NONE;
    let mut special = false;
    if c == K_SPECIAL && byte(str + 1) != 0 && byte(str + 2) != 0 {
        if c_int::from(byte(str + 1)) == KS_MODIFIER {
            modifiers = ModMask::from_bits(c_int::from(byte(str + 2)));
            str += 3;
            c = c_int::from(byte(str));
        }
        if c == K_SPECIAL && byte(str + 1) != 0 && byte(str + 2) != 0 {
            c = to_special(byte(str + 1), byte(str + 2));
            str += 2;
        }
        if c < 0 || !modifiers.is_empty() {
            special = true;
        }
    }

    if c >= 0 && utf8len_tab[c as usize] > 1 {
        // Try to un-escape a multi-byte character after the modifiers.
        match unescape_at(&text[str.min(text.len())..]) {
            // Illegal byte.
            None => *at = str + 1,
            Some((ch, used)) => {
                // `special` is set, so get_special_key_name() renders it.
                c = char_at(cstr::in_bytes(&ch).to_bytes());
                *at = str + used;
            }
        }
    } else {
        // Single-byte character, NUL or illegal byte.
        *at = str + usize::from(byte(str) != 0);
    }

    if special
        || c < b' ' as c_int
        || (replace_spaces && c == b' ' as c_int)
        || (replace_lt && c == b'<' as c_int)
    {
        let name = get_special_key_name(c, modifiers);
        out.extend_from_slice(cstr::in_chars(&name).to_bytes());
        return;
    }
    // Below 256 here: a multibyte character either came back whole above or
    // made the key special.
    out.push(c as u8);
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
pub fn msg_display_elided(text: &CStr, hl_id: c_int) {
    let bytes = text.to_bytes();
    let mut tail = bytes.len();
    let room = usize::try_from(Columns.get() - msg_col.get()).unwrap_or(0);
    if !ui_has(kUIMessages) && bytes.len() > room && room >= 20 {
        tail = (room - 3) / 2;
        msg_display_bytes(&bytes[..tail], hl_id, false);
        msg_str_hl(c"...", SPECIAL_HL, false);
    }
    msg_display_bytes(&bytes[bytes.len() - tail..], hl_id, false);
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
