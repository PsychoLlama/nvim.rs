//! Vimscript typval_T operations for Neovim
//!
//! This crate provides Rust implementations of typval-related functions
//! from `src/nvim/eval/typval.c`. It uses an opaque handle pattern where
//! `typval_T*` pointers are treated as opaque handles, with field access
//! done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int};

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
    /// Convert from C integer to VarType.
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

/// Opaque handle to a Vimscript typval (`typval_T*`).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypevalHandle(*const std::ffi::c_void);

impl TypevalHandle {
    /// Create a new typval handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `typval_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *const std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for typval_T fields.
// These will be defined in eval/typval.c
extern "C" {
    /// Get the v_type field from a typval.
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    /// Get the v_number field from a typval (only valid if v_type == VAR_NUMBER).
    fn nvim_tv_get_number(tv: TypevalHandle) -> i64;

    /// Get the v_bool field from a typval (only valid if v_type == VAR_BOOL).
    fn nvim_tv_get_bool(tv: TypevalHandle) -> c_int;

    /// Get the v_special field from a typval (only valid if v_type == VAR_SPECIAL).
    fn nvim_tv_get_special(tv: TypevalHandle) -> c_int;

    /// Get the v_float field from a typval (only valid if v_type == VAR_FLOAT).
    fn nvim_tv_get_float(tv: TypevalHandle) -> f64;

    /// Get the v_string field from a typval (only valid if v_type == VAR_STRING or VAR_FUNC).
    fn nvim_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char;

    /// Check if v_list is NULL (only valid if v_type == VAR_LIST).
    fn nvim_tv_list_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_dict is NULL (only valid if v_type == VAR_DICT).
    fn nvim_tv_dict_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_blob is NULL (only valid if v_type == VAR_BLOB).
    fn nvim_tv_blob_is_null(tv: TypevalHandle) -> c_int;

    /// Check if v_partial is NULL (only valid if v_type == VAR_PARTIAL).
    fn nvim_tv_partial_is_null(tv: TypevalHandle) -> c_int;
}

// =============================================================================
// Type checking predicates
// =============================================================================

/// Get the VarType of a typval.
#[inline]
fn tv_type_impl(tv: TypevalHandle) -> VarType {
    if tv.is_null() {
        return VarType::Unknown;
    }
    // SAFETY: We check for null above.
    let type_int = unsafe { nvim_tv_get_type(tv) };
    VarType::from_c_int(type_int).unwrap_or(VarType::Unknown)
}

/// FFI wrapper: get the type of a typval.
#[no_mangle]
pub extern "C" fn rs_tv_type(tv: TypevalHandle) -> c_int {
    tv_type_impl(tv) as c_int
}

/// Check if typval is a number.
#[inline]
fn tv_is_number_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Number
}

/// FFI wrapper for tv_is_number.
#[no_mangle]
pub extern "C" fn rs_tv_is_number(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_number_impl(tv))
}

/// Check if typval is a string.
#[inline]
fn tv_is_string_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::String
}

/// FFI wrapper for tv_is_string.
#[no_mangle]
pub extern "C" fn rs_tv_is_string(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_string_impl(tv))
}

/// Check if typval is a float.
#[inline]
fn tv_is_float_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Float
}

/// FFI wrapper for tv_is_float.
#[no_mangle]
pub extern "C" fn rs_tv_is_float(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_float_impl(tv))
}

/// Check if typval is a bool.
#[inline]
fn tv_is_bool_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Bool
}

/// FFI wrapper for tv_is_bool.
#[no_mangle]
pub extern "C" fn rs_tv_is_bool(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_bool_impl(tv))
}

/// Check if typval is a list.
#[inline]
fn tv_is_list_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::List
}

/// FFI wrapper for tv_is_list.
#[no_mangle]
pub extern "C" fn rs_tv_is_list(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_list_impl(tv))
}

/// Check if typval is a dict.
#[inline]
fn tv_is_dict_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Dict
}

/// FFI wrapper for tv_is_dict.
#[no_mangle]
pub extern "C" fn rs_tv_is_dict(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_dict_impl(tv))
}

/// Check if typval is a blob.
#[inline]
fn tv_is_blob_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Blob
}

