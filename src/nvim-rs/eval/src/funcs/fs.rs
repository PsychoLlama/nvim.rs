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
#![allow(clippy::struct_field_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::if_not_else)]

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
    // Phase 2 additions
    fn os_fileinfo_link(fname: *const u8, info: *mut c_void) -> bool;
    fn nvim_fileinfo_mode(info: *const c_void) -> u64;
    fn nvim_fileinfo_get_size(info: *const c_void) -> u64;
    fn os_remove(name: *const u8) -> c_int;
    fn os_rmdir(name: *const u8) -> c_int;
    fn delete_recursive(name: *const u8) -> c_int;
    fn vim_copyfile(from: *const u8, to: *const u8) -> c_int;
    fn shorten_dir_len(str: *mut u8, trim_len: c_int);
    fn nvim_tv_get_number(tv: *const c_void) -> i64;
    fn nvim_tv_get_type(tv: *const c_void) -> c_int;
    #[link_name = "emsg"]
    fn fs_emsg(s: *const c_char) -> c_int;
    #[allow(clashing_extern_declarations)]
    #[link_name = "semsg"]
    fn fs_semsg(fmt: *const c_char, arg: *const c_char) -> c_int;
    // Phase 3 accessors
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_get_curtab() -> *mut c_void;
    fn nvim_win_get_localdir(wp: *const c_void) -> *const c_char;
    fn nvim_tab_get_localdir(tp: *const c_void) -> *const c_char;
    fn nvim_get_globaldir() -> *const c_char;
    fn nvim_changedir_func(dir: *const c_char, scope: c_int) -> bool;
    fn nvim_tv_get_vstring(tv: *const c_void) -> *const c_char;
    fn nvim_os_dirname(buf: *mut c_char, len: usize) -> c_int;
    fn nvim_find_win_by_nr(tv: *const c_void, tp: *mut c_void) -> *mut c_void;
    fn nvim_fs_mkdir_recurse(
        dir: *const c_char,
        mode: u32,
        failed_dir: *mut *mut c_char,
        created: *mut *mut c_char,
    ) -> c_int;
    fn nvim_vim_mkdir_emsg(name: *const c_char, prot: c_int) -> c_int;
    fn nvim_can_add_defer() -> bool;
    fn nvim_add_defer_delete(tv: *mut c_void, argc: c_int);
    fn nvim_FullName_save(fname: *const c_char, force: bool) -> *mut c_char;
    fn nvim_fs_path_tail(fname: *const c_char) -> *const c_char;
    fn nvim_path_tail_with_sep(fname: *mut c_char) -> *mut c_char;
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    fn nvim_os_strerror(err_no: c_int) -> *const c_char;
    // readdir accessors
    fn nvim_readdir_core(
        gap: *mut GarrayT,
        path: *const c_char,
        context: *mut c_void,
        checkitem: unsafe extern "C" fn(*mut c_void, *const c_char) -> i64,
    ) -> c_int;
    fn nvim_fs_ga_get_len(gap: *const GarrayT) -> c_int;
    fn nvim_fs_ga_get_str(gap: *const GarrayT, i: c_int) -> *const c_char;
    fn nvim_ga_clear_strings(gap: *mut GarrayT);
    fn nvim_fs_tv_list_alloc_ret(rettv: *mut c_void, len: isize);
    fn nvim_tv_list_append_string(list: *mut c_void, s: *const c_char, len: isize);
    fn nvim_tv_get_list_from_rettv(rettv: *const c_void) -> *mut c_void;
    fn nvim_eval_readdir_expr(expr: *const c_void, name: *const c_char) -> i64;
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    #[link_name = "xstrdup"]
    fn xstrdup_c(s: *const c_char) -> *mut c_char;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dst_len: usize) -> usize;
}

