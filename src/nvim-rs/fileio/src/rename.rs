//! File rename, copy, and name-modification utilities.
//!
//! This module provides:
//! - `rs_modname`: Generate a modified filename with a new extension
//! - `rs_rename_with_tmp`: Rename via temp file when src/dest may be same
//! - `rs_vim_rename`: Cross-filesystem rename with copy fallback
//! - `rs_vim_copyfile`: Copy a file preserving symlinks and ACLs

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int};

// MAXPATHL = 4096 on Linux (PATH_MAX)
const MAXPATHL: usize = 4096;
// BASENAMELEN = NAME_MAX - 5 = 255 - 5 = 250
const BASENAMELEN: usize = 250;

// C constants
const OK: c_int = 1;
const FAIL: c_int = 0;

extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn os_dirname(buf: *mut c_char, len: usize) -> c_int;
    fn add_pathsep(path: *mut c_char) -> bool;
    fn path_tail(p: *const c_char) -> *mut c_char;
    fn vim_ispathsep(c: c_int) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    fn os_fileinfo(fname: *const c_char, info: *mut crate::FileInfoHandle) -> c_int;
    fn os_fileinfo_id_equal(
        a: *const crate::FileInfoHandle,
        b: *const crate::FileInfoHandle,
    ) -> c_int;
    fn os_fileinfo_link(fname: *const c_char, info: *mut crate::FileInfoHandle) -> c_int;
    fn os_path_exists(path: *const c_char) -> c_int;
    fn os_rename(from: *const c_char, to: *const c_char) -> c_int;
    fn os_remove(path: *const c_char) -> c_int;
    fn os_get_acl(fname: *const c_char) -> *mut c_void;
    fn os_set_acl(fname: *const c_char, acl: *mut c_void);
    fn os_free_acl(acl: *mut c_void);
    fn os_copy(from: *const c_char, to: *const c_char, flags: c_int) -> c_int;
}

// libc readlink/symlink are not needed through libc crate since we use extern C for platform funcs
extern "C" {
    fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: libc::size_t) -> libc::ssize_t;
    fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int;
}

use std::ffi::c_void;

extern "C" {
    /// The 'fileignorecase' option global.
    static mut p_fic: c_int;
}

// UV_FS_COPYFILE_EXCL = 1 (but 0 means don't use it, matches the C fallback define)
const UV_FS_COPYFILE_EXCL: c_int = 1;

// =============================================================================
// modname
// =============================================================================

