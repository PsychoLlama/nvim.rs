//! Script host functions for :ruby, :python3, :perl and related commands
//!
//! Ports of `script_host_execute`, `script_host_execute_file`,
//! `script_host_do_range` and the nine `ex_*` thin wrappers.

use std::ffi::{c_char, c_int};

/// Maximum path length (matches MAXPATHL in C)
const MAXPATHL: usize = 4096;

// Opaque handle types for C structs
#[repr(C)]
pub struct ExArgHandle {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct ListHandle {
    _opaque: [u8; 0],
}

extern "C" {
    // --- exarg_T field accessors (kept, access struct fields) ---
    fn nvim_ex2_eap_get_arg(eap: *mut ExArgHandle) -> *mut c_char;
    fn nvim_ex2_eap_get_line1(eap: *mut ExArgHandle) -> i32;
    fn nvim_ex2_eap_get_line2(eap: *mut ExArgHandle) -> i32;
    fn nvim_ex2_eap_get_skip(eap: *mut ExArgHandle) -> c_int;

    // --- C library functions (via link_name to real symbols) ---
    #[link_name = "script_get"]
    fn nvim_ex2_script_get(eap: *mut ExArgHandle, lenp: *mut usize) -> *mut c_char;
    #[link_name = "tv_list_alloc"]
    fn nvim_ex2_tv_list_alloc(len: isize) -> *mut ListHandle;
    #[link_name = "tv_list_append_allocated_string"]
    fn nvim_ex2_tv_list_append_allocated_string(l: *mut ListHandle, s: *mut c_char);
    #[link_name = "tv_list_append_number"]
    fn nvim_ex2_tv_list_append_number(l: *mut ListHandle, n: i64);
    #[link_name = "tv_list_append_string"]
    fn nvim_ex2_tv_list_append_string(l: *mut ListHandle, s: *const c_char, len: isize);
    // nvim_ex2_eval_call_provider: special wrapper (stack-local rettv), kept
    fn nvim_ex2_eval_call_provider(
        provider: *const c_char,
        method: *const c_char,
        arguments: *mut ListHandle,
        discard: bool,
    );
    #[link_name = "vim_FullName"]
    fn nvim_ex2_vim_fullname(
        fname: *const c_char,
        buf: *mut c_char,
        len: usize,
        force: bool,
    ) -> c_int;
}

// ---------------------------------------------------------------------------
// Internal implementations
// ---------------------------------------------------------------------------

/// Port of `script_host_execute` (was static in C)
unsafe fn script_host_execute(name: *const c_char, eap: *mut ExArgHandle) {
    let mut len: usize = 0;
    let script = unsafe { nvim_ex2_script_get(eap, &mut len) };

    if !script.is_null() {
        let args = unsafe { nvim_ex2_tv_list_alloc(3) };
        // script (ownership transferred)
        unsafe { nvim_ex2_tv_list_append_allocated_string(args, script) };
        // current range
        let line1 = unsafe { nvim_ex2_eap_get_line1(eap) };
        let line2 = unsafe { nvim_ex2_eap_get_line2(eap) };
        unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line1)) };
        unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line2)) };

        unsafe { nvim_ex2_eval_call_provider(name, b"execute\0".as_ptr().cast(), args, true) };
    }
}

/// Port of `script_host_execute_file` (was static in C)
unsafe fn script_host_execute_file(name: *const c_char, eap: *mut ExArgHandle) {
    if unsafe { nvim_ex2_eap_get_skip(eap) } != 0 {
        return;
    }

    let mut buffer = [0u8; MAXPATHL];
    let arg = unsafe { nvim_ex2_eap_get_arg(eap) };
    unsafe {
        nvim_ex2_vim_fullname(arg, buffer.as_mut_ptr().cast(), MAXPATHL, false);
    }

    let args = unsafe { nvim_ex2_tv_list_alloc(3) };
    // filename
    unsafe { nvim_ex2_tv_list_append_string(args, buffer.as_ptr().cast(), -1) };
    // current range
    let line1 = unsafe { nvim_ex2_eap_get_line1(eap) };
    let line2 = unsafe { nvim_ex2_eap_get_line2(eap) };
    unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line1)) };
    unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line2)) };

    unsafe { nvim_ex2_eval_call_provider(name, b"execute_file\0".as_ptr().cast(), args, true) };
}

/// Port of `script_host_do_range` (was static in C)
unsafe fn script_host_do_range(name: *const c_char, eap: *mut ExArgHandle) {
    if unsafe { nvim_ex2_eap_get_skip(eap) } != 0 {
        return;
    }

    let args = unsafe { nvim_ex2_tv_list_alloc(3) };
    let line1 = unsafe { nvim_ex2_eap_get_line1(eap) };
    let line2 = unsafe { nvim_ex2_eap_get_line2(eap) };
    let arg = unsafe { nvim_ex2_eap_get_arg(eap) };
    unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line1)) };
    unsafe { nvim_ex2_tv_list_append_number(args, i64::from(line2)) };
    unsafe { nvim_ex2_tv_list_append_string(args, arg, -1) };

    unsafe { nvim_ex2_eval_call_provider(name, b"do_range\0".as_ptr().cast(), args, true) };
}

// ---------------------------------------------------------------------------
// Exported functions — directly exported as their C names via export_name
// ---------------------------------------------------------------------------

#[export_name = "ex_ruby"]
pub unsafe extern "C" fn rs_ex_ruby(eap: *mut ExArgHandle) {
    unsafe { script_host_execute(b"ruby\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_rubyfile"]
pub unsafe extern "C" fn rs_ex_rubyfile(eap: *mut ExArgHandle) {
    unsafe { script_host_execute_file(b"ruby\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_rubydo"]
pub unsafe extern "C" fn rs_ex_rubydo(eap: *mut ExArgHandle) {
    unsafe { script_host_do_range(b"ruby\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_python3"]
pub unsafe extern "C" fn rs_ex_python3(eap: *mut ExArgHandle) {
    unsafe { script_host_execute(b"python3\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_py3file"]
pub unsafe extern "C" fn rs_ex_py3file(eap: *mut ExArgHandle) {
    unsafe { script_host_execute_file(b"python3\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_pydo3"]
pub unsafe extern "C" fn rs_ex_pydo3(eap: *mut ExArgHandle) {
    unsafe { script_host_do_range(b"python3\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_perl"]
pub unsafe extern "C" fn rs_ex_perl(eap: *mut ExArgHandle) {
    unsafe { script_host_execute(b"perl\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_perlfile"]
pub unsafe extern "C" fn rs_ex_perlfile(eap: *mut ExArgHandle) {
    unsafe { script_host_execute_file(b"perl\0".as_ptr().cast(), eap) };
}

#[export_name = "ex_perldo"]
pub unsafe extern "C" fn rs_ex_perldo(eap: *mut ExArgHandle) {
    unsafe { script_host_do_range(b"perl\0".as_ptr().cast(), eap) };
}
