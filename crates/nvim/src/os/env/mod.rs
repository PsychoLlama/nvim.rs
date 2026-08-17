//! Environment variables, and the home directory.
//!
//! # Boundary
//!
//! Every read and write of the process environment goes through libuv
//! (`uv_os_getenv`/`uv_os_setenv`/`uv_os_unsetenv`), which is what makes it
//! work the same on Windows; enumerating it does not, so that reads `environ`
//! directly. `uname` supplies the hostname.
//!
//! - This file: getting, setting and enumerating variables, plus the resolved
//!   home directory.
//! - [`expand`]: `$VAR` and `~` inside a path.
//! - [`dirs`]: `$VIM`/`$VIMRUNTIME` and the `~` shorthand going back out.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod dirs;
pub mod expand;

pub use dirs::{
    home_replace, home_replace_save, vim_env_iter, vim_env_iter_rev, vim_get_prefix_from_exepath,
    vim_getenv,
};
pub use expand::{expand_env, expand_env_esc, expand_env_save, expand_env_save_opt};

use crate::charset::skipwhite;
use crate::event::libuv::{
    uv_err_name, uv_os_getenv, uv_os_homedir, uv_os_setenv, uv_os_unsetenv, uv_strerror,
};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::main::{IObuff, NameBuff, didset_vim, didset_vimruntime, nvim_testing, os_buf};
use crate::memory::{xfree, xmalloc, xmemcpyz, xmemdupz, xstrdup, xstrlcat, xstrlcpy};
use crate::message::internal_error;
use crate::os::fs::{os_dirname, os_realpath};
use crate::os::libc::{
    environ, getpid, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp, strpbrk,
};
use crate::os::uv_error::{UV_EINVAL, UV_ENOBUFS, UV_ENOENT, UV_UNKNOWN};
use crate::path::{path_is_absolute, path_tail, path_tail_with_sep, vim_ispathsep};
use crate::strings::striequal;
use crate::types::{expand_T, int64_t, size_t};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

unsafe extern "C" {
    fn uname(__name: *mut utsname) -> c_int;
}

/// `<sys/utsname.h>`'s answer, of which only `nodename` is read.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utsname {
    pub sysname: [c_char; 65],
    pub nodename: [c_char; 65],
    pub release: [c_char; 65],
    pub version: [c_char; 65],
    pub machine: [c_char; 65],
    pub domainname: [c_char; 65],
}

// The libuv error codes this file distinguishes, retyped from the `c_int`
// anonymous enum c2rust emitted.

const MAXPATHL: usize = 4096;
const IOSIZE: usize = 1024 + 1;
const OK: c_int = 1;
/// How much of a name `get_env_name` copies into `xp_buf`.
const EXPAND_BUF_LEN: size_t = 256;
/// `$PATH`'s separator, and it as a string.
const ENV_SEPCHAR: c_char = b':' as c_char;
const ENV_SEPSTR: &CStr = c":";
/// No prescribed maximum for `$PATH` on Unix.
const MAX_ENVPATHLEN: usize = c_int::MAX as usize;

pub static default_vim_dir: GlobalCell<*mut c_char> =
    GlobalCell::new(c"/usr/local/share/nvim".as_ptr().cast_mut());
pub static default_vimruntime_dir: GlobalCell<*mut c_char> = GlobalCell::new(
    concat!(env!("NVIM_DEFAULT_VIMRUNTIME_DIR"), "\0").as_ptr() as *const c_char as *mut c_char,
);
pub static default_lib_dir: GlobalCell<*mut c_char> = GlobalCell::new(
    concat!(env!("NVIM_DEFAULT_LIB_DIR"), "\0").as_ptr() as *const c_char as *mut c_char,
);

pub fn env_init() {
    // SAFETY: a static name.
    nvim_testing.set(unsafe { os_env_exists(c"NVIM_TEST".as_ptr(), false) });
}

