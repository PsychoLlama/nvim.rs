//! Lua executor helper functions
//!
//! This module provides Rust implementations of helper functions used in
//! Lua script execution and callback handling.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// LuaRetMode enum values (from executor.h)
// =============================================================================

/// kRetObject = 0
const RET_OBJECT: c_int = 0;
/// kRetNilBool = 1
const RET_NIL_BOOL: c_int = 1;
/// kRetLuaref = 2
const RET_LUAREF: c_int = 2;
/// kRetMulti = 3
const RET_MULTI: c_int = 3;

/// LUA_MULTRET constant (from lua.h)
const LUA_MULTRET: c_int = -1;

/// LUA_NOREF constant
const LUA_NOREF: c_int = -2;

/// LUA_REFNIL constant
const LUA_REFNIL: c_int = -1;

// =============================================================================
// luv_err_t enum values
// =============================================================================

/// kCallback error type
const LUV_ERR_CALLBACK: c_int = 0;
/// kThread error type
const LUV_ERR_THREAD: c_int = 1;
/// kThreadCallback error type
const LUV_ERR_THREAD_CALLBACK: c_int = 2;

// =============================================================================
// Validation helpers
// =============================================================================

/// Validate timeout for vim.wait (must be >= 0)
#[no_mangle]
pub extern "C" fn rs_wait_timeout_valid(timeout: i64) -> bool {
    timeout >= 0
}

/// Validate interval for vim.wait (must be >= 0)
#[no_mangle]
pub extern "C" fn rs_wait_interval_valid(interval: i64) -> bool {
    interval >= 0
}

/// Get default interval for vim.wait (200ms)
#[no_mangle]
pub extern "C" fn rs_wait_default_interval() -> i64 {
    200
}

/// Check if reference is valid (not NOREF)
#[no_mangle]
pub extern "C" fn rs_lua_ref_valid(ref_: c_int) -> bool {
    ref_ > 0
}

/// Check if reference is NOREF
#[no_mangle]
pub extern "C" fn rs_lua_is_noref(ref_: c_int) -> bool {
    ref_ == LUA_NOREF
}

/// Check if reference is REFNIL
#[no_mangle]
pub extern "C" fn rs_lua_is_refnil(ref_: c_int) -> bool {
    ref_ == LUA_REFNIL
}

// =============================================================================
// Mode helpers
// =============================================================================

/// Calculate mode_ret() - returns LUA_MULTRET for kRetMulti, 1 otherwise
#[no_mangle]
pub extern "C" fn rs_lua_mode_ret(mode: c_int) -> c_int {
    if mode == RET_MULTI {
        LUA_MULTRET
    } else {
        1
    }
}

/// Check if mode is kRetMulti
#[no_mangle]
pub extern "C" fn rs_lua_is_multi_mode(mode: c_int) -> bool {
    mode == RET_MULTI
}

/// Check if mode is kRetNilBool
#[no_mangle]
pub extern "C" fn rs_lua_is_nil_bool_mode(mode: c_int) -> bool {
    mode == RET_NIL_BOOL
}

/// Check if mode is kRetLuaref
#[no_mangle]
pub extern "C" fn rs_lua_is_luaref_mode(mode: c_int) -> bool {
    mode == RET_LUAREF
}

/// Check if mode is kRetObject
#[no_mangle]
pub extern "C" fn rs_lua_is_object_mode(mode: c_int) -> bool {
    mode == RET_OBJECT
}

/// Get kRetObject constant
#[no_mangle]
pub extern "C" fn rs_lua_ret_object() -> c_int {
    RET_OBJECT
}

/// Get kRetNilBool constant
#[no_mangle]
pub extern "C" fn rs_lua_ret_nil_bool() -> c_int {
    RET_NIL_BOOL
}

/// Get kRetLuaref constant
#[no_mangle]
pub extern "C" fn rs_lua_ret_luaref() -> c_int {
    RET_LUAREF
}

/// Get kRetMulti constant
#[no_mangle]
pub extern "C" fn rs_lua_ret_multi() -> c_int {
    RET_MULTI
}

// =============================================================================
// luv_err_t helpers
// =============================================================================

/// Get kCallback constant
#[no_mangle]
pub extern "C" fn rs_luv_err_callback() -> c_int {
    LUV_ERR_CALLBACK
}

