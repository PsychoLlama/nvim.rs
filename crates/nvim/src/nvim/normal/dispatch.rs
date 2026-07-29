//! Turning a keystroke into a command: the table lookup, the count
//! and register that may precede it, and the bookkeeping that follows it.
//!
//! `find_command` resolves a character to a row of `nv_cmds`,
//! `normal_get_command_count` stacks the digits before it, and
//! `normal_get_additional_char` reads the extra character a row asks for.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn nv_compare(mut s1: *const c_void, mut s2: *const c_void) -> c_int {
    let mut c1: c_int = (*nv_cmds.ptr())[*(s1 as *const int16_t) as usize].cmd_char;
    let mut c2: c_int = (*nv_cmds.ptr())[*(s2 as *const int16_t) as usize].cmd_char;
    if c1 < 0 as c_int {
        c1 = -c1;
    }
    if c2 < 0 as c_int {
        c2 = -c2;
    }
    return if c1 == c2 {
        0 as c_int
    } else if c1 > c2 {
        1 as c_int
    } else {
        -1 as c_int
    };
}

pub unsafe extern "C" fn init_normal_cmds() {
    '_c2rust_label: {
        if ::core::mem::size_of::<[nv_cmd; 188]>()
            .wrapping_div(::core::mem::size_of::<nv_cmd>())
            .wrapping_div(
                (::core::mem::size_of::<[nv_cmd; 188]>()
                    .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                    == 0) as c_int as usize,
            )
            <= 32767 as usize
        {
        } else {
            __assert_fail(
                b"NV_CMDS_SIZE <= SHRT_MAX\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                390 as c_uint,
                b"void init_normal_cmds(void)\0".as_ptr() as *const c_char,
            );
        }
    };
    let mut i: int16_t = 0 as int16_t;
    while (i as c_int) < NV_CMDS_SIZE as int16_t as c_int {
        (*nv_cmd_idx.ptr())[i as usize] = i;
        i += 1;
    }
    qsort(
        nv_cmd_idx.ptr() as *mut c_void,
        NV_CMDS_SIZE,
        ::core::mem::size_of::<int16_t>(),
        Some(nv_compare as unsafe extern "C" fn(*const c_void, *const c_void) -> c_int),
    );
    let mut i_0: int16_t = 0;
    i_0 = 0 as int16_t;
    while (i_0 as c_int) < NV_CMDS_SIZE as int16_t as c_int {
        if i_0 as c_int != (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i_0 as usize] as usize].cmd_char {
            break;
        }
        i_0 += 1;
    }
    nv_max_linear.set(i_0 as c_int - 1 as c_int);
}

pub(crate) unsafe extern "C" fn find_command(mut cmdchar: c_int) -> c_int {
    if cmdchar >= 0x100 as c_int {
        return -1 as c_int;
    }
    if cmdchar < 0 as c_int {
        cmdchar = -cmdchar;
    }
    '_c2rust_label: {
        if nv_max_linear.get()
            < ::core::mem::size_of::<[nv_cmd; 188]>()
                .wrapping_div(::core::mem::size_of::<nv_cmd>())
                .wrapping_div(
                    (::core::mem::size_of::<[nv_cmd; 188]>()
                        .wrapping_rem(::core::mem::size_of::<nv_cmd>())
                        == 0) as c_int as usize,
                ) as c_int
        {
        } else {
            __assert_fail(
                b"nv_max_linear < (int)NV_CMDS_SIZE\0".as_ptr() as *const c_char,
                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                428 as c_uint,
                b"int find_command(int)\0".as_ptr() as *const c_char,
            );
        }
    };
    if cmdchar <= nv_max_linear.get() {
        return (*nv_cmd_idx.ptr())[cmdchar as usize] as c_int;
    }
    let mut bot: c_int = nv_max_linear.get() + 1 as c_int;
    let mut top: c_int = NV_CMDS_SIZE.wrapping_sub(1 as usize) as c_int;
    let mut idx: c_int = -1 as c_int;
    while bot <= top {
        let mut i: c_int = (top + bot) / 2 as c_int;
        let mut c: c_int = (*nv_cmds.ptr())[(*nv_cmd_idx.ptr())[i as usize] as usize].cmd_char;
        if c < 0 as c_int {
            c = -c;
        }
        if cmdchar == c {
            idx = (*nv_cmd_idx.ptr())[i as usize] as c_int;
            break;
        } else if cmdchar > c {
            bot = i + 1 as c_int;
        } else {
            top = i - 1 as c_int;
        }
    }
    return idx;
}

