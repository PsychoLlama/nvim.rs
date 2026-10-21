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

#![forbid(unsafe_code)]

use crate::global_cell::GlobalCell;
use crate::id_table::{IdTable, TableId};
use crate::keycodes::ModMask;
use crate::ops::Op;
use crate::types::AutoEvent;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};

use crate::autocmd::state::{last_cursormoved, last_cursormoved_win};
use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::{buf_get_changedtick, fileinfo};
use crate::diff::diff_update;
use crate::diff::state::diff_need_scrollbind;
use crate::drawscreen::state::{
    clear_cmdline, do_redraw, must_redraw, redraw_cmdline, redraw_mode, skip_redraw,
};
use crate::drawscreen::{
    UPD_INVERTED, redraw_curbuf_later, redraw_statuslines, setcursor, show_cursor_info_later,
    showmode, update_screen,
};
use crate::eval::gc::may_garbage_collect;
use crate::eval::vars::set_vcount;
use crate::ex_docmd::do_exmode;
use crate::ex_docmd::state::{ex_normal_busy, global_busy};
use crate::ex_eval::discard_current_exception;
use crate::ex_eval::state::did_throw;
use crate::ex_getln::{curbuf_locked, text_locked, text_locked_msg};
use crate::fileio::check_timestamps;
use crate::fileio::state::{did_check_timestamps, need_check_timestamps};
use crate::fold::{fold_adjust_visual, fold_check_close, fold_open_cursor, has_any_folding};
use crate::getchar::state::{KeyTyped, got_int, mod_mask, reg_executing, reg_recording};
use crate::getchar::{char_avail, readbuf1_empty, safe_vgetc, stuff_empty, typeahead, vgetc};
use crate::message::state::{
    did_emsg, did_wait_return, emsg_on_display, emsg_silent, in_assert_fails, keep_msg,
    keep_msg_hl_id, msg_didany, msg_didout, msg_hist_off, msg_nowait, msg_scroll, msg_silent,
    need_fileinfo, need_wait_return, quit_more,
};
use crate::message::{has_keep_msg, may_clear_sb_text, msg, msg_delay, wait_return};
use crate::normal::{
    NormalState, NvFlags, check_scrollbind, clear_op, clear_op_beep, end_visual_mode, find_command,
    normal_execute, nv_cmds, unshift_special, visual_active,
};
use crate::option::shortmess;
use crate::option::vars::{fdo_flags, p_smd};
use crate::options::kOptFdoFlagAll;
use crate::os::time::os_time_raw;
use crate::pos::equalpos;
use crate::profile::startup_timing;
use crate::profile::{time_finish, time_msg_at};
use crate::state::mode::{
    State, exmode_active, finish_op, km_startsel, km_stopsel, opcount, restart_edit,
};
use crate::state::{
    MODE_INSERT, MODE_NORMAL, MODE_NORMAL_BUSY, ModeState, may_trigger_modechanged,
    may_trigger_safestate, state_enter, state_no_longer_safe,
};
use crate::terminal::terminal_check_refresh;
use crate::types::{CmdArg, NUL, OpArg, OpType, Outcome, Pos, ShmFlag, int64_t};
use crate::ui::{ui_cursor_shape, ui_flush};
use crate::window::{may_make_initial_scroll_size_snapshot, may_trigger_win_scrolled_resized};
use crate::winlayer::graph::cmdwin_result;
use core::ffi::{c_int, c_uint};

use crate::r#move::{update_curswant, update_topline, validate_cursor};

/// Every running normal-mode frame's operator, owned at a fixed address,
/// and the innermost `normal_enter`'s: upstream's `current_oap`, which
/// [`op_pending`] reads from inside whatever the running command re-entered.
///
/// The operators live here rather than in their [`NormalState`]s because
/// the state is lent to the handler mutably for the whole command, and
/// `op_pending` must still be able to read the operator meanwhile: a handle
/// ([`Op`]) to a table slot is read one field access at a time, as every
/// other handle in the tree is.
struct NormalFrames {
    table: IdTable<OpArg>,
    current: Option<TableId<OpArg>>,
}

static NORMAL_FRAMES: GlobalCell<NormalFrames> = GlobalCell::new(NormalFrames {
    table: IdTable::new(),
    current: None,
});

