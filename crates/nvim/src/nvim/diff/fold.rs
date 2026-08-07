//! The per-line answers the drawer and the fold code ask.
//!
//! `diff_check_with_linestatus` says what a line's diff status is -- changed,
//! added, filler, or unchanged -- which is what both the highlighting and
//! `diff_check_fill`'s filler count are read off.  `diff_infold` decides whether
//! a line belongs inside a closed diff fold, and `diff_redraw` is what marks the
//! diffed windows dirty after the block list moves.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn diff_redraw(mut dofold: bool) {
    unsafe {
        let mut wp_other: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut used_max_fill_other: bool = false_0 != 0;
        let mut used_max_fill_curwin: bool = false_0 != 0;
        need_diff_redraw.set(false_0 != 0);
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if !((*wp).w_onebuf_opt.wo_diff == 0 || !buf_valid((*wp).w_buffer)) {
                redraw_later(wp, UPD_SOME_VALID);
                if wp != curwin.get() {
                    wp_other = wp;
                }
                if dofold as ::core::ffi::c_int != 0
                    && foldmethodIsDiff(wp) as ::core::ffi::c_int != 0
                {
                    foldUpdateAll(wp);
                }
                let mut n: ::core::ffi::c_int = diff_check_fill(wp, (*wp).w_topline);
                if wp != curwin.get() && (*wp).w_topfill > 0 as ::core::ffi::c_int
                    || n > 0 as ::core::ffi::c_int
                {
                    if (*wp).w_topfill > n {
                        (*wp).w_topfill = if n > 0 as ::core::ffi::c_int {
                            n
                        } else {
                            0 as ::core::ffi::c_int
                        };
                    } else if n > 0 as ::core::ffi::c_int && n > (*wp).w_topfill {
                        (*wp).w_topfill = n;
                        if wp == curwin.get() {
                            used_max_fill_curwin = true_0 != 0;
                        } else if !wp_other.is_null() {
                            used_max_fill_other = true_0 != 0;
                        }
                    }
                    check_topfill(wp, false_0 != 0);
                }
            }
            wp = (*wp).w_next;
        }
        if !wp_other.is_null() && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
            if used_max_fill_curwin {
                diff_set_topline(wp_other, curwin.get());
            } else if used_max_fill_other {
                diff_set_topline(curwin.get(), wp_other);
            }
        }
    }
}

