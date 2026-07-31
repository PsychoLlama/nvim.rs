//! Getting what the emulator holds into the buffer the user sees.
//!
//! Nothing draws when output arrives. vterm's callbacks only record which
//! rows changed ([`invalidate_terminal`]) and put the terminal on a queue;
//! a ten-millisecond timer then drains the queue and mirrors each
//! terminal's screen into its buffer's lines. Batching that way is what
//! keeps a program printing thousands of lines a second from running the
//! editor's redraw thousands of times a second.
//!
//! One refresh is four steps, in this order: tell the child about a resize
//! ([`refresh_size`]), append the rows that scrolled off
//! ([`refresh_scrollback`](super::scrollback::refresh_scrollback)), replace
//! the rows still on screen ([`refresh_screen`]), and move every window's
//! cursor to follow ([`adjust_topline_cursor`]). The order matters: line
//! numbers are relative to how much scrollback exists, so the scrollback
//! has to settle before the screen is written at the right lines.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::change::changed_lines;
use crate::src::nvim::cursor_shape::shape_table;
use crate::src::nvim::cursor_shape::{SHAPE_BLOCK, SHAPE_HOR, SHAPE_IDX_TERM, SHAPE_VER};
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_move_events, multiqueue_new_child, multiqueue_process_events,
};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curwin, exiting, main_loop};
use crate::src::nvim::mbyte::mb_check_adjust_col;
use crate::src::nvim::memline::{ml_append_buf, ml_replace_buf};
use crate::src::nvim::r#move::{curs_columns, set_topline};
use crate::src::nvim::types::{
    MultiQueue, Terminal, TimeWatcher, WinInfo, buf_T, colnr_T, linenr_T, size_t, uint16_t,
    uint64_t, win_T,
};
use crate::src::nvim::ui::{ui_busy_start, ui_busy_stop, ui_mode_info_set};
use crate::src::nvim::vterm::vterm::vterm_get_size;
use core::ffi::{c_int, c_void};

use super::mode::terminal_check_cursor;
use super::scrollback::{fetch_row, refresh_scrollback};
use super::{all_windows, buf_for_handle, is_focused, row_to_linenr};

/// How long to let damage accumulate before mirroring it into the buffer,
/// in milliseconds.
const REFRESH_DELAY: uint64_t = 10;

/// The timer that drains [`INVALIDATED`]. Its own event queue is a child of
/// the main loop's, so that draining it runs only refresh work.
static REFRESH_TIMER: GlobalCell<TimeWatcher> = GlobalCell::new(TimeWatcher::EMPTY);

/// Whether [`REFRESH_TIMER`] is already armed.
static REFRESH_PENDING: GlobalCell<bool> = GlobalCell::new(false);

/// Terminals with damage the buffer has not seen yet, in the order they
/// first took damage.
static INVALIDATED: GlobalCell<Vec<*mut Terminal>> = GlobalCell::new(Vec::new());

pub unsafe fn terminal_init() {
    unsafe {
        time_watcher_init(
            main_loop.ptr(),
            REFRESH_TIMER.ptr(),
            ::core::ptr::null_mut(),
        );
        (*REFRESH_TIMER.ptr()).events = multiqueue_new_child((*main_loop.ptr()).events);
    }
}

pub unsafe fn terminal_teardown() {
    unsafe {
        time_watcher_stop(REFRESH_TIMER.ptr());
        multiqueue_free((*REFRESH_TIMER.ptr()).events);
        time_watcher_close(REFRESH_TIMER.ptr(), None);
        INVALIDATED.with_mut(Vec::clear);
    }
}

/// The queue refresh work is deferred onto. Draining it is
/// [`terminal_check_refresh`].
pub(super) fn refresh_queue() -> *mut MultiQueue {
    // SAFETY: read of a global on the main thread, as everywhere else.
    unsafe { (*REFRESH_TIMER.ptr()).events }
}

/// Mark `rows` as needing a redraw and arm the refresh timer.
///
/// `None` means the screen's contents did not change but the terminal still
/// needs looking at — the cursor moved, or the scrollback shifted.
///
/// Nothing is queued while the child holds synchronized output open: it has
/// asked for the screen to stay as it is until it says otherwise, and the
/// damage it is making meanwhile would be a half-drawn frame.
pub unsafe fn invalidate_terminal(term: *mut Terminal, rows: Option<(c_int, c_int)>) {
    unsafe {
        if let Some((start_row, end_row)) = rows {
            (*term).invalid_start = (*term).invalid_start.min(start_row);
            (*term).invalid_end = (*term).invalid_end.max(end_row);
        }
        if (*term).synchronized_output {
            return;
        }
        INVALIDATED.with_mut(|queued| {
            if !queued.contains(&term) {
                queued.push(term);
            }
        });
        if !REFRESH_PENDING.get() {
            time_watcher_start(
                REFRESH_TIMER.ptr(),
                Some(refresh_timer_cb),
                REFRESH_DELAY,
                0,
            );
            REFRESH_PENDING.set(true);
        }
    }
}

