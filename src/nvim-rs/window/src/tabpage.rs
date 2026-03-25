//! Tab page management functions.
//!
//! This module provides Rust implementations of tab page operations
//! from `src/nvim/window.c`.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_int, c_void};

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
    static Rows: c_int;
    static Columns: c_int;
    static mut redraw_tabline: bool;
    fn xfree(ptr: *mut c_void);
}

extern "C" {
    /// Get curtab.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get lastused_tabpage.
    fn nvim_get_lastused_tabpage() -> TabpageHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get tabpage_move_disallowed global.
    fn nvim_al_get_tabpage_move_disallowed() -> c_int;

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
    redraw_tabline = true;
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

/// C export: `tabpage_move` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(export_name = "tabpage_move")]
pub unsafe extern "C" fn tabpage_move(nr: c_int) {
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

/// C export: `goto_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(export_name = "goto_tabpage")]
pub unsafe extern "C" fn goto_tabpage(n: c_int) {
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

/// C export: `unuse_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
#[unsafe(export_name = "unuse_tabpage")]
pub unsafe extern "C" fn unuse_tabpage(tp: TabpageHandle) {
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

/// C export: `use_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// tp must be a valid, non-null tabpage pointer.
#[unsafe(export_name = "use_tabpage")]
pub unsafe extern "C" fn use_tabpage(tp: TabpageHandle) {
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

// =============================================================================
// Phase 2 migrations: close_tabpage, make_tabpages, goto_tabpage_lastused,
// goto_tabpage_win
// =============================================================================

extern "C" {
    /// Get p_tpm (tabpagemax option).
    fn nvim_get_p_tpm() -> i64;

    /// block_autocmds.
    fn nvim_block_autocmds();

    /// unblock_autocmds.
    fn nvim_unblock_autocmds();

    /// win_enter wrapper.
    fn nvim_win_enter(wp: WinHandle, undo_sync: c_int);

    /// rs_win_valid.
    fn rs_win_valid(wp: WinHandle) -> c_int;
}

/// FAIL constant.
const FAIL: c_int = 0;

/// EVENT_* constants matching auevents_enum.generated.h
const EVENT_BUFENTER: c_int = 3;
const EVENT_BUFLEAVE: c_int = 7;
const EVENT_TABENTER: c_int = 110;
const EVENT_TABLEAVE: c_int = 111;
const EVENT_WINENTER: c_int = 136;
const EVENT_WINLEAVE: c_int = 137;
const EVENT_WINNEW: c_int = 138;

/// Close tabpage `tab` which has no windows.
/// There must be another tabpage or this will crash.
/// Equivalent to C `close_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn close_tabpage_impl(tab: TabpageHandle) {
    let first = nvim_get_first_tabpage();

    let ptp = if tab == first {
        let next = nvim_tabpage_get_next(tab);
        nvim_set_first_tabpage(next);
        next
    } else {
        // Find the tabpage before tab.
        let mut p = first;
        while !nvim_tabpage_get_next(p).is_null() && nvim_tabpage_get_next(p) != tab {
            p = nvim_tabpage_get_next(p);
        }
        // assert: nvim_tabpage_get_next(p) == tab
        nvim_tabpage_set_next(p, nvim_tabpage_get_next(tab));
        p
    };

    nvim_al_goto_tabpage_tp(ptp, 0, 0);
    free_tabpage_impl(tab);
}

/// Create up to `maxcount` tabpages with empty windows.
/// Returns the actual number of tabpages created.
/// Equivalent to C `make_tabpages()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn make_tabpages_impl(maxcount: c_int) -> c_int {
    // Limit to 'tabpagemax' tabs.
    #[allow(clippy::cast_possible_truncation)]
    let p_tpm = nvim_get_p_tpm() as c_int;
    let count = maxcount.min(p_tpm);

    // Don't execute autocommands while creating the tab pages.
    nvim_block_autocmds();

    let mut todo = count - 1;
    while todo > 0 {
        if rs_win_new_tabpage(0, std::ptr::null()) == FAIL {
            break;
        }
        todo -= 1;
    }

    nvim_unblock_autocmds();

    // Return actual number of tab pages.
    count - todo
}

/// Go to the last used tabpage if valid.
/// Returns true on success, false if lastused_tabpage is not valid.
/// Equivalent to C `goto_tabpage_lastused()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn goto_tabpage_lastused_impl() -> bool {
    let lastused = nvim_get_lastused_tabpage();
    if !valid_tabpage_impl(lastused) {
        return false;
    }
    nvim_al_goto_tabpage_tp(lastused, 1, 1);
    true
}

/// Enter window `wp` in tab page `tp`.
/// Equivalent to C `goto_tabpage_win()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn goto_tabpage_win_impl(tp: TabpageHandle, wp: WinHandle) {
    nvim_al_goto_tabpage_tp(tp, 1, 1);
    let curtab = nvim_get_curtab();
    if curtab == tp && rs_win_valid(wp) != 0 {
        nvim_win_enter(wp, 1);
    }
}

/// FFI: Close tabpage `tab` which has no windows.
///
/// # Safety
/// tab must be a valid, non-null tabpage with no windows.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_close_tabpage(tab: TabpageHandle) {
    close_tabpage_impl(tab);
}

