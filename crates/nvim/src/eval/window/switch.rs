//! Making another window current for the duration of a call, which is what
//! `win_execute()` and the API's window-scoped entry points use.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::XString;
use crate::normal::{set_visual_active, visual_active, with_visual_anchor};
use crate::os::fs::current_dir;
use crate::pos::equalpos;
use crate::types::Failed;
use crate::window::valid_tab;
use crate::window::valid_win;

/// Switch to a window for executing user code.
///
/// The caller must call [`win_execute_after`] afterwards whatever the answer
/// is, because the saved state is written before the switch is attempted.
pub fn win_execute_before(args: &mut WinExecute, window: Win, tabpage: TabPage) -> bool {
    args.wp = Some(window.id());
    args.curpos = window.w_cursor;
    args.cwd = None;
    args.apply_acd = false;
    args.save_sfname = None;
    if !window.is_current()
        && (!Win::current().w_localdir.is_null()
            || !window.w_localdir.is_null()
            || !tabpage.is_current()
                && (!TabPage::current().tp_localdir.is_null() || !tabpage.tp_localdir.is_null())
            || p_acd())
    {
        args.cwd = current_dir();
    }
    if let Some(cwd) = &args.cwd
        && p_acd()
    {
        // 'autochdir' will move the working directory itself when the
        // window is entered; `apply_acd` records that it has already
        // landed where the saved one says, so the restore can skip it.
        let buf = Buf::current();
        let short = buf.name.short();
        if short.is_some() && buf.name.shown().map(CStr::as_ptr) == short.map(CStr::as_ptr) {
            args.save_sfname = short.map(XString::from_cstr);
        }
        do_autochdir();
        if let Some(autocwd) = current_dir() {
            args.apply_acd = cwd.as_cstr() == autocwd.as_cstr();
        }
    }
    if switch_win_noblock(&mut args.switchwin, window, Some(tabpage), true).is_ok() {
        check_cursor(Win::current());
        return true;
    }
    false
}

/// Restore the previous window after executing user code.
///
/// `args` is the value [`win_execute_before`] was handed.
pub fn win_execute_after(args: &mut WinExecute) {
    restore_win_noblock(&mut args.switchwin, true);
    if args.apply_acd {
        args.save_sfname = None;
        do_autochdir();
    } else if let Some(cwd) = &args.cwd {
        os_chdir(cwd.as_cstr());
        if let Some(short) = args.save_sfname.take() {
            Buf::current().name.set_short(Some(short));
        }
    }
    // `valid_win` re-checks the saved window, because the code that ran may
    // have closed it.
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
pub fn f_win_execute(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    let id = number_as_int(arg_number(args, 0));
    let Some((wp, tp)) = win_and_tab_by_id(id) else {
        return;
    };
    let mut saved = WinExecute::default();
    if win_execute_before(&mut saved, wp, tp) {
        execute_common(args, result, 1);
    }
    win_execute_after(&mut saved);
}

/// Make `win` the current window and `tabpage` the current tab page.
///
/// [`restore_win`] MUST be called to undo this, `Err` included. No
/// autocommands run until it is.
///
/// `no_display` keeps the display untouched: no redraw is triggered and
/// another tab page is only half entered.
pub fn switch_win(
    switchwin: &mut SwitchWin,
    win: Win,
    tabpage: Option<TabPage>,
    no_display: bool,
) -> Result<(), Failed> {
    block_autocmds();
    switch_win_noblock(switchwin, win, tabpage, no_display)
}

/// [`switch_win`] without blocking autocommands.
pub fn switch_win_noblock(
    switchwin: &mut SwitchWin,
    win: Win,
    tabpage: Option<TabPage>,
    no_display: bool,
) -> Result<(), Failed> {
    *switchwin = SwitchWin::default();
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
pub fn restore_win(switchwin: &mut SwitchWin, no_display: bool) {
    restore_win_noblock(switchwin, no_display);
    unblock_autocmds();
}

/// [`restore_win`] without unblocking autocommands.
///
/// Both saved handles are re-checked before being entered, because the code
/// that ran may have closed them.
pub fn restore_win_noblock(switchwin: &mut SwitchWin, no_display: bool) {
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

/// A window made current by [`switch_win`] for as long as this value lives:
/// [`restore_win`] runs when it is dropped, whether the switch worked or not.
pub(crate) struct WinSwitch {
    saved: SwitchWin,
    no_display: bool,
}

impl WinSwitch {
    /// [`switch_win`] to `win` (and `tabpage`, when given), answering the
    /// guard that switches back and whether the switch worked (`win` was
    /// still valid). No autocommands run until the guard is dropped.
    pub(crate) fn enter(win: Win, tabpage: Option<TabPage>, no_display: bool) -> (WinSwitch, bool) {
        let mut saved = SwitchWin::default();
        let entered = switch_win(&mut saved, win, tabpage, no_display);
        (WinSwitch { saved, no_display }, entered.is_ok())
    }
}

impl Drop for WinSwitch {
    fn drop(&mut self) {
        restore_win(&mut self.saved, self.no_display);
    }
}
