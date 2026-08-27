//! `gq` and `gw`: reflowing whole paragraphs.
//!
//! [`format_lines`] is the engine -- walk the range, decide where each
//! paragraph ends, join it into one line and let `insertchar` re-wrap it --
//! and [`op_format`] the operator around it. [`fex_format`] is the
//! 'formatexpr' escape hatch [`op_formatexpr`] tries first.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win, windows};
use core::ffi::{c_char, c_int, c_long};

use super::*;
use crate::ascii::ascii_isspace;
use crate::change::del_bytes;
use crate::charset::getwhitecols_curline;
use crate::cursor::{
    check_cursor, coladvance, dec_cursor, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr,
};
use crate::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::edit::{BeginlineOpts, beginline, insertchar};
use crate::eval::eval_to_number;
use crate::eval::vars::{set_vim_var_char, set_vim_var_nr, set_vim_var_string};
use crate::ex_docmd::cmdmod_has;
use crate::getchar::beep_flush;
use crate::guard::{Lock, Script};
use crate::indent::{
    get_expr_indent, get_indent, get_indent_lnum, get_lisp_indent, get_number_indent, set_indent,
};
use crate::indent_c::{cindent_on, get_c_indent};
use crate::main::{State, curwin, got_int, p_smd, saved_cursor};
use crate::mark::mark_col_adjust;
use crate::memline::ml_get;
use crate::memory::{xfree, xstrdup};
use crate::message::msgmore;
use crate::ops::{Op, do_join};
use crate::option::was_set_insecurely;
use crate::options::kOptFormatexpr;
use crate::os::cshim::strncmp;
use crate::os::input::line_breakcheck;
use crate::pos::MAXCOL;
use crate::search::check_linecomment;
use crate::state::{MODE_INSERT, MODE_NORMAL};
use crate::types::{
    CmdModFlags, FAIL, INSCHAR_COM_LIST, INSCHAR_DO_COM, INSCHAR_FORMAT, INSCHAR_NO_FEX, NUL,
    OptionSetFlags, Vv, colnr_T, linenr_T, oparg_T, ptrdiff_t, size_t, varnumber_T,
};
use crate::ui::ui_cursor_shape;
use crate::undo::{u_save, u_save_cursor};

/// The `gq` / `gw` operator.
///
/// `keep_cursor` is `gw`: the cursor goes back to where the command was
/// given, adjusted for the lines that were joined and split under it.
///
/// # Safety
/// `oap` must be a live operator argument over the current buffer.
pub(crate) unsafe fn op_format(oap: *mut oparg_T, keep_cursor: bool) {
    // SAFETY: the caller's promise -- a live operator argument.
    let oap = unsafe { Op::new(oap) };
    let mut old_line_count = cur_buf().b_ml.ml_line_count;

    // Put the cursor where the command was given, so `u` can put it back.
    cur_win().w_cursor = oap.cursor_start;
    if u_save(oap.start.lnum - 1, oap.end.lnum + 1) == FAIL {
        return;
    }
    cur_win().w_cursor = oap.start;

    if oap.is_VIsual {
        // When nothing changes, the Visual selection still has to go.
        redraw_curbuf_later(UPD_INVERTED);
    }
    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        // The `'[` mark goes at the start of the formatted area.
        cur_buf().b_op_start = oap.start;
    }
    if keep_cursor {
        saved_cursor.set(oap.cursor_start);
    }

    unsafe { format_lines(oap.line_count, keep_cursor) };

    // Leave the cursor on the first non-blank of the last formatted line.
    // If it moved a line back (`Q}` does that), step forward so `.`
    // carries on with the next lines.
    if oap.end_adjusted && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count {
        cur_win().w_cursor.lnum += 1;
    }
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    old_line_count = cur_buf().b_ml.ml_line_count - old_line_count;
    unsafe { msgmore(old_line_count as c_int) };

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        // The `']` mark goes at the end of it.
        cur_buf().b_op_end = cur_win().w_cursor;
    }
    if keep_cursor {
        cur_win().w_cursor = saved_cursor.get();
        saved_cursor.set(saved_cursor.get().with_lnum(0));
        // Formatting may have made the position invalid.
        check_cursor(unsafe { Win::current() });
    }
    if oap.is_VIsual {
        // `FOR_ALL_WINDOWS_IN_TAB(wp, curtab)`. The macro's tab page test is
        // `curtab == curtab` here, so it always takes the `firstwin` arm --
        // which is what [`windows`] walks.
        for mut wp in windows() {
            if wp.w_old_cursor_lnum != 0 {
                // Lines were inserted or deleted: move the end of the
                // Visual area that has to be redrawn with them.
                if wp.w_old_cursor_lnum > wp.w_old_visual_lnum {
                    wp.w_old_cursor_lnum += old_line_count;
                } else {
                    wp.w_old_visual_lnum += old_line_count;
                }
            }
        }
    }
}