/// C export: `close_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// tab must be a valid, non-null tabpage with no windows.
#[unsafe(export_name = "close_tabpage")]
pub unsafe extern "C" fn close_tabpage(tab: TabpageHandle) {
    close_tabpage_impl(tab);
}

/// FFI: Create up to `maxcount` tabpages with empty windows.
///
/// Returns the actual number of tabpages.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_make_tabpages(maxcount: c_int) -> c_int {
    make_tabpages_impl(maxcount)
}

/// C export: `make_tabpages` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[unsafe(export_name = "make_tabpages")]
pub unsafe extern "C" fn make_tabpages(maxcount: c_int) -> c_int {
    make_tabpages_impl(maxcount)
}

/// FFI: Go to the last used tabpage.
///
/// Returns 1 on success, 0 if last used tabpage is not valid.
///
/// # Safety
/// Calls C accessor functions.
#[allow(clippy::must_use_candidate)]
#[unsafe(export_name = "goto_tabpage_lastused")]
pub unsafe extern "C" fn rs_goto_tabpage_lastused() -> bool {
    goto_tabpage_lastused_impl()
}

/// FFI: Enter window `wp` in tab page `tp`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_goto_tabpage_win(tp: TabpageHandle, wp: WinHandle) {
    goto_tabpage_win_impl(tp, wp);
}

/// C export: `goto_tabpage_win` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(export_name = "goto_tabpage_win")]
pub unsafe extern "C" fn goto_tabpage_win(tp: TabpageHandle, wp: WinHandle) {
    goto_tabpage_win_impl(tp, wp);
}

// =============================================================================
// may_open_tabpage implementation
// =============================================================================

extern "C" {
    /// Get postponed_split_tab global.
    fn nvim_get_postponed_split_tab() -> c_int;

    /// Set postponed_split_tab global.
    fn nvim_set_postponed_split_tab(val: c_int);

    /// Get cmdmod.cmod_tab.
    fn nvim_get_cmdmod_tab() -> c_int;

    /// Set cmdmod.cmod_tab.
    fn nvim_set_cmdmod_tab(val: c_int);

    /// Call apply_autocmds(EVENT_TABNEWENTERED, ...).
    fn nvim_apply_autocmds_tabnewentered();
}

/// Open a new tab page if `:tab cmd` was used.
///
/// Port of C `may_open_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn may_open_tabpage_impl() -> c_int {
    let cmod_tab = nvim_get_cmdmod_tab();
    let postponed = nvim_get_postponed_split_tab();
    let n = if cmod_tab == 0 { postponed } else { cmod_tab };

    if n == 0 {
        return FAIL;
    }

    nvim_set_cmdmod_tab(0); // reset it to avoid doing it twice
    nvim_set_postponed_split_tab(0);

    let status = rs_win_new_tabpage(n, std::ptr::null());
    if status != FAIL {
        nvim_apply_autocmds_tabnewentered();
    }
    status
}