/// Run whatever refresh work has come due. Called from the editor's idle
/// paths, since the timer only queues.
pub unsafe fn terminal_check_refresh() {
    unsafe { multiqueue_process_events(refresh_queue()) };
}

unsafe extern "C" fn refresh_timer_cb(_watcher: *mut TimeWatcher, _data: *mut c_void) {
    unsafe {
        REFRESH_PENDING.set(false);
        if exiting.get() {
            return;
        }
        block_autocmds();
        // Taken rather than iterated in place: refreshing runs editor code
        // that damages terminals, and those belong to the next round.
        let to_refresh = INVALIDATED.with_mut(::core::mem::take);
        for term in to_refresh {
            if !(*term).synchronized_output {
                refresh_terminal(term);
            }
        }
        unblock_autocmds();
    }
}

/// Refresh `term` one last time before it is freed, if it was waiting on
/// the timer, and take it off the queue.
pub(super) unsafe fn refresh_before_destroy(term: *mut Terminal) {
    unsafe {
        if !INVALIDATED.with(|queued| queued.contains(&term)) {
            return;
        }
        block_autocmds();
        refresh_terminal(term);
        unblock_autocmds();
        // By value, not by index: refreshing can have queued more.
        INVALIDATED.with_mut(|queued| queued.retain(|&queued| queued != term));
    }
}

/// Mirror everything `term` has accumulated into its buffer.
pub unsafe fn refresh_terminal(term: *mut Terminal) {
    unsafe {
        let buf = buf_for_handle((*term).buf_handle);
        if buf.is_null() {
            return;
        }
        let ml_before = (*buf).b_ml.ml_line_count;
        let resized = refresh_size(term);
        refresh_scrollback(term, buf);
        refresh_screen(term, buf);
        let ml_added = ((*buf).b_ml.ml_line_count - ml_before) as c_int;
        adjust_topline_cursor(term, buf, ml_added);

        if resized {
            // The child now knows the width, so a window scrolled sideways
            // is showing a column that no longer means anything.
            for wp in windows_showing(buf) {
                if (*wp).w_leftcol != 0 {
                    (*wp).w_leftcol = 0 as colnr_T;
                    curs_columns(wp, 1);
                }
            }
        }
        // Events the child's output produced were held back until the
        // buffer agreed with the screen; it does now.
        multiqueue_move_events((*main_loop.ptr()).events, (*term).pending.events);
    }
}

/// Tell the child about a resize the editor already applied to vterm.
///
/// Returns whether anything was sent.
unsafe fn refresh_size(term: *mut Terminal) -> bool {
    unsafe {
        if !(*term).pending.resize || (*term).closed {
            return false;
        }
        (*term).pending.resize = false;
        let mut width = 0;
        let mut height = 0;
        vterm_get_size((*term).vt, &raw mut height, &raw mut width);
        // vterm reflowed everything; none of the old rows can be trusted.
        (*term).invalid_start = 0;
        (*term).invalid_end = height;
        (*term).opts.resize_cb.expect("non-null function pointer")(
            width as uint16_t,
            height as uint16_t,
            (*term).opts.data,
        );
        true
    }
}

/// `'scrollback'` changed; trim or extend to match, but only once the
/// scrollback has been sized at all.
pub unsafe fn on_scrollback_option_changed(term: *mut Terminal) {
    unsafe {
        if (*term).sb.is_sized() {
            refresh_terminal(term);
        }
    }
}

