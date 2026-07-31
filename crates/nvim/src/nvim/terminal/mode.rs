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

use crate::src::nvim::autocmd::{
    EVENT_TERMENTER, EVENT_TERMLEAVE, EVENT_TEXTCHANGEDT, apply_autocmds, has_event,
};
use crate::src::nvim::buffer::{buf_get_changedtick, do_buffer};
use crate::src::nvim::cursor::coladvance;
use crate::src::nvim::cursor_shape::{SHAPE_CURSOR, parse_shape_opt};
use crate::src::nvim::drawscreen::{
    redraw_later, redraw_statuslines, setcursor, show_cursor_info_later, showmode, unshowmode,
    update_screen,
};
use crate::src::nvim::ex_docmd::do_cmdline;
use crate::src::nvim::getchar::{getcmdkeycmd, map_execute_lua, merge_modifiers, paste_repeat};
use crate::src::nvim::main::{
    RedrawingDisabled, State, clear_cmdline, curbuf, curwin, got_int, mapped_ctrl_c, mod_mask,
    must_redraw, redraw_cmdline, redraw_mode, restart_edit, stop_insert_mode, window_handles,
};
use crate::src::nvim::memory::{strequal, xstrdup};
use crate::src::nvim::r#move::{set_topline, validate_cursor};
use crate::src::nvim::options::kOptCuloptFlagNumber;
use crate::src::nvim::optionstr::free_string_option;
use crate::src::nvim::state::{may_trigger_modechanged, state_enter, state_handle_k_event};
use crate::src::nvim::types::{
    OptInt, Terminal, VimState, WinInfo, colnr_T, handle_T, linenr_T, pos_T, size_t, uint8_t,
    win_T, winopt_T,
};
use crate::src::nvim::ui::{ui_busy_stop, ui_cursor_shape, ui_flush};
use crate::src::nvim::vterm::state::{
    vterm_obtain_state, vterm_state_focus_in, vterm_state_focus_out,
};
use crate::src::nvim::window::{may_trigger_win_scrolled_resized, win_valid};
use core::ffi::{c_char, c_int, c_void};

use super::input::{
    Ctrl_BSL, Ctrl_C, Ctrl_N, Ctrl_O, K_COMMAND, K_EVENT, K_IGNORE, K_LUA, K_NOP, K_PASTE_START,
    is_mouse_key, send_mouse_event, terminal_send_key,
};
use super::refresh::{
    adjust_topline_cursor, invalidate_terminal, refresh_cursor, terminal_check_refresh,
};
use super::{
    MODE_TERMINAL, UPD_SOME_VALID, UPD_VALID, map_get_int_ptr_t, terminal_check_size,
    terminal_set_state,
};

const DOBUF_WIPE: c_int = 4;
const DOBUF_FIRST: c_int = 1;
const FORWARD: c_int = 1;

/// One terminal-mode session, as the editor's state stack sees it.
///
/// `state` is first so that a `*mut TerminalState` and the `*mut VimState`
/// the stack hands back to the callbacks are the same address.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalState {
    pub state: VimState,
    pub term: *mut Terminal,
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
    /// The window whose options [`set_terminal_winopts`] changed, zero when
    /// none are changed.
    pub save_curwin_handle: handle_T,
    pub save_w_p_cul: bool,
    pub save_w_p_culopt: *mut c_char,
    pub save_w_p_culopt_flags: uint8_t,
    pub save_w_p_cuc: c_int,
    pub save_w_p_so: OptInt,
    pub save_w_p_siso: OptInt,
}

impl TerminalState {
    fn new(term: *mut Terminal) -> Self {
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
            save_curwin_handle: 0,
            save_w_p_cul: false,
            save_w_p_culopt: ::core::ptr::null_mut(),
            save_w_p_culopt_flags: 0,
            save_w_p_cuc: 0,
            save_w_p_so: 0,
            save_w_p_siso: 0,
        }
    }
}

