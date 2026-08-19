//! `'inccommand'`: running the command being typed against a preview buffer.
//!
//! [`cmdpreview_may_show`] is the entry point — it saves everything the
//! command could change, executes it with `cmdpreview` set, shows the result
//! either in place or in a split preview window, and restores.  The
//! `cmdpreview_save_*` / `cmdpreview_restore_*` pairs are that save-restore.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{CmdModFlags, ExArgt, FAIL, OptionSetFlags, kErrorTypeNone};

/// The buffer `'inccommand'` previews into, or 0 when there is none yet.
pub fn cmdpreview_get_bufnr() -> handle_T {
    cmdpreview_bufnr.get()
}

/// The namespace the preview's extmarks and highlights live in.
pub fn cmdpreview_get_ns() -> ::core::ffi::c_int {
    cmdpreview_ns.get()
}

/// Set up the command preview buffer, creating it if it does not exist.
///
/// Answers NULL if the buffer could not be made ready.
pub(crate) unsafe fn cmdpreview_open_buf() -> *mut buf_T {
    unsafe {
        let mut cmdpreview_buf = if cmdpreview_bufnr.get() != 0 {
            buflist_findnr(cmdpreview_bufnr.get())
        } else {
            ::core::ptr::null_mut::<buf_T>()
        };

        // If the preview buffer doesn't exist, open one.
        if cmdpreview_buf.is_null() {
            let Ok(bufnr) = nvim_create_buf(false, true) else {
                return ::core::ptr::null_mut::<buf_T>();
            };
            cmdpreview_buf = buflist_findnr(bufnr);
        }

        // The preview buffer cannot preview itself.
        if cmdpreview_buf == curbuf.get() {
            return ::core::ptr::null_mut::<buf_T>();
        }

        // Rename the preview buffer.
        let mut aco = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
        let retv = rename_buffer(c"[Preview]".as_ptr().cast_mut());
        aucmd_restbuf(&raw mut aco);

        if retv == FAIL {
            return ::core::ptr::null_mut::<buf_T>();
        }

        // Temporarily switch to the preview buffer to set it up.
        aucmd_prepbuf(&raw mut aco, cmdpreview_buf);
        buf_clear();
        (*curbuf.get()).b_p_ma = true_0;
        (*curbuf.get()).b_p_ul = -1;
        // Reset 'textwidth', which a ftplugin may have set.
        (*curbuf.get()).b_p_tw = 0;
        aucmd_restbuf(&raw mut aco);
        cmdpreview_bufnr.set((*cmdpreview_buf).handle);

        cmdpreview_buf
    }
}

/// Open the command preview window, if it is not already open, and return to
/// the original window.  Answers NULL if it could not be opened.
pub(crate) unsafe fn cmdpreview_open_win(cmdpreview_buf: *mut buf_T) -> *mut win_T {
    unsafe {
        let save_curwin = curwin.get();

        if win_split(
            p_cwh.get() as ::core::ffi::c_int,
            WSP_BOT as ::core::ffi::c_int,
        ) == FAIL
        {
            return ::core::ptr::null_mut::<win_T>();
        }

        let preview_win = curwin.get();
        let mut err: Error = ERROR_INIT;

        // Switch to the preview buffer. C's TRY_WRAP.
        let mut tstate: TryState = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        let result = do_buffer(
            DOBUF_GOTO as ::core::ffi::c_int,
            DOBUF_FIRST as ::core::ffi::c_int,
            FORWARD as ::core::ffi::c_int,
            (*cmdpreview_buf).handle,
            0,
        );
        try_leave(&raw mut tstate, &raw mut err);

        if err.type_0 != kErrorTypeNone || result == FAIL {
            api_clear_error(&raw mut err);
            return ::core::ptr::null_mut::<win_T>();
        }

        (*curwin.get()).w_onebuf_opt.wo_cul = false_0;
        (*curwin.get()).w_onebuf_opt.wo_cuc = false_0;
        (*curwin.get()).w_onebuf_opt.wo_spell = false_0;
        (*curwin.get()).w_onebuf_opt.wo_fen = false_0;

        win_enter(save_curwin, false);
        preview_win
    }
}