/// Mirror of C garray_T struct (from garray_defs.h).
/// Used for passing to C garray accessor functions by pointer.
#[repr(C)]
struct GarrayT {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl GarrayT {
    const fn zeroed() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// CdScope enum values (from ex_cmds_defs.h)
const CD_SCOPE_INVALID: c_int = -1;
const CD_SCOPE_WINDOW: c_int = 0; // kCdScopeWindow = MIN_CD_SCOPE
const CD_SCOPE_TABPAGE: c_int = 1;
const CD_SCOPE_GLOBAL: c_int = 2; // kCdScopeGlobal = MAX_CD_SCOPE
const MIN_CD_SCOPE: c_int = CD_SCOPE_WINDOW;
const MAX_CD_SCOPE: c_int = CD_SCOPE_GLOBAL;

// C OK/FAIL
const FAIL: c_int = 0;

// MAXPATHL from path.h
const MAXPATHL: usize = 4096;

// NUMBUFLEN from vim_defs.h
const NUMBUFLEN: usize = 65;

// FileInfo opaque size: uv_stat_t is 224 bytes on Linux x86_64
// Use 256 bytes to be safe
const FILEINFO_SIZE: usize = 256;

// OK/FAIL from nvim/types_defs.h
const OK: c_int = 1;

// File mode constants (POSIX)
const S_IFMT: u64 = 0o170_000;
const S_IFREG: u64 = 0o100_000;
const S_IFDIR: u64 = 0o040_000;
const S_IFLNK: u64 = 0o120_000;
const S_IFBLK: u64 = 0o060_000;
const S_IFCHR: u64 = 0o020_000;
const S_IFIFO: u64 = 0o010_000;
const S_IFSOCK: u64 = 0o140_000;

// VAR_UNKNOWN type value
const VAR_UNKNOWN: i32 = 0;

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

// =============================================================================
// Phase 2: File info and stat functions
// =============================================================================

/// "getfsize({fname})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getfsize"]
pub unsafe extern "C" fn rs_f_getfsize(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let mut file_info = [0u8; FILEINFO_SIZE];
    let result = if unsafe { os_fileinfo(buf.as_ptr(), file_info.as_mut_ptr().cast()) } {
        let filesize = unsafe { nvim_fileinfo_get_size(file_info.as_ptr().cast()) };
        if unsafe { os_isdir(buf.as_ptr()) } {
            0i64
        } else {
            let n = filesize as i64;
            // Non-perfect overflow check
            if n as u64 == filesize {
                n
            } else {
                -2
            }
        }
    } else {
        -1
    };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "getftype({fname})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getftype"]
pub unsafe extern "C" fn rs_f_getftype(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let buf = unsafe { tv_to_cstring(argvars) };
    let mut file_info = [0u8; FILEINFO_SIZE];
    let s = if unsafe { os_fileinfo_link(buf.as_ptr(), file_info.as_mut_ptr().cast()) } {
        let mode = unsafe { nvim_fileinfo_mode(file_info.as_ptr().cast()) };
        let fmt = mode & S_IFMT;
        let t: &[u8] = if fmt == S_IFREG {
            b"file\0"
        } else if fmt == S_IFDIR {
            b"dir\0"
        } else if fmt == S_IFLNK {
            b"link\0"
        } else if fmt == S_IFBLK {
            b"bdev\0"
        } else if fmt == S_IFCHR {
            b"cdev\0"
        } else if fmt == S_IFIFO {
            b"fifo\0"
        } else if fmt == S_IFSOCK {
            b"socket\0"
        } else {
            b"other\0"
        };
        unsafe { xstrdup_u8(t.as_ptr().cast()).cast() }
    } else {
        std::ptr::null_mut()
    };
    unsafe { nvim_tv_set_string(rettv, s) };
}

/// "filecopy()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_filecopy"]
pub unsafe extern "C" fn rs_f_filecopy(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    if unsafe { rs_check_secure() } != 0
        || unsafe { tv_check_for_string_arg(argvars, 0) } == -1
        || unsafe { tv_check_for_string_arg(argvars, 1) } == -1
    {
        return;
    }
    let from_buf = unsafe { tv_to_cstring(argvars) };
    let mut file_info = [0u8; FILEINFO_SIZE];
    if unsafe { os_fileinfo_link(from_buf.as_ptr(), file_info.as_mut_ptr().cast()) } {
        let mode = unsafe { nvim_fileinfo_mode(file_info.as_ptr().cast()) };
        let fmt = mode & S_IFMT;
        if fmt == S_IFREG || fmt == S_IFLNK {
            let tv1 = unsafe { argvar_at(argvars, 1) };
            let mut nbuf = vec![0u8; NUMBUFLEN];
            let to_ptr = unsafe { nvim_tv_get_string_buf(tv1.as_ptr(), nbuf.as_mut_ptr()) };
            let result = unsafe { vim_copyfile(from_buf.as_ptr(), to_ptr) } == OK;
            rettv_set_bool(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
        }
    }
}

/// "delete()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_delete"]
pub unsafe extern "C" fn rs_f_delete(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, -1);
    if unsafe { rs_check_secure() } != 0 {
        return;
    }
    let name_buf = unsafe { tv_to_cstring(argvars) };
    if name_buf[0] == 0 {
        unsafe { fs_emsg(c"E475: Invalid argument".as_ptr()) };
        return;
    }
    let tv1 = unsafe { argvar_at(argvars, 1) };
    let tv1_type = unsafe { nvim_tv_get_type(tv1.as_ptr()) };
    let mut nbuf = vec![0u8; NUMBUFLEN];
    let flags_ptr = if tv1_type == VAR_UNKNOWN {
        c"".as_ptr()
    } else {
        unsafe { nvim_tv_get_string_buf(tv1.as_ptr(), nbuf.as_mut_ptr()).cast() }
    };
    let result = if unsafe { *flags_ptr == 0 } {
        // delete a file
        if unsafe { os_remove(name_buf.as_ptr()) } == 0 {
            0
        } else {
            -1
        }
    } else {
        let flags_slice = unsafe { std::ffi::CStr::from_ptr(flags_ptr).to_bytes() };
        if flags_slice == b"d" {
            // delete an empty directory
            if unsafe { os_rmdir(name_buf.as_ptr()) } == 0 {
                0
            } else {
                -1
            }
        } else if flags_slice == b"rf" {
            // delete recursively
            unsafe { delete_recursive(name_buf.as_ptr()) }
        } else {
            // Invalid flags - emit error and return current value (-1)
            unsafe { fs_semsg(c"E475: Invalid argument: %s".as_ptr(), flags_ptr) };
            return;
        }
    };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, i64::from(result));
}

/// "pathshorten()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_pathshorten"]
pub unsafe extern "C" fn rs_f_pathshorten(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    let tv1 = unsafe { argvar_at(argvars, 1) };
    let tv1_type = unsafe { nvim_tv_get_type(tv1.as_ptr()) };
    let trim_len = if tv1_type == VAR_UNKNOWN {
        1
    } else {
        let n = unsafe { nvim_tv_get_number(tv1.as_ptr()) };
        if n < 1 {
            1
        } else {
            n as c_int
        }
    };
    let s = unsafe { tv_to_cstring_chk(argvars) }.map_or(std::ptr::null_mut(), |p| {
        let dup = unsafe { xstrdup_u8(p.as_ptr().cast()) };
        if !dup.is_null() {
            unsafe { shorten_dir_len(dup.cast(), trim_len) };
        }
        dup.cast()
    });
    unsafe { nvim_tv_set_string(rettv, s) };
}

// =============================================================================
// Phase 3: Directory and filesystem navigation functions
// =============================================================================

// Typval layout mirrors: argvars[N] where each element is TYPVAL_SIZE=16 bytes.
// For typval fields we use argvar_at() from dispatch.
// For direct field access (v_type check, v_string etc) we use nvim_tv_get_*.

// VAR_STRING type value (from typval_defs.h)
const VAR_STRING: i32 = 6;
// VAR_NUMBER type value
const VAR_NUMBER: i32 = 2;

/// "chdir({dir} [, {scope}])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_chdir"]
pub unsafe extern "C" fn rs_f_chdir(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    // rettv->v_type = VAR_STRING; rettv->vval.v_string = NULL
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };

