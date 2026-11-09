//! Environment variables, and the home directory.
//!
//! # Boundary
//!
//! Reading one variable is `std::env` (`getenv(3)` under std's lock);
//! enumerating them walks `environ`; writing goes through libuv
//! (`uv_os_setenv`/`uv_os_unsetenv`), whose failures are logged by libuv's
//! name for them. libuv also answers the home directory when `$HOME` is
//! unset and supplies the random bytes, and `uname` the hostname. Those
//! calls and the `environ` walk are this file's only foreign ones.
//!
//! Getting, setting and enumerating variables, plus the resolved home
//! directory. What the editor makes of them -- `$VAR` and `~` inside a path,
//! `$VIM`/`$VIMRUNTIME`, the XDG directories -- is `crate::env`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// Unsafe perimeter: the `os/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::cmdexpand::defs::{Candidate, Expand};
use crate::cstr;
use crate::event::libuv::{uv_os_homedir, uv_os_setenv, uv_os_unsetenv};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::memory::XString;
use crate::message::IOSIZE;
use crate::message::internal_error;
use crate::message_fmt::msg_cstr;
use crate::os::cshim::environ;
use crate::os::fs::{dirname_within, os_err_name, os_strerror, realpath};
use crate::os::libc::{int64_t, size_t};
use crate::os::state::{DIDSET_VIM, DIDSET_VIMRUNTIME};
use crate::path::MAXPATHL;
use crate::path::{path_is_absolute, tail_with_sep_index};
use crate::startup::NVIM_TESTING;
use crate::types::Failed;
use ::libc::{uname, utsname};
use core::ffi::{CStr, c_int};
use std::ffi::{CString, OsStr};
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};

/// The log file the user asked for.
pub(crate) const ENV_LOGFILE: &CStr = c"NVIM_LOG_FILE";
/// The log file the user (or the default) wanted, set when it could not be
/// created; `_core/log.lua:check_log_file` reports it at startup.
pub(crate) const ENV_LOGFILE_WANT: &CStr = c"__NVIM_LOG_FILE_WANT";
/// A child's way back to its parent: the parent's server address.
pub(crate) const ENV_NVIM: &CStr = c"NVIM";
/// Why this instance started: `restart` or `restart!` in a server that
/// `:restart` spawned, read into `v:startreason` at startup and unset.
pub(crate) const ENV_STARTREASON: &CStr = c"__NVIM_STARTREASON";
/// Set by the test harness: log a line once the server (or the TUI) is up,
/// so a test can wait for it.
pub(crate) const ENV_TEST_LOG: &CStr = c"__NVIM_TEST_LOG";

/// How much of a name `get_env_name` copies into `xp_buf`.
const EXPAND_BUF_LEN: size_t = 256;
/// `$PATH`'s separator.
pub(crate) const ENV_SEPCHAR: u8 = b':';
/// No prescribed maximum for `$PATH` on Unix.
const MAX_ENVPATHLEN: usize = c_int::MAX as usize;

/// Where the build installed nvim's Lua runtime files; empty when it did not
/// say.
pub(crate) const DEFAULT_LIB_DIR: &CStr =
    match CStr::from_bytes_with_nul(concat!(env!("NVIM_DEFAULT_LIB_DIR"), "\0").as_bytes()) {
        Ok(dir) => dir,
        Err(_) => panic!("NVIM_DEFAULT_LIB_DIR holds a NUL"),
    };

pub fn env_init() {
    NVIM_TESTING.set(env_exists(c"NVIM_TEST", false));
}

/// `ELOG("uv_os_*(%s) failed: %d %s")`, for a write of the environment that
/// libuv refused.
fn log_uv_failure(func: &'static CStr, call: &str, name: &CStr, code: c_int) {
    let (name, why) = (msg_cstr(name), msg_cstr(os_err_name(code)));
    logmsg!(
        LOGLVL_ERR,
        func,
        0,
        "uv_os_{call}({name}) failed: {code} {why}"
    );
}