/// `ELOG("uv_os_*(%s) failed: %d %s")`. Which failures are worth reporting
/// differs per caller — `os_getenv` ignores "no such variable" and "no idea",
/// `os_env_exists` ignores "your buffer is too small" because that is its
/// success — so the guard stays at the call site and only the message is
/// shared.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn log_uv_failure(func: &CStr, fmt: &CStr, name: *const c_char, r: c_int) {
    // SAFETY: the caller's contract; `logmsg` is printf-shaped, and
    // `uv_err_name` answers a static string.
    unsafe {
        logmsg_c!(
            LOGLVL_ERR,
            ptr::null(),
            func.as_ptr(),
            0,
            true,
            fmt.as_ptr(),
            name,
            r,
            uv_err_name(r),
        );
    }
}

/// The three messages, spelled as upstream spells them.
const GETENV_FAILED: &CStr = c"uv_os_getenv(%s) failed: %d %s";
const SETENV_FAILED: &CStr = c"uv_os_setenv(%s) failed: %d %s";
const UNSETENV_FAILED: &CStr = c"uv_os_unsetenv(%s) failed: %d %s";

/// `getenv()`, but NULL for an empty value. The result is newly allocated.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv(name: *const c_char) -> *mut c_char {
    /// Big enough that most variables never need the second call.
    const INIT_SIZE: usize = 64;
    // SAFETY: the caller's contract; `size` is `buf`'s length going in and
    // the value's length coming out, which is what the retry is sized from.
    unsafe {
        if *name == 0 {
            return ptr::null_mut();
        }
        let mut e: *mut c_char = ptr::null_mut();
        let mut size: size_t = INIT_SIZE;
        let mut buf: [c_char; INIT_SIZE] = [0; INIT_SIZE];
        let mut r = uv_os_getenv(name, buf.as_mut_ptr(), &raw mut size);
        if r == UV_ENOBUFS {
            e = xmalloc(size) as *mut c_char;
            r = uv_os_getenv(name, e, &raw mut size);
            if r != 0 || size == 0 || *e == 0 {
                xfree(e.cast());
                e = ptr::null_mut();
            }
        } else if r != 0 || size == 0 || buf[0] == 0 {
            e = ptr::null_mut();
        } else {
            // NB: `size` includes the NUL terminator, except when it does not.
            e = xmemdupz(buf.as_ptr().cast(), size) as *mut c_char;
        }
        if r != 0 && r != UV_ENOENT && r != UV_UNKNOWN {
            log_uv_failure(c"os_getenv", GETENV_FAILED, name, r);
        }
        e
    }
}

/// `getenv()` into `buf` rather than allocating; truncated if it does not fit.
///
/// Answers `buf`, or NULL when the variable is unset or empty.
///
/// # Safety
/// `name` must be NUL-terminated and `buf` writable for `bufsize` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv_buf(
    name: *const c_char,
    buf: *mut c_char,
    bufsize: size_t,
) -> *mut c_char {
    // SAFETY: the caller's contract; the retry buffer is sized from what
    // libuv reported and the copy back into `buf` is bounded by `bufsize`.
    unsafe {
        if *name == 0 {
            return ptr::null_mut();
        }
        let mut size = bufsize;
        let mut r = uv_os_getenv(name, buf, &raw mut size);
        if r == UV_ENOBUFS {
            let e = xmalloc(size) as *mut c_char;
            r = uv_os_getenv(name, e, &raw mut size);
            if r == 0 && size != 0 && *e != 0 {
                xmemcpyz(buf.cast(), e.cast(), bufsize.min(size) - 1);
            }
            xfree(e.cast());
        }
        if r != 0 || size == 0 || *buf == 0 {
            if r != 0 && r != UV_ENOENT && r != UV_UNKNOWN {
                log_uv_failure(c"os_getenv_buf", GETENV_FAILED, name, r);
            }
            return ptr::null_mut();
        }
        buf
    }
}

/// [`os_getenv_buf`] into `NameBuff`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenv_noalloc(name: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract; `NameBuff` is `MAXPATHL` bytes.
    unsafe { os_getenv_buf(name, NameBuff.ptr().cast(), MAXPATHL) }
}

