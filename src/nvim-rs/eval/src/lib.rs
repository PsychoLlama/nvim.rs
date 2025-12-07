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

/// Variable flavour types for persistence (`ShaDa`) handling.
///
/// These match the C enum `var_flavour_T` in eval.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarFlavour {
    /// Variable doesn't start with uppercase
    Default = 1,
    /// Variable starts with uppercase, has some lowercase
    Session = 2,
    /// Variable is all uppercase
    Shada = 4,
}

/// Determine the "flavour" of a variable name for persistence handling.
///
/// - All uppercase (e.g., "FOO") -> Shada
/// - Starts with uppercase but has lowercase (e.g., "Foo") -> Session
/// - Starts with lowercase -> Default
///
/// # Safety
///
/// `varname` must be a valid null-terminated C string.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub unsafe extern "C" fn rs_var_flavour(varname: *const c_char) -> VarFlavour {
    if varname.is_null() {
        return VarFlavour::Default;
    }

    let mut p = varname;

    // Check first character - must be uppercase to be Session or Shada
    #[allow(clippy::cast_sign_loss)]
    let first = *p as u8;
    if !ascii_isupper(first) {
        return VarFlavour::Default;
    }

    // Move to next character and check all remaining
    p = p.add(1);
    loop {
        #[allow(clippy::cast_sign_loss)]
        let c = *p as u8;
        if c == 0 {
            break;
        }
        // If any lowercase letter found, it's Session flavour
        if ascii_islower(c) {
            return VarFlavour::Session;
        }
        p = p.add(1);
    }

    // All uppercase -> Shada flavour
    VarFlavour::Shada
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
    #[allow(clippy::cast_sign_loss)]
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

    #[test]
    fn test_var_flavour() {
        use std::ffi::CString;

        // All uppercase -> Shada
        let all_upper = CString::new("FOO").unwrap();
        let single_upper = CString::new("X").unwrap();
        let upper_with_numbers = CString::new("FOO123").unwrap();
        let upper_underscore = CString::new("FOO_BAR").unwrap();

        // Mixed case (starts uppercase with lowercase) -> Session
        let mixed = CString::new("Foo").unwrap();
        let mixed2 = CString::new("FooBar").unwrap();
        let mixed_mid = CString::new("FOo").unwrap();

        // Starts lowercase -> Default
        let lower = CString::new("foo").unwrap();
        let lower_mixed = CString::new("fooBar").unwrap();
        let underscore_start = CString::new("_foo").unwrap();
        let number_start = CString::new("123foo").unwrap();
        let empty = CString::new("").unwrap();

        unsafe {
            // All uppercase -> Shada
            assert_eq!(rs_var_flavour(all_upper.as_ptr()), VarFlavour::Shada);
            assert_eq!(rs_var_flavour(single_upper.as_ptr()), VarFlavour::Shada);
            assert_eq!(
                rs_var_flavour(upper_with_numbers.as_ptr()),
                VarFlavour::Shada
            );
            assert_eq!(rs_var_flavour(upper_underscore.as_ptr()), VarFlavour::Shada);

            // Mixed case -> Session
            assert_eq!(rs_var_flavour(mixed.as_ptr()), VarFlavour::Session);
            assert_eq!(rs_var_flavour(mixed2.as_ptr()), VarFlavour::Session);
            assert_eq!(rs_var_flavour(mixed_mid.as_ptr()), VarFlavour::Session);

            // Starts lowercase/other -> Default
            assert_eq!(rs_var_flavour(lower.as_ptr()), VarFlavour::Default);
            assert_eq!(rs_var_flavour(lower_mixed.as_ptr()), VarFlavour::Default);
            assert_eq!(
                rs_var_flavour(underscore_start.as_ptr()),
                VarFlavour::Default
            );
            assert_eq!(rs_var_flavour(number_start.as_ptr()), VarFlavour::Default);
            assert_eq!(rs_var_flavour(empty.as_ptr()), VarFlavour::Default);

            // Null pointer -> Default
            assert_eq!(rs_var_flavour(std::ptr::null()), VarFlavour::Default);
        }
    }
}
