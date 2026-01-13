//! Provider state management and operations
//!
//! This module handles provider lifecycle: lookup, enable/disable,
//! bulk operations, and namespace highlight state.
//!
//! # Architecture
//!
//! Provider state follows a state machine pattern:
//! - Disabled: Provider is off (default or after errors)
//! - Active: Provider will receive callbacks
//! - WinDisabled: Skipped for current window (resets on new window)
//! - RedrawDisabled: Skipped for current redraw (resets on next redraw)
//!
//! # State Transitions
//!
//! ```text
//! Disabled ←─────────────────────────────────────────────────┐
//!    ↓ (set provider)                                        │
//! Active ──┬── (win callback returns false) ─→ WinDisabled   │
//!          └── (start callback returns false) ─→ RedrawDisabled
//!          └── (too many errors) ────────────────────────────┘
//!
//! WinDisabled ──(new window)──→ Active
//! RedrawDisabled ──(new redraw)──→ (check start callback)
//! ```

use std::ffi::c_int;

use crate::constants::{
    CB_MAX_ERROR, DECOR_PROVIDER_ACTIVE, DECOR_PROVIDER_DISABLED, DECOR_PROVIDER_REDRAW_DISABLED,
    LUA_NOREF,
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

/// Check if provider should be invoked for 'conceal_line' callback.
#[no_mangle]
pub extern "C" fn rs_decor_provider_should_conceal_line(
    state: c_int,
    has_conceal_cb: bool,
) -> bool {
    state != DECOR_PROVIDER_DISABLED && has_conceal_cb
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

/// Get state for provider without start callback at redraw start.
/// If provider is not disabled and has no start callback, becomes active.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_no_start(current_state: c_int) -> c_int {
    if current_state == DECOR_PROVIDER_DISABLED {
        current_state
    } else {
        DECOR_PROVIDER_ACTIVE
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

/// Get state after error occurred during callback.
/// If error count reaches max, returns disabled state.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_after_error(
    current_state: c_int,
    error_count: u8,
) -> c_int {
    if error_count >= CB_MAX_ERROR {
        DECOR_PROVIDER_DISABLED
    } else {
        current_state
    }
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

/// Check if position is before skip position (can invoke callback).
#[no_mangle]
pub extern "C" fn rs_decor_provider_can_invoke_range(
    skip_row: c_int,
    skip_col: c_int,
    start_row: c_int,
    start_col: c_int,
) -> bool {
    start_row > skip_row || (start_row == skip_row && start_col >= skip_col)
}

// =============================================================================
// Redraw State Tracking
// =============================================================================

/// Represents state changes from a provider callback result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CallbackResult {
    /// Whether callback executed successfully.
    pub success: bool,
    /// Whether callback returned true/continue.
    pub should_continue: bool,
    /// New skip row (from range callback).
    pub skip_row: c_int,
    /// New skip col (from range callback).
    pub skip_col: c_int,
}

impl CallbackResult {
    /// Create a success result that continues.
    #[must_use]
    pub const fn success_continue() -> Self {
        Self {
            success: true,
            should_continue: true,
            skip_row: 0,
            skip_col: 0,
        }
    }

    /// Create a success result that stops.
    #[must_use]
    pub const fn success_stop() -> Self {
        Self {
            success: true,
            should_continue: false,
            skip_row: 0,
            skip_col: 0,
        }
    }

    /// Create a success result with skip update.
    #[must_use]
    pub const fn success_with_skip(skip_row: c_int, skip_col: c_int) -> Self {
        Self {
            success: true,
            should_continue: true,
            skip_row,
            skip_col,
        }
    }

    /// Create an error result.
    #[must_use]
    pub const fn error() -> Self {
        Self {
            success: false,
            should_continue: false,
            skip_row: 0,
            skip_col: 0,
        }
    }
}

/// FFI: Create success/continue result.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_success_continue() -> CallbackResult {
    CallbackResult::success_continue()
}

/// FFI: Create success/stop result.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_success_stop() -> CallbackResult {
    CallbackResult::success_stop()
}

/// FFI: Create success with skip result.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_success_with_skip(
    skip_row: c_int,
    skip_col: c_int,
) -> CallbackResult {
    CallbackResult::success_with_skip(skip_row, skip_col)
}

/// FFI: Create error result.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_error() -> CallbackResult {
    CallbackResult::error()
}

/// FFI: Check if result indicates success.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_is_success(result: CallbackResult) -> bool {
    result.success
}

/// FFI: Check if result indicates continue.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_should_continue(result: CallbackResult) -> bool {
    result.should_continue
}

/// FFI: Check if result has skip update.
#[no_mangle]
pub extern "C" fn rs_provider_cb_result_has_skip(result: CallbackResult) -> bool {
    result.skip_row != 0 || result.skip_col != 0
}

// =============================================================================
// Namespace Highlight State
// =============================================================================

/// Check if namespace highlight needs refresh.
/// Returns true if hl_valid changed or hl_cached is false.
#[no_mangle]
pub extern "C" fn rs_decor_provider_hl_needs_refresh(
    current_hl_valid: c_int,
    cached_hl_valid: c_int,
    hl_cached: bool,
) -> bool {
    !hl_cached || current_hl_valid != cached_hl_valid
}

/// Compute new hl_valid value after highlight definition.
/// This is typically current_tick or some incrementing value.
#[no_mangle]
pub extern "C" fn rs_decor_provider_compute_hl_valid(current_tick: c_int) -> c_int {
    current_tick
}

// =============================================================================
// Provider Validation
// =============================================================================

