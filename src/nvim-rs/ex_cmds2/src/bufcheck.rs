//! Buffer change checking utilities
//!
//! This module provides utilities for checking buffer changes and prompting for save.

use std::ffi::c_int;

// =============================================================================
// Dialog Response Values
// =============================================================================

/// Dialog response types (matching vim's VIM_YES, VIM_NO, etc.)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResponse {
    /// User chose Yes
    Yes = 2,
    /// User chose No
    No = 3,
    /// User chose Cancel
    Cancel = 4,
    /// User chose Yes to All
    All = 5,
    /// User chose Discard All
    DiscardAll = 6,
}

impl DialogResponse {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            2 => Some(Self::Yes),
            3 => Some(Self::No),
            4 => Some(Self::Cancel),
            5 => Some(Self::All),
            6 => Some(Self::DiscardAll),
            _ => None,
        }
    }

    /// Check if response is affirmative (Yes or All)
    pub fn is_affirmative(self) -> bool {
        matches!(self, Self::Yes | Self::All)
    }

    /// Check if response is negative (No or DiscardAll)
    pub fn is_negative(self) -> bool {
        matches!(self, Self::No | Self::DiscardAll)
    }

    /// Check if response is cancel
    pub fn is_cancel(self) -> bool {
        self == Self::Cancel
    }
}

/// Get VIM_YES value
#[no_mangle]
pub extern "C" fn rs_vim_yes() -> c_int {
    DialogResponse::Yes as c_int
}

/// Get VIM_NO value
#[no_mangle]
pub extern "C" fn rs_vim_no() -> c_int {
    DialogResponse::No as c_int
}

/// Get VIM_CANCEL value
#[no_mangle]
pub extern "C" fn rs_vim_cancel() -> c_int {
    DialogResponse::Cancel as c_int
}

/// Get VIM_ALL value
#[no_mangle]
pub extern "C" fn rs_vim_all() -> c_int {
    DialogResponse::All as c_int
}

/// Get VIM_DISCARDALL value
#[no_mangle]
pub extern "C" fn rs_vim_discardall() -> c_int {
    DialogResponse::DiscardAll as c_int
}

/// Check if dialog response is affirmative
#[no_mangle]
pub extern "C" fn rs_dialog_is_affirmative(response: c_int) -> bool {
    DialogResponse::from_int(response).is_some_and(|r| r.is_affirmative())
}

/// Check if dialog response is negative
#[no_mangle]
pub extern "C" fn rs_dialog_is_negative(response: c_int) -> bool {
    DialogResponse::from_int(response).is_some_and(|r| r.is_negative())
}

/// Check if dialog response is cancel
#[no_mangle]
pub extern "C" fn rs_dialog_is_cancel(response: c_int) -> bool {
    DialogResponse::from_int(response).is_some_and(|r| r.is_cancel())
}

// =============================================================================
// Buffer Number Tracking
// =============================================================================

/// State for tracking buffer numbers during check_changed_any
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufnumTracker {
    /// Current count of buffer numbers
    pub count: c_int,
    /// Capacity of buffer array
    pub capacity: usize,
}

/// Initialize buffer number tracker
#[no_mangle]
pub extern "C" fn rs_bufnum_tracker_init(capacity: usize) -> BufnumTracker {
    BufnumTracker { count: 0, capacity }
}

/// Check if buffer number is already tracked
#[no_mangle]
pub unsafe extern "C" fn rs_bufnum_contains(bufnrs: *const c_int, count: c_int, nr: c_int) -> bool {
    if bufnrs.is_null() || count <= 0 {
        return false;
    }

    for i in 0..count {
        if *bufnrs.add(i as usize) == nr {
            return true;
        }
    }
    false
}

/// Add buffer number if not already present
///
/// Returns new count, or -1 if array is full
#[no_mangle]
pub unsafe extern "C" fn rs_bufnum_add(
    bufnrs: *mut c_int,
    count: c_int,
    capacity: usize,
    nr: c_int,
) -> c_int {
    if bufnrs.is_null() || count < 0 {
        return -1;
    }

    // Check if already present
    for i in 0..count {
        if *bufnrs.add(i as usize) == nr {
            return count;
        }
    }

    // Check capacity
    if count as usize >= capacity {
        return -1;
    }

    // Add new number
    *bufnrs.add(count as usize) = nr;
    count + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_response() {
        assert_eq!(rs_vim_yes(), 2);
        assert_eq!(rs_vim_no(), 3);
        assert_eq!(rs_vim_cancel(), 4);
        assert_eq!(rs_vim_all(), 5);
        assert_eq!(rs_vim_discardall(), 6);
    }

    #[test]
    fn test_dialog_predicates() {
        assert!(rs_dialog_is_affirmative(2)); // VIM_YES
        assert!(rs_dialog_is_affirmative(5)); // VIM_ALL
        assert!(!rs_dialog_is_affirmative(3)); // VIM_NO

        assert!(rs_dialog_is_negative(3)); // VIM_NO
        assert!(rs_dialog_is_negative(6)); // VIM_DISCARDALL
        assert!(!rs_dialog_is_negative(2)); // VIM_YES

        assert!(rs_dialog_is_cancel(4));
        assert!(!rs_dialog_is_cancel(2));
    }

    #[test]
    fn test_bufnum_tracking() {
        unsafe {
            let mut bufnrs = [0; 10];

            assert!(!rs_bufnum_contains(bufnrs.as_ptr(), 0, 5));

            let count = rs_bufnum_add(bufnrs.as_mut_ptr(), 0, 10, 5);
            assert_eq!(count, 1);
            assert!(rs_bufnum_contains(bufnrs.as_ptr(), count, 5));
            assert!(!rs_bufnum_contains(bufnrs.as_ptr(), count, 10));

            // Adding same number should not increase count
            let count = rs_bufnum_add(bufnrs.as_mut_ptr(), count, 10, 5);
            assert_eq!(count, 1);

            let count = rs_bufnum_add(bufnrs.as_mut_ptr(), count, 10, 10);
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn test_bufnum_tracker_init() {
        let tracker = rs_bufnum_tracker_init(100);
        assert_eq!(tracker.count, 0);
        assert_eq!(tracker.capacity, 100);
    }
}
