//! VimL system() and systemlist() built-in functions.
//!
//! Implements `rs_f_system` and `rs_f_systemlist` (and shared helper
//! `get_system_output_impl`), plus `rs_tv_to_argv` which converts a
//! typval_T (String or List) to a null-terminated argv array.
//!
//! Migrated from `eval_shim.c` Phase 2 (eval_shim pass 6).

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // ----- typval accessors -----
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_tv_set_type(tv: *mut c_void, vtype: c_int);
    #[allow(dead_code)]
    fn nvim_tv_get_vstring(tv: *mut c_void) -> *mut c_char;
    fn nvim_tv_set_vstring_raw(tv: *mut c_void, s: *mut c_char);
    fn nvim_eval_tv_get_list(tv: *const c_void) -> *mut c_void;
    fn nvim_tv_set_v_list(tv: *mut c_void, l: *mut c_void);
    #[link_name = "tv_get_string_chk"]
    fn nvim_eval_tv_string_chk(tv: *mut c_void) -> *const c_char;
    #[link_name = "tv_get_string"]
    fn nvim_eval_tv_get_str(tv: *mut c_void) -> *const c_char;
    fn nvim_tv_get_number(tv: *const c_void) -> i64;

    // ----- list accessors -----
    fn nvim_tv_list_len(l: *const c_void) -> c_int;
    fn nvim_tv_list_first(l: *const c_void) -> *mut c_void; // listitem_T *
    fn nvim_list_item_next(l: *mut c_void, item: *mut c_void) -> *mut c_void;
    fn nvim_list_item_tv(item: *mut c_void) -> *mut c_void; // typval_T *
    fn nvim_tv_list_ref(l: *mut c_void);
    #[link_name = "tv_list_alloc_ret"]
    fn nvim_tv_list_alloc_ret(rettv: *mut c_void, count_hint: isize) -> *mut c_void;
    fn nvim_tv_list_alloc() -> *mut c_void;
    #[link_name = "encode_list_write"]
    fn nvim_encode_list_write(list: *mut c_void, str: *const c_char, len: usize);

    // ----- shell / os -----
    #[link_name = "shell_build_argv"]
    fn nvim_shell_build_argv(cmd: *const c_char, extra: *const c_char) -> *mut *mut c_char;
    #[link_name = "shell_free_argv"]
    fn nvim_shell_free_argv(argv: *mut *mut c_char);
    #[link_name = "shell_argv_to_str"]
    fn nvim_shell_argv_to_str(argv: *mut *mut c_char) -> *mut c_char;
    fn nvim_eval_os_can_exe(name: *const c_char, abspath: *mut *mut c_char) -> bool;
    #[link_name = "os_system"]
    fn nvim_os_system(
        argv: *mut *mut c_char,
        input: *const c_char,
        input_len: usize,
        output_out: *mut *mut c_char,
        nread_out: *mut usize,
    ) -> c_int;

    // ----- profiling -----
    fn nvim_do_profiling_active() -> bool;
    #[link_name = "prof_child_enter"]
    fn nvim_prof_child_enter(tm: *mut u64);
    #[link_name = "prof_child_exit"]
    fn nvim_prof_child_exit(tm: *mut u64);

    // ----- verbose -----
    fn nvim_p_verbose_get() -> c_int;
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn msg_puts(s: *const c_char);
    fn nvim_smsg_system_cmd(cmdstr: *const c_char);

    // ----- VV_SHELL_ERROR -----
    fn nvim_set_shell_error(status: c_int);

    // ----- error messages: now in crate::errors -----

    // ----- memory -----
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_xstrdup(str: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    #[link_name = "memchrsub"]
    fn nvim_eval_memchrsub(data: *mut c_char, c: c_char, x: c_char, len: usize);

    // ----- secure check -----
    fn rs_check_secure() -> c_int;

    // ----- save_tv_as_string (renamed Rust export) -----
    fn save_tv_as_string(tv: *mut c_void, len: *mut isize, endnl: bool, crlf: bool) -> *mut c_char;
}

// =============================================================================
// VarType constants (must match C enum vartype_T)
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;

// =============================================================================
// typval_T size (verified by _Static_assert in testing.c)
// =============================================================================

/// Size of a single typval_T in bytes.
const TYPVAL_SIZE: usize = 16;

/// Get pointer to argvars[idx].
///
/// # Safety
/// argvars must point to at least (idx+1) typval_T entries.
#[inline]
unsafe fn argvars_at(argvars: *mut c_void, idx: usize) -> *mut c_void {
    argvars.cast::<u8>().add(idx * TYPVAL_SIZE).cast::<c_void>()
}

// =============================================================================
// Private helpers
// =============================================================================

