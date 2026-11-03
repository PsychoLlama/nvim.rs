//! [`insertchar`] -- putting one typed character into the buffer.
//!
//! The common path, and the one that has to be fast: everything that is not
//! a special key ends here.  Three things make it more than an insert.
//!
//! It may *wrap* first ([`wrap_before_insert`]): 'textwidth' and
//! 'formatoptions' decide whether this character pushes the line over, and
//! either 'formatexpr' or `internal_format` does the breaking.  It may have
//! to *end a comment* ([`end_pending_comment`]): after an auto-indent that
//! opened one, typing the last character of the 'comments' end leader
//! replaces the middle leader with the end leader.  And it *batches* -- while
//! more plain characters are already available and none of the conditions
//! above can trigger, it collects up to `INPUT_BUFLEN` of them into one
//! `ins_str` rather than one call apiece, which is what makes pasted or
//! mapped text fast.
//!
//! [`do_insert_char_pre`] is the `InsertCharPre` autocommand, which may
//! replace the character with a whole *string*; it is one of the several
//! reasons the batch path has to be given up.
//!
//! [`echeck_abbr`] is here because an abbreviation is triggered by the
//! *non*-word character that ends the word, which is the character being
//! inserted.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::{self, byte_at};
use crate::mbyte::encode_char;
use crate::memline::Lines;
use crate::memory::XString;
use crate::narrow::len_as_int;
use crate::option::split_option_part;
use crate::types::Key;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::guard::Lock;
use crate::optionstr::{OptString, OptStringRef};
use crate::types::{FoFlag, MB_MAXCHAR};

/// Upstream's `ISSPECIAL`: a key that needs processing other than the
/// simple insert this file can do -- a special key, a control character,
/// anything from DEL up, and `0` and `^`.
///
/// `<Esc>` ends the insert and CR/NL open a line; `0` and `^` are here
/// because either can be followed by CTRL-D.  What is left is exactly the
/// printable ASCII characters but those two, one byte each.
const fn needs_processing(c: Key) -> bool {
    !matches!(c.code(), 0x20..0x7f) || c.is('0') || c.is('^')
}

/// Insert one character, formatting and batching as described in the module
/// doc.
///
/// `c` is the character, or NUL to ask for formatting only.  `flags` is
/// `INSCHAR_FORMAT` (force formatting), `INSCHAR_CTRLV` (typed just after
/// CTRL-V) and `INSCHAR_NO_FEX` (do not use 'formatexpr'); the whole value is
/// passed straight through to `internal_format`, which also reads
/// `INSCHAR_DO_COM` and `INSCHAR_COM_LIST`.  `second_indent` is the indent
/// for a second line, if not negative.
pub(crate) fn insertchar(c: Key, flags: c_int, second_indent: c_int) {
    let textwidth = comp_textwidth(flags & INSCHAR_FORMAT.cast_signed() != 0);
    wrap_before_insert(c, flags, second_indent, textwidth);

    if c.is_nul() {
        return; // only formatting was wanted
    }

    end_pending_comment(c);
    end_comment_pending.set(None);

    did_ai.set(false);
    did_si.set(false);
    can_si.set(false);
    can_si_back.set(false);

    // With input already pending, grab up to INPUT_BUFLEN characters at
    // once; this speeds up ordinary text input considerably.  Not with
    // 'cindent' or 'indentexpr', which may want to re-indent on a `:` or
    // any other character, and not with an `InsertCharPre` autocommand,
    // which has to see every character one at a time.  The event test
    // comes before `vpeekc` because the autocommand can change the input
    // buffer.
    if let Some(first) = c.as_byte().filter(|_| !needs_processing(c))
        && !has_event(AutoEvent::InsertCharPre)
        && !test_disable_char_avail.get()
        && !vpeekc().is_nul()
        && State.get() & REPLACE_FLAG == 0
        && !cindent_on()
        && !p_ri()
    {
        let mut buf = [0u8; INPUT_BUFLEN as usize];
        let mut virtcol: ColNr = ColNr::ZERO;

        buf[0] = first;
        let mut i = 1;
        if textwidth > 0 {
            virtcol = get_nolist_virtcol();
        }
        // Stop when there is nothing more to take, on a special
        // character (a command key), when the buffer is full, at the
        // 'textwidth' boundary, or where an abbreviation may need
        // checking -- a non-word character after a word character.
        // A key that does not need processing is one printable ASCII
        // byte (NUL, nothing typed, needs it), so the byte is the key.
        loop {
            let next = vpeekc();
            let take = !needs_processing(next)
                && i < buf.len()
                && (textwidth == 0 || {
                    virtcol += byte2cells(c_int::from(buf[i - 1]));
                    virtcol < ColNr::new(textwidth)
                })
                && !(!no_abbr.get()
                    && !vim_iswordc(next.code())
                    && vim_iswordc(c_int::from(buf[i - 1])));
            if !take {
                break;
            }
            buf[i] = vgetc().as_byte().unwrap_or(first);
            i += 1;
        }

        reset_digraph(); // clear digraphs
        do_digraph(Key::byte(buf[i - 1])); // may start one
        let typed = &buf[..i];
        ins_str(typed);

        // After CTRL-V the first character is recorded literally, and
        // the rest as themselves.
        let redo_from = if flags & INSCHAR_CTRLV.cast_signed() != 0 {
            redo_literal(Key::byte(typed[0]));
            1
        } else {
            0
        };
        if redo_from < typed.len() {
            append_to_redobuff_literally(&typed[redo_from..]);
        }
    } else {
        let cc = c.utf8_len();
        if cc > 1 {
            let mut buf = [0u8; MB_MAXCHAR];
            let len = encode_char(c.code(), &mut buf);
            ins_char_bytes(&buf[..len]);
            append_to_redobuff_char(c);
        } else {
            ins_char(c);
            if flags & INSCHAR_CTRLV.cast_signed() != 0 {
                redo_literal(c);
            } else {
                append_to_redobuff_char(c);
            }
        }
    }
}

