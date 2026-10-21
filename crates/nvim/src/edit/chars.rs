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

use crate::cstr::{self, byte_at};
use crate::mbyte::encode_char;
use crate::memline::Lines;
use crate::memory::XString;
use crate::option::split_option_part;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::guard::Lock;
use crate::optionstr::{OptString, OptStringRef};
use crate::types::{FoFlag, MB_MAXCHAR, NUL};

/// Upstream's `ISSPECIAL`: a character that needs processing other than the
/// simple insert this file can do.
///
/// `<Esc>` ends the insert and CR/NL open a line; `0` and `^` are here
/// because either can be followed by CTRL-D.
const fn is_special(c: c_int) -> bool {
    c < b' ' as c_int || c >= DEL || c == b'0' as c_int || c == b'^' as c_int
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
pub(crate) fn insertchar(c: c_int, flags: c_int, second_indent: c_int) {
    let textwidth = comp_textwidth(flags & INSCHAR_FORMAT as c_int != 0);
    wrap_before_insert(c, flags, second_indent, textwidth);

    if c == NUL {
        return; // only formatting was wanted
    }

    end_pending_comment(c);
    end_comment_pending.set(NUL);

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
    if !is_special(c)
        && utf_char2len(c) == 1
        && !has_event(AutoEvent::InsertCharPre)
        && !test_disable_char_avail.get()
        && vpeekc() != NUL
        && State.get() & REPLACE_FLAG == 0
        && !cindent_on()
        && !p_ri()
    {
        let mut buf = [0u8; INPUT_BUFLEN as usize];
        let mut virtcol: ColNr = 0;

        buf[0] = c as u8;
        let mut i = 1;
        if textwidth > 0 {
            virtcol = get_nolist_virtcol();
        }
        // Stop when there is nothing more to take, on a special
        // character (a command key), when the buffer is full, at the
        // 'textwidth' boundary, or where an abbreviation may need
        // checking -- a non-word character after a word character.
        loop {
            let next = vpeekc();
            let take = next != NUL
                && !is_special(next)
                && utf8len_tab[next as usize] as c_int == 1
                && i < INPUT_BUFLEN
                && (textwidth == 0 || {
                    virtcol += byte2cells(c_int::from(buf[i as usize - 1]));
                    virtcol < textwidth
                })
                && !(!no_abbr.get()
                    && !vim_iswordc(next)
                    && vim_iswordc(c_int::from(buf[i as usize - 1])));
            if !take {
                break;
            }
            buf[i as usize] = vgetc() as u8;
            i += 1;
        }

        do_digraph(-1); // clear digraphs
        do_digraph(c_int::from(buf[i as usize - 1])); // may start one
        let typed = &buf[..i as usize];
        ins_str(typed);

        // After CTRL-V the first character is recorded literally, and
        // the rest as themselves.
        let redo_from = if flags & INSCHAR_CTRLV as c_int != 0 {
            redo_literal(c_int::from(typed[0]));
            1
        } else {
            0
        };
        if redo_from < typed.len() {
            append_to_redobuff_literally(&typed[redo_from..]);
        }
    } else {
        let cc = utf_char2len(c);
        if cc > 1 {
            let mut buf = [0u8; MB_MAXCHAR];
            let len = encode_char(c, &mut buf);
            ins_char_bytes(&buf[..len]);
            append_to_redobuff_char(c);
        } else {
            ins_char(c);
            if flags & INSCHAR_CTRLV as c_int != 0 {
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
fn wrap_before_insert(c: c_int, flags: c_int, second_indent: c_int, textwidth: c_int) {
    let force_format = flags & INSCHAR_FORMAT as c_int;
    let fo_ins_blank = has_format_option(FoFlag::INS_BLANK);
    let fo_ins_long = has_format_option(FoFlag::INS_LONG);

    if textwidth <= 0 {
        return;
    }
    let wanted = force_format != 0
        || (!ascii_iswhite(c)
            && !(State.get() & REPLACE_FLAG != 0
                && State.get() & VREPLACE_FLAG == 0
                && cursor_byte() != NUL as u8)
            && (Win::current().w_cursor.lnum != Insstart.get().lnum
                || ((!fo_ins_long || Insstart_textlen.get() <= textwidth)
                    && (!fo_ins_blank || Insstart_blank_vcol.get() <= textwidth))));
    if !wanted {
        return;
    }

    // Format with 'formatexpr' when it is set; use the internal
    // formatting when it is not, or when it answered non-zero.
    let mut do_internal = true;
    let virtcol = get_nolist_virtcol() + char2cells(if c != NUL { c } else { gchar_cursor() });

    if Buf::current().b_p_fex.first_byte() as c_int != NUL
        && flags & INSCHAR_NO_FEX as c_int == 0
        && (force_format != 0 || virtcol > textwidth)
    {
        do_internal = fex_format(Win::current().w_cursor.lnum, 1, c) != 0;
        // Saving for undo may be needed again, e.g. when the expression
        // called setline().
        ins_need_undo.set(true);
    }
    if do_internal {
        internal_format(textwidth, second_indent, flags, c == NUL, c);
    }
}

/// After an auto-indent that opened a comment, does `c` finish it?
///
/// `end_comment_pending` holds the last character of the 'comments' end
/// leader.  When it arrives, the *middle* leader that was auto-indented in
/// has to come off and the end leader go in -- all but its last character,
/// which the caller inserts as an ordinary one.
fn end_pending_comment(c: c_int) {
    if !did_ai.get() || c != end_comment_pending.get() {
        return;
    }

    // Find the comment leader this line starts with.
    let lnum = Win::current().w_cursor.lnum;
    let (lead, flags) = get_leader_len(Lines::current().line(lnum), false, true);
    // The middle item's flags, searched as upstream searches them: to the
    // end of 'comments'.
    let Some(flags) = flags
        .filter(|flags| lead > 0 && flags.with_rest(|rest| rest.contains(&(COM_MIDDLE as u8))))
    else {
        return; // just checking
    };

    let (middle_len, lead_end) = {
        let buffer = Buf::current();
        let com = buffer.b_p_com.bytes();
        let before = |p: usize| p.checked_sub(1).map_or(NUL as u8, |prev| com[prev]);
        let mut p = flags.offset();
        // Skip the middle-comment string.
        while byte_at(com, p) != 0 && before(p) != b':' {
            p += 1; // find the end of the middle flags
        }
        let (middle, rest) = split_option_part(&com[p..], COM_MAX_LEN as usize, b",");
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
        let (lead_end, _) = split_option_part(&com[p..], COM_MAX_LEN as usize, b",");
        (middle_len as c_int, lead_end)
    };

    // Skip the white space before the cursor, then back over the middle
    // leader.
    let mut i = Win::current().w_cursor.col;
    {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        while i > 0 && ascii_iswhite(c_int::from(line[i as usize - 1])) {
            i -= 1;
        }
    }
    i -= middle_len;

    // Check some expected things before going on.
    if i >= 0
        && lead_end
            .last()
            .is_some_and(|&last| c_int::from(last) == end_comment_pending.get())
    {
        // Backspace over everything being replaced.
        backspace_until_column(i);
        // Insert the end-comment string except for its last character,
        // which the caller inserts as an ordinary one.
        ins_bytes(&lead_end[..lead_end.len() - 1]);
    }
}

/// Check the word in front of the cursor for an abbreviation.
///
/// Called when the non-identifier character `c` has been entered.  When an
/// abbreviation is recognised it is removed from the text and the replacement
/// is put into the typeahead buffer, followed by `c`.
pub(crate) fn echeck_abbr(c: c_int) -> bool {
    // Not in 'paste' mode, not when disabled, and not just after moving
    // around with the cursor keys.
    if p_paste() || no_abbr.get() || arrow_used.get() {
        return false;
    }

    let start_col = if Win::current().w_cursor.lnum == Insstart.get().lnum {
        Insstart.get().col
    } else {
        0
    };
    let cursor = Win::current().w_cursor;
    let col = usize::try_from(cursor.col).unwrap_or(0);
    // A copy of the line before the cursor: the expansion can run user code,
    // which may change the line.
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let line = line[..col.min(line.len())].to_vec();
    check_abbr(c, line, start_col)
}

/// Run the `InsertCharPre` autocommand for `c`.
///
/// Answers the replacement string when the autocommand changed `v:char`, and
/// `None` to go on inserting `c` -- which is also the answer when there is no
/// such autocommand at all.
pub(crate) fn do_insert_char_pre(c: c_int) -> Option<XString> {
    if c == Ctrl_RSB || !has_event(AutoEvent::InsertCharPre) {
        return None;
    }

    let save_state = State.get();
    let mut buf = [0u8; MB_MAXBYTES];
    let buflen = encode_char(c, &mut buf);

    let locked = Lock::text();
    // Up to a NUL, which is where every reader stops: `c` may be one.
    let typed = cstr::until_nul(&buf[..buflen]);
    set_vim_var_string(Vv::Char, Some(typed));

    let mut res = None;
    if ins_apply_autocmds(AutoEvent::InsertCharPre) != 0 {
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
    byte_at(Lines::current().line(cursor.lnum), cursor.col as usize)
}
