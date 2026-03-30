//! Simple self-contained VimL built-in functions (Phase 1).
//!
//! These are trivially self-contained functions migrated from `src/nvim/eval/funcs.c`.
//! Phase 1 inlines the C accessor body into Rust, calling underlying C APIs directly.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C accessor declarations (still-delegated functions)
// =============================================================================

extern "C" {
    fn nvim_eval_api_info(argvars: *const c_void, rettv: *mut c_void);
    // nvim_eval_byte2line: inlined into rs_f_byte2line below
    // nvim_eval_line2byte: inlined into rs_f_line2byte below
    // nvim_eval_gettext: inlined into rs_f_gettext below
    // nvim_eval_keytrans: inlined into rs_f_keytrans below
    // nvim_eval_luaeval: inlined into rs_f_luaeval below
    // nvim_eval_pum_getpos: inlined into rs_f_pum_getpos below
    // nvim_eval_wordcount: inlined into rs_f_wordcount below
    // nvim_eval_menu_get: inlined into rs_f_menu_get below
    fn nvim_eval_visualmode(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Direct underlying C function declarations (inlined Phase 1 functions)
// =============================================================================

extern "C" {
    // timer_stopall
    fn timer_stop_all();

    // soundfold / getenv / visualmode - string ownership
    fn eval_soundfold(word: *const u8) -> *mut u8;
    fn nvim_tv_set_string(tv: *mut c_void, s: *mut u8);

    // synIDtrans
    fn nvim_tv_get_number(tv: *const c_void) -> i64;
    fn syn_get_final_id(id: c_int) -> c_int;

    // garbagecollect
    static mut want_garbage_collect: bool;
    static mut garbage_collect_at_exit: bool;
    fn nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;

    // getenv / setenv
    fn vim_getenv(name: *const u8) -> *mut u8;
    fn vim_setenv_ext(name: *const u8, val: *const u8);
    fn vim_unsetenv_ext(var: *const u8);

    // setenv: get string with buf (buf must be NUMBUFLEN bytes)
    fn nvim_tv_get_string_buf(tv: *const c_void, buf: *mut u8) -> *const u8;
    // mode: duplicate a C string (uses nvim allocator)
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // shiftwidth
    fn nvim_eval_get_sw_value_col(col: c_int) -> c_int;
    fn nvim_eval_get_sw_value() -> c_int;

    // mode
    fn nvim_eval_get_mode(buf: *mut u8);
    fn nvim_eval_non_zero_arg(argvars: *const c_void, idx: c_int) -> c_int;

    // nextnonblank / prevnonblank
    fn nvim_eval_tv_get_lnum(argvars: *const c_void) -> i32;
    fn nvim_eval_curbuf_ml_line_count() -> i32;
    fn ml_get(lnum: i32) -> *const c_char;
    #[link_name = "skipwhite"]
    fn nvim_skipwhite(p: *const c_char) -> *const c_char;

    // wildmenumode
    fn nvim_eval_wildmenumode_check() -> c_int;

    // set_special_null for getenv
    fn nvim_tv_set_special_null(tv: *mut c_void);

    // gettext: translate a string (gettext() / _() macro)
    fn gettext(msgid: *const c_char) -> *const c_char;
    // tv_check_for_nonempty_string_arg: validate arg is non-empty string
    fn tv_check_for_nonempty_string_arg(argvars: *const c_void, idx: c_int) -> c_int;

    // pum_getpos / wordcount inlining
    fn tv_dict_alloc_ret(rettv: *mut c_void);
    #[link_name = "nvim_tv_get_dict"]
    fn nvim_tv_get_dict_ptr(tv: *const c_void) -> *const c_void;
    fn pum_set_event_info(dict: *mut c_void);
    fn cursor_pos_info(dict: *mut c_void);

    // luaeval inlining
    #[link_name = "tv_get_string_chk"]
    fn s_tv_get_string_chk(tv: *mut c_void) -> *const c_char;
    fn nlua_typval_eval(str: NvimApiString, arg: *const c_void, rettv: *mut c_void);

    // byte2line / line2byte inlining
    fn nvim_get_curbuf() -> *mut c_void;
    fn rs_ml_find_line_or_offset(
        buf: *mut c_void,
        lnum: i64,
        offp: *mut c_int,
        no_ff: c_int,
    ) -> c_int;

    // menu_get inlining
    fn get_menu_cmd_modes(
        cmd: *const c_char,
        force_menu: bool,
        noremap: *mut c_int,
        unmenu: *mut c_int,
    ) -> c_int;
    fn menu_get(path_name: *mut c_char, modes: c_int, list: *mut c_void);
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
    // nvim_eval_byte2line: inlined — rs_ml_find_line_or_offset delegation
    #[allow(clippy::cast_possible_truncation)]
    let mut boff = nvim_tv_get_number(argvars) as c_int - 1;
    let result = if boff < 0 {
        -1i64
    } else {
        let curbuf = nvim_get_curbuf();
        i64::from(rs_ml_find_line_or_offset(curbuf, 0, &raw mut boff, 0))
    };
    nvim_tv_set_number(rettv, result);
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
    // nvim_eval_line2byte: inlined — rs_ml_find_line_or_offset delegation
    let lnum = nvim_eval_tv_get_lnum(argvars);
    let ml_count = nvim_eval_curbuf_ml_line_count();
    let mut result = if lnum < 1 || lnum > ml_count + 1 {
        -1i64
    } else {
        let curbuf = nvim_get_curbuf();
        i64::from(rs_ml_find_line_or_offset(
            curbuf,
            i64::from(lnum),
            std::ptr::null_mut(),
            0,
        ))
    };
    if result >= 0 {
        result += 1;
    }
    nvim_tv_set_number(rettv, result);
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
    // nvim_eval_gettext: inlined — gettext delegation
    const FAIL: c_int = 0;
    if tv_check_for_nonempty_string_arg(argvars, 0) == FAIL {
        return;
    }
    let s = nvim_tv_get_string_ptr(argvars);
    let translated = gettext(s.cast::<c_char>());
    let copy = xstrdup(translated);
    nvim_tv_set_string(rettv, copy.cast::<u8>());
}

/// "garbagecollect([atexit])" function - trigger garbage collection
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_garbagecollect"]
pub unsafe extern "C" fn rs_f_garbagecollect(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // VAR_UNKNOWN = 0
    want_garbage_collect = true;
    if nvim_tv_get_type(argvars) != 0 && nvim_tv_get_number(argvars) == 1 {
        garbage_collect_at_exit = true;
    }
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
    nvim_tv_set_number(rettv, 0); // FAIL
    let pid = nvim_tv_get_number(argvars);
    if pid == 0 {
        let _ = emsg((&raw const E_INVARG).cast::<c_char>());
        return;
    }
    // SIGINT on Linux/Unix (matches C code's #ifndef MSWIN branch)
    libc::kill(pid as libc::pid_t, libc::SIGINT);
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
    let name = nvim_tv_get_string_ptr(argvars); // returns *const u8
    let p = vim_getenv(name); // takes *const u8, returns *mut u8
    if p.is_null() {
        nvim_tv_set_special_null(rettv);
        return;
    }
    nvim_tv_set_string(rettv, p); // takes ownership of *mut u8
}

/// "setenv(name, value)" function - set environment variable
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setenv"]
pub unsafe extern "C" fn rs_f_setenv(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // NUMBUFLEN = 65
    const NUMBUFLEN: usize = 65;
    if rs_check_secure() != 0 {
        return;
    }
    let mut namebuf = [0u8; NUMBUFLEN];
    let mut valbuf = [0u8; NUMBUFLEN];
    let name = nvim_tv_get_string_buf(argvars, namebuf.as_mut_ptr());
    // argvars[1] is at offset TYPVAL_SZ bytes
    let arg1 = argvars.cast::<u8>().add(TYPVAL_SZ).cast::<c_void>();
    // VAR_SPECIAL = 8 means null special
    if nvim_tv_get_type(arg1) == 8 {
        vim_unsetenv_ext(name);
    } else {
        let val = nvim_tv_get_string_buf(arg1, valbuf.as_mut_ptr());
        vim_setenv_ext(name, val);
    }
}

/// "pum_getpos()" function - get popup menu position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_pum_getpos"]
pub unsafe extern "C" fn rs_f_pum_getpos(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // nvim_eval_pum_getpos: inlined — pum_set_event_info delegation
    tv_dict_alloc_ret(rettv);
    let dict = nvim_tv_get_dict_ptr(rettv).cast_mut();
    pum_set_event_info(dict);
}

/// "wordcount()" function - word count information dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_wordcount"]
pub unsafe extern "C" fn rs_f_wordcount(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // nvim_eval_wordcount: inlined — cursor_pos_info delegation
    tv_dict_alloc_ret(rettv);
    let dict = nvim_tv_get_dict_ptr(rettv).cast_mut();
    cursor_pos_info(dict);
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
    let s = nvim_tv_get_string_ptr(argvars);
    let folded = eval_soundfold(s);
    nvim_tv_set_string(rettv, folded);
}

/// "wildmenumode()" function - check if wildmenu is active
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_wildmenumode"]
pub unsafe extern "C" fn rs_f_wildmenumode(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_tv_set_number(rettv, i64::from(nvim_eval_wildmenumode_check()));
}

/// "timer_stopall()" function - stop all timers
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_timer_stopall"]
pub unsafe extern "C" fn rs_f_timer_stopall(
    _argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    timer_stop_all();
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
    let id = nvim_tv_get_number(argvars) as c_int;
    let result = if id > 0 { syn_get_final_id(id) } else { 0 };
    nvim_tv_set_number(rettv, i64::from(result));
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
    // nvim_eval_keytrans: inlined — vim_strsave_escape_ks + str2special_save
    const FAIL: c_int = 0;
    nvim_tv_set_string(rettv, std::ptr::null_mut());
    if tv_check_for_string_arg(argvars, 0) == FAIL {
        return;
    }
    let s = nvim_tv_get_string_ptr(argvars);
    if s.is_null() {
        return;
    }
    let escaped = vim_strsave_escape_ks(s.cast_mut().cast::<c_char>());
    let result = str2special_save(escaped, true, true);
    xfree(escaped.cast::<c_void>());
    nvim_tv_set_string(rettv, result.cast::<u8>());
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
    // nvim_eval_luaeval: inlined — nlua_typval_eval delegation
    let str_ptr = s_tv_get_string_chk(argvars.cast_mut());
    if str_ptr.is_null() {
        return;
    }
    let api_str = NvimApiString::from_cstr(str_ptr.cast::<u8>());
    let arg1 = arg_at(argvars, 1);
    nlua_typval_eval(api_str, arg1, rettv);
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
    nvim_tv_set_number(rettv, 0);
    // VAR_UNKNOWN = 0
    if nvim_tv_get_type(argvars) != 0 {
        let mut error = false;
        let col = nvim_tv_get_number_chk(argvars, &raw mut error);
        if error || col < 0 {
            return;
        }
        nvim_tv_set_number(rettv, i64::from(nvim_eval_get_sw_value_col(col as c_int)));
        return;
    }
    nvim_tv_set_number(rettv, i64::from(nvim_eval_get_sw_value()));
}

/// "mode([expr])" function - current editing mode
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_mode"]
pub unsafe extern "C" fn rs_f_mode(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    let mut buf = [0u8; 4]; // MODE_MAX_LENGTH = 4
    nvim_eval_get_mode(buf.as_mut_ptr());
    if nvim_eval_non_zero_arg(argvars, 0) == 0 {
        buf[1] = 0; // NUL-terminate at index 1 (like C: buf[1] = NUL)
    }
    let copy = xstrdup(buf.as_ptr().cast::<c_char>());
    nvim_tv_set_string(rettv, copy.cast::<u8>());
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
    let max_lnum = nvim_eval_curbuf_ml_line_count();
    let mut lnum = nvim_eval_tv_get_lnum(argvars);
    loop {
        if lnum < 0 || lnum > max_lnum {
            lnum = 0;
            break;
        }
        let line = ml_get(lnum);
        let after_ws = nvim_skipwhite(line);
        if !after_ws.is_null() && *after_ws != 0 {
            break;
        }
        lnum += 1;
    }
    nvim_tv_set_number(rettv, i64::from(lnum));
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
    let max_lnum = nvim_eval_curbuf_ml_line_count();
    let mut lnum = nvim_eval_tv_get_lnum(argvars);
    if lnum < 1 || lnum > max_lnum {
        lnum = 0;
    } else {
        while lnum >= 1 {
            let line = ml_get(lnum);
            let after_ws = nvim_skipwhite(line);
            if !after_ws.is_null() && *after_ws != 0 {
                break;
            }
            lnum -= 1;
        }
    }
    nvim_tv_set_number(rettv, i64::from(lnum));
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
    // nvim_eval_menu_get: inlined — menu_get delegation
    // kListLenMayKnow = -3
    const K_LIST_LEN_MAY_KNOW: isize = -3;
    // MENU_ALL_MODES = (1 << 7) - 1 = 127
    const MENU_ALL_MODES: c_int = 127;
    let list = nvim_tv_list_alloc_ret(rettv, K_LIST_LEN_MAY_KNOW);
    let mut modes = MENU_ALL_MODES;
    let arg1 = arg_at(argvars, 1);
    if nvim_tv_get_type(arg1) == VAR_STRING {
        let strmodes = nvim_tv_get_string_ptr(arg1);
        modes = get_menu_cmd_modes(
            strmodes.cast::<c_char>(),
            false,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
    // Cast the string to *mut c_char (menu_get takes non-const)
    let path = nvim_tv_get_string_ptr(argvars).cast::<c_char>().cast_mut();
    menu_get(path, modes, list);
}

/// Size of typval_T in bytes (must match C sizeof(typval_T) = 16).
const TYPVAL_SZ: usize = 16;

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

    // --- keytrans inlining ---
    fn tv_check_for_string_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn vim_strsave_escape_ks(s: *mut c_char) -> *mut c_char;
    fn str2special_save(s: *const c_char, replace_spaces: bool, replace_lt: bool) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
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