/// Break the line in two or more pieces before `c` is inserted, if
/// 'textwidth' and 'formatoptions' say so.
///
/// Always when the caller asked for formatting only (`INSCHAR_FORMAT`), and
/// always when 'formatoptions' has `a` and the line ends in white space.
/// Otherwise: not when inserting a blank; not when an existing character is
/// being replaced, unless in `MODE_VREPLACE`; and, on the line the insert
/// started on, only when 'formatoptions' lacks `l` or the line was not
/// already too long, and lacks `b` or a blank was inserted at or before
/// 'textwidth'.
fn wrap_before_insert(c: Key, flags: c_int, second_indent: c_int, textwidth: c_int) {
    let force_format = flags & INSCHAR_FORMAT.cast_signed();
    let fo_ins_blank = has_format_option(FoFlag::INS_BLANK);
    let fo_ins_long = has_format_option(FoFlag::INS_LONG);

    if textwidth <= 0 {
        return;
    }
    let wanted = force_format != 0
        || (!ascii_iswhite(c.code())
            && !(State.get() & REPLACE_FLAG != 0
                && State.get() & VREPLACE_FLAG == 0
                && cursor_byte() != 0)
            && (Win::current().w_cursor.lnum != Insstart.get().lnum
                || ((!fo_ins_long || Insstart_textlen.get() <= ColNr::new(textwidth))
                    && (!fo_ins_blank || Insstart_blank_vcol.get() <= ColNr::new(textwidth)))));
    if !wanted {
        return;
    }

    // Format with 'formatexpr' when it is set; use the internal
    // formatting when it is not, or when it answered non-zero.
    let mut do_internal = true;
    let virtcol =
        get_nolist_virtcol() + char2cells(if c.is_nul() { gchar_cursor() } else { c.code() });

    if Buf::current().b_p_fex.first_byte() != 0
        && flags & INSCHAR_NO_FEX.cast_signed() == 0
        && (force_format != 0 || virtcol > ColNr::new(textwidth))
    {
        do_internal = fex_format(Win::current().w_cursor.lnum, 1, c.code()) != 0;
        // Saving for undo may be needed again, e.g. when the expression
        // called setline().
        ins_need_undo.set(true);
    }
    if do_internal {
        internal_format(textwidth, second_indent, flags, c.is_nul(), c.code());
    }
}

