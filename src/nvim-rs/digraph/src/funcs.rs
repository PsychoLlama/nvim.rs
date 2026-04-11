//! `VimL` built-in digraph functions.
//!
//! This module implements `f_digraph_get` and `f_digraph_set` migrated from
//! `src/nvim/digraph.c`.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_void};

use libc::c_int;

// =============================================================================
// C accessor declarations
// =============================================================================

extern "C" {
    /// Get string from typval with error checking. Returns NULL on type error.
    /// Uses a static buffer for non-string types. buf must be at least NUMBUFLEN bytes.
    fn nvim_tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;

    /// Get string from typval with error checking (no buf). Returns NULL on error.
    fn nvim_tv_get_string_chk(tv: *const c_void, out_len: *mut usize) -> *const u8;

    /// Set typval type.
    fn nvim_tv_set_type(tv: *mut c_void, v_type: c_int);

    /// Set typval bool value.
    fn nvim_tv_set_bool(tv: *mut c_void, val: c_int);

    /// Set typval string value (takes ownership).
    fn nvim_tv_set_string(tv: *mut c_void, s: *mut u8);

    /// Advance a string pointer by one character, return codepoint.
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;

    /// Convert codepoint to UTF-8 bytes.
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    /// Duplicate a string.
    fn xstrdup(s: *const c_char) -> *mut c_char;

    /// Display a formatted error message.
    fn semsg(fmt: *const c_char, ...) -> c_int;

    /// Translate a string via gettext.
    fn nvim_gettext(s: *const c_char) -> *const c_char;
}

// Rust functions from this crate
extern "C" {
    /// Look up a digraph by its two characters.
    #[link_name = "digraph_get"]
    fn rs_digraph_get(char1: c_int, char2: c_int, meta_char: bool) -> c_int;

    /// Register a user digraph.
    fn rs_registerdigraph(char1: c_int, char2: c_int, result: c_int);

    /// Check digraph characters are valid (now a Rust export).
    fn check_digraph_chars_valid(char1: c_int, char2: c_int) -> bool;
}

// VarType integer constants (from typval_defs.h)
const VAR_BOOL: c_int = 7;

// kBoolVarFalse = 0, kBoolVarTrue = 1
const K_BOOL_VAR_FALSE: c_int = 0;
const K_BOOL_VAR_TRUE: c_int = 1;

// TYPVAL_SIZE: must match C sizeof(typval_T) (guarded by _Static_assert in testing.c)
const TYPVAL_SIZE: usize = 16;

// NUMBUFLEN from nvim/vim_defs.h
const NUMBUFLEN: usize = 65;

/// Get a typval from the argvars array by index.
///
/// # Safety
/// argvars must be a valid pointer to an array of at least `index + 1` `typval_T` values.
#[inline]
const unsafe fn argvar_at(argvars: *const c_void, index: usize) -> *const c_void {
    let offset = index * TYPVAL_SIZE;
    unsafe { argvars.cast::<u8>().add(offset).cast::<c_void>() }
}

/// Error strings for digraph validation.
const E_DIGRAPH_MUST_BE_TWO_CHARS: &[u8] = b"E1214: Digraph must be just two characters: %s\0";
const E_DIGRAPH_ARG_MUST_BE_ONE_CHAR: &[u8] = b"E1215: Digraph must be one character: %s\0";

/// Get the two digraph characters from a typval.
///
/// Returns `Some((char1, char2))` on success, `None` on failure.
/// Emits an error message on failure.
///
/// # Safety
/// `arg` must be a valid `*const typval_T`.
#[allow(clippy::similar_names)]
unsafe fn get_digraph_chars_impl(arg: *const c_void) -> Option<(c_int, c_int)> {
    let mut buf = [0u8; NUMBUFLEN];
    let chars = unsafe { nvim_tv_get_string_buf_chk(arg, buf.as_mut_ptr().cast::<c_char>()) };

    if chars.is_null() {
        return None;
    }

    let p = chars;
    if unsafe { *p } == 0 {
        // Empty string
        let fmt = unsafe { nvim_gettext(E_DIGRAPH_MUST_BE_TWO_CHARS.as_ptr().cast::<c_char>()) };
        unsafe { semsg(fmt, chars) };
        return None;
    }

    let mut pp = p;
    let char1 = unsafe { mb_cptr2char_adv(&raw mut pp) };

    if unsafe { *pp } == 0 {
        // Only one character
        let fmt = unsafe { nvim_gettext(E_DIGRAPH_MUST_BE_TWO_CHARS.as_ptr().cast::<c_char>()) };
        unsafe { semsg(fmt, chars) };
        return None;
    }

    let char2 = unsafe { mb_cptr2char_adv(&raw mut pp) };

    if unsafe { *pp } != 0 {
        // More than two characters
        let fmt = unsafe { nvim_gettext(E_DIGRAPH_MUST_BE_TWO_CHARS.as_ptr().cast::<c_char>()) };
        unsafe { semsg(fmt, chars) };
        return None;
    }

    if !unsafe { check_digraph_chars_valid(char1, char2) } {
        return None;
    }

    Some((char1, char2))
}

