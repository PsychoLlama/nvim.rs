//! `'inccommand'`: running the command being typed against a preview buffer.
//!
//! [`cmdpreview_may_show`] is the entry point — it saves everything the
//! command could change, executes it with `cmdpreview` set, shows the result
//! either in place or in a split preview window, and restores.  The
//! `cmdpreview_save_*` / `cmdpreview_restore_*` pairs are that save-restore.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::ex_docmd::{cmdmod_add_flags, cmdmod_set_split, cmdmod_set_tab};
use crate::guard::{Allow, Suppress};
use crate::types::{CmdModFlags, ExArgt, FAIL, OptionSetFlags};
use crate::winlayer::{Buf, Live, TabPage, Win, windows_in_tab};

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
    let mut cmdpreview_buf = if cmdpreview_bufnr.get() != 0 {
        find_buf(cmdpreview_bufnr.get()).map_or(::core::ptr::null_mut(), |mut b| b.raw())
    } else {
        ::core::ptr::null_mut::<buf_T>()
    };

    // If the preview buffer doesn't exist, open one.
    if cmdpreview_buf.is_null() {
        // SAFETY: creating a scratch buffer needs only a live editor.
        let created = unsafe { nvim_create_buf(false, true) };
        let Ok(bufnr) = created else {
            return ::core::ptr::null_mut::<buf_T>();
        };
        cmdpreview_buf = find_buf(bufnr).map_or(::core::ptr::null_mut(), |mut b| b.raw());
    }

    // The preview buffer cannot preview itself.
    if cmdpreview_buf == curbuf.get() {
        return ::core::ptr::null_mut::<buf_T>();
    }

    // Rename the preview buffer.
    let mut aco = aco_save_T::default();
    unsafe { aucmd_prepbuf(&raw mut aco, cmdpreview_buf) };
    let retv = unsafe { rename_buffer(c"[Preview]".as_ptr().cast_mut()) };
    unsafe { aucmd_restbuf(&raw mut aco) };

    if retv == FAIL {
        return ::core::ptr::null_mut::<buf_T>();
    }

    // Temporarily switch to the preview buffer to set it up.
    unsafe { aucmd_prepbuf(&raw mut aco, cmdpreview_buf) };
    buf_clear();
    cur_buf().b_p_ma = 1;
    cur_buf().b_p_ul = -1;
    // Reset 'textwidth', which a ftplugin may have set.
    cur_buf().b_p_tw = 0;
    unsafe { aucmd_restbuf(&raw mut aco) };
    cmdpreview_bufnr.set(unsafe { (*cmdpreview_buf).handle });

    cmdpreview_buf
}

/// Open the command preview window, if it is not already open, and return to
/// the original window.  Answers NULL if it could not be opened.
pub(crate) unsafe fn cmdpreview_open_win(cmdpreview_buf: *mut buf_T) -> *mut win_T {
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
    unsafe { try_enter(&raw mut tstate) };
    let result = do_buffer(
        DOBUF_GOTO as ::core::ffi::c_int,
        DOBUF_FIRST as ::core::ffi::c_int,
        FORWARD as ::core::ffi::c_int,
        unsafe { (*cmdpreview_buf).handle },
        0,
    );
    unsafe { try_leave(&raw mut tstate, &raw mut err) };

    if err.is_set() || result == FAIL {
        err.clear();
        return ::core::ptr::null_mut::<win_T>();
    }

    cur_win().w_onebuf_opt.wo_cul = 0;
    cur_win().w_onebuf_opt.wo_cuc = 0;
    cur_win().w_onebuf_opt.wo_spell = 0;
    cur_win().w_onebuf_opt.wo_fen = 0;

    unsafe { win_enter(save_curwin, false) };
    preview_win
}

/// Close any open command preview windows.
pub(crate) unsafe fn cmdpreview_close_win() {
    let buf = if cmdpreview_bufnr.get() != 0 {
        find_buf(cmdpreview_bufnr.get()).map_or(::core::ptr::null_mut(), |mut b| b.raw())
    } else {
        ::core::ptr::null_mut::<buf_T>()
    };
    if !buf.is_null() {
        unsafe { close_windows(buf, false) };
    }
}