/// `gq` when 'formatexpr' is set. As documented, a non-zero answer from the
/// expression means "I did not handle it", and the internal formatter runs.
///
/// # Safety
/// `oap` must be a live operator argument over the current buffer.
pub(crate) unsafe fn op_formatexpr(oap: *mut oparg_T) {
    // SAFETY: the caller's promise -- a live operator argument.
    let op = unsafe { Op::new(oap) };
    if op.is_VIsual {
        // When nothing changes, the Visual selection still has to go.
        redraw_curbuf_later(UPD_INVERTED);
    }
    if unsafe { fex_format(op.start.lnum, op.line_count as c_long, NUL) } != 0 {
        unsafe { op_format(oap, false) };
    }
}

/// Evaluate 'formatexpr' over `count` lines from `lnum`, with `c` the
/// character about to be inserted (or NUL).
///
/// # Safety
/// There must be a current buffer and window.
pub(crate) unsafe fn fex_format(lnum: linenr_T, count: c_long, c: c_int) -> c_int {
    let use_sandbox =
        unsafe { was_set_insecurely(curwin.get(), kOptFormatexpr, OptionSetFlags::LOCAL) };

    unsafe { set_vim_var_nr(Vv::Lnum, lnum as varnumber_T) };
    unsafe { set_vim_var_nr(Vv::Count, count as varnumber_T) };
    unsafe { set_vim_var_char(c) };

    // Copy it: the option can be changed while it is running.
    let fex = unsafe { xstrdup(cur_buf().b_p_fex) };
    // Errors go against the script that set `'formatexpr'`.
    let script_ctx = Script::context(cur_buf().b_p_script_ctx[kBufOptFormatexpr as usize]);
    let r = {
        let _sandboxed = use_sandbox.then(Lock::sandbox);
        unsafe { eval_to_number(fex, true) as c_int }
    };
    unsafe { set_vim_var_string(Vv::Char, ::core::ptr::null::<c_char>(), -1 as ptrdiff_t) };
    unsafe { xfree(fex as *mut ::core::ffi::c_void) };
    drop(script_ctx);
    r
}

/// The indent the first line of a paragraph gets before it is re-wrapped.
///
/// The very first line formatted keeps whatever indent it has; after that the
/// indent is recomputed by whichever engine is in force, so that a paragraph
/// moved under a different one lines up.
///
/// # Safety
/// There must be a current line.
unsafe fn paragraph_indent(first_line: linenr_T) -> c_int {
    if cur_win().w_cursor.lnum == first_line {
        get_indent()
    } else if cur_buf().b_p_lisp != 0 {
        unsafe { get_lisp_indent() }
    } else if unsafe { cindent_on() } {
        if unsafe { *cur_buf().b_p_inde } as c_int != NUL {
            unsafe { get_expr_indent() }
        } else {
            unsafe { get_c_indent() }
        }
    } else {
        get_indent()
    }
}

