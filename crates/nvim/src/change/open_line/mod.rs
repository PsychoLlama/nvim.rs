//! `open_line` -- the new line `o`, `O`, `<CR>` and an auto-wrap all make.
//!
//! One question, asked as a pipeline, which is how it is split:
//!
//! | phase | where |
//! | --- | --- |
//! | cut the current line in two at the cursor | [`open_line`] |
//! | guess an indent from 'autoindent'/'smartindent' | [`smart`] |
//! | decide and build the comment leader | [`comment`] |
//! | put the new line in the buffer | [`append_new_line`] |
//! | apply the indent, feeding the replace stack | [`apply_new_indent`] |
//! | shorten the old line and splice the extmarks | [`truncate_old_line`] |
//! | reindent with 'cindent'/'indentexpr'/'lisp' | [`reindent_new_line`] |
//!
//! Two things make it harder than that reads.
//!
//! **Virtual Replace mode does not add a line at all** unless the cursor is
//! on the last line: it starts *replacing* the next one. So the new text is
//! built as if a line were being opened, then taken back off, the original
//! line put back, and the text re-inserted character by character through
//! `ins_bytes` so that every replaced byte reaches the replace stack. That is
//! the `next_line` copy and the epilogue at the bottom.
//!
//! **It is reentrant with `textformat.rs`**: `internal_format` calls it to
//! break a line, and `OPENLINE_FORMAT` is how it knows. The `did_do_comment`
//! out-parameter is set when a comment leader was deliberately put in front
//! of the new line, which is what stops the second half of a broken line from
//! starting a *new* comment.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use self::comment::{LeaderContext, build_leader, indent_after_comment_end, plan_leader};
use self::smart::smart_indent;
use super::*;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_flags, cmdmod_set_flags};
use crate::types::{FAIL, FoFlag, NUL};

mod comment;
mod smart;

/// Whether Replace mode -- but not *Virtual* Replace mode, which keeps the
/// replace stack itself -- is active.
///
/// Upstream's `REPLACE_NORMAL(s)`.
fn replace_normal(state: c_int) -> bool {
    state & REPLACE_FLAG != 0 && state & VREPLACE_FLAG == 0
}

/// A prompt line cannot have anything inserted above it, so `O` on one moves
/// the prompt down instead: the prompt text is taken off the current line and
/// prepended to what will become the new line.
///
/// Answers the allocated replacement for `p_extra`, or null if this was not a
/// prompt line. Freezes the marks (`CmdModFlags::LOCKMARKS`) for the move, so that
/// `b_prompt_start` stays where it is.
///
/// # Safety
/// The cursor must be on a valid line of the current buffer.
unsafe fn move_prompt_down(p_extra: *mut c_char) -> *mut c_char {
    if !unsafe { bt_prompt(curbuf.get()) }
        || cur_win().w_cursor.lnum != cur_buf().b_prompt_start.mark.lnum
    {
        return ::core::ptr::null_mut();
    }
    let prompt_line = ml_get(cur_win().w_cursor.lnum);
    let prompt = unsafe { prompt_text() };
    let prompt_len = unsafe { strlen(prompt) };
    if unsafe { strncmp(prompt_line, prompt, prompt_len) } != 0 {
        return ::core::ptr::null_mut();
    }
    // STRMOVE: take the prompt off the front of the line.
    let rest = prompt_line.wrapping_add(prompt_len);
    // SAFETY: the prompt is the first `prompt_len` bytes of the line, so the
    // rest of it -- including the NUL -- fits where the prompt was.
    unsafe { memmove(prompt_line.cast(), rest.cast(), strlen(rest) + 1) };
    cmdmod_add_flags(CmdModFlags::LOCKMARKS);
    unsafe { ml_replace(cur_win().w_cursor.lnum, prompt_line, true) };
    unsafe { concat_str(prompt, p_extra) }
}

