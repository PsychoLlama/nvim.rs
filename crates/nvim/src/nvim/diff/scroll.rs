//! Keeping the windows lined up, and moving between changes.
//!
//! `diff_set_topline` is what `'scrollbind'` calls in diff mode: given the
//! partner window's topline and topfill, it computes this one's so that the same
//! change is on the same screen row.  `diff_move_to` is `]c`/`[c`, and
//! `diff_get_corresponding_line` the line-number mapping the two share.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn find_top_diff_block(
    mut thistopdiff: *mut *mut diff_T,
    mut next_adjacent_blocks: *mut *mut diff_T,
    mut fromidx: ::core::ffi::c_int,
    mut topline: ::core::ffi::c_int,
) {
    unsafe {
        let mut topdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut localtopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut topdiffchange: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        topdiff = (*curtab.get()).tp_first_diff;
        while !topdiff.is_null() {
            if localtopdiff.is_null() || topdiffchange != 0 {
                localtopdiff = topdiff;
                topdiffchange = 0 as ::core::ffi::c_int;
            }
            if topline as linenr_T >= (*topdiff).df_lnum[fromidx as usize]
                && topline as linenr_T
                    <= (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize]
            {
                if (*thistopdiff).is_null() {
                    *thistopdiff = localtopdiff;
                }
            }
            if !(!(*topdiff).df_next.is_null()
                && (*(*topdiff).df_next).df_lnum[fromidx as usize]
                    == (*topdiff).df_lnum[fromidx as usize] + (*topdiff).df_count[fromidx as usize])
            {
                topdiffchange = 1 as ::core::ffi::c_int;
                if !(*thistopdiff).is_null() {
                    *next_adjacent_blocks = (*topdiff).df_next;
                    break;
                }
            }
            topdiff = (*topdiff).df_next;
        }
    }
}

