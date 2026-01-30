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
    fn nvim_curbuf_get_p_lw() -> *const c_char;
    fn nvim_get_p_lispwords() -> *const c_char;
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

// =============================================================================
// Lisp indentation helpers
// =============================================================================

/// Check if a string matches a word in the lispwords list.
///
/// Used by `get_lisp_indent()` to determine if certain keywords require
/// "body" indenting rules (e.g., `let`, `do`, etc.).
///
/// # Safety
/// - `p` must point to a valid null-terminated C string
/// - Accesses current buffer state for 'lispwords' option
#[no_mangle]
pub unsafe extern "C" fn rs_lisp_match(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Get the lispwords option - prefer buffer-local, fall back to global
    let b_lw = nvim_curbuf_get_p_lw();
    let word = if !b_lw.is_null() && *b_lw != 0 {
        b_lw
    } else {
        nvim_get_p_lispwords()
    };

    if word.is_null() {
        return 0;
    }

    // Iterate through comma-separated words in lispwords
    let mut word_ptr = word;
    while *word_ptr != 0 {
        // Find the end of the current word (terminated by comma or NUL)
        let word_start = word_ptr;
        let mut word_len = 0usize;
        while *word_ptr != 0 && *word_ptr != b',' as c_char {
            word_len += 1;
            word_ptr = word_ptr.add(1);
        }

        // Compare the word with p
        if word_len > 0 {
            let mut matches = true;
            for i in 0..word_len {
                if *word_start.add(i) != *p.add(i) {
                    matches = false;
                    break;
                }
            }
            // Also check that p[word_len] is whitespace or NUL
            if matches {
                let next_char = *p.add(word_len);
                if next_char == 0 || next_char == b' ' as c_char || next_char == b'\t' as c_char {
                    return 1;
                }
            }
        }

        // Skip the comma separator
        if *word_ptr == b',' as c_char {
            word_ptr = word_ptr.add(1);
        }
    }

    0
}

#[cfg(test)]
mod tests {
    // Tests would require mocking C functions
}