/// Save `buf`'s whole undo state, so the preview's edits can be taken back.
pub(crate) fn cmdpreview_save_undo(cp_undoinfo: &mut CpUndoInfo, buf: Buf) {
    cp_undoinfo.save_b_u_synced = buf.b_u_synced;
    cp_undoinfo.save_b_u_oldhead = buf.b_u_oldhead;
    cp_undoinfo.save_b_u_newhead = buf.b_u_newhead;
    cp_undoinfo.save_b_u_curhead = buf.b_u_curhead;
    cp_undoinfo.save_b_u_numhead = buf.b_u_numhead;
    cp_undoinfo.save_b_u_seq_last = buf.b_u_seq_last;
    cp_undoinfo.save_b_u_save_nr_last = buf.b_u_save_nr_last;
    cp_undoinfo.save_b_u_seq_cur = buf.b_u_seq_cur;
    cp_undoinfo.save_b_u_time_cur = buf.b_u_time_cur;
    cp_undoinfo.save_b_u_save_nr_cur = buf.b_u_save_nr_cur;
    cp_undoinfo.save_b_u_line_ptr = buf.b_u_line_ptr;
    cp_undoinfo.save_b_u_line_lnum = buf.b_u_line_lnum;
    cp_undoinfo.save_b_u_line_colnr = buf.b_u_line_colnr;
}

/// Put back the undo state [`cmdpreview_save_undo`] recorded.
pub(crate) fn cmdpreview_restore_undo(cp_undoinfo: &CpUndoInfo, mut buf: Buf) {
    buf.b_u_oldhead = cp_undoinfo.save_b_u_oldhead;
    buf.b_u_newhead = cp_undoinfo.save_b_u_newhead;
    buf.b_u_curhead = cp_undoinfo.save_b_u_curhead;
    buf.b_u_numhead = cp_undoinfo.save_b_u_numhead;
    buf.b_u_seq_last = cp_undoinfo.save_b_u_seq_last;
    buf.b_u_save_nr_last = cp_undoinfo.save_b_u_save_nr_last;
    buf.b_u_seq_cur = cp_undoinfo.save_b_u_seq_cur;
    buf.b_u_time_cur = cp_undoinfo.save_b_u_time_cur;
    buf.b_u_save_nr_cur = cp_undoinfo.save_b_u_save_nr_cur;
    buf.b_u_line_ptr = cp_undoinfo.save_b_u_line_ptr;
    buf.b_u_line_lnum = cp_undoinfo.save_b_u_line_lnum;
    buf.b_u_line_colnr = cp_undoinfo.save_b_u_line_colnr;
    if buf.b_u_curhead.is_none() {
        buf.b_u_synced = cp_undoinfo.save_b_u_synced;
    }
}