pub(crate) unsafe extern "C" fn normal_get_additional_char(mut s: *mut NormalState) {
    let mut cp: *mut c_int = ::core::ptr::null_mut::<c_int>();
    let mut repl: bool = false_0 != 0;
    let mut lit: bool = false_0 != 0;
    let mut lang: bool = false;
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    did_cursorhold.set(true_0 != 0);
    if (*s).ca.cmdchar == 'g' as c_int {
        (*s).ca.nchar = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).ca.nchar >= 0 as c_int
        {
            if (*s).ca.nchar < 256 as c_int {
                (*s).ca.nchar = (*langmap_mapchar.ptr())[(*s).ca.nchar as usize] as c_int;
            } else {
                (*s).ca.nchar = langmap_adjust_mb((*s).ca.nchar);
            }
        }
        (*s).need_flushbuf =
            (*s).need_flushbuf as c_int | add_to_showcmd((*s).ca.nchar) as c_int != 0;
        if (*s).ca.nchar == 'r' as c_int
            || (*s).ca.nchar == '\'' as c_int
            || (*s).ca.nchar == '`' as c_int
            || (*s).ca.nchar == Ctrl_BSL
        {
            cp = &raw mut (*s).ca.extra_char;
            if (*s).ca.nchar != 'r' as c_int {
                lit = true_0 != 0;
            } else {
                repl = true_0 != 0;
            }
        } else {
            cp = ::core::ptr::null_mut::<c_int>();
        }
    } else {
        if (*s).ca.cmdchar == 'r' as c_int {
            repl = true_0 != 0;
        }
        cp = &raw mut (*s).ca.nchar;
    }
    lang =
        repl as c_int != 0 || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0;
    if !cp.is_null() {
        let mut langmap_active: bool = false_0 != 0;
        if repl {
            State.set(MODE_REPLACE as c_int);
            ui_cursor_shape_no_check_conceal();
        }
        if lang as c_int != 0 && (*curbuf.get()).b_p_iminsert == B_IMODE_LMAP as OptInt {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
            if repl {
                State.set(MODE_LREPLACE as c_int);
            } else {
                State.set(MODE_LANGMAP as c_int);
            }
            langmap_active = true_0 != 0;
        }
        *cp = plain_vgetc();
        if langmap_active {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        State.set(MODE_NORMAL_BUSY as c_int);
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd(*cp) as c_int != 0;
        if !lit {
            if *cp == Ctrl_K
                && ((*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_LANG != 0
                    || cp == &raw mut (*s).ca.extra_char)
                && vim_strchr(p_cpo.get(), CPO_DIGRAPH).is_null()
            {
                (*s).c = get_digraph(false_0 != 0);
                if (*s).c > 0 as c_int {
                    *cp = (*s).c;
                    del_from_showcmd(3 as c_int);
                    (*s).need_flushbuf =
                        (*s).need_flushbuf as c_int | add_to_showcmd(*cp) as c_int != 0;
                }
            }
            if *p_langmap.get() as c_int != 0
                && !lang
                && (p_lrm.get() != 0
                    || (if vgetc_busy.get() != 0 {
                        (typebuf_maplen() == 0 as c_int) as c_int
                    } else {
                        KeyTyped.get() as c_int
                    }) != 0)
                && KeyStuffed.get() == 0
                && *cp >= 0 as c_int
            {
                if *cp < 256 as c_int {
                    *cp = (*langmap_mapchar.ptr())[*cp as usize] as c_int;
                } else {
                    *cp = langmap_adjust_mb(*cp);
                }
            }
        }
        if cp == &raw mut (*s).ca.extra_char
            && (*s).ca.nchar == Ctrl_BSL
            && ((*s).ca.extra_char == Ctrl_N || (*s).ca.extra_char == Ctrl_G)
        {
            (*s).ca.cmdchar = Ctrl_BSL;
            (*s).ca.nchar = (*s).ca.extra_char;
            (*s).idx = find_command((*s).ca.cmdchar);
        } else if ((*s).ca.nchar == 'n' as c_int || (*s).ca.nchar == 'N' as c_int)
            && (*s).ca.cmdchar == 'g' as c_int
        {
            (*(*s).ca.oap).op_type = get_op_type(*cp, NUL);
        } else if *cp == Ctrl_BSL {
            let mut towait: c_int = if p_ttm.get() >= 0 as OptInt {
                p_ttm.get() as c_int
            } else {
                p_tm.get() as c_int
            };
            loop {
                (*s).c = vpeekc();
                if !((*s).c <= 0 as c_int && towait > 0 as c_int) {
                    break;
                }
                do_sleep(
                    (if towait > 50 as c_int {
                        50 as c_int
                    } else {
                        towait
                    }) as int64_t,
                    false_0 != 0,
                );
                towait -= 50 as c_int;
            }
            if (*s).c > 0 as c_int {
                (*s).c = plain_vgetc();
                if (*s).c != Ctrl_N && (*s).c != Ctrl_G {
                    vungetc((*s).c);
                } else {
                    (*s).ca.cmdchar = Ctrl_BSL;
                    (*s).ca.nchar = (*s).c;
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                827 as c_uint,
                                b"void normal_get_additional_char(NormalState *)\0".as_ptr()
                                    as *const c_char,
                            );
                        }
                    };
                }
            }
        }
        if lang {
            (*no_mapping.ptr()) -= 1;
            let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
            let mut prev_code: c_int = (*s).ca.nchar;
            loop {
                (*s).c = vpeekc();
                if !((*s).c > 0 as c_int
                    && ((*s).c >= 0x100 as c_int
                        || (*utf8len_tab.ptr())[vpeekc() as usize] as c_int > 1 as c_int))
                {
                    break;
                }
                (*s).c = plain_vgetc();
                if !utf_iscomposing(prev_code, (*s).c, &raw mut state) {
                    vungetc((*s).c);
                    break;
                } else {
                    if (*s).ca.nchar_len == 0 as c_int {
                        (*s).ca.nchar_len = utf_char2bytes(
                            (*s).ca.nchar,
                            &raw mut (*s).ca.nchar_composing as *mut c_char,
                        );
                    }
                    if (*s).ca.nchar_len + utf_char2len((*s).c)
                        < ::core::mem::size_of::<[c_char; 32]>() as c_int
                    {
                        (*s).ca.nchar_len += utf_char2bytes(
                            (*s).c,
                            (&raw mut (*s).ca.nchar_composing as *mut c_char)
                                .offset((*s).ca.nchar_len as isize),
                        );
                    }
                    prev_code = (*s).c;
                }
            }
            (*s).ca.nchar_composing[(*s).ca.nchar_len as usize] = NUL as c_char;
            (*no_mapping.ptr()) += 1;
            (*no_u_sync.ptr()) += 1;
            gotchars_ignore();
            (*no_u_sync.ptr()) -= 1;
        }
    }
    (*no_mapping.ptr()) -= 1;
    (*allow_keys.ptr()) -= 1;
}

