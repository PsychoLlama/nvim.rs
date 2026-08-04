//! The command-line window (`q:`, `q/`, `'cedit'`).
//!
//! [`open_cmdwin`] opens a real buffer holding the history, runs a nested
//! `main_loop` over it, and turns whatever line the cursor was on into the
//! command line's answer.  The `*_locked` guards are here because they are
//! what stops that window being opened from somewhere it cannot unwind.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn text_locked() -> bool {
    unsafe {
        if cmdwin_type.get() != 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
        if expr_map_locked() {
            return true_0 != 0;
        }
        return textlock.get() != 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn text_locked_msg() {
    unsafe {
        emsg(gettext(get_text_locked_msg()));
    }
}

pub unsafe extern "C" fn get_text_locked_msg() -> *const ::core::ffi::c_char {
    if cmdwin_type.get() != 0 as ::core::ffi::c_int {
        return &raw const e_cmdwin as *const ::core::ffi::c_char;
    } else {
        return &raw const e_textlock as *const ::core::ffi::c_char;
    };
}

pub unsafe extern "C" fn text_or_buf_locked() -> bool {
    unsafe {
        if text_locked() {
            text_locked_msg();
            return true_0 != 0;
        }
        return curbuf_locked();
    }
}

pub unsafe extern "C" fn curbuf_locked() -> bool {
    unsafe {
        if (*curbuf.get()).b_ro_locked > 0 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_cannot_edit_other_buf as *const ::core::ffi::c_char,
            ));
            return true_0 != 0;
        }
        return allbuf_locked();
    }
}