/// Shared logic for `f_digraph_set` and `f_digraph_setlist`.
///
/// Validates and registers a digraph from two typval arguments.
/// Returns true on success, false on failure (error already reported).
///
/// # Safety
/// Both pointers must be valid `*const typval_T`.
#[export_name = "digraph_set_common"]
pub unsafe extern "C" fn digraph_set_common_impl(
    argchars: *const c_void,
    argdigraph: *const c_void,
) -> bool {
    let Some((char1, char2)) = (unsafe { get_digraph_chars_impl(argchars) }) else {
        return false;
    };

    let mut buf_digraph = [0u8; NUMBUFLEN];
    let digraph = unsafe {
        nvim_tv_get_string_buf_chk(argdigraph, buf_digraph.as_mut_ptr().cast::<c_char>())
    };

    if digraph.is_null() {
        return false;
    }

    let mut p = digraph;
    let n = unsafe { mb_cptr2char_adv(&raw mut p) };

    if unsafe { *p } != 0 {
        // More than one character
        let fmt = unsafe { nvim_gettext(E_DIGRAPH_ARG_MUST_BE_ONE_CHAR.as_ptr().cast::<c_char>()) };
        unsafe { semsg(fmt, digraph) };
        return false;
    }

    unsafe { rs_registerdigraph(char1, char2, n) };
    true
}

// =============================================================================
// Phase 2: VimL digraph_get() and digraph_set()
// =============================================================================

/// `digraph_get()` `VimL` function.
///
/// Returns the character for the given two-character digraph.
///
/// # Safety
/// argvars and rettv must be valid `typval_T` pointers.
#[export_name = "f_digraph_get"]
pub unsafe extern "C" fn rs_f_digraph_get(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: c_void,
) {
    // rettv->v_type = VAR_STRING; rettv->vval.v_string = NULL
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };

    let tv0 = unsafe { argvar_at(argvars, 0) };
    let mut len: usize = 0;
    let digraphs = unsafe { nvim_tv_get_string_chk(tv0, &raw mut len) };

    if digraphs.is_null() {
        return;
    }

    if len != 2 {
        let fmt = unsafe { nvim_gettext(E_DIGRAPH_MUST_BE_TWO_CHARS.as_ptr().cast::<c_char>()) };
        unsafe { semsg(fmt, digraphs.cast::<c_char>()) };
        return;
    }

    let c1 = unsafe { c_int::from(*digraphs) };
    let c2 = unsafe { c_int::from(*digraphs.add(1)) };
    let code = unsafe { rs_digraph_get(c1, c2, false) };

    let mut buf = [0u8; 7];
    let blen = unsafe { utf_char2bytes(code, buf.as_mut_ptr().cast::<c_char>()) } as usize;
    buf[blen] = 0;

    let dup = unsafe { xstrdup(buf.as_ptr().cast::<c_char>()) };
    unsafe { nvim_tv_set_string(rettv, dup.cast::<u8>()) };
}

/// `digraph_set()` `VimL` function.
///
/// Sets a user digraph. Returns v:true on success, v:false on failure.
///
/// # Safety
/// argvars and rettv must be valid `typval_T` pointers.
#[export_name = "f_digraph_set"]
pub unsafe extern "C" fn rs_f_digraph_set(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: c_void,
) {
    // rettv->v_type = VAR_BOOL; rettv->vval.v_bool = kBoolVarFalse
    unsafe { nvim_tv_set_type(rettv, VAR_BOOL) };
    unsafe { nvim_tv_set_bool(rettv, K_BOOL_VAR_FALSE) };

    let tv0 = unsafe { argvar_at(argvars, 0) };
    let tv1 = unsafe { argvar_at(argvars, 1) };

    if !unsafe { digraph_set_common_impl(tv0, tv1) } {
        return;
    }

    unsafe { nvim_tv_set_bool(rettv, K_BOOL_VAR_TRUE) };
}

