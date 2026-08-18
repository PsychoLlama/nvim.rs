//! File metadata: permissions, ownership, timestamps and identity.
//!
//! Every answer here comes from libuv's `uv_fs_*` synchronous calls, which
//! is why each function is one [`fs_request`] around a `uv_fs_t`. `FileInfo`
//! and `FileID` are the two shapes upstream wraps `uv_stat_t` in — the first
//! is the whole stat buffer, the second only the (device, inode) pair that
//! says two paths are the same file.
//!
//! The exception is [`os_copy_xattr`], which is `getxattr`/`setxattr` rather
//! than libuv and is the only thing in this file with a shape of its own.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::os::uv_error::UV_EINVAL;
use core::ffi::{CStr, c_char, c_double, c_int, c_void};
use core::ptr;

use super::{
    LIBUV_SUCCESS, NO_LOOP, R_OK, UV_STAT_T_INIT, W_OK, fs_ok, fs_request, fs_result, os_isdir,
};
use crate::event::libuv::{
    uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_fchown, uv_fs_fstat, uv_fs_lstat, uv_fs_stat,
    uv_fs_utime,
};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{
    FileID, FileInfo, int32_t, size_t, ssize_t, uint64_t, uv_file, uv_gid_t, uv_stat_t, uv_uid_t,
    vim_acl_T,
};
use ::libc::{__errno_location, getuid, getxattr, listxattr, setxattr};

/// The `errno` values [`os_copy_xattr`] treats as "this filesystem simply
/// does not do that", plus the three it reports.
const ENOTSUP: c_int = 95;
const EPERM: c_int = 1;
const E2BIG: c_int = 7;
const EACCES: c_int = 13;
const ERANGE: c_int = 34;

/// The three `E15xx` messages [`os_copy_xattr`] reports. Read-only text, so
/// `CStr` constants rather than the mutable `[c_char; N]` statics c2rust
/// transmuted the C string literals into.
const E_XATTR_ERANGE: &CStr = c"E1506: Buffer too small to copy xattr value or key";
const E_XATTR_E2BIG: &CStr =
    c"E1508: Size of the extended attribute value is larger than the maximum size allowed";
const E_XATTR_OTHER: &CStr = c"E1509: Error occurred when reading or writing extended attribute";

/// `stat(2)` on `name` into `statbuf`. Answers 0 or a libuv error code, and
/// leaves `statbuf` untouched on failure.
///
/// # Safety
/// `name` must be null or a NUL-terminated string, and `statbuf` writable.
pub(crate) unsafe fn os_stat(name: *const c_char, statbuf: *mut uv_stat_t) -> c_int {
    if name.is_null() {
        return UV_EINVAL;
    }
    fs_request(
        // SAFETY: the caller's NUL-terminated path.
        |request| unsafe { uv_fs_stat(NO_LOOP, request, name, None) },
        |result, request| {
            if result == LIBUV_SUCCESS {
                // SAFETY: the caller's out-parameter.
                unsafe { *statbuf = request.statbuf };
            }
            result
        },
    )
}

/// `name`'s permission bits — the whole `st_mode`, in fact — or a negative
/// libuv error code.
///
/// # Safety
/// `name` must be null or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getperm(name: *const c_char) -> int32_t {
    let mut statbuf = UV_STAT_T_INIT;
    // SAFETY: the caller's path; `statbuf` is this frame's.
    let stat_result = unsafe { os_stat(name, &raw mut statbuf) };
    if stat_result == LIBUV_SUCCESS {
        statbuf.st_mode as int32_t
    } else {
        stat_result
    }
}

/// Sets `name`'s permission bits. Answers `OK` or `FAIL`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setperm(name: *const c_char, perm: c_int) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_ok(|req| unsafe { uv_fs_chmod(NO_LOOP, req, name, perm, None) })
}

/// The keys `listxattr` answered, which is a run of NUL-terminated names.
///
/// Upstream walks it with `key += strlen(key) + 1`; this stops at the end of
/// the buffer instead of past it, which is the same walk for any answer the
/// kernel actually gives.
fn xattr_keys(blob: &[u8]) -> impl Iterator<Item = &CStr> {
    let mut rest = blob;
    ::core::iter::from_fn(move || {
        let key = CStr::from_bytes_until_nul(rest).ok()?;
        rest = &rest[key.to_bytes().len() + 1..];
        Some(key)
    })
}

