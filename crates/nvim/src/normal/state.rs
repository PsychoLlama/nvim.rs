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

use crate::ops::Op;
use crate::winlayer::{Buf, Win};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::autocmd::{
    EVENT_BUFMODIFIEDSET, EVENT_CURSORMOVED, EVENT_TEXTCHANGED, apply_autocmds, has_event,
};
use crate::buffer::{buf_get_changedtick, fileinfo};
use crate::diff::ex_diffupdate;
use crate::drawscreen::{
    UPD_INVERTED, redraw_curbuf_later, redraw_statuslines, setcursor, show_cursor_info_later,
    showmode, update_screen,
};
use crate::eval::vars::set_vcount;
use crate::ex_docmd::do_exmode;
use crate::ex_eval::discard_current_exception;
use crate::ex_getln::{curbuf_locked, text_locked, text_locked_msg};
use crate::fileio::check_timestamps;
use crate::fold::{fold_adjust_visual, fold_check_close, fold_open_cursor, has_any_folding};
use crate::getchar::{char_avail, readbuf1_empty, safe_vgetc, stuff_empty, typeahead, vgetc};
use crate::main::{
    KeyTyped, State, clear_cmdline, cmdwin_result, curbuf, curtab, curwin, did_check_timestamps,
    did_emsg, did_throw, did_wait_return, diff_need_scrollbind, do_redraw, emsg_on_display,
    emsg_silent, ex_normal_busy, exmode_active, fdo_flags, finish_op, global_busy, got_int,
    in_assert_fails, keep_msg, keep_msg_hl_id, km_startsel, km_stopsel, last_cursormoved,
    last_cursormoved_win, may_garbage_collect, mod_mask, msg_didany, msg_didout, msg_hist_off,
    msg_nowait, msg_scroll, msg_silent, must_redraw, need_check_timestamps, need_fileinfo,
    need_wait_return, opcount, p_smd, quit_more, redraw_cmdline, redraw_mode, reg_executing,
    reg_recording, restart_edit, skip_redraw, time_fd,
};
use crate::memory::{xfree, xstrdup};
use crate::message::{may_clear_sb_text, msg, msg_delay, wait_return};
use crate::normal::{
    CA_COMMAND_BUSY, MOD_MASK_SHIFT, NV_NCH, NV_NCH_ALW, NV_NCH_NOP, NV_SS, NV_SSS, NV_STS,
    NormalState, check_scrollbind, clear_op, clear_op_beep, clearopbeep, current_oap,
    end_visual_mode, find_command, normal_execute, nv_cmds, unshift_special, visual_active,
};
use crate::option::shortmess;
use crate::options::kOptFdoFlagAll;
use crate::pos::equalpos;
use crate::profile::{time_finish, time_msg};
use crate::state::{
    MODE_INSERT, MODE_NORMAL, MODE_NORMAL_BUSY, may_trigger_modechanged, may_trigger_safestate,
    state_enter, state_no_longer_safe,
};
use crate::terminal::terminal_check_refresh;
use crate::types::{NUL, OP_NOP, ShmFlag, VimState, cmdarg_T, event_T, int64_t, oparg_T};
use crate::ui::{ui_cursor_shape, ui_flush};
use crate::window::{may_make_initial_scroll_size_snapshot, may_trigger_win_scrolled_resized};
use ::libc::time;
use core::ffi::{c_int, c_uint, c_void};

use crate::r#move::{update_curswant, update_topline, validate_cursor};

/// One pass of the normal-mode loop, which the caller has promised is live.
/// [`CmdArg`]'s shape.
///
/// The state has to be reached through a pointer rather than a `&mut`: the
/// loop publishes the address of its own `oa` in `current_oap`, so a `&mut`
/// spanning a handler would alias what [`op_pending`] reads.
#[derive(Clone, Copy)]
pub(crate) struct NormalStateRef(*mut NormalState);