/// Tell the child whether it has focus, so it can show its own cursor
/// accordingly.
unsafe fn terminal_focus(term: *const Terminal, focus: bool) {
    unsafe {
        let state = vterm_obtain_state((*term).vt);
        if focus {
            vterm_state_focus_in(state);
        } else {
            vterm_state_focus_out(state);
        }
    }
}

/// Force the current window's options to what a terminal needs, saving what
/// they were.
///
/// `'cursorline'` survives only as `'culopt'=number`, which highlights the
/// number column and leaves the child's own colours alone; everything else
/// that would draw over the child's output is turned off.
unsafe fn set_terminal_winopts(s: *mut TerminalState) {
    unsafe {
        assert!(
            (*s).save_curwin_handle == 0,
            "terminal window options saved twice"
        );
        let wp = curwin.get();
        (*s).save_curwin_handle = (*wp).handle;
        (*s).save_w_p_cul = (*wp).w_onebuf_opt.wo_cul != 0;
        (*s).save_w_p_culopt = ::core::ptr::null_mut();
        (*s).save_w_p_culopt_flags = (*wp).w_p_culopt_flags;
        (*s).save_w_p_cuc = (*wp).w_onebuf_opt.wo_cuc;
        (*s).save_w_p_so = (*wp).w_onebuf_opt.wo_so;
        (*s).save_w_p_siso = (*wp).w_onebuf_opt.wo_siso;

        if (*wp).w_onebuf_opt.wo_cul != 0
            && (*wp).w_p_culopt_flags as c_int & kOptCuloptFlagNumber as c_int != 0
        {
            if !strequal((*wp).w_onebuf_opt.wo_culopt, c"number".as_ptr()) {
                (*s).save_w_p_culopt = (*wp).w_onebuf_opt.wo_culopt;
                (*wp).w_onebuf_opt.wo_culopt = xstrdup(c"number".as_ptr());
            }
            (*wp).w_p_culopt_flags = kOptCuloptFlagNumber as uint8_t;
        } else {
            (*wp).w_onebuf_opt.wo_cul = 0;
        }
        (*wp).w_onebuf_opt.wo_cuc = 0;
        (*wp).w_onebuf_opt.wo_so = 0 as OptInt;
        (*wp).w_onebuf_opt.wo_siso = 0 as OptInt;

        if (*wp).w_onebuf_opt.wo_cuc != (*s).save_w_p_cuc {
            redraw_later(wp, UPD_SOME_VALID);
        } else if (*wp).w_onebuf_opt.wo_cul != (*s).save_w_p_cul as c_int
            || ((*wp).w_onebuf_opt.wo_cul != 0
                && (*wp).w_p_culopt_flags != (*s).save_w_p_culopt_flags)
        {
            redraw_later(wp, UPD_VALID);
        }
    }
}

/// Put back what [`set_terminal_winopts`] changed.
///
/// The window may have moved to a different buffer since, in which case the
/// options to restore are the ones remembered for the terminal's buffer
/// rather than the window's live set. If neither can be found there is
/// nothing to restore but the saved `'culopt'` string, which is freed
/// either way.
unsafe fn unset_terminal_winopts(s: *mut TerminalState) {
    unsafe {
        assert!(
            (*s).save_curwin_handle != 0,
            "terminal window options restored without being saved"
        );
        if let Some(winopts) = saved_winopts(s) {
            if !(*s).save_w_p_culopt.is_null() {
                free_string_option((*winopts).wo_culopt);
                (*winopts).wo_culopt = (*s).save_w_p_culopt;
                (*s).save_w_p_culopt = ::core::ptr::null_mut();
            }
            (*winopts).wo_cul = (*s).save_w_p_cul as c_int;
            (*winopts).wo_cuc = (*s).save_w_p_cuc;
            (*winopts).wo_so = (*s).save_w_p_so;
            (*winopts).wo_siso = (*s).save_w_p_siso;
        }
        free_string_option((*s).save_w_p_culopt);
        (*s).save_curwin_handle = 0;
    }
}

