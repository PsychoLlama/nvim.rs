//! Terminal mode: the editor loop that runs while the user types at a
//! terminal.
//!
//! [`terminal_enter`] pushes a [`TerminalState`] onto the editor's state
//! stack and does not return until the user leaves. While it is up, keys go
//! to the child rather than to normal mode ([`terminal_execute`]) and the
//! editor's cursor is dragged along behind the emulator's
//! ([`terminal_check_cursor`]).
//!
//! Two things make it more than a loop. Several window options would fight
//! the child for the screen — `'cursorline'`, `'cursorcolumn'`,
//! `'scrolloff'` — so they are saved and forced
//! ([`set_terminal_winopts`]) and put back on the way out, including when
//! the user moves to a different window mid-session. And the terminal can
//! die while its own mode is running, so nearly every step re-checks
//! whether the buffer still has a terminal at all; `refcount` holds the
//! `Terminal` alive across anything that might run autocommands.
//!
//! `CTRL-\` is the escape hatch: `CTRL-\ CTRL-N` leaves for normal mode and
//! `CTRL-\ CTRL-O` leaves for a single command. Both are recognised here
//! rather than being sent to the child.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::autocmd::{apply_autocmds, has_event};
use crate::buffer::{buf_get_changedtick, do_buffer};
use crate::cursor::coladvance;
use crate::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::drawscreen::{
    UPD_SOME_VALID, UPD_VALID, redraw_statuslines, setcursor, show_cursor_info_later, showmode,
    unshowmode, update_screen,
};
use crate::ex_docmd::{DoCmdOpts, do_cmdline};
use crate::getchar::{getcmdkeycmd, map_execute_lua, merge_modifiers, paste_repeat};
use crate::guard::Allow;
use crate::keycodes::{Ctrl_BSL, Ctrl_C, Ctrl_N, Ctrl_O, Key};
use crate::main::{
    State, clear_cmdline, got_int, mapped_ctrl_c, mod_mask, must_redraw, redraw_cmdline,
    redraw_mode, restart_edit, stop_insert_mode,
};
use crate::memory::{strequal, xstrdup};
use crate::r#move::{set_topline, validate_cursor};
use crate::options::kOptCuloptFlagNumber;
use crate::optionstr::free_string_option;
use crate::state::{MODE_TERMINAL, may_trigger_modechanged, state_enter, state_handle_k_event};
use crate::types::AutoEvent;
use crate::types::{OptInt, VimState, colnr_T, linenr_T, pos_T, uint8_t, winopt_T};
use crate::ui::{ui_busy_stop, ui_cursor_shape, ui_flush};
use crate::vterm::state::entry::{vterm_state_focus_in, vterm_state_focus_out};
use crate::window::{may_trigger_win_scrolled_resized, win_valid};
use crate::winlayer::{Buf, Win, WinId};
use core::ffi::{c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};

use super::input::{is_mouse_key, send_mouse_event, terminal_send_key};
use super::refresh::{
    adjust_topline_cursor, invalidate_terminal, refresh_cursor, terminal_check_refresh,
};
use super::{Term, row_to_linenr, terminal_check_size, terminal_set_state};
use crate::search::FORWARD;

const DOBUF_WIPE: c_int = 4;
const DOBUF_FIRST: c_int = 1;

/// One terminal-mode session, as the editor's state stack sees it.
///
/// `state` is first so that a `*mut TerminalState` and the `*mut VimState`
/// the stack hands back to the callbacks are the same address.
#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct TerminalState {
    pub state: VimState,
    pub term: Term,
    /// `RedrawingDisabled` on entry. Terminal mode has to redraw.
    pub save_rd: c_int,
    /// The terminal ended while its mode was running; wipe it on the way
    /// out rather than mid-callback.
    pub close: bool,
    /// `CTRL-\` was seen and the next key decides what it meant.
    pub got_bsl: bool,
    /// It meant `CTRL-\ CTRL-O`: leave for exactly one command.
    pub got_bsl_o: bool,
    /// What the UI was last told about cursor visibility.
    pub cursor_visible: bool,
    /// The window whose options [`set_terminal_winopts`] changed, `None` when
    /// none are changed.
    ///
    /// A [`WinId`] and not a `Win`: terminal mode runs the whole editor
    /// between saving these options and restoring them, so the window may be
    /// closed by then and its address must not be held. See `winlayer`'s
    /// re-entry rule.
    pub save_curwin: Option<WinId>,
    pub save_w_p_cul: bool,
    pub save_w_p_culopt: *mut c_char,
    pub save_w_p_culopt_flags: uint8_t,
    pub save_w_p_cuc: c_int,
    pub save_w_p_so: OptInt,
    pub save_w_p_siso: OptInt,
}

