//! Tab page management functions.
//!
//! This module provides Rust implementations of tab page operations
//! from `src/nvim/window.c`.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, TabpageHandle, WinHandle};

// Import list module functions we depend on
use crate::list::{
    get_tabpage_firstwin, nvim_get_first_tabpage, nvim_tabpage_get_next, nvim_win_get_next,
    win_valid_any_tab_impl,
};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curtab.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get lastused_tabpage.
    fn nvim_get_lastused_tabpage() -> TabpageHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get tabpage_move_disallowed global.
    fn nvim_al_get_tabpage_move_disallowed() -> c_int;

    /// Set redraw_tabline global.
    fn nvim_set_redraw_tabline(val: c_int);

    /// Set tp_next field on a tabpage.
    fn nvim_tabpage_set_next(tp: TabpageHandle, next: TabpageHandle);

    /// Set first_tabpage global.
    fn nvim_set_first_tabpage(tp: TabpageHandle);

    /// Call text_locked().
    fn nvim_text_locked() -> bool;

    /// Call text_locked_msg().
    fn nvim_text_locked_msg();

    /// Call beep_flush().
    fn nvim_beep_flush();

    /// Call goto_tabpage_tp() (triggers autocmds, stays in C).
    fn nvim_al_goto_tabpage_tp(tp: TabpageHandle, trigger_enter: c_int, trigger_leave: c_int);

    /// Get tcl_flags (tabclose option flags).
    fn nvim_win_get_tcl_flags() -> c_int;

    // --- unuse_tabpage / use_tabpage dependencies ---
    /// Get topframe global.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get firstwin global.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get lastwin global.
    fn nvim_get_lastwin() -> WinHandle;

    /// Get curwin global.
    fn nvim_get_curwin() -> WinHandle;

    /// Set curtab global.
    fn nvim_set_curtab(tp: TabpageHandle);

    /// Set topframe global.
    fn nvim_set_topframe(fr: *mut Frame);

    /// Set firstwin global (and curtab->tp_firstwin).
    fn nvim_set_firstwin(wp: WinHandle);

    /// Set lastwin global (and curtab->tp_lastwin).
    fn nvim_set_lastwin(wp: WinHandle);

    /// Set curwin global.
    fn nvim_set_curwin(wp: WinHandle);

    /// Set tp->tp_topframe.
    fn nvim_tabpage_set_topframe(tp: TabpageHandle, fr: *mut Frame);

    /// Set tp->tp_firstwin.
    fn nvim_tabpage_set_firstwin(tp: TabpageHandle, wp: WinHandle);

    /// Set tp->tp_lastwin.
    fn nvim_tabpage_set_lastwin(tp: TabpageHandle, wp: WinHandle);

    /// Set tp->tp_curwin.
    fn nvim_tabpage_set_curwin(tp: TabpageHandle, wp: WinHandle);

    /// Get tp->tp_topframe.
    fn nvim_tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame;

    /// Get tp->tp_firstwin.
    fn nvim_tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle;

    /// Get tp->tp_lastwin.
    fn nvim_tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle;

    /// Get tp->tp_curwin.
    fn nvim_tabpage_get_curwin(tp: TabpageHandle) -> WinHandle;
}

// =============================================================================
// Tabpage Validation
// =============================================================================

/// Check if "tpc" is a pointer to an existing tabpage.
///
/// This is the Rust equivalent of `valid_tabpage()` in window.c.
#[inline]
#[must_use]
pub(crate) fn valid_tabpage_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Iterate over all tabpages using FOR_ALL_TABS pattern
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            return true;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    false
}

