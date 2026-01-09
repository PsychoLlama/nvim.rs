//! Execution stack management
//!
//! This module handles the execution stack (exestack) which tracks the source
//! of currently executing code for error messages and debugging.

use std::ffi::{c_char, c_int};

use crate::{EstackArgT, EstackHandle, EtypeT, LinenrT, ScidT};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Execution stack accessors
    fn nvim_exestack_get_len() -> c_int;
    fn nvim_exestack_get_entry(idx: c_int) -> EstackHandle;
    fn nvim_estack_get_lnum(entry: EstackHandle) -> LinenrT;
    fn nvim_estack_get_name(entry: EstackHandle) -> *const c_char;
    fn nvim_estack_get_type(entry: EstackHandle) -> c_int;

    // Source context accessors
    fn nvim_estack_get_sctx_sid(entry: EstackHandle) -> ScidT;
}

// =============================================================================
// Stack Entry Access
// =============================================================================

/// Get the top entry of the execution stack (most recent).
///
/// Returns null handle if the stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_top() -> EstackHandle {
    let len = nvim_exestack_get_len();
    if len <= 0 {
        return EstackHandle::null();
    }
    nvim_exestack_get_entry(len - 1)
}

/// Get an entry from the execution stack by index.
///
/// Index 0 is the bottom (oldest), len-1 is the top (newest).
/// Returns null handle if index is out of bounds.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_get(idx: c_int) -> EstackHandle {
    let len = nvim_exestack_get_len();
    if idx < 0 || idx >= len {
        return EstackHandle::null();
    }
    nvim_exestack_get_entry(idx)
}

/// Get the line number from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_lnum(entry: EstackHandle) -> LinenrT {
    if entry.is_null() {
        return 0;
    }
    nvim_estack_get_lnum(entry)
}

/// Get the name from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_name(entry: EstackHandle) -> *const c_char {
    if entry.is_null() {
        return std::ptr::null();
    }
    nvim_estack_get_name(entry)
}

/// Get the type from an execution stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_type(entry: EstackHandle) -> c_int {
    if entry.is_null() {
        return EtypeT::Top as c_int;
    }
    nvim_estack_get_type(entry)
}

/// Get the script ID from an execution stack entry (for Script/Modeline types).
#[no_mangle]
pub unsafe extern "C" fn rs_estack_entry_sid(entry: EstackHandle) -> ScidT {
    if entry.is_null() {
        return 0;
    }
    nvim_estack_get_sctx_sid(entry)
}

// =============================================================================
// Stack Search Functions
// =============================================================================

/// Find the most recent script entry in the execution stack.
///
/// Returns the index of the entry, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_find_script() -> c_int {
    let len = nvim_exestack_get_len();

    // Search from top to bottom
    for i in (0..len).rev() {
        let entry = nvim_exestack_get_entry(i);
        let entry_type = nvim_estack_get_type(entry);

        if entry_type == EtypeT::Script as c_int {
            return i;
        }
    }

    -1
}

/// Find the most recent entry with a given type in the execution stack.
///
/// Returns the index of the entry, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_find_type(etype: c_int) -> c_int {
    let len = nvim_exestack_get_len();

    // Search from top to bottom
    for i in (0..len).rev() {
        let entry = nvim_exestack_get_entry(i);
        let entry_type = nvim_estack_get_type(entry);

        if entry_type == etype {
            return i;
        }
    }

    -1
}

/// Check if a given entry type is on the execution stack.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_has_type(etype: c_int) -> bool {
    rs_estack_find_type(etype) >= 0
}

// =============================================================================
// Stack Information
// =============================================================================

/// Get info about the execution stack suitable for display.
///
/// Returns the entry type at the given stack depth (0 = top).
/// Returns ETYPE_TOP if depth is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_type_at_depth(depth: c_int) -> c_int {
    let len = nvim_exestack_get_len();
    let idx = len - 1 - depth;

    if idx < 0 || idx >= len {
        return EtypeT::Top as c_int;
    }

    let entry = nvim_exestack_get_entry(idx);
    nvim_estack_get_type(entry)
}

/// Count how many entries of a given type are on the stack.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_count_type(etype: c_int) -> c_int {
    let len = nvim_exestack_get_len();
    let mut count = 0;

    for i in 0..len {
        let entry = nvim_exestack_get_entry(i);
        if nvim_estack_get_type(entry) == etype {
            count += 1;
        }
    }

    count
}

// =============================================================================
// estack_sfile() Helper
// =============================================================================

/// Determine what to return for estack_sfile() based on the argument.
///
/// Returns the appropriate stack index, or -1 if nothing should be returned.
#[no_mangle]
pub unsafe extern "C" fn rs_estack_sfile_index(which: c_int) -> c_int {
    let len = nvim_exestack_get_len();
    if len <= 0 {
        return -1;
    }

    match EstackArgT::from_int(which) {
        Some(EstackArgT::Sfile | EstackArgT::Script) => {
            // Return the top script entry for <sfile> or <script>
            rs_estack_find_script()
        }
        Some(EstackArgT::Stack) => {
            // Return top for <stack>
            len - 1
        }
        Some(EstackArgT::None) | None => -1,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etype_values() {
        // Ensure enum values match C definitions
        assert_eq!(EtypeT::Top as c_int, 0);
        assert_eq!(EtypeT::Script as c_int, 1);
        assert_eq!(EtypeT::Ufunc as c_int, 2);
        assert_eq!(EtypeT::Aucmd as c_int, 3);
        assert_eq!(EtypeT::Modeline as c_int, 4);
        assert_eq!(EtypeT::Except as c_int, 5);
        assert_eq!(EtypeT::Args as c_int, 6);
        assert_eq!(EtypeT::Env as c_int, 7);
        assert_eq!(EtypeT::Internal as c_int, 8);
        assert_eq!(EtypeT::Spell as c_int, 9);
    }

    #[test]
    fn test_estack_arg_values() {
        assert_eq!(EstackArgT::None as c_int, 0);
        assert_eq!(EstackArgT::Sfile as c_int, 1);
        assert_eq!(EstackArgT::Stack as c_int, 2);
        assert_eq!(EstackArgT::Script as c_int, 3);
    }
}
