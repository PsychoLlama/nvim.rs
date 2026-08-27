//! Making another window current for the duration of a call, which is what
//! `win_execute()` and the API's window-scoped entry points use.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::pos::equalpos;
use crate::types::VAR_STRING;

/// Switch to a window for executing user code.
///
/// The caller must call [`win_execute_after`] afterwards whatever the answer
/// is, because the saved state is written before the switch is attempted.
///
/// # Safety
/// `args` must point at a writable `win_execute_T`, and `wp`/`tp` must be a
/// live window and tab page.
pub unsafe fn win_execute_before(
    args: *mut win_execute_T,
    wp: *mut win_T,
    tp: *mut tabpage_T,
) -> bool {
    // SAFETY: the caller's obligation. `args` is the caller's own storage and
    // nothing below can reach it, so the exclusive borrow is sound; `autocwd`
    // is a live local and `os_dirname` fills at most `MAXPATHL` bytes.
    let (args, win, tab) = unsafe { (&mut *args, Win::new(wp), TabPage::new(tp)) };
    args.wp = wp;
    args.curpos = win.w_cursor;
    args.cwd_status = FAIL;
    args.apply_acd = false;
    args.save_sfname = ptr::null_mut();
    // SAFETY: live window and tab page handles, and the globals they are
    // compared against are set from startup to exit.
    // The working directory only has to be saved when running the code
    // there could change it: a different window or tab page with a
    // `:lcd`/`:tcd` of its own, or 'autochdir'.
    if !win.is_current()
        && (!cur_win().w_localdir.is_null()
            || !win.w_localdir.is_null()
            || !tab.is_current()
                && (!unsafe { (*curtab.get()).tp_localdir }.is_null()
                    || !tab.tp_localdir.is_null())
            || p_acd.get() != 0)
    {
        args.cwd_status = unsafe { os_dirname(args.cwd.as_mut_ptr(), size_of_val(&args.cwd)) };
    }
    if args.cwd_status == OK && p_acd.get() != 0 {
        // 'autochdir' will move the working directory itself when the
        // window is entered; `apply_acd` records that it has already
        // landed where the saved one says, so the restore can skip it.
        let buf = cur_buf();
        if !buf.b_sfname.is_null() && buf.b_fname == buf.b_sfname {
            args.save_sfname = unsafe { xstrdup(buf.b_sfname) };
        }
        do_autochdir();
        let mut autocwd: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        if unsafe { os_dirname(autocwd.as_mut_ptr(), size_of_val(&autocwd)) } == OK {
            args.apply_acd = unsafe { strcmp(args.cwd.as_mut_ptr(), autocwd.as_mut_ptr()) } == 0;
        }
    }
    if unsafe { switch_win_noblock(&raw mut args.switchwin, wp, tp, true) } == OK {
        check_cursor(cur_win());
        return true;
    }
    false
}

/// Restore the previous window after executing user code.
///
/// # Safety
/// `args` must be the value [`win_execute_before`] was handed.
pub unsafe fn win_execute_after(args: *mut win_execute_T) {
    // SAFETY: the caller's obligation. `args` is the caller's own storage and
    // nothing below can reach it; `win_valid` re-checks the saved window,
    // because the code that ran may have closed it.
    let args = unsafe { &mut *args };
    unsafe { restore_win_noblock(&raw mut args.switchwin, true) };
    if args.apply_acd {
        unsafe { xfree(args.save_sfname.cast()) };
        do_autochdir();
    } else if args.cwd_status == OK {
        unsafe { os_chdir(args.cwd.as_mut_ptr()) };
        if !args.save_sfname.is_null() {
            let mut buf = cur_buf();
            unsafe { xfree(buf.b_sfname.cast()) };
            buf.b_sfname = args.save_sfname;
            buf.b_fname = buf.b_sfname;
        }
    }
    if win_valid(args.wp) {
        let mut win = unsafe { Win::new(args.wp) };
        if !equalpos(args.curpos, win.w_cursor) {
            win.w_redr_status = true;
        }
    }
    check_cursor(cur_win());
    if visual_active() {
        with_visual_anchor(|anchor| check_pos(cur_buf(), anchor));
    }
}

/// `win_execute({winid}, {command} [, {silent}])`.
pub unsafe fn f_win_execute(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the arguments and `rettv` are live typvals; the saved state is a
    // live local that `win_execute_after` is given whatever happens between.
    let id = number_as_int(arg_number(args, 0));
    let Some((wp, tp)) = win_and_tab_by_id(id) else {
        return;
    };
    let mut saved: win_execute_T = unsafe { mem::zeroed() };
    if unsafe { win_execute_before(&raw mut saved, wp.raw(), tp.raw()) } {
        unsafe { execute_common(argvars, rettv, 1) };
    }
    unsafe { win_execute_after(&raw mut saved) };
}