/// File `op` in the frame table, answering its slot and a handle to it.
///
/// Private, non-generic and out of line: the table's cell must not be
/// reached from a generic or inlined front (`just state-exports`).
#[inline(never)]
fn frame_insert(op: OpArg) -> (TableId<OpArg>, Op) {
    NORMAL_FRAMES.with_mut(|frames| {
        let (id, _) = frames.table.insert((), op);
        (id, frames.table.view(id))
    })
}

/// Empty `id`'s slot, answering the operator it held.
#[inline(never)]
fn frame_remove(id: TableId<OpArg>) -> OpArg {
    NORMAL_FRAMES.with_mut(|frames| frames.table.remove(id))
}

/// Empty `id`'s slot.
#[inline(never)]
fn frame_free(id: TableId<OpArg>) {
    NORMAL_FRAMES.with_mut(|frames| frames.table.free(id));
}

/// Make `id` the innermost frame, answering the one it replaces.
#[inline(never)]
fn frame_make_current(id: Option<TableId<OpArg>>) -> Option<TableId<OpArg>> {
    NORMAL_FRAMES.with_mut(|frames| core::mem::replace(&mut frames.current, id))
}

/// The innermost `normal_enter`'s operator, if one is running.
#[inline(never)]
fn innermost_op() -> Option<Op> {
    NORMAL_FRAMES.with(|frames| frames.current.map(|id| frames.table.view(id)))
}

/// Run `f` on an operator of its own, starting as `op`, filed in the frame
/// table for the call -- for a command built outside a normal-mode frame
/// (`:normal`, a mouse scroll, `CTRL-W ]`). Answers what `f` answered and
/// the operator as `f` left it. Not the innermost frame: `op_pending` keeps
/// reading the one that was.
pub(crate) fn with_frame_op<R>(op: OpArg, f: impl FnOnce(Op) -> R) -> (R, OpArg) {
    let (id, handle) = frame_insert(op);
    let result = f(handle);
    (result, frame_remove(id))
}

/// Run `f` on a fresh command pending on an operator of its own; see
/// [`with_frame_op`].
pub(crate) fn with_scratch_cmd<R>(f: impl FnOnce(&mut CmdArg) -> R) -> R {
    with_frame_op(OpArg::ZERO, |op| f(&mut CmdArg::new(op))).0
}

impl CmdArg {
    /// The operator this command is pending on.
    #[inline(always)]
    pub(crate) fn op(&self) -> Op {
        self.oap
    }
}

/// A fresh state whose commands are pending on `oa`.
fn new_state(oa: Op) -> NormalState {
    NormalState {
        command_finished: false,
        ctrl_w: false,
        need_flushbuf: false,
        set_prevcount: false,
        previous_got_int: false,
        cmdwin: false,
        noexmode: false,
        toplevel: false,
        oa,
        ca: CmdArg::new(oa),
        mapped_len: 0,
        old_mapped_len: 0,
        idx: 0,
        c: 0,
        old_col: 0,
        old_pos: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
    }
}

impl NormalState {
    /// The operator the frame carries, which is the one `op_pending` reads.
    #[inline(always)]
    pub(crate) fn op(&self) -> Op {
        self.oa
    }
}

/// Refuse a command that would change text while the text is locked.
///
/// Beeps and clears the pending operator when there is one to clear.
pub(crate) fn check_text_locked(op: Option<Op>) -> bool {
    if !text_locked() {
        return false;
    }
    if let Some(op) = op {
        clear_op_beep(op);
    }
    text_locked_msg();
    true
}

/// As [`check_text_locked`], and also refuse while the current buffer is
/// locked. A locked buffer clears the operator without a beep.
pub(crate) fn check_text_or_curbuf_locked(op: Option<Op>) -> bool {
    if check_text_locked(op) {
        return true;
    }
    if !curbuf_locked() {
        return false;
    }
    if let Some(op) = op {
        clear_op(op);
    }
    true
}

/// Whether a command is half-typed: an operator waiting for its motion, a
/// count or a register already given.
///
/// Reads the operator the innermost `normal_enter`/`normal_cmd` installed.
pub(crate) fn op_pending() -> bool {
    let Some(op) = innermost_op() else {
        return true;
    };
    let (prev_opcount, prev_count0, op_type, regname) =
        (op.prev_opcount, op.prev_count0, op.op_type, op.regname);
    !(!finish_op.get()
        && prev_opcount == 0
        && prev_count0 == 0
        && op_type == OpType::Nop
        && regname == NUL)
}

