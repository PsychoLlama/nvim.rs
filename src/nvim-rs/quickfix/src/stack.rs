//! Quickfix stack operations
//!
//! This module provides stack-level operations for quickfix and location lists,
//! including stack navigation, push/pop operations, and list management.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_errors_doc)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of lists in a quickfix stack
pub const LISTCOUNT: usize = 10;

/// Invalid list ID
pub const INVALID_QFIDX: c_int = -1;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to quickfix stack (`qf_info_T`)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfStackHandle(pub *mut c_void);

impl QfStackHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// External C Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_qf_get_listcount(qi: QfStackHandle) -> c_int;
    fn nvim_qf_get_curlist_idx(qi: QfStackHandle) -> c_int;
    fn nvim_qf_set_curlist_idx(qi: QfStackHandle, idx: c_int);
    fn nvim_qf_set_listcount(qi: QfStackHandle, count: c_int);
    fn nvim_qf_get_list_at_mut(qi: QfStackHandle, idx: c_int) -> *mut c_void;
    fn nvim_qf_clear_list_struct(qfl: *mut c_void);
}

// =============================================================================
// Stack State
// =============================================================================

/// Quickfix stack state information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfStackState {
    /// Number of lists in the stack
    pub list_count: c_int,
    /// Index of current list (0-based)
    pub cur_idx: c_int,
    /// Whether the stack is empty
    pub is_empty: bool,
    /// Whether the stack is full
    pub is_full: bool,
}

impl Default for QfStackState {
    fn default() -> Self {
        Self {
            list_count: 0,
            cur_idx: INVALID_QFIDX,
            is_empty: true,
            is_full: false,
        }
    }
}

impl QfStackState {
    /// Get state from a quickfix stack handle
    pub fn from_handle(qi: QfStackHandle) -> Self {
        if qi.is_null() {
            return Self::default();
        }

        unsafe {
            let list_count = nvim_qf_get_listcount(qi);
            let cur_idx = nvim_qf_get_curlist_idx(qi);

            Self {
                list_count,
                cur_idx,
                is_empty: list_count == 0,
                is_full: list_count >= LISTCOUNT as c_int,
            }
        }
    }

    /// Check if we can add a new list to the stack
    pub const fn can_add_list(&self) -> bool {
        !self.is_full
    }

    /// Check if we can navigate to an older list
    pub const fn can_go_older(&self) -> bool {
        self.cur_idx > 0
    }

    /// Check if we can navigate to a newer list
    pub const fn can_go_newer(&self) -> bool {
        self.cur_idx < self.list_count - 1
    }
}

// =============================================================================
// Stack Navigation
// =============================================================================

/// Result of a stack navigation operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfStackNavResult {
    /// Navigation succeeded
    Success = 0,
    /// Stack is empty
    Empty = 1,
    /// Already at oldest list
    AtOldest = 2,
    /// Already at newest list
    AtNewest = 3,
    /// Index out of range
    OutOfRange = 4,
    /// Invalid stack handle
    InvalidStack = 5,
}

/// Calculate the target index for navigation
pub fn calculate_nav_target(
    state: &QfStackState,
    direction: NavDirection,
    count: c_int,
) -> Result<c_int, QfStackNavResult> {
    if state.is_empty {
        return Err(QfStackNavResult::Empty);
    }

    let target = match direction {
        NavDirection::Older => state.cur_idx - count,
        NavDirection::Newer => state.cur_idx + count,
        NavDirection::Absolute(idx) => idx,
        NavDirection::First => 0,
        NavDirection::Last => state.list_count - 1,
    };

    if target < 0 {
        Err(QfStackNavResult::AtOldest)
    } else if target >= state.list_count {
        Err(QfStackNavResult::AtNewest)
    } else {
        Ok(target)
    }
}

/// Navigation direction for stack traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    /// Go to older list (lower index)
    Older,
    /// Go to newer list (higher index)
    Newer,
    /// Go to specific index
    Absolute(c_int),
    /// Go to first (oldest) list
    First,
    /// Go to last (newest) list
    Last,
}

// =============================================================================
// Stack Push/Pop
// =============================================================================

/// Result of adding a new list to the stack
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfPushResult {
    /// Whether the push was successful
    pub success: bool,
    /// Index where the new list was placed
    pub new_idx: c_int,
    /// Number of old lists that were dropped
    pub dropped: c_int,
}