impl TerminalState {
    fn new(term: Term) -> Self {
        Self {
            state: VimState {
                check: Some(terminal_check),
                execute: Some(terminal_execute),
            },
            term,
            save_rd: 0,
            close: false,
            got_bsl: false,
            got_bsl_o: false,
            cursor_visible: true,
            save_curwin: None,
            save_w_p_cul: false,
            save_w_p_culopt: ::core::ptr::null_mut(),
            save_w_p_culopt_flags: 0,
            save_w_p_cuc: 0,
            save_w_p_so: 0,
            save_w_p_siso: 0,
        }
    }
}

/// One terminal-mode session the caller has promised is live.
///
/// The editor's state stack holds the session by pointer and hands it back
/// to [`terminal_check`] and [`terminal_execute`] as the [`VimState`] it
/// starts with, and everything either of those reaches can run Vimscript
/// that comes back through them — so the pointer stays raw and `Session` is
/// [`Copy`], exactly as [`Term`] is for the terminal. Every borrow it
/// produces lives for one expression.
#[derive(Clone, Copy)]
struct Session(*mut TerminalState);

impl Deref for Session {
    type Target = TerminalState;

    #[inline(always)]
    fn deref(&self) -> &TerminalState {
        // SAFETY: `Session::new`'s promise — the session is live.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Session {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut TerminalState {
        // SAFETY: as above.
        unsafe { &mut *self.0 }
    }
}

impl Session {
    /// # Safety
    /// `s` must stay a live session for as long as the value is used.
    #[inline(always)]
    const unsafe fn new(s: *mut TerminalState) -> Self {
        Self(s)
    }

    /// The session a `VimState` the state stack handed back belongs to.
    ///
    /// # Safety
    /// `state` must be the state of a live [`TerminalState`], which is the
    /// only thing this module's callbacks are ever installed on.
    #[inline(always)]
    const unsafe fn of(state: *mut VimState) -> Self {
        Self(state.cast())
    }

    /// The half the editor's state stack takes. `state` is the session's
    /// first field, so the two addresses are the same one.
    #[inline(always)]
    fn vim_state(self) -> *mut VimState {
        self.0.cast()
    }
}

/// The window the editor is working in.
fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The buffer the editor is working in.
fn current_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// Tell the child whether it has focus, so it can show its own cursor
/// accordingly.
fn terminal_focus(term: Term, focus: bool) {
    let state = term.state();
    if focus {
        // SAFETY: the emulator's own state machine.
        unsafe { vterm_state_focus_in(state.0) };
    } else {
        // SAFETY: as above.
        unsafe { vterm_state_focus_out(state.0) };
    }
}

/// Force the current window's options to what a terminal needs, saving what
/// they were.
///
/// `'cursorline'` survives only as `'culopt'=number`, which highlights the
/// number column and leaves the child's own colours alone; everything else
/// that would draw over the child's output is turned off.
fn set_terminal_winopts(mut s: Session) {
    assert!(
        s.save_curwin.is_none(),
        "terminal window options saved twice"
    );
    let mut wp = current_win();
    s.save_curwin = Some(wp.id());
    s.save_w_p_cul = wp.w_onebuf_opt.wo_cul != 0;
    s.save_w_p_culopt = ::core::ptr::null_mut();
    s.save_w_p_culopt_flags = wp.w_p_culopt_flags;
    s.save_w_p_cuc = wp.w_onebuf_opt.wo_cuc;
    s.save_w_p_so = wp.w_onebuf_opt.wo_so;
    s.save_w_p_siso = wp.w_onebuf_opt.wo_siso;

    if wp.w_onebuf_opt.wo_cul != 0
        && wp.w_p_culopt_flags as c_int & kOptCuloptFlagNumber as c_int != 0
    {
        let culopt = wp.w_onebuf_opt.wo_culopt;
        // SAFETY: `'culopt'` is a NUL-terminated option string, compared
        // against one of this crate's own.
        if !unsafe { strequal(culopt, c"number".as_ptr()) } {
            s.save_w_p_culopt = culopt;
            // SAFETY: copies one of this crate's own strings; the copy is
            // the window's until `unset_terminal_winopts` frees it.
            wp.w_onebuf_opt.wo_culopt = unsafe { xstrdup(c"number".as_ptr()) };
        }
        wp.w_p_culopt_flags = kOptCuloptFlagNumber as uint8_t;
    } else {
        wp.w_onebuf_opt.wo_cul = 0;
    }
    wp.w_onebuf_opt.wo_cuc = 0;
    wp.w_onebuf_opt.wo_so = 0 as OptInt;
    wp.w_onebuf_opt.wo_siso = 0 as OptInt;

    if wp.w_onebuf_opt.wo_cuc != s.save_w_p_cuc {
        wp.redraw_later(UPD_SOME_VALID);
    } else if wp.w_onebuf_opt.wo_cul != s.save_w_p_cul as c_int
        || (wp.w_onebuf_opt.wo_cul != 0 && wp.w_p_culopt_flags != s.save_w_p_culopt_flags)
    {
        wp.redraw_later(UPD_VALID);
    }
}

