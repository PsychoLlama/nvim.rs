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

#![forbid(unsafe_code)]

use crate::cstr::byte_at;
use crate::guard::Suppress;
use crate::memline::{Lines, ml_append_terminated};
use crate::memory::XString;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use self::comment::{LeaderContext, build_leader, indent_after_comment_end, plan_leader};
use self::smart::smart_indent;
use super::*;
use crate::edit::buf_prompt_text_owned;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_flags, cmdmod_set_flags};
use crate::mbyte::char_at;
use crate::optionstr::OptString;
use crate::types::{FoFlag, NUL};

mod comment;
mod smart;

/// Whether Replace mode -- but not *Virtual* Replace mode, which keeps the
/// replace stack itself -- is active.
///
/// Upstream's `REPLACE_NORMAL(s)`.
fn replace_normal(state: c_int) -> bool {
    state & REPLACE_FLAG != 0 && state & VREPLACE_FLAG == 0
}

/// How many bytes of white space start `text`.
fn white_run(text: &[u8]) -> usize {
    text.iter()
        .take_while(|&&b| ascii_iswhite(c_int::from(b)))
        .count()
}

/// A prompt line cannot have anything inserted above it, so `O` on one moves
/// the prompt down instead: the prompt text is taken off the current line and
/// prepended to `text`, what will become the new line.
///
/// Answers the new line, or `None` if this was not a prompt line. Freezes
/// the marks (`CmdModFlags::LOCKMARKS`) for the move, so that
/// `b_prompt_start` stays where it is.
fn move_prompt_down(text: &[u8]) -> Option<XString> {
    if !buf_is_prompt(current_buf())
        || Win::current().w_cursor.lnum != Buf::current().b_prompt_start.mark.lnum
    {
        return None;
    }
    let lnum = Win::current().w_cursor.lnum;
    let prompt = buf_prompt_text_owned(Buf::current());
    let prompt = prompt.as_bytes();
    let rest = {
        let mut lines = Buf::current().lines();
        let line = lines.line(lnum);
        if !line.starts_with(prompt) {
            return None;
        }
        // Take the prompt off the front of the line.
        XString::from_bytes(&line[prompt.len()..])
    };
    cmdmod_add_flags(CmdModFlags::LOCKMARKS);
    let _ = ml_replace_owned(lnum, rest);
    Some(concat(prompt, text))
}

/// Put `text` in the buffer as the new line.
///
/// Answers `Some(did_append)` -- false meaning Virtual Replace overwrote the
/// following line instead of adding one -- or `None` when `ml_append` failed.
/// The cursor is left on the line *above* the new one either way.
fn append_new_line(text: &XString, old_cursor: Pos) -> Option<bool> {
    if State.get() & VREPLACE_FLAG == 0 || old_cursor.lnum >= orig_line_count.get() {
        let line = text.as_bytes_with_nul();
        if ml_append_terminated(Win::current().w_cursor.lnum, line, false).is_err() {
            return None;
        }
        // changed_lines() is postponed: calling it here would upset
        // marker folding.
        let below = Win::current().w_cursor.lnum + 1;
        let max = MAXLNUM;
        mark_adjust(below, max, 1, 0, kExtmarkNOOP);
        return Some(true);
    }

    // Virtual Replace: start replacing the next line.
    Win::current().w_cursor.lnum += 1;
    if Win::current().w_cursor.lnum >= Insstart.get().lnum + vr_lines_changed.get() {
        // NL to a new line, BS back, NL again: don't save the new line
        // for undo twice. Errors are ignored.
        let _ = u_save_cursor();
        vr_lines_changed.set(vr_lines_changed.get() + 1);
    }
    let _ = ml_replace_buf_text(Buf::current(), Win::current().w_cursor.lnum, text);
    changed_bytes(Win::current().w_cursor.lnum, 0);
    Win::current().w_cursor.lnum -= 1;
    Some(false)
}

