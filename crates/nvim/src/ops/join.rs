//! `J` and `gJ` -- joining lines.
//!
//! [`do_join`] builds the joined line in **one** allocation, which is why it
//! is two passes over the same lines. The first ([`measure_join`]) walks
//! forwards and only measures: how much of each line survives once
//! [`skip_comment`] has trimmed a comment leader, and how many spaces go in
//! each seam. The second ([`assemble_join`]) walks *backwards* and copies,
//! filling the buffer from its end -- which is also the order marks have to
//! move in, since each line's marks land at a column that depends on where the
//! lines after it ended up.
//!
//! The seam width is the fiddly part: normally one space, two after `.`, `?`
//! or `!` under 'joinspaces', none before `)` or after a TAB, none at all for
//! `gJ`, and none between two multi-byte characters under the
//! 'formatoptions' `M`/`B` rules -- which exist because CJK text has no spaces
//! between words and a join must not invent one.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::guard::Suppress;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::option::cpo_has;
use crate::types::{CpoFlag, Failed, FoFlag, NUL};

/// Where the text of a line starts once its comment leader is skipped, and
/// whether the line ends *inside* an unclosed comment.
///
/// `process` false only answers the second question and leaves `line` alone;
/// that is how the caller decides whether the *next* line's leader may be
/// removed at all, since a leader is only noise when the line before it was a
/// comment too.
///
/// # Safety
/// `line` must be a NUL-terminated string; `is_comment` must be writable.
pub unsafe fn skip_comment(
    mut line: *mut c_char,
    process: bool,
    include_space: bool,
    is_comment: *mut bool,
) -> *mut c_char {
    // SAFETY: the caller's promise -- `line` is NUL-terminated and
    // `is_comment` writable, and every flag string below is one the comment
    // parser just handed back.
    let mut comment_flags: *mut c_char = ::core::ptr::null_mut();
    let leader_offset = unsafe { get_last_leader_offset(line, &raw mut comment_flags) };

    unsafe { *is_comment = false };
    if leader_offset != -1 {
        // Does the line end with an unclosed comment? It does unless the
        // last leader's flags carry COM_END.
        comment_flags = unsafe { skip_to_end_or_colon(comment_flags) };
        if unsafe { *comment_flags } as c_int != COM_END {
            unsafe { *is_comment = true };
        }
    }

    if !process {
        return line;
    }

    let flagsp = &raw mut comment_flags;
    let lead_len = unsafe { get_leader_len(line, flagsp, false, include_space) };
    if lead_len == 0 {
        return line;
    }

    // A colon means this is not the closing part of a three-part comment.
    // Those are left alone: removing them would be annoying.
    comment_flags = unsafe { skip_to_end_or_colon(comment_flags) };
    let flag = unsafe { *comment_flags } as c_int;
    if flag == ':' as c_int || flag == NUL {
        line = unsafe { line.offset(lead_len as isize) };
    }
    line
}

/// Walk a 'comments' flag string to its `COM_END`, its colon, or its end,
/// whichever comes first.
///
/// # Safety
/// `flags` must be a NUL-terminated string.
unsafe fn skip_to_end_or_colon(mut flags: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's promise -- the walk stops at the string's NUL.
    loop {
        let flag = unsafe { *flags } as c_int;
        if flag == 0 || flag == COM_END || flag == ':' as c_int {
            return flags;
        }
        flags = unsafe { flags.offset(1) };
    }
}

/// What [`measure_join`] worked out and [`assemble_join`] replays.
struct JoinPlan {
    /// Spaces to put in front of line `t`; `count` entries, allocated by the
    /// caller.
    spaces: *mut c_char,
    /// Bytes of comment leader skipped on line `t`, or null when
    /// 'formatoptions' does not have `j`; `count` entries.
    comments: *mut c_int,
    /// Length of the joined line, spaces included.
    sumsize: c_int,
    /// Length of the *last* line joined, after skipping.
    currsize: c_int,
    /// The last line joined, after skipping its leader and white space.
    curr: *mut c_char,
    /// The last line joined, as `ml_get` answered it.
    curr_start: *mut c_char,
}