/// Get kThread constant
#[no_mangle]
pub extern "C" fn rs_luv_err_thread() -> c_int {
    LUV_ERR_THREAD
}

/// Get kThreadCallback constant
#[no_mangle]
pub extern "C" fn rs_luv_err_thread_callback() -> c_int {
    LUV_ERR_THREAD_CALLBACK
}

/// Check if error type is kCallback
#[no_mangle]
pub extern "C" fn rs_luv_is_callback_err(err_type: c_int) -> bool {
    err_type == LUV_ERR_CALLBACK
}

/// Check if error type is kThread
#[no_mangle]
pub extern "C" fn rs_luv_is_thread_err(err_type: c_int) -> bool {
    err_type == LUV_ERR_THREAD
}

/// Check if error type is kThreadCallback
#[no_mangle]
pub extern "C" fn rs_luv_is_thread_callback_err(err_type: c_int) -> bool {
    err_type == LUV_ERR_THREAD_CALLBACK
}

// =============================================================================
// String/buffer size calculations
// =============================================================================

/// Calculate luaeval header size: sizeof("local _A=select(1,...) return (") - 1
#[no_mangle]
pub extern "C" fn rs_lua_eval_header_size() -> usize {
    // "local _A=select(1,...) return ("
    31
}

/// Calculate luaeval total command size
#[no_mangle]
pub extern "C" fn rs_lua_eval_cmd_size(str_size: usize) -> usize {
    // header + str + ")"
    31 + str_size + 1
}

/// Calculate luado header size: sizeof("return function(line, linenr) ") - 1
#[no_mangle]
pub extern "C" fn rs_lua_do_header_size() -> usize {
    // "return function(line, linenr) "
    30
}

/// Calculate luado end size: sizeof(" end") - 1
#[no_mangle]
pub extern "C" fn rs_lua_do_end_size() -> usize {
    // " end"
    4
}

/// Calculate luado total command size
#[no_mangle]
pub extern "C" fn rs_lua_do_cmd_size(cmd_size: usize) -> usize {
    // header + cmd + end
    30 + cmd_size + 4
}

/// Calculate lua call header size: sizeof("return ") - 1
#[no_mangle]
pub extern "C" fn rs_lua_call_header_size() -> usize {
    // "return "
    7
}

/// Calculate lua call suffix size: sizeof("(...)") - 1
#[no_mangle]
pub extern "C" fn rs_lua_call_suffix_size() -> usize {
    // "(...)"
    5
}

/// Calculate lua call total command size
#[no_mangle]
pub extern "C" fn rs_lua_call_cmd_size(str_size: usize) -> usize {
    // header + str + suffix
    7 + str_size + 5
}

// =============================================================================
// Stack position calculations
// =============================================================================

/// Calculate position for lua_insert in nlua_pcall: -2 - nargs
#[no_mangle]
pub extern "C" fn rs_lua_pcall_insert_pos(nargs: c_int) -> c_int {
    -2 - nargs
}

/// Calculate position for lua_remove after successful nlua_pcall: -1 - nresults
#[no_mangle]
pub extern "C" fn rs_lua_pcall_remove_pos(nresults: c_int) -> c_int {
    -1 - nresults
}

/// Calculate nresults when LUA_MULTRET: lua_gettop(lstate) - (pre_top - nargs - 1)
#[no_mangle]
pub extern "C" fn rs_lua_calc_multret_nresults(top: c_int, pre_top: c_int, nargs: c_int) -> c_int {
    top - (pre_top - nargs - 1)
}

/// Calculate nresult for fast callback when LUA_MULTRET
#[no_mangle]
pub extern "C" fn rs_lua_fast_cb_nresult(top: c_int, pre_top: c_int, nargs: c_int) -> c_int {
    top - pre_top + nargs + 1
}

// =============================================================================
// nargs calculation helpers (for nlua_do_ucmd)
// =============================================================================

/// Calculate nargs character based on command arguments
///
/// Returns: '0' for no args, '1' for exactly one, '?' for optional,
///          '+' for one or more, '*' for any number
#[no_mangle]
pub extern "C" fn rs_ucmd_nargs_char(
    has_extra: bool,
    has_nospc: bool,
    has_needarg: bool,
) -> c_char {
    if has_extra {
        if has_nospc {
            if has_needarg {
                b'1' as c_char
            } else {
                b'?' as c_char
            }
        } else if has_needarg {
            b'+' as c_char
        } else {
            b'*' as c_char
        }
    } else {
        b'0' as c_char
    }
}