/// Generate a modified file name: fname with extension ext.
///
/// If prepend_dot is true, a dot is prepended before the basename to
/// create a "hidden" file (e.g., ".foo.ext").
///
/// Returns an xmalloc-allocated string, or NULL on failure.
/// The caller must xfree() the result.
///
/// # Safety
/// `fname` may be null. `ext` must be a valid non-null C string.
#[export_name = "modname"]
pub unsafe extern "C" fn rs_modname(
    fname: *const c_char,
    ext: *const c_char,
    prepend_dot: bool,
) -> *mut c_char {
    if ext.is_null() {
        return std::ptr::null_mut();
    }

    let ext_bytes = unsafe { std::ffi::CStr::from_ptr(ext).to_bytes() };
    let extlen = ext_bytes.len();
    let mut prepend_dot = prepend_dot;

    let retval: *mut c_char;
    let fnamelen: usize;

    // If there is no file name, use the current directory name.
    if fname.is_null() || unsafe { *fname } == 0 {
        retval = unsafe { xmalloc(MAXPATHL + extlen + 3) };
        if retval.is_null() {
            return std::ptr::null_mut();
        }
        if unsafe { os_dirname(retval, MAXPATHL) } == FAIL
            || unsafe { libc::strlen(retval as *const libc::c_char) } == 0
        {
            unsafe { xfree(retval as *mut c_void) };
            return std::ptr::null_mut();
        }
        unsafe { add_pathsep(retval) };
        fnamelen = unsafe { libc::strlen(retval as *const libc::c_char) };
        prepend_dot = false; // nothing to prepend a dot to
    } else {
        fnamelen = unsafe { libc::strlen(fname as *const libc::c_char) };
        retval = unsafe { xmalloc(fnamelen + extlen + 3) };
        if retval.is_null() {
            return std::ptr::null_mut();
        }
        unsafe { libc::strcpy(retval as *mut libc::c_char, fname as *const libc::c_char) };
    }

    // Search backwards until we hit a path separator, then truncate
    // the basename to BASENAMELEN characters.
    let mut ptr = unsafe { retval.add(fnamelen) };
    while ptr > retval {
        // MB_PTR_BACK: ptr -= utf_head_off(retval, ptr-1) + 1
        let off = unsafe { utf_head_off(retval, ptr.offset(-1)) } as usize;
        ptr = unsafe { ptr.offset(-((off + 1) as isize)) };
        if unsafe { vim_ispathsep(*ptr as c_int) } != 0 {
            ptr = unsafe { ptr.add(1) };
            break;
        }
    }

    // Truncate if basename is too long.
    let base_len = unsafe { libc::strlen(ptr as *const libc::c_char) };
    if base_len > BASENAMELEN {
        unsafe { *ptr.add(BASENAMELEN) = 0 };
    }

    // Append the extension.
    let s = unsafe { ptr.add(libc::strlen(ptr as *const libc::c_char)) };
    unsafe { libc::strcpy(s as *mut libc::c_char, ext as *const libc::c_char) };

    // Prepend a dot if requested.
    if prepend_dot {
        let e = unsafe { path_tail(retval) };
        if !e.is_null() && unsafe { *e } != b'.' as c_char {
            // STRMOVE(e+1, e): shift string right by 1
            let e_len = unsafe { libc::strlen(e as *const libc::c_char) };
            // memmove(e+1, e, e_len+1) -- includes NUL
            unsafe { libc::memmove(e.add(1) as *mut c_void, e as *const c_void, e_len + 1) };
            unsafe { *e = b'.' as c_char };
        }
    }

    // Check that the result is actually different from fname.
    if !fname.is_null()
        && unsafe { libc::strcmp(fname as *const libc::c_char, retval as *const libc::c_char) } == 0
    {
        // Search for a character to replace with '_'.
        let mut s = unsafe {
            retval
                .add(libc::strlen(retval as *const libc::c_char))
                .offset(-1)
        };
        while s >= ptr {
            if unsafe { *s } != b'_' as c_char {
                unsafe { *s = b'_' as c_char };
                break;
            }
            s = unsafe { s.offset(-1) };
        }
        if s < ptr {
            // All underscores, set first char to 'v'
            unsafe { *ptr = b'v' as c_char };
        }
    }

    retval
}

// =============================================================================
// rename_with_tmp (internal helper)
// =============================================================================

/// Rename a file via a temp name when source and destination may be the same
/// file (e.g., on case-insensitive FAT32 filesystems).
///
/// Returns 0 on success, -1 on failure.
unsafe fn rename_with_tmp_impl(from: *const c_char, to: *const c_char) -> c_int {
    let from_len = unsafe { libc::strlen(from as *const libc::c_char) };
    if from_len >= MAXPATHL - 5 {
        return -1;
    }

    let mut tempname = [0u8; MAXPATHL + 1];
    unsafe {
        libc::strcpy(
            tempname.as_mut_ptr() as *mut libc::c_char,
            from as *const libc::c_char,
        )
    };

    for n in 123..99999i32 {
        let tail_ptr = unsafe { path_tail(tempname.as_ptr() as *const c_char) };
        if tail_ptr.is_null() {
            return -1;
        }
        let tail_offset =
            unsafe { tail_ptr.offset_from(tempname.as_ptr() as *const c_char) } as usize;
        let remaining = MAXPATHL + 1 - tail_offset;
        let tail_buf = &mut tempname[tail_offset..];

        // snprintf equivalent: write decimal n into tail
        let n_str = format!("{n}\0");
        let n_bytes = n_str.as_bytes();
        if n_bytes.len() > remaining {
            return -1;
        }
        tail_buf[..n_bytes.len()].copy_from_slice(n_bytes);

        let tempname_ptr = tempname.as_ptr() as *const c_char;
        if unsafe { os_path_exists(tempname_ptr) } == 0 {
            // temp name doesn't exist, try renaming
            if unsafe { os_rename(from, tempname_ptr) } == OK {
                if unsafe { os_rename(tempname_ptr, to) } == OK {
                    return 0;
                }
                // Second step failed, try to move back
                unsafe { os_rename(tempname_ptr, from) };
                return -1;
            }
            // First rename failed, give up
            return -1;
        }
    }
    -1
}

// =============================================================================
// vim_rename
// =============================================================================