impl QfPushResult {
    /// Create a failed push result
    pub const fn failed() -> Self {
        Self {
            success: false,
            new_idx: INVALID_QFIDX,
            dropped: 0,
        }
    }
}

/// Calculate what happens when pushing a new list
pub fn calculate_push(state: &QfStackState) -> QfPushResult {
    if state.is_full {
        // Need to drop the oldest list
        QfPushResult {
            success: true,
            new_idx: (LISTCOUNT - 1) as c_int,
            dropped: 1,
        }
    } else {
        // Can add without dropping
        QfPushResult {
            success: true,
            new_idx: state.list_count,
            dropped: 0,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get quickfix stack state
#[no_mangle]
pub extern "C" fn rs_qf_stack_get_state(qi: QfStackHandle) -> QfStackState {
    QfStackState::from_handle(qi)
}

/// FFI export: Check if stack can add a list
#[no_mangle]
pub extern "C" fn rs_qf_stack_can_add_list(qi: QfStackHandle) -> c_int {
    c_int::from(QfStackState::from_handle(qi).can_add_list())
}

/// FFI export: Check if stack can go older
#[no_mangle]
pub extern "C" fn rs_qf_stack_can_go_older(qi: QfStackHandle) -> c_int {
    c_int::from(QfStackState::from_handle(qi).can_go_older())
}

/// FFI export: Check if stack can go newer
#[no_mangle]
pub extern "C" fn rs_qf_stack_can_go_newer(qi: QfStackHandle) -> c_int {
    c_int::from(QfStackState::from_handle(qi).can_go_newer())
}

/// FFI export: Calculate nav target for older
#[no_mangle]
pub extern "C" fn rs_qf_stack_nav_older_target(qi: QfStackHandle, count: c_int) -> c_int {
    let state = QfStackState::from_handle(qi);
    calculate_nav_target(&state, NavDirection::Older, count).unwrap_or(INVALID_QFIDX)
}

/// FFI export: Calculate nav target for newer
#[no_mangle]
pub extern "C" fn rs_qf_stack_nav_newer_target(qi: QfStackHandle, count: c_int) -> c_int {
    let state = QfStackState::from_handle(qi);
    calculate_nav_target(&state, NavDirection::Newer, count).unwrap_or(INVALID_QFIDX)
}

/// FFI export: Calculate nav target for first list
#[no_mangle]
pub extern "C" fn rs_qf_stack_nav_first_target(qi: QfStackHandle) -> c_int {
    let state = QfStackState::from_handle(qi);
    calculate_nav_target(&state, NavDirection::First, 0).unwrap_or(INVALID_QFIDX)
}

/// FFI export: Calculate nav target for last list
#[no_mangle]
pub extern "C" fn rs_qf_stack_nav_last_target(qi: QfStackHandle) -> c_int {
    let state = QfStackState::from_handle(qi);
    calculate_nav_target(&state, NavDirection::Last, 0).unwrap_or(INVALID_QFIDX)
}

/// FFI export: Calculate nav target for absolute index
#[no_mangle]
pub extern "C" fn rs_qf_stack_nav_absolute_target(qi: QfStackHandle, idx: c_int) -> c_int {
    let state = QfStackState::from_handle(qi);
    calculate_nav_target(&state, NavDirection::Absolute(idx), 0).unwrap_or(INVALID_QFIDX)
}

/// FFI export: Calculate push result
#[no_mangle]
pub extern "C" fn rs_qf_stack_calculate_push(qi: QfStackHandle) -> QfPushResult {
    let state = QfStackState::from_handle(qi);
    calculate_push(&state)
}

/// FFI export: Get the LISTCOUNT constant
#[no_mangle]
pub extern "C" fn rs_qf_listcount() -> c_int {
    LISTCOUNT as c_int
}

/// FFI export: Get the INVALID_QFIDX constant
#[no_mangle]
pub extern "C" fn rs_qf_invalid_idx() -> c_int {
    INVALID_QFIDX
}

/// FFI export: Check if an index is valid for the stack
#[no_mangle]
pub extern "C" fn rs_qf_stack_idx_valid(qi: QfStackHandle, idx: c_int) -> c_int {
    let state = QfStackState::from_handle(qi);
    c_int::from(idx >= 0 && idx < state.list_count)
}

/// FFI export: Get remaining capacity in the stack
#[no_mangle]
pub extern "C" fn rs_qf_stack_remaining_capacity(qi: QfStackHandle) -> c_int {
    let state = QfStackState::from_handle(qi);
    (LISTCOUNT as c_int) - state.list_count
}

/// FFI export: Check if stack is empty
#[no_mangle]
pub extern "C" fn rs_qf_stack_is_empty(qi: QfStackHandle) -> c_int {
    c_int::from(QfStackState::from_handle(qi).is_empty)
}

/// FFI export: Check if stack is full
#[no_mangle]
pub extern "C" fn rs_qf_stack_is_full(qi: QfStackHandle) -> c_int {
    c_int::from(QfStackState::from_handle(qi).is_full)
}

/// FFI export: Get the list count from stack
#[no_mangle]
pub extern "C" fn rs_qf_stack_list_count(qi: QfStackHandle) -> c_int {
    QfStackState::from_handle(qi).list_count
}

/// FFI export: Get current list index from stack
#[no_mangle]
pub extern "C" fn rs_qf_stack_cur_idx(qi: QfStackHandle) -> c_int {
    QfStackState::from_handle(qi).cur_idx
}

/// FFI export: Get NavResult Success value
#[no_mangle]
pub extern "C" fn rs_qf_nav_result_success() -> c_int {
    QfStackNavResult::Success as c_int
}

/// FFI export: Get NavResult Empty value
#[no_mangle]
pub extern "C" fn rs_qf_nav_result_empty() -> c_int {
    QfStackNavResult::Empty as c_int
}

/// FFI export: Get NavResult AtOldest value
#[no_mangle]
pub extern "C" fn rs_qf_nav_result_at_oldest() -> c_int {
    QfStackNavResult::AtOldest as c_int
}

/// FFI export: Get NavResult AtNewest value
#[no_mangle]
pub extern "C" fn rs_qf_nav_result_at_newest() -> c_int {
    QfStackNavResult::AtNewest as c_int
}

// =============================================================================
// Phase 5: Stack mutation helpers
// =============================================================================

/// Decrement qf_curlist with wrap-around to qf_listcount - 1 when at 0.
///
/// Mirrors C `nvim_qf_decr_curlist`.
///
/// # Safety
///
/// - `qi` must be a valid non-null pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_decr_curlist(qi: QfStackHandle) {
    if qi.is_null() {
        return;
    }
    let curlist = nvim_qf_get_curlist_idx(qi);
    let listcount = nvim_qf_get_listcount(qi);
    if curlist == 0 {
        nvim_qf_set_curlist_idx(qi, listcount - 1);
    } else {
        nvim_qf_set_curlist_idx(qi, curlist - 1);
    }
}

/// Decrement qf_listcount if it is greater than zero.
///
/// Mirrors C `nvim_qf_decr_listcount`.
///
/// # Safety
///
/// - `qi` must be a valid non-null pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_decr_listcount(qi: QfStackHandle) {
    if qi.is_null() {
        return;
    }
    let listcount = nvim_qf_get_listcount(qi);
    if listcount > 0 {
        nvim_qf_set_listcount(qi, listcount - 1);
    }
}

/// Zero (memset) the list slot at index `qf_listcount - 1`.
///
/// Mirrors C `nvim_qf_zero_top_list`.
///
/// # Safety
///
/// - `qi` must be a valid non-null pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_zero_top_list(qi: QfStackHandle) {
    if qi.is_null() {
        return;
    }
    let listcount = nvim_qf_get_listcount(qi);
    if listcount > 0 {
        let qfl = nvim_qf_get_list_at_mut(qi, listcount - 1);
        if !qfl.is_null() {
            nvim_qf_clear_list_struct(qfl);
        }
    }
}

