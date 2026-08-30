//! The filesystem layer: everything nvim asks the OS about a path, a
//! directory or an open file descriptor.
//!
//! Almost all of it is libuv's synchronous `uv_fs_*` family, and almost
//! every function here is the same shape around it — fill a `uv_fs_t`, run
//! it, read `result` back, clean the request up, answer `OK`/`FAIL`. That
//! shape is [`fs_request`] and its two shorthands, written once; upstream
//! writes it out per function and c2rust wrote the zero initialiser out
//! with it, which is where 1,386 of this file's original 2,726 lines went.
//!
//! Metadata is in [`meta`] and directories in [`dir`]; what is left here is
//! the current directory, executability, and the read/write/copy calls that
//! are libc rather than libuv.
//!
//! **Nothing here is exported by name any more**, but the entry points are
//! still raw-pointer functions: `crates/nvim/tests/unit/fs.rs` drives them
//! from outside the crate, and every surviving `unsafe` unit is a libuv or
//! libc call that has no Rust equivalent, so the deny buys no narrower
//! obligation on those. What it does buy is the *rest* of each body — the
//! mode arithmetic, the `$PATH` walk, the short-read loops — as checked
//! code, which is where this file's unchecked lines actually lived.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::os::uv_error::{UV_EAGAIN, UV_EINTR, UV_EINVAL, UV_UNKNOWN};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};

use crate::api::private::helpers::cstr_as_string;
use crate::event::libuv::{
    uv_chdir, uv_cwd, uv_exepath, uv_fs_access, uv_fs_close, uv_fs_copyfile, uv_fs_fsync,
    uv_fs_lstat, uv_fs_open, uv_fs_realpath, uv_fs_req_cleanup, uv_strerror,
    uv_translate_sys_error,
};
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::main::{g_stats, p_verbose, stdin_fd};
use crate::memory::{xfree, xmalloc, xstrlcpy};
use crate::message::{verbose_enter, verbose_leave};
use crate::message_fmt::c_str;
use crate::os::env::os_getenv;
use crate::path::{append_path, gettail_dir, save_abs_path};
use crate::smsg;
use crate::types::libc::STDIN_FILENO;
use crate::types::{
    FAIL, FILE, Failed, OK, OptInt, iovec, ptrdiff_t, size_t, uv__queue, uv__work, uv_buf_t,
    uv_file, uv_fs_t, uv_fs_type, uv_loop_s, uv_loop_t, uv_req_type, uv_stat_t, uv_timespec_t,
};
use crate::ui::ui_call_chdir;
use ::libc::{__errno_location, abort, dup, fcntl, fdopen, read, readv, strerror, write};

pub mod dir;
pub mod meta;

pub use dir::*;
pub use meta::*;

/// Many `uv_fs_*` functions answer this on success.
const LIBUV_SUCCESS: c_int = 0;

/// libuv's "do it now, on this thread" spelling: the synchronous form of a
/// `uv_fs_*` call is a null loop paired with a null callback.
const NO_LOOP: *mut uv_loop_t = ptr::null_mut();

/// `access(2)`'s mode bits, which `uv_fs_access` takes as they are.
const R_OK: c_int = 4;
const W_OK: c_int = 2;
const X_OK: c_int = 1;

/// `open(2)` flags, in the combinations [`fopen_flags`] hands out.
const O_RDONLY: c_int = 0;
const O_WRONLY: c_int = 0o1;
const O_RDWR: c_int = 0o2;
const O_CREAT: c_int = 0o100;
const O_TRUNC: c_int = 0o1000;
const O_APPEND: c_int = 0o2000;

/// `fcntl(2)`'s close-on-exec commands and flag.
const F_GETFD: c_int = 1;
const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;

/// What [`os_nodetype`] answers: an ordinary file or directory, something
/// writable like a character device or socket, or something that is neither.
pub const NODE_NORMAL: c_int = 0;
pub const NODE_WRITABLE: c_int = 1;
pub const NODE_OTHER: c_int = 2;

/// `stat(2)`'s file-type field and the three types this family asks about.
const S_IFMT: u64 = 0o170000;
const S_IFREG: u64 = 0o100000;
const S_IFDIR: u64 = 0o40000;
const S_IFBLK: u64 = 0o60000;