/// FFI wrapper for tv_is_blob.
#[no_mangle]
pub extern "C" fn rs_tv_is_blob(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_blob_impl(tv))
}

/// Check if typval is a function reference (VAR_FUNC or VAR_PARTIAL).
#[inline]
fn tv_is_func_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Func || t == VarType::Partial
}

/// FFI wrapper for tv_is_func.
#[no_mangle]
pub extern "C" fn rs_tv_is_func(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_func_impl(tv))
}

/// Check if typval is special (null).
#[inline]
fn tv_is_special_impl(tv: TypevalHandle) -> bool {
    tv_type_impl(tv) == VarType::Special
}

/// FFI wrapper for tv_is_special.
#[no_mangle]
pub extern "C" fn rs_tv_is_special(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_special_impl(tv))
}

/// Check if typval has a numeric type (number or float).
#[inline]
fn tv_is_numeric_impl(tv: TypevalHandle) -> bool {
    let t = tv_type_impl(tv);
    t == VarType::Number || t == VarType::Float
}

/// FFI wrapper for tv_is_numeric.
#[no_mangle]
pub extern "C" fn rs_tv_is_numeric(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_numeric_impl(tv))
}

// =============================================================================
// Value extraction (simple - no error reporting)
// =============================================================================

/// Get the number value from a typval (returns 0 if not a number).
#[inline]
fn tv_get_number_simple_impl(tv: TypevalHandle) -> i64 {
    if tv.is_null() {
        return 0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Number => unsafe { nvim_tv_get_number(tv) },
        VarType::Bool => {
            // v_bool: 0 = false, 1 = true
            let b = unsafe { nvim_tv_get_bool(tv) };
            i64::from(b)
        }
        _ => 0,
    }
}

/// FFI wrapper: get number value from typval (0 if not a number).
#[no_mangle]
pub extern "C" fn rs_tv_get_number_simple(tv: TypevalHandle) -> i64 {
    tv_get_number_simple_impl(tv)
}

/// Get the float value from a typval (returns 0.0 if not a float or number).
#[inline]
fn tv_get_float_simple_impl(tv: TypevalHandle) -> f64 {
    if tv.is_null() {
        return 0.0;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Float => unsafe { nvim_tv_get_float(tv) },
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n as f64
        }
        _ => 0.0,
    }
}

/// FFI wrapper: get float value from typval (0.0 if not a float).
#[no_mangle]
pub extern "C" fn rs_tv_get_float_simple(tv: TypevalHandle) -> f64 {
    tv_get_float_simple_impl(tv)
}

/// Get the bool value from a typval (returns false if not a bool/number).
#[inline]
fn tv_get_bool_simple_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return false;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Bool => {
            let b = unsafe { nvim_tv_get_bool(tv) };
            b != 0
        }
        VarType::Number => {
            let n = unsafe { nvim_tv_get_number(tv) };
            n != 0
        }
        _ => false,
    }
}

/// FFI wrapper: get bool value from typval (false if not a bool).
#[no_mangle]
pub extern "C" fn rs_tv_get_bool_simple(tv: TypevalHandle) -> c_int {
    c_int::from(tv_get_bool_simple_impl(tv))
}

/// Get the string pointer from a typval (returns NULL if not a string).
#[inline]
fn tv_get_string_ptr_impl(tv: TypevalHandle) -> *const c_char {
    if tv.is_null() {
        return std::ptr::null();
    }
    let t = tv_type_impl(tv);
    if t == VarType::String {
        unsafe { nvim_tv_get_string_ptr(tv) }
    } else {
        std::ptr::null()
    }
}

/// FFI wrapper: get string pointer from typval (NULL if not a string).
#[no_mangle]
pub extern "C" fn rs_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char {
    tv_get_string_ptr_impl(tv)
}

// =============================================================================
// Emptiness/truthiness checks
// =============================================================================