// =============================================================================
// Phase 3: VimL digraph_getlist() and digraph_setlist()
// =============================================================================

extern "C" {
    /// Allocate a new `list_T` and set rettv to it.
    fn nvim_tv_list_alloc_ret(rettv: *mut c_void, len: isize) -> *mut c_void;

    /// Allocate a new empty `list_T`.
    fn nvim_tv_list_alloc() -> *mut c_void;

    /// Append a string to a list.
    fn nvim_tv_list_append_string(list: *mut c_void, s: *const c_char, len: isize);

    /// Append a sublist to a list.
    fn nvim_tv_list_append_list(l: *mut c_void, itemlist: *mut c_void);

    /// Get the type of a typval.
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;

    /// Get the list pointer from a typval.
    fn nvim_tv_get_list(tv: *const c_void) -> *mut c_void;

    /// Get the length of a list.
    fn nvim_list_get_len(l: *const c_void) -> c_int;

    /// Get the first item in a list.
    fn nvim_list_get_first(l: *const c_void) -> *mut c_void;

    /// Get the next listitem.
    fn nvim_listitem_get_next(li: *const c_void) -> *mut c_void;

    /// Get the typval from a listitem.
    fn nvim_listitem_get_tv(li: *mut c_void) -> *mut c_void;

    /// Check for optional bool arg. Returns OK (0) or FAIL (-1).
    fn nvim_tv_check_for_opt_bool_arg(args: *const c_void, idx: c_int) -> c_int;

    /// Get bool value from typval.
    fn nvim_tv_get_bool(tv: *const c_void) -> c_int;

    /// Get default digraph table length.
    fn rs_get_digraphdefault_len() -> c_int;

    /// Iterate default digraphs with a callback.
    fn rs_digraph_iterate_default(
        callback: unsafe extern "C" fn(u8, u8, c_int, *mut c_void) -> c_int,
        ctx: *mut c_void,
    ) -> c_int;

    /// Iterate user digraphs with a callback.
    fn rs_digraph_iterate_user(
        callback: unsafe extern "C" fn(u8, u8, c_int, *mut c_void) -> c_int,
        ctx: *mut c_void,
    ) -> c_int;

    /// Display an error message.
    fn emsg(s: *const c_char) -> c_int;
}

// VarType constants
const VAR_UNKNOWN: c_int = 0;
const VAR_LIST: c_int = 6;

// Error message for digraph_setlist
const E_SETLIST_MUST_BE_LIST: &[u8] =
    b"E1216: digraph_setlist() argument must be a list of lists with two items ";

// OK/FAIL constants (matching C)
const OK: c_int = 0;

/// Callback context for `digraph_getlist` iteration.
struct GetlistCtx {
    list: *mut c_void,
}

/// Callback for digraph iteration that appends entries to a `VimL` list.
///
/// # Safety
/// `ctx` must be a valid `*mut GetlistCtx`.
unsafe extern "C" fn getlist_callback(
    char1: u8,
    char2: u8,
    result: c_int,
    ctx: *mut c_void,
) -> c_int {
    let gctx = ctx.cast::<GetlistCtx>();
    let outer_list = unsafe { (*gctx).list };

    // Allocate a 2-element sublist
    let sub = unsafe { nvim_tv_list_alloc() };

    // Append it to the outer list
    unsafe { nvim_tv_list_append_list(outer_list, sub) };

    // Build "{c1}{c2}\0" string
    let mut key_buf = [0u8; 3];
    key_buf[0] = char1;
    key_buf[1] = char2;
    key_buf[2] = 0;
    unsafe { nvim_tv_list_append_string(sub, key_buf.as_ptr().cast::<c_char>(), -1) };

    // Build UTF-8 result string
    let mut val_buf = [0u8; 8];
    let blen = unsafe { utf_char2bytes(result, val_buf.as_mut_ptr().cast::<c_char>()) } as usize;
    val_buf[blen] = 0;
    unsafe { nvim_tv_list_append_string(sub, val_buf.as_ptr().cast::<c_char>(), -1) };

    // Continue if not interrupted (got_int check is done by rs_digraph_iterate_*)
    1
}

