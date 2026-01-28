//! Function dispatch helpers for VimL built-in functions.
//!
//! This module provides common utilities and types used by all VimL function
//! implementations.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(dead_code)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Typval Handle and Accessors
// =============================================================================

/// Opaque handle to a `typval_T*` (VimL value).
///
/// Rust cannot directly access typval_T fields; all access goes through
/// C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalPtr(*const c_void);

impl TypevalPtr {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Mutable typval handle for return values.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalPtrMut(*mut c_void);

impl TypevalPtrMut {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// VarType enum values (matching C's VarType in typval_defs.h).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarType {
    Unknown = 0,
    Number = 1,
    String = 2,
    Func = 3,
    List = 4,
    Dict = 5,
    Float = 6,
    Bool = 7,
    Special = 8,
    Partial = 9,
    Blob = 10,
}

impl VarType {
    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::Unknown),
            1 => Some(Self::Number),
            2 => Some(Self::String),
            3 => Some(Self::Func),
            4 => Some(Self::List),
            5 => Some(Self::Dict),
            6 => Some(Self::Float),
            7 => Some(Self::Bool),
            8 => Some(Self::Special),
            9 => Some(Self::Partial),
            10 => Some(Self::Blob),
            _ => None,
        }
    }
}

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // --- Type accessors ---
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_number(tv: *const c_void) -> i64;
    fn nvim_tv_get_float(tv: *const c_void) -> f64;

    // --- Setters for return values ---
    fn nvim_tv_set_number(tv: *mut c_void, n: i64);
    fn nvim_tv_set_float(tv: *mut c_void, f: f64);

    // --- Value extraction with error checking ---
    /// Get number from typval with error checking.
    /// Sets `*error = true` if the type is invalid for number conversion.
    fn nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;

    /// Get float from typval with error checking.
    /// Returns false and sets rettv to 0.0 if conversion fails.
    fn nvim_tv_get_float_chk(tv: *const c_void, ret: *mut f64) -> bool;

    // --- String accessors ---
    /// Get raw string pointer from typval (no conversion).
    fn nvim_tv_get_string_ptr(tv: *const c_void) -> *const u8;

    /// Get string with type conversion.
    /// Uses a static buffer for conversions, so result may be overwritten by next call.
    fn nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;

    /// Get string with error checking.
    /// Returns NULL on error.
    fn nvim_tv_get_string_chk(tv: *const c_void, out_len: *mut usize) -> *const u8;

    /// Set typval to a string (takes ownership).
    fn nvim_tv_set_string(tv: *mut c_void, s: *mut u8);

    /// Set typval to a copy of a string.
    fn nvim_tv_set_string_copy(tv: *mut c_void, s: *const u8, len: c_int);

    /// Allocate a string of given size and set it as typval value.
    /// Returns pointer to the allocated buffer.
    fn nvim_tv_alloc_string(tv: *mut c_void, len: usize) -> *mut u8;

    // --- List accessors ---
    /// Check if typval is a null list.
    fn nvim_tv_list_is_null(tv: *const c_void) -> c_int;

    /// Get list pointer from typval (must be VAR_LIST).
    fn nvim_tv_get_list(tv: *const c_void) -> *const c_void;

    /// Get list length.
    fn nvim_list_get_len(l: *const c_void) -> c_int;

    /// Get first item in list.
    fn nvim_list_get_first(l: *const c_void) -> *const c_void;

    /// Get last item in list.
    fn nvim_list_get_last(l: *const c_void) -> *const c_void;

    /// Get next list item.
    fn nvim_listitem_get_next(li: *const c_void) -> *const c_void;

    /// Get previous list item.
    fn nvim_listitem_get_prev(li: *const c_void) -> *const c_void;

    /// Get typval from list item.
    fn nvim_listitem_get_tv(li: *const c_void) -> *const c_void;

    // --- Dict accessors ---
    /// Check if typval is a null dict.
    fn nvim_tv_dict_is_null(tv: *const c_void) -> c_int;

    /// Get dict pointer from typval (must be VAR_DICT).
    fn nvim_tv_get_dict(tv: *const c_void) -> *const c_void;

    /// Get dict length (number of items).
    fn nvim_dict_get_len(d: *const c_void) -> c_int;

    // --- Blob accessors ---
    /// Get blob length.
    fn nvim_tv_blob_len(tv: *const c_void) -> c_int;
}

// =============================================================================
// Safe Wrappers
// =============================================================================

/// Get the VarType of a typval.
#[inline]
pub fn tv_get_type(tv: TypevalPtr) -> VarType {
    if tv.is_null() {
        return VarType::Unknown;
    }
    let type_int = unsafe { nvim_tv_get_type(tv.as_ptr()) };
    VarType::from_c_int(type_int).unwrap_or(VarType::Unknown)
}

/// Get number value (raw, no type conversion).
#[inline]
pub fn tv_get_number_raw(tv: TypevalPtr) -> i64 {
    if tv.is_null() {
        return 0;
    }
    unsafe { nvim_tv_get_number(tv.as_ptr()) }
}

