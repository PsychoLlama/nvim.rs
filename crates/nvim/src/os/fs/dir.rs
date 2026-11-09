//! Creating, removing and walking directories, and moving files between
//! them.
//!
//! Same shape as the rest of `os/fs`: one [`fs_request`] per call, with
//! `req.result` read back and the request cleaned up on the way out. The
//! two exceptions are [`mkdir_recurse`] and [`file_mkdir`], which walk
//! a path apart and put it back together and are the only string work here.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::PATHSEP;
use crate::os::uv_error::UV_EOF;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::{LIBUV_SUCCESS, NO_LOOP, dir_exists, fs_ok, fs_request, fs_result};
use crate::cstr;
use crate::event::libuv::{
    uv_fs_mkdir, uv_fs_mkdtemp, uv_fs_rename, uv_fs_req_cleanup, uv_fs_rmdir, uv_fs_scandir,
    uv_fs_scandir_next, uv_fs_unlink,
};
use crate::message::{E_NONAME, emsg};
use crate::message_fmt::msg_cstr;
use crate::os::cshim::gettext;
use crate::os::fs::Directory;
use crate::os::libc::int32_t;
use crate::path::{
    dir_of_file_exists, full_name_save, past_head_index, tail_with_sep_index, vim_ispathsep,
};
use crate::semsg;

/// Whether `path` names anything at all.
pub fn path_exists(path: &CStr) -> bool {
    super::stat(path).is_ok()
}

/// Renames `path` to `new_path`. Answers `OK` or `FAIL`.
pub fn os_rename(path: &CStr, new_path: &CStr) -> Result<(), c_int> {
    // SAFETY: both are NUL-terminated and outlive the call.
    fs_ok(|req| unsafe { uv_fs_rename(NO_LOOP, req, path.as_ptr(), new_path.as_ptr(), None) })
}

/// Makes one directory. Answers 0 or a libuv error code.
pub fn os_mkdir(path: &CStr, mode: int32_t) -> c_int {
    // SAFETY: `path` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_mkdir(NO_LOOP, req, path.as_ptr(), mode as c_int, None) })
}

/// Why [`mkdir_recurse`] gave up: libuv's code, the directory that could not
/// be made, and libuv's own word for the code -- which is here rather than at
/// the caller because `uv_strerror` is a C call and this is the side of the
/// boundary that makes those.
#[derive(Debug)]
pub struct MkdirFailure {
    pub code: c_int,
    pub dir: Option<CString>,
    pub why: &'static CStr,
}

/// Makes `dir` and every missing directory above it. Success answers the
/// full name of the *first* directory it had to make, or `None` when they
/// all existed already.
///
/// Two walks over one buffer: the first truncates `dir` at each separator
/// until what is left is a directory that exists, and the second puts the
/// components back one at a time, creating each.
pub fn mkdir_recurse(dir: &CStr, mode: int32_t) -> Result<Option<CString>, MkdirFailure> {
    let mut curdir: Vec<u8> = dir.to_bytes_with_nul().to_vec();
    let real_end = curdir.len() - 1;
    // Where the walk back has to stop: "/" on Unix, "c:/" on Windows.
    let past_head = past_head_index(cstr::until_nul(&curdir));
    let past_head_save = curdir[past_head];

    let mut e = real_end;
    // `curdir` is NUL-terminated throughout.
    while !dir_exists(cstr::in_bytes(&curdir)) {
        e = tail_with_sep_index(cstr::until_nul(&curdir));
        if e <= past_head {
            curdir[past_head] = 0;
            break;
        }
        curdir[e] = 0;
    }

    let mut created = None;
    while e != real_end {
        if e > past_head {
            curdir[e] = PATHSEP;
        } else {
            curdir[past_head] = past_head_save;
        }
        // The component the last truncation cut off, which putting the
        // separator back has just re-joined.
        let component_len = curdir[e..]
            .iter()
            .position(|&b| b == 0)
            // `curdir` always ends in the NUL it was copied with.
            .unwrap_or(real_end - e);
        let component = &curdir[e..e + component_len];
        let all_separators = component.iter().all(|&b| b == PATHSEP);
        e += component_len;
        if e == real_end && all_separators {
            // The path ends with something like "////". Ignore this.
            break;
        }
        // `curdir` is NUL-terminated at `e`, and the whole of it at
        // `real_end`, so the walk always finds a terminator.
        let so_far = CStr::from_bytes_until_nul(&curdir).unwrap_or(c"");
        let code = os_mkdir(so_far, mode);
        if code != 0 {
            return Err(MkdirFailure {
                code,
                dir: Some(cstr::owned(&curdir[..e])),
                why: super::os_strerror(code),
            });
        }
        if created.is_none() {
            created = Some(cstr::owned(&full_name_save(cstr::in_bytes(&curdir), false)));
        }
    }
    Ok(created)
}