/// Check if a tabpage has any valid window.
///
/// This is the Rust equivalent of `valid_tabpage_win()` in window.c.
#[inline]
fn valid_tabpage_win_impl(tpc: TabpageHandle) -> bool {
    if tpc.is_null() {
        return false;
    }

    // Find the tabpage in the list
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        if tp == tpc {
            // Found the tabpage - check if any window is valid
            let mut wp = get_tabpage_firstwin(tp);
            while !wp.is_null() {
                if win_valid_any_tab_impl(wp) {
                    return true;
                }
                wp = unsafe { nvim_win_get_next(wp) };
            }
            return false;
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    // Tabpage not found - shouldn't happen
    false
}

// =============================================================================
// Tabpage Finding
// =============================================================================

/// Get the 1-based index of a tabpage.
///
/// This is the Rust equivalent of `tabpage_index()` in window.c.
#[inline]
fn tabpage_index_impl(ftp: TabpageHandle) -> c_int {
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && tp != ftp {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    i
}

/// Find tab page by 1-based number.
///
/// This is the Rust equivalent of `find_tabpage()` in window.c.
#[inline]
fn find_tabpage_impl(n: c_int) -> TabpageHandle {
    let mut i: c_int = 1;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() && i != n {
        i += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    tp
}

/// Find the tabpage that contains a given window.
///
/// This is the Rust equivalent of `win_find_tabpage()` in window.c.
#[inline]
fn win_find_tabpage_impl(win: WinHandle) -> TabpageHandle {
    if win.is_null() {
        return TabpageHandle::null();
    }

    // FOR_ALL_TAB_WINDOWS pattern
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        let mut wp = get_tabpage_firstwin(tp);
        while !wp.is_null() {
            if wp == win {
                return tp;
            }
            wp = unsafe { nvim_win_get_next(wp) };
        }
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    TabpageHandle::null()
}

// =============================================================================
// Tabpage Counting and Navigation
// =============================================================================

/// Count total tabpages.
fn count_tabpages_impl() -> c_int {
    let mut count = 0;
    let mut tp = unsafe { nvim_get_first_tabpage() };
    while !tp.is_null() {
        count += 1;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    count
}

/// Get the last tabpage.
fn last_tabpage_impl() -> TabpageHandle {
    let mut tp = unsafe { nvim_get_first_tabpage() };
    let mut last = tp;
    while !tp.is_null() {
        last = tp;
        tp = unsafe { nvim_tabpage_get_next(tp) };
    }
    last
}

/// Get the previous tabpage (before curtab).
fn prev_tabpage_impl() -> TabpageHandle {
    unsafe {
        let curtab = nvim_get_curtab();
        if curtab.is_null() {
            return TabpageHandle::null();
        }

        let mut tp = nvim_get_first_tabpage();
        let mut prev = TabpageHandle::null();
        while !tp.is_null() && tp != curtab {
            prev = tp;
            tp = nvim_tabpage_get_next(tp);
        }
        prev
    }
}

/// Get the next tabpage (after curtab).
fn next_tabpage_impl() -> TabpageHandle {
    unsafe {
        let curtab = nvim_get_curtab();
        if curtab.is_null() {
            return TabpageHandle::null();
        }
        nvim_tabpage_get_next(curtab)
    }
}

/// Check if this is the only tabpage.
fn only_one_tabpage_impl() -> bool {
    unsafe {
        let first = nvim_get_first_tabpage();
        first.is_null() || nvim_tabpage_get_next(first).is_null()
    }
}

/// Check if curtab is the first tabpage.
fn is_first_tabpage_impl() -> bool {
    unsafe {
        let curtab = nvim_get_curtab();
        let first = nvim_get_first_tabpage();
        curtab == first
    }
}

/// Check if curtab is the last tabpage.
fn is_last_tabpage_impl() -> bool {
    unsafe {
        let curtab = nvim_get_curtab();
        if curtab.is_null() {
            return false;
        }
        nvim_tabpage_get_next(curtab).is_null()
    }
}

// =============================================================================
// Tabpage Window Queries
// =============================================================================

/// Count windows in a tabpage.
fn count_windows_in_tabpage_impl(tp: TabpageHandle) -> c_int {
    let mut count = 0;
    let mut wp = get_tabpage_firstwin(tp);
    while !wp.is_null() {
        count += 1;
        wp = unsafe { nvim_win_get_next(wp) };
    }
    count
}

/// Count non-floating windows in a tabpage.
fn count_nonfloating_in_tabpage_impl(tp: TabpageHandle) -> c_int {
    unsafe {
        let mut count = 0;
        let mut wp = get_tabpage_firstwin(tp);
        while !wp.is_null() {
            if nvim_win_get_floating(wp) == 0 {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Get the last used tabpage if valid.
fn get_lastused_tabpage_impl() -> TabpageHandle {
    unsafe {
        let lastused = nvim_get_lastused_tabpage();
        if !lastused.is_null() && valid_tabpage_impl(lastused) {
            return lastused;
        }
        TabpageHandle::null()
    }
}

/// Check if there's a valid last used tabpage.
fn has_lastused_tabpage_impl() -> bool {
    !get_lastused_tabpage_impl().is_null()
}

// =============================================================================
// FFI Exports
// =============================================================================
//
// Note: Several tabpage FFI functions are exported from lib.rs:
// - rs_valid_tabpage - validates tabpage exists
// - rs_valid_tabpage_win - check if tabpage has valid window
// - rs_tabpage_index - get 1-based index of tabpage
// - rs_find_tabpage - find tabpage by 1-based number
// - rs_one_tabpage - check if only one tabpage
// - rs_win_find_tabpage - find tabpage containing window
// - rs_tabpage_win_valid - check if window valid in tabpage
//
// This module provides additional tabpage operations.

/// FFI: Count total tabpages.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_count() -> c_int {
    count_tabpages_impl()
}

/// FFI: Get last tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_last() -> TabpageHandle {
    last_tabpage_impl()
}

/// FFI: Get previous tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_prev() -> TabpageHandle {
    prev_tabpage_impl()
}

/// FFI: Get next tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_next() -> TabpageHandle {
    next_tabpage_impl()
}

/// FFI: Check if curtab is first.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_is_first() -> c_int {
    c_int::from(is_first_tabpage_impl())
}

/// FFI: Check if curtab is last.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_is_last() -> c_int {
    c_int::from(is_last_tabpage_impl())
}

/// FFI: Count windows in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_count_windows(tp: TabpageHandle) -> c_int {
    count_windows_in_tabpage_impl(tp)
}

/// FFI: Count non-floating windows in tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_count_nonfloating(tp: TabpageHandle) -> c_int {
    count_nonfloating_in_tabpage_impl(tp)
}

