//! Miscellaneous VimL built-in functions.
//!
//! This module implements trivially self-contained functions from
//! `src/nvim/eval/funcs.c` that were previously thin C wrappers.

use std::ffi::{c_char, c_int, c_void};

use super::dispatch::{rettv_set_number, TypevalPtrMut};

// =============================================================================
// C accessor declarations
// =============================================================================

extern "C" {
    static mut got_int: bool;
    fn nvim_get_vgetc_busy() -> c_int;
    fn nvim_curbuf_get_did_filetype() -> c_int;
    fn nvim_curbuf_get_u_seq_cur() -> c_int;
    fn nvim_get_reg_executing() -> c_int;
    fn nvim_get_reg_recording() -> c_int;
    fn nvim_get_reg_recorded() -> c_int;
    // Direct underlying functions (replaced nvim_eval_* one-liner wrappers)
    fn ui_current_col() -> u32;
    fn ui_current_row() -> u32;
    fn pum_visible() -> bool;
    // os_get_pid already declared in Phase 3 extern block below
    fn nvim_eval_get_col(argvars: *const c_void, rettv: *mut c_void, charcol: bool);
    fn nvim_eval_getpos_both(
        argvars: *const c_void,
        rettv: *mut c_void,
        getcurpos: bool,
        charcol: bool,
    );
    fn nvim_eval_get_windows_version() -> *const c_char;

    // For return_register: set a string on rettv (copied)
    fn nvim_tv_set_string_copy(tv: *mut c_void, s: *const u8, len: c_int);
}

// =============================================================================
// No-op / constant-return functions
// =============================================================================

/// "foreground()" function - no-op on non-GUI builds
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_foreground"]
pub unsafe extern "C" fn rs_f_foreground(
    _argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // intentionally empty - no-op on all supported platforms
}

/// "getfontname()" function - always returns empty string (GUI not supported)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getfontname"]
pub unsafe extern "C" fn rs_f_getfontname(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Returns NULL string (VAR_STRING with NULL value = empty)
    nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);
}

/// "windowsversion()" function - returns empty string (Windows not supported)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_windowsversion"]
pub unsafe extern "C" fn rs_f_windowsversion(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Pass -1 to trigger xstrdup semantics in nvim_tv_set_string_copy
    let ver = nvim_eval_get_windows_version();
    nvim_tv_set_string_copy(rettv, ver.cast::<u8>(), -1);
}

// =============================================================================
// Simple global-read functions
// =============================================================================

/// "getpid()" function - returns the process ID
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getpid"]
pub unsafe extern "C" fn rs_f_getpid(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, os_get_pid());
}

/// "localtime()" function - returns current time in seconds
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_localtime"]
pub unsafe extern "C" fn rs_f_localtime(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    // SAFETY: time() with NULL is always safe
    let t = libc::time(std::ptr::null_mut());
    rettv_set_number(rettv, t as i64);
}

/// "screencol()" function - returns current screen column (1-based)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screencol"]
pub unsafe extern "C" fn rs_f_screencol(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(ui_current_col()) + 1);
}

/// "screenrow()" function - returns current screen row (1-based)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_screenrow"]
pub unsafe extern "C" fn rs_f_screenrow(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(ui_current_row()) + 1);
}

/// "eventhandler()" function - returns true if inside an event handler
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_eventhandler"]
pub unsafe extern "C" fn rs_f_eventhandler(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(nvim_get_vgetc_busy()));
}

/// "did_filetype()" function - returns true if FileType autocommand was fired
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_did_filetype"]
pub unsafe extern "C" fn rs_f_did_filetype(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(nvim_curbuf_get_did_filetype()));
}

/// "changenr()" function - returns current change number
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_changenr"]
pub unsafe extern "C" fn rs_f_changenr(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(nvim_curbuf_get_u_seq_cur()));
}

/// "interrupt()" function - sets got_int to interrupt Vim
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_interrupt"]
pub unsafe extern "C" fn rs_f_interrupt(
    _argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe {
        got_int = true;
    }
}

/// "pumvisible()" function - returns true if popup menu is visible
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_pumvisible"]
pub unsafe extern "C" fn rs_f_pumvisible(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(pum_visible()));
}

// =============================================================================
// Register functions
// =============================================================================

/// Helper: set rettv to a 1-character string for a register name.
///
/// If `regname` is 0, returns an empty string.
unsafe fn set_register_string(rettv: *mut c_void, regname: c_int) {
    if regname == 0 {
        nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);
    } else {
        // regname is an ASCII character code (0-127 range for valid registers)
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let buf = [regname as u8];
        nvim_tv_set_string_copy(rettv, buf.as_ptr(), 1);
    }
}

/// "reg_executing()" function - returns the register being executed
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reg_executing"]
pub unsafe extern "C" fn rs_f_reg_executing(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    set_register_string(rettv, nvim_get_reg_executing());
}

/// "reg_recording()" function - returns the register being recorded to
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reg_recording"]
pub unsafe extern "C" fn rs_f_reg_recording(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    set_register_string(rettv, nvim_get_reg_recording());
}

/// "reg_recorded()" function - returns the last recorded register
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reg_recorded"]
pub unsafe extern "C" fn rs_f_reg_recorded(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    set_register_string(rettv, nvim_get_reg_recorded());
}

// =============================================================================
// Delegation functions (to static C helpers)
// =============================================================================

/// "charcol()" function - get cursor column in characters
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_charcol"]
pub unsafe extern "C" fn rs_f_charcol(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_get_col(argvars, rettv, true);
}

/// "col()" function - get cursor column in bytes
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_col"]
pub unsafe extern "C" fn rs_f_col(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_get_col(argvars, rettv, false);
}

/// "getcharpos()" function - get position as character-indexed list
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcharpos"]
pub unsafe extern "C" fn rs_f_getcharpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getpos_both(argvars, rettv, false, true);
}

/// "getcurpos()" function - get cursor position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcurpos"]
pub unsafe extern "C" fn rs_f_getcurpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getpos_both(argvars, rettv, true, false);
}

/// "getcursorcharpos()" function - get cursor position in character offsets
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcursorcharpos"]
pub unsafe extern "C" fn rs_f_getcursorcharpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getpos_both(argvars, rettv, true, true);
}

/// "getpos()" function - get position as byte-indexed list
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getpos"]
pub unsafe extern "C" fn rs_f_getpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getpos_both(argvars, rettv, false, false);
}

// =============================================================================
// Phase 3: Medium-complexity functions (all delegated to C accessors)
// =============================================================================

