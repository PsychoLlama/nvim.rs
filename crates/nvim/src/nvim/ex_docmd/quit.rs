//! Leaving: a window, a tab page, a buffer, or the editor.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ex_bunload(mut eap: *mut exarg_T) {
    (*eap).errmsg = do_bufdel(
        if (*eap).cmdidx as c_int == CMD_bdelete as c_int {
            DOBUF_DEL as c_int
        } else if (*eap).cmdidx as c_int == CMD_bwipeout as c_int {
            DOBUF_WIPE as c_int
        } else {
            DOBUF_UNLOAD as c_int
        },
        (*eap).arg,
        (*eap).addr_count,
        (*eap).line1 as c_int,
        (*eap).line2 as c_int,
        (*eap).forceit,
    );
}

pub unsafe extern "C" fn before_quit_autocmds(
    mut wp: *mut win_T,
    mut quit_all: bool,
    mut forceit: bool,
) -> bool {
    if *get_vim_var_str(VV_EXITREASON) as c_int == NUL {
        set_vim_var_string(
            VV_EXITREASON,
            b"quit\0".as_ptr() as *const c_char,
            ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as usize) as ptrdiff_t,
        );
    }
    apply_autocmds(
        EVENT_QUITPRE,
        ::core::ptr::null_mut::<c_char>(),
        ::core::ptr::null_mut::<c_char>(),
        false_0 != 0,
        (*wp).w_buffer,
    );
    if !win_valid(wp)
        || curbuf_locked() as c_int != 0
        || (*(*wp).w_buffer).b_nwindows == 1 as c_int && (*(*wp).w_buffer).b_locked > 0 as c_int
    {
        set_vim_var_string(
            VV_EXITREASON,
            ::core::ptr::null::<c_char>(),
            -1 as ptrdiff_t,
        );
        return true_0 != 0;
    }
    if quit_all as c_int != 0
        || check_more(false_0 != 0, forceit) == OK && only_one_window() as c_int != 0
    {
        apply_autocmds(
            EVENT_EXITPRE,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        if !win_valid(wp)
            || curbuf_locked() as c_int != 0
            || (*curbuf.get()).b_nwindows == 1 as c_int && (*curbuf.get()).b_locked > 0 as c_int
        {
            set_vim_var_string(
                VV_EXITREASON,
                ::core::ptr::null::<c_char>(),
                -1 as ptrdiff_t,
            );
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn ex_quit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count > 0 as c_int {
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        while !(*wp).w_next.is_null() {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            wp = (*wp).w_next;
        }
    } else {
        wp = curwin.get();
    }
    if curbuf_locked() {
        return;
    }
    if before_quit_autocmds(wp, false_0 != 0, (*eap).forceit != 0) {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK && only_one_window() as c_int != 0 {
        exiting.set(true_0 != 0);
    }
    if !buf_hide((*wp).w_buffer)
        && check_changed(
            (*wp).w_buffer,
            (if p_awa.get() != 0 {
                CCGD_AW as c_int
            } else {
                0 as c_int
            }) | (if (*eap).forceit != 0 {
                CCGD_FORCEIT as c_int
            } else {
                0 as c_int
            }) | CCGD_EXCMD as c_int,
        ) as c_int
            != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as c_int != 0
            && check_changed_any((*eap).forceit != 0, true_0 != 0) as c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() as c_int != 0
            && (firstwin.get() == lastwin.get() || (*eap).addr_count == 0 as c_int)
        {
            getout(0 as c_int);
        }
        not_exiting(save_exiting);
        win_close(
            wp,
            !buf_hide((*wp).w_buffer) || (*eap).forceit != 0,
            (*eap).forceit != 0,
        );
    };
}

/// `:cquit` never returns — `getout` exits the process. The signature
/// still says `()` because the command table holds one fn pointer type
/// and a `-> !` fn item does not coerce to it.
pub(crate) unsafe extern "C" fn ex_cquit(mut eap: *mut exarg_T) {
    let mut status: c_int = if (*eap).addr_count > 0 as c_int {
        (*eap).line2 as c_int
    } else {
        EXIT_FAILURE
    };
    ui_call_error_exit(status as Integer);
    getout(status);
}

pub unsafe extern "C" fn before_quit_all(mut eap: *mut exarg_T) -> c_int {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(if (*eap).forceit != 0 {
            -(253 as c_int + ((KE_XF1 as c_int) << 8 as c_int))
        } else {
            -(253 as c_int + ((KE_XF2 as c_int) << 8 as c_int))
        });
        return FAIL;
    }
    if text_locked() {
        text_locked_msg();
        return FAIL;
    }
    if before_quit_autocmds(curwin.get(), true_0 != 0, (*eap).forceit != 0) {
        return FAIL;
    }
    return OK;
}

pub(crate) unsafe extern "C" fn ex_quitall(mut eap: *mut exarg_T) {
    if before_quit_all(eap) == FAIL {
        return;
    }
    let mut save_exiting: bool = exiting.get();
    exiting.set(true_0 != 0);
    if (*eap).forceit != 0 || !check_changed_any(false_0 != 0, false_0 != 0) {
        getout(0 as c_int);
    }
    not_exiting(save_exiting);
}

pub(crate) unsafe extern "C" fn ex_close(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut winnr: c_int = 0 as c_int;
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
    } else if !text_locked() && !curbuf_locked() {
        if (*eap).addr_count == 0 as c_int {
            ex_win_close(
                (*eap).forceit,
                curwin.get(),
                ::core::ptr::null_mut::<tabpage_T>(),
            );
        } else {
            let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                firstwin.get()
            } else {
                (*curtab.get()).tp_firstwin
            };
            while !wp.is_null() {
                winnr += 1;
                if winnr as linenr_T == (*eap).line2 {
                    win = wp;
                    break;
                } else {
                    wp = (*wp).w_next;
                }
            }
            if win.is_null() {
                win = lastwin.get();
            }
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
        }
    }
}