/// FFI: Get last used tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_lastused() -> TabpageHandle {
    get_lastused_tabpage_impl()
}

/// FFI: Check if last used tabpage exists.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_has_lastused() -> c_int {
    c_int::from(has_lastused_tabpage_impl())
}

// =============================================================================
// Tabpage Navigation Helpers
// =============================================================================

/// Find target tabpage for goto_tabpage with count n.
///
/// Returns the target tabpage based on count:
/// - n == 0: next tabpage (wrap around)
/// - n < 0: previous n times (wrap around)
/// - n > 0: tabpage at position n
/// - n == 9999: last tabpage
fn goto_tabpage_find_target_impl(n: c_int) -> TabpageHandle {
    unsafe {
        let first = nvim_get_first_tabpage();
        let curtab = nvim_get_curtab();

        // Only one tabpage
        if first.is_null() || nvim_tabpage_get_next(first).is_null() {
            return TabpageHandle::null();
        }

        if n == 0 {
            // No count, go to next tab page, wrap around end
            let next = nvim_tabpage_get_next(curtab);
            if next.is_null() {
                return first;
            }
            return next;
        }

        if n < 0 {
            // Go to previous tab page, wrap around end. N times.
            let mut ttp = curtab;
            for _ in n..0 {
                // Find the tabpage before ttp
                let mut tp = first;
                while !tp.is_null() && nvim_tabpage_get_next(tp) != ttp {
                    tp = nvim_tabpage_get_next(tp);
                }
                if tp.is_null() {
                    // ttp was first, wrap to last
                    tp = first;
                    while !nvim_tabpage_get_next(tp).is_null() {
                        tp = nvim_tabpage_get_next(tp);
                    }
                }
                ttp = tp;
            }
            return ttp;
        }

        if n == 9999 {
            // Go to last tabpage
            let mut tp = first;
            while !nvim_tabpage_get_next(tp).is_null() {
                tp = nvim_tabpage_get_next(tp);
            }
            return tp;
        }

        // Go to tabpage at position n
        find_tabpage_impl(n)
    }
}

/// Check if tabpage move is valid (not moving to same position).
fn tabpage_move_is_valid_impl(nr: c_int) -> bool {
    unsafe {
        let first = nvim_get_first_tabpage();
        let curtab = nvim_get_curtab();

        // Only one tabpage - can't move
        if first.is_null() || nvim_tabpage_get_next(first).is_null() {
            return false;
        }

        // Find target position
        let mut n = 1;
        let mut tp = first;
        while !tp.is_null() && nvim_tabpage_get_next(tp) != TabpageHandle::null() && n < nr {
            tp = nvim_tabpage_get_next(tp);
            n += 1;
        }

        // Can't move if target is curtab or position right before curtab
        if tp == curtab {
            return false;
        }
        if nr > 0 && !nvim_tabpage_get_next(tp).is_null() && nvim_tabpage_get_next(tp) == curtab {
            return false;
        }

        true
    }
}

