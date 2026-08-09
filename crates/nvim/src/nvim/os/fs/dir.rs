//! Creating, removing and walking directories, and moving files between
//! them.
//!
//! Same shape as the rest of `os/fs`: one [`fs_request`] per call, with
//! `req.result` read back and the request cleaned up on the way out. The
//! two exceptions are [`os_mkdir_recurse`] and [`os_file_mkdir`], which walk
//! a path apart and put it back together and are the only string work here.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr;

use super::{
    LIBUV_SUCCESS, NO_LOOP, PATHSEP, TEMP_FILE_PATH_MAXLEN, UV_EOF, UV_STAT_T_INIT, fs_ok,
    fs_request, fs_result, os_isdir, os_stat,
};
use crate::semsg_c;
use crate::src::nvim::event::libuv::{
    uv_fs_mkdir, uv_fs_mkdtemp, uv_fs_rename, uv_fs_req_cleanup, uv_fs_rmdir, uv_fs_scandir,
    uv_fs_scandir_next, uv_fs_unlink, uv_strerror,
};
use crate::src::nvim::main::{e_mkdir, e_noname};
use crate::src::nvim::memory::{xfree, xmemdupz, xstrlcpy};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::path::{
    FullName_save, dir_of_file_exists, get_past_head, path_tail_with_sep, vim_ispathsep,
};
use crate::src::nvim::types::{Directory, int32_t, size_t};

/// Whether `path` names anything at all.
///
/// # Safety
/// `path` must be null or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_path_exists(path: *const c_char) -> bool {
    let mut statbuf = UV_STAT_T_INIT;
    // SAFETY: the caller's path; `statbuf` is this frame's.
    unsafe { os_stat(path, &raw mut statbuf) == LIBUV_SUCCESS }
}

/// Renames `path` to `new_path`. Answers `OK` or `FAIL`.
///
/// # Safety
/// Both must be NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_rename(path: *const c_char, new_path: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated paths.
    fs_ok(|req| unsafe { uv_fs_rename(NO_LOOP, req, path, new_path, None) })
}

/// Makes one directory. Answers 0 or a libuv error code.
///
/// # Safety
/// `path` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir(path: *const c_char, mode: int32_t) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_mkdir(NO_LOOP, req, path, mode as c_int, None) })
}

