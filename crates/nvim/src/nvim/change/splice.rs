//! Who has to be told that lines changed, and what they are told.
//!
//! One splice -- lines `lnum`..`lnume` replaced, `xtra` lines net -- reaches
//! five audiences, and this file is the fan-out: the change list and the
//! `'[`/`']` marks (`changed_common`), every window's `w_lines` display cache
//! (`changed_lines_invalidate_win`), folds and 'cursorline'/'relativenumber'
//! state, the extmark tree (`extmark_splice`, via `changed_lines`), and the
//! buffer-update RPC and Lua callbacks (`buf_updates_send_changes`).  The last
//! two are why this file has no cheap test: `lua/buffer_updates_spec` is its
//! real gate, not any key-sequence sweep.
//!
//! `changed_bytes` is the one-line case, `changed_lines` the general one, and
//! `appended_lines`/`deleted_lines` the two that also move marks.  The `_buf`
//! and `_mark` suffixes are the same question asked of a buffer that may not be
//! current, and asked without touching the marks.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn changed_lines_invalidate_win(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    unsafe {
        if (*wp).w_cursor.lnum <= lnum {
            let mut i: ::core::ffi::c_int = find_wl_entry(wp, lnum);
            if i >= 0 as ::core::ffi::c_int
                && (*wp).w_cursor.lnum > (*(*wp).w_lines.offset(i as isize)).wl_lnum
            {
                changed_line_abv_curs_win(wp);
            }
        }
        if (*wp).w_cursor.lnum > lnum {
            changed_line_abv_curs_win(wp);
        } else if (*wp).w_cursor.lnum == lnum && (*wp).w_cursor.col >= col {
            changed_cline_bef_curs(wp);
        }
        if (*wp).w_botline >= lnum {
            if xtra < 0 as linenr_T {
                invalidate_botline_win(wp);
            } else {
                approximate_botline_win(wp);
            }
        }
        if xtra < 0 as linenr_T
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && buf_meta_total((*wp).w_buffer, kMTMetaInline) != 0
            || xtra != 0 as linenr_T && buf_meta_total((*wp).w_buffer, kMTMetaLines) != 0
        {
            lnume += 1;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*wp).w_lines_valid {
            if (*(*wp).w_lines.offset(i_0 as isize)).wl_valid {
                if (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum >= lnum {
                    if i_0 == 0 as ::core::ffi::c_int
                        || (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum < lnume
                    {
                        (*(*wp).w_lines.offset(i_0 as isize)).wl_valid = false_0 != 0;
                    } else if xtra != 0 as linenr_T {
                        (*(*wp).w_lines.offset(i_0 as isize)).wl_lnum += xtra;
                        (*(*wp).w_lines.offset(i_0 as isize)).wl_foldend += xtra;
                        (*(*wp).w_lines.offset(i_0 as isize)).wl_lastlnum += xtra;
                    }
                } else if (*(*wp).w_lines.offset(i_0 as isize)).wl_lastlnum >= lnum {
                    (*(*wp).w_lines.offset(i_0 as isize)).wl_valid = false_0 != 0;
                }
            }
            i_0 += 1;
        }
    }
}

