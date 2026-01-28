//! Unified option accessor API
//!
//! This module provides a consolidated API for getting and setting option values.
//! It unifies the scattered option accessors into a single, type-safe interface.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit
#![allow(clippy::cast_possible_wrap)] // FFI with C char types

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Option Value Types
// =============================================================================

/// Option value type enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptValType {
    /// Nil/null value
    Nil = -1,
    /// Boolean (on/off)
    Boolean = 0,
    /// Number (integer)
    Number = 1,
    /// String
    String = 2,
}

impl OptValType {
    /// Convert from C integer
    pub fn from_c_int(v: c_int) -> Self {
        match v {
            0 => Self::Boolean,
            1 => Self::Number,
            2 => Self::String,
            _ => Self::Nil,
        }
    }

    /// Convert to C integer
    pub fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Option Scope
// =============================================================================

/// Option scope for get/set operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptAccessScope {
    /// Global scope only
    Global = 0x01,
    /// Window-local scope
    Local = 0x02,
    /// Both global and local (for :set without modifiers)
    Both = 0x03,
}

impl OptAccessScope {
    /// Convert from C flags
    pub fn from_flags(flags: c_int) -> Self {
        match flags & 0x03 {
            0x01 => Self::Global,
            0x02 => Self::Local,
            _ => Self::Both,
        }
    }

    /// Convert to C flags
    pub fn to_flags(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Option Value Union
// =============================================================================

/// Unified option value that can hold any option type
#[repr(C)]
pub union OptValueData {
    pub boolean: c_int,
    pub number: i64,
    pub string: *mut c_char,
}

/// Unified option value structure
#[repr(C)]
pub struct OptValue {
    /// Type of the value
    pub vtype: OptValType,
    /// The value data
    pub data: OptValueData,
}

impl Default for OptValue {
    fn default() -> Self {
        Self {
            vtype: OptValType::Nil,
            data: OptValueData { boolean: 0 },
        }
    }
}

impl OptValue {
    /// Create a nil value
    pub fn nil() -> Self {
        Self::default()
    }

    /// Create a boolean value
    pub fn boolean(val: bool) -> Self {
        Self {
            vtype: OptValType::Boolean,
            data: OptValueData {
                boolean: c_int::from(val),
            },
        }
    }

    /// Create a number value
    pub fn number(val: i64) -> Self {
        Self {
            vtype: OptValType::Number,
            data: OptValueData { number: val },
        }
    }

    /// Create a string value (takes ownership of the pointer)
    pub fn string(val: *mut c_char) -> Self {
        Self {
            vtype: OptValType::String,
            data: OptValueData { string: val },
        }
    }

    /// Check if this is a nil value
    pub fn is_nil(&self) -> bool {
        self.vtype == OptValType::Nil
    }
}

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // Option lookup
    fn findoption(name: *const c_char) -> c_int;
    fn findoption_len(name: *const c_char, len: usize) -> c_int;

    // Option type queries
    fn option_has_type(opt_idx: c_int, opt_type: c_int) -> c_int;

    // Option value access
    fn get_varp_scope(opt: *const c_void, scope: c_int) -> *mut c_void;
    fn optval_from_varp(opt_idx: c_int, varp: *const c_void) -> OptValue;

    // Option setting
    fn set_option_value(opt_idx: c_int, value: *const c_char, opt_flags: c_int) -> *const c_char;

    // Options array
    fn nvim_get_options_array() -> *const c_void;
}

/// Invalid option index
const K_OPT_INVALID: c_int = -1;

/// Size of vimoption_T struct (for array indexing)
const VIMOPTION_SIZE: usize = 128;

// =============================================================================
// Unified Get/Set API
// =============================================================================

/// Get an option value by name.
///
/// # Arguments
/// * `name` - Option name (null-terminated)
/// * `scope` - Scope flags (OPT_GLOBAL, OPT_LOCAL, or both)
///
/// # Returns
/// The option value, or a nil value if the option doesn't exist.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option(name: *const c_char, scope: c_int) -> OptValue {
    if name.is_null() {
        return OptValue::nil();
    }

    let opt_idx = findoption(name);
    if opt_idx == K_OPT_INVALID {
        return OptValue::nil();
    }

    rs_get_option_by_idx(opt_idx, scope)
}

/// Get an option value by index.
///
/// # Arguments
/// * `opt_idx` - Option index in options array
/// * `scope` - Scope flags
///
/// # Returns
/// The option value, or a nil value if invalid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_by_idx(opt_idx: c_int, scope: c_int) -> OptValue {
    if opt_idx == K_OPT_INVALID {
        return OptValue::nil();
    }

    let opt = nvim_get_options_array()
        .cast::<u8>()
        .add(opt_idx as usize * VIMOPTION_SIZE);

    let varp = get_varp_scope(opt.cast(), scope);
    if varp.is_null() {
        return OptValue::nil();
    }

    optval_from_varp(opt_idx, varp)
}