const PATHSEP: c_char = b'/' as c_char;
/// `$PATH`'s entry separator.
const ENV_SEPCHAR: u8 = b':';
/// The buffer [`os_mkdtemp`] fills, sized by its callers.
pub const TEMP_FILE_PATH_MAXLEN: c_int = 256;

/// Whether a `stat(2)` mode names a directory.
fn is_dir(mode: u64) -> bool {
    mode & S_IFMT == S_IFDIR
}

/// `uv_stat_t statbuf` — the buffer [`os_stat`] fills. c2rust wrote all
/// eighteen fields out at each of the five sites that declare one.
const UV_STAT_T_INIT: uv_stat_t = uv_stat_t {
    st_dev: 0,
    st_mode: 0,
    st_nlink: 0,
    st_uid: 0,
    st_gid: 0,
    st_rdev: 0,
    st_ino: 0,
    st_size: 0,
    st_blksize: 0,
    st_blocks: 0,
    st_flags: 0,
    st_gen: 0,
    st_atim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_mtim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_ctim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
    st_birthtim: uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    },
};

/// `uv_fs_t req = { 0 }`. c2rust wrote all sixty-six fields out at each of
/// the twenty-one sites that need one -- 1,386 of this file's lines -- and
/// every one of them is zero, `UV_UNKNOWN_REQ` and `UV_FS_CUSTOM` included.
const UV_FS_T_INIT: uv_fs_t = uv_fs_t {
    data: ptr::null_mut(),
    type_0: 0 as uv_req_type,
    reserved: [ptr::null_mut(); 6],
    fs_type: 0 as uv_fs_type,
    loop_0: ptr::null_mut(),
    cb: None,
    result: 0,
    ptr: ptr::null_mut(),
    path: ptr::null(),
    statbuf: UV_STAT_T_INIT,
    new_path: ptr::null(),
    file: 0,
    flags: 0,
    mode: 0,
    nbufs: 0,
    bufs: ptr::null_mut(),
    off: 0,
    uid: 0,
    gid: 0,
    atime: 0.,
    mtime: 0.,
    work_req: uv__work {
        work: None,
        done: None,
        loop_0: ptr::null_mut::<uv_loop_s>(),
        wq: uv__queue {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        },
    },
    bufsml: [uv_buf_t {
        base: ptr::null_mut(),
        len: 0,
    }; 4],
};

/// Run one synchronous `uv_fs_*` request and answer whatever `read` makes
/// of it.
///
/// Every `uv_fs_*` call in this family is made with a null loop and a null
/// callback, which is libuv's "do it now, on this thread" spelling. Under
/// that spelling the starter's return value and `request.result` are the
/// same number, and `read` is handed both so each caller keeps testing the
/// one upstream tested. Cleanup runs whatever the answer was — which is the
/// point of writing this once, because `request.ptr` and `request.path` do
/// not survive it and `read` is the last place they can be read.
///
/// This is a *safe* function: the request is one it owns and fully
/// initialises, and `start` and `read` are safe closures, so a caller who
/// puts a raw pointer in one is the one carrying that obligation.
fn fs_request<T>(
    start: impl FnOnce(*mut uv_fs_t) -> c_int,
    read: impl FnOnce(c_int, &uv_fs_t) -> T,
) -> T {
    let mut request: uv_fs_t = UV_FS_T_INIT;
    let result = start(&raw mut request);
    let answer = read(result, &request);
    // SAFETY: `request` is a fully initialised `uv_fs_t` this frame owns,
    // and libuv accepts a cleanup on any request, started or not — an
    // untouched one has a null `path` and `ptr` and the call is a no-op.
    unsafe { uv_fs_req_cleanup(&raw mut request) };
    answer
}

/// [`fs_request`] for the common case: the starter's own answer, with
/// nothing read out of the request.
fn fs_result(start: impl FnOnce(*mut uv_fs_t) -> c_int) -> c_int {
    fs_request(start, |result, _| result)
}

/// [`fs_request`] answering `OK`/`FAIL`, which is what most of this
/// family's callers want.
fn fs_ok(start: impl FnOnce(*mut uv_fs_t) -> c_int) -> c_int {
    if fs_result(start) == LIBUV_SUCCESS {
        OK
    } else {
        FAIL
    }
}

/// The libuv code for the failure `errno` is reporting, clearing `errno`
/// afterwards the way the read/write loops below expect.
fn take_errno() -> c_int {
    // SAFETY: `__errno_location` answers this thread's own `errno` slot,
    // which is always readable and writable.
    unsafe {
        let error = uv_translate_sys_error(*__errno_location());
        *__errno_location() = 0;
        error
    }
}