/// Get the position where curtab would move to.
fn tabpage_move_get_target_impl(nr: c_int) -> TabpageHandle {
    unsafe {
        let first = nvim_get_first_tabpage();

        if first.is_null() {
            return TabpageHandle::null();
        }

        let mut n = 1;
        let mut tp = first;
        while !tp.is_null() && !nvim_tabpage_get_next(tp).is_null() && n < nr {
            tp = nvim_tabpage_get_next(tp);
            n += 1;
        }
        tp
    }
}

// =============================================================================
// Tabpage Creation Helpers
// =============================================================================

/// Check if a new tabpage can be created.
///
/// Returns error code:
/// - 0: can create
/// - 1: in command-line window
fn can_create_tabpage_impl() -> c_int {
    // The actual cmdwin check needs to be done in C
    // This is a placeholder for the Rust helper structure
    0
}

/// Determine insert position for new tabpage.
///
/// after == 0: after current
/// after == 1: at start (becomes first)
/// after > 1: after tabpage N
fn new_tabpage_insert_position_impl(after: c_int) -> TabpageHandle {
    unsafe {
        let curtab = nvim_get_curtab();
        let first = nvim_get_first_tabpage();

        if after == 1 {
            // Insert at start - return null to indicate first position
            return TabpageHandle::null();
        }

        if after == 0 {
            // Insert after current tabpage
            return curtab;
        }

        // Insert after tabpage N
        let mut tp = first;
        let mut n = 1;
        while !tp.is_null() && n < after {
            if nvim_tabpage_get_next(tp).is_null() {
                break;
            }
            tp = nvim_tabpage_get_next(tp);
            n += 1;
        }
        tp
    }
}

// =============================================================================
// Additional FFI Exports
// =============================================================================

/// FFI: Find target tabpage for goto_tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_goto_target(n: c_int) -> TabpageHandle {
    goto_tabpage_find_target_impl(n)
}

/// FFI: Check if tabpage move is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_move_is_valid(nr: c_int) -> c_int {
    c_int::from(tabpage_move_is_valid_impl(nr))
}

/// FFI: Get target position for tabpage move.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_move_target(nr: c_int) -> TabpageHandle {
    tabpage_move_get_target_impl(nr)
}

/// FFI: Check if tabpage can be created.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_can_create() -> c_int {
    can_create_tabpage_impl()
}

/// FFI: Get insert position for new tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_insert_position(after: c_int) -> TabpageHandle {
    new_tabpage_insert_position_impl(after)
}

// =============================================================================
// Window/Tabpage Lookup
// =============================================================================

extern "C" {
    fn nvim_win_get_handle(wp: WinHandle) -> c_int;
    fn nvim_win_has_winnr(wp: WinHandle, tp: TabpageHandle) -> c_int;
}

/// Find the tabpage number and window number for a window ID.
///
/// Sets `*tabnr` and `*winnr` to the 1-based indices, or 0 if not found.
///
/// Equivalent to C `win_get_tabwin()` (window.c L8080).
fn win_get_tabwin_impl(id: c_int) -> (c_int, c_int) {
    unsafe {
        let mut tnum: c_int = 1;

        let mut tp = nvim_get_first_tabpage();
        while !tp.is_null() {
            let mut wnum: c_int = 1;
            let mut wp = get_tabpage_firstwin(tp);
            while !wp.is_null() {
                if nvim_win_get_handle(wp) == id {
                    if nvim_win_has_winnr(wp, tp) != 0 {
                        return (tnum, wnum);
                    }
                    return (0, 0);
                }
                wnum += nvim_win_has_winnr(wp, tp);
                wp = nvim_win_get_next(wp);
            }
            tnum += 1;
            tp = nvim_tabpage_get_next(tp);
        }

        (0, 0)
    }
}