/// Check if fargs should contain args for single-arg commands
#[no_mangle]
pub extern "C" fn rs_ucmd_fargs_single(has_nospc: bool, has_needarg: bool, arg_len: usize) -> bool {
    has_nospc && (has_needarg || arg_len > 0)
}

/// Check if fargs should be empty for optional-arg commands
#[no_mangle]
pub extern "C" fn rs_ucmd_fargs_empty(has_nospc: bool, has_needarg: bool, arg_len: usize) -> bool {
    has_nospc && !has_needarg && arg_len == 0
}

// =============================================================================
// Print helpers
// =============================================================================

/// Check if should add space separator in print (curargidx < nargs)
#[no_mangle]
pub extern "C" fn rs_lua_print_needs_space(curargidx: c_int, nargs: c_int) -> bool {
    curargidx < nargs
}

/// Calculate print error argument index (1-based)
#[no_mangle]
pub extern "C" fn rs_lua_print_error_argidx(curargidx: c_int) -> c_int {
    curargidx
}

// =============================================================================
// Wait result helpers
// =============================================================================

/// Calculate wait result nresults when callback_result is true but no results
#[no_mangle]
pub extern "C" fn rs_wait_nresults_empty() -> c_int {
    1
}

/// Calculate wait result return count: nresults + 1
#[no_mangle]
pub extern "C" fn rs_wait_return_count(nresults: c_int) -> c_int {
    nresults + 1
}

/// Get wait timeout error code (-1)
#[no_mangle]
pub extern "C" fn rs_wait_timeout_code() -> c_int {
    -1
}

/// Get wait interrupted error code (-2)
#[no_mangle]
pub extern "C" fn rs_wait_interrupted_code() -> c_int {
    -2
}

/// Check if wait condition returned no results
#[no_mangle]
pub extern "C" fn rs_wait_no_results(nresults: c_int) -> bool {
    nresults == 0
}

// =============================================================================
// Module path helpers
// =============================================================================

/// Check if buffer size is less than IOSIZE (typically used for lcmd allocation)
#[no_mangle]
pub extern "C" fn rs_lcmd_fits_iosize(lcmd_len: usize, iosize: usize) -> bool {
    lcmd_len < iosize
}

/// Calculate module name buffer offset for path conversion
#[no_mangle]
pub extern "C" fn rs_module_name_offset() -> usize {
    1 // '@' prefix
}

/// Calculate module name suffix offset
#[no_mangle]
pub extern "C" fn rs_module_suffix_offset(name_len: usize) -> usize {
    1 + name_len // '@' + name
}

// =============================================================================
// nlua_ref helpers
// =============================================================================

/// Check if ref needs tracking (ref > 0)
#[no_mangle]
pub extern "C" fn rs_lua_ref_needs_tracking(ref_: c_int) -> bool {
    ref_ > 0
}

/// Increment ref count
#[no_mangle]
pub extern "C" fn rs_lua_ref_count_inc(count: c_int) -> c_int {
    count + 1
}

/// Decrement ref count
#[no_mangle]
pub extern "C" fn rs_lua_ref_count_dec(count: c_int) -> c_int {
    count - 1
}

// =============================================================================
// Callable table helpers
// =============================================================================

/// Check if typval v_type is VAR_DICT
#[no_mangle]
pub extern "C" fn rs_typval_is_dict(v_type: c_int) -> bool {
    v_type == 6 // VAR_DICT = 6
}

/// Check if typval v_type is VAR_LIST
#[no_mangle]
pub extern "C" fn rs_typval_is_list(v_type: c_int) -> bool {
    v_type == 5 // VAR_LIST = 5
}

/// Check if lua_table_ref indicates Lua origin
#[no_mangle]
pub extern "C" fn rs_has_lua_table_ref(ref_: c_int) -> bool {
    ref_ != LUA_NOREF
}

// =============================================================================
// Error string helpers
// =============================================================================

/// Calculate error string length limit for vim.call
#[no_mangle]
pub extern "C" fn rs_lua_call_name_limit(name_len: usize) -> usize {
    if name_len < 100 {
        name_len
    } else {
        100
    }
}