/// Save the state of every window and buffer in the current tab page, and put
/// the options the preview must not be disturbed by out of the way.
pub(crate) fn cmdpreview_prepare(mut cpinfo: Cp) {
    // C's `kv_push` onto one of `CpInfo`'s two kvecs. A macro rather than
    // a function because `CpBufInfoVec` and `CpWinInfoVec` are separate
    // structs differing only in element type, with nothing relating
    // their fields to be generic over.
    macro_rules! kv_push {
        ($vec:expr, $value:expr) => {{
            let value = $value;
            let v = &raw mut $vec;
            // SAFETY: `v` addresses a kvec field of the live `CpInfo`, whose
            // `items` is its own allocation of `capacity` elements.
            if unsafe { (*v).size } == unsafe { (*v).capacity } {
                let grown = unsafe { (*v).capacity };
                unsafe { (*v).capacity = if grown != 0 { grown << 1 } else { 8 } };
                unsafe {
                    (*v).items = xrealloc(
                        (*v).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of_val(&value) * (*v).capacity,
                    ) as *mut _
                };
            }
            unsafe { *(*v).items.add((*v).size) = value };
            unsafe { (*v).size += 1 };
        }};
    }

    let mut saved_bufs: Set_ptr_t = SET_INIT;

    cpinfo.buf_info = CP_INFO_INIT.buf_info;
    cpinfo.win_info = CP_INFO_INIT.win_info;

    // C's FOR_ALL_WINDOWS_IN_TAB(win, curtab).
    for mut win in windows_in_tab(cur_tab()) {
        let mut buf = win.buffer();

        // Don't save the state of the command preview buffer or window.
        if buf.handle == cmdpreview_bufnr.get() {
            continue;
        }

        // SAFETY: `saved_bufs` is this frame's set, and the key is a pointer
        // value the set only ever compares.
        let seen = unsafe { set_has_ptr_t(&raw mut saved_bufs, buf.raw() as ptr_t) };
        if !seen {
            let mut cp_bufinfo = CP_BUF_INFO_INIT;
            cp_bufinfo.buf = buf.raw();
            cp_bufinfo.save_b_p_ma = buf.b_p_ma;
            cp_bufinfo.save_b_p_ul = buf.b_p_ul;
            cp_bufinfo.save_b_changed = buf.b_changed;
            cp_bufinfo.save_b_op_start = buf.b_op_start;
            cp_bufinfo.save_b_op_end = buf.b_op_end;
            cp_bufinfo.save_changedtick = buf_get_changedtick(buf);
            cmdpreview_save_undo(&mut cp_bufinfo.undo_info, buf);
            kv_push!(cpinfo.buf_info, cp_bufinfo);
            // SAFETY: as the `set_has_ptr_t` above.
            unsafe {
                set_put_ptr_t(
                    &raw mut saved_bufs,
                    buf.raw() as ptr_t,
                    ::core::ptr::null_mut::<*mut ptr_t>(),
                )
            };

            // SAFETY: `buf` is a live buffer of the current tab page.
            u_clearall(buf);
            // Make sure every change can be undone.
            buf.b_p_ul = INT_MAX as OptInt;
        }

        let mut cp_wininfo = CP_WIN_INFO_INIT;
        cp_wininfo.win = win.raw();
        // Save the window's cursor position and view state.
        cp_wininfo.save_w_cursor = win.w_cursor;
        cp_wininfo.save_viewstate = save_viewstate(win);
        // Save 'cursorline' and 'cursorcolumn'.
        cp_wininfo.save_w_p_cul = win.w_onebuf_opt.wo_cul;
        cp_wininfo.save_w_p_cuc = win.w_onebuf_opt.wo_cuc;
        kv_push!(cpinfo.win_info, cp_wininfo);

        // Both would otherwise mess up the preview's highlights.
        win.w_onebuf_opt.wo_cul = 0;
        win.w_onebuf_opt.wo_cuc = 0;
    }

    // C's set_destroy. Its trailing `= SET_INIT` is a dead store on a
    // local that is never read again, and is left out.
    // SAFETY: both are `saved_bufs`' own allocations.
    unsafe { xfree(saved_bufs.keys as *mut ::core::ffi::c_void) };
    unsafe { xfree(saved_bufs.h.hash as *mut ::core::ffi::c_void) };

    cpinfo.save_hls = p_hls.get() != 0;
    cpinfo.save_cmdmod = cmdmod.with(Clone::clone);
    // SAFETY: `save_view` is this `CpInfo`'s own growarray.
    unsafe { win_size_save(&raw mut cpinfo.save_view) };
    save_search_patterns();

    // No search highlighting during a live substitution.
    p_hls.set(0);
    // Disable the :leftabove/:botright, :tab and swap-file modifiers.
    cmdmod_set_split(0);
    cmdmod_set_tab(0);
    cmdmod_add_flags(CmdModFlags::NOSWAPFILE);

    // SAFETY: syncing undo needs only a live editor.
    u_sync(true);
}

