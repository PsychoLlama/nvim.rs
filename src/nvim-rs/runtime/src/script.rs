//! Script item management
//!
//! This module handles scriptitem_T tracking for sourced scripts.

use std::ffi::{c_char, c_int};

use crate::{ScidT, ScriptItemHandle};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    fn nvim_script_items_get_len() -> c_int;
    fn nvim_script_item_get(id: ScidT) -> ScriptItemHandle;
    fn nvim_scriptitem_get_name(si: ScriptItemHandle) -> *const c_char;
    fn nvim_scriptitem_is_lua(si: ScriptItemHandle) -> bool;
    fn nvim_scriptitem_get_prof_on(si: ScriptItemHandle) -> bool;
}

// =============================================================================
// Script Item Access
// =============================================================================

/// Get a script item by ID.
///
/// Returns null handle if ID is invalid.
pub unsafe fn rs_script_item_get(id: ScidT) -> ScriptItemHandle {
    if id <= 0 || id > nvim_script_items_get_len() {
        return ScriptItemHandle::null();
    }
    nvim_script_item_get(id)
}

/// Get the name of a script item.
pub unsafe fn rs_script_item_name(si: ScriptItemHandle) -> *const c_char {
    if si.is_null() {
        return std::ptr::null();
    }
    nvim_scriptitem_get_name(si)
}

/// Check if a script item is a Lua script.
pub unsafe fn rs_script_item_is_lua(si: ScriptItemHandle) -> bool {
    if si.is_null() {
        return false;
    }
    nvim_scriptitem_is_lua(si)
}

/// Check if a script item has profiling enabled.
pub unsafe fn rs_script_item_profiling(si: ScriptItemHandle) -> bool {
    if si.is_null() {
        return false;
    }
    nvim_scriptitem_get_prof_on(si)
}

// =============================================================================
// Script ID Utilities
// =============================================================================

/// Get the total number of sourced scripts.
pub unsafe fn rs_script_count() -> c_int {
    nvim_script_items_get_len()
}

/// Check if a script ID is valid.
pub unsafe fn rs_script_id_is_valid(id: ScidT) -> bool {
    id > 0 && id <= nvim_script_items_get_len()
}

/// Get the name of a script by ID.
///
/// Returns null if ID is invalid.
pub unsafe fn rs_script_name_by_id(id: ScidT) -> *const c_char {
    let si = rs_script_item_get(id);
    rs_script_item_name(si)
}

// =============================================================================
// Script Search
// =============================================================================

/// Find a script by name.
///
/// Returns the script ID if found, 0 if not found.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
pub unsafe fn rs_script_find_by_name(name: *const c_char) -> ScidT {
    if name.is_null() {
        return 0;
    }

    let count = nvim_script_items_get_len();
    for id in 1..=count {
        let si = nvim_script_item_get(id);
        let si_name = nvim_scriptitem_get_name(si);

        if !si_name.is_null() && c_str_eq(name, si_name) {
            return id;
        }
    }

    0
}

/// Compare two C strings for equality.
unsafe fn c_str_eq(a: *const c_char, b: *const c_char) -> bool {
    if a.is_null() || b.is_null() {
        return a.is_null() && b.is_null();
    }

    let mut pa = a;
    let mut pb = b;

    loop {
        let ca = *pa;
        let cb = *pb;

        if ca != cb {
            return false;
        }
        if ca == 0 {
            return true;
        }

        pa = pa.add(1);
        pb = pb.add(1);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_null() {
        assert!(ScriptItemHandle::null().is_null());
    }

    #[test]
    fn test_c_str_eq() {
        unsafe {
            use std::ffi::CString;

            let a = CString::new("test").unwrap();
            let b = CString::new("test").unwrap();
            let c = CString::new("other").unwrap();

            assert!(c_str_eq(a.as_ptr(), b.as_ptr()));
            assert!(!c_str_eq(a.as_ptr(), c.as_ptr()));
            assert!(c_str_eq(std::ptr::null(), std::ptr::null()));
            assert!(!c_str_eq(a.as_ptr(), std::ptr::null()));
        }
    }
}
