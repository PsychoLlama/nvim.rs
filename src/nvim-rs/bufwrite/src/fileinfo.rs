//! File info and permission checking for buffer write operations.
//!
//! Mirrors the C `check_mtime`, `get_fileinfo_os`, and `get_fileinfo` functions.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_lossless)]

use std::ffi::{c_char, c_int};

use crate::error::BwError;
use crate::ffi::{BufHandle, FileInfoHandle, FAIL, OK};

// Node type constants (from os/fs_defs.h)
#[allow(dead_code)]
const NODE_NORMAL: c_int = 0;
const NODE_WRITABLE: c_int = 1;
const NODE_OTHER: c_int = 2;

extern "C" {
    // OS functions
    fn nvim_bw_os_fileinfo(fname: *const c_char, info: FileInfoHandle) -> c_int;
    fn nvim_bw_os_nodetype(fname: *const c_char) -> c_int;
    #[cfg(not(unix))]
    fn nvim_bw_os_getperm(fname: *const c_char) -> c_int;
    #[cfg(not(unix))]
    fn nvim_bw_os_isdir(fname: *const c_char) -> c_int;
    fn nvim_bw_os_file_is_writable(fname: *const c_char) -> c_int;

    // FileInfo field accessors
    #[cfg(unix)]
    fn nvim_bw_fi_get_st_mode(info: FileInfoHandle) -> c_int;
    #[cfg(unix)]
    fn nvim_bw_fi_is_regular(info: FileInfoHandle) -> c_int;
    #[cfg(unix)]
    fn nvim_bw_fi_is_dir(info: FileInfoHandle) -> c_int;

    // Buffer mtime
    fn nvim_bw_buf_get_mtime_read(buf: BufHandle) -> i64;
    fn nvim_bw_buf_get_mtime_read_ns(buf: BufHandle) -> i64;
    fn nvim_bw_time_differs(info: FileInfoHandle, mtime: i64, mtime_ns: i64) -> c_int;

    // Globals
    fn nvim_bw_set_msg_scroll(val: c_int);
    fn nvim_bw_set_msg_silent(val: c_int);

    // Dialog
    fn nvim_bw_msg(msg: *const c_char, hlf: c_int);
    fn nvim_bw_ask_yesno(msg: *const c_char) -> c_int;

    // Options
    fn nvim_bw_cpo_contains(c: c_int) -> c_int;

    // Gettext
    fn nvim_bw_gettext(msg: *const c_char) -> *const c_char;
}

// HLF_E value (from highlight_defs.h)
const HLF_E: c_int = 6;

// CPO_FWRITE = 'W'
const CPO_FWRITE: c_int = b'W' as c_int;

/// Check modification time of file before writing to it.
///
/// # Safety
///
/// `buf` and `file_info` must be valid handles.
unsafe fn check_mtime_inner(buf: BufHandle, file_info: FileInfoHandle) -> c_int {
    let mtime_read = unsafe { nvim_bw_buf_get_mtime_read(buf) };
    if mtime_read == 0 {
        return OK;
    }
    let mtime_read_ns = unsafe { nvim_bw_buf_get_mtime_read_ns(buf) };
    if unsafe { nvim_bw_time_differs(file_info, mtime_read, mtime_read_ns) } != 0 {
        unsafe { nvim_bw_set_msg_scroll(1) }; // Don't overwrite messages here.
        unsafe { nvim_bw_set_msg_silent(0) }; // Must give this prompt.
        // Don't use emsg() here, don't want to flush the buffers.
        let msg_text = unsafe {
            nvim_bw_gettext(c"WARNING: The file has been changed since reading it!!!".as_ptr())
        };
        unsafe { nvim_bw_msg(msg_text, HLF_E) };
        let prompt = unsafe { nvim_bw_gettext(c"Do you really want to write to it".as_ptr()) };
        if unsafe { nvim_bw_ask_yesno(prompt) } == c_int::from(b'n') {
            return FAIL;
        }
        unsafe { nvim_bw_set_msg_scroll(0) }; // Always overwrite the file message now.
    }
    OK
}

/// Unix: get file info for the given filename.
///
/// # Safety
///
/// All pointers must be valid.
#[cfg(unix)]
unsafe fn get_fileinfo_os_inner(
    fname: *const c_char,
    file_info_old: FileInfoHandle,
    _overwriting: bool,
    perm: *mut c_int,
    device: *mut bool,
    newfile: *mut bool,
    err: *mut BwError,
) -> c_int {
    unsafe { *perm = -1 };
    if unsafe { nvim_bw_os_fileinfo(fname, file_info_old) } == 0 {
        unsafe { *newfile = true };
    } else {
        unsafe { *perm = nvim_bw_fi_get_st_mode(file_info_old) };
        if unsafe { nvim_bw_fi_is_regular(file_info_old) } == 0 {
            // not a file
            if unsafe { nvim_bw_fi_is_dir(file_info_old) } != 0 {
                let num = c"E502".as_ptr();
                let msg = unsafe { nvim_bw_gettext(c"is a directory".as_ptr()) };
                unsafe { *err = BwError::with_num(num, msg) };
                return FAIL;
            }
            if unsafe { nvim_bw_os_nodetype(fname) } != NODE_WRITABLE {
                let num = c"E503".as_ptr();
                let msg = unsafe { nvim_bw_gettext(c"is not a file or writable device".as_ptr()) };
                unsafe { *err = BwError::with_num(num, msg) };
                return FAIL;
            }
            // It's a device of some kind (or a fifo) which we can write to
            // but for which we can't make a backup.
            unsafe {
                *device = true;
                *newfile = true;
                *perm = -1;
            }
        }
    }
    OK
}