// =============================================================================
// Phase 11: Stack resize and location list sync
// =============================================================================

/// Opaque window handle
type WinHandle = *mut c_void;
/// Opaque quickfix info handle
type QfInfoHandleMut = *mut c_void;

extern "C" {
    // Stack accessors (new for Phase 11)
    fn nvim_qf_get_maxcount(qi: *const c_void) -> c_int;
    fn nvim_qf_resize_lists_array(qi: *mut c_void, n: c_int);

    // From lifecycle.rs (same crate, same link unit)
    fn rs_qf_pop_stack(qi: *mut c_void, adjust: bool);
    fn rs_qf_update_buffer(qi: *mut c_void, old_last: *const c_void);
    fn nvim_get_ql_info() -> *mut c_void;
    fn nvim_qf_is_ll_window(wp: *const c_void) -> bool;
    fn rs_ll_get_or_alloc_list(wp: *mut c_void) -> *mut c_void;

    // Window field accessors
    fn nvim_win_get_llist_ref(win: *const c_void) -> *const c_void;
    fn nvim_win_get_p_lhi(wp: *const c_void) -> c_int;
    fn nvim_win_set_p_lhi(win: *mut c_void, v: c_int);
    fn nvim_qf_win_get_llist(wp: *const c_void) -> *mut c_void;
    fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> *mut c_void;

    // Window iteration (FOR_ALL_WINDOWS_IN_TAB)
    fn nvim_get_firstwin() -> *mut c_void;
    fn nvim_qf_win_next(win: *const c_void) -> *mut c_void;
    fn nvim_win_is_qf_win(win: *const c_void) -> bool;
}

