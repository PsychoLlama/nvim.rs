//! Commands that change the text without entering insert mode, and the
//! ones whose whole job is to enter it.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_addsub(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
    } else if !VIsual_active.get() && (*(*cap).oap).op_type == OP_NOP as c_int {
        prep_redo_cmd(cap);
        (*(*cap).oap).op_type = if (*cap).cmdchar == Ctrl_A {
            OP_NR_ADD as c_int
        } else {
            OP_NR_SUB as c_int
        };
        op_addsub((*cap).oap, (*cap).count1 as linenr_T, (*cap).arg != 0);
        (*(*cap).oap).op_type = OP_NOP as c_int;
    } else if VIsual_active.get() {
        nv_operator(cap);
    } else {
        clearop((*cap).oap);
    };
}

pub(crate) unsafe extern "C" fn nv_replace(mut cap: *mut cmdarg_T) {
    let mut had_ctrl_v: c_int = 0;
    if checkclearop((*cap).oap) {
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if (*cap).nchar == Ctrl_V || (*cap).nchar == Ctrl_Q {
        had_ctrl_v = Ctrl_V;
        (*cap).nchar = get_literal(false_0 != 0);
        if (*cap).nchar > DEL {
            had_ctrl_v = NUL;
        }
    } else {
        had_ctrl_v = NUL;
    }
    if (*cap).nchar < 0 as c_int {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if got_int.get() {
            got_int.set(false_0 != 0);
        }
        if had_ctrl_v != 0 {
            if (*cap).nchar == CAR {
                (*cap).nchar = REPLACE_CR_NCHAR as c_int;
            } else if (*cap).nchar == NL {
                (*cap).nchar = REPLACE_NL_NCHAR as c_int;
            }
        }
        nv_operator(cap);
        return;
    }
    if virtual_active(curwin.get()) {
        if u_save_cursor() == false_0 {
            return;
        }
        if gchar_cursor() == NUL {
            coladvance_force(getviscol() + (*cap).count1);
            '_c2rust_label: {
                if (*cap).count1 <= 2147483647 as c_int {
                } else {
                    __assert_fail(
                        b"cap->count1 <= INT_MAX\0".as_ptr() as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        4553 as c_uint,
                        b"void nv_replace(cmdarg_T *)\0".as_ptr() as *const c_char,
                    );
                }
            };
            (*curwin.get()).w_cursor.col -= (*cap).count1;
        } else if gchar_cursor() == TAB {
            coladvance_force(getviscol());
        }
    }
    if (get_cursor_pos_len() as size_t) < (*cap).count1 as c_uint as size_t
        || mb_charlen(get_cursor_pos_ptr()) < (*cap).count1
    {
        clearopbeep((*cap).oap);
        return;
    }
    if had_ctrl_v != Ctrl_V
        && (*cap).nchar == '\t' as c_int
        && ((*curbuf.get()).b_p_et != 0 || p_sta.get() != 0)
    {
        stuffnumReadbuff((*cap).count1);
        stuffcharReadbuff('R' as c_int);
        stuffcharReadbuff('\t' as c_int);
        stuffcharReadbuff(ESC);
        return;
    }
    if u_save_cursor() == false_0 {
        return;
    }
    if had_ctrl_v != Ctrl_V && ((*cap).nchar == '\r' as c_int || (*cap).nchar == '\n' as c_int) {
        del_chars((*cap).count1, false_0);
        stuffcharReadbuff('\r' as c_int);
        stuffcharReadbuff(ESC);
        invoke_edit(cap, true_0, 'r' as c_int, false_0);
    } else {
        prep_redo(
            (*(*cap).oap).regname,
            (*cap).count1,
            NUL,
            'r' as c_int,
            NUL,
            had_ctrl_v,
            0 as c_int,
        );
        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
        let old_State: c_int = State.get();
        if (*cap).nchar_len > 0 as c_int {
            AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut c_char);
        } else {
            AppendCharToRedobuff((*cap).nchar);
        }
        let mut n: c_int = (*cap).count1;
        while n > 0 as c_int {
            State.set(MODE_REPLACE as c_int);
            if (*cap).nchar == Ctrl_E || (*cap).nchar == Ctrl_Y {
                let mut c: c_int = ins_copychar(
                    (*curwin.get()).w_cursor.lnum
                        + (if (*cap).nchar == Ctrl_Y {
                            -1 as linenr_T
                        } else {
                            1 as linenr_T
                        }),
                );
                if c != NUL {
                    ins_char(c);
                } else {
                    (*curwin.get()).w_cursor.col += 1;
                }
            } else if (*cap).nchar_len != 0 {
                ins_char_bytes(
                    &raw mut (*cap).nchar_composing as *mut c_char,
                    (*cap).nchar_len as size_t,
                );
            } else {
                ins_char((*cap).nchar);
            }
            State.set(old_State);
            n -= 1;
        }
        (*curwin.get()).w_cursor.col -= 1;
        mb_adjust_cursor();
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        (*curwin.get()).w_set_curswant = true_0;
        set_last_insert((*cap).nchar);
    }
    foldUpdateAfterInsert();
}