/// Calculate error buffer size for vim.call
#[no_mangle]
pub extern "C" fn rs_lua_call_error_buf_size(name_len: usize) -> usize {
    let limited = if name_len < 100 { name_len } else { 100 };
    limited + 26 // sizeof("Vimscript function \"\"")
}

// =============================================================================
// Expansion helpers
// =============================================================================

/// Check if prefix_len is valid for expansion
#[no_mangle]
pub extern "C" fn rs_expand_prefix_valid(prefix_len: isize, patlen: isize) -> bool {
    prefix_len >= 0 && prefix_len <= patlen
}

/// Check if expansion has results
#[no_mangle]
pub extern "C" fn rs_expand_has_results(num_results: c_int) -> bool {
    num_results > 0
}

// =============================================================================
// Luado helpers
// =============================================================================

/// Transform NUL bytes to newlines in a buffer
///
/// # Safety
/// - `buf` must be a valid pointer to a buffer of at least `len` bytes
/// - The buffer must be writable
#[no_mangle]
pub unsafe extern "C" fn rs_luado_transform_nuls(buf: *mut c_char, len: usize) {
    if buf.is_null() {
        return;
    }
    let slice = std::slice::from_raw_parts_mut(buf.cast::<u8>(), len);
    for byte in slice {
        if *byte == 0 {
            *byte = b'\n';
        }
    }
}

/// Check if line index exceeds buffer line count
#[no_mangle]
pub extern "C" fn rs_luado_line_exceeds(line: c_int, line_count: c_int) -> bool {
    line > line_count
}

// =============================================================================
// Sctx helpers
// =============================================================================

/// Check if verbose level requires sctx handling
#[no_mangle]
pub extern "C" fn rs_sctx_needs_verbose(p_verbose: c_int) -> bool {
    p_verbose > 0
}

/// Check if source is internal (starts with '@' followed by ignorelist match)
///
/// # Safety
/// - `source` must be a valid C string pointer or NULL
/// - `pattern` must be a valid C string pointer
#[no_mangle]
pub unsafe extern "C" fn rs_sctx_source_ignored(
    source: *const c_char,
    pattern: *const c_char,
    pattern_len: usize,
) -> bool {
    if source.is_null() || pattern.is_null() {
        return false;
    }

    let source = std::ffi::CStr::from_ptr(source);
    let source_bytes = source.to_bytes();

    // Check if source starts with '@'
    if source_bytes.is_empty() || source_bytes[0] != b'@' {
        return false;
    }

    let pattern = std::ffi::CStr::from_ptr(pattern);
    let pattern_bytes = pattern.to_bytes();

    // Compare source+1 with pattern
    let check_len = pattern_len.min(pattern_bytes.len());
    if source_bytes.len() < 1 + check_len {
        return false;
    }

    source_bytes[1..=check_len] == pattern_bytes[..check_len]
}

/// Check if debug info what is 'C' (C function)
#[no_mangle]
pub extern "C" fn rs_sctx_is_c_func(what: c_char) -> bool {
    what == b'C' as c_char
}

/// Check if source starts with '@'
#[no_mangle]
pub extern "C" fn rs_sctx_source_is_file(first_char: c_char) -> bool {
    first_char == b'@' as c_char
}

// =============================================================================
// ucmd helpers
// =============================================================================

/// Get split string for smods based on cmod_split flags
#[no_mangle]
pub extern "C" fn rs_ucmd_split_string(
    above: bool,
    below: bool,
    top: bool,
    bot: bool,
) -> *const c_char {
    if above {
        c"aboveleft".as_ptr()
    } else if below {
        c"belowright".as_ptr()
    } else if top {
        c"topleft".as_ptr()
    } else if bot {
        c"botright".as_ptr()
    } else {
        c"".as_ptr()
    }
}

/// Calculate adjusted tab value for smods (cmod_tab - 1)
#[no_mangle]
pub extern "C" fn rs_ucmd_smods_tab(cmod_tab: c_int) -> c_int {
    cmod_tab - 1
}

/// Calculate adjusted verbose value for smods (cmod_verbose - 1)
#[no_mangle]
pub extern "C" fn rs_ucmd_smods_verbose(cmod_verbose: c_int) -> c_int {
    cmod_verbose - 1
}

// =============================================================================
// Preview helpers
// =============================================================================