/// Put `p_extra` in the buffer as the new line.
///
/// Answers `Some(did_append)` -- false meaning Virtual Replace overwrote the
/// following line instead of adding one -- or `None` when `ml_append` failed.
/// The cursor is left on the line *above* the new one either way.
///
/// # Safety
/// The cursor must be on a valid line and `p_extra` NUL-terminated.
unsafe fn append_new_line(p_extra: *mut c_char, old_cursor: pos_T) -> Option<bool> {
    if State.get() & VREPLACE_FLAG == 0 || old_cursor.lnum >= orig_line_count.get() {
        if unsafe { ml_append(cur_win().w_cursor.lnum, p_extra, 0, false) } == FAIL {
            return None;
        }
        // changed_lines() is postponed: calling it here would upset
        // marker folding.
        let below = cur_win().w_cursor.lnum + 1;
        let max = MAXLNUM as linenr_T;
        // SAFETY: the editor exists.
        unsafe { mark_adjust(below, max, 1, 0, kExtmarkNOOP) };
        return Some(true);
    }

    // Virtual Replace: start replacing the next line.
    cur_win().w_cursor.lnum += 1;
    if cur_win().w_cursor.lnum >= Insstart.get().lnum + vr_lines_changed.get() {
        // NL to a new line, BS back, NL again: don't save the new line
        // for undo twice. Errors are ignored.
        u_save_cursor();
        vr_lines_changed.set(vr_lines_changed.get() + 1);
    }
    unsafe { ml_replace(cur_win().w_cursor.lnum, p_extra, true) };
    unsafe { changed_bytes(cur_win().w_cursor.lnum, 0) };
    cur_win().w_cursor.lnum -= 1;
    Some(false)
}

/// Indent the new line, and account for the columns that takes.
///
/// `less_cols` and `newcol` are adjusted in place; the return value is the
/// possibly-shifted indent.
///
/// # Safety
/// The cursor must be on the line above the new one.
unsafe fn apply_new_indent(
    mut newindent: c_int,
    saved_line: *mut c_char,
    less_cols: &mut colnr_T,
    newcol: &mut colnr_T,
    no_si: bool,
) {
    cur_win().w_cursor.lnum += 1;
    if did_si.get() {
        let sw = unsafe { get_sw_value(curbuf.get()) };
        if p_sr.get() != 0 {
            newindent -= newindent % sw;
        }
        newindent += sw;
    }

    if cur_buf().b_p_ci != 0 {
        unsafe { copy_indent(newindent, saved_line) };
        // Keep 'preserveindent' on so that later fiddling with the line
        // does not undo the copy; restored at the end of `open_line`.
        cur_buf().b_p_pi = true as c_int;
    } else {
        unsafe { set_indent(newindent, SIN_INSERT | SIN_NOMARK) };
    }
    *less_cols -= cur_win().w_cursor.col;
    ai_col.set(cur_win().w_cursor.col);

    // In Replace mode every character of the new indent needs a NUL on
    // the replace stack, for when BS deletes it.
    if replace_normal(State.get()) {
        for _ in 0..cur_win().w_cursor.col {
            unsafe { replace_push_nul() };
        }
    }
    *newcol += cur_win().w_cursor.col;
    if no_si {
        did_si.set(false);
    }
}

