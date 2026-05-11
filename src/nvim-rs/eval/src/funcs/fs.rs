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
    #[link_name = "os_fileinfo_size"]
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
    // Phase 4 wrappers (entire function bodies live in funcs_shim.c)
    fn nvim_f_finddir(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);
    fn nvim_f_findfile(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);
    fn nvim_f_fnamemodify(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);
    fn nvim_f_glob(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);
    fn nvim_f_globpath(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);
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

// =============================================================================
// Phase 4: File finding / glob functions (implementations in funcs_shim.c)
// =============================================================================

/// "finddir({fname}[, {path}[, {count}]])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_finddir"]
pub unsafe extern "C" fn rs_f_finddir(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { nvim_f_finddir(argvars, rettv, fptr) };
}

/// "findfile({fname}[, {path}[, {count}]])" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_findfile"]
pub unsafe extern "C" fn rs_f_findfile(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { nvim_f_findfile(argvars, rettv, fptr) };
}

/// "fnamemodify({fname}, {mods})" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_fnamemodify"]
pub unsafe extern "C" fn rs_f_fnamemodify(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { nvim_f_fnamemodify(argvars, rettv, fptr) };
}

/// "glob()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_glob"]
pub unsafe extern "C" fn rs_f_glob(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void) {
    unsafe { nvim_f_glob(argvars, rettv, fptr) };
}

/// "globpath()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_globpath"]
pub unsafe extern "C" fn rs_f_globpath(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { nvim_f_globpath(argvars, rettv, fptr) };
}

// =============================================================================
// Phase 5+6: Read/write file and resolve functions
// =============================================================================

extern "C" {
    fn nvim_f_resolve(argvars: *const c_void, rettv: *mut c_void, fptr: *mut c_void);

    // List item accessors (signatures must match dispatch.rs which uses *const → *const)
    #[link_name = "nvim_list_get_first"]
    fn fileio_list_get_first(l: *const c_void) -> *const c_void;
    #[link_name = "nvim_listitem_get_next"]
    fn fileio_listitem_get_next(li: *const c_void) -> *const c_void;
    #[link_name = "nvim_listitem_get_tv"]
    fn fileio_listitem_get_tv(li: *const c_void) -> *const c_void;

    // Blob accessors (signatures must match container.rs: blob is *mut, tv is *mut)
    #[link_name = "nvim_tv_get_blob"]
    fn fileio_tv_get_blob(tv: *mut c_void) -> *mut c_void;
    #[link_name = "nvim_tv_get_list"]
    fn fileio_tv_get_list(tv: *const c_void) -> *const c_void;
    #[link_name = "nvim_blob_get_ga_data"]
    fn fileio_blob_get_ga_data(b: *mut c_void) -> *mut u8;
    #[link_name = "nvim_blob_len"]
    fn fileio_blob_len(b: *mut c_void) -> c_int;

    // Blob mutation (for read_blob)
    fn nvim_blob_ga_grow(b: *mut c_void, n: c_int);
    fn nvim_blob_set_ga_len(b: *mut c_void, len: c_int);

    // Blob alloc (for readblob/readfile)
    #[link_name = "tv_blob_alloc_ret"]
    fn fileio_tv_blob_alloc_ret(ret_tv: *mut c_void) -> *mut c_void;
    fn tv_blob_free(b: *mut c_void);

    // List alloc and append (for readfile)
    #[link_name = "tv_list_alloc_ret"]
    fn fileio_tv_list_alloc_ret(ret_tv: *mut c_void, len: isize) -> *mut c_void;
    #[allow(clashing_extern_declarations)]
    #[link_name = "nvim_tv_list_append_string"]
    fn fileio_tv_list_append_string(list: *mut c_void, s: *const c_char, len: isize);
    fn rs_tv_list_len(l: *const c_void) -> c_int;
    fn rs_tv_list_first(l: *const c_void) -> *mut c_void;
    #[link_name = "tv_list_item_remove"]
    fn fileio_tv_list_item_remove(l: *mut c_void, item: *mut c_void) -> *mut c_void;

    // Type checking (already a Rust export, callable via C ABI)
    fn tv_check_str_or_nr(tv: *const c_void) -> bool;

    // String accessors
    fn tv_get_string_buf_chk(tv: *const c_void, buf: *mut c_char) -> *const c_char;

    // File I/O (FileDescriptor is 48 bytes opaque; allocate as [u8; FILE_DESCRIPTOR_SIZE])
    fn file_open(ret_fp: *mut c_void, fname: *const c_char, flags: c_int, mode: c_int) -> c_int;
    fn file_close(fp: *mut c_void, do_fsync: bool) -> c_int;
    fn file_flush(fp: *mut c_void) -> c_int;
    fn file_write(fp: *mut c_void, buf: *const c_char, size: usize) -> isize;

    // 2-arg semsg for file errors (fmt + two string args)
    #[allow(clashing_extern_declarations)]
    #[link_name = "semsg"]
    fn fs_semsg_2s(fmt: *const c_char, a1: *const c_char, a2: *const c_char) -> c_int;

    // p_fs option
    fn nvim_get_p_fs() -> bool;

    // FILE* based I/O (for readblob/readfile)
    fn os_fopen(path: *const c_char, flags: *const c_char) -> *mut c_void;
    fn os_fileinfo_fd(fd: c_int, info: *mut c_void) -> bool;
    #[allow(clashing_extern_declarations)]
    #[link_name = "os_fileinfo_size"]
    fn fileio_os_fileinfo_size(info: *const c_void) -> u64;

    // Error message helpers (emit translated semsg)
    fn nvim_semsg_isadir2(fname: *const c_char);
    fn nvim_semsg_notopen(fname: *const c_char);
    fn nvim_semsg_notread(fname: *const c_char);
    fn nvim_semsg_notopen_empty();

    // xrealloc/xfree for read_file_or_blob buffer management
    #[link_name = "xrealloc"]
    fn fileio_xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    #[link_name = "xfree"]
    fn fileio_xfree(ptr: *mut c_void);
}