pub(crate) unsafe extern "C" fn normal_invert_horizontal(mut s: *mut NormalState) {
    match (*s).ca.cmdchar {
        108 => {
            (*s).ca.cmdchar = 'h' as c_int;
        }
        K_RIGHT => {
            (*s).ca.cmdchar = K_LEFT;
        }
        K_S_RIGHT => {
            (*s).ca.cmdchar = K_S_LEFT;
        }
        -22269 => {
            (*s).ca.cmdchar = -(253 as c_int + ((KE_C_LEFT as c_int) << 8 as c_int));
        }
        104 => {
            (*s).ca.cmdchar = 'l' as c_int;
        }
        K_LEFT => {
            (*s).ca.cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*s).ca.cmdchar = K_S_RIGHT;
        }
        -22013 => {
            (*s).ca.cmdchar = -(253 as c_int + ((KE_C_RIGHT as c_int) << 8 as c_int));
        }
        62 => {
            (*s).ca.cmdchar = '<' as c_int;
        }
        60 => {
            (*s).ca.cmdchar = '>' as c_int;
        }
        _ => {}
    }
    (*s).idx = find_command((*s).ca.cmdchar);
}

pub(crate) unsafe extern "C" fn normal_get_command_count(mut s: *mut NormalState) -> bool {
    if VIsual_active.get() as c_int != 0 && VIsual_select.get() as c_int != 0 {
        return false_0 != 0;
    }
    while (*s).c >= '1' as c_int && (*s).c <= '9' as c_int
        || (*s).ca.count0 != 0 as c_int
            && ((*s).c == K_DEL
                || (*s).c == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int))
                || (*s).c == '0' as c_int)
    {
        if (*s).c == K_DEL || (*s).c == -(253 as c_int + ((KE_KDEL as c_int) << 8 as c_int)) {
            (*s).ca.count0 /= 10 as c_int;
            del_from_showcmd(4 as c_int);
        } else if (*s).ca.count0 > 99999999 as c_int {
            (*s).ca.count0 = 999999999 as c_int;
        } else {
            (*s).ca.count0 = (*s).ca.count0 * 10 as c_int + ((*s).c - '0' as c_int);
        }
        if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
            set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
        }
        if (*s).ctrl_w {
            (*no_mapping.ptr()) += 1;
            (*allow_keys.ptr()) += 1;
        }
        (*no_zero_mapping.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as c_int
        {
            if (*s).c < 256 as c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_zero_mapping.ptr()) -= 1;
        if (*s).ctrl_w {
            (*no_mapping.ptr()) -= 1;
            (*allow_keys.ptr()) -= 1;
        }
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd((*s).c) as c_int != 0;
    }
    if (*s).c == Ctrl_W && !(*s).ctrl_w && (*s).oa.op_type == OP_NOP as c_int {
        (*s).ctrl_w = true_0 != 0;
        (*s).ca.opcount = (*s).ca.count0;
        (*s).ca.count0 = 0 as c_int;
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        (*s).c = plain_vgetc();
        if *p_langmap.get() as c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as c_int) as c_int
                } else {
                    KeyTyped.get() as c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && (*s).c >= 0 as c_int
        {
            if (*s).c < 256 as c_int {
                (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
            } else {
                (*s).c = langmap_adjust_mb((*s).c);
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        (*s).need_flushbuf = (*s).need_flushbuf as c_int | add_to_showcmd((*s).c) as c_int != 0;
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn normal_finish_command(mut s: *mut NormalState) {
    let mut did_visual_op: bool = false_0 != 0;
    if !(*s).command_finished {
        if !finish_op.get()
            && (*s).oa.op_type == 0
            && ((*s).idx < 0 as c_int
                || (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_KEEPREG == 0)
        {
            clearop(&raw mut (*s).oa);
            set_reg_var(get_default_register_name());
        }
        if (*s).old_mapped_len > 0 as c_int {
            (*s).old_mapped_len = typebuf_maplen();
        }
        if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int))
            && (*s).ca.cmdchar != -(253 as c_int + ((KE_MOUSEMOVE as c_int) << 8 as c_int))
        {
            did_visual_op = VIsual_active.get() as c_int != 0
                && (*s).oa.op_type != OP_NOP as c_int
                && (*s).oa.op_type != OP_COLON as c_int;
            do_pending_operator(&raw mut (*s).ca, (*s).old_col, false_0 != 0);
        }
        if normal_need_redraw_mode_message(s) {
            normal_redraw_mode_message(s);
        }
    }
    msg_nowait.set(false_0 != 0);
    if finish_op.get() as c_int != 0 || did_visual_op as c_int != 0 {
        set_reg_var(get_default_register_name());
    }
    let prev_finish_op: bool = finish_op.get();
    if (*s).oa.op_type == OP_NOP as c_int {
        finish_op.set(false_0 != 0);
        may_trigger_modechanged();
    }
    if prev_finish_op as c_int != 0
        || (*s).ca.cmdchar == 'r' as c_int
        || (*s).ca.cmdchar == 'g' as c_int && (*s).ca.nchar == 'r' as c_int
    {
        ui_cursor_shape();
    }
    if (*s).oa.op_type == OP_NOP as c_int
        && (*s).oa.regname == 0 as c_int
        && (*s).ca.cmdchar != -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int))
    {
        clear_showcmd();
    }
    checkpcmark();
    xfree((*s).ca.searchbuf as *mut c_void);
    mb_check_adjust_col(curwin.get() as *mut c_void);
    if (*curwin.get()).w_onebuf_opt.wo_scb != 0 && (*s).toplevel as c_int != 0 {
        validate_cursor(curwin.get());
        do_check_scrollbind(true_0 != 0);
    }
    if (*curwin.get()).w_onebuf_opt.wo_crb != 0 && (*s).toplevel as c_int != 0 {
        validate_cursor(curwin.get());
        do_check_cursorbind();
    }
    if (*s).oa.op_type == OP_NOP as c_int
        && (restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as c_int
            || restart_VIsual_select.get() == 1 as c_int)
        && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty() as c_int != 0
        && (*s).oa.regname == 0 as c_int
    {
        if restart_VIsual_select.get() == 1 as c_int {
            VIsual_select.set(true_0 != 0);
            VIsual_select_reg.set(0 as c_int);
            may_trigger_modechanged();
            showmode();
            restart_VIsual_select.set(0 as c_int);
        }
        if restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && (*s).old_mapped_len == 0 as c_int
        {
            edit(restart_edit.get(), false_0 != 0, 1 as c_int);
        }
    }
    if restart_VIsual_select.get() == 2 as c_int {
        restart_VIsual_select.set(1 as c_int);
    }
    opcount.set((*s).ca.opcount);
}

pub(crate) unsafe extern "C" fn normal_execute(mut state: *mut VimState, mut key: c_int) -> c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    (*s).command_finished = false_0 != 0;
    (*s).ctrl_w = false_0 != 0;
    (*s).old_col = (*curwin.get()).w_curswant as c_int;
    (*s).c = key;
    if *p_langmap.get() as c_int != 0
        && get_real_state() != MODE_SELECT as c_int
        && (p_lrm.get() != 0
            || (if vgetc_busy.get() != 0 {
                (typebuf_maplen() == 0 as c_int) as c_int
            } else {
                KeyTyped.get() as c_int
            }) != 0)
        && KeyStuffed.get() == 0
        && (*s).c >= 0 as c_int
    {
        if (*s).c < 256 as c_int {
            (*s).c = (*langmap_mapchar.ptr())[(*s).c as usize] as c_int;
        } else {
            (*s).c = langmap_adjust_mb((*s).c);
        }
    }
    if restart_edit.get() == 0 as c_int {
        (*s).old_mapped_len = 0 as c_int;
    } else if (*s).old_mapped_len != 0
        || VIsual_active.get() as c_int != 0
            && (*s).mapped_len == 0 as c_int
            && typebuf_maplen() > 0 as c_int
    {
        (*s).old_mapped_len = typebuf_maplen();
    }
    if (*s).c == NUL {
        (*s).c = K_ZERO;
    }
    if VIsual_active.get() as c_int != 0
        && VIsual_select.get() as c_int != 0
        && (vim_isprintc((*s).c) as c_int != 0
            || (*s).c == NL
            || (*s).c == CAR
            || (*s).c == K_KENTER)
    {
        let mut len: c_int = ins_char_typebuf(vgetc_char.get(), vgetc_mod_mask.get(), true_0 != 0);
        if KeyTyped.get() {
            ungetchars(len);
        }
        if restart_edit.get() != 0 as c_int {
            (*s).c = 'd' as c_int;
        } else {
            (*s).c = 'c' as c_int;
        }
        msg_nowait.set(true_0 != 0);
        (*s).old_mapped_len = 0 as c_int;
    }
    (*s).need_flushbuf = add_to_showcmd((*s).c);
    while normal_get_command_count(s) {}
    if (*s).c == -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int)) {
        (*s).oa.prev_opcount = (*s).ca.opcount;
        (*s).oa.prev_count0 = (*s).ca.count0;
    } else if (*s).ca.opcount != 0 as c_int {
        if (*s).ca.count0 != 0 {
            if (*s).ca.opcount >= 999999999 as c_int / (*s).ca.count0 {
                (*s).ca.count0 = 999999999 as c_int;
            } else {
                (*s).ca.count0 *= (*s).ca.opcount;
            }
        } else {
            (*s).ca.count0 = (*s).ca.opcount;
        }
    }
    (*s).ca.opcount = (*s).ca.count0;
    (*s).ca.count1 = if (*s).ca.count0 == 0 as c_int {
        1 as c_int
    } else {
        (*s).ca.count0
    };
    if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
        set_vcount(
            (*s).ca.count0 as int64_t,
            (*s).ca.count1 as int64_t,
            (*s).set_prevcount,
        );
    }
    if (*s).ctrl_w {
        (*s).ca.nchar = (*s).c;
        (*s).ca.cmdchar = Ctrl_W;
    } else {
        (*s).ca.cmdchar = (*s).c;
    }
    (*s).idx = find_command((*s).ca.cmdchar);
    if (*s).idx < 0 as c_int {
        clearopbeep(&raw mut (*s).oa);
        (*s).command_finished = true_0 != 0;
    } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_NCW != 0
        && check_text_or_curbuf_locked(&raw mut (*s).oa) as c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else if VIsual_active.get() as c_int != 0
        && normal_handle_special_visual_command(s) as c_int != 0
    {
        (*s).command_finished = true_0 != 0;
    } else {
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && KeyTyped.get() as c_int != 0
            && KeyStuffed.get() == 0
            && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_RL != 0
        {
            normal_invert_horizontal(s);
        }
        if normal_need_additional_char(s) {
            normal_get_additional_char(s);
        }
        if (*s).need_flushbuf {
            ui_flush();
        }
        if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int))
            && (*s).ca.cmdchar != -(253 as c_int + ((KE_EVENT as c_int) << 8 as c_int))
        {
            did_cursorhold.set(false_0 != 0);
        }
        State.set(MODE_NORMAL as c_int);
        if (*s).ca.nchar == ESC || (*s).ca.extra_char == ESC {
            clearop(&raw mut (*s).oa);
            (*s).command_finished = true_0 != 0;
        } else {
            if (*s).ca.cmdchar != -(253 as c_int + ((KE_IGNORE as c_int) << 8 as c_int)) {
                msg_didout.set(false_0 != 0);
                msg_col.set(0 as c_int);
            }
            (*s).old_pos = (*curwin.get()).w_cursor;
            if !VIsual_active.get() && km_startsel.get() as c_int != 0 {
                if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SS != 0 {
                    start_selection();
                    unshift_special(&raw mut (*s).ca);
                    (*s).idx = find_command((*s).ca.cmdchar);
                    '_c2rust_label: {
                        if (*s).idx >= 0 as c_int {
                        } else {
                            __assert_fail(
                                b"s->idx >= 0\0".as_ptr() as *const c_char,
                                b"src/nvim/normal.rs\0".as_ptr() as *const c_char,
                                1239 as c_uint,
                                b"int normal_execute(VimState *, int)\0".as_ptr() as *const c_char,
                            );
                        }
                    };
                } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SSS != 0
                    && mod_mask.get() & MOD_MASK_SHIFT != 0
                {
                    start_selection();
                    (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
                }
            }
            (*s).ca.arg = (*nv_cmds.ptr())[(*s).idx as usize].cmd_arg as c_int;
            (*nv_cmds.ptr())[(*s).idx as usize]
                .cmd_func
                .expect("non-null function pointer")(&raw mut (*s).ca);
        }
    }
    normal_finish_command(s);
    return 1 as c_int;
}