/// Check if a typval is "empty" (falsy in Vimscript terms).
///
/// - Numbers: 0 is empty
/// - Strings: empty string or NULL is empty
/// - Lists: NULL or empty list is empty
/// - Dicts: NULL or empty dict is empty
/// - Blobs: NULL or empty blob is empty
/// - Bools: false is empty
/// - Special: always empty (v:null)
/// - Floats: 0.0 is empty
/// - Funcs/Partials: never empty (always truthy if set)
#[inline]
fn tv_is_empty_impl(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return true;
    }
    let t = tv_type_impl(tv);
    match t {
        VarType::Unknown => true,
        VarType::Number => unsafe { nvim_tv_get_number(tv) == 0 },
        VarType::String => {
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            if s.is_null() {
                true
            } else {
                // Check if first byte is NUL (empty string)
                unsafe { *s == 0 }
            }
        }
        VarType::Float => unsafe { nvim_tv_get_float(tv) == 0.0 },
        VarType::Bool => unsafe { nvim_tv_get_bool(tv) == 0 },
        VarType::Special => true, // v:null is always empty
        VarType::List => unsafe { nvim_tv_list_is_null(tv) != 0 },
        VarType::Dict => unsafe { nvim_tv_dict_is_null(tv) != 0 },
        VarType::Blob => unsafe { nvim_tv_blob_is_null(tv) != 0 },
        VarType::Func => {
            // Function reference is non-empty if string is set
            let s = unsafe { nvim_tv_get_string_ptr(tv) };
            s.is_null() || unsafe { *s == 0 }
        }
        VarType::Partial => unsafe { nvim_tv_partial_is_null(tv) != 0 },
    }
}

/// FFI wrapper: check if typval is empty/falsy.
#[no_mangle]
pub extern "C" fn rs_tv_is_empty(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_empty_impl(tv))
}

/// Check if a typval is "truthy" (non-empty in Vimscript terms).
#[inline]
fn tv_is_truthy_impl(tv: TypevalHandle) -> bool {
    !tv_is_empty_impl(tv)
}

/// FFI wrapper: check if typval is truthy.
#[no_mangle]
pub extern "C" fn rs_tv_is_truthy(tv: TypevalHandle) -> c_int {
    c_int::from(tv_is_truthy_impl(tv))
}

// =============================================================================
// Opaque handle types for list_T, dict_T, blob_T
// =============================================================================

/// Opaque handle to a Vimscript list (`list_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListHandle(*const std::ffi::c_void);

impl ListHandle {
    /// Create a new list handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Vimscript dict (`dict_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictHandle(*const std::ffi::c_void);

impl DictHandle {
    /// Create a new dict handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a Vimscript blob (`blob_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobHandle(*const std::ffi::c_void);

impl BlobHandle {
    /// Create a new blob handle from a raw pointer.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *const std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a list item (`listitem_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemHandle(*const std::ffi::c_void);

impl ListItemHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions for list_T, dict_T, blob_T
extern "C" {
    // Typval -> container getters
    fn nvim_tv_get_list(tv: TypevalHandle) -> ListHandle;
    fn nvim_tv_get_dict(tv: TypevalHandle) -> DictHandle;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> BlobHandle;

    // List accessors
    fn nvim_list_get_len(l: ListHandle) -> c_int;
    fn nvim_list_get_lock(l: ListHandle) -> c_int;
    fn nvim_list_has_watchers(l: ListHandle) -> c_int;
    fn nvim_list_get_first(l: ListHandle) -> ListItemHandle;
    fn nvim_list_get_last(l: ListHandle) -> ListItemHandle;

    // List cache accessors (for tv_list_find optimization)
    fn nvim_list_get_idx(l: ListHandle) -> c_int;
    fn nvim_list_get_idx_item(l: ListHandle) -> ListItemHandle;
    fn nvim_list_set_idx(l: ListHandle, idx: c_int);
    fn nvim_list_set_idx_item(l: ListHandle, item: ListItemHandle);
    fn nvim_list_get_copyid(l: ListHandle) -> c_int;
    fn nvim_list_get_copylist(l: ListHandle) -> ListHandle;

    // Listitem accessors
    fn nvim_listitem_get_next(li: ListItemHandle) -> ListItemHandle;
    fn nvim_listitem_get_prev(li: ListItemHandle) -> ListItemHandle;
    fn nvim_listitem_get_tv(li: ListItemHandle) -> TypevalHandle;

    // List setters (for mutation operations)
    fn nvim_list_set_first(l: ListHandle, item: ListItemHandle);
    fn nvim_list_set_last(l: ListHandle, item: ListItemHandle);

    // Listitem setters (for mutation operations)
    fn nvim_listitem_set_next(li: ListItemHandle, next: ListItemHandle);
    fn nvim_listitem_set_prev(li: ListItemHandle, prev: ListItemHandle);

    // Dict accessors
    fn nvim_dict_get_ht_used(d: DictHandle) -> usize;
    fn nvim_dict_get_lock(d: DictHandle) -> c_int;
    fn nvim_dict_has_watchers(d: DictHandle) -> c_int;

    // Blob accessors
    fn nvim_blob_get_len(b: BlobHandle) -> c_int;
    fn nvim_blob_get_lock(b: BlobHandle) -> c_int;
    fn nvim_blob_get_byte(b: BlobHandle, idx: c_int) -> u8;
}

// =============================================================================
// List operations
// =============================================================================

/// Get the number of items in a list.
/// Returns 0 if the list is NULL.
#[inline]
fn tv_list_len_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 0;
    }
    unsafe { nvim_list_get_len(l) }
}

/// FFI wrapper: get list length.
#[no_mangle]
pub extern "C" fn rs_tv_list_len(l: ListHandle) -> c_int {
    tv_list_len_impl(l)
}

/// Get list lock status.
/// Returns VAR_FIXED (2) for NULL lists.
#[inline]
fn tv_list_locked_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_list_get_lock(l) }
}