impl JoinPlan {
    /// Spaces to put in front of line `t`.
    ///
    /// The two arrays are `count` entries each and `t` never leaves that
    /// range, which is what makes these three safe -- the raw pointers stay
    /// so that the walk is an index rather than a bounds check.
    #[inline(always)]
    fn spaces_at(&self, t: linenr_T) -> c_int {
        // SAFETY: `t` is below `count`, the length both arrays were made at.
        c_int::from(unsafe { *self.spaces.offset(t as isize) })
    }

    /// One more space in front of line `t`.
    #[inline(always)]
    fn add_space(&mut self, t: linenr_T) {
        // SAFETY: as [`JoinPlan::spaces_at`].
        unsafe { *self.spaces.offset(t as isize) += 1 };
    }

    /// Bytes of comment leader skipped on line `t`; only with `j` in
    /// 'formatoptions', where `comments` is non-null.
    #[inline(always)]
    fn comment_at(&self, t: linenr_T) -> c_int {
        // SAFETY: as [`JoinPlan::spaces_at`].
        unsafe { *self.comments.offset(t as isize) }
    }

    /// Record the bytes of comment leader skipped on line `t`.
    #[inline(always)]
    fn set_comment(&mut self, t: linenr_T, len: c_int) {
        // SAFETY: as [`JoinPlan::spaces_at`].
        unsafe { *self.comments.offset(t as isize) = len };
    }
}

/// `J` and `gJ`: join `count` lines from the cursor.
///
/// `insert_space` is the difference between them: `J` puts a space in each
/// seam, `gJ` joins the text as it stands. `use_formatoptions` is false for
/// the callers -- backspace over a line break, and the charwise delete in
/// `op_delete` -- that must not have comment leaders removed underneath them.
/// `setmark` false leaves `'[`/`']` to the caller.
///
/// # Safety
/// The cursor line plus `count - 1` must exist in the current buffer.
pub unsafe fn do_join(
    count: size_t,
    insert_space: bool,
    save_undo: bool,
    use_formatoptions: bool,
    setmark: bool,
) -> Result<(), Failed> {
    debug_assert!(count >= 1);
    // SAFETY: the caller's promise -- the cursor line plus `count - 1` exist.
    // The two arrays are `count` entries each, which is what the walks index.
    let remove_comments = use_formatoptions && has_format_option(FoFlag::REMOVE_COMS);

    let above = cur_win().w_cursor.lnum - 1;
    let past = cur_win().w_cursor.lnum + count as linenr_T;
    if save_undo {
        u_save(above, past)?;
    }

    // The per-line space counts are wanted twice: to size the one
    // allocation the joined line goes in, and to place each line in it.
    let mut plan = JoinPlan {
        spaces: unsafe { xcalloc(count, 1) } as *mut c_char,
        comments: if remove_comments {
            let n = ::core::mem::size_of::<c_int>();
            unsafe { xcalloc(count, n) as *mut c_int }
        } else {
            ::core::ptr::null_mut()
        },
        sumsize: 0,
        currsize: 0,
        curr: ::core::ptr::null_mut(),
        curr_start: ::core::ptr::null_mut(),
    };

    let ret = if measure_join(count, insert_space, setmark, &mut plan).is_err() {
        Err(Failed)
    } else {
        assemble_join(count, insert_space, setmark, &mut plan);
        Ok(())
    };

    unsafe { xfree(plan.spaces as *mut c_void) };
    if remove_comments {
        unsafe { xfree(plan.comments as *mut c_void) };
    }
    ret
}

