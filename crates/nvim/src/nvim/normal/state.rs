//! The normal-mode state machine: one pass of the loop, and the
//! checks it runs between keystrokes.
//!
//! `normal_enter` installs the state and `normal_cmd` is one command. Everything
//! named `normal_check_*` runs once per iteration when the typeahead is empty,
//! which is what makes them the editor's idle work.

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C" fn normal_state_init(mut s: *mut NormalState) {
    memset(
        s as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<NormalState>(),
    );
    (*s).state.check =
        Some(normal_check as unsafe extern "C" fn(*mut VimState) -> c_int) as state_check_callback;
    (*s).state.execute = Some(normal_execute as unsafe extern "C" fn(*mut VimState, c_int) -> c_int)
        as state_execute_callback;
}

pub(crate) unsafe extern "C" fn check_text_locked(mut oap: *mut oparg_T) -> bool {
    if !text_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearopbeep(oap);
    }
    text_locked_msg();
    return true_0 != 0;
}

pub unsafe extern "C" fn check_text_or_curbuf_locked(mut oap: *mut oparg_T) -> bool {
    if check_text_locked(oap) {
        return true_0 != 0;
    }
    if !curbuf_locked() {
        return false_0 != 0;
    }
    if !oap.is_null() {
        clearop(oap);
    }
    return true_0 != 0;
}

pub unsafe extern "C" fn op_pending() -> bool {
    return !(!(*current_oap.ptr()).is_null()
        && !finish_op.get()
        && (*current_oap.get()).prev_opcount == 0 as c_int
        && (*current_oap.get()).prev_count0 == 0 as c_int
        && (*current_oap.get()).op_type == OP_NOP as c_int
        && (*current_oap.get()).regname == NUL);
}

pub unsafe extern "C" fn normal_enter(mut cmdwin: bool, mut noexmode: bool) {
    let mut state: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut state);
    let mut prev_oap: *mut oparg_T = current_oap.get();
    current_oap.set(&raw mut state.oa);
    state.cmdwin = cmdwin;
    state.noexmode = noexmode;
    state.toplevel = (!cmdwin || cmdwin_result.get() == 0 as c_int) && !noexmode;
    state_enter(&raw mut state.state);
    current_oap.set(prev_oap);
}

pub(crate) unsafe extern "C" fn normal_prepare(mut s: *mut NormalState) {
    memset(
        &raw mut (*s).ca as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdarg_T>(),
    );
    (*s).ca.oap = &raw mut (*s).oa;
    (*s).ca.opcount = opcount.get();
    let mut c: c_int = finish_op.get() as c_int;
    finish_op.set((*s).oa.op_type != OP_NOP as c_int);
    if finish_op.get() as c_int != c {
        ui_cursor_shape();
    }
    may_trigger_modechanged();
    (*s).set_prevcount = false_0 != 0;
    if !finish_op.get() && (*s).oa.regname == 0 {
        (*s).ca.opcount = 0 as c_int;
        (*s).set_prevcount = true_0 != 0;
    }
    if (*s).oa.prev_opcount > 0 as c_int || (*s).oa.prev_count0 > 0 as c_int {
        (*s).ca.opcount = (*s).oa.prev_opcount;
        (*s).ca.count0 = (*s).oa.prev_count0;
        (*s).oa.prev_opcount = 0 as c_int;
        (*s).oa.prev_count0 = 0 as c_int;
    }
    (*s).mapped_len = typebuf_maplen();
    State.set(MODE_NORMAL_BUSY as c_int);
    if (*s).toplevel as c_int != 0 && readbuf1_empty() as c_int != 0 {
        set_vcount_ca(&raw mut (*s).ca, &raw mut (*s).set_prevcount);
    }
}