/// FFI wrapper: get list lock status.
#[no_mangle]
pub extern "C" fn rs_tv_list_locked(l: ListHandle) -> c_int {
    tv_list_locked_impl(l)
}

/// Check whether list has watchers.
#[inline]
fn tv_list_has_watchers_impl(l: ListHandle) -> bool {
    if l.is_null() {
        return false;
    }
    unsafe { nvim_list_has_watchers(l) != 0 }
}

/// FFI wrapper: check if list has watchers.
#[no_mangle]
pub extern "C" fn rs_tv_list_has_watchers(l: ListHandle) -> c_int {
    c_int::from(tv_list_has_watchers_impl(l))
}

/// Get first list item.
/// Returns NULL for NULL or empty lists.
#[inline]
fn tv_list_first_impl(l: ListHandle) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_first(l) }
}

/// FFI wrapper: get first list item.
#[no_mangle]
pub extern "C" fn rs_tv_list_first(l: ListHandle) -> ListItemHandle {
    tv_list_first_impl(l)
}

/// Get last list item.
/// Returns NULL for NULL or empty lists.
#[inline]
fn tv_list_last_impl(l: ListHandle) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_last(l) }
}

/// FFI wrapper: get last list item.
#[no_mangle]
pub extern "C" fn rs_tv_list_last(l: ListHandle) -> ListItemHandle {
    tv_list_last_impl(l)
}

/// Normalize list index: return either -1 or non-negative index.
#[inline]
fn tv_list_uidx_impl(l: ListHandle, mut n: c_int) -> c_int {
    let len = tv_list_len_impl(l);
    // Negative index is relative to the end.
    if n < 0 {
        n += len;
    }
    // Check for index out of range.
    if n < 0 || n >= len {
        return -1;
    }
    n
}

/// FFI wrapper: normalize list index.
#[no_mangle]
pub extern "C" fn rs_tv_list_uidx(l: ListHandle, n: c_int) -> c_int {
    tv_list_uidx_impl(l, n)
}

/// Get copy ID of a list (used for cycle detection during copy).
#[inline]
fn tv_list_copyid_impl(l: ListHandle) -> c_int {
    if l.is_null() {
        return 0;
    }
    unsafe { nvim_list_get_copyid(l) }
}

/// FFI wrapper: get list copy ID.
#[no_mangle]
pub extern "C" fn rs_tv_list_copyid(l: ListHandle) -> c_int {
    tv_list_copyid_impl(l)
}

/// Get the latest copy of a list (set during tv_list_copy).
#[inline]
fn tv_list_latest_copy_impl(l: ListHandle) -> ListHandle {
    if l.is_null() {
        return ListHandle(std::ptr::null());
    }
    unsafe { nvim_list_get_copylist(l) }
}

/// FFI wrapper: get list's latest copy.
#[no_mangle]
pub extern "C" fn rs_tv_list_latest_copy(l: ListHandle) -> ListHandle {
    tv_list_latest_copy_impl(l)
}

// =============================================================================
// Listitem operations
// =============================================================================

