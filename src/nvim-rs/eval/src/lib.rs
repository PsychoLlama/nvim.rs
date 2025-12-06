//! Evaluation utilities for Neovim
//!
//! This crate provides functions for evaluating VimL/Lua expressions,
//! including character validation for variable and function names.

use std::ffi::c_int;

/// The autoload character used in function/variable names.
const AUTOLOAD_CHAR: u8 = b'#';

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII letter (A-Z or a-z).
#[inline]
const fn ascii_isalpha(c: u8) -> bool {
    ascii_isupper(c) || ascii_islower(c)
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if a character is an ASCII alphanumeric character (A-Z, a-z, 0-9).
#[inline]
const fn ascii_isalnum(c: u8) -> bool {
    ascii_isalpha(c) || ascii_isdigit(c)
}

/// Check if character `c` can be used in a variable or function name.
/// Does not include '{' or '}' for magic braces.
///
/// Valid characters: alphanumeric, underscore, colon, or autoload char (#).
#[no_mangle]
pub extern "C" fn rs_eval_isnamec(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalnum(c) || c == b'_' || c == b':' || c == AUTOLOAD_CHAR
}

/// Check if character `c` can be used as the first character in a
/// variable or function name (excluding '{' and '}').
///
/// Valid first characters: alphabetic or underscore.
#[no_mangle]
pub extern "C" fn rs_eval_isnamec1(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalpha(c) || c == b'_'
}

/// Check if character `c` can be used as the first character of a
/// dictionary key.
///
/// Valid dictionary key characters: alphanumeric or underscore.
#[no_mangle]
pub extern "C" fn rs_eval_isdictc(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    ascii_isalnum(c) || c == b'_'
}

use std::ffi::c_char;

/// Skip past the v:lua function name.
///
/// Valid characters in a v:lua function name are:
/// - Alphanumeric (A-Z, a-z, 0-9)
/// - Underscore (_)
/// - Hyphen (-)
/// - Dot (.)
/// - Single quote (')
///
/// # Safety
///
/// `p` must be a valid null-terminated C string pointing to the character
/// AFTER the "v:lua." prefix.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub unsafe extern "C" fn rs_skip_luafunc_name(p: *const c_char) -> *const c_char {
    let mut ptr = p;
    loop {
        #[allow(clippy::cast_sign_loss)]
        let c = *ptr as u8;
        if ascii_isalnum(c) || c == b'_' || c == b'-' || c == b'.' || c == b'\'' {
            ptr = ptr.add(1);
        } else {
            break;
        }
    }
    ptr
}