/// First pass: measure the joined line without moving anything.
///
/// Answers `Err` when the user interrupted it, which is why the walk calls
/// `line_breakcheck` -- a join can be over a very large count.
///
/// `plan.spaces` (and `plan.comments`, when non-null) must have `count`
/// entries; the cursor line plus `count - 1` must exist.
fn measure_join(
    count: size_t,
    insert_space: bool,
    setmark: bool,
    plan: &mut JoinPlan,
) -> Result<(), Failed> {
    // The last character of the line before, and the one before it: the
    // seam rules below are all about those two.
    let mut endcurr1 = NUL;
    let mut endcurr2 = NUL;
    let mut prev_was_comment = false;

    // SAFETY: every line the walk reaches exists (the caller's promise), so
    // `ml_get` answers a live NUL-terminated line, and `plan.curr` stays
    // inside the line `plan.curr_start` begins.
    for t in 0..count as linenr_T {
        plan.curr_start = ml_get(cur_win().w_cursor.lnum + t);
        plan.curr = plan.curr_start;

        if t == 0 && setmark && !cmdmod_has(CmdModFlags::LOCKMARKS) {
            let mut buf = cur_win().buffer();
            buf.b_op_start.lnum = cur_win().w_cursor.lnum;
            buf.b_op_start.col = unsafe { cstr::bytes_at(plan.curr) }.len() as colnr_T;
        }

        if !plan.comments.is_null() {
            // The leader is only noise when the line before was a comment
            // too; otherwise just ask whether *this* line is one.
            let was = &raw mut prev_was_comment;
            if t > 0 && prev_was_comment {
                let new_curr = unsafe { skip_comment(plan.curr, true, insert_space, was) };
                let skipped = unsafe { new_curr.offset_from(plan.curr) } as c_int;
                plan.set_comment(t, skipped);
                plan.curr = new_curr;
            } else {
                plan.curr = unsafe { skip_comment(plan.curr, false, insert_space, was) };
            }
        }

        if insert_space && t > 0 {
            plan.curr = unsafe { skipwhite(plan.curr) };
            let at = unsafe { *plan.curr } as c_int;
            if at != NUL
                && at != ')' as c_int
                && plan.sumsize != 0
                && endcurr1 != TAB
                // 'formatoptions' M: no space between two multi-byte
                // characters. B: no space if either side is a character
                // that eats one.
                && (!has_format_option(FoFlag::MBYTE_JOIN)
                    || (unsafe { utf_ptr2char(plan.curr) } < 0x100 && endcurr1 < 0x100))
                && (!has_format_option(FoFlag::MBYTE_JOIN2)
                    || (unsafe { utf_ptr2char(plan.curr) } < 0x100
                        && !utf_eat_space(endcurr1))
                    || (endcurr1 < 0x100
                        && !unsafe { utf_eat_space(utf_ptr2char(plan.curr)) }))
            {
                if endcurr1 == ' ' as c_int {
                    // The line already ends in a space; look one further
                    // back for the 'joinspaces' test below.
                    endcurr1 = endcurr2;
                } else {
                    plan.add_space(t);
                }
                // 'joinspaces': two spaces after the end of a sentence.
                if p_js.get() != 0
                    && (endcurr1 == '.' as c_int
                        || endcurr1 == '?' as c_int
                        || endcurr1 == '!' as c_int)
                {
                    plan.add_space(t);
                }
            }
        }

        let added = plan.spaces_at(t);
        if t > 0 && curbuf_splice_pending.get() == 0 {
            let removed = unsafe { plan.curr.offset_from(plan.curr_start) } as colnr_T;
            let row = cur_win().w_cursor.lnum as c_int - 1;
            let (old, new) = ((removed + 1) as bcount_t, added as bcount_t);
            let op = kExtmarkUndo;
            unsafe {
                extmark_splice(
                    curbuf.get(),
                    row,
                    plan.sumsize,
                    1,
                    removed,
                    old,
                    0,
                    added,
                    new,
                    op,
                )
            };
        }

        plan.currsize = unsafe { cstr::bytes_at(plan.curr) }.len() as c_int;
        plan.sumsize += plan.currsize + added;

        endcurr1 = NUL;
        endcurr2 = NUL;
        if insert_space && plan.currsize > 0 {
            let mut cend = unsafe { plan.curr.offset(plan.currsize as isize) };
            cend = unsafe { mb_ptr_back(plan.curr, cend) };
            endcurr1 = unsafe { utf_ptr2char(cend) };
            if cend > plan.curr {
                cend = unsafe { mb_ptr_back(plan.curr, cend) };
                endcurr2 = unsafe { utf_ptr2char(cend) };
            }
        }

        line_breakcheck();
        if got_int.get() {
            return Err(Failed);
        }
    }
    Ok(())
}

/// Upstream's `MB_PTR_BACK`: step `p` back over the character in front of it.
///
/// # Safety
/// `line` must be the start of the string `p` points into, and `p` must be
/// past it.
unsafe fn mb_ptr_back(line: *const c_char, p: *mut c_char) -> *mut c_char {
    unsafe { p.offset(-(utf_head_off(line, p.offset(-1)) as isize + 1)) }
}

