//! Lambda / alloc_ufunc / register_luafunc migration.
//!
//! Phase 2 (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `get_lambda_name` (Rust: internal `get_lambda_name_str`)
//! - `alloc_ufunc`     (Rust: rs_alloc_ufunc)
//! - `register_luafunc` (Rust: rs_register_luafunc, keeps C linkage name)

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicI32, Ordering};

// NUMBUFLEN = 65 (matches C define in vim_defs.h)
const NUMBUFLEN: usize = 65;
// lambda name buffer: "<lambda>" (8) + NUMBUFLEN for the integer
const LAMBDA_BUF_SIZE: usize = 8 + NUMBUFLEN;

// Static counter and buffer for lambda names (mirrors C static state)
static LAMBDA_NO: AtomicI32 = AtomicI32::new(0);

extern "C" {
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_sizeof_ufunc_header() -> usize;
    fn nvim_ufunc_init_name(fp: *mut c_void, name: *const c_char, namelen: usize);
    fn nvim_ufunc_init_luaref_fields(fp: *mut c_void, lua_ref: c_int);
    fn nvim_func_ht_add_fp(fp: *mut c_void);
    fn nvim_ufunc_get_name_ptr(fp: *mut c_void) -> *mut c_char;
}

// =============================================================================
// get_lambda_name (internal)
// =============================================================================

/// Generate the next lambda name.
///
/// Returns a stack-allocated buffer containing "<lambda>N\0".
/// Caller gets a slice up to (but not including) the NUL.
fn get_lambda_name_bytes() -> ([u8; LAMBDA_BUF_SIZE], usize) {
    let n = LAMBDA_NO.fetch_add(1, Ordering::Relaxed) + 1;
    let mut buf = [0u8; LAMBDA_BUF_SIZE];
    let prefix = b"<lambda>";
    buf[..prefix.len()].copy_from_slice(prefix);
    // Format the integer manually to avoid formatting machinery in no_std-like context
    let n_str = n.to_string();
    let n_bytes = n_str.as_bytes();
    let len = prefix.len() + n_bytes.len();
    buf[prefix.len()..len].copy_from_slice(n_bytes);
    buf[len] = 0; // NUL terminate
    (buf, len)
}

// =============================================================================
// rs_alloc_ufunc
// =============================================================================

/// Allocate a ufunc_T for a function with the given name and length.
/// Returns a pointer to the newly allocated (xcalloc'd) ufunc_T.
/// The caller is responsible for further initialization.
///
/// # Safety
/// `name` must be a valid pointer to at least `namelen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_alloc_ufunc(name: *const c_char, namelen: usize) -> *mut c_void {
    let header_size = unsafe { nvim_sizeof_ufunc_header() };
    let total_size = header_size + namelen + 1;
    let fp = unsafe { xcalloc(1, total_size) };
    if !fp.is_null() {
        unsafe { nvim_ufunc_init_name(fp, name, namelen) };
    }
    fp
}

// =============================================================================
// rs_get_lambda_name (exported for C callers in get_lambda_tv, register_luafunc)
// =============================================================================

/// Write a new lambda name into `buf` (capacity `bufsize`).
/// Returns the number of bytes written (not counting NUL), or 0 on overflow.
///
/// # Safety
/// `buf` must point to at least `bufsize` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_get_lambda_name(buf: *mut c_char, bufsize: usize) -> usize {
    if buf.is_null() || bufsize == 0 {
        return 0;
    }
    let (bytes, len) = get_lambda_name_bytes();
    let copy_len = len.min(bufsize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buf, copy_len + 1);
    }
    copy_len
}

// =============================================================================
// rs_register_luafunc
// =============================================================================

/// Register a Lua callback as a named lambda ufunc.
/// Returns `fp->uf_name` (the function name string).
///
/// Mirrors C `register_luafunc(ref)`.
///
/// # Safety
/// `lua_ref` must be a valid LuaRef (int).
#[unsafe(export_name = "register_luafunc")]
pub unsafe extern "C" fn rs_register_luafunc(lua_ref: c_int) -> *mut c_char {
    let (name_bytes, name_len) = get_lambda_name_bytes();
    let fp = unsafe { rs_alloc_ufunc(name_bytes.as_ptr().cast::<c_char>(), name_len) };
    if fp.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_ufunc_init_luaref_fields(fp, lua_ref) };
    unsafe { nvim_func_ht_add_fp(fp) };
    // coverity[leaked_storage] - intentional: fp lives until func_free
    unsafe { nvim_ufunc_get_name_ptr(fp) }
}