/// Copies every extended attribute of `from_file` onto `to_file`.
///
/// Two passes over the key list: the first asks each value's length without
/// reading it, so that the second has one buffer big enough for all of them.
/// A filesystem that does not do xattrs at all answers nothing and is not an
/// error; the three that are get an `E15xx` message.
///
/// # Safety
/// Both paths must be null or NUL-terminated strings.
pub unsafe fn os_copy_xattr(from_file: *const c_char, to_file: *const c_char) {
    if from_file.is_null() {
        return;
    }
    // SAFETY: the caller's NUL-terminated path; a null list is how the size
    // is asked for.
    let size = unsafe { listxattr(from_file, ptr::null_mut(), 0) };
    if size <= 0 {
        // Not supported, or no attributes to copy.
        return;
    }
    let mut keys = vec![0u8; size as size_t];
    // SAFETY: `keys` holds exactly the `size` bytes just asked for.
    let used = unsafe { listxattr(from_file, keys.as_mut_ptr().cast(), keys.len()) };
    // `errno` is read below rather than each call's return value, and it is
    // sticky, so it starts clear. It is not cleared again inside the loop:
    // upstream reports the first failure that set it.
    // SAFETY: `__errno_location` answers this thread's own slot.
    unsafe { *__errno_location() = 0 };
    let keys = &keys[..used.max(0) as size_t];

    let mut max_vallen: ssize_t = 0;
    let mut val: Vec<u8> = Vec::new();
    let mut errmsg: Option<&CStr> = None;
    'rounds: for round in 0..2 {
        let copying = round == 1;
        for key in xattr_keys(keys) {
            // Round zero passes no buffer, which is how `getxattr` is asked
            // for a length.
            let (val_ptr, val_len) = if copying {
                (val.as_mut_ptr().cast::<c_void>(), max_vallen as size_t)
            } else {
                (ptr::null_mut(), 0)
            };
            // SAFETY: `key` is NUL-terminated, and `val_ptr` addresses
            // `val_len` writable bytes (or is null with `val_len` zero).
            let vallen = unsafe { getxattr(from_file, key.as_ptr(), val_ptr, val_len) };
            let copied = vallen >= 0
                && copying
                // SAFETY: `val`'s first `vallen` bytes were just written.
                && unsafe { setxattr(to_file, key.as_ptr(), val_ptr, vallen as size_t, 0) } == 0;
            // SAFETY: this thread's own `errno`.
            let error = unsafe { *__errno_location() };
            if !copied && error != 0 {
                match error {
                    E2BIG => {
                        errmsg = Some(E_XATTR_E2BIG);
                        break 'rounds;
                    }
                    // The filesystem does not do this attribute; skip it.
                    ENOTSUP | EACCES | EPERM => {}
                    ERANGE => {
                        errmsg = Some(E_XATTR_ERANGE);
                        break 'rounds;
                    }
                    _ => {
                        errmsg = Some(E_XATTR_OTHER);
                        break 'rounds;
                    }
                }
            }
            if !copying && vallen > max_vallen {
                max_vallen = vallen;
            }
        }
        if copying {
            break;
        }
        val = vec![0u8; max_vallen as size_t + 1];
    }
    if let Some(errmsg) = errmsg {
        // SAFETY: the message is a NUL-terminated constant.
        unsafe { emsg(gettext(errmsg.as_ptr())) };
    }
}

/// Access control lists, which nvim does not implement on any platform:
/// upstream's `os_get_acl` answers NULL and the other two are `if (aclent
/// == NULL) return;` followed by nothing. Kept because `fileio.c` calls all
/// three around every write.
pub fn os_get_acl(_fname: *const c_char) -> vim_acl_T {
    ptr::null_mut()
}

pub fn os_set_acl(_fname: *const c_char, _aclent: vim_acl_T) {}

pub fn os_free_acl(_aclent: vim_acl_T) {}