pub unsafe extern "C" fn diff_check_with_linestatus(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut linestatus: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = (*wp).w_buffer;
        if !linestatus.is_null() {
            *linestatus = 0 as ::core::ffi::c_int;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        if (*curtab.get()).tp_first_diff.is_null() || (*wp).w_onebuf_opt.wo_diff == 0 {
            return 0 as ::core::ffi::c_int;
        }
        if lnum < 1 as linenr_T || lnum > (*buf).b_ml.ml_line_count + 1 as linenr_T {
            return 0 as ::core::ffi::c_int;
        }
        let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, curtab.get());
        if idx == DB_COUNT {
            return 0 as ::core::ffi::c_int;
        }
        if hasFolding(
            wp,
            lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            ::core::ptr::null_mut::<linenr_T>(),
        ) as ::core::ffi::c_int
            != 0
            || decor_conceal_line(
                wp,
                lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                false_0 != 0,
            ) as ::core::ffi::c_int
                != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
                break;
            }
            dp = (*dp).df_next;
        }
        if dp.is_null() || lnum < (*dp).df_lnum[idx as usize] {
            return 0 as ::core::ffi::c_int;
        }
        if lnum >= (*wp).w_topline
            && lnum < (*wp).w_botline
            && !(*dp).is_linematched
            && diff_linematch(dp) as ::core::ffi::c_int != 0
            && diff_check_sanity(curtab.get(), dp) != 0
        {
            run_linematch_algorithm(dp);
        }
        let mut num_fill: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while lnum == (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            if diff_flags.get() & DIFF_FILLER != 0 {
                let mut maxcount: ::core::ffi::c_int = get_max_diff_length(dp);
                num_fill +=
                    (maxcount as linenr_T - (*dp).df_count[idx as usize]) as ::core::ffi::c_int;
            }
            if !(!(*dp).df_next.is_null()
                && lnum >= (*(*dp).df_next).df_lnum[idx as usize]
                && lnum
                    <= (*(*dp).df_next).df_lnum[idx as usize]
                        + (*(*dp).df_next).df_count[idx as usize])
            {
                break;
            }
            dp = (*dp).df_next;
        }
        if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
            let mut zero: bool = false_0 != 0;
            let mut cmp: bool = false_0 != 0;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if i != idx && !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                    if (*dp).df_count[i as usize] == 0 as linenr_T {
                        zero = true_0 != 0;
                    } else {
                        if (*dp).df_count[i as usize] != (*dp).df_count[idx as usize] {
                            if !linestatus.is_null() {
                                *linestatus = -1 as ::core::ffi::c_int;
                            }
                            return num_fill;
                        }
                        cmp = true_0 != 0;
                    }
                }
                i += 1;
            }
            if cmp {
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < DB_COUNT {
                    if i_0 != idx
                        && !(*curtab.get()).tp_diffbuf[i_0 as usize].is_null()
                        && (*dp).df_count[i_0 as usize] != 0 as linenr_T
                    {
                        if !diff_equal_entry(dp, idx as usize, i_0 as usize) {
                            if !linestatus.is_null() {
                                *linestatus = -1 as ::core::ffi::c_int;
                            }
                            return num_fill;
                        }
                    }
                    i_0 += 1;
                }
            }
            if !zero {
                return num_fill;
            }
            if !linestatus.is_null() {
                *linestatus = -2 as ::core::ffi::c_int;
            }
            return num_fill;
        }
        return num_fill;
    }
}

pub unsafe extern "C" fn diff_check_fill(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        if diff_flags.get() & DIFF_FILLER == 0 {
            return 0 as ::core::ffi::c_int;
        }
        let mut n: ::core::ffi::c_int =
            diff_check_with_linestatus(wp, lnum, ::core::ptr::null_mut::<::core::ffi::c_int>());
        return if n > 0 as ::core::ffi::c_int {
            n
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn diff_infold(mut wp: *mut win_T, mut lnum: linenr_T) -> bool {
    unsafe {
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            return false_0 != 0;
        }
        let mut idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut other: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer {
                idx = i;
            } else if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                other = true_0 != 0;
            }
            i += 1;
        }
        if idx == -1 as ::core::ffi::c_int || !other {
            return false_0 != 0;
        }
        if (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
        }
        if (*curtab.get()).tp_first_diff.is_null() {
            return true_0 != 0;
        }
        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if (*dp).df_lnum[idx as usize] - diff_context.get() as linenr_T > lnum {
                break;
            }
            if (*dp).df_lnum[idx as usize]
                + (*dp).df_count[idx as usize]
                + diff_context.get() as linenr_T
                > lnum
            {
                return false_0 != 0;
            }
            dp = (*dp).df_next;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn diff_fold_update(
    mut dp: *mut diff_T,
    mut skip_idx: ::core::ffi::c_int,
) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if (*curtab.get()).tp_diffbuf[i as usize] == (*wp).w_buffer && i != skip_idx {
                    foldUpdate(
                        wp,
                        (*dp).df_lnum[i as usize],
                        (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize],
                    );
                }
                i += 1;
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn f_diff_filler(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number =
            (if 0 as ::core::ffi::c_int > diff_check_fill(curwin.get(), tv_get_lnum(argvars)) {
                0 as ::core::ffi::c_int
            } else {
                diff_check_fill(curwin.get(), tv_get_lnum(argvars))
            }) as varnumber_T;
    }
}
