//! Commands that do not belong to a family: the no-ops, the error
//! handler, `:`, the CTRL-key odds and ends, and leaving a mode.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_ignore(mut cap: *mut cmdarg_T) {
    (*cap).retval |= CA_COMMAND_BUSY as c_int;
}

pub(crate) unsafe extern "C" fn nv_nop(mut _cap: *mut cmdarg_T) {}

pub(crate) unsafe extern "C" fn nv_error(mut cap: *mut cmdarg_T) {
    clearopbeep((*cap).oap);
}

pub(crate) unsafe extern "C" fn nv_help(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        ex_help(::core::ptr::null_mut::<exarg_T>());
    }
}

pub(crate) unsafe extern "C" fn nv_colon(mut cap: *mut cmdarg_T) {
    let mut cmd_result: bool = false;
    let mut is_cmdkey: bool =
        (*cap).cmdchar == -(253 as c_int + ((KE_COMMAND as c_int) << 8 as c_int));
    let mut is_lua: bool = (*cap).cmdchar == -(253 as c_int + ((KE_LUA as c_int) << 8 as c_int));
    if VIsual_active.get() as c_int != 0 && !is_cmdkey && !is_lua {
        nv_operator(cap);
        return;
    }
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        (*(*cap).oap).motion_type = kMTCharWise;
        (*(*cap).oap).inclusive = false_0 != 0;
    } else if (*cap).count0 != 0 && !is_cmdkey && !is_lua {
        stuffcharReadbuff('.' as c_int);
        if (*cap).count0 > 1 as c_int {
            stuffReadbuff(b",.+\0".as_ptr() as *const c_char);
            stuffnumReadbuff((*cap).count0 - 1 as c_int);
        }
    }
    if KeyTyped.get() {
        msg_ext_set_trigger(b"typed_cmd\0".as_ptr() as *const c_char);
        compute_cmdrow();
    }
    if is_lua {
        cmd_result = map_execute_lua(true_0 != 0, false_0 != 0);
    } else {
        cmd_result = do_cmdline(
            ::core::ptr::null_mut::<c_char>(),
            if is_cmdkey as c_int != 0 {
                Some(
                    getcmdkeycmd
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                )
            } else {
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                )
            },
            NULL,
            if (*(*cap).oap).op_type != OP_NOP as c_int {
                DOCMD_KEEPLINE as c_int
            } else {
                0 as c_int
            },
        ) != 0;
    }
    msg_ext_set_trigger(b"\0".as_ptr() as *const c_char);
    if cmd_result as c_int == false_0 {
        clearop((*cap).oap);
    } else if (*(*cap).oap).op_type != OP_NOP as c_int
        && ((*(*cap).oap).start.lnum > (*curbuf.get()).b_ml.ml_line_count
            || (*(*cap).oap).start.col > ml_get_len((*(*cap).oap).start.lnum)
            || did_emsg.get() != 0)
    {
        clearopbeep((*cap).oap);
    }
}

pub(crate) unsafe extern "C" fn nv_ctrlg(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        VIsual_select.set(!VIsual_select.get());
        may_trigger_modechanged();
        showmode();
    } else if !checkclearop((*cap).oap) {
        fileinfo((*cap).count0, false_0, true_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn nv_ctrlh(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        (*cap).cmdchar = 'x' as c_int;
        v_visop(cap);
    } else {
        nv_left(cap);
    };
}

pub(crate) unsafe extern "C" fn nv_clear(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    syn_stack_free_all((*curwin.get()).w_s);
    let mut wp: *mut win_T = if curtab.get() == curtab.get() {
        firstwin.get()
    } else {
        (*curtab.get()).tp_firstwin
    };
    while !wp.is_null() {
        (*(*wp).w_s).b_syn_slow = false_0 != 0;
        wp = (*wp).w_next;
    }
    redraw_later(curwin.get(), UPD_CLEAR as c_int);
}

pub(crate) unsafe extern "C" fn nv_ctrlo(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        VIsual_select.set(false_0 != 0);
        may_trigger_modechanged();
        showmode();
        restart_VIsual_select.set(2 as c_int);
    } else {
        (*cap).count1 = -(*cap).count1;
        nv_pcmark(cap);
    };
}

