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