/// Calculate argument count for preview callback (3 for preview, 1 otherwise)
#[no_mangle]
pub extern "C" fn rs_ucmd_preview_argc(preview: bool) -> c_int {
    if preview {
        3
    } else {
        1
    }
}

/// Calculate return count for preview callback (1 for preview, 0 otherwise)
#[no_mangle]
pub extern "C" fn rs_ucmd_preview_retc(preview: bool) -> c_int {
    c_int::from(preview)
}

/// Validate preview return value (0, 1, or 2)
#[no_mangle]
pub extern "C" fn rs_ucmd_preview_retval_valid(retval: c_int) -> bool {
    (0..=2).contains(&retval)
}

// =============================================================================
// funcref_str helpers
// =============================================================================

/// Check if source is a file (starts with '@')
#[no_mangle]
pub extern "C" fn rs_funcref_is_file_source(first_char: c_char) -> bool {
    first_char == b'@' as c_char
}

/// Check if line number is valid for funcref string
#[no_mangle]
pub extern "C" fn rs_funcref_linedefined_valid(linedefined: c_int) -> bool {
    linedefined >= 0
}

// =============================================================================
// on_key helpers
// =============================================================================

/// Get initial value for save_got_int
#[no_mangle]
pub extern "C" fn rs_on_key_save_got_int(got_int: c_int) -> c_int {
    got_int
}

