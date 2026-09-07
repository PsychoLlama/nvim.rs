//! Making another window current for the duration of a call, which is what
//! `win_execute()` and the API's window-scoped entry points use.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::pos::equalpos;
use crate::types::Failed;
use crate::types::VAR_STRING;
use crate::window::valid_tab;
use crate::window::valid_win;

/// Switch to a window for executing user code.
///
/// The caller must call [`win_execute_after`] afterwards whatever the answer
/// is, because the saved state is written before the switch is attempted.
///
/// # Safety
/// `args` must point at a writable `WinExecute`, and `window`/`tabpage` must be a
/// live window and tab page.
pub unsafe fn win_execute_before(args: *mut WinExecute, window: Win, tabpage: TabPage) -> bool {
    // SAFETY: the caller's obligation. `args` is the caller's own storage and
    // nothing below can reach it, so the exclusive borrow is sound; `autocwd`
    // is a live local and `os_dirname` fills at most `MAXPATHL` bytes.
    let (args, win, tab) = unsafe { (&mut *args, window, tabpage) };
    args.wp = Some(window.id());
    args.curpos = win.w_cursor;
    args.cwd_status = Err(Failed);
    args.apply_acd = false;
    args.save_sfname = ptr::null_mut();
    if !win.is_current()
        && (!Win::current().w_localdir.is_null()
            || !win.w_localdir.is_null()
            || !tab.is_current()
                && (!TabPage::current().tp_localdir.is_null() || !tab.tp_localdir.is_null())
            || p_acd.get() != 0)
    {
        args.cwd_status = unsafe { os_dirname(args.cwd.as_mut_ptr(), size_of_val(&args.cwd)) };
    }
    if args.cwd_status.is_ok() && p_acd.get() != 0 {
        // 'autochdir' will move the working directory itself when the
        // window is entered; `apply_acd` records that it has already
        // landed where the saved one says, so the restore can skip it.
        let buf = Buf::current();
        if !buf.b_sfname.is_null() && buf.b_fname == buf.b_sfname {
            args.save_sfname = unsafe { xstrdup(buf.b_sfname) };
        }
        do_autochdir();
        let mut autocwd: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        if unsafe { os_dirname(autocwd.as_mut_ptr(), size_of_val(&autocwd)) }.is_ok() {
            args.apply_acd = unsafe { cstr::eq(args.cwd.as_mut_ptr(), autocwd.as_mut_ptr()) };
        }
    }
    if unsafe { switch_win_noblock(&raw mut args.switchwin, window, Some(tabpage), true) }.is_ok() {
        check_cursor(Win::current());
        return true;
    }
    false
}

/// Restore the previous window after executing user code.
///
/// # Safety
/// `args` must be the value [`win_execute_before`] was handed.
pub unsafe fn win_execute_after(args: *mut WinExecute) {
    // SAFETY: the caller's obligation. `args` is the caller's own storage and
    // nothing below can reach it; `win_valid` re-checks the saved window,
    // because the code that ran may have closed it.
    let args = unsafe { &mut *args };
    unsafe { restore_win_noblock(&raw mut args.switchwin, true) };
    if args.apply_acd {
        unsafe { xfree(args.save_sfname.cast()) };
        do_autochdir();
    } else if args.cwd_status.is_ok() {
        unsafe { os_chdir(args.cwd.as_mut_ptr()) };
        if !args.save_sfname.is_null() {
            let mut buf = Buf::current();
            unsafe { xfree(buf.b_sfname.cast()) };
            buf.b_sfname = args.save_sfname;
            buf.b_fname = buf.b_sfname;
        }
    }
    if let Some(mut win) = args.wp.and_then(valid_win)
        && !equalpos(args.curpos, win.w_cursor)
    {
        win.w_redr_status = true;
    }
    check_cursor(Win::current());
    if visual_active() {
        with_visual_anchor(|anchor| check_pos(Buf::current(), anchor));
    }
}

