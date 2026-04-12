//! Sign map and namespace registry — Rust-owned storage
//!
//! This module owns `sign_map` (previously C `PMap(cstr_t)`) and `sign_ns`
//! (previously C `kvec_t(Integer)`), eliminating ~14 C accessor shim functions.
//!
//! Memory model for `sign_map`:
//! - Keys are Rust `String`s (heap-allocated, dropped when removed from map).
//! - Values are `*mut SignT` allocated with `xcalloc`/`xfree` via FFI.
//! - `sign_T.sn_name` is a separate `xstrdup` allocation (its own C heap block),
//!   freed by `rs_sign_undefine_by_name` via `xfree(sn_name)`.
//!
//! Thread safety: Neovim is single-threaded for sign operations, so `static mut`
//! with `unsafe` access is acceptable.

use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void, CStr};

use crate::SignHandle;

// =============================================================================
// C FFI Declarations
// =============================================================================

extern "C" {
    fn describe_ns(ns: c_int, unknown: *const c_char) -> *const c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_create_namespace(name: *const c_char) -> c_int;
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;
}

// =============================================================================
// Static Storage
// =============================================================================

static mut SIGN_MAP: Option<HashMap<String, *mut crate::SignT>> = None;
static mut SIGN_NS: Option<Vec<i64>> = None;

#[allow(static_mut_refs)]
#[inline]
unsafe fn sign_map() -> &'static mut HashMap<String, *mut crate::SignT> {
    SIGN_MAP.get_or_insert_with(HashMap::new)
}

#[allow(static_mut_refs)]
#[inline]
unsafe fn sign_ns() -> &'static mut Vec<i64> {
    SIGN_NS.get_or_insert_with(Vec::new)
}

// =============================================================================
// sign_map accessor functions (replaces C shims)
// =============================================================================

/// Get sign by name. Returns null if not found.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_map_get(name: *const c_char) -> SignHandle {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let key = CStr::from_ptr(name).to_str().unwrap_or("");
    sign_map().get(key).copied().unwrap_or(std::ptr::null_mut())
}

/// Check if sign exists by name. Returns 1 if exists, 0 otherwise.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_map_has(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    let key = CStr::from_ptr(name).to_str().unwrap_or("");
    c_int::from(sign_map().contains_key(key))
}

/// Get number of signs in the map.
///
/// # Safety
/// Safe to call from any context in single-threaded Neovim.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn nvim_sign_map_size() -> c_int {
    sign_map().len() as c_int
}

/// Get the nth key from the sign map (for iteration).
///
/// Returns null if idx is out of range.
/// The returned pointer is valid until the map is modified.
///
/// # Safety
/// Returned pointer is only valid as long as the map entry exists.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn nvim_sign_map_get_nth_key(idx: c_int) -> *mut c_char {
    if idx < 0 {
        return std::ptr::null_mut();
    }
    sign_map()
        .keys()
        .nth(idx as usize)
        .map_or(std::ptr::null_mut(), |k| {
            k.as_ptr().cast::<c_char>().cast_mut()
        })
}

/// Get the nth value from the sign map (for iteration).
///
/// Returns null if idx is out of range.
///
/// # Safety
/// Returned pointer is valid as long as the sign_T is not freed.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn nvim_sign_map_get_nth_value(idx: c_int) -> SignHandle {
    if idx < 0 {
        return std::ptr::null_mut();
    }
    sign_map()
        .values()
        .nth(idx as usize)
        .copied()
        .unwrap_or(std::ptr::null_mut())
}

/// Remove sign from map by name. Returns the sign_T pointer (caller must free it).
/// Returns null if not found.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
/// Caller is responsible for freeing the returned sign_T.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_map_del(name: *const c_char) -> SignHandle {
    if name.is_null() {
        return std::ptr::null_mut();
    }
    let key = CStr::from_ptr(name).to_str().unwrap_or("");
    sign_map().remove(key).unwrap_or(std::ptr::null_mut())
}

/// Get or create sign entry. Returns the sign_T pointer and sets *is_new.
///
/// If created, allocates a new sign_T with xcalloc and xstrdup's the name
/// for `sn_name`. The map key is a separate Rust String.
///
/// # Safety
/// `name` must be a valid null-terminated C string (not null).
/// `is_new` must be a valid pointer to bool.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_map_get_or_create(
    name: *const c_char,
    is_new: *mut bool,
) -> SignHandle {
    if name.is_null() {
        *is_new = false;
        return std::ptr::null_mut();
    }
    let Ok(key_str) = CStr::from_ptr(name).to_str() else {
        *is_new = false;
        return std::ptr::null_mut();
    };

    let map = sign_map();
    if let Some(&sp) = map.get(key_str) {
        *is_new = false;
        return sp;
    }

    // Create new sign_T
    let sp = xcalloc(1, std::mem::size_of::<crate::SignT>()).cast::<crate::SignT>();
    if sp.is_null() {
        *is_new = false;
        return std::ptr::null_mut();
    }
    // Set sn_name to its own xstrdup allocation
    (*sp).sn_name = xstrdup(name);

    map.insert(key_str.to_owned(), sp);
    *is_new = true;
    sp
}

// =============================================================================
// sign_ns accessor functions (replaces C shims)
// =============================================================================

/// Get number of sign namespaces.
///
/// # Safety
/// Safe to call from any context in single-threaded Neovim.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn nvim_sign_ns_size() -> c_int {
    sign_ns().len() as c_int
}

/// Get sign namespace id at index. Returns -1 if out of range.
///
/// # Safety
/// Safe to call from any context in single-threaded Neovim.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn nvim_sign_ns_get(idx: c_int) -> i64 {
    if idx < 0 {
        return -1;
    }
    sign_ns().get(idx as usize).copied().unwrap_or(-1)
}

/// Get sign namespace name at index. Returns null if out of range.
///
/// # Safety
/// `idx` must be in range. Returned pointer is valid until next call to `describe_ns`.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn nvim_sign_ns_get_name(idx: c_int) -> *mut c_char {
    if idx < 0 {
        return std::ptr::null_mut();
    }
    let Some(&ns) = sign_ns().get(idx as usize) else {
        return std::ptr::null_mut();
    };
    let name = describe_ns(ns as c_int, c"".as_ptr());
    // describe_ns returns a const pointer; cast to mut for C API compat
    name.cast_mut()
}

/// Push a namespace ID to the sign namespace list.
///
/// # Safety
/// Safe to call from any context in single-threaded Neovim.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_ns_push(ns: i64) {
    sign_ns().push(ns);
}

// =============================================================================
// Namespace creation (replaces C shims that wrap nvim_create_namespace)
// =============================================================================

/// Create or get a namespace by name for sign use. Returns namespace ID.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_create_namespace_cstr(name: *const c_char) -> c_int {
    nvim_create_namespace(name)
}

/// Check if a namespace exists by name. Returns 1 if exists, 0 otherwise.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn nvim_sign_namespace_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    c_int::from(nvim_namespace_lookup(name) != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(static_mut_refs)]
    fn test_sign_ns_push_get() {
        unsafe {
            // Reset for test isolation
            SIGN_NS = Some(Vec::new());
            nvim_sign_ns_push(42);
            nvim_sign_ns_push(100);
            assert_eq!(nvim_sign_ns_size(), 2);
            assert_eq!(nvim_sign_ns_get(0), 42);
            assert_eq!(nvim_sign_ns_get(1), 100);
            assert_eq!(nvim_sign_ns_get(2), -1);
            SIGN_NS = None;
        }
    }
}