/// FFI: Open a new tab page if `:tab cmd` was used.
///
/// Replaces C `may_open_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_may_open_tabpage() -> c_int {
    may_open_tabpage_impl()
}

// =============================================================================
// Phase 7: leave_tabpage, enter_tabpage, goto_tabpage_tp
// =============================================================================

extern "C" {
    // leave_tabpage dependencies (cross-crate)
    fn rs_reset_VIsual_and_resel();
    fn nvim_get_curbuf() -> crate::BufHandle;
    fn nvim_apply_autocmds_event(event: c_int);
    fn nvim_reset_dragwin();
    fn nvim_tabpage_set_prevwin(tp: TabpageHandle, wp: WinHandle);
    fn nvim_tabpage_set_old_rows_avail(tp: TabpageHandle, val: c_int);
    fn nvim_tabpage_get_old_columns(tp: TabpageHandle) -> c_int;
    fn nvim_tabpage_set_old_columns(tp: TabpageHandle, val: c_int);
    fn nvim_get_rows_avail() -> c_int;
    fn nvim_get_prevwin() -> WinHandle;
    fn nvim_set_firstwin_null();
    fn nvim_set_lastwin_null();

    // enter_tabpage dependencies
    static mut p_ch: i64;
    fn nvim_get_curtab_ch_used() -> c_int;
    fn nvim_set_cmdheight_for_tabpage(new_ch: i64);
    fn nvim_set_curtab_ch_used(val: i64);
    fn nvim_tabpage_get_prevwin(tp: TabpageHandle) -> WinHandle;
    fn nvim_tabpage_get_firstwin_winrow(tp: TabpageHandle) -> c_int;
    fn nvim_set_prevwin(wp: WinHandle);
    fn nvim_win_float_update_statusline();
    fn nvim_set_diff_need_scrollbind(val: bool);
    fn nvim_tabpage_get_old_rows_avail(tp: TabpageHandle) -> c_int;
    fn nvim_get_starting() -> c_int;
    fn nvim_set_lastused_tabpage_from_rust(tp: TabpageHandle);
    fn nvim_redraw_all_later(type_: c_int);

    // goto_tabpage_tp dependencies
    static cmdwin_type: c_int;
    fn nvim_emsg_e_cmdwin();
    fn nvim_set_keep_msg_null();
    fn nvim_set_skip_win_fix_scroll(val: c_int);
    fn nvim_win_get_buffer(wp: WinHandle) -> crate::BufHandle;
}

// WEE_* flags for win_enter_ext (must match C enum)
const WEE_CURWIN_INVALID: c_int = 0x02;
const WEE_TRIGGER_ENTER_AUTOCMDS: c_int = 0x08;
const WEE_TRIGGER_LEAVE_AUTOCMDS: c_int = 0x10;

// OK/FAIL constants
const OK: c_int = 1;
const LEAVE_FAIL: c_int = 0;

/// Rust port of C static `leave_tabpage()`.
///
/// Prepares for leaving the current tab page. Fires BufLeave/WinLeave/TabLeave
/// autocmds if requested and checks curtab stability after each. Saves current
/// window pointers to the tabpage and clears firstwin/lastwin.
///
/// Returns OK (1) on success, FAIL (0) if autocmds changed curtab.
///
/// # Safety
/// Calls C accessor functions and fires autocmds.
unsafe fn leave_tabpage_impl(new_curbuf: crate::BufHandle, trigger_leave: bool) -> c_int {
    let tp = nvim_get_curtab();

    crate::focus::rs_leaving_window(nvim_get_curwin());
    rs_reset_VIsual_and_resel();

    if trigger_leave {
        let curbuf = nvim_get_curbuf();
        if new_curbuf != curbuf {
            nvim_apply_autocmds_event(EVENT_BUFLEAVE);
            if nvim_get_curtab() != tp {
                return LEAVE_FAIL;
            }
        }
        nvim_apply_autocmds_event(EVENT_WINLEAVE);
        if nvim_get_curtab() != tp {
            return LEAVE_FAIL;
        }
        nvim_apply_autocmds_event(EVENT_TABLEAVE);
        if nvim_get_curtab() != tp {
            return LEAVE_FAIL;
        }
    }

    nvim_reset_dragwin();
    nvim_tabpage_set_curwin(tp, nvim_get_curwin());
    nvim_tabpage_set_prevwin(tp, nvim_get_prevwin());
    nvim_tabpage_set_firstwin(tp, nvim_get_firstwin());
    nvim_tabpage_set_lastwin(tp, nvim_get_lastwin());
    nvim_tabpage_set_old_rows_avail(tp, nvim_get_rows_avail());
    if nvim_tabpage_get_old_columns(tp) != -1 {
        nvim_tabpage_set_old_columns(tp, Columns);
    }
    nvim_set_firstwin_null();
    nvim_set_lastwin_null();
    OK
}