/// Put back everything [`cmdpreview_prepare`] saved, undoing the preview's
/// changes to every buffer it touched.
pub(crate) fn cmdpreview_restore_state(mut cpinfo: Cp) {
    let mut i: size_t = 0;
    while i < cpinfo.buf_info.size {
        // SAFETY: `buf_info` holds `size` initialised entries.
        let mut cp_bufinfo: CpBufInfo = unsafe { *cpinfo.buf_info.items.add(i) };
        // SAFETY: the buffer was live when `cmdpreview_prepare` recorded it,
        // and autocommands were blocked throughout the preview.
        let mut buf = unsafe { Buf::new(cp_bufinfo.buf) };

        buf.b_changed = cp_bufinfo.save_b_changed;

        // SAFETY: clearing the preview namespace's marks in a live buffer.
        unsafe {
            extmark_clear(
                buf.raw(),
                cmdpreview_ns.get() as uint32_t,
                0,
                0,
                MAXLNUM as ::core::ffi::c_int,
                MAXCOL,
            )
        };

        // Undo all the changes the preview made to this buffer.
        if buf.b_u_seq_cur != cp_bufinfo.undo_info.save_b_u_seq_cur {
            let start = if buf.b_u_curhead.is_some() {
                buf.b_u_curhead
            } else {
                buf.b_u_newhead
            };
            // SAFETY: `start` is a header of `buf`'s own undo chain.
            let chain = unsafe { header_chain(buf, start, |uh| uh.uh_next) };
            let count = chain.count() as ::core::ffi::c_int;

            let mut aco = aco_save_T::default();
            // SAFETY: `aco` is this frame's, and every `prepbuf` below is
            // paired with the `restbuf` that follows it.
            unsafe { aucmd_prepbuf(&raw mut aco, buf.raw()) };
            if cur_buf().b_u_synced as ::core::ffi::c_int == 0 {
                // SAFETY: syncing undo needs only a live editor.
                u_sync(true);
            }
            // SAFETY: undoing `count` states of the buffer just entered.
            if !unsafe { u_undo_and_forget(count, false) } {
                // SAFETY: `abort` never returns.
                unsafe { abort() };
            }
            // SAFETY: pairs with the `aucmd_prepbuf` above.
            unsafe { aucmd_restbuf(&raw mut aco) };
        }

        // SAFETY: `buf` is live, and its undo state is its own.
        u_blockfree(buf);
        cmdpreview_restore_undo(&cp_bufinfo.undo_info, buf);

        buf.b_op_start = cp_bufinfo.save_b_op_start;
        buf.b_op_end = cp_bufinfo.save_b_op_end;

        // SAFETY: the changed-tick lives in `buf`'s own variable dictionary.
        let tick = buf_get_changedtick(buf);
        if cp_bufinfo.save_changedtick != tick {
            // SAFETY: as above.
            unsafe { buf_set_changedtick(buf.raw(), cp_bufinfo.save_changedtick) };
        }

        buf.b_p_ul = cp_bufinfo.save_b_p_ul;
        buf.b_p_ma = cp_bufinfo.save_b_p_ma;
        i += 1;
    }

    let mut i: size_t = 0;
    while i < cpinfo.win_info.size {
        // SAFETY: `win_info` holds `size` initialised entries.
        let cp_wininfo: CpWinInfo = unsafe { *cpinfo.win_info.items.add(i) };
        // SAFETY: as the buffers above -- recorded live, autocommands blocked.
        let mut win = unsafe { Win::new(cp_wininfo.win) };

        win.w_cursor = cp_wininfo.save_w_cursor;
        restore_viewstate(win, cp_wininfo.save_viewstate);
        win.w_onebuf_opt.wo_cul = cp_wininfo.save_w_p_cul;
        win.w_onebuf_opt.wo_cuc = cp_wininfo.save_w_p_cuc;
        // SAFETY: `win` is live.
        update_topline(win);
        i += 1;
    }

    cmdmod.set(cpinfo.save_cmdmod.clone());
    p_hls.set(cpinfo.save_hls as ::core::ffi::c_int);
    restore_search_patterns();
    win_size_restore(&raw mut cpinfo.save_view);
    // SAFETY: `save_view` is the growarray `win_size_save` filled.
    unsafe { ga_clear(&raw mut cpinfo.save_view) };

    // SAFETY: both `items` are this `CpInfo`'s own allocations.
    unsafe { xfree(cpinfo.win_info.items as *mut ::core::ffi::c_void) };
    unsafe { xfree(cpinfo.buf_info.items as *mut ::core::ffi::c_void) };
    cpinfo.win_info = CP_INFO_INIT.win_info;
    cpinfo.buf_info = CP_INFO_INIT.buf_info;
}

