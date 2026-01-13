//! Evaluation utilities for Neovim
//!
//! This crate provides functions for evaluating VimL/Lua expressions,
//! including character validation for variable and function names,
//! and type conversion utilities.
//!
//! ## Modules
//!
//! - [`funcs`]: VimL built-in functions (abs, sin, split, join, etc.)

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

pub mod funcs;
pub mod operator;
pub mod types;

use std::ffi::c_int;
use std::sync::atomic::{AtomicI32, Ordering};

// =============================================================================
// CopyID for recursive traversal
// =============================================================================

/// Increment value for copy IDs (last bit is reserved for previous_funccal).
const COPYID_INC: i32 = 2;

/// Current copy ID for traversing lists and dicts.
/// This is used to avoid endless recursiveness when serializing or garbage collecting.
static CURRENT_COPY_ID: AtomicI32 = AtomicI32::new(0);

/// Get the next (unique) copy ID.
///
/// Used for traversing nested structures (e.g., when serializing them or
/// garbage collecting). Increments by 2 because the last bit is used for
/// `previous_funccal` and normally ignored when comparing.
#[no_mangle]
pub extern "C" fn rs_get_copyID() -> c_int {
    CURRENT_COPY_ID.fetch_add(COPYID_INC, Ordering::Relaxed) + COPYID_INC
}

// =============================================================================
// TriState type and conversions
// =============================================================================

// C TriState enum values (from types_defs.h)
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;

/// Convert an integer to a TriState value.
///
/// - `val == 0` -> `kFalse`
/// - `val >= 1` -> `kTrue`
/// - `val < 0` (or other) -> `kNone`
///
/// Equivalent to C macro `TRISTATE_FROM_INT(val)`.
#[no_mangle]
pub extern "C" fn rs_tristate_from_int(val: c_int) -> c_int {
    if val == 0 {
        K_FALSE
    } else if val >= 1 {
        K_TRUE
    } else {
        K_NONE
    }
}

/// Convert a TriState value to a boolean with a default.
///
/// - `val == kTrue` -> `true`
/// - `val == kFalse` -> `false`
/// - `val == kNone` -> `default`
///
/// Equivalent to C macro `TRISTATE_TO_BOOL(val, default)`.
#[no_mangle]
pub extern "C" fn rs_tristate_to_bool(val: c_int, default: bool) -> bool {
    if val == K_TRUE {
        true
    } else if val == K_FALSE {
        false
    } else {
        default
    }
}

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

// =============================================================================
// Eval State Accessors
// =============================================================================

extern "C" {
    fn nvim_get_callback_depth() -> c_int;
    fn nvim_get_echo_hl_id() -> c_int;
}

/// Get the current callback nesting depth.
///
/// # Safety
/// Calls C accessor function for callback_depth static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_callback_depth() -> c_int {
    nvim_get_callback_depth()
}

/// Get the :echo highlight id.
///
/// # Safety
/// Calls C accessor function for echo_hl_id static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_echo_hl_id() -> c_int {
    nvim_get_echo_hl_id()
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

extern "C" {
    fn nvim_get_current_funccal_fc_returned() -> c_int;
}

/// Check if the current function was ended by a ":return" command.
///
/// # Safety
/// Calls C accessor function for function call state.
#[no_mangle]
pub unsafe extern "C" fn rs_current_func_returned() -> c_int {
    nvim_get_current_funccal_fc_returned()
}

// =============================================================================
// Partial Functions
// =============================================================================

/// Opaque handle for partial_T struct
type PartialHandle = *const std::ffi::c_void;

extern "C" {
    fn nvim_partial_get_pt_name(pt: PartialHandle) -> *mut c_char;
    fn nvim_partial_get_pt_func_uf_name(pt: PartialHandle) -> *mut c_char;
}

/// Default empty string for partial_name return value.
static EMPTY_STRING: &[u8] = b"\0";

extern "C" {
    fn nvim_get_vlua_partial() -> PartialHandle;
}