    let tv0 = unsafe { argvar_at(argvars, 0) };
    if unsafe { nvim_tv_get_type(tv0.as_ptr()) } != VAR_STRING {
        // Returning an empty string means it failed
        return;
    }

    // Get the current directory to return as the old cwd
    let cwd_buf = unsafe { xmalloc(MAXPATHL) };
    let old_cwd: *mut c_char = if unsafe { nvim_os_dirname(cwd_buf.cast(), MAXPATHL) } == OK {
        unsafe { xstrdup_c(cwd_buf.cast()) }
    } else {
        std::ptr::null_mut()
    };
    unsafe { xfree(cwd_buf) };

    // Determine scope
    let tv1 = unsafe { argvar_at(argvars, 1) };
    let tv1_type = unsafe { nvim_tv_get_type(tv1.as_ptr()) };
    let scope = if tv1_type == VAR_UNKNOWN {
        // No scope argument: use window local if set, else tab local, else global
        let curwin = unsafe { nvim_get_curwin() };
        let curtab = unsafe { nvim_get_curtab() };
        if !unsafe { nvim_win_get_localdir(curwin) }.is_null() {
            CD_SCOPE_WINDOW
        } else if !unsafe { nvim_tab_get_localdir(curtab) }.is_null() {
            CD_SCOPE_TABPAGE
        } else {
            CD_SCOPE_GLOBAL
        }
    } else {
        let mut nbuf = vec![0u8; NUMBUFLEN];
        let s = unsafe { nvim_tv_get_string_buf(tv1.as_ptr(), nbuf.as_mut_ptr()) };
        let scope_str = unsafe { std::ffi::CStr::from_ptr(s.cast()).to_bytes() };
        if scope_str == b"global" {
            CD_SCOPE_GLOBAL
        } else if scope_str == b"tabpage" {
            CD_SCOPE_TABPAGE
        } else if scope_str == b"window" {
            CD_SCOPE_WINDOW
        } else {
            unsafe { fs_semsg(c"E475: Invalid argument: %s".as_ptr(), s.cast::<c_char>()) };
            unsafe { xfree(old_cwd.cast()) };
            return;
        }
    };

