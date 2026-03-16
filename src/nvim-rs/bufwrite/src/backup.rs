//! Backup file creation for buffer writing.
//!
//! Mirrors the C `buf_write_make_backup` function.

#![allow(clippy::cast_lossless)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::fn_params_excessive_bools)]

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

#[cfg(unix)]
use libc::O_NOFOLLOW;
use libc::{snprintf, strlen, O_CREAT, O_EXCL, O_WRONLY};
#[cfg(not(unix))]
const O_NOFOLLOW: c_int = 0;

const UV_FS_COPYFILE_FICLONE: c_int = 0x0002;

use crate::error::BwError;
use crate::ffi::{AclHandle, FileInfoHandle, FAIL, OK};

// kOptBkcFlag constants (from generated option_vars.generated.h)
const BKC_YES: c_uint = 0x01;
const BKC_AUTO: c_uint = 0x02;
#[allow(dead_code)]
const BKC_NO: c_uint = 0x04;
const BKC_BREAKSYMLINK: c_uint = 0x08;
const BKC_BREAKHARDLINK: c_uint = 0x10;

// CPO_FWRITE
const CPO_FWRITE: c_int = b'W' as c_int;

// NUL
const NUL: c_char = 0;

// Size constants
const IOSIZE: usize = 1025;
const MAXPATHL: usize = 4096;

unsafe extern "C" {
    static mut p_bex: *const c_char;
    static mut p_bdir: *mut c_char;
    static mut p_bk: c_int;
    static mut IObuff: [c_char; IOSIZE];
}