/// Get next list item.
#[inline]
fn tv_listitem_next_impl(li: ListItemHandle) -> ListItemHandle {
    if li.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_next(li) }
}

/// FFI wrapper: get next list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_next(li: ListItemHandle) -> ListItemHandle {
    tv_listitem_next_impl(li)
}

/// Get previous list item.
#[inline]
fn tv_listitem_prev_impl(li: ListItemHandle) -> ListItemHandle {
    if li.is_null() {
        return ListItemHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_prev(li) }
}

/// FFI wrapper: get previous list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_prev(li: ListItemHandle) -> ListItemHandle {
    tv_listitem_prev_impl(li)
}

/// Get typval from list item.
#[inline]
fn tv_listitem_tv_impl(li: ListItemHandle) -> TypevalHandle {
    if li.is_null() {
        return TypevalHandle(std::ptr::null());
    }
    unsafe { nvim_listitem_get_tv(li) }
}

/// FFI wrapper: get typval from list item.
#[no_mangle]
pub extern "C" fn rs_tv_listitem_tv(li: ListItemHandle) -> TypevalHandle {
    tv_listitem_tv_impl(li)
}

// =============================================================================
// List find operation (tv_list_find)
// =============================================================================

/// Find list item at index n.
///
/// This is a full implementation of `tv_list_find` from C.
/// It uses the list's cached index for optimization.
#[inline]
fn tv_list_find_impl(l: ListHandle, n: c_int) -> ListItemHandle {
    if l.is_null() {
        return ListItemHandle(std::ptr::null());
    }

    // Normalize index
    let n = tv_list_uidx_impl(l, n);
    if n == -1 {
        return ListItemHandle(std::ptr::null());
    }

    let len = tv_list_len_impl(l);
    let cached_item = unsafe { nvim_list_get_idx_item(l) };
    let cached_idx = unsafe { nvim_list_get_idx(l) };

    let (mut item, mut idx) = if !cached_item.is_null() {
        // Use cached index for optimization
        if n < cached_idx / 2 {
            // Closest to start
            (tv_list_first_impl(l), 0)
        } else if n > (cached_idx + len) / 2 {
            // Closest to end
            (tv_list_last_impl(l), len - 1)
        } else {
            // Closest to cached
            (cached_item, cached_idx)
        }
    } else {
        // No cache, choose start or end
        if n < len / 2 {
            (tv_list_first_impl(l), 0)
        } else {
            (tv_list_last_impl(l), len - 1)
        }
    };

    // Search forward
    while n > idx {
        item = tv_listitem_next_impl(item);
        idx += 1;
    }

    // Search backward
    while n < idx {
        item = tv_listitem_prev_impl(item);
        idx -= 1;
    }

    // Update cache
    unsafe {
        nvim_list_set_idx(l, idx);
        nvim_list_set_idx_item(l, item);
    }

    item
}

/// FFI wrapper: find list item at index.
#[no_mangle]
pub extern "C" fn rs_tv_list_find(l: ListHandle, n: c_int) -> ListItemHandle {
    tv_list_find_impl(l, n)
}

/// Get the index of a list item within a list.
/// Returns -1 if the list is NULL or item is not in the list.
#[inline]
fn tv_list_idx_of_item_impl(l: ListHandle, item: ListItemHandle) -> c_int {
    if l.is_null() {
        return -1;
    }

    let mut idx = 0;
    let mut li = tv_list_first_impl(l);
    while !li.is_null() {
        if li.0 == item.0 {
            return idx;
        }
        li = tv_listitem_next_impl(li);
        idx += 1;
    }
    -1
}

/// FFI wrapper: get index of list item in list.
#[no_mangle]
pub extern "C" fn rs_tv_list_idx_of_item(l: ListHandle, item: ListItemHandle) -> c_int {
    tv_list_idx_of_item_impl(l, item)
}

