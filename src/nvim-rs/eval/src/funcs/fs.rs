//! Filesystem VimL built-in functions.
//!
//! This module implements filesystem-related VimL functions migrated from
//! `src/nvim/eval/fs.c`.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, c_void};

use super::dispatch::{argvar_at, rettv_set_bool, rettv_set_number, TypevalPtrMut};

// =============================================================================
// C accessor declarations
// =============================================================================

extern "C" {
    fn rs_check_secure() -> c_int;
    fn os_can_exe(name: *const u8, path: *mut *mut u8, use_path: bool) -> c_int;
    fn vim_tempname() -> *mut u8;
    fn simplify_filename(filename: *mut u8);
    fn vim_rename(from: *const u8, to: *const u8) -> c_int;
    fn os_fileinfo(fname: *const u8, info: *mut c_void) -> bool;
    fn nvim_fileinfo_mtime(info: *const c_void) -> i64;
    fn os_getperm(fname: *const u8) -> i32;
    fn file_pat_to_reg_pat(
        pat: *const u8,
        pat_end: *const u8,
        allow_dirs: *const u8,
        no_bslash: bool,
    ) -> *mut u8;
    fn tv_check_for_string_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn tv_check_for_nonempty_string_arg(argvars: *const c_void, idx: c_int) -> c_int;
    fn nvim_tv_set_string(tv: *mut c_void, s: *mut u8);
    fn nvim_tv_get_string(tv: *const c_void, out_len: *mut usize) -> *const u8;
    fn nvim_tv_get_string_chk(tv: *const c_void, out_len: *mut usize) -> *const u8;
    fn nvim_tv_get_string_buf(tv: *const c_void, buf: *mut u8) -> *const u8;
    #[link_name = "xstrdup"]
    fn xstrdup_u8(s: *const c_char) -> *mut c_char;
    fn os_isdir(name: *const u8) -> bool;
    fn os_file_is_readable(name: *const u8) -> bool;
    fn os_file_is_writable(name: *const u8) -> c_int;
    fn path_is_absolute(fname: *const u8) -> bool;
}

// NUMBUFLEN from vim_defs.h
const NUMBUFLEN: usize = 65;

// FileInfo opaque size: uv_stat_t is 224 bytes on Linux x86_64
// Use 256 bytes to be safe
const FILEINFO_SIZE: usize = 256;

// =============================================================================
// Internal helpers
// =============================================================================

/// Get a null-terminated C string from a typval, returning a Vec<u8>.
#[inline]
unsafe fn tv_to_cstring(tv: *const c_void) -> Vec<u8> {
    let mut len: usize = 0;
    let ptr = unsafe { nvim_tv_get_string(tv, &raw mut len) };
    if ptr.is_null() || len == 0 {
        // Return empty null-terminated string
        vec![0u8]
    } else {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let mut v = slice.to_vec();
        v.push(0);
        v
    }
}

/// Get a null-terminated C string from a typval (with chk), or None on type error.
#[inline]
unsafe fn tv_to_cstring_chk(tv: *const c_void) -> Option<Vec<u8>> {
    let mut len: usize = 0;
    let ptr = unsafe { nvim_tv_get_string_chk(tv, &raw mut len) };
    if ptr.is_null() {
        None
    } else {
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let mut v = slice.to_vec();
        v.push(0);
        Some(v)
    }
}

// =============================================================================
// Phase 1: Simple one-liner and small functions
// =============================================================================

/// "browse(save, title, initdir, default)" function - no-op stub
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_browse"]
pub unsafe extern "C" fn rs_f_browse(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // Returns empty string (NULL v_string means empty in VimL)
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };
}

/// "browsedir(title, initdir)" function - delegates to f_browse
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_browsedir"]
pub unsafe extern "C" fn rs_f_browsedir(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { rs_f_browse(argvars, rettv, fptr) };
}