/// Indent the new line, and account for the columns that takes.
///
/// `less_cols` and `newcol` are adjusted in place. `saved_line` is the old
/// line, whose indent 'copyindent' copies. The cursor must be on the line
/// above the new one.
fn apply_new_indent(
    mut newindent: c_int,
    saved_line: &[u8],
    less_cols: &mut ColNr,
    newcol: &mut ColNr,
    no_si: bool,
) {
    Win::current().w_cursor.lnum += 1;
    if did_si.get() {
        let sw = get_sw_value(Buf::current());
        if p_sr() {
            newindent -= newindent % sw;
        }
        newindent += sw;
    }

    if Buf::current().b_p_ci != 0 {
        copy_indent(newindent, saved_line);
        // Keep 'preserveindent' on so that later fiddling with the line
        // does not undo the copy; restored at the end of `open_line`.
        Buf::current().b_p_pi = true as c_int;
    } else {
        set_indent(newindent, SIN_INSERT | SIN_NOMARK);
    }
    *less_cols -= Win::current().w_cursor.col;
    ai_col.set(Win::current().w_cursor.col);

    // In Replace mode every character of the new indent needs a NUL on
    // the replace stack, for when BS deletes it.
    if replace_normal(State.get()) {
        for _ in 0..Win::current().w_cursor.col {
            replace_push_nul();
        }
    }
    *newcol += Win::current().w_cursor.col;
    if no_si {
        did_si.set(false);
    }
}

/// Where [`truncate_old_line`] splices the extmarks of the line that was
/// split.
struct Split {
    lnum: LineNr,
    mincol: ColNr,
    less_cols: ColNr,
    less_cols_off: ColNr,
}

/// Cut the old line off at the cursor and tell the extmark tree about both
/// halves of the split.
///
/// `saved_line` becomes the buffer's line. The cursor must be back on the
/// old line, and `did_append` describe what [`append_new_line`] did.
fn truncate_old_line(
    mut saved_line: Vec<u8>,
    flags: c_int,
    trunc_line: bool,
    split: Split,
    did_append: bool,
) {
    saved_line.truncate(Win::current().w_cursor.col as usize);
    // Remove trailing white space, unless the caller asked to keep it --
    // and only when the line being left was auto-indented, so that white
    // space the user typed on purpose survives.
    if trunc_line && flags & OPENLINE_KEEPTRAIL == 0 {
        truncate_spaces(&mut saved_line);
    }
    let new_len = saved_line.len() as c_int;
    let _ = ml_replace_owned(Win::current().w_cursor.lnum, XString::from_vec(saved_line));

    let mut cols_spliced = 0;
    if new_len < Win::current().w_cursor.col {
        // Trailing white space went as well as the split.
        let cb = Buf::current();
        let row = Win::current().w_cursor.lnum - 1;
        let gone = Win::current().w_cursor.col - new_len;
        extmark_splice_cols(cb, row, new_len, gone, 0, kExtmarkUndo);
        cols_spliced = gone;
    }

    if did_append {
        let Split {
            lnum,
            mincol,
            less_cols,
            less_cols_off,
        } = split;
        // Move the extmarks of the line the cursor is on; the
        // mark_adjust() in `append_new_line` took care of the lines below.
        let added = mincol - 1 + less_cols_off - less_cols;
        let cb = Buf::current();
        let row = lnum - 1;
        let at = mincol - 1 - cols_spliced;
        let off = less_cols_off;
        let old_b = off as BCount;
        let new_b = (1 + added) as BCount;
        let undo = kExtmarkUndo;
        let (cur_lnum, cur_col) = (Win::current().w_cursor.lnum, Win::current().w_cursor.col);
        extmark_splice(cb, row, at, 0, off, old_b, 1, added, new_b, undo);
        changed_lines(cb, cur_lnum, cur_col, cur_lnum + 1, 1, true);
        // Move marks that were after the break onto the new line.
        if flags & OPENLINE_MARKFIX != 0 {
            mark_col_adjust(cur_lnum, cur_col + off, 1, -less_cols, 0);
        }
    } else {
        changed_bytes(Win::current().w_cursor.lnum, Win::current().w_cursor.col);
    }
}