/// Makes `dir` and every missing directory above it. Answers 0, or the
/// libuv error code of the first `mkdir` that failed.
///
/// Two walks over one buffer: the first truncates `dir` at each separator
/// until what is left is a directory that exists, and the second puts the
/// components back one at a time, creating each. `failed_dir` receives an
/// allocated copy of the path that could not be made; a non-null `created`
/// receives the full name of the *first* directory that was.
///
/// # Safety
/// `dir` must be a NUL-terminated string and `failed_dir` writable;
/// `created` must be null or point at a pointer that is null or owned.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir_recurse(
    dir: *const c_char,
    mode: int32_t,
    failed_dir: *mut *mut c_char,
    created: *mut *mut c_char,
) -> c_int {
    // SAFETY: the caller's NUL-terminated directory name.
    let mut curdir: Vec<u8> = unsafe { CStr::from_ptr(dir) }.to_bytes_with_nul().to_vec();
    let real_end = curdir.len() - 1;
    // Where the walk back has to stop: "/" on Unix, "c:/" on Windows.
    // SAFETY: `curdir` is a NUL-terminated string this frame owns, and
    // `get_past_head` answers a pointer inside it.
    let past_head = unsafe {
        get_past_head(curdir.as_ptr().cast()).offset_from(curdir.as_ptr().cast::<c_char>())
    } as usize;
    let past_head_save = curdir[past_head];

    let mut e = real_end;
    // SAFETY: `curdir` is NUL-terminated throughout, and
    // `path_tail_with_sep` answers a pointer inside it.
    while !unsafe { os_isdir(curdir.as_ptr().cast()) } {
        // SAFETY: as above.
        e = unsafe {
            path_tail_with_sep(curdir.as_mut_ptr().cast())
                .offset_from(curdir.as_ptr().cast::<c_char>())
        } as usize;
        if e <= past_head {
            curdir[past_head] = 0;
            break;
        }
        curdir[e] = 0;
    }

    while e != real_end {
        if e > past_head {
            curdir[e] = PATHSEP as u8;
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
        let all_separators = component.iter().all(|&b| b == PATHSEP as u8);
        e += component_len;
        if e == real_end && all_separators {
            // The path ends with something like "////". Ignore this.
            break;
        }
        // SAFETY: `curdir` is NUL-terminated at `e`.
        let ret = unsafe { os_mkdir(curdir.as_ptr().cast(), mode) };
        if ret != 0 {
            // SAFETY: the caller's out-parameter, which takes the copy over.
            unsafe { *failed_dir = xmemdupz(curdir.as_ptr().cast(), e).cast() };
            return ret;
        }
        // SAFETY: the caller's out-parameter, checked non-null.
        if !created.is_null() && unsafe { (*created).is_null() } {
            // SAFETY: same; it takes the allocation over.
            unsafe { *created = FullName_save(curdir.as_ptr().cast(), false) };
        }
    }
    0
}

/// Makes the directory `fname` would live in, if it is missing.
///
/// Answers 0, or the libuv error code — having reported it — when the
/// directories could not be made. `fname` is left as it was found.
///
/// # Safety
/// `fname` must be a writable NUL-terminated string.
pub unsafe extern "C" fn os_file_mkdir(fname: *mut c_char, mode: int32_t) -> c_int {
    // SAFETY: the caller's writable NUL-terminated file name. The tail is
    // cut off with a NUL for the duration and put back before returning.
    unsafe {
        if dir_of_file_exists(fname) {
            return 0;
        }
        let tail = path_tail_with_sep(fname);
        // `tail` is past `fname`'s start here (`dir_of_file_exists`
        // answers true when they are equal), so the byte before it exists
        // even when the tail itself is empty — which is the case upstream's
        // `tail + strlen(tail) - 1` is written for.
        let last_char = *tail.add(CStr::from_ptr(tail).to_bytes().len()).sub(1);
        if vim_ispathsep(last_char as c_int) {
            emsg(gettext((&raw const e_noname).cast()));
            return -1;
        }
        let c = *tail;
        *tail = 0;
        let mut failed_dir: *mut c_char = ptr::null_mut();
        let r = os_mkdir_recurse(fname, mode, &raw mut failed_dir, ptr::null_mut());
        if r < 0 {
            semsg_c!(
                gettext((&raw const e_mkdir).cast()),
                failed_dir,
                uv_strerror(r),
            );
            xfree(failed_dir.cast());
        }
        *tail = c;
        r
    }
}

/// Makes a uniquely named temporary directory from `templ`, whose trailing
/// `XXXXXX` is replaced, and writes its path into `path`.
///
/// # Safety
/// `templ` must be a NUL-terminated string and `path` must address
/// [`TEMP_FILE_PATH_MAXLEN`] writable bytes.
pub unsafe extern "C" fn os_mkdtemp(templ: *const c_char, path: *mut c_char) -> c_int {
    // `request.path` is the directory libuv made, and cleanup frees it.
    fs_request(
        // SAFETY: the caller's NUL-terminated template.
        |request| unsafe { uv_fs_mkdtemp(NO_LOOP, request, templ, None) },
        |result, request| {
            if result == LIBUV_SUCCESS {
                // SAFETY: `request.path` is libuv's NUL-terminated answer,
                // alive until cleanup, and `path` is the caller's buffer.
                unsafe { xstrlcpy(path, request.path, TEMP_FILE_PATH_MAXLEN as size_t) };
            }
            result
        },
    )
}

/// Removes an empty directory. Answers 0 or a libuv error code.
///
/// # Safety
/// `path` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_rmdir(path: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_rmdir(NO_LOOP, req, path, None) })
}

/// Opens `path` for walking, answering whether it holds anything.
///
/// The `Directory` owns a live `uv_fs_t` until [`os_closedir`] runs, which
/// is why this one is not an [`fs_request`].
///
/// # Safety
/// `dir` must be writable and `path` a NUL-terminated string.
pub unsafe extern "C" fn os_scandir(dir: *mut Directory, path: *const c_char) -> bool {
    // SAFETY: the caller's `Directory` and NUL-terminated path.
    let r = unsafe { uv_fs_scandir(NO_LOOP, &raw mut (*dir).request, path, 0, None) };
    if r < 0 {
        // SAFETY: the request is the one just started, however it went.
        unsafe { os_closedir(dir) };
    }
    r >= 0
}

/// The next entry's name, or null when the walk is over.
///
/// # Safety
/// `dir` must be a `Directory` [`os_scandir`] succeeded on.
pub unsafe extern "C" fn os_scandir_next(dir: *mut Directory) -> *const c_char {
    // SAFETY: the caller's open `Directory`; the name lives in its request.
    unsafe {
        let err = uv_fs_scandir_next(&raw mut (*dir).request, &raw mut (*dir).ent);
        if err != UV_EOF {
            (*dir).ent.name
        } else {
            ptr::null()
        }
    }
}

/// Releases what [`os_scandir`] allocated.
///
/// # Safety
/// `dir` must be a `Directory` [`os_scandir`] was called on.
pub unsafe extern "C" fn os_closedir(dir: *mut Directory) {
    // SAFETY: the caller's `Directory`, whose request is theirs to clean up.
    unsafe { uv_fs_req_cleanup(&raw mut (*dir).request) };
}

/// Removes a file. Answers 0 or a libuv error code.
///
/// # Safety
/// `path` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_remove(path: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_unlink(NO_LOOP, req, path, None) })
}