/// Run normal mode until something asks to leave it.
///
/// `cmdwin` means this is the command-line window's own normal mode and
/// `noexmode` that Ex mode must not be entered from it; only a normal mode
/// that is neither is "toplevel", which is what decides whether `v:count` is
/// published.
pub(crate) fn normal_enter(cmdwin: bool, noexmode: bool) {
    let (id, op) = frame_insert(OpArg::ZERO);
    let mut state = new_state(op);
    state.cmdwin = cmdwin;
    state.noexmode = noexmode;
    state.toplevel = (!cmdwin || cmdwin_result.get() == 0) && !noexmode;
    // The innermost operator is the one `op_pending` reports on; the outer
    // one is put back on the way out.
    let outer = frame_make_current(Some(id));
    state_enter(ModeState::normal(&mut state));
    frame_make_current(outer);
    frame_free(id);
}

/// Set up `s.ca` for the command about to be read.
///
/// `inline`: called once per key from `normal_cmd`, and out of line the pair
/// cost inbench `typeahead` 3-6 % of its cycles at flat instructions.
#[inline]
pub(crate) fn normal_prepare(ns: &mut NormalState) {
    ns.ca.reset();
    ns.ca.opcount = opcount.get();

    // 'finish_op' drives the cursor shape, so a change to it is a redraw.
    let was_finishing = finish_op.get();
    finish_op.set(ns.oa.op_type != OpType::Nop);
    if finish_op.get() != was_finishing {
        ui_cursor_shape();
    }
    may_trigger_modechanged();

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
        set_vcount_ca(&ns.ca, ns.set_prevcount);
        ns.set_prevcount = false;
    }
}