/// `win_execute({winid}, {command} [, {silent}])`.
pub unsafe fn f_win_execute(args: *mut TypVal, result: *mut TypVal, _fptr: EvalFuncData) {
    let (args, result) = frame!(args, result);
    result.v_type = VAR_STRING;
    result.vval.v_string = ptr::null_mut();
    // SAFETY: the arguments and `result` are live typvals; the saved state is a
    // live local that `win_execute_after` is given whatever happens between.
    let id = number_as_int(arg_number(args, 0));
    let Some((wp, tp)) = win_and_tab_by_id(id) else {
        return;
    };
    let mut saved: WinExecute = unsafe { mem::zeroed() };
    if unsafe { win_execute_before(&raw mut saved, wp, tp) } {
        unsafe { execute_common(args.ptr(0), result, 1) };
    }
    unsafe { win_execute_after(&raw mut saved) };
}

/// Make `win` the current window and `tabpage` the current tab page.
///
/// [`restore_win`] MUST be called to undo this, `Err` included. No
/// autocommands run until it is.
///
/// `no_display` keeps the display untouched: no redraw is triggered and
/// another tab page is only half entered.
///
/// # Safety
/// `switchwin` must be writable, `win` a live window and `tabpage` a live tab page
/// or NULL.
pub unsafe fn switch_win(
    switchwin: *mut SwitchWin,
    win: Win,
    tabpage: Option<TabPage>,
    no_display: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation.
    unsafe { block_autocmds() };
    unsafe { switch_win_noblock(switchwin, win, tabpage, no_display) }
}

/// [`switch_win`] without blocking autocommands.
///
/// # Safety
/// As [`switch_win`].
pub unsafe fn switch_win_noblock(
    switchwin: *mut SwitchWin,
    win: Win,
    tabpage: Option<TabPage>,
    no_display: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's obligation. `switchwin` is the caller's own
    // storage and nothing below can reach it, so the exclusive borrow is
    // sound; all-zero is a valid `SwitchWin`.
    let into = switchwin.cast::<u8>();
    unsafe { into.write_bytes(0, size_of::<SwitchWin>()) };
    let switchwin = unsafe { &mut *switchwin };
    switchwin.sw_curwin = Win::current_or_none().map(Win::id);
    if win.is_current() {
        switchwin.sw_same_win = true;
    } else {
        // A Visual selection belongs to the window it was made in.
        switchwin.sw_visual_active = visual_active();
        set_visual_active(false);
    }
    if let Some(tabpage) = tabpage {
        switchwin.sw_curtab = Some(TabPage::current().id());
        if no_display {
            unuse_tabpage(TabPage::current());
            use_tabpage(tabpage);
        } else {
            goto_tabpage_tp(tabpage, false, false);
        }
    }
    let Some(win) = valid_win(win.id()) else {
        return Err(Failed);
    };
    win.make_current();
    win.buffer().make_current();
    Ok(())
}

/// Restore the tab page and window [`switch_win`] saved, if they are still
/// valid.
///
/// # Safety
/// `switchwin` must be the value [`switch_win`] was handed.
pub unsafe fn restore_win(switchwin: *mut SwitchWin, no_display: bool) {
    // SAFETY: the caller's obligation.
    unsafe { restore_win_noblock(switchwin, no_display) };
    unsafe { unblock_autocmds() };
}

/// [`restore_win`] without unblocking autocommands.
///
/// # Safety
/// As [`restore_win`].
pub unsafe fn restore_win_noblock(switchwin: *mut SwitchWin, no_display: bool) {
    // SAFETY: the caller's obligation. `switchwin` is the caller's own
    // storage and nothing below can reach it; both saved pointers are
    // re-checked before being entered, because the code that ran may have
    // closed them.
    let switchwin = unsafe { &mut *switchwin };
    if let Some(back) = switchwin.sw_curtab.and_then(valid_tab) {
        if no_display {
            // `unuse_tabpage` writes the current window back into the tab
            // page it is leaving; that is the wrong window here, because
            // the caller only half entered this one.
            let mut leaving = TabPage::current();
            let old_tp_curwin = leaving.tp_curwin;
            unuse_tabpage(leaving);
            leaving.tp_curwin = old_tp_curwin;
            use_tabpage(back);
        } else {
            goto_tabpage_tp(back, false, false);
        }
    }
    if !switchwin.sw_same_win {
        set_visual_active(switchwin.sw_visual_active);
    }
    // The saved window is live or freed, which `valid_win` tells apart.
    if let Some(win) = switchwin.sw_curwin.and_then(valid_win) {
        win.make_current();
        win.buffer().make_current();
    }
}