/// Cut the old line off at the cursor and tell the extmark tree about both
/// halves of the split.
///
/// Takes ownership of `saved_line` (it becomes the buffer's line), so the
/// caller must not free it afterwards.
///
/// # Safety
/// The cursor must be back on the old line, and `did_append` describe what
/// [`append_new_line`] did.
#[allow(clippy::too_many_arguments)]
unsafe fn truncate_old_line(
    saved_line: *mut c_char,
    flags: c_int,
    trunc_line: bool,
    lnum: linenr_T,
    mincol: colnr_T,
    less_cols: colnr_T,
    less_cols_off: colnr_T,
    did_append: bool,
) {
    unsafe { *saved_line.offset(cur_win().w_cursor.col as isize) = NUL as c_char };
    // Remove trailing white space, unless the caller asked to keep it --
    // and only when the line being left was auto-indented, so that white
    // space the user typed on purpose survives.
    if trunc_line && flags & OPENLINE_KEEPTRAIL == 0 {
        unsafe { truncate_spaces(saved_line, cur_win().w_cursor.col as size_t) };
    }
    unsafe { ml_replace(cur_win().w_cursor.lnum, saved_line, false) };

    let new_len = unsafe { strlen(saved_line) } as c_int;
    let mut cols_spliced = 0;
    if new_len < cur_win().w_cursor.col {
        // Trailing white space went as well as the split.
        let cb = curbuf.get();
        let row = cur_win().w_cursor.lnum - 1;
        let gone = cur_win().w_cursor.col - new_len;
        // SAFETY: the current buffer is live, and the row is the line just
        // replaced.
        unsafe { extmark_splice_cols(cb, row, new_len, gone, 0, kExtmarkUndo) };
        cols_spliced = gone;
    }

    if did_append {
        // Move the extmarks of the line the cursor is on; the
        // mark_adjust() in `append_new_line` took care of the lines below.
        let added = mincol - 1 + less_cols_off - less_cols;
        let cb = curbuf.get();
        let row = lnum - 1;
        let at = mincol - 1 - cols_spliced;
        let off = less_cols_off;
        let old_b = off as bcount_t;
        let new_b = (1 + added) as bcount_t;
        let undo = kExtmarkUndo;
        let (cur_lnum, cur_col) = (cur_win().w_cursor.lnum, cur_win().w_cursor.col);
        // SAFETY: the current buffer is live, and the row names the line that
        // was just split.
        unsafe {
            extmark_splice(cb, row, at, 0, off, old_b, 1, added, new_b, undo);
            changed_lines(Buf::new(cb), cur_lnum, cur_col, cur_lnum + 1, 1, true);
        }
        // Move marks that were after the break onto the new line.
        if flags & OPENLINE_MARKFIX != 0 {
            // SAFETY: the editor exists.
            unsafe { mark_col_adjust(cur_lnum, cur_col + off, 1, -less_cols, 0) };
        }
    } else {
        unsafe { changed_bytes(cur_win().w_cursor.lnum, cur_win().w_cursor.col) };
    }
}

/// Reindent the new line with 'lisp', 'cindent' or 'indentexpr', whichever
/// applies.
///
/// Virtual Replace handles the replace stack itself, so `State` is faked to
/// plain Insert first to stop `change_indent()` from touching it.
///
/// # Safety
/// The cursor must be on the new line.
unsafe fn reindent_new_line(leader: *mut c_char, do_cindent: bool) {
    let vreplace_mode = if State.get() & VREPLACE_FLAG != 0 {
        let saved = State.get();
        State.set(MODE_INSERT);
        saved
    } else {
        0
    };

    if p_paste.get() == 0 {
        if leader.is_null()
            && !unsafe { use_indentexpr_for_lisp() }
            && cur_buf().b_p_lisp != 0
            && cur_buf().b_p_ai != 0
        {
            unsafe { fixthisline(Some(get_lisp_indent as unsafe fn() -> c_int)) };
            ai_col.set(unsafe { getwhitecols_curline() } as colnr_T);
        } else if do_cindent || (cur_buf().b_p_ai != 0 && unsafe { use_indentexpr_for_lisp() }) {
            unsafe { do_c_expr_indent() };
            ai_col.set(unsafe { getwhitecols_curline() } as colnr_T);
        }
    }

    if vreplace_mode != 0 {
        State.set(vreplace_mode);
    }
}