/// Reverse a list in-place by swapping next/prev pointers.
#[inline]
fn tv_list_reverse_impl(l: ListHandle) {
    if l.is_null() {
        return;
    }

    let len = tv_list_len_impl(l);
    if len <= 1 {
        return;
    }

    // Swap lv_first and lv_last
    let first = tv_list_first_impl(l);
    let last = tv_list_last_impl(l);
    unsafe {
        nvim_list_set_first(l, last);
        nvim_list_set_last(l, first);
    }

    // Iterate through and swap li_next and li_prev for each item.
    // After swapping first/last, lv_first now points to old last.
    // We traverse using li_next AFTER swapping it (which points to old li_prev).
    let mut li = unsafe { nvim_list_get_first(l) };
    while !li.is_null() {
        let next = tv_listitem_next_impl(li);
        let prev = tv_listitem_prev_impl(li);
        unsafe {
            nvim_listitem_set_next(li, prev);
            nvim_listitem_set_prev(li, next);
        }
        // After swap, li_next now points to what was li_prev.
        // We need to follow that to continue "backwards" through original list.
        li = tv_listitem_next_impl(li);
    }

    // Update the cached index: new_idx = len - old_idx - 1
    let old_idx = unsafe { nvim_list_get_idx(l) };
    let new_idx = len - old_idx - 1;
    unsafe {
        nvim_list_set_idx(l, new_idx);
    }
}

/// FFI wrapper: reverse list in-place.
#[no_mangle]
pub extern "C" fn rs_tv_list_reverse(l: ListHandle) {
    tv_list_reverse_impl(l);
}

// =============================================================================
// Dict operations
// =============================================================================

/// Get the number of items in a dictionary.
/// Returns 0 if the dict is NULL.
#[inline]
fn tv_dict_len_impl(d: DictHandle) -> i64 {
    if d.is_null() {
        return 0;
    }
    unsafe { nvim_dict_get_ht_used(d) as i64 }
}

/// FFI wrapper: get dict length.
#[no_mangle]
pub extern "C" fn rs_tv_dict_len(d: DictHandle) -> i64 {
    tv_dict_len_impl(d)
}

/// Get dict lock status.
#[inline]
fn tv_dict_locked_impl(d: DictHandle) -> c_int {
    if d.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_dict_get_lock(d) }
}

/// FFI wrapper: get dict lock status.
#[no_mangle]
pub extern "C" fn rs_tv_dict_locked(d: DictHandle) -> c_int {
    tv_dict_locked_impl(d)
}

/// Check if dictionary is watched.
#[inline]
fn tv_dict_is_watched_impl(d: DictHandle) -> bool {
    if d.is_null() {
        return false;
    }
    unsafe { nvim_dict_has_watchers(d) != 0 }
}

/// FFI wrapper: check if dict is watched.
#[no_mangle]
pub extern "C" fn rs_tv_dict_is_watched(d: DictHandle) -> c_int {
    c_int::from(tv_dict_is_watched_impl(d))
}

// =============================================================================
// Blob operations
// =============================================================================

/// Get the length of the data in the blob, in bytes.
/// Returns 0 if the blob is NULL.
#[inline]
fn tv_blob_len_impl(b: BlobHandle) -> c_int {
    if b.is_null() {
        return 0;
    }
    unsafe { nvim_blob_get_len(b) }
}

/// FFI wrapper: get blob length.
#[no_mangle]
pub extern "C" fn rs_tv_blob_len(b: BlobHandle) -> c_int {
    tv_blob_len_impl(b)
}

/// Get blob lock status.
#[inline]
fn tv_blob_locked_impl(b: BlobHandle) -> c_int {
    if b.is_null() {
        return 2; // VAR_FIXED
    }
    unsafe { nvim_blob_get_lock(b) }
}

/// FFI wrapper: get blob lock status.
#[no_mangle]
pub extern "C" fn rs_tv_blob_locked(b: BlobHandle) -> c_int {
    tv_blob_locked_impl(b)
}

/// Get the byte at index `idx` in the blob.
/// Caller must ensure blob is non-NULL and idx is valid.
#[inline]
fn tv_blob_get_impl(b: BlobHandle, idx: c_int) -> u8 {
    unsafe { nvim_blob_get_byte(b, idx) }
}

/// FFI wrapper: get byte from blob.
#[no_mangle]
pub extern "C" fn rs_tv_blob_get(b: BlobHandle, idx: c_int) -> u8 {
    tv_blob_get_impl(b, idx)
}