/// Whether the current user owns `fname`.
///
/// Asks through both `stat` and `lstat` — the answer has to hold for the
/// link as well as its target, which is what makes it worth trusting.
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn os_file_owned(fname: *const c_char) -> bool {
    // SAFETY: `getuid` cannot fail, and `fname` is the caller's path.
    unsafe {
        let uid = getuid() as uint64_t;
        let mut finfo = FileInfo {
            stat: UV_STAT_T_INIT,
        };
        let file_owned = os_fileinfo(fname, &raw mut finfo) && finfo.stat.st_uid == uid;
        let link_owned = os_fileinfo_link(fname, &raw mut finfo) && finfo.stat.st_uid == uid;
        file_owned && link_owned
    }
}

/// `chown(2)`. An owner or group of -1 leaves that ID alone.
///
/// # Safety
/// `path` must be a NUL-terminated string.
pub unsafe fn os_chown(path: *const c_char, owner: uv_uid_t, group: uv_gid_t) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_chown(NO_LOOP, req, path, owner, group, None) })
}

/// `fchown(2)`. An owner or group of -1 leaves that ID alone.
///
/// # Safety
/// `fd` must be a descriptor this process owns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fchown(fd: c_int, owner: uv_uid_t, group: uv_gid_t) -> c_int {
    // SAFETY: the caller's descriptor.
    fs_result(|req| unsafe { uv_fs_fchown(NO_LOOP, req, fd as uv_file, owner, group, None) })
}

/// Sets `path`'s access and modification times, in seconds since the epoch.
///
/// # Safety
/// `path` must be a NUL-terminated string.
pub unsafe fn os_file_settime(path: *const c_char, atime: c_double, mtime: c_double) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_utime(NO_LOOP, req, path, atime, mtime, None) })
}

/// Whether `name` may be read.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_readable(name: *const c_char) -> bool {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name, R_OK, None) }) == 0
}

/// Whether `name` may be written: 0 not at all, 1 as a file, 2 as a
/// directory.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_writable(name: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    if fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name, W_OK, None) }) != 0 {
        return 0;
    }
    // SAFETY: same.
    if unsafe { os_isdir(name) } { 2 } else { 1 }
}

/// Fills `file_info` from `path`, answering whether it could.
///
/// # Safety
/// `path` must be null or a NUL-terminated string, and `file_info` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo(path: *const c_char, file_info: *mut FileInfo) -> bool {
    // SAFETY: the caller's out-parameter; upstream zeroes it first so that a
    // failed call leaves a defined value behind.
    unsafe {
        *file_info = FileInfo {
            stat: UV_STAT_T_INIT,
        }
    };
    // SAFETY: the caller's path, and the stat buffer inside their `FileInfo`.
    unsafe { os_stat(path, &raw mut (*file_info).stat) == LIBUV_SUCCESS }
}

/// [`os_fileinfo`] without following a symlink — the link's own metadata.
///
/// # Safety
/// `path` must be null or a NUL-terminated string, and `file_info` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_link(path: *const c_char, file_info: *mut FileInfo) -> bool {
    // SAFETY: the caller's out-parameter.
    unsafe {
        *file_info = FileInfo {
            stat: UV_STAT_T_INIT,
        }
    };
    if path.is_null() {
        return false;
    }
    fs_request(
        // SAFETY: the caller's NUL-terminated path.
        |request| unsafe { uv_fs_lstat(NO_LOOP, request, path, None) },
        |result, request| {
            let ok = result == LIBUV_SUCCESS;
            if ok {
                // SAFETY: the caller's out-parameter.
                unsafe { (*file_info).stat = request.statbuf };
            }
            ok
        },
    )
}

/// [`os_fileinfo`] for an already-open descriptor.
///
/// # Safety
/// `file_descriptor` must be a descriptor this process owns, and
/// `file_info` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_fd(file_descriptor: c_int, file_info: *mut FileInfo) -> bool {
    // SAFETY: the caller's out-parameter.
    unsafe {
        *file_info = FileInfo {
            stat: UV_STAT_T_INIT,
        }
    };
    fs_request(
        // SAFETY: the caller's descriptor.
        |request| unsafe { uv_fs_fstat(NO_LOOP, request, file_descriptor as uv_file, None) },
        |result, request| {
            let ok = result == LIBUV_SUCCESS;
            if ok {
                // SAFETY: the caller's out-parameter.
                unsafe { (*file_info).stat = request.statbuf };
            }
            ok
        },
    )
}

