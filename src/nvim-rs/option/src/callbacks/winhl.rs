//! Phase 3: winhighlight option callback
//!
//! This module ports `parse_winhl_opt` and `did_set_winhighlight` from
//! `option_shim.c` and `optionstr.c` to Rust.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int, c_void};

use super::{callback_ok, CallbackResult};

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // optset_T field accessors
    fn nvim_optset_get_varp_str(args: *const c_void) -> *const c_char;
    fn nvim_optset_get_win(args: *const c_void) -> crate::WinHandle;
    fn nvim_optset_get_varp_ptr(args: *const c_void) -> *const c_void;

    // Window winhighlight field accessors
    fn nvim_win_get_ns_hl_winhl(win: crate::WinHandle) -> c_int;
    fn nvim_win_set_ns_hl_winhl(win: crate::WinHandle, val: c_int);
    fn nvim_win_get_ns_hl(win: crate::WinHandle) -> c_int;
    fn nvim_win_set_ns_hl(win: crate::WinHandle, val: c_int);
    fn nvim_win_set_hl_needs_update(win: crate::WinHandle, val: bool);
    fn nvim_win_get_p_winhl_addr(win: crate::WinHandle) -> *const c_void;

    // Namespace / highlight operations
    /// Creates or refreshes the per-window namespace for winhighlight.
    /// Returns the namespace ID (creates it if w_ns_hl_winhl == 0,
    /// otherwise bumps dp->hl_valid and returns the existing id).
    fn nvim_winhl_ns_prepare(win: crate::WinHandle) -> c_int;

    /// Call syn_check_group(name, len) and return the group id.
    #[link_name = "syn_check_group"]
    fn nvim_syn_check_group_for_winhl(name: *const c_char, len: usize) -> c_int;

    /// Call rs_ns_hl_def(ns_id, hl_id_link, attrs_with_global, hl_id)
    fn nvim_winhl_ns_hl_def(ns_hl: c_int, hl_id_link: c_int, hl_id: c_int);

    /// Return a pointer to the empty string option (empty_string_option).
    /// (Already exists in window_shim.c as returning char*)
    fn nvim_get_empty_string_option() -> *mut c_char;

    /// Return win->w_p_winhl (the current winhighlight option string).
    fn nvim_win_get_p_winhl(win: crate::WinHandle) -> *const c_char;

    /// Find first occurrence of c in p, or NUL terminator.
    #[link_name = "xstrchrnul"]
    fn rs_xstrchrnul(p: *const c_char, c: c_char) -> *mut c_char;
}

// =============================================================================
// Implementation
// =============================================================================

/// Port of C `parse_winhl_opt(winhl, wp)`.
///
/// Parses a `winhighlight` string like `"Normal:MyNormal,Visual:MyVisual"` and
/// registers the highlight overrides in the window's per-window namespace.
///
/// Returns true on success, false on invalid input.
///
/// # Safety
/// - `winhl` may be NULL (falls back to empty string).
/// - `win` may be NULL (validation-only mode, no side effects).
#[no_mangle]
pub unsafe extern "C" fn rs_parse_winhl_opt(winhl: *const c_char, win: crate::WinHandle) -> bool {
    // Choose which string to parse
    let p_start: *const c_char = if !winhl.is_null() {
        winhl
    } else if !win.is_null() {
        nvim_win_get_p_winhl(win)
    } else {
        nvim_get_empty_string_option().cast()
    };

    // If w_ns_hl_winhl < 0, 'winhighlight' shouldn't be used for this window.
    // Only validate; do not apply side effects.
    let win_effective: crate::WinHandle = if !win.is_null() && nvim_win_get_ns_hl_winhl(win) < 0 {
        std::ptr::null_mut()
    } else {
        win
    };

    // Empty string: clear the namespace link and mark update needed
    if *p_start == 0 {
        if !win_effective.is_null() {
            let ns_hl_winhl = nvim_win_get_ns_hl_winhl(win_effective);
            let ns_hl = nvim_win_get_ns_hl(win_effective);
            if ns_hl_winhl > 0 && ns_hl == ns_hl_winhl {
                nvim_win_set_ns_hl(win_effective, 0);
                nvim_win_set_hl_needs_update(win_effective, true);
            }
        }
        return true;
    }

    // Prepare namespace (create or invalidate existing) if we're applying changes
    let ns_hl = if win_effective.is_null() {
        0
    } else {
        let ns_id = nvim_winhl_ns_prepare(win_effective);
        nvim_win_set_ns_hl(win_effective, ns_id);
        nvim_win_set_ns_hl_winhl(win_effective, ns_id);
        ns_id
    };

    // Parse "HlFrom:HlTo" entries
    let mut p = p_start;
    while *p != 0 {
        let colon = {
            let mut q = p;
            while *q != 0 && *q != b':' as c_char {
                q = q.add(1);
            }
            if *q == 0 {
                return false; // no colon found
            }
            q
        };

        let nlen = colon.offset_from(p) as usize;
        let hi = colon.add(1); // start of HlTo name
        let commap = rs_xstrchrnul(hi, b',' as c_char);
        let len = commap.offset_from(hi) as usize;

        // Validate HlTo (may be empty to clear the link)
        let hl_id = if len > 0 {
            let id = nvim_syn_check_group_for_winhl(hi, len);
            if id == 0 {
                return false;
            }
            id
        } else {
            -1 // empty HlTo is invalid per C implementation
        };

        // Validate HlFrom
        let hl_id_link = if nlen > 0 {
            let id = nvim_syn_check_group_for_winhl(p, nlen);
            if id == 0 {
                return false;
            }
            id
        } else {
            0
        };

        // Apply highlight definition if we have a valid window
        if !win_effective.is_null() {
            nvim_winhl_ns_hl_def(ns_hl, hl_id_link, hl_id);
        }

        // Advance past comma (or end of string)
        p = if *commap != 0 {
            commap.add(1)
        } else {
            c"".as_ptr()
        };
    }

    if !win_effective.is_null() {
        nvim_win_set_hl_needs_update(win_effective, true);
    }
    true
}

/// Callback for 'winhighlight' option.
/// Validates and applies `winhighlight` highlight overrides.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_winhighlight(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp_ptr(args);
    let val = nvim_optset_get_varp_str(args);

    // Pass win only when varp IS win->w_p_winhl (mirrors C behavior)
    let win_for_parse: crate::WinHandle = if win.is_null() {
        std::ptr::null_mut()
    } else {
        let winhl_addr = nvim_win_get_p_winhl_addr(win);
        if varp == winhl_addr {
            win
        } else {
            std::ptr::null_mut()
        }
    };

    if !rs_parse_winhl_opt(val, win_for_parse) {
        return c"E474: Invalid argument".as_ptr();
    }
    callback_ok()
}