/// Sync a location list window's `lhistory` value to its parent window.
///
/// Equivalent to C `qf_sync_llw_to_win`.
///
/// # Safety
///
/// - `llw` must be a valid pointer to a `win_T` that is a location list window.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_sync_llw_to_win(llw: WinHandle) {
    if llw.is_null() {
        return;
    }
    let llist_ref = nvim_win_get_llist_ref(llw.cast_const());
    if llist_ref.is_null() {
        return;
    }
    let wp = nvim_qf_find_win_with_loclist(llist_ref);
    if !wp.is_null() {
        let lhi = nvim_win_get_p_lhi(llw.cast_const());
        nvim_win_set_p_lhi(wp, lhi);
    }
}

/// Sync a window's `lhistory` value to its location list window, if any.
///
/// Equivalent to C `qf_sync_win_to_llw`.
///
/// # Safety
///
/// - `pwp` must be a valid pointer to a `win_T`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_sync_win_to_llw(pwp: WinHandle) {
    if pwp.is_null() {
        return;
    }
    let llw = nvim_qf_win_get_llist(pwp.cast_const());
    if llw.is_null() {
        return;
    }
    let lhi = nvim_win_get_p_lhi(pwp.cast_const());
    // FOR_ALL_WINDOWS_IN_TAB(wp, curtab)
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        let llist_ref = nvim_win_get_llist_ref(wp.cast_const());
        if llist_ref == llw.cast_const() && nvim_win_is_qf_win(wp.cast_const()) {
            nvim_win_set_p_lhi(wp, lhi);
            return;
        }
        wp = nvim_qf_win_next(wp.cast_const());
    }
}

/// Resize the quickfix/location lists array to hold `n` lists.
///
/// Equivalent to C `qf_resize_stack_base`. If shrinking below the current
/// list count, pops excess lists first. Reallocates and zero-fills the array.
///
/// # Safety
///
/// - `qi` must be a valid non-null pointer to a `qf_info_T` struct.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_resize_stack_base(qi: QfInfoHandleMut, n: c_int) {
    if qi.is_null() {
        return;
    }
    let qi_handle = QfStackHandle(qi);
    let maxcount = nvim_qf_get_maxcount(qi.cast_const());
    if n == maxcount {
        return;
    }
    let listcount = nvim_qf_get_listcount(qi_handle);
    if n < maxcount && n < listcount {
        let amount_to_rm = listcount - n;
        for _ in 0..amount_to_rm {
            rs_qf_pop_stack(qi, true);
        }
    }
    nvim_qf_resize_lists_array(qi, n);
    rs_qf_update_buffer(qi, std::ptr::null());
}

/// Resize the global quickfix stack to hold `n` lists.
///
/// Equivalent to C `qf_resize_stack`.
///
/// # Safety
///
/// - Must be called from the Neovim main thread.
/// - `ql_info` must be initialized.
#[export_name = "qf_resize_stack"]
pub unsafe extern "C" fn rs_qf_resize_stack(n: c_int) {
    let qi = nvim_get_ql_info();
    if !qi.is_null() {
        rs_qf_resize_stack_base(qi, n);
    }
}