pub(crate) unsafe extern "C" fn ex_pclose(mut eap: *mut exarg_T) {
    let mut win: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !win.is_null() {
        if (*win).w_onebuf_opt.wo_pvw != 0 {
            ex_win_close((*eap).forceit, win, ::core::ptr::null_mut::<tabpage_T>());
            break;
        } else {
            win = (*win).w_next;
        }
    }
}

pub unsafe extern "C" fn ex_win_close(
    mut forceit: c_int,
    mut win: *mut win_T,
    mut tp: *mut tabpage_T,
) {
    if is_aucmd_win(win) {
        emsg(gettext(&raw const e_autocmd_close as *const c_char));
        return;
    }
    if !(*win).w_floating && window_layout_locked(CMD_close) as c_int != 0 {
        return;
    }
    let mut buf: *mut buf_T = (*win).w_buffer;
    let mut need_hide: bool = bufIsChanged(buf) as c_int != 0 && (*buf).b_nwindows <= 1 as c_int;
    if need_hide as c_int != 0 && !buf_hide(buf) && forceit == 0 {
        if (p_confirm.get() != 0 || (*cmdmod.ptr()).cmod_flags & CMOD_CONFIRM as c_int != 0)
            && p_write.get() != 0
        {
            let mut bufref: bufref_T = bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            };
            set_bufref(&raw mut bufref, buf);
            dialog_changed(buf, false_0 != 0);
            if bufref_valid(&raw mut bufref) as c_int != 0 && bufIsChanged(buf) as c_int != 0 {
                return;
            }
            need_hide = false_0 != 0;
        } else {
            no_write_message();
            return;
        }
    }
    if tp.is_null() {
        win_close(win, !need_hide && !buf_hide(buf), forceit != 0);
    } else {
        win_close_othertab(
            win,
            (!need_hide && !buf_hide(buf)) as c_int,
            tp,
            forceit != 0,
        );
    };
}

pub(crate) unsafe extern "C" fn ex_tabclose(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        emsg(gettext(
            b"E784: Cannot close last tab page\0".as_ptr() as *const c_char
        ));
        return;
    }
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    let mut tp: *mut tabpage_T = find_tabpage(tab_number);
    if tp.is_null() {
        beep_flush();
        return;
    }
    if tp != curtab.get() {
        tabpage_close_other(tp, (*eap).forceit);
        return;
    } else if !text_locked() && !curbuf_locked() {
        tabpage_close((*eap).forceit);
    }
}

pub(crate) unsafe extern "C" fn ex_tabonly(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if (*first_tabpage.get()).tp_next.is_null() {
        msg(
            gettext(b"Already only one tab page\0".as_ptr() as *const c_char),
            0 as c_int,
        );
        return;
    }
    if window_layout_locked(CMD_tabonly) {
        return;
    }
    let mut tab_number: c_int = get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }
    goto_tabpage(tab_number);
    let mut done: c_int = 0 as c_int;
    while done < 1000 as c_int {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            if (*tp).tp_topframe != topframe.get() {
                tabpage_close_other(tp as *mut tabpage_T, (*eap).forceit);
                if valid_tabpage(tp as *mut tabpage_T) {
                    done = 1000 as c_int;
                }
                break;
            } else {
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        '_c2rust_label: {
            if !(*first_tabpage.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"first_tabpage\0".as_ptr() as *const c_char,
                    b"src/nvim/ex_docmd.rs\0".as_ptr() as *const c_char,
                    5361 as c_uint,
                    b"void ex_tabonly(exarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        if (*first_tabpage.get()).tp_next.is_null() {
            break;
        }
        done += 1;
    }
}

