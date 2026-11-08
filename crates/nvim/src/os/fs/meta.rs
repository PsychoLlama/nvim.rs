//! File metadata: permissions, ownership, timestamps and identity.
//!
//! Every answer here comes from libuv's `uv_fs_*` synchronous calls, which
//! is why each function is one [`fs_request`] around a `uv_fs_t`. `FileInfo`
//! and `FileID` are the two shapes upstream wraps `uv_stat_t` in — the first
//! is the whole stat buffer, the second only the (device, inode) pair that
//! says two paths are the same file.
//!
//! The exception is [`copy_xattr`], which is `getxattr`/`setxattr` rather
//! than libuv and is the only thing in this file with a shape of its own.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::ffi::{CStr, c_double, c_int, c_void};
use core::ptr;

use std::fs::File;
use std::io;
use std::os::fd::IntoRawFd as _;

use super::{LIBUV_SUCCESS, NO_LOOP, R_OK, W_OK, dir_exists, fs_ok, fs_request, fs_result};
use crate::event::libuv::{
    uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close, uv_fs_lstat, uv_fs_stat, uv_fs_utime,
};
use crate::event::uv::{uv_gid_t, uv_stat_t, uv_uid_t};
use crate::log::g_stats;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::fs::{FileID, FileInfo};
use crate::os::libc::{int32_t, size_t, ssize_t, uint64_t};
use crate::os::uv_error::uv_code;
use crate::types::VimAcl;
use ::libc::{E2BIG, getxattr, listxattr, setxattr};

/// The `errno` values [`copy_xattr`] treats as "this filesystem simply
/// does not do that", plus the three it reports.
const ENOTSUP: c_int = 95;
const EPERM: c_int = 1;
const EACCES: c_int = 13;
const ERANGE: c_int = 34;

/// The three `E15xx` messages [`copy_xattr`] reports. Read-only text, so
/// `CStr` constants rather than the mutable `[c_char; N]` statics c2rust
/// transmuted the C string literals into.
const E_XATTR_ERANGE: &CStr = c"E1506: Buffer too small to copy xattr value or key";
const E_XATTR_E2BIG: &CStr =
    c"E1508: Size of the extended attribute value is larger than the maximum size allowed";
const E_XATTR_OTHER: &CStr = c"E1509: Error occurred when reading or writing extended attribute";

/// `stat(2)` on `name`: its stat buffer, or a libuv error code.
pub(crate) fn stat(name: &CStr) -> Result<uv_stat_t, c_int> {
    fs_request(
        // SAFETY: `name` is NUL-terminated and outlives the call.
        |request| unsafe { uv_fs_stat(NO_LOOP, request, name.as_ptr(), None) },
        |result, request| {
            if result == LIBUV_SUCCESS {
                Ok(request.statbuf)
            } else {
                Err(result)
            }
        },
    )
}

/// `name`'s permission bits -- the whole `st_mode`, in fact -- or a negative
/// libuv error code.
pub fn getperm(name: &CStr) -> int32_t {
    match stat(name) {
        Ok(statbuf) => statbuf.st_mode as int32_t,
        Err(error) => error,
    }
}

/// Sets `name`'s permission bits, or answers the libuv error code.
pub fn os_setperm(name: &CStr, perm: c_int) -> Result<(), c_int> {
    // SAFETY: `name` is NUL-terminated and outlives the call.
    fs_ok(|req| unsafe { uv_fs_chmod(NO_LOOP, req, name.as_ptr(), perm, None) })
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
pub fn copy_xattr(from: &CStr, to: &CStr) {
    let (from_file, to_file) = (from.as_ptr(), to.as_ptr());
    // SAFETY: a NUL-terminated path; a null list is how the size is asked
    // for.
    let size = unsafe { listxattr(from_file, ptr::null_mut(), 0) };
    if size <= 0 {
        // Not supported, or no attributes to copy.
        return;
    }
    let mut keys = vec![0u8; size as size_t];
    // SAFETY: `keys` holds exactly the `size` bytes just asked for.
    let used = unsafe { listxattr(from_file, keys.as_mut_ptr().cast(), keys.len()) };
    let keys = &keys[..used.max(0) as size_t];
    // Upstream clears `errno` here and then tests it after every call rather
    // than each call's own answer, so a failure stays reported until another
    // one replaces it. `error` is that `errno`: zero to start with, and set
    // by each call that fails.
    let mut error: c_int = 0;
    let failed = || io::Error::last_os_error().raw_os_error().unwrap_or(0);

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
            if vallen < 0 {
                error = failed();
            }
            let copied = vallen >= 0
                && copying
                // SAFETY: `val`'s first `vallen` bytes were just written.
                && (unsafe { setxattr(to_file, key.as_ptr(), val_ptr, vallen as size_t, 0) } == 0
                    || {
                        error = failed();
                        false
                    });
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
        emsg(gettext(errmsg));
    }
}