/// FFI export: Rust replacement for C static `leave_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_leave_tabpage(
    new_curbuf: crate::BufHandle,
    trigger_leave: c_int,
) -> c_int {
    leave_tabpage_impl(new_curbuf, trigger_leave != 0)
}

// UPD_NOT_VALID constant (matches C define)
const UPD_NOT_VALID: c_int = 2;

/// Rust port of C static `enter_tabpage()`.
///
/// Switches to tab page `tp`. Syncs cmdheight, fires win_enter_ext, recomputes
/// layout, updates lastused_tabpage, and fires TabEnter/BufEnter autocmds.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn enter_tabpage_impl(
    tp: TabpageHandle,
    old_curbuf: crate::BufHandle,
    trigger_enter: bool,
    trigger_leave: bool,
) {
    // Capture the w_winrow of tp->tp_firstwin before use_tabpage changes firstwin.
    let old_off_winrow = nvim_tabpage_get_firstwin_winrow(tp);
    let next_prevwin = nvim_tabpage_get_prevwin(tp);
    let old_curtab = nvim_get_curtab();

    use_tabpage_impl(tp);

    if old_curtab != nvim_get_curtab() {
        crate::ui_flush::rs_tabpage_check_windows(old_curtab);
        let tp_ch_used = i64::from(nvim_get_curtab_ch_used());
        if p_ch != tp_ch_used {
            // Temporarily swap tp_ch_used with p_ch so set_option_value
            // sees the stored value, then restore after.
            nvim_set_curtab_ch_used(p_ch);
            nvim_set_cmdheight_for_tabpage(tp_ch_used);
        }
    }

    let mut enter_flags = WEE_CURWIN_INVALID;
    if trigger_enter {
        enter_flags |= WEE_TRIGGER_ENTER_AUTOCMDS;
    }
    if trigger_leave {
        enter_flags |= WEE_TRIGGER_LEAVE_AUTOCMDS;
    }
    crate::enter::rs_win_enter_ext(nvim_tabpage_get_curwin(tp), enter_flags);
    nvim_set_prevwin(next_prevwin);

    crate::statusline::rs_last_status(0);
    nvim_win_float_update_statusline();
    crate::rs_win_comp_pos();
    nvim_set_diff_need_scrollbind(true);
    nvim_reset_dragwin();

    // The tabpage line may have appeared or disappeared; also check ROWS_AVAIL.
    let curtab = nvim_get_curtab();
    let firstwin_winrow = nvim_tabpage_get_firstwin_winrow(curtab);
    if nvim_tabpage_get_old_rows_avail(curtab) != nvim_get_rows_avail()
        || old_off_winrow != firstwin_winrow
    {
        crate::resize::screen::rs_win_new_screen_rows();
    }
    if nvim_tabpage_get_old_columns(curtab) != Columns {
        if nvim_get_starting() == 0 {
            crate::resize::screen::rs_win_new_screen_cols();
            nvim_tabpage_set_old_columns(curtab, Columns);
        } else {
            nvim_tabpage_set_old_columns(curtab, -1);
        }
    }

    nvim_set_lastused_tabpage_from_rust(old_curtab);

    if trigger_enter {
        nvim_apply_autocmds_event(EVENT_TABENTER);
        let curbuf = nvim_get_curbuf();
        if old_curbuf != curbuf {
            nvim_apply_autocmds_event(EVENT_BUFENTER);
        }
    }

    nvim_redraw_all_later(UPD_NOT_VALID);
}