pub(crate) unsafe extern "C" fn nv_hat(mut cap: *mut cmdarg_T) {
    if !checkclearopq((*cap).oap) {
        buflist_getfile(
            (*cap).count0,
            0 as linenr_T,
            GETF_SETMARK as c_int | GETF_ALT as c_int,
            false_0,
        );
    }
}

pub(crate) unsafe extern "C" fn nv_window(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == ':' as c_int {
        (*cap).cmdchar = ':' as c_int;
        (*cap).nchar = NUL;
        nv_colon(cap);
    } else if !checkclearop((*cap).oap) {
        do_window((*cap).nchar, (*cap).count0, NUL);
    }
}

pub(crate) unsafe extern "C" fn nv_suspend(mut cap: *mut cmdarg_T) {
    clearop((*cap).oap);
    if VIsual_active.get() {
        end_visual_mode();
    }
    do_cmdline_cmd(b"st\0".as_ptr() as *const c_char);
}

pub(crate) unsafe extern "C" fn nv_normal(mut cap: *mut cmdarg_T) {
    if (*cap).nchar == Ctrl_N || (*cap).nchar == Ctrl_G {
        clearop((*cap).oap);
        if restart_edit.get() != 0 as c_int && mode_displayed.get() as c_int != 0 {
            clear_cmdline.set(true_0 != 0);
        }
        restart_edit.set(0 as c_int);
        if cmdwin_type.get() != 0 as c_int {
            cmdwin_result.set(Ctrl_C);
        }
        if VIsual_active.get() {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED as c_int);
        }
    } else {
        clearopbeep((*cap).oap);
    };
}

pub(crate) unsafe extern "C" fn nv_esc(mut cap: *mut cmdarg_T) {
    let mut no_reason: bool = (*(*cap).oap).op_type == OP_NOP as c_int
        && (*cap).opcount == 0 as c_int
        && (*cap).count0 == 0 as c_int
        && (*(*cap).oap).regname == 0 as c_int;
    if (*cap).arg != 0 {
        if restart_edit.get() == 0 as c_int
            && cmdwin_type.get() == 0 as c_int
            && !VIsual_active.get()
            && no_reason as c_int != 0
        {
            if anyBufIsChanged() {
                msg(
                    gettext(
                        b"Type  :qa!  and press <Enter> to abandon all changes and exit Nvim\0"
                            .as_ptr() as *const c_char,
                    ),
                    0 as c_int,
                );
            } else {
                msg(
                    gettext(
                        b"Type  :qa  and press <Enter> to exit Nvim\0".as_ptr() as *const c_char
                    ),
                    0 as c_int,
                );
            }
        }
        if restart_edit.get() != 0 as c_int {
            redraw_mode.set(true_0 != 0);
        }
        restart_edit.set(0 as c_int);
        if cmdwin_type.get() != 0 as c_int {
            cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
            got_int.set(false_0 != 0);
            return;
        }
    } else if cmdwin_type.get() != 0 as c_int
        && ex_normal_busy.get() != 0
        && typebuf_was_empty.get() as c_int != 0
    {
        cmdwin_result.set(-(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)));
        return;
    }
    if VIsual_active.get() {
        end_visual_mode();
        check_cursor_col(curwin.get());
        (*curwin.get()).w_set_curswant = true_0;
        redraw_curbuf_later(UPD_INVERTED as c_int);
    } else if no_reason {
        vim_beep(kOptBoFlagEsc as c_int as c_uint);
    }
    clearop((*cap).oap);
}

pub(crate) unsafe extern "C" fn nv_paste(mut cap: *mut cmdarg_T) {
    paste_repeat((*cap).count1);
}

pub(crate) unsafe extern "C" fn nv_event(mut cap: *mut cmdarg_T) {
    may_garbage_collect.set(false_0 != 0);
    let mut may_restart: bool =
        restart_edit.get() != 0 as c_int || restart_VIsual_select.get() != 0 as c_int;
    state_handle_k_event();
    finish_op.set(false_0 != 0);
    if may_restart {
        (*cap).retval |= CA_COMMAND_BUSY as c_int;
    }
}
