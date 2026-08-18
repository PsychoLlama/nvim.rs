//! Building one replacement, and putting the rebuilt line in the buffer.
//!
//! The replacement is produced twice: [`build_replacement`] first calls
//! `vim_regsub_multi` with no destination, purely to learn how long the
//! result will be, then again to write it -- the "measure or write" shape,
//! and the reason `\=` expressions are evaluated twice per match.  Both
//! passes run under `textlock`, so the expression cannot change the text
//! under the match.
//!
//! [`commit_line`] is the other half: the rebuilt line only reaches the
//! buffer once there is no further match on it, because replacing it earlier
//! would change what the pattern sees.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::exec::{Sub, SubArgs, save_undo_once};
use super::{sub_grow_buf, subflags};
use crate::change::{appended_lines, changed_bytes, deleted_lines};
use crate::ex_cmds::{
    CAR, LineData, NUL, OK, REGSUB_BACKSLASH, REGSUB_COPY, REGSUB_MAGIC, kExtmarkNOOP, kExtmarkUndo,
};
use crate::ex_eval::aborting;
use crate::extmark::extmark_splice;
use crate::main::{curbuf, curwin, sandbox, sub_nsubs, textlock};
use crate::mark::mark_adjust;
use crate::mbyte::utfc_ptr2len;
use crate::memline::{ml_append, ml_delete, ml_get, ml_replace};
use crate::memory::xfree;
use crate::option::magic_isset;
use crate::os::cshim::memmove;
use crate::pos::MAXLNUM;
use crate::regexp::vim_regsub_multi;
use crate::types::{bcount_t, colnr_T, linenr_T, lpos_T, size_t};
use crate::undo::{u_inssub, u_savedel, u_savesub};
use ::libc::{strcat, strlen};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// The `vim_regsub_multi` flags this command uses, without the copy bit.
fn regsub_flags() -> c_int {
    REGSUB_BACKSLASH as c_int
        | if magic_isset() {
            REGSUB_MAGIC as c_int
        } else {
            0 as c_int
        }
}

/// Turn every CTRL-M in the freshly written replacement into a real line
/// break, and halve the backslashes that protect one.
///
/// That is Vi compatible: a line break can be avoided by preceding the
/// CTRL-M with a backslash, and to insert a backslash they must be doubled
/// in the replacement and are halved here.
///
/// # Safety
/// Main thread; `new_end` must point into `st.new_start`, which must hold the
/// NUL-terminated replacement.
unsafe fn split_carriage_returns(st: &mut Sub, new_end: *mut c_char) {
    let mut p1 = new_end;
    // SAFETY: `p1` walks the NUL-terminated replacement.
    while unsafe { *p1 } as c_int != NUL {
        // SAFETY: as above; the second byte is readable because the first was
        // not the terminator.
        let (here, next) = unsafe { (*p1 as c_int, *p1.add(1) as c_int) };
        if here == '\\' as c_int && next != NUL {
            // Remove the backslash, and correct the byte count that
            // extmark_splice() will be given.
            st.sublen -= 1;
            // SAFETY: moving the tail, terminator included, one byte down.
            unsafe {
                memmove(
                    p1 as *mut c_void,
                    p1.add(1) as *const c_void,
                    strlen(p1.add(1)).wrapping_add(1 as size_t),
                )
            };
        } else if here == CAR {
            // Prepare for undo of the line about to be split.
            // SAFETY: `lnum` is a line of the buffer.
            if unsafe { u_inssub(st.lnum) } == OK {
                // SAFETY: the pieces are all live; the appended line is the
                // text up to the CR, which is then removed from the buffer.
                unsafe {
                    *p1 = NUL as c_char; // truncate up to the CR
                    ml_append(
                        st.lnum - 1 as linenr_T,
                        st.new_start,
                        (p1.offset_from(st.new_start) + 1_isize) as colnr_T,
                        false,
                    );
                    mark_adjust(
                        st.lnum + 1 as linenr_T,
                        MAXLNUM as linenr_T,
                        1 as linenr_T,
                        0 as linenr_T,
                        kExtmarkNOOP,
                    );
                }
                if subflags.with(|flags| flags.do_ask) {
                    // SAFETY: the line was just appended.
                    unsafe { appended_lines(st.lnum - 1 as linenr_T, 1 as linenr_T) };
                } else {
                    if st.first_line == 0 as linenr_T {
                        st.first_line = st.lnum;
                    }
                    st.last_line = st.lnum + 1 as linenr_T;
                }
                // All line numbers increase.
                st.sub_firstlnum += 1;
                st.lnum += 1;
                st.line2 += 1;
                // Move the cursor to the new line, like Vi.
                // SAFETY: the current window is live.
                unsafe { (*curwin.get()).w_cursor.lnum += 1 };
                // Copy the rest.
                // SAFETY: both point into the replacement buffer.
                unsafe {
                    memmove(
                        st.new_start as *mut c_void,
                        p1.add(1) as *const c_void,
                        strlen(p1.add(1)).wrapping_add(1 as size_t),
                    )
                };
                // Restart from the beginning of what is left; the step below
                // puts `p1` back on `new_start`.
                p1 = st.new_start.wrapping_offset(-1);
            }
        } else {
            // SAFETY: `p1` is on a non-NUL byte of the replacement.
            p1 = p1.wrapping_add(unsafe { utfc_ptr2len(p1) } as usize - 1);
        }
        p1 = p1.wrapping_add(1);
    }
}