/// Changes the current directory to `path`.
///
/// Answers 0, or a negative libuv error code.
///
/// # Safety
/// `path` must be a NUL-terminated string.
pub unsafe fn os_chdir(path: *const c_char) -> c_int {
    if p_verbose.get() >= 5 as OptInt {
        // SAFETY: the caller's NUL-terminated path, and `%s` is the one
        // conversion the format string asks for.
        unsafe {
            verbose_enter();
            smsg!(0, "chdir({})", c_str(path));
            verbose_leave();
        }
    }
    // SAFETY: the caller's NUL-terminated path.
    let err = unsafe { uv_chdir(path) };
    if err == 0 {
        // SAFETY: same; `cstr_as_string` borrows it for the length only.
        ui_call_chdir(unsafe { cstr_as_string(path) });
    }
    err
}

/// Reads the name of the current directory into `buf`, which holds `len`
/// bytes. On failure the *error message* is left there instead.
///
/// Answers `Err` on failure.
///
/// # Safety
/// `buf` must address `len` writable bytes.
pub unsafe fn os_dirname(buf: *mut c_char, mut len: size_t) -> Result<(), Failed> {
    // SAFETY: the caller's buffer. libuv reports the answer's length back
    // through `len`, which is what bounds the error message copy.
    unsafe {
        let error_number = uv_cwd(buf, &raw mut len);
        if error_number != LIBUV_SUCCESS {
            xstrlcpy(buf, uv_strerror(error_number), len);
            return Err(Failed);
        }
        Ok(())
    }
}

/// Whether `name` is a directory and *not* a symlink to one.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn os_isrealdir(name: *const c_char) -> bool {
    // `lstat`, not `stat`: a symlink to a directory is not one, though
    // `os_isdir` says it is.
    fs_request(
        // SAFETY: the caller's NUL-terminated path.
        |request| unsafe { uv_fs_lstat(NO_LOOP, request, name, None) },
        |result, request| result == LIBUV_SUCCESS && is_dir(request.statbuf.st_mode),
    )
}

/// Whether `name` exists and is a directory.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn os_isdir(name: *const c_char) -> bool {
    // SAFETY: the caller's NUL-terminated path.
    let mode = unsafe { os_getperm(name) };
    mode >= 0 && is_dir(mode as u64)
}

/// What `name` is: an ordinary file or directory (or nothing at all), a
/// writable device, or something else.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn os_nodetype(name: *const c_char) -> c_int {
    let mut statbuf = UV_STAT_T_INIT;
    // SAFETY: the caller's NUL-terminated path; `statbuf` is this frame's.
    if unsafe { os_stat(name, &raw mut statbuf) } != 0 {
        return NODE_NORMAL; // The file does not exist.
    }
    // Read the mode rather than asking `uv_guess_handle`, which does not
    // distinguish a block device from a directory.
    match statbuf.st_mode & S_IFMT {
        S_IFREG | S_IFDIR => NODE_NORMAL,
        S_IFBLK => NODE_OTHER, // a block device is not writable
        // Everything else is writable: `buf_write` expects NODE_WRITABLE for
        // the character device /dev/stderr.
        _ => NODE_WRITABLE,
    }
}

/// The absolute path of the running executable, into `buffer`, whose size
/// goes in and whose used length comes out through `size`. May fail when
/// procfs is missing (#6734).
///
/// # Safety
/// `buffer` must address `*size` writable bytes.
pub unsafe fn os_exepath(buffer: *mut c_char, size: *mut size_t) -> c_int {
    // SAFETY: the caller's buffer and its length, both non-null.
    unsafe { uv_exepath(buffer, size) }
}

/// Whether `name` is an executable file — found in `$PATH` when `use_path`
/// is set and `name` carries no directory, and otherwise resolved as given.
/// A non-null `abspath` receives the resolved path, allocated.
///
/// # Safety
/// `name` must be a NUL-terminated string, and `abspath` null or writable.
pub unsafe fn os_can_exe(name: *const c_char, abspath: *mut *mut c_char, use_path: bool) -> bool {
    // SAFETY: the caller's contract, passed straight through.
    unsafe {
        let has_dir = gettail_dir(name) != name;
        if !use_path || has_dir {
            // A bare name has to come from `$PATH`: files in the current
            // directory are not executable by name.
            return (use_path || has_dir) && is_executable(name, abspath);
        }
        is_executable_in_path(name, abspath)
    }
}

