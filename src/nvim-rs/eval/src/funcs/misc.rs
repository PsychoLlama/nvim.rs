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
    fn nvim_get_vgetc_busy() -> c_int;
    fn nvim_curbuf_get_did_filetype() -> c_int;
    fn nvim_curbuf_get_u_seq_cur() -> c_int;
    fn nvim_set_got_int(v: c_int);
    fn nvim_get_reg_executing() -> c_int;
    fn nvim_get_reg_recording() -> c_int;
    fn nvim_get_reg_recorded() -> c_int;
    fn nvim_eval_ui_current_col() -> c_int;
    fn nvim_eval_ui_current_row() -> c_int;
    fn nvim_eval_pum_visible() -> c_int;
    fn nvim_eval_os_get_pid() -> c_int;
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
    rettv_set_number(rettv, i64::from(nvim_eval_os_get_pid()));
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
    rettv_set_number(rettv, i64::from(nvim_eval_ui_current_col()) + 1);
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
    rettv_set_number(rettv, i64::from(nvim_eval_ui_current_row()) + 1);
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
    nvim_set_got_int(1);
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
    rettv_set_number(rettv, i64::from(nvim_eval_pum_visible()));
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