/// Substitute one match into the line being built.
///
/// Returning early is upstream's `goto skip`: the expression failed, the
/// command was aborted, or this is only a `:s///n` count.
///
/// # Safety
/// Main thread; `st` must describe a live match.
pub(super) unsafe fn build_replacement(
    st: &mut Sub,
    _args: &SubArgs,
    current_match: &mut super::super::SubResult,
) {
    st.lnum_start = st.lnum; // save the start lnum
    // SAFETY: the current buffer is live.
    let save_ma = unsafe { (*curbuf.get()).b_p_ma };
    let save_sandbox = sandbox.get();
    if subflags.with(|flags| flags.do_count) {
        // Prevent a function from accidentally changing the buffer.
        // SAFETY: as above.
        unsafe { (*curbuf.get()).b_p_ma = 0 };
        sandbox.set(sandbox.get() + 1);
    }
    // Save the flags for recursion: they can change for e.g.
    // ":s/^/\=execute("s#^##gn")".
    let subflags_save = subflags.get();

    // Disallow changing text or switching window in an expression, and get
    // the length of the substitution part including the NUL.  When it fails
    // sublen is zero.
    textlock.set(textlock.get() + 1);
    // SAFETY: the match and the copied line are live; with a null
    // destination and length 0 this only measures.
    st.sublen = unsafe {
        vim_regsub_multi(
            &raw mut st.regmatch,
            st.sub_firstlnum - st.regmatch.startpos[0].lnum,
            st.sub,
            st.sub_firstline,
            0 as c_int,
            regsub_flags(),
        )
    };
    textlock.set(textlock.get() - 1);

    // If getting the substitute string caused an error, don't do the
    // replacement.  Don't keep flags set by a recursive call.
    subflags.set(subflags_save);
    // SAFETY: main thread.
    if st.sublen == 0 as c_int || aborting() || subflags.with(|flags| flags.do_count) {
        // SAFETY: the current buffer is live.
        unsafe { (*curbuf.get()).b_p_ma = save_ma };
        sandbox.set(save_sandbox);
        return;
    }

    // Need room for the result so far in new_start (not for the first sub in
    // the line), the original text up to the match, the length of the
    // substituted part, and the original text after the match.
    // SAFETY: `sub_firstlnum + nmatch - 1` is a line of the buffer.
    let p1 = unsafe {
        if st.nmatch == 1 as c_int {
            st.sub_firstline
        } else {
            let lastlnum = st.sub_firstlnum + st.nmatch as linenr_T - 1 as linenr_T;
            st.nmatch_tl += st.nmatch as linenr_T - 1 as linenr_T;
            ml_get(lastlnum)
        }
    };
    let copy_len = st.regmatch.startpos[0].col - st.copycol;
    // SAFETY: `p1` is a live line and the buffer is ours to grow.
    let mut new_end = unsafe {
        let needed =
            strlen(p1) as c_int - st.regmatch.endpos[0].col + copy_len + st.sublen + 1 as c_int;
        sub_grow_buf(&mut st.new_start, &mut st.new_start_len, needed)
    };

    // Copy the text up to the part that matched.
    // SAFETY: `copy_len` bytes from `copycol` are inside the copied line, and
    // the growth above made room for them.
    unsafe {
        memmove(
            new_end as *mut c_void,
            st.sub_firstline.add(st.copycol as usize) as *const c_void,
            copy_len as size_t,
        )
    };
    new_end = new_end.wrapping_add(copy_len as usize);

    if st.new_start_len - copy_len < st.sublen {
        st.sublen = st.new_start_len - copy_len - 1 as c_int;
    }

    // Only now can we know where the match will actually start in the new
    // text.
    // SAFETY: both point into the same allocation.
    let start_col = unsafe { new_end.offset_from(st.new_start) } as c_int;
    current_match.start.col = start_col as colnr_T;

    textlock.set(textlock.get() + 1);
    // SAFETY: `new_end` has room for `sublen` bytes, as just computed.
    unsafe {
        vim_regsub_multi(
            &raw mut st.regmatch,
            st.sub_firstlnum - st.regmatch.startpos[0].lnum,
            st.sub,
            new_end,
            st.sublen,
            REGSUB_COPY as c_int | regsub_flags(),
        )
    };
    textlock.set(textlock.get() - 1);
    sub_nsubs.set(sub_nsubs.get() + 1);
    st.did_sub = true;

    // Move the cursor to the start of the line, to avoid it being beyond the
    // end of the line after the substitution.
    // SAFETY: the current window is live.
    unsafe { (*curwin.get()).w_cursor.col = 0 as colnr_T };

    // Remember the next character to be copied.
    st.copycol = st.regmatch.endpos[0].col;

    // SAFETY: the buffer is live.
    unsafe { st.adjust_sub_firstlnum() };

    // TODO(bfredl): this has some robustness issues, look into later.
    let start: lpos_T = st.regmatch.startpos[0];
    let end: lpos_T = st.regmatch.endpos[0];
    let mut replaced_bytes = 0 as bcount_t;
    let mut i = 0 as c_int;
    while i < st.nmatch - 1 as c_int {
        // SAFETY: the lines of a multi-line match are all in the buffer.
        replaced_bytes +=
            unsafe { strlen(ml_get(st.lnum_start + i as linenr_T)) } as bcount_t + 1 as bcount_t;
        i += 1;
    }
    replaced_bytes += (end.col - start.col) as bcount_t;

    // Save the line number before processing newlines.
    let lnum_before_newlines = st.lnum;
    // SAFETY: `new_end` points into the replacement just written.
    unsafe { split_carriage_returns(st, new_end) };

    // SAFETY: the replacement is NUL-terminated.
    let new_endcol = unsafe { strlen(st.new_start) } as colnr_T;
    current_match.end.col = new_endcol;
    current_match.end.lnum = st.lnum;

    let matchcols = end.col
        - if end.lnum == start.lnum {
            start.col
        } else {
            0 as colnr_T
        };
    let subcols = new_endcol
        - if st.lnum == st.lnum_start {
            start_col
        } else {
            0 as c_int
        };
    // SAFETY: the cursor is on a line of the buffer.
    unsafe { save_undo_once(st) };

    // Store the extmark data for this match; the whole batch is sent once the
    // line has been replaced.
    st.line_matches.push(LineData {
        start_col,
        start,
        end,
        matchcols,
        matchbytes: replaced_bytes,
        subcols,
        subbytes: (st.sublen - 1 as c_int) as bcount_t,
        lnum_before: lnum_before_newlines,
        lnum_after: st.lnum,
    });
}