impl NormalStateRef {
    /// # Safety
    /// `s` must stay a live `NormalState` for as long as the value is used.
    pub(crate) const unsafe fn new(s: *mut NormalState) -> Self {
        Self(s)
    }
    /// The pointer, for a callee that still takes one.
    pub(crate) fn raw(self) -> *mut NormalState {
        self.0
    }
}

impl Deref for NormalStateRef {
    type Target = NormalState;
    fn deref(&self) -> &NormalState {
        // SAFETY: the constructor's promise -- a live state.
        unsafe { &*self.0 }
    }
}

impl DerefMut for NormalStateRef {
    fn deref_mut(&mut self) -> &mut NormalState {
        // SAFETY: as `deref`; the borrow lasts only as long as the field
        // access that asked for it.
        unsafe { &mut *self.0 }
    }
}

/// The command argument a normal-mode handler is running, which the caller
/// has promised is live.
///
/// One pointer, and building one reads nothing: [`crate::winlayer::Win`]'s
/// shape, for the two structures every `nv_*` handler is threaded through.
/// Field access goes through `Deref`, so it costs no `unsafe` at the site;
/// the operator it is pending on is [`crate::ops::Op`], the same shape.
#[derive(Clone, Copy)]
pub(crate) struct CmdArg(*mut cmdarg_T);

impl CmdArg {
    /// # Safety
    /// `cap` must stay a live command argument for as long as the value is
    /// used.
    pub(crate) const unsafe fn new(cap: *mut cmdarg_T) -> Self {
        Self(cap)
    }
    /// The operator this command is pending on.
    pub(crate) fn op(self) -> Op {
        // SAFETY: a live command argument's operator is live.
        unsafe { Op::new(self.oap) }
    }
}

impl Deref for CmdArg {
    type Target = cmdarg_T;
    fn deref(&self) -> &cmdarg_T {
        // SAFETY: the constructor's promise -- a live command argument.
        unsafe { &*self.0 }
    }
}

impl DerefMut for CmdArg {
    fn deref_mut(&mut self) -> &mut cmdarg_T {
        // SAFETY: as `deref`; the borrow lasts only as long as the field
        // access that asked for it.
        unsafe { &mut *self.0 }
    }
}

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
    // SAFETY (throughout): `oap` is null or the caller's operator.
    if !unsafe { text_locked() } {
        return false;
    }
    if !oap.is_null() {
        // SAFETY: past the null check, `oap` is the caller's live operator.
        clear_op_beep(unsafe { Op::new(oap) });
    }
    unsafe { text_locked_msg() };
    true
}

/// As [`check_text_locked`], and also refuse while the current buffer is
/// locked. A locked buffer clears the operator without a beep.
pub(crate) unsafe fn check_text_or_curbuf_locked(oap: *mut oparg_T) -> bool {
    // SAFETY (throughout): `oap` is null or the caller's operator.
    if unsafe { check_text_locked(oap) } {
        return true;
    }
    if !unsafe { curbuf_locked() } {
        return false;
    }
    if !oap.is_null() {
        // SAFETY: past the null check, `oap` is the caller's live operator.
        clear_op(unsafe { Op::new(oap) });
    }
    true
}

