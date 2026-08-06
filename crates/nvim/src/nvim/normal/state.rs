//! The normal-mode state machine: one pass of the loop, and the checks it
//! runs between keystrokes.
//!
//! `normal_enter` installs the state and hands it to `state_enter`, which
//! then alternates [`normal_check`] and
//! [`normal_execute`](super::dispatch::normal_execute) until one of them says
//! to stop. Everything named `normal_check_*` runs once per iteration and
//! only when nothing is waiting to be typed, which is what makes it the
//! editor's idle work: autocommands, fold closing, the redraw.
//!
//! This is the per-keystroke path, so `GlobalCell` is read through `get`/`ptr`
//! and never through `with`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::src::nvim::autocmd::{
    EVENT_BUFMODIFIEDSET, EVENT_CURSORMOVED, EVENT_TEXTCHANGED, apply_autocmds, has_event,
};
use crate::src::nvim::buffer::{buf_get_changedtick, fileinfo};
use crate::src::nvim::diff::ex_diffupdate;
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, redraw_curbuf_later, redraw_statuslines, setcursor, show_cursor_info_later,
    showmode, update_screen,
};
use crate::src::nvim::eval::vars::set_vcount;
use crate::src::nvim::ex_docmd::do_exmode;
use crate::src::nvim::ex_eval::discard_current_exception;
use crate::src::nvim::ex_getln::{curbuf_locked, text_locked, text_locked_msg};
use crate::src::nvim::fileio::check_timestamps;
use crate::src::nvim::fold::{foldAdjustVisual, foldCheckClose, foldOpenCursor, hasAnyFolding};
use crate::src::nvim::getchar::{
    char_avail, readbuf1_empty, safe_vgetc, stuff_empty, typebuf_maplen, typebuf_typed, vgetc,
};
use crate::src::nvim::main::{
    KeyTyped, State, VIsual_active, clear_cmdline, cmdwin_result, curbuf, curtab, curwin,
    did_check_timestamps, did_emsg, did_throw, did_wait_return, diff_need_scrollbind, do_redraw,
    emsg_on_display, emsg_silent, ex_normal_busy, exmode_active, fdo_flags, finish_op, global_busy,
    got_int, in_assert_fails, keep_msg, keep_msg_hl_id, km_startsel, km_stopsel, last_cursormoved,
    last_cursormoved_win, may_garbage_collect, mod_mask, msg_didany, msg_didout, msg_hist_off,
    msg_nowait, msg_scroll, msg_silent, must_redraw, need_check_timestamps, need_fileinfo,
    need_wait_return, opcount, p_smd, quit_more, redraw_cmdline, redraw_mode, reg_executing,
    reg_recording, restart_edit, skip_redraw, time_fd,
};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{may_clear_sb_text, msg, msg_delay, wait_return};
use crate::src::nvim::normal::{
    CA_COMMAND_BUSY, MOD_MASK_SHIFT, NUL, NV_NCH, NV_NCH_ALW, NV_NCH_NOP, NV_SS, NV_SSS, NV_STS,
    NormalState, SHM_FILEINFO, check_scrollbind, clearop, clearopbeep, current_oap,
    end_visual_mode, false_0, find_command, normal_execute, nv_cmds, true_0, unshift_special,
};
use crate::src::nvim::option::shortmess;
use crate::src::nvim::options::kOptFdoFlagAll;
use crate::src::nvim::os::libc::time;
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::profile::{time_finish, time_msg};
use crate::src::nvim::state::{
    MODE_INSERT, MODE_NORMAL, MODE_NORMAL_BUSY, may_trigger_modechanged, may_trigger_safestate,
    state_enter, state_no_longer_safe,
};
use crate::src::nvim::terminal::terminal_check_refresh;
use crate::src::nvim::types::{OP_NOP, VimState, cmdarg_T, int64_t, oparg_T};
use crate::src::nvim::ui::{ui_cursor_shape, ui_flush};
use crate::src::nvim::window::{
    may_make_initial_scroll_size_snapshot, may_trigger_win_scrolled_resized,
};
use core::ffi::{c_int, c_uint, c_void};

use crate::src::nvim::r#move::{update_curswant, update_topline, validate_cursor};