/// Whether environment variable `name` is defined, empty or not.
///
/// `nonempty` treats an empty value as "does not exist".
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_env_exists(name: *const c_char, nonempty: bool) -> bool {
    // SAFETY: the caller's contract. A two-byte buffer is deliberate: the
    // value does not matter, and `UV_ENOBUFS` already means "found".
    unsafe {
        if *name == 0 {
            return false;
        }
        let mut buf: [c_char; 2] = [0; 2];
        let mut size: size_t = buf.len();
        let r = uv_os_getenv(name, buf.as_mut_ptr(), &raw mut size);
        debug_assert!(r != UV_EINVAL);
        if r != 0 && r != UV_ENOENT && r != UV_ENOBUFS {
            log_uv_failure(c"os_env_exists", GETENV_FAILED, name, r);
        }
        (r == 0 && (!nonempty || size > 0)) || r == UV_ENOBUFS
    }
}

/// Set an environment variable. `overwrite` of 0 leaves an existing one
/// alone. Answers 0 or -1.
///
/// # Safety
/// `name` and `value` must be NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setenv(
    name: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    // SAFETY: the caller's contract.
    unsafe {
        if *name == 0 {
            return -1;
        }
        if overwrite == 0 && os_env_exists(name, false) {
            return 0;
        }
        let r = uv_os_setenv(name, value);
        debug_assert!(r != UV_EINVAL);
        if r != 0 {
            log_uv_failure(c"os_setenv", SETENV_FAILED, name, r);
        }
        if r == 0 { 0 } else { -1 }
    }
}

/// Unset an environment variable. Answers 0 or -1.
///
/// # Safety
/// `name` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_unsetenv(name: *const c_char) -> c_int {
    // SAFETY: the caller's contract.
    unsafe {
        if *name == 0 {
            return -1;
        }
        let r = uv_os_unsetenv(name);
        if r != 0 {
            log_uv_failure(c"os_unsetenv", UNSETENV_FAILED, name, r);
        }
        if r == 0 { 0 } else { -1 }
    }
}

/// How many variables the environment block holds.
pub fn os_get_fullenv_size() -> size_t {
    // SAFETY: `environ` is libc's own NULL-terminated block.
    unsafe {
        let mut len = 0;
        while !(*environ.add(len)).is_null() {
            len += 1;
        }
        len
    }
}

/// Free what [`os_copy_fullenv`] allocated.
///
/// # Safety
/// `env` must be NULL or a NULL-terminated vector of owned strings.
pub unsafe fn os_free_fullenv(env: *mut *mut c_char) {
    if env.is_null() {
        return;
    }
    // SAFETY: the caller's contract.
    unsafe {
        let mut it = env;
        while !(*it).is_null() {
            xfree((*it).cast());
            *it = ptr::null_mut();
            it = it.add(1);
        }
        xfree(env.cast());
    }
}

/// Copy the environment into `env` as newly allocated `"NAME=VALUE"` strings.
/// The caller frees them, with [`os_free_fullenv`].
///
/// # Safety
/// `env` must be writable for `env_size` pointers.
pub unsafe fn os_copy_fullenv(env: *mut *mut c_char, env_size: size_t) {
    // SAFETY: the caller's contract; `environ` is NULL-terminated, so the
    // walk stops at whichever end comes first.
    unsafe {
        for i in 0..env_size {
            if (*environ.add(i)).is_null() {
                break;
            }
            *env.add(i) = xstrdup(*environ.add(i));
        }
    }
}

/// The *name* of the environment variable at `index`, newly allocated, or
/// NULL when there is none.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getenvname_at_index(index: size_t) -> *mut c_char {
    // SAFETY: `environ` is libc's own NULL-terminated block; the bound check
    // walks it rather than trusting `index`.
    unsafe {
        for i in 0..=index {
            if (*environ.add(i)).is_null() {
                return ptr::null_mut();
            }
        }
        let str = *environ.add(index);
        debug_assert!(!str.is_null());
        let end = strchr(str, b'=' as c_int);
        debug_assert!(!end.is_null());
        let len = end.offset_from(str);
        debug_assert!(len > 0);
        xmemdupz(str.cast(), len as size_t) as *mut c_char
    }
}