/// Close any open command preview windows.
pub(crate) unsafe fn cmdpreview_close_win() {
    unsafe {
        let buf = if cmdpreview_bufnr.get() != 0 {
            buflist_findnr(cmdpreview_bufnr.get())
        } else {
            ::core::ptr::null_mut::<buf_T>()
        };
        if !buf.is_null() {
            close_windows(buf, false);
        }
    }
}

/// Save `buf`'s whole undo state, so the preview's edits can be taken back.
pub(crate) unsafe fn cmdpreview_save_undo(cp_undoinfo: *mut CpUndoInfo, buf: *mut buf_T) {
    unsafe {
        (*cp_undoinfo).save_b_u_synced = (*buf).b_u_synced;
        (*cp_undoinfo).save_b_u_oldhead = (*buf).b_u_oldhead;
        (*cp_undoinfo).save_b_u_newhead = (*buf).b_u_newhead;
        (*cp_undoinfo).save_b_u_curhead = (*buf).b_u_curhead;
        (*cp_undoinfo).save_b_u_numhead = (*buf).b_u_numhead;
        (*cp_undoinfo).save_b_u_seq_last = (*buf).b_u_seq_last;
        (*cp_undoinfo).save_b_u_save_nr_last = (*buf).b_u_save_nr_last;
        (*cp_undoinfo).save_b_u_seq_cur = (*buf).b_u_seq_cur;
        (*cp_undoinfo).save_b_u_time_cur = (*buf).b_u_time_cur;
        (*cp_undoinfo).save_b_u_save_nr_cur = (*buf).b_u_save_nr_cur;
        (*cp_undoinfo).save_b_u_line_ptr = (*buf).b_u_line_ptr;
        (*cp_undoinfo).save_b_u_line_lnum = (*buf).b_u_line_lnum;
        (*cp_undoinfo).save_b_u_line_colnr = (*buf).b_u_line_colnr;
    }
}

/// Put back what [`cmdpreview_save_undo`] saved.
///
/// `b_u_synced` is only restored when the undo tree is not mid-undo, which is
/// upstream's guard against re-synchronising a half-applied change.
pub(crate) unsafe fn cmdpreview_restore_undo(cp_undoinfo: *const CpUndoInfo, buf: *mut buf_T) {
    unsafe {
        (*buf).b_u_oldhead = (*cp_undoinfo).save_b_u_oldhead;
        (*buf).b_u_newhead = (*cp_undoinfo).save_b_u_newhead;
        (*buf).b_u_curhead = (*cp_undoinfo).save_b_u_curhead;
        (*buf).b_u_numhead = (*cp_undoinfo).save_b_u_numhead;
        (*buf).b_u_seq_last = (*cp_undoinfo).save_b_u_seq_last;
        (*buf).b_u_save_nr_last = (*cp_undoinfo).save_b_u_save_nr_last;
        (*buf).b_u_seq_cur = (*cp_undoinfo).save_b_u_seq_cur;
        (*buf).b_u_time_cur = (*cp_undoinfo).save_b_u_time_cur;
        (*buf).b_u_save_nr_cur = (*cp_undoinfo).save_b_u_save_nr_cur;
        (*buf).b_u_line_ptr = (*cp_undoinfo).save_b_u_line_ptr;
        (*buf).b_u_line_lnum = (*cp_undoinfo).save_b_u_line_lnum;
        (*buf).b_u_line_colnr = (*cp_undoinfo).save_b_u_line_colnr;
        if (*buf).b_u_curhead.is_null() {
            (*buf).b_u_synced = (*cp_undoinfo).save_b_u_synced;
        }
    }
}