/// Put back what [`set_terminal_winopts`] changed.
///
/// The window may have moved to a different buffer since, in which case the
/// options to restore are the ones remembered for the terminal's buffer
/// rather than the window's live set. If neither can be found there is
/// nothing to restore but the saved `'culopt'` string, which is freed
/// either way.
fn unset_terminal_winopts(mut s: Session) {
    assert!(
        s.save_curwin.is_some(),
        "terminal window options restored without being saved"
    );
    if let Some(winopts) = saved_winopts(s) {
        // SAFETY: either a live window's own option set, or the copy the
        // terminal's buffer kept of it.
        let winopts = unsafe { &mut *winopts };
        if !s.save_w_p_culopt.is_null() {
            // SAFETY: the `'culopt'` the window is holding, which the saved
            // one replaces.
            unsafe { free_string_option(winopts.wo_culopt) };
            winopts.wo_culopt = s.save_w_p_culopt;
            s.save_w_p_culopt = ::core::ptr::null_mut();
        }
        winopts.wo_cul = s.save_w_p_cul as c_int;
        winopts.wo_cuc = s.save_w_p_cuc;
        winopts.wo_so = s.save_w_p_so;
        winopts.wo_siso = s.save_w_p_siso;
    }
    // SAFETY: the copy `set_terminal_winopts` made, or null, which this
    // takes as "nothing to free".
    unsafe { free_string_option(s.save_w_p_culopt) };
    s.save_curwin = None;
}

/// Where [`unset_terminal_winopts`] should write, redrawing the window on
/// the way if it is still showing the terminal.
fn saved_winopts(s: Session) -> Option<*mut winopt_T> {
    let mut wp = s.save_curwin?.get()?;
    if wp.buffer().handle != s.term.buf_handle {
        // The window went elsewhere; the terminal's buffer kept a copy of
        // the options this window had while it was showing it.
        let buf = s.term.buf()?;
        let (wininfos, count) = (buf.b_wininfo.items, buf.b_wininfo.size);
        for i in 0..count {
            // SAFETY: the buffer's own array of `count` live entries.
            let wip = unsafe { *wininfos.add(i) };
            // SAFETY: as above; nothing here frees one.
            let (win, optset, opts) =
                unsafe { ((*wip).wi_win, (*wip).wi_optset, &raw mut (*wip).wi_opt) };
            if win == wp.raw() && optset {
                return Some(opts);
            }
        }
        return None;
    }

    // Not `Win::valid`: `wp` came out of the registry just above, so it is
    // registered by construction. What is asked here is the *layout*
    // question — is it still on this tab page's window list — which stays a
    // list walk. SAFETY: the pointer is only compared.
    if win_valid(wp.raw()) {
        if s.save_w_p_cuc != wp.w_onebuf_opt.wo_cuc {
            wp.redraw_later(UPD_SOME_VALID);
        } else if s.save_w_p_cul as c_int != wp.w_onebuf_opt.wo_cul
            || (s.save_w_p_cul && s.save_w_p_culopt_flags != wp.w_p_culopt_flags)
        {
            wp.redraw_later(UPD_VALID);
        }
    }
    wp.w_p_culopt_flags = s.save_w_p_culopt_flags;
    // SAFETY: a live window's own option set.
    Some(unsafe { &raw mut (*wp.raw()).w_onebuf_opt })
}