/// Reindent the new line with 'lisp', 'cindent' or 'indentexpr', whichever
/// applies; 'lisp' only when no comment leader was built.
///
/// Virtual Replace handles the replace stack itself, so `State` is faked to
/// plain Insert first to stop `change_indent()` from touching it. The cursor
/// must be on the new line.
fn reindent_new_line(has_leader: bool, do_cindent: bool) {
    let vreplace_mode = if State.get() & VREPLACE_FLAG != 0 {
        let saved = State.get();
        State.set(MODE_INSERT);
        saved
    } else {
        0
    };

    if !p_paste() {
        if !has_leader
            && !use_indentexpr_for_lisp()
            && Buf::current().b_p_lisp != 0
            && Buf::current().b_p_ai != 0
        {
            fixthisline(Some(get_lisp_indent as unsafe fn() -> c_int));
            ai_col.set(getwhitecols_curline() as ColNr);
        } else if do_cindent || (Buf::current().b_p_ai != 0 && use_indentexpr_for_lisp()) {
            do_c_expr_indent();
            ai_col.set(getwhitecols_curline() as ColNr);
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
/// new one. The cursor must be on a valid line of the current buffer.
///
/// Answers false only when `ml_append` failed.
pub fn open_line(
    dir: c_int,
    flags: c_int,
    second_line_indent: c_int,
    did_do_comment: Option<&mut bool>,
) -> bool {
    let do_si = may_do_si();
    let saved_pi = Buf::current().b_p_pi;
    let lnum = Win::current().w_cursor.lnum;
    let mincol = Win::current().w_cursor.col + 1;
    let col = Win::current().w_cursor.col as usize;

    // A copy of the current line, so that it can be cut in two.
    let mut saved_line = Lines::current().line(lnum).to_vec();
    let mut next_line = None;

    if State.get() & VREPLACE_FLAG != 0 {
        // Virtual Replace keeps a copy of the line it is about to start
        // replacing. The new line is built empty so that the indent and
        // leader machinery below can do as it likes; what it produces is
        // taken off again at the bottom and inserted character by
        // character over the original.  -- webb.
        next_line = Some(if lnum < orig_line_count.get() {
            XString::from_bytes(Lines::current().line(lnum + 1))
        } else {
            XString::new()
        });

        // A NL replaces the rest of the line, so everything past the
        // cursor goes on the replace stack. Twice, because BS over a NL
        // expects it.
        replace_push_nul();
        replace_push_nul();
        replace_push(&saved_line[col..]);
        saved_line.truncate(col);
    }

    // What moves to the new line: the tail of the old one, from `extra` on.
    // Until the leader is built the line is read as ending there -- the C
    // writes a NUL over the cut and puts the byte back afterwards.
    let mut extra = None;
    let mut extra_len = 0;
    let mut first_char = NUL;
    if State.get() & MODE_INSERT != 0 && State.get() & VREPLACE_FLAG == 0 {
        extra = Some(col);
        let tail = &saved_line[col..];
        if do_si {
            // 'smartindent' wants the first character after the break.
            first_char = c_int::from(byte_at(tail, white_run(tail)));
        }
        extra_len = tail.len() as c_int;
    }
    let cut = extra.unwrap_or(saved_line.len());

    u_clearline(Buf::current()); // "U" cannot undo added lines
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
    } else if Buf::current().b_p_ai != 0 || do_si {
        let head = &saved_line[..cut];
        newindent = Buf::current().indent_width(head);
        if newindent == 0 && flags & OPENLINE_COM_LIST == 0 {
            newindent = second_line_indent; // CTRL-D in Insert mode
        }
        // Text moving to the next line that starts with `{` gets no extra
        // indent, so that a <CR> before the `{` of "if (cond) {" works.
        if !trunc_line
            && do_si
            && !head.is_empty()
            && (extra.is_none() || first_char != '{' as c_int)
        {
            (newindent, no_si) = smart_indent(dir, flags, head, newindent);
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
    // The short circuits are upstream's: `in_cinkeys` reads 'cinkeys' and
    // the cursor line.
    let do_cindent = !p_paste()
        && (Buf::current().b_p_cin != 0
            || c_int::from(Buf::current().b_p_inde.first_byte()) != NUL)
        && in_cinkeys(key, ' ' as c_int, linewhite(Win::current().w_cursor.lnum))
        && flags & OPENLINE_FORCE_INDENT == 0;

    // Does the current line start with a comment leader that should be
    // repeated in front of the new line?
    end_comment_pending.set(NUL);
    let mut lead_flags = None;
    let mut comment_start = 0;
    let mut lead_len = 0;
    if flags & OPENLINE_DO_COM != 0 {
        let head = &saved_line[..cut];
        (lead_len, lead_flags) = get_leader_len(head, dir == BACKWARD, true);
        if lead_len == 0
            && Buf::current().b_p_cin != 0
            && do_cindent
            && dir == FORWARD
            && (!has_format_option(FoFlag::NO_OPEN_COMS) || flags & OPENLINE_FORMAT != 0)
        {
            // A line comment after code: `code(); // why`.
            comment_start = check_linecomment(head);
            if comment_start != MAXCOL {
                let (len, tried) = get_leader_len(&head[comment_start as usize..], false, true);
                // One out-parameter for both scans upstream: a scan that
                // tried nothing leaves the first one's answer.
                lead_flags = tried.or(lead_flags);
                lead_len = len;
                if lead_len != 0 {
                    lead_len += comment_start;
                    if let Some(did_do_comment) = did_do_comment {
                        *did_do_comment = true;
                    }
                }
            }
        }
    }

    let mut leader = None;
    let mut newcol: ColNr = 0;
    if let Some(lead_flags) = lead_flags.filter(|_| lead_len > 0) {
        let head = &saved_line[..cut];
        let plan = plan_leader(dir, lead_len, lead_flags, head, extra.is_some());
        lead_len = plan.lead_len;
        if lead_len > 0 {
            let ctx = LeaderContext {
                comment_start,
                extra_len,
                second_line_indent,
            };
            let built = build_leader(plan, lead_flags, head, ctx, do_si, newindent);
            leader = Some(built.text);
            lead_len = built.lead_len;
            newcol = built.newcol;
            newindent = built.newindent;
        } else if let Some(comment_end) = plan.comment_end {
            newindent = indent_after_comment_end(comment_end, head, do_si, newindent);
        }
    }
    let has_leader = leader.is_some();

    // Only reached with dir == FORWARD, in Insert or Replace state.
    let mut less_cols: ColNr = 0;
    let mut less_cols_off: ColNr = 0;
    let mut tail = &saved_line[saved_line.len()..];
    if let Some(mut from) = extra {
        // With 'autoindent' or OPENLINE_DELSPACES, skip to the first
        // non-blank. In Replace mode the blanks go on the replace stack,
        // preceded by a NUL, so BS can put them back.
        if replace_normal(State.get()) {
            replace_push_nul(); // end of the extra blanks
        }
        if Buf::current().b_p_ai != 0 || flags & OPENLINE_DELSPACES != 0 {
            while matches!(byte_at(&saved_line, from), b' ' | b'\t')
                && !utf_iscomposing_first(char_at(&saved_line[from + 1..]))
            {
                if replace_normal(State.get()) {
                    // The byte the loop just tested. Always ASCII.
                    replace_push(&saved_line[from..=from]);
                }
                from += 1;
                less_cols_off += 1;
            }
        }
        // Columns for the marks, adjusted for the ones just removed.
        less_cols = from as ColNr;
        tail = &saved_line[from..];
    }

    let mut new_text = match leader {
        Some(mut text) => {
            if flags & OPENLINE_COM_LIST != 0 && second_line_indent > 0 {
                // 'formatlistpat': pad after the comment leader so that the
                // text lines up with the first line's. The white space
                // *before* the leader is `set_indent`'s job below.
                let padding = second_line_indent - (newindent + text.len() as c_int);
                if padding > 0 {
                    text.resize(text.len() + padding as usize, b' ');
                    less_cols -= padding;
                    newcol += padding;
                }
            }
            text.extend_from_slice(tail);
            did_ai.set(true); // so that truncating blanks works with comments
            less_cols -= lead_len;
            XString::from_vec(text)
        }
        None => {
            end_comment_pending.set(NUL); // there was no leader after all
            XString::from_bytes(tail) // an empty line when nothing moves
        }
    };

    let splice = Suppress::splice();
    let old_cursor = Win::current().w_cursor;
    let old_cmod_flags = cmdmod_flags();
    if dir == BACKWARD {
        if let Some(moved) = move_prompt_down(&new_text) {
            new_text = moved;
        }
        Win::current().w_cursor.lnum -= 1;
    }

    let mut retval = false;
    'theend: {
        let found = append_new_line(&new_text, old_cursor);
        let Some(mut did_append) = found else {
            break 'theend;
        };

        let uncounted = Suppress::delete_count();
        if newindent != 0 || did_si.get() {
            apply_new_indent(newindent, &saved_line, &mut less_cols, &mut newcol, no_si);
        }
        drop(uncounted);

        // One NUL on the replace stack per character of the leader, for
        // when BS deletes it.
        if replace_normal(State.get()) {
            while lead_len > 0 {
                replace_push_nul();
                lead_len -= 1;
            }
        }

        Win::current().w_cursor = old_cursor;
        if dir == FORWARD {
            if trunc_line || State.get() & MODE_INSERT != 0 {
                let split = Split {
                    lnum,
                    mincol,
                    less_cols,
                    less_cols_off,
                };
                // The buffer owns the line now.
                let line = core::mem::take(&mut saved_line);
                truncate_old_line(line, flags, trunc_line, split, did_append);
                did_append = false;
            }
            // Put the cursor on the new line. `old_cursor`, not
            // `w_cursor`: a scroll above may have moved the latter.
            Win::current().w_cursor.lnum = old_cursor.lnum + 1;
        }
        if did_append {
            let cb = Buf::current();
            let at = Win::current().w_cursor.lnum;
            let extra = BCount::try_from(cb.lines().line_len(at)).unwrap_or(0);
            extmark_splice(cb, at - 1, 0, 0, 0, 0, 1, 0, 1 + extra, kExtmarkUndo);
            changed_lines(cb, at, 0, at, 1, true);
        }
        drop(splice);

        Win::current().w_cursor.col = newcol;
        Win::current().w_cursor.coladd = 0;

        reindent_new_line(has_leader, do_cindent);

        if State.get() & VREPLACE_FLAG != 0 {
            // Take what ended up on the new line back off, put the
            // original line back, and insert the new text character by
            // character so that each replaced byte reaches the replace
            // stack.
            let at = Win::current().w_cursor.lnum;
            let new_text = Lines::current().line(at).to_vec();
            if let Some(next_line) = next_line.take() {
                let _ = ml_replace_owned(at, next_line);
            }
            Win::current().w_cursor.col = 0;
            Win::current().w_cursor.coladd = 0;
            ins_bytes(&new_text); // calls changed_bytes()
        }
        retval = true;
    }

    Buf::current().b_p_pi = saved_pi;
    cmdmod_set_flags(old_cmod_flags);
    retval
}
