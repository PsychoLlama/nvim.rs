//! Simple self-contained VimL built-in functions (Phase 1).
//!
//! These are trivially self-contained functions migrated from `src/nvim/eval/funcs.c`.
//! Each delegates to a thin C accessor.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_api_info(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_byte2line(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_line2byte(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_gettext(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_garbagecollect(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_debugbreak(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_getenv(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_setenv(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_pum_getpos(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_wordcount(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_soundfold(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_wildmenumode(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_timer_stopall(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_synIDtrans(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_keytrans(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_luaeval(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_shiftwidth(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_mode(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_visualmode(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_nextnonblank(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_prevnonblank(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_menu_get(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 1: Simple self-contained functions
// =============================================================================

/// "api_info()" function - returns API metadata
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_api_info"]
pub unsafe extern "C" fn rs_f_api_info(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_api_info(argvars, rettv);
}

/// "byte2line(byte)" function - convert byte offset to line number
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_byte2line"]
pub unsafe extern "C" fn rs_f_byte2line(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_byte2line(argvars, rettv);
}

/// "line2byte(lnum)" function - convert line number to byte offset
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_line2byte"]
pub unsafe extern "C" fn rs_f_line2byte(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_line2byte(argvars, rettv);
}

/// "gettext(text)" function - translate a string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_gettext"]
pub unsafe extern "C" fn rs_f_gettext(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_gettext(argvars, rettv);
}

/// "garbagecollect([atexit])" function - trigger garbage collection
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_garbagecollect"]
pub unsafe extern "C" fn rs_f_garbagecollect(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_garbagecollect(argvars, rettv);
}

/// "debugbreak(pid)" function - send SIGINT to process
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_debugbreak"]
pub unsafe extern "C" fn rs_f_debugbreak(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_debugbreak(argvars, rettv);
}

/// "getenv(name)" function - get environment variable
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getenv"]
pub unsafe extern "C" fn rs_f_getenv(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getenv(argvars, rettv);
}

/// "setenv(name, value)" function - set environment variable
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setenv"]
pub unsafe extern "C" fn rs_f_setenv(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_setenv(argvars, rettv);
}

/// "pum_getpos()" function - get popup menu position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_pum_getpos"]
pub unsafe extern "C" fn rs_f_pum_getpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_pum_getpos(argvars, rettv);
}

/// "wordcount()" function - word count information dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_wordcount"]
pub unsafe extern "C" fn rs_f_wordcount(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_wordcount(argvars, rettv);
}

/// "soundfold(word)" function - fold a word using 'spelllang' soundfold
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_soundfold"]
pub unsafe extern "C" fn rs_f_soundfold(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_soundfold(argvars, rettv);
}

/// "wildmenumode()" function - check if wildmenu is active
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_wildmenumode"]
pub unsafe extern "C" fn rs_f_wildmenumode(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_wildmenumode(argvars, rettv);
}

/// "timer_stopall()" function - stop all timers
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_timer_stopall"]
pub unsafe extern "C" fn rs_f_timer_stopall(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_timer_stopall(argvars, rettv);
}

/// "synIDtrans(id)" function - get final syntax ID (following links)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[allow(non_snake_case)]
#[export_name = "f_synIDtrans"]
pub unsafe extern "C" fn rs_f_synIDtrans(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_synIDtrans(argvars, rettv);
}

/// "keytrans(string)" function - translate key notation to printable form
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_keytrans"]
pub unsafe extern "C" fn rs_f_keytrans(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_keytrans(argvars, rettv);
}

/// "luaeval(expr [, expr])" function - evaluate Lua expression
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_luaeval"]
pub unsafe extern "C" fn rs_f_luaeval(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_luaeval(argvars, rettv);
}

/// "shiftwidth([col])" function - effective value of 'shiftwidth'
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_shiftwidth"]
pub unsafe extern "C" fn rs_f_shiftwidth(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_shiftwidth(argvars, rettv);
}

/// "mode([expr])" function - current editing mode
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_mode"]
pub unsafe extern "C" fn rs_f_mode(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_mode(argvars, rettv);
}

/// "visualmode([expr])" function - last visual mode used
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_visualmode"]
pub unsafe extern "C" fn rs_f_visualmode(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_visualmode(argvars, rettv);
}

/// "nextnonblank(lnum)" function - find next non-blank line
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_nextnonblank"]
pub unsafe extern "C" fn rs_f_nextnonblank(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_nextnonblank(argvars, rettv);
}

/// "prevnonblank(lnum)" function - find previous non-blank line
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_prevnonblank"]
pub unsafe extern "C" fn rs_f_prevnonblank(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_prevnonblank(argvars, rettv);
}

/// "menu_get(path [, modes])" function - get menu items
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_menu_get"]
pub unsafe extern "C" fn rs_f_menu_get(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_menu_get(argvars, rettv);
}

// =============================================================================
// Phase 1: Trivial delegation wrappers (plan 40f0fb72)
// =============================================================================

/// Size of a `typval_T` in bytes. Verified by a static assert in testing.c.
const TYPVAL_SIZE: usize = 16;

/// C String struct matching Neovim API `String` type (`{char *data, size_t size}`).
///
/// Used to call `nvim_feedkeys` which takes API String values by value.
#[repr(C)]
struct NvimApiString {
    data: *const c_char,
    size: usize,
}

impl NvimApiString {
    /// Create from a NUL-terminated C string pointer (may be null → empty string).
    unsafe fn from_cstr(s: *const u8) -> Self {
        if s.is_null() {
            Self {
                data: std::ptr::null(),
                size: 0,
            }
        } else {
            Self {
                data: s.cast::<c_char>(),
                size: libc::strlen(s.cast::<c_char>()),
            }
        }
    }

