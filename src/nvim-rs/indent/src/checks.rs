//! Boolean check functions for indentation.
//!
//! This module provides simple predicate functions for indentation decisions.

use std::ffi::{c_char, c_int};

// External C accessor functions
extern "C" {
    fn nvim_curbuf_get_p_si() -> c_int;
    fn nvim_curbuf_get_p_cin() -> c_int;
    fn nvim_curbuf_get_ind_hash_comment() -> c_int;
    fn nvim_curbuf_get_p_lisp() -> c_int;
    fn nvim_curbuf_get_inde_ptr() -> *const c_char;
    fn nvim_curbuf_get_p_lop() -> *const c_char;
    fn nvim_in_cinkeys(keytyped: c_int, when: c_char, line_is_empty: bool) -> bool;
}

// =============================================================================
// Preprocessor and indentation checks
// =============================================================================

/// Check if lines starting with '#' should be left aligned.
///
/// Returns true if:
/// - 'smartindent' is set and 'cindent' is off, OR
/// - 'cindent' is on and '#' is in 'cinkeys' and b_ind_hash_comment is 0
///
/// # Safety
/// Accesses current buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_preprocs_left() -> bool {
    let si = nvim_curbuf_get_p_si() != 0;
    let cin = nvim_curbuf_get_p_cin() != 0;

    if si && !cin {
        return true;
    }

    if cin && nvim_in_cinkeys(b'#' as c_int, b' ' as c_char, true) {
        return nvim_curbuf_get_ind_hash_comment() == 0;
    }

    false
}

/// Check if 'indentexpr' should be used for Lisp indenting.
///
/// Returns true if:
/// - 'lisp' is set, AND
/// - 'indentexpr' is non-empty, AND
/// - 'lispoptions' is "expr:1"
///
/// # Safety
/// Accesses current buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_use_indentexpr_for_lisp() -> bool {
    // Check if lisp is set
    if nvim_curbuf_get_p_lisp() == 0 {
        return false;
    }

    // Check if indentexpr is non-empty
    let inde = nvim_curbuf_get_inde_ptr();
    if inde.is_null() || *inde == 0 {
        return false;
    }

    // Check if lispoptions is "expr:1"
    let lop = nvim_curbuf_get_p_lop();
    if lop.is_null() {
        return false;
    }

    // Compare with "expr:1"
    let expected = b"expr:1\0";
    let mut i = 0;
    loop {
        let c1 = *lop.add(i) as u8;
        let c2 = expected[i];
        if c1 != c2 {
            return false;
        }
        if c1 == 0 {
            return true;
        }
        i += 1;
        if i >= expected.len() {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would require mocking C functions
}