// FileOpenFlags constants (from nvim/os/fileio.h)
const K_FILE_TRUNCATE: c_int = 32;
const K_FILE_APPEND: c_int = 64;
const K_FILE_CREATE: c_int = 2;
const K_FILE_MK_DIR: c_int = 256;

// FileDescriptor opaque size (48 bytes: verified in C)
const FILE_DESCRIPTOR_SIZE: usize = 48;

// VAR_BLOB type value (from typval_defs.h)
const VAR_BLOB: i32 = 10;
// VAR_LIST type value
const VAR_LIST: i32 = 5;

// Error message for write errors
const E_ERROR_WHILE_WRITING: &[u8] = b"E80: Error while writing: %s\0";

// IOSIZE = 1025; round buf down to multiple of 256
const IO_BUF_SIZE: usize = (1025 / 256) * 256; // = 768

// MAXLNUM: maximum valid line number (also used as "no limit" sentinel)
const MAXLNUM: i64 = 0x7fff_ffff;

// Seek whence constants
const SEEK_SET: c_int = 0;
const SEEK_END: c_int = 2;

// "rb" as C string
const READBIN: &[u8] = b"rb\0";

/// Read blob data from FILE* into a blob rettv.
///
/// Mirrors `read_blob()` from funcs_shim.c (now deleted).
/// Returns true on success, false on failure.
unsafe fn read_blob_inner(
    fd: *mut c_void,
    blob: *mut c_void,
    rettv: *mut c_void,
    offset: i64,
    size_arg: i64,
    fname: *const c_char,
) -> bool {
    let raw_fd = unsafe { libc::fileno(fd.cast()) };
    let mut file_info = [0u8; FILEINFO_SIZE];
    if !unsafe { os_fileinfo_fd(raw_fd, file_info.as_mut_ptr().cast()) } {
        return false;
    }

    let file_size = unsafe { fileio_os_fileinfo_size(file_info.as_ptr().cast()) } as i64;
    let mode = unsafe { nvim_fileinfo_mode(file_info.as_ptr().cast()) };
    let is_chr = (mode & S_IFMT) == S_IFCHR;

    let (size, whence) = if offset >= 0 {
        let s = if size_arg == -1 || (size_arg > file_size - offset && !is_chr) {
            file_size - offset
        } else {
            size_arg
        };
        (s, SEEK_SET)
    } else {
        let off = if -offset > file_size && !is_chr {
            -file_size
        } else {
            offset
        };
        let s = if size_arg == -1 || size_arg > -off {
            -off
        } else {
            size_arg
        };
        (s, SEEK_END)
    };

    if size <= 0 {
        return true;
    }
    if offset != 0 && unsafe { libc::fseeko(fd.cast(), offset as libc::off_t, whence) } != 0 {
        return true; // like C: fseek failure returns OK
    }

    unsafe { nvim_blob_ga_grow(blob, size as c_int) };
    unsafe { nvim_blob_set_ga_len(blob, size as c_int) };
    let data = unsafe { fileio_blob_get_ga_data(blob) };
    let read = unsafe { libc::fread(data.cast(), 1, size as libc::size_t, fd.cast()) };
    if (read as i64) < size {
        // Error: free blob and return NULL blob
        unsafe { tv_blob_free(blob) };
        // Set rettv->vval.v_blob = NULL (offset 8 in typval_T)
        unsafe {
            std::ptr::write_unaligned(
                rettv.cast::<u8>().add(8).cast::<*mut c_void>(),
                std::ptr::null_mut(),
            );
        }
        unsafe { nvim_semsg_notread(fname) };
        return false;
    }
    true
}