/// The value of `name` in the process environment, empty or not, or `None`
/// when it is not set. An empty name is never set.
fn env_value(name: &CStr) -> Option<Vec<u8>> {
    if name.is_empty() {
        return None;
    }
    std::env::var_os(OsStr::from_bytes(name.to_bytes())).map(std::ffi::OsString::into_vec)
}

/// `getenv()`, but `None` for an empty value as well as an unset one:
/// upstream's `os_getenv`.
pub fn getenv_owned(name: &CStr) -> Option<XString> {
    env_value(name)
        .filter(|value| !value.is_empty())
        .map(XString::from_vec)
}

/// [`getenv_owned`] cut to what a `room`-byte buffer holds with its NUL:
/// upstream's `os_getenv_buf`, whose truncation callers rely on.
pub fn env_get_within(name: &CStr, room: usize) -> Option<Vec<u8>> {
    let mut value = env_value(name).filter(|value| !value.is_empty())?;
    value.truncate(room.saturating_sub(1));
    (!value.is_empty()).then_some(value)
}

/// [`env_get_within`] a `MAXPATHL` buffer, which upstream's `NameBuff` was:
/// the value, cut to fit, and empty when the variable is unset or empty.
pub fn env_get_bounded(name: &CStr) -> Vec<u8> {
    env_get_within(name, MAXPATHL as usize).unwrap_or_default()
}

/// Whether environment variable `name` is defined. `nonempty` treats an
/// empty value as "does not exist".
pub fn env_exists(name: &CStr, nonempty: bool) -> bool {
    env_value(name).is_some_and(|value| !nonempty || !value.is_empty())
}

/// Set an environment variable, overwriting it. Answers whether it was set.
pub fn env_set(name: &CStr, value: &CStr) -> bool {
    if name.is_empty() {
        return false;
    }
    // SAFETY: two NUL-terminated strings, which `setenv` copies.
    let code = unsafe { uv_os_setenv(name.as_ptr(), value.as_ptr()) };
    if code != 0 {
        log_uv_failure(c"os_setenv", "setenv", name, code);
    }
    code == 0
}

/// Unset an environment variable. Answers whether it is gone.
pub fn env_unset(name: &CStr) -> bool {
    if name.is_empty() {
        return false;
    }
    // SAFETY: a NUL-terminated name.
    let code = unsafe { uv_os_unsetenv(name.as_ptr()) };
    if code != 0 {
        log_uv_failure(c"os_unsetenv", "unsetenv", name, code);
    }
    code == 0
}

/// A copy of the environment, as `"NAME=VALUE"` strings in its order.
///
/// A walk over `environ` rather than `std::env::vars_os`: that splits each
/// entry into two allocations, which `environ()` would then join and split
/// again, and `jobstart()` copies the whole environment for every job.
pub fn copy_environment() -> Vec<Vec<u8>> {
    let mut env = Vec::new();
    // SAFETY: `environ` is libc's NULL-terminated block of NUL-terminated
    // strings, and each entry is copied before anything can change it.
    unsafe {
        let mut entry = environ;
        while !(*entry).is_null() {
            env.push(CStr::from_ptr(*entry).to_bytes().to_vec());
            entry = entry.add(1);
        }
    }
    env
}

/// The *name* of the environment variable at `index` (what precedes its
/// first `=`), or `None` past the end.
pub fn env_name_at(index: usize) -> Option<Vec<u8>> {
    // SAFETY: as [`copy_environment`]; the walk stops at the NULL that ends
    // the block, however large `index` is.
    let entry = unsafe {
        let mut entry = environ;
        for _ in 0..index {
            if (*entry).is_null() {
                return None;
            }
            entry = entry.add(1);
        }
        (!(*entry).is_null()).then(|| CStr::from_ptr(*entry))
    }?;
    let entry = entry.to_bytes();
    let end = entry.iter().position(|&b| b == b'=').unwrap_or(entry.len());
    Some(entry[..end].to_vec())
}

