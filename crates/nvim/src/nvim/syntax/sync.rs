//! Finding a line whose syntax state is known — `:syntax sync`.
//!
//! [`syn_sync`] answers "where can parsing safely start for line N", which is
//! what keeps highlighting a line in the middle of a big file from costing a
//! parse of the whole file. Every strategy `:syntax sync` offers lives here:
//! a fixed number of lines back, `fromstart`, a C comment scan, line
//! continuations, and the `grouphere`/`groupthere` sync patterns.
//! [`syn_cmd_sync`] is the command that configures them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

#[allow(unused_imports)]
use super::*;

/// Find a synchronisation point for line `start_lnum`, setting `current_lnum`
/// and the current state to it.
///
/// One of three methods, in this order: search backwards for the end of a
/// C comment, search backwards for the `:syntax sync match` patterns, or simply
/// start a given number of lines above.
///
/// `last_valid` is the last cached state before `start_lnum` that is still
/// trustworthy; running into it during the backward scan ends the search.
pub(crate) unsafe fn syn_sync(wp: *mut win_T, start_lnum: linenr_T, last_valid: *mut synstate_T) {
    unsafe {
        // Clear any current state that might be hanging around.
        invalidate_current_state();

        let start_lnum = sync_backoff(start_lnum);
        current_lnum.set(start_lnum);

        let flags = (*syn_block.get()).b_syn_sync_flags;
        if flags & SF_CCOMMENT != 0 {
            sync_by_ccomment(wp, start_lnum);
        } else if flags & SF_MATCH != 0 {
            sync_by_match(start_lnum, last_valid);
        }
        validate_current_state();
    }
}

/// How far above `start_lnum` parsing starts by default.
///
/// At least "minlines" back, but further, so that scrolling backwards does not
/// resync on every line: it then resyncs one line in N, where N is minlines
/// times 1.5 -- or times 2 when minlines is small. Watch out for overflow when
/// minlines is MAXLNUM.
unsafe fn sync_backoff(start_lnum: linenr_T) -> linenr_T {
    unsafe {
        let minlines = (*syn_block.get()).b_syn_sync_minlines;
        if minlines > start_lnum {
            return 1;
        }
        let mut back = if minlines == 1 {
            1
        } else if minlines < 10 {
            minlines * 2
        } else {
            minlines * 3 / 2
        };
        let maxlines = (*syn_block.get()).b_syn_sync_maxlines;
        if maxlines != 0 && back > maxlines {
            back = maxlines;
        }
        if back >= start_lnum {
            1
        } else {
            start_lnum - back
        }
    }
}

/// Search backwards for the end of a C-style comment, and if the start line
/// turns out to be inside one, push the syntax item that defines it.
unsafe fn sync_by_ccomment(wp: *mut win_T, mut start_lnum: linenr_T) {
    unsafe {
        // `find_start_comment` works on the current buffer, so make syn_buf it
        // for a moment.
        let curwin_save = curwin.get();
        curwin.set(wp);
        let curbuf_save = curbuf.get();
        curbuf.set(syn_buf.get());

        // Skip lines that end in a backslash.
        while start_lnum > 1 {
            let l = ml_get(start_lnum - 1);
            if *l as c_int == NUL
                || *l.offset(ml_get_len(start_lnum - 1) as isize - 1) as c_int != '\\' as c_int
            {
                break;
            }
            start_lnum -= 1;
        }
        current_lnum.set(start_lnum);

        // Set the cursor to the start of the search.
        let cursor_save = (*wp).w_cursor;
        (*wp).w_cursor.lnum = start_lnum;
        (*wp).w_cursor.col = 0;

        // Restrict the search for the end of the comment to "maxlines".
        if !find_start_comment((*syn_block.get()).b_syn_sync_maxlines as c_int).is_null() {
            let mut idx = syn_pattern_count();
            while idx > 0 {
                idx -= 1;
                let spp = syn_pattern(idx);
                if (*spp).sp_syn.id as c_int == (*syn_block.get()).b_syn_sync_id as c_int
                    && (*spp).sp_type as c_int == SPTYPE_START
                {
                    validate_current_state();
                    push_current_state(idx);
                    update_si_attr(state_len() - 1);
                    break;
                }
            }
        }

        (*wp).w_cursor = cursor_save;
        curwin.set(curwin_save);
        curbuf.set(curbuf_save);
    }
}

/// Where a `:syntax sync match` matched, and what it said to do there.
struct SyncPoint {
    /// The sync item's flags -- `grouphere` or `groupthere`.
    flags: c_int,
    /// The pattern index of the group to push, or negative for none.
    match_idx: c_int,
    /// Where the match itself began.
    lnum: linenr_T,
    col: c_int,
    /// Where it ended.
    m_endpos: lpos_T,
}