/// Where [`unset_terminal_winopts`] should write, redrawing the window on
/// the way if it is still showing the terminal.
unsafe fn saved_winopts(s: *mut TerminalState) -> Option<*mut winopt_T> {
    unsafe {
        let wp = map_get_int_ptr_t(window_handles.ptr(), (*s).save_curwin_handle) as *mut win_T;
        if wp.is_null() {
            return None;
        }
        if (*(*wp).w_buffer).handle != (*(*s).term).buf_handle {
            // The window went elsewhere; the terminal's buffer kept a copy
            // of the options this window had while it was showing it.
            let buf = super::buf_for_handle((*(*s).term).buf_handle);
            if buf.is_null() {
                return None;
            }
            let mut i: size_t = 0;
            while i < (*buf).b_wininfo.size {
                let wip: *mut WinInfo = *(*buf).b_wininfo.items.add(i);
                if (*wip).wi_win == wp && (*wip).wi_optset {
                    return Some(&raw mut (*wip).wi_opt);
                }
                i += 1;
            }
            return None;
        }

        if win_valid(wp) {
            if (*s).save_w_p_cuc != (*wp).w_onebuf_opt.wo_cuc {
                redraw_later(wp, UPD_SOME_VALID);
            } else if (*s).save_w_p_cul as c_int != (*wp).w_onebuf_opt.wo_cul
                || ((*s).save_w_p_cul && (*s).save_w_p_culopt_flags != (*wp).w_p_culopt_flags)
            {
                redraw_later(wp, UPD_VALID);
            }
        }
        (*wp).w_p_culopt_flags = (*s).save_w_p_culopt_flags;
        Some(&raw mut (*wp).w_onebuf_opt)
    }
}