pub(crate) unsafe extern "C" fn nv_Replace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'c' as c_int;
        (*cap).nchar = NUL;
        VIsual_mode_orig.set(VIsual_mode.get());
        VIsual_mode.set('V' as c_int);
        nv_operator(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
    } else {
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(
            cap,
            false_0,
            if (*cap).arg != 0 {
                'V' as c_int
            } else {
                'R' as c_int
            },
            false_0,
        );
    };
}

pub(crate) unsafe extern "C" fn nv_vreplace(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        (*cap).cmdchar = 'r' as c_int;
        (*cap).nchar = (*cap).extra_char;
        nv_replace(cap);
        return;
    }
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*curbuf.get()).b_p_ma == 0 {
        emsg(gettext(&raw const e_modifiable as *const c_char));
    } else {
        if (*cap).extra_char == Ctrl_V || (*cap).extra_char == Ctrl_Q {
            (*cap).extra_char = get_literal(false_0 != 0);
        }
        if (*cap).extra_char < ' ' as c_int {
            stuffcharReadbuff(Ctrl_V);
        }
        stuffcharReadbuff((*cap).extra_char);
        stuffcharReadbuff(ESC);
        if virtual_active(curwin.get()) {
            coladvance(curwin.get(), getviscol());
        }
        invoke_edit(cap, true_0, 'v' as c_int, false_0);
    };
}

pub(crate) unsafe extern "C" fn n_swapchar(mut cap: *mut cmdarg_T) {
    let mut did_change: bool = false_0 != 0;
    if checkclearopq((*cap).oap) {
        return;
    }
    if *ml_get((*curwin.get()).w_cursor.lnum) as c_int == NUL
        && vim_strchr(p_ww.get(), '~' as c_int).is_null()
    {
        clearopbeep((*cap).oap);
        return;
    }
    prep_redo_cmd(cap);
    if u_save_cursor() == false_0 {
        return;
    }
    let mut startpos: pos_T = (*curwin.get()).w_cursor;
    let mut n: c_int = (*cap).count1;
    while n > 0 as c_int {
        did_change = did_change as c_int
            | swapchar((*(*cap).oap).op_type, &raw mut (*curwin.get()).w_cursor) as c_int
            != 0;
        inc_cursor();
        if gchar_cursor() == NUL {
            if !(!vim_strchr(p_ww.get(), '~' as c_int).is_null()
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count)
            {
                break;
            }
            (*curwin.get()).w_cursor.lnum += 1;
            (*curwin.get()).w_cursor.col = 0 as c_int as colnr_T;
            if n > 1 as c_int {
                if u_savesub((*curwin.get()).w_cursor.lnum) == false_0 {
                    break;
                }
                u_clearline(curbuf.get());
            }
        }
        n -= 1;
    }
    check_cursor(curwin.get());
    (*curwin.get()).w_set_curswant = true_0;
    if did_change {
        changed_lines(
            curbuf.get(),
            startpos.lnum,
            startpos.col,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            0 as linenr_T,
            true_0 != 0,
        );
        (*curbuf.get()).b_op_start = startpos;
        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
        if (*curbuf.get()).b_op_end.col > 0 as c_int {
            (*curbuf.get()).b_op_end.col -= 1;
        }
    }
}