/// Search backwards, one line at a time, for a `:syntax sync match`.
unsafe fn sync_by_match(start_lnum: linenr_T, last_valid: *mut synstate_T) {
    unsafe {
        let maxlines = (*syn_block.get()).b_syn_sync_maxlines;
        let break_lnum = if maxlines != 0 && start_lnum > maxlines {
            start_lnum - maxlines
        } else {
            0
        };

        let mut end_lnum = start_lnum;
        let mut lnum = start_lnum;
        loop {
            lnum -= 1;
            if lnum <= break_lnum {
                break;
            }

            // This can take a long time: stop when CTRL-C is pressed.
            line_breakcheck();
            if got_int.get() {
                invalidate_current_state();
                current_lnum.set(start_lnum);
                break;
            }
            // Have we run into a saved state stack that is still valid?
            if !last_valid.is_null() && lnum == (*last_valid).sst_lnum {
                load_current_state(last_valid);
                break;
            }
            // Does the previous line have the line-continuation pattern?
            if lnum > 1 && syn_match_linecont(lnum - 1) {
                continue;
            }

            // Start with nothing on the state stack.
            validate_current_state();
            let found = scan_for_sync_point(lnum, end_lnum, start_lnum);

            let Some(found) = found else {
                end_lnum = lnum;
                invalidate_current_state();
                continue;
            };

            // Put the item the sync point named on the state stack. With no
            // item named, leave the stack empty.
            clear_current_state();
            if found.match_idx >= 0 {
                push_current_state(found.match_idx);
                update_si_attr(state_len() - 1);
            }
            if found.flags & HL_SYNC_HERE != 0 {
                // "grouphere": continue from the sync point match to the end of
                // that line, and start parsing at the next one.
                current_lnum.set(found.m_endpos.lnum);
                current_col.set(found.m_endpos.col);
                if state_len() > 0 {
                    let cur_si = state_top();
                    (*cur_si).si_h_startpos.lnum = found.lnum;
                    (*cur_si).si_h_startpos.col = found.col;
                    update_si_end(cur_si, current_col.get(), true);
                    check_keepend();
                }
                syn_finish_line(false);
                current_lnum.set(current_lnum.get() + 1);
            } else {
                // "groupthere": parsing starts at the line we synced for, with
                // the item already in effect.
                current_lnum.set(start_lnum);
            }
            break;
        }

        // Ran into the start of the file, or exceeded the maximum number of
        // lines. (Every `break` above leaves `lnum` above `break_lnum`, so this
        // only fires on the loop's own exhaustion.)
        if lnum <= break_lnum {
            invalidate_current_state();
            current_lnum.set(break_lnum + 1);
        }
    }
}

/// Parse lines `from`..`end_lnum` looking for a sync point, answering the last
/// one found in them.
///
/// The scan does not stop at the first sync point: it keeps looking further on
/// in the line, so the one that wins is the closest to `end_lnum`.
unsafe fn scan_for_sync_point(
    from: linenr_T,
    end_lnum: linenr_T,
    start_lnum: linenr_T,
) -> Option<SyncPoint> {
    unsafe {
        let mut found: Option<SyncPoint> = None;
        current_lnum.set(from);
        while current_lnum.get() < end_lnum {
            syn_start_line();
            loop {
                let had_sync_point = syn_finish_line(true);
                if !had_sync_point || state_len() == 0 {
                    break;
                }
                let cur_si = state_top();
                if (*cur_si).si_m_endpos.lnum > start_lnum {
                    // Ignore a match that reaches past where we started.
                    current_lnum.set(end_lnum);
                    break;
                }
                let (flags, match_idx) = if (*cur_si).si_idx < 0 {
                    (0, KEYWORD_IDX) // cannot happen?
                } else {
                    let spp = syn_pattern((*cur_si).si_idx);
                    ((*spp).sp_flags, (*spp).sp_sync_idx)
                };
                let m_endpos = (*cur_si).si_m_endpos;
                found = Some(SyncPoint {
                    flags,
                    match_idx,
                    lnum: current_lnum.get(),
                    col: current_col.get(),
                    m_endpos,
                });

                // Continue after the match, being aware of a zero-length one.
                if m_endpos.lnum > current_lnum.get() {
                    current_lnum.set(m_endpos.lnum);
                    current_col.set(m_endpos.col);
                    if current_lnum.get() >= end_lnum {
                        break;
                    }
                } else if m_endpos.col > current_col.get() {
                    current_col.set(m_endpos.col);
                } else {
                    current_col.set(current_col.get() + 1);
                }

                // syn_current_attr() skipped the check for an item that ends
                // here; do it now. Be careful not to go past the NUL.
                let prev_col = current_col.get();
                if *syn_getcurline().offset(current_col.get() as isize) as c_int != NUL {
                    current_col.set(current_col.get() + 1);
                }
                check_state_ends();
                current_col.set(prev_col);
            }
            current_lnum.set(current_lnum.get() + 1);
        }
        // A sync point whose item has no flags names nothing to sync on, which
        // upstream spells as `if (found_flags)` -- the zero case falls through
        // to the next line back.
        found.filter(|f| f.flags != 0)
    }
}

