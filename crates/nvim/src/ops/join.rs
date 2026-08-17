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

use core::ffi::{c_char, c_int, c_void};

use super::*;

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
    unsafe {
        let mut comment_flags: *mut c_char = ::core::ptr::null_mut();
        let leader_offset = get_last_leader_offset(line, &raw mut comment_flags);

        *is_comment = false;
        if leader_offset != -1 {
            // Does the line end with an unclosed comment? It does unless the
            // last leader's flags carry COM_END.
            comment_flags = skip_to_end_or_colon(comment_flags);
            if *comment_flags as c_int != COM_END {
                *is_comment = true;
            }
        }

        if !process {
            return line;
        }

        let lead_len = get_leader_len(line, &raw mut comment_flags, false, include_space);
        if lead_len == 0 {
            return line;
        }

        // A colon means this is not the closing part of a three-part comment.
        // Those are left alone: removing them would be annoying.
        comment_flags = skip_to_end_or_colon(comment_flags);
        if *comment_flags as c_int == ':' as c_int || *comment_flags as c_int == NUL {
            line = line.offset(lead_len as isize);
        }
        line
    }
}

/// Walk a 'comments' flag string to its `COM_END`, its colon, or its end,
/// whichever comes first.
///
/// # Safety
/// `flags` must be a NUL-terminated string.
unsafe fn skip_to_end_or_colon(mut flags: *mut c_char) -> *mut c_char {
    unsafe {
        while *flags != 0 {
            if *flags as c_int == COM_END || *flags as c_int == ':' as c_int {
                break;
            }
            flags = flags.offset(1);
        }
        flags
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
) -> c_int {
    unsafe {
        debug_assert!(count >= 1);
        let remove_comments = use_formatoptions && has_format_option(FO_REMOVE_COMS);

        if save_undo
            && u_save(
                (*curwin.get()).w_cursor.lnum - 1,
                (*curwin.get()).w_cursor.lnum + count as linenr_T,
            ) == FAIL
        {
            return FAIL;
        }

        // The per-line space counts are wanted twice: to size the one
        // allocation the joined line goes in, and to place each line in it.
        let mut plan = JoinPlan {
            spaces: xcalloc(count, 1) as *mut c_char,
            comments: if remove_comments {
                xcalloc(count, ::core::mem::size_of::<c_int>()) as *mut c_int
            } else {
                ::core::ptr::null_mut()
            },
            sumsize: 0,
            currsize: 0,
            curr: ::core::ptr::null_mut(),
            curr_start: ::core::ptr::null_mut(),
        };

        let ret = if measure_join(count, insert_space, setmark, &mut plan) == FAIL {
            FAIL
        } else {
            assemble_join(count, insert_space, setmark, &mut plan);
            OK
        };

        xfree(plan.spaces as *mut c_void);
        if remove_comments {
            xfree(plan.comments as *mut c_void);
        }
        ret
    }
}

/// First pass: measure the joined line without moving anything.
///
/// Answers `FAIL` when the user interrupted it, which is why the walk calls
/// `line_breakcheck` -- a join can be over a very large count.
///
/// # Safety
/// `plan.spaces` (and `plan.comments`, when non-null) must have `count`
/// entries; the cursor line plus `count - 1` must exist.
unsafe fn measure_join(
    count: size_t,
    insert_space: bool,
    setmark: bool,
    plan: &mut JoinPlan,
) -> c_int {
    unsafe {
        // The last character of the line before, and the one before it: the
        // seam rules below are all about those two.
        let mut endcurr1 = NUL;
        let mut endcurr2 = NUL;
        let mut prev_was_comment = false;

        for t in 0..count as linenr_T {
            plan.curr_start = ml_get((*curwin.get()).w_cursor.lnum + t);
            plan.curr = plan.curr_start;

            if t == 0 && setmark && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
                (*(*curwin.get()).w_buffer).b_op_start.lnum = (*curwin.get()).w_cursor.lnum;
                (*(*curwin.get()).w_buffer).b_op_start.col = strlen(plan.curr) as colnr_T;
            }

            if !plan.comments.is_null() {
                // The leader is only noise when the line before was a comment
                // too; otherwise just ask whether *this* line is one.
                if t > 0 && prev_was_comment {
                    let new_curr =
                        skip_comment(plan.curr, true, insert_space, &raw mut prev_was_comment);
                    *plan.comments.offset(t as isize) = new_curr.offset_from(plan.curr) as c_int;
                    plan.curr = new_curr;
                } else {
                    plan.curr =
                        skip_comment(plan.curr, false, insert_space, &raw mut prev_was_comment);
                }
            }

            if insert_space && t > 0 {
                plan.curr = skipwhite(plan.curr);
                if *plan.curr as c_int != NUL
                    && *plan.curr as c_int != ')' as c_int
                    && plan.sumsize != 0
                    && endcurr1 != TAB
                    // 'formatoptions' M: no space between two multi-byte
                    // characters. B: no space if either side is a character
                    // that eats one.
                    && (!has_format_option(FO_MBYTE_JOIN)
                        || (utf_ptr2char(plan.curr) < 0x100 && endcurr1 < 0x100))
                    && (!has_format_option(FO_MBYTE_JOIN2)
                        || (utf_ptr2char(plan.curr) < 0x100 && !utf_eat_space(endcurr1))
                        || (endcurr1 < 0x100 && !utf_eat_space(utf_ptr2char(plan.curr))))
                {
                    if endcurr1 == ' ' as c_int {
                        // The line already ends in a space; look one further
                        // back for the 'joinspaces' test below.
                        endcurr1 = endcurr2;
                    } else {
                        *plan.spaces.offset(t as isize) += 1;
                    }
                    // 'joinspaces': two spaces after the end of a sentence.
                    if p_js.get() != 0
                        && (endcurr1 == '.' as c_int
                            || endcurr1 == '?' as c_int
                            || endcurr1 == '!' as c_int)
                    {
                        *plan.spaces.offset(t as isize) += 1;
                    }
                }
            }

            if t > 0 && curbuf_splice_pending.get() == 0 {
                let removed = plan.curr.offset_from(plan.curr_start) as colnr_T;
                extmark_splice(
                    curbuf.get(),
                    (*curwin.get()).w_cursor.lnum as c_int - 1,
                    plan.sumsize,
                    1,
                    removed,
                    (removed + 1) as bcount_t,
                    0,
                    colnr_T::from(*plan.spaces.offset(t as isize)),
                    *plan.spaces.offset(t as isize) as bcount_t,
                    kExtmarkUndo,
                );
            }

            plan.currsize = strlen(plan.curr) as c_int;
            plan.sumsize += plan.currsize + c_int::from(*plan.spaces.offset(t as isize));

            endcurr1 = NUL;
            endcurr2 = NUL;
            if insert_space && plan.currsize > 0 {
                let mut cend = plan.curr.offset(plan.currsize as isize);
                cend = mb_ptr_back(plan.curr, cend);
                endcurr1 = utf_ptr2char(cend);
                if cend > plan.curr {
                    cend = mb_ptr_back(plan.curr, cend);
                    endcurr2 = utf_ptr2char(cend);
                }
            }

            line_breakcheck();
            if got_int.get() {
                return FAIL;
            }
        }
        OK
    }
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
/// # Safety
/// `plan` must be as [`measure_join`] left it.
unsafe fn assemble_join(count: size_t, insert_space: bool, setmark: bool, plan: &mut JoinPlan) {
    unsafe {
        // The column the last line starts at, for the cursor below.
        let col =
            plan.sumsize - plan.currsize - c_int::from(*plan.spaces.offset(count as isize - 1));

        let newp_len = plan.sumsize as size_t;
        let newp = xmallocz(newp_len) as *mut c_char;
        let mut cend = newp.offset(plan.sumsize as isize);

        // The four edits below are one splice as far as extmarks and the
        // buffer-update RPC are concerned.
        *curbuf_splice_pending.ptr() += 1;

        let mut t = count as linenr_T - 1;
        loop {
            cend = cend.offset(-(plan.currsize as isize));
            memmove(
                cend as *mut c_void,
                plan.curr as *const c_void,
                plan.currsize as size_t,
            );
            let spaces_t = c_int::from(*plan.spaces.offset(t as isize));
            if spaces_t > 0 {
                cend = cend.offset(-(spaces_t as isize));
                memset(cend as *mut c_void, ' ' as c_int, spaces_t as size_t);
            }

            // Marks move from each deleted line onto the joined one. Not Vi
            // compatible -- Vi deletes them -- but better. If more spaces are
            // deleted than added, a mark inside them moves no further than
            // what was added.
            let spaces_removed =
                (plan.curr.offset_from(plan.curr_start) - spaces_t as isize) as c_int;
            mark_col_adjust(
                (*curwin.get()).w_cursor.lnum + t,
                0,
                -t,
                (cend.offset_from(newp) - spaces_removed as isize) as colnr_T,
                spaces_removed,
            );

            if t == 0 {
                break;
            }

            plan.curr_start = ml_get((*curwin.get()).w_cursor.lnum + t - 1);
            plan.curr = plan.curr_start;
            if !plan.comments.is_null() {
                plan.curr = plan
                    .curr
                    .offset(*plan.comments.offset((t - 1) as isize) as isize);
            }
            if insert_space && t > 1 {
                plan.curr = skipwhite(plan.curr);
            }
            plan.currsize = strlen(plan.curr) as c_int;
            t -= 1;
        }

        ml_replace_len((*curwin.get()).w_cursor.lnum, newp, newp_len, false);

        if setmark && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
            (*(*curwin.get()).w_buffer).b_op_end.lnum = (*curwin.get()).w_cursor.lnum;
            (*(*curwin.get()).w_buffer).b_op_end.col = plan.sumsize;
        }

        // Only the first line's change is reported here; `del_lines` reports
        // the lines it deletes.
        changed_lines(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum,
            plan.currsize,
            (*curwin.get()).w_cursor.lnum + 1,
            0,
            true,
        );

        // Delete the following lines with the cursor moved there briefly.
        // `del_lines` may move it up again if the last line went, so the line
        // number is kept.
        let joined_lnum = (*curwin.get()).w_cursor.lnum;
        (*curwin.get()).w_cursor.lnum += 1;
        del_lines(count as linenr_T - 1, false);
        (*curwin.get()).w_cursor.lnum = joined_lnum;
        *curbuf_splice_pending.ptr() -= 1;
        (*curbuf.get()).deleted_bytes2 = 0;

        // 'cpoptions' `q`: Vi puts the cursor at the column of the *first*
        // join, Vim at the column of the last.
        (*curwin.get()).w_cursor.col = if !vim_strchr(p_cpo.get(), CPO_JOINCOL).is_null() {
            plan.currsize
        } else {
            col
        };
        check_cursor_col(curwin.get());
        (*curwin.get()).w_cursor.coladd = 0;
        (*curwin.get()).w_set_curswant = true_0;
    }
}