pub(crate) unsafe extern "C" fn prep_redo_cmd(mut cap: *mut cmdarg_T) {
    prep_redo(
        (*(*cap).oap).regname,
        (*cap).count0,
        NUL,
        (*cap).cmdchar,
        NUL,
        NUL,
        NUL,
    );
    if (*cap).nchar_len > 0 as c_int {
        AppendToRedobuff(&raw mut (*cap).nchar_composing as *mut c_char);
    } else {
        AppendCharToRedobuff((*cap).nchar);
    };
}

pub unsafe extern "C" fn prep_redo(
    mut regname: c_int,
    mut num: c_int,
    mut cmd1: c_int,
    mut cmd2: c_int,
    mut cmd3: c_int,
    mut cmd4: c_int,
    mut cmd5: c_int,
) {
    prep_redo_num2(regname, num, cmd1, cmd2, 0 as c_int, cmd3, cmd4, cmd5);
}

pub unsafe extern "C" fn prep_redo_num2(
    mut regname: c_int,
    mut num1: c_int,
    mut cmd1: c_int,
    mut cmd2: c_int,
    mut num2: c_int,
    mut cmd3: c_int,
    mut cmd4: c_int,
    mut cmd5: c_int,
) {
    ResetRedobuff();
    if regname != 0 as c_int {
        AppendCharToRedobuff('"' as c_int);
        AppendCharToRedobuff(regname);
    }
    if num1 != 0 as c_int {
        AppendNumberToRedobuff(num1);
    }
    if cmd1 != NUL {
        AppendCharToRedobuff(cmd1);
    }
    if cmd2 != NUL {
        AppendCharToRedobuff(cmd2);
    }
    if num2 != 0 as c_int {
        AppendNumberToRedobuff(num2);
    }
    if cmd3 != NUL {
        AppendCharToRedobuff(cmd3);
    }
    if cmd4 != NUL {
        AppendCharToRedobuff(cmd4);
    }
    if cmd5 != NUL {
        AppendCharToRedobuff(cmd5);
    }
}

