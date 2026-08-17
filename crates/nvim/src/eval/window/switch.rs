use super::*;
use crate::pos::equalpos;
use crate::types::VAR_STRING;

/// Switch to a window for executing user code.
/// Caller must call win_execute_after() later regardless of return value.
///
/// Returns whether switching the window succeeded.
pub unsafe extern "C" fn win_execute_before(
    args: *mut win_execute_T,
    wp: *mut win_T,
    tp: *mut tabpage_T,
) -> bool {
    (*args).wp = wp;
    (*args).curpos = (*wp).w_cursor;
    (*args).cwd_status = FAIL;
    (*args).apply_acd = false;
    (*args).save_sfname = ptr::null_mut();
    if curwin.get() != wp
        && (!(*curwin.get()).w_localdir.is_null()
            || !(*wp).w_localdir.is_null()
            || curtab.get() != tp
                && (!(*curtab.get()).tp_localdir.is_null() || !(*tp).tp_localdir.is_null())
            || p_acd.get() != 0)
    {
        (*args).cwd_status = os_dirname(&raw mut (*args).cwd as *mut c_char, MAXPATHL as size_t);
    }
    if (*args).cwd_status == OK && p_acd.get() != 0 {
        if !(*curbuf.get()).b_sfname.is_null()
            && (*curbuf.get()).b_fname == (*curbuf.get()).b_sfname
        {
            (*args).save_sfname = xstrdup((*curbuf.get()).b_sfname);
        }
        do_autochdir();
        let mut autocwd: [c_char; 4096] = [0; 4096];
        if os_dirname(&raw mut autocwd as *mut c_char, MAXPATHL as size_t) == OK {
            (*args).apply_acd = strcmp(
                &raw mut (*args).cwd as *mut c_char,
                &raw mut autocwd as *mut c_char,
            ) == 0;
        }
    }
    if switch_win_noblock(&raw mut (*args).switchwin, wp, tp, true) == OK {
        check_cursor(curwin.get());
        return true;
    }
    false
}
/// Restore the previous window after executing user code.
pub unsafe extern "C" fn win_execute_after(args: *mut win_execute_T) {
    restore_win_noblock(&raw mut (*args).switchwin, true);
    if (*args).apply_acd {
        xfree((*args).save_sfname as *mut c_void);
        do_autochdir();
    } else if (*args).cwd_status == OK {
        os_chdir(&raw mut (*args).cwd as *mut c_char);
        if !(*args).save_sfname.is_null() {
            xfree((*curbuf.get()).b_sfname as *mut c_void);
            (*curbuf.get()).b_sfname = (*args).save_sfname;
            (*curbuf.get()).b_fname = (*curbuf.get()).b_sfname;
        }
    }
    if win_valid((*args).wp) && !equalpos((*args).curpos, (*(*args).wp).w_cursor) {
        (*(*args).wp).w_redr_status = true;
    }
    check_cursor(curwin.get());
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
}
/// "win_execute(win_id, command)" function
pub unsafe extern "C" fn f_win_execute(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ptr::null_mut();
    let mut id: c_int = tv_get_number(argvars) as c_int;
    let mut tp: *mut tabpage_T = ptr::null_mut();
    let mut wp: *mut win_T = win_id2wp_tp(id, &raw mut tp);
    if wp.is_null() || tp.is_null() {
        return;
    }
    let mut win_execute_args: win_execute_T = mem::zeroed();
    if win_execute_before(&raw mut win_execute_args, wp, tp) {
        execute_common(argvars, rettv, 1);
    }
    win_execute_after(&raw mut win_execute_args);
}
/// Set "win" to be the curwin and "tp" to be the current tab page.
/// restore_win() MUST be called to undo, also when FAIL is returned.
/// No autocommands will be executed until restore_win() is called.
///
/// `no_display` — if true the display won't be affected, no redraw is
///                    triggered, another tabpage access is limited.
///
/// Returns FAIL if switching to "win" failed.
pub unsafe extern "C" fn switch_win(
    switchwin: *mut switchwin_T,
    win: *mut win_T,
    tp: *mut tabpage_T,
    no_display: bool,
) -> c_int {
    block_autocmds();
    switch_win_noblock(switchwin, win, tp, no_display)
}
/// As switch_win() but without blocking autocommands.
pub unsafe extern "C" fn switch_win_noblock(
    switchwin: *mut switchwin_T,
    win: *mut win_T,
    tp: *mut tabpage_T,
    no_display: bool,
) -> c_int {
    memset(switchwin as *mut c_void, 0, size_of::<switchwin_T>());
    (*switchwin).sw_curwin = curwin.get();
    if win == curwin.get() {
        (*switchwin).sw_same_win = true;
    } else {
        (*switchwin).sw_visual_active = VIsual_active.get();
        VIsual_active.set(false);
    }
    if !tp.is_null() {
        (*switchwin).sw_curtab = curtab.get();
        if no_display {
            unuse_tabpage(curtab.get());
            use_tabpage(tp);
        } else {
            goto_tabpage_tp(tp, false, false);
        }
    }
    if !win_valid(win) {
        return FAIL;
    }
    curwin.set(win);
    curbuf.set((*curwin.get()).w_buffer);
    OK
}
/// Restore current tabpage and window saved by switch_win(), if still valid.
/// When "no_display" is true the display won't be affected, no redraw is
/// triggered.
pub unsafe extern "C" fn restore_win(switchwin: *mut switchwin_T, mut no_display: bool) {
    restore_win_noblock(switchwin, no_display);
    unblock_autocmds();
}
/// As restore_win() but without unblocking autocommands.
pub unsafe extern "C" fn restore_win_noblock(switchwin: *mut switchwin_T, no_display: bool) {
    if !(*switchwin).sw_curtab.is_null() && valid_tabpage((*switchwin).sw_curtab) {
        if no_display {
            let old_tp_curwin: *mut win_T = (*curtab.get()).tp_curwin;
            unuse_tabpage(curtab.get());
            (*curtab.get()).tp_curwin = old_tp_curwin;
            use_tabpage((*switchwin).sw_curtab);
        } else {
            goto_tabpage_tp((*switchwin).sw_curtab, false, false);
        }
    }
    if !(*switchwin).sw_same_win {
        VIsual_active.set((*switchwin).sw_visual_active);
    }
    if win_valid((*switchwin).sw_curwin) {
        curwin.set((*switchwin).sw_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}