/// Run the command being typed as a preview, if it supports one.
///
/// Answers true when a preview was shown.
pub(crate) unsafe fn cmdpreview_may_show(_s: *mut CommandLineState) -> bool {
    let mut ea: exarg_T = EXARG_T_INIT;
    let mut cmdinfo: CmdParseInfo = CMD_PARSE_INFO_INIT;
    let mut cmdpreview_type = 0;
    // A copy of the command line, so `parse_cmdline` can modify it --
    // it advances this pointer, so it has to be the local itself.
    let mut cmdline = unsafe { xstrdup(Cc::current().text()) };
    let mut errormsg = None;

    // C's `goto end`: everything below happens only when the command line
    // parses *and* the command supports a preview.
    'end: {
        // Block errors while parsing the command line, and don't update
        // v:errmsg.
        let no_emsg = Suppress::emsg();
        let parsed = unsafe {
            parse_cmdline(
                &raw mut cmdline,
                &raw mut ea,
                &raw mut cmdinfo,
                &mut errormsg,
            )
        };
        drop(no_emsg);
        if !parsed {
            break 'end;
        }

        // Is the command previewable? If not, don't attempt a preview.
        if !ea.argt.has(ExArgt::PREVIEW) {
            unsafe { undo_cmdmod(&mut cmdinfo.cmdmod) };
            break 'end;
        }

        // The cursor may be at the end of the message grid rather than at
        // cmdspos. Put it there in case the preview callback flushes.
        // #30696
        unsafe { cursorcmd() };
        // Flush now: an external command line may itself wish to update
        // the screen, which is disallowed during cmdpreview.
        cmdline_ui_flush();

        // Swap an invalid command range.
        if ea.argt.has(ExArgt::RANGE) && ea.line1 > ea.line2 {
            ::core::mem::swap(&mut ea.line1, &mut ea.line2);
        }

        let mut cpinfo: CpInfo = CP_INFO_INIT;
        // 'inccommand' = "split"
        let mut icm_split =
            unsafe { *p_icm.get() } as ::core::ffi::c_int == 's' as ::core::ffi::c_int;
        let mut cmdpreview_buf = ::core::ptr::null_mut::<buf_T>();
        let mut cmdpreview_win = ::core::ptr::null_mut::<win_T>();

        // Block error reporting (the command may be incomplete), but
        // still update v:errmsg; block messages, namely ones that prompt;
        // block events.
        emsg_silent.set(emsg_silent.get() + 1);
        let silenced = Suppress::messages();
        unsafe { block_autocmds() };

        // SAFETY: `cpinfo` lives in this frame for the whole preview.
        cmdpreview_prepare(unsafe { Cp::new(&raw mut cpinfo) });

        // Open the preview buffer if 'inccommand' is "split".
        if icm_split && {
            cmdpreview_buf = unsafe { cmdpreview_open_buf() };
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
            cmdpreview_ns
                .set(unsafe { nvim_create_namespace(String_0::NULL) } as ::core::ffi::c_int);
        }

        cmdpreview.set(true);

        // Execute the preview callback: its return value says whether to
        // show a preview and whether to open the preview window. It also
        // makes the changes and highlights the preview shows.
        let mut err: Error = ERROR_INIT;
        let mut tstate: TryState = TRY_STATE_INIT;
        unsafe { try_enter(&raw mut tstate) };
        cmdpreview_type = unsafe { execute_cmd(&raw mut ea, &raw mut cmdinfo, true) };
        unsafe { try_leave(&raw mut tstate, &raw mut err) };
        if err.is_set() {
            err.clear();
            cmdpreview_type = 0;
        }

        // With 'inccommand' = "split" and a callback answering 2, open the
        // preview window.
        if icm_split && cmdpreview_type == 2 && {
            cmdpreview_win = unsafe { cmdpreview_open_win(cmdpreview_buf) };
            cmdpreview_win.is_null()
        } {
            // Not enough room for the preview window: preview without it.
            cmdpreview_type = 1;
        }

        // A nonzero answer means the screen has to be updated now.
        if cmdpreview_type != 0 {
            let _redraw = Allow::redraw();
            unsafe { update_screen() };
        }

        // Close the preview window if it is open.
        if icm_split && cmdpreview_type == 2 && !cmdpreview_win.is_null() {
            unsafe { cmdpreview_close_win() };
        }

        // SAFETY: as the `cmdpreview_prepare` above.
        cmdpreview_restore_state(unsafe { Cp::new(&raw mut cpinfo) });

        unsafe { unblock_autocmds() };
        drop(silenced);
        emsg_silent.set(emsg_silent.get() - 1);
        unsafe { redrawcmdline() };
    }

    unsafe { xfree(cmdline as *mut ::core::ffi::c_void) };
    cmdpreview_type != 0
}

/// [`Live`]'s shape for the save/restore state of one `'inccommand'` run,
/// which lives in [`cmdpreview_may_show`]'s frame.
pub(crate) type Cp = Live<CpInfo>;

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