/// Replace the buffer lines that mirror the emulator's screen.
///
/// Only the rows marked invalid are re-read. Rows past the end of the
/// buffer are appended instead — that is how a terminal buffer grows to a
/// full screen after it opens.
pub unsafe fn refresh_screen(term: *mut Terminal, buf: *mut buf_T) {
    unsafe {
        let mut changed = 0;
        let mut added = 0;
        let mut height = 0;
        let mut width = 0;
        vterm_get_size((*term).vt, &raw mut height, &raw mut width);
        (*term).invalid_end = (*term).invalid_end.min(height);
        if (*term).invalid_start >= (*term).invalid_end {
            clear_invalid(term);
            return;
        }

        let first_linenr = row_to_linenr(term, (*term).invalid_start);
        for (offset, row) in ((*term).invalid_start..(*term).invalid_end).enumerate() {
            let linenr = (first_linenr + offset as c_int) as linenr_T;
            fetch_row(term, row, width);
            let text = (*term).textbuf.as_mut_ptr();
            // Past the end of the buffer means the terminal is still
            // filling out its first screen.
            if linenr <= (*buf).b_ml.ml_line_count {
                ml_replace_buf(buf, linenr, text, true, false);
                changed += 1;
            } else {
                ml_append_buf(buf, linenr - 1, text, 0 as colnr_T, false);
                added += 1;
            }
        }

        (*term).old_height = height;
        let change_start = row_to_linenr(term, (*term).invalid_start);
        let change_end = change_start + changed;
        clear_invalid(term);
        changed_lines(
            buf,
            change_start as linenr_T,
            0 as colnr_T,
            change_end as linenr_T,
            added as linenr_T,
            true,
        );
    }
}

/// Reset the damaged-row range to "nothing damaged" — an empty range that
/// any real damage widens.
unsafe fn clear_invalid(term: *mut Terminal) {
    unsafe {
        (*term).invalid_start = c_int::MAX;
        (*term).invalid_end = -1;
    }
}

/// Every window showing `buf`.
unsafe fn windows_showing(buf: *mut buf_T) -> impl Iterator<Item = *mut win_T> {
    // SAFETY: as `all_windows`; the predicate only reads `w_buffer`.
    unsafe { all_windows().filter(move |&wp| (*wp).w_buffer == buf) }
}

/// Keep every window on `buf` looking at the bottom of the terminal.
///
/// A window whose cursor was on the last line before `added` lines arrived
/// was following the output, and keeps following. One that had scrolled up
/// stays where it was, clamped to the buffer.
pub unsafe fn adjust_topline_cursor(term: *mut Terminal, buf: *mut buf_T, added: c_int) {
    unsafe {
        let ml_end = (*buf).b_ml.ml_line_count;
        for wp in windows_showing(buf) {
            if wp == curwin.get() && is_focused(term) {
                terminal_check_cursor();
                continue;
            }
            if ml_end == (*wp).w_cursor.lnum + added as linenr_T {
                (*wp).w_cursor.lnum = ml_end;
                set_topline(
                    wp,
                    ((*wp).w_cursor.lnum - (*wp).w_view_height as linenr_T + 1).max(1),
                );
            } else {
                (*wp).w_cursor.lnum = (*wp).w_cursor.lnum.min(ml_end);
            }
            mb_check_adjust_col(wp as *mut c_void);
        }

        // Windows are not the only things remembering a line: the buffer's
        // own last-cursor mark and the per-window info follow too.
        if ml_end == (*buf).b_last_cursor.mark.lnum + added as linenr_T {
            (*buf).b_last_cursor.mark.lnum = ml_end;
        }
        let mut i: size_t = 0;
        while i < (*buf).b_wininfo.size {
            let wip: *mut WinInfo = *(*buf).b_wininfo.items.add(i);
            if ml_end == (*wip).wi_mark.mark.lnum + added as linenr_T {
                (*wip).wi_mark.mark.lnum = ml_end;
            }
            i += 1;
        }
    }
}

/// Track the emulator's cursor: hide the editor's while the child says the
/// cursor is invisible, and republish the cursor shape when it changes.
///
/// Only for the terminal the user is typing at — the shape is global.
pub unsafe fn refresh_cursor(term: *mut Terminal, cursor_visible: &mut bool) {
    unsafe {
        if !is_focused(term) {
            return;
        }
        if (*term).cursor.visible != *cursor_visible {
            *cursor_visible = (*term).cursor.visible;
            // "Busy" is what hides the cursor; the UI has no other way to
            // be told the cursor is not to be drawn.
            if *cursor_visible {
                ui_busy_stop();
            } else {
                ui_busy_start();
            }
        }
        if !(*term).pending.cursor {
            return;
        }
        (*term).pending.cursor = false;

        let entry = &mut (*shape_table.ptr())[SHAPE_IDX_TERM as usize];
        let blink = if (*term).cursor.blink { 500 } else { 0 };
        entry.blinkon = blink;
        entry.blinkoff = blink;
        // vterm's DECSCUSR shapes, which do not line up with the editor's.
        // An unknown shape leaves the previous one in place.
        match (*term).cursor.shape {
            1 => entry.shape = SHAPE_BLOCK,
            2 => {
                entry.shape = SHAPE_HOR;
                entry.percentage = 20;
            }
            3 => {
                entry.shape = SHAPE_VER;
                entry.percentage = 25;
            }
            _ => {}
        }
        ui_mode_info_set();
    }
}
