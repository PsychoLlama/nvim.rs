//! Selecting an operator, a register or a recording, and replaying
//! them.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_regreplay(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    loop {
        let c2rust_fresh11 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh11 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg(reg_recorded.get(), false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}

pub(crate) unsafe extern "C" fn nv_undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_LOWER as c_int || VIsual_active.get() as c_int != 0 {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'u' as c_int;
        nv_operator(cap);
    } else {
        nv_kundo(cap);
    };
}

pub(crate) unsafe extern "C" fn nv_kundo(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}

pub(crate) unsafe extern "C" fn nv_regname(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as c_int {
        (*cap).nchar = get_expr_register();
    }
    if (*cap).nchar != NUL && valid_yank_reg((*cap).nchar, false_0 != 0) as c_int != 0 {
        (*(*cap).oap).regname = (*cap).nchar;
        (*cap).opcount = (*cap).count0;
        set_reg_var((*(*cap).oap).regname);
    } else {
        clearopbeep((*cap).oap);
    };
}

pub(crate) unsafe extern "C" fn nv_dot(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if start_redo(
        (*cap).count0,
        restart_edit.get() != 0 as c_int && !arrow_used.get(),
    ) == false_0
    {
        clearopbeep((*cap).oap);
    }
}

pub(crate) unsafe extern "C" fn nv_redo_or_register(mut cap: *mut cmdarg_T) {
    if VIsual_select.get() as c_int != 0 && VIsual_active.get() as c_int != 0 {
        (*no_mapping.ptr()) += 1;
        let mut reg: c_int = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && reg >= 0 as c_int
        {
            if reg < 256 as c_int {
                reg = (*langmap_mapchar.ptr())[reg as usize] as c_int;
            } else {
                reg = langmap_adjust_mb(reg);
            }
        }
        (*no_mapping.ptr()) -= 1;
        if reg == '"' as c_int {
            reg = 0 as c_int;
        }
        VIsual_select_reg.set(if valid_yank_reg(reg, true_0 != 0) as c_int != 0 {
            reg
        } else {
            0 as c_int
        });
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_redo((*cap).count1);
    (*curwin.get()).w_set_curswant = true_0;
}

pub(crate) unsafe extern "C" fn nv_Undo(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_UPPER as c_int || VIsual_active.get() as c_int != 0 {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'U' as c_int;
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    u_undoline();
    (*curwin.get()).w_set_curswant = true_0;
}

pub(crate) unsafe extern "C" fn nv_operator(mut cap: *mut cmdarg_T) {
    let mut op_type: c_int = get_op_type((*cap).cmdchar, (*cap).nchar);
    if bt_prompt(curbuf.get()) as c_int != 0
        && op_is_change(op_type) != 0
        && !prompt_curpos_editable()
    {
        clearopbeep((*cap).oap);
        return;
    }
    if op_type == (*(*cap).oap).op_type {
        nv_lineop(cap);
    } else if !checkclearop((*cap).oap) {
        (*(*cap).oap).start = (*curwin.get()).w_cursor;
        (*(*cap).oap).op_type = op_type;
        set_op_var(op_type);
    }
}

pub(crate) unsafe extern "C" fn set_op_var(mut optype: c_int) {
    if optype == OP_NOP as c_int {
        set_vim_var_string(VV_OP, ::core::ptr::null::<c_char>(), 0 as ptrdiff_t);
    } else {
        let mut opchars: [c_char; 3] = [0; 3];
        let mut opchar0: c_int = get_op_char(optype);
        '_c2rust_label: {
            if opchar0 >= 0 as c_int && opchar0 <= 127 as c_int * 2 as c_int + 1 as c_int {
            } else {
                __assert_fail(
                    b"opchar0 >= 0 && opchar0 <= UCHAR_MAX\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    5876 as c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const c_char,
                );
            }
        };
        opchars[0 as c_int as usize] = opchar0 as c_char;
        let mut opchar1: c_int = get_extra_op_char(optype);
        '_c2rust_label_0: {
            if opchar1 >= 0 as c_int && opchar1 <= 127 as c_int * 2 as c_int + 1 as c_int {
            } else {
                __assert_fail(
                    b"opchar1 >= 0 && opchar1 <= UCHAR_MAX\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    5880 as c_uint,
                    b"void set_op_var(int)\0".as_ptr() as *const c_char,
                );
            }
        };
        opchars[1 as c_int as usize] = opchar1 as c_char;
        opchars[2 as c_int as usize] = NUL as c_char;
        set_vim_var_string(VV_OP, &raw mut opchars as *mut c_char, 2 as ptrdiff_t);
    };
}

pub(crate) unsafe extern "C" fn nv_lineop(mut cap: *mut cmdarg_T) {
    (*(*cap).oap).motion_type = kMTLineWise;
    if cursor_down(
        (*cap).count1 - 1 as c_int,
        (*(*cap).oap).op_type == OP_NOP as c_int,
    ) == false_0
    {
        clearopbeep((*cap).oap);
    } else if (*(*cap).oap).op_type == OP_DELETE as c_int
        && (*(*cap).oap).motion_force != 'v' as c_int
        && (*(*cap).oap).motion_force != Ctrl_V
        || (*(*cap).oap).op_type == OP_LSHIFT as c_int
        || (*(*cap).oap).op_type == OP_RSHIFT as c_int
    {
        beginline(BL_SOL as c_int | BL_FIX as c_int);
    } else if (*(*cap).oap).op_type != OP_YANK as c_int {
        beginline(BL_WHITE as c_int | BL_FIX as c_int);
    }
}

pub(crate) unsafe extern "C" fn nv_record(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_FORMAT as c_int {
        (*cap).cmdchar = 'g' as c_int;
        (*cap).nchar = 'q' as c_int;
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == ':' as c_int || (*cap).nchar == '/' as c_int || (*cap).nchar == '?' as c_int
    {
        if cmdwin_type.get() != 0 as c_int {
            emsg(gettext(e_cmdline_window_already_open.as_ptr()));
            return;
        }
        stuffcharReadbuff((*cap).nchar);
        stuffcharReadbuff(-(253 as c_int + ((KE_CMDWIN as c_int) << 8 as c_int)));
    } else if reg_executing.get() == 0 as c_int && do_record((*cap).nchar) == FAIL {
        clearopbeep((*cap).oap);
    }
}

pub(crate) unsafe extern "C" fn nv_at(mut cap: *mut cmdarg_T) {
    if checkclearop((*cap).oap) {
        return;
    }
    if (*cap).nchar == '=' as c_int {
        if get_expr_register() == NUL {
            return;
        }
    }
    loop {
        let c2rust_fresh13 = (*cap).count1;
        (*cap).count1 = (*cap).count1 - 1;
        if !(c2rust_fresh13 != 0 && !got_int.get()) {
            break;
        }
        if do_execreg((*cap).nchar, false_0, false_0, false_0) == false_0 {
            clearopbeep((*cap).oap);
            break;
        } else {
            line_breakcheck();
        }
    }
}