pub(crate) unsafe extern "C" fn normal_handle_special_visual_command(
    mut s: *mut NormalState,
) -> bool {
    if km_stopsel.get() as c_int != 0
        && (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_STS != 0
        && mod_mask.get() & MOD_MASK_SHIFT == 0
    {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED as c_int);
    }
    if km_startsel.get() {
        if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SS != 0 {
            unshift_special(&raw mut (*s).ca);
            (*s).idx = find_command((*s).ca.cmdchar);
            if (*s).idx < 0 as c_int {
                clearopbeep(&raw mut (*s).oa);
                return true_0 != 0;
            }
        } else if (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int & NV_SSS != 0
            && mod_mask.get() & MOD_MASK_SHIFT != 0
        {
            (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
        }
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn normal_need_additional_char(mut s: *mut NormalState) -> bool {
    let mut flags: c_int = (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int;
    let mut pending_op: bool = (*s).oa.op_type != OP_NOP as c_int;
    let mut cmdchar: c_int = (*s).ca.cmdchar;
    return flags & NV_NCH != 0
        && (flags & NV_NCH_NOP == NV_NCH_NOP && !pending_op
            || flags & NV_NCH_ALW == NV_NCH_ALW
            || cmdchar == 'q' as c_int
                && !pending_op
                && reg_recording.get() == 0 as c_int
                && reg_executing.get() == 0 as c_int
            || (cmdchar == 'a' as c_int || cmdchar == 'i' as c_int)
                && (pending_op as c_int != 0 || VIsual_active.get() as c_int != 0));
}

pub(crate) unsafe extern "C" fn normal_need_redraw_mode_message(mut s: *mut NormalState) -> bool {
    return (p_smd.get() != 0
        && msg_silent.get() == 0 as c_int
        && (restart_edit.get() != 0 as c_int
            || VIsual_active.get() as c_int != 0
                && (*s).old_pos.lnum == (*curwin.get()).w_cursor.lnum
                && (*s).old_pos.col == (*curwin.get()).w_cursor.col)
        && (clear_cmdline.get() as c_int != 0 || redraw_cmdline.get() as c_int != 0)
        && (msg_didout.get() as c_int != 0
            || msg_didany.get() as c_int != 0 && msg_scroll.get() != 0)
        && !msg_nowait.get()
        && KeyTyped.get() as c_int != 0
        || restart_edit.get() != 0 as c_int
            && !VIsual_active.get()
            && msg_scroll.get() != 0
            && emsg_on_display.get() as c_int != 0)
        && (*s).oa.regname == 0 as c_int
        && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty() as c_int != 0
        && typebuf_typed() != 0
        && emsg_silent.get() == 0 as c_int
        && !in_assert_fails.get()
        && !did_wait_return.get()
        && (*s).oa.op_type == OP_NOP as c_int;
}

pub(crate) unsafe extern "C" fn normal_redraw_mode_message(mut _s: *mut NormalState) {
    let mut save_State: c_int = State.get();
    if restart_edit.get() != 0 as c_int {
        State.set(MODE_INSERT as c_int);
    }
    if must_redraw.get() != 0 && !(*keep_msg.ptr()).is_null() && !emsg_on_display.get() {
        let mut kmsg: *mut c_char = ::core::ptr::null_mut::<c_char>();
        kmsg = keep_msg.get();
        keep_msg.set(::core::ptr::null_mut::<c_char>());
        setcursor();
        update_screen();
        keep_msg.set(kmsg);
        kmsg = xstrdup(keep_msg.get());
        msg(kmsg, keep_msg_hl_id.get());
        xfree(kmsg as *mut c_void);
    }
    setcursor();
    ui_cursor_shape();
    ui_flush();
    if msg_scroll.get() != 0 || emsg_on_display.get() as c_int != 0 {
        msg_delay(1003 as uint64_t, true_0 != 0);
    }
    msg_delay(3003 as uint64_t, false_0 != 0);
    State.set(save_State);
    msg_scroll.set(false_0);
    emsg_on_display.set(false_0 != 0);
}

pub(crate) unsafe extern "C" fn normal_check_stuff_buffer(mut _s: *mut NormalState) {
    if stuff_empty() {
        did_check_timestamps.set(false_0 != 0);
        if need_check_timestamps.get() {
            check_timestamps(false_0);
        }
        if need_wait_return.get() {
            wait_return(false_0);
        }
    }
}

pub(crate) unsafe extern "C" fn normal_check_interrupt(mut s: *mut NormalState) {
    if got_int.get() {
        if (*s).noexmode as c_int != 0
            && global_busy.get() != 0
            && !exmode_active.get()
            && (*s).previous_got_int as c_int != 0
        {
            exmode_active.set(true_0 != 0);
            State.set(MODE_NORMAL as c_int);
        } else if global_busy.get() == 0 || !exmode_active.get() {
            if !quit_more.get() {
                vgetc();
            }
            got_int.set(false_0 != 0);
        }
        (*s).previous_got_int = true_0 != 0;
    } else {
        (*s).previous_got_int = false_0 != 0;
    };
}

pub(crate) unsafe extern "C" fn normal_check_window_scrolled(mut _s: *mut NormalState) {
    if !finish_op.get() {
        may_trigger_win_scrolled_resized();
    }
}

pub(crate) unsafe extern "C" fn normal_check_cursor_moved(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_CURSORMOVED) as c_int != 0
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
    {
        apply_autocmds(
            EVENT_CURSORMOVED,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set((*curwin.get()).w_cursor);
    }
}

pub(crate) unsafe extern "C" fn normal_check_text_changed(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_TEXTCHANGED) as c_int != 0
        && (*curbuf.get()).b_last_changedtick != buf_get_changedtick(curbuf.get())
    {
        apply_autocmds(
            EVENT_TEXTCHANGED,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
    }
}

pub(crate) unsafe extern "C" fn normal_check_buffer_modified(mut _s: *mut NormalState) {
    if !finish_op.get()
        && has_event(EVENT_BUFMODIFIEDSET) as c_int != 0
        && (*curbuf.get()).b_changed_invalid as c_int == true_0
    {
        apply_autocmds(
            EVENT_BUFMODIFIEDSET,
            ::core::ptr::null_mut::<c_char>(),
            ::core::ptr::null_mut::<c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*curbuf.get()).b_changed_invalid = false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn normal_check_safe_state(mut _s: *mut NormalState) {
    may_trigger_safestate(!op_pending() && restart_edit.get() == 0 as c_int);
}

pub(crate) unsafe extern "C" fn normal_check_folds(mut _s: *mut NormalState) {
    foldAdjustVisual();
    if hasAnyFolding(curwin.get()) != 0 && !char_avail() {
        foldCheckClose();
        if fdo_flags.get() & kOptFdoFlagAll as c_int as c_uint != 0 {
            foldOpenCursor();
        }
    }
}

pub(crate) unsafe extern "C" fn normal_redraw(mut _s: *mut NormalState) {
    update_topline(curwin.get());
    validate_cursor(curwin.get());
    show_cursor_info_later(false_0 != 0);
    if must_redraw.get() != 0 {
        update_screen();
    } else {
        redraw_statuslines();
        if redraw_cmdline.get() as c_int != 0
            || clear_cmdline.get() as c_int != 0
            || redraw_mode.get() as c_int != 0
        {
            showmode();
        }
    }
    (*curbuf.get()).b_last_used = time(::core::ptr::null_mut::<time_t>());
    if !(*keep_msg.ptr()).is_null() {
        let p: *mut c_char = xstrdup(keep_msg.get());
        msg_hist_off.set(true_0 != 0);
        msg(p, keep_msg_hl_id.get());
        msg_hist_off.set(false_0 != 0);
        xfree(p as *mut c_void);
    }
    if need_fileinfo.get() as c_int != 0 && !shortmess(SHM_FILEINFO as c_int) {
        fileinfo(false_0, true_0, false_0 != 0);
        need_fileinfo.set(false_0 != 0);
    }
    emsg_on_display.set(false_0 != 0);
    did_emsg.set(false_0);
    msg_didany.set(false_0 != 0);
    may_clear_sb_text();
    setcursor();
}

pub(crate) unsafe extern "C" fn normal_check(mut state: *mut VimState) -> c_int {
    let mut s: *mut NormalState = state as *mut NormalState;
    normal_check_stuff_buffer(s);
    normal_check_interrupt(s);
    if did_throw.get() as c_int != 0 && ex_normal_busy.get() == 0 {
        discard_current_exception();
    }
    if !exmode_active.get() {
        msg_scroll.set(false_0);
    }
    quit_more.set(false_0 != 0);
    state_no_longer_safe(::core::ptr::null::<c_char>());
    if skip_redraw.get() as c_int != 0 || exmode_active.get() as c_int != 0 {
        skip_redraw.set(false_0 != 0);
        setcursor();
    } else if do_redraw.get() as c_int != 0 || stuff_empty() as c_int != 0 {
        terminal_check_refresh();
        update_topline(curwin.get());
        validate_cursor(curwin.get());
        normal_check_cursor_moved(s);
        normal_check_text_changed(s);
        normal_check_window_scrolled(s);
        normal_check_buffer_modified(s);
        normal_check_safe_state(s);
        if (*curtab.get()).tp_diff_update != 0 || (*curtab.get()).tp_diff_invalid != 0 {
            ex_diffupdate(::core::ptr::null_mut::<exarg_T>());
            (*curtab.get()).tp_diff_update = false_0;
        }
        if diff_need_scrollbind.get() {
            check_scrollbind(0 as linenr_T, 0 as c_int);
            diff_need_scrollbind.set(false_0 != 0);
        }
        normal_check_folds(s);
        normal_redraw(s);
        do_redraw.set(false_0 != 0);
        if !(*time_fd.ptr()).is_null() {
            if !(*time_fd.ptr()).is_null() {
                time_msg(
                    b"first screen update\0".as_ptr() as *const c_char,
                    ::core::ptr::null::<proftime_T>(),
                );
            }
            time_finish();
        }
        may_make_initial_scroll_size_snapshot();
    }
    may_garbage_collect.set(!(*s).cmdwin && !(*s).noexmode);
    update_curswant();
    if exmode_active.get() {
        if (*s).noexmode {
            return 0 as c_int;
        }
        do_exmode();
        return -1 as c_int;
    }
    if (*s).cmdwin as c_int != 0 && cmdwin_result.get() != 0 as c_int {
        return 0 as c_int;
    }
    normal_prepare(s);
    return 1 as c_int;
}

pub(crate) unsafe extern "C" fn set_vcount_ca(
    mut cap: *mut cmdarg_T,
    mut set_prevcount: *mut bool,
) {
    let mut count: int64_t = (*cap).count0 as int64_t;
    if (*cap).opcount != 0 as c_int {
        count = (*cap).opcount as int64_t
            * (if count == 0 as int64_t {
                1 as int64_t
            } else {
                count
            });
    }
    set_vcount(
        count,
        if count == 0 as int64_t {
            1 as int64_t
        } else {
            count
        },
        *set_prevcount,
    );
    *set_prevcount = false_0 != 0;
}

pub unsafe extern "C" fn normal_cmd(mut oap: *mut oparg_T, mut toplevel: bool) {
    let mut s: NormalState = NormalState {
        state: VimState {
            check: None,
            execute: None,
        },
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa: oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        },
        ca: cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<c_char>(),
        },
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    };
    normal_state_init(&raw mut s);
    s.toplevel = toplevel;
    s.oa = *oap;
    normal_prepare(&raw mut s);
    normal_execute(&raw mut s.state, safe_vgetc());
    *oap = s.oa;
}
