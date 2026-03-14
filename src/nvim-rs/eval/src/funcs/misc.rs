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

// =============================================================================
// Phase 3: Medium-complexity functions (all delegated to C accessors)
// =============================================================================

extern "C" {
    fn nvim_eval_char2nr(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_nr2char(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_str2float(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_escape(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_shellescape(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_fnameescape(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_hostname(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_empty(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_copy(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_deepcopy(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_len(argvars: *const c_void, rettv: *mut c_void);
    fn nvim_eval_ctx_size() -> c_int;
    fn nvim_eval_ctxpop();
    fn nvim_eval_max_min(argvars: *const c_void, rettv: *mut c_void, domax: bool);
    fn nvim_eval_set_position(argvars: *const c_void, rettv: *mut c_void, charpos: bool);
    fn nvim_eval_set_cursorpos(argvars: *const c_void, rettv: *mut c_void, charcol: bool);
    fn nvim_eval_searchpair_cmn(argvars: *const c_void) -> c_int;
    fn nvim_eval_find_some_match(argvars: *const c_void, rettv: *mut c_void, kind: c_int);
}

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
    nvim_eval_char2nr(argvars, rettv);
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
    nvim_eval_escape(argvars, rettv);
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
    nvim_eval_shellescape(argvars, rettv);
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
    nvim_eval_fnameescape(argvars, rettv);
}

/// "hostname()" function - get the hostname
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_hostname"]
pub unsafe extern "C" fn rs_f_hostname(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    nvim_eval_hostname(argvars, rettv);
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
    nvim_eval_empty(argvars, rettv);
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
    nvim_eval_len(argvars, rettv);
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
    rettv_set_number(rettv, i64::from(nvim_eval_ctx_size()));
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
    nvim_eval_ctxpop();
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