    // Get the directory string from argvars[0]
    let dir_ptr = unsafe { nvim_tv_get_vstring(tv0.as_ptr()) };
    if unsafe { nvim_changedir_func(dir_ptr, scope) } {
        unsafe { nvim_tv_set_string(rettv, old_cwd.cast()) };
    } else {
        // Directory change failed - free old_cwd, return empty string
        unsafe { xfree(old_cwd.cast()) };
    }
}

/// "getcwd([{win}[, {tab}]])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_getcwd"]
pub unsafe extern "C" fn rs_f_getcwd(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };

    let mut scope = CD_SCOPE_INVALID;
    let mut scope_number = [0i32; 2]; // [window_idx, tabpage_idx]

    // Parse arguments: for i in MIN_CD_SCOPE..MAX_CD_SCOPE
    for i in MIN_CD_SCOPE..MAX_CD_SCOPE {
        let tv = unsafe { argvar_at(argvars, i as usize) };
        let tv_type = unsafe { nvim_tv_get_type(tv.as_ptr()) };
        if tv_type == VAR_UNKNOWN {
            break;
        }
        if tv_type != VAR_NUMBER {
            unsafe { fs_emsg(c"E475: Invalid argument".as_ptr()) };
            return;
        }
        let num = unsafe { nvim_tv_get_number(tv.as_ptr()) } as i32;
        if num < -1 {
            unsafe { fs_emsg(c"E475: Invalid argument".as_ptr()) };
            return;
        }
        scope_number[i as usize] = num;
        if num >= 0 && scope == CD_SCOPE_INVALID {
            scope = i;
        } else if num < 0 {
            scope = i + 1;
        }
    }

    // Find tabpage
    let mut tp = unsafe { nvim_get_curtab() };
    if scope_number[CD_SCOPE_TABPAGE as usize] > 0 {
        extern "C" {
            #[link_name = "rs_find_tabpage"]
            fn find_tabpage_by_num(n: c_int) -> *mut c_void;
        }
        tp = unsafe { find_tabpage_by_num(scope_number[CD_SCOPE_TABPAGE as usize]) };
        if tp.is_null() {
            unsafe { fs_emsg(c"E5000: Cannot find tab number.".as_ptr()) };
            return;
        }
    }

    // Find window
    let mut win = unsafe { nvim_get_curwin() };
    if scope_number[CD_SCOPE_WINDOW as usize] >= 0 {
        if scope_number[CD_SCOPE_TABPAGE as usize] < 0 {
            unsafe {
                fs_emsg(c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr())
            };
            return;
        }
        if scope_number[CD_SCOPE_WINDOW as usize] > 0 {
            let tv0 = unsafe { argvar_at(argvars, 0) };
            win = unsafe { nvim_find_win_by_nr(tv0.as_ptr(), tp) };
            if win.is_null() {
                unsafe { fs_emsg(c"E5002: Cannot find window number.".as_ptr()) };
                return;
            }
        }
    }

    let cwd_buf = unsafe { xmalloc(MAXPATHL) };
    let cwd: *mut c_char = cwd_buf.cast();

    let from: *const c_char = match scope {
        CD_SCOPE_WINDOW => {
            let local = unsafe { nvim_win_get_localdir(win) };
            if !local.is_null() {
                local
            } else {
                let tlocal = unsafe { nvim_tab_get_localdir(tp) };
                if !tlocal.is_null() {
                    tlocal
                } else {
                    let gdir = unsafe { nvim_get_globaldir() };
                    if !gdir.is_null() {
                        gdir
                    } else {
                        if unsafe { nvim_os_dirname(cwd, MAXPATHL) } == FAIL {
                            unsafe { *cwd = 0 };
                        }
                        std::ptr::null()
                    }
                }
            }
        }
        CD_SCOPE_TABPAGE => {
            let tlocal = unsafe { nvim_tab_get_localdir(tp) };
            if !tlocal.is_null() {
                tlocal
            } else {
                let gdir = unsafe { nvim_get_globaldir() };
                if !gdir.is_null() {
                    gdir
                } else {
                    if unsafe { nvim_os_dirname(cwd, MAXPATHL) } == FAIL {
                        unsafe { *cwd = 0 };
                    }
                    std::ptr::null()
                }
            }
        }
        CD_SCOPE_GLOBAL => {
            let gdir = unsafe { nvim_get_globaldir() };
            if !gdir.is_null() {
                gdir
            } else {
                if unsafe { nvim_os_dirname(cwd, MAXPATHL) } == FAIL {
                    unsafe { *cwd = 0 };
                }
                std::ptr::null()
            }
        }
        _ => {
            // CD_SCOPE_INVALID: called without arguments
            if unsafe { nvim_os_dirname(cwd, MAXPATHL) } == FAIL {
                unsafe { *cwd = 0 };
            }
            std::ptr::null()
        }
    };

    if !from.is_null() {
        unsafe { xstrlcpy(cwd, from, MAXPATHL) };
    }

    let result = unsafe { xstrdup_c(cwd) };
    unsafe { xfree(cwd_buf) };
    unsafe { nvim_tv_set_string(rettv, result.cast()) };
}