/// Join the line below into the paragraph, first taking off whatever the
/// formatting is going to put back: the comment leader, or the extra indent
/// 'formatoptions' `2` gave the second line.
///
/// Answers false when the join failed and the walk has to stop.
///
/// # Safety
/// There must be a current line, and it must be modifiable.
unsafe fn join_next_line(
    next_leader_len: c_int,
    second_indent: c_int,
    line_count: linenr_T,
) -> bool {
    cur_win().w_cursor.lnum += 1;
    cur_win().w_cursor.col = 0;
    if line_count < 0 && u_save_cursor() == FAIL {
        return false;
    }
    let strip = if next_leader_len > 0 {
        next_leader_len
    } else if second_indent > 0 {
        // The "leader" `FoFlag::Q_SECOND` left behind.
        unsafe { getwhitecols_curline() as c_int }
    } else {
        0
    };
    if strip > 0 {
        unsafe { del_bytes(strip as colnr_T, false, false) };
        unsafe { mark_col_adjust(cur_win().w_cursor.lnum, 0, 0, -(strip as colnr_T), 0) };
    }
    cur_win().w_cursor.lnum -= 1;
    if unsafe { do_join(2 as size_t, true, false, false, false) } == FAIL {
        beep_flush();
        return false;
    }
    true
}