/// Check if two blobs are equal (byte-by-byte comparison).
/// Empty and NULL blobs are considered equal.
#[inline]
fn tv_blob_equal_impl(b1: BlobHandle, b2: BlobHandle) -> bool {
    let len1 = tv_blob_len_impl(b1);
    let len2 = tv_blob_len_impl(b2);

    // empty and NULL are considered the same
    if len1 == 0 && len2 == 0 {
        return true;
    }
    if b1.0 == b2.0 {
        return true;
    }
    if len1 != len2 {
        return false;
    }

    // Compare byte by byte
    for i in 0..len1 {
        if tv_blob_get_impl(b1, i) != tv_blob_get_impl(b2, i) {
            return false;
        }
    }
    true
}

/// FFI wrapper: check if two blobs are equal.
#[no_mangle]
pub extern "C" fn rs_tv_blob_equal(b1: BlobHandle, b2: BlobHandle) -> bool {
    tv_blob_equal_impl(b1, b2)
}

// =============================================================================
// Typval -> container conversions
// =============================================================================

/// Get the list from a typval (returns NULL handle if not a list or NULL).
#[inline]
fn tv_get_list_impl(tv: TypevalHandle) -> ListHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::List {
        return ListHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_list(tv) }
}

/// FFI wrapper: get list from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_list(tv: TypevalHandle) -> ListHandle {
    tv_get_list_impl(tv)
}

/// Get the dict from a typval (returns NULL handle if not a dict or NULL).
#[inline]
fn tv_get_dict_impl(tv: TypevalHandle) -> DictHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::Dict {
        return DictHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_dict(tv) }
}

/// FFI wrapper: get dict from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_dict(tv: TypevalHandle) -> DictHandle {
    tv_get_dict_impl(tv)
}

/// Get the blob from a typval (returns NULL handle if not a blob or NULL).
#[inline]
fn tv_get_blob_impl(tv: TypevalHandle) -> BlobHandle {
    if tv.is_null() || tv_type_impl(tv) != VarType::Blob {
        return BlobHandle(std::ptr::null());
    }
    unsafe { nvim_tv_get_blob(tv) }
}

/// FFI wrapper: get blob from typval.
#[no_mangle]
pub extern "C" fn rs_tv_get_blob(tv: TypevalHandle) -> BlobHandle {
    tv_get_blob_impl(tv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vartype_from_c_int() {
        assert_eq!(VarType::from_c_int(0), Some(VarType::Unknown));
        assert_eq!(VarType::from_c_int(1), Some(VarType::Number));
        assert_eq!(VarType::from_c_int(2), Some(VarType::String));
        assert_eq!(VarType::from_c_int(10), Some(VarType::Blob));
        assert_eq!(VarType::from_c_int(99), None);
    }

    #[test]
    fn test_typval_handle_null() {
        let handle = unsafe { TypevalHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_type_impl(handle), VarType::Unknown);
        assert!(!tv_is_number_impl(handle));
        assert!(!tv_is_string_impl(handle));
        assert!(!tv_is_float_impl(handle));
        assert!(tv_is_empty_impl(handle));
    }

    #[test]
    fn test_list_handle_null() {
        let handle = unsafe { ListHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_list_len_impl(handle), 0);
        assert_eq!(tv_list_locked_impl(handle), 2); // VAR_FIXED
        assert!(!tv_list_has_watchers_impl(handle));
        assert!(tv_list_first_impl(handle).is_null());
        assert!(tv_list_last_impl(handle).is_null());
    }

    #[test]
    fn test_dict_handle_null() {
        let handle = unsafe { DictHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_dict_len_impl(handle), 0);
        assert_eq!(tv_dict_locked_impl(handle), 2); // VAR_FIXED
        assert!(!tv_dict_is_watched_impl(handle));
    }

    #[test]
    fn test_blob_handle_null() {
        let handle = unsafe { BlobHandle::from_ptr(std::ptr::null()) };
        assert!(handle.is_null());
        assert_eq!(tv_blob_len_impl(handle), 0);
        assert_eq!(tv_blob_locked_impl(handle), 2); // VAR_FIXED
    }

    #[test]
    fn test_list_uidx() {
        // With null list (len=0), all indices should return -1
        let null_list = unsafe { ListHandle::from_ptr(std::ptr::null()) };
        assert_eq!(tv_list_uidx_impl(null_list, 0), -1);
        assert_eq!(tv_list_uidx_impl(null_list, -1), -1);
    }
}