/// "haslocaldir([{win}[, {tab}]])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_haslocaldir"]
pub unsafe extern "C" fn rs_f_haslocaldir(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, 0);

    let mut scope = CD_SCOPE_INVALID;
    let mut scope_number = [0i32; 2];

    for i in MIN_CD_SCOPE..MAX_CD_SCOPE {
        let tv = unsafe { argvar_at(argvars, i as usize) };
        let tv_type = unsafe { nvim_tv_get_type(tv.as_ptr()) };
        if tv_type == VAR_UNKNOWN {
            break;
        }
        if tv_type != VAR_NUMBER {
            unsafe { fs_emsg(c"E475: Invalid argument".as_ptr()) };
            return;
        }
        let num = unsafe { nvim_tv_get_number(tv.as_ptr()) } as i32;
        if num < -1 {
            unsafe { fs_emsg(c"E475: Invalid argument".as_ptr()) };
            return;
        }
        scope_number[i as usize] = num;
        if num >= 0 && scope == CD_SCOPE_INVALID {
            scope = i;
        } else if num < 0 {
            scope = i + 1;
        }
    }

    // Default to window scope if not specified
    if scope == CD_SCOPE_INVALID {
        scope = MIN_CD_SCOPE;
    }

    // Find tabpage
    let mut tp = unsafe { nvim_get_curtab() };
    if scope_number[CD_SCOPE_TABPAGE as usize] > 0 {
        extern "C" {
            #[link_name = "rs_find_tabpage"]
            fn find_tabpage_by_num2(n: c_int) -> *mut c_void;
        }
        tp = unsafe { find_tabpage_by_num2(scope_number[CD_SCOPE_TABPAGE as usize]) };
        if tp.is_null() {
            unsafe { fs_emsg(c"E5000: Cannot find tab number.".as_ptr()) };
            return;
        }
    }

    // Find window
    let mut win = unsafe { nvim_get_curwin() };
    if scope_number[CD_SCOPE_WINDOW as usize] >= 0 {
        if scope_number[CD_SCOPE_TABPAGE as usize] < 0 {
            unsafe {
                fs_emsg(c"E5001: Higher scope cannot be -1 if lower scope is >= 0.".as_ptr())
            };
            return;
        }
        if scope_number[CD_SCOPE_WINDOW as usize] > 0 {
            let tv0 = unsafe { argvar_at(argvars, 0) };
            win = unsafe { nvim_find_win_by_nr(tv0.as_ptr(), tp) };
            if win.is_null() {
                unsafe { fs_emsg(c"E5002: Cannot find window number.".as_ptr()) };
                return;
            }
        }
    }

    let result = match scope {
        CD_SCOPE_WINDOW => i64::from(!unsafe { nvim_win_get_localdir(win) }.is_null()),
        CD_SCOPE_TABPAGE => i64::from(!unsafe { nvim_tab_get_localdir(tp) }.is_null()),
        CD_SCOPE_GLOBAL => 0, // Global scope never has a local directory
        _ => {
            // CD_SCOPE_INVALID: should never get here
            return;
        }
    };
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, result);
}