pub(crate) unsafe extern "C" fn nv_subst(mut cap: *mut cmdarg_T) {
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        clearopbeep((*cap).oap);
        return;
    }
    if VIsual_active.get() {
        if (*cap).cmdchar == 'S' as c_int {
            VIsual_mode_orig.set(VIsual_mode.get());
            VIsual_mode.set('V' as c_int);
        }
        (*cap).cmdchar = 'c' as c_int;
        nv_operator(cap);
    } else {
        nv_optrans(cap);
    };
}

pub(crate) unsafe extern "C" fn nv_abbrev(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_DEL
        || (*cap).cmdchar == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int))
    {
        (*cap).cmdchar = 'x' as c_int;
    }
    if VIsual_active.get() {
        v_visop(cap);
    } else {
        nv_optrans(cap);
    };
}

pub(crate) unsafe extern "C" fn nv_optrans(mut cap: *mut cmdarg_T) {
    static ar: GlobalCell<[*const c_char; 8]> = GlobalCell::new([
        b"dl\0".as_ptr() as *const c_char,
        b"dh\0".as_ptr() as *const c_char,
        b"d$\0".as_ptr() as *const c_char,
        b"c$\0".as_ptr() as *const c_char,
        b"cl\0".as_ptr() as *const c_char,
        b"cc\0".as_ptr() as *const c_char,
        b"yy\0".as_ptr() as *const c_char,
        b":s\r\0".as_ptr() as *const c_char,
    ]);
    static str: GlobalCell<*const c_char> =
        GlobalCell::new(b"xXDCsSY&\0".as_ptr() as *const c_char);
    if !checkclearopq((*cap).oap) {
        if (*cap).count0 != 0 {
            stuffnumReadbuff((*cap).count0);
        }
        stuffReadbuff(
            (*ar.ptr())[strchr(str.get(), (*cap).cmdchar as c_char as c_int).offset_from(str.get())
                as usize] as *const c_char,
        );
    }
    (*cap).opcount = 0 as c_int;
}

pub(crate) unsafe extern "C" fn n_opencmd(mut cap: *mut cmdarg_T) {
    if checkclearopq((*cap).oap) {
        return;
    }
    if (*cap).cmdchar == 'O' as c_int {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            &raw mut (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        );
    } else {
        hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        );
    }
    (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    if u_save(
        (*curwin.get()).w_cursor.lnum
            - (if (*cap).cmdchar == 'O' as c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
        (*curwin.get()).w_cursor.lnum
            + (if (*cap).cmdchar == 'o' as c_int {
                1 as linenr_T
            } else {
                0 as linenr_T
            }),
    ) != 0
        && open_line(
            if (*cap).cmdchar == 'O' as c_int {
                BACKWARD as c_int
            } else {
                FORWARD as c_int
            },
            if has_format_option(FO_OPEN_COMS) as c_int != 0 {
                OPENLINE_DO_COM as c_int
            } else {
                0 as c_int
            },
            0 as c_int,
            ::core::ptr::null_mut::<bool>(),
        ) as c_int
            != 0
    {
        if win_cursorline_standout(curwin.get()) {
            (*curwin.get()).w_valid &= !VALID_CROW;
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, true_0);
    }
}

pub(crate) unsafe extern "C" fn nv_tilde(mut cap: *mut cmdarg_T) {
    if p_to.get() == 0 && !VIsual_active.get() && (*(*cap).oap).op_type != OP_TILDE as c_int {
        if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
            clearopbeep((*cap).oap);
            return;
        }
        n_swapchar(cap);
    } else {
        nv_operator(cap);
    };
}

