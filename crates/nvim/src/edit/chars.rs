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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::guard::Lock;
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
///
/// # Safety
/// Must run with a live `curwin`/`curbuf`.
pub(crate) unsafe fn insertchar(c: c_int, flags: c_int, second_indent: c_int) {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let textwidth = unsafe { comp_textwidth(flags & INSCHAR_FORMAT as c_int != 0) };
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
        && !unsafe { has_event(EVENT_INSERTCHARPRE) }
        && !test_disable_char_avail.get()
        && unsafe { vpeekc() } != NUL
        && State.get() & REPLACE_FLAG == 0
        && !unsafe { cindent_on() }
        && p_ri.get() == 0
    {
        let mut buf: [c_char; INPUT_BUFLEN as usize + 1] = [0; INPUT_BUFLEN as usize + 1];
        let mut virtcol: colnr_T = 0;

        buf[0] = c as c_char;
        let mut i = 1;
        if textwidth > 0 {
            virtcol = unsafe { get_nolist_virtcol() };
        }
        // Stop when there is nothing more to take, on a special
        // character (a command key), when the buffer is full, at the
        // 'textwidth' boundary, or where an abbreviation may need
        // checking -- a non-word character after a word character.
        loop {
            let next = unsafe { vpeekc() };
            let take = next != NUL
                && !is_special(next)
                && utf8len_tab[next as usize] as c_int == 1
                && i < INPUT_BUFLEN
                && (textwidth == 0 || {
                    virtcol += unsafe { byte2cells(buf[i as usize - 1] as uint8_t as c_int) };
                    virtcol < textwidth
                })
                && !(!no_abbr.get()
                    && !unsafe { vim_iswordc(next) }
                    && unsafe { vim_iswordc(buf[i as usize - 1] as uint8_t as c_int) });
            if !take {
                break;
            }
            buf[i as usize] = unsafe { vgetc() } as c_char;
            i += 1;
        }

        do_digraph(-1); // clear digraphs
        do_digraph(buf[i as usize - 1] as uint8_t as c_int); // may start one
        buf[i as usize] = NUL as c_char;
        unsafe { ins_str(buf.as_mut_ptr(), i as size_t) };

        // After CTRL-V the first character is recorded literally, and
        // the rest as themselves.
        let redo_from = if flags & INSCHAR_CTRLV as c_int != 0 {
            redo_literal(buf[0] as uint8_t as c_int);
            1
        } else {
            0
        };
        if buf[redo_from] as c_int != NUL {
            unsafe { append_to_redobuff_literally(buf.as_mut_ptr().add(redo_from), -1) };
        }
    } else {
        let cc = utf_char2len(c);
        if cc > 1 {
            let mut buf: [c_char; MB_MAXCHAR + 1] = [0; MB_MAXCHAR + 1];
            unsafe { utf_char2bytes(c, buf.as_mut_ptr()) };
            buf[cc as usize] = NUL as c_char;
            unsafe { ins_char_bytes(buf.as_mut_ptr(), cc as size_t) };
            append_to_redobuff_char(c);
        } else {
            unsafe { ins_char(c) };
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
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    let force_format = flags & INSCHAR_FORMAT as c_int;
    let fo_ins_blank = unsafe { has_format_option(FoFlag::INS_BLANK) };
    let fo_ins_long = unsafe { has_format_option(FoFlag::INS_LONG) };

    if textwidth <= 0 {
        return;
    }
    let wanted = force_format != 0
        || (!ascii_iswhite(c)
            && !(State.get() & REPLACE_FLAG != 0
                && State.get() & VREPLACE_FLAG == 0
                && unsafe { *get_cursor_pos_ptr() } as c_int != NUL)
            && (cur_win().w_cursor.lnum != Insstart.get().lnum
                || ((!fo_ins_long || Insstart_textlen.get() <= textwidth)
                    && (!fo_ins_blank || Insstart_blank_vcol.get() <= textwidth))));
    if !wanted {
        return;
    }

    // Format with 'formatexpr' when it is set; use the internal
    // formatting when it is not, or when it answered non-zero.
    let mut do_internal = true;
    let virtcol = unsafe { get_nolist_virtcol() }
        + unsafe { char2cells(if c != NUL { c } else { gchar_cursor() }) };

    if unsafe { *cur_buf().b_p_fex } as c_int != NUL
        && flags & INSCHAR_NO_FEX as c_int == 0
        && (force_format != 0 || virtcol > textwidth)
    {
        do_internal = unsafe { fex_format(cur_win().w_cursor.lnum, 1, c) } != 0;
        // Saving for undo may be needed again, e.g. when the expression
        // called setline().
        ins_need_undo.set(true);
    }
    if do_internal {
        unsafe { internal_format(textwidth, second_indent, flags, c == NUL, c) };
    }
}

/// After an auto-indent that opened a comment, does `c` finish it?
///
/// `end_comment_pending` holds the last character of the 'comments' end
/// leader.  When it arrives, the *middle* leader that was auto-indented in
/// has to come off and the end leader go in -- all but its last character,
/// which the caller inserts as an ordinary one.
fn end_pending_comment(c: c_int) {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    if !did_ai.get() || c != end_comment_pending.get() {
        return;
    }

    // Find the comment leader this line starts with.
    let mut p: *mut c_char = ::core::ptr::null_mut();
    let line = unsafe { get_cursor_line_ptr() };
    let mut i = unsafe { get_leader_len(line, &raw mut p, false, true) };
    if i <= 0 || unsafe { vim_strchr(p, COM_MIDDLE) }.is_null() {
        return; // just checking
    }

    let mut lead_end: [c_char; COM_MAX_LEN as usize] = [0; COM_MAX_LEN as usize];

    // Skip the middle-comment string.
    while unsafe { *p } as c_int != 0 && unsafe { *p.offset(-1) } as c_int != b':' as c_int {
        p = unsafe { p.offset(1) }; // find the end of the middle flags
    }
    let comma = c",".as_ptr().cast_mut();
    let out = lead_end.as_mut_ptr();
    let mut middle_len =
        unsafe { copy_option_part(&raw mut p, out, COM_MAX_LEN as size_t, comma) } as c_int;
    // Trailing white space does not count towards `middle_len`.
    while middle_len > 0 && ascii_iswhite(lead_end[middle_len as usize - 1] as c_int) {
        middle_len -= 1;
    }

    // Find the end-comment string.
    while unsafe { *p } as c_int != 0 && unsafe { *p.offset(-1) } as c_int != b':' as c_int {
        p = unsafe { p.offset(1) }; // find the end of the end flags
    }
    let end_len =
        unsafe { copy_option_part(&raw mut p, out, COM_MAX_LEN as size_t, comma) } as c_int;

    // Skip the white space before the cursor, then back over the middle
    // leader.
    i = cur_win().w_cursor.col;
    while i > 0 && ascii_iswhite(unsafe { *line.offset(i as isize - 1) } as c_int) {
        i -= 1;
    }
    i -= middle_len;

    // Check some expected things before going on.
    if i >= 0
        && end_len > 0
        && lead_end[end_len as usize - 1] as uint8_t as c_int == end_comment_pending.get()
    {
        // Backspace over everything being replaced.
        unsafe { backspace_until_column(i) };
        // Insert the end-comment string except for its last character,
        // which the caller inserts as an ordinary one.
        unsafe { ins_bytes_len(lead_end.as_mut_ptr(), (end_len - 1) as size_t) };
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
    if p_paste.get() != 0 || no_abbr.get() || arrow_used.get() {
        return false;
    }

    let start_col = if cur_win().w_cursor.lnum == Insstart.get().lnum {
        Insstart.get().col
    } else {
        0
    };
    let col = cur_win().w_cursor.col;
    // SAFETY: `curwin`/`curbuf` are live, so the cursor's line is too.
    unsafe { check_abbr(c, get_cursor_line_ptr(), col, start_col) }
}

/// Run the `InsertCharPre` autocommand for `c`.
///
/// Answers an allocated replacement string when the autocommand changed
/// `v:char`, and null to go on inserting `c` -- which is also the answer when
/// there is no such autocommand at all.
pub(crate) fn do_insert_char_pre(c: c_int) -> *mut c_char {
    // SAFETY: every `unsafe` call below is an editor-wide routine whose only
    // precondition is the live `curwin`/`curbuf` this mode runs with.
    // The strings walked below are NUL-terminated lines of that buffer, and
    // every step stops at the NUL.
    if c == Ctrl_RSB || !unsafe { has_event(EVENT_INSERTCHARPRE) } {
        return ::core::ptr::null_mut();
    }

    let save_state = State.get();
    let mut buf: [c_char; MB_MAXBYTES + 1] = [0; MB_MAXBYTES + 1];
    let buflen = unsafe { utf_char2bytes(c, buf.as_mut_ptr()) } as size_t;
    buf[buflen] = NUL as c_char;

    let locked = Lock::text();
    unsafe { set_vim_var_string(Vv::Char, buf.as_mut_ptr(), buflen as ptrdiff_t) };

    let mut res = ::core::ptr::null_mut();
    if unsafe { ins_apply_autocmds(EVENT_INSERTCHARPRE) } != 0
        && unsafe { strcmp(buf.as_mut_ptr(), get_vim_var_str(Vv::Char)) } != 0
    {
        res = unsafe { xstrdup(get_vim_var_str(Vv::Char)) };
    }

    unsafe { set_vim_var_string(Vv::Char, ::core::ptr::null(), -1) };
    drop(locked);
    State.set(save_state);
    res
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