/// "mkdir({dir} [, {flags} [, {prot}]])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_mkdir"]
pub unsafe extern "C" fn rs_f_mkdir(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, 0); // FAIL = 0

    if unsafe { rs_check_secure() } != 0 {
        return;
    }

    let mut nbuf = vec![0u8; NUMBUFLEN];
    let tv0 = unsafe { argvar_at(argvars, 0) };
    let dir_ptr = unsafe { nvim_tv_get_string_buf(tv0.as_ptr(), nbuf.as_mut_ptr()) };
    if dir_ptr.is_null() || unsafe { *dir_ptr } == 0 {
        return;
    }

    // Remove trailing slashes: if path_tail points to NUL, set path_tail_with_sep to NUL
    let dir_c: *const c_char = dir_ptr.cast();
    let tail = unsafe { nvim_fs_path_tail(dir_c) };
    if unsafe { *tail } == 0 {
        // dir ends with slash - truncate at path_tail_with_sep
        let dir_dup = unsafe { xstrdup_c(dir_c) };
        let tail_with_sep = unsafe { nvim_path_tail_with_sep(dir_dup) };
        unsafe { *tail_with_sep = 0 };
        unsafe { do_mkdir_inner(dir_dup, argvars, rettv) };
        unsafe { xfree(dir_dup.cast()) };
        return;
    }

    unsafe { do_mkdir_inner(dir_c, argvars, rettv) };
}