/// FFI: Find tabpage and window number for a window handle ID.
///
/// Writes the 1-based tab number and window number to the provided pointers.
///
/// # Safety
/// `tabnr` and `winnr` must be valid, non-null pointers to `c_int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_get_tabwin(id: c_int, tabnr: *mut c_int, winnr: *mut c_int) {
    let (t, w) = win_get_tabwin_impl(id);
    if !tabnr.is_null() {
        *tabnr = t;
    }
    if !winnr.is_null() {
        *winnr = w;
    }
}

// Note: The following FFI exports are in lib.rs:
// - rs_tabpage_index - get tabpage index
// - rs_win_find_tabpage - find tabpage containing window
// - rs_valid_tabpage - validate tabpage
// - rs_valid_tabpage_win - validate tabpage has valid window
// - rs_one_tabpage - check if only one tabpage
// - rs_find_tabpage - find tabpage by index

// =============================================================================
// alt_tabpage implementation
// =============================================================================

// Constants for tcl_flags ('tabclose' option)
const TCL_FLAG_LEFT: c_int = 0x01;
const TCL_FLAG_USELAST: c_int = 0x02;

/// Find the alternate tabpage when closing the current one.
///
/// Port of the C `alt_tabpage()` static function.
fn alt_tabpage_impl() -> TabpageHandle {
    unsafe {
        let flags = nvim_win_get_tcl_flags();

        // Use the last accessed tab page, if possible.
        if (flags & TCL_FLAG_USELAST) != 0 {
            let lastused = nvim_get_lastused_tabpage();
            if valid_tabpage_impl(lastused) {
                return lastused;
            }
        }

        let curtab = nvim_get_curtab();
        let first = nvim_get_first_tabpage();

        // Use the next tab page if possible (forward), unless 'left' flag is set
        // or we're already at the first tab.
        let forward = !nvim_tabpage_get_next(curtab).is_null()
            && ((flags & TCL_FLAG_LEFT) == 0 || curtab == first);

        if forward {
            nvim_tabpage_get_next(curtab)
        } else {
            // Use the previous tab page.
            let mut tp = first;
            while !nvim_tabpage_get_next(tp).is_null() && nvim_tabpage_get_next(tp) != curtab {
                tp = nvim_tabpage_get_next(tp);
            }
            tp
        }
    }
}

// =============================================================================
// tabpage_move implementation
// =============================================================================

/// Move the current tab page to after tab page number `nr`.
///
/// Port of the C `tabpage_move()` function.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn tabpage_move_impl(nr: c_int) {
    let first = nvim_get_first_tabpage();
    let curtab = nvim_get_curtab();

    // Only one tabpage or move disallowed: nothing to do.
    if nvim_tabpage_get_next(first).is_null() {
        return;
    }
    if nvim_al_get_tabpage_move_disallowed() != 0 {
        return;
    }

    // Find the target position (tp_dst).
    let mut n: c_int = 1;
    let mut tp = first;
    while !nvim_tabpage_get_next(tp).is_null() && n < nr {
        tp = nvim_tabpage_get_next(tp);
        n += 1;
    }

    // No-op: already at the target position or one before.
    if tp == curtab
        || (nr > 0 && !nvim_tabpage_get_next(tp).is_null() && nvim_tabpage_get_next(tp) == curtab)
    {
        return;
    }

    let tp_dst = tp;

    // Remove curtab from the list.
    if curtab == first {
        let next = nvim_tabpage_get_next(curtab);
        nvim_set_first_tabpage(next);
    } else {
        // Find the tabpage before curtab.
        let mut prev = first;
        let mut tp2 = nvim_tabpage_get_next(first);
        while !tp2.is_null() && tp2 != curtab {
            prev = tp2;
            tp2 = nvim_tabpage_get_next(tp2);
        }
        if tp2.is_null() {
            // "cannot happen"
            return;
        }
        nvim_tabpage_set_next(prev, nvim_tabpage_get_next(curtab));
    }

    // Re-insert curtab at the target position.
    if nr <= 0 {
        // Move to front.
        nvim_tabpage_set_next(curtab, nvim_get_first_tabpage());
        nvim_set_first_tabpage(curtab);
    } else {
        nvim_tabpage_set_next(curtab, nvim_tabpage_get_next(tp_dst));
        nvim_tabpage_set_next(tp_dst, curtab);
    }

    // Signal that the tabline needs redrawing.
    nvim_set_redraw_tabline(1);
}

// =============================================================================
// goto_tabpage implementation
// =============================================================================