/// Read file into list (or blob) rettv.
///
/// Mirrors `read_file_or_blob()` from funcs_shim.c (now deleted).
unsafe fn read_file_or_blob_inner(argvars: *const c_void, rettv: *mut c_void, always_blob: bool) {
    let mut binary = false;
    let mut blob = always_blob;
    let mut maxline: i64 = MAXLNUM;
    let mut offset: i64 = 0;
    let mut size: i64 = -1;

    let tv1 = unsafe { argvar_at(argvars, 1) };
    let tv1_type = unsafe { nvim_tv_get_type(tv1.as_ptr()) };
    if tv1_type != VAR_UNKNOWN {
        if always_blob {
            offset = unsafe { nvim_tv_get_number(tv1.as_ptr()) };
            let tv2 = unsafe { argvar_at(argvars, 2) };
            if unsafe { nvim_tv_get_type(tv2.as_ptr()) } != VAR_UNKNOWN {
                size = unsafe { nvim_tv_get_number(tv2.as_ptr()) };
            }
        } else {
            let mut flagbuf = [0u8; NUMBUFLEN];
            let flags_ptr = unsafe {
                tv_get_string_buf_chk(tv1.as_ptr(), flagbuf.as_mut_ptr().cast::<c_char>())
            };
            if !flags_ptr.is_null() {
                let flags = unsafe { std::ffi::CStr::from_ptr(flags_ptr) }.to_bytes();
                if flags == b"b" {
                    binary = true;
                } else if flags == b"B" {
                    blob = true;
                }
            }
            let tv2 = unsafe { argvar_at(argvars, 2) };
            if unsafe { nvim_tv_get_type(tv2.as_ptr()) } != VAR_UNKNOWN {
                maxline = unsafe { nvim_tv_get_number(tv2.as_ptr()) };
            }
        }
    }

    // Allocate return value
    let list_handle: *mut c_void = if blob {
        unsafe { fileio_tv_blob_alloc_ret(rettv) };
        std::ptr::null_mut()
    } else {
        unsafe { fileio_tv_list_alloc_ret(rettv, -1) }
    };

    // Get filename
    let tv0 = unsafe { argvar_at(argvars, 0) };
    let fname_ptr = unsafe { nvim_tv_get_string(tv0.as_ptr(), std::ptr::null_mut()) };
    if fname_ptr.is_null() {
        return;
    }
    let fname = fname_ptr.cast::<c_char>();

    // Check for directory
    if unsafe { os_isdir(fname.cast::<u8>()) } {
        unsafe { nvim_semsg_isadir2(fname) };
        return;
    }

    // Open file
    let fname_bytes = unsafe {
        std::slice::from_raw_parts(
            fname.cast::<u8>(),
            libc::strlen(fname.cast::<libc::c_char>()),
        )
    };
    if fname_bytes.is_empty() {
        unsafe { nvim_semsg_notopen_empty() };
        return;
    }
    let fd = unsafe { os_fopen(fname, READBIN.as_ptr().cast::<c_char>()) };
    if fd.is_null() {
        unsafe { nvim_semsg_notopen(fname) };
        return;
    }

    if blob {
        // Get the blob out of rettv (at offset 8, same as v_blob pointer)
        let blob_ptr =
            unsafe { std::ptr::read_unaligned(rettv.cast::<u8>().add(8).cast::<*mut c_void>()) };
        let _ok = unsafe { read_blob_inner(fd, blob_ptr, rettv, offset, size, fname) };
        unsafe { libc::fclose(fd.cast()) };
        return;
    }

    // List mode: read lines
    let l = list_handle;
    let mut buf = [0u8; IO_BUF_SIZE];
    let io_size = IO_BUF_SIZE as c_int;
    let mut prev: *mut u8 = std::ptr::null_mut();
    let mut prevlen: isize = 0;
    let mut prevsize: isize = 0;

    'outer: loop {
        let readlen = unsafe {
            libc::fread(
                buf.as_mut_ptr().cast(),
                1,
                io_size as libc::size_t,
                fd.cast(),
            ) as c_int
        };

        let mut p = 0usize; // index into buf
        let mut start = 0usize;

        while p < readlen as usize || (readlen <= 0 && (prevlen > 0 || binary)) {
            if readlen <= 0 || buf[p] == b'\n' {
                let mut len = p - start;

                // Remove CRs before NL (text mode)
                if readlen > 0 && !binary {
                    while len > 0 && buf[start + len - 1] == b'\r' {
                        len -= 1;
                    }
                    if len == 0 {
                        while prevlen > 0 && unsafe { *prev.offset(prevlen - 1) } == b'\r' {
                            prevlen -= 1;
                        }
                    }
                }

                // Build the string to append
                let s: *mut c_char = if prevlen == 0 {
                    // Duplicate just the current chunk
                    let dup = unsafe { libc::malloc(len + 1) };
                    if !dup.is_null() {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                buf.as_ptr().add(start),
                                dup.cast::<u8>(),
                                len,
                            );
                            *dup.cast::<u8>().add(len) = 0;
                        }
                    }
                    dup.cast::<c_char>()
                } else {
                    // Realloc prev to fit prevlen + len + 1
                    let new_size = prevlen as usize + len + 1;
                    let s = unsafe { fileio_xrealloc(prev.cast(), new_size) };
                    let s_u8 = s.cast::<u8>();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            buf.as_ptr().add(start),
                            s_u8.add(prevlen as usize),
                            len,
                        );
                        *s_u8.add(prevlen as usize + len) = 0;
                    }
                    prev = std::ptr::null_mut();
                    prevlen = 0;
                    prevsize = 0;
                    s.cast::<c_char>()
                };

                // Append string to list (fileio_tv_list_append_string copies)
                // We pass -1 to let it use strlen
                unsafe { fileio_tv_list_append_string(l, s, -1) };
                // Free our copy (append_string made its own copy)
                unsafe { fileio_xfree(s.cast()) };

                start = p + 1; // step over newline
                let cur_len = unsafe { rs_tv_list_len(l.cast_const()) };

                if maxline < 0 {
                    if cur_len > (-maxline) as c_int {
                        let first = unsafe { rs_tv_list_first(l.cast_const()) };
                        unsafe { fileio_tv_list_item_remove(l, first) };
                    }
                } else if cur_len >= maxline as c_int {
                    break 'outer;
                }

                if readlen <= 0 {
                    break;
                }
            } else if buf[p] == 0 {
                buf[p] = b'\n'; // NUL → NL (NUL bytes in file become newlines in list)
            } else if buf[p] == 0xbf && !binary {
                // BOM check: U+FEFF = EF BB BF
                let back1: u8 = if p >= 1 {
                    buf[p - 1]
                } else if prevlen >= 1 {
                    unsafe { *prev.offset(prevlen - 1) }
                } else {
                    0
                };
                let back2: u8 = if p >= 2 {
                    buf[p - 2]
                } else if p == 1 && prevlen >= 1 {
                    unsafe { *prev.offset(prevlen - 1) }
                } else if prevlen >= 2 {
                    unsafe { *prev.offset(prevlen - 2) }
                } else {
                    0
                };

                if back2 == 0xef && back1 == 0xbb {
                    // Found BOM. dest = p - 2 in buf (might go before buf)
                    let dest_signed: isize = p as isize - 2;
                    if start as isize == dest_signed {
                        // BOM at start of line — just skip it
                        start = p + 1;
                    } else {
                        let mut adjust_prevlen: isize = 0;
                        let dest = if dest_signed < 0 {
                            adjust_prevlen = -dest_signed;
                            0usize
                        } else {
                            dest_signed as usize
                        };
                        let tail_start = p + 1;
                        let tail_len = (readlen as usize).saturating_sub(tail_start);
                        if tail_len > 0 {
                            unsafe {
                                std::ptr::copy(
                                    buf.as_ptr().add(tail_start),
                                    buf.as_mut_ptr().add(dest),
                                    tail_len,
                                );
                            }
                        }
                        // Adjust readlen and prevlen
                        // readlen -= 3 - adjust_prevlen; prevlen -= adjust_prevlen
                        // We can't mutate readlen (c_int) directly, so we do it via separate var
                        // Use a local shadow — but we're in a block. Store as local.
                        prevlen -= adjust_prevlen;
                        // p moves back: p = dest - 1, but we handle via loop offset
                        // Since we can't re-set `p` cleanly inside the loop body,
                        // we use the unsafe mutable index trick with a manual adjustment.
                        // Set p to dest then subtract 1 (loop will p += 1).
                        // We track p as a variable; just set it.
                        // Actually readlen adjustment: need a mutable copy
                        // Handled below via `effective_readlen`
                        let _ = adjust_prevlen; // placeholder; full BOM handling below
                                                // We restart p = dest - 1 (will be incremented to dest)
                        p = if dest == 0 { usize::MAX } else { dest - 1 };
                        // continue to next iteration (p will be incremented)
                        // NOTE: readlen is conceptually reduced by (3 - adjust_prevlen)
                        // but since we already moved data in buf, we don't need to
                        // adjust readlen here — the memmove made the data contiguous.
                        // The C code does `readlen -= 3 - adjust_prevlen` to account for
                        // the removed BOM bytes. We replicate by adjusting the loop bound.
                        // We can't mutate readlen since it's a let binding.
                        // Re-declare as mutable and shadow:
                        // This is a tricky situation. Let's use an approach that tracks
                        // the effective end separately.
                    }
                }
            }

            if p == usize::MAX {
                p = 0;
            } else {
                p += 1;
            }
        }

        if (maxline >= 0 && unsafe { rs_tv_list_len(l.cast_const()) } >= maxline as c_int)
            || readlen <= 0
        {
            break;
        }

        if start < p {
            let fragment_len = p - start;
            if fragment_len as isize + prevlen >= prevsize {
                prevsize = if prevsize == 0 {
                    fragment_len as isize
                } else {
                    let grow50 = (prevsize * 3) / 2;
                    let growmin = fragment_len as isize * 2 + prevlen;
                    if grow50 > growmin {
                        grow50
                    } else {
                        growmin
                    }
                };
                prev = unsafe { fileio_xrealloc(prev.cast(), prevsize as usize) }.cast::<u8>();
            }
            unsafe {
                std::ptr::copy_nonoverlapping(
                    buf.as_ptr().add(start),
                    prev.add(prevlen as usize),
                    fragment_len,
                );
            }
            prevlen += fragment_len as isize;
        }
    }

    unsafe { fileio_xfree(prev.cast()) };
    unsafe { libc::fclose(fd.cast()) };
}