pub unsafe extern "C" fn allbuf_locked() -> bool {
    unsafe {
        if allbuf_lock.get() > 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E811: Not allowed to change buffer information now\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn cmdline_init() {
    unsafe {
        memset(
            ccline.ptr() as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<CmdlineInfo>(),
        );
    }
}

pub unsafe extern "C" fn did_set_cedit(mut _args: *mut optset_T) -> *const ::core::ffi::c_char {
    unsafe {
        if *p_cedit.get() as ::core::ffi::c_int == NUL {
            cedit_key.set(-1 as ::core::ffi::c_int);
        } else {
            let mut n: ::core::ffi::c_int = string_to_key(p_cedit.get());
            if n == 0 as ::core::ffi::c_int || vim_isprintc(n) as ::core::ffi::c_int != 0 {
                return &raw const e_invarg as *const ::core::ffi::c_char;
            }
            cedit_key.set(n);
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn open_cmdwin() -> ::core::ffi::c_int {
    unsafe {
        let mut old_curbuf: bufref_T = bufref_T::default();
        let mut bufref: bufref_T = bufref_T::default();
        let mut old_curwin: *mut win_T = curwin.get();
        let mut i: ::core::ffi::c_int = 0;
        let mut winsizes: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut save_restart_edit: ::core::ffi::c_int = restart_edit.get();
        let mut save_State: ::core::ffi::c_int = State.get();
        let mut save_exmode: bool = exmode_active.get();
        let mut save_cmdmsg_rl: bool = cmdmsg_rl.get();
        if text_or_buf_locked() as ::core::ffi::c_int != 0
            || cmdwin_type.get() != 0 as ::core::ffi::c_int
            || cmdline_star.get() > 0 as ::core::ffi::c_int
        {
            beep_flush();
            return -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        set_bufref(&raw mut old_curbuf, curbuf.get());
        win_size_save(&raw mut winsizes);
        pum_undisplay(true_0 != 0);
        (*cmdmod.ptr()).cmod_tab = 0 as ::core::ffi::c_int;
        (*cmdmod.ptr()).cmod_flags |= CMOD_NOSWAPFILE;
        if win_split(
            p_cwh.get() as ::core::ffi::c_int,
            WSP_BOT as ::core::ffi::c_int,
        ) == FAIL
        {
            beep_flush();
            ga_clear(&raw mut winsizes);
            return -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        if !win_valid(old_curwin)
            || curwin.get() == old_curwin
            || !bufref_valid(&raw mut old_curbuf)
            || (*old_curwin).w_buffer != old_curbuf.br_buf
        {
            beep_flush();
            ga_clear(&raw mut winsizes);
            return Ctrl_C;
        }
        got_int.set(false_0 != 0);
        cmdwin_type.set(get_cmdline_type());
        cmdwin_level.set((*ccline.ptr()).level);
        cmdwin_win.set(curwin.get());
        cmdwin_old_curwin.set(old_curwin);
        let newbuf_status: ::core::ffi::c_int = buf_open_scratch(
            0 as handle_T,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        let cmdwin_valid: bool = win_valid(cmdwin_win.get());
        if newbuf_status == FAIL
            || !cmdwin_valid
            || curwin.get() != cmdwin_win.get()
            || !win_valid(old_curwin)
            || !bufref_valid(&raw mut old_curbuf)
            || (*old_curwin).w_buffer != old_curbuf.br_buf
        {
            if newbuf_status == OK {
                set_bufref(&raw mut bufref, curbuf.get());
            }
            if cmdwin_valid as ::core::ffi::c_int != 0 && !last_window(cmdwin_win.get()) {
                win_close(cmdwin_win.get(), true_0 != 0, false_0 != 0);
            }
            if newbuf_status == OK
                && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && bufref.br_buf != curbuf.get()
            {
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    bufref.br_buf,
                    DOBUF_WIPE as ::core::ffi::c_int,
                    false_0 != 0,
                    false_0 != 0,
                );
            }
            cmdwin_type.set(0 as ::core::ffi::c_int);
            cmdwin_level.set(0 as ::core::ffi::c_int);
            cmdwin_win.set(::core::ptr::null_mut::<win_T>());
            cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());
            beep_flush();
            ga_clear(&raw mut winsizes);
            return Ctrl_C;
        }
        cmdwin_buf.set(curbuf.get());
        set_option_value_give_err(
            kOptBufhidden,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"wipe\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    },
                },
            },
            OPT_LOCAL,
        );
        (*curbuf.get()).b_p_ma = true_0;
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;
        (*curwin.get()).w_onebuf_opt.wo_rl = cmdmsg_rl.get() as ::core::ffi::c_int;
        cmdmsg_rl.set(false_0 != 0);
        (*curbuf.get()).b_ro_locked += 1;
        need_wait_return.set(false_0 != 0);
        let histtype: ::core::ffi::c_int = hist_char2type(cmdwin_type.get()) as ::core::ffi::c_int;
        if histtype == HIST_CMD as ::core::ffi::c_int
            || histtype == HIST_DEBUG as ::core::ffi::c_int
        {
            if p_wc.get() == TAB as OptInt {
                add_map(
                    b"<Tab>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"<C-X><C-V>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    MODE_INSERT,
                    true_0 != 0,
                );
                add_map(
                    b"<Tab>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    b"a<C-X><C-V>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    MODE_NORMAL,
                    true_0 != 0,
                );
            }
            set_option_value_give_err(
                kOptFiletype,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: String_0 {
                            data: b"vim\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                },
                OPT_LOCAL,
            );
        }
        (*curbuf.get()).b_ro_locked -= 1;
        (*curbuf.get()).b_p_tw = 0 as OptInt;
        init_history();
        if get_hislen() > 0 as ::core::ffi::c_int && histtype != HIST_INVALID as ::core::ffi::c_int
        {
            i = get_hisidx(histtype);
            if i >= 0 as ::core::ffi::c_int {
                let mut lnum: linenr_T = 0 as linenr_T;
                loop {
                    i += 1;
                    if i == get_hislen() {
                        i = 0 as ::core::ffi::c_int;
                    }
                    if let Some(entry) = hist_entry_ref(histtype, i) {
                        let c2rust_fresh31 = lnum;
                        lnum = lnum + 1;
                        ml_append(
                            c2rust_fresh31,
                            entry.text as *mut ::core::ffi::c_char,
                            0 as colnr_T,
                            false_0 != 0,
                        );
                    }
                    if i == get_hisidx(histtype) {
                        break;
                    }
                }
            }
        }
        ml_replace(
            (*curbuf.get()).b_ml.ml_line_count,
            (*ccline.ptr()).cmdbuff,
            true_0 != 0,
        );
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*curwin.get()).w_cursor.col = (*ccline.ptr()).cmdpos as colnr_T;
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
        ui_ext_cmdline_hide(false_0 != 0);
        redraw_later(curwin.get(), UPD_SOME_VALID);
        exmode_active.set(false_0 != 0);
        State.set(MODE_NORMAL);
        setmouse();
        clear_showcmd();
        cmdwin_result.set(0 as ::core::ffi::c_int);
        trigger_cmd_autocmd(cmdwin_type.get(), EVENT_CMDWINENTER);
        if restart_edit.get() != 0 as ::core::ffi::c_int {
            stuffcharReadbuff(
                -(253 as ::core::ffi::c_int
                    + ((KE_NOP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)),
            );
        }
        i = RedrawingDisabled.get();
        RedrawingDisabled.set(0 as ::core::ffi::c_int);
        let mut save_count: ::core::ffi::c_int = crate::src::nvim::clipboard::save_batch_count();
        normal_enter(true_0 != 0, false_0 != 0);
        RedrawingDisabled.set(i);
        crate::src::nvim::clipboard::restore_batch_count(save_count);
        let save_KeyTyped: bool = KeyTyped.get();
        trigger_cmd_autocmd(cmdwin_type.get(), EVENT_CMDWINLEAVE);
        KeyTyped.set(save_KeyTyped);
        cmdwin_type.set(0 as ::core::ffi::c_int);
        cmdwin_level.set(0 as ::core::ffi::c_int);
        cmdwin_buf.set(::core::ptr::null_mut::<buf_T>());
        cmdwin_win.set(::core::ptr::null_mut::<win_T>());
        cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());
        exmode_active.set(save_exmode);
        if !win_valid(old_curwin)
            || !bufref_valid(&raw mut old_curbuf)
            || (*old_curwin).w_buffer != old_curbuf.br_buf
        {
            cmdwin_result.set(Ctrl_C);
            emsg(gettext(
                (e_active_window_or_buffer_changed_or_deleted.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
        } else {
            let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
            if aborting() as ::core::ffi::c_int != 0
                && cmdwin_result.get()
                    != -(253 as ::core::ffi::c_int
                        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                cmdwin_result.set(Ctrl_C);
            }
            dealloc_cmdbuff();
            if cmdwin_result.get()
                == -(253 as ::core::ffi::c_int
                    + ((KE_XF1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || cmdwin_result.get()
                    == -(253 as ::core::ffi::c_int
                        + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                let mut p: *const ::core::ffi::c_char = if cmdwin_result.get()
                    == -(253 as ::core::ffi::c_int
                        + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    b"qa\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"qa!\0".as_ptr() as *const ::core::ffi::c_char
                };
                let mut plen: size_t = (if cmdwin_result.get()
                    == -(253 as ::core::ffi::c_int
                        + ((KE_XF2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    2 as ::core::ffi::c_int
                } else {
                    3 as ::core::ffi::c_int
                }) as size_t;
                if histtype == HIST_CMD as ::core::ffi::c_int {
                    (*ccline.ptr()).cmdbuff =
                        xmemdupz(p as *const ::core::ffi::c_void, plen) as *mut ::core::ffi::c_char;
                    (*ccline.ptr()).cmdlen = plen as ::core::ffi::c_int;
                    (*ccline.ptr()).cmdbufflen =
                        plen as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
                    cmdwin_result.set(CAR);
                } else {
                    stuffcharReadbuff(':' as ::core::ffi::c_int);
                    stuffReadbuff(p);
                    stuffcharReadbuff(CAR);
                }
            } else if cmdwin_result.get() == Ctrl_C {
                (*ccline.ptr()).cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                (*ccline.ptr()).cmdlen = get_cursor_line_len() as ::core::ffi::c_int;
                (*ccline.ptr()).cmdbufflen = (*ccline.ptr()).cmdlen + 1 as ::core::ffi::c_int;
                (*ccline.ptr()).cmdbuff =
                    xstrnsave(get_cursor_line_ptr(), (*ccline.ptr()).cmdlen as size_t);
            }
            if (*ccline.ptr()).cmdbuff.is_null() {
                (*ccline.ptr()).cmdbuff = xmemdupz(
                    b"\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    0 as size_t,
                ) as *mut ::core::ffi::c_char;
                (*ccline.ptr()).cmdlen = 0 as ::core::ffi::c_int;
                (*ccline.ptr()).cmdbufflen = 1 as ::core::ffi::c_int;
                (*ccline.ptr()).cmdpos = 0 as ::core::ffi::c_int;
                cmdwin_result.set(Ctrl_C);
            } else {
                (*ccline.ptr()).cmdpos = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                if (*ccline.ptr()).cmdpos == (*ccline.ptr()).cmdlen - 1 as ::core::ffi::c_int
                    || (*ccline.ptr()).cmdpos > (*ccline.ptr()).cmdlen
                {
                    (*ccline.ptr()).cmdpos = (*ccline.ptr()).cmdlen;
                }
                if cmdwin_result.get()
                    == -(253 as ::core::ffi::c_int
                        + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
                    redrawcmd();
                }
            }
            (*curwin.get()).w_onebuf_opt.wo_cole = 0 as OptInt;
            wp = curwin.get();
            set_bufref(&raw mut bufref, curbuf.get());
            skip_win_fix_cursor.set(true_0 != 0);
            win_goto(old_curwin);
            if win_valid(wp) as ::core::ffi::c_int != 0 && wp != curwin.get() {
                win_close(wp, true_0 != 0, false_0 != 0);
            }
            if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && bufref.br_buf != curbuf.get()
            {
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    bufref.br_buf,
                    DOBUF_WIPE as ::core::ffi::c_int,
                    false_0 != 0,
                    false_0 != 0,
                );
            }
            win_size_restore(&raw mut winsizes);
            skip_win_fix_cursor.set(false_0 != 0);
        }
        ga_clear(&raw mut winsizes);
        restart_edit.set(save_restart_edit);
        cmdmsg_rl.set(save_cmdmsg_rl);
        State.set(save_State);
        may_trigger_modechanged();
        setmouse();
        setcursor();
        return cmdwin_result.get();
    }
}

pub unsafe extern "C" fn is_in_cmdwin() -> bool {
    unsafe {
        return cmdwin_type.get() != 0 as ::core::ffi::c_int && get_cmdline_type() == NUL;
    }
}