/// Navigate to a tab page by number.
///
/// Port of the C `goto_tabpage()` function.
///
/// # Safety
/// Calls C accessor functions including goto_tabpage_tp.
unsafe fn goto_tabpage_impl(n: c_int) {
    if nvim_text_locked() {
        nvim_text_locked_msg();
        return;
    }

    let first = nvim_get_first_tabpage();
    // If there is only one tabpage, it can't work.
    if nvim_tabpage_get_next(first).is_null() {
        if n > 1 {
            nvim_beep_flush();
        }
        return;
    }

    let curtab = nvim_get_curtab();
    let tp;

    if n == 0 {
        // No count: go to next tab page, wrap around end.
        let next = nvim_tabpage_get_next(curtab);
        tp = if next.is_null() { first } else { next };
    } else if n < 0 {
        // "gT": go to previous tab page, wrap around. N times.
        let mut ttp = curtab;
        let mut i = n;
        while i < 0 {
            let mut prev = first;
            // Find the tabpage before ttp
            while !nvim_tabpage_get_next(prev).is_null() && nvim_tabpage_get_next(prev) != ttp {
                prev = nvim_tabpage_get_next(prev);
            }
            // If ttp was first_tabpage, there's no tabpage "before" it
            // in a simple scan - but the C code finds the prev going from
            // first. If first == ttp, the loop terminates immediately with
            // prev = first, so ttp = first (wrap). Actually the C code:
            //   for (tp = first; tp->tp_next != ttp && tp->tp_next != NULL; tp = tp->tp_next) {}
            // This gives prev = the last tab before ttp OR the last tab if ttp is first.
            // If ttp is first, the loop finds the last tab (tp->tp_next is NULL when all checked).
            // Our code: if first == ttp, inner loop doesn't advance, prev stays first.
            // That's correct since prev->tp_next != ttp (first->tp_next != first) would be false only
            // if it's a circular list. Let me re-check:
            // In C: `for (tp=first; tp->tp_next != ttp && tp->tp_next != NULL; tp=tp->tp_next) {}`
            // This iterates while tp->tp_next != ttp (not found prev yet) AND not at end.
            // So it finds the tabpage whose next is ttp, or the last one if ttp is first.
            // Our code finds the same thing.
            ttp = prev;
            i += 1;
        }
        tp = ttp;
    } else if n == 9999 {
        // Go to last tab page.
        let mut t = first;
        while !nvim_tabpage_get_next(t).is_null() {
            t = nvim_tabpage_get_next(t);
        }
        tp = t;
    } else {
        // Go to tab page "n".
        let found = find_tabpage_impl(n);
        if found.is_null() {
            nvim_beep_flush();
            return;
        }
        tp = found;
    }

    nvim_al_goto_tabpage_tp(tp, 1, 1);
}

// =============================================================================
// FFI exports for Phase 2 functions
// =============================================================================

/// FFI: Find the alternate tabpage when closing the current one.
///
/// Replaces C `alt_tabpage()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_alt_tabpage() -> TabpageHandle {
    alt_tabpage_impl()
}

/// FFI: Move the current tab page to after position `nr`.
///
/// Replaces C `tabpage_move()`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_tabpage_move(nr: c_int) {
    tabpage_move_impl(nr);
}

/// FFI: Navigate to tab page number `n`.
///
/// Replaces C `goto_tabpage()`.
///
/// # Safety
/// Calls C accessor functions including goto_tabpage_tp.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_goto_tabpage(n: c_int) {
    goto_tabpage_impl(n);
}

// =============================================================================
// Tabpage Transition Validation
// =============================================================================

/// Check if transitioning to a tabpage would be a no-op.
///
/// Returns true if the target is the current tabpage or invalid.
fn is_tabpage_transition_noop_impl(tp: TabpageHandle) -> bool {
    if tp.is_null() {
        return true;
    }
    unsafe {
        let curtab = nvim_get_curtab();
        tp == curtab
    }
}

/// Check if a tabpage transition is valid (target exists and differs from current).
fn is_tabpage_transition_valid_impl(tp: TabpageHandle) -> bool {
    if tp.is_null() {
        return false;
    }
    unsafe {
        let curtab = nvim_get_curtab();
        if tp == curtab {
            return false;
        }
        valid_tabpage_impl(tp)
    }
}