/// Whether a command is half-typed: an operator waiting for its motion, a
/// count or a register already given.
///
/// Reads the operator the innermost `normal_enter`/`normal_cmd` installed.
pub(crate) fn op_pending() -> bool {
    let oap = current_oap.get();
    // SAFETY: `current_oap` is null or points at a live caller's `oparg_T`,
    // and the `&&` chain only reaches the reads past the null check.
    !(!oap.is_null()
        && !finish_op.get()
        && unsafe { (*oap).prev_opcount } == 0
        && unsafe { (*oap).prev_count0 } == 0
        && unsafe { (*oap).op_type } == OP_NOP
        && unsafe { (*oap).regname } == NUL)
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
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    ns.ca = unsafe { core::mem::zeroed() };
    ns.ca.oap = &raw mut ns.oa;
    ns.ca.opcount = opcount.get();

    // 'finish_op' drives the cursor shape, so a change to it is a redraw.
    let was_finishing = finish_op.get();
    finish_op.set(ns.oa.op_type != OP_NOP);
    if finish_op.get() != was_finishing {
        unsafe { ui_cursor_shape() };
    }
    unsafe { may_trigger_modechanged() };

    // With no operator and no register pending, the count starts over --
    // and `set_prevcount` remembers to publish it as v:prevcount.
    ns.set_prevcount = false;
    if !finish_op.get() && ns.oa.regname == 0 {
        ns.ca.opcount = 0;
        ns.set_prevcount = true;
    }
    // A count the previous command stashed comes back here.
    if ns.oa.prev_opcount > 0 || ns.oa.prev_count0 > 0 {
        ns.ca.opcount = ns.oa.prev_opcount;
        ns.ca.count0 = ns.oa.prev_count0;
        ns.oa.prev_opcount = 0;
        ns.oa.prev_count0 = 0;
    }

    ns.mapped_len = typeahead().maplen();
    State.set(MODE_NORMAL_BUSY);
    if ns.toplevel && readbuf1_empty() {
        unsafe { set_vcount_ca(&raw mut ns.ca, &mut ns.set_prevcount) };
    }
}

/// Apply 'keymodel' to the command just looked up.
///
/// Answers whether the command was rejected outright, which happens when
/// unshifting a special key leaves a character no table row claims.
pub(crate) unsafe fn normal_handle_special_visual_command(s: *mut NormalState) -> bool {
    // SAFETY (throughout): `s` is the caller's live state and `s.idx` is a valid row.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let flags = nv_cmds[ns.idx as usize].cmd_flags as c_int;
    // "stopsel": an unshifted movement ends the selection.
    if km_stopsel.get() && flags & NV_STS != 0 && mod_mask.get() & MOD_MASK_SHIFT == 0 {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED);
    }
    if km_startsel.get() {
        if flags & NV_SS != 0 {
            // A shifted special key becomes its unshifted self, and the
            // table has to be consulted again for the new character.
            unsafe { unshift_special(&raw mut ns.ca) };
            ns.idx = find_command(ns.ca.cmdchar);
            if ns.idx < 0 {
                unsafe { clearopbeep(&raw mut ns.oa) };
                return true;
            }
        } else if flags & NV_SSS != 0 && mod_mask.get() & MOD_MASK_SHIFT != 0 {
            mod_mask.set(mod_mask.get() & !MOD_MASK_SHIFT);
        }
    }
    false
}

/// Whether this command wants a second character read for it.
///
/// `NV_NCH_ALW` always does; `NV_NCH_NOP` only when no operator is pending.
/// `q`, `a` and `i` are spelled out because whether they take one depends on
/// state rather than on the row: `q` only starts a recording when none is
/// running, and `a`/`i` are text objects rather than insert commands only
/// while an operator or Visual mode is waiting for them.
pub(crate) unsafe fn normal_need_additional_char(s: *mut NormalState) -> bool {
    // SAFETY (throughout): `s` is the caller's live state and `s.idx` is a valid row.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let flags = nv_cmds[ns.idx as usize].cmd_flags as c_int;
    let pending_op = ns.oa.op_type != OP_NOP;
    let cmdchar = ns.ca.cmdchar;
    flags & NV_NCH != 0
        && (flags & NV_NCH_NOP == NV_NCH_NOP && !pending_op
            || flags & NV_NCH_ALW == NV_NCH_ALW
            || cmdchar == 'q' as c_int
                && !pending_op
                && reg_recording.get() == 0
                && reg_executing.get() == 0
            || (cmdchar == 'a' as c_int || cmdchar == 'i' as c_int)
                && (pending_op || visual_active()))
}