/// Save `syn_buf`'s character table and install the one `syntax iskeyword` set.
///
/// A no-op when the syntax has no `iskeyword` of its own, in which case
/// [`restore_chartab`] is a no-op too and the saved buffer is never read.
pub(crate) unsafe fn save_chartab(chartab: *mut c_char) {
    unsafe {
        if (*syn_block.get()).b_syn_isk == empty_string_option.ptr() as *mut c_char {
            return;
        }
        memmove(
            chartab as *mut ::core::ffi::c_void,
            &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *const ::core::ffi::c_void,
            32,
        );
        memmove(
            &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
            &raw mut (*(*syn_win.get()).w_s).b_syn_chartab as *mut uint8_t
                as *const ::core::ffi::c_void,
            32,
        );
    }
}

/// Put back what [`save_chartab`] saved.
pub(crate) unsafe fn restore_chartab(chartab: *mut c_char) {
    unsafe {
        if (*(*syn_win.get()).w_s).b_syn_isk != empty_string_option.ptr() as *mut c_char {
            memmove(
                &raw mut (*syn_buf.get()).b_chartab as *mut uint64_t as *mut ::core::ffi::c_void,
                chartab as *const ::core::ffi::c_void,
                32,
            );
        }
    }
}

/// Does line `lnum` match the `:syntax sync linecont` pattern, i.e. does the
/// line after it continue it?
pub(crate) unsafe fn syn_match_linecont(lnum: linenr_T) -> bool {
    unsafe {
        if (*syn_block.get()).b_syn_linecont_prog.is_null() {
            return false;
        }
        let mut buf_chartab: [c_char; 32] = [0; 32];
        save_chartab(&raw mut buf_chartab as *mut c_char);

        let mut regmatch = regmmatch_T {
            regprog: (*syn_block.get()).b_syn_linecont_prog,
            startpos: [lpos_T { lnum: 0, col: 0 }; 10],
            endpos: [lpos_T { lnum: 0, col: 0 }; 10],
            rmm_matchcol: 0,
            rmm_ic: (*syn_block.get()).b_syn_linecont_ic,
            rmm_maxcol: 0,
        };
        let r = syn_regexec(
            &raw mut regmatch,
            lnum,
            0,
            &raw mut (*syn_block.get()).b_syn_linecont_time,
        );
        (*syn_block.get()).b_syn_linecont_prog = regmatch.regprog;

        restore_chartab(&raw mut buf_chartab as *mut c_char);
        r
    }
}