/// Non-Unix: get file info for the given filename.
///
/// # Safety
///
/// All pointers must be valid.
#[cfg(not(unix))]
unsafe fn get_fileinfo_os_inner(
    fname: *const c_char,
    file_info_old: FileInfoHandle,
    overwriting: bool,
    perm: *mut c_int,
    device: *mut bool,
    newfile: *mut bool,
    err: *mut BwError,
) -> c_int {
    // Check for a writable device name.
    let nodetype = if fname.is_null() {
        NODE_OTHER
    } else {
        unsafe { nvim_bw_os_nodetype(fname) }
    };
    if nodetype == NODE_OTHER {
        let num = c"E503".as_ptr();
        let msg = unsafe { nvim_bw_gettext(c"is not a file or writable device".as_ptr()) };
        unsafe { *err = BwError::with_num(num, msg) };
        return FAIL;
    }
    if nodetype == NODE_WRITABLE {
        unsafe {
            *device = true;
            *newfile = true;
            *perm = -1;
        }
    } else {
        unsafe { *perm = nvim_bw_os_getperm(fname) };
        if unsafe { *perm } < 0 {
            unsafe { *newfile = true };
        } else if unsafe { nvim_bw_os_isdir(fname) } != 0 {
            let num = c"E502".as_ptr();
            let msg = unsafe { nvim_bw_gettext(c"is a directory".as_ptr()) };
            unsafe { *err = BwError::with_num(num, msg) };
            return FAIL;
        }
        if overwriting {
            unsafe { nvim_bw_os_fileinfo(fname, file_info_old) };
        }
    }
    OK
}

/// Get file info, check permissions, and optionally check mtime.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn get_fileinfo_inner(
    buf: BufHandle,
    fname: *const c_char,
    overwriting: bool,
    forceit: bool,
    file_info_old: FileInfoHandle,
    perm: *mut c_int,
    device: *mut bool,
    newfile: *mut bool,
    readonly: *mut bool,
    err: *mut BwError,
) -> c_int {
    if unsafe {
        get_fileinfo_os_inner(
            fname,
            file_info_old,
            overwriting,
            perm,
            device,
            newfile,
            err,
        )
    } == FAIL
    {
        return FAIL;
    }

    unsafe { *readonly = false };

    if !unsafe { *device } && !unsafe { *newfile } {
        // Check if the file is really writable
        unsafe { *readonly = nvim_bw_os_file_is_writable(fname) == 0 };

        if !forceit && unsafe { *readonly } {
            if unsafe { nvim_bw_cpo_contains(CPO_FWRITE) } != 0 {
                let num = c"E504".as_ptr();
                let msg = unsafe {
                    nvim_bw_gettext(
                        c"is read-only (cannot override: \"W\" in 'cpoptions')".as_ptr(),
                    )
                };
                unsafe { *err = BwError::with_num(num, msg) };
            } else {
                let num = c"E505".as_ptr();
                let msg = unsafe { nvim_bw_gettext(c"is read-only (add ! to override)".as_ptr()) };
                unsafe { *err = BwError::with_num(num, msg) };
            }
            return FAIL;
        }

        // If 'forceit' is false, check if the timestamp hasn't changed since reading the file.
        if overwriting && !forceit {
            let retval = unsafe { check_mtime_inner(buf, file_info_old) };
            if retval == FAIL {
                return FAIL;
            }
        }
    }
    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check modification time of file, before writing to it.
///
/// # Safety
///
/// `buf` and `file_info` must be valid handles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_check_mtime(buf: BufHandle, file_info: FileInfoHandle) -> c_int {
    unsafe { check_mtime_inner(buf, file_info) }
}

/// Get file info for the given filename (platform-dispatched).
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_fileinfo_os(
    fname: *const c_char,
    file_info_old: FileInfoHandle,
    overwriting: c_int,
    perm: *mut c_int,
    device: *mut bool,
    newfile: *mut bool,
    err: *mut BwError,
) -> c_int {
    unsafe {
        get_fileinfo_os_inner(
            fname,
            file_info_old,
            overwriting != 0,
            perm,
            device,
            newfile,
            err,
        )
    }
}

/// Get file info, check permissions, and optionally check mtime.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_get_fileinfo(
    buf: BufHandle,
    fname: *const c_char,
    overwriting: c_int,
    forceit: c_int,
    file_info_old: FileInfoHandle,
    perm: *mut c_int,
    device: *mut bool,
    newfile: *mut bool,
    readonly: *mut bool,
    err: *mut BwError,
) -> c_int {
    unsafe {
        get_fileinfo_inner(
            buf,
            fname,
            overwriting != 0,
            forceit != 0,
            file_info_old,
            perm,
            device,
            newfile,
            readonly,
            err,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(NODE_NORMAL, 0);
        assert_eq!(NODE_WRITABLE, 1);
        assert_eq!(NODE_OTHER, 2);
        assert_eq!(HLF_E, 6);
        assert_eq!(CPO_FWRITE, c_int::from(b'W'));
    }
}