/// Whether `name` is a regular file the process may execute, storing its
/// resolved absolute path through a non-null `abspath`.
///
/// # Safety
/// `name` must be a NUL-terminated string, and `abspath` null or writable.
unsafe fn is_executable(name: *const c_char, abspath: *mut *mut c_char) -> bool {
    // SAFETY: the caller's NUL-terminated name.
    let mode = unsafe { os_getperm(name) };
    if mode < 0 {
        return false;
    }
    // Only a regular file is worth asking `access(2)` about.
    let ok = mode as u64 & S_IFMT == S_IFREG
        // SAFETY: the caller's NUL-terminated name.
        && fs_result(|req| unsafe { uv_fs_access(NO_LOOP, req, name, X_OK, None) }) == 0;
    if ok && !abspath.is_null() {
        // SAFETY: the caller's out-parameter, checked non-null; it takes
        // ownership of the allocation.
        unsafe { *abspath = save_abs_path(name) };
    }
    ok
}

/// Whether any `$PATH` entry holds an executable called `name`, in the
/// order `$PATH` lists them.
///
/// # Safety
/// `name` must be a NUL-terminated string, and `abspath` null or writable.
unsafe fn is_executable_in_path(name: *const c_char, abspath: *mut *mut c_char) -> bool {
    // SAFETY: `os_getenv` answers an owned NUL-terminated string or null.
    let path_env = unsafe { os_getenv(c"PATH".as_ptr()) };
    if path_env.is_null() {
        return false;
    }
    // SAFETY: non-null, and `os_getenv`'s answer is NUL-terminated. Copied
    // out because `is_executable` below can reach code that reads `$PATH`.
    let path = unsafe { CStr::from_ptr(path_env) }.to_bytes().to_vec();
    // SAFETY: the caller's NUL-terminated name.
    let name_len = unsafe { CStr::from_ptr(name) }.to_bytes().len();
    // Longest case: the whole of `$PATH` as one entry, a separator, `name`
    // and the NUL.
    let mut buf = vec![0u8; name_len + path.len() + 2];
    let mut rv = false;
    // `xstrchrnul` stops at the NUL, so the walk visits every entry
    // including the empty ones and ends with the last.
    for entry in path.split(|&b| b == ENV_SEPCHAR) {
        buf[..entry.len()].copy_from_slice(entry);
        buf[entry.len()] = 0;
        // SAFETY: `buf` is NUL-terminated at `entry.len()` and holds the
        // length passed, which is long enough for any entry plus `name`.
        let _ = unsafe { append_path(buf.as_mut_ptr().cast(), name, buf.len()) };
        // SAFETY: `buf` is NUL-terminated, and `abspath` is the caller's.
        if unsafe { is_executable(buf.as_ptr().cast(), abspath) } {
            rv = true;
            break;
        }
    }
    // SAFETY: `os_getenv` handed ownership of `path_env` over.
    unsafe { xfree(path_env.cast()) };
    rv
}

/// Opens or creates `path`, answering the new descriptor or a negative
/// libuv error code. `mode` is the permission set a newly created file
/// gets, subject to the umask.
///
/// # Safety
/// `path` must be null or a NUL-terminated string.
pub unsafe fn os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    if path.is_null() {
        return UV_EINVAL; // `uv_fs_open` asserts on NULL. #7561
    }
    // SAFETY: the caller's NUL-terminated path.
    fs_result(|req| unsafe { uv_fs_open(NO_LOOP, req, path, flags, mode, None) })
}

