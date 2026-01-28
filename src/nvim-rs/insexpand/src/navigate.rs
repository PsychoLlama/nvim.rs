//! Match navigation support.
//!
//! This module provides helper functions for navigating through completion matches.

use std::os::raw::{c_int, c_void};

use crate::match_list::ComplMatch;

// C accessor functions
extern "C" {
    fn nvim_compl_get_first_match() -> ComplMatch;

    fn nvim_compl_match_get_next(m: ComplMatch) -> ComplMatch;
    fn nvim_compl_match_get_prev(m: ComplMatch) -> ComplMatch;

    fn nvim_compl_is_first_match(m: ComplMatch) -> c_int;
    fn nvim_compl_match_at_original_text(m: ComplMatch) -> c_int;

    fn nvim_get_compl_direction() -> c_int;
}

// Direction constants
const FORWARD: c_int = 1;
#[allow(dead_code)]
const BACKWARD: c_int = -1;

/// Check if a match is at the original text position.
#[inline]
unsafe fn match_at_original_text(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_match_at_original_text(m) != 0
}

/// Check if a match is the first match.
#[inline]
unsafe fn is_first_match(m: ComplMatch) -> bool {
    !m.is_null() && nvim_compl_is_first_match(m) != 0
}

/// Navigate to the next match in the list.
///
/// Skips the original text entry. Returns null if no next match.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_next(current: *mut c_void) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return std::ptr::null_mut();
    }

    let mut next = nvim_compl_match_get_next(m);

    // Skip original text entry
    while !next.is_null() && match_at_original_text(next) {
        // Check for wrap-around
        if is_first_match(next) {
            next = nvim_compl_match_get_next(next);
            if is_first_match(next) {
                return std::ptr::null_mut(); // Only original text in list
            }
        } else {
            next = nvim_compl_match_get_next(next);
        }
    }

    next.0
}

/// Navigate to the previous match in the list.
///
/// Skips the original text entry. Returns null if no previous match.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_prev(current: *mut c_void) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return std::ptr::null_mut();
    }

    let mut prev = nvim_compl_match_get_prev(m);

    // Skip original text entry
    while !prev.is_null() && match_at_original_text(prev) {
        prev = nvim_compl_match_get_prev(prev);
        // Check for wrap-around
        if is_first_match(prev) {
            return std::ptr::null_mut();
        }
    }

    prev.0
}

/// Navigate in the current direction.
///
/// Uses the compl_direction to decide forward or backward.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_direction(current: *mut c_void) -> *mut c_void {
    let dir = nvim_get_compl_direction();
    if dir == FORWARD {
        rs_navigate_next(current)
    } else {
        rs_navigate_prev(current)
    }
}

/// Navigate opposite to the current direction.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_opposite(current: *mut c_void) -> *mut c_void {
    let dir = nvim_get_compl_direction();
    if dir == FORWARD {
        rs_navigate_prev(current)
    } else {
        rs_navigate_next(current)
    }
}

/// Get the first non-original match in the list.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_first_match() -> *mut c_void {
    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return std::ptr::null_mut();
    }

    // First is usually the original text, so skip it
    let next = nvim_compl_match_get_next(first);
    if next.is_null() || is_first_match(next) {
        // Only original text in list
        return std::ptr::null_mut();
    }

    next.0
}

/// Check if the current match is at the end of the list.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_at_end(current: *mut c_void) -> c_int {
    let m = ComplMatch(current);
    if m.is_null() {
        return 1;
    }

    let next = nvim_compl_match_get_next(m);
    c_int::from(next.is_null() || is_first_match(next))
}

/// Check if the current match is at the start of the list.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_at_start(current: *mut c_void) -> c_int {
    let m = ComplMatch(current);
    if m.is_null() {
        return 1;
    }

    // We're at start if previous is null or is the first match (original text)
    let prev = nvim_compl_match_get_prev(m);
    c_int::from(prev.is_null() || is_first_match(prev))
}

// =============================================================================
// Phase 2: Extended Navigation Functions
// =============================================================================

/// Navigate by count in the given direction.
///
/// Returns the match after moving `count` positions in `dir` direction.
/// Positive dir is forward, negative is backward.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_by_count(
    start: *mut c_void,
    count: c_int,
    dir: c_int,
) -> *mut c_void {
    let mut m = ComplMatch(start);
    if m.is_null() || count == 0 {
        return start;
    }

    let is_forward = dir >= 0;
    let mut remaining = count.abs();

    while remaining > 0 && !m.is_null() {
        let next_m = if is_forward {
            nvim_compl_match_get_next(m)
        } else {
            nvim_compl_match_get_prev(m)
        };

        if next_m.is_null() {
            break;
        }

        // Skip original text
        if match_at_original_text(next_m) {
            m = next_m;
            continue;
        }

        // Check for wrap-around
        if is_first_match(next_m) && !is_forward {
            break; // Don't wrap when going backward
        }

        m = next_m;
        remaining -= 1;
    }

    m.0
}

/// Navigate to the match that wraps around in the given direction.
///
/// Used for CTRL-N/CTRL-P behavior when the list is cyclic.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_wrap(current: *mut c_void, dir: c_int) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return std::ptr::null_mut();
    }

    let first = nvim_compl_get_first_match();
    if first.is_null() {
        return std::ptr::null_mut();
    }

    let is_forward = dir >= 0;

    if is_forward {
        // Going forward: next match, or wrap to start
        let next = nvim_compl_match_get_next(m);
        if next.is_null() || is_first_match(next) {
            // Wrap to first non-original match
            let after_first = nvim_compl_match_get_next(first);
            if !after_first.is_null() && !match_at_original_text(after_first) {
                return after_first.0;
            }
            // If only original text, stay at current
            return current;
        }
        // Skip original text
        if match_at_original_text(next) {
            let after = nvim_compl_match_get_next(next);
            if !after.is_null() {
                return after.0;
            }
        }
        next.0
    } else {
        // Going backward: prev match, or wrap to end
        let prev = nvim_compl_match_get_prev(m);
        if prev.is_null() {
            return current;
        }
        // Skip original text
        if match_at_original_text(prev) {
            let before = nvim_compl_match_get_prev(prev);
            if !before.is_null() {
                return before.0;
            }
        }
        prev.0
    }
}

/// Check if navigating would cause a wrap-around.
///
/// Returns true if navigating in `dir` from `current` would wrap.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_would_wrap(current: *mut c_void, dir: c_int) -> c_int {
    let m = ComplMatch(current);
    if m.is_null() {
        return 0;
    }

    let is_forward = dir >= 0;

    if is_forward {
        let next = nvim_compl_match_get_next(m);
        c_int::from(next.is_null() || is_first_match(next))
    } else {
        let prev = nvim_compl_match_get_prev(m);
        c_int::from(prev.is_null() || (is_first_match(m) && match_at_original_text(prev)))
    }
}

/// Get the effective current match for display.
///
/// If the current match is the original text, returns the first real match
/// based on direction. Otherwise returns the current match.
#[no_mangle]
pub unsafe extern "C" fn rs_navigate_effective_match(
    current: *mut c_void,
    dir: c_int,
) -> *mut c_void {
    let m = ComplMatch(current);
    if m.is_null() {
        return std::ptr::null_mut();
    }

    if !match_at_original_text(m) {
        return current;
    }

    // Current is original text, find first real match
    let is_forward = dir >= 0;
    let next = if is_forward {
        nvim_compl_match_get_next(m)
    } else {
        nvim_compl_match_get_prev(m)
    };

    if next.is_null() || is_first_match(next) {
        return current;
    }

    next.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }
}