/// Whether the mode message the last command scrolled away has to be put
/// back before the next key is read.
pub(crate) unsafe fn normal_need_redraw_mode_message(s: *mut NormalState) -> bool {
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    let showing_mode = p_smd.get() != 0
        && msg_silent.get() == 0
        && (restart_edit.get() != 0
            || visual_active()
                && ns.old_pos.lnum == cur_win().w_cursor.lnum
                && ns.old_pos.col == cur_win().w_cursor.col)
        && (clear_cmdline.get() || redraw_cmdline.get())
        && (msg_didout.get() || msg_didany.get() && msg_scroll.get() != 0)
        && !msg_nowait.get()
        && KeyTyped.get();
    // The other way in: an error is on display and insert mode is
    // pending, with no Visual selection to describe instead.
    let error_on_display = restart_edit.get() != 0
        && !visual_active()
        && msg_scroll.get() != 0
        && emsg_on_display.get();

    (showing_mode || error_on_display)
        && ns.oa.regname == 0
        && ns.ca.retval & CA_COMMAND_BUSY as c_int == 0
        && stuff_empty()
        && typeahead().maplen() == 0
        && emsg_silent.get() == 0
        && !in_assert_fails.get()
        && !did_wait_return.get()
        && ns.oa.op_type == OP_NOP
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
    if must_redraw.get() != 0 && !keep_msg.get().is_null() && !emsg_on_display.get() {
        // The redraw must not print the kept message itself, so it is
        // taken out of the global for the duration and put back after.
        let kmsg = keep_msg.get();
        keep_msg.set(ptr::null_mut());
        unsafe { setcursor() };
        unsafe { update_screen() };
        keep_msg.set(kmsg);
        let copy = unsafe { xstrdup(keep_msg.get()) };
        unsafe { msg(copy, keep_msg_hl_id.get()) };
        unsafe { xfree(copy.cast::<c_void>()) };
    }
    unsafe { setcursor() };
    unsafe { ui_cursor_shape() };
    unsafe { ui_flush() };
    if msg_scroll.get() != 0 || emsg_on_display.get() {
        unsafe { msg_delay(1003, true) };
    }
    unsafe { msg_delay(3003, false) };
    State.set(save_state);
    msg_scroll.set(0);
    emsg_on_display.set(false);
}

/// File timestamps and a pending "Press ENTER", once the stuff buffer runs
/// dry.
fn normal_check_stuff_buffer() {
    // SAFETY (throughout): all three are global editor state.
    if stuff_empty() {
        did_check_timestamps.set(false);
        if need_check_timestamps.get() {
            unsafe { check_timestamps(0) };
        }
        if need_wait_return.get() {
            unsafe { wait_return(0) };
        }
    }
}

/// Absorb an interrupt.
///
/// A second CTRL-C while `:global` is running and Ex mode was asked for is
/// what gets you into Ex mode; otherwise the interrupt is swallowed, along
/// with the key that caused it when the more-prompt is not up.
unsafe fn normal_check_interrupt(s: *mut NormalState) {
    // SAFETY (throughout): `s` is the caller's live state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    if !got_int.get() {
        ns.previous_got_int = false;
        return;
    }
    if ns.noexmode && global_busy.get() != 0 && !exmode_active.get() && ns.previous_got_int {
        exmode_active.set(true);
        State.set(MODE_NORMAL);
    } else if global_busy.get() == 0 || !exmode_active.get() {
        if !quit_more.get() {
            unsafe { vgetc() };
        }
        got_int.set(false);
    }
    ns.previous_got_int = true;
}

fn normal_check_window_scrolled() {
    if !finish_op.get() {
        // SAFETY: fires autocommands for the current window.
        unsafe { may_trigger_win_scrolled_resized() };
    }
}