/// Save the state of every window and buffer in the current tab page, and put
/// the options the preview must not be disturbed by out of the way.
pub(crate) unsafe fn cmdpreview_prepare(cpinfo: *mut CpInfo) {
    unsafe {
        // C's `kv_push` onto one of `CpInfo`'s two kvecs. A macro rather than
        // a function because the two differ only in element type, and the
        // kvecs are c2rust's anonymous structs with no name to be generic
        // over.
        macro_rules! kv_push {
            ($vec:expr, $value:expr) => {{
                let value = $value;
                let v = &raw mut $vec;
                if (*v).size == (*v).capacity {
                    (*v).capacity = if (*v).capacity != 0 {
                        (*v).capacity << 1
                    } else {
                        8
                    };
                    (*v).items = xrealloc(
                        (*v).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of_val(&value) * (*v).capacity,
                    ) as *mut _;
                }
                *(*v).items.add((*v).size) = value;
                (*v).size += 1;
            }};
        }

        let mut saved_bufs: Set_ptr_t = SET_INIT;

        (*cpinfo).buf_info = CP_INFO_INIT.buf_info;
        (*cpinfo).win_info = CP_INFO_INIT.win_info;

        // C's FOR_ALL_WINDOWS_IN_TAB(win, curtab), which for the *current* tab
        // page is the plain `firstwin` walk.
        let mut win = firstwin.get();
        while !win.is_null() {
            let buf = (*win).w_buffer;

            // Don't save the state of the command preview buffer or window.
            if (*buf).handle != cmdpreview_bufnr.get() {
                if !set_has_ptr_t(&raw mut saved_bufs, buf as ptr_t) {
                    let mut cp_bufinfo = CP_BUF_INFO_INIT;
                    cp_bufinfo.buf = buf;
                    cp_bufinfo.save_b_p_ma = (*buf).b_p_ma;
                    cp_bufinfo.save_b_p_ul = (*buf).b_p_ul;
                    cp_bufinfo.save_b_changed = (*buf).b_changed;
                    cp_bufinfo.save_b_op_start = (*buf).b_op_start;
                    cp_bufinfo.save_b_op_end = (*buf).b_op_end;
                    cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
                    cmdpreview_save_undo(&raw mut cp_bufinfo.undo_info, buf);
                    kv_push!((*cpinfo).buf_info, cp_bufinfo);
                    set_put_ptr_t(
                        &raw mut saved_bufs,
                        buf as ptr_t,
                        ::core::ptr::null_mut::<*mut ptr_t>(),
                    );

                    u_clearall(buf);
                    // Make sure every change can be undone.
                    (*buf).b_p_ul = INT_MAX as OptInt;
                }

                let mut cp_wininfo = CP_WIN_INFO_INIT;
                cp_wininfo.win = win;
                // Save the window's cursor position and view state.
                cp_wininfo.save_w_cursor = (*win).w_cursor;
                save_viewstate(win, &raw mut cp_wininfo.save_viewstate);
                // Save 'cursorline' and 'cursorcolumn'.
                cp_wininfo.save_w_p_cul = (*win).w_onebuf_opt.wo_cul;
                cp_wininfo.save_w_p_cuc = (*win).w_onebuf_opt.wo_cuc;
                kv_push!((*cpinfo).win_info, cp_wininfo);

                // Both would otherwise mess up the preview's highlights.
                (*win).w_onebuf_opt.wo_cul = false_0;
                (*win).w_onebuf_opt.wo_cuc = false_0;
            }

            win = (*win).w_next;
        }

        // C's set_destroy. Its trailing `= SET_INIT` is a dead store on a
        // local that is never read again, and is left out.
        xfree(saved_bufs.keys as *mut ::core::ffi::c_void);
        xfree(saved_bufs.h.hash as *mut ::core::ffi::c_void);

        (*cpinfo).save_hls = p_hls.get() != 0;
        (*cpinfo).save_cmdmod = cmdmod.get();
        win_size_save(&raw mut (*cpinfo).save_view);
        save_search_patterns();

        // No search highlighting during a live substitution.
        p_hls.set(false_0);
        // Disable the :leftabove/:botright, :tab and swap-file modifiers.
        (*cmdmod.ptr()).cmod_split = 0;
        (*cmdmod.ptr()).cmod_tab = 0;
        (*cmdmod.ptr()).cmod_flags |= CmdModFlags::NOSWAPFILE;

        u_sync(true);
    }
}