/// Fill `out` from the operating system's entropy source. `false` when
/// there is none to be had.
pub(crate) fn os_random(out: &mut [u8]) -> bool {
    // SAFETY: a synchronous `uv_random` (no loop, no request, no callback)
    // fills `out`, whose length it is told.
    let rc = unsafe {
        crate::event::libuv::uv_random(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            out.as_mut_ptr().cast(),
            out.len(),
            0,
            None,
        )
    };
    rc == 0
}

/// This process's id.
pub fn process_id() -> int64_t {
    int64_t::from(std::process::id())
}

/// Tell the OS that nvim is an interactive application, so it is scheduled
/// like a GUI app. macOS only; nothing to do here.
pub fn os_hint_priority() {}

/// The machine's hostname into `hostname`, NUL-terminated and truncated to
/// fit; empty when the system will not say.
///
/// # Panics
/// When `hostname` is empty: there is no room for the terminator.
pub fn hostname_into(hostname: &mut [u8]) {
    assert!(!hostname.is_empty(), "no room for the terminator");
    let mut names = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    // SAFETY: `uname` fills in the record it is handed, which is this
    // frame's.
    let failed = unsafe { uname(&raw mut names) } < 0;
    let nodename = names.nodename.map(::core::ffi::c_char::cast_unsigned);
    let name = if failed {
        &[][..]
    } else {
        cstr::until_nul(&nodename)
    };
    let kept = name.len().min(hostname.len() - 1);
    hostname[..kept].copy_from_slice(&name[..kept]);
    hostname[kept] = 0;
}

/// The "real", resolved user home directory, as [`init_homedir`] worked it
/// out.
static HOMEDIR: GlobalCell<Option<XString>> = GlobalCell::new(None);

/// `f` of the resolved home directory, or of `None` before
/// [`init_homedir`] found one.
pub(crate) fn with_home_dir<R>(f: impl FnOnce(Option<&[u8]>) -> R) -> R {
    HOMEDIR.with(|home| f(home.as_deref()))
}

/// Resolve the user's home directory into [`HOMEDIR`]:
///
/// 1. `$HOME`,
/// 2. libuv's idea of it,
/// 3. the path each of those resolves to through its links,
/// 4. and, failing all of that, the current working directory.
pub fn init_homedir() {
    // In case this is a second call.
    HOMEDIR.set(None);

    let home = getenv_owned(c"HOME")
        .map(|home| home.to_vec())
        .or_else(uv_home_dir);
    // Resolve links, so the answer is the "real" directory -- cut to fit, as
    // upstream's `IOSIZE` buffer cuts it.
    let home = home.map(|home| match realpath(&cstr::owned(&home)) {
        Some(mut real) => {
            real.truncate(IOSIZE as usize - 1);
            real
        }
        None => home,
    });
    // Last resort: wherever nvim was started.
    let home = match home {
        Some(home) if !home.is_empty() => Some(home),
        other => dirname_within(MAXPATHL as usize).ok().or(other),
    };
    if let Some(home) = home {
        HOMEDIR.set(Some(XString::from_vec(home)));
    }
}

/// libuv's answer for the home directory (`$HOME`, else the password
/// database), when it fits a `MAXPATHL` buffer.
fn uv_home_dir() -> Option<Vec<u8>> {
    let mut buf = vec![0u8; MAXPATHL as usize];
    let mut size: size_t = buf.len();
    // http://docs.libuv.org/en/v1.x/misc.html#c.uv_os_homedir
    // SAFETY: libuv writes at most `size` bytes into `buf`, which is that
    // long, and the answer's length back into `size`.
    let code = unsafe { uv_os_homedir(buf.as_mut_ptr().cast(), &raw mut size) };
    if code == 0 && size < buf.len() {
        buf.truncate(size);
        return Some(buf);
    }
    let why = msg_cstr(os_strerror(code));
    logmsg!(
        LOGLVL_ERR,
        c"os_uv_homedir",
        0,
        "uv_os_homedir() failed {code}: {why}"
    );
    None
}