extern "C" {
    // Still-delegated C accessors
    fn nvim_eval_nr2char(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_str2float(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_copy(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_deepcopy(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_max_min(argvars: *const c_void, rettv: *mut c_void, domax: bool);
    fn nvim_eval_set_position(argvars: *const c_void, rettv: *mut c_void, charpos: bool);
    fn nvim_eval_set_cursorpos(argvars: *const c_void, rettv: *mut c_void, charcol: bool);
    fn nvim_eval_searchpair_cmn(argvars: *const c_void) -> c_int;
    fn nvim_eval_find_some_match(argvars: *const c_void, rettv: *mut c_void, kind: c_int);

    // Direct underlying functions (Phase 2 inlining)
    fn utf_ptr2char(p: *const u8) -> c_int;
    fn tv_check_num(tv: *const c_void) -> bool;
    fn os_get_hostname(hostname: *mut u8, len: usize);
    fn nvim_eval_ctx_size_impl() -> c_int; // shim for (int)ctx_size()
    fn nvim_eval_ctxpop_impl(); // shim for ctxpop + error msg
    fn internal_error(where_: *const u8);
    fn vim_strsave_escaped(string: *const u8, esc_chars: *const u8) -> *mut u8;
    fn vim_strsave_shellescape(string: *const u8, do_special: bool, do_newline: bool) -> *mut u8;
    fn vim_strsave_fnameescape(fname: *const u8, what: c_int) -> *mut u8;

    // typval string get: returns *const u8 (matches dispatch.rs convention)
    #[link_name = "nvim_tv_get_type"]
    fn p3_misc_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_number"]
    fn p3_misc_tv_get_number(tv: *const c_void) -> i64;
    #[link_name = "nvim_tv_get_float"]
    fn p3_misc_tv_get_float(tv: *const c_void) -> f64;
    #[link_name = "nvim_tv_get_string"]
    fn p3_misc_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    #[link_name = "nvim_tv_set_number"]
    fn p3_misc_tv_set_number(tv: *mut c_void, n: i64);
    #[link_name = "nvim_tv_set_string"]
    fn p3_misc_tv_set_string(tv: *mut c_void, s: *mut u8);
    #[link_name = "nvim_tv_get_string_ptr"]
    fn p3_misc_tv_get_string_ptr(tv: *const c_void) -> *const u8;
    #[link_name = "nvim_tv_get_list"]
    fn p3_misc_tv_get_list(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_list_get_len"]
    fn p3_misc_tv_list_len(l: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_dict"]
    fn p3_misc_tv_get_dict(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_dict_get_len"]
    fn p3_misc_tv_dict_len(d: *const c_void) -> c_int;
    #[link_name = "nvim_tv_blob_len"]
    fn p3_misc_tv_blob_len(tv: *const c_void) -> c_int;
    fn nvim_eval_tv_bool_is_true(tv: *const c_void) -> c_int;
    fn nvim_eval_tv_special_is_null(tv: *const c_void) -> c_int;
    fn nvim_eval_non_zero_arg(argvars: *const c_void, idx: c_int) -> c_int;
    #[link_name = "xstrdup"]
    fn p3_misc_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "emsg"]
    fn p3_misc_emsg(msg: *const c_char) -> c_int;
}

// =============================================================================
// Phase 2: VarType constants for inlined functions
// =============================================================================

const VAR_UNKNOWN_P2M: c_int = 0;
const VAR_NUMBER_P2M: c_int = 1;
const VAR_STRING_P2M: c_int = 2;
const VAR_FUNC_P2M: c_int = 3;
const VAR_FLOAT_P2M: c_int = 6;
const VAR_BOOL_P2M: c_int = 7;
const VAR_SPECIAL_P2M: c_int = 8;
const VAR_PARTIAL_P2M: c_int = 9;

/// VSE_NONE: no special escaping
const VSE_NONE: c_int = 0;

/// "char2nr()" function - convert UTF-8 character to number
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_char2nr"]
pub unsafe extern "C" fn rs_f_char2nr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let arg1 = arg_at_p2(argvars, 1);
    if p3_misc_tv_get_type(arg1) != VAR_UNKNOWN_P2M && !tv_check_num(arg1) {
        return;
    }
    let s = p3_misc_tv_get_string_ptr(argvars);
    let result = utf_ptr2char(s);
    p3_misc_tv_set_number(rettv, i64::from(result));
}

/// "nr2char()" function - convert number to UTF-8 character string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_nr2char"]
pub unsafe extern "C" fn rs_f_nr2char(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_nr2char(argvars, rettv);
}

/// "str2float()" function - convert string to float
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_str2float"]
pub unsafe extern "C" fn rs_f_str2float(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_str2float(argvars, rettv);
}

/// "escape()" function - escape special characters in string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_escape"]
pub unsafe extern "C" fn rs_f_escape(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let mut len: usize = 0;
    let s = p3_misc_tv_get_string(argvars, &raw mut len);
    let arg1 = arg_at_p2(argvars, 1);
    let esc_chars = p3_misc_tv_get_string_ptr(arg1);
    let result = vim_strsave_escaped(s, esc_chars);
    p3_misc_tv_set_string(rettv, result);
}

/// "shellescape()" function - shell-escape a string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_shellescape"]
pub unsafe extern "C" fn rs_f_shellescape(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let do_special = nvim_eval_non_zero_arg(argvars, 1) != 0;
    let s = p3_misc_tv_get_string_ptr(argvars);
    let result = vim_strsave_shellescape(s, do_special, do_special);
    p3_misc_tv_set_string(rettv, result);
}

/// "fnameescape()" function - escape filename special characters
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_fnameescape"]
pub unsafe extern "C" fn rs_f_fnameescape(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let s = p3_misc_tv_get_string_ptr(argvars);
    let result = vim_strsave_fnameescape(s, VSE_NONE);
    p3_misc_tv_set_string(rettv, result);
}

/// "hostname()" function - get the hostname
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_hostname"]
pub unsafe extern "C" fn rs_f_hostname(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let mut buf = [0u8; 256];
    os_get_hostname(buf.as_mut_ptr(), 256);
    let copy = p3_misc_xstrdup(buf.as_ptr().cast::<c_char>());
    p3_misc_tv_set_string(rettv, copy.cast::<u8>());
}

/// "empty()" function - check if value is empty
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_empty"]
pub unsafe extern "C" fn rs_f_empty(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let n = match p3_misc_tv_get_type(argvars) {
        VAR_STRING_P2M | VAR_FUNC_P2M => {
            // empty if string pointer is null or starts with NUL
            let s = p3_misc_tv_get_string_ptr(argvars);
            s.is_null() || *s == 0
        }
        VAR_PARTIAL_P2M => false,
        VAR_NUMBER_P2M => p3_misc_tv_get_number(argvars) == 0,
        VAR_FLOAT_P2M => p3_misc_tv_get_float(argvars) == 0.0,
        VAR_LIST_P2 => {
            let l = p3_misc_tv_get_list(argvars);
            l.is_null() || p3_misc_tv_list_len(l) == 0
        }
        5 => {
            // VAR_DICT
            let d = p3_misc_tv_get_dict(argvars);
            d.is_null() || p3_misc_tv_dict_len(d) == 0
        }
        VAR_BOOL_P2M => nvim_eval_tv_bool_is_true(argvars) == 0,
        VAR_SPECIAL_P2M => nvim_eval_tv_special_is_null(argvars) != 0,
        10 => {
            // VAR_BLOB
            p3_misc_tv_blob_len(argvars) == 0
        }
        _ => {
            // VAR_UNKNOWN: internal error
            internal_error(c"f_empty(UNKNOWN)".as_ptr().cast::<u8>());
            true
        }
    };
    p3_misc_tv_set_number(rettv, i64::from(n));
}

/// "copy()" function - shallow copy a value
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_copy"]
pub unsafe extern "C" fn rs_f_copy(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_copy(argvars, rettv);
}

/// "deepcopy()" function - deep copy a value
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_deepcopy"]
pub unsafe extern "C" fn rs_f_deepcopy(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_deepcopy(argvars, rettv);
}

/// "len()" function - length of string/list/dict/blob
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_len"]
pub unsafe extern "C" fn rs_f_len(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    let result: i64 = match p3_misc_tv_get_type(argvars) {
        VAR_STRING_P2M | VAR_NUMBER_P2M => {
            let mut len: usize = 0;
            let s = p3_misc_tv_get_string(argvars, &raw mut len);
            if s.is_null() {
                0
            } else {
                #[allow(clippy::cast_possible_wrap)]
                {
                    len as i64
                }
            }
        }
        10 => {
            // VAR_BLOB
            i64::from(p3_misc_tv_blob_len(argvars))
        }
        VAR_LIST_P2 => {
            let l = p3_misc_tv_get_list(argvars);
            if l.is_null() {
                0
            } else {
                i64::from(p3_misc_tv_list_len(l))
            }
        }
        5 => {
            // VAR_DICT
            let d = p3_misc_tv_get_dict(argvars);
            if d.is_null() {
                0
            } else {
                i64::from(p3_misc_tv_dict_len(d))
            }
        }
        _ => {
            // VAR_UNKNOWN, VAR_BOOL, VAR_SPECIAL, VAR_FLOAT, VAR_PARTIAL, VAR_FUNC
            let _ = p3_misc_emsg(c"E701: Invalid type for len()".as_ptr());
            return;
        }
    };
    p3_misc_tv_set_number(rettv, result);
}

/// "ctxsize()" function - context stack size
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ctxsize"]
pub unsafe extern "C" fn rs_f_ctxsize(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(nvim_eval_ctx_size_impl()));
}

/// "ctxpop()" function - pop context from stack
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ctxpop"]
pub unsafe extern "C" fn rs_f_ctxpop(
    _argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_ctxpop_impl();
}

/// "max()" function - maximum value in list or dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_max"]
pub unsafe extern "C" fn rs_f_max(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_max_min(argvars, rettv, true);
}

/// "min()" function - minimum value in list or dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_min"]
pub unsafe extern "C" fn rs_f_min(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_max_min(argvars, rettv, false);
}

/// "setcharpos()" function - set position using character offsets
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setcharpos"]
pub unsafe extern "C" fn rs_f_setcharpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_set_position(argvars, rettv, true);
}

/// "setpos()" function - set position using byte offsets
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setpos"]
pub unsafe extern "C" fn rs_f_setpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_set_position(argvars, rettv, false);
}

/// "setcursorcharpos()" function - set cursor position using character offsets
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setcursorcharpos"]
pub unsafe extern "C" fn rs_f_setcursorcharpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_set_cursorpos(argvars, rettv, true);
}

/// "cursor()" function - set cursor position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_cursor"]
pub unsafe extern "C" fn rs_f_cursor(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_set_cursorpos(argvars, rettv, false);
}

/// "searchpair()" function - search for a matching bracket pair
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_searchpair"]
pub unsafe extern "C" fn rs_f_searchpair(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    rettv_set_number(rettv, i64::from(nvim_eval_searchpair_cmn(argvars)));
}

/// "match()" function - find pattern match position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_match"]
pub unsafe extern "C" fn rs_f_match(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_find_some_match(argvars, rettv, 0); // kSomeMatch
}

/// "matchend()" function - find pattern match end position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_matchend"]
pub unsafe extern "C" fn rs_f_matchend(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_find_some_match(argvars, rettv, 1); // kSomeMatchEnd
}

/// "matchlist()" function - find pattern match and return submatches
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_matchlist"]
pub unsafe extern "C" fn rs_f_matchlist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_find_some_match(argvars, rettv, 2); // kSomeMatchList
}