/// Check if the given partial is the special v:lua value for calling lua functions.
///
/// # Safety
///
/// `partial` may be null (returns false).
#[no_mangle]
pub unsafe extern "C" fn rs_is_luafunc(partial: PartialHandle) -> bool {
    partial == nvim_get_vlua_partial()
}

/// Get the function name of a partial.
///
/// Returns the pt_name if set, otherwise pt_func->uf_name if set,
/// otherwise an empty string.
///
/// # Safety
///
/// `pt` may be null (returns empty string).
/// If non-null, `pt` must be a valid pointer to a partial_T struct.
#[no_mangle]
#[allow(clippy::as_ptr_cast_mut)]
pub unsafe extern "C" fn rs_partial_name(pt: PartialHandle) -> *mut c_char {
    if pt.is_null() {
        // Safe: caller should not mutate the returned empty string
        return EMPTY_STRING.as_ptr() as *mut c_char;
    }

    let pt_name = nvim_partial_get_pt_name(pt);
    if !pt_name.is_null() {
        return pt_name;
    }

    let func_name = nvim_partial_get_pt_func_uf_name(pt);
    if !func_name.is_null() {
        return func_name;
    }

    // Safe: caller should not mutate the returned empty string
    EMPTY_STRING.as_ptr() as *mut c_char
}

// =============================================================================
// Variable Scope Constants
// =============================================================================

/// Variable scope types (for variable lookup).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VarScope {
    /// Global scope (g:)
    #[default]
    Global = 0,
    /// Local function scope (l:)
    Local = 1,
    /// Script scope (s:)
    Script = 2,
    /// Argument scope (a:)
    Argument = 3,
    /// Vim predefined scope (v:)
    Vim = 4,
    /// Buffer local (b:)
    Buffer = 5,
    /// Window local (w:)
    Window = 6,
    /// Tab page local (t:)
    Tab = 7,
}

impl VarScope {
    /// Convert from raw integer.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Global,
            1 => Self::Local,
            2 => Self::Script,
            3 => Self::Argument,
            4 => Self::Vim,
            5 => Self::Buffer,
            6 => Self::Window,
            7 => Self::Tab,
            _ => Self::Global,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// FFI export: Get VarScope::Global constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_global() -> c_int {
    VarScope::Global.to_raw()
}

/// FFI export: Get VarScope::Local constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_local() -> c_int {
    VarScope::Local.to_raw()
}

/// FFI export: Get VarScope::Script constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_script() -> c_int {
    VarScope::Script.to_raw()
}

/// FFI export: Get VarScope::Argument constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_argument() -> c_int {
    VarScope::Argument.to_raw()
}

/// FFI export: Get VarScope::Vim constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_vim() -> c_int {
    VarScope::Vim.to_raw()
}

/// FFI export: Get VarScope::Buffer constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_buffer() -> c_int {
    VarScope::Buffer.to_raw()
}

/// FFI export: Get VarScope::Window constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_window() -> c_int {
    VarScope::Window.to_raw()
}

/// FFI export: Get VarScope::Tab constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_tab() -> c_int {
    VarScope::Tab.to_raw()
}

/// FFI export: Get VarFlavour::Default constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_flavour_default() -> c_int {
    VarFlavour::Default as c_int
}

/// FFI export: Get VarFlavour::Session constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_flavour_session() -> c_int {
    VarFlavour::Session as c_int
}

/// FFI export: Get VarFlavour::Shada constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_var_flavour_shada() -> c_int {
    VarFlavour::Shada as c_int
}

/// FFI export: Get COPYID_INC constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_copyid_inc() -> c_int {
    COPYID_INC
}

/// FFI export: Get kNone constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_k_none() -> c_int {
    K_NONE
}

/// FFI export: Get kFalse constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_k_false() -> c_int {
    K_FALSE
}

/// FFI export: Get kTrue constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_k_true() -> c_int {
    K_TRUE
}

