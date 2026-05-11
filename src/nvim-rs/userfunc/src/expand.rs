//! Function-name completion iterator for VimL.
//!
//! Phase 3 (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `get_user_func_name` — ExpandGeneric callback for `:call <Tab>` completion.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    // func hashtab iteration primitives (Phase 3 accessors)
    fn nvim_func_ht_array() -> *mut c_void;
    fn nvim_func_ht_used() -> usize;
    fn nvim_func_ht_changed() -> c_int;
    fn nvim_hashitem_advance(hi: *mut c_void) -> *mut c_void;
    fn nvim_hashitem_is_empty(hi: *const c_void) -> c_int;
    fn nvim_hashitem_to_ufunc(hi: *const c_void) -> *mut c_void;

    // ufunc_T accessors
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_name(fp: *mut c_void) -> *const c_char;
    fn nvim_ufunc_get_namelen(fp: *mut c_void) -> usize;
    fn nvim_ufunc_get_varargs(fp: *mut c_void) -> c_int;
    fn nvim_ufunc_get_args_empty(fp: *mut c_void) -> c_int;

    // IObuff / IOSIZE / EXPAND_USER_FUNC
    fn nvim_get_iobuff() -> *mut c_char;
    fn nvim_get_iosize_int() -> c_int;
    fn nvim_expand_user_func() -> c_int;
    fn nvim_iobuff_xstrlcpy_at(offset: c_int, src: *const c_char);

    // cat_func_name (listing.rs)
    fn rs_cat_func_name(buf: *mut c_char, bufsize: usize, fp: *mut c_void) -> c_int;

    // expand_T xp_context accessor
    fn nvim_expand_get_context(xp: *const c_void) -> c_int;
}

// FC_DICT flag (matches userfunc.h)
const FC_DICT: c_int = 0x04;

// Static iteration cursor (mirrors C static state in get_user_func_name)
// These are reset when idx == 0.
static mut DONE: usize = 0;
static mut CHANGED: c_int = 0;
static mut HI: *mut c_void = std::ptr::null_mut();

// =============================================================================
// rs_get_user_func_name (export_name = "get_user_func_name")
// =============================================================================

/// ExpandGeneric callback: return the next user-defined function name for
/// cmdline completion.
///
/// On `idx == 0`, the iteration cursor is reset to the start of func_hashtab.
/// Returns:
/// - `""` for dict/lambda functions (skip them but don't stop iteration)
/// - A pointer to `IObuff` (with name, optionally with `(` or `()`)
/// - `NULL` when iteration is exhausted or the hash table was modified
///
/// # Safety
/// Accesses and mutates static mutable state. Must only be called from the
/// main VimL thread (single-threaded context).
#[unsafe(export_name = "get_user_func_name")]
pub unsafe extern "C" fn rs_get_user_func_name(xp: *const c_void, idx: c_int) -> *mut c_char {
    if idx == 0 {
        unsafe {
            DONE = 0;
            HI = nvim_func_ht_array();
            CHANGED = nvim_func_ht_changed();
        }
    }

    // Guard: pointer must be initialised
    if unsafe { HI.is_null() } {
        return std::ptr::null_mut();
    }

    let changed_now = unsafe { nvim_func_ht_changed() };
    let used = unsafe { nvim_func_ht_used() };

    if changed_now != unsafe { CHANGED } || unsafe { DONE } >= used {
        return std::ptr::null_mut();
    }

    // Advance past empty slots (skip on the first call after reset too, matching
    // the C `if (done++ > 0) hi++; while (HASHITEM_EMPTY(hi)) hi++;` pattern).
    if unsafe { DONE } > 0 {
        unsafe { HI = nvim_hashitem_advance(HI) };
    }
    unsafe { DONE += 1 };

    // Skip empty slots
    while unsafe { nvim_hashitem_is_empty(HI) } != 0 {
        unsafe { HI = nvim_hashitem_advance(HI) };
    }

    let fp = unsafe { nvim_hashitem_to_ufunc(HI) };
    if fp.is_null() {
        return std::ptr::null_mut();
    }

    let flags = unsafe { nvim_ufunc_get_flags(fp) };
    let name = unsafe { nvim_ufunc_get_name(fp) };

    // Skip dict functions and lambdas
    if flags & FC_DICT != 0 {
        return c"".as_ptr().cast_mut();
    }
    // Check for "<lambda>" prefix (first 8 bytes)
    let namelen_for_check = unsafe { nvim_ufunc_get_namelen(fp) };
    let name_bytes =
        unsafe { std::slice::from_raw_parts(name.cast::<u8>(), 8.min(namelen_for_check)) };
    if name_bytes == b"<lambda>" {
        return c"".as_ptr().cast_mut();
    }

    let iosize = unsafe { nvim_get_iosize_int() };
    let namelen = unsafe { nvim_ufunc_get_namelen(fp) } as c_int;

    // Prevent overflow
    if namelen + 4 >= iosize {
        return name.cast_mut();
    }

    let iobuff = unsafe { nvim_get_iobuff() };
    let len = unsafe { rs_cat_func_name(iobuff, iosize as usize, fp) };

    let xp_context = unsafe { nvim_expand_get_context(xp) };
    if xp_context != unsafe { nvim_expand_user_func() } {
        unsafe { nvim_iobuff_xstrlcpy_at(len, c"(".as_ptr()) };
        if unsafe { nvim_ufunc_get_varargs(fp) } == 0
            && unsafe { nvim_ufunc_get_args_empty(fp) } != 0
        {
            unsafe { nvim_iobuff_xstrlcpy_at(len + 1, c")".as_ptr()) };
        }
    }

    iobuff
}