/// Get the target tabpage for a goto_tabpage_tp call.
///
/// Validates that:
/// 1. Target is not null
/// 2. Target is not current tabpage
/// 3. Target is a valid tabpage
///
/// Returns the valid target or null if invalid.
fn validate_tabpage_transition_impl(tp: TabpageHandle) -> TabpageHandle {
    if is_tabpage_transition_valid_impl(tp) {
        tp
    } else {
        TabpageHandle::null()
    }
}

/// FFI: Check if tabpage transition is no-op.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_transition_is_noop(tp: TabpageHandle) -> c_int {
    c_int::from(is_tabpage_transition_noop_impl(tp))
}

/// FFI: Check if tabpage transition is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_transition_is_valid(tp: TabpageHandle) -> c_int {
    c_int::from(is_tabpage_transition_valid_impl(tp))
}

/// FFI: Validate and get target tabpage.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_validate_transition(tp: TabpageHandle) -> TabpageHandle {
    validate_tabpage_transition_impl(tp)
}

// =============================================================================
// unuse_tabpage / use_tabpage
// =============================================================================

/// Save current window pointers to tab page.
///
/// Port of C `unuse_tabpage()`.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
unsafe fn unuse_tabpage_impl(tp: TabpageHandle) {
    if tp.is_null() {
        return;
    }
    nvim_tabpage_set_topframe(tp, nvim_get_topframe());
    nvim_tabpage_set_firstwin(tp, nvim_get_firstwin());
    nvim_tabpage_set_lastwin(tp, nvim_get_lastwin());
    nvim_tabpage_set_curwin(tp, nvim_get_curwin());
}

/// Restore window pointers from tab page.
///
/// Port of C `use_tabpage()`.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
unsafe fn use_tabpage_impl(tp: TabpageHandle) {
    if tp.is_null() {
        return;
    }
    nvim_set_curtab(tp);
    nvim_set_topframe(nvim_tabpage_get_topframe(tp));
    nvim_set_firstwin(nvim_tabpage_get_firstwin(tp));
    nvim_set_lastwin(nvim_tabpage_get_lastwin(tp));
    nvim_set_curwin(nvim_tabpage_get_curwin(tp));
}

/// FFI export for `unuse_tabpage`.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_unuse_tabpage(tp: TabpageHandle) {
    unuse_tabpage_impl(tp);
}

/// FFI export for `use_tabpage`.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_use_tabpage(tp: TabpageHandle) {
    use_tabpage_impl(tp);
}

/// FFI: Get the tabpage to transition to after closing current.
///
/// Returns the best alternate tabpage when the current is being closed.
/// Prefers lastused_tabpage if valid, otherwise uses neighbor.
#[unsafe(no_mangle)]
pub extern "C" fn rs_tabpage_close_alternate() -> TabpageHandle {
    unsafe {
        // First try lastused
        let lastused = nvim_get_lastused_tabpage();
        if !lastused.is_null() && valid_tabpage_impl(lastused) {
            let curtab = nvim_get_curtab();
            if lastused != curtab {
                return lastused;
            }
        }

        // Fall back to next or previous
        let curtab = nvim_get_curtab();
        let next = nvim_tabpage_get_next(curtab);
        if !next.is_null() {
            return next;
        }

        // Use previous (find it)
        prev_tabpage_impl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabpage_handle_null() {
        let handle = TabpageHandle::null();
        assert!(handle.is_null());
        assert!(!valid_tabpage_impl(handle));
    }

    #[test]
    fn test_null_tabpage_operations() {
        let null_tp = TabpageHandle::null();
        assert_eq!(count_windows_in_tabpage_impl(null_tp), 0);
        assert_eq!(count_nonfloating_in_tabpage_impl(null_tp), 0);
    }

    #[test]
    fn test_null_window_find_tabpage() {
        let null_win = WinHandle::null();
        assert!(win_find_tabpage_impl(null_win).is_null());
    }

    #[test]
    fn test_can_create_tabpage() {
        // Basic check - cmdwin is checked in C
        assert_eq!(can_create_tabpage_impl(), 0);
    }

    #[test]
    fn test_win_get_tabwin_not_found() {
        // With no windows initialized, any ID should return (0, 0)
        let (tabnr, winnr) = win_get_tabwin_impl(99999);
        assert_eq!(tabnr, 0);
        assert_eq!(winnr, 0);
    }
}