/// Apply 'keymodel' to the command just looked up.
///
/// Answers whether the command was rejected outright, which happens when
/// unshifting a special key leaves a character no table row claims.
pub(crate) fn normal_handle_special_visual_command(ns: &mut NormalState) -> bool {
    let flags = nv_cmds[ns.idx as usize].cmd_flags;
    // "stopsel": an unshifted movement ends the selection.
    if km_stopsel.get() && flags.has(NvFlags::STS) && !mod_mask.get().has(ModMask::SHIFT) {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED);
    }
    if km_startsel.get() {
        if flags.has(NvFlags::SS) {
            // A shifted special key becomes its unshifted self, and the
            // table has to be consulted again for the new character.
            unshift_special(&mut ns.ca);
            ns.idx = find_command(ns.ca.cmdchar);
            if ns.idx < 0 {
                clear_op_beep(ns.op());
                return true;
            }
        } else if flags.has(NvFlags::SSS) && mod_mask.get().has(ModMask::SHIFT) {
            mod_mask.set(mod_mask.get().without(ModMask::SHIFT));
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
pub(crate) fn normal_need_additional_char(ns: &mut NormalState) -> bool {
    let flags = nv_cmds[ns.idx as usize].cmd_flags;
    let pending_op = ns.oa.op_type != OpType::Nop;
    let cmdchar = ns.ca.cmdchar;
    flags.has(NvFlags::NCH)
        && (flags.has_all(NvFlags::NCH_NOP) && !pending_op
            || flags.has_all(NvFlags::NCH_ALW)
            || cmdchar == 'q' as c_int
                && !pending_op
                && reg_recording.get() == 0
                && reg_executing.get() == 0
            || (cmdchar == 'a' as c_int || cmdchar == 'i' as c_int)
                && (pending_op || visual_active()))
}

/// Whether the mode message the last command scrolled away has to be put
/// back before the next key is read.
pub(crate) fn normal_need_redraw_mode_message(ns: &mut NormalState) -> bool {
    let showing_mode = p_smd()
        && msg_silent.get() == 0
        && (restart_edit.get() != 0
            || visual_active()
                && ns.old_pos.lnum == Win::current().w_cursor.lnum
                && ns.old_pos.col == Win::current().w_cursor.col)
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
        && !ns.ca.outcome.has(Outcome::COMMAND_BUSY)
        && stuff_empty()
        && typeahead().maplen() == 0
        && emsg_silent.get() == 0
        && !in_assert_fails.get()
        && !did_wait_return.get()
        && ns.oa.op_type == OpType::Nop
}

/// Redraw the screen the message was scrolled off, show the message again,
/// and pause long enough for it to be read.
pub(crate) fn normal_redraw_mode_message() {
    let save_state = State.get();
    if restart_edit.get() != 0 {
        State.set(MODE_INSERT);
    }
    if must_redraw.get() != 0 && has_keep_msg() && !emsg_on_display.get() {
        // The redraw must not print the kept message itself, so it is
        // taken out for the duration and put back after.
        let kmsg = keep_msg.take();
        setcursor();
        let _ = update_screen();
        keep_msg.set(kmsg);
        // A copy: `msg` replaces the kept message.
        if let Some(copy) = keep_msg.with(Clone::clone) {
            msg(copy.as_cstr(), keep_msg_hl_id.get());
        }
    }
    setcursor();
    ui_cursor_shape();
    ui_flush();
    if msg_scroll.get() != 0 || emsg_on_display.get() {
        msg_delay(1003, true);
    }
    msg_delay(3003, false);
    State.set(save_state);
    msg_scroll.set(0);
    emsg_on_display.set(false);
}

/// File timestamps and a pending "Press ENTER", once the stuff buffer runs
/// dry.
fn normal_check_stuff_buffer() {
    if stuff_empty() {
        did_check_timestamps.set(false);
        if need_check_timestamps.get() {
            check_timestamps(0);
        }
        if need_wait_return.get() {
            wait_return(0);
        }
    }
}

/// Absorb an interrupt.
///
/// A second CTRL-C while `:global` is running and Ex mode was asked for is
/// what gets you into Ex mode; otherwise the interrupt is swallowed, along
/// with the key that caused it when the more-prompt is not up.
fn normal_check_interrupt(ns: &mut NormalState) {
    if !got_int.get() {
        ns.previous_got_int = false;
        return;
    }
    if ns.noexmode && global_busy.get() != 0 && !exmode_active.get() && ns.previous_got_int {
        exmode_active.set(true);
        State.set(MODE_NORMAL);
    } else if global_busy.get() == 0 || !exmode_active.get() {
        if !quit_more.get() {
            vgetc();
        }
        got_int.set(false);
    }
    ns.previous_got_int = true;
}

fn normal_check_window_scrolled() {
    if !finish_op.get() {
        may_trigger_win_scrolled_resized();
    }
}

fn normal_check_cursor_moved() {
    // SAFETY (throughout): reads the current window and fires an autocommand.
    if !finish_op.get()
        && has_event(AutoEvent::CursorMoved)
        && (last_cursormoved_win.get() != Win::current_or_none().map(Win::id)
            || !equalpos(last_cursormoved.get(), Win::current().w_cursor))
    {
        fire_on_curbuf(AutoEvent::CursorMoved);
        last_cursormoved_win.set(Win::current_or_none().map(Win::id));
        last_cursormoved.set(Win::current().w_cursor);
    }
}

fn normal_check_text_changed() {
    // SAFETY (throughout): reads the current buffer and fires an autocommand.
    if !finish_op.get()
        && has_event(AutoEvent::TextChanged)
        && Buf::current().b_last_changedtick != buf_get_changedtick(Buf::current())
    {
        fire_on_curbuf(AutoEvent::TextChanged);
        Buf::current().b_last_changedtick = buf_get_changedtick(Buf::current());
    }
}

fn normal_check_buffer_modified() {
    // SAFETY (throughout): reads the current buffer and fires an autocommand.
    if !finish_op.get()
        && has_event(AutoEvent::BufModifiedSet)
        && Buf::current().b_changed_invalid as c_int == 1
    {
        fire_on_curbuf(AutoEvent::BufModifiedSet);
        Buf::current().b_changed_invalid = false;
    }
}

fn normal_check_safe_state() {
    may_trigger_safestate(!op_pending() && restart_edit.get() == 0);
}

fn normal_check_folds() {
    fold_adjust_visual();
    if has_any_folding(Win::current()) != 0 && !char_avail() {
        fold_check_close();
        if fdo_flags.get() & kOptFdoFlagAll as c_int as c_uint != 0 {
            fold_open_cursor();
        }
    }
}

/// The idle redraw: scroll the cursor into view, update the screen, and put
/// back the message the last command left to be shown.
fn normal_redraw() {
    // SAFETY (throughout): all of this is the current window's and buffer's own state.
    update_topline(Win::current());
    validate_cursor(Win::current());
    show_cursor_info_later(false);
    if must_redraw.get() != 0 {
        let _ = update_screen();
    } else {
        redraw_statuslines();
        if redraw_cmdline.get() || clear_cmdline.get() || redraw_mode.get() {
            showmode();
        }
    }
    Buf::current().b_last_used = os_time_raw();
    // `msg` replaces the kept message, so it is handed a copy -- and the
    // message is not added to the history a second time.
    if let Some(copy) = keep_msg.with(Clone::clone) {
        msg_hist_off.set(true);
        msg(copy.as_cstr(), keep_msg_hl_id.get());
        msg_hist_off.set(false);
    }
    if need_fileinfo.get() && !shortmess(ShmFlag::FILEINFO) {
        fileinfo(0, 1, false);
        need_fileinfo.set(false);
    }
    emsg_on_display.set(false);
    did_emsg.set(0);
    msg_didany.set(false);
    may_clear_sb_text();
    setcursor();
}

/// One iteration of the state loop's check half.
///
/// Answers 1 to go on and read a command, 0 to leave normal mode, and -1 to
/// leave it because Ex mode ran instead.
pub(crate) fn normal_check(ns: &mut NormalState) -> c_int {
    normal_check_stuff_buffer();
    normal_check_interrupt(ns);
    if did_throw.get() && ex_normal_busy.get() == 0 {
        discard_current_exception();
    }
    if !exmode_active.get() {
        msg_scroll.set(0);
    }
    quit_more.set(false);
    state_no_longer_safe(None);

    if skip_redraw.get() || exmode_active.get() {
        skip_redraw.set(false);
        setcursor();
    } else if do_redraw.get() || stuff_empty() {
        terminal_check_refresh();
        update_topline(Win::current());
        validate_cursor(Win::current());
        normal_check_cursor_moved();
        normal_check_text_changed();
        normal_check_window_scrolled();
        normal_check_buffer_modified();
        normal_check_safe_state();
        if TabPage::current().tp_diff_update != 0 || TabPage::current().tp_diff_invalid != 0 {
            diff_update(None);
            TabPage::current().tp_diff_update = 0;
        }
        if diff_need_scrollbind.get() {
            check_scrollbind(0, 0);
            diff_need_scrollbind.set(false);
        }
        normal_check_folds();
        normal_redraw();
        do_redraw.set(false);
        // The first screen update is the end of startup profiling.
        if startup_timing() {
            time_msg_at(c"first screen update");
            time_finish();
        }
        may_make_initial_scroll_size_snapshot();
    }

    // Collecting is only safe where no caller up the stack is holding a
    // value: the command-line window and Ex mode both are.
    may_garbage_collect.set(!ns.cmdwin && !ns.noexmode);
    update_curswant();

    if exmode_active.get() {
        if ns.noexmode {
            return 0;
        }
        do_exmode();
        return -1;
    }
    if ns.cmdwin && cmdwin_result.get() != 0 {
        return 0;
    }
    normal_prepare(ns);
    1
}

/// Publish the count the command was given as `v:count` and `v:count1`.
///
/// An operator's count and the motion's multiply; a zero count reports as 1
/// in `v:count1` and as itself in `v:count`.
///
/// `set_prevcount` publishes `v:prevcount` too, which only the first
/// publication of a command does; the caller clears its own flag afterwards,
/// because the flag and the command are sibling fields of one frame and a
/// callee cannot borrow both.
pub(crate) fn set_vcount_ca(cmd_arg: &CmdArg, set_prevcount: bool) {
    let mut count = cmd_arg.count0 as int64_t;
    if cmd_arg.opcount != 0 {
        count = cmd_arg.opcount as int64_t * if count == 0 { 1 } else { count };
    }
    set_vcount(count, if count == 0 { 1 } else { count }, set_prevcount);
}

/// Run exactly one normal-mode command, pending on `op` -- an operator the
/// caller filed for its whole run of commands ([`with_frame_op`]), which
/// carries what one command leaves pending into the next.
///
/// This is what `:normal` and the operator-pending machinery re-enter through.
#[inline]
pub(crate) fn normal_cmd(op: Op, toplevel: bool) {
    let mut s = new_state(op);
    s.toplevel = toplevel;
    normal_prepare(&mut s);
    normal_execute(&mut s, safe_vgetc());
}

/// Fire `event` on the current buffer, with no file name to match against.
fn fire_on_curbuf(event: AutoEvent) {
    apply_autocmds(event, None, None, false, Buf::current_or_none());
}
