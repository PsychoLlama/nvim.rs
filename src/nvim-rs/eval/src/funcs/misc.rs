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
    #[link_name = "nvim_tv_list_find_nr"]
    fn p2_nvim_tv_list_find_nr(l: *mut c_void, n: c_int, error_out: *mut bool) -> i64;
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
    fn nvim_strings_get_e_invarg2() -> *const c_char;
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
    let n1 = p2_nvim_tv_list_find_nr(list_mut, 0, &raw mut error);
    let n2 = p2_nvim_tv_list_find_nr(list_mut, 1, &raw mut error);
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
        let _ = semsg(nvim_strings_get_e_invarg2(), mode_str);
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
    // nvim_tv_list_alloc_ret: DLLEXPORT wrapper, returns list_T*
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
    /// Get typval from list item (returns mutable).
    fn nvim_list_item_tv(item: *mut c_void) -> *mut c_void;

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
        // Get mutable item typvals
        let itv0 = nvim_list_item_tv(tv_list_find(list_mut, 0));
        let itv1 = nvim_list_item_tv(tv_list_find(list_mut, 1));
        let itv2 = nvim_list_item_tv(tv_list_find(list_mut, 2));
        let itv3 = nvim_list_item_tv(tv_list_find(list_mut, 3));

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