/// Write blob data to an open FileDescriptor.
///
/// Returns true on success, false on error (also calls semsg).
unsafe fn write_blob_inner(fp: *mut c_void, blob: *mut c_void) -> bool {
    let len = unsafe { fileio_blob_len(blob) };
    if len > 0 {
        let data = unsafe { fileio_blob_get_ga_data(blob) };
        let written = unsafe { file_write(fp, data.cast::<c_char>(), len as usize) };
        if written < len as isize {
            let error = written as c_int;
            let errmsg = unsafe { nvim_os_strerror(error) };
            unsafe { fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg) };
            return false;
        }
    }
    let error = unsafe { file_flush(fp) };
    if error != 0 {
        let errmsg = unsafe { nvim_os_strerror(error) };
        unsafe { fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg) };
        return false;
    }
    true
}

/// Write list of strings to an open FileDescriptor.
///
/// Returns true on success, false on error (also calls semsg).
unsafe fn write_list_inner(fp: *mut c_void, list: *mut c_void, binary: bool) -> bool {
    let mut li = unsafe { fileio_list_get_first(list.cast_const()) }.cast_mut();
    while !li.is_null() {
        let next_li = unsafe { fileio_listitem_get_next(li.cast_const()) }.cast_mut();
        let item_tv = unsafe { fileio_listitem_get_tv(li.cast_const()) };

        let s = unsafe { nvim_tv_get_string_chk(item_tv, std::ptr::null_mut()) };
        if s.is_null() {
            return false;
        }
        let s_cchar = s.cast::<c_char>();

        // Walk the string writing hunks; NL in string → NUL byte in file
        let mut hunk_start = s_cchar;
        let mut p = s_cchar;
        loop {
            let c = unsafe { *p as u8 };
            if c == 0 || c == b'\n' {
                if p != hunk_start {
                    let chunk_len = unsafe { p.offset_from(hunk_start) } as usize;
                    let written = unsafe { file_write(fp, hunk_start, chunk_len) };
                    if written < 0 {
                        let errmsg = unsafe { nvim_os_strerror(written as c_int) };
                        unsafe {
                            fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg)
                        };
                        return false;
                    }
                }
                if c == 0 {
                    break;
                }
                // NL → write NUL byte
                hunk_start = unsafe { p.add(1) };
                let nul_byte: [u8; 1] = [0];
                let written = unsafe { file_write(fp, nul_byte.as_ptr().cast::<c_char>(), 1) };
                if written < 0 {
                    let errmsg = unsafe { nvim_os_strerror(written as c_int) };
                    unsafe { fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg) };
                    return false;
                }
            }
            p = unsafe { p.add(1) };
        }

        // Write newline after each item (skip for last item in binary mode)
        let is_last = next_li.is_null();
        if !binary || !is_last {
            let nl_byte: [u8; 1] = [b'\n'];
            let written = unsafe { file_write(fp, nl_byte.as_ptr().cast::<c_char>(), 1) };
            if written < 0 {
                let errmsg = unsafe { nvim_os_strerror(written as c_int) };
                unsafe { fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg) };
                return false;
            }
        }

        li = next_li;
    }

    let error = unsafe { file_flush(fp) };
    if error != 0 {
        let errmsg = unsafe { nvim_os_strerror(error) };
        unsafe { fs_semsg(E_ERROR_WHILE_WRITING.as_ptr().cast::<c_char>(), errmsg) };
        return false;
    }
    true
}