/// The `idx`th environment variable's name, for completion, cut to the
/// length upstream's scratch buffer held.
pub fn get_env_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
    let mut name = env_name_at(idx)?;
    name.truncate(EXPAND_BUF_LEN - 1);
    let name = CString::new(name).expect("a variable name holds no NUL");
    Some(Candidate::Owned(name))
}

/// Why [`setenv_append_path`] left `$PATH` alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathNotAppended {
    /// `fname` was relative, which no caller should hand in: said as an
    /// internal error.
    NotAbsolute(Failed),
    /// `$PATH` would grow past what the environment holds; nothing said.
    TooLong,
}

/// Append the directory holding `fname` to `$PATH`.
pub fn setenv_append_path(fname: &CStr) -> Result<(), PathNotAppended> {
    if !path_is_absolute(fname) {
        internal_error(c"os_setenv_append_path()");
        return Err(PathNotAppended::NotAbsolute(Failed));
    }
    let dir = &fname.to_bytes()[..tail_with_sep_index(fname.to_bytes())];
    debug_assert!(dir.len() + 1 < MAXPATHL as usize);
    let path = getenv_owned(c"PATH")
        .map(|path| path.to_vec())
        .unwrap_or_default();
    if path.len() + dir.len() + 2 >= MAX_ENVPATHLEN {
        return Err(PathNotAppended::TooLong);
    }
    let mut joined = path;
    if joined.last().is_some_and(|&last| last != ENV_SEPCHAR) {
        joined.push(ENV_SEPCHAR);
    }
    joined.extend_from_slice(dir);
    env_set(c"PATH", &cstr::owned(&joined));
    Ok(())
}

/// Whether the shell `sh` looks like it resolves to `cmd.exe`: `cmd` or
/// `cmd.exe` in any case, or a path ending in `cmd.exe`, or `$COMSPEC`
/// naming one.
pub fn shell_is_cmdexe(sh: &[u8]) -> bool {
    let tail_is_cmdexe = |path: &[u8]| path[tail_index(path)..].eq_ignore_ascii_case(b"cmd.exe");
    if sh.is_empty() {
        return false;
    }
    if sh.eq_ignore_ascii_case(b"$COMSPEC") {
        return tail_is_cmdexe(&env_get_bounded(c"COMSPEC"));
    }
    sh.eq_ignore_ascii_case(b"cmd.exe") || sh.eq_ignore_ascii_case(b"cmd") || tail_is_cmdexe(sh)
}

/// Where `path_tail` would say the last component of `path` starts.
fn tail_index(path: &[u8]) -> usize {
    let tail = crate::path::path_tail(&cstr::owned(path)).to_bytes().len();
    path.len() - tail
}

/// [`env_set`] plus the side effects: `$HOME` is resolved again and
/// `$VIM`/`$VIMRUNTIME` looked up again.
pub(crate) fn vim_setenv_named(name: &CStr, value: &CStr) {
    env_set(name, value);
    let name = name.to_bytes();
    if name.eq_ignore_ascii_case(b"HOME") {
        init_homedir();
    } else if DIDSET_VIM.get() && name.eq_ignore_ascii_case(b"VIM") {
        DIDSET_VIM.set(false);
    } else if DIDSET_VIMRUNTIME.get() && name.eq_ignore_ascii_case(b"VIMRUNTIME") {
        DIDSET_VIMRUNTIME.set(false);
    }
}

/// [`env_unset`] plus the side effects: `$VIM`/`$VIMRUNTIME` have to be
/// looked up again. `HOMEDIR` deliberately keeps its old value until `$HOME`
/// is set again.
pub(crate) fn vim_unsetenv_named(name: &CStr) {
    env_unset(name);
    let name = name.to_bytes();
    if name.eq_ignore_ascii_case(b"VIM") {
        DIDSET_VIM.set(false);
    } else if name.eq_ignore_ascii_case(b"VIMRUNTIME") {
        DIDSET_VIMRUNTIME.set(false);
    }
}
