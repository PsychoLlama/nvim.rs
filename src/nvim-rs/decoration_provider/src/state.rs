//! Provider state management and operations
//!
//! This module handles provider lifecycle: lookup, enable/disable,
//! bulk operations, and namespace highlight state.
//!
//! # Implementation Status
//!
//! This module will be expanded in Phase 205.

use std::ffi::c_int;

use crate::constants::{
    CB_MAX_ERROR, DECOR_PROVIDER_ACTIVE, DECOR_PROVIDER_DISABLED, DECOR_PROVIDER_REDRAW_DISABLED,
};
use crate::types::DecorProviderState;

// =============================================================================
// Provider State Operations
// =============================================================================

/// Reset a provider's state to active if it was only win-disabled.
/// Used at the start of processing a new window.
#[no_mangle]
pub extern "C" fn rs_decor_provider_reset_win_state(state: c_int) -> c_int {
    let s = DecorProviderState::from_c_int(state);
    if s == DecorProviderState::WinDisabled {
        DecorProviderState::Active.to_c_int()
    } else {
        state
    }
}

/// Get the next state when a provider returns false from a callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_disable_for_win(state: c_int) -> c_int {
    let s = DecorProviderState::from_c_int(state);
    if s == DecorProviderState::Active {
        DecorProviderState::WinDisabled.to_c_int()
    } else {
        state
    }
}

/// Get the next state when provider 'start' callback returns false.
#[no_mangle]
pub extern "C" fn rs_decor_provider_disable_for_redraw(_state: c_int) -> c_int {
    DecorProviderState::RedrawDisabled.to_c_int()
}

/// Check if provider should be invoked for 'start' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_start(state: c_int, has_start_cb: bool) -> bool {
    state != DECOR_PROVIDER_DISABLED && has_start_cb
}

/// Check if provider should be invoked for 'win' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_win(state: c_int, has_win_cb: bool) -> bool {
    state == DECOR_PROVIDER_ACTIVE && has_win_cb
}

/// Check if provider should be invoked for 'line' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_line(state: c_int, has_line_cb: bool) -> bool {
    state == DECOR_PROVIDER_ACTIVE && has_line_cb
}

/// Check if provider should be invoked for 'range' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_range(state: c_int, has_range_cb: bool) -> bool {
    state == DECOR_PROVIDER_ACTIVE && has_range_cb
}

/// Check if provider should be invoked for 'buf' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_buf(state: c_int, has_buf_cb: bool) -> bool {
    state == DECOR_PROVIDER_ACTIVE && has_buf_cb
}

/// Check if provider should be invoked for 'end' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_end(state: c_int, has_end_cb: bool) -> bool {
    state != DECOR_PROVIDER_DISABLED && has_end_cb
}

/// Check if provider should be invoked for 'spell_nav' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_spell_nav(state: c_int, has_spell_cb: bool) -> bool {
    state != DECOR_PROVIDER_DISABLED && has_spell_cb
}

/// Get the effective state after 'start' callback completes.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_after_start(
    current_state: c_int,
    callback_returned_true: bool,
) -> c_int {
    if current_state == DECOR_PROVIDER_DISABLED {
        current_state
    } else if callback_returned_true {
        DECOR_PROVIDER_ACTIVE
    } else {
        DECOR_PROVIDER_REDRAW_DISABLED
    }
}

// =============================================================================
// Error Handling
// =============================================================================

/// Check if error count has reached the maximum.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_disable_on_error(error_count: u8) -> bool {
    error_count >= CB_MAX_ERROR
}

/// Get the new error count after an error.
#[no_mangle]
pub extern "C" fn rs_decor_provider_increment_error(error_count: u8) -> u8 {
    error_count.saturating_add(1)
}

/// Reset error count (used after successful callback).
#[no_mangle]
pub extern "C" fn rs_decor_provider_reset_error_count() -> u8 {
    0
}

// =============================================================================
// Skip Position State
// =============================================================================