/// A zeroed state with its two callbacks installed.
///
/// Upstream declares the structure, memsets it and then sets the callbacks;
/// the transpile spelled the declaration as a 70-line literal naming every
/// field, all of which the memset immediately overwrote. `zeroed()` is that
/// memset, and `kMTCharWise` -- the one field the literal gave a name rather
/// than a zero -- is itself 0.
fn new_state() -> NormalState {
    // SAFETY: `NormalState` is scalars, raw pointers and nested C structs
    // with no niche; all-zero is a valid value for every field, and it is
    // exactly what the C original memsets it to.
    let mut s: NormalState = unsafe { core::mem::zeroed() };
    s.state.check = Some(normal_check);
    s.state.execute = Some(normal_execute);
    s
}

/// Refuse a command that would change text while the text is locked.
///
/// Beeps and clears the pending operator when there is one to clear.
pub(crate) unsafe fn check_text_locked(oap: *mut oparg_T) -> bool {
    // SAFETY: `oap` is null or the caller's operator.
    unsafe {
        if !text_locked() {
            return false;
        }
        if !oap.is_null() {
            clearopbeep(oap);
        }
        text_locked_msg();
        true
    }
}

/// As [`check_text_locked`], and also refuse while the current buffer is
/// locked. A locked buffer clears the operator without a beep.
pub(crate) unsafe fn check_text_or_curbuf_locked(oap: *mut oparg_T) -> bool {
    // SAFETY: `oap` is null or the caller's operator.
    unsafe {
        if check_text_locked(oap) {
            return true;
        }
        if !curbuf_locked() {
            return false;
        }
        if !oap.is_null() {
            clearop(oap);
        }
        true
    }
}

/// Whether a command is half-typed: an operator waiting for its motion, a
/// count or a register already given.
///
/// Reads the operator the innermost `normal_enter`/`normal_cmd` installed.
pub(crate) fn op_pending() -> bool {
    // SAFETY: `current_oap` is null or points at a live caller's `oparg_T`.
    unsafe {
        let oap = current_oap.get();
        !(!oap.is_null()
            && !finish_op.get()
            && (*oap).prev_opcount == 0
            && (*oap).prev_count0 == 0
            && (*oap).op_type == OP_NOP as c_int
            && (*oap).regname == NUL)
    }
}

/// Run normal mode until something asks to leave it.
///
/// `cmdwin` means this is the command-line window's own normal mode and
/// `noexmode` that Ex mode must not be entered from it; only a normal mode
/// that is neither is "toplevel", which is what decides whether `v:count` is
/// published.
pub(crate) fn normal_enter(cmdwin: bool, noexmode: bool) {
    let mut state = new_state();
    // The innermost operator is the one `op_pending` reports on; the outer
    // one is put back on the way out.
    let prev_oap = current_oap.get();
    current_oap.set(&raw mut state.oa);
    state.cmdwin = cmdwin;
    state.noexmode = noexmode;
    state.toplevel = (!cmdwin || cmdwin_result.get() == 0) && !noexmode;
    // SAFETY: `state` outlives the call.
    unsafe { state_enter(&raw mut state.state) };
    current_oap.set(prev_oap);
}

/// Set up `s.ca` for the command about to be read.
pub(crate) unsafe fn normal_prepare(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state.
    unsafe {
        (*s).ca = core::mem::zeroed();
        (*s).ca.oap = &raw mut (*s).oa;
        (*s).ca.opcount = opcount.get();

        // 'finish_op' drives the cursor shape, so a change to it is a redraw.
        let was_finishing = finish_op.get();
        finish_op.set((*s).oa.op_type != OP_NOP as c_int);
        if finish_op.get() != was_finishing {
            ui_cursor_shape();
        }
        may_trigger_modechanged();

        // With no operator and no register pending, the count starts over --
        // and `set_prevcount` remembers to publish it as v:prevcount.
        (*s).set_prevcount = false;
        if !finish_op.get() && (*s).oa.regname == 0 {
            (*s).ca.opcount = 0;
            (*s).set_prevcount = true;
        }
        // A count the previous command stashed comes back here.
        if (*s).oa.prev_opcount > 0 || (*s).oa.prev_count0 > 0 {
            (*s).ca.opcount = (*s).oa.prev_opcount;
            (*s).ca.count0 = (*s).oa.prev_count0;
            (*s).oa.prev_opcount = 0;
            (*s).oa.prev_count0 = 0;
        }

        (*s).mapped_len = typebuf_maplen();
        State.set(MODE_NORMAL_BUSY);
        if (*s).toplevel && readbuf1_empty() {
            set_vcount_ca(&raw mut (*s).ca, &mut (*s).set_prevcount);
        }
    }
}