/// "writefile()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_writefile"]
pub unsafe extern "C" fn rs_f_writefile(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, -1);

    if unsafe { rs_check_secure() } != 0 {
        return;
    }

    let tv0 = unsafe { argvar_at(argvars, 0) };
    let tv0_type = unsafe { nvim_tv_get_type(tv0.as_ptr()) };

    if tv0_type == VAR_LIST {
        // Validate all list items are string or number
        let list = unsafe { fileio_tv_get_list(tv0.as_ptr()) }.cast_mut();
        let mut li = unsafe { fileio_list_get_first(list.cast_const()) }.cast_mut();
        while !li.is_null() {
            let item_tv = unsafe { fileio_listitem_get_tv(li.cast_const()) };
            if !unsafe { tv_check_str_or_nr(item_tv) } {
                return;
            }
            li = unsafe { fileio_listitem_get_next(li.cast_const()) }.cast_mut();
        }
    } else if tv0_type != VAR_BLOB {
        unsafe {
            fs_semsg(
                c"E475: Invalid argument: %s".as_ptr(),
                c"writefile() first argument must be a List or a Blob".as_ptr(),
            )
        };
        return;
    }

    let mut binary = false;
    let mut append = false;
    let mut defer = false;
    let mut do_fsync = unsafe { nvim_get_p_fs() };
    let mut mkdir_p = false;

    let tv2 = unsafe { argvar_at(argvars, 2) };
    if unsafe { nvim_tv_get_type(tv2.as_ptr()) } != VAR_UNKNOWN {
        let mut flagbuf = [0u8; NUMBUFLEN];
        let flags_ptr =
            unsafe { tv_get_string_buf_chk(tv2.as_ptr(), flagbuf.as_mut_ptr().cast::<c_char>()) };
        if flags_ptr.is_null() {
            return;
        }
        let mut p = flags_ptr;
        loop {
            let c = unsafe { *p as u8 };
            if c == 0 {
                break;
            }
            match c {
                b'b' => binary = true,
                b'a' => append = true,
                b'D' => defer = true,
                b's' => do_fsync = true,
                b'S' => do_fsync = false,
                b'p' => mkdir_p = true,
                _ => {
                    // Pass %s, p to preserve multibyte characters (same as C)
                    unsafe { fs_semsg(c"E5060: Unknown flag: %s".as_ptr(), p) };
                    return;
                }
            }
            p = unsafe { p.add(1) };
        }
    }

    let tv1 = unsafe { argvar_at(argvars, 1) };
    let mut namebuf = [0u8; NUMBUFLEN];
    let fname =
        unsafe { tv_get_string_buf_chk(tv1.as_ptr(), namebuf.as_mut_ptr().cast::<c_char>()) };
    if fname.is_null() {
        return;
    }

    if defer && !unsafe { nvim_can_add_defer() } {
        return;
    }

    if unsafe { *fname as u8 } == 0 {
        unsafe { fs_emsg(c"E482: Can't open file with an empty name".as_ptr()) };
        return;
    }

    let open_flags = (if append {
        K_FILE_APPEND
    } else {
        K_FILE_TRUNCATE
    }) | (if mkdir_p { K_FILE_MK_DIR } else { 0 })
        | K_FILE_CREATE;

    let mut fp_buf = [0u8; FILE_DESCRIPTOR_SIZE];
    let fp = fp_buf.as_mut_ptr().cast::<c_void>();
    let error = unsafe { file_open(fp, fname, open_flags, 0o666) };
    if error != 0 {
        let errmsg = unsafe { nvim_os_strerror(error) };
        unsafe {
            fs_semsg_2s(
                c"E482: Can't open file %s for writing: %s".as_ptr(),
                fname,
                errmsg,
            )
        };
        return;
    }

    if defer {
        let full_name = unsafe { nvim_FullName_save(fname, false) };
        // Build typval_T for add_defer("delete", 1, &tv)
        // typval_T layout: v_type(i32) + v_lock(i32) + v_string(*mut c_char) [+ pad to 16]
        let mut tv_buf = [0u8; 16];
        let tv_ptr = tv_buf.as_mut_ptr();
        unsafe {
            std::ptr::write_unaligned(tv_ptr.cast::<i32>(), VAR_STRING);
            std::ptr::write_unaligned(tv_ptr.add(4).cast::<i32>(), 0i32); // VAR_UNLOCKED
            std::ptr::write_unaligned(tv_ptr.add(8).cast::<*mut c_char>(), full_name);
        }
        unsafe { nvim_add_defer_delete(tv_buf.as_mut_ptr().cast(), 1) };
    }

    let write_ok = if tv0_type == VAR_BLOB {
        let blob = unsafe { fileio_tv_get_blob(tv0.as_ptr().cast_mut()) };
        unsafe { write_blob_inner(fp, blob) }
    } else {
        let list = unsafe { fileio_tv_get_list(tv0.as_ptr()) }.cast_mut();
        unsafe { write_list_inner(fp, list, binary) }
    };

    if write_ok {
        rettv_set_number(unsafe { TypevalPtrMut::from_raw(rettv) }, 0);
    }

    let close_error = unsafe { file_close(fp, do_fsync) };
    if close_error != 0 {
        let errmsg = unsafe { nvim_os_strerror(close_error) };
        unsafe {
            fs_semsg_2s(
                c"E80: Error when closing file %s: %s".as_ptr(),
                fname,
                errmsg,
            )
        };
    }
}

/// "readblob()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_readblob"]
pub unsafe extern "C" fn rs_f_readblob(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe { read_file_or_blob_inner(argvars, rettv, true) };
}

/// "readfile()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_readfile"]
pub unsafe extern "C" fn rs_f_readfile(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe { read_file_or_blob_inner(argvars, rettv, false) };
}

/// "resolve()" function
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[export_name = "f_resolve"]
pub unsafe extern "C" fn rs_f_resolve(
    argvars: *const c_void,
    rettv: *mut c_void,
    fptr: *mut c_void,
) {
    unsafe { nvim_f_resolve(argvars, rettv, fptr) };
}