/// Get a boolean option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `scope` - Scope flags
///
/// # Returns
/// The boolean value (0 or 1), or -1 if the option doesn't exist or isn't boolean.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_bool(name: *const c_char, scope: c_int) -> c_int {
    let val = rs_get_option(name, scope);
    if val.vtype != OptValType::Boolean {
        return -1;
    }
    val.data.boolean
}

/// Get a number option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `scope` - Scope flags
/// * `default_val` - Default value if option doesn't exist
///
/// # Returns
/// The number value, or default_val if invalid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_number(
    name: *const c_char,
    scope: c_int,
    default_val: i64,
) -> i64 {
    let val = rs_get_option(name, scope);
    if val.vtype != OptValType::Number {
        return default_val;
    }
    val.data.number
}

/// Get a string option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `scope` - Scope flags
///
/// # Returns
/// Pointer to the string value (do NOT free), or NULL if invalid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_string(name: *const c_char, scope: c_int) -> *const c_char {
    let val = rs_get_option(name, scope);
    if val.vtype != OptValType::String {
        return ptr::null();
    }
    val.data.string
}

/// Set an option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `value` - New value (as string representation)
/// * `scope` - Scope flags
///
/// # Returns
/// NULL on success, error message pointer on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option(
    name: *const c_char,
    value: *const c_char,
    scope: c_int,
) -> *const c_char {
    if name.is_null() {
        return c"E474: Invalid argument".as_ptr();
    }

    let opt_idx = findoption(name);
    if opt_idx == K_OPT_INVALID {
        return c"E518: Unknown option".as_ptr();
    }

    set_option_value(opt_idx, value, scope)
}

/// Set a boolean option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `value` - New boolean value (0 or non-zero)
/// * `scope` - Scope flags
///
/// # Returns
/// NULL on success, error message pointer on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_bool(
    name: *const c_char,
    value: c_int,
    scope: c_int,
) -> *const c_char {
    if name.is_null() {
        return c"E474: Invalid argument".as_ptr();
    }

    let opt_idx = findoption(name);
    if opt_idx == K_OPT_INVALID {
        return c"E518: Unknown option".as_ptr();
    }

    // Verify it's a boolean option
    if option_has_type(opt_idx, 0) == 0 {
        // 0 = kOptValTypeBoolean
        return c"E474: Not a boolean option".as_ptr();
    }

    let val_str = if value != 0 { c"1" } else { c"0" };
    set_option_value(opt_idx, val_str.as_ptr(), scope)
}

/// Set a number option value by name.
///
/// # Arguments
/// * `name` - Option name
/// * `value` - New number value
/// * `scope` - Scope flags
///
/// # Returns
/// NULL on success, error message pointer on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_number(
    name: *const c_char,
    value: i64,
    scope: c_int,
) -> *const c_char {
    if name.is_null() {
        return c"E474: Invalid argument".as_ptr();
    }

    let opt_idx = findoption(name);
    if opt_idx == K_OPT_INVALID {
        return c"E518: Unknown option".as_ptr();
    }

    // Verify it's a number option
    if option_has_type(opt_idx, 1) == 0 {
        // 1 = kOptValTypeNumber
        return c"E474: Not a number option".as_ptr();
    }

    // Convert number to string
    let mut buf = [0i8; 32];
    let len = i64_to_str(value, &mut buf);
    buf[len] = 0;

    set_option_value(opt_idx, buf.as_ptr(), scope)
}

/// Convert i64 to string in buffer, return length.
fn i64_to_str(mut value: i64, buf: &mut [i8; 32]) -> usize {
    let negative = value < 0;
    if negative {
        value = -value;
    }

    // Build string backwards
    let mut pos = 31usize;
    loop {
        pos -= 1;
        buf[pos] = b'0' as i8 + (value % 10) as i8;
        value /= 10;
        if value == 0 {
            break;
        }
    }

    if negative {
        pos -= 1;
        buf[pos] = b'-' as i8;
    }

    // Move to start of buffer
    let len = 31 - pos;
    for i in 0..len {
        buf[i] = buf[pos + i];
    }
    len
}

// =============================================================================
// Option Query Functions
// =============================================================================

/// Check if an option exists.
///
/// # Arguments
/// * `name` - Option name
///
/// # Returns
/// 1 if exists, 0 if not.
#[no_mangle]
pub unsafe extern "C" fn rs_option_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    c_int::from(findoption(name) != K_OPT_INVALID)
}

/// Get the type of an option.
///
/// # Arguments
/// * `name` - Option name
///
/// # Returns
/// The option type, or -1 if option doesn't exist.
#[no_mangle]
pub unsafe extern "C" fn rs_option_type(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let opt_idx = findoption(name);
    if opt_idx == K_OPT_INVALID {
        return -1;
    }

    // Check types in order
    if option_has_type(opt_idx, 0) != 0 {
        return 0; // Boolean
    }
    if option_has_type(opt_idx, 1) != 0 {
        return 1; // Number
    }
    if option_has_type(opt_idx, 2) != 0 {
        return 2; // String
    }

    -1
}