/// Whether two `FileInfo`s name the same file — the (device, inode) pair,
/// not the path.
///
/// # Safety
/// Both must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_id_equal(
    file_info_1: *const FileInfo,
    file_info_2: *const FileInfo,
) -> bool {
    // SAFETY: the caller's readable `FileInfo`s.
    unsafe {
        (*file_info_1).stat.st_ino == (*file_info_2).stat.st_ino
            && (*file_info_1).stat.st_dev == (*file_info_2).stat.st_dev
    }
}

/// The (device, inode) pair out of a `FileInfo`.
///
/// # Safety
/// `file_info` must be readable and `file_id` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_id(file_info: *const FileInfo, file_id: *mut FileID) {
    // SAFETY: the caller's in- and out-parameters.
    unsafe {
        (*file_id).inode = (*file_info).stat.st_ino;
        (*file_id).device_id = (*file_info).stat.st_dev;
    }
}

/// # Safety
/// `file_info` must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_inode(file_info: *const FileInfo) -> uint64_t {
    // SAFETY: the caller's readable `FileInfo`.
    unsafe { (*file_info).stat.st_ino }
}

/// # Safety
/// `file_info` must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_size(file_info: *const FileInfo) -> uint64_t {
    // SAFETY: the caller's readable `FileInfo`.
    unsafe { (*file_info).stat.st_size }
}

/// # Safety
/// `file_info` must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_hardlinks(file_info: *const FileInfo) -> uint64_t {
    // SAFETY: the caller's readable `FileInfo`.
    unsafe { (*file_info).stat.st_nlink }
}

/// # Safety
/// `file_info` must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_blocksize(file_info: *const FileInfo) -> uint64_t {
    // SAFETY: the caller's readable `FileInfo`.
    unsafe { (*file_info).stat.st_blksize }
}

/// The (device, inode) pair for `path`, answering whether it could be read.
///
/// # Safety
/// `path` must be null or a NUL-terminated string, and `file_id` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid(path: *const c_char, file_id: *mut FileID) -> bool {
    let mut statbuf = UV_STAT_T_INIT;
    // SAFETY: the caller's path; `statbuf` is this frame's.
    if unsafe { os_stat(path, &raw mut statbuf) } != LIBUV_SUCCESS {
        return false;
    }
    // SAFETY: the caller's out-parameter.
    unsafe {
        (*file_id).inode = statbuf.st_ino;
        (*file_id).device_id = statbuf.st_dev;
    }
    true
}

/// Whether two `FileID`s name the same file.
///
/// # Safety
/// Both must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid_equal(
    file_id_1: *const FileID,
    file_id_2: *const FileID,
) -> bool {
    // SAFETY: the caller's readable `FileID`s.
    unsafe {
        (*file_id_1).inode == (*file_id_2).inode && (*file_id_1).device_id == (*file_id_2).device_id
    }
}

/// [`os_fileid_equal`] against a `FileInfo` instead of a second `FileID`.
///
/// # Safety
/// Both must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid_equal_fileinfo(
    file_id: *const FileID,
    file_info: *const FileInfo,
) -> bool {
    // SAFETY: the caller's readable structs.
    unsafe {
        (*file_id).inode == (*file_info).stat.st_ino
            && (*file_id).device_id == (*file_info).stat.st_dev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xattr_keys_walks_the_nul_separated_list() {
        let keys: Vec<&str> = xattr_keys(b"user.one\0user.two\0")
            .map(|k| k.to_str().unwrap())
            .collect();
        assert_eq!(keys, ["user.one", "user.two"]);
        assert_eq!(xattr_keys(b"").count(), 0);
        // A trailing run with no NUL is not a key; upstream would read past
        // the buffer here, and no kernel answers that shape.
        assert_eq!(xattr_keys(b"user.one\0trunc").count(), 1);
    }
}
