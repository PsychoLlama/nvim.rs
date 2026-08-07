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
//! This family is still on `allow(unsafe_op_in_unsafe_fn)`, and
//! deliberately so. **57 of its 60 ratcheted units are the export
//! declarations themselves**, which the deny discounts and an equal number
//! of blanket-wrapped bodies then puts straight back: measured both ways
//! round, 60 before and 60 after, at a cost of 113 lines and not one
//! narrower obligation. Adopting the deny here means rewriting the bodies,
//! which is Cargo.toml's own rule and a slice of its own.

#![allow(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_cwd, uv_exepath, uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close,
    uv_fs_copyfile, uv_fs_fchown, uv_fs_fstat, uv_fs_fsync, uv_fs_lstat, uv_fs_mkdir,
    uv_fs_mkdtemp, uv_fs_open, uv_fs_realpath, uv_fs_rename, uv_fs_req_cleanup, uv_fs_rmdir,
    uv_fs_scandir, uv_fs_scandir_next, uv_fs_stat, uv_fs_unlink, uv_fs_utime, uv_strerror,
    uv_translate_sys_error,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::main::{e_mkdir, e_noname, g_stats, p_verbose, stdin_fd};
use crate::src::nvim::memory::{
    memcnt, xfree, xmalloc, xmemcpyz, xmemdupz, xstrchrnul, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{emsg, semsg, smsg, verbose_enter, verbose_leave};
use crate::src::nvim::os::env::os_getenv;
use crate::src::nvim::os::libc::{
    __errno_location, abort, dup, fcntl, fdopen, gettext, getuid, getxattr, listxattr, memset,
    read, readv, setxattr, strerror, strlen, write,
};
use crate::src::nvim::path::{
    FullName_save, append_path, dir_of_file_exists, get_past_head, gettail_dir, path_tail_with_sep,
    save_abs_path, vim_ispathsep,
};
use crate::src::nvim::types::libc::STDIN_FILENO;
use crate::src::nvim::types::{
    Directory, FILE, FileID, FileInfo, OptInt, int32_t, iovec, ptrdiff_t, size_t, ssize_t, uid_t,
    uint64_t, uv__queue, uv__work, uv_buf_t, uv_file, uv_fs_t, uv_fs_type, uv_gid_t, uv_loop_s,
    uv_loop_t, uv_req_type, uv_stat_t, uv_timespec_t, uv_uid_t, vim_acl_T,
};
use crate::src::nvim::ui::ui_call_chdir;
pub mod dir;
pub mod meta;

pub use dir::*;
pub use meta::*;

pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_EMLINK: C2Rust_Unnamed_5 = -31;
pub const UV_EOF: C2Rust_Unnamed_5 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_5 = -4094;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_ELOOP: C2Rust_Unnamed_5 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_5 = -21;
pub const UV_EINVAL: C2Rust_Unnamed_5 = -22;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
pub const UV_EEXIST: C2Rust_Unnamed_5 = -17;
pub const UV_EBADF: C2Rust_Unnamed_5 = -9;
pub const UV_EAGAIN: C2Rust_Unnamed_5 = -11;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
/// `access(2)`'s mode bits, which `uv_fs_access` takes as they are.
const R_OK: ::core::ffi::c_int = 4;
const W_OK: ::core::ffi::c_int = 2;
const X_OK: ::core::ffi::c_int = 1;
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const F_GETFD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const F_SETFD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FD_CLOEXEC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_NORMAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
/// The three `E15xx` messages `os_copy_xattr` reports. Read-only text, so
/// `CStr` constants rather than the mutable `[c_char; N]` statics c2rust
/// transmuted the C string literals into.
const E_XATTR_ERANGE: &::core::ffi::CStr = c"E1506: Buffer too small to copy xattr value or key";
const E_XATTR_E2BIG: &::core::ffi::CStr =
    c"E1508: Size of the extended attribute value is larger than the maximum size allowed";
const E_XATTR_OTHER: &::core::ffi::CStr =
    c"E1509: Error occurred when reading or writing extended attribute";
static kLibuvSuccess: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
/// `uv_fs_t req = { 0 }`. c2rust wrote all sixty-six fields out at each of
/// the twenty-one sites that need one -- 1,386 of this file's lines -- and
/// every one of them is zero, `UV_UNKNOWN_REQ` and `UV_FS_CUSTOM` included.
const UV_FS_T_INIT: uv_fs_t = uv_fs_t {
    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    type_0: UV_UNKNOWN_REQ,
    reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
    fs_type: UV_FS_CUSTOM,
    loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
    cb: None,
    result: 0,
    ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    path: ::core::ptr::null::<::core::ffi::c_char>(),
    statbuf: uv_stat_t {
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
    },
    new_path: ::core::ptr::null::<::core::ffi::c_char>(),
    file: 0,
    flags: 0,
    mode: 0,
    nbufs: 0,
    bufs: ::core::ptr::null_mut::<uv_buf_t>(),
    off: 0,
    uid: 0,
    gid: 0,
    atime: 0.,
    mtime: 0.,
    work_req: uv__work {
        work: None,
        done: None,
        loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
        wq: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
    },
    bufsml: [uv_buf_t {
        base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
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
    start: impl FnOnce(*mut uv_fs_t) -> ::core::ffi::c_int,
    read: impl FnOnce(::core::ffi::c_int, &uv_fs_t) -> T,
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
fn fs_result(start: impl FnOnce(*mut uv_fs_t) -> ::core::ffi::c_int) -> ::core::ffi::c_int {
    fs_request(start, |result, _| result)
}

/// [`fs_request`] answering `OK`/`FAIL`, which is what most of this
/// family's callers want.
fn fs_ok(start: impl FnOnce(*mut uv_fs_t) -> ::core::ffi::c_int) -> ::core::ffi::c_int {
    if fs_result(start) == kLibuvSuccess.get() {
        OK
    } else {
        FAIL
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_chdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if p_verbose.get() >= 5 as OptInt {
        verbose_enter();
        smsg(0 as ::core::ffi::c_int, c"chdir(%s)".as_ptr(), path);
        verbose_leave();
    }
    let mut err: ::core::ffi::c_int = uv_chdir(path);
    if err == 0 as ::core::ffi::c_int {
        ui_call_chdir(cstr_as_string(path));
    }
    return err;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_dirname(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut error_number: ::core::ffi::c_int = 0;
    error_number = uv_cwd(buf, &raw mut len);
    if error_number != kLibuvSuccess.get() {
        xstrlcpy(buf, uv_strerror(error_number), len);
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn os_isrealdir(mut name: *const ::core::ffi::c_char) -> bool {
    // A symlink to a directory is not one; `os_isdir` says it is.
    fs_request(
        |request| uv_fs_lstat(::core::ptr::null_mut::<uv_loop_t>(), request, name, None),
        |result, request| {
            let mode = request.statbuf.st_mode & __S_IFMT as uint64_t;
            result == kLibuvSuccess.get() && mode == 0o40000 as uint64_t
        },
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_isdir(mut name: *const ::core::ffi::c_char) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    return mode & __S_IFMT as int32_t == 0o40000 as int32_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_nodetype(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut statbuf: uv_stat_t = uv_stat_t {
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
    if 0 as ::core::ffi::c_int != os_stat(name, &raw mut statbuf) {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t
        || statbuf.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t
    {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
        return NODE_OTHER;
    }
    return NODE_WRITABLE;
}
pub unsafe extern "C" fn os_exepath(
    mut buffer: *mut ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    return uv_exepath(buffer, size);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_can_exe(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
    mut use_path: bool,
) -> bool {
    if !use_path || gettail_dir(name) != name {
        return (use_path as ::core::ffi::c_int != 0 || gettail_dir(name) != name)
            && is_executable(name, abspath) as ::core::ffi::c_int != 0;
    }
    return is_executable_in_path(name, abspath);
}
unsafe extern "C" fn is_executable(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    let mut r: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if mode & __S_IFMT as int32_t == 0o100000 as int32_t {
        r = fs_result(|req| {
            uv_fs_access(::core::ptr::null_mut::<uv_loop_t>(), req, name, X_OK, None)
        });
    }
    let ok: bool = r == 0 as ::core::ffi::c_int;
    if ok as ::core::ffi::c_int != 0 && !abspath.is_null() {
        *abspath = save_abs_path(name);
    }
    return ok;
}
unsafe extern "C" fn is_executable_in_path(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut path_env: *mut ::core::ffi::c_char = os_getenv(c"PATH".as_ptr());
    if path_env.is_null() {
        return false_0 != 0;
    }
    let mut path: *mut ::core::ffi::c_char = xstrdup(path_env);
    let bufsize: size_t = strlen(name)
        .wrapping_add(strlen(path))
        .wrapping_add(2 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = path;
    let mut rv: bool = false_0 != 0;
    loop {
        let mut e: *mut ::core::ffi::c_char = xstrchrnul(p, ENV_SEPCHAR as ::core::ffi::c_char);
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            e.offset_from(p) as size_t,
        );
        append_path(buf, name, bufsize);
        if is_executable(buf, abspath) {
            rv = true_0 != 0;
            break;
        } else {
            if *e as ::core::ffi::c_int != ENV_SEPCHAR {
                break;
            }
            p = e.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(path as *mut ::core::ffi::c_void);
    xfree(path_env as *mut ::core::ffi::c_void);
    return rv;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_open(
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if path.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    fs_result(|req| {
        uv_fs_open(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            flags,
            mode,
            None,
        )
    })
}
pub unsafe extern "C" fn os_fopen(
    mut path: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut FILE {
    debug_assert!(!flags.is_null() && strlen(flags) > 0 as size_t && strlen(flags) <= 2 as size_t);
    let mut iflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'b' as ::core::ffi::c_int
    {
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDONLY;
            }
            119 => {
                iflags = O_WRONLY | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_WRONLY | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    } else {
        debug_assert!(
            *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '+' as ::core::ffi::c_int
        );
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDWR;
            }
            119 => {
                iflags = O_RDWR | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_RDWR | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    }
    let mut fd: ::core::ffi::c_int = os_open(path, iflags, 0o666 as ::core::ffi::c_int);
    if fd < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<FILE>();
    }
    return fdopen(fd, flags);
}
pub unsafe extern "C" fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut e: ::core::ffi::c_int = 0;
    let mut fdflags: ::core::ffi::c_int = fcntl(fd, F_GETFD);
    if fdflags < 0 as ::core::ffi::c_int {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            c"os_set_cloexec".as_ptr(),
            497 as ::core::ffi::c_int,
            true_0 != 0,
            c"Failed to get flags on descriptor %d: %s".as_ptr(),
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    if fdflags & FD_CLOEXEC == 0 as ::core::ffi::c_int
        && fcntl(fd, F_SETFD, fdflags | FD_CLOEXEC) == -1 as ::core::ffi::c_int
    {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            c"os_set_cloexec".as_ptr(),
            504 as ::core::ffi::c_int,
            true_0 != 0,
            c"Failed to set CLOEXEC on descriptor %d: %s".as_ptr(),
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_close(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_close(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            fd as uv_file,
            None,
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_dup(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    loop {
        ret = dup(fd);
        if ret < 0 as ::core::ffi::c_int {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if error == UV_EINTR as ::core::ffi::c_int {
                continue;
            }
            return error;
        } else {
            return ret;
        }
    }
}
pub unsafe extern "C" fn os_open_stdin_fd() -> ::core::ffi::c_int {
    let mut stdin_dup_fd: ::core::ffi::c_int = 0;
    if stdin_fd.get() > 0 as ::core::ffi::c_int {
        stdin_dup_fd = stdin_fd.get();
    } else {
        stdin_dup_fd = os_dup(STDIN_FILENO);
    }
    return stdin_dup_fd;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_read(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    ret_buf: *mut ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    if ret_buf.is_null() {
        debug_assert!(size == 0 as size_t);
        return 0 as ptrdiff_t;
    }
    let mut read_bytes: size_t = 0 as size_t;
    while read_bytes != size {
        debug_assert!(size >= read_bytes);
        let cur_read_bytes: ptrdiff_t = read(
            fd,
            ret_buf.add(read_bytes) as *mut ::core::ffi::c_void,
            size.wrapping_sub(read_bytes),
        ) as ptrdiff_t;
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
        }
        if cur_read_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else {
            if cur_read_bytes != 0 as ptrdiff_t {
                continue;
            }
            *ret_eof = true_0 != 0;
            break;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_readv(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    mut iov: *mut iovec,
    mut iov_size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    let mut read_bytes: size_t = 0 as size_t;
    let mut toread: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < iov_size {
        debug_assert!(
            toread <= (18446744073709551615 as size_t).wrapping_sub((*iov.add(i)).iov_len)
        );
        toread = toread.wrapping_add((*iov.add(i)).iov_len);
        i = i.wrapping_add(1);
    }
    while read_bytes < toread && iov_size != 0 && !*ret_eof {
        let mut cur_read_bytes: ptrdiff_t =
            readv(fd, iov, iov_size as ::core::ffi::c_int) as ptrdiff_t;
        if cur_read_bytes == 0 as ptrdiff_t {
            *ret_eof = true_0 != 0;
        }
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
            while iov_size != 0 && cur_read_bytes != 0 {
                if cur_read_bytes < (*iov).iov_len as ptrdiff_t {
                    (*iov).iov_len = (*iov).iov_len.wrapping_sub(cur_read_bytes as size_t);
                    (*iov).iov_base = ((*iov).iov_base as *mut ::core::ffi::c_char)
                        .offset(cur_read_bytes)
                        as *mut ::core::ffi::c_void;
                    cur_read_bytes = 0 as ptrdiff_t;
                } else {
                    cur_read_bytes -= (*iov).iov_len as ptrdiff_t;
                    iov_size = iov_size.wrapping_sub(1);
                    iov = iov.offset(1);
                }
            }
        } else {
            if cur_read_bytes >= 0 as ptrdiff_t {
                continue;
            }
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_write(
    fd: ::core::ffi::c_int,
    buf: *const ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    if buf.is_null() {
        debug_assert!(size == 0 as size_t);
        return 0 as ptrdiff_t;
    }
    let mut written_bytes: size_t = 0 as size_t;
    while written_bytes != size {
        debug_assert!(size >= written_bytes);
        let cur_written_bytes: ptrdiff_t = write(
            fd,
            buf.add(written_bytes) as *const ::core::ffi::c_void,
            size.wrapping_sub(written_bytes),
        ) as ptrdiff_t;
        if cur_written_bytes > 0 as ptrdiff_t {
            written_bytes = written_bytes.wrapping_add(cur_written_bytes as size_t);
        }
        if cur_written_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else if cur_written_bytes == 0 as ptrdiff_t {
            return UV_UNKNOWN as ::core::ffi::c_int as ptrdiff_t;
        }
    }
    return written_bytes as ptrdiff_t;
}
pub unsafe extern "C" fn os_copy(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_copyfile(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            new_path,
            flags,
            None,
        )
    })
}
pub unsafe extern "C" fn os_fsync(mut fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let r = fs_result(|req| {
        uv_fs_fsync(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            fd as uv_file,
            None,
        )
    });
    (*g_stats.ptr()).fsync += 1;
    r
}
pub unsafe extern "C" fn os_realpath(
    mut name: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    // `request.ptr` is the resolved path and `uv_fs_req_cleanup` frees
    // it, so the copy has to happen inside the read.
    fs_request(
        |request| uv_fs_realpath(::core::ptr::null_mut::<uv_loop_t>(), request, name, None),
        |result, request| {
            if result != kLibuvSuccess.get() {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            if buf.is_null() {
                buf = xmalloc(len) as *mut ::core::ffi::c_char;
            }
            xstrlcpy(buf, request.ptr as *const ::core::ffi::c_char, len);
            buf
        },
    )
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENOTSUP: ::core::ffi::c_int = EOPNOTSUPP;
pub const EOPNOTSUPP: ::core::ffi::c_int = 95;
pub const EPERM: ::core::ffi::c_int = 1;
pub const E2BIG: ::core::ffi::c_int = 7;
pub const EACCES: ::core::ffi::c_int = 13;
pub const ERANGE: ::core::ffi::c_int = 34;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TEMP_FILE_PATH_MAXLEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
