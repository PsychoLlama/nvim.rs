//! Simple self-contained VimL built-in functions (Phase 1).
//!
//! These are trivially self-contained functions migrated from `src/nvim/eval/funcs.c`.
//! Each delegates to a thin C accessor.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_void;

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