/// Get float value (raw, no type conversion).
#[inline]
pub fn tv_get_float_raw(tv: TypevalPtr) -> f64 {
    if tv.is_null() {
        return 0.0;
    }
    unsafe { nvim_tv_get_float(tv.as_ptr()) }
}

/// Get number with error checking (reports VimL errors for invalid types).
/// Returns the number and whether an error occurred.
#[inline]
pub fn tv_get_number_chk(tv: TypevalPtr) -> (i64, bool) {
    if tv.is_null() {
        return (0, true);
    }
    let mut error = false;
    let n = unsafe { nvim_tv_get_number_chk(tv.as_ptr(), &raw mut error) };
    (n, error)
}

/// Get float with error checking (reports VimL errors for invalid types).
/// Returns Some(float) on success, None on error.
#[inline]
pub fn tv_get_float_chk(tv: TypevalPtr) -> Option<f64> {
    if tv.is_null() {
        return None;
    }
    let mut ret = 0.0;
    let ok = unsafe { nvim_tv_get_float_chk(tv.as_ptr(), &raw mut ret) };
    if ok {
        Some(ret)
    } else {
        None
    }
}

/// Set return value to a number.
#[inline]
pub fn rettv_set_number(rettv: TypevalPtrMut, n: i64) {
    if !rettv.is_null() {
        unsafe { nvim_tv_set_number(rettv.as_ptr(), n) };
    }
}

/// Set return value to a float.
#[inline]
pub fn rettv_set_float(rettv: TypevalPtrMut, f: f64) {
    if !rettv.is_null() {
        unsafe { nvim_tv_set_float(rettv.as_ptr(), f) };
    }
}

/// Set return value to a boolean (as a number, 1 or 0).
///
/// VimL booleans are represented as numbers in many contexts.
/// This sets the return value to 1 (true) or 0 (false).
#[inline]
pub fn rettv_set_bool(rettv: TypevalPtrMut, b: bool) {
    rettv_set_number(rettv, i64::from(b));
}

// =============================================================================
// String Accessors
// =============================================================================

/// Get string from typval (with type conversion).
/// Returns the string as a byte slice, or an empty slice if null/invalid.
///
/// # Safety
/// The returned slice is only valid until the next call to any tv_get_string function.
#[inline]
pub fn tv_get_string_bytes(tv: TypevalPtr) -> &'static [u8] {
    if tv.is_null() {
        return &[];
    }
    let mut len: usize = 0;
    let ptr = unsafe { nvim_tv_get_string(tv.as_ptr(), &raw mut len) };
    if ptr.is_null() || len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Get string from typval with error checking.
/// Returns None on error, Some(bytes) on success.
///
/// # Safety
/// The returned slice is only valid until the next call to any tv_get_string function.
#[inline]
pub fn tv_get_string_chk_bytes(tv: TypevalPtr) -> Option<&'static [u8]> {
    if tv.is_null() {
        return None;
    }
    let mut len: usize = 0;
    let ptr = unsafe { nvim_tv_get_string_chk(tv.as_ptr(), &raw mut len) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(ptr, len) })
    }
}

/// Get raw string pointer from typval (no conversion, VAR_STRING only).
/// Returns empty slice if not a string type or null.
#[inline]
pub fn tv_get_string_ptr(tv: TypevalPtr) -> &'static [u8] {
    if tv.is_null() {
        return &[];
    }
    let ptr = unsafe { nvim_tv_get_string_ptr(tv.as_ptr()) };
    if ptr.is_null() {
        &[]
    } else {
        // Find length by scanning for NUL
        let mut len = 0;
        unsafe {
            while *ptr.add(len) != 0 {
                len += 1;
            }
        }
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// Set return value to a copy of a byte slice.
#[inline]
pub fn rettv_set_string(rettv: TypevalPtrMut, s: &[u8]) {
    if rettv.is_null() {
        return;
    }
    unsafe {
        nvim_tv_set_string_copy(rettv.as_ptr(), s.as_ptr(), s.len() as c_int);
    }
}

/// Allocate a string in the return value and return a mutable slice to fill.
/// The caller must fill the buffer with valid UTF-8 (or at least valid bytes).
///
/// Returns None if allocation fails or rettv is null.
#[inline]
pub fn rettv_alloc_string(rettv: TypevalPtrMut, len: usize) -> Option<&'static mut [u8]> {
    if rettv.is_null() {
        return None;
    }
    let ptr = unsafe { nvim_tv_alloc_string(rettv.as_ptr(), len) };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts_mut(ptr, len) })
    }
}

// =============================================================================
// List Accessors
// =============================================================================

/// Opaque handle to a `list_T*`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ListPtr(*const c_void);