pub unsafe extern "C" fn tabpage_close(mut forceit: c_int) {
    if window_layout_locked(CMD_tabclose) {
        return;
    }
    trigger_tabclosedpre(curtab.get());
    (*curtab.get()).tp_did_tabclosedpre = true_0 != 0;
    let save_curtab: *mut tabpage_T = curtab.get();
    while (*curwin.get()).w_floating {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if !(firstwin.get() == lastwin.get()) {
        close_others(true_0, forceit);
    }
    if firstwin.get() == lastwin.get() {
        ex_win_close(forceit, curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
    }
    if curtab.get() == save_curtab {
        (*curtab.get()).tp_did_tabclosedpre = false_0 != 0;
    }
}

pub unsafe extern "C" fn tabpage_close_other(mut tp: *mut tabpage_T, mut forceit: c_int) {
    let mut done: c_int = 0 as c_int;
    let mut prev_idx: [c_char; 65] = [0; 65];
    if window_layout_locked(CMD_SIZE) {
        return;
    }
    trigger_tabclosedpre(tp);
    (*tp).tp_did_tabclosedpre = true_0 != 0;
    loop {
        done += 1;
        if done >= 1000 as c_int {
            break;
        }
        snprintf(
            &raw mut prev_idx as *mut c_char,
            ::core::mem::size_of::<[c_char; 65]>(),
            b"%i\0".as_ptr() as *const c_char,
            tabpage_index(tp),
        );
        let mut wp: *mut win_T = (*tp).tp_lastwin;
        ex_win_close(forceit, wp, tp);
        if !valid_tabpage(tp) {
            break;
        }
        if (*tp).tp_lastwin != wp {
            continue;
        }
        done = 1000 as c_int;
        break;
    }
    if done >= 1000 as c_int {
        (*tp).tp_did_tabclosedpre = false_0 != 0;
        return;
    }
}

pub(crate) unsafe extern "C" fn ex_only(mut eap: *mut exarg_T) {
    if window_layout_locked(CMD_only) {
        return;
    }
    if (*eap).addr_count > 0 as c_int {
        let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wnr: linenr_T = (*eap).line2;
        wp = firstwin.get();
        loop {
            wnr -= 1;
            if wnr <= 0 as linenr_T {
                break;
            }
            if (*wp).w_next.is_null() {
                break;
            }
            wp = (*wp).w_next;
        }
        if wp != curwin.get() {
            win_goto(wp);
        }
    }
    close_others(true_0, (*eap).forceit);
}

pub(crate) unsafe extern "C" fn ex_hide(mut eap: *mut exarg_T) {
    if (*eap).skip != 0 {
        return;
    }
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    if (*eap).addr_count == 0 as c_int {
        win = curwin.get();
    } else {
        let mut winnr: c_int = 0 as c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            winnr += 1;
            if winnr as linenr_T == (*eap).line2 {
                win = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if win.is_null() {
            win = lastwin.get();
        }
    }
    if !(*win).w_floating && window_layout_locked(CMD_hide) as c_int != 0 {
        return;
    }
    win_close(win, false_0 != 0, (*eap).forceit != 0);
}

pub(crate) unsafe extern "C" fn ex_stop(mut eap: *mut exarg_T) {
    if (*eap).forceit == 0 {
        autowrite_all();
    }
    may_trigger_vim_suspend_resume(true_0 != 0);
    ui_call_suspend();
    ui_flush();
}

pub(crate) unsafe extern "C" fn ex_exit(mut eap: *mut exarg_T) {
    if cmdwin_type.get() != 0 as c_int {
        cmdwin_result.set(Ctrl_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }
    let mut save_exiting: bool = exiting.get();
    if check_more(false_0 != 0, (*eap).forceit != 0) == OK && only_one_window() as c_int != 0 {
        exiting.set(true_0 != 0);
    }
    if ((*eap).cmdidx as c_int == CMD_wq as c_int || curbufIsChanged() as c_int != 0)
        && do_write(eap) == FAIL
        || before_quit_autocmds(curwin.get(), false_0 != 0, (*eap).forceit != 0) as c_int != 0
        || check_more(true_0 != 0, (*eap).forceit != 0) == FAIL
        || only_one_window() as c_int != 0
            && check_changed_any((*eap).forceit != 0, false_0 != 0) as c_int != 0
    {
        not_exiting(save_exiting);
    } else {
        if only_one_window() {
            getout(0 as c_int);
        }
        not_exiting(save_exiting);
        win_close(
            curwin.get(),
            !buf_hide((*curwin.get()).w_buffer),
            (*eap).forceit != 0,
        );
    };
}