    /// Empty string.
    const fn empty() -> Self {
        Self {
            data: std::ptr::null(),
            size: 0,
        }
    }
}

extern "C" {
    // --- feedkeys ---
    fn nvim_feedkeys(keys: NvimApiString, mode: NvimApiString, escape_ks: bool);

    // --- tagfiles ---
    /// Allocate a new TagFileIterator (heap). Must be freed with rs_tagfname_free.
    fn rs_tagfname_new() -> *mut c_void;
    /// Free a TagFileIterator allocated by rs_tagfname_new.
    fn rs_tagfname_free(tnp: *mut c_void);
    /// Get next tag file name. first=1 on first call, 0 thereafter.
    fn rs_get_tagfname(tnp: *mut c_void, first: c_int, buf: *mut c_char) -> c_int;

    // --- taglist ---
    /// Fill list with tag match dicts for the given pattern/file.
    fn rs_get_tags(list: *mut c_void, pat: *const c_char, buf_fname: *const c_char) -> c_int;

    // --- list alloc/append (eval_shim.c) ---
    /// Set rettv to VAR_LIST and return the newly allocated list_T*.
    #[link_name = "tv_list_alloc_ret"]
    fn nvim_tv_list_alloc_ret(rettv: *mut c_void, count_hint: isize) -> *mut c_void;
    /// Append a copy of `s` (len=-1 uses strlen) to list `l`.
    #[link_name = "tv_list_append_string"]
    fn nvim_tv_list_append_string(l: *mut c_void, s: *const c_char, len: isize);

    // --- serverstop ---
    fn rs_server_stop(addr: *const c_char) -> c_int;

    // --- secure check (feedkeys, serverstop) ---
    fn rs_check_secure() -> c_int;

    // --- type/string accessors ---
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_get_string_ptr(tv: *const c_void) -> *const u8;
    fn nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    fn nvim_tv_set_number(tv: *mut c_void, n: i64);

    // --- error messages ---
    fn emsg(msg: *const c_char) -> c_int;

    // e_invarg: C string constant for "invalid argument" error
    #[link_name = "e_invarg"]
    static E_INVARG: c_char;
}

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
const OK: c_int = 1;
/// kListLenUnknown — hints list allocation not to pre-allocate.
const K_LIST_LEN_UNKNOWN: isize = -1;
/// Maximum path length (matches C MAXPATHL).
const MAXPATHL: usize = 4096;

/// Get pointer to `argvars[idx]` (each element is `TYPVAL_SIZE` bytes).
///
/// # Safety
/// `argvars` must point to at least `idx + 1` valid `typval_T` entries.
#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE)
        .cast::<c_void>()
}

/// "feedkeys(string [, mode])" function - insert keys into the type-ahead buffer.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_feedkeys"]
pub unsafe extern "C" fn rs_f_feedkeys(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if rs_check_secure() != 0 {
        return;
    }

    let mut len: usize = 0;
    let keys_ptr = nvim_tv_get_string(arg_at(argvars, 0), &raw mut len);
    let keys = NvimApiString::from_cstr(keys_ptr);

    let mode = if nvim_tv_get_type(arg_at(argvars, 1)) == VAR_UNKNOWN {
        NvimApiString::empty()
    } else {
        let mut len2: usize = 0;
        NvimApiString::from_cstr(nvim_tv_get_string(arg_at(argvars, 1), &raw mut len2))
    };

    nvim_feedkeys(keys, mode, true);
}

/// "tagfiles()" function - return list of tag file names.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_tagfiles"]
pub unsafe extern "C" fn rs_f_tagfiles(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let list = nvim_tv_list_alloc_ret(rettv, K_LIST_LEN_UNKNOWN);

    let tn = rs_tagfname_new();
    if tn.is_null() {
        return;
    }

    let mut fname_buf = vec![0u8; MAXPATHL];
    let fname = fname_buf.as_mut_ptr().cast::<c_char>();

    let mut first: c_int = 1;
    while rs_get_tagfname(tn, first, fname) == OK {
        nvim_tv_list_append_string(list, fname, -1);
        first = 0;
    }
    rs_tagfname_free(tn);
}

/// "taglist(pat [, filename])" function - return list of tag matches as dicts.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_taglist"]
pub unsafe extern "C" fn rs_f_taglist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Default return: false (0)
    nvim_tv_set_number(rettv, 0);

    let mut len: usize = 0;
    let tag_pattern = nvim_tv_get_string(arg_at(argvars, 0), &raw mut len);
    if tag_pattern.is_null() || *tag_pattern == 0 {
        return;
    }

    let fname: *const c_char = if nvim_tv_get_type(arg_at(argvars, 1)) == VAR_UNKNOWN {
        std::ptr::null()
    } else {
        let mut flen: usize = 0;
        nvim_tv_get_string(arg_at(argvars, 1), &raw mut flen).cast::<c_char>()
    };

    let list = nvim_tv_list_alloc_ret(rettv, K_LIST_LEN_UNKNOWN);
    rs_get_tags(list, tag_pattern.cast::<c_char>(), fname);
}

/// "serverstop(address)" function - stop a named server.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_serverstop"]
pub unsafe extern "C" fn rs_f_serverstop(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if rs_check_secure() != 0 {
        return;
    }

    nvim_tv_set_number(rettv, 0);

    if nvim_tv_get_type(arg_at(argvars, 0)) == VAR_STRING {
        let s = nvim_tv_get_string_ptr(arg_at(argvars, 0));
        if !s.is_null() {
            let rv = rs_server_stop(s.cast::<c_char>());
            nvim_tv_set_number(rettv, i64::from(rv));
        }
    } else {
        let _ = emsg((&raw const E_INVARG).cast::<c_char>());
    }
}