/// Check if position should be skipped based on skip state.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_skip_range(
    skip_row: c_int,
    skip_col: c_int,
    end_row: c_int,
    end_col: c_int,
) -> bool {
    skip_row > end_row || (skip_row == end_row && skip_col >= end_col)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DECOR_PROVIDER_WIN_DISABLED;

    #[test]
    fn test_reset_win_state() {
        assert_eq!(
            rs_decor_provider_reset_win_state(DECOR_PROVIDER_WIN_DISABLED),
            DECOR_PROVIDER_ACTIVE
        );
        assert_eq!(
            rs_decor_provider_reset_win_state(DECOR_PROVIDER_ACTIVE),
            DECOR_PROVIDER_ACTIVE
        );
        assert_eq!(
            rs_decor_provider_reset_win_state(DECOR_PROVIDER_DISABLED),
            DECOR_PROVIDER_DISABLED
        );
    }

    #[test]
    fn test_disable_for_win() {
        assert_eq!(
            rs_decor_provider_disable_for_win(DECOR_PROVIDER_ACTIVE),
            DECOR_PROVIDER_WIN_DISABLED
        );
        assert_eq!(
            rs_decor_provider_disable_for_win(DECOR_PROVIDER_DISABLED),
            DECOR_PROVIDER_DISABLED
        );
    }

    #[test]
    fn test_should_callbacks() {
        // start: invoked if not disabled and has callback
        assert!(rs_decor_provider_should_start(DECOR_PROVIDER_ACTIVE, true));
        assert!(!rs_decor_provider_should_start(
            DECOR_PROVIDER_DISABLED,
            true
        ));
        assert!(!rs_decor_provider_should_start(
            DECOR_PROVIDER_ACTIVE,
            false
        ));

        // win: only if active and has callback
        assert!(rs_decor_provider_should_win(DECOR_PROVIDER_ACTIVE, true));
        assert!(!rs_decor_provider_should_win(
            DECOR_PROVIDER_WIN_DISABLED,
            true
        ));

        // line: only if active and has callback
        assert!(rs_decor_provider_should_line(DECOR_PROVIDER_ACTIVE, true));
        assert!(!rs_decor_provider_should_line(
            DECOR_PROVIDER_WIN_DISABLED,
            true
        ));

        // end: invoked if not disabled and has callback
        assert!(rs_decor_provider_should_end(DECOR_PROVIDER_ACTIVE, true));
        assert!(rs_decor_provider_should_end(
            DECOR_PROVIDER_REDRAW_DISABLED,
            true
        ));
        assert!(!rs_decor_provider_should_end(DECOR_PROVIDER_DISABLED, true));
    }

    #[test]
    fn test_state_after_start() {
        // If disabled, stays disabled
        assert_eq!(
            rs_decor_provider_state_after_start(DECOR_PROVIDER_DISABLED, true),
            DECOR_PROVIDER_DISABLED
        );

        // If callback returns true, becomes active
        assert_eq!(
            rs_decor_provider_state_after_start(DECOR_PROVIDER_ACTIVE, true),
            DECOR_PROVIDER_ACTIVE
        );

        // If callback returns false, becomes redraw disabled
        assert_eq!(
            rs_decor_provider_state_after_start(DECOR_PROVIDER_ACTIVE, false),
            DECOR_PROVIDER_REDRAW_DISABLED
        );
    }

    #[test]
    fn test_error_handling() {
        assert!(!rs_decor_provider_should_disable_on_error(0));
        assert!(!rs_decor_provider_should_disable_on_error(4));
        assert!(rs_decor_provider_should_disable_on_error(5));
        assert!(rs_decor_provider_should_disable_on_error(10));

        assert_eq!(rs_decor_provider_increment_error(0), 1);
        assert_eq!(rs_decor_provider_increment_error(4), 5);
        assert_eq!(rs_decor_provider_increment_error(255), 255); // saturating
    }

    #[test]
    fn test_skip_range() {
        // skip_row > end_row
        assert!(rs_decor_provider_should_skip_range(10, 0, 5, 0));

        // skip_row == end_row, skip_col >= end_col
        assert!(rs_decor_provider_should_skip_range(5, 10, 5, 10));
        assert!(rs_decor_provider_should_skip_range(5, 11, 5, 10));

        // Should not skip
        assert!(!rs_decor_provider_should_skip_range(5, 0, 10, 0));
        assert!(!rs_decor_provider_should_skip_range(5, 5, 5, 10));
    }
}