pub(crate) unsafe extern "C" fn syn_cmd_sync(
    mut eap: *mut exarg_T,
    mut _syncing: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg_start: *mut ::core::ffi::c_char = (*eap).arg;
        let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut illegal: bool = false_0 != 0;
        let mut finished: bool = false_0 != 0;
        if ends_excmd(*arg_start as ::core::ffi::c_int) != 0 {
            syn_cmd_list(eap, true_0);
            return;
        }
        while ends_excmd(*arg_start as ::core::ffi::c_int) == 0 {
            let mut arg_end: *mut ::core::ffi::c_char = skiptowhite(arg_start);
            let mut next_arg: *mut ::core::ffi::c_char = skipwhite(arg_end);
            xfree(key as *mut ::core::ffi::c_void);
            key = vim_strnsave_up(arg_start, arg_end.offset_from(arg_start) as size_t);
            if strcmp(key, b"CCOMMENT\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_flags |= SF_CCOMMENT;
                }
                if ends_excmd(*next_arg as ::core::ffi::c_int) == 0 {
                    arg_end = skiptowhite(next_arg);
                    if (*eap).skip == 0 {
                        (*(*curwin.get()).w_s).b_syn_sync_id =
                            syn_check_group(next_arg, arg_end.offset_from(next_arg) as size_t)
                                as int16_t;
                    }
                    next_arg = skipwhite(arg_end);
                } else if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_id =
                        syn_name2id(b"Comment\0".as_ptr() as *const ::core::ffi::c_char) as int16_t;
                }
            } else if strncmp(
                key,
                b"LINES\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"MINLINES\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"MAXLINES\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                ) == 0 as ::core::ffi::c_int
                || strncmp(
                    key,
                    b"LINEBREAKS\0".as_ptr() as *const ::core::ffi::c_char,
                    10 as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'S' as ::core::ffi::c_int
                {
                    arg_end = key.offset(6 as ::core::ffi::c_int as isize);
                } else if *key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'L' as ::core::ffi::c_int
                {
                    arg_end = key.offset(11 as ::core::ffi::c_int as isize);
                } else {
                    arg_end = key.offset(9 as ::core::ffi::c_int as isize);
                }
                if *arg_end.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '=' as ::core::ffi::c_int
                    || !ascii_isdigit(*arg_end as ::core::ffi::c_int)
                {
                    illegal = true_0 != 0;
                    break;
                } else {
                    let mut n: linenr_T =
                        getdigits_int32(&raw mut arg_end, false_0 != 0, 0 as int32_t);
                    if (*eap).skip == 0 {
                        if *key.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'B' as ::core::ffi::c_int
                        {
                            (*(*curwin.get()).w_s).b_syn_sync_linebreaks = n;
                        } else if *key.offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'A' as ::core::ffi::c_int
                        {
                            (*(*curwin.get()).w_s).b_syn_sync_maxlines = n;
                        } else {
                            (*(*curwin.get()).w_s).b_syn_sync_minlines = n;
                        }
                    }
                }
            } else if strcmp(key, b"FROMSTART\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if (*eap).skip == 0 {
                    (*(*curwin.get()).w_s).b_syn_sync_minlines =
                        MAXLNUM as ::core::ffi::c_int as linenr_T;
                    (*(*curwin.get()).w_s).b_syn_sync_maxlines =
                        0 as ::core::ffi::c_int as linenr_T;
                }
            } else if strcmp(key, b"LINECONT\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                if *next_arg as ::core::ffi::c_int == NUL {
                    illegal = true_0 != 0;
                    break;
                } else if !(*(*curwin.get()).w_s).b_syn_linecont_pat.is_null() {
                    emsg(gettext(
                        b"E403: syntax sync: line continuations pattern specified twice\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    finished = true_0 != 0;
                    break;
                } else {
                    arg_end = skip_regexp(
                        next_arg.offset(1 as ::core::ffi::c_int as isize),
                        *next_arg as ::core::ffi::c_int,
                        true_0,
                    );
                    if *arg_end as ::core::ffi::c_int != *next_arg as ::core::ffi::c_int {
                        illegal = true_0 != 0;
                        break;
                    } else {
                        if (*eap).skip == 0 {
                            (*(*curwin.get()).w_s).b_syn_linecont_pat = xstrnsave(
                                next_arg.offset(1 as ::core::ffi::c_int as isize),
                                (arg_end.offset_from(next_arg) as size_t).wrapping_sub(1 as size_t),
                            );
                            (*(*curwin.get()).w_s).b_syn_linecont_ic =
                                (*(*curwin.get()).w_s).b_syn_ic;
                            let mut cpo_save: *mut ::core::ffi::c_char = p_cpo.get();
                            p_cpo.set(empty_string_option.ptr() as *mut ::core::ffi::c_char);
                            (*(*curwin.get()).w_s).b_syn_linecont_prog =
                                vim_regcomp((*(*curwin.get()).w_s).b_syn_linecont_pat, RE_MAGIC);
                            p_cpo.set(cpo_save);
                            syn_clear_time(&mut (*(*curwin.get()).w_s).b_syn_linecont_time);
                            if (*(*curwin.get()).w_s).b_syn_linecont_prog.is_null() {
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut (*(*curwin.get()).w_s).b_syn_linecont_pat
                                        as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL;
                                let _ = *ptr_;
                                finished = true_0 != 0;
                                break;
                            }
                        }
                        next_arg = skipwhite(arg_end.offset(1 as ::core::ffi::c_int as isize));
                    }
                }
            } else {
                (*eap).arg = next_arg;
                if strcmp(key, b"MATCH\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_match(eap, true_0);
                } else if strcmp(key, b"REGION\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_region(eap, true_0);
                } else if strcmp(key, b"CLEAR\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    syn_cmd_clear(eap, true_0);
                } else {
                    illegal = true_0 != 0;
                }
                finished = true_0 != 0;
                break;
            }
            arg_start = next_arg;
        }
        xfree(key as *mut ::core::ffi::c_void);
        if illegal {
            semsg(
                gettext(b"E404: Illegal arguments: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg_start,
            );
        } else if !finished {
            (*eap).nextcmd = check_nextcmd(arg_start);
            redraw_curbuf_later(UPD_SOME_VALID);
            syn_stack_free_all((*curwin.get()).w_s);
        }
    }
}