/// Check the function name after "v:lua.".
///
/// Returns the length of the function name if valid, 0 otherwise.
/// If `paren` is true, the name must be followed by '('.
/// If `paren` is false, the name must be followed by NUL.
///
/// # Safety
///
/// `str` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_luafunc_name(str: *const c_char, paren: bool) -> c_int {
    let end = rs_skip_luafunc_name(str);
    let expected_char = if paren { b'(' } else { 0 };
    #[allow(clippy::cast_sign_loss)]
    if *end as u8 != expected_char {
        return 0;
    }
    // Calculate the length
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let len = end.offset_from(str) as c_int;
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_isnamec() {
        // Alphanumeric
        assert!(rs_eval_isnamec(c_int::from(b'a')));
        assert!(rs_eval_isnamec(c_int::from(b'Z')));
        assert!(rs_eval_isnamec(c_int::from(b'0')));
        assert!(rs_eval_isnamec(c_int::from(b'9')));

        // Special allowed characters
        assert!(rs_eval_isnamec(c_int::from(b'_')));
        assert!(rs_eval_isnamec(c_int::from(b':')));
        assert!(rs_eval_isnamec(c_int::from(b'#'))); // AUTOLOAD_CHAR

        // Not allowed
        assert!(!rs_eval_isnamec(c_int::from(b'{')));
        assert!(!rs_eval_isnamec(c_int::from(b'}')));
        assert!(!rs_eval_isnamec(c_int::from(b' ')));
        assert!(!rs_eval_isnamec(c_int::from(b'.')));
        assert!(!rs_eval_isnamec(-1));
        assert!(!rs_eval_isnamec(256));
    }

    #[test]
    fn test_eval_isnamec1() {
        // Alphabetic
        assert!(rs_eval_isnamec1(c_int::from(b'a')));
        assert!(rs_eval_isnamec1(c_int::from(b'Z')));
        assert!(rs_eval_isnamec1(c_int::from(b'_')));

        // Not allowed as first char
        assert!(!rs_eval_isnamec1(c_int::from(b'0')));
        assert!(!rs_eval_isnamec1(c_int::from(b':')));
        assert!(!rs_eval_isnamec1(c_int::from(b'#')));
        assert!(!rs_eval_isnamec1(-1));
    }

    #[test]
    fn test_eval_isdictc() {
        // Alphanumeric
        assert!(rs_eval_isdictc(c_int::from(b'a')));
        assert!(rs_eval_isdictc(c_int::from(b'Z')));
        assert!(rs_eval_isdictc(c_int::from(b'0')));
        assert!(rs_eval_isdictc(c_int::from(b'_')));

        // Not allowed
        assert!(!rs_eval_isdictc(c_int::from(b':')));
        assert!(!rs_eval_isdictc(c_int::from(b'#')));
        assert!(!rs_eval_isdictc(c_int::from(b' ')));
        assert!(!rs_eval_isdictc(-1));
    }

    #[test]
    fn test_skip_luafunc_name() {
        use std::ffi::CString;

        let simple = CString::new("myfunc(").unwrap();
        let dotted = CString::new("module.func(").unwrap();
        let with_hyphen = CString::new("my-func(").unwrap();
        let with_quote = CString::new("require'foo'(").unwrap();
        let empty = CString::new("(").unwrap();
        let with_underscore = CString::new("my_func_name").unwrap();

        unsafe {
            // Simple function name
            let result = rs_skip_luafunc_name(simple.as_ptr());
            assert_eq!(*result as u8, b'(');

            // Dotted module path
            let result = rs_skip_luafunc_name(dotted.as_ptr());
            assert_eq!(*result as u8, b'(');

            // With hyphen
            let result = rs_skip_luafunc_name(with_hyphen.as_ptr());
            assert_eq!(*result as u8, b'(');

            // With single quote
            let result = rs_skip_luafunc_name(with_quote.as_ptr());
            assert_eq!(*result as u8, b'(');

            // Empty (starts with '(')
            let result = rs_skip_luafunc_name(empty.as_ptr());
            assert_eq!(*result as u8, b'(');

            // Ends with NUL
            let result = rs_skip_luafunc_name(with_underscore.as_ptr());
            assert_eq!(*result as u8, 0);
        }
    }

    #[test]
    fn test_check_luafunc_name() {
        use std::ffi::CString;

        let valid_paren = CString::new("myfunc(").unwrap();
        let valid_nul = CString::new("myfunc").unwrap();
        let invalid_paren = CString::new("myfunc").unwrap();
        let invalid_nul = CString::new("myfunc(").unwrap();
        let empty = CString::new("(").unwrap();

        unsafe {
            // Valid with paren=true
            assert_eq!(rs_check_luafunc_name(valid_paren.as_ptr(), true), 6);

            // Valid with paren=false
            assert_eq!(rs_check_luafunc_name(valid_nul.as_ptr(), false), 6);

            // Invalid: expects paren but not found
            assert_eq!(rs_check_luafunc_name(invalid_paren.as_ptr(), true), 0);

            // Invalid: expects NUL but found paren
            assert_eq!(rs_check_luafunc_name(invalid_nul.as_ptr(), false), 0);

            // Empty name (length 0) is still valid if followed by expected char
            assert_eq!(rs_check_luafunc_name(empty.as_ptr(), true), 0);
        }
    }
}