/// Put back everything [`cmdpreview_prepare`] saved, undoing the preview's
/// changes to every buffer it touched.
pub(crate) unsafe fn cmdpreview_restore_state(cpinfo: *mut CpInfo) {
    unsafe {
        let mut i: size_t = 0;
        while i < (*cpinfo).buf_info.size {
            let mut cp_bufinfo: CpBufInfo = *(*cpinfo).buf_info.items.add(i);
            let buf = cp_bufinfo.buf;

            (*buf).b_changed = cp_bufinfo.save_b_changed;

            extmark_clear(
                buf,
                cmdpreview_ns.get() as uint32_t,
                0,
                0,
                MAXLNUM as ::core::ffi::c_int,
                MAXCOL,
            );

            // Undo all the changes the preview made to this buffer.
            if (*buf).b_u_seq_cur != cp_bufinfo.undo_info.save_b_u_seq_cur {
                let mut count = 0;
                let mut uhp = if !(*buf).b_u_curhead.is_null() {
                    (*buf).b_u_curhead
                } else {
                    (*buf).b_u_newhead
                };
                while !uhp.is_null() {
                    uhp = (*uhp).uh_next.ptr;
                    count += 1;
                }

                let mut aco = aco_save_T::default();
                aucmd_prepbuf(&raw mut aco, buf);
                if (*curbuf.get()).b_u_synced as ::core::ffi::c_int == false_0 {
                    u_sync(true);
                }
                if !u_undo_and_forget(count, false) {
                    abort();
                }
                aucmd_restbuf(&raw mut aco);
            }

            u_blockfree(buf);
            cmdpreview_restore_undo(&raw mut cp_bufinfo.undo_info, buf);

            (*buf).b_op_start = cp_bufinfo.save_b_op_start;
            (*buf).b_op_end = cp_bufinfo.save_b_op_end;

            if cp_bufinfo.save_changedtick != buf_get_changedtick(buf) {
                buf_set_changedtick(buf, cp_bufinfo.save_changedtick);
            }

            (*buf).b_p_ul = cp_bufinfo.save_b_p_ul;
            (*buf).b_p_ma = cp_bufinfo.save_b_p_ma;
            i += 1;
        }

        let mut i: size_t = 0;
        while i < (*cpinfo).win_info.size {
            let mut cp_wininfo: CpWinInfo = *(*cpinfo).win_info.items.add(i);
            let win = cp_wininfo.win;

            (*win).w_cursor = cp_wininfo.save_w_cursor;
            restore_viewstate(win, &raw mut cp_wininfo.save_viewstate);
            (*win).w_onebuf_opt.wo_cul = cp_wininfo.save_w_p_cul;
            (*win).w_onebuf_opt.wo_cuc = cp_wininfo.save_w_p_cuc;
            update_topline(win);
            i += 1;
        }

        cmdmod.set((*cpinfo).save_cmdmod);
        p_hls.set((*cpinfo).save_hls as ::core::ffi::c_int);
        restore_search_patterns();
        win_size_restore(&raw mut (*cpinfo).save_view);
        ga_clear(&raw mut (*cpinfo).save_view);

        xfree((*cpinfo).win_info.items as *mut ::core::ffi::c_void);
        (*cpinfo).win_info = CP_INFO_INIT.win_info;
        xfree((*cpinfo).buf_info.items as *mut ::core::ffi::c_void);
        (*cpinfo).buf_info = CP_INFO_INIT.buf_info;
    }
}