/// Calculate new got_int value: save | current
#[no_mangle]
pub extern "C" fn rs_on_key_restore_got_int(save_got_int: c_int, current_got_int: c_int) -> c_int {
    save_got_int | current_got_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_validation() {
        assert!(rs_wait_timeout_valid(0));
        assert!(rs_wait_timeout_valid(100));
        assert!(!rs_wait_timeout_valid(-1));

        assert!(rs_wait_interval_valid(0));
        assert!(rs_wait_interval_valid(200));
        assert!(!rs_wait_interval_valid(-1));

        assert_eq!(rs_wait_default_interval(), 200);
    }

    #[test]
    fn test_lua_ref_validation() {
        assert!(rs_lua_ref_valid(1));
        assert!(rs_lua_ref_valid(100));
        assert!(!rs_lua_ref_valid(0));
        assert!(!rs_lua_ref_valid(-1));
        assert!(!rs_lua_ref_valid(-2));

        assert!(rs_lua_is_noref(-2));
        assert!(!rs_lua_is_noref(-1));
        assert!(!rs_lua_is_noref(0));

        assert!(rs_lua_is_refnil(-1));
        assert!(!rs_lua_is_refnil(-2));
    }

    #[test]
    fn test_mode_ret() {
        assert_eq!(rs_lua_mode_ret(RET_MULTI), LUA_MULTRET);
        assert_eq!(rs_lua_mode_ret(RET_OBJECT), 1);
        assert_eq!(rs_lua_mode_ret(RET_NIL_BOOL), 1);
        assert_eq!(rs_lua_mode_ret(RET_LUAREF), 1);
    }

    #[test]
    fn test_mode_checks() {
        assert!(rs_lua_is_multi_mode(RET_MULTI));
        assert!(!rs_lua_is_multi_mode(RET_OBJECT));

        assert!(rs_lua_is_nil_bool_mode(RET_NIL_BOOL));
        assert!(!rs_lua_is_nil_bool_mode(RET_OBJECT));

        assert!(rs_lua_is_luaref_mode(RET_LUAREF));
        assert!(!rs_lua_is_luaref_mode(RET_OBJECT));

        assert!(rs_lua_is_object_mode(RET_OBJECT));
        assert!(!rs_lua_is_object_mode(RET_MULTI));
    }

    #[test]
    fn test_buffer_size_calculations() {
        // luaeval: "local _A=select(1,...) return (" = 31 chars
        assert_eq!(rs_lua_eval_header_size(), 31);
        assert_eq!(rs_lua_eval_cmd_size(10), 31 + 10 + 1);

        // luado: "return function(line, linenr) " = 30, " end" = 4
        assert_eq!(rs_lua_do_header_size(), 30);
        assert_eq!(rs_lua_do_end_size(), 4);
        assert_eq!(rs_lua_do_cmd_size(10), 30 + 10 + 4);

        // call: "return " = 7, "(...)" = 5
        assert_eq!(rs_lua_call_header_size(), 7);
        assert_eq!(rs_lua_call_suffix_size(), 5);
        assert_eq!(rs_lua_call_cmd_size(10), 7 + 10 + 5);
    }

    #[test]
    fn test_stack_calculations() {
        assert_eq!(rs_lua_pcall_insert_pos(2), -4);
        assert_eq!(rs_lua_pcall_insert_pos(0), -2);

        assert_eq!(rs_lua_pcall_remove_pos(1), -2);
        assert_eq!(rs_lua_pcall_remove_pos(3), -4);

        assert_eq!(rs_lua_calc_multret_nresults(10, 5, 2), 10 - (5 - 2 - 1));
        assert_eq!(rs_lua_fast_cb_nresult(10, 5, 2), 10 - 5 + 2 + 1);
    }

    #[test]
    fn test_ucmd_nargs() {
        // No extra args -> '0'
        assert_eq!(rs_ucmd_nargs_char(false, false, false), b'0' as c_char);

        // Extra + nospc + needarg -> '1'
        assert_eq!(rs_ucmd_nargs_char(true, true, true), b'1' as c_char);

        // Extra + nospc + !needarg -> '?'
        assert_eq!(rs_ucmd_nargs_char(true, true, false), b'?' as c_char);

        // Extra + !nospc + needarg -> '+'
        assert_eq!(rs_ucmd_nargs_char(true, false, true), b'+' as c_char);

        // Extra + !nospc + !needarg -> '*'
        assert_eq!(rs_ucmd_nargs_char(true, false, false), b'*' as c_char);
    }

    #[test]
    fn test_wait_results() {
        assert_eq!(rs_wait_nresults_empty(), 1);
        assert_eq!(rs_wait_return_count(2), 3);
        assert_eq!(rs_wait_timeout_code(), -1);
        assert_eq!(rs_wait_interrupted_code(), -2);
        assert!(rs_wait_no_results(0));
        assert!(!rs_wait_no_results(1));
    }

    #[test]
    fn test_ref_tracking() {
        assert!(rs_lua_ref_needs_tracking(1));
        assert!(!rs_lua_ref_needs_tracking(0));
        assert!(!rs_lua_ref_needs_tracking(-1));

        assert_eq!(rs_lua_ref_count_inc(5), 6);
        assert_eq!(rs_lua_ref_count_dec(5), 4);
    }

    #[test]
    fn test_typval_types() {
        assert!(rs_typval_is_dict(6));
        assert!(!rs_typval_is_dict(5));

        assert!(rs_typval_is_list(5));
        assert!(!rs_typval_is_list(6));

        assert!(rs_has_lua_table_ref(1));
        assert!(!rs_has_lua_table_ref(-2)); // LUA_NOREF
    }

    #[test]
    fn test_ucmd_smods() {
        assert_eq!(rs_ucmd_smods_tab(2), 1);
        assert_eq!(rs_ucmd_smods_tab(0), -1);
        assert_eq!(rs_ucmd_smods_verbose(3), 2);
    }

    #[test]
    fn test_preview() {
        assert_eq!(rs_ucmd_preview_argc(true), 3);
        assert_eq!(rs_ucmd_preview_argc(false), 1);
        assert_eq!(rs_ucmd_preview_retc(true), 1);
        assert_eq!(rs_ucmd_preview_retc(false), 0);

        assert!(rs_ucmd_preview_retval_valid(0));
        assert!(rs_ucmd_preview_retval_valid(1));
        assert!(rs_ucmd_preview_retval_valid(2));
        assert!(!rs_ucmd_preview_retval_valid(-1));
        assert!(!rs_ucmd_preview_retval_valid(3));
    }

    #[test]
    fn test_funcref() {
        assert!(rs_funcref_is_file_source(b'@' as c_char));
        assert!(!rs_funcref_is_file_source(b'[' as c_char));

        assert!(rs_funcref_linedefined_valid(0));
        assert!(rs_funcref_linedefined_valid(100));
        assert!(!rs_funcref_linedefined_valid(-1));
    }

    #[test]
    fn test_expansion() {
        assert!(rs_expand_prefix_valid(0, 10));
        assert!(rs_expand_prefix_valid(5, 10));
        assert!(rs_expand_prefix_valid(10, 10));
        assert!(!rs_expand_prefix_valid(-1, 10));
        assert!(!rs_expand_prefix_valid(11, 10));

        assert!(rs_expand_has_results(1));
        assert!(!rs_expand_has_results(0));
    }
}