/// Run terminal mode for the current buffer's terminal until the user
/// leaves it.
///
/// Returns whether it was left by `CTRL-\ CTRL-O`, which the caller turns
/// into a single normal-mode command before coming back.
pub unsafe fn terminal_enter() -> bool {
    unsafe {
        let buf = curbuf.get();
        assert!(
            !(*buf).terminal.is_null(),
            "terminal mode entered on a buffer with no terminal"
        );
        let mut s = TerminalState::new((*buf).terminal);
        let s = &raw mut s;

        stop_insert_mode.set(false);
        terminal_check_size((*s).term);

        let save_state = State.get();
        (*s).save_rd = RedrawingDisabled.get();
        State.set(MODE_TERMINAL);
        *mapped_ctrl_c.ptr() |= MODE_TERMINAL;
        RedrawingDisabled.set(0);
        set_terminal_winopts(s);
        (*(*s).term).pending.cursor = true;
        adjust_topline_cursor((*s).term, buf, 0);
        showmode();
        ui_cursor_shape();
        terminal_focus((*s).term, true);
        (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());

        (*(*s).term).refcount += 1;
        apply_autocmds(
            EVENT_TERMENTER,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        may_trigger_modechanged();
        (*(*s).term).refcount -= 1;
        if (*(*s).term).buf_handle == 0 {
            (*s).close = true;
        }

        state_enter(&raw mut (*s).state);

        if !(*s).got_bsl_o {
            restart_edit.set(0);
        }
        State.set(save_state);
        RedrawingDisabled.set((*s).save_rd);
        if !(*s).cursor_visible {
            ui_busy_stop();
        }
        parse_shape_opt(SHAPE_CURSOR);
        unset_terminal_winopts(s);
        terminal_focus((*s).term, false);
        (*curbuf.get()).b_last_changedtick = buf_get_changedtick(curbuf.get());
        if (*curbuf.get()).terminal == (*s).term && !(*s).close {
            terminal_check_cursor();
        }
        if restart_edit.get() != 0 {
            showmode();
        } else {
            unshowmode(true);
        }
        ui_cursor_shape();

        // TermLeave can reach the terminal, so hold it open even though it
        // is about to be destroyed.
        if (*s).close {
            (*(*s).term).refcount += 1;
        }
        apply_autocmds(
            EVENT_TERMLEAVE,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            curbuf.get(),
        );
        if (*s).close {
            (*(*s).term).refcount -= 1;
            let buf_handle = (*(*s).term).buf_handle;
            (*(*s).term).destroy = true;
            (*(*s).term)
                .opts
                .close_cb
                .expect("non-null function pointer")((*(*s).term).opts.data);
            if buf_handle != 0 {
                do_buffer(DOBUF_WIPE, DOBUF_FIRST, FORWARD, buf_handle, 1);
            }
        }
        (*s).got_bsl_o
    }
}

/// Put the editor's cursor where the emulator's is.
///
/// In terminal mode the cursor sits exactly on the emulator's column. In
/// normal mode it has to sit *on* a character rather than after one, so it
/// steps back — forward in a right-to-left window.
pub(super) unsafe fn terminal_check_cursor() {
    unsafe {
        let term = (*curbuf.get()).terminal;
        let win = curwin.get();
        let buf = curbuf.get();
        let cursor_line = super::row_to_linenr(term, (*term).cursor.row) as linenr_T;
        (*win).w_cursor.lnum = (*buf).b_ml.ml_line_count.min(cursor_line);

        // Terminal windows always show the bottom of the buffer.
        let topline = ((*buf).b_ml.ml_line_count - (*win).w_view_height as linenr_T + 1).max(1);
        if topline != (*win).w_topline {
            set_topline(win, topline);
        }

        if (*term).suspended && State.get() & MODE_TERMINAL != 0 {
            (*win).w_cursor = pos_T {
                lnum: (*buf).b_ml.ml_line_count,
                col: 0,
                coladd: 0,
            };
            return;
        }
        let off = if State.get() & MODE_TERMINAL != 0 {
            0
        } else if (*win).w_onebuf_opt.wo_rl != 0 {
            1
        } else {
            -1
        };
        coladvance(win, ((*term).cursor.col + off).max(0) as colnr_T);
    }
}

/// Follow the user moving between windows or buffers without leaving
/// terminal mode.
///
/// Returns false once the current buffer has no terminal at all, which is
/// how the mode loop learns to stop.
unsafe fn terminal_check_focus(s: *mut TerminalState) -> bool {
    unsafe {
        if (*curbuf.get()).terminal.is_null() {
            return false;
        }
        if (*s).save_curwin_handle != (*curwin.get()).handle {
            unset_terminal_winopts(s);
            set_terminal_winopts(s);
        }
        if (*s).term != (*curbuf.get()).terminal {
            terminal_focus((*s).term, false);
            // The terminal being left was already finished; nothing else
            // will come back to close it.
            if (*s).close {
                (*(*s).term).destroy = true;
                (*(*s).term)
                    .opts
                    .close_cb
                    .expect("non-null function pointer")((*(*s).term).opts.data);
                (*s).close = false;
            }
            (*s).term = (*curbuf.get()).terminal;
            (*(*s).term).pending.cursor = true;
            invalidate_terminal((*s).term, None);
            terminal_focus((*s).term, true);
        }
        true
    }
}

/// The mode loop's per-iteration work: refresh, redraw, and place the
/// cursor. Returning zero leaves terminal mode.
unsafe extern "C" fn terminal_check(state: *mut VimState) -> c_int {
    unsafe {
        let s = state as *mut TerminalState;
        debug_assert!(
            !(*s).close || ((*(*s).term).buf_handle == 0 && (*s).term != (*curbuf.get()).terminal),
            "a terminal marked closed is still the current buffer's"
        );
        if stop_insert_mode.get() || !terminal_check_focus(s) {
            return 0;
        }
        terminal_check_refresh();
        terminal_check_cursor();
        validate_cursor(curwin.get());

        // TextChangedT observers can close the terminal.
        (*(*s).term).refcount += 1;
        if has_event(EVENT_TEXTCHANGEDT)
            && (*curbuf.get()).b_last_changedtick_i != buf_get_changedtick(curbuf.get())
        {
            apply_autocmds(
                EVENT_TEXTCHANGEDT,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
            (*curbuf.get()).b_last_changedtick_i = buf_get_changedtick(curbuf.get());
        }
        may_trigger_win_scrolled_resized();
        (*(*s).term).refcount -= 1;
        if (*(*s).term).buf_handle == 0 {
            (*s).close = true;
        }

        // Those autocommands could have moved the user somewhere else.
        if !terminal_check_focus(s) {
            return 0;
        }
        terminal_check_cursor();
        validate_cursor(curwin.get());
        show_cursor_info_later(false);
        if must_redraw.get() != 0 {
            update_screen();
        } else {
            redraw_statuslines();
            if clear_cmdline.get() || redraw_cmdline.get() || redraw_mode.get() {
                showmode();
            }
        }
        setcursor();
        refresh_cursor((*s).term, &mut (*s).cursor_visible);
        ui_flush();
        1
    }
}

/// Dispatch one key. Returning zero leaves terminal mode.
unsafe extern "C" fn terminal_execute(state: *mut VimState, key: c_int) -> c_int {
    unsafe {
        let s = state as *mut TerminalState;
        // `merge_modifiers` folds a pending modifier into the key so that
        // e.g. `<C-\>` is one code to compare against.
        let mut mods = mod_mask.get();
        let mod_key = merge_modifiers(key, &raw mut mods);

        // Keys the editor handles itself. Everything but CTRL-N/CTRL-O is
        // dealt with here and done; those two continue below because they
        // are what a preceding CTRL-\ was waiting for.
        if is_mouse_key(mod_key) {
            if send_mouse_event((*s).term, key) {
                return 0;
            }
            return 1;
        }
        match mod_key {
            K_PASTE_START => {
                paste_repeat(1);
                return 1;
            }
            K_EVENT => {
                // An event handler can close the terminal.
                (*(*s).term).refcount += 1;
                state_handle_k_event();
                (*(*s).term).refcount -= 1;
                if (*(*s).term).buf_handle == 0 {
                    (*s).close = true;
                }
                return 1;
            }
            K_COMMAND => {
                do_cmdline(
                    ::core::ptr::null_mut(),
                    Some(getcmdkeycmd),
                    ::core::ptr::null_mut::<c_void>(),
                    0,
                );
                return 1;
            }
            K_LUA => {
                map_execute_lua(false, false);
                return 1;
            }
            K_IGNORE | K_NOP => return 1,
            Ctrl_N | Ctrl_O => {
                // CTRL-\ CTRL-N leaves for normal mode; CTRL-\ CTRL-O
                // leaves for one command and comes back.
                if (*s).got_bsl {
                    if mod_key == Ctrl_N {
                        return 0;
                    }
                    (*s).got_bsl_o = true;
                    restart_edit.set(b'I' as c_int);
                    return 0;
                }
            }
            _ => {}
        }

        // CTRL-C in terminal mode belongs to the child, so the interrupt
        // the editor recorded is dropped.
        if mod_key == Ctrl_C {
            got_int.set(false);
        }
        if mod_key == Ctrl_BSL && !(*s).got_bsl {
            (*s).got_bsl = true;
        } else if (*(*s).term).suspended {
            // Any key wakes a suspended child rather than being sent to it.
            (*(*s).term)
                .opts
                .resume_cb
                .expect("non-null function pointer")((*(*s).term).opts.data);
            terminal_set_state((*s).term, false);
        } else {
            if (*(*s).term).closed {
                (*s).close = true;
                return 0;
            }
            (*s).got_bsl = false;
            terminal_send_key((*s).term, key);
        }
        1
    }
}