pub(crate) unsafe extern "C" fn checkclearop(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as c_int {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}

pub(crate) unsafe extern "C" fn checkclearopq(mut oap: *mut oparg_T) -> bool {
    if (*oap).op_type == OP_NOP as c_int && !VIsual_active.get() {
        return false_0 != 0;
    }
    clearopbeep(oap);
    return true_0 != 0;
}

pub unsafe extern "C" fn clearop(mut oap: *mut oparg_T) {
    (*oap).op_type = OP_NOP as c_int;
    (*oap).regname = 0 as c_int;
    (*oap).motion_force = NUL;
    (*oap).use_reg_one = false_0 != 0;
    motion_force.set(NUL);
}

pub unsafe extern "C" fn clearopbeep(mut oap: *mut oparg_T) {
    clearop(oap);
    beep_flush();
}

pub(crate) unsafe extern "C" fn unshift_special(mut cap: *mut cmdarg_T) {
    match (*cap).cmdchar {
        K_S_RIGHT => {
            (*cap).cmdchar = K_RIGHT;
        }
        K_S_LEFT => {
            (*cap).cmdchar = K_LEFT;
        }
        -1277 => {
            (*cap).cmdchar = K_UP;
        }
        -1533 => {
            (*cap).cmdchar = K_DOWN;
        }
        K_S_HOME => {
            (*cap).cmdchar = K_HOME;
        }
        K_S_END => {
            (*cap).cmdchar = K_END;
        }
        _ => {}
    }
    (*cap).cmdchar = simplify_key((*cap).cmdchar, mod_mask.ptr());
}

pub unsafe extern "C" fn may_clear_cmdline() {
    if mode_displayed.get() {
        clear_cmdline.set(true_0 != 0);
    } else {
        clear_showcmd();
    };
}
