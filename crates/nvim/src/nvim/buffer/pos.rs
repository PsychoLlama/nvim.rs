//! Where the cursor was -- the per-window remembered position.
//!
//! Every window remembers, for every buffer it has shown, the cursor position
//! and the topline it was at, in a `wininfo_T`.  [`buflist_setfpos`] records
//! one, [`find_wininfo`] picks the entry to restore (preferring this window,
//! then this tab page, then any), [`get_winopts`] restores the window-local
//! options and folds along with it, and [`buflist_findfmark`] answers the
//! same question for a mark rather than a window.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::fold::{clearFolding, cloneFoldGrowArray, deleteFoldRecurse};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curtab, curwin, firstwin, p_fdls};
use crate::src::nvim::mark::mark_view_make;
use crate::src::nvim::memory::{xcalloc, xrealloc};
use crate::src::nvim::option::{clear_winopt, copy_winopt, didset_window_options};
use crate::src::nvim::os::libc::memmove;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::types::{
    AdditionalData, OptInt, Timestamp, WinInfo, buf_T, colnr_T, fmark_T, fmarkv_T, linenr_T, pos_T,
    size_t, win_T,
};
use crate::src::nvim::winfloat::win_set_minimal_style;

pub unsafe extern "C" fn buflist_setfpos(
    buf: *mut buf_T,
    win: *mut win_T,
    mut lnum: linenr_T,
    mut col: colnr_T,
    mut copy_options: bool,
) {
    unsafe {
        let mut wip: *mut WinInfo = ::core::ptr::null_mut::<WinInfo>();
        let mut i: size_t = 0;
        i = 0 as size_t;
        while i < (*buf).b_wininfo.size {
            wip = *(*buf).b_wininfo.items.add(i);
            if (*wip).wi_win == win {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == (*buf).b_wininfo.size {
            wip = xcalloc(1 as size_t, ::core::mem::size_of::<WinInfo>()) as *mut WinInfo;
            (*wip).wi_win = win;
            if lnum == 0 as linenr_T {
                lnum = 1 as ::core::ffi::c_int as linenr_T;
            }
        } else {
            (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_sub(1 as size_t);
            (i < (*buf).b_wininfo.size
                && !memmove(
                    (*buf).b_wininfo.items.add(i) as *mut ::core::ffi::c_void,
                    (*buf).b_wininfo.items.add(i.wrapping_add(1 as size_t))
                        as *const ::core::ffi::c_void,
                    (*buf)
                        .b_wininfo
                        .size
                        .wrapping_sub(i)
                        .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
                )
                .is_null()) as ::core::ffi::c_int;
            if copy_options as ::core::ffi::c_int != 0
                && (*wip).wi_optset as ::core::ffi::c_int != 0
            {
                clear_winopt(&raw mut (*wip).wi_opt);
                deleteFoldRecurse(&raw mut (*wip).wi_folds);
            }
        }
        if lnum != 0 as linenr_T {
            (*wip).wi_mark.mark.lnum = lnum;
            (*wip).wi_mark.mark.col = col;
            if !win.is_null() {
                (*wip).wi_mark.view = mark_view_make(win, (*wip).wi_mark.mark);
            }
        }
        if !win.is_null() {
            (*wip).wi_changelistidx = (*win).w_changelistidx;
        }
        if copy_options as ::core::ffi::c_int != 0 && !win.is_null() {
            copy_winopt(&raw mut (*win).w_onebuf_opt, &raw mut (*wip).wi_opt);
            (*wip).wi_fold_manual = (*win).w_fold_manual;
            cloneFoldGrowArray(&raw mut (*win).w_folds, &raw mut (*wip).wi_folds);
            (*wip).wi_optset = true_0 != 0;
        }
        if (*buf).b_wininfo.size == (*buf).b_wininfo.capacity {
            (*buf).b_wininfo.capacity = if (*buf).b_wininfo.capacity != 0 {
                (*buf).b_wininfo.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*buf).b_wininfo.items = xrealloc(
                (*buf).b_wininfo.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<*mut WinInfo>().wrapping_mul((*buf).b_wininfo.capacity),
            ) as *mut *mut WinInfo;
        } else {
        };
        (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_add(1);
        memmove(
            (*buf)
                .b_wininfo
                .items
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (*buf)
                .b_wininfo
                .items
                .offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            (*buf)
                .b_wininfo
                .size
                .wrapping_sub(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
        );
        *(*buf)
            .b_wininfo
            .items
            .offset(0 as ::core::ffi::c_int as isize) = wip;
    }
}

unsafe extern "C" fn wininfo_other_tab_diff(mut wip: *mut WinInfo) -> bool {
    unsafe {
        if (*wip).wi_opt.wo_diff == 0 {
            return false_0 != 0;
        }
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wip).wi_win == wp {
                return false_0 != 0;
            }
            wp = (*wp).w_next;
        }
        return true_0 != 0;
    }
}

unsafe extern "C" fn find_wininfo(
    mut buf: *mut buf_T,
    mut need_options: bool,
    mut skip_diff_buffer: bool,
) -> *mut WinInfo {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < (*buf).b_wininfo.size {
            let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.add(i);
            if (*wip).wi_win == curwin.get()
                && (!skip_diff_buffer || !wininfo_other_tab_diff(wip))
                && (!need_options || (*wip).wi_optset as ::core::ffi::c_int != 0)
            {
                return wip;
            }
            i = i.wrapping_add(1);
        }
        if skip_diff_buffer {
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*buf).b_wininfo.size {
                let mut wip_0: *mut WinInfo = *(*buf).b_wininfo.items.add(i_0);
                if !wininfo_other_tab_diff(wip_0)
                    && (!need_options
                        || (*wip_0).wi_optset as ::core::ffi::c_int != 0
                        || !(*wip_0).wi_win.is_null() && (*(*wip_0).wi_win).w_buffer == buf)
                {
                    return wip_0;
                }
                i_0 = i_0.wrapping_add(1);
            }
        } else if (*buf).b_wininfo.size != 0 {
            return *(*buf)
                .b_wininfo
                .items
                .offset(0 as ::core::ffi::c_int as isize);
        }
        return ::core::ptr::null_mut::<WinInfo>();
    }
}

pub unsafe extern "C" fn get_winopts(mut buf: *mut buf_T) {
    unsafe {
        clear_winopt(&raw mut (*curwin.get()).w_onebuf_opt);
        clearFolding(curwin.get());
        let wip: *mut WinInfo = find_wininfo(buf, true_0 != 0, true_0 != 0);
        if !wip.is_null()
            && (*wip).wi_win != curwin.get()
            && !(*wip).wi_win.is_null()
            && (*(*wip).wi_win).w_buffer == buf
            && (*(*wip).wi_win).w_config.style as ::core::ffi::c_uint
                != kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut wp: *mut win_T = (*wip).wi_win;
            copy_winopt(
                &raw mut (*wp).w_onebuf_opt,
                &raw mut (*curwin.get()).w_onebuf_opt,
            );
            (*curwin.get()).w_fold_manual = (*wp).w_fold_manual;
            (*curwin.get()).w_foldinvalid = true_0 != 0;
            cloneFoldGrowArray(&raw mut (*wp).w_folds, &raw mut (*curwin.get()).w_folds);
        } else if !wip.is_null()
            && (*wip).wi_optset as ::core::ffi::c_int != 0
            && ((*wip).wi_win.is_null()
                || (*wip).wi_win == curwin.get()
                || (*(*wip).wi_win).w_config.style as ::core::ffi::c_uint
                    != kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            copy_winopt(
                &raw mut (*wip).wi_opt,
                &raw mut (*curwin.get()).w_onebuf_opt,
            );
            (*curwin.get()).w_fold_manual = (*wip).wi_fold_manual;
            (*curwin.get()).w_foldinvalid = true_0 != 0;
            cloneFoldGrowArray(&raw mut (*wip).wi_folds, &raw mut (*curwin.get()).w_folds);
        } else {
            copy_winopt(
                &raw mut (*curwin.get()).w_allbuf_opt,
                &raw mut (*curwin.get()).w_onebuf_opt,
            );
        }
        if !wip.is_null() {
            (*curwin.get()).w_changelistidx = (*wip).wi_changelistidx;
        }
        if (*curwin.get()).w_config.style as ::core::ffi::c_uint
            == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            didset_window_options(curwin.get(), false_0 != 0);
            win_set_minimal_style(curwin.get());
        }
        if p_fdls.get() >= 0 as OptInt {
            (*curwin.get()).w_onebuf_opt.wo_fdl = p_fdls.get();
        }
        didset_window_options(curwin.get(), false_0 != 0);
    }
}

pub unsafe extern "C" fn buflist_findfmark(mut buf: *mut buf_T) -> *mut fmark_T {
    unsafe {
        static no_position: GlobalCell<fmark_T> = GlobalCell::new(fmark_T {
            mark: pos_T {
                lnum: 1 as linenr_T,
                col: 0 as colnr_T,
                coladd: 0 as colnr_T,
            },
            fnum: 0 as ::core::ffi::c_int,
            timestamp: 0 as Timestamp,
            view: fmarkv_T {
                topline_offset: MAXLNUM as ::core::ffi::c_int as linenr_T,
                skipcol: 0 as colnr_T,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        });
        let wip: *mut WinInfo = find_wininfo(buf, false_0 != 0, false_0 != 0);
        return if wip.is_null() {
            no_position.ptr()
        } else {
            &raw mut (*wip).wi_mark
        };
    }
}

pub unsafe extern "C" fn buflist_findlnum(mut buf: *mut buf_T) -> linenr_T {
    unsafe {
        return (*buflist_findfmark(buf)).mark.lnum;
    }
}