/// "isabsolutepath()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_isabsolutepath"]
pub unsafe extern "C" fn rs_f_isabsolutepath(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let result = buf[0] != 0 && unsafe { path_is_absolute(buf.as_ptr()) };
    rettv_set_bool(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "isdirectory()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_isdirectory"]
pub unsafe extern "C" fn rs_f_isdirectory(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let result = buf[0] != 0 && unsafe { os_isdir(buf.as_ptr()) };
    rettv_set_bool(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "filereadable()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_filereadable"]
pub unsafe extern "C" fn rs_f_filereadable(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let p = buf.as_ptr();
    let result = buf[0] != 0 && unsafe { !os_isdir(p) && os_file_is_readable(p) };
    rettv_set_bool(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "filewritable()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_filewritable"]
pub unsafe extern "C" fn rs_f_filewritable(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let result = if buf[0] != 0 {
        unsafe { os_file_is_writable(buf.as_ptr()) }
    } else {
        0
    };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, i64::from(result));
}

/// "executable()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_executable"]
pub unsafe extern "C" fn rs_f_executable(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if unsafe { tv_check_for_string_arg(argvars, 0) } == -1 {
        return;
    }
    let buf = unsafe { tv_to_cstring(argvars) };
    let result = unsafe { os_can_exe(buf.as_ptr(), std::ptr::null_mut(), true) };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, i64::from(result));
}

/// "tempname()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_tempname"]
pub unsafe extern "C" fn rs_f_tempname(
    _argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let s = unsafe { vim_tempname() };
    unsafe { nvim_tv_set_string(rettv, s) };
}

/// "simplify()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_simplify"]
pub unsafe extern "C" fn rs_f_simplify(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let dup = unsafe { xstrdup_u8(buf.as_ptr().cast()) };
    if !dup.is_null() {
        unsafe { simplify_filename(dup.cast()) };
    }
    unsafe { nvim_tv_set_string(rettv, dup.cast()) };
}

/// "rename({from}, {to})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_rename"]
pub unsafe extern "C" fn rs_f_rename(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if unsafe { rs_check_secure() } != 0 {
        rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, -1);
        return;
    }
    let from_buf = unsafe { tv_to_cstring(argvars) };
    let tv1 = unsafe { argvar_at(argvars, 1) };
    let mut nbuf = vec![0u8; NUMBUFLEN];
    let to_ptr = unsafe { nvim_tv_get_string_buf(tv1.as_ptr(), nbuf.as_mut_ptr()) };
    let result = unsafe { vim_rename(from_buf.as_ptr(), to_ptr) };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, i64::from(result));
}

/// "getftime({fname})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getftime"]
pub unsafe extern "C" fn rs_f_getftime(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let mut file_info = [0u8; FILEINFO_SIZE];
    let result = if unsafe { os_fileinfo(buf.as_ptr(), file_info.as_mut_ptr().cast()) } {
        unsafe { nvim_fileinfo_mtime(file_info.as_ptr().cast()) }
    } else {
        -1
    };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "glob2regpat()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_glob2regpat"]
pub unsafe extern "C" fn rs_f_glob2regpat(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let result = unsafe { tv_to_cstring_chk(argvars) }.map_or(std::ptr::null_mut(), |pat| unsafe {
        file_pat_to_reg_pat(pat.as_ptr(), std::ptr::null(), std::ptr::null(), false)
    });
    unsafe { nvim_tv_set_string(rettv, result) };
}

/// "exepath()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_exepath"]
pub unsafe extern "C" fn rs_f_exepath(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if unsafe { tv_check_for_nonempty_string_arg(argvars, 0) } == -1 {
        return;
    }
    let buf = unsafe { tv_to_cstring(argvars) };
    let mut path: *mut u8 = std::ptr::null_mut();
    unsafe { os_can_exe(buf.as_ptr(), &raw mut path, true) };
    unsafe { nvim_tv_set_string(rettv, path) };
}

/// "getfperm({fname})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getfperm"]
pub unsafe extern "C" fn rs_f_getfperm(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let file_perm = unsafe { os_getperm(buf.as_ptr()) };
    let s = if file_perm >= 0 {
        let flags = b"rwx";
        let mut perm = b"---------".to_vec();
        for i in 0..9usize {
            if file_perm & (1 << (8 - i)) != 0 {
                perm[i] = flags[i % 3];
            }
        }
        perm.push(0);
        unsafe { xstrdup_u8(perm.as_ptr().cast()).cast() }
    } else {
        std::ptr::null_mut()
    };
    unsafe { nvim_tv_set_string(rettv, s) };
}
