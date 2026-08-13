//! Entering a window -- `win_goto()`, `win_enter()` and the directional
//! moves.
//!
//! [`win_enter_ext`] is the one that actually changes `curwin`: it fires
//! `WinLeave`/`BufLeave` and `WinEnter`/`BufEnter`, syncs undo, updates the
//! window-local directory ([`win_fix_current_dir`]) and revalidates the
//! cursor -- and every one of those may close the window it was entering.
//! [`win_vert_neighbor`] and [`win_horz_neighbor`] answer which window lies
//! in a given direction, and the `buf_jump_open_*` pair finds a window
//! already showing a buffer.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_WINENTER, EVENT_WINLEAVE, EVENT_WINNEW, apply_autocmds,
};
use crate::src::nvim::buffer::{do_autochdir, maketitle};
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_later, redrawWinline};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::ex_getln::text_or_buf_locked;
use crate::src::nvim::file_search::do_autocmd_dirchanged;
use crate::src::nvim::fileio::shorten_fnames;
use crate::src::nvim::getchar::beep_flush;
use crate::src::nvim::main::{
    VIsual_active, curbuf, curtab, curwin, first_tabpage, firstwin, globaldir, last_chdir_reason,
    msg_scrolled, p_acd, p_spk, p_wh, p_wiw, prevwin, redraw_tabline, restart_edit,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::mouse::setmouse;
use crate::src::nvim::r#move::{changed_line_abv_curs, update_topline};
use crate::src::nvim::normal::reset_VIsual_and_resel;
use crate::src::nvim::option::buf_copy_options;
use crate::src::nvim::os::fs::{os_chdir, os_dirname};
use crate::src::nvim::path::pathcmp;
use crate::src::nvim::state::{
    MODE_CMDLINE, MODE_NORMAL, MODE_TERMINAL, get_real_state, virtual_active,
};
use crate::src::nvim::types::{
    CdScope, OptInt, buf_T, colnr_T, frame_T, kCdScopeGlobal, kCdScopeTabpage, kCdScopeWindow,
    size_t, tabpage_T, win_T,
};
use crate::src::nvim::undo::u_sync;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_goto(mut wp: *mut win_T) {
    unsafe {
        let mut owp: *mut win_T = curwin.get();
        if text_or_buf_locked() {
            beep_flush();
            return;
        }
        if (*wp).w_buffer != curbuf.get() {
            reset_VIsual_and_resel();
        } else if VIsual_active.get() {
            (*wp).w_cursor = (*curwin.get()).w_cursor;
        }
        if !win_valid(wp) {
            return;
        }
        win_enter(wp, true_0 != 0);
        if win_valid(owp) as ::core::ffi::c_int != 0
            && (*owp).w_onebuf_opt.wo_cole > 0 as OptInt
            && msg_scrolled.get() == 0
        {
            redrawWinline(owp, (*owp).w_cursor.lnum);
        }
        if (*curwin.get()).w_onebuf_opt.wo_cole > 0 as OptInt && msg_scrolled.get() == 0 {
            redrawWinline(curwin.get(), (*curwin.get()).w_cursor.lnum);
        }
    }
}

pub unsafe extern "C" fn win_find_tabpage(mut win: *mut win_T) -> *mut tabpage_T {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if wp == win {
                    return tp as *mut tabpage_T;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return ::core::ptr::null_mut::<tabpage_T>();
    }
}

pub unsafe extern "C" fn win_vert_neighbor(
    mut tp: *mut tabpage_T,
    mut wp: *mut win_T,
    mut up: bool,
    mut count: ::core::ffi::c_int,
) -> *mut win_T {
    unsafe {
        let mut foundfr: *mut frame_T = (*wp).w_frame;
        if (*wp).w_floating {
            return if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                && !(*prevwin.get()).w_floating
            {
                prevwin.get()
            } else {
                firstwin.get()
            };
        }
        '_end: loop {
            let c2rust_fresh2 = count;
            count = count - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            let mut nfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            let mut fr: *mut frame_T = foundfr;
            loop {
                if fr == (*tp).tp_topframe {
                    break '_end;
                }
                if up {
                    nfr = (*fr).fr_prev;
                } else {
                    nfr = (*fr).fr_next;
                }
                if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL && !nfr.is_null() {
                    break;
                }
                fr = (*fr).fr_parent;
            }
            loop {
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
                    foundfr = nfr;
                    break;
                } else {
                    fr = (*nfr).fr_child;
                    if (*nfr).fr_layout as ::core::ffi::c_int == FR_ROW {
                        while !(*fr).fr_next.is_null()
                            && (*frame2win(fr)).w_wincol + (*fr).fr_width
                                <= (*wp).w_wincol + (*wp).w_wcol
                        {
                            fr = (*fr).fr_next;
                        }
                    }
                    if (*nfr).fr_layout as ::core::ffi::c_int == FR_COL
                        && up as ::core::ffi::c_int != 0
                    {
                        while !(*fr).fr_next.is_null() {
                            fr = (*fr).fr_next;
                        }
                    }
                    nfr = fr;
                }
            }
        }
        return if !foundfr.is_null() {
            (*foundfr).fr_win
        } else {
            ::core::ptr::null_mut::<win_T>()
        };
    }
}