/// Convert a string with embedded NLs to a VimL list.
///
/// Each NL-separated segment becomes a list item.  If `keepempty` is false and
/// the string ends with NL, that trailing empty entry is dropped.
///
/// # Safety
/// - `str` must be a valid pointer to at least `len` bytes.
unsafe fn string_to_list(str: *const c_char, len: usize, keepempty: bool) -> *mut c_void {
    let effective_len = if !keepempty && len > 0 && *str.add(len - 1) == b'\n' as c_char {
        len - 1
    } else {
        len
    };
    let list = nvim_tv_list_alloc();
    nvim_encode_list_write(list, str, effective_len);
    list
}

// =============================================================================
// Phase 2: rs_tv_to_argv
// =============================================================================

/// Build a null-terminated argv array from a VimL typval_T (String or List).
///
/// - If `cmd_tv` is a String, delegates to `shell_build_argv` (shell semantics).
/// - If `cmd_tv` is a List, validates the first element is executable and builds
///   a direct argv array.
///
/// Returns the argv array on success (caller must free with `shell_free_argv`).
/// Returns NULL on failure.
///
/// # Safety
/// - `cmd_tv` must be a valid non-null typval pointer.
/// - `cmd_out` and `executable` may be null.
#[export_name = "tv_to_argv"]
pub unsafe extern "C" fn rs_tv_to_argv(
    cmd_tv: *mut c_void,
    cmd_out: *mut *const c_char,
    executable: *mut bool,
) -> *mut *mut c_char {
    let vtype = nvim_tv_get_type(cmd_tv);

    if vtype == VAR_STRING {
        // String => shell semantics
        let cmd_str = nvim_eval_tv_get_str(cmd_tv);
        if !cmd_out.is_null() {
            *cmd_out = cmd_str;
        }
        return nvim_shell_build_argv(cmd_str, ptr::null());
    }

    if vtype != VAR_LIST {
        crate::errors::semsg_tv_to_argv_type();
        return ptr::null_mut();
    }

    let cmd_list = nvim_eval_tv_get_list(cmd_tv.cast_const());
    let list_len = nvim_tv_list_len(cmd_list);
    if list_len == 0 {
        crate::errors::emsg_tv_to_argv_empty();
        return ptr::null_mut();
    }

    // Get string value of first list element
    let first_item = nvim_tv_list_first(cmd_list);
    let first_tv = nvim_list_item_tv(first_item);
    let cmd0 = nvim_eval_tv_string_chk(first_tv);

    let mut exe_resolved: *mut c_char = ptr::null_mut();
    if cmd0.is_null() || !nvim_eval_os_can_exe(cmd0, &raw mut exe_resolved) {
        if !cmd0.is_null() && !executable.is_null() {
            // Emit "'<cmd>' is not executable" error
            let msg = build_not_executable_msg(cmd0);
            crate::errors::semsg_tv_to_argv_notexe(msg.as_ptr().cast::<c_char>());
            *executable = false;
        }
        return ptr::null_mut();
    }

    if !cmd_out.is_null() {
        *cmd_out = exe_resolved;
    }

    // Allocate result_argv with (list_len + 1) slots (zero-initialized = null-terminated)
    let result_argv = nvim_xcalloc((list_len + 1) as usize, std::mem::size_of::<*mut c_char>())
        .cast::<*mut c_char>();

    if result_argv.is_null() {
        xfree(exe_resolved.cast::<c_void>());
        return ptr::null_mut();
    }

    // Iterate list items and duplicate strings
    let mut i = 0usize;
    let mut item = nvim_tv_list_first(cmd_list);
    while !item.is_null() {
        let tv = nvim_list_item_tv(item);
        let s = nvim_eval_tv_string_chk(tv);
        if s.is_null() {
            // tv_get_string_chk already emitted an error
            nvim_shell_free_argv(result_argv);
            xfree(exe_resolved.cast::<c_void>());
            return ptr::null_mut();
        }
        *result_argv.add(i) = nvim_xstrdup(s);
        i += 1;
        item = nvim_list_item_next(cmd_list, item);
    }

    // Replace result_argv[0] with the absolute resolved path
    xfree((*result_argv).cast::<c_void>());
    *result_argv = exe_resolved;

    result_argv
}

/// Build "'<name>' is not executable" as a null-terminated byte string.
unsafe fn build_not_executable_msg(name: *const c_char) -> Vec<u8> {
    let mut len = 0usize;
    while *name.add(len) != 0 {
        len += 1;
    }
    let name_bytes = std::slice::from_raw_parts(name.cast::<u8>(), len);
    let mut msg = Vec::with_capacity(1 + len + 20);
    msg.push(b'\'');
    msg.extend_from_slice(name_bytes);
    msg.extend_from_slice(b"' is not executable\0");
    msg
}

// =============================================================================
// Phase 2: get_system_output_impl (internal)
// =============================================================================