/// This process's id.
#[unsafe(no_mangle)]
pub extern "C" fn os_get_pid() -> int64_t {
    // SAFETY: `getpid` takes no arguments.
    unsafe { getpid() as int64_t }
}

/// Tell the OS that nvim is an interactive application, so it is scheduled
/// like a GUI app. macOS only; nothing to do here.
pub fn os_hint_priority() {}

/// The machine's hostname, into `hostname`, truncated to `size`.
///
/// # Safety
/// `hostname` must be writable for `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_hostname(hostname: *mut c_char, size: size_t) {
    // SAFETY: the caller's contract; `vutsname` is a local `uname` fills in,
    // and `nodename` is NUL-terminated when it succeeds.
    unsafe {
        let mut vutsname: utsname = core::mem::zeroed();
        if uname(&raw mut vutsname) < 0 {
            *hostname = 0;
        } else {
            xstrlcpy(hostname, vutsname.nodename.as_ptr(), size);
        }
    }
}

/// The "real", resolved user home directory, as [`init_homedir`] worked it
/// out.
pub(crate) static homedir: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// Resolve the user's home directory into [`homedir`]:
///
/// 1. `$HOME`,
/// 2. libuv's idea of it,
/// 3. the path each of those resolves to through its links,
/// 4. and, failing all of that, the current working directory.
pub fn init_homedir() {
    // SAFETY: every path below is a NUL-terminated string, and `IObuff` and
    // `os_buf` are the tree's scratch buffers, `IOSIZE` and `MAXPATHL` long.
    unsafe {
        // In case this is a second call.
        xfree(homedir.get().cast());
        homedir.set(ptr::null_mut());

        let mut var = os_getenv(c"HOME".as_ptr());
        let tofree = var;

        if var.is_null() {
            var = os_uv_homedir();
        }
        // Resolve links, so the answer is the "real" directory.
        if !var.is_null() && !os_realpath(var, IObuff.ptr().cast(), IOSIZE).is_null() {
            var = IObuff.ptr().cast();
        }
        // Last resort: wherever nvim was started.
        if (var.is_null() || *var == 0) && os_dirname(os_buf.ptr().cast(), MAXPATHL) == OK {
            var = os_buf.ptr().cast();
        }
        if !var.is_null() {
            homedir.set(xstrdup(var));
        }
        xfree(tofree.cast());
    }
}

/// libuv's answer for the home directory, in a static buffer, or NULL.
static homedir_buf: GlobalCell<[c_char; MAXPATHL]> = GlobalCell::new([0; MAXPATHL]);

fn os_uv_homedir() -> *mut c_char {
    // SAFETY: `homedir_buf` is this module's own static and libuv writes at
    // most `homedir_size` bytes into it.
    unsafe {
        let buf = homedir_buf.ptr().cast::<c_char>();
        *buf = 0;
        let mut homedir_size: size_t = MAXPATHL;
        // http://docs.libuv.org/en/v1.x/misc.html#c.uv_os_homedir
        let ret_value = uv_os_homedir(buf, &raw mut homedir_size);
        if ret_value == 0 && homedir_size < MAXPATHL {
            return buf;
        }
        logmsg_c!(
            LOGLVL_ERR,
            ptr::null(),
            c"os_uv_homedir".as_ptr(),
            0,
            true,
            c"uv_os_homedir() failed %d: %s".as_ptr(),
            ret_value,
            uv_strerror(ret_value),
        );
        *buf = 0;
        ptr::null_mut()
    }
}

/// `ExpandGeneric` source for environment variable names.
///
/// # Safety
/// Called through the `ItemGetter` table; `xp` must be a live [`expand_T`].
pub unsafe extern "C" fn get_env_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    debug_assert!(idx >= 0);
    // SAFETY: the caller's contract; `xp_buf` is `EXPAND_BUF_LEN` bytes.
    unsafe {
        let envname = os_getenvname_at_index(idx as size_t);
        if envname.is_null() {
            return ptr::null_mut();
        }
        xstrlcpy((*xp).xp_buf.as_mut_ptr(), envname, EXPAND_BUF_LEN);
        xfree(envname.cast());
        (*xp).xp_buf.as_mut_ptr()
    }
}