/// Apply 'keymodel' to the command just looked up.
///
/// Answers whether the command was rejected outright, which happens when
/// unshifting a special key leaves a character no table row claims.
pub(crate) unsafe fn normal_handle_special_visual_command(s: *mut NormalState) -> bool {
    // SAFETY: `s` is the caller's live state and `s.idx` is a valid row.
    unsafe {
        let flags = (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int;
        // "stopsel": an unshifted movement ends the selection.
        if km_stopsel.get() && flags & NV_STS != 0 && mod_mask.get() & MOD_MASK_SHIFT == 0 {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED);
        }
        if km_startsel.get() {
            if flags & NV_SS != 0 {
                // A shifted special key becomes its unshifted self, and the
                // table has to be consulted again for the new character.
                unshift_special(&raw mut (*s).ca);
                (*s).idx = find_command((*s).ca.cmdchar);
                if (*s).idx < 0 {
                    clearopbeep(&raw mut (*s).oa);
                    return true;
                }
            } else if flags & NV_SSS != 0 && mod_mask.get() & MOD_MASK_SHIFT != 0 {
                (*mod_mask.ptr()) &= !MOD_MASK_SHIFT;
            }
        }
        false
    }
}

/// Whether this command wants a second character read for it.
///
/// `NV_NCH_ALW` always does; `NV_NCH_NOP` only when no operator is pending.
/// `q`, `a` and `i` are spelled out because whether they take one depends on
/// state rather than on the row: `q` only starts a recording when none is
/// running, and `a`/`i` are text objects rather than insert commands only
/// while an operator or Visual mode is waiting for them.
pub(crate) unsafe fn normal_need_additional_char(s: *mut NormalState) -> bool {
    // SAFETY: `s` is the caller's live state and `s.idx` is a valid row.
    unsafe {
        let flags = (*nv_cmds.ptr())[(*s).idx as usize].cmd_flags as c_int;
        let pending_op = (*s).oa.op_type != OP_NOP as c_int;
        let cmdchar = (*s).ca.cmdchar;
        flags & NV_NCH != 0
            && (flags & NV_NCH_NOP == NV_NCH_NOP && !pending_op
                || flags & NV_NCH_ALW == NV_NCH_ALW
                || cmdchar == 'q' as c_int
                    && !pending_op
                    && reg_recording.get() == 0
                    && reg_executing.get() == 0
                || (cmdchar == 'a' as c_int || cmdchar == 'i' as c_int)
                    && (pending_op || VIsual_active.get()))
    }
}

/// Whether the mode message the last command scrolled away has to be put
/// back before the next key is read.
pub(crate) unsafe fn normal_need_redraw_mode_message(s: *mut NormalState) -> bool {
    // SAFETY: `s` is the caller's live state.
    unsafe {
        let showing_mode = p_smd.get() != 0
            && msg_silent.get() == 0
            && (restart_edit.get() != 0
                || VIsual_active.get()
                    && (*s).old_pos.lnum == (*curwin.get()).w_cursor.lnum
                    && (*s).old_pos.col == (*curwin.get()).w_cursor.col)
            && (clear_cmdline.get() || redraw_cmdline.get())
            && (msg_didout.get() || msg_didany.get() && msg_scroll.get() != 0)
            && !msg_nowait.get()
            && KeyTyped.get();
        // The other way in: an error is on display and insert mode is
        // pending, with no Visual selection to describe instead.
        let error_on_display = restart_edit.get() != 0
            && !VIsual_active.get()
            && msg_scroll.get() != 0
            && emsg_on_display.get();

        (showing_mode || error_on_display)
            && (*s).oa.regname == 0
            && (*s).ca.retval & CA_COMMAND_BUSY as c_int == 0
            && stuff_empty()
            && typebuf_typed() != 0
            && emsg_silent.get() == 0
            && !in_assert_fails.get()
            && !did_wait_return.get()
            && (*s).oa.op_type == OP_NOP as c_int
    }
}

/// Redraw the screen the message was scrolled off, show the message again,
/// and pause long enough for it to be read.
pub(crate) fn normal_redraw_mode_message() {
    let save_state = State.get();
    if restart_edit.get() != 0 {
        State.set(MODE_INSERT);
    }
    // SAFETY: `keep_msg` is null or an owned string; the copy is what makes
    // it safe to pass to `msg`, which may free the global.
    unsafe {
        if must_redraw.get() != 0 && !(*keep_msg.ptr()).is_null() && !emsg_on_display.get() {
            // The redraw must not print the kept message itself, so it is
            // taken out of the global for the duration and put back after.
            let kmsg = keep_msg.get();
            keep_msg.set(ptr::null_mut());
            setcursor();
            update_screen();
            keep_msg.set(kmsg);
            let copy = xstrdup(keep_msg.get());
            msg(copy, keep_msg_hl_id.get());
            xfree(copy.cast::<c_void>());
        }
        setcursor();
        ui_cursor_shape();
        ui_flush();
        if msg_scroll.get() != 0 || emsg_on_display.get() {
            msg_delay(1003, true);
        }
        msg_delay(3003, false);
    }
    State.set(save_state);
    msg_scroll.set(false_0);
    emsg_on_display.set(false);
}

