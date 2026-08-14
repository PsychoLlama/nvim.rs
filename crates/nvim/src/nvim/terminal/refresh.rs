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
use crate::src::nvim::channel::main_loop_events;
use crate::src::nvim::cursor_shape::SHAPE_VER;
use crate::src::nvim::cursor_shape::{SHAPE_BLOCK, SHAPE_HOR, SHAPE_IDX_TERM, update_shape_entry};
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_move_events, multiqueue_new_child, multiqueue_process_events,
};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{exiting, main_loop};
use crate::src::nvim::mbyte::mb_check_adjust_col;
use crate::src::nvim::memline::{ml_append_buf, ml_replace_buf};
use crate::src::nvim::r#move::{curs_columns, set_topline};
use crate::src::nvim::types::{
    MultiQueue, Terminal, TimeWatcher, WinInfo, colnr_T, linenr_T, uint16_t, uint64_t,
};
use crate::src::nvim::ui::{ui_busy_start, ui_busy_stop, ui_mode_info_set};
use crate::src::nvim::winlayer::{Buf, Win, tab_windows};
use core::ffi::{c_int, c_void};

use super::mode::terminal_check_cursor;
use super::scrollback::{fetch_row, refresh_scrollback};
use super::{Term, is_focused, row_to_linenr};

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
static INVALIDATED: GlobalCell<Vec<Term>> = GlobalCell::new(Vec::new());

pub unsafe fn terminal_init() {
    // SAFETY: the main loop is up, and the timer is this module's own,
    // untouched until `terminal_teardown` closes it.
    unsafe { time_watcher_init(main_loop.ptr(), timer(), ::core::ptr::null_mut()) };
    // SAFETY: as above; the queue is a child of the main loop's.
    unsafe { (*timer()).events = multiqueue_new_child(main_loop_events()) };
}

pub unsafe fn terminal_teardown() {
    // SAFETY: the timer this module started, stopped and closed once.
    unsafe { time_watcher_stop(timer()) };
    // SAFETY: as above, freeing the queue `terminal_init` made.
    unsafe { multiqueue_free((*timer()).events) };
    // SAFETY: as above.
    unsafe { time_watcher_close(timer(), None) };
    INVALIDATED.with_mut(Vec::clear);
}

/// The refresh timer itself. One escape hatch for the whole module: the
/// `time_watcher_*` entry points take the watcher by pointer.
fn timer() -> *mut TimeWatcher {
    REFRESH_TIMER.ptr()
}

/// The queue refresh work is deferred onto. Draining it is
/// [`terminal_check_refresh`].
pub(super) fn refresh_queue() -> *mut MultiQueue {
    // SAFETY: read of a global on the main thread, as everywhere else.
    unsafe { (*timer()).events }
}

/// Mark `rows` as needing a redraw and arm the refresh timer.
///
/// `None` means the screen's contents did not change but the terminal still
/// needs looking at — the cursor moved, or the scrollback shifted.
///
/// Nothing is queued while the child holds synchronized output open: it has
/// asked for the screen to stay as it is until it says otherwise, and the
/// damage it is making meanwhile would be a half-drawn frame.
pub fn invalidate_terminal(mut term: Term, rows: Option<(c_int, c_int)>) {
    if let Some((start_row, end_row)) = rows {
        term.invalid_start = term.invalid_start.min(start_row);
        term.invalid_end = term.invalid_end.max(end_row);
    }
    if term.synchronized_output {
        return;
    }
    INVALIDATED.with_mut(|queued| {
        if !queued.contains(&term) {
            queued.push(term);
        }
    });
    if !REFRESH_PENDING.get() {
        // SAFETY: this module's own timer, armed with its own callback.
        unsafe { time_watcher_start(timer(), Some(refresh_timer_cb), REFRESH_DELAY, 0) };
        REFRESH_PENDING.set(true);
    }
}

/// Run whatever refresh work has come due. Called from the editor's idle
/// paths, since the timer only queues.
pub unsafe fn terminal_check_refresh() {
    // SAFETY: the refresh queue, whose events are this module's own.
    unsafe { multiqueue_process_events(refresh_queue()) };
}