/// `fopen(3)`'s mode string as `open(2)` flags, per the table in its
/// manpage. A first byte outside `rwa` has no answer, and upstream aborts.
///
/// The second byte is `b` (which changes nothing on any platform nvim
/// builds for), absent, or `+`; upstream asserts the last case rather than
/// testing it, so anything else is read as `+` here too.
fn fopen_flags(mode: &[u8]) -> Option<c_int> {
    let update = !matches!(mode.first_chunk::<2>().map(|m| m[1]), None | Some(b'b'));
    debug_assert!(!update || mode.get(1) == Some(&b'+'));
    Some(match (mode.first()?, update) {
        (b'r', false) => O_RDONLY,
        (b'w', false) => O_WRONLY | O_CREAT | O_TRUNC,
        (b'a', false) => O_WRONLY | O_CREAT | O_APPEND,
        (b'r', true) => O_RDWR,
        (b'w', true) => O_RDWR | O_CREAT | O_TRUNC,
        (b'a', true) => O_RDWR | O_CREAT | O_APPEND,
        _ => return None,
    })
}
/// `fopen(3)` over [`os_open`], so that every open in the tree goes through
/// one place. Answers null when the file cannot be opened.
///
/// # Safety
/// `path` must be null or NUL-terminated, and `flags` a NUL-terminated
/// `fopen` mode string.
pub unsafe fn os_fopen(path: *const c_char, flags: *const c_char) -> *mut FILE {
    debug_assert!(!flags.is_null());
    // SAFETY: the caller's NUL-terminated mode string.
    let mode = unsafe { CStr::from_ptr(flags) }.to_bytes();
    debug_assert!(!mode.is_empty() && mode.len() <= 2);
    let Some(iflags) = fopen_flags(mode) else {
        // Every call site in the tree passes a literal mode, so a mode with
        // no `open(2)` spelling is a bug in the caller rather than anything
        // the user did — upstream aborts, and so does this.
        // SAFETY: `abort` is libc's, and does not return.
        unsafe { abort() }
    };
    // SAFETY: the caller's path; `fd` is this frame's until `fdopen` adopts
    // it, and `flags` is the mode string it was just parsed from.
    unsafe {
        let fd = os_open(path, iflags, 0o666);
        if fd < 0 {
            return ptr::null_mut();
        }
        fdopen(fd, flags)
    }
}

/// Marks `fd` close-on-exec. Answers 0, or -1 with `errno` set.
///
/// # Safety
/// `fd` must be a descriptor this process owns.
pub unsafe fn os_set_cloexec(fd: c_int) -> c_int {
    // SAFETY: `fcntl` on the caller's descriptor, and `errno` is this
    // thread's. The line numbers are the sites in `v0.12.4:os/fs.c`.
    unsafe {
        let fdflags = fcntl(fd, F_GETFD);
        if fdflags < 0 {
            let e = *__errno_location();
            logmsg_c!(
                LOGLVL_ERR,
                ptr::null::<c_char>(),
                c"os_set_cloexec".as_ptr(),
                497,
                true,
                c"Failed to get flags on descriptor %d: %s".as_ptr(),
                fd,
                strerror(e),
            );
            *__errno_location() = e;
            return -1;
        }
        if fdflags & FD_CLOEXEC == 0 && fcntl(fd, F_SETFD, fdflags | FD_CLOEXEC) == -1 {
            let e = *__errno_location();
            logmsg_c!(
                LOGLVL_ERR,
                ptr::null::<c_char>(),
                c"os_set_cloexec".as_ptr(),
                504,
                true,
                c"Failed to set CLOEXEC on descriptor %d: %s".as_ptr(),
                fd,
                strerror(e),
            );
            *__errno_location() = e;
            return -1;
        }
        0
    }
}

/// Closes `fd`. Answers 0 or a libuv error code.
///
/// # Safety
/// `fd` must be a descriptor this process owns.
pub unsafe fn os_close(fd: c_int) -> c_int {
    // SAFETY: the caller's descriptor.
    fs_result(|req| unsafe { uv_fs_close(NO_LOOP, req, fd as uv_file, None) })
}

/// Duplicates `fd`, retrying through `EINTR`. Answers the new descriptor or
/// a negative libuv error code.
///
/// # Safety
/// `fd` must be a descriptor this process owns.
pub unsafe fn os_dup(fd: c_int) -> c_int {
    loop {
        // SAFETY: `dup` on the caller's descriptor.
        let ret = unsafe { dup(fd) };
        if ret >= 0 {
            return ret;
        }
        let error = take_errno();
        if error != UV_EINTR {
            return error;
        }
    }
}

/// The descriptor to read stdin from: the one `--` handed us if there is
/// one, and otherwise a duplicate of the real stdin.
pub unsafe fn os_open_stdin_fd() -> c_int {
    if stdin_fd.get() > 0 {
        stdin_fd.get()
    } else {
        // SAFETY: `STDIN_FILENO` is always a descriptor of this process.
        unsafe { os_dup(STDIN_FILENO) }
    }
}