/// Find option index by name (accessor API version).
///
/// # Arguments
/// * `name` - Option name
///
/// # Returns
/// Option index, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_accessor_find_option(name: *const c_char) -> c_int {
    if name.is_null() {
        return K_OPT_INVALID;
    }
    findoption(name)
}

/// Find option index by name with length (accessor API version).
///
/// # Arguments
/// * `name` - Option name (may not be null-terminated)
/// * `len` - Length of name
///
/// # Returns
/// Option index, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn rs_accessor_find_option_len(name: *const c_char, len: usize) -> c_int {
    if name.is_null() || len == 0 {
        return K_OPT_INVALID;
    }
    findoption_len(name, len)
}

// =============================================================================
// Option Change Notification
// =============================================================================

/// Option change event data
#[repr(C)]
pub struct OptChangeEvent {
    /// Option index
    pub opt_idx: c_int,
    /// Option name (borrowed, do not free)
    pub name: *const c_char,
    /// Old value
    pub old_value: OptValue,
    /// New value
    pub new_value: OptValue,
    /// Scope flags
    pub scope: c_int,
}

/// Callback type for option change notifications
pub type OptChangeCallback = unsafe extern "C" fn(event: *const OptChangeEvent);

/// Maximum number of option change callbacks
const MAX_OPT_CHANGE_CALLBACKS: usize = 8;

/// Storage for registered callbacks
/// Using raw pointers instead of static mut to avoid UB warnings
mod callback_storage {
    use super::{OptChangeCallback, OptChangeEvent, MAX_OPT_CHANGE_CALLBACKS};
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);

    // Use a fixed-size array of function pointers
    // This is safe because function pointers are Copy and Send+Sync
    static CALLBACKS: [AtomicPtr<()>; MAX_OPT_CHANGE_CALLBACKS] = [
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
        AtomicPtr::new(std::ptr::null_mut()),
    ];

    pub fn register(callback: OptChangeCallback) -> bool {
        let count = CALLBACK_COUNT.load(Ordering::SeqCst);
        if count >= MAX_OPT_CHANGE_CALLBACKS {
            return false;
        }
        CALLBACKS[count].store(callback as *mut (), Ordering::SeqCst);
        CALLBACK_COUNT.store(count + 1, Ordering::SeqCst);
        true
    }

    pub fn notify(event: *const OptChangeEvent) {
        let count = CALLBACK_COUNT.load(Ordering::SeqCst);
        for callback in CALLBACKS.iter().take(count) {
            let ptr = callback.load(Ordering::SeqCst);
            if !ptr.is_null() {
                let cb: OptChangeCallback = unsafe { std::mem::transmute(ptr) };
                unsafe { cb(event) };
            }
        }
    }
}

/// Register a callback for option change notifications.
///
/// # Arguments
/// * `callback` - The callback function
///
/// # Returns
/// 1 on success, 0 if no more slots available.
#[no_mangle]
pub extern "C" fn rs_register_opt_change_callback(callback: OptChangeCallback) -> c_int {
    c_int::from(callback_storage::register(callback))
}

/// Notify all registered callbacks of an option change.
///
/// # Arguments
/// * `event` - The change event
#[no_mangle]
pub unsafe extern "C" fn rs_notify_opt_change(event: *const OptChangeEvent) {
    if event.is_null() {
        return;
    }
    callback_storage::notify(event);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_val_type_conversion() {
        assert_eq!(OptValType::from_c_int(0), OptValType::Boolean);
        assert_eq!(OptValType::from_c_int(1), OptValType::Number);
        assert_eq!(OptValType::from_c_int(2), OptValType::String);
        assert_eq!(OptValType::from_c_int(-1), OptValType::Nil);
        assert_eq!(OptValType::from_c_int(99), OptValType::Nil);
    }

    #[test]
    fn test_opt_access_scope_conversion() {
        assert_eq!(OptAccessScope::from_flags(0x01), OptAccessScope::Global);
        assert_eq!(OptAccessScope::from_flags(0x02), OptAccessScope::Local);
        assert_eq!(OptAccessScope::from_flags(0x03), OptAccessScope::Both);
        assert_eq!(OptAccessScope::from_flags(0x00), OptAccessScope::Both);
    }

    #[test]
    fn test_opt_value_constructors() {
        let nil = OptValue::nil();
        assert!(nil.is_nil());
        assert_eq!(nil.vtype, OptValType::Nil);

        let bool_val = OptValue::boolean(true);
        assert_eq!(bool_val.vtype, OptValType::Boolean);
        assert_eq!(unsafe { bool_val.data.boolean }, 1);

        let num_val = OptValue::number(42);
        assert_eq!(num_val.vtype, OptValType::Number);
        assert_eq!(unsafe { num_val.data.number }, 42);

        let str_val = OptValue::string(std::ptr::null_mut());
        assert_eq!(str_val.vtype, OptValType::String);
    }
}