unsafe extern "C" fn calculate_topfill_and_topline(
    fromidx: ::core::ffi::c_int,
    toidx: ::core::ffi::c_int,
    from_topline: ::core::ffi::c_int,
    from_topfill: ::core::ffi::c_int,
    mut topfill: *mut ::core::ffi::c_int,
    mut topline: *mut linenr_T,
) {
    unsafe {
        let mut thistopdiff: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut next_adjacent_blocks: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut virtual_lines_passed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        find_top_diff_block(
            &raw mut thistopdiff,
            &raw mut next_adjacent_blocks,
            fromidx,
            from_topline,
        );
        let mut curdif: *mut diff_T = thistopdiff;
        while !curdif.is_null()
            && (*curdif).df_lnum[fromidx as usize] + (*curdif).df_count[fromidx as usize]
                <= from_topline as linenr_T
        {
            virtual_lines_passed += get_max_diff_length(curdif);
            curdif = (*curdif).df_next;
        }
        if curdif != next_adjacent_blocks {
            virtual_lines_passed += (from_topline as linenr_T - (*curdif).df_lnum[fromidx as usize])
                as ::core::ffi::c_int;
        }
        virtual_lines_passed -= from_topfill;
        if virtual_lines_passed < 0 as ::core::ffi::c_int {
            virtual_lines_passed = 0 as ::core::ffi::c_int;
        }
        let mut curlinenum_to: ::core::ffi::c_int = if !thistopdiff.is_null() {
            (*thistopdiff).df_lnum[toidx as usize] as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        let mut virt_lines_left: ::core::ffi::c_int = virtual_lines_passed;
        curdif = thistopdiff;
        while virt_lines_left > 0 as ::core::ffi::c_int
            && !curdif.is_null()
            && curdif != next_adjacent_blocks
        {
            curlinenum_to += (if (virt_lines_left as linenr_T) < (*curdif).df_count[toidx as usize]
            {
                virt_lines_left as linenr_T
            } else {
                (*curdif).df_count[toidx as usize]
            }) as ::core::ffi::c_int;
            virt_lines_left -= if virt_lines_left < get_max_diff_length(curdif) {
                virt_lines_left
            } else {
                get_max_diff_length(curdif)
            };
            curdif = (*curdif).df_next;
        }
        let mut max_virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut dp: *mut diff_T = thistopdiff;
        while !dp.is_null() {
            if (*dp).df_lnum[toidx as usize] + (*dp).df_count[toidx as usize]
                <= curlinenum_to as linenr_T
            {
                max_virt_lines += get_max_diff_length(dp);
                dp = (*dp).df_next;
            } else {
                if (*dp).df_lnum[toidx as usize] <= curlinenum_to as linenr_T {
                    max_virt_lines += (curlinenum_to as linenr_T - (*dp).df_lnum[toidx as usize])
                        as ::core::ffi::c_int;
                }
                break;
            }
        }
        if diff_flags.get() & DIFF_FILLER != 0 {
            *topfill = max_virt_lines - virtual_lines_passed;
        }
        *topline = curlinenum_to as linenr_T;
    }
}

pub unsafe extern "C" fn diff_set_topline(mut fromwin: *mut win_T, mut towin: *mut win_T) {
    unsafe {
        let mut frombuf: *mut buf_T = (*fromwin).w_buffer;
        let mut fromidx: ::core::ffi::c_int = diff_buf_idx(frombuf, curtab.get());
        if fromidx == DB_COUNT {
            return;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        let mut lnum: linenr_T = (*fromwin).w_topline;
        (*towin).w_topfill = 0 as ::core::ffi::c_int;
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if lnum <= (*dp).df_lnum[fromidx as usize] + (*dp).df_count[fromidx as usize] {
                break;
            }
            dp = (*dp).df_next;
        }
        if dp.is_null() {
            (*towin).w_topline =
                (*(*towin).w_buffer).b_ml.ml_line_count - ((*frombuf).b_ml.ml_line_count - lnum);
        } else {
            let mut toidx: ::core::ffi::c_int = diff_buf_idx((*towin).w_buffer, curtab.get());
            if toidx == DB_COUNT {
                return;
            }
            (*towin).w_topline =
                lnum + ((*dp).df_lnum[toidx as usize] - (*dp).df_lnum[fromidx as usize]);
            if lnum >= (*dp).df_lnum[fromidx as usize] {
                calculate_topfill_and_topline(
                    fromidx,
                    toidx,
                    (*fromwin).w_topline as ::core::ffi::c_int,
                    (*fromwin).w_topfill,
                    &raw mut (*towin).w_topfill,
                    &raw mut (*towin).w_topline,
                );
            }
        }
        (*towin).w_botfill = false_0 != 0;
        if (*towin).w_topline > (*(*towin).w_buffer).b_ml.ml_line_count {
            (*towin).w_topline = (*(*towin).w_buffer).b_ml.ml_line_count;
            (*towin).w_botfill = true_0 != 0;
        }
        if (*towin).w_topline < 1 as linenr_T {
            (*towin).w_topline = 1 as ::core::ffi::c_int as linenr_T;
            (*towin).w_topfill = 0 as ::core::ffi::c_int;
        }
        invalidate_botline_win(towin);
        changed_line_abv_curs_win(towin);
        check_topfill(towin, false_0 != 0);
        hasFolding(
            towin,
            (*towin).w_topline,
            &raw mut (*towin).w_topline,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    }
}

pub unsafe extern "C" fn diff_move_to(
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lnum: linenr_T = (*curwin.get()).w_cursor.lnum;
        let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
            return FAIL;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        if (*curtab.get()).tp_first_diff.is_null() {
            return FAIL;
        }
        loop {
            count -= 1;
            if count < 0 as ::core::ffi::c_int {
                break;
            }
            if dir == BACKWARD as ::core::ffi::c_int
                && lnum <= (*(*curtab.get()).tp_first_diff).df_lnum[idx as usize]
            {
                break;
            }
            let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
            dp = (*curtab.get()).tp_first_diff;
            while !dp.is_null() {
                if dir == FORWARD as ::core::ffi::c_int && lnum < (*dp).df_lnum[idx as usize]
                    || dir == BACKWARD as ::core::ffi::c_int
                        && ((*dp).df_next.is_null()
                            || lnum <= (*(*dp).df_next).df_lnum[idx as usize])
                {
                    lnum = (*dp).df_lnum[idx as usize];
                    break;
                } else {
                    dp = (*dp).df_next;
                }
            }
        }
        lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count {
            lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        if lnum == (*curwin.get()).w_cursor.lnum {
            return FAIL;
        }
        setpcmark();
        (*curwin.get()).w_cursor.lnum = lnum;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        return OK;
    }
}

unsafe extern "C" fn diff_get_corresponding_line_int(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    unsafe {
        let mut baseline: linenr_T = 0 as linenr_T;
        let mut idx1: ::core::ffi::c_int = diff_buf_idx(buf1, curtab.get());
        let mut idx2: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx1 == DB_COUNT || idx2 == DB_COUNT || (*curtab.get()).tp_first_diff.is_null() {
            return lnum1;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        if (*curtab.get()).tp_first_diff.is_null() {
            return lnum1;
        }
        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if (*dp).df_lnum[idx1 as usize] > lnum1 {
                return lnum1 - baseline;
            }
            if (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize] > lnum1 {
                baseline = lnum1 - (*dp).df_lnum[idx1 as usize];
                baseline = if baseline < (*dp).df_count[idx2 as usize] {
                    baseline
                } else {
                    (*dp).df_count[idx2 as usize]
                };
                return (*dp).df_lnum[idx2 as usize] + baseline;
            }
            if (*dp).df_lnum[idx1 as usize] == lnum1
                && (*dp).df_count[idx1 as usize] == 0 as linenr_T
                && (*dp).df_lnum[idx2 as usize] <= (*curwin.get()).w_cursor.lnum
                && (*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]
                    > (*curwin.get()).w_cursor.lnum
            {
                return (*curwin.get()).w_cursor.lnum;
            }
            baseline = (*dp).df_lnum[idx1 as usize] + (*dp).df_count[idx1 as usize]
                - ((*dp).df_lnum[idx2 as usize] + (*dp).df_count[idx2 as usize]);
            dp = (*dp).df_next;
        }
        return lnum1 - baseline;
    }
}

pub unsafe extern "C" fn diff_get_corresponding_line(
    mut buf1: *mut buf_T,
    mut lnum1: linenr_T,
) -> linenr_T {
    unsafe {
        let mut lnum: linenr_T = diff_get_corresponding_line_int(buf1, lnum1);
        return if lnum < (*curbuf.get()).b_ml.ml_line_count {
            lnum
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
    }
}

pub unsafe extern "C" fn diff_lnum_win(mut lnum: linenr_T, mut wp: *mut win_T) -> linenr_T {
    unsafe {
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx == DB_COUNT {
            return 0 as linenr_T;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
                break;
            }
            dp = (*dp).df_next;
        }
        if dp.is_null() {
            return (*(*wp).w_buffer).b_ml.ml_line_count
                - ((*curbuf.get()).b_ml.ml_line_count - lnum);
        }
        let mut i: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
        if i == DB_COUNT {
            return 0 as linenr_T;
        }
        let mut n: linenr_T = lnum + ((*dp).df_lnum[i as usize] - (*dp).df_lnum[idx as usize]);
        return if n < (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] {
            n
        } else {
            (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize]
        };
    }
}