/// Access control lists, which nvim does not implement on any platform:
/// upstream's `os_get_acl` answers NULL and the other two are `if (aclent
/// == NULL) return;` followed by nothing. Kept because `fileio.c` calls all
/// three around every write.
pub fn os_get_acl(_fname: &CStr) -> VimAcl {
    ptr::null_mut()
}

pub fn os_set_acl(_fname: &CStr, _aclent: VimAcl) {}

pub fn os_free_acl(_aclent: VimAcl) {}

/// Whether the current user owns `fname`.
///
/// Asks through both `stat` and `lstat` — the answer has to hold for the
/// link as well as its target, which is what makes it worth trusting.
pub fn os_file_owned(fname: &CStr) -> bool {
    let uid = uint64_t::from(crate::os::users::os_getuid());
    let owned = |info: Option<FileInfo>| info.is_some_and(|info| info.stat.st_uid == uid);
    owned(file_info(fname)) && owned(link_info(fname))
}

/// `chown(2)`. An owner or group of -1 leaves that ID alone.
pub fn os_chown(path: &CStr, owner: uv_uid_t, group: uv_gid_t) -> c_int {
    // SAFETY: `path` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_chown(NO_LOOP, req, path.as_ptr(), owner, group, None) })
}

/// `fsync(2)` of a file the caller holds, counted in `nvim__stats()`: `Err`
/// is the libuv error code.
pub(crate) fn fsync_file(file: &File) -> Result<(), c_int> {
    let synced = file.sync_all().map_err(|error| uv_code(&error));
    g_stats.with_mut(|stats| stats.fsync += 1);
    synced
}

/// Close `file`, answering the libuv error code the close failed with,
/// which dropping a `File` would swallow.
pub(crate) fn close_file(file: File) -> Result<(), c_int> {
    let fd = file.into_raw_fd();
    // SAFETY: the descriptor the file owned, handed over and closed once.
    match fs_result(|req| unsafe { uv_fs_close(NO_LOOP, req, fd, None) }) {
        0 => Ok(()),
        error => Err(error),
    }
}

/// Sets `path`'s access and modification times, in seconds since the epoch.
pub fn os_file_settime(path: &CStr, atime: c_double, mtime: c_double) -> c_int {
    // SAFETY: `path` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_utime(NO_LOOP, req, path.as_ptr(), atime, mtime, None) })
}

/// Whether `name` may be read.
pub fn os_file_is_readable(name: &CStr) -> bool {
    // SAFETY: `name` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name.as_ptr(), R_OK, None) }) == 0
}

/// Whether `name` may be written: 0 not at all, 1 as a file, 2 as a
/// directory.
pub fn os_file_is_writable(name: &CStr) -> c_int {
    // SAFETY: `name` is NUL-terminated and outlives the call.
    if fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name.as_ptr(), W_OK, None) }) != 0 {
        return 0;
    }
    if dir_exists(name) { 2 } else { 1 }
}

/// Whether `path` names anything, a symbolic link included:
/// [`link_info`] answering only that.
pub(crate) fn link_exists(path: &CStr) -> bool {
    link_info(path).is_some()
}

/// `path`'s metadata, following a symlink, or `None` when it cannot be
/// taken.
pub fn file_info(path: &CStr) -> Option<FileInfo> {
    stat(path).ok().map(|stat| FileInfo { stat })
}

/// [`file_info`] without following a symlink: the link's own metadata, or
/// `None` when it cannot be taken.
pub fn link_info(path: &CStr) -> Option<FileInfo> {
    fs_request(
        // SAFETY: `path` is NUL-terminated and outlives the call.
        |request| unsafe { uv_fs_lstat(NO_LOOP, request, path.as_ptr(), None) },
        |result, request| {
            (result == LIBUV_SUCCESS).then_some(FileInfo {
                stat: request.statbuf,
            })
        },
    )
}

/// The (device, inode) pair for `path`, or `None` when it cannot be read.
pub(crate) fn file_id(path: &CStr) -> Option<FileID> {
    file_info(path).map(|info| info.id())
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