/// Make `win` the current window and `tp` the current tab page.
///
/// [`restore_win`] MUST be called to undo this, `FAIL` included. No
/// autocommands run until it is.
///
/// `no_display` keeps the display untouched: no redraw is triggered and
/// another tab page is only half entered.
///
/// # Safety
/// `switchwin` must be writable, `win` a live window and `tp` a live tab page
/// or NULL.
pub unsafe fn switch_win(
    switchwin: *mut switchwin_T,
    win: *mut win_T,
    tp: *mut tabpage_T,
    no_display: bool,
) -> c_int {
    // SAFETY: the caller's obligation.
    unsafe { block_autocmds() };
    unsafe { switch_win_noblock(switchwin, win, tp, no_display) }
}

/// [`switch_win`] without blocking autocommands.
///
/// # Safety
/// As [`switch_win`].
pub unsafe fn switch_win_noblock(
    switchwin: *mut switchwin_T,
    win: *mut win_T,
    tp: *mut tabpage_T,
    no_display: bool,
) -> c_int {
    // SAFETY: the caller's obligation. `switchwin` is the caller's own
    // storage and nothing below can reach it, so the exclusive borrow is
    // sound; all-zero is a valid `switchwin_T`.
    unsafe { memset(switchwin.cast(), 0, size_of::<switchwin_T>()) };
    let switchwin = unsafe { &mut *switchwin };
    switchwin.sw_curwin = curwin.get();
    if win == curwin.get() {
        switchwin.sw_same_win = true;
    } else {
        // A Visual selection belongs to the window it was made in.
        switchwin.sw_visual_active = visual_active();
        set_visual_active(false);
    }
    // SAFETY: a live tab page or NULL, and `win_valid` re-checks the window
    // before it is entered -- entering the tab page can close it.
    if !tp.is_null() {
        switchwin.sw_curtab = curtab.get();
        if no_display {
            unsafe { unuse_tabpage(curtab.get()) };
            unsafe { use_tabpage(tp) };
        } else {
            unsafe { goto_tabpage_tp(tp, false, false) };
        }
    }
    if !win_valid(win) {
        return FAIL;
    }
    curwin.set(win);
    curbuf.set(unsafe { Win::new(win) }.w_buffer);
    OK
}

/// Restore the tab page and window [`switch_win`] saved, if they are still
/// valid.
///
/// # Safety
/// `switchwin` must be the value [`switch_win`] was handed.
pub unsafe fn restore_win(switchwin: *mut switchwin_T, no_display: bool) {
    // SAFETY: the caller's obligation.
    unsafe { restore_win_noblock(switchwin, no_display) };
    unsafe { unblock_autocmds() };
}

/// [`restore_win`] without unblocking autocommands.
///
/// # Safety
/// As [`restore_win`].
pub unsafe fn restore_win_noblock(switchwin: *mut switchwin_T, no_display: bool) {
    // SAFETY: the caller's obligation. `switchwin` is the caller's own
    // storage and nothing below can reach it; both saved pointers are
    // re-checked before being entered, because the code that ran may have
    // closed them.
    let switchwin = unsafe { &mut *switchwin };
    if !switchwin.sw_curtab.is_null() && valid_tabpage(switchwin.sw_curtab) {
        if no_display {
            // `unuse_tabpage` writes the current window back into the tab
            // page it is leaving; that is the wrong window here, because
            // the caller only half entered this one.
            let mut leaving = cur_tab();
            let old_tp_curwin = leaving.tp_curwin;
            unsafe { unuse_tabpage(leaving.raw()) };
            leaving.tp_curwin = old_tp_curwin;
            unsafe { use_tabpage(switchwin.sw_curtab) };
        } else {
            unsafe { goto_tabpage_tp(switchwin.sw_curtab, false, false) };
        }
    }
    if !switchwin.sw_same_win {
        set_visual_active(switchwin.sw_visual_active);
    }
    // SAFETY: the saved window is live or freed, which `win_valid` tells
    // apart, and a live window's buffer is live.
    if win_valid(switchwin.sw_curwin) {
        curwin.set(switchwin.sw_curwin);
        curbuf.set(unsafe { Win::new(switchwin.sw_curwin) }.w_buffer);
    }
}