/// Run the command being typed as a preview, if it supports one.
///
/// Answers true when a preview was shown.
pub(crate) unsafe fn cmdpreview_may_show(_s: *mut CommandLineState) -> bool {
    unsafe {
        let mut ea: exarg_T = EXARG_T_INIT;
        let mut cmdinfo: CmdParseInfo = CMD_PARSE_INFO_INIT;
        let mut cmdpreview_type = 0;
        // A copy of the command line, so `parse_cmdline` can modify it --
        // it advances this pointer, so it has to be the local itself.
        let mut cmdline = xstrdup((*ccline.ptr()).cmdbuff);
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();

        // C's `goto end`: everything below happens only when the command line
        // parses *and* the command supports a preview.
        'end: {
            // Block errors while parsing the command line, and don't update
            // v:errmsg.
            emsg_off.set(emsg_off.get() + 1);
            let parsed = parse_cmdline(
                &raw mut cmdline,
                &raw mut ea,
                &raw mut cmdinfo,
                &raw mut errormsg,
            );
            emsg_off.set(emsg_off.get() - 1);
            if !parsed {
                break 'end;
            }

            // Is the command previewable? If not, don't attempt a preview.
            if !ea.argt.has(ExArgt::PREVIEW) {
                undo_cmdmod(&raw mut cmdinfo.cmdmod);
                break 'end;
            }

            // The cursor may be at the end of the message grid rather than at
            // cmdspos. Put it there in case the preview callback flushes.
            // #30696
            cursorcmd();
            // Flush now: an external command line may itself wish to update
            // the screen, which is disallowed during cmdpreview.
            cmdline_ui_flush();

            // Swap an invalid command range.
            if ea.argt.has(ExArgt::RANGE) && ea.line1 > ea.line2 {
                ::core::mem::swap(&mut ea.line1, &mut ea.line2);
            }

            let mut cpinfo: CpInfo = CP_INFO_INIT;
            // 'inccommand' = "split"
            let mut icm_split = *p_icm.get() as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
            let mut cmdpreview_buf = ::core::ptr::null_mut::<buf_T>();
            let mut cmdpreview_win = ::core::ptr::null_mut::<win_T>();

            // Block error reporting (the command may be incomplete), but
            // still update v:errmsg; block messages, namely ones that prompt;
            // block events.
            emsg_silent.set(emsg_silent.get() + 1);
            msg_silent.set(msg_silent.get() + 1);
            block_autocmds();

            cmdpreview_prepare(&raw mut cpinfo);

            // Open the preview buffer if 'inccommand' is "split".
            if icm_split && {
                cmdpreview_buf = cmdpreview_open_buf();
                cmdpreview_buf.is_null()
            } {
                // Failed to create the preview buffer, so disable the preview.
                set_option_direct(
                    kOptInccommand,
                    static_optval(c"nosplit"),
                    OptionSetFlags::NONE,
                    SID_NONE,
                );
                icm_split = false;
            }
            // Set up the preview namespace if it is not already set.
            if cmdpreview_ns.get() == 0 {
                cmdpreview_ns.set(nvim_create_namespace(String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                }) as ::core::ffi::c_int);
            }

            cmdpreview.set(true);

            // Execute the preview callback: its return value says whether to
            // show a preview and whether to open the preview window. It also
            // makes the changes and highlights the preview shows.
            let mut err: Error = ERROR_INIT;
            let mut tstate: TryState = TRY_STATE_INIT;
            try_enter(&raw mut tstate);
            cmdpreview_type = execute_cmd(&raw mut ea, &raw mut cmdinfo, true);
            try_leave(&raw mut tstate, &raw mut err);
            if err.type_0 != kErrorTypeNone {
                api_clear_error(&raw mut err);
                cmdpreview_type = 0;
            }

            // With 'inccommand' = "split" and a callback answering 2, open the
            // preview window.
            if icm_split && cmdpreview_type == 2 && {
                cmdpreview_win = cmdpreview_open_win(cmdpreview_buf);
                cmdpreview_win.is_null()
            } {
                // Not enough room for the preview window: preview without it.
                cmdpreview_type = 1;
            }

            // A nonzero answer means the screen has to be updated now.
            if cmdpreview_type != 0 {
                let save_rd = RedrawingDisabled.get();
                RedrawingDisabled.set(0);
                update_screen();
                RedrawingDisabled.set(save_rd);
            }

            // Close the preview window if it is open.
            if icm_split && cmdpreview_type == 2 && !cmdpreview_win.is_null() {
                cmdpreview_close_win();
            }

            cmdpreview_restore_state(&raw mut cpinfo);

            unblock_autocmds();
            msg_silent.set(msg_silent.get() - 1);
            emsg_silent.set(emsg_silent.get() - 1);
            redrawcmdline();
        }

        xfree(cmdline as *mut ::core::ffi::c_void);
        cmdpreview_type != 0
    }
}