fn normal_check_cursor_moved() {
    // SAFETY (throughout): reads the current window and fires an autocommand.
    if !finish_op.get()
        && has_event(EVENT_CURSORMOVED)
        && (last_cursormoved_win.get() != curwin.get()
            || !equalpos(last_cursormoved.get(), cur_win().w_cursor))
    {
        fire_on_curbuf(EVENT_CURSORMOVED);
        last_cursormoved_win.set(curwin.get());
        last_cursormoved.set(cur_win().w_cursor);
    }
}

fn normal_check_text_changed() {
    // SAFETY (throughout): reads the current buffer and fires an autocommand.
    if !finish_op.get()
        && has_event(EVENT_TEXTCHANGED)
        && cur_buf().b_last_changedtick != unsafe { buf_get_changedtick(curbuf.get()) }
    {
        fire_on_curbuf(EVENT_TEXTCHANGED);
        cur_buf().b_last_changedtick = unsafe { buf_get_changedtick(curbuf.get()) };
    }
}

fn normal_check_buffer_modified() {
    // SAFETY (throughout): reads the current buffer and fires an autocommand.
    if !finish_op.get()
        && has_event(EVENT_BUFMODIFIEDSET)
        && cur_buf().b_changed_invalid as c_int == 1
    {
        fire_on_curbuf(EVENT_BUFMODIFIEDSET);
        cur_buf().b_changed_invalid = false;
    }
}

fn normal_check_safe_state() {
    // SAFETY: fires SafeState autocommands.
    unsafe { may_trigger_safestate(!op_pending() && restart_edit.get() == 0) };
}

fn normal_check_folds() {
    // SAFETY (throughout): reads and adjusts the current window's folds.
    unsafe { fold_adjust_visual() };
    if unsafe { has_any_folding(curwin.get()) } != 0 && !unsafe { char_avail() } {
        unsafe { fold_check_close() };
        if fdo_flags.get() & kOptFdoFlagAll as c_int as c_uint != 0 {
            unsafe { fold_open_cursor() };
        }
    }
}

/// The idle redraw: scroll the cursor into view, update the screen, and put
/// back the message the last command left to be shown.
fn normal_redraw() {
    // SAFETY (throughout): all of this is the current window's and buffer's own state.
    unsafe { update_topline(curwin.get()) };
    unsafe { validate_cursor(curwin.get()) };
    unsafe { show_cursor_info_later(false) };
    if must_redraw.get() != 0 {
        unsafe { update_screen() };
    } else {
        unsafe { redraw_statuslines() };
        if redraw_cmdline.get() || clear_cmdline.get() || redraw_mode.get() {
            unsafe { showmode() };
        }
    }
    cur_buf().b_last_used = unsafe { time(ptr::null_mut()) };
    if !keep_msg.get().is_null() {
        // `msg` may free the global, so it is handed a copy -- and the
        // message is not added to the history a second time.
        let copy = unsafe { xstrdup(keep_msg.get()) };
        msg_hist_off.set(true);
        unsafe { msg(copy, keep_msg_hl_id.get()) };
        msg_hist_off.set(false);
        unsafe { xfree(copy.cast::<c_void>()) };
    }
    if need_fileinfo.get() && !shortmess(ShmFlag::FILEINFO) {
        unsafe { fileinfo(0, 1, false) };
        need_fileinfo.set(false);
    }
    emsg_on_display.set(false);
    did_emsg.set(0);
    msg_didany.set(false);
    unsafe { may_clear_sb_text() };
    unsafe { setcursor() };
}