/// Makes the directory `fname` would live in, if it is missing.
///
/// `Err` carries the libuv error code -- having reported it -- when the
/// directories could not be made (-1 for a name that is all directory).
pub(crate) fn file_mkdir(fname: &CStr, mode: int32_t) -> Result<(), c_int> {
    let name = fname.to_bytes();
    if dir_of_file_exists(name) {
        return Ok(());
    }
    let tail = tail_with_sep_index(name);
    // `tail` is past the start here (`dir_of_file_exists` answers true when
    // it is not), so the byte before the end exists even when the tail
    // itself is empty -- the case upstream's `tail + strlen(tail) - 1` is
    // written for.
    if vim_ispathsep(c_int::from(name[name.len() - 1])) {
        emsg(gettext(E_NONAME));
        return Err(-1);
    }
    match mkdir_recurse(&cstr::owned(&name[..tail]), mode) {
        Ok(_) => Ok(()),
        Err(failure) => {
            let dir = failure.dir.as_deref().unwrap_or(c"");
            semsg!(
                "E739: Cannot create directory {}: {}",
                msg_cstr(dir),
                msg_cstr(failure.why)
            );
            Err(failure.code)
        }
    }
}

/// Makes a uniquely named temporary directory from `templ`, whose trailing
/// `XXXXXX` is replaced: its path, or a libuv error code.
pub fn os_mkdtemp(templ: &CStr) -> Result<CString, c_int> {
    // `request.path` is the directory libuv made, and cleanup frees it.
    let mut made = None;
    let result = fs_request(
        // SAFETY: `templ` is NUL-terminated and outlives the call.
        |request| unsafe { uv_fs_mkdtemp(NO_LOOP, request, templ.as_ptr(), None) },
        |result, request| {
            if result == LIBUV_SUCCESS {
                // SAFETY: `request.path` is libuv's NUL-terminated answer,
                // alive until cleanup; it is copied out here.
                made = Some(unsafe { CStr::from_ptr(request.path) }.to_owned());
            }
            result
        },
    );
    made.ok_or(result)
}

/// Removes an empty directory. Answers 0 or a libuv error code.
pub fn os_rmdir(path: &CStr) -> c_int {
    // SAFETY: `path` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_rmdir(NO_LOOP, req, path.as_ptr(), None) })
}

/// Opens `path` for walking, answering whether it holds anything.
///
/// The `Directory` owns a live `uv_fs_t` until [`os_closedir`] runs, which
/// is why this one is not an [`fs_request`].
///
pub fn os_scandir(dir: &mut Directory, path: &CStr) -> bool {
    // SAFETY: the request is `dir`'s own, and `path` is NUL-terminated.
    let r = unsafe { uv_fs_scandir(NO_LOOP, &raw mut dir.request, path.as_ptr(), 0, None) };
    if r < 0 {
        os_closedir(dir);
    }
    r >= 0
}

/// The next entry's name, or `None` when the walk is over. The name lives
/// in `dir`'s request until the next call, which the borrow says.
pub fn os_scandir_next(dir: &mut Directory) -> Option<&CStr> {
    // SAFETY: the request is `dir`'s own; a walk that was never started, or
    // has ended, answers `UV_EOF` and touches nothing.
    let err = unsafe { uv_fs_scandir_next(&raw mut dir.request, &raw mut dir.ent) };
    // SAFETY: on success `ent.name` is libuv's NUL-terminated name, alive
    // in the request until the next call -- which needs `dir` mutably, so
    // it cannot happen while the answer is borrowed.
    (err != UV_EOF).then(|| unsafe { CStr::from_ptr(dir.ent.name) })
}

/// Releases what [`os_scandir`] allocated.
pub fn os_closedir(dir: &mut Directory) {
    // SAFETY: the request is `dir`'s own, and cleaning a request that was
    // never started is libuv's own no-op.
    unsafe { uv_fs_req_cleanup(&raw mut dir.request) };
}

/// Removes a file. Answers 0 or a libuv error code.
pub fn os_remove(path: &CStr) -> c_int {
    // SAFETY: `path` is NUL-terminated and outlives the call.
    fs_result(|req| unsafe { uv_fs_unlink(NO_LOOP, req, path.as_ptr(), None) })
}