/// Reflow `line_count` lines from the cursor; a negative count means "to the
/// end of this paragraph".
///
/// The walk carries the answers about *three* lines at once -- the one above,
/// the current one and the one below -- because a paragraph ends where any of
/// the three stops matching. When it does, the paragraph is one line by then
/// (each pass joins the next one onto it) and `insertchar` re-wraps it;
/// otherwise the next line is joined on and the pass repeats without
/// advancing.
///
/// `avoid_fex` suppresses 'formatexpr', which is how `op_format` avoids
/// re-entering the expression that already declined the job.
///
/// The caller must have saved the first line for undo; the ones after it are
/// saved here.
///
/// # Safety
/// There must be a current line, and it must be modifiable.
pub(crate) unsafe fn format_lines(line_count: linenr_T, avoid_fex: bool) {
    let mut prev_is_end_par = false;
    let mut next_is_start_par = false;
    let mut leader = Leader::NONE;
    let mut next_leader = Leader::NONE;
    let mut advance = true;
    // The indent the paragraph's second line asks for, comment-aware.
    let mut second_indent: c_int = -1;
    let mut first_par_line = true;
    let mut need_set_indent = true;
    let first_line = cur_win().w_cursor.lnum;
    let mut force_format = false;
    let old_state = State.get();

    // The length at which a line is formatted whether or not the
    // paragraph has ended: 3 * 'textwidth'.
    let max_len = unsafe { comp_textwidth(true) } * 3;

    let do_comments = has_format_option(FoFlag::Q_COMS);
    // Format comments with `n` or `2`.
    let mut do_comments_list = false;
    let do_second_indent = has_format_option(FoFlag::Q_SECOND);
    let do_number_indent = has_format_option(FoFlag::Q_NUMBER);
    let do_trail_white = has_format_option(FoFlag::WHITE_PAR);

    // The previous and current lines.
    let mut is_not_par = if cur_win().w_cursor.lnum > 1 {
        unsafe { fmt_check_par(cur_win().w_cursor.lnum - 1, &mut leader, do_comments) }
    } else {
        true
    };
    let mut next_is_not_par =
        unsafe { fmt_check_par(cur_win().w_cursor.lnum, &mut next_leader, do_comments) };
    let mut is_end_par = is_not_par || next_is_not_par;
    if !is_end_par && do_trail_white {
        is_end_par = !unsafe { ends_in_white(cur_win().w_cursor.lnum - 1) };
    }

    cur_win().w_cursor.lnum -= 1;
    let mut count = line_count as c_long;
    while count != 0 && !got_int.get() {
        if advance {
            cur_win().w_cursor.lnum += 1;
            prev_is_end_par = is_end_par;
            is_not_par = next_is_not_par;
            leader = next_leader;
        }

        // The last line to be formatted.
        if count == 1 || cur_win().w_cursor.lnum == cur_buf().b_ml.ml_line_count {
            next_is_not_par = true;
            next_leader = Leader::NONE;
        } else {
            next_is_not_par = unsafe {
                fmt_check_par(cur_win().w_cursor.lnum + 1, &mut next_leader, do_comments)
            };
            if do_number_indent {
                next_is_start_par = unsafe { get_number_indent(cur_win().w_cursor.lnum + 1) } > 0;
            }
        }
        advance = true;
        is_end_par = is_not_par || next_is_not_par || next_is_start_par;
        if !is_end_par && do_trail_white {
            is_end_par = !unsafe { ends_in_white(cur_win().w_cursor.lnum) };
        }

        if is_not_par {
            // Skip lines that are not in a paragraph.
            if line_count < 0 {
                break;
            }
        } else {
            // On a paragraph's first line, look at the second line's
            // indent -- but not for comments or empty lines.
            if first_par_line
                && (do_second_indent || do_number_indent)
                && prev_is_end_par
                && cur_win().w_cursor.lnum < cur_buf().b_ml.ml_line_count
            {
                let no_comment = leader.len == 0 && next_leader.len == 0;
                if do_second_indent
                    && unsafe { *ml_get(cur_win().w_cursor.lnum + 1) } as c_int != NUL
                {
                    if no_comment {
                        second_indent = unsafe { get_indent_lnum(cur_win().w_cursor.lnum + 1) };
                    } else {
                        second_indent = next_leader.len;
                        do_comments_list = true;
                    }
                } else if do_number_indent {
                    // `get_number_indent` is comment-aware itself, so
                    // both of upstream's arms ask it the same thing; only
                    // the list flag differs. It is never cleared again --
                    // one comment-bearing paragraph turns it on for the
                    // rest of the run.
                    second_indent = unsafe { get_number_indent(cur_win().w_cursor.lnum) };
                    if !no_comment {
                        do_comments_list = true;
                    }
                }
            }

            // A change of comment leader ends the paragraph.
            if cur_win().w_cursor.lnum >= cur_buf().b_ml.ml_line_count
                || !unsafe { same_leader(cur_win().w_cursor.lnum, leader, next_leader) }
            {
                // Except when the next line opens a line comment and this
                // one has a line comment after some text: then the
                // paragraph does not really end.
                if next_leader.flags.is_null()
                    || unsafe { strncmp(next_leader.flags, c"://".as_ptr(), 3 as size_t) } != 0
                    || unsafe { check_linecomment(get_cursor_line_ptr()) } == MAXCOL
                {
                    is_end_par = true;
                }
            }

            // At the end of a paragraph, or with a line getting long,
            // format it.
            if is_end_par || force_format {
                if need_set_indent {
                    // Rewrite the first line's indent with the minimal
                    // number of tabs and spaces the options ask for.
                    unsafe { set_indent(paragraph_indent(first_line), SIN_CHANGED as c_int) };
                }

                // Put the cursor on the last non-space.
                State.set(MODE_NORMAL); // don't go past end-of-line
                coladvance(unsafe { Win::current() }, MAXCOL);
                while cur_win().w_cursor.col != 0 && ascii_isspace(gchar_cursor()) {
                    dec_cursor();
                }

                // Format, without 'showmode'.
                State.set(MODE_INSERT); // for open_line()
                let smd_save = p_smd.get();
                p_smd.set(0);
                unsafe {
                    insertchar(
                        NUL,
                        INSCHAR_FORMAT as c_int
                            + if do_comments {
                                INSCHAR_DO_COM as c_int
                            } else {
                                0
                            }
                            + if do_comments && do_comments_list {
                                INSCHAR_COM_LIST as c_int
                            } else {
                                0
                            }
                            + if avoid_fex {
                                INSCHAR_NO_FEX as c_int
                            } else {
                                0
                            },
                        second_indent,
                    )
                };
                State.set(old_state);
                p_smd.set(smd_save);
                // `insertchar` can have run `:normal`, which updates the
                // cursor shape; put it back.
                unsafe { ui_cursor_shape() };

                second_indent = -1;
                // At the end of a paragraph the next one needs its indent
                // set as well.
                need_set_indent = is_end_par;
                if is_end_par {
                    // A negative count means "stop at the end of this
                    // paragraph".
                    if line_count < 0 {
                        break;
                    }
                    first_par_line = true;
                }
                force_format = false;
            }

            // Still in the same paragraph: join the next line on.
            if !is_end_par {
                advance = false;
                if !unsafe { join_next_line(next_leader.len, second_indent, line_count) } {
                    break;
                }
                first_par_line = false;
                // A line getting long is formatted next time round.
                force_format = get_cursor_line_len() > max_len;
            }
        }
        line_breakcheck();
        count -= 1;
    }
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
