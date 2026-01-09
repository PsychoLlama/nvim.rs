//! Sign definition management
//!
//! This module handles creating, updating, and removing sign definitions.
//! Sign definitions specify the visual properties of signs (text, highlights, icons).

use std::ffi::{c_char, c_int};

use crate::{DecorSignHighlightHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // Sign definition accessors
    fn nvim_sign_get_name(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_icon(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_text_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_line_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_num_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_cul_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;

    // Sign map operations
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;
    fn nvim_sign_map_has(name: *const c_char) -> bool;
}

// =============================================================================
// Sign Definition Queries
// =============================================================================

/// Check if a sign with the given name is defined.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_is_defined(name: *const c_char) -> bool {
    if name.is_null() {
        return false;
    }
    nvim_sign_map_has(name)
}

/// Get a sign definition by name.
///
/// Returns a null handle if the sign is not defined.
///
/// # Safety
///
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_by_name(name: *const c_char) -> SignHandle {
    if name.is_null() {
        return SignHandle::null();
    }
    nvim_sign_map_get(name)
}

// =============================================================================
// Sign Definition Properties
// =============================================================================

/// Get the name of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_name(sp: SignHandle) -> *const c_char {
    if sp.is_null() {
        return std::ptr::null();
    }
    nvim_sign_get_name(sp)
}

/// Get the icon path of a sign definition.
///
/// Returns null if no icon is set.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_icon(sp: SignHandle) -> *const c_char {
    if sp.is_null() {
        return std::ptr::null();
    }
    nvim_sign_get_icon(sp)
}

/// Get the text highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_text_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_text_hl(sp)
}

/// Get the line highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_line_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_line_hl(sp)
}

/// Get the number column highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_num_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_num_hl(sp)
}

/// Get the cursorline highlight ID of a sign definition.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_cul_hl(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    nvim_sign_get_cul_hl(sp)
}

/// Get the priority of a sign definition.
///
/// Returns SIGN_DEF_PRIO if not explicitly set (-1) or if handle is null.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_get_priority(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return SIGN_DEF_PRIO;
    }
    let prio = nvim_sign_get_priority(sp);
    if prio == -1 {
        SIGN_DEF_PRIO
    } else {
        prio
    }
}

/// Check if a sign definition has any highlight configured.
///
/// Returns true if text_hl, line_hl, num_hl, or cul_hl is set.
///
/// # Safety
///
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_def_has_highlight(sp: SignHandle) -> bool {
    if sp.is_null() {
        return false;
    }
    nvim_sign_get_text_hl(sp) > 0
        || nvim_sign_get_line_hl(sp) > 0
        || nvim_sign_get_num_hl(sp) > 0
        || nvim_sign_get_cul_hl(sp) > 0
}

// =============================================================================
// DecorSignHighlight Properties
// =============================================================================

extern "C" {
    fn nvim_decor_sh_get_priority(sh: DecorSignHighlightHandle) -> u16;
    fn nvim_decor_sh_get_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char;
    fn nvim_decor_sh_get_sign_add_id(sh: DecorSignHighlightHandle) -> c_int;
}

/// Get the priority of a placed sign (from DecorSignHighlight).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_priority(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return SIGN_DEF_PRIO;
    }
    c_int::from(nvim_decor_sh_get_priority(sh))
}

/// Get the sign name from a placed sign (from DecorSignHighlight).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char {
    if sh.is_null() {
        return std::ptr::null();
    }
    nvim_decor_sh_get_sign_name(sh)
}

/// Get the text highlight ID from a placed sign.
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_hl_id(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return 0;
    }
    nvim_decor_sh_get_hl_id(sh)
}

/// Get the sign_add_id from a placed sign (used for sorting recency).
///
/// # Safety
///
/// `sh` must be a valid DecorSignHighlight handle.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_sh_get_sign_add_id(sh: DecorSignHighlightHandle) -> c_int {
    if sh.is_null() {
        return 0;
    }
    nvim_decor_sh_get_sign_add_id(sh)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_handle_null() {
        let handle = SignHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_decor_sh_handle_null() {
        let handle = DecorSignHighlightHandle::null();
        assert!(handle.is_null());
    }
}