/// File timestamps and a pending "Press ENTER", once the stuff buffer runs
/// dry.
fn normal_check_stuff_buffer() {
    // SAFETY: all three are global editor state.
    unsafe {
        if stuff_empty() {
            did_check_timestamps.set(false);
            if need_check_timestamps.get() {
                check_timestamps(false_0);
            }
            if need_wait_return.get() {
                wait_return(false_0);
            }
        }
    }
}

/// Absorb an interrupt.
///
/// A second CTRL-C while `:global` is running and Ex mode was asked for is
/// what gets you into Ex mode; otherwise the interrupt is swallowed, along
/// with the key that caused it when the more-prompt is not up.
unsafe fn normal_check_interrupt(s: *mut NormalState) {
    // SAFETY: `s` is the caller's live state.
    unsafe {
        if !got_int.get() {
            (*s).previous_got_int = false;
            return;
        }
        if (*s).noexmode && global_busy.get() != 0 && !exmode_active.get() && (*s).previous_got_int
        {
            exmode_active.set(true);
            State.set(MODE_NORMAL);
        } else if global_busy.get() == 0 || !exmode_active.get() {
            if !quit_more.get() {
                vgetc();
            }
            got_int.set(false);
        }
        (*s).previous_got_int = true;
    }
}

fn normal_check_window_scrolled() {
    if !finish_op.get() {
        // SAFETY: fires autocommands for the current window.
        unsafe { may_trigger_win_scrolled_resized() };
    }
}