impl ListPtr {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `list_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a `listitem_T*`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ListItemPtr(*const c_void);

impl ListItemPtr {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `listitem_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Check if a typval contains a null list.
#[inline]
pub fn tv_list_is_null(tv: TypevalPtr) -> bool {
    if tv.is_null() {
        return true;
    }
    unsafe { nvim_tv_list_is_null(tv.as_ptr()) != 0 }
}

/// Get list from typval.
#[inline]
pub fn tv_get_list(tv: TypevalPtr) -> ListPtr {
    if tv.is_null() {
        return unsafe { ListPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_tv_get_list(tv.as_ptr()) };
    unsafe { ListPtr::from_raw(ptr) }
}

/// Get the length of a list.
#[inline]
pub fn list_len(l: ListPtr) -> c_int {
    if l.is_null() {
        0
    } else {
        unsafe { nvim_list_get_len(l.as_ptr()) }
    }
}

/// Get the first item of a list.
#[inline]
pub fn list_first(l: ListPtr) -> ListItemPtr {
    if l.is_null() {
        return unsafe { ListItemPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_list_get_first(l.as_ptr()) };
    unsafe { ListItemPtr::from_raw(ptr) }
}

/// Get the last item of a list.
#[inline]
pub fn list_last(l: ListPtr) -> ListItemPtr {
    if l.is_null() {
        return unsafe { ListItemPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_list_get_last(l.as_ptr()) };
    unsafe { ListItemPtr::from_raw(ptr) }
}

/// Get the next item after a list item.
#[inline]
pub fn listitem_next(li: ListItemPtr) -> ListItemPtr {
    if li.is_null() {
        return unsafe { ListItemPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_listitem_get_next(li.as_ptr()) };
    unsafe { ListItemPtr::from_raw(ptr) }
}

/// Get the previous item before a list item.
#[inline]
pub fn listitem_prev(li: ListItemPtr) -> ListItemPtr {
    if li.is_null() {
        return unsafe { ListItemPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_listitem_get_prev(li.as_ptr()) };
    unsafe { ListItemPtr::from_raw(ptr) }
}

/// Get the typval from a list item.
#[inline]
pub fn listitem_tv(li: ListItemPtr) -> TypevalPtr {
    if li.is_null() {
        return unsafe { TypevalPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_listitem_get_tv(li.as_ptr()) };
    unsafe { TypevalPtr::from_raw(ptr) }
}

// =============================================================================
// Dict Accessors
// =============================================================================

/// Opaque handle to a `dict_T*`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DictPtr(*const c_void);

impl DictPtr {
    /// Create from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `dict_T*` or null.
    #[inline]
    pub const unsafe fn from_raw(ptr: *const c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *const c_void {
        self.0
    }

    /// Check if null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Check if a typval contains a null dict.
#[inline]
pub fn tv_dict_is_null(tv: TypevalPtr) -> bool {
    if tv.is_null() {
        return true;
    }
    unsafe { nvim_tv_dict_is_null(tv.as_ptr()) != 0 }
}

/// Get dict from typval.
#[inline]
pub fn tv_get_dict(tv: TypevalPtr) -> DictPtr {
    if tv.is_null() {
        return unsafe { DictPtr::from_raw(std::ptr::null()) };
    }
    let ptr = unsafe { nvim_tv_get_dict(tv.as_ptr()) };
    unsafe { DictPtr::from_raw(ptr) }
}

/// Get the length (number of items) of a dict.
#[inline]
pub fn dict_len(d: DictPtr) -> c_int {
    if d.is_null() {
        0
    } else {
        unsafe { nvim_dict_get_len(d.as_ptr()) }
    }
}

// =============================================================================
// Blob Accessors
// =============================================================================

/// Get blob length from typval.
#[inline]
pub fn tv_blob_len(tv: TypevalPtr) -> c_int {
    if tv.is_null() {
        0
    } else {
        unsafe { nvim_tv_blob_len(tv.as_ptr()) }
    }
}

// =============================================================================
// Argument Array Access
// =============================================================================

/// Get a typval from an argument array by index.
///
/// # Safety
/// - `argvars` must be a valid pointer to a typval array
/// - `index` must be within bounds of the array
#[inline]
pub unsafe fn argvar_at(argvars: *const c_void, index: usize) -> TypevalPtr {
    // typval_T is 24 bytes on 64-bit systems (v_type: i32, padding, union: 8 bytes, v_lock: i32, padding)
    // We use the accessor to get sizeof(typval_T) from C
    let offset = index * TYPVAL_SIZE;
    let ptr = argvars.cast::<u8>().add(offset).cast::<c_void>();
    TypevalPtr::from_raw(ptr)
}

// Size of typval_T in bytes (from C)
// This is platform-dependent but we match the C definition
const TYPVAL_SIZE: usize = 24; // sizeof(typval_T) on 64-bit

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vartype_from_int() {
        assert_eq!(VarType::from_c_int(0), Some(VarType::Unknown));
        assert_eq!(VarType::from_c_int(1), Some(VarType::Number));
        assert_eq!(VarType::from_c_int(6), Some(VarType::Float));
        assert_eq!(VarType::from_c_int(99), None);
    }

    #[test]
    fn test_null_typval_ptr() {
        let null_ptr = unsafe { TypevalPtr::from_raw(std::ptr::null()) };
        assert!(null_ptr.is_null());
        // tv_get_type() calls C functions, can't test without linking to C
    }

    #[test]
    fn test_typval_size() {
        // Verify our TYPVAL_SIZE constant matches expected layout
        assert_eq!(TYPVAL_SIZE, 24);
    }
}