/// After an auto-indent that opened a comment, does `c` finish it?
///
/// `end_comment_pending` holds the last character of the 'comments' end
/// leader.  When it arrives, the *middle* leader that was auto-indented in
/// has to come off and the end leader go in -- all but its last character,
/// which the caller inserts as an ordinary one.
fn end_pending_comment(c: Key) {
    if !did_ai.get() || end_comment_pending.get().map(Key::byte) != Some(c) {
        return;
    }

    // Find the comment leader this line starts with.
    let lnum = Win::current().w_cursor.lnum;
    let (lead, flags) = get_leader_len(Lines::current().line(lnum), false, true);
    // The middle item's flags, searched as upstream searches them: to the
    // end of 'comments'.
    let Some(flags) =
        flags.filter(|flags| lead > 0 && flags.with_rest(|rest| rest.contains(&COM_MIDDLE)))
    else {
        return; // just checking
    };

    let (middle_len, lead_end) = {
        let buffer = Buf::current();
        let com = buffer.b_p_com.bytes();
        let before = |p: usize| p.checked_sub(1).map_or(0, |prev| com[prev]);
        let mut p = flags.offset();
        // Skip the middle-comment string.
        while byte_at(com, p) != 0 && before(p) != b':' {
            p += 1; // find the end of the middle flags
        }
        let (middle, rest) = split_option_part(&com[p..], COM_MAX_LEN, b",");
        p = com.len() - rest.len();
        // Trailing white space does not count towards `middle_len`.
        let mut middle_len = middle.len();
        while middle_len > 0 && ascii_iswhite(c_int::from(middle[middle_len - 1])) {
            middle_len -= 1;
        }

        // Find the end-comment string.
        while byte_at(com, p) != 0 && before(p) != b':' {
            p += 1; // find the end of the end flags
        }
        let (lead_end, _) = split_option_part(&com[p..], COM_MAX_LEN, b",");
        (len_as_int(middle_len), lead_end)
    };

    // Skip the white space before the cursor, then back over the middle
    // leader.
    let mut i = Win::current().w_cursor.col;
    {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        while i > ColNr::ZERO && ascii_iswhite(c_int::from(line[i.as_usize() - 1])) {
            i -= 1;
        }
    }
    i -= middle_len;

    // Check some expected things before going on.
    if i >= ColNr::ZERO
        && lead_end
            .last()
            .is_some_and(|&last| Some(last) == end_comment_pending.get())
    {
        // Backspace over everything being replaced.
        backspace_until_column(i.get());
        // Insert the end-comment string except for its last character,
        // which the caller inserts as an ordinary one.
        ins_bytes(&lead_end[..lead_end.len() - 1]);
    }
}

/// Check the word in front of the cursor for an abbreviation.
///
/// Called when the non-identifier character `c` has been entered.  When an
/// abbreviation is recognised it is removed from the text and the replacement
/// is put into the typeahead buffer, followed by `c` -- after a CTRL-V unless
/// `literal` (a key that is no control character) says it needs none.
pub(crate) fn echeck_abbr(c: Key, literal: bool) -> bool {
    // Not in 'paste' mode, not when disabled, and not just after moving
    // around with the cursor keys.
    if p_paste() || no_abbr.get() || arrow_used.get() {
        return false;
    }

    let start_col = if Win::current().w_cursor.lnum == Insstart.get().lnum {
        Insstart.get().col
    } else {
        ColNr::ZERO
    };
    let cursor = Win::current().w_cursor;
    let col = usize::try_from(cursor.col).unwrap_or(0);
    // A copy of the line before the cursor: the expansion can run user code,
    // which may change the line.
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let line = line[..col.min(line.len())].to_vec();
    check_abbr(c, literal, line, start_col.get())
}

/// Run the `InsertCharPre` autocommand for `c`.
///
/// Answers the replacement string when the autocommand changed `v:char`, and
/// `None` to go on inserting `c` -- which is also the answer when there is no
/// such autocommand at all.
pub(crate) fn do_insert_char_pre(c: Key) -> Option<XString> {
    if c == Key::CTRL_RSB || !has_event(AutoEvent::InsertCharPre) {
        return None;
    }

    let save_state = State.get();
    let mut buf = [0u8; MB_MAXBYTES];
    let buflen = encode_char(c.code(), &mut buf);

    let locked = Lock::text();
    // Up to a NUL, which is where every reader stops: `c` may be one.
    let typed = cstr::until_nul(&buf[..buflen]);
    set_vim_var_string(Vv::Char, Some(typed));

    let mut res = None;
    if ins_apply_autocmds(AutoEvent::InsertCharPre) {
        let now = vim_var_bytes(Vv::Char);
        if now != typed {
            res = Some(XString::from_bytes(cstr::until_nul(&now)));
        }
    }

    set_vim_var_string(Vv::Char, None);
    drop(locked);
    State.set(save_state);
    res
}

/// The byte under the cursor, NUL past the end of its line.
fn cursor_byte() -> u8 {
    let cursor = Win::current().w_cursor;
    byte_at(Lines::current().line(cursor.lnum), cursor.col.as_usize())
}

#[cfg(test)]
mod tests {
    //! Which keys the batch path in [`insertchar`] may take: one printable
    //! ASCII byte that cannot start `0 CTRL-D`/`^ CTRL-D`.

    use super::needs_processing;
    use crate::keycodes::SpecialKey;
    use crate::types::Key;

    #[test]
    fn only_printable_ascii_but_zero_and_caret_is_batched() {
        for b in 0u8..=0xff {
            let batched = (0x20..0x7f).contains(&b) && b != b'0' && b != b'^';
            assert_eq!(needs_processing(Key::byte(b)), !batched, "{b:#04x}");
        }
        for key in [
            Key::from_char('\u{e9}'),
            Key::from_char('\u{65e5}'),
            Key::from(SpecialKey::Left),
            Key::from(SpecialKey::Kenter),
        ] {
            assert!(needs_processing(key), "{key:?}");
        }
    }
}