/// Resize a location list stack for window `wp` to hold `n` lists.
///
/// Equivalent to C `ll_resize_stack`. Syncs `lhistory` first, then
/// gets-or-allocates the list and resizes.
///
/// # Safety
///
/// - `wp` must be a valid non-null pointer to a `win_T`.
#[export_name = "ll_resize_stack"]
pub unsafe extern "C" fn rs_ll_resize_stack(wp: WinHandle, n: c_int) {
    if wp.is_null() {
        return;
    }
    if nvim_qf_is_ll_window(wp.cast_const()) {
        rs_qf_sync_llw_to_win(wp);
    } else {
        rs_qf_sync_win_to_llw(wp);
    }
    let qi = rs_ll_get_or_alloc_list(wp);
    if !qi.is_null() {
        rs_qf_resize_stack_base(qi, n);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_state_default() {
        let state = QfStackState::default();
        assert!(state.is_empty);
        assert!(!state.is_full);
        assert_eq!(state.list_count, 0);
        assert_eq!(state.cur_idx, INVALID_QFIDX);
    }

    #[test]
    fn test_stack_state_navigation() {
        let state = QfStackState {
            list_count: 5,
            cur_idx: 2,
            is_empty: false,
            is_full: false,
        };

        assert!(state.can_go_older());
        assert!(state.can_go_newer());

        // At first list
        let first = QfStackState {
            list_count: 5,
            cur_idx: 0,
            is_empty: false,
            is_full: false,
        };
        assert!(!first.can_go_older());
        assert!(first.can_go_newer());

        // At last list
        let last = QfStackState {
            list_count: 5,
            cur_idx: 4,
            is_empty: false,
            is_full: false,
        };
        assert!(last.can_go_older());
        assert!(!last.can_go_newer());
    }

    #[test]
    fn test_calculate_nav_target() {
        let state = QfStackState {
            list_count: 5,
            cur_idx: 2,
            is_empty: false,
            is_full: false,
        };

        // Go older
        assert_eq!(calculate_nav_target(&state, NavDirection::Older, 1), Ok(1));
        assert_eq!(calculate_nav_target(&state, NavDirection::Older, 2), Ok(0));
        assert_eq!(
            calculate_nav_target(&state, NavDirection::Older, 3),
            Err(QfStackNavResult::AtOldest)
        );

        // Go newer
        assert_eq!(calculate_nav_target(&state, NavDirection::Newer, 1), Ok(3));
        assert_eq!(calculate_nav_target(&state, NavDirection::Newer, 2), Ok(4));
        assert_eq!(
            calculate_nav_target(&state, NavDirection::Newer, 3),
            Err(QfStackNavResult::AtNewest)
        );

        // First/Last
        assert_eq!(calculate_nav_target(&state, NavDirection::First, 0), Ok(0));
        assert_eq!(calculate_nav_target(&state, NavDirection::Last, 0), Ok(4));
    }

    #[test]
    fn test_calculate_push() {
        // Not full
        let state = QfStackState {
            list_count: 5,
            cur_idx: 4,
            is_empty: false,
            is_full: false,
        };
        let result = calculate_push(&state);
        assert!(result.success);
        assert_eq!(result.new_idx, 5);
        assert_eq!(result.dropped, 0);

        // Full
        let full = QfStackState {
            list_count: 10,
            cur_idx: 9,
            is_empty: false,
            is_full: true,
        };
        let result = calculate_push(&full);
        assert!(result.success);
        assert_eq!(result.new_idx, 9);
        assert_eq!(result.dropped, 1);
    }

    #[test]
    fn test_push_result_failed() {
        let failed = QfPushResult::failed();
        assert!(!failed.success);
        assert_eq!(failed.new_idx, INVALID_QFIDX);
    }

    #[test]
    fn test_nav_result_values() {
        assert_eq!(QfStackNavResult::Success as c_int, 0);
        assert_eq!(QfStackNavResult::Empty as c_int, 1);
        assert_eq!(QfStackNavResult::AtOldest as c_int, 2);
        assert_eq!(QfStackNavResult::AtNewest as c_int, 3);
    }
}