/// Delete the lines a multi-line match consumed: their text has already been
/// appended to the rebuilt line, so the buffer does not need it.
///
/// Answers false when undo could not be saved.
///
/// # Safety
/// Main thread; `st.lnum` must be the rebuilt line.
unsafe fn delete_matched_lines(st: &mut Sub) -> bool {
    st.lnum += 1;
    // SAFETY: the lines below `lnum` are the ones the match spanned.
    if unsafe { u_savedel(st.lnum, st.nmatch_tl) } != OK {
        return false;
    }
    let mut i = 0 as linenr_T;
    while i < st.nmatch_tl {
        // SAFETY: as above.
        unsafe { ml_delete(st.lnum) };
        i += 1;
    }
    // SAFETY: as above.
    unsafe {
        mark_adjust(
            st.lnum,
            st.lnum + st.nmatch_tl - 1 as linenr_T,
            MAXLNUM as linenr_T,
            -st.nmatch_tl,
            kExtmarkNOOP,
        )
    };
    if subflags.with(|flags| flags.do_ask) {
        // SAFETY: as above.
        unsafe { deleted_lines(st.lnum, st.nmatch_tl) };
    }
    st.lnum -= 1;
    st.line2 -= st.nmatch_tl; // the number of lines decreases
    st.nmatch_tl = 0 as linenr_T;
    true
}