/// FFI export: Rust replacement for C static `enter_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_enter_tabpage(
    tp: TabpageHandle,
    old_curbuf: crate::BufHandle,
    trigger_enter: c_int,
    trigger_leave: c_int,
) {
    enter_tabpage_impl(tp, old_curbuf, trigger_enter != 0, trigger_leave != 0);
}

/// Rust port of C `goto_tabpage_tp()`.
///
/// Orchestrates tab page switching: fires leave autocmds via leave_tabpage,
/// then enter autocmds via enter_tabpage.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn goto_tabpage_tp_impl(tp: TabpageHandle, trigger_enter: bool, trigger_leave: bool) {
    if (trigger_enter || trigger_leave) && cmdwin_type != 0 {
        nvim_emsg_e_cmdwin();
        return;
    }

    nvim_set_keep_msg_null();

    nvim_set_skip_win_fix_scroll(1);

    let curtab = nvim_get_curtab();
    if tp != curtab {
        // Get the buffer that will be current in tp
        let tp_curwin = nvim_tabpage_get_curwin(tp);
        let new_curbuf = nvim_win_get_buffer(tp_curwin);
        let leave_ok = leave_tabpage_impl(new_curbuf, trigger_leave);
        if leave_ok == OK {
            if valid_tabpage_impl(tp) {
                enter_tabpage_impl(tp, nvim_get_curbuf(), trigger_enter, trigger_leave);
            } else {
                let curtab2 = nvim_get_curtab();
                enter_tabpage_impl(curtab2, nvim_get_curbuf(), trigger_enter, trigger_leave);
            }
        }
    }

    nvim_set_skip_win_fix_scroll(0);
}

/// FFI export: Rust replacement for C `goto_tabpage_tp()`.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "goto_tabpage_tp"]
pub unsafe extern "C" fn rs_goto_tabpage_tp_impl(
    tp: TabpageHandle,
    trigger_enter: c_int,
    trigger_leave: c_int,
) {
    goto_tabpage_tp_impl(tp, trigger_enter != 0, trigger_leave != 0);
}

// =============================================================================
// Phase 8: win_new_tabpage migration
// =============================================================================

extern "C" {
    /// Allocate first window in a new tabpage from oldwin.
    /// Returns OK (1) or FAIL (0).
    fn rs_win_alloc_firstwin(oldwin: WinHandle) -> c_int;

    /// Copy tp_localdir from src to dst (xstrdup, NULL-safe).
    fn nvim_tabpage_copy_localdir(dst: TabpageHandle, src: TabpageHandle);

    /// Free a tabpage allocated on the failure path.
    #[link_name = "nvim_xfree_tabpage_raw"]
    fn nvim_xfree_tabpage(tp: TabpageHandle);

    /// If curbuf has a terminal, call terminal_check_size.
    fn nvim_curbuf_terminal_check_size();

    /// Fire EVENT_TABNEW autocmd with optional filename.
    fn nvim_apply_autocmds_tabnew(filename: *const u8);

    /// Set w_winrow on a window.
    fn nvim_win_set_winrow(wp: WinHandle, val: c_int);

    /// Set w_prev_winrow on a window.
    fn nvim_win_set_prev_winrow(wp: WinHandle, val: c_int);

    /// rs_tabline_height.
    fn rs_tabline_height() -> c_int;

    /// rs_win_comp_scroll for curwin.
    fn rs_win_comp_scroll(wp: WinHandle);
}

/// UPD_NOT_VALID constant (matches C define = 40).
const UPD_NOT_VALID_TAB: c_int = 40;