/// "matchstr()" function - find pattern match and return matched string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_matchstr"]
pub unsafe extern "C" fn rs_f_matchstr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_find_some_match(argvars, rettv, 3); // kSomeMatchStr
}

/// "matchstrpos()" function - find pattern match, return string and position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_matchstrpos"]
pub unsafe extern "C" fn rs_f_matchstrpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_find_some_match(argvars, rettv, 4); // kSomeMatchStrPos
}

// =============================================================================
// Phase 4 C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_execute(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_flatten(argvars: *const c_void, rettv: *mut c_void, make_copy: bool);
    fn nvim_eval_common_function(argvars: *const c_void, rettv: *mut c_void, is_funcref: bool);
    fn nvim_eval_hlID(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_hlexists(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_input(argvars: *const c_void, rettv: *mut c_void, dialog: bool);
    fn nvim_eval_json_encode(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_libcall(argvars: *const c_void, rettv: *mut c_void, retstr: bool);
    fn nvim_eval_script_host_eval(name: *const c_char, argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_search(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_searchpairpos(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_swapfilelist(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_swapinfo(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 4: Simple delegation functions
// =============================================================================

/// "execute()" function - execute Ex commands, capture output
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_execute"]
pub unsafe extern "C" fn rs_f_execute(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_execute(argvars, rettv);
}

/// "flatten()" function - flatten a list in-place
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_flatten"]
pub unsafe extern "C" fn rs_f_flatten(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_flatten(argvars, rettv, false);
}

/// "flattennew()" function - flatten a list, returning a new list
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_flattennew"]
pub unsafe extern "C" fn rs_f_flattennew(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_flatten(argvars, rettv, true);
}

/// "funcref()" function - create a Funcref from a function reference
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_funcref"]
pub unsafe extern "C" fn rs_f_funcref(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_common_function(argvars, rettv, true);
}

/// "function()" function - create a Funcref from a function name
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_function"]
pub unsafe extern "C" fn rs_f_function(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_common_function(argvars, rettv, false);
}

/// "hlID()" function - get highlight group ID by name
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[allow(non_snake_case)]
#[export_name = "f_hlID"]
pub unsafe extern "C" fn rs_f_hlID(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_hlID(argvars, rettv);
}

/// "hlexists()" function - check if highlight group exists
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_hlexists"]
pub unsafe extern "C" fn rs_f_hlexists(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_hlexists(argvars, rettv);
}

/// "input()" function - prompt the user for input
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_input"]
pub unsafe extern "C" fn rs_f_input(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_input(argvars, rettv, false);
}

/// "inputdialog()" function - prompt the user via a dialog
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_inputdialog"]
pub unsafe extern "C" fn rs_f_inputdialog(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_input(argvars, rettv, true);
}

/// "json_encode()" function - encode a value to JSON string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_json_encode"]
pub unsafe extern "C" fn rs_f_json_encode(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_json_encode(argvars, rettv);
}

/// "libcall()" function - call a function in an external library (returns string)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_libcall"]
pub unsafe extern "C" fn rs_f_libcall(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_libcall(argvars, rettv, true);
}

/// "libcallnr()" function - call a function in an external library (returns number)
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_libcallnr"]
pub unsafe extern "C" fn rs_f_libcallnr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_libcall(argvars, rettv, false);
}

/// "py3eval()" function - evaluate a Python 3 expression
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_py3eval"]
pub unsafe extern "C" fn rs_f_py3eval(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_script_host_eval(c"python3".as_ptr(), argvars, rettv);
}

/// "perleval()" function - evaluate a Perl expression
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_perleval"]
pub unsafe extern "C" fn rs_f_perleval(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_script_host_eval(c"perl".as_ptr(), argvars, rettv);
}

/// "rubyeval()" function - evaluate a Ruby expression
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_rubyeval"]
pub unsafe extern "C" fn rs_f_rubyeval(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_script_host_eval(c"ruby".as_ptr(), argvars, rettv);
}

/// "search()" function - search for a pattern
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_search"]
pub unsafe extern "C" fn rs_f_search(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_search(argvars, rettv);
}

/// "searchpairpos()" function - search for matching bracket pair, return position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_searchpairpos"]
pub unsafe extern "C" fn rs_f_searchpairpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_searchpairpos(argvars, rettv);
}

/// "swapfilelist()" function - get list of swap files
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_swapfilelist"]
pub unsafe extern "C" fn rs_f_swapfilelist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_swapfilelist(argvars, rettv);
}

/// "swapinfo()" function - get info about a swap file
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_swapinfo"]
pub unsafe extern "C" fn rs_f_swapinfo(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_swapinfo(argvars, rettv);
}

// =============================================================================
// Phase 6 C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_ctxget(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_ctxpush(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_ctxset(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_getcharsearch(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_setcharsearch(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_getreg(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_getregtype(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_getreginfo(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_state(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_searchdecl(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_searchpos(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 6: Context, register, and state functions
// =============================================================================

/// "ctxget()" function - get context from stack
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ctxget"]
pub unsafe extern "C" fn rs_f_ctxget(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_ctxget(argvars, rettv);
}

/// "ctxpush()" function - push context to stack
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ctxpush"]
pub unsafe extern "C" fn rs_f_ctxpush(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_ctxpush(argvars, rettv);
}

/// "ctxset()" function - set context in stack
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_ctxset"]
pub unsafe extern "C" fn rs_f_ctxset(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_ctxset(argvars, rettv);
}

/// "getcharsearch()" function - get character search info
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcharsearch"]
pub unsafe extern "C" fn rs_f_getcharsearch(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getcharsearch(argvars, rettv);
}

/// "setcharsearch()" function - set character search info
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setcharsearch"]
pub unsafe extern "C" fn rs_f_setcharsearch(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_setcharsearch(argvars, rettv);
}

/// "getreg()" function - get register contents
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getreg"]
pub unsafe extern "C" fn rs_f_getreg(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getreg(argvars, rettv);
}

/// "getregtype()" function - get register type
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getregtype"]
pub unsafe extern "C" fn rs_f_getregtype(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getregtype(argvars, rettv);
}

/// "getreginfo()" function - get register info dict
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getreginfo"]
pub unsafe extern "C" fn rs_f_getreginfo(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_getreginfo(argvars, rettv);
}

/// "state()" function - get current editor state string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_state"]
pub unsafe extern "C" fn rs_f_state(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_state(argvars, rettv);
}

/// "searchdecl()" function - search for declaration
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_searchdecl"]
pub unsafe extern "C" fn rs_f_searchdecl(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_searchdecl(argvars, rettv);
}

/// "searchpos()" function - search for pattern, return position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_searchpos"]
pub unsafe extern "C" fn rs_f_searchpos(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_searchpos(argvars, rettv);
}

// =============================================================================
// Phase 8 C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_spellbadword(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_spellsuggest(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_submatch(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_substitute(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_synID(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_synIDattr(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_synconcealed(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_synstack(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 8: Syntax and spell functions
// =============================================================================

/// "spellbadword()" function - find a badly spelled word
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_spellbadword"]
pub unsafe extern "C" fn rs_f_spellbadword(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_spellbadword(argvars, rettv);
}

/// "spellsuggest()" function - suggest correct spellings
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_spellsuggest"]
pub unsafe extern "C" fn rs_f_spellsuggest(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_spellsuggest(argvars, rettv);
}

/// "submatch()" function - get submatch from last regex match
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_submatch"]
pub unsafe extern "C" fn rs_f_submatch(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_submatch(argvars, rettv);
}

/// "substitute()" function - string substitution
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_substitute"]
pub unsafe extern "C" fn rs_f_substitute(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_substitute(argvars, rettv);
}

/// "synID()" function - get syntax ID at a position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[allow(non_snake_case)]
#[export_name = "f_synID"]
pub unsafe extern "C" fn rs_f_synID(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_synID(argvars, rettv);
}

/// "synIDattr()" function - get attribute of a syntax ID
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[allow(non_snake_case)]
#[export_name = "f_synIDattr"]
pub unsafe extern "C" fn rs_f_synIDattr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_synIDattr(argvars, rettv);
}

/// "synconcealed()" function - check if position is concealed
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_synconcealed"]
pub unsafe extern "C" fn rs_f_synconcealed(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_synconcealed(argvars, rettv);
}

/// "synstack()" function - get syntax ID stack at a position
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_synstack"]
pub unsafe extern "C" fn rs_f_synstack(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_synstack(argvars, rettv);
}

// =============================================================================
// Phase 9 C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_index(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_indexof(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_range(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_repeat(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_reduce(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 9: Data structure functions
// =============================================================================

/// "index()" function - find item in list/blob
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_index"]
pub unsafe extern "C" fn rs_f_index(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_index(argvars, rettv);
}

/// "indexof()" function - find item in list/blob matching expr
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_indexof"]
pub unsafe extern "C" fn rs_f_indexof(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_indexof(argvars, rettv);
}

/// "range()" function - create a list of numbers
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_range"]
pub unsafe extern "C" fn rs_f_range(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_range(argvars, rettv);
}

/// "repeat()" function - repeat a string/list/blob
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_repeat"]
pub unsafe extern "C" fn rs_f_repeat(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_repeat(argvars, rettv);
}

/// "reduce()" function - reduce a list/blob/string with a function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reduce"]
pub unsafe extern "C" fn rs_f_reduce(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_reduce(argvars, rettv);
}

// =============================================================================
// Phase 10 C accessor declarations
// =============================================================================

extern "C" {
    fn nvim_eval_eval(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_exists(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_has(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_json_decode(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_printf(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_sha256(argvars: *const c_void, rettv: *mut c_void);
}

// =============================================================================
// Phase 10: has(), eval(), exists() and related
// =============================================================================

/// "eval()" function - evaluate an expression string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_eval"]
pub unsafe extern "C" fn rs_f_eval(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_eval(argvars, rettv);
}

/// "exists()" function - check if a variable/function/option exists
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_exists"]
pub unsafe extern "C" fn rs_f_exists(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_exists(argvars, rettv);
}

/// "has()" function - check if a feature is supported
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_has"]
pub unsafe extern "C" fn rs_f_has(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    nvim_eval_has(argvars, rettv);
}

/// "json_decode()" function - decode a JSON string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_json_decode"]
pub unsafe extern "C" fn rs_f_json_decode(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_json_decode(argvars, rettv);
}

/// "printf()" function - format a string
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_printf"]
pub unsafe extern "C" fn rs_f_printf(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_printf(argvars, rettv);
}

/// "sha256()" function - compute SHA256 hash
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_sha256"]
pub unsafe extern "C" fn rs_f_sha256(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_sha256(argvars, rettv);
}

// =============================================================================
// Phase 2: Simple self-contained functions (plan 40f0fb72)
// =============================================================================

/// Size of a single `typval_T` in bytes (verified by static assert in testing.c).
const TYPVAL_SIZE_P2: usize = 16;

/// Get pointer to `argvars[idx]`.
///
/// # Safety
/// `argvars` must point to at least `idx + 1` valid `typval_T` entries.
#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at_p2(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE_P2)
        .cast::<c_void>()
}

/// VarType constants (C's vartype_T).
const VAR_LIST_P2: c_int = 4;
/// NUMBUFLEN for number→string conversion buffers.
const NUMBUFLEN_P2: usize = 65;
/// C OK/FAIL constants.
const OK_P2: c_int = 1;

// Phase 2 extern C declarations.
// Names prefixed `p2_` in the Rust binding to avoid clashing with dispatch.rs
// re-declarations of the same C symbols (Rust warns on type-identical re-declarations,
// and errors on type-differing ones).
extern "C" {
    // Type/value accessors
    #[link_name = "nvim_tv_get_type"]
    fn p2_nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_string_chk"]
    fn p2_nvim_tv_get_string_chk(tv: *const c_void, out_len: *mut usize) -> *const u8;
    fn tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;
    #[link_name = "nvim_tv_get_list"]
    fn p2_nvim_tv_get_list(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_list_get_len"]
    fn p2_nvim_list_get_len(l: *const c_void) -> c_int;
    fn tv_list_find_nr(l: *mut c_void, n: c_int, error_out: *mut bool) -> i64;
    #[link_name = "nvim_tv_set_number"]
    fn p2_nvim_tv_set_number(tv: *mut c_void, n: i64);
    #[link_name = "nvim_tv_set_float"]
    fn p2_nvim_tv_set_float(tv: *mut c_void, f: f64);
    #[link_name = "nvim_tv_set_string_copy"]
    fn p2_nvim_tv_set_string_copy(tv: *mut c_void, s: *const u8, len: c_int);

    // Filesystem
    fn os_setperm(name: *const c_char, perm: c_int) -> c_int;

    // Profile timing
    fn profile_signed(tm: u64) -> i64;
    fn profile_msg(tm: u64) -> *const c_char;

    // Error messages
    fn nvim_get_e_invarg2() -> *const c_char;
    fn semsg(fmt: *const c_char, ...) -> c_int;
}

/// Convert a 2-element number list to a `u64` proftime_T.
///
/// The list `[high, low]` encodes a nanosecond timestamp as two signed 32-bit values.
/// Returns `None` if the argument is not a valid 2-element list.
///
/// # Safety
/// `arg` must be a valid `typval_T*`.
unsafe fn list2proftime(arg: *const c_void) -> Option<u64> {
    if p2_nvim_tv_get_type(arg) != VAR_LIST_P2 {
        return None;
    }
    let list = p2_nvim_tv_get_list(arg);
    if list.is_null() || p2_nvim_list_get_len(list) != 2 {
        return None;
    }
    let list_mut = list.cast_mut();
    let mut error = false;
    let n1 = tv_list_find_nr(list_mut, 0, &raw mut error);
    let n2 = tv_list_find_nr(list_mut, 1, &raw mut error);
    if error {
        return None;
    }
    // The list stores [high, low] where each is a truncated i32 stored as i64.
    // Recombine into a u64 proftime_T.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    let high = u64::from(n1 as i32 as u32);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    let low = u64::from(n2 as i32 as u32);
    Some((high << 32) | low)
}

/// "id({expr})" function - return string representation of argument's address.
///
/// Returns a hex pointer string like "0x7fff12345678".
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_id"]
pub unsafe extern "C" fn rs_f_id(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    let s = format!("{argvars:p}");
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    p2_nvim_tv_set_string_copy(rettv, s.as_ptr(), s.len() as c_int);
}

/// "setfperm(fname, mode)" function - set file permissions from rwxrwxrwx string.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_setfperm"]
pub unsafe extern "C" fn rs_f_setfperm(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    p2_nvim_tv_set_number(rettv, 0);

    let mut len: usize = 0;
    let fname = p2_nvim_tv_get_string_chk(arg_at_p2(argvars, 0), &raw mut len);
    if fname.is_null() {
        return;
    }

    let mut modebuf = [0u8; NUMBUFLEN_P2];
    let mode_str =
        tv_get_string_buf_chk(arg_at_p2(argvars, 1), modebuf.as_mut_ptr().cast::<c_char>());
    if mode_str.is_null() {
        return;
    }

    // mode string must be exactly 9 characters (rwxrwxrwx)
    let mode_bytes = std::ffi::CStr::from_ptr(mode_str).to_bytes();
    if mode_bytes.len() != 9 {
        let _ = semsg(nvim_get_e_invarg2(), mode_str);
        return;
    }

    let mut mask: c_int = 1;
    let mut mode: c_int = 0;
    for i in (0..9_usize).rev() {
        if mode_bytes[i] != b'-' {
            mode |= mask;
        }
        mask <<= 1;
    }

    let rv = os_setperm(fname.cast::<c_char>(), mode);
    p2_nvim_tv_set_number(rettv, i64::from(rv == OK_P2));
}

/// "reltimefloat({time})" function - convert reltime list to float seconds.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reltimefloat"]
pub unsafe extern "C" fn rs_f_reltimefloat(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    p2_nvim_tv_set_float(rettv, 0.0);
    if let Some(tm) = list2proftime(arg_at_p2(argvars, 0)) {
        let signed_ns = profile_signed(tm);
        #[allow(clippy::cast_precision_loss)]
        p2_nvim_tv_set_float(rettv, signed_ns as f64 / 1_000_000_000.0);
    }
}

/// "reltimestr({time})" function - convert reltime list to string representation.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reltimestr"]
pub unsafe extern "C" fn rs_f_reltimestr(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Default: NULL string
    p2_nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);
    if let Some(tm) = list2proftime(arg_at_p2(argvars, 0)) {
        let s = profile_msg(tm);
        if !s.is_null() {
            // profile_msg returns a static buffer; copy it into the rettv
            p2_nvim_tv_set_string_copy(rettv, s.cast::<u8>(), -1);
        }
    }
}

// =============================================================================
// Phase 3: Reltime and random functions (plan 40f0fb72)
// =============================================================================

use std::sync::Mutex;

/// Global xoshiro128** PRNG state, initialized lazily.
static GLOBAL_RAND_STATE: Mutex<Option<[u32; 4]>> = Mutex::new(None);

/// VarType constant: VAR_NUMBER (used by f_rand/f_srand).
const VAR_NUMBER_P3: c_int = 1;
/// VarType constant: VAR_UNKNOWN (no argument).
const VAR_UNKNOWN_P3: c_int = 0;
/// VarType constant: VAR_LIST.
const VAR_LIST_P3: c_int = 4;

/// TYPVAL_SIZE for Phase 3 arg_at usage.
const TYPVAL_SIZE_P3: usize = 16;

extern "C" {
    // Timing (profile.c)
    fn profile_start() -> u64;
    fn profile_end(tm: u64) -> u64;
    #[link_name = "profile_sub"]
    fn c_profile_sub(tm1: u64, tm2: u64) -> u64;

    // List construction for reltime/srand return values
    // tv_list_alloc_ret: returns list_T* set in rettv
    #[link_name = "tv_list_alloc_ret"]
    fn nvim_tv_list_alloc_ret(rettv: *mut c_void, count_hint: isize) -> *mut c_void;
    fn tv_list_append_number(l: *mut c_void, nr: i64);

    // Type/value accessors for f_rand list argument
    #[link_name = "nvim_tv_get_type"]
    fn p3_nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_number"]
    fn p3_nvim_tv_get_number(tv: *const c_void) -> i64;
    #[link_name = "nvim_tv_set_number"]
    fn p3_nvim_tv_set_number(tv: *mut c_void, n: i64);
    #[link_name = "nvim_tv_get_number_chk"]
    fn p3_nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    #[link_name = "nvim_tv_get_list"]
    fn p3_nvim_tv_get_list(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_list_get_len"]
    fn p3_nvim_list_get_len(l: *const c_void) -> c_int;
    /// Get list item at index (returns mutable item handle).
    fn tv_list_find(l: *mut c_void, idx: c_int) -> *mut c_void;

    // OS functions for random seeding
    fn os_hrtime() -> u64;
    fn os_get_pid() -> i64;

    // libuv synchronous random fill:
    //   uv_random(NULL, NULL, buf, buflen, 0, NULL) → int (0 = success)
    fn uv_random(
        loop_: *mut c_void,
        req: *mut c_void,
        buf: *mut c_void,
        buflen: usize,
        flags: u32,
        cb: *mut c_void,
    ) -> c_int;

    // Error helpers
    #[link_name = "nvim_tv_get_string"]
    fn p3_nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    #[link_name = "semsg"]
    fn p3_semsg(fmt: *const c_char, ...) -> c_int;
    #[link_name = "e_invarg2"]
    static P3_E_INVARG2: c_char;
}

/// Initialize a 32-bit random seed using the OS random source or fallback.
///
/// Mirrors C's `init_srand()`: tries `uv_random`; on failure falls back to
/// `os_hrtime() XOR os_get_pid()`.
///
/// # Safety
/// Calls libuv and OS functions; safe to call from Rust FFI context.
unsafe fn init_srand() -> u32 {
    let mut buf = [0u8; 4];
    if uv_random(
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        buf.as_mut_ptr().cast::<c_void>(),
        4,
        0,
        std::ptr::null_mut(),
    ) == 0
    {
        u32::from_ne_bytes(buf)
    } else {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let seed = (os_hrtime() as u32) ^ (os_get_pid() as u32);
        seed
    }
}

/// Get or initialize the global xoshiro128** state, returning `[x, y, z, w]`.
///
/// # Safety
/// Must be called only from Rust FFI context (single-threaded VimL eval).
unsafe fn global_rand_state() -> [u32; 4] {
    let mut guard = GLOBAL_RAND_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    #[allow(clippy::option_if_let_else)]
    if let Some(state) = *guard {
        state
    } else {
        let mut x = init_srand();
        let s = [
            crate::funcs::random::splitmix32(&mut x),
            crate::funcs::random::splitmix32(&mut x),
            crate::funcs::random::splitmix32(&mut x),
            crate::funcs::random::splitmix32(&mut x),
        ];
        *guard = Some(s);
        s
    }
}

/// Write back the xoshiro128** state to the global slot.
unsafe fn set_global_rand_state(state: [u32; 4]) {
    let mut guard = GLOBAL_RAND_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(state);
}

#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at_p3(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE_P3)
        .cast::<c_void>()
}

/// "rand([{expr}])" function - return a pseudo-random number.
///
/// With no argument uses a global xoshiro128** state.
/// With a list argument (4 numbers), uses that list as the state and updates it in place.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_rand"]
pub unsafe extern "C" fn rs_f_rand(argvars: *const c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    use crate::funcs::random::shuffle_xoshiro128starstar;

    p3_nvim_tv_set_number(rettv, -1);

    let arg0 = arg_at_p3(argvars, 0);
    let arg0_type = p3_nvim_tv_get_type(arg0);

    let result;
    if arg0_type == VAR_UNKNOWN_P3 {
        // No argument: use/update global state
        let [mut gx, mut gy, mut gz, mut gw] = global_rand_state();
        result = shuffle_xoshiro128starstar(&mut gx, &mut gy, &mut gz, &mut gw);
        set_global_rand_state([gx, gy, gz, gw]);
    } else if arg0_type == VAR_LIST_P3 {
        let list = p3_nvim_tv_get_list(arg0);
        if list.is_null() || p3_nvim_list_get_len(list) != 4 {
            let mut len = 0usize;
            let s = p3_nvim_tv_get_string(arg0, &raw mut len);
            let _ = p3_semsg(&raw const P3_E_INVARG2, s);
            return;
        }
        let list_mut = list.cast_mut();
        // Get mutable item typvals (inlined TV_LIST_ITEM_TV: li_tv at offset 16)
        let itv0 = crate::typval::list_item_tv(tv_list_find(list_mut, 0)).cast::<c_void>();
        let itv1 = crate::typval::list_item_tv(tv_list_find(list_mut, 1)).cast::<c_void>();
        let itv2 = crate::typval::list_item_tv(tv_list_find(list_mut, 2)).cast::<c_void>();
        let itv3 = crate::typval::list_item_tv(tv_list_find(list_mut, 3)).cast::<c_void>();

        // All must be VAR_NUMBER
        if p3_nvim_tv_get_type(itv0.cast_const()) != VAR_NUMBER_P3
            || p3_nvim_tv_get_type(itv1.cast_const()) != VAR_NUMBER_P3
            || p3_nvim_tv_get_type(itv2.cast_const()) != VAR_NUMBER_P3
            || p3_nvim_tv_get_type(itv3.cast_const()) != VAR_NUMBER_P3
        {
            let mut len = 0usize;
            let s = p3_nvim_tv_get_string(arg0, &raw mut len);
            let _ = p3_semsg(&raw const P3_E_INVARG2, s);
            return;
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut gx = p3_nvim_tv_get_number(itv0.cast_const()) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut gy = p3_nvim_tv_get_number(itv1.cast_const()) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut gz = p3_nvim_tv_get_number(itv2.cast_const()) as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut gw = p3_nvim_tv_get_number(itv3.cast_const()) as u32;

        result = shuffle_xoshiro128starstar(&mut gx, &mut gy, &mut gz, &mut gw);

        // Write back updated state into the list
        #[allow(clippy::cast_lossless)]
        p3_nvim_tv_set_number(itv0, gx as i64);
        #[allow(clippy::cast_lossless)]
        p3_nvim_tv_set_number(itv1, gy as i64);
        #[allow(clippy::cast_lossless)]
        p3_nvim_tv_set_number(itv2, gz as i64);
        #[allow(clippy::cast_lossless)]
        p3_nvim_tv_set_number(itv3, gw as i64);
    } else {
        let mut len = 0usize;
        let s = p3_nvim_tv_get_string(arg0, &raw mut len);
        let _ = p3_semsg(&raw const P3_E_INVARG2, s);
        return;
    }

    #[allow(clippy::cast_lossless)]
    p3_nvim_tv_set_number(rettv, result as i64);
}

/// "srand([{expr}])" function - initialize random seed, returns 4-element list.
///
/// With no argument uses `uv_random` (or `os_hrtime ^ os_get_pid`).
/// With a number argument uses it as the seed.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_srand"]
pub unsafe extern "C" fn rs_f_srand(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    use crate::funcs::random::splitmix32;

    let list = nvim_tv_list_alloc_ret(rettv, 4);

    let arg0 = arg_at_p3(argvars, 0);
    let arg0_type = p3_nvim_tv_get_type(arg0);

    let mut x = if arg0_type == VAR_UNKNOWN_P3 {
        init_srand()
    } else {
        let mut error = false;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let v = p3_nvim_tv_get_number_chk(arg0, &raw mut error) as u32;
        if error {
            return;
        }
        v
    };

    tv_list_append_number(list, i64::from(splitmix32(&mut x)));
    tv_list_append_number(list, i64::from(splitmix32(&mut x)));
    tv_list_append_number(list, i64::from(splitmix32(&mut x)));
    tv_list_append_number(list, i64::from(splitmix32(&mut x)));
}

/// "reltime([{start}[, {end}]])" function.
///
/// Returns a list `[high, low]` encoding a proftime_T timestamp:
/// - 0 args: current time
/// - 1 arg: elapsed time since `{start}`
/// - 2 args: difference `{end} - {start}`
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_reltime"]
pub unsafe extern "C" fn rs_f_reltime(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let arg0 = arg_at_p3(argvars, 0);
    let arg1 = arg_at_p3(argvars, 1);

    let res: u64 = if p3_nvim_tv_get_type(arg0) == VAR_UNKNOWN_P3 {
        // no arguments: current time
        profile_start()
    } else if p3_nvim_tv_get_type(arg1) == VAR_UNKNOWN_P3 {
        // one argument: elapsed since start
        match list2proftime(arg0) {
            Some(start) => profile_end(start),
            None => return,
        }
    } else {
        // two arguments: end - start
        let Some(start) = list2proftime(arg0) else {
            return;
        };
        let Some(end) = list2proftime(arg1) else {
            return;
        };
        c_profile_sub(end, start)
    };

    // Encode proftime_T (u64) as [high, low] list of i32 values stored as i64.
    // This mirrors the C union { struct { int32_t low, high; } split; proftime_T prof; }.
    // Note: the struct has `low` first (lower address), then `high`.
    // On little-endian: bytes 0..3 = low, bytes 4..7 = high.
    let bytes = res.to_ne_bytes();
    #[allow(clippy::cast_possible_truncation)]
    let low = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    #[allow(clippy::cast_possible_truncation)]
    let high = i32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

    let list = nvim_tv_list_alloc_ret(rettv, 2);
    tv_list_append_number(list, i64::from(high));
    tv_list_append_number(list, i64::from(low));
}

// =============================================================================
// Phase 4: Server, channel, and confirm functions (plan 40f0fb72)
// =============================================================================

const TYPVAL_SIZE_P4: usize = 16;
const VAR_UNKNOWN_P4: c_int = 0;
const VAR_STRING_P4: c_int = 2;
const VAR_NUMBER_P4: c_int = 1;

// ChannelPart enum values from channel_defs.h
const K_CHANNEL_PART_STDIN: c_int = 0;
const K_CHANNEL_PART_STDOUT: c_int = 1;
const K_CHANNEL_PART_STDERR: c_int = 2;
const K_CHANNEL_PART_RPC: c_int = 3;
const K_CHANNEL_PART_ALL: c_int = 4;

// VIM dialog type constants from message.h
const VIM_GENERIC: c_int = 0;
const VIM_ERROR: c_int = 1;
const VIM_WARNING: c_int = 2;
const VIM_INFO: c_int = 3;
const VIM_QUESTION: c_int = 4;

extern "C" {
    // Type/value accessors for Phase 4
    #[link_name = "nvim_tv_get_type"]
    fn p4_nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_number"]
    fn p4_nvim_tv_get_number(tv: *const c_void) -> i64;
    #[link_name = "nvim_tv_get_number_chk"]
    fn p4_nvim_tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;
    #[link_name = "nvim_tv_set_number"]
    fn p4_nvim_tv_set_number(tv: *mut c_void, n: i64);
    #[link_name = "nvim_tv_get_string_ptr"]
    fn p4_nvim_tv_get_string_ptr(tv: *const c_void) -> *const u8;
    #[link_name = "nvim_tv_get_string"]
    fn p4_nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    #[link_name = "nvim_tv_set_string_copy"]
    fn p4_nvim_tv_set_string_copy(tv: *mut c_void, s: *const u8, len: c_int);

    // rs_check_secure - already exported from Rust (check if sandbox mode)
    #[link_name = "rs_check_secure"]
    fn p4_rs_check_secure() -> c_int;

    // Error functions
    #[link_name = "emsg"]
    fn p4_emsg(msg: *const c_char) -> c_int;
    #[link_name = "semsg"]
    fn p4_semsg(fmt: *const c_char, ...) -> c_int;
    #[link_name = "e_invarg"]
    static P4_E_INVARG: c_char;

    // channel_close(id, part, &error) -> bool
    fn channel_close(id: u64, part: c_int, error: *mut *const c_char) -> bool;

    // server functions
    fn server_address_new(name: *const c_char) -> *mut c_char;
    fn server_start(addr: *const c_char) -> c_int;
    fn server_address_list(size: *mut usize) -> *mut *mut c_char;

    // libuv error string (for server_start failures)
    fn uv_strerror(err: c_int) -> *const c_char;

    // do_dialog for f_confirm
    fn do_dialog(
        dialtype: c_int,
        title: *const c_char,
        message: *const c_char,
        buttons: *const c_char,
        dflt: c_int,
        textfield: *const c_char,
        ex_cmd: c_int,
    ) -> c_int;

    // tv_get_string_chk: returns NULL-terminated string or NULL on error
    #[link_name = "tv_get_string_chk"]
    fn p4_tv_get_string_chk(tv: *mut c_void) -> *const c_char;

    // Memory management
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at_p4(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE_P4)
        .cast::<c_void>()
}

/// "chanclose(id[, stream])" function - close a channel or stream.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_chanclose"]
pub unsafe extern "C" fn rs_f_chanclose(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    p4_nvim_tv_set_number(rettv, 0);

    if p4_rs_check_secure() != 0 {
        return;
    }

    let arg0 = arg_at_p4(argvars, 0);
    let arg1 = arg_at_p4(argvars, 1);
    let arg0_type = p4_nvim_tv_get_type(arg0);
    let arg1_type = p4_nvim_tv_get_type(arg1);

    if arg0_type != VAR_NUMBER_P4 || (arg1_type != VAR_STRING_P4 && arg1_type != VAR_UNKNOWN_P4) {
        let _ = p4_emsg(&raw const P4_E_INVARG);
        return;
    }

    let part = if arg1_type == VAR_STRING_P4 {
        let stream = p4_nvim_tv_get_string_ptr(arg1);
        if stream.is_null() {
            let _ = p4_emsg(&raw const P4_E_INVARG);
            return;
        }
        // Compare the stream name to known values
        let s = std::ffi::CStr::from_ptr(stream.cast::<c_char>());
        match s.to_bytes() {
            b"stdin" => K_CHANNEL_PART_STDIN,
            b"stdout" => K_CHANNEL_PART_STDOUT,
            b"stderr" => K_CHANNEL_PART_STDERR,
            b"rpc" => K_CHANNEL_PART_RPC,
            _ => {
                let _ = p4_semsg(
                    c"Invalid channel stream \"%s\"".as_ptr(),
                    stream.cast::<c_char>(),
                );
                return;
            }
        }
    } else {
        K_CHANNEL_PART_ALL
    };

    #[allow(clippy::cast_sign_loss)]
    let id = p4_nvim_tv_get_number(arg0) as u64;
    let mut error_ptr: *const c_char = std::ptr::null();
    let ok = channel_close(id, part, &raw mut error_ptr);
    p4_nvim_tv_set_number(rettv, i64::from(ok));
    if !ok {
        let _ = p4_emsg(error_ptr);
    }
}

/// "serverstart([{address}])" function - start a server at given address.
///
/// Returns the server address, or an empty string on failure.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_serverstart"]
pub unsafe extern "C" fn rs_f_serverstart(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Default: empty string return
    p4_nvim_tv_set_string_copy(rettv, std::ptr::null(), 0);

    if p4_rs_check_secure() != 0 {
        return;
    }

    let arg0 = arg_at_p4(argvars, 0);
    let arg0_type = p4_nvim_tv_get_type(arg0);

    // Get or generate the address
    let address: *mut c_char = if arg0_type == VAR_UNKNOWN_P4 {
        server_address_new(std::ptr::null())
    } else {
        if arg0_type != VAR_STRING_P4 {
            let _ = p4_emsg(&raw const P4_E_INVARG);
            return;
        }
        let mut len = 0usize;
        let s = p4_nvim_tv_get_string(arg0, &raw mut len);
        xstrdup(s.cast::<c_char>())
    };

    let result = server_start(address);
    xfree(address.cast::<c_void>());

    if result != 0 {
        if result > 0 {
            let _ = p4_semsg(
                c"Failed to start server: %s".as_ptr(),
                c"Unknown system error".as_ptr(),
            );
        } else {
            let _ = p4_semsg(c"Failed to start server: %s".as_ptr(), uv_strerror(result));
        }
        return;
    }

    // Return the last address from server_address_list (the newly started server).
    let mut n: usize = 0;
    let addrs = server_address_list(&raw mut n);
    if addrs.is_null() || n == 0 {
        return;
    }

    // The last entry is the newly started server
    let last = *addrs.add(n - 1);
    // Copy the address into rettv then free all entries
    p4_nvim_tv_set_string_copy(rettv, last.cast::<u8>(), -1);

    for i in 0..n {
        xfree((*addrs.add(i)).cast::<c_void>());
    }
    xfree(addrs.cast::<c_void>());
}

/// "confirm(msg [, choices [, default [, type]]])" function.
///
/// Displays a dialog and returns the choice (1-based), or 0 on error.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_confirm"]
pub unsafe extern "C" fn rs_f_confirm(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    p4_nvim_tv_set_number(rettv, 0);

    let mut buf = [0u8; 65usize];
    let mut buf2 = [0u8; 65usize];
    let mut error = false;

    let arg0 = arg_at_p4(argvars, 0);
    let arg1 = arg_at_p4(argvars, 1);
    let arg2 = arg_at_p4(argvars, 2);
    let arg3 = arg_at_p4(argvars, 3);

    let message = p4_tv_get_string_chk(arg0.cast_mut());
    if message.is_null() {
        error = true;
    }

    let buttons: *const c_char = if p4_nvim_tv_get_type(arg1) == VAR_UNKNOWN_P4 {
        std::ptr::null()
    } else {
        // tv_get_string_buf_chk is already declared in Phase 2
        let b = tv_get_string_buf_chk(arg1, buf.as_mut_ptr().cast::<c_char>());
        if b.is_null() {
            error = true;
        }
        b
    };

    let mut def: c_int = 1;
    if !error && p4_nvim_tv_get_type(arg2) != VAR_UNKNOWN_P4 {
        #[allow(clippy::cast_possible_truncation)]
        {
            def = p4_nvim_tv_get_number_chk(arg2, &raw mut error) as c_int;
        }
    }

    let mut dialtype: c_int = VIM_GENERIC;
    if !error && p4_nvim_tv_get_type(arg3) != VAR_UNKNOWN_P4 {
        let typestr = tv_get_string_buf_chk(arg3, buf2.as_mut_ptr().cast::<c_char>());
        if typestr.is_null() {
            error = true;
        } else {
            #[allow(clippy::cast_sign_loss)]
            let first = (*typestr as u8).to_ascii_uppercase();
            dialtype = match first {
                b'E' => VIM_ERROR,
                b'Q' => VIM_QUESTION,
                b'I' => VIM_INFO,
                b'W' => VIM_WARNING,
                _ => VIM_GENERIC,
            };
        }
    }

    // Default buttons: "&Ok"
    let effective_buttons: *const c_char =
        if buttons.is_null() || (!buttons.is_null() && *buttons == 0) {
            c"&Ok".as_ptr()
        } else {
            buttons
        };

    if !error {
        let result = do_dialog(
            dialtype,
            std::ptr::null(),
            message,
            effective_buttons,
            def,
            std::ptr::null(),
            0,
        );
        p4_nvim_tv_set_number(rettv, i64::from(result));
    }
}

// =============================================================================
// Phase 5: Time functions f_strftime and f_strptime (plan 40f0fb72)
// =============================================================================

use libc::{mktime, strftime, time_t, tm};

const TYPVAL_SIZE_P5: usize = 16;
const VAR_UNKNOWN_P5: c_int = 0;
const CONV_NONE_P5: c_int = 0;

/// `vimconv_T` from `mbyte_defs.h` - struct for encoding conversion.
///
/// Layout (on Linux x86-64):
/// - `vc_type:   int`   (4 bytes offset 0)
/// - `vc_factor: int`   (4 bytes offset 4)
/// - `vc_fd:     void*` (8 bytes offset 8, iconv_t = void* on Linux)
/// - `vc_fail:   bool`  (1 byte  offset 16)
///
/// Total: 24 bytes (with 7 bytes padding at end)
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct VimConv {
    vc_type: c_int,
    vc_factor: c_int,
    vc_fd: *mut c_void,
    vc_fail: bool,
}

impl VimConv {
    const fn none() -> Self {
        Self {
            vc_type: 0, // CONV_NONE
            vc_factor: 1,
            vc_fd: std::ptr::null_mut(),
            vc_fail: false,
        }
    }
}

extern "C" {
    // Encoding
    fn enc_locale() -> *mut c_char;
    fn convert_setup(vcp: *mut VimConv, from: *mut c_char, to: *mut c_char) -> c_int;
    fn string_convert(vcp: *const VimConv, ptr: *mut c_char, lenp: *mut usize) -> *mut c_char;
    static p_enc: *mut c_char;

    // OS time functions
    fn os_localtime_r(clock: *const time_t, result: *mut tm) -> *mut tm;
    fn os_strptime(str_: *const c_char, format: *const c_char, tm: *mut tm) -> *const c_char;

    // Type accessors for Phase 5
    #[link_name = "nvim_tv_get_type"]
    fn p5_nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_number"]
    fn p5_nvim_tv_get_number(tv: *const c_void) -> i64;
    #[link_name = "nvim_tv_get_string"]
    fn p5_nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    #[link_name = "nvim_tv_set_string"]
    fn p5_nvim_tv_set_string(tv: *mut c_void, s: *mut u8);
    #[link_name = "nvim_tv_set_number"]
    fn p5_nvim_tv_set_number(tv: *mut c_void, n: i64);
    #[link_name = "xstrdup"]
    fn p5_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn p5_xfree(ptr: *mut c_void);
}

#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at_p5(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE_P5)
        .cast::<c_void>()
}

/// "strftime({fmt} [, {time}])" function - format time as string.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_strftime"]
pub unsafe extern "C" fn rs_f_strftime(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let arg0 = arg_at_p5(argvars, 0);
    let arg1 = arg_at_p5(argvars, 1);

    // Get format string
    let mut fmt_len = 0usize;
    let fmt_raw = p5_nvim_tv_get_string(arg0, &raw mut fmt_len);

    // Get time value (or use current time)
    let seconds: time_t = if p5_nvim_tv_get_type(arg1) == VAR_UNKNOWN_P5 {
        libc::time(std::ptr::null_mut())
    } else {
        p5_nvim_tv_get_number(arg1)
    };

    // Get local time
    let mut curtime = std::mem::zeroed::<tm>();
    let curtime_ptr = os_localtime_r(&raw const seconds, &raw mut curtime);
    if curtime_ptr.is_null() {
        // Invalid time
        p5_nvim_tv_set_string(rettv, p5_xstrdup(c"(Invalid)".as_ptr()).cast::<u8>());
        return;
    }

    // Set up encoding conversion (fmt may need to be converted to locale encoding)
    let mut conv = VimConv::none();
    let enc = enc_locale();
    convert_setup(&raw mut conv, p_enc, enc);

    // Possibly convert the format string to locale encoding
    let fmt_ptr: *mut c_char = if conv.vc_type == CONV_NONE_P5 {
        fmt_raw.cast::<c_char>().cast_mut()
    } else {
        let converted = string_convert(
            &raw const conv,
            fmt_raw.cast::<c_char>().cast_mut(),
            std::ptr::null_mut(),
        );
        if converted.is_null() {
            convert_setup(&raw mut conv, std::ptr::null_mut(), std::ptr::null_mut());
            p5_xfree(enc.cast::<c_void>());
            return;
        }
        converted
    };

    // Format the time
    let mut result_buf = [0u8; 256];
    let n = strftime(
        result_buf.as_mut_ptr().cast::<c_char>(),
        result_buf.len(),
        fmt_ptr,
        curtime_ptr,
    );
    if n == 0 {
        result_buf[0] = 0;
    }

    // Free converted format string
    if conv.vc_type != CONV_NONE_P5 {
        p5_xfree(fmt_ptr.cast::<c_void>());
    }

    // Convert result back to p_enc if needed
    convert_setup(&raw mut conv, enc, p_enc);
    if conv.vc_type == CONV_NONE_P5 {
        p5_nvim_tv_set_string(
            rettv,
            p5_xstrdup(result_buf.as_ptr().cast::<c_char>()).cast::<u8>(),
        );
    } else {
        let s = string_convert(
            &raw const conv,
            result_buf.as_mut_ptr().cast::<c_char>(),
            std::ptr::null_mut(),
        );
        p5_nvim_tv_set_string(rettv, s.cast::<u8>());
    }

    // Release conversion descriptors and locale string
    convert_setup(&raw mut conv, std::ptr::null_mut(), std::ptr::null_mut());
    p5_xfree(enc.cast::<c_void>());
}

/// "strptime({format}, {timestring})" function - parse time string.
///
/// Returns a Unix timestamp (seconds since epoch), or 0 on error.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_strptime"]
pub unsafe extern "C" fn rs_f_strptime(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    p5_nvim_tv_set_number(rettv, 0);

    let arg0 = arg_at_p5(argvars, 0);
    let arg1 = arg_at_p5(argvars, 1);

    let mut fmt_len = 0usize;
    let fmt_raw = p5_nvim_tv_get_string(arg0, &raw mut fmt_len);
    let mut str_len = 0usize;
    let str_raw = p5_nvim_tv_get_string(arg1, &raw mut str_len);

    // Set up encoding conversion
    let mut conv = VimConv::none();
    let enc = enc_locale();
    convert_setup(&raw mut conv, p_enc, enc);

    // Possibly convert format string to locale encoding
    let fmt_ptr: *mut c_char = if conv.vc_type == CONV_NONE_P5 {
        fmt_raw.cast::<c_char>().cast_mut()
    } else {
        string_convert(
            &raw const conv,
            fmt_raw.cast::<c_char>().cast_mut(),
            std::ptr::null_mut(),
        )
    };

    if !fmt_ptr.is_null() {
        let mut tmval = std::mem::zeroed::<tm>();
        tmval.tm_isdst = -1;

        let rest = os_strptime(str_raw.cast::<c_char>(), fmt_ptr, &raw mut tmval);
        if !rest.is_null() {
            let t = mktime(&raw mut tmval);
            if t != -1 {
                p5_nvim_tv_set_number(rettv, t);
            }
        }
    }

    if conv.vc_type != CONV_NONE_P5 {
        p5_xfree(fmt_ptr.cast::<c_void>());
    }
    convert_setup(&raw mut conv, std::ptr::null_mut(), std::ptr::null_mut());
    p5_xfree(enc.cast::<c_void>());
}

// =============================================================================
// Phase 6: Dict watcher functions (plan 40f0fb72)
// =============================================================================

use crate::callback::CallbackT;

const TYPVAL_SIZE_P6: usize = 16;
const VAR_DICT_P6: c_int = 5;
const VAR_FUNC_P6: c_int = 3;
const VAR_NUMBER_P6: c_int = 1;
const VAR_STRING_P6: c_int = 2;

extern "C" {
    // Type/value accessors for Phase 6
    #[link_name = "nvim_tv_get_type"]
    fn p6_nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "nvim_tv_get_dict"]
    fn p6_nvim_tv_get_dict(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_tv_dict_is_null"]
    fn p6_nvim_tv_dict_is_null(tv: *const c_void) -> c_int;

    // rs_check_secure
    #[link_name = "rs_check_secure"]
    fn p6_rs_check_secure() -> c_int;

    // Error helpers
    #[link_name = "semsg"]
    fn p6_semsg(fmt: *const c_char, ...) -> c_int;
    #[link_name = "emsg"]
    fn p6_emsg(s: *const c_char) -> c_int;
    #[link_name = "e_invarg2"]
    static P6_E_INVARG2: c_char;
    #[link_name = "e_readonlyvar"]
    static P6_E_READONLYVAR: c_char;

    // tv_get_string_chk (via p4's link name alias)
    #[link_name = "tv_get_string_chk"]
    fn p6_tv_get_string_chk(tv: *mut c_void) -> *const c_char;

    // rs_callback_from_typval: Rust-exported, returns bool
    fn rs_callback_from_typval(callback: *mut CallbackT, arg: *const c_void) -> bool;

    // tv_dict_watcher_add/remove: takes dict ptr, key_pattern, len, callback by value
    fn tv_dict_watcher_add(
        dict: *mut c_void,
        key_pattern: *const c_char,
        key_pattern_len: usize,
        callback: CallbackT,
    );
    fn tv_dict_watcher_remove(
        dict: *mut c_void,
        key_pattern: *const c_char,
        key_pattern_len: usize,
        callback: CallbackT,
    ) -> bool;

    // callback_free
    fn callback_free(cb: *mut CallbackT);
}

#[inline]
#[allow(clippy::ptr_as_ptr)]
unsafe fn arg_at_p6(argvars: *const c_void, idx: usize) -> *const c_void {
    (argvars as *const u8)
        .add(idx * TYPVAL_SIZE_P6)
        .cast::<c_void>()
}

/// "dictwatcheradd(dict, key, funcref)" function.
///
/// Registers a callback to be called when a dict key matching `key` changes.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_dictwatcheradd"]
pub unsafe extern "C" fn rs_f_dictwatcheradd(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if p6_rs_check_secure() != 0 {
        return;
    }

    let arg0 = arg_at_p6(argvars, 0);
    let arg1 = arg_at_p6(argvars, 1);
    let arg2 = arg_at_p6(argvars, 2);

    if p6_nvim_tv_get_type(arg0) != VAR_DICT_P6 {
        let _ = p6_semsg(&raw const P6_E_INVARG2, c"dict".as_ptr());
        return;
    }

    // Check for null dict (read-only)
    if p6_nvim_tv_dict_is_null(arg0) != 0 {
        let msg = c"dictwatcheradd() argument";
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let _ = p6_semsg(
            &raw const P6_E_READONLYVAR,
            msg.count_bytes() as c_int,
            msg.as_ptr(),
        );
        return;
    }

    let arg1_type = p6_nvim_tv_get_type(arg1);
    if arg1_type != VAR_STRING_P6 && arg1_type != VAR_NUMBER_P6 {
        let _ = p6_semsg(&raw const P6_E_INVARG2, c"key".as_ptr());
        return;
    }

    let key_pattern = p6_tv_get_string_chk(arg1.cast_mut());
    if key_pattern.is_null() {
        return;
    }
    let key_pattern_len = std::ffi::CStr::from_ptr(key_pattern).to_bytes().len();

    let mut callback = std::mem::zeroed::<CallbackT>();
    if !rs_callback_from_typval(&raw mut callback, arg2) {
        let _ = p6_semsg(&raw const P6_E_INVARG2, c"funcref".as_ptr());
        return;
    }

    let dict = p6_nvim_tv_get_dict(arg0).cast_mut();
    tv_dict_watcher_add(dict, key_pattern, key_pattern_len, callback);
}

/// "dictwatcherdel(dict, key, funcref)" function.
///
/// Removes a previously registered dict watcher callback.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_dictwatcherdel"]
pub unsafe extern "C" fn rs_f_dictwatcherdel(
    argvars: *const c_void,
    _rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if p6_rs_check_secure() != 0 {
        return;
    }

    let arg0 = arg_at_p6(argvars, 0);
    let arg1 = arg_at_p6(argvars, 1);
    let arg2 = arg_at_p6(argvars, 2);

    if p6_nvim_tv_get_type(arg0) != VAR_DICT_P6 {
        let _ = p6_semsg(&raw const P6_E_INVARG2, c"dict".as_ptr());
        return;
    }

    let arg2_type = p6_nvim_tv_get_type(arg2);
    if arg2_type != VAR_FUNC_P6 && arg2_type != VAR_STRING_P6 {
        let _ = p6_semsg(&raw const P6_E_INVARG2, c"funcref".as_ptr());
        return;
    }

    let key_pattern = p6_tv_get_string_chk(arg1.cast_mut());
    if key_pattern.is_null() {
        return;
    }
    let key_pattern_len = std::ffi::CStr::from_ptr(key_pattern).to_bytes().len();

    let mut callback = std::mem::zeroed::<CallbackT>();
    if !rs_callback_from_typval(&raw mut callback, arg2) {
        return;
    }

    let dict = p6_nvim_tv_get_dict(arg0).cast_mut();
    // `tv_dict_watcher_remove` takes Callback by value (C copy semantics).
    // We need a copy to keep `callback` alive for `callback_free` afterward.
    let callback_copy = std::ptr::read(&raw const callback);
    if !tv_dict_watcher_remove(dict, key_pattern, key_pattern_len, callback_copy) {
        let _ = p6_emsg(c"Couldn't find a watcher matching key and callback".as_ptr());
    }

    callback_free(&raw mut callback);
}