/// Add a new line below or above the current one.
///
/// `dir` is `FORWARD` (`o`, `<CR>`) or `BACKWARD` (`O`). `flags` is the
/// `OPENLINE_*` set. `second_line_indent` is the indent wanted after `CTRL-D`
/// in Insert mode -- or, with `OPENLINE_COM_LIST`, the *column* a 'formatlistpat'
/// item's text should line up at. `did_do_comment` is set to true when a
/// comment leader was deliberately put in front of the new line.
///
/// The caller takes care of undo. Virtual Replace may touch any number of
/// lines, so this calls `u_save_cursor()` again itself when it starts on a
/// new one.
///
/// Answers false only when `ml_append` failed.
///
/// # Safety
/// The cursor must be on a valid line of the current buffer.
pub unsafe fn open_line(
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    did_do_comment: *mut bool,
) -> bool {
    let do_si = unsafe { may_do_si() };
    let saved_pi = cur_buf().b_p_pi;
    let lnum = cur_win().w_cursor.lnum;
    let mincol = cur_win().w_cursor.col + 1;

    // A copy of the current line, so that it can be cut in two.
    let mut saved_line = copy_cursor_line();
    let mut next_line: *mut c_char = ::core::ptr::null_mut();

    if State.get() & VREPLACE_FLAG != 0 {
        // Virtual Replace keeps a copy of the line it is about to start
        // replacing. The new line is built empty so that the indent and
        // leader machinery below can do as it likes; what it produces is
        // taken off again at the bottom and inserted character by
        // character over the original.  -- webb.
        next_line = if cur_win().w_cursor.lnum < orig_line_count.get() {
            let next = cur_win().w_cursor.lnum + 1;
            // SAFETY: `next` is a line of the current buffer, as just tested.
            unsafe { xstrnsave(ml_get(next), ml_get_len(next) as size_t) }
        } else {
            // SAFETY: a static empty string.
            unsafe { xstrdup(c"".as_ptr()) }
        };

        // A NL replaces the rest of the line, so everything past the
        // cursor goes on the replace stack. Twice, because BS over a NL
        // expects it.
        unsafe { replace_push_nul() };
        unsafe { replace_push_nul() };
        let p = unsafe { saved_line.offset(cur_win().w_cursor.col as isize) };
        unsafe { replace_push(p, strlen(p)) };
        unsafe { *p = NUL as c_char };
    }

    // What moves to the new line: the tail of the old one. Cut with a NUL
    // for now, and remember the byte so it can be put back.
    let mut p_extra: *mut c_char = ::core::ptr::null_mut();
    let mut extra_len = 0;
    let mut saved_char = NUL as c_char;
    let mut first_char = NUL;
    if State.get() & MODE_INSERT != 0 && State.get() & VREPLACE_FLAG == 0 {
        p_extra = unsafe { saved_line.offset(cur_win().w_cursor.col as isize) };
        if do_si {
            // 'smartindent' wants the first character after the break.
            first_char = c_int::from(unsafe { *skipwhite(p_extra) } as u8);
        }
        extra_len = unsafe { strlen(p_extra) } as c_int;
        saved_char = unsafe { *p_extra };
        unsafe { *p_extra = NUL as c_char };
    }

    u_clearline(cur_buf()); // "U" cannot undo added lines
    did_si.set(false);
    ai_col.set(0);

    // An auto-indent means nothing was typed on the line being left, so
    // it should be truncated. Also true when only a comment leader was
    // inserted automatically, which sets did_ai too.
    let trunc_line = dir == FORWARD && did_ai.get();

    let mut newindent = 0;
    let mut no_si = false;
    if flags & OPENLINE_FORCE_INDENT != 0 {
        newindent = second_line_indent;
    } else if cur_buf().b_p_ai != 0 || do_si {
        let ts = cur_buf().b_p_ts;
        let vts = cur_buf().b_p_vts_array;
        // SAFETY: `saved_line` is this frame's NUL-terminated copy of the
        // line, and the tabstops are the buffer's own.
        newindent = unsafe { indent_size_ts(saved_line, ts, vts) };
        if newindent == 0 && flags & OPENLINE_COM_LIST == 0 {
            newindent = second_line_indent; // CTRL-D in Insert mode
        }
        // Text moving to the next line that starts with `{` gets no extra
        // indent, so that a <CR> before the `{` of "if (cond) {" works.
        if !trunc_line
            && do_si
            && c_int::from(unsafe { *saved_line }) != NUL
            && (p_extra.is_null() || first_char != '{' as c_int)
        {
            (newindent, no_si) = unsafe { smart_indent(dir, flags, saved_line, newindent) };
        }
        if do_si {
            can_si.set(true);
        }
        did_ai.set(true);
    }

    // Whether to reindent once the line is open.
    let key = if dir == FORWARD {
        KEY_OPEN_FORW
    } else {
        KEY_OPEN_BACK
    };
    // SAFETY: `b_p_inde` is the buffer's own `'indentexpr'` string, and
    // `in_cinkeys` reads `'cinkeys'` and the cursor line; the short circuits
    // are upstream's.
    let do_cindent = p_paste.get() == 0
        && (cur_buf().b_p_cin != 0 || c_int::from(unsafe { *cur_buf().b_p_inde }) != NUL)
        && unsafe { in_cinkeys(key, ' ' as c_int, linewhite(cur_win().w_cursor.lnum)) }
        && flags & OPENLINE_FORCE_INDENT == 0;

    // Does the current line start with a comment leader that should be
    // repeated in front of the new line?
    end_comment_pending.set(NUL);
    let mut lead_flags: *mut c_char = ::core::ptr::null_mut();
    let mut comment_start = 0;
    let mut lead_len = 0;
    if flags & OPENLINE_DO_COM != 0 {
        lead_len =
            unsafe { get_leader_len(saved_line, &raw mut lead_flags, dir == BACKWARD, true) };
        if lead_len == 0
            && cur_buf().b_p_cin != 0
            && do_cindent
            && dir == FORWARD
            && (!has_format_option(FoFlag::NO_OPEN_COMS) || flags & OPENLINE_FORMAT != 0)
        {
            // A line comment after code: `code(); // why`.
            comment_start = unsafe { check_linecomment(saved_line) };
            if comment_start != MAXCOL {
                let at = saved_line.wrapping_offset(comment_start as isize);
                let out = &raw mut lead_flags;
                // SAFETY: `at` is inside this frame's NUL-terminated copy of
                // the line, and `lead_flags` is a local.
                lead_len = unsafe { get_leader_len(at, out, false, true) };
                if lead_len != 0 {
                    lead_len += comment_start;
                    if !did_do_comment.is_null() {
                        unsafe { *did_do_comment = true };
                    }
                }
            }
        }
    }

    let mut leader: *mut c_char = ::core::ptr::null_mut();
    let mut allocated: *mut c_char = ::core::ptr::null_mut();
    let mut newcol: colnr_T = 0;
    if lead_len > 0 {
        let plan = unsafe { plan_leader(dir, lead_len, lead_flags, saved_line, p_extra) };
        lead_len = plan.lead_len;
        if lead_len > 0 {
            let ctx = LeaderContext {
                comment_start,
                extra_len,
                second_line_indent,
            };
            // SAFETY: `lead_flags` points into 'comments' and `saved_line` is
            // this frame's copy of the line.
            let built =
                unsafe { build_leader(plan, lead_flags, saved_line, ctx, do_si, newindent) };
            leader = built.leader;
            allocated = built.allocated;
            lead_len = built.lead_len;
            newcol = built.newcol;
            newindent = built.newindent;
        } else if !plan.comment_end.is_null() {
            newindent =
                unsafe { indent_after_comment_end(plan.comment_end, saved_line, do_si, newindent) };
        }
    }

    // Only reached with dir == FORWARD, in Insert or Replace state.
    let mut less_cols: colnr_T = 0;
    let mut less_cols_off: colnr_T = 0;
    if !p_extra.is_null() {
        unsafe { *p_extra = saved_char }; // put back the byte the NUL replaced

        // With 'autoindent' or OPENLINE_DELSPACES, skip to the first
        // non-blank. In Replace mode the blanks go on the replace stack,
        // preceded by a NUL, so BS can put them back.
        if replace_normal(State.get()) {
            unsafe { replace_push_nul() }; // end of the extra blanks
        }
        if cur_buf().b_p_ai != 0 || flags & OPENLINE_DELSPACES != 0 {
            while (c_int::from(unsafe { *p_extra }) == ' ' as c_int
                || c_int::from(unsafe { *p_extra }) == '\t' as c_int)
                && !utf_iscomposing_first(unsafe { utf_ptr2char(p_extra.add(1)) })
            {
                if replace_normal(State.get()) {
                    unsafe { replace_push(p_extra, 1) }; // always ascii, len = 1
                }
                p_extra = unsafe { p_extra.add(1) };
                less_cols_off += 1;
            }
        }
        // Columns for the marks, adjusted for the ones just removed.
        less_cols = unsafe { p_extra.offset_from(saved_line) } as colnr_T;
    }
    if p_extra.is_null() {
        p_extra = c"".as_ptr().cast_mut(); // append an empty line
    }

    if lead_len > 0 {
        if flags & OPENLINE_COM_LIST != 0 && second_line_indent > 0 {
            // 'formatlistpat': pad after the comment leader so that the
            // text lines up with the first line's. The white space
            // *before* the leader is `set_indent`'s job below.
            let padding = second_line_indent - (newindent + unsafe { strlen(leader) } as c_int);
            for _ in 0..padding {
                unsafe { strcat(leader, c" ".as_ptr()) };
                less_cols -= 1;
                newcol += 1;
            }
        }
        unsafe { strcat(leader, p_extra) };
        p_extra = leader;
        did_ai.set(true); // so that truncating blanks works with comments
        less_cols -= lead_len;
    } else {
        end_comment_pending.set(NUL); // there was no leader after all
    }

    curbuf_splice_pending.set(curbuf_splice_pending.get() + 1);
    let old_cursor = cur_win().w_cursor;
    let old_cmod_flags = cmdmod_flags();
    let mut prompt_moved: *mut c_char = ::core::ptr::null_mut();
    if dir == BACKWARD {
        prompt_moved = unsafe { move_prompt_down(p_extra) };
        if !prompt_moved.is_null() {
            p_extra = prompt_moved;
        }
        cur_win().w_cursor.lnum -= 1;
    }

    let mut retval = false;
    'theend: {
        let found = unsafe { append_new_line(p_extra, old_cursor) };
        let Some(mut did_append) = found else {
            break 'theend;
        };

        inhibit_delete_count.set(inhibit_delete_count.get() + 1);
        if newindent != 0 || did_si.get() {
            unsafe { apply_new_indent(newindent, saved_line, &mut less_cols, &mut newcol, no_si) };
        }
        inhibit_delete_count.set(inhibit_delete_count.get() - 1);

        // One NUL on the replace stack per character of the leader, for
        // when BS deletes it.
        if replace_normal(State.get()) {
            while lead_len > 0 {
                unsafe { replace_push_nul() };
                lead_len -= 1;
            }
        }

        cur_win().w_cursor = old_cursor;
        if dir == FORWARD {
            if trunc_line || State.get() & MODE_INSERT != 0 {
                unsafe {
                    truncate_old_line(
                        saved_line,
                        flags,
                        trunc_line,
                        lnum,
                        mincol,
                        less_cols,
                        less_cols_off,
                        did_append,
                    )
                };
                // The buffer owns the line now.
                saved_line = ::core::ptr::null_mut();
                did_append = false;
            }
            // Put the cursor on the new line. `old_cursor`, not
            // `w_cursor`: a scroll above may have moved the latter.
            cur_win().w_cursor.lnum = old_cursor.lnum + 1;
        }
        if did_append {
            let cb = curbuf.get();
            let at = cur_win().w_cursor.lnum;
            // SAFETY: the current buffer is live and `at` is the new line.
            let extra = ml_get_len(at) as bcount_t;
            // SAFETY: as above.
            unsafe {
                extmark_splice(cb, at - 1, 0, 0, 0, 0, 1, 0, 1 + extra, kExtmarkUndo);
                changed_lines(Buf::new(cb), at, 0, at, 1, true);
            }
        }
        curbuf_splice_pending.set(curbuf_splice_pending.get() - 1);

        cur_win().w_cursor.col = newcol;
        cur_win().w_cursor.coladd = 0;

        unsafe { reindent_new_line(leader, do_cindent) };

        if State.get() & VREPLACE_FLAG != 0 {
            // Take what ended up on the new line back off, put the
            // original line back, and insert the new text character by
            // character so that each replaced byte reaches the replace
            // stack.
            let new_text = copy_cursor_line();
            let at = cur_win().w_cursor.lnum;
            // SAFETY: `next_line` is this frame's allocation, which the buffer
            // takes over.
            unsafe { ml_replace(at, next_line, false) };
            cur_win().w_cursor.col = 0;
            cur_win().w_cursor.coladd = 0;
            // SAFETY: `new_text` is this frame's NUL-terminated allocation.
            unsafe {
                ins_bytes(new_text); // calls changed_bytes()
                xfree(new_text.cast());
            }
            next_line = ::core::ptr::null_mut(); // the buffer owns it now
        }
        retval = true;
    }

    cur_buf().b_p_pi = saved_pi;
    free_str(saved_line);
    free_str(next_line);
    free_str(allocated);
    free_str(prompt_moved);
    cmdmod_set_flags(old_cmod_flags);
    retval
}

/// A fresh copy of the line the cursor is on.
fn copy_cursor_line() -> *mut c_char {
    // SAFETY: the cursor is on a valid line, so the pointer and the length
    // describe the same line.
    unsafe { xstrnsave(get_cursor_line_ptr(), get_cursor_line_len() as size_t) }
}

/// Release one of [`open_line`]'s own strings; a null is fine.
fn free_str(p: *mut c_char) {
    // SAFETY: this frame's own allocation, or null.
    unsafe { xfree(p.cast::<c_void>()) };
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