/// Check if namespace ID is valid for provider lookup.
#[no_mangle]
pub extern "C" fn rs_decor_provider_ns_id_valid(ns_id: c_int) -> bool {
    ns_id > 0
}

/// Check if provider has any active callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_callbacks(
    redraw_start: c_int,
    redraw_buf: c_int,
    redraw_win: c_int,
    redraw_line: c_int,
    redraw_end: c_int,
) -> bool {
    redraw_start != LUA_NOREF
        || redraw_buf != LUA_NOREF
        || redraw_win != LUA_NOREF
        || redraw_line != LUA_NOREF
        || redraw_end != LUA_NOREF
}

/// Check if provider has range-specific callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_range_callbacks(
    redraw_line: c_int,
    redraw_range: c_int,
) -> bool {
    redraw_line != LUA_NOREF || redraw_range != LUA_NOREF
}

/// Check if provider has any callbacks at all.
#[no_mangle]
pub extern "C" fn rs_decor_provider_has_any_ref(
    redraw_start: c_int,
    redraw_buf: c_int,
    redraw_win: c_int,
    redraw_line: c_int,
    redraw_range: c_int,
    redraw_end: c_int,
    hl_def: c_int,
    spell_nav: c_int,
    conceal_line: c_int,
) -> bool {
    redraw_start != LUA_NOREF
        || redraw_buf != LUA_NOREF
        || redraw_win != LUA_NOREF
        || redraw_line != LUA_NOREF
        || redraw_range != LUA_NOREF
        || redraw_end != LUA_NOREF
        || hl_def != LUA_NOREF
        || spell_nav != LUA_NOREF
        || conceal_line != LUA_NOREF
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
    fn test_state_no_start() {
        assert_eq!(
            rs_decor_provider_state_no_start(DECOR_PROVIDER_ACTIVE),
            DECOR_PROVIDER_ACTIVE
        );
        assert_eq!(
            rs_decor_provider_state_no_start(DECOR_PROVIDER_REDRAW_DISABLED),
            DECOR_PROVIDER_ACTIVE
        );
        assert_eq!(
            rs_decor_provider_state_no_start(DECOR_PROVIDER_DISABLED),
            DECOR_PROVIDER_DISABLED
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

        assert_eq!(
            rs_decor_provider_state_after_error(DECOR_PROVIDER_ACTIVE, 4),
            DECOR_PROVIDER_ACTIVE
        );
        assert_eq!(
            rs_decor_provider_state_after_error(DECOR_PROVIDER_ACTIVE, 5),
            DECOR_PROVIDER_DISABLED
        );
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

    #[test]
    fn test_can_invoke_range() {
        // start >= skip
        assert!(rs_decor_provider_can_invoke_range(5, 5, 5, 5));
        assert!(rs_decor_provider_can_invoke_range(5, 5, 10, 0));
        assert!(rs_decor_provider_can_invoke_range(5, 5, 5, 10));

        // start < skip
        assert!(!rs_decor_provider_can_invoke_range(5, 5, 5, 4));
        assert!(!rs_decor_provider_can_invoke_range(5, 5, 4, 10));
    }

    #[test]
    fn test_callback_result() {
        let res = CallbackResult::success_continue();
        assert!(res.success);
        assert!(res.should_continue);
        assert!(!rs_provider_cb_result_has_skip(res));

        let res = CallbackResult::success_stop();
        assert!(res.success);
        assert!(!res.should_continue);

        let res = CallbackResult::success_with_skip(10, 5);
        assert!(res.success);
        assert!(res.should_continue);
        assert!(rs_provider_cb_result_has_skip(res));
        assert_eq!(res.skip_row, 10);
        assert_eq!(res.skip_col, 5);

        let res = CallbackResult::error();
        assert!(!res.success);
        assert!(!res.should_continue);
    }

    #[test]
    fn test_hl_needs_refresh() {
        // Not cached -> needs refresh
        assert!(rs_decor_provider_hl_needs_refresh(5, 5, false));

        // Cached but different -> needs refresh
        assert!(rs_decor_provider_hl_needs_refresh(6, 5, true));

        // Cached and same -> no refresh needed
        assert!(!rs_decor_provider_hl_needs_refresh(5, 5, true));
    }

    #[test]
    fn test_ns_id_valid() {
        assert!(rs_decor_provider_ns_id_valid(1));
        assert!(rs_decor_provider_ns_id_valid(100));
        assert!(!rs_decor_provider_ns_id_valid(0));
        assert!(!rs_decor_provider_ns_id_valid(-1));
    }

    #[test]
    fn test_has_callbacks() {
        // No callbacks
        assert!(!rs_decor_provider_has_callbacks(
            LUA_NOREF, LUA_NOREF, LUA_NOREF, LUA_NOREF, LUA_NOREF
        ));

        // Has start callback
        assert!(rs_decor_provider_has_callbacks(
            1, LUA_NOREF, LUA_NOREF, LUA_NOREF, LUA_NOREF
        ));

        // Has line callback
        assert!(rs_decor_provider_has_callbacks(
            LUA_NOREF, LUA_NOREF, LUA_NOREF, 1, LUA_NOREF
        ));
    }

    #[test]
    fn test_has_range_callbacks() {
        assert!(!rs_decor_provider_has_range_callbacks(LUA_NOREF, LUA_NOREF));
        assert!(rs_decor_provider_has_range_callbacks(1, LUA_NOREF));
        assert!(rs_decor_provider_has_range_callbacks(LUA_NOREF, 1));
        assert!(rs_decor_provider_has_range_callbacks(1, 1));
    }
}
