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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::autocmd::defs::AutoEvent;
use crate::global_cell::GlobalCell;
use crate::id_table::{IdTable, TableId};
use crate::keycodes::ModMask;
use crate::ops::Op;
use crate::state::Step;
use crate::types::ColNr;
use crate::types::Key;
use crate::types::LineNr;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};

use crate::autocmd::state::{LAST_CURSORMOVED, LAST_CURSORMOVED_WIN};
use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::{buf_get_changedtick, fileinfo};
use crate::diff::diff_update;
use crate::diff::state::DIFF_NEED_SCROLLBIND;
use crate::drawscreen::state::{
    CLEAR_CMDLINE, DO_REDRAW, MUST_REDRAW, REDRAW_CMDLINE, REDRAW_MODE, SKIP_REDRAW,
};
use crate::drawscreen::{
    UPD_INVERTED, redraw_curbuf_later, redraw_statuslines, setcursor, show_cursor_info_later,
    showmode, update_screen,
};
use crate::eval::gc::MAY_GARBAGE_COLLECT;
use crate::eval::vars::set_vcount;
use crate::ex_docmd::do_exmode;
use crate::ex_docmd::state::{EX_NORMAL_BUSY, GLOBAL_BUSY};
use crate::ex_eval::discard_current_exception;
use crate::ex_eval::state::DID_THROW;
use crate::ex_getln::{curbuf_locked, text_locked, text_locked_msg};
use crate::fileio::check_timestamps;
use crate::fileio::state::{DID_CHECK_TIMESTAMPS, NEED_CHECK_TIMESTAMPS};
use crate::fold::{fold_adjust_visual, fold_check_close, fold_open_cursor, has_any_folding};
use crate::getchar::state::{GOT_INT, KEY_TYPED, MOD_MASK, REG_EXECUTING, REG_RECORDING};
use crate::getchar::{char_avail, readbuf1_empty, safe_vgetc, stuff_empty, typeahead, vgetc};
use crate::message::state::{
    DID_EMSG, DID_WAIT_RETURN, EMSG_ON_DISPLAY, EMSG_SILENT, IN_ASSERT_FAILS, KEEP_MSG,
    KEEP_MSG_HL_ID, MSG_DIDANY, MSG_DIDOUT, MSG_HIST_OFF, MSG_NOWAIT, MSG_SCROLL, MSG_SILENT,
    NEED_FILEINFO, NEED_WAIT_RETURN, QUIT_MORE,
};
use crate::message::{PromptRedraw, has_keep_msg, may_clear_sb_text, msg, msg_delay, wait_return};
use crate::normal::defs::{CmdArg, OpArg, Outcome};
use crate::normal::{
    NV_CMD_TABLE, NormalState, NvCmd, NvFlags, check_scrollbind, clear_op, clear_op_beep,
    end_visual_mode, find_command, normal_execute, unshift_special, visual_active,
};
use crate::ops::OpType;
use crate::option::defs::ShmFlag;
use crate::option::shortmess;
use crate::option::vars::{FDO_FLAGS, p_smd};
use crate::options::OPT_FDO_FLAG_ALL;
use crate::os::libc::int64_t;
use crate::os::time::os_time_raw;
use crate::pos::equalpos;
use crate::profile::startup_timing;
use crate::profile::{time_finish, time_msg_at};
use crate::state::mode::{
    EXMODE_ACTIVE, FINISH_OP, KM_STARTSEL, KM_STOPSEL, OPCOUNT, RESTART_EDIT, STATE,
};
use crate::state::{
    MODE_INSERT, MODE_NORMAL, MODE_NORMAL_BUSY, ModeState, may_trigger_modechanged,
    may_trigger_safestate, state_enter, state_no_longer_safe,
};
use crate::terminal::terminal_check_refresh;
use crate::types::Pos;
use crate::ui::{ui_cursor_shape, ui_flush};
use crate::window::{may_make_initial_scroll_size_snapshot, may_trigger_win_scrolled_resized};
use crate::winlayer::graph::CMDWIN_RESULT;

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
#[derive(Debug)]
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
        idx: None,
        c: Key::NUL,
        old_col: 0,
        old_pos: Pos {
            lnum: LineNr::ZERO,
            col: ColNr::ZERO,
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

    /// The row of [`NV_CMD_TABLE`] the command resolved to; only asked once it
    /// has resolved to one.
    #[inline(always)]
    pub(crate) fn row(&self) -> &'static NvCmd {
        &NV_CMD_TABLE[self.idx.expect("the command has a row")]
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
    !(!FINISH_OP.get()
        && prev_opcount == 0
        && prev_count0 == 0
        && op_type == OpType::Nop
        && regname.is_unnamed())
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
    state.toplevel = (!cmdwin || CMDWIN_RESULT.get().is_nul()) && !noexmode;
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
    ns.ca.opcount = OPCOUNT.get();

    // 'finish_op' drives the cursor shape, so a change to it is a redraw.
    let was_finishing = FINISH_OP.get();
    FINISH_OP.set(ns.oa.op_type != OpType::Nop);
    if FINISH_OP.get() != was_finishing {
        ui_cursor_shape();
    }
    may_trigger_modechanged();

    // With no operator and no register pending, the count starts over --
    // and `set_prevcount` remembers to publish it as v:prevcount.
    ns.set_prevcount = false;
    if !FINISH_OP.get() && ns.oa.regname.is_unnamed() {
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
    STATE.set(MODE_NORMAL_BUSY);
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
    let flags = ns.row().cmd_flags;
    // "stopsel": an unshifted movement ends the selection.
    if KM_STOPSEL.get() && flags.has(NvFlags::STS) && !MOD_MASK.get().has(ModMask::SHIFT) {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED);
    }
    if KM_STARTSEL.get() {
        if flags.has(NvFlags::SS) {
            // A shifted special key becomes its unshifted self, and the
            // table has to be consulted again for the new character.
            unshift_special(&mut ns.ca);
            ns.idx = find_command(ns.ca.cmdchar);
            if ns.idx.is_none() {
                clear_op_beep(ns.op());
                return true;
            }
        } else if flags.has(NvFlags::SSS) && MOD_MASK.get().has(ModMask::SHIFT) {
            MOD_MASK.set(MOD_MASK.get().without(ModMask::SHIFT));
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
    let flags = ns.row().cmd_flags;
    let pending_op = ns.oa.op_type != OpType::Nop;
    let cmdchar = ns.ca.cmdchar;
    flags.has(NvFlags::NCH)
        && (flags.has_all(NvFlags::NCH_NOP) && !pending_op
            || flags.has_all(NvFlags::NCH_ALW)
            || cmdchar.is('q')
                && !pending_op
                && REG_RECORDING.get() == 0
                && REG_EXECUTING.get() == 0
            || (cmdchar.is('a') || cmdchar.is('i')) && (pending_op || visual_active()))
}

/// Whether the mode message the last command scrolled away has to be put
/// back before the next key is read.
pub(crate) fn normal_need_redraw_mode_message(ns: &mut NormalState) -> bool {
    let showing_mode = p_smd()
        && MSG_SILENT.get() == 0
        && (!RESTART_EDIT.get().is_nul()
            || visual_active()
                && ns.old_pos.lnum == Win::current().w_cursor.lnum
                && ns.old_pos.col == Win::current().w_cursor.col)
        && (CLEAR_CMDLINE.get() || REDRAW_CMDLINE.get())
        && (MSG_DIDOUT.get() || MSG_DIDANY.get() && MSG_SCROLL.get() != 0)
        && !MSG_NOWAIT.get()
        && KEY_TYPED.get();
    // The other way in: an error is on display and insert mode is
    // pending, with no Visual selection to describe instead.
    let error_on_display = !RESTART_EDIT.get().is_nul()
        && !visual_active()
        && MSG_SCROLL.get() != 0
        && EMSG_ON_DISPLAY.get();

    (showing_mode || error_on_display)
        && ns.oa.regname.is_unnamed()
        && !ns.ca.outcome.has(Outcome::COMMAND_BUSY)
        && stuff_empty()
        && typeahead().maplen() == 0
        && EMSG_SILENT.get() == 0
        && !IN_ASSERT_FAILS.get()
        && !DID_WAIT_RETURN.get()
        && ns.oa.op_type == OpType::Nop
}

/// Redraw the screen the message was scrolled off, show the message again,
/// and pause long enough for it to be read.
pub(crate) fn normal_redraw_mode_message() {
    let save_state = STATE.get();
    if !RESTART_EDIT.get().is_nul() {
        STATE.set(MODE_INSERT);
    }
    if MUST_REDRAW.get() != 0 && has_keep_msg() && !EMSG_ON_DISPLAY.get() {
        // The redraw must not print the kept message itself, so it is
        // taken out for the duration and put back after.
        let kmsg = KEEP_MSG.take();
        setcursor();
        let _ = update_screen();
        KEEP_MSG.set(kmsg);
        // A copy: `msg` replaces the kept message.
        if let Some(copy) = KEEP_MSG.with(Clone::clone) {
            msg(copy.as_cstr(), KEEP_MSG_HL_ID.get());
        }
    }
    setcursor();
    ui_cursor_shape();
    ui_flush();
    if MSG_SCROLL.get() != 0 || EMSG_ON_DISPLAY.get() {
        msg_delay(1003, true);
    }
    msg_delay(3003, false);
    STATE.set(save_state);
    MSG_SCROLL.set(0);
    EMSG_ON_DISPLAY.set(false);
}

/// File timestamps and a pending "Press ENTER", once the stuff buffer runs
/// dry.
fn normal_check_stuff_buffer() {
    if stuff_empty() {
        DID_CHECK_TIMESTAMPS.set(false);
        if NEED_CHECK_TIMESTAMPS.get() {
            check_timestamps(false);
        }
        if NEED_WAIT_RETURN.get() {
            wait_return(PromptRedraw::IfScrolled);
        }
    }
}

/// Absorb an interrupt.
///
/// A second CTRL-C while `:global` is running and Ex mode was asked for is
/// what gets you into Ex mode; otherwise the interrupt is swallowed, along
/// with the key that caused it when the more-prompt is not up.
fn normal_check_interrupt(ns: &mut NormalState) {
    if !GOT_INT.get() {
        ns.previous_got_int = false;
        return;
    }
    if ns.noexmode && GLOBAL_BUSY.get() != 0 && !EXMODE_ACTIVE.get() && ns.previous_got_int {
        EXMODE_ACTIVE.set(true);
        STATE.set(MODE_NORMAL);
    } else if GLOBAL_BUSY.get() == 0 || !EXMODE_ACTIVE.get() {
        if !QUIT_MORE.get() {
            vgetc();
        }
        GOT_INT.set(false);
    }
    ns.previous_got_int = true;
}

fn normal_check_window_scrolled() {
    if !FINISH_OP.get() {
        may_trigger_win_scrolled_resized();
    }
}

fn normal_check_cursor_moved() {
    if !FINISH_OP.get()
        && has_event(AutoEvent::CursorMoved)
        && (LAST_CURSORMOVED_WIN.get() != Win::current_or_none().map(Win::id)
            || !equalpos(LAST_CURSORMOVED.get(), Win::current().w_cursor))
    {
        fire_on_curbuf(AutoEvent::CursorMoved);
        LAST_CURSORMOVED_WIN.set(Win::current_or_none().map(Win::id));
        LAST_CURSORMOVED.set(Win::current().w_cursor);
    }
}

fn normal_check_text_changed() {
    if !FINISH_OP.get()
        && has_event(AutoEvent::TextChanged)
        && Buf::current().b_last_changedtick != buf_get_changedtick(Buf::current())
    {
        fire_on_curbuf(AutoEvent::TextChanged);
        Buf::current().b_last_changedtick = buf_get_changedtick(Buf::current());
    }
}

fn normal_check_buffer_modified() {
    if !FINISH_OP.get() && has_event(AutoEvent::BufModifiedSet) && Buf::current().b_changed_invalid
    {
        fire_on_curbuf(AutoEvent::BufModifiedSet);
        Buf::current().b_changed_invalid = false;
    }
}

fn normal_check_safe_state() {
    may_trigger_safestate(!op_pending() && RESTART_EDIT.get().is_nul());
}

fn normal_check_folds() {
    fold_adjust_visual();
    if has_any_folding(Win::current()) != 0 && !char_avail() {
        fold_check_close();
        if FDO_FLAGS.get() & OPT_FDO_FLAG_ALL != 0 {
            fold_open_cursor();
        }
    }
}

/// The idle redraw: scroll the cursor into view, update the screen, and put
/// back the message the last command left to be shown.
fn normal_redraw() {
    update_topline(Win::current());
    validate_cursor(Win::current());
    show_cursor_info_later(false);
    if MUST_REDRAW.get() != 0 {
        let _ = update_screen();
    } else {
        redraw_statuslines();
        if REDRAW_CMDLINE.get() || CLEAR_CMDLINE.get() || REDRAW_MODE.get() {
            showmode();
        }
    }
    Buf::current().b_last_used = os_time_raw();
    // `msg` replaces the kept message, so it is handed a copy -- and the
    // message is not added to the history a second time.
    if let Some(copy) = KEEP_MSG.with(Clone::clone) {
        MSG_HIST_OFF.set(true);
        msg(copy.as_cstr(), KEEP_MSG_HL_ID.get());
        MSG_HIST_OFF.set(false);
    }
    if NEED_FILEINFO.get() && !shortmess(ShmFlag::FILEINFO) {
        fileinfo(0, 1, false);
        NEED_FILEINFO.set(false);
    }
    EMSG_ON_DISPLAY.set(false);
    DID_EMSG.set(0);
    MSG_DIDANY.set(false);
    may_clear_sb_text();
    setcursor();
}

/// One iteration of the state loop's check half.
///
/// Answers [`Step::Go`] to go on and read a command, [`Step::Leave`] to
/// leave normal mode, and [`Step::Again`] to be asked again because Ex mode
/// ran instead.
pub(crate) fn normal_check(ns: &mut NormalState) -> Step {
    normal_check_stuff_buffer();
    normal_check_interrupt(ns);
    if DID_THROW.get() && EX_NORMAL_BUSY.get() == 0 {
        discard_current_exception();
    }
    if !EXMODE_ACTIVE.get() {
        MSG_SCROLL.set(0);
    }
    QUIT_MORE.set(false);
    state_no_longer_safe(None);

    if SKIP_REDRAW.get() || EXMODE_ACTIVE.get() {
        SKIP_REDRAW.set(false);
        setcursor();
    } else if DO_REDRAW.get() || stuff_empty() {
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
        if DIFF_NEED_SCROLLBIND.get() {
            check_scrollbind(0, 0);
            DIFF_NEED_SCROLLBIND.set(false);
        }
        normal_check_folds();
        normal_redraw();
        DO_REDRAW.set(false);
        // The first screen update is the end of startup profiling.
        if startup_timing() {
            time_msg_at(c"first screen update");
            time_finish();
        }
        may_make_initial_scroll_size_snapshot();
    }

    // Collecting is only safe where no caller up the stack is holding a
    // value: the command-line window and Ex mode both are.
    MAY_GARBAGE_COLLECT.set(!ns.cmdwin && !ns.noexmode);
    update_curswant();

    if EXMODE_ACTIVE.get() {
        if ns.noexmode {
            return Step::Leave;
        }
        do_exmode();
        return Step::Again;
    }
    if ns.cmdwin && !CMDWIN_RESULT.get().is_nul() {
        return Step::Leave;
    }
    normal_prepare(ns);
    Step::Go
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
    let mut count = int64_t::from(cmd_arg.count0);
    if cmd_arg.opcount != 0 {
        count = int64_t::from(cmd_arg.opcount) * if count == 0 { 1 } else { count };
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