/// Core implementation of system() / systemlist().
///
/// # Safety
/// - `argvars` must be a valid pointer to at least 3 contiguous typval_T.
/// - `rettv` must be a valid typval pointer.
unsafe fn get_system_output_impl(argvars: *mut c_void, rettv: *mut c_void, retlist: bool) {
    // Initialize rettv as VAR_STRING with NULL string
    nvim_tv_set_type(rettv, VAR_STRING);
    nvim_tv_set_vstring_raw(rettv, ptr::null_mut());

    if rs_check_secure() != 0 {
        return;
    }

    let arg0 = argvars_at(argvars, 0);
    let arg1 = argvars_at(argvars, 1);
    let arg2 = argvars_at(argvars, 2);

    // Get optional input string from argvars[1]
    let mut input_len: isize = 0;
    let input = save_tv_as_string(arg1, &raw mut input_len, false, false);
    if input_len < 0 {
        // Error already reported
        return;
    }

    // Build cmd_argv from argvars[0]
    let mut executable = true;
    let cmd_argv = rs_tv_to_argv(arg0, ptr::null_mut(), &raw mut executable);
    if cmd_argv.is_null() {
        if !executable {
            nvim_set_shell_error(-1);
        }
        xfree(input.cast::<c_void>());
        return;
    }

    // Verbose output: log command if p_verbose > 3
    if nvim_p_verbose_get() > 3 {
        let cmdstr = nvim_shell_argv_to_str(cmd_argv);
        verbose_enter_scroll();
        nvim_smsg_system_cmd(cmdstr);
        msg_puts(c"\n\n".as_ptr());
        verbose_leave_scroll();
        xfree(cmdstr.cast::<c_void>());
    }

    // Profiling
    let profiling = nvim_do_profiling_active();
    let mut wait_time: u64 = 0;
    if profiling {
        nvim_prof_child_enter(&raw mut wait_time);
    }

    // Execute the system command
    let mut nread: usize = 0;
    let mut res: *mut c_char = ptr::null_mut();
    let status = nvim_os_system(
        cmd_argv,
        input,
        input_len as usize,
        &raw mut res,
        &raw mut nread,
    );

    if profiling {
        nvim_prof_child_exit(&raw mut wait_time);
    }

    xfree(input.cast::<c_void>());
    nvim_set_shell_error(status);

    if res.is_null() {
        if retlist {
            // Empty list for no output
            nvim_tv_list_alloc_ret(rettv, 0);
            nvim_tv_set_type(rettv, VAR_LIST);
        } else {
            // Empty string
            let empty = nvim_xstrdup(c"".as_ptr());
            nvim_tv_set_vstring_raw(rettv, empty);
        }
        return;
    }

    if retlist {
        // Check keepempty from argvars[2] (only if argvars[1] and [2] are set)
        let keepempty =
            if nvim_tv_get_type(arg1) != VAR_UNKNOWN && nvim_tv_get_type(arg2) != VAR_UNKNOWN {
                nvim_tv_get_number(arg2) != 0
            } else {
                false
            };
        let list = string_to_list(res, nread, keepempty);
        nvim_tv_list_ref(list);
        nvim_tv_set_type(rettv, VAR_LIST);
        nvim_tv_set_v_list(rettv, list);
        xfree(res.cast::<c_void>());
    } else {
        // Replace NUL bytes with SOH (1) to avoid string truncation
        nvim_eval_memchrsub(res, 0, 1, nread);
        #[cfg(target_os = "windows")]
        translate_crnl(res, nread);
        nvim_tv_set_vstring_raw(rettv, res);
    }
}

/// Translate CR+NL sequences to NL only (Windows-specific).
#[cfg(target_os = "windows")]
unsafe fn translate_crnl(res: *mut c_char, _nread: usize) {
    let mut d = res;
    let mut s = res;
    loop {
        let c = *s;
        if c == 0 {
            break;
        }
        if c == b'\r' as c_char && *s.add(1) == b'\n' as c_char {
            s = s.add(1);
        }
        *d = *s;
        d = d.add(1);
        s = s.add(1);
    }
    *d = 0;
}

// =============================================================================
// Phase 2: FFI exports
// =============================================================================

/// VimL `system()` entry point -- returns output as a string.
///
/// # Safety
/// - `argvars` must point to at least 3 contiguous typval_T.
/// - `rettv` must be a valid typval pointer.
/// - `fptr` is unused (VimL EvalFuncData).
#[no_mangle]
pub unsafe extern "C" fn f_system(argvars: *mut c_void, rettv: *mut c_void, _fptr: *mut c_void) {
    get_system_output_impl(argvars, rettv, false);
}

/// VimL `systemlist()` entry point -- returns output as a list.
///
/// # Safety
/// See `rs_f_system`.
#[no_mangle]
pub unsafe extern "C" fn f_systemlist(
    argvars: *mut c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    get_system_output_impl(argvars, rettv, true);
}