/// Reads up to `size` bytes from `fd` into `ret_buf`, handling short reads
/// and `EINTR` but no other error. `ret_eof` is set to whether the read
/// stopped at end of file.
///
/// Answers the number of bytes read, or a libuv error code (< 0).
///
/// # Safety
/// `ret_eof` must be writable, and `ret_buf` must address `size` writable
/// bytes or be null with `size` zero.
pub unsafe fn os_read(
    fd: c_int,
    ret_eof: *mut bool,
    ret_buf: *mut c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    // SAFETY: the caller's out-parameter.
    unsafe { *ret_eof = false };
    if ret_buf.is_null() {
        debug_assert!(size == 0);
        return 0;
    }
    // SAFETY: the caller promises `size` writable bytes at `ret_buf`, and
    // nothing below aliases them.
    let buf = unsafe { slice::from_raw_parts_mut(ret_buf.cast::<u8>(), size) };
    let mut read_bytes: size_t = 0;
    while read_bytes != size {
        let rest = &mut buf[read_bytes..];
        // SAFETY: `rest` is a live writable slice of exactly its own length.
        let cur = unsafe { read(fd, rest.as_mut_ptr().cast::<c_void>(), rest.len()) };
        if cur > 0 {
            read_bytes += cur as size_t;
        } else if cur == 0 {
            // SAFETY: the caller's out-parameter.
            unsafe { *ret_eof = true };
            break;
        } else {
            let error = take_errno();
            if non_blocking && error == UV_EAGAIN {
                break;
            }
            if error != UV_EINTR && error != UV_EAGAIN {
                return error as ptrdiff_t;
            }
        }
    }
    read_bytes as ptrdiff_t
}

/// [`os_read`] over a scatter list: reads into `iov`'s buffers in order,
/// advancing the entries it fills as it goes.
///
/// Answers the number of bytes read, or a libuv error code (< 0).
///
/// # Safety
/// `ret_eof` must be writable and `iov` must address `iov_size` live
/// `iovec`s whose buffers are writable for their stated lengths.
pub unsafe fn os_readv(
    fd: c_int,
    ret_eof: *mut bool,
    iov: *mut iovec,
    iov_size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    // SAFETY: the caller's out-parameter.
    unsafe { *ret_eof = false };
    // SAFETY: the caller promises `iov_size` live entries at `iov`.
    let vecs = unsafe { slice::from_raw_parts_mut(iov, iov_size) };
    let mut toread: size_t = 0;
    for v in vecs.iter() {
        debug_assert!(toread <= size_t::MAX - v.iov_len);
        toread += v.iov_len;
    }
    // Index of the first entry not yet filled; the ones before it are done.
    let mut first = 0;
    let mut read_bytes: size_t = 0;
    let mut eof = false;
    while read_bytes < toread && first < vecs.len() && !eof {
        let pending = &mut vecs[first..];
        // SAFETY: `pending` is the caller's array, still live, and `readv`
        // only writes into the buffers its entries name.
        let cur = unsafe { readv(fd, pending.as_ptr(), pending.len() as c_int) };
        if cur == 0 {
            eof = true;
            // SAFETY: the caller's out-parameter.
            unsafe { *ret_eof = true };
        }
        if cur > 0 {
            read_bytes += cur as size_t;
            // Retire the entries `readv` filled and shorten the one it
            // stopped inside.
            let mut left = cur as size_t;
            while first < vecs.len() && left != 0 {
                let head = &mut vecs[first];
                if left < head.iov_len {
                    head.iov_len -= left;
                    // SAFETY: `left` bytes of this entry's buffer were just
                    // written, so the offset is inside it.
                    head.iov_base = unsafe { head.iov_base.cast::<u8>().add(left) }.cast();
                    left = 0;
                } else {
                    left -= head.iov_len;
                    first += 1;
                }
            }
        } else if cur < 0 {
            let error = take_errno();
            if non_blocking && error == UV_EAGAIN {
                break;
            }
            if error != UV_EINTR && error != UV_EAGAIN {
                return error as ptrdiff_t;
            }
        }
    }
    read_bytes as ptrdiff_t
}