/// Rust replacement for C `win_new_tabpage`.
///
/// Creates a new tab page with one window, links it into the tabpage list,
/// fires WinNew/WinEnter/TabNew/TabEnter autocmds.
///
/// # Safety
/// Calls C accessor functions. Must only be called from the main Neovim thread.
unsafe fn win_new_tabpage_impl(after: c_int, filename: *const u8) -> c_int {
    // Check for command-line window
    if cmdwin_type != 0 {
        nvim_emsg_e_cmdwin();
        return FAIL;
    }

    let old_curtab = nvim_get_curtab();

    // Allocate the new tabpage
    let newtp = alloc_tabpage_impl();

    // Leave the current tabpage (fires BufLeave/WinLeave/TabLeave autocmds)
    let curbuf = nvim_get_curbuf();
    if leave_tabpage_impl(curbuf, true) == FAIL {
        nvim_xfree_tabpage(newtp);
        return FAIL;
    }

    // Copy tp_localdir from old curtab
    nvim_tabpage_copy_localdir(newtp, old_curtab);

    // Set curtab to new tabpage
    nvim_set_curtab(newtp);

    // Allocate first window for the new tabpage
    let old_curwin = nvim_tabpage_get_curwin(old_curtab);
    if rs_win_alloc_firstwin(old_curwin) == OK {
        // Link new tabpage into the list
        if after == 1 {
            // New tab page becomes the first one
            nvim_tabpage_set_next(newtp, nvim_get_first_tabpage());
            nvim_set_first_tabpage(newtp);
        } else {
            // Find insert position
            let insert_after = if after == 0 {
                old_curtab
            } else {
                // after > 1: insert after tab N
                let mut n = 2;
                let mut tp = nvim_get_first_tabpage();
                while !nvim_tabpage_get_next(tp).is_null() && n < after {
                    tp = nvim_tabpage_get_next(tp);
                    n += 1;
                }
                tp
            };
            nvim_tabpage_set_next(newtp, nvim_tabpage_get_next(insert_after));
            nvim_tabpage_set_next(insert_after, newtp);
        }

        // Set tp_firstwin / tp_lastwin / tp_curwin to curwin
        let curwin = nvim_get_curwin();
        nvim_tabpage_set_firstwin(newtp, curwin);
        nvim_tabpage_set_lastwin(newtp, curwin);
        nvim_tabpage_set_curwin(newtp, curwin);

        // Initialize window and frame sizes
        crate::resize::screen::rs_win_init_size();
        let firstwin = nvim_get_firstwin();
        let tabline_row = rs_tabline_height();
        nvim_win_set_winrow(firstwin, tabline_row);
        nvim_win_set_prev_winrow(firstwin, tabline_row);
        rs_win_comp_scroll(curwin);

        // Set tp_topframe to topframe
        nvim_tabpage_set_topframe(newtp, nvim_get_topframe());

        // Update status line
        crate::statusline::rs_last_status(0);

        // Terminal size check
        nvim_curbuf_terminal_check_size();

        // Schedule a full redraw
        nvim_redraw_all_later(UPD_NOT_VALID_TAB);

        // Check windows in old tabpage
        crate::ui_flush::rs_tabpage_check_windows(old_curtab);

        // Record old curtab as lastused
        nvim_set_lastused_tabpage_from_rust(old_curtab);

        // entering_window for the new curwin
        crate::focus::rs_entering_window(curwin);

        // Fire autocmds in order: WinNew, WinEnter, TabNew, TabEnter
        nvim_apply_autocmds_event(EVENT_WINNEW);
        nvim_apply_autocmds_event(EVENT_WINENTER);
        nvim_apply_autocmds_tabnew(filename);
        nvim_apply_autocmds_event(EVENT_TABENTER);

        return OK;
    }

    // Failed: restore previous tabpage
    let curbuf2 = nvim_get_curbuf();
    enter_tabpage_impl(nvim_get_curtab(), curbuf2, true, true);
    FAIL
}

/// FFI: Rust replacement for C `win_new_tabpage`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_win_new_tabpage(after: c_int, filename: *const u8) -> c_int {
    win_new_tabpage_impl(after, filename)
}

/// C export: `win_new_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions.
#[must_use]
#[unsafe(export_name = "win_new_tabpage")]
pub unsafe extern "C" fn win_new_tabpage(after: c_int, filename: *const u8) -> c_int {
    win_new_tabpage_impl(after, filename)
}