/// One iteration of the state loop's check half.
///
/// Answers 1 to go on and read a command, 0 to leave normal mode, and -1 to
/// leave it because Ex mode ran instead.
///
/// Keeps the raw signature: it is installed as a `state_check_callback` and
/// `state_enter` calls it through that pointer.
pub(crate) unsafe fn normal_check(state: *mut VimState) -> c_int {
    // SAFETY (throughout): `state` is the `VimState` at the head of our own `NormalState`,
    // which is what we handed to `state_enter`.
    let s = state as *mut NormalState;
    // SAFETY: `state` is the caller's live normal-mode state.
    let mut ns = unsafe { NormalStateRef::new(s) };
    normal_check_stuff_buffer();
    unsafe { normal_check_interrupt(ns.raw()) };
    if did_throw.get() && ex_normal_busy.get() == 0 {
        unsafe { discard_current_exception() };
    }
    if !exmode_active.get() {
        msg_scroll.set(0);
    }
    quit_more.set(false);
    unsafe { state_no_longer_safe(ptr::null()) };

    if skip_redraw.get() || exmode_active.get() {
        skip_redraw.set(false);
        unsafe { setcursor() };
    } else if do_redraw.get() || stuff_empty() {
        unsafe { terminal_check_refresh() };
        unsafe { update_topline(curwin.get()) };
        unsafe { validate_cursor(curwin.get()) };
        normal_check_cursor_moved();
        normal_check_text_changed();
        normal_check_window_scrolled();
        normal_check_buffer_modified();
        normal_check_safe_state();
        if unsafe { (*curtab.get()).tp_diff_update } != 0
            || unsafe { (*curtab.get()).tp_diff_invalid } != 0
        {
            unsafe { ex_diffupdate(ptr::null_mut()) };
            unsafe { (*curtab.get()).tp_diff_update = 0 };
        }
        if diff_need_scrollbind.get() {
            unsafe { check_scrollbind(0, 0) };
            diff_need_scrollbind.set(false);
        }
        normal_check_folds();
        normal_redraw();
        do_redraw.set(false);
        // The first screen update is the end of startup profiling.
        if !time_fd.get().is_null() {
            unsafe { time_msg(c"first screen update".as_ptr(), ptr::null()) };
            time_finish();
        }
        unsafe { may_make_initial_scroll_size_snapshot() };
    }

    // Collecting is only safe where no caller up the stack is holding a
    // value: the command-line window and Ex mode both are.
    may_garbage_collect.set(!ns.cmdwin && !ns.noexmode);
    unsafe { update_curswant() };

    if exmode_active.get() {
        if ns.noexmode {
            return 0;
        }
        unsafe { do_exmode() };
        return -1;
    }
    if ns.cmdwin && cmdwin_result.get() != 0 {
        return 0;
    }
    unsafe { normal_prepare(ns.raw()) };
    1
}

/// Publish the count the command was given as `v:count` and `v:count1`.
///
/// An operator's count and the motion's multiply; a zero count reports as 1
/// in `v:count1` and as itself in `v:count`.
pub(crate) unsafe fn set_vcount_ca(cap: *mut cmdarg_T, set_prevcount: &mut bool) {
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    let mut count = ca.count0 as int64_t;
    if ca.opcount != 0 {
        count = ca.opcount as int64_t * if count == 0 { 1 } else { count };
    }
    unsafe { set_vcount(count, if count == 0 { 1 } else { count }, *set_prevcount) };
    *set_prevcount = false;
}

/// Run exactly one normal-mode command, from an operator the caller owns.
///
/// This is what `:normal` and the operator-pending machinery re-enter through.
pub(crate) unsafe fn normal_cmd(oap: *mut oparg_T, toplevel: bool) {
    let mut s = new_state();
    s.toplevel = toplevel;
    // SAFETY: `oap` is the caller's live operator, and `s` outlives the call.
    s.oa = unsafe { *oap };
    unsafe { normal_prepare(&raw mut s) };
    unsafe { normal_execute(&raw mut s.state, safe_vgetc()) };
    unsafe { *oap = s.oa };
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

/// Fire `event` on the current buffer, with no file name to match against.
fn fire_on_curbuf(event: event_T) {
    let (fname, fname_io) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: `curbuf` is the live buffer the event is about.
    unsafe { apply_autocmds(event, fname, fname_io, false, curbuf.get()) };
}