pub(crate) unsafe extern "C" fn win_goto_ver(mut up: bool, mut count: ::core::ffi::c_int) {
    unsafe {
        let mut win: *mut win_T = win_vert_neighbor(curtab.get(), curwin.get(), up, count);
        if !win.is_null() {
            win_goto(win);
        }
    }
}

pub unsafe extern "C" fn win_horz_neighbor(
    mut tp: *mut tabpage_T,
    mut wp: *mut win_T,
    mut left: bool,
    mut count: ::core::ffi::c_int,
) -> *mut win_T {
    unsafe {
        let mut foundfr: *mut frame_T = (*wp).w_frame;
        if (*wp).w_floating {
            return if win_valid(prevwin.get()) as ::core::ffi::c_int != 0
                && !(*prevwin.get()).w_floating
            {
                prevwin.get()
            } else {
                firstwin.get()
            };
        }
        '_end: loop {
            let c2rust_fresh1 = count;
            count = count - 1;
            if c2rust_fresh1 == 0 {
                break;
            }
            let mut nfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            let mut fr: *mut frame_T = foundfr;
            loop {
                if fr == (*tp).tp_topframe {
                    break '_end;
                }
                if left {
                    nfr = (*fr).fr_prev;
                } else {
                    nfr = (*fr).fr_next;
                }
                if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW && !nfr.is_null() {
                    break;
                }
                fr = (*fr).fr_parent;
            }
            loop {
                if (*nfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
                    foundfr = nfr;
                    break;
                } else {
                    fr = (*nfr).fr_child;
                    if (*nfr).fr_layout as ::core::ffi::c_int == FR_COL {
                        while !(*fr).fr_next.is_null()
                            && (*frame2win(fr)).w_winrow + (*fr).fr_height
                                <= (*wp).w_winrow + (*wp).w_wrow
                        {
                            fr = (*fr).fr_next;
                        }
                    }
                    if (*nfr).fr_layout as ::core::ffi::c_int == FR_ROW
                        && left as ::core::ffi::c_int != 0
                    {
                        while !(*fr).fr_next.is_null() {
                            fr = (*fr).fr_next;
                        }
                    }
                    nfr = fr;
                }
            }
        }
        return if !foundfr.is_null() {
            (*foundfr).fr_win
        } else {
            ::core::ptr::null_mut::<win_T>()
        };
    }
}