/// Core logic for `digraph_getlist()`.
///
/// # Safety
/// `rettv` must be a valid `*mut typval_T`.
unsafe fn digraph_getlist_common_impl(list_all: bool, rettv: *mut c_void) {
    // Allocate list into rettv
    let len = unsafe { rs_get_digraphdefault_len() } as isize;
    let outer = unsafe { nvim_tv_list_alloc_ret(rettv, len) };

    let mut ctx = GetlistCtx { list: outer };
    let ctx_ptr: *mut GetlistCtx = &raw mut ctx;

    if list_all {
        unsafe {
            rs_digraph_iterate_default(getlist_callback, ctx_ptr.cast::<c_void>());
        }
    }
    unsafe {
        rs_digraph_iterate_user(getlist_callback, ctx_ptr.cast::<c_void>());
    }
}

/// `digraph_getlist()` `VimL` function.
///
/// Returns list of all or user digraphs.
///
/// # Safety
/// argvars and rettv must be valid `typval_T` pointers.
#[export_name = "f_digraph_getlist"]
pub unsafe extern "C" fn rs_f_digraph_getlist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: c_void,
) {
    // tv_check_for_opt_bool_arg(argvars, 0) -- FAIL means return early
    let check = unsafe { nvim_tv_check_for_opt_bool_arg(argvars, 0) };
    if check != OK {
        return;
    }

    let tv0 = argvars;
    let tv0_type: c_int = unsafe { nvim_tv_get_type(tv0) };
    let flag_list_all = if tv0_type == VAR_UNKNOWN {
        false
    } else {
        let bool_val: c_int = unsafe { nvim_tv_get_bool(tv0) };
        bool_val != 0
    };

    unsafe { digraph_getlist_common_impl(flag_list_all, rettv) };
}

/// `digraph_setlist()` `VimL` function.
///
/// Sets multiple user digraphs from a list of `[chars, result]` pairs.
///
/// # Safety
/// argvars and rettv must be valid `typval_T` pointers.
#[export_name = "f_digraph_setlist"]
pub unsafe extern "C" fn rs_f_digraph_setlist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: c_void,
) {
    unsafe { nvim_tv_set_type(rettv, VAR_BOOL) };
    unsafe { nvim_tv_set_bool(rettv, K_BOOL_VAR_FALSE) };

    let tv0 = argvars;
    if unsafe { nvim_tv_get_type(tv0) } != VAR_LIST {
        let msg = unsafe { nvim_gettext(E_SETLIST_MUST_BE_LIST.as_ptr().cast::<c_char>()) };
        unsafe { emsg(msg) };
        return;
    }

    let pl = unsafe { nvim_tv_get_list(tv0) };
    if pl.is_null() {
        // Empty list is success
        unsafe { nvim_tv_set_bool(rettv, K_BOOL_VAR_TRUE) };
        return;
    }

    // Iterate over list items
    let mut pli = unsafe { nvim_list_get_first(pl) };
    while !pli.is_null() {
        let item_tv = unsafe { nvim_listitem_get_tv(pli) };
        if item_tv.is_null() || unsafe { nvim_tv_get_type(item_tv.cast::<c_void>()) } != VAR_LIST {
            let msg = unsafe { nvim_gettext(E_SETLIST_MUST_BE_LIST.as_ptr().cast::<c_char>()) };
            unsafe { emsg(msg) };
            return;
        }

        let l = unsafe { nvim_tv_get_list(item_tv.cast::<c_void>()) };
        if l.is_null() || unsafe { nvim_list_get_len(l) } != 2 {
            let msg = unsafe { nvim_gettext(E_SETLIST_MUST_BE_LIST.as_ptr().cast::<c_char>()) };
            unsafe { emsg(msg) };
            return;
        }

        // Get first and second items from sublist
        let first = unsafe { nvim_list_get_first(l) };
        if first.is_null() {
            let msg = unsafe { nvim_gettext(E_SETLIST_MUST_BE_LIST.as_ptr().cast::<c_char>()) };
            unsafe { emsg(msg) };
            return;
        }
        let first_tv = unsafe { nvim_listitem_get_tv(first) };

        let second = unsafe { nvim_listitem_get_next(first) };
        if second.is_null() {
            let msg = unsafe { nvim_gettext(E_SETLIST_MUST_BE_LIST.as_ptr().cast::<c_char>()) };
            unsafe { emsg(msg) };
            return;
        }
        let second_tv = unsafe { nvim_listitem_get_tv(second) };

        if !unsafe {
            digraph_set_common_impl(first_tv.cast::<c_void>(), second_tv.cast::<c_void>())
        } {
            return;
        }

        pli = unsafe { nvim_listitem_get_next(pli) };
    }

    unsafe { nvim_tv_set_bool(rettv, K_BOOL_VAR_TRUE) };
}
