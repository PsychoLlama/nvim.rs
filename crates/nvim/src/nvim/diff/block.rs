//! The diff block list, and which buffers are in it.
//!
//! A tabpage owns a linked list of `diff_T` blocks, each naming a line range in
//! every one of the (up to eight) buffers `tp_diffbuf` holds.  This file owns
//! both halves: `diff_buf_add`/`diff_buf_delete`/`diff_buf_idx` are the
//! registry, and `diff_alloc_new`/`diff_free`/`diff_check_sanity` the list.
//! `diff_mark_adjust_tp` is the one that keeps the list correct across an edit
//! without recomputing it -- and it is only ever *read* under the external diff,
//! because `diff_internal()` makes the tabpage invalid instead.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn clear_diffblock(mut dp: *mut diff_T) {
    unsafe {
        ga_clear(&raw mut (*dp).df_changes);
        xfree(dp as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn diff_buf_delete(mut buf: *mut buf_T) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
            if i != DB_COUNT {
                (*tp).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
                (*tp).tp_diff_invalid = true_0;
                if tp == curtab.get() {
                    need_diff_redraw.set(true_0 != 0);
                    redraw_later(curwin.get(), UPD_VALID);
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub unsafe extern "C" fn diff_buf_adjust(mut win: *mut win_T) {
    unsafe {
        if (*win).w_onebuf_opt.wo_diff == 0 {
            let mut found_win: bool = false_0 != 0;
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == (*win).w_buffer && (*wp).w_onebuf_opt.wo_diff != 0 {
                    found_win = true_0 != 0;
                }
                wp = (*wp).w_next;
            }
            if !found_win {
                let mut i: ::core::ffi::c_int = diff_buf_idx((*win).w_buffer, curtab.get());
                if i != DB_COUNT {
                    (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
                    (*curtab.get()).tp_diff_invalid = true_0;
                    diff_redraw(true_0 != 0);
                }
            }
        } else {
            diff_buf_add((*win).w_buffer);
        };
    }
}

pub unsafe extern "C" fn diff_buf_add(mut buf: *mut buf_T) {
    unsafe {
        if diff_buf_idx(buf, curtab.get()) != DB_COUNT {
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if (*curtab.get()).tp_diffbuf[i as usize].is_null() {
                (*curtab.get()).tp_diffbuf[i as usize] = buf as *mut buf_T;
                (*curtab.get()).tp_diff_invalid = true_0;
                diff_redraw(true_0 != 0);
                return;
            }
            i += 1;
        }
        semsg(
            gettext(
                b"E96: Cannot diff more than %d buffers\0".as_ptr() as *const ::core::ffi::c_char
            ),
            DB_COUNT,
        );
    }
}

pub(crate) unsafe extern "C" fn diff_buf_clear() {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[i as usize].is_null() {
                (*curtab.get()).tp_diffbuf[i as usize] = ::core::ptr::null_mut::<buf_T>();
                (*curtab.get()).tp_diff_invalid = true_0;
                diff_redraw(true_0 != 0);
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn diff_buf_idx(
    mut buf: *mut buf_T,
    mut tp: *mut tabpage_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx: ::core::ffi::c_int = 0;
        idx = 0 as ::core::ffi::c_int;
        while idx < DB_COUNT {
            if (*tp).tp_diffbuf[idx as usize] == buf {
                break;
            }
            idx += 1;
        }
        return idx;
    }
}

pub unsafe extern "C" fn diff_invalidate(mut buf: *mut buf_T) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut i: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
            if i != DB_COUNT {
                (*tp).tp_diff_invalid = true_0;
                if tp == curtab.get() {
                    diff_redraw(true_0 != 0);
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub unsafe extern "C" fn diff_mark_adjust(
    mut buf: *mut buf_T,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut idx: ::core::ffi::c_int = diff_buf_idx(buf, tp as *mut tabpage_T);
            if idx != DB_COUNT {
                diff_mark_adjust_tp(
                    tp as *mut tabpage_T,
                    idx,
                    line1,
                    line2,
                    amount,
                    amount_after,
                );
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

unsafe extern "C" fn diff_mark_adjust_tp(
    mut tp: *mut tabpage_T,
    mut idx: ::core::ffi::c_int,
    mut line1: linenr_T,
    mut line2: linenr_T,
    mut amount: linenr_T,
    mut amount_after: linenr_T,
) {
    unsafe {
        if diff_internal() != 0 {
            (*tp).tp_diff_invalid = true_0;
            (*tp).tp_diff_update = true_0;
        }
        let mut inserted: linenr_T = 0;
        let mut deleted: linenr_T = 0;
        if line2 == MAXLNUM as ::core::ffi::c_int as linenr_T {
            inserted = amount;
            deleted = 0 as ::core::ffi::c_int as linenr_T;
        } else if amount_after > 0 as linenr_T {
            inserted = amount_after;
            deleted = 0 as ::core::ffi::c_int as linenr_T;
        } else {
            inserted = 0 as ::core::ffi::c_int as linenr_T;
            deleted = -amount_after;
        }
        let mut dprev: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut dp: *mut diff_T = (*tp).tp_first_diff;
        let mut lnum_deleted: linenr_T = line1;
        loop {
            if (dp.is_null()
                || (*dp).df_lnum[idx as usize] - 1 as linenr_T > line2
                || line2 == MAXLNUM as ::core::ffi::c_int as linenr_T
                    && (*dp).df_lnum[idx as usize] > line1)
                && (dprev.is_null()
                    || (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize] < line1)
                && !diff_busy.get()
            {
                let mut dnext: *mut diff_T = diff_alloc_new(tp, dprev, dp);
                (*dnext).df_lnum[idx as usize] = line1;
                (*dnext).df_count[idx as usize] = inserted;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < DB_COUNT {
                    if !(*tp).tp_diffbuf[i as usize].is_null() && i != idx {
                        if dprev.is_null() {
                            (*dnext).df_lnum[i as usize] = line1;
                        } else {
                            (*dnext).df_lnum[i as usize] = line1
                                + ((*dprev).df_lnum[i as usize] + (*dprev).df_count[i as usize])
                                - ((*dprev).df_lnum[idx as usize]
                                    + (*dprev).df_count[idx as usize]);
                        }
                        (*dnext).df_count[i as usize] = deleted;
                    }
                    i += 1;
                }
            }
            if dp.is_null() {
                break;
            }
            let mut last: linenr_T =
                (*dp).df_lnum[idx as usize] + (*dp).df_count[idx as usize] - 1 as linenr_T;
            if last >= line1 - 1 as linenr_T {
                if diff_busy.get() {
                    if (*dp).df_lnum[idx as usize] > line2 {
                        (*dp).df_lnum[idx as usize] += amount_after;
                    }
                    dprev = dp;
                    dp = (*dp).df_next;
                    continue;
                } else if (*dp).df_lnum[idx as usize]
                    - (deleted + inserted != 0 as linenr_T) as ::core::ffi::c_int
                    > line2
                {
                    if amount_after == 0 as linenr_T {
                        break;
                    }
                    (*dp).df_lnum[idx as usize] += amount_after;
                } else {
                    let mut check_unchanged: bool = false_0 != 0;
                    if deleted > 0 as linenr_T {
                        let mut n: linenr_T = 0;
                        let mut off: linenr_T = 0 as linenr_T;
                        if (*dp).df_lnum[idx as usize] >= line1 {
                            if last <= line2 {
                                if !(*dp).df_next.is_null()
                                    && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T
                                        <= line2
                                {
                                    n = (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                                    deleted -= n;
                                    n -= (*dp).df_count[idx as usize];
                                    lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                                } else {
                                    n = deleted - (*dp).df_count[idx as usize];
                                }
                                (*dp).df_count[idx as usize] = 0 as ::core::ffi::c_int as linenr_T;
                            } else {
                                off = (*dp).df_lnum[idx as usize] - lnum_deleted;
                                n = off;
                                (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                                    as ::core::ffi::c_int
                                    - (line2 - (*dp).df_lnum[idx as usize] + 1 as linenr_T)
                                        as ::core::ffi::c_int)
                                    as linenr_T;
                                check_unchanged = true_0 != 0;
                            }
                            (*dp).df_lnum[idx as usize] = line1;
                        } else if last < line2 {
                            (*dp).df_count[idx as usize] = ((*dp).df_count[idx as usize]
                                as ::core::ffi::c_int
                                - (last - lnum_deleted + 1 as linenr_T) as ::core::ffi::c_int)
                                as linenr_T;
                            if !(*dp).df_next.is_null()
                                && (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T <= line2
                            {
                                n = (*(*dp).df_next).df_lnum[idx as usize] - 1 as linenr_T - last;
                                deleted -= (*(*dp).df_next).df_lnum[idx as usize] - lnum_deleted;
                                lnum_deleted = (*(*dp).df_next).df_lnum[idx as usize];
                            } else {
                                n = line2 - last;
                            }
                            check_unchanged = true_0 != 0;
                        } else {
                            n = 0 as ::core::ffi::c_int as linenr_T;
                            (*dp).df_count[idx as usize] -= deleted;
                        }
                        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        while i_0 < DB_COUNT {
                            if !(*tp).tp_diffbuf[i_0 as usize].is_null() && i_0 != idx {
                                if (*dp).df_lnum[i_0 as usize] > off {
                                    (*dp).df_lnum[i_0 as usize] -= off;
                                } else {
                                    (*dp).df_lnum[i_0 as usize] =
                                        1 as ::core::ffi::c_int as linenr_T;
                                }
                                (*dp).df_count[i_0 as usize] += n;
                            }
                            i_0 += 1;
                        }
                    } else if (*dp).df_lnum[idx as usize] <= line1 {
                        (*dp).df_count[idx as usize] += inserted;
                        check_unchanged = true_0 != 0;
                    } else {
                        (*dp).df_lnum[idx as usize] += inserted;
                    }
                    if check_unchanged {
                        diff_check_unchanged(tp, dp);
                    }
                }
            }
            if !dprev.is_null()
                && !(*dp).is_linematched
                && !diff_busy.get()
                && (*dprev).df_lnum[idx as usize] + (*dprev).df_count[idx as usize]
                    == (*dp).df_lnum[idx as usize]
            {
                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_1 < DB_COUNT {
                    if !(*tp).tp_diffbuf[i_1 as usize].is_null() {
                        (*dprev).df_count[i_1 as usize] += (*dp).df_count[i_1 as usize];
                    }
                    i_1 += 1;
                }
                dp = diff_free(tp, dprev, dp);
            } else {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
        dprev = ::core::ptr::null_mut::<diff_T>();
        dp = (*tp).tp_first_diff;
        while !dp.is_null() {
            let mut i_2: ::core::ffi::c_int = 0;
            i_2 = 0 as ::core::ffi::c_int;
            while i_2 < DB_COUNT {
                if !(*tp).tp_diffbuf[i_2 as usize].is_null()
                    && (*dp).df_count[i_2 as usize] != 0 as linenr_T
                {
                    break;
                }
                i_2 += 1;
            }
            if i_2 == DB_COUNT {
                dp = diff_free(tp, dprev, dp);
            } else {
                dprev = dp;
                dp = (*dp).df_next;
            }
        }
        if tp == curtab.get() {
            need_diff_redraw.set(true_0 != 0);
            diff_need_scrollbind.set(true_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn diff_alloc_new(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    unsafe {
        let mut dnew: *mut diff_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<diff_T>()) as *mut diff_T;
        (*dnew).is_linematched = false_0 != 0;
        (*dnew).df_next = dp;
        if dprev.is_null() {
            (*tp).tp_first_diff = dnew;
        } else {
            (*dprev).df_next = dnew;
        }
        (*dnew).has_changes = false_0 != 0;
        ga_init(
            &raw mut (*dnew).df_changes,
            ::core::mem::size_of::<diffline_change_T>() as ::core::ffi::c_int,
            20 as ::core::ffi::c_int,
        );
        return dnew;
    }
}

pub(crate) unsafe extern "C" fn diff_free(
    mut tp: *mut tabpage_T,
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
) -> *mut diff_T {
    unsafe {
        let mut ret: *mut diff_T = (*dp).df_next;
        clear_diffblock(dp);
        if dprev.is_null() {
            (*tp).tp_first_diff = ret;
        } else {
            (*dprev).df_next = ret;
        }
        return ret;
    }
}

unsafe extern "C" fn diff_check_unchanged(mut tp: *mut tabpage_T, mut dp: *mut diff_T) {
    unsafe {
        let mut i_org: ::core::ffi::c_int = 0;
        i_org = 0 as ::core::ffi::c_int;
        while i_org < DB_COUNT {
            if !(*tp).tp_diffbuf[i_org as usize].is_null() {
                break;
            }
            i_org += 1;
        }
        if i_org == DB_COUNT {
            return;
        }
        if diff_check_sanity(tp, dp) == FAIL {
            return;
        }
        let mut off_org: linenr_T = 0 as linenr_T;
        let mut off_new: linenr_T = 0 as linenr_T;
        let mut dir: ::core::ffi::c_int = FORWARD as ::core::ffi::c_int;
        loop {
            while (*dp).df_count[i_org as usize] > 0 as linenr_T {
                if dir == BACKWARD as ::core::ffi::c_int {
                    off_org = (*dp).df_count[i_org as usize] - 1 as linenr_T;
                }
                let mut line_org: *mut ::core::ffi::c_char = xstrdup(ml_get_buf(
                    (*tp).tp_diffbuf[i_org as usize] as *mut buf_T,
                    (*dp).df_lnum[i_org as usize] + off_org,
                ));
                let mut i_new: ::core::ffi::c_int = 0;
                i_new = i_org + 1 as ::core::ffi::c_int;
                while i_new < DB_COUNT {
                    if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                        if dir == BACKWARD as ::core::ffi::c_int {
                            off_new = (*dp).df_count[i_new as usize] - 1 as linenr_T;
                        }
                        if off_new < 0 as linenr_T || off_new >= (*dp).df_count[i_new as usize] {
                            break;
                        }
                        if diff_cmp(
                            line_org,
                            ml_get_buf(
                                (*tp).tp_diffbuf[i_new as usize] as *mut buf_T,
                                (*dp).df_lnum[i_new as usize] + off_new,
                            ),
                        ) != 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                    }
                    i_new += 1;
                }
                xfree(line_org as *mut ::core::ffi::c_void);
                if i_new != DB_COUNT {
                    break;
                }
                i_new = i_org;
                while i_new < DB_COUNT {
                    if !(*tp).tp_diffbuf[i_new as usize].is_null() {
                        if dir == FORWARD as ::core::ffi::c_int {
                            (*dp).df_lnum[i_new as usize] += 1;
                        }
                        (*dp).df_count[i_new as usize] -= 1;
                    }
                    i_new += 1;
                }
            }
            if dir == BACKWARD as ::core::ffi::c_int {
                break;
            }
            dir = BACKWARD as ::core::ffi::c_int;
        }
    }
}

pub(crate) unsafe extern "C" fn diff_check_sanity(
    mut tp: *mut tabpage_T,
    mut dp: *mut diff_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < DB_COUNT {
            if !(*tp).tp_diffbuf[i as usize].is_null() {
                if (*dp).df_lnum[i as usize] + (*dp).df_count[i as usize] - 1 as linenr_T
                    > (*(*tp).tp_diffbuf[i as usize]).b_ml.ml_line_count
                {
                    return FAIL;
                }
            }
            i += 1;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn diff_copy_entry(
    mut dprev: *mut diff_T,
    mut dp: *mut diff_T,
    mut idx_orig: ::core::ffi::c_int,
    mut idx_new: ::core::ffi::c_int,
) {
    unsafe {
        let mut off: linenr_T = 0;
        if dprev.is_null() {
            off = 0 as ::core::ffi::c_int as linenr_T;
        } else {
            off = (*dprev).df_lnum[idx_orig as usize] + (*dprev).df_count[idx_orig as usize]
                - ((*dprev).df_lnum[idx_new as usize] + (*dprev).df_count[idx_new as usize]);
        }
        (*dp).df_lnum[idx_new as usize] = (*dp).df_lnum[idx_orig as usize] - off;
        (*dp).df_count[idx_new as usize] = (*dp).df_count[idx_orig as usize];
    }
}

pub unsafe extern "C" fn diff_clear(mut tp: *mut tabpage_T) {
    unsafe {
        let mut next_p: *mut diff_T = ::core::ptr::null_mut::<diff_T>();
        let mut p: *mut diff_T = (*tp).tp_first_diff;
        while !p.is_null() {
            next_p = (*p).df_next;
            clear_diffblock(p);
            p = next_p;
        }
        (*tp).tp_first_diff = ::core::ptr::null_mut::<diff_T>();
    }
}

pub(crate) unsafe extern "C" fn get_max_diff_length(mut dp: *const diff_T) -> ::core::ffi::c_int {
    unsafe {
        let mut maxlength: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while k < DB_COUNT {
            if !(*curtab.get()).tp_diffbuf[k as usize].is_null() {
                if (*dp).df_count[k as usize] > maxlength as linenr_T {
                    maxlength = (*dp).df_count[k as usize] as ::core::ffi::c_int;
                }
            }
            k += 1;
        }
        return maxlength;
    }
}

pub(crate) unsafe extern "C" fn valid_diff(mut diff: *mut diff_T) -> bool {
    unsafe {
        let mut dp: *mut diff_T = (*curtab.get()).tp_first_diff;
        while !dp.is_null() {
            if dp == diff {
                return true_0 != 0;
            }
            dp = (*dp).df_next;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn diff_mode_buf(mut buf: *mut buf_T) -> bool {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if diff_buf_idx(buf, tp as *mut tabpage_T) != DB_COUNT {
                return true_0 != 0;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return false_0 != 0;
    }
}