pub unsafe extern "C" fn set_cursor_for_append_to_line() {
    (*curwin.get()).w_set_curswant = true_0;
    if get_ve_flags(curwin.get()) == kOptVeFlagAll as c_int as c_uint {
        let save_State: c_int = State.get();
        State.set(MODE_INSERT as c_int);
        coladvance(curwin.get(), MAXCOL as c_int);
        State.set(save_State);
    } else {
        (*curwin.get()).w_cursor.col += strlen(get_cursor_pos_ptr()) as colnr_T;
    };
}

pub(crate) unsafe extern "C" fn nv_edit(mut cap: *mut cmdarg_T) {
    if (*cap).cmdchar == K_INS
        || (*cap).cmdchar == -(253 as c_int + ((KE_KINS as c_int) << 8 as c_int))
    {
        (*cap).cmdchar = 'i' as c_int;
    }
    if VIsual_active.get() as c_int != 0
        && ((*cap).cmdchar == 'A' as c_int || (*cap).cmdchar == 'I' as c_int)
    {
        v_visop(cap);
    } else if ((*cap).cmdchar == 'a' as c_int || (*cap).cmdchar == 'i' as c_int)
        && ((*(*cap).oap).op_type != OP_NOP as c_int || VIsual_active.get() as c_int != 0)
    {
        nv_object(cap);
    } else if (*curbuf.get()).b_p_ma == 0 && (*curbuf.get()).terminal.is_null() {
        emsg(gettext(&raw const e_modifiable as *const c_char));
        clearop((*cap).oap);
    } else if !checkclearopq((*cap).oap) {
        match (*cap).cmdchar {
            65 => {
                set_cursor_for_append_to_line();
            }
            73 => {
                beginline(BL_WHITE as c_int);
            }
            97 => {
                if virtual_active(curwin.get()) as c_int != 0
                    && ((*curwin.get()).w_cursor.coladd > 0 as c_int
                        || *get_cursor_pos_ptr() as c_int == NUL
                        || *get_cursor_pos_ptr() as c_int == TAB)
                {
                    (*curwin.get()).w_cursor.coladd += 1;
                } else if *get_cursor_pos_ptr() as c_int != NUL {
                    inc_cursor();
                }
            }
            _ => {}
        }
        if (*curwin.get()).w_cursor.coladd != 0 && (*cap).cmdchar != 'A' as c_int {
            let mut save_State: c_int = State.get();
            State.set(MODE_INSERT as c_int);
            coladvance(curwin.get(), getviscol());
            State.set(save_State);
        }
        invoke_edit(cap, false_0, (*cap).cmdchar, false_0);
    }
}

pub(crate) unsafe extern "C" fn invoke_edit(
    mut cap: *mut cmdarg_T,
    mut repl: c_int,
    mut cmd: c_int,
    mut startln: c_int,
) {
    let mut restart_edit_save: c_int = 0 as c_int;
    if repl != 0 || !stuff_empty() {
        restart_edit_save = restart_edit.get();
    } else {
        restart_edit_save = 0 as c_int;
    }
    restart_edit.set(0 as c_int);
    if (*cap).cmdchar != 'O' as c_int && (*cap).cmdchar != 'o' as c_int {
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
    }
    if edit(cmd, startln != 0, (*cap).count1) {
        (*cap).retval |= CA_COMMAND_BUSY as c_int;
    }
    if restart_edit.get() == 0 as c_int {
        restart_edit.set(restart_edit_save);
    }
}