extern "C" {
    // FileInfo operations
    #[link_name = "os_fileinfo_hardlinks"]
    fn os_fileinfo_hardlinks(fi: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo_link"]
    fn os_fileinfo_link(fname: *const c_char, fi: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo_id_equal"]
    fn os_fileinfo_id_equal(fi1: FileInfoHandle, fi2: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo"]
    fn os_fileinfo(fname: *const c_char, fi: FileInfoHandle) -> c_int;

    // Path operations
    #[link_name = "path_tail"]
    fn path_tail(fname: *const c_char) -> *const c_char;
    #[link_name = "after_pathsep"]
    fn after_pathsep(dir: *const c_char, p: *const c_char) -> c_int;
    #[link_name = "make_percent_swname"]
    fn make_percent_swname(
        dir: *const c_char,
        p: *const c_char,
        fname: *const c_char,
    ) -> *mut c_char;
    #[link_name = "modname"]
    fn modname(fname: *const c_char, ext: *const c_char, prepend_dot: bool) -> *mut c_char;
    #[link_name = "get_file_in_dir"]
    fn get_file_in_dir(fname: *const c_char, dir: *const c_char) -> *mut c_char;
    #[link_name = "os_path_exists"]
    fn os_path_exists(fname: *const c_char) -> c_int;

    // File system operations
    #[link_name = "os_open"]
    fn os_open(fname: *const c_char, flags: c_int, perm: c_int) -> c_int;
    #[link_name = "os_close"]
    fn os_close(fd: c_int) -> c_int;
    #[link_name = "os_remove"]
    fn os_remove(fname: *const c_char) -> c_int;
    #[link_name = "os_copy"]
    fn os_copy(src: *const c_char, dst: *const c_char, flags: c_int) -> c_int;
    #[link_name = "os_setperm"]
    fn os_setperm(fname: *const c_char, perm: c_int) -> c_int;
    fn nvim_bw_os_set_acl(fname: *const c_char, acl: AclHandle);
    #[link_name = "vim_rename"]
    fn vim_rename(src: *const c_char, dst: *const c_char) -> c_int;

    // Unix-specific
    #[link_name = "os_fchown"]
    fn os_fchown(fd: c_int, uid: u32, gid: u32);
    #[link_name = "os_chown"]
    fn os_chown(fname: *const c_char, uid: u32, gid: u32) -> c_int;
    #[link_name = "os_file_settime"]
    fn os_file_settime(fname: *const c_char, atime: f64, mtime: f64);
    fn nvim_bw_fi_get_st_uid(fi: FileInfoHandle) -> u32;
    fn nvim_bw_fi_get_st_gid(fi: FileInfoHandle) -> u32;
    fn nvim_bw_fi_get_st_mode(fi: FileInfoHandle) -> c_int;
    fn nvim_bw_fi_get_atime_sec(fi: FileInfoHandle) -> i64;
    fn nvim_bw_fi_get_mtime_sec(fi: FileInfoHandle) -> i64;
    // Copy xattr (wraps #ifdef HAVE_XATTR)
    fn nvim_bw_os_copy_xattr(src: *const c_char, dst: *const c_char);

    // Utility
    #[link_name = "copy_option_part"]
    fn copy_option_part(
        dirp: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep: *const c_char,
    ) -> usize;
    fn nvim_bw_os_mkdir_recurse(
        dir: *const c_char,
        perm: c_int,
        failed_dir: *mut *mut c_char,
    ) -> c_int;
    #[link_name = "os_isdir"]
    fn os_isdir(fname: *const c_char) -> c_int;
    fn nvim_bw_cpo_contains(c: c_int) -> c_int;
    fn nvim_bw_xfree(ptr: *mut c_char);
    fn nvim_bw_gettext(s: *const c_char) -> *const c_char;
    fn nvim_bw_semsg_3(fmt: *const c_char, a: *const c_char, b: *const c_char, c: *const c_char);
    fn nvim_bw_os_strerror(errnum: c_int) -> *const c_char;
    fn nvim_bw_XFREE_CLEAR(pp: *mut *mut c_char);

    // Sizes
    fn nvim_bw_sizeof_FileInfo() -> usize;
}

// Error messages (translated at call site)
const E509: &std::ffi::CStr = c"E509: Cannot create backup file (add ! to override)";
const E510: &std::ffi::CStr = c"E510: Can't make backup file (add ! to override)";
const E504: &std::ffi::CStr = c"E504";
const ERR_READONLY: &std::ffi::CStr = c"is read-only (cannot override: \"W\" in 'cpoptions')";

/// Try to create a backup directory if needed, emitting E303 on failure.
///
/// # Safety
///
/// `iobuff` and `dirp_ptr` must be valid.
unsafe fn ensure_backup_dir(iobuff: *const c_char) {
    let mut failed_dir: *mut c_char = ptr::null_mut();
    let ret = unsafe { nvim_bw_os_mkdir_recurse(iobuff, 0o755, &raw mut failed_dir) };
    if ret != 0 {
        let fmt = unsafe {
            nvim_bw_gettext(c"E303: Unable to create directory \"%s\" for backup file: %s".as_ptr())
        };
        let strerr = unsafe { nvim_bw_os_strerror(ret) };
        unsafe { nvim_bw_semsg_3(fmt, failed_dir, strerr, ptr::null()) };
        unsafe { nvim_bw_xfree(failed_dir) };
    }
}

/// Determine whether we should copy rather than rename.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn determine_backup_copy(
    fname: *const c_char,
    file_info_old: FileInfoHandle,
    bkc: c_uint,
    append: bool,
    perm: c_int,
    backup_copy: &mut bool,
) {
    // Stack-allocated FileInfo — dynamically sized to match C struct
    let fi_size = unsafe { nvim_bw_sizeof_FileInfo() };
    let mut file_info_buf = vec![0u8; fi_size];
    let file_info: FileInfoHandle = file_info_buf.as_mut_ptr().cast();

    if (bkc & BKC_YES != 0) || append {
        *backup_copy = true;
    } else if bkc & BKC_AUTO != 0 {
        // Don't rename the file when:
        // - it's a hard link
        // - it's a symbolic link
        // - we don't have write permission in the directory
        if unsafe { os_fileinfo_hardlinks(file_info_old) } > 1
            || unsafe { os_fileinfo_link(fname, file_info) } == 0
            || unsafe { os_fileinfo_id_equal(file_info, file_info_old) } == 0
        {
            *backup_copy = true;
        } else {
            // Check if we can create a file and set the owner/group to
            // the ones from the original file.
            let tail = unsafe { path_tail(fname) };
            let dirlen = unsafe { tail.offset_from(fname) } as usize;
            let maxpathl = MAXPATHL;
            assert!(dirlen < maxpathl);

            // Use a stack buffer for tmp_fname
            let mut tmp_fname = vec![0u8; maxpathl];
            let tmp_ptr = tmp_fname.as_mut_ptr().cast::<c_char>();
            unsafe {
                ptr::copy_nonoverlapping(fname, tmp_ptr, dirlen);
                *tmp_ptr.add(dirlen) = 0;
            }

            // Find a temp filename that doesn't exist
            let mut i: c_int = 4913;
            loop {
                unsafe {
                    snprintf(tmp_ptr.add(dirlen), maxpathl - dirlen, c"%d".as_ptr(), i);
                }
                if unsafe { os_fileinfo_link(tmp_ptr, file_info) } == 0 {
                    break;
                }
                i += 123;
            }

            let open_flags = O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW;
            let fd = unsafe { os_open(tmp_ptr, open_flags, perm) };
            if fd < 0 {
                *backup_copy = true;
            } else {
                #[cfg(unix)]
                {
                    let uid = unsafe { nvim_bw_fi_get_st_uid(file_info_old) };
                    let gid = unsafe { nvim_bw_fi_get_st_gid(file_info_old) };
                    unsafe { os_fchown(fd, uid, gid) };
                    if unsafe { os_fileinfo(tmp_ptr, file_info) } == 0
                        || unsafe { nvim_bw_fi_get_st_uid(file_info) } != uid
                        || unsafe { nvim_bw_fi_get_st_gid(file_info) } != gid
                        || unsafe { nvim_bw_fi_get_st_mode(file_info) } != perm
                    {
                        *backup_copy = true;
                    }
                }
                unsafe { os_close(fd) };
                unsafe { os_remove(tmp_ptr) };
            }
        }
    }

    // Break symlinks and/or hardlinks if we've been asked to.
    #[cfg(unix)]
    if (bkc & BKC_BREAKSYMLINK != 0) || (bkc & BKC_BREAKHARDLINK != 0) {
        let mut file_info_buf2 = vec![0u8; unsafe { nvim_bw_sizeof_FileInfo() }];
        let file_info2: FileInfoHandle = file_info_buf2.as_mut_ptr().cast();
        let link_ok = unsafe { os_fileinfo_link(fname, file_info2) } != 0;

        // Symlinks
        if (bkc & BKC_BREAKSYMLINK != 0)
            && link_ok
            && unsafe { os_fileinfo_id_equal(file_info2, file_info_old) } == 0
        {
            *backup_copy = false;
        }

        // Hardlinks
        if (bkc & BKC_BREAKHARDLINK != 0)
            && unsafe { os_fileinfo_hardlinks(file_info_old) } > 1
            && (!link_ok || unsafe { os_fileinfo_id_equal(file_info2, file_info_old) } != 0)
        {
            *backup_copy = false;
        }
    }
}

/// Make a backup by copying the file.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn backup_by_copy(
    fname: *const c_char,
    file_info_old: FileInfoHandle,
    acl: AclHandle,
    perm: c_int,
    forceit: bool,
    backup_ext: *const c_char,
    backupp: *mut *mut c_char,
    err: *mut BwError,
) -> c_int {
    let mut some_error = false;
    let iobuff = std::ptr::addr_of_mut!(IObuff).cast::<c_char>();
    let iosize = IOSIZE;

    let mut dirp = unsafe { p_bdir };

    while unsafe { *dirp } != NUL {
        let dir_len = unsafe { copy_option_part(&raw mut dirp, iobuff, iosize, c",".as_ptr()) };
        let p = unsafe { iobuff.add(dir_len) };

        if unsafe { *dirp } == NUL && unsafe { os_isdir(iobuff) } == 0 {
            unsafe { ensure_backup_dir(iobuff) };
        }

        if unsafe { after_pathsep(iobuff, p) } != 0 && unsafe { *p.sub(1) == *p.sub(2) } {
            // Ends with '//', use full path
            let swname = unsafe { make_percent_swname(iobuff, p, fname) };
            if !swname.is_null() {
                unsafe { *backupp = modname(swname, backup_ext, false) };
                unsafe { nvim_bw_xfree(swname) };
            }
        }

        let rootname = unsafe { get_file_in_dir(fname, iobuff) };
        if rootname.is_null() {
            some_error = true;
            // goto nobackup equivalent — break out
            break;
        }

        // Stack-allocated FileInfo for checking backup file
        let mut file_info_new_buf = vec![0u8; unsafe { nvim_bw_sizeof_FileInfo() }];
        let file_info_new: FileInfoHandle = file_info_new_buf.as_mut_ptr().cast();

        // Make the backup file name
        if unsafe { *backupp }.is_null() {
            unsafe { *backupp = modname(rootname, backup_ext, false) };
        }

        if unsafe { *backupp }.is_null() {
            unsafe { nvim_bw_xfree(rootname) };
            some_error = true;
            break;
        }

        // Check if backup file already exists
        if unsafe { os_fileinfo(*backupp, file_info_new) } != 0 {
            if unsafe { os_fileinfo_id_equal(file_info_new, file_info_old) } != 0 {
                // Backup file is same as original file
                unsafe { nvim_bw_XFREE_CLEAR(backupp) };
            } else if unsafe { p_bk == 0 } {
                // Not keeping backups — try to use another name
                let bkp = unsafe { *backupp };
                let bkplen = unsafe { strlen(bkp) };
                let extlen = unsafe { strlen(backup_ext) };
                let offset = if bkplen > 1 + extlen {
                    bkplen - 1 - extlen
                } else {
                    0
                };
                let wp = unsafe { bkp.add(offset) };
                unsafe { *wp = b'z' as c_char };
                while unsafe { *wp } > b'a' as c_char
                    && unsafe { os_fileinfo(*backupp, file_info_new) } != 0
                {
                    unsafe { *wp -= 1 };
                }
                if unsafe { *wp } == b'a' as c_char {
                    unsafe { nvim_bw_XFREE_CLEAR(backupp) };
                }
            }
        }

        unsafe { nvim_bw_xfree(rootname) };

        // Try to create the backup file
        if !unsafe { *backupp }.is_null() {
            // Remove old backup
            unsafe { os_remove(*backupp) };

            // Copy the file
            if unsafe { os_copy(fname, *backupp, UV_FS_COPYFILE_FICLONE) } != 0 {
                unsafe {
                    *err = BwError::with_msg(nvim_bw_gettext(E509.as_ptr()));
                    nvim_bw_XFREE_CLEAR(backupp);
                }
                continue;
            }

            // Set file protection same as original file, but strip s-bit
            unsafe { os_setperm(*backupp, perm & 0o777) };

            #[cfg(unix)]
            {
                // Try to set group of backup same as original file
                let old_gid = unsafe { nvim_bw_fi_get_st_gid(file_info_old) };
                let new_gid = unsafe { nvim_bw_fi_get_st_gid(file_info_new) };
                if new_gid != old_gid && unsafe { os_chown(*backupp, u32::MAX, old_gid) } != 0 {
                    unsafe {
                        os_setperm(*backupp, (perm & 0o707) | ((perm & 0o7) << 3));
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                let atime = unsafe { nvim_bw_fi_get_atime_sec(file_info_old) } as f64;
                #[allow(clippy::cast_precision_loss)]
                let mtime = unsafe { nvim_bw_fi_get_mtime_sec(file_info_old) } as f64;
                unsafe { os_file_settime(*backupp, atime, mtime) };
            }

            unsafe { nvim_bw_os_set_acl(*backupp, acl) };
            unsafe { nvim_bw_os_copy_xattr(fname, *backupp) };
            unsafe { *err = BwError::with_msg(ptr::null()) };
            break;
        }
    }

    // nobackup:
    if unsafe { *backupp }.is_null() && unsafe { (*err).msg }.is_null() {
        unsafe {
            *err = BwError::with_msg(nvim_bw_gettext(E509.as_ptr()));
        }
    }
    // Ignore errors when forceit is true
    if (some_error || !unsafe { (*err).msg }.is_null()) && !forceit {
        return FAIL;
    }
    unsafe { *err = BwError::with_msg(ptr::null()) };
    OK
}

/// Make a backup by renaming the original file.
///
/// # Safety
///
/// All pointers must be valid.
unsafe fn backup_by_rename(
    fname: *const c_char,
    file_readonly: bool,
    forceit: bool,
    backup_ext: *const c_char,
    backupp: *mut *mut c_char,
    err: *mut BwError,
) -> c_int {
    // If 'cpoptions' includes the "W" flag, we don't want to
    // overwrite a read-only file.
    if file_readonly && unsafe { nvim_bw_cpo_contains(CPO_FWRITE) } != 0 {
        unsafe {
            *err = BwError::with_num(E504.as_ptr(), nvim_bw_gettext(ERR_READONLY.as_ptr()));
        }
        return FAIL;
    }

    let iobuff = std::ptr::addr_of_mut!(IObuff).cast::<c_char>();
    let iosize = IOSIZE;
    let mut dirp = unsafe { p_bdir };

    while unsafe { *dirp } != NUL {
        let dir_len = unsafe { copy_option_part(&raw mut dirp, iobuff, iosize, c",".as_ptr()) };
        let p = unsafe { iobuff.add(dir_len) };

        if unsafe { *dirp } == NUL && unsafe { os_isdir(iobuff) } == 0 {
            unsafe { ensure_backup_dir(iobuff) };
        }

        if unsafe { after_pathsep(iobuff, p) } != 0 && unsafe { *p.sub(1) == *p.sub(2) } {
            // Path ends with '//', use full path
            let swname = unsafe { make_percent_swname(iobuff, p, fname) };
            if !swname.is_null() {
                unsafe { *backupp = modname(swname, backup_ext, false) };
                unsafe { nvim_bw_xfree(swname) };
            }
        }

        if unsafe { *backupp }.is_null() {
            let rootname = unsafe { get_file_in_dir(fname, iobuff) };
            if rootname.is_null() {
                unsafe { *backupp = ptr::null_mut() };
            } else {
                unsafe {
                    *backupp = modname(rootname, backup_ext, false);
                    nvim_bw_xfree(rootname);
                }
            }
        }

        if !unsafe { *backupp }.is_null() {
            // If we are not going to keep the backup file, don't
            // delete an existing one, try to use another name.
            if unsafe { p_bk == 0 } && unsafe { os_path_exists(*backupp) } != 0 {
                let bkp = unsafe { *backupp };
                let bkplen = unsafe { strlen(bkp) };
                let extlen = unsafe { strlen(backup_ext) };
                let offset = if bkplen > 1 + extlen {
                    bkplen - 1 - extlen
                } else {
                    0
                };
                let wp = unsafe { bkp.add(offset) };
                unsafe { *wp = b'z' as c_char };
                while unsafe { *wp } > b'a' as c_char && unsafe { os_path_exists(*backupp) } != 0 {
                    unsafe { *wp -= 1 };
                }
                if unsafe { *wp } == b'a' as c_char {
                    unsafe { nvim_bw_XFREE_CLEAR(backupp) };
                }
            }
        }

        if !unsafe { *backupp }.is_null() {
            // Delete any existing backup and rename
            if unsafe { vim_rename(fname, *backupp) } == 0 {
                break;
            }
            unsafe { nvim_bw_XFREE_CLEAR(backupp) };
        }
    }

    if unsafe { *backupp }.is_null() && !forceit {
        unsafe {
            *err = BwError::with_msg(nvim_bw_gettext(E510.as_ptr()));
        }
        return FAIL;
    }
    OK
}

/// Create a backup file for the buffer being written.
///
/// Replaces C `buf_write_make_backup`.
///
/// # Safety
///
/// All pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_buf_write_make_backup(
    fname: *mut c_char,
    append: c_int,
    file_info_old: FileInfoHandle,
    acl: AclHandle,
    perm: c_int,
    bkc: c_uint,
    file_readonly: c_int,
    forceit: c_int,
    backup_copyp: *mut bool,
    backupp: *mut *mut c_char,
    err: *mut BwError,
) -> c_int {
    let mut backup_copy = false;

    unsafe {
        determine_backup_copy(
            fname,
            file_info_old,
            bkc,
            append != 0,
            perm,
            &mut backup_copy,
        );
    }

    // Get backup extension
    let backup_ext = if unsafe { *p_bex } == NUL {
        c".bak".as_ptr()
    } else {
        unsafe { p_bex }
    };

    let result = if backup_copy {
        unsafe {
            backup_by_copy(
                fname,
                file_info_old,
                acl,
                perm,
                forceit != 0,
                backup_ext,
                backupp,
                err,
            )
        }
    } else {
        unsafe {
            backup_by_rename(
                fname,
                file_readonly != 0,
                forceit != 0,
                backup_ext,
                backupp,
                err,
            )
        }
    };

    unsafe { *backup_copyp = backup_copy };
    result
}
