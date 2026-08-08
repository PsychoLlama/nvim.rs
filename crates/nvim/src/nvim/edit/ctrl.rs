//! The CTRL- commands that change what Insert mode *is*.
//!
//! `ins_esc` ends it, and it is the delicate one: the cursor moves back one
//! character, the `count` may mean the whole insert repeats, and Replace
//! mode's stack has to be cleared.  `ins_ctrl_o` leaves it for exactly one
//! Normal-mode command; `ins_insert` switches between Insert and Replace;
//! `ins_ctrl_g` is the CTRL-G prefix (`CTRL-G j`, `CTRL-G k`, `CTRL-G u`,
//! `CTRL-G U`); `ins_ctrl_hat` and `ins_ctrl_` toggle the language mappings
//! and 'revins'.  `ins_reg` is CTRL-R, which inserts a register's contents
//! and is here rather than beside `insertchar` because what it really does
//! is stuff text into the input stream.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ins_reg() {
    unsafe {
        let mut need_redraw: bool = false_0 != 0;
        let mut literally: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut vis_active: ::core::ffi::c_int = VIsual_active.get() as ::core::ffi::c_int;
        pc_status.set(PC_STATUS_UNSET);
        if redrawing() as ::core::ffi::c_int != 0 && !char_avail() {
            ins_redraw(false_0 != 0);
            edit_putchar('"' as ::core::ffi::c_int, true_0 != 0);
            add_to_showcmd_c(Ctrl_R);
        }
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut regname: ::core::ffi::c_int = plain_vgetc();
        if *p_langmap.get() as ::core::ffi::c_int != 0
            && true
            && (p_lrm.get() != 0
                || (if vgetc_busy.get() != 0 {
                    (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                } else {
                    KeyTyped.get() as ::core::ffi::c_int
                }) != 0)
            && KeyStuffed.get() == 0
            && regname >= 0 as ::core::ffi::c_int
        {
            if regname < 256 as ::core::ffi::c_int {
                regname = (*langmap_mapchar.ptr())[regname as usize] as ::core::ffi::c_int;
            } else {
                regname = langmap_adjust_mb(regname);
            }
        }
        if regname == Ctrl_R || regname == Ctrl_O || regname == Ctrl_P {
            literally = regname;
            add_to_showcmd_c(literally);
            regname = plain_vgetc();
            if *p_langmap.get() as ::core::ffi::c_int != 0
                && true
                && (p_lrm.get() != 0
                    || (if vgetc_busy.get() != 0 {
                        (typebuf_maplen() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
                    } else {
                        KeyTyped.get() as ::core::ffi::c_int
                    }) != 0)
                && KeyStuffed.get() == 0
                && regname >= 0 as ::core::ffi::c_int
            {
                if regname < 256 as ::core::ffi::c_int {
                    regname = (*langmap_mapchar.ptr())[regname as usize] as ::core::ffi::c_int;
                } else {
                    regname = langmap_adjust_mb(regname);
                }
            }
        }
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        (*no_u_sync.ptr()) += 1;
        if regname == '=' as ::core::ffi::c_int {
            let mut curpos: pos_T = (*curwin.get()).w_cursor;
            u_sync_once.set(2 as ::core::ffi::c_int);
            regname = get_expr_register();
            (*curwin.get()).w_cursor = curpos;
            check_cursor(curwin.get());
        }
        if regname == NUL || !valid_yank_reg(regname, false_0 != 0) {
            vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
            need_redraw = true_0 != 0;
        } else {
            let mut reg: *mut yankreg_T =
                get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
            if literally == Ctrl_O || literally == Ctrl_P {
                AppendCharToRedobuff(Ctrl_R);
                AppendCharToRedobuff(literally);
                AppendCharToRedobuff(regname);
                do_put(
                    regname,
                    ::core::ptr::null_mut::<yankreg_T>(),
                    BACKWARD as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    (if literally == Ctrl_P {
                        PUT_FIXINDENT as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) | PUT_CURSEND as ::core::ffi::c_int,
                );
            } else if (*reg).y_size > 1 as size_t
                && is_literal_register(regname) as ::core::ffi::c_int != 0
            {
                AppendCharToRedobuff(Ctrl_R);
                AppendCharToRedobuff(regname);
                do_put(
                    regname,
                    ::core::ptr::null_mut::<yankreg_T>(),
                    BACKWARD as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    PUT_CURSEND as ::core::ffi::c_int,
                );
            } else if insert_reg(
                regname,
                ::core::ptr::null_mut::<yankreg_T>(),
                literally != 0,
            ) == FAIL
            {
                vim_beep(kOptBoFlagRegister as ::core::ffi::c_int as ::core::ffi::c_uint);
                need_redraw = true_0 != 0;
            } else if stop_insert_mode.get() {
                need_redraw = true_0 != 0;
            }
        }
        (*no_u_sync.ptr()) -= 1;
        if u_sync_once.get() == 1 as ::core::ffi::c_int {
            ins_need_undo.set(true_0 != 0);
        }
        u_sync_once.set(0 as ::core::ffi::c_int);
        if need_redraw as ::core::ffi::c_int != 0 || stuff_empty() as ::core::ffi::c_int != 0 {
            edit_unputchar();
        }
        clear_showcmd();
        if vis_active == 0 && VIsual_active.get() as ::core::ffi::c_int != 0 {
            end_visual_mode();
        }
    }
}

pub(crate) unsafe extern "C" fn ins_ctrl_g() {
    unsafe {
        setcursor();
        (*no_mapping.ptr()) += 1;
        (*allow_keys.ptr()) += 1;
        let mut c: ::core::ffi::c_int = plain_vgetc();
        (*no_mapping.ptr()) -= 1;
        (*allow_keys.ptr()) -= 1;
        match c {
            K_UP | Ctrl_K | 107 => {
                ins_up(true_0 != 0);
            }
            K_DOWN | Ctrl_J | 106 => {
                ins_down(true_0 != 0);
            }
            117 => {
                u_sync(true_0 != 0);
                ins_need_undo.set(true_0 != 0);
                update_Insstart_orig.set(false_0 != 0);
                Insstart.set((*curwin.get()).w_cursor);
            }
            85 => {
                dont_sync_undo.set(kNone);
            }
            ESC => {}
            _ => {
                vim_beep(kOptBoFlagCtrlg as ::core::ffi::c_int as ::core::ffi::c_uint);
            }
        };
    }
}

pub(crate) unsafe extern "C" fn ins_ctrl_hat() {
    unsafe {
        if map_to_exists_mode(
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            MODE_LANGMAP,
            false_0 != 0,
        ) {
            if State.get() & MODE_LANGMAP != 0 {
                (*curbuf.get()).b_p_iminsert = B_IMODE_NONE as OptInt;
                (*State.ptr()) &= !(MODE_LANGMAP);
            } else {
                (*curbuf.get()).b_p_iminsert = B_IMODE_LMAP as OptInt;
                (*State.ptr()) |= MODE_LANGMAP;
            }
        }
        set_iminsert_global(curbuf.get());
        showmode();
        status_redraw_curbuf();
    }
}

pub(crate) unsafe extern "C" fn ins_esc(
    mut count: *mut ::core::ffi::c_int,
    mut cmdchar: ::core::ffi::c_int,
    mut nomove: bool,
) -> bool {
    unsafe {
        static disabled_redraw: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        check_spell_redraw();
        let mut temp: ::core::ffi::c_int = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
        if disabled_redraw.get() {
            (*RedrawingDisabled.ptr()) -= 1;
            disabled_redraw.set(false_0 != 0);
        }
        if !arrow_used.get() {
            if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
                AppendToRedobuff(ESC_STR.as_ptr());
            }
            if *count > 0 as ::core::ffi::c_int {
                line_breakcheck();
                if got_int.get() {
                    *count = 0 as ::core::ffi::c_int;
                }
            }
            *count -= 1;
            if *count > 0 as ::core::ffi::c_int {
                if !vim_strchr(p_cpo.get(), CPO_REPLCNT).is_null() {
                    (*State.ptr()) &= !(REPLACE_FLAG);
                }
                start_redo_ins();
                if cmdchar == 'r' as ::core::ffi::c_int || cmdchar == 'v' as ::core::ffi::c_int {
                    stuffRedoReadbuff(ESC_STR.as_ptr());
                }
                (*RedrawingDisabled.ptr()) += 1;
                disabled_redraw.set(true_0 != 0);
                return false_0 != 0;
            }
            stop_insert(
                &raw mut (*curwin.get()).w_cursor,
                true_0,
                nomove as ::core::ffi::c_int,
            );
            undisplay_dollar();
        }
        if cmdchar != 'r' as ::core::ffi::c_int && cmdchar != 'v' as ::core::ffi::c_int {
            ins_apply_autocmds(EVENT_INSERTLEAVEPRE);
        }
        if restart_edit.get() == NUL && temp == (*curwin.get()).w_cursor.col {
            (*curwin.get()).w_set_curswant = true_0;
        }
        if (*cmdmod.ptr()).cmod_flags & CMOD_KEEPJUMPS as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            let mut view: fmarkv_T = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
            let fmarkp___: *mut fmark_T = &raw mut (*curbuf.get()).b_last_insert;
            free_fmark(*fmarkp___);
            let fmarkp__: *mut fmark_T = fmarkp___;
            (*fmarkp__).mark = (*curwin.get()).w_cursor;
            (*fmarkp__).fnum = (*curbuf.get()).handle as ::core::ffi::c_int;
            (*fmarkp__).timestamp = os_time();
            (*fmarkp__).view = view;
            (*fmarkp__).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        }
        if !nomove
            && ((*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int
                || (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int)
            && (restart_edit.get() == NUL || gchar_cursor() == NUL && !VIsual_active.get())
            && !revins_on.get()
        {
            if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                || get_ve_flags(curwin.get())
                    == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                oneleft();
                if restart_edit.get() != NUL {
                    (*curwin.get()).w_cursor.coladd += 1;
                }
            } else {
                (*curwin.get()).w_cursor.col -= 1;
                (*curwin.get()).w_valid &= !(VALID_WCOL | VALID_VIRTCOL);
                mb_adjust_cursor();
            }
        }
        State.set(MODE_NORMAL);
        may_trigger_modechanged();
        if gchar_cursor() == TAB || buf_meta_total(curbuf.get(), kMTMetaInline) > 0 as uint32_t {
            (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        }
        setmouse();
        ui_cursor_shape();
        if reg_recording.get() != 0 as ::core::ffi::c_int || restart_edit.get() != NUL {
            showmode();
        } else if p_smd.get() != 0
            && (got_int.get() as ::core::ffi::c_int != 0 || !skip_showmode())
            && !(p_ch.get() == 0 as OptInt && !ui_has(kUIMessages))
        {
            unshowmode(false_0 != 0);
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn ins_ctrl_() {
    unsafe {
        if revins_on.get() as ::core::ffi::c_int != 0
            && revins_chars.get() != 0
            && revins_scol.get() >= 0 as ::core::ffi::c_int
        {
            while gchar_cursor() != NUL && {
                let c2rust_fresh4 = revins_chars.get();
                revins_chars.set(revins_chars.get() - 1);
                c2rust_fresh4 != 0
            } {
                (*curwin.get()).w_cursor.col += 1;
            }
        }
        p_ri.set((p_ri.get() == 0) as ::core::ffi::c_int);
        revins_on.set(State.get() == MODE_INSERT && p_ri.get() != 0);
        if revins_on.get() {
            revins_scol.set((*curwin.get()).w_cursor.col as ::core::ffi::c_int);
            (*revins_legal.ptr()) += 1;
            revins_chars.set(0 as ::core::ffi::c_int);
            undisplay_dollar();
        } else {
            revins_scol.set(-1 as ::core::ffi::c_int);
        }
        showmode();
    }
}

pub(crate) unsafe extern "C" fn ins_insert(mut replaceState: ::core::ffi::c_int) {
    unsafe {
        set_vim_var_string(
            VV_INSERTMODE,
            if State.get() & REPLACE_FLAG != 0 {
                b"i\0".as_ptr() as *const ::core::ffi::c_char
            } else if replaceState == MODE_VREPLACE {
                b"v\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"r\0".as_ptr() as *const ::core::ffi::c_char
            },
            1 as ptrdiff_t,
        );
        ins_apply_autocmds(EVENT_INSERTCHANGE);
        if State.get() & REPLACE_FLAG != 0 {
            State.set(MODE_INSERT | State.get() & MODE_LANGMAP);
        } else {
            State.set(replaceState | State.get() & MODE_LANGMAP);
        }
        may_trigger_modechanged();
        AppendCharToRedobuff(K_INS);
        showmode();
        ui_cursor_shape();
    }
}

pub(crate) unsafe extern "C" fn ins_ctrl_o() {
    unsafe {
        restart_VIsual_select.set(0 as ::core::ffi::c_int);
        if State.get() & VREPLACE_FLAG != 0 {
            restart_edit.set('V' as ::core::ffi::c_int);
        } else if State.get() & REPLACE_FLAG != 0 {
            restart_edit.set('R' as ::core::ffi::c_int);
        } else {
            restart_edit.set('I' as ::core::ffi::c_int);
        }
        if virtual_active(curwin.get()) {
            ins_at_eol.set(false_0 != 0);
        } else {
            ins_at_eol.set(gchar_cursor() == NUL);
        };
    }
}

pub unsafe extern "C" fn get_can_cindent() -> bool {
    return can_cindent.get();
}

pub unsafe extern "C" fn set_can_cindent(mut val: bool) {
    can_cindent.set(val);
}