// =============================================================================
// Phase 12: rs_alloc_tabpage + rs_free_tabpage + rs_new_frame
// =============================================================================

extern "C" {
    /// Allocate raw tabpage_T (xcalloc only).
    fn nvim_alloc_tabpage_raw() -> TabpageHandle;

    /// Init handle field and insert into tabpage_handles map.
    fn nvim_tabpage_init_handle(tp: TabpageHandle);

    /// Init tp_vars dict and t: variable scope.
    fn nvim_tabpage_init_vars(tp: TabpageHandle);

    /// Set tp_diff_invalid.
    fn nvim_tabpage_set_diff_invalid(tp: TabpageHandle, val: c_int);

    /// Set tp_ch_used = p_ch.
    fn nvim_tabpage_set_ch_used_from_p_ch(tp: TabpageHandle);

    /// Remove tp from tabpage_handles map.
    fn nvim_tabpage_pmap_del(tp: TabpageHandle);

    /// Clear t: variables and unref the dict.
    fn nvim_tabpage_clear_vars(tp: TabpageHandle);

    /// xfree the raw tabpage struct.
    fn nvim_xfree_tabpage_raw(tp: TabpageHandle);

    /// Get lastused_tabpage.
    fn nvim_get_lastused_tabpage_raw() -> TabpageHandle;

    /// Set lastused_tabpage = NULL.
    fn nvim_set_lastused_tabpage_null();

    /// Clear diff state for a tabpage (from diff crate).
    fn rs_diff_clear(tp: TabpageHandle);

    /// Clear snapshot slot `idx` for tabpage `tp`.
    fn rs_clear_snapshot(tp: TabpageHandle, idx: c_int);
}

/// SNAP_COUNT constant (number of snapshot slots per tabpage).
const SNAP_COUNT: c_int = 2;

/// Allocate and initialize a new tabpage_T.
///
/// Port of C `alloc_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
unsafe fn alloc_tabpage_impl() -> TabpageHandle {
    let tp = nvim_alloc_tabpage_raw();
    nvim_tabpage_init_handle(tp);
    nvim_tabpage_init_vars(tp);
    nvim_tabpage_set_diff_invalid(tp, 1);
    nvim_tabpage_set_ch_used_from_p_ch(tp);
    tp
}

/// Free a tabpage and all its resources.
///
/// Port of C `free_tabpage()`.
///
/// # Safety
/// Calls C accessor functions. `tp` must be a valid, non-null tabpage.
unsafe fn free_tabpage_impl(tp: TabpageHandle) {
    nvim_tabpage_pmap_del(tp);
    rs_diff_clear(tp);
    for idx in 0..SNAP_COUNT {
        rs_clear_snapshot(tp, idx);
    }
    nvim_tabpage_clear_vars(tp);

    // Clear lastused_tabpage if it points to this tabpage.
    if nvim_get_lastused_tabpage_raw() == tp {
        nvim_set_lastused_tabpage_null();
    }

    let ts = tp.as_tabpage_mut();
    xfree(ts.tp_localdir.cast::<c_void>());
    xfree(ts.tp_prevdir.cast::<c_void>());
    nvim_xfree_tabpage_raw(tp);
}

/// FFI: Allocate and initialize a new tabpage_T.
///
/// Replaces C `alloc_tabpage()`.
///
/// # Safety
/// Calls C accessor functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_alloc_tabpage() -> TabpageHandle {
    alloc_tabpage_impl()
}

/// FFI: Free a tabpage and all its resources.
///
/// Replaces C `free_tabpage()`.
///
/// # Safety
/// Calls C accessor functions. `tp` must be a valid, non-null tabpage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_free_tabpage(tp: TabpageHandle) {
    free_tabpage_impl(tp);
}

/// C export: `free_tabpage` — eliminates the C thin wrapper.
///
/// # Safety
/// Calls C accessor functions. `tp` must be a valid, non-null tabpage.
#[unsafe(export_name = "free_tabpage")]
pub unsafe extern "C" fn free_tabpage(tp: TabpageHandle) {
    free_tabpage_impl(tp);
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