pub(crate) unsafe extern "C" fn win_goto_hor(mut left: bool, mut count: ::core::ffi::c_int) {
    unsafe {
        let mut win: *mut win_T = win_horz_neighbor(curtab.get(), curwin.get(), left, count);
        if !win.is_null() {
            win_goto(win);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn win_enter(mut wp: *mut win_T, mut undo_sync: bool) {
    unsafe {
        win_enter_ext(
            wp,
            (if undo_sync as ::core::ffi::c_int != 0 {
                WEE_UNDO_SYNC as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
        );
    }
}

pub(crate) unsafe extern "C" fn win_enter_ext(wp: *mut win_T, flags: ::core::ffi::c_int) {
    unsafe {
        let mut other_buffer: bool = false_0 != 0;
        let curwin_invalid: bool = flags & WEE_CURWIN_INVALID as ::core::ffi::c_int != 0;
        if wp == curwin.get() && !curwin_invalid {
            return;
        }
        if !curwin_invalid {
            leaving_window(curwin.get());
        }
        if !curwin_invalid && flags & WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int != 0 {
            if (*wp).w_buffer != curbuf.get() {
                apply_autocmds(
                    EVENT_BUFLEAVE,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
                other_buffer = true_0 != 0;
                if !win_valid(wp) {
                    return;
                }
            }
            apply_autocmds(
                EVENT_WINLEAVE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if !win_valid(wp) {
                return;
            }
            if aborting() {
                return;
            }
        }
        if flags & WEE_UNDO_SYNC as ::core::ffi::c_int != 0 && curbuf.get() != (*wp).w_buffer {
            u_sync(false_0 != 0);
        }
        if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int && !curwin_invalid {
            update_topline(curwin.get());
        }
        if (*wp).w_buffer != curbuf.get() {
            buf_copy_options(
                (*wp).w_buffer,
                BCO_ENTER as ::core::ffi::c_int | BCO_NOHELP as ::core::ffi::c_int,
            );
        }
        if !curwin_invalid {
            prevwin.set(curwin.get());
            (*curwin.get()).w_redr_status = true_0 != 0;
        }
        curwin.set(wp);
        curbuf.set((*wp).w_buffer);
        check_cursor(curwin.get());
        if !virtual_active(curwin.get()) {
            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
        }
        if *p_spk.get() as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
            changed_line_abv_curs();
        } else {
            win_fix_cursor(get_real_state() & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL) != 0);
        }
        win_fix_current_dir();
        entering_window(curwin.get());
        if flags & WEE_TRIGGER_NEW_AUTOCMDS as ::core::ffi::c_int != 0 {
            apply_autocmds(
                EVENT_WINNEW,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        if flags & WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int != 0 {
            apply_autocmds(
                EVENT_WINENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
            if other_buffer {
                apply_autocmds(
                    EVENT_BUFENTER,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                );
            }
        }
        maketitle();
        (*curwin.get()).w_redr_status = true_0 != 0;
        redraw_tabline.set(true_0 != 0);
        if restart_edit.get() != 0 {
            redraw_later(curwin.get(), UPD_VALID);
        }
        if (*curwin.get()).w_hl_attr_normal != (*curwin.get()).w_hl_attr_normalnc {
            redraw_later(curwin.get(), UPD_NOT_VALID);
        }
        if !(*prevwin.ptr()).is_null() {
            if (*prevwin.get()).w_hl_attr_normal != (*prevwin.get()).w_hl_attr_normalnc {
                redraw_later(prevwin.get(), UPD_NOT_VALID);
            }
        }
        if ((*curwin.get()).w_height as OptInt) < p_wh.get()
            && (*curwin.get()).w_onebuf_opt.wo_wfh == 0
            && !(*curwin.get()).w_floating
        {
            win_setheight(p_wh.get() as ::core::ffi::c_int);
        } else if (*curwin.get()).w_height == 0 as ::core::ffi::c_int {
            win_setheight(1 as ::core::ffi::c_int);
        }
        if ((*curwin.get()).w_width as OptInt) < p_wiw.get()
            && (*curwin.get()).w_onebuf_opt.wo_wfw == 0
            && !(*curwin.get()).w_floating
        {
            win_setwidth(p_wiw.get() as ::core::ffi::c_int);
        }
        setmouse();
        do_autochdir();
    }
}

pub unsafe extern "C" fn win_fix_current_dir() {
    unsafe {
        let mut new_dir: *mut ::core::ffi::c_char = if !(*curwin.get()).w_localdir.is_null() {
            (*curwin.get()).w_localdir
        } else {
            (*curtab.get()).tp_localdir
        };
        let mut cwd: [::core::ffi::c_char; 4096] = [0; 4096];
        if os_dirname(&raw mut cwd as *mut ::core::ffi::c_char, MAXPATHL as size_t) != OK {
            cwd[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        if !new_dir.is_null() {
            if (*globaldir.ptr()).is_null() {
                if cwd[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL {
                    globaldir.set(xstrdup(&raw mut cwd as *mut ::core::ffi::c_char));
                }
            }
            let mut dir_differs: bool = pathcmp(
                new_dir,
                &raw mut cwd as *mut ::core::ffi::c_char,
                -1 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int;
            if p_acd.get() == 0 && dir_differs as ::core::ffi::c_int != 0 {
                do_autocmd_dirchanged(
                    new_dir,
                    (if !(*curwin.get()).w_localdir.is_null() {
                        kCdScopeWindow as ::core::ffi::c_int
                    } else {
                        kCdScopeTabpage as ::core::ffi::c_int
                    }) as CdScope,
                    kCdCauseWindow,
                    true_0 != 0,
                );
            }
            if os_chdir(new_dir) == 0 as ::core::ffi::c_int {
                if p_acd.get() == 0 && dir_differs as ::core::ffi::c_int != 0 {
                    do_autocmd_dirchanged(
                        new_dir,
                        (if !(*curwin.get()).w_localdir.is_null() {
                            kCdScopeWindow as ::core::ffi::c_int
                        } else {
                            kCdScopeTabpage as ::core::ffi::c_int
                        }) as CdScope,
                        kCdCauseWindow,
                        false_0 != 0,
                    );
                }
            }
            last_chdir_reason.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            shorten_fnames(true_0);
        } else if !(*globaldir.ptr()).is_null() {
            let mut dir_differs_0: bool = pathcmp(
                globaldir.get(),
                &raw mut cwd as *mut ::core::ffi::c_char,
                -1 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int;
            if p_acd.get() == 0 && dir_differs_0 as ::core::ffi::c_int != 0 {
                do_autocmd_dirchanged(globaldir.get(), kCdScopeGlobal, kCdCauseWindow, true_0 != 0);
            }
            if os_chdir(globaldir.get()) == 0 as ::core::ffi::c_int {
                if p_acd.get() == 0 && dir_differs_0 as ::core::ffi::c_int != 0 {
                    do_autocmd_dirchanged(
                        globaldir.get(),
                        kCdScopeGlobal,
                        kCdCauseWindow,
                        false_0 != 0,
                    );
                }
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                globaldir.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            last_chdir_reason.set(::core::ptr::null_mut::<::core::ffi::c_char>());
            shorten_fnames(true_0);
        }
    }
}

pub unsafe extern "C" fn buf_jump_open_win(mut buf: *mut buf_T) -> *mut win_T {
    unsafe {
        if (*curwin.get()).w_buffer == buf {
            win_enter(curwin.get(), false_0 != 0);
            return curwin.get();
        }
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                win_enter(wp, false_0 != 0);
                return wp;
            }
            wp = (*wp).w_next;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}

pub unsafe extern "C" fn buf_jump_open_tab(mut buf: *mut buf_T) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = buf_jump_open_win(buf);
        if !wp.is_null() {
            return wp;
        }
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if tp != curtab.get() {
                let mut wp_0: *mut win_T = if tp == curtab.get() {
                    firstwin.get()
                } else {
                    (*tp).tp_firstwin
                };
                while !wp_0.is_null() {
                    if (*wp_0).w_buffer == buf {
                        goto_tabpage_win(tp as *mut tabpage_T, wp_0);
                        if curwin.get() != wp_0 {
                            wp_0 = ::core::ptr::null_mut::<win_T>();
                        }
                        return wp_0;
                    }
                    wp_0 = (*wp_0).w_next;
                }
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        return ::core::ptr::null_mut::<win_T>();
    }
}
