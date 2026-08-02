//! The mode message on the last line -- `--INSERT--` and friends.
//!
//! [`showmode`] is the whole of it: what to print for the current mode, the
//! `CTRL-X` submode text Insert-mode completion puts there instead, the
//! `recording @q` suffix, and clearing the command line around all of it.
//! [`unshowmode`] and [`clearmode`] take it away again.
//!
//! [`comp_col`] is here because it is the same real estate: it decides how many
//! columns at the right of the last line belong to the ruler and to `'showcmd'`,
//! which is what bounds every message printed there.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn skip_showmode() -> bool {
    unsafe {
        if global_busy.get() != 0
            || msg_silent.get() != 0 as ::core::ffi::c_int
            || !redrawing()
            || char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get()
        {
            redraw_mode.set(true_0 != 0);
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn showmode() -> ::core::ffi::c_int {
    unsafe {
        let mut length: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        msg_ext_ui_flush();
        msg_grid_validate();
        let mut do_mode: bool = p_smd.get() != 0
            && msg_silent.get() == 0 as ::core::ffi::c_int
            && (State.get() & MODE_TERMINAL != 0
                || State.get() & MODE_INSERT != 0
                || restart_edit.get() != NUL
                || VIsual_active.get() as ::core::ffi::c_int != 0);
        let mut can_show_mode: bool =
            p_ch.get() != 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0;
        if (do_mode as ::core::ffi::c_int != 0 || reg_recording.get() != 0 as ::core::ffi::c_int)
            && can_show_mode as ::core::ffi::c_int != 0
        {
            if skip_showmode() {
                return 0 as ::core::ffi::c_int;
            }
            let mut nwr_save: bool = need_wait_return.get();
            msg_check_for_delay(false_0 != 0);
            let mut need_clear: bool = clear_cmdline.get();
            if clear_cmdline.get() as ::core::ffi::c_int != 0
                && cmdline_row.get() < Rows.get() - 1 as ::core::ffi::c_int
            {
                msg_clr_cmdline();
            }
            msg_pos_mode();
            let mut hl_id: ::core::ffi::c_int = HLF_CM;
            msg_no_more.set(true_0 != 0);
            let mut save_lines_left: ::core::ffi::c_int = lines_left.get();
            lines_left.set(0 as ::core::ffi::c_int);
            if do_mode {
                msg_puts_hl(
                    b"--\0".as_ptr() as *const ::core::ffi::c_char,
                    hl_id,
                    false_0 != 0,
                );
                if !(*edit_submode.ptr()).is_null()
                    && !shortmess(SHM_COMPLETIONMENU as ::core::ffi::c_int)
                {
                    if ui_has(kUIMessages) {
                        length = INT_MAX;
                    } else {
                        length =
                            (Rows.get() - msg_row.get()) * Columns.get() - 3 as ::core::ffi::c_int;
                    }
                    if !(*edit_submode_extra.ptr()).is_null() {
                        length -= vim_strsize(edit_submode_extra.get());
                    }
                    if length > 0 as ::core::ffi::c_int {
                        if !(*edit_submode_pre.ptr()).is_null() {
                            length -= vim_strsize(edit_submode_pre.get());
                        }
                        if length - vim_strsize(edit_submode.get()) > 0 as ::core::ffi::c_int {
                            if !(*edit_submode_pre.ptr()).is_null() {
                                msg_puts_hl(edit_submode_pre.get(), hl_id, false_0 != 0);
                            }
                            msg_puts_hl(edit_submode.get(), hl_id, false_0 != 0);
                        }
                        if !(*edit_submode_extra.ptr()).is_null() {
                            msg_puts_hl(
                                b" \0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                            let mut sub_id: ::core::ffi::c_int = if (edit_submode_highl.get()
                                as ::core::ffi::c_uint)
                                < HLF_COUNT as ::core::ffi::c_uint
                            {
                                edit_submode_highl.get() as ::core::ffi::c_int
                            } else {
                                hl_id
                            };
                            msg_puts_hl(edit_submode_extra.get(), sub_id, false_0 != 0);
                        }
                    }
                } else {
                    if State.get() & MODE_TERMINAL != 0 {
                        msg_puts_hl(
                            gettext(b" TERMINAL\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if State.get() & VREPLACE_FLAG != 0 {
                        msg_puts_hl(
                            gettext(b" VREPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if State.get() & REPLACE_FLAG != 0 {
                        msg_puts_hl(
                            gettext(b" REPLACE\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if State.get() & MODE_INSERT != 0 {
                        if p_ri.get() != 0 {
                            msg_puts_hl(
                                gettext(b" REVERSE\0".as_ptr() as *const ::core::ffi::c_char),
                                hl_id,
                                false_0 != 0,
                            );
                        }
                        msg_puts_hl(
                            gettext(b" INSERT\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if restart_edit.get() == 'I' as ::core::ffi::c_int
                        || restart_edit.get() == 'i' as ::core::ffi::c_int
                        || restart_edit.get() == 'a' as ::core::ffi::c_int
                        || restart_edit.get() == 'A' as ::core::ffi::c_int
                    {
                        if !(*curbuf.get()).terminal.is_null() {
                            msg_puts_hl(
                                gettext(b" (terminal)\0".as_ptr() as *const ::core::ffi::c_char),
                                hl_id,
                                false_0 != 0,
                            );
                        } else {
                            msg_puts_hl(
                                gettext(b" (insert)\0".as_ptr() as *const ::core::ffi::c_char),
                                hl_id,
                                false_0 != 0,
                            );
                        }
                    } else if restart_edit.get() == 'R' as ::core::ffi::c_int {
                        msg_puts_hl(
                            gettext(b" (replace)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    } else if restart_edit.get() == 'V' as ::core::ffi::c_int {
                        msg_puts_hl(
                            gettext(b" (vreplace)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                    if State.get() & MODE_LANGMAP != 0 {
                        if (*curwin.get()).w_onebuf_opt.wo_arab != 0 {
                            msg_puts_hl(
                                gettext(b" Arabic\0".as_ptr() as *const ::core::ffi::c_char),
                                hl_id,
                                false_0 != 0,
                            );
                        } else if let Some(keymap_name) = keymap_str(curwin.get()) {
                            let buf = NameBuff.ptr() as *mut ::core::ffi::c_char;
                            let plen = vim_snprintf(
                                buf,
                                MAXPATHL as size_t,
                                b" (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                                keymap_name.as_ptr(),
                            );
                            if plen > 0 && plen <= MAXPATHL - 1 {
                                msg_puts_hl(buf, hl_id, false_0 != 0);
                            }
                        }
                    }
                    if State.get() & MODE_INSERT != 0 && p_paste.get() != 0 {
                        msg_puts_hl(
                            gettext(b" (paste)\0".as_ptr() as *const ::core::ffi::c_char),
                            hl_id,
                            false_0 != 0,
                        );
                    }
                    if VIsual_active.get() {
                        let mut p: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        match (if VIsual_select.get() as ::core::ffi::c_int != 0 {
                            4 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) + (VIsual_mode.get() == Ctrl_V) as ::core::ffi::c_int
                            * 2 as ::core::ffi::c_int
                            + (VIsual_mode.get() == 'V' as ::core::ffi::c_int) as ::core::ffi::c_int
                        {
                            0 => {
                                p = b" VISUAL\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                            1 => {
                                p = b" VISUAL LINE\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                            2 => {
                                p = b" VISUAL BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                            4 => {
                                p = b" SELECT\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                            5 => {
                                p = b" SELECT LINE\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                            _ => {
                                p = b" SELECT BLOCK\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                            }
                        }
                        msg_puts_hl(gettext(p), hl_id, false_0 != 0);
                    }
                    msg_puts_hl(
                        b" --\0".as_ptr() as *const ::core::ffi::c_char,
                        hl_id,
                        false_0 != 0,
                    );
                }
                need_clear = true_0 != 0;
            }
            if reg_recording.get() != 0 as ::core::ffi::c_int && (*edit_submode.ptr()).is_null() {
                recording_mode(hl_id);
                need_clear = true_0 != 0;
            }
            mode_displayed.set(true_0 != 0);
            if need_clear as ::core::ffi::c_int != 0
                || clear_cmdline.get() as ::core::ffi::c_int != 0
                || redraw_mode.get() as ::core::ffi::c_int != 0
            {
                msg_clr_eos();
            }
            msg_didout.set(false_0 != 0);
            length = msg_col.get();
            msg_col.set(0 as ::core::ffi::c_int);
            msg_no_more.set(false_0 != 0);
            lines_left.set(save_lines_left);
            need_wait_return.set(nwr_save);
        } else if clear_cmdline.get() as ::core::ffi::c_int != 0
            && msg_silent.get() == 0 as ::core::ffi::c_int
        {
            msg_clr_cmdline();
        } else if redraw_mode.get() {
            msg_pos_mode();
            msg_clr_eos();
        }
        msg_ext_flush_showmode();
        if VIsual_active.get() {
            clear_showcmd();
        }
        redraw_ruler();
        redraw_cmdline.set(false_0 != 0);
        redraw_mode.set(false_0 != 0);
        clear_cmdline.set(false_0 != 0);
        return length;
    }
}

pub(crate) unsafe extern "C" fn msg_pos_mode() {
    msg_col.set(0 as ::core::ffi::c_int);
    msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
}

pub unsafe extern "C" fn unshowmode(mut force: bool) {
    unsafe {
        if !redrawing() || !force && char_avail() as ::core::ffi::c_int != 0 && !KeyTyped.get() {
            redraw_cmdline.set(true_0 != 0);
        } else {
            clearmode();
        };
    }
}

pub unsafe extern "C" fn clearmode() {
    unsafe {
        let save_msg_row: ::core::ffi::c_int = msg_row.get();
        let save_msg_col: ::core::ffi::c_int = msg_col.get();
        msg_ext_ui_flush();
        msg_pos_mode();
        if reg_recording.get() != 0 as ::core::ffi::c_int {
            recording_mode(HLF_CM);
        }
        msg_clr_eos();
        msg_ext_flush_showmode();
        msg_col.set(save_msg_col);
        msg_row.set(save_msg_row);
    }
}

pub(crate) unsafe extern "C" fn recording_mode(mut hl_id: ::core::ffi::c_int) {
    unsafe {
        if shortmess(SHM_RECORDING as ::core::ffi::c_int) {
            return;
        }
        msg_puts_hl(
            gettext(b"recording\0".as_ptr() as *const ::core::ffi::c_char),
            hl_id,
            false_0 != 0,
        );
        let mut s: [::core::ffi::c_char; 4] = [0; 4];
        snprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            b" @%c\0".as_ptr() as *const ::core::ffi::c_char,
            reg_recording.get(),
        );
        msg_puts_hl(&raw mut s as *mut ::core::ffi::c_char, hl_id, false_0 != 0);
    }
}

pub const COL_RULER: ::core::ffi::c_int = 17 as ::core::ffi::c_int;

pub unsafe extern "C" fn comp_col() {
    unsafe {
        let mut last_has_status: bool = last_stl_height(false_0 != 0) > 0 as ::core::ffi::c_int;
        sc_col.set(0 as ::core::ffi::c_int);
        ru_col.set(0 as ::core::ffi::c_int);
        if p_ru.get() != 0 {
            ru_col.set(
                (if ru_wid.get() != 0 {
                    ru_wid.get()
                } else {
                    COL_RULER
                }) + 1 as ::core::ffi::c_int,
            );
            if !last_has_status {
                sc_col.set(ru_col.get());
            }
        }
        if p_sc.get() != 0 && *p_sloc.get() as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
            (*sc_col.ptr()) += SHOWCMD_COLS as ::core::ffi::c_int;
            if p_ru.get() == 0 || last_has_status as ::core::ffi::c_int != 0 {
                (*sc_col.ptr()) += 1;
            }
        }
        '_c2rust_label: {
            if sc_col.get() >= 0 as ::core::ffi::c_int
                && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + sc_col.get()
                    <= Columns.get()
            {
            } else {
                __assert_fail(
                    b"sc_col >= 0 && INT_MIN + sc_col <= Columns\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1128 as ::core::ffi::c_uint,
                    b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        sc_col.set(Columns.get() - sc_col.get());
        '_c2rust_label_0: {
            if ru_col.get() >= 0 as ::core::ffi::c_int
                && -2147483647 as ::core::ffi::c_int - 1 as ::core::ffi::c_int + ru_col.get()
                    <= Columns.get()
            {
            } else {
                __assert_fail(
                    b"ru_col >= 0 && INT_MIN + ru_col <= Columns\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1131 as ::core::ffi::c_uint,
                    b"void comp_col(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        ru_col.set(Columns.get() - ru_col.get());
        if sc_col.get() <= 0 as ::core::ffi::c_int {
            sc_col.set(1 as ::core::ffi::c_int);
        }
        if ru_col.get() <= 0 as ::core::ffi::c_int {
            ru_col.set(1 as ::core::ffi::c_int);
        }
        set_vim_var_nr(
            VV_ECHOSPACE,
            (sc_col.get() - 1 as ::core::ffi::c_int) as varnumber_T,
        );
    }
}