/// Run terminal mode for the current buffer's terminal until the user
/// leaves it.
///
/// Returns whether it was left by `CTRL-\ CTRL-O`, which the caller turns
/// into a single normal-mode command before coming back.
pub(crate) unsafe fn terminal_enter() -> bool {
    let buf = current_buf();
    assert!(
        !buf.terminal.is_null(),
        "terminal mode entered on a buffer with no terminal"
    );
    // SAFETY: the current buffer's own terminal, which the assertion above
    // says is there.
    let mut session = TerminalState::new(unsafe { Term::new(buf.terminal) });
    // SAFETY: the session outlives every use below — the state stack is
    // left before this frame is.
    let mut s = unsafe { Session::new(&raw mut session) };

    stop_insert_mode.set(false);
    // SAFETY: a live terminal.
    unsafe { terminal_check_size(s.term.raw()) };

    let save_state = State.get();
    State.set(MODE_TERMINAL);
    mapped_ctrl_c.set(mapped_ctrl_c.get() | MODE_TERMINAL);
    let redraw = Allow::redraw();
    set_terminal_winopts(s);
    s.term.pending.cursor = true;
    adjust_topline_cursor(s.term, buf, 0);
    // SAFETY: draws the mode message.
    unsafe { showmode() };
    // SAFETY: publishes the cursor shape to every attached UI.
    unsafe { ui_cursor_shape() };
    terminal_focus(s.term, true);
    let mut buf = current_buf();
    // SAFETY: a live buffer's own change counter.
    buf.b_last_changedtick_i = buf_get_changedtick(buf);

    s.term.refcount.retain();
    let none = ::core::ptr::null_mut();
    // SAFETY: TermEnter against a live buffer; nothing of the terminal or
    // the session is borrowed across it.
    unsafe { apply_autocmds(AutoEvent::TermEnter, none, none, false, buf.raw()) };
    // SAFETY: reports the mode change, which can run autocommands too.
    unsafe { may_trigger_modechanged() };
    s.term.refcount.release();
    if s.term.buf_handle == 0 {
        s.close = true;
    }

    // SAFETY: the stack drives the session through its own `VimState`, and
    // gives it back before this returns.
    unsafe { state_enter(s.vim_state()) };

    if !s.got_bsl_o {
        restart_edit.set(0);
    }
    State.set(save_state);
    drop(redraw);
    if !s.cursor_visible {
        ui_busy_stop();
    }
    // SAFETY: re-reads `'guicursor'` now that terminal mode is over.
    unsafe { parse_shape_opt(SHAPE_CURSOR) };
    unset_terminal_winopts(s);
    terminal_focus(s.term, false);
    let mut buf = current_buf();
    // SAFETY: a live buffer's own change counter.
    buf.b_last_changedtick = buf_get_changedtick(buf);
    if buf.terminal == s.term.raw() && !s.close {
        terminal_check_cursor(s.term);
    }
    if restart_edit.get() != 0 {
        // SAFETY: draws the mode message.
        unsafe { showmode() };
    } else {
        // SAFETY: clears it.
        unsafe { unshowmode(true) };
    }
    // SAFETY: publishes the cursor shape to every attached UI.
    unsafe { ui_cursor_shape() };

    // TermLeave can reach the terminal, so hold it open even though it is
    // about to be destroyed.
    if s.close {
        s.term.refcount.retain();
    }
    // SAFETY: TermLeave against a live buffer, as above.
    unsafe { apply_autocmds(AutoEvent::TermLeave, none, none, false, current_buf().raw()) };
    if s.close {
        s.term.refcount.release();
        let buf_handle = s.term.buf_handle;
        s.term.destroy = true;
        // Read out before the call: the channel's close callback is free to
        // free the terminal.
        let (close_cb, data) = (s.term.opts.close_cb, s.term.opts.data);
        // SAFETY: the callback the channel registered, taking the data it
        // registered with it.
        unsafe { close_cb.expect("non-null function pointer")(data) };
        if buf_handle != 0 {
            let _ = do_buffer(DOBUF_WIPE, DOBUF_FIRST, FORWARD, buf_handle, 1);
        }
    }
    s.got_bsl_o
}