/// FFI export: Get AUTOLOAD_CHAR constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_autoload_char() -> c_int {
    c_int::from(AUTOLOAD_CHAR)
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

    #[test]
    fn test_tristate_from_int() {
        // Zero -> kFalse (0)
        assert_eq!(rs_tristate_from_int(0), K_FALSE);

        // Positive -> kTrue (1)
        assert_eq!(rs_tristate_from_int(1), K_TRUE);
        assert_eq!(rs_tristate_from_int(42), K_TRUE);
        assert_eq!(rs_tristate_from_int(i32::MAX), K_TRUE);

        // Negative -> kNone (-1)
        assert_eq!(rs_tristate_from_int(-1), K_NONE);
        assert_eq!(rs_tristate_from_int(-42), K_NONE);
        assert_eq!(rs_tristate_from_int(i32::MIN), K_NONE);
    }

    #[test]
    fn test_tristate_to_bool() {
        // kTrue -> true regardless of default
        assert!(rs_tristate_to_bool(K_TRUE, false));
        assert!(rs_tristate_to_bool(K_TRUE, true));

        // kFalse -> false regardless of default
        assert!(!rs_tristate_to_bool(K_FALSE, false));
        assert!(!rs_tristate_to_bool(K_FALSE, true));

        // kNone -> default
        assert!(!rs_tristate_to_bool(K_NONE, false));
        assert!(rs_tristate_to_bool(K_NONE, true));
    }

    #[test]
    fn test_get_copyid() {
        // Each call should return a value 2 greater than the last
        let first = rs_get_copyID();
        let second = rs_get_copyID();
        let third = rs_get_copyID();

        assert_eq!(second - first, 2);
        assert_eq!(third - second, 2);
    }

    #[test]
    fn test_tristate_constants() {
        // Verify tristate constants match C definitions
        assert_eq!(K_NONE, -1);
        assert_eq!(K_FALSE, 0);
        assert_eq!(K_TRUE, 1);
    }

    #[test]
    fn test_copyid_inc_constant() {
        // Verify COPYID_INC matches expected value
        assert_eq!(COPYID_INC, 2);
    }

    #[test]
    fn test_autoload_char_constant() {
        // Verify AUTOLOAD_CHAR matches C definition
        assert_eq!(AUTOLOAD_CHAR, b'#');
    }

    #[test]
    fn test_var_flavour_enum_values() {
        // Verify VarFlavour enum values match C definition
        assert_eq!(VarFlavour::Default as c_int, 1);
        assert_eq!(VarFlavour::Session as c_int, 2);
        assert_eq!(VarFlavour::Shada as c_int, 4);
    }

    #[test]
    fn test_ascii_helpers() {
        // Test ascii_isupper
        for c in b'A'..=b'Z' {
            assert!(ascii_isupper(c));
        }
        assert!(!ascii_isupper(b'a'));
        assert!(!ascii_isupper(b'0'));

        // Test ascii_islower
        for c in b'a'..=b'z' {
            assert!(ascii_islower(c));
        }
        assert!(!ascii_islower(b'A'));
        assert!(!ascii_islower(b'0'));

        // Test ascii_isdigit
        for c in b'0'..=b'9' {
            assert!(ascii_isdigit(c));
        }
        assert!(!ascii_isdigit(b'a'));

        // Test ascii_isalpha
        for c in b'A'..=b'Z' {
            assert!(ascii_isalpha(c));
        }
        for c in b'a'..=b'z' {
            assert!(ascii_isalpha(c));
        }
        assert!(!ascii_isalpha(b'0'));

        // Test ascii_isalnum
        for c in b'0'..=b'9' {
            assert!(ascii_isalnum(c));
        }
        for c in b'A'..=b'Z' {
            assert!(ascii_isalnum(c));
        }
        for c in b'a'..=b'z' {
            assert!(ascii_isalnum(c));
        }
        assert!(!ascii_isalnum(b' '));
    }
}
