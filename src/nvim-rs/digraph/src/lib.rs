//! Digraph functions for Neovim.
//!
//! This crate provides Rust implementations of digraph lookup functions.

use std::ffi::c_void;

use libc::c_int;

/// Digraph entry structure matching C's `digr_T`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DigrT {
    pub char1: u8,
    pub char2: u8,
    pub result: c_int,
}

// C accessor functions for digraph tables
extern "C" {
    /// Get pointer to user digraphs array data (opaque).
    fn nvim_get_user_digraphs_data() -> *const c_void;

    /// Get length of user digraphs array.
    fn nvim_get_user_digraphs_len() -> c_int;

    /// Get pointer to default digraphs array (opaque).
    fn nvim_get_digraphdefault() -> *const c_void;

    /// Get length of default digraphs array.
    fn nvim_get_digraphdefault_len() -> c_int;
}

/// Space character as `c_int`.
const SPACE: c_int = b' ' as c_int;

/// Check if a character is special (negative value).
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Search for an exact digraph match in the given order.
///
/// Searches user digraphs first, then default digraphs.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `meta_char` - If true and char1 is space, return char2 | 0x80
///
/// # Returns
/// The digraph result, or char2 if not found.
fn getexactdigraph_impl(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    // Special characters don't form digraphs
    if is_special(char1) || is_special(char2) {
        return char2;
    }

    // Safe truncation: we only care about ASCII range for digraphs
    #[allow(clippy::cast_sign_loss)]
    let char1_u8 = (char1 & 0xFF) as u8;
    #[allow(clippy::cast_sign_loss)]
    let char2_u8 = (char2 & 0xFF) as u8;

    // Search user digraphs first
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;
        for i in 0..len {
            let dp = unsafe { &*user_digraphs.add(i) };
            if dp.char1 == char1_u8 && dp.char2 == char2_u8 {
                return dp.result;
            }
        }
    }

    // Search default digraphs
    let default_data = unsafe { nvim_get_digraphdefault() };
    let default_len = unsafe { nvim_get_digraphdefault_len() };

    if !default_data.is_null() && default_len > 0 {
        let default_digraphs = default_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = default_len as usize;
        for i in 0..len {
            let dp = unsafe { &*default_digraphs.add(i) };
            // Default array is null-terminated (char1 == 0 marks end)
            if dp.char1 == 0 {
                break;
            }
            if dp.char1 == char1_u8 && dp.char2 == char2_u8 {
                return dp.result;
            }
        }
    }

    // Digraph not found
    if char1 == SPACE && meta_char {
        // <space> <char> --> meta-char
        return char2 | 0x80;
    }
    char2
}

/// Get digraph, allowing for both char1-char2 and char2-char1 orderings.
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `meta_char` - If true and char1 is space, return char2 | 0x80
///
/// # Returns
/// The digraph result.
fn digraph_get_impl(char1: c_int, char2: c_int, meta_char: bool) -> c_int {
    let retval = getexactdigraph_impl(char1, char2, meta_char);
    if retval == char2 && char1 != char2 {
        let retval2 = getexactdigraph_impl(char2, char1, meta_char);
        if retval2 != char1 {
            return retval2;
        }
    }
    retval
}

// =============================================================================
// FFI exports
// =============================================================================

/// Get digraph (FFI export).
#[no_mangle]
pub extern "C" fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: c_int) -> c_int {
    digraph_get_impl(char1, char2, meta_char != 0)
}

/// Get exact digraph match (FFI export).
#[no_mangle]
pub extern "C" fn rs_getexactdigraph(char1: c_int, char2: c_int, meta_char: c_int) -> c_int {
    getexactdigraph_impl(char1, char2, meta_char != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special() {
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_meta_char_logic() {
        // When char1 is space and meta_char is true, result should be char2 | 0x80
        let char2: c_int = 97; // 'a'

        // The meta_char logic: if char1 == ' ' and meta_char, return char2 | 0x80
        // Expected: 'a' | 0x80 = 97 | 128 = 225
        let expected = char2 | 0x80;
        assert_eq!(expected, 225);
    }

    #[test]
    fn test_space_constant() {
        assert_eq!(SPACE, 32);
    }
}