/// Append the directory holding `fname` to `$PATH`. Answers whether it was.
///
/// # Safety
/// `fname` must be an absolute, NUL-terminated path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setenv_append_path(fname: *const c_char) -> bool {
    // SAFETY: the caller's contract; `os_buf` is `MAXPATHL` bytes and the
    // assertion below is what keeps the directory inside it.
    unsafe {
        if !path_is_absolute(fname) {
            internal_error(c"os_setenv_append_path()".as_ptr());
            return false;
        }
        let tail = path_tail_with_sep(fname.cast_mut());
        let dirlen = tail.offset_from(fname) as size_t;
        debug_assert!(tail >= fname.cast_mut() && dirlen + 1 < MAXPATHL);
        xmemcpyz(os_buf.ptr().cast(), fname.cast(), dirlen);

        let path = os_getenv(c"PATH".as_ptr());
        let pathlen = if path.is_null() { 0 } else { strlen(path) };
        let newlen = pathlen + dirlen + 2;
        let mut retval = false;
        if newlen < MAX_ENVPATHLEN {
            let temp = xmalloc(newlen) as *mut c_char;
            if pathlen == 0 {
                *temp = 0;
            } else {
                xstrlcpy(temp, path, newlen);
                if ENV_SEPCHAR != *path.add(pathlen - 1) {
                    xstrlcat(temp, ENV_SEPSTR.as_ptr(), newlen);
                }
            }
            xstrlcat(temp, os_buf.ptr().cast(), newlen);
            os_setenv(c"PATH".as_ptr(), temp, 1);
            xfree(temp.cast());
            retval = true;
        }
        xfree(path.cast());
        retval
    }
}

/// Whether `sh` looks like it resolves to `cmd.exe`.
///
/// # Safety
/// `sh` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_shell_is_cmdexe(sh: *const c_char) -> bool {
    // SAFETY: the caller's contract; `path_tail` answers a pointer inside its
    // argument, and `$COMSPEC` lands in `NameBuff`.
    unsafe {
        if *sh == 0 {
            return false;
        }
        if striequal(sh, c"$COMSPEC".as_ptr()) {
            let comspec = os_getenv_noalloc(c"COMSPEC".as_ptr());
            return striequal(c"cmd.exe".as_ptr(), path_tail(comspec));
        }
        if striequal(sh, c"cmd.exe".as_ptr()) || striequal(sh, c"cmd".as_ptr()) {
            return true;
        }
        striequal(c"cmd.exe".as_ptr(), path_tail(sh))
    }
}

/// [`os_unsetenv`] plus the side effects: `$VIM`/`$VIMRUNTIME` have to be
/// looked up again. `homedir` deliberately keeps its old value until `$HOME`
/// is set again.
///
/// # Safety
/// `var` must be a NUL-terminated string.
pub unsafe fn vim_unsetenv_ext(var: *const c_char) {
    // SAFETY: the caller's contract.
    unsafe {
        os_unsetenv(var);
        if strcasecmp(var, c"VIM".as_ptr()) == 0 {
            didset_vim.set(false);
        } else if strcasecmp(var, c"VIMRUNTIME".as_ptr()) == 0 {
            didset_vimruntime.set(false);
        }
    }
}

/// [`os_setenv`] plus the side effects.
///
/// # Safety
/// `name` and `val` must be NUL-terminated strings.
pub unsafe fn vim_setenv_ext(name: *const c_char, val: *const c_char) {
    // SAFETY: the caller's contract.
    unsafe {
        os_setenv(name, val, 1);
        if strcasecmp(name, c"HOME".as_ptr()) == 0 {
            init_homedir();
        } else if didset_vim.get() && strcasecmp(name, c"VIM".as_ptr()) == 0 {
            didset_vim.set(false);
        } else if didset_vimruntime.get() && strcasecmp(name, c"VIMRUNTIME".as_ptr()) == 0 {
            didset_vimruntime.set(false);
        }
    }
}