pub(crate) unsafe extern "C" fn nv_join(mut cap: *mut cmdarg_T) {
    if VIsual_active.get() {
        nv_operator(cap);
        return;
    }
    if checkclearop((*cap).oap) {
        return;
    }
    (*cap).count0 = if (*cap).count0 > 2 as c_int {
        (*cap).count0
    } else {
        2 as c_int
    };
    if (*curwin.get()).w_cursor.lnum + (*cap).count0 as linenr_T - 1 as linenr_T
        > (*curbuf.get()).b_ml.ml_line_count
    {
        if (*cap).count0 <= 2 as c_int {
            clearopbeep((*cap).oap);
            return;
        }
        (*cap).count0 = ((*curbuf.get()).b_ml.ml_line_count - (*curwin.get()).w_cursor.lnum
            + 1 as linenr_T) as c_int;
    }
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        (*cap).nchar,
    );
    do_join(
        (*cap).count0 as size_t,
        (*cap).nchar == NUL,
        true_0 != 0,
        true_0 != 0,
        true_0 != 0,
    );
}

pub(crate) unsafe extern "C" fn nv_put(mut cap: *mut cmdarg_T) {
    nv_put_opt(cap, false_0 != 0);
}

pub(crate) unsafe extern "C" fn nv_put_opt(mut cap: *mut cmdarg_T, mut fix_indent: bool) {
    let mut savereg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut empty: bool = false_0 != 0;
    let mut was_visual: bool = false_0 != 0;
    let mut dir: c_int = 0;
    let mut flags: c_int = 0 as c_int;
    let save_fen: c_int = (*curwin.get()).w_onebuf_opt.wo_fen;
    if (*(*cap).oap).op_type != OP_NOP as c_int {
        if (*(*cap).oap).op_type == OP_DELETE as c_int && (*cap).cmdchar == 'p' as c_int {
            clearop((*cap).oap);
            '_c2rust_label: {
                if (*cap).opcount >= 0 as c_int {
                } else {
                    __assert_fail(
                        b"cap->opcount >= 0\0".as_ptr() as *const c_char,
                        b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                        6502 as c_uint,
                        b"void nv_put_opt(cmdarg_T *, _Bool)\0".as_ptr() as *const c_char,
                    );
                }
            };
            nv_diffgetput(true_0 != 0, (*cap).opcount as size_t);
        } else {
            clearopbeep((*cap).oap);
        }
        return;
    }
    if bt_prompt(curbuf.get()) as c_int != 0 && !prompt_curpos_editable() {
        if (*curwin.get()).w_cursor.lnum == (*curbuf.get()).b_prompt_start.mark.lnum {
            (*curwin.get()).w_cursor.col = (*curbuf.get()).b_prompt_start.mark.col;
            (*cap).cmdchar = 'P' as c_int;
        } else {
            clearopbeep((*cap).oap);
            return;
        }
    }
    if fix_indent {
        dir = if (*cap).cmdchar == ']' as c_int && (*cap).nchar == 'p' as c_int {
            FORWARD as c_int
        } else {
            BACKWARD as c_int
        };
        flags |= PUT_FIXINDENT as c_int;
    } else {
        dir = if (*cap).cmdchar == 'P' as c_int
            || ((*cap).cmdchar == 'g' as c_int || (*cap).cmdchar == 'z' as c_int)
                && (*cap).nchar == 'P' as c_int
        {
            BACKWARD as c_int
        } else {
            FORWARD as c_int
        };
    }
    prep_redo_cmd(cap);
    if (*cap).cmdchar == 'g' as c_int {
        flags |= PUT_CURSEND as c_int;
    } else if (*cap).cmdchar == 'z' as c_int {
        flags |= PUT_BLOCK_INNER as c_int;
    }
    if VIsual_active.get() {
        was_visual = true_0 != 0;
        let mut regname: c_int = (*(*cap).oap).regname;
        let mut keep_registers: bool = (*cap).cmdchar == 'P' as c_int;
        let mut clipoverwrite: bool = (regname == '+' as c_int || regname == '*' as c_int)
            && cb_flags.get()
                & (kOptCbFlagUnnamed as c_int | kOptCbFlagUnnamedplus as c_int) as c_uint
                != 0;
        if regname == 0 as c_int
            || regname == '"' as c_int
            || clipoverwrite as c_int != 0
            || ascii_isdigit(regname) as c_int != 0
            || regname == '-' as c_int
        {
            savereg = copy_register(regname);
        }
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
        if !VIsual_active.get() || VIsual_mode.get() == 'V' as c_int || regname != '.' as c_int {
            (*cap).cmdchar = 'd' as c_int;
            (*cap).nchar = NUL;
            (*(*cap).oap).regname = if keep_registers as c_int != 0 {
                '_' as c_int
            } else {
                NUL
            };
            (*msg_silent.ptr()) += 1;
            nv_operator(cap);
            do_pending_operator(cap, 0 as c_int, false_0 != 0);
            empty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0;
            (*msg_silent.ptr()) -= 1;
            (*(*cap).oap).regname = regname;
        }
        if VIsual_mode.get() == 'V' as c_int {
            flags |= PUT_LINE as c_int;
        } else if VIsual_mode.get() == 'v' as c_int {
            flags |= PUT_LINE_SPLIT as c_int;
        }
        if VIsual_mode.get() == Ctrl_V && dir == FORWARD as c_int {
            flags |= PUT_LINE_FORWARD as c_int;
        }
        dir = BACKWARD as c_int;
        if VIsual_mode.get() != 'V' as c_int
            && (*curwin.get()).w_cursor.col < (*curbuf.get()).b_op_start.col
            || VIsual_mode.get() == 'V' as c_int
                && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_op_start.lnum
        {
            dir = FORWARD as c_int;
        }
        VIsual_active.set(true_0 != 0);
    }
    do_put((*(*cap).oap).regname, savereg, dir, (*cap).count1, flags);
    if !savereg.is_null() {
        free_register(savereg);
        xfree(savereg as *mut c_void);
    }
    if was_visual {
        if save_fen != 0 {
            (*curwin.get()).w_onebuf_opt.wo_fen = true_0;
        }
        (*curbuf.get()).b_visual.vi_start = (*curbuf.get()).b_op_start;
        (*curbuf.get()).b_visual.vi_end = (*curbuf.get()).b_op_end;
        if *p_sel.get() as c_int == 'e' as c_int {
            inc(&raw mut (*curbuf.get()).b_visual.vi_end);
        }
    }
    if empty as c_int != 0 && *ml_get((*curbuf.get()).b_ml.ml_line_count) as c_int == NUL {
        ml_delete_flags((*curbuf.get()).b_ml.ml_line_count, ML_DEL_MESSAGE as c_int);
        deleted_lines(
            (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
            1 as linenr_T,
        );
        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            coladvance(curwin.get(), MAXCOL as c_int);
        }
    }
    auto_format(false_0 != 0, true_0 != 0);
}

pub(crate) unsafe extern "C" fn nv_open(mut cap: *mut cmdarg_T) {
    if (*(*cap).oap).op_type == OP_DELETE as c_int && (*cap).cmdchar == 'o' as c_int {
        clearop((*cap).oap);
        '_c2rust_label: {
            if (*cap).opcount >= 0 as c_int {
            } else {
                __assert_fail(
                    b"cap->opcount >= 0\0".as_ptr() as *const c_char,
                    b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                    6645 as c_uint,
                    b"void nv_open(cmdarg_T *)\0".as_ptr() as *const c_char,
                );
            }
        };
        nv_diffgetput(false_0 != 0, (*cap).opcount as size_t);
    } else if VIsual_active.get() {
        v_swap_corners((*cap).cmdchar);
    } else if bt_prompt(curbuf.get()) as c_int != 0
        && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_prompt_start.mark.lnum
    {
        clearopbeep((*cap).oap);
    } else {
        n_opencmd(cap);
    };
}