fn normal_check_cursor_moved() {
    // SAFETY: reads the current window and fires an autocommand.
    unsafe {
        if !finish_op.get()
            && has_event(EVENT_CURSORMOVED)
            && (last_cursormoved_win.get() != curwin.get()
                || !equalpos(last_cursormoved.get(), (*curwin.get()).w_cursor))
        {
            apply_autocmds(
                EVENT_CURSORMOVED,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
            last_cursormoved_win.set(curwin.get());
            last_cursormoved.set((*curwin.get()).w_cursor);
        }
    }
}

fn normal_check_text_changed() {
    // SAFETY: reads the current buffer and fires an autocommand.
    unsafe {
        if !finish_op.get()
            && has_event(EVENT_TEXTCHANGED)
            && (*curbuf.get()).b_last_changedtick != buf_get_changedtick(curbuf.get())
        {
            apply_autocmds(
                EVENT_TEXTCHANGED,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
            (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
        }
    }
}

fn normal_check_buffer_modified() {
    // SAFETY: reads the current buffer and fires an autocommand.
    unsafe {
        if !finish_op.get()
            && has_event(EVENT_BUFMODIFIEDSET)
            && (*curbuf.get()).b_changed_invalid as c_int == true_0
        {
            apply_autocmds(
                EVENT_BUFMODIFIEDSET,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
            (*curbuf.get()).b_changed_invalid = false;
        }
    }
}

fn normal_check_safe_state() {
    // SAFETY: fires SafeState autocommands.
    unsafe { may_trigger_safestate(!op_pending() && restart_edit.get() == 0) };
}

fn normal_check_folds() {
    // SAFETY: reads and adjusts the current window's folds.
    unsafe {
        foldAdjustVisual();
        if hasAnyFolding(curwin.get()) != 0 && !char_avail() {
            foldCheckClose();
            if fdo_flags.get() & kOptFdoFlagAll as c_int as c_uint != 0 {
                foldOpenCursor();
            }
        }
    }
}

/// The idle redraw: scroll the cursor into view, update the screen, and put
/// back the message the last command left to be shown.
fn normal_redraw() {
    // SAFETY: all of this is the current window's and buffer's own state.
    unsafe {
        update_topline(curwin.get());
        validate_cursor(curwin.get());
        show_cursor_info_later(false);
        if must_redraw.get() != 0 {
            update_screen();
        } else {
            redraw_statuslines();
            if redraw_cmdline.get() || clear_cmdline.get() || redraw_mode.get() {
                showmode();
            }
        }
        (*curbuf.get()).b_last_used = time(ptr::null_mut());
        if !(*keep_msg.ptr()).is_null() {
            // `msg` may free the global, so it is handed a copy -- and the
            // message is not added to the history a second time.
            let copy = xstrdup(keep_msg.get());
            msg_hist_off.set(true);
            msg(copy, keep_msg_hl_id.get());
            msg_hist_off.set(false);
            xfree(copy.cast::<c_void>());
        }
        if need_fileinfo.get() && !shortmess(SHM_FILEINFO as c_int) {
            fileinfo(false_0, true_0, false);
            need_fileinfo.set(false);
        }
        emsg_on_display.set(false);
        did_emsg.set(false_0);
        msg_didany.set(false);
        may_clear_sb_text();
        setcursor();
    }
}

/// One iteration of the state loop's check half.
///
/// Answers 1 to go on and read a command, 0 to leave normal mode, and -1 to
/// leave it because Ex mode ran instead.
///
/// Kept `extern "C"`: it is installed as a `state_check_callback` and
/// `state_enter` calls it through that pointer.
pub(crate) unsafe extern "C" fn normal_check(state: *mut VimState) -> c_int {
    // SAFETY: `state` is the `VimState` at the head of our own `NormalState`,
    // which is what we handed to `state_enter`.
    unsafe {
        let s = state as *mut NormalState;
        normal_check_stuff_buffer();
        normal_check_interrupt(s);
        if did_throw.get() && ex_normal_busy.get() == 0 {
            discard_current_exception();
        }
        if !exmode_active.get() {
            msg_scroll.set(false_0);
        }
        quit_more.set(false);
        state_no_longer_safe(ptr::null());

        if skip_redraw.get() || exmode_active.get() {
            skip_redraw.set(false);
            setcursor();
        } else if do_redraw.get() || stuff_empty() {
            terminal_check_refresh();
            update_topline(curwin.get());
            validate_cursor(curwin.get());
            normal_check_cursor_moved();
            normal_check_text_changed();
            normal_check_window_scrolled();
            normal_check_buffer_modified();
            normal_check_safe_state();
            if (*curtab.get()).tp_diff_update != 0 || (*curtab.get()).tp_diff_invalid != 0 {
                ex_diffupdate(ptr::null_mut());
                (*curtab.get()).tp_diff_update = false_0;
            }
            if diff_need_scrollbind.get() {
                check_scrollbind(0, 0);
                diff_need_scrollbind.set(false);
            }
            normal_check_folds();
            normal_redraw();
            do_redraw.set(false);
            // The first screen update is the end of startup profiling.
            if !(*time_fd.ptr()).is_null() {
                time_msg(c"first screen update".as_ptr(), ptr::null());
                time_finish();
            }
            may_make_initial_scroll_size_snapshot();
        }

        // Collecting is only safe where no caller up the stack is holding a
        // value: the command-line window and Ex mode both are.
        may_garbage_collect.set(!(*s).cmdwin && !(*s).noexmode);
        update_curswant();

        if exmode_active.get() {
            if (*s).noexmode {
                return 0;
            }
            do_exmode();
            return -1;
        }
        if (*s).cmdwin && cmdwin_result.get() != 0 {
            return 0;
        }
        normal_prepare(s);
        1
    }
}

/// Publish the count the command was given as `v:count` and `v:count1`.
///
/// An operator's count and the motion's multiply; a zero count reports as 1
/// in `v:count1` and as itself in `v:count`.
pub(crate) unsafe fn set_vcount_ca(cap: *mut cmdarg_T, set_prevcount: &mut bool) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        let mut count = (*cap).count0 as int64_t;
        if (*cap).opcount != 0 {
            count = (*cap).opcount as int64_t * if count == 0 { 1 } else { count };
        }
        set_vcount(count, if count == 0 { 1 } else { count }, *set_prevcount);
    }
    *set_prevcount = false;
}

/// Run exactly one normal-mode command, from an operator the caller owns.
///
/// This is what `:normal` and the operator-pending machinery re-enter through.
pub(crate) unsafe fn normal_cmd(oap: *mut oparg_T, toplevel: bool) {
    let mut s = new_state();
    s.toplevel = toplevel;
    // SAFETY: `oap` is the caller's live operator, and `s` outlives the call.
    unsafe {
        s.oa = *oap;
        normal_prepare(&raw mut s);
        normal_execute(&raw mut s.state, safe_vgetc());
        *oap = s.oa;
    }
}
