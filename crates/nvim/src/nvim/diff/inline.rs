//! Which columns of a changed line changed.
//!
//! `diff_find_change` answers, for one line of one window, the list of changed
//! column ranges the drawer paints `DiffText` over.  `diff_find_change_simple` is
//! the `inline:simple` rule -- one range, from the first differing byte to the
//! last -- and `diff_change_parse` is how the drawer reads a range back out.
//! `f_diff_hlID` is the Vimscript front door to the same answer, and the only
//! way to observe any of it without a screen.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn diff_update_line(mut lnum: linenr_T) {
    unsafe {
        if diff_flags.get() & ALL_INLINE_DIFF == 0 {
            return;
        }
        let mut idx: ::core::ffi::c_int = diff_buf_idx(curbuf.get(), curtab.get());
        if idx == DB_COUNT {
            return;
        }
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if lnum <= (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
                break;
            }
            dp = (*dp).df_next;
        }
        if !dp.is_null() {
            (*dp).has_changes = false_0 != 0;
            (*dp).df_changes.ga_len = 0 as ::core::ffi::c_int;
        }
    }
}

static simple_diffline_change: GlobalCell<diffline_change_T> = GlobalCell::new(diffline_change_T {
    dc_start: [0; 8],
    dc_end: [0; 8],
    dc_start_lnum_off: [0; 8],
    dc_end_lnum_off: [0; 8],
});

pub unsafe extern "C" fn diff_change_parse(
    mut diffline: *mut diffline_T,
    mut change: *mut diffline_change_T,
    mut change_start: *mut ::core::ffi::c_int,
    mut change_end: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*change).dc_start_lnum_off[(*diffline).bufidx as usize] < (*diffline).lineoff {
            *change_start = 0 as ::core::ffi::c_int;
        } else {
            *change_start = (*change).dc_start[(*diffline).bufidx as usize] as ::core::ffi::c_int;
        }
        if (*change).dc_end_lnum_off[(*diffline).bufidx as usize] > (*diffline).lineoff {
            *change_end = INT_MAX;
        } else {
            *change_end = (*change).dc_end[(*diffline).bufidx as usize] as ::core::ffi::c_int;
        }
        if change == simple_diffline_change.ptr() {
            return false_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if i != (*diffline).bufidx {
                if (*change).dc_start[i as usize] != (*change).dc_end[i as usize]
                    || (*change).dc_end_lnum_off[i as usize]
                        != (*change).dc_start_lnum_off[i as usize]
                {
                    return false_0 != 0;
                }
            }
            i += 1;
        }
        return true_0 != 0;
    }
}