unsafe extern "C" fn refresh_timer_cb(_watcher: *mut TimeWatcher, _data: *mut c_void) {
    REFRESH_PENDING.set(false);
    if exiting.get() {
        return;
    }
    // SAFETY: refreshing runs editor code, which must not fire autocommands
    // from the middle of the event loop; paired with the unblock below.
    unsafe { block_autocmds() };
    // Taken rather than iterated in place: refreshing runs editor code that
    // damages terminals, and those belong to the next round.
    for term in INVALIDATED.with_mut(::core::mem::take) {
        if !term.synchronized_output {
            refresh_terminal(term);
        }
    }
    // SAFETY: as above.
    unsafe { unblock_autocmds() };
}

/// Refresh `term` one last time before it is freed, if it was waiting on
/// the timer, and take it off the queue.
pub(super) fn refresh_before_destroy(term: Term) {
    if !INVALIDATED.with(|queued| queued.contains(&term)) {
        return;
    }
    // SAFETY: as in `refresh_timer_cb`; paired with the unblock below.
    unsafe { block_autocmds() };
    refresh_terminal(term);
    // SAFETY: as above.
    unsafe { unblock_autocmds() };
    // By value, not by index: refreshing can have queued more.
    INVALIDATED.with_mut(|queued| queued.retain(|&queued| queued != term));
}

/// Mirror everything `term` has accumulated into its buffer.
pub fn refresh_terminal(mut term: Term) {
    let Some(buf) = term.buf() else {
        return;
    };
    let ml_before = buf.line_count();
    let resized = refresh_size(term);
    refresh_scrollback(term, buf);
    refresh_screen(term, buf);
    let ml_added = (buf.line_count() - ml_before) as c_int;
    adjust_topline_cursor(term, buf, ml_added);

    if resized {
        // The child now knows the width, so a window scrolled sideways is
        // showing a column that no longer means anything.
        for mut wp in windows_showing(buf) {
            if wp.w_leftcol != 0 {
                wp.w_leftcol = 0 as colnr_T;
                // SAFETY: a window of the current tab page's own list.
                unsafe { curs_columns(wp.raw(), 1) };
            }
        }
    }
    // Events the child's output produced were held back until the buffer
    // agreed with the screen; it does now.
    let events = term.pending.events;
    // SAFETY: the terminal's own queue and the main loop's, both live.
    unsafe { multiqueue_move_events(main_loop_events(), events) };
}

/// Tell the child about a resize the editor already applied to vterm.
///
/// Returns whether anything was sent.
fn refresh_size(mut term: Term) -> bool {
    if !term.pending.resize || term.closed {
        return false;
    }
    term.pending.resize = false;
    let (height, width) = term.size();
    // vterm reflowed everything; none of the old rows can be trusted.
    term.invalid_start = 0;
    term.invalid_end = height;
    // Read out before the call: the channel is free to re-enter.
    let (resize_cb, data) = (term.opts.resize_cb, term.opts.data);
    let (width, height) = (width as uint16_t, height as uint16_t);
    // SAFETY: the callback the channel registered, taking the data it
    // registered with it.
    unsafe { resize_cb.expect("non-null function pointer")(width, height, data) };
    true
}

/// `'scrollback'` changed; trim or extend to match, but only once the
/// scrollback has been sized at all.
pub unsafe fn on_scrollback_option_changed(term: *mut Terminal) {
    // SAFETY: the caller hands over a live terminal.
    let term = unsafe { Term::new(term) };
    if term.sb.is_sized() {
        refresh_terminal(term);
    }
}

/// Replace the buffer lines that mirror the emulator's screen.
///
/// Only the rows marked invalid are re-read. Rows past the end of the
/// buffer are appended instead — that is how a terminal buffer grows to a
/// full screen after it opens.
pub fn refresh_screen(mut term: Term, buf: Buf) {
    let mut changed = 0;
    let mut added = 0;
    let (height, width) = term.size();
    term.invalid_end = term.invalid_end.min(height);
    if term.invalid_start >= term.invalid_end {
        clear_invalid(term);
        return;
    }

    let first_linenr = row_to_linenr(term, term.invalid_start);
    for (offset, row) in (term.invalid_start..term.invalid_end).enumerate() {
        let linenr = (first_linenr + offset as c_int) as linenr_T;
        fetch_row(term, row, width);
        let text = term.textbuf.as_mut_ptr();
        // Past the end of the buffer means the terminal is still filling
        // out its first screen.
        if linenr <= buf.line_count() {
            // SAFETY: a live buffer and a line of it, taking the row this
            // terminal's own line buffer holds.
            unsafe { ml_replace_buf(buf.raw(), linenr, text, true, false) };
            changed += 1;
        } else {
            // SAFETY: as above, appending past the last line.
            unsafe { ml_append_buf(buf.raw(), linenr - 1, text, 0 as colnr_T, false) };
            added += 1;
        }
    }

    term.old_height = height;
    let change_start = row_to_linenr(term, term.invalid_start) as linenr_T;
    let change_end = change_start + changed as linenr_T;
    clear_invalid(term);
    let (buf, added) = (buf.raw(), added as linenr_T);
    // SAFETY: a live buffer, reporting the lines replaced and appended
    // above.
    unsafe { changed_lines(buf, change_start, 0 as colnr_T, change_end, added, true) };
}