/// Rename a file, falling back to copy if cross-filesystem rename fails.
///
/// Returns 0 on success, -1 on failure.
///
/// # Safety
/// `from` and `to` must be valid non-null C strings.
#[export_name = "vim_rename"]
pub unsafe extern "C" fn rs_vim_rename(from: *const c_char, to: *const c_char) -> c_int {
    if from.is_null() || to.is_null() {
        return -1;
    }

    let mut use_tmp_file = false;

    // When names are identical, there is nothing to do.
    // When they refer to the same file (ignoring case) but the names differ,
    // go through a temp file.
    if unsafe { path_fnamecmp(from, to) } == 0 {
        let fic_val = unsafe { p_fic };
        let from_tail = unsafe { path_tail(from) };
        let to_tail = unsafe { path_tail(to) };
        if fic_val != 0
            && !from_tail.is_null()
            && !to_tail.is_null()
            && unsafe {
                libc::strcmp(
                    from_tail as *const libc::c_char,
                    to_tail as *const libc::c_char,
                )
            } != 0
        {
            use_tmp_file = true;
        } else {
            return 0;
        }
    }

    // Fail if the "from" file doesn't exist.
    let mut from_info = std::mem::MaybeUninit::<[u8; 256]>::uninit();
    if unsafe { os_fileinfo(from, from_info.as_mut_ptr() as *mut crate::FileInfoHandle) } == 0 {
        return -1;
    }

    // Check if source and destination are the same inode.
    let mut to_info = std::mem::MaybeUninit::<[u8; 256]>::uninit();
    if unsafe { os_fileinfo(to, to_info.as_mut_ptr() as *mut crate::FileInfoHandle) } != 0
        && unsafe {
            os_fileinfo_id_equal(
                from_info.as_ptr() as *const crate::FileInfoHandle,
                to_info.as_ptr() as *const crate::FileInfoHandle,
            )
        } != 0
    {
        use_tmp_file = true;
    }

    if use_tmp_file {
        return unsafe { rename_with_tmp_impl(from, to) };
    }

    // Delete the "to" file first (required on some systems).
    unsafe { os_remove(to) };

    // Try a normal rename.
    if unsafe { os_rename(from, to) } == OK {
        return 0;
    }

    // Rename failed, try copying.
    if unsafe { rs_vim_copyfile(from, to) } != OK {
        return -1;
    }

    // Copy succeeded, remove the source.
    let mut from_info2 = std::mem::MaybeUninit::<[u8; 256]>::uninit();
    if unsafe { os_fileinfo(from, from_info2.as_mut_ptr() as *mut crate::FileInfoHandle) } != 0 {
        unsafe { os_remove(from) };
    }

    0
}

// =============================================================================
// vim_copyfile
// =============================================================================

/// Copy a file, preserving symlinks and ACLs.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// `from` and `to` must be valid C strings (may be null, will return FAIL).
#[export_name = "vim_copyfile"]
pub unsafe extern "C" fn rs_vim_copyfile(from: *const c_char, to: *const c_char) -> c_int {
    if from.is_null() || to.is_null() {
        return FAIL;
    }

    // HAVE_READLINK: check if source is a symlink and replicate the link
    {
        let mut from_info = std::mem::MaybeUninit::<[u8; 256]>::uninit();
        let has_info =
            unsafe { os_fileinfo_link(from, from_info.as_mut_ptr() as *mut crate::FileInfoHandle) };
        if has_info != 0 {
            // Check if it's a symlink by checking S_ISLNK
            // We use readlink: if it returns > 0, it's a symlink
            let mut linkbuf = [0u8; MAXPATHL + 1];
            let len = unsafe { readlink(from, linkbuf.as_mut_ptr() as *mut c_char, MAXPATHL) };
            if len > 0 {
                // It's a symlink: create a new symlink at `to`
                linkbuf[len as usize] = 0;
                let ret = unsafe { symlink(linkbuf.as_ptr() as *const c_char, to) };
                return if ret == 0 { OK } else { FAIL };
            }
            // len <= 0 means not a symlink or error, fall through to regular copy
        }
    }

    // Get ACL from original file.
    let acl = unsafe { os_get_acl(from) };

    // Copy the file.
    if unsafe { os_copy(from, to, UV_FS_COPYFILE_EXCL) } != 0 {
        unsafe { os_free_acl(acl) };
        return FAIL;
    }

    // Set the ACL on the new file.
    unsafe { os_set_acl(to, acl) };
    unsafe { os_free_acl(acl) };

    OK
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_constants() {
        // Sanity check constants match C definitions
        assert_eq!(super::MAXPATHL, 4096);
        // BASENAMELEN = NAME_MAX - 5. On Linux NAME_MAX = 255, so BASENAMELEN = 250
        assert_eq!(super::BASENAMELEN, 250);
    }
}