/// Writes `size` bytes from `buf` to `fd`, handling short writes and
/// `EINTR` but no other error.
///
/// Answers the number of bytes written, or a libuv error code (< 0).
///
/// # Safety
/// `buf` must address `size` readable bytes, or be null with `size` zero.
pub unsafe fn os_write(
    fd: c_int,
    buf: *const c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    if buf.is_null() {
        debug_assert!(size == 0);
        return 0;
    }
    // SAFETY: the caller promises `size` readable bytes at `buf`.
    let bytes = unsafe { slice::from_raw_parts(buf.cast::<u8>(), size) };
    let mut written_bytes: size_t = 0;
    while written_bytes != size {
        let rest = &bytes[written_bytes..];
        // SAFETY: `rest` is a live slice of exactly its own length.
        let cur = unsafe { write(fd, rest.as_ptr().cast::<c_void>(), rest.len()) };
        if cur > 0 {
            written_bytes += cur as size_t;
        } else if cur == 0 {
            // A zero-length write of a non-empty buffer has no explanation.
            return UV_UNKNOWN as ptrdiff_t;
        } else {
            let error = take_errno();
            if non_blocking && error == UV_EAGAIN {
                break;
            }
            if error != UV_EINTR && error != UV_EAGAIN {
                return error as ptrdiff_t;
            }
        }
    }
    written_bytes as ptrdiff_t
}

/// Copies `path` onto `new_path` with libuv's `copyfile` flags.
///
/// # Safety
/// Both paths must be NUL-terminated strings.
pub unsafe fn os_copy(path: *const c_char, new_path: *const c_char, flags: c_int) -> c_int {
    // SAFETY: the caller's NUL-terminated paths.
    fs_result(|req| unsafe { uv_fs_copyfile(NO_LOOP, req, path, new_path, flags, None) })
}

/// Flushes `fd` to disk, counting the call in `nvim__stats()`.
///
/// # Safety
/// `fd` must be a descriptor this process owns.
pub unsafe fn os_fsync(fd: c_int) -> c_int {
    // SAFETY: the caller's descriptor.
    let r = fs_result(|req| unsafe { uv_fs_fsync(NO_LOOP, req, fd as uv_file, None) });
    g_stats.with_mut(|stats| stats.fsync += 1);
    r
}

/// Resolves `name` to a real path, into `buf` if that is non-null and into
/// a fresh `len`-byte allocation otherwise. Answers null on failure.
///
/// # Safety
/// `buf` must be null or address `len` writable bytes, and `name` must be a
/// NUL-terminated string.
pub unsafe fn os_realpath(name: *const c_char, mut buf: *mut c_char, len: size_t) -> *mut c_char {
    // `request.ptr` is the resolved path and `uv_fs_req_cleanup` frees
    // it, so the copy has to happen inside the read.
    fs_request(
        // SAFETY: the caller's NUL-terminated name.
        |request| unsafe { uv_fs_realpath(NO_LOOP, request, name, None) },
        |result, request| {
            if result != LIBUV_SUCCESS {
                return ptr::null_mut();
            }
            if buf.is_null() {
                // SAFETY: `len` is what the caller asked the answer to fit.
                buf = unsafe { xmalloc(len) }.cast();
            }
            // SAFETY: `request.ptr` is libuv's NUL-terminated answer, alive
            // until cleanup, and `buf` holds `len` bytes.
            unsafe { xstrlcpy(buf, request.ptr.cast(), len) };
            buf
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fopen_flags_matches_the_fopen_table() {
        assert_eq!(fopen_flags(b"r"), Some(O_RDONLY));
        assert_eq!(fopen_flags(b"rb"), Some(O_RDONLY));
        assert_eq!(fopen_flags(b"w"), Some(O_WRONLY | O_CREAT | O_TRUNC));
        assert_eq!(fopen_flags(b"a"), Some(O_WRONLY | O_CREAT | O_APPEND));
        assert_eq!(fopen_flags(b"r+"), Some(O_RDWR));
        assert_eq!(fopen_flags(b"w+"), Some(O_RDWR | O_CREAT | O_TRUNC));
        assert_eq!(fopen_flags(b"a+"), Some(O_RDWR | O_CREAT | O_APPEND));
        // Neither an empty mode nor an unknown first byte has an answer.
        assert_eq!(fopen_flags(b""), None);
        assert_eq!(fopen_flags(b"x"), None);
    }

    #[test]
    fn is_dir_reads_only_the_type_field() {
        assert!(is_dir(S_IFDIR | 0o755));
        assert!(!is_dir(S_IFREG | 0o755));
        assert!(!is_dir(0o40000 - 1));
    }
}