unsafe fn do_mkdir_inner(dir: *const c_char, argvars: *const c_void, rettv: *mut c_void) {
    let mut prot: c_int = 0o755;

    let tv1 = unsafe { argvar_at(argvars, 1) };
    let tv1_type = unsafe { nvim_tv_get_type(tv1.as_ptr()) };

    let mut defer_flag = false;
    let mut defer_recurse = false;
    let mut do_recurse = false;

    if tv1_type != VAR_UNKNOWN {
        let tv2 = unsafe { argvar_at(argvars, 2) };
        let tv2_type = unsafe { nvim_tv_get_type(tv2.as_ptr()) };
        if tv2_type != VAR_UNKNOWN {
            let p = unsafe { nvim_tv_get_number(tv2.as_ptr()) } as c_int;
            if p == -1 {
                return;
            }
            prot = p;
        }
        let mut nbuf2 = vec![0u8; NUMBUFLEN];
        let arg2 = unsafe { nvim_tv_get_string_buf(tv1.as_ptr(), nbuf2.as_mut_ptr()) };
        let arg2_c: *const c_char = arg2.cast();
        defer_flag = !unsafe { nvim_vim_strchr(arg2_c, c_int::from(b'D')) }.is_null();
        defer_recurse = !unsafe { nvim_vim_strchr(arg2_c, c_int::from(b'R')) }.is_null();

        if (defer_flag || defer_recurse) && !unsafe { nvim_can_add_defer() } {
            return;
        }

        if !unsafe { nvim_vim_strchr(arg2_c, c_int::from(b'p')) }.is_null() {
            do_recurse = true;
        }
    }

    let mut created: *mut c_char = std::ptr::null_mut();

    if do_recurse {
        let mut failed_dir: *mut c_char = std::ptr::null_mut();
        let created_ptr = if defer_flag || defer_recurse {
            &raw mut created
        } else {
            std::ptr::null_mut()
        };
        let ret =
            unsafe { nvim_fs_mkdir_recurse(dir, prot as u32, &raw mut failed_dir, created_ptr) };
        if ret != 0 {
            let err_str = unsafe { nvim_os_strerror(ret) };
            unsafe { fs_semsg(c"E739: Cannot create directory %s: %s".as_ptr(), err_str) };
            unsafe { xfree(failed_dir.cast()) };
            return;
        }
        rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, 1); // OK
    } else {
        let result = unsafe { nvim_vim_mkdir_emsg(dir, prot) };
        rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, i64::from(result));
    }

    // Handle "D" and "R": deferred deletion of the created directory
    let is_ok = unsafe { nvim_tv_get_number(rettv.cast()) } == 1;
    if is_ok && created.is_null() && (defer_flag || defer_recurse) {
        created = unsafe { nvim_FullName_save(dir, false) };
    }
    if !created.is_null() {
        // Build typval_T[2] for add_defer("delete", 2, tv)
        // Each typval_T is TYPVAL_SIZE=16 bytes.
        // Layout: v_type(i32) + v_lock(i32) + v_string(*mut c_char) [+ padding to 16]
        let typval_size = 16usize;
        let mut tv_buf = vec![0u8; typval_size * 2];
        let tv0_ptr = tv_buf.as_mut_ptr();
        unsafe {
            std::ptr::write_unaligned(tv0_ptr.cast::<i32>(), VAR_STRING);
            std::ptr::write_unaligned(tv0_ptr.add(4).cast::<i32>(), 0); // VAR_UNLOCKED
            std::ptr::write_unaligned(tv0_ptr.add(8).cast::<*mut c_char>(), created);
        }
        let suffix = if defer_recurse {
            unsafe { xstrdup_c(c"rf".as_ptr()) }
        } else {
            unsafe { xstrdup_c(c"d".as_ptr()) }
        };
        let tv1_ptr = unsafe { tv0_ptr.add(typval_size) };
        unsafe {
            std::ptr::write_unaligned(tv1_ptr.cast::<i32>(), VAR_STRING);
            std::ptr::write_unaligned(tv1_ptr.add(4).cast::<i32>(), 0); // VAR_UNLOCKED
            std::ptr::write_unaligned(tv1_ptr.add(8).cast::<*mut c_char>(), suffix);
        }
        unsafe { nvim_add_defer_delete(tv_buf.as_mut_ptr().cast(), 2) };
    }
}

/// Readdir callback: calls nvim_eval_readdir_expr with context = typval_T*.
unsafe extern "C" fn readdir_checkitem_trampoline(
    context: *mut c_void,
    name: *const c_char,
) -> i64 {
    unsafe { nvim_eval_readdir_expr(context, name) }
}

/// "readdir({path} [, {expr}])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_readdir"]
pub unsafe extern "C" fn rs_f_readdir(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe { nvim_fs_tv_list_alloc_ret(rettv, -1) }; // kListLenUnknown = -1

    let path_buf = unsafe { tv_to_cstring(argvars) };
    let expr_tv = unsafe { argvar_at(argvars, 1) };

    let mut ga = GarrayT::zeroed();
    let ret = unsafe {
        nvim_readdir_core(
            &raw mut ga,
            path_buf.as_ptr().cast(),
            expr_tv.as_ptr().cast_mut(),
            readdir_checkitem_trampoline,
        )
    };

    if ret == OK && unsafe { nvim_fs_ga_get_len(&raw const ga) } > 0 {
        let list = unsafe { nvim_tv_get_list_from_rettv(rettv) };
        let len = unsafe { nvim_fs_ga_get_len(&raw const ga) };
        for i in 0..len {
            let s = unsafe { nvim_fs_ga_get_str(&raw const ga, i) };
            unsafe { nvim_tv_list_append_string(list, s, -1) };
        }
    }
    unsafe { nvim_ga_clear_strings(&raw mut ga) };
}
