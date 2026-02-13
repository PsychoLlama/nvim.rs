//! Digraph registration and reverse lookup.
//!
//! Functions for adding user-defined digraphs and looking up digraphs by result.

use std::ffi::c_void;
use std::ptr;

use libc::c_int;

use crate::DigrT;

// C accessor functions for user digraphs garray
extern "C" {
    /// Get pointer to user digraphs array data.
    fn nvim_get_user_digraphs_data() -> *const c_void;

    /// Get length of user digraphs array.
    fn nvim_get_user_digraphs_len() -> c_int;

    /// Get pointer to user digraphs garray for mutation.
    #[allow(dead_code)]
    fn nvim_get_user_digraphs_ptr() -> *mut c_void;

    /// Grow the user digraphs garray by n items.
    fn nvim_user_digraphs_grow(n: c_int);

    /// Increment the user digraphs garray length.
    fn nvim_user_digraphs_inc_len();
}

/// Register a user digraph.
///
/// If the digraph already exists, replaces its result value.
/// Otherwise, appends a new digraph to the user digraphs array.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `result` - The character produced by this digraph
fn registerdigraph_impl(char1: c_int, char2: c_int, result: c_int) {
    #[allow(clippy::cast_sign_loss)]
    let char1_u8 = (char1 & 0xFF) as u8;
    #[allow(clippy::cast_sign_loss)]
    let char2_u8 = (char2 & 0xFF) as u8;

    // Search for existing digraph to update
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data as *mut DigrT;
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;

        for i in 0..len {
            let dp = unsafe { &mut *user_digraphs.add(i) };
            if dp.char1 == char1_u8 && dp.char2 == char2_u8 {
                // Update existing digraph
                dp.result = result;
                return;
            }
        }
    }

    // Add new digraph - grow array and append
    unsafe { nvim_user_digraphs_grow(1) };

    // Get pointer to new slot (after growing)
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() {
        let user_digraphs = user_data as *mut DigrT;
        #[allow(clippy::cast_sign_loss)]
        let idx = user_len as usize;

        let dp = unsafe { &mut *user_digraphs.add(idx) };
        dp.char1 = char1_u8;
        dp.char2 = char2_u8;
        dp.result = result;

        unsafe { nvim_user_digraphs_inc_len() };
    }
}

/// Register a user digraph (FFI export).
#[no_mangle]
pub extern "C" fn rs_registerdigraph(char1: c_int, char2: c_int, result: c_int) {
    registerdigraph_impl(char1, char2, result);
}

/// Find a digraph for a given character value.
///
/// Searches user digraphs first, then default digraphs.
///
/// # Arguments
/// * `val` - The character value to find a digraph for
/// * `out_char1` - Output: first character of found digraph
/// * `out_char2` - Output: second character of found digraph
///
/// # Returns
/// `true` if a digraph was found, `false` otherwise.
fn get_digraph_for_char_impl(val: c_int, out_char1: &mut u8, out_char2: &mut u8) -> bool {
    // Search user digraphs first
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;

        for i in 0..len {
            let dp = unsafe { &*user_digraphs.add(i) };
            if dp.result == val {
                *out_char1 = dp.char1;
                *out_char2 = dp.char2;
                return true;
            }
        }
    }

    // Search default digraphs (now a Rust slice, no FFI hop)
    for dp in crate::data::DIGRAPH_DEFAULT {
        if dp.result == val {
            *out_char1 = dp.char1;
            *out_char2 = dp.char2;
            return true;
        }
    }

    false
}

/// Find a digraph for a given character value (FFI export).
///
/// # Returns
/// `1` if found (`out_char1`/`out_char2` populated), `0` if not found.
///
/// # Safety
/// `out_char1` and `out_char2` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_get_digraph_for_char(
    val: c_int,
    out_char1: *mut u8,
    out_char2: *mut u8,
) -> c_int {
    if out_char1.is_null() || out_char2.is_null() {
        return 0;
    }

    let mut char1: u8 = 0;
    let mut char2: u8 = 0;

    if get_digraph_for_char_impl(val, &mut char1, &mut char2) {
        unsafe {
            ptr::write(out_char1, char1);
            ptr::write(out_char2, char2);
        }
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_truncation() {
        // Verify truncation logic
        let char1: c_int = 0x141; // 'A' + 0x100
        #[allow(clippy::cast_sign_loss)]
        let char1_u8 = (char1 & 0xFF) as u8;
        assert_eq!(char1_u8, b'A');
    }

    #[test]
    fn test_char_truncation_negative() {
        // Negative values truncate to their lower 8 bits
        let neg1: c_int = -1;
        #[allow(clippy::cast_sign_loss)]
        let truncated = (neg1 & 0xFF) as u8;
        assert_eq!(truncated, 0xFF);
    }
}