pub unsafe extern "C" fn changed_lines_invalidate_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf {
                    changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

unsafe extern "C" fn changed_common(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    unsafe {
        changed(buf);
        let mut win: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !win.is_null() {
            if (*win).w_buffer == buf && (*win).w_onebuf_opt.wo_diff != 0 && diff_internal() != 0 {
                (*curtab.get()).tp_diff_update = true_0;
                diff_update_line(lnum);
            }
            win = (*win).w_next;
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut view: fmarkv_T = fmarkv_T {
                topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                skipcol: 0 as colnr_T,
            };
            if (*curwin.get()).w_buffer == buf {
                if lnum >= (*curwin.get()).w_topline && lnum <= (*curwin.get()).w_botline {
                    view = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
                }
            }
            let fmarkp___: *mut fmark_T = &raw mut (*buf).b_last_change;
            free_fmark(*fmarkp___);
            let fmarkp__: *mut fmark_T = fmarkp___;
            (*fmarkp__).mark = pos_T {
                lnum: lnum,
                col: col,
                coladd: 0 as colnr_T,
            };
            (*fmarkp__).fnum = (*buf).handle as ::core::ffi::c_int;
            (*fmarkp__).timestamp = os_time();
            (*fmarkp__).view = view;
            (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
            if (*buf).b_new_change as ::core::ffi::c_int != 0
                || (*buf).b_changelistlen == 0 as ::core::ffi::c_int
            {
                let mut add: bool = false;
                if (*buf).b_changelistlen == 0 as ::core::ffi::c_int {
                    add = true_0 != 0;
                } else {
                    let mut p: *mut pos_T = &raw mut (*(&raw mut (*buf).b_changelist
                        as *mut fmark_T)
                        .offset(((*buf).b_changelistlen - 1 as ::core::ffi::c_int) as isize))
                    .mark;
                    if (*p).lnum != lnum {
                        add = true_0 != 0;
                    } else {
                        let mut cols: ::core::ffi::c_int = comp_textwidth(false_0 != 0);
                        if cols == 0 as ::core::ffi::c_int {
                            cols = 79 as ::core::ffi::c_int;
                        }
                        add = (*p).col as ::core::ffi::c_int + cols < col
                            || col as ::core::ffi::c_int + cols < (*p).col;
                    }
                }
                if add {
                    (*buf).b_new_change = false_0 != 0;
                    if (*buf).b_changelistlen == JUMPLISTSIZE {
                        (*buf).b_changelistlen = JUMPLISTSIZE - 1 as ::core::ffi::c_int;
                        memmove(
                            &raw mut (*buf).b_changelist as *mut fmark_T
                                as *mut ::core::ffi::c_void,
                            (&raw mut (*buf).b_changelist as *mut fmark_T)
                                .offset(1 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<fmark_T>()
                                .wrapping_mul((JUMPLISTSIZE - 1 as ::core::ffi::c_int) as size_t),
                        );
                        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                        while !tp.is_null() {
                            let mut wp: *mut win_T = if tp == curtab.get() {
                                firstwin.get()
                            } else {
                                (*tp).tp_firstwin
                            };
                            while !wp.is_null() {
                                if (*wp).w_buffer == buf
                                    && (*wp).w_changelistidx > 0 as ::core::ffi::c_int
                                {
                                    (*wp).w_changelistidx -= 1;
                                }
                                wp = (*wp).w_next;
                            }
                            tp = (*tp).tp_next as *mut tabpage_T;
                        }
                    }
                    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                    while !tp_0.is_null() {
                        let mut wp_0: *mut win_T = if tp_0 == curtab.get() {
                            firstwin.get()
                        } else {
                            (*tp_0).tp_firstwin
                        };
                        while !wp_0.is_null() {
                            if (*wp_0).w_buffer == buf
                                && (*wp_0).w_changelistidx == (*buf).b_changelistlen
                            {
                                (*wp_0).w_changelistidx += 1;
                            }
                            wp_0 = (*wp_0).w_next;
                        }
                        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
                    }
                    (*buf).b_changelistlen += 1;
                }
            }
            (*buf).b_changelist[((*buf).b_changelistlen - 1 as ::core::ffi::c_int) as usize] =
                (*buf).b_last_change;
            if (*curwin.get()).w_buffer == buf {
                (*curwin.get()).w_changelistidx = (*buf).b_changelistlen;
            }
        }
        if (*curwin.get()).w_buffer == buf && VIsual_active.get() as ::core::ffi::c_int != 0 {
            check_visual_pos();
        }
        let mut tp_1: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp_1.is_null() {
            let mut wp_1: *mut win_T = if tp_1 == curtab.get() {
                firstwin.get()
            } else {
                (*tp_1).tp_firstwin
            };
            while !wp_1.is_null() {
                if (*wp_1).w_buffer == buf {
                    if !redraw_not_allowed.get() && (*wp_1).w_redr_type < UPD_VALID {
                        (*wp_1).w_redr_type = UPD_VALID;
                    }
                    if xtra != 0 as linenr_T && (*wp_1).w_redraw_top != 0 as linenr_T {
                        redraw_later(wp_1, UPD_NOT_VALID);
                    }
                    let mut last: linenr_T = lnume + xtra - 1 as linenr_T;
                    if (*wp_1).w_skipcol > 0 as ::core::ffi::c_int
                        && (last < (*wp_1).w_topline
                            || (*wp_1).w_topline >= lnum
                                && (*wp_1).w_topline < lnume
                                && linetabsize_eol(wp_1, (*wp_1).w_topline)
                                    <= (*wp_1).w_skipcol as ::core::ffi::c_int
                                        + sms_marker_overlap(wp_1, -1 as ::core::ffi::c_int))
                    {
                        (*wp_1).w_skipcol = 0 as ::core::ffi::c_int as colnr_T;
                    }
                    foldUpdate(wp_1, lnum, last);
                    let mut folded: bool = hasFoldingWin(
                        wp_1,
                        lnum,
                        &raw mut lnum,
                        ::core::ptr::null_mut::<linenr_T>(),
                        false_0 != 0,
                        ::core::ptr::null_mut::<foldinfo_T>(),
                    );
                    if (*wp_1).w_cursor.lnum == lnum {
                        (*wp_1).w_cline_folded = folded;
                    }
                    folded = hasFoldingWin(
                        wp_1,
                        last,
                        ::core::ptr::null_mut::<linenr_T>(),
                        &raw mut last,
                        false_0 != 0,
                        ::core::ptr::null_mut::<foldinfo_T>(),
                    );
                    if (*wp_1).w_cursor.lnum == last {
                        (*wp_1).w_cline_folded = folded;
                    }
                    changed_lines_invalidate_win(wp_1, lnum, col, lnume, xtra);
                    if hasAnyFolding(wp_1) != 0 {
                        set_topline(wp_1, (*wp_1).w_topline);
                    }
                    if (*wp_1).w_onebuf_opt.wo_rnu != 0 && xtra != 0 as linenr_T {
                        (*wp_1).w_last_cursor_lnum_rnu = 0 as ::core::ffi::c_int as linenr_T;
                    }
                    if (*wp_1).w_onebuf_opt.wo_cul != 0 && (*wp_1).w_last_cursorline >= lnum {
                        if (*wp_1).w_last_cursorline < lnume {
                            (*wp_1).w_last_cursorline = 0 as ::core::ffi::c_int as linenr_T;
                        } else {
                            (*wp_1).w_last_cursorline += xtra;
                        }
                    }
                }
                if wp_1 == curwin.get()
                    && xtra != 0 as linenr_T
                    && search_hl_has_cursor_lnum.get() >= lnum
                {
                    (*search_hl_has_cursor_lnum.ptr()) += xtra;
                }
                wp_1 = (*wp_1).w_next;
            }
            tp_1 = (*tp_1).tp_next as *mut tabpage_T;
        }
        set_must_redraw(UPD_VALID);
        if last_cursormoved_win.get() == curwin.get()
            && (*curwin.get()).w_buffer == buf
            && lnum <= (*curwin.get()).w_cursor.lnum
            && lnume + (if xtra < 0 as linenr_T { -xtra } else { xtra })
                > (*curwin.get()).w_cursor.lnum
        {
            (*last_cursormoved.ptr()).lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
    }
}

pub unsafe extern "C" fn changed_bytes(mut lnum: linenr_T, mut col: colnr_T) {
    unsafe {
        changed_lines_redraw_buf(curbuf.get(), lnum, lnum + 1 as linenr_T, 0 as linenr_T);
        changed_common(curbuf.get(), lnum, col, lnum + 1 as linenr_T, 0 as linenr_T);
        if spell_check_window(curwin.get()) as ::core::ffi::c_int != 0
            && lnum < (*curbuf.get()).b_ml.ml_line_count
            && vim_strchr(p_cpo.get(), CPO_DOLLAR).is_null()
        {
            redrawWinline(curwin.get(), lnum + 1 as linenr_T);
        }
        buf_updates_send_changes(curbuf.get(), lnum, 1 as int64_t, 1 as int64_t);
        if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                    redraw_later(wp, UPD_VALID);
                    let mut wlnum: linenr_T = diff_lnum_win(lnum, wp);
                    if wlnum > 0 as linenr_T {
                        changed_lines_redraw_buf(
                            (*wp).w_buffer,
                            wlnum,
                            wlnum + 1 as linenr_T,
                            0 as linenr_T,
                        );
                    }
                }
                wp = (*wp).w_next;
            }
        }
    }
}

pub unsafe extern "C" fn inserted_bytes(
    mut lnum: linenr_T,
    mut start_col: colnr_T,
    mut old_col: ::core::ffi::c_int,
    mut new_col: ::core::ffi::c_int,
) {
    unsafe {
        if curbuf_splice_pending.get() == 0 as ::core::ffi::c_int {
            extmark_splice_cols(
                curbuf.get(),
                lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                start_col,
                old_col as colnr_T,
                new_col as colnr_T,
                kExtmarkUndo,
            );
        }
        changed_bytes(lnum, start_col);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn appended_lines_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut count: linenr_T,
) {
    unsafe {
        changed_lines(
            buf,
            lnum + 1 as linenr_T,
            0 as colnr_T,
            lnum + 1 as linenr_T,
            count,
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn appended_lines(mut lnum: linenr_T, mut count: linenr_T) {
    unsafe {
        appended_lines_buf(curbuf.get(), lnum, count);
    }
}

pub unsafe extern "C" fn appended_lines_mark(mut lnum: linenr_T, mut count: ::core::ffi::c_int) {
    unsafe {
        mark_adjust(
            lnum + 1 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            count as linenr_T,
            0 as linenr_T,
            kExtmarkUndo,
        );
        changed_lines(
            curbuf.get(),
            lnum + 1 as linenr_T,
            0 as colnr_T,
            lnum + 1 as linenr_T,
            count as linenr_T,
            true_0 != 0,
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn deleted_lines_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut count: linenr_T,
) {
    unsafe {
        changed_lines(buf, lnum, 0 as colnr_T, lnum + count, -count, true_0 != 0);
    }
}

pub unsafe extern "C" fn deleted_lines(mut lnum: linenr_T, mut count: linenr_T) {
    unsafe {
        deleted_lines_buf(curbuf.get(), lnum, count);
    }
}

pub unsafe extern "C" fn deleted_lines_mark(mut lnum: linenr_T, mut count: ::core::ffi::c_int) {
    unsafe {
        let mut made_empty: bool =
            count > 0 as ::core::ffi::c_int && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
        mark_adjust(
            lnum,
            lnum + count as linenr_T - 1 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -(count as linenr_T),
            kExtmarkNOOP,
        );
        extmark_adjust(
            curbuf.get(),
            lnum,
            lnum + count as linenr_T - 1 as linenr_T,
            MAXLNUM as ::core::ffi::c_int as linenr_T,
            -(count as linenr_T)
                + (if made_empty as ::core::ffi::c_int != 0 {
                    1 as linenr_T
                } else {
                    0 as linenr_T
                }),
            kExtmarkUndo,
        );
        changed_lines(
            curbuf.get(),
            lnum,
            0 as colnr_T,
            lnum + count as linenr_T,
            -count as linenr_T,
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn changed_lines_redraw_buf(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
) {
    unsafe {
        if xtra != 0 as linenr_T
            && (*(&raw mut (*buf).b_marktree as *mut MarkTree)).n_keys > 0 as size_t
        {
            lnume = (lnume as ::core::ffi::c_int
                + (1 as ::core::ffi::c_int
                    + (xtra < 0 as linenr_T && buf_meta_total(buf, kMTMetaLines) != 0)
                        as ::core::ffi::c_int)) as linenr_T;
        }
        if (*buf).b_mod_set {
            (*buf).b_mod_top = if (*buf).b_mod_top < lnum {
                (*buf).b_mod_top
            } else {
                lnum
            };
            if lnum < (*buf).b_mod_bot {
                (*buf).b_mod_bot += xtra;
                (*buf).b_mod_bot = if (*buf).b_mod_bot > lnum {
                    (*buf).b_mod_bot
                } else {
                    lnum
                };
            }
            (*buf).b_mod_bot = if (*buf).b_mod_bot > lnume + xtra {
                (*buf).b_mod_bot
            } else {
                lnume + xtra
            };
            (*buf).b_mod_xlines += xtra;
        } else {
            (*buf).b_mod_set = true_0 != 0;
            (*buf).b_mod_top = lnum;
            (*buf).b_mod_bot = lnume + xtra;
            (*buf).b_mod_xlines = xtra;
        };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn changed_lines(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut lnume: linenr_T,
    mut xtra: linenr_T,
    mut do_buf_event: bool,
) {
    unsafe {
        changed_lines_redraw_buf(buf, lnum, lnume, xtra);
        if xtra == 0 as linenr_T
            && (*curwin.get()).w_onebuf_opt.wo_diff != 0
            && (*curwin.get()).w_buffer == buf
            && diff_internal() == 0
        {
            let mut wlnum: linenr_T = 0;
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                    redraw_later(wp, UPD_VALID);
                    wlnum = diff_lnum_win(lnum, wp);
                    if wlnum > 0 as linenr_T {
                        changed_lines_redraw_buf(
                            (*wp).w_buffer,
                            wlnum,
                            lnume - lnum + wlnum,
                            0 as linenr_T,
                        );
                    }
                }
                wp = (*wp).w_next;
            }
        }
        changed_common(buf, lnum, col, lnume, xtra);
        if do_buf_event {
            let mut num_added: int64_t = (lnume + xtra - lnum) as int64_t;
            let mut num_removed: int64_t = (lnume - lnum) as int64_t;
            buf_updates_send_changes(buf, lnum, num_added, num_removed);
        }
    }
}