/// Reset the damaged-row range to "nothing damaged" — an empty range that
/// any real damage widens.
fn clear_invalid(mut term: Term) {
    term.invalid_start = c_int::MAX;
    term.invalid_end = -1;
}

/// Every window showing `buf`.
fn windows_showing(buf: Buf) -> impl Iterator<Item = Win> {
    tab_windows().filter(move |wp| wp.w_buffer == buf.raw())
}

/// Keep every window on `buf` looking at the bottom of the terminal.
///
/// A window whose cursor was on the last line before `added` lines arrived
/// was following the output, and keeps following. One that had scrolled up
/// stays where it was, clamped to the buffer.
pub fn adjust_topline_cursor(term: Term, mut buf: Buf, added: c_int) {
    let ml_end = buf.line_count();
    for mut wp in windows_showing(buf) {
        if wp.is_current() && is_focused(term) {
            terminal_check_cursor(term);
            continue;
        }
        if ml_end == wp.w_cursor.lnum + added as linenr_T {
            wp.w_cursor.lnum = ml_end;
            let topline = (wp.w_cursor.lnum - wp.w_view_height as linenr_T + 1).max(1);
            // SAFETY: a window of the current tab page's own list.
            unsafe { set_topline(wp.raw(), topline) };
        } else {
            wp.w_cursor.lnum = wp.w_cursor.lnum.min(ml_end);
        }
        // SAFETY: as above; the column is clamped against the line the
        // cursor was just moved to.
        unsafe { mb_check_adjust_col(wp.raw() as *mut c_void) };
    }

    // Windows are not the only things remembering a line: the buffer's own
    // last-cursor mark and the per-window info follow too.
    if ml_end == buf.b_last_cursor.mark.lnum + added as linenr_T {
        buf.b_last_cursor.mark.lnum = ml_end;
    }
    let (wininfos, count) = (buf.b_wininfo.items, buf.b_wininfo.size);
    for i in 0..count {
        // SAFETY: the buffer's own array of `count` live entries, none of
        // which anything above frees.
        let wip: &mut WinInfo = unsafe { &mut **wininfos.add(i) };
        if ml_end == wip.wi_mark.mark.lnum + added as linenr_T {
            wip.wi_mark.mark.lnum = ml_end;
        }
    }
}

/// Track the emulator's cursor: hide the editor's while the child says the
/// cursor is invisible, and republish the cursor shape when it changes.
///
/// Only for the terminal the user is typing at — the shape is global.
pub fn refresh_cursor(mut term: Term, cursor_visible: &mut bool) {
    if !is_focused(term) {
        return;
    }
    if term.cursor.visible != *cursor_visible {
        *cursor_visible = term.cursor.visible;
        // "Busy" is what hides the cursor; the UI has no other way to be
        // told the cursor is not to be drawn.
        if *cursor_visible {
            ui_busy_stop();
        } else {
            ui_busy_start();
        }
    }
    if !term.pending.cursor {
        return;
    }
    term.pending.cursor = false;
    let blink = if term.cursor.blink { 500 } else { 0 };
    let shape = term.cursor.shape;
    update_shape_entry(SHAPE_IDX_TERM, |entry| {
        entry.blinkon = blink;
        entry.blinkoff = blink;
        // vterm's DECSCUSR shapes, which do not line up with the editor's.
        // An unknown shape leaves the previous one in place.
        match shape {
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
    });
    // Publishes the shape table just written to every attached UI.
    ui_mode_info_set();
}