/// Put the rebuilt line into the buffer, with the extmark splices the
/// substitutions on it produced.
///
/// Answers false when undo could not be saved, which abandons the line.
///
/// # Safety
/// Main thread; `st.new_start` must hold the rebuilt line.
pub(super) unsafe fn commit_line(st: &mut Sub) -> bool {
    // Copy the rest of the line, the part that didn't match.  "matchcol" has
    // to be adjusted using the end of the line as reference, because the
    // substitute may have changed the number of characters; same for
    // "prev_matchcol".
    // SAFETY: the growth policy left room for the tail, and both strings are
    // NUL-terminated.
    let old_len = unsafe {
        strcat(st.new_start, st.sub_firstline.add(st.copycol as usize));
        strlen(st.sub_firstline) as colnr_T
    };
    st.matchcol = old_len - st.matchcol;
    st.prev_matchcol = old_len - st.prev_matchcol;

    // SAFETY: `lnum` is a line of the buffer.
    if unsafe { u_savesub(st.lnum) } != OK {
        return false;
    }
    // SAFETY: `ml_replace` takes ownership of the rebuilt line.
    unsafe { ml_replace(st.lnum, st.new_start, true) };

    // Call extmark_splice for each match on this line.
    for m in &st.line_matches {
        // SAFETY: the current buffer is live and the data describes it.
        unsafe {
            extmark_splice(
                curbuf.get(),
                m.lnum_before as c_int - 1 as c_int,
                m.start_col as colnr_T,
                m.end.lnum as c_int - m.start.lnum as c_int,
                m.matchcols as colnr_T,
                m.matchbytes,
                m.lnum_after as c_int - m.lnum_before as c_int,
                m.subcols as colnr_T,
                m.subbytes,
                kExtmarkUndo,
            )
        };
    }
    // Reset the match data for the next line.
    st.line_matches.clear();

    if st.nmatch_tl > 0 as linenr_T {
        // SAFETY: the rebuilt line is in the buffer.
        if !unsafe { delete_matched_lines(st) } {
            return false;
        }
    }

    // When asking, undo is saved each time, so the changed flag must be set
    // each time too.
    if subflags.with(|flags| flags.do_ask) {
        // SAFETY: `lnum` is a line of the buffer.
        unsafe { changed_bytes(st.lnum, 0 as colnr_T) };
    } else {
        if st.first_line == 0 as linenr_T {
            st.first_line = st.lnum;
        }
        st.last_line = st.lnum + 1 as linenr_T;
    }

    st.sub_firstlnum = st.lnum;
    // Free the temp buffer; the rebuilt line becomes the old text.
    // SAFETY: our own allocation.  `ml_replace` was told to *copy*, so the
    // rebuilt line is still ours and becomes the old text for the next match.
    unsafe { xfree(st.sub_firstline as *mut c_void) };
    st.sub_firstline = st.new_start;
    st.new_start = ptr::null_mut();
    // SAFETY: the new old-text is NUL-terminated.
    let new_len = unsafe { strlen(st.sub_firstline) } as colnr_T;
    st.matchcol = new_len - st.matchcol;
    st.prev_matchcol = new_len - st.prev_matchcol;
    st.copycol = 0 as colnr_T;
    true
}