/// Second pass: build the joined line and move the marks onto it.
///
/// Walks *backwards*, filling the one allocation from its end, because each
/// line's column offset in the result depends on everything after it. The
/// forward pass left `plan.curr`/`currsize` on the last line, which is where
/// this one starts.
///
/// `plan` must be as [`measure_join`] left it.
fn assemble_join(count: size_t, insert_space: bool, setmark: bool, plan: &mut JoinPlan) {
    // SAFETY: `plan.sumsize` is what `measure_join` counted for exactly the
    // lines copied back here, so `newp` has room for all of them; every line
    // the backwards walk asks for is one the forwards walk already read.
    let last = count as linenr_T - 1;
    // The column the last line starts at, for the cursor below.
    let col = plan.sumsize - plan.currsize - plan.spaces_at(last);

    let newp_len = plan.sumsize as size_t;
    let newp = unsafe { xmallocz(newp_len) } as *mut c_char;
    let mut cend = unsafe { newp.offset(plan.sumsize as isize) };

    // The four edits below are one splice as far as extmarks and the
    // buffer-update RPC are concerned.
    let splice = Suppress::splice();

    let mut t = last;
    loop {
        let spaces_t = plan.spaces_at(t);
        // Where the mark adjustment below wants the line to have landed.
        let (at, spaces_removed) = unsafe {
            cend = cend.offset(-(plan.currsize as isize));
            let n = plan.currsize as size_t;
            cend.cast::<u8>().copy_from(plan.curr.cast(), n);
            if spaces_t > 0 {
                cend = cend.offset(-(spaces_t as isize));
                cend.cast::<u8>().write_bytes(b' ', spaces_t as size_t);
            }
            let removed = (plan.curr.offset_from(plan.curr_start) - spaces_t as isize) as c_int;
            (
                (cend.offset_from(newp) - removed as isize) as colnr_T,
                removed,
            )
        };

        // Marks move from each deleted line onto the joined one. Not Vi
        // compatible -- Vi deletes them -- but better. If more spaces are
        // deleted than added, a mark inside them moves no further than
        // what was added.
        let lnum = cur_win().w_cursor.lnum + t;
        unsafe { mark_col_adjust(lnum, 0, -t, at, spaces_removed) };

        if t == 0 {
            break;
        }

        plan.curr_start = ml_get(cur_win().w_cursor.lnum + t - 1);
        plan.curr = plan.curr_start;
        if !plan.comments.is_null() {
            let skipped = plan.comment_at(t - 1);
            plan.curr = unsafe { plan.curr.offset(skipped as isize) };
        }
        if insert_space && t > 1 {
            plan.curr = unsafe { skipwhite(plan.curr) };
        }
        plan.currsize = unsafe { cstr::bytes_at(plan.curr) }.len() as c_int;
        t -= 1;
    }

    let _ = unsafe { ml_replace_len(cur_win().w_cursor.lnum, newp, newp_len, false) };

    if setmark && !cmdmod_has(CmdModFlags::LOCKMARKS) {
        let mut buf = cur_win().buffer();
        buf.b_op_end.lnum = cur_win().w_cursor.lnum;
        buf.b_op_end.col = plan.sumsize;
    }

    // Only the first line's change is reported here; `del_lines` reports
    // the lines it deletes.
    let (lnum, next) = (cur_win().w_cursor.lnum, cur_win().w_cursor.lnum + 1);
    changed_lines(cur_buf(), lnum, plan.currsize, next, 0, true);

    // Delete the following lines with the cursor moved there briefly.
    // `del_lines` may move it up again if the last line went, so the line
    // number is kept.
    let joined_lnum = cur_win().w_cursor.lnum;
    cur_win().w_cursor.lnum += 1;
    unsafe { del_lines(count as linenr_T - 1, false) };
    cur_win().w_cursor.lnum = joined_lnum;
    drop(splice);
    cur_buf().deleted_bytes2 = 0;

    // 'cpoptions' `q`: Vi puts the cursor at the column of the *first*
    // join, Vim at the column of the last.
    cur_win().w_cursor.col = if cpo_has(CpoFlag::JOINCOL) {
        plan.currsize
    } else {
        col
    };
    check_cursor_col(unsafe { Win::current() });
    cur_win().w_cursor.coladd = 0;
    cur_win().w_set_curswant = true;
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