/// Put the editor's cursor where the emulator's is.
///
/// In terminal mode the cursor sits exactly on the emulator's column. In
/// normal mode it has to sit *on* a character rather than after one, so it
/// steps back — forward in a right-to-left window.
pub(super) fn terminal_check_cursor(term: Term) {
    let mut win = current_win();
    let buf = current_buf();
    let cursor_line = row_to_linenr(term, term.cursor.row) as linenr_T;
    win.w_cursor.lnum = buf.line_count().min(cursor_line);

    // Terminal windows always show the bottom of the buffer.
    let topline = (buf.line_count() - win.w_view_height as linenr_T + 1).max(1);
    if topline != win.w_topline {
        // SAFETY: a live window.
        set_topline(win, topline);
    }

    if term.suspended && State.get() & MODE_TERMINAL != 0 {
        win.w_cursor = pos_T {
            lnum: buf.line_count(),
            col: 0,
            coladd: 0,
        };
        return;
    }
    let off = if State.get() & MODE_TERMINAL != 0 {
        0
    } else if win.w_onebuf_opt.wo_rl != 0 {
        1
    } else {
        -1
    };
    let col = (term.cursor.col + off).max(0) as colnr_T;
    // SAFETY: a live window, and a column of the line its cursor is on.
    coladvance(win, col);
}

/// Follow the user moving between windows or buffers without leaving
/// terminal mode.
///
/// Returns false once the current buffer has no terminal at all, which is
/// how the mode loop learns to stop.
fn terminal_check_focus(mut s: Session) -> bool {
    if current_buf().terminal.is_null() {
        return false;
    }
    if s.save_curwin != Some(current_win().id()) {
        unset_terminal_winopts(s);
        set_terminal_winopts(s);
    }
    if s.term.raw() != current_buf().terminal {
        terminal_focus(s.term, false);
        // The terminal being left was already finished; nothing else will
        // come back to close it.
        if s.close {
            s.term.destroy = true;
            // Read out before the call: the channel's close callback is
            // free to free the terminal.
            let (close_cb, data) = (s.term.opts.close_cb, s.term.opts.data);
            // SAFETY: the callback the channel registered, taking the data
            // it registered with it.
            unsafe { close_cb.expect("non-null function pointer")(data) };
            s.close = false;
        }
        // SAFETY: the terminal of the buffer the user moved to.
        s.term = unsafe { Term::new(current_buf().terminal) };
        s.term.pending.cursor = true;
        invalidate_terminal(s.term, None);
        terminal_focus(s.term, true);
    }
    true
}

/// The mode loop's per-iteration work: refresh, redraw, and place the
/// cursor. Returning zero leaves terminal mode.
unsafe fn terminal_check(state: *mut VimState) -> c_int {
    // SAFETY: the state stack hands back the session this module pushed.
    let mut s = unsafe { Session::of(state) };
    debug_assert!(
        !s.close || (s.term.buf_handle == 0 && s.term.raw() != current_buf().terminal),
        "a terminal marked closed is still the current buffer's"
    );
    if stop_insert_mode.get() || !terminal_check_focus(s) {
        return 0;
    }
    // SAFETY: drains the refresh queue, which is this module's own.
    unsafe { terminal_check_refresh() };
    terminal_check_cursor(s.term);
    // SAFETY: a live window.
    validate_cursor(current_win());

    // TextChangedT observers can close the terminal.
    s.term.refcount.retain();
    // SAFETY: reads the editor's own event table.
    let observed = has_event(AutoEvent::TextChangedT);
    let mut buf = current_buf();
    // SAFETY: a live buffer's own change counter.
    if observed && buf.b_last_changedtick_i != buf_get_changedtick(buf) {
        let none = ::core::ptr::null_mut();
        // SAFETY: TextChangedT against a live buffer; nothing of the
        // terminal or the session is borrowed across it.
        unsafe { apply_autocmds(AutoEvent::TextChangedT, none, none, false, buf.raw()) };
        let mut buf = current_buf();
        // SAFETY: as above.
        buf.b_last_changedtick_i = buf_get_changedtick(buf);
    }
    // SAFETY: reports scrolls and resizes, which run autocommands.
    unsafe { may_trigger_win_scrolled_resized() };
    s.term.refcount.release();
    if s.term.buf_handle == 0 {
        s.close = true;
    }

    // Those autocommands could have moved the user somewhere else.
    if !terminal_check_focus(s) {
        return 0;
    }
    terminal_check_cursor(s.term);
    // SAFETY: a live window.
    validate_cursor(current_win());
    // SAFETY: schedules the cursor-position report.
    unsafe { show_cursor_info_later(false) };
    if must_redraw.get() != 0 {
        // SAFETY: redraws the screen.
        let _ = unsafe { update_screen() };
    } else {
        // SAFETY: redraws the status lines only.
        unsafe { redraw_statuslines() };
        if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
            // SAFETY: draws the mode message.
            unsafe { showmode() };
        }
    }
    // SAFETY: puts the terminal cursor where the window says.
    unsafe { setcursor() };
    // Read out and written back: the shape goes to every attached UI, and
    // nothing of the session is borrowed while it does.
    let mut cursor_visible = s.cursor_visible;
    refresh_cursor(s.term, &mut cursor_visible);
    s.cursor_visible = cursor_visible;
    // SAFETY: flushes what the redraw produced to every attached UI.
    unsafe { ui_flush() };
    1
}

