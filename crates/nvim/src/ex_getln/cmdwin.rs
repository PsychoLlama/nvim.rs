//! The command-line window (`q:`, `q/`, `'cedit'`).
//!
//! [`open_cmdwin`] opens a real buffer holding the history, runs a nested
//! `main_loop` over it, and turns whatever line the cursor was on into the
//! command line's answer.  The `*_locked` guards are here because they are
//! what stops that window being opened from somewhere it cannot unwind.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Allow;
use crate::keycodes::Ctrl_C;
use crate::types::{CmdModFlags, FAIL, NUL, OK, OptionSetFlags};

/// True when the text must not be changed and we cannot switch to another
/// window or buffer — editing the command line, and the like.
pub unsafe fn text_locked() -> bool {
    unsafe {
        if cmdwin_type.get() != 0 {
            return true;
        }
        if expr_map_locked() {
            return true;
        }
        textlock.get() != 0
    }
}

/// Report a command that is not allowed while the cmdline window is open or
/// the command line is being edited another way.
pub unsafe fn text_locked_msg() {
    unsafe {
        emsg(gettext(get_text_locked_msg()));
    }
}

/// The message [`text_locked_msg`] gives: which of the two locks is on.
pub fn get_text_locked_msg() -> *const ::core::ffi::c_char {
    if cmdwin_type.get() != 0 {
        e_cmdwin.as_ptr()
    } else {
        e_textlock.as_ptr()
    }
}

/// Check for text, window or buffer locked; report and answer true if it is.
pub unsafe fn text_or_buf_locked() -> bool {
    unsafe {
        if text_locked() {
            text_locked_msg();
            return true;
        }
        curbuf_locked()
    }
}

/// Check `curbuf->b_ro_locked` and `allbuf_lock`; report and answer true if
/// either is set.
pub unsafe fn curbuf_locked() -> bool {
    unsafe {
        if (*curbuf.get()).b_ro_locked > 0 {
            emsg(gettext(e_cannot_edit_other_buf.as_ptr()));
            return true;
        }
        allbuf_locked()
    }
}

/// Check `allbuf_lock`; report and answer true if it is set.
pub unsafe fn allbuf_locked() -> bool {
    unsafe {
        if allbuf_lock.get() > 0 {
            emsg(gettext(
                c"E811: Not allowed to change buffer information now".as_ptr(),
            ));
            return true;
        }
        false
    }
}

/// Zero the command-line state at startup.
pub fn cmdline_init() {
    ccline.set(CMDLINE_INFO_INIT);
}

/// `'cedit'` changed: re-derive the key that opens the command-line window.
pub unsafe fn did_set_cedit(_args: *mut optset_T) -> *const ::core::ffi::c_char {
    unsafe {
        if *p_cedit.get() as ::core::ffi::c_int == NUL {
            cedit_key.set(-1);
        } else {
            let n = string_to_key(p_cedit.get());
            if n == 0 || vim_isprintc(n) {
                return e_invarg.as_ptr();
            }
            cedit_key.set(n);
        }
        ::core::ptr::null::<::core::ffi::c_char>()
    }
}