unsafe extern "C" fn diff_find_change_simple(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut dp: *const diff_T,
    mut idx: ::core::ffi::c_int,
    mut startp: *mut ::core::ffi::c_int,
    mut endp: *mut ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut line_org: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if diff_flags.get() & DIFF_INLINE_NONE != 0 {
            line_org = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            line_org = xstrdup(ml_get_buf((*wp).w_buffer, lnum));
        }
        let mut si_org: ::core::ffi::c_int = 0;
        let mut si_new: ::core::ffi::c_int = 0;
        let mut ei_org: ::core::ffi::c_int = 0;
        let mut ei_new: ::core::ffi::c_int = 0;
        let mut added: bool = true_0 != 0;
        let mut off: linenr_T = lnum - (*dp).df_lnum[idx as usize];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[i as usize].is_null() && i != idx {
                if off < (*dp).df_count[i as usize] {
                    added = false_0 != 0;
                    if diff_flags.get() & DIFF_INLINE_NONE != 0 {
                        break;
                    }
                    let mut line_new: *mut ::core::ffi::c_char = ml_get_buf(
                        (*curtab.get()).tp_diffbuf[i as usize] as *mut buf_T,
                        (*dp).df_lnum[i as usize] + off,
                    );
                    si_new = 0 as ::core::ffi::c_int;
                    si_org = si_new;
                    while *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL {
                        if diff_flags.get() & DIFF_IWHITE != 0
                            && ascii_iswhite(*line_org.offset(si_org as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && ascii_iswhite(*line_new.offset(si_new as isize) as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            || diff_flags.get() & DIFF_IWHITEALL != 0
                                && (ascii_iswhite(
                                    *line_org.offset(si_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                                    || ascii_iswhite(
                                        *line_new.offset(si_new as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0)
                        {
                            si_org = skipwhite(line_org.offset(si_org as isize))
                                .offset_from(line_org)
                                as ::core::ffi::c_int;
                            si_new = skipwhite(line_new.offset(si_new as isize))
                                .offset_from(line_new)
                                as ::core::ffi::c_int;
                        } else {
                            let mut l: ::core::ffi::c_int = 0;
                            if !diff_equal_char(
                                line_org.offset(si_org as isize),
                                line_new.offset(si_new as isize),
                                &raw mut l,
                            ) {
                                break;
                            }
                            si_org += l;
                            si_new += l;
                        }
                    }
                    si_org -= utf_head_off(line_org, line_org.offset(si_org as isize));
                    si_new -= utf_head_off(line_new, line_new.offset(si_new as isize));
                    *startp = if *startp < si_org { *startp } else { si_org };
                    if *line_org.offset(si_org as isize) as ::core::ffi::c_int != NUL
                        || *line_new.offset(si_new as isize) as ::core::ffi::c_int != NUL
                    {
                        ei_org = strlen(line_org) as ::core::ffi::c_int;
                        ei_new = strlen(line_new) as ::core::ffi::c_int;
                        while ei_org >= *startp
                            && ei_new >= si_new
                            && ei_org >= 0 as ::core::ffi::c_int
                            && ei_new >= 0 as ::core::ffi::c_int
                        {
                            if diff_flags.get() & DIFF_IWHITE != 0
                                && ascii_iswhite(
                                    *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                                && ascii_iswhite(
                                    *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                ) as ::core::ffi::c_int
                                    != 0
                                || diff_flags.get() & DIFF_IWHITEALL != 0
                                    && (ascii_iswhite(
                                        *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0
                                        || ascii_iswhite(
                                            *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                        )
                                            as ::core::ffi::c_int
                                            != 0)
                            {
                                while ei_org >= *startp
                                    && ascii_iswhite(
                                        *line_org.offset(ei_org as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0
                                {
                                    ei_org -= 1;
                                }
                                while ei_new >= si_new
                                    && ascii_iswhite(
                                        *line_new.offset(ei_new as isize) as ::core::ffi::c_int
                                    ) as ::core::ffi::c_int
                                        != 0
                                {
                                    ei_new -= 1;
                                }
                            } else {
                                let mut p1: *const ::core::ffi::c_char =
                                    line_org.offset(ei_org as isize);
                                let mut p2: *const ::core::ffi::c_char =
                                    line_new.offset(ei_new as isize);
                                p1 = p1.offset(-(utf_head_off(line_org, p1) as isize));
                                p2 = p2.offset(-(utf_head_off(line_new, p2) as isize));
                                let mut l_0: ::core::ffi::c_int = 0;
                                if !diff_equal_char(p1, p2, &raw mut l_0) {
                                    break;
                                }
                                ei_org -= l_0;
                                ei_new -= l_0;
                            }
                        }
                        *endp = if *endp > ei_org { *endp } else { ei_org };
                    }
                }
            }
            i += 1;
        }
        xfree(line_org as *mut ::core::ffi::c_void);
        return added;
    }
}

pub unsafe extern "C" fn diff_find_change(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut diffline: *mut diffline_T,
) -> bool {
    unsafe {
        let mut idx: ::core::ffi::c_int = diff_buf_idx((*wp).w_buffer, curtab.get());
        if idx == DB_COUNT {
            return false_0 != 0;
        }
        let mut dp: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        dp = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if lnum < (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] {
                break;
            }
            dp = (*dp).df_next;
        }
        if dp.is_null() || diff_check_sanity(curtab.get(), dp) == FAIL {
            return false_0 != 0;
        }
        let mut off: ::core::ffi::c_int =
            lnum as ::core::ffi::c_int - (*dp).df_lnum[idx as usize] as ::core::ffi::c_int;
        if diff_flags.get() & ALL_INLINE_DIFF == 0 {
            let mut change_start: ::core::ffi::c_int = MAXCOL as ::core::ffi::c_int;
            let mut change_end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut ret: ::core::ffi::c_int = diff_find_change_simple(
                wp,
                lnum,
                dp,
                idx,
                &raw mut change_start,
                &raw mut change_end,
            ) as ::core::ffi::c_int;
            change_end += 1 as ::core::ffi::c_int;
            memset(
                simple_diffline_change.ptr() as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<diffline_change_T>(),
            );
            (*diffline).changes = simple_diffline_change.ptr();
            (*diffline).num_changes = 1 as ::core::ffi::c_int;
            (*diffline).bufidx = idx;
            (*diffline).lineoff = (lnum - (*dp).df_lnum[idx as usize]) as ::core::ffi::c_int;
            (*simple_diffline_change.ptr()).dc_start[idx as usize] = change_start as colnr_T;
            (*simple_diffline_change.ptr()).dc_end[idx as usize] = change_end as colnr_T;
            (*simple_diffline_change.ptr()).dc_start_lnum_off[idx as usize] = off;
            (*simple_diffline_change.ptr()).dc_end_lnum_off[idx as usize] = off;
            return ret != 0;
        }
        if !(*dp).has_changes {
            diff_find_change_inline_diff(dp);
        }
        let mut changes: *mut garray_T = &raw mut (*dp).df_changes;
        let mut num_changes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut change_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*diffline).changes = ::core::ptr::null_mut::<diffline_change_T>();
        change_idx = 0 as ::core::ffi::c_int;
        while change_idx < (*changes).ga_len {
            let mut change: *mut diffline_change_T =
                ((*dp).df_changes.ga_data as *mut diffline_change_T).offset(change_idx as isize);
            if (*change).dc_end_lnum_off[idx as usize] >= off {
                if (*change).dc_start_lnum_off[idx as usize] > off {
                    break;
                }
                if (*diffline).changes.is_null() {
                    (*diffline).changes = change;
                }
                num_changes += 1;
            }
            change_idx += 1;
        }
        (*diffline).num_changes = num_changes;
        (*diffline).bufidx = idx;
        (*diffline).lineoff = off;
        let mut added: bool = false_0 != 0;
        if num_changes == 1 as ::core::ffi::c_int && change_idx == (*dp).df_changes.ga_len {
            added = true_0 != 0;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < DB_COUNT {
                if idx != i {
                    if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                        let mut change_0: *mut diffline_change_T = ((*dp).df_changes.ga_data
                            as *mut diffline_change_T)
                            .offset(((*dp).df_changes.ga_len - 1 as ::core::ffi::c_int) as isize);
                        if (*change_0).dc_start_lnum_off[i as usize] != INT_MAX {
                            added = false_0 != 0;
                            break;
                        }
                    }
                }
                i += 1;
            }
        }
        return added;
    }
}

pub unsafe extern "C" fn f_diff_hlID(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        static prev_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
        static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0 as varnumber_T);
        static fnum: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        static prev_diff_flags: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static change_start: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static change_end: GlobalCell<::core::ffi::c_int> =
            GlobalCell::new(0 as ::core::ffi::c_int);
        static hlID: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
        let mut diffline: diffline_T = diffline_S {
            changes: ::core::ptr::null_mut::<diffline_change_T>(),
            num_changes: 0,
            bufidx: 0,
            lineoff: 0,
        };
        let cache_results: bool = diff_flags.get() & ALL_INLINE_DIFF == 0;
        let mut lnum: linenr_T = tv_get_lnum(argvars);
        if lnum < 0 as linenr_T {
            lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        if !cache_results
            || lnum != prev_lnum.get()
            || changedtick.get() != buf_get_changedtick(curbuf.get())
            || fnum.get() != (*curbuf.get()).handle
            || diff_flags.get() != prev_diff_flags.get()
        {
            let mut linestatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            diff_check_with_linestatus(curwin.get(), lnum, &raw mut linestatus);
            if linestatus < 0 as ::core::ffi::c_int {
                if linestatus == -1 as ::core::ffi::c_int {
                    change_start.set(MAXCOL as ::core::ffi::c_int);
                    change_end.set(-1 as ::core::ffi::c_int);
                    if diff_find_change(curwin.get(), lnum, &raw mut diffline) {
                        hlID.set(HLF_ADD);
                    } else {
                        hlID.set(HLF_CHD);
                        if diffline.num_changes > 0 as ::core::ffi::c_int
                            && cache_results as ::core::ffi::c_int != 0
                        {
                            change_start.set(
                                (*diffline.changes.offset(0 as ::core::ffi::c_int as isize))
                                    .dc_start[diffline.bufidx as usize]
                                    as ::core::ffi::c_int,
                            );
                            change_end.set(
                                (*diffline.changes.offset(0 as ::core::ffi::c_int as isize)).dc_end
                                    [diffline.bufidx as usize]
                                    as ::core::ffi::c_int,
                            );
                        }
                    }
                } else {
                    hlID.set(HLF_ADD);
                }
            } else {
                hlID.set(HLF_NONE);
            }
            if cache_results {
                prev_lnum.set(lnum);
                changedtick.set(buf_get_changedtick(curbuf.get()));
                fnum.set((*curbuf.get()).handle as ::core::ffi::c_int);
                prev_diff_flags.set(diff_flags.get());
            }
        }
        if hlID.get() as ::core::ffi::c_uint == HLF_CHD as ::core::ffi::c_uint
            || hlID.get() as ::core::ffi::c_uint == HLF_TXD as ::core::ffi::c_uint
        {
            let mut col: ::core::ffi::c_int =
                tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int;
            if cache_results {
                if col >= change_start.get() && col < change_end.get() {
                    hlID.set(HLF_TXD);
                } else {
                    hlID.set(HLF_CHD);
                }
            } else {
                hlID.set(HLF_CHD);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < diffline.num_changes {
                    let mut added: bool = diff_change_parse(
                        &raw mut diffline,
                        diffline.changes.offset(i as isize),
                        change_start.ptr(),
                        change_end.ptr(),
                    );
                    if col >= change_start.get() && col < change_end.get() {
                        hlID.set(
                            (if added as ::core::ffi::c_int != 0 {
                                HLF_TXA
                            } else {
                                HLF_TXD
                            }) as hlf_T,
                        );
                        break;
                    } else {
                        if col < change_start.get() {
                            break;
                        }
                        i += 1;
                    }
                }
            }
        }
        (*rettv).vval.v_number = hlID.get() as varnumber_T;
    }
}