/// Dispatch one key. Returning zero leaves terminal mode.
unsafe fn terminal_execute(state: *mut VimState, key: c_int) -> c_int {
    // SAFETY: the state stack hands back the session this module pushed.
    let mut s = unsafe { Session::of(state) };
    // `merge_modifiers` folds a pending modifier into the key so that e.g.
    // `<C-\>` is one code to compare against.
    let mut mods = mod_mask.get();
    let mod_key = merge_modifiers(key, &mut mods);

    // Keys the editor handles itself. Everything but CTRL-N/CTRL-O is dealt
    // with here and done; those two continue below because they are what a
    // preceding CTRL-\ was waiting for.
    if is_mouse_key(mod_key) {
        if send_mouse_event(s.term, key) {
            return 0;
        }
        return 1;
    }
    match Key::try_from(mod_key) {
        Ok(Key::PasteStart) => {
            // SAFETY: replays the paste the editor has buffered.
            unsafe { paste_repeat(1) };
            return 1;
        }
        Ok(Key::Event) => {
            // An event handler can close the terminal.
            s.term.refcount.retain();
            // SAFETY: runs whatever the main loop had queued.
            unsafe { state_handle_k_event() };
            s.term.refcount.release();
            if s.term.buf_handle == 0 {
                s.close = true;
            }
            return 1;
        }
        Ok(Key::Command) => {
            let (none, data) = (::core::ptr::null_mut(), ::core::ptr::null_mut::<c_void>());
            // SAFETY: runs the command the key carries, which is read back
            // by `getcmdkeycmd` rather than passed here.
            let _ = unsafe { do_cmdline(none, Some(getcmdkeycmd), data, DoCmdOpts::NONE) };
            return 1;
        }
        Ok(Key::Lua) => {
            // SAFETY: runs the Lua callback the key carries.
            unsafe { map_execute_lua(false, false) };
            return 1;
        }
        Ok(Key::Ignore | Key::Nop) => return 1,
        _ => {}
    }
    // CTRL-\ CTRL-N leaves for normal mode; CTRL-\ CTRL-O leaves for one
    // command and comes back.
    if s.got_bsl && matches!(mod_key, Ctrl_N | Ctrl_O) {
        if mod_key == Ctrl_N {
            return 0;
        }
        s.got_bsl_o = true;
        restart_edit.set(b'I' as c_int);
        return 0;
    }

    // CTRL-C in terminal mode belongs to the child, so the interrupt the
    // editor recorded is dropped.
    if mod_key == Ctrl_C {
        got_int.set(false);
    }
    if mod_key == Ctrl_BSL && !s.got_bsl {
        s.got_bsl = true;
    } else if s.term.suspended {
        // Any key wakes a suspended child rather than being sent to it.
        //
        // Read out before the call: the channel's resume callback may
        // re-enter.
        let (resume_cb, data) = (s.term.opts.resume_cb, s.term.opts.data);
        // SAFETY: the callback the channel registered, taking the data it
        // registered with it.
        unsafe { resume_cb.expect("non-null function pointer")(data) };
        // SAFETY: a live terminal.
        unsafe { terminal_set_state(s.term.raw(), false) };
    } else {
        if s.term.closed {
            s.close = true;
            return 0;
        }
        s.got_bsl = false;
        terminal_send_key(s.term, key);
    }
    1
}