/// Open a window on the current command line and its history, and edit in it.
///
/// Returns when the window is closed, with `CAR` if the command is to be
/// executed, `Ctrl_C` if it is to be abandoned, and `K_IGNORE` if editing
/// continues.
pub(crate) unsafe fn open_cmdwin() -> ::core::ffi::c_int {
    unsafe {
        let mut old_curbuf = bufref_T::default();
        let mut bufref = bufref_T::default();
        let old_curwin = curwin.get();
        // Uninitialised in the C; `win_size_save` below fills it, and every
        // path that reaches `ga_clear` has been through it.
        let mut winsizes = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let save_restart_edit = restart_edit.get();
        let save_State = State.get();
        let save_exmode = exmode_active.get();
        let save_cmdmsg_rl = cmdmsg_rl.get();

        // Can't do this when text or buffer is locked, can't do it
        // recursively, and can't do it when typing a password.
        if text_or_buf_locked() || cmdwin_type.get() != 0 || cmdline_star.get() > 0 {
            beep_flush();
            return K_IGNORE;
        }

        set_bufref(&raw mut old_curbuf, curbuf.get());

        // Save current window sizes.
        win_size_save(&raw mut winsizes);

        // When using completion in Insert mode with <C-R>=<C-F> one can open
        // the command line window, but we don't want the popup menu then.
        pum_undisplay(true);

        // Don't use a new tab page.
        (*cmdmod.ptr()).cmod_tab = 0;
        (*cmdmod.ptr()).cmod_flags |= CmdModFlags::NOSWAPFILE;

        // Create a window for the command-line buffer.
        if win_split(
            p_cwh.get() as ::core::ffi::c_int,
            WSP_BOT as ::core::ffi::c_int,
        ) == FAIL
        {
            beep_flush();
            ga_clear(&raw mut winsizes);
            return K_IGNORE;
        }
        // win_split() autocommands may have messed with the old window or
        // buffer. Treat it as abandoning this command line.
        if !win_valid(old_curwin)
            || curwin.get() == old_curwin
            || !bufref_valid(&raw mut old_curbuf)
            || (*old_curwin).w_buffer != old_curbuf.br_buf
        {
            beep_flush();
            ga_clear(&raw mut winsizes);
            return Ctrl_C;
        }
        // Don't let quitting the More prompt make this fail.
        got_int.set(false);

        // Set the "cmdwin_*" variables before any autocommand can mess
        // things up.
        cmdwin_type.set(get_cmdline_type());
        cmdwin_level.set((*ccline.ptr()).level);
        cmdwin_win.set(curwin.get());
        cmdwin_old_curwin.set(old_curwin);

        // Create the empty command-line buffer. Be especially cautious of
        // BufLeave autocommands from do_ecmd(): the cmdwin restrictions do
        // not apply to them.
        let newbuf_status = buf_open_scratch(0, ::core::ptr::null_mut::<::core::ffi::c_char>());
        let cmdwin_valid = win_valid(cmdwin_win.get());
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
            if cmdwin_valid && !last_window(cmdwin_win.get()) {
                win_close(cmdwin_win.get(), true, false);
            }
            // win_close() autocommands may have already deleted the buffer.
            if newbuf_status == OK && bufref_valid(&raw mut bufref) && bufref.br_buf != curbuf.get()
            {
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    bufref.br_buf,
                    DOBUF_WIPE as ::core::ffi::c_int,
                    false,
                    false,
                );
            }

            cmdwin_type.set(0);
            cmdwin_level.set(0);
            cmdwin_win.set(::core::ptr::null_mut::<win_T>());
            cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());
            beep_flush();
            ga_clear(&raw mut winsizes);
            return Ctrl_C;
        }
        cmdwin_buf.set(curbuf.get());

        // The command-line buffer has bufhidden=wipe, unlike a true
        // "scratch" buffer.
        set_option_value_give_err(kOptBufhidden, static_optval(c"wipe"), OptionSetFlags::LOCAL);
        (*curbuf.get()).b_p_ma = 1;
        (*curwin.get()).w_onebuf_opt.wo_fen = 0;
        (*curwin.get()).w_onebuf_opt.wo_rl = cmdmsg_rl.get() as ::core::ffi::c_int;
        cmdmsg_rl.set(false);

        // Don't allow switching to another buffer.
        (*curbuf.get()).b_ro_locked += 1;

        // Showing the prompt may have set need_wait_return; reset it.
        need_wait_return.set(false);

        let histtype = hist_char2type(cmdwin_type.get());
        if histtype == HIST_CMD || histtype == HIST_DEBUG {
            if p_wc.get() == TAB as OptInt {
                add_map(
                    c"<Tab>".as_ptr().cast_mut(),
                    c"<C-X><C-V>".as_ptr().cast_mut(),
                    MODE_INSERT,
                    true,
                );
                add_map(
                    c"<Tab>".as_ptr().cast_mut(),
                    c"a<C-X><C-V>".as_ptr().cast_mut(),
                    MODE_NORMAL,
                    true,
                );
            }
            set_option_value_give_err(kOptFiletype, static_optval(c"vim"), OptionSetFlags::LOCAL);
        }
        (*curbuf.get()).b_ro_locked -= 1;

        // Reset 'textwidth' after setting 'filetype' (the Vim filetype plugin
        // sets 'textwidth' to 78).
        (*curbuf.get()).b_p_tw = 0;

        // Fill the buffer with the history.
        init_history();
        if get_hislen() > 0 && histtype != HIST_INVALID {
            let mut i = get_hisidx(histtype);
            if i >= 0 {
                let mut lnum: linenr_T = 0;
                // C's do-while: `get_hisidx` is re-read at the test, because
                // `ml_append`'s autocommands can move it.
                loop {
                    i += 1;
                    if i == get_hislen() {
                        i = 0;
                    }
                    if let Some(entry) = hist_entry_ref(histtype, i) {
                        ml_append(lnum, entry.text as *mut ::core::ffi::c_char, 0, false);
                        lnum += 1;
                    }
                    if i == get_hisidx(histtype) {
                        break;
                    }
                }
            }
        }

        // Replace the empty last line with the current command line and put
        // the cursor there.
        ml_replace(
            (*curbuf.get()).b_ml.ml_line_count,
            (*ccline.ptr()).cmdbuff,
            true,
        );
        (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*curwin.get()).w_cursor.col = (*ccline.ptr()).cmdpos as colnr_T;
        changed_line_abv_curs();
        invalidate_botline_win(curwin.get());
        ui_ext_cmdline_hide(false);
        redraw_later(curwin.get(), UPD_SOME_VALID);

        // No Ex mode here.
        exmode_active.set(false);

        State.set(MODE_NORMAL);
        setmouse();
        clear_showcmd();

        // Reset here so a CmdwinEnter autocommand can set it.
        cmdwin_result.set(0);

        trigger_cmd_autocmd(cmdwin_type.get(), EVENT_CMDWINENTER);
        if restart_edit.get() != 0 {
            // An autocmd ran ":startinsert".
            stuff_readbuf_char(K_NOP);
        }

        let redraw = Allow::redraw();
        let save_count = crate::clipboard::save_batch_count();

        // Call the main loop until <CR> or CTRL-C is typed.
        normal_enter(true, false);

        drop(redraw);
        crate::clipboard::restore_batch_count(save_count);

        let save_KeyTyped = KeyTyped.get();
        trigger_cmd_autocmd(cmdwin_type.get(), EVENT_CMDWINLEAVE);
        // Restore KeyTyped in case an autocommand modified it.
        KeyTyped.set(save_KeyTyped);

        cmdwin_type.set(0);
        cmdwin_level.set(0);
        cmdwin_buf.set(::core::ptr::null_mut::<buf_T>());
        cmdwin_win.set(::core::ptr::null_mut::<win_T>());
        cmdwin_old_curwin.set(::core::ptr::null_mut::<win_T>());

        exmode_active.set(save_exmode);

        // Safety check: the old window or buffer was changed or deleted.
        // It is a bug when this happens.
        if !win_valid(old_curwin)
            || !bufref_valid(&raw mut old_curbuf)
            || (*old_curwin).w_buffer != old_curbuf.br_buf
        {
            cmdwin_result.set(Ctrl_C);
            emsg(gettext(
                e_active_window_or_buffer_changed_or_deleted.as_ptr(),
            ));
        } else {
            // Autocmds may abort script processing.
            if aborting() && cmdwin_result.get() != K_IGNORE {
                cmdwin_result.set(Ctrl_C);
            }
            // Set the new command line from the cmdline buffer.
            dealloc_cmdbuff();

            if cmdwin_result.get() == K_XF1 || cmdwin_result.get() == K_XF2 {
                // ":qa[!]" was typed.
                let (p, plen) = if cmdwin_result.get() == K_XF2 {
                    (c"qa", 2 as size_t)
                } else {
                    (c"qa!", 3 as size_t)
                };

                if histtype == HIST_CMD {
                    // Execute the command directly.
                    (*ccline.ptr()).cmdbuff =
                        xmemdupz(p.as_ptr() as *const ::core::ffi::c_void, plen)
                            as *mut ::core::ffi::c_char;
                    (*ccline.ptr()).cmdlen = plen as ::core::ffi::c_int;
                    (*ccline.ptr()).cmdbufflen = plen as ::core::ffi::c_int + 1;
                    cmdwin_result.set(CAR);
                } else {
                    // First need to cancel what we were doing.
                    stuff_readbuf_char(':' as ::core::ffi::c_int);
                    stuff_readbuf(p.as_ptr());
                    stuff_readbuf_char(CAR);
                }
            } else if cmdwin_result.get() == Ctrl_C {
                // ":q" or ":close": don't execute any command and don't
                // modify the cmdline window.
                (*ccline.ptr()).cmdbuff = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                (*ccline.ptr()).cmdlen = get_cursor_line_len() as ::core::ffi::c_int;
                (*ccline.ptr()).cmdbufflen = (*ccline.ptr()).cmdlen + 1;
                (*ccline.ptr()).cmdbuff =
                    xstrnsave(get_cursor_line_ptr(), (*ccline.ptr()).cmdlen as size_t);
            }

            if (*ccline.ptr()).cmdbuff.is_null() {
                (*ccline.ptr()).cmdbuff = xmemdupz(c"".as_ptr() as *const ::core::ffi::c_void, 0)
                    as *mut ::core::ffi::c_char;
                (*ccline.ptr()).cmdlen = 0;
                (*ccline.ptr()).cmdbufflen = 1;
                (*ccline.ptr()).cmdpos = 0;
                cmdwin_result.set(Ctrl_C);
            } else {
                (*ccline.ptr()).cmdpos = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                // If the cursor is on the last character, it probably should
                // be after it.
                if (*ccline.ptr()).cmdpos == (*ccline.ptr()).cmdlen - 1
                    || (*ccline.ptr()).cmdpos > (*ccline.ptr()).cmdlen
                {
                    (*ccline.ptr()).cmdpos = (*ccline.ptr()).cmdlen;
                }
                if cmdwin_result.get() == K_IGNORE {
                    (*ccline.ptr()).cmdspos = cmd_screencol((*ccline.ptr()).cmdpos);
                    redrawcmd();
                }
            }

            // Avoid the command-line window's first character being
            // concealed.
            (*curwin.get()).w_onebuf_opt.wo_cole = 0;
            // First go back to the original window.
            let wp = curwin.get();
            set_bufref(&raw mut bufref, curbuf.get());
            skip_win_fix_cursor.set(true);
            win_goto(old_curwin);

            // win_goto() may trigger an autocommand that already closes the
            // cmdline window.
            if win_valid(wp) && wp != curwin.get() {
                win_close(wp, true, false);
            }

            // win_close() may have already wiped the buffer when 'bh' is set
            // to 'wipe'; autocommands may have closed other windows.
            if bufref_valid(&raw mut bufref) && bufref.br_buf != curbuf.get() {
                close_buffer(
                    ::core::ptr::null_mut::<win_T>(),
                    bufref.br_buf,
                    DOBUF_WIPE as ::core::ffi::c_int,
                    false,
                    false,
                );
            }

            // Restore window sizes.
            win_size_restore(&raw mut winsizes);
            skip_win_fix_cursor.set(false);
        }

        ga_clear(&raw mut winsizes);
        restart_edit.set(save_restart_edit);
        cmdmsg_rl.set(save_cmdmsg_rl);

        State.set(save_State);
        may_trigger_modechanged();
        setmouse();
        setcursor();

        cmdwin_result.get()
    }
}

/// True when in the cmdwin, and not editing the command line.
pub unsafe fn is_in_cmdwin() -> bool {
    unsafe { cmdwin_type.get() != 0 && get_cmdline_type() == NUL }
}
