//! The `$NVIM_LOG_FILE` log: where the file lives, how a line is framed, and
//! the lock and recursion guard that keep two lines from interleaving.
//!
//! A log line is written in three parts — [`logmsg_begin`] takes the lock and
//! writes the `LVL date.millis name func:line:` prefix, the caller writes the
//! payload, [`logmsg_finish`] terminates and releases. [`logmsg_line`] is
//! that sequence spelled once, and [`logmsg!`] / [`logmsg_tagged!`] are how
//! the ~100 call sites across the tree spell it: a `format_args!` template
//! the compiler checks, not a `printf` format the compiler cannot see.
//!
//! The file is chosen once, by [`log_init`], from `$NVIM_LOG_FILE` with two
//! fallbacks (`$XDG_STATE_HOME/nvim/nvim.log`, then `./nvim.log`) and a last
//! resort of stderr. Whichever wins is published back into the environment so
//! that child processes and `_core/log.lua` agree on it.
//!
//! Everything here holds a real recursive `uv_mutex_t` across the write,
//! because upstream logs from the libuv callbacks as well as from the editor
//! proper. The mutex is the one piece of state that still has to be reached
//! as a raw pointer ([`mutex_ptr`]); libuv takes its address.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::eval::vars::get_vim_var_str;
use crate::event::libuv::{
    uv_gettimeofday, uv_mutex_init_recursive, uv_mutex_lock, uv_mutex_unlock, uv_print_all_handles,
    uv_strerror,
};
use crate::global_cell::GlobalCell;
use crate::main::{g_min_log_level, g_stats, ui_client_channel_id};
use crate::memory::xfree;
use crate::message_fmt::c_str;
use crate::msg_schedule_semsg;
use crate::os::env::{expand_env, os_get_pid, os_getenv_buf, os_setenv};
use crate::os::fs::{os_isdir, os_mkdir_recurse};
use crate::os::stdpaths::{get_xdg_home, stdpaths_user_state_subpath};
use crate::os::time::{os_localtime, tm_zeroed};
use crate::path::path_tail;
use crate::types::{
    FILE, UV_MUTEX_INIT, Vv, XDGVarType, int32_t, uv_loop_t, uv_mutex_t, uv_timeval64_t,
};
/// `#[macro_export]` publishes at the crate root; this re-export lets callers
/// name the macro where the rest of the logging API lives, and brings it into
/// scope here ahead of its own textual definition.
pub(crate) use crate::{logmsg, logmsg_tagged};
use core::ffi::{CStr, c_char, c_int, c_void};
use std::ffi::CString;

use crate::os::cshim::{snprintf, stderr, stdout};
use ::libc::{
    __errno_location, fclose, fflush, fopen, fprintf, fputc, fputs, fwrite, strerror, strftime,
};

/// The levels [`logmsg_line`] takes, and 'verbose' compares against.
pub(crate) const LOGLVL_DBG: c_int = 1;
pub(crate) const LOGLVL_INF: c_int = 2;
pub(crate) const LOGLVL_WRN: c_int = 3;
pub(crate) const LOGLVL_ERR: c_int = 4;

const kXDGStateHome: XDGVarType = 3;
/// `MAXPATHL + 1`: what `log_file_path[]` is declared as upstream, and the
/// bound `expand_env`/`xstrlcpy` truncate the path to.
const LOG_PATH_SIZE: usize = 4096 + 1;
const ENV_LOGFILE: &CStr = c"NVIM_LOG_FILE";
const ENV_LOGFILE_REF: &CStr = c"$NVIM_LOG_FILE";
const ENV_NVIM: &CStr = c"NVIM";

/// The log file decided by [`log_path_init`], or `None` while logging still
/// falls back to stderr (before `log_init`, or when no candidate path was
/// writable).
static LOG_FILE_PATH: GlobalCell<Option<CString>> = GlobalCell::new(None);
static DID_LOG_INIT: GlobalCell<bool> = GlobalCell::new(false);
static MUTEX: GlobalCell<uv_mutex_t> = GlobalCell::new(UV_MUTEX_INIT);

/// The mutex's address, which is the only way libuv can be handed it. The
/// sole raw-pointer escape left in this module; every other piece of state
/// is read and written through the cell API.
fn mutex_ptr() -> *mut uv_mutex_t {
    MUTEX.as_raw()
}

/// Set while a log line is being written, so a log call made from inside one
/// is refused rather than interleaved. `DID_RECURSION_MSG` keeps that refusal
/// to a single user-visible complaint per session.
static LOGGING: GlobalCell<bool> = GlobalCell::new(false);
static DID_RECURSION_MSG: GlobalCell<bool> = GlobalCell::new(false);

// ---------------------------------------------------------------------------
// Choosing the log file.

/// Whether `fname` names something that can be appended to. Creates it if it
/// does not exist; an empty name never can.
fn log_try_create(fname: &[u8]) -> bool {
    let Ok(fname) = CString::new(fname) else {
        return false;
    };
    if fname.as_bytes().is_empty() {
        return false;
    }
    // SAFETY: a NUL-terminated name, and the handle is closed here or never
    // escapes.
    let log_file = unsafe { fopen(fname.as_ptr(), c"a".as_ptr()) };
    if log_file.is_null() {
        return false;
    }
    unsafe { fclose(log_file) };
    true
}

/// `xstrlcpy` into a `LOG_PATH_SIZE` buffer, as an answer instead of a
/// side effect: the path cut to fit, and whether `src` was too long to fit
/// (upstream's `len >= size` test).
fn fit_path(src: &[u8]) -> (Vec<u8>, bool) {
    let overflowed = src.len() >= LOG_PATH_SIZE;
    (src[..src.len().min(LOG_PATH_SIZE - 1)].to_vec(), overflowed)
}

/// `os_setenv(name, value, true)` for a path that is not NUL-terminated yet.
fn setenv_path(name: &CStr, value: &[u8]) {
    let Ok(value) = CString::new(value) else {
        return;
    };
    // SAFETY: two NUL-terminated strings; os_setenv copies both.
    unsafe { os_setenv(name.as_ptr(), value.as_ptr(), 1) };
}

fn path_is_dir(path: &[u8]) -> bool {
    let Ok(path) = CString::new(path) else {
        return false;
    };
    // SAFETY: a NUL-terminated path.
    unsafe { os_isdir(path.as_ptr()) }
}

/// Expand `$NVIM_LOG_FILE`. Returns the expansion and whether the user
/// actually set the variable (`expand_env` leaves the reference verbatim
/// when it is unset).
fn expand_log_file_var() -> (Vec<u8>, bool) {
    let mut buf = [0 as c_char; LOG_PATH_SIZE];
    // SAFETY: `buf` holds `LOG_PATH_SIZE` chars and the bound passed is one
    // less, as upstream's is; the source is a NUL-terminated literal that
    // `expand_env` only reads.
    let src = ENV_LOGFILE_REF.as_ptr().cast_mut();
    let (into, cap) = (buf.as_mut_ptr(), LOG_PATH_SIZE as c_int - 1);
    unsafe { expand_env(src, into, cap) };
    let expanded = cstr::in_chars(&buf).to_bytes().to_vec();
    let user_set = expanded != ENV_LOGFILE_REF.to_bytes();
    (expanded, user_set)
}

/// Make `$XDG_STATE_HOME` if it does not exist. Returns the libuv error and
/// the directory that failed, for the warning [`log_path_init`] logs once the
/// log file is open.
fn ensure_state_home() -> (c_int, Option<CString>) {
    let mut failed_dir: *mut c_char = core::ptr::null_mut();
    let mut log_dir_failure = 0;
    // SAFETY: `get_xdg_home` hands back an owned NUL-terminated path, freed
    // here; `os_mkdir_recurse` writes at most one owned string to
    // `failed_dir`, which is read back and freed by the caller.
    let loghome = get_xdg_home(kXDGStateHome);
    if !unsafe { os_isdir(loghome) } {
        log_dir_failure = unsafe {
            os_mkdir_recurse(
                loghome,
                0o700 as int32_t,
                &raw mut failed_dir,
                core::ptr::null_mut(),
            )
        };
    }
    unsafe { xfree(loghome as *mut c_void) };
    let owned = (!failed_dir.is_null()).then(|| unsafe { CStr::from_ptr(failed_dir) }.to_owned());
    unsafe { xfree(failed_dir as *mut c_void) };
    (log_dir_failure, owned)
}

/// The default log path, `$XDG_STATE_HOME/nvim/nvim.log`.
fn default_log_path() -> Vec<u8> {
    // SAFETY: a NUL-terminated literal in, an owned NUL-terminated path out,
    // freed here.
    let p = unsafe { stdpaths_user_state_subpath(c"nvim.log".as_ptr(), 0, true) };
    let owned = unsafe { CStr::from_ptr(p) }.to_bytes().to_vec();
    unsafe { xfree(p as *mut c_void) };
    owned
}

/// Decide the log file path and publish it as `$NVIM_LOG_FILE`.
///
/// Tries `$NVIM_LOG_FILE`, then `$XDG_STATE_HOME/nvim/nvim.log`, then
/// `./nvim.log`, and finally gives up — after which every log line goes to
/// stderr. Whenever a *wanted* path had to be abandoned it is left in
/// `$__NVIM_LOG_FILE_WANT`, which `_core/log.lua:check_log_file` reports.
fn log_path_init() {
    let (wanted, user_set) = expand_log_file_var();

    if user_set && !wanted.is_empty() && !path_is_dir(&wanted) && log_try_create(&wanted) {
        // The user's path works. Upstream leaves $NVIM_LOG_FILE alone here:
        // it already holds this value.
        LOG_FILE_PATH.set(CString::new(wanted).ok());
        return;
    }
    if user_set {
        setenv_path(c"__NVIM_LOG_FILE_WANT", &wanted);
    }

    // Making the directory comes before the path that lives in it, and its
    // failure can only be *logged* once a log file is open.
    let (log_dir_failure, failed_dir) = ensure_state_home();

    let (mut path, mut overflowed) = fit_path(&default_log_path());
    if overflowed || !log_try_create(&path) {
        if !user_set {
            setenv_path(c"__NVIM_LOG_FILE_WANT", &path);
        }
        (path, overflowed) = fit_path(b"nvim.log");
    }
    if overflowed || !log_try_create(&path) {
        LOG_FILE_PATH.set(None); // Fall back to stderr.
        return;
    }

    setenv_path(ENV_LOGFILE, &path);
    LOG_FILE_PATH.set(CString::new(path).ok());
    if log_dir_failure != 0 {
        // `as_ref`, not `map_or` by value: moving the `CString` into the
        // closure drops it at the end of the expression, and the pointer
        // `as_ptr` answered is into the buffer that just went. glibc's `%s`
        // read the freed bytes without ASan noticing, because the scan is
        // inside libc; `CStr::from_ptr`'s `strlen` is intercepted.
        let dir = failed_dir
            .as_ref()
            .map_or(core::ptr::null(), |d| d.as_ptr());
        // SAFETY: `dir` borrows `failed_dir`, which outlives the call;
        // `uv_strerror` returns a static string.
        let (dir, why) = unsafe { (c_str(dir), c_str(uv_strerror(log_dir_failure))) };
        logmsg!(
            LOGLVL_WRN,
            c"log_path_init",
            106,
            "Failed to create directory {dir} for writing logs: {why}"
        );
    }
}

/// Set up the log mutex and decide the log file. Called from startup, after
/// `init_homedir` and `set_init_1` — the path depends on both (#11501).
pub(crate) fn log_init() {
    // SAFETY: initialising the process-wide mutex once, before any lock.
    unsafe { uv_mutex_init_recursive(mutex_ptr()) };
    log_path_init();
    DID_LOG_INIT.set(true);
}

fn log_lock() {
    // SAFETY: the mutex was initialised by `log_init`; recursive, so a
    // nested lock on this thread is well defined.
    unsafe { uv_mutex_lock(mutex_ptr()) };
}

fn log_unlock() {
    // SAFETY: paired with a `log_lock` on this thread.
    unsafe { uv_mutex_unlock(mutex_ptr()) };
}

// ---------------------------------------------------------------------------
// Writing a line.

/// The half of a log line that runs before its payload: the initialisation,
/// level and recursion guards, the log lock, the file handle, and the
/// date/level/name/location prefix.
///
/// Returns the open log file with the lock held and the prefix already
/// written, or null when the line is not going to be written at all — in
/// which case nothing is held and the caller is done. **Every non-null
/// return has to be paired with a [`logmsg_finish`]**; [`logmsg_line`] is
/// that pairing, and is how this should be called.
///
/// # Safety
/// `context` and `func_name` are NUL-terminated or null, and outlive the
/// call.
pub(crate) unsafe fn logmsg_begin(
    log_level: c_int,
    context: *const c_char,
    func_name: *const c_char,
    line_num: c_int,
) -> *mut FILE {
    if !DID_LOG_INIT.get() {
        // set_init_1 may try logging before we are ready (#10183).
        g_stats.with_mut(|s| s.log_skip += 1);
        return core::ptr::null_mut();
    }
    if log_level < g_min_log_level.get() {
        return core::ptr::null_mut();
    }
    log_lock();
    if LOGGING.get() {
        if !DID_RECURSION_MSG.get() {
            DID_RECURSION_MSG.set(true);
            let who = if func_name.is_null() {
                context
            } else {
                func_name
            };
            // SAFETY: the caller's strings, formatted with matching verbs.
            let who = unsafe { c_str(who) };
            msg_schedule_semsg!("E5430: {who}:{}: recursive log!", line_num);
        }
        g_stats.with_mut(|s| s.log_skip += 1);
        log_unlock();
        return core::ptr::null_mut();
    }
    LOGGING.set(true);
    let log_file = open_log_file();
    // SAFETY: `log_file` is open, and the caller's strings outlive the call.
    if !unsafe { log_write_prefix(log_file, log_level, context, func_name, line_num) } {
        // The prefix is the head of the line; with none written there is
        // nothing to append to, so release everything here and report the
        // same failure the payload would have.
        // SAFETY: `log_file` is what this call just took.
        unsafe { logmsg_finish(log_file, false, false) };
        return core::ptr::null_mut();
    }
    log_file
}

/// The half of a log line that runs after its payload: the end-of-line, the
/// flush, and releasing what [`logmsg_begin`] took.
///
/// `payload_ok` says whether the payload was written; a failed payload skips
/// the terminator and the flush but still releases. Returns whether the whole
/// line landed.
///
/// # Safety
/// `log_file` is the non-null handle a [`logmsg_begin`] call returned, and
/// this is its first `logmsg_finish`.
pub(crate) unsafe fn logmsg_finish(log_file: *mut FILE, eol: bool, payload_ok: bool) -> bool {
    let mut ret = payload_ok;
    // SAFETY: the caller's open handle; `stderr`/`stdout` are the two the
    // module borrows rather than owns.
    if ret {
        if eol {
            unsafe { fputc(b'\n' as c_int, log_file) };
        }
        if unsafe { fflush(log_file) } == EOF {
            ret = false;
        }
    }
    if log_file != unsafe { stderr } && log_file != unsafe { stdout } {
        unsafe { fclose(log_file) };
    }
    LOGGING.set(false);
    log_unlock();
    ret
}

const EOF: c_int = -1;

/// Write one line to the log at `log_level`, tagged with
/// `context`/`func_name`/`line_num` and terminated by a newline when `eol`.
/// Answers whether the line landed.
///
/// `text` renders the payload — [`logmsg_begin`] takes the lock and writes
/// the prefix, this writes what `text` produced, [`logmsg_finish`]
/// terminates it and releases. Same handle, same order, same bytes as the C
/// wrapper.
///
/// **`text` runs only when the line is actually going to be written.** C
/// evaluated every variadic argument before `logmsg()` could decide, and the
/// macro that replaced it did the same on purpose; a closure lets the
/// arguments of a `LOGLVL_DBG` line cost nothing in a build that is not
/// logging at that level, which is what the per-message RPC trace wanted.
/// Nothing passed here has a side effect, so the only difference is the
/// cost.
///
/// The two tags are `'static` literals, which is the whole difference
/// between this and the `fprintf` it replaced: a log line no longer has a
/// raw pointer anywhere in it, so the call site needs no `unsafe`.
pub(crate) fn logmsg_line(
    log_level: c_int,
    context: Option<&'static CStr>,
    func_name: Option<&'static CStr>,
    line_num: c_int,
    eol: bool,
    text: impl FnOnce() -> String,
) -> bool {
    let ptr = |s: Option<&'static CStr>| s.map_or(core::ptr::null(), CStr::as_ptr);
    // SAFETY: two `'static` C strings, or null.
    let log_file = unsafe { logmsg_begin(log_level, ptr(context), ptr(func_name), line_num) };
    if log_file.is_null() {
        return false;
    }
    let payload = crate::message_fmt::to_bytes(&text());
    // SAFETY: `log_file` is the open handle `logmsg_begin` just answered, and
    // `payload` is a live buffer of exactly that many bytes.
    let written = unsafe { fwrite(payload.as_ptr().cast(), 1, payload.len(), log_file) };
    // SAFETY: as above; this is `log_file`'s one `logmsg_finish`.
    unsafe { logmsg_finish(log_file, eol, written == payload.len()) }
}

/// One log line: the level, who is logging, the line number they are at, and
/// a `format_args!` template.
///
/// ```ignore
/// logmsg!(LOGLVL_ERR, c"os_proc_tree_kill", 103, "invalid pid {pid}");
/// ```
///
/// The template is a Rust literal and the arguments are `Display`s, so the
/// compiler checks the two against each other — which is the whole point of
/// retiring the `fprintf` this replaced, where a `&CStr` handed to a `%s`
/// passed its *length word* off as string bytes. A pointer argument goes
/// through `message_fmt`'s adaptors (`c_str`, `msg_bytes`, `msg_addr`) the
/// same way a message's does; bytes that are not UTF-8 survive.
///
/// `who` is a `CStr` literal — the upstream function name, which the log
/// prefix prints with the line number.
#[macro_export]
macro_rules! logmsg {
    ($level:expr, $who:expr, $line:expr, $($fmt:tt)*) => {
        $crate::log::logmsg_line(
            $level,
            ::core::option::Option::None,
            ::core::option::Option::Some($who),
            $line,
            true,
            || ::std::format!($($fmt)*),
        )
    };
}

/// [`logmsg!`] for a line that carries a *tag* instead of a source location:
/// the RPC trace and the UI event log, which name a channel or an event
/// rather than a function.
///
/// `eol` is false for the trace lines that append their payload to the line
/// before them.
#[macro_export]
macro_rules! logmsg_tagged {
    ($level:expr, $tag:expr, $eol:expr, $($fmt:tt)*) => {
        $crate::log::logmsg_line(
            $level,
            ::core::option::Option::Some($tag),
            ::core::option::Option::None,
            -1,
            $eol,
            || ::std::format!($($fmt)*),
        )
    };
}

/// Dump libuv's handle table to the log — the `:checkhealth`-adjacent view
/// of what the event loop is still holding.
///
/// # Safety
/// `loop_0` is a live `uv_loop_t *`.
pub(crate) unsafe fn log_uv_handles(loop_0: *mut c_void) {
    log_lock();
    let log_file = open_log_file();
    // SAFETY: the caller's loop, and a handle this call owns except for the
    // two standard streams.
    unsafe { uv_print_all_handles(loop_0 as *mut uv_loop_t, log_file) };
    if log_file != unsafe { stderr } && log_file != unsafe { stdout } {
        unsafe { fclose(log_file) };
    }
    log_unlock();
}

/// Open the log file for appending, or stderr when there is no usable one —
/// which is also reported, once per attempt, on stderr itself.
fn open_log_file() -> *mut FILE {
    // SAFETY: `errno` is this thread's; `fopen` takes a NUL-terminated path
    // held alive by the cell borrow.
    let f = unsafe {
        *__errno_location() = 0;
        LOG_FILE_PATH.with(|path| match path {
            Some(path) => fopen(path.as_ptr(), c"a".as_ptr()),
            None => core::ptr::null_mut(),
        })
    };
    if !f.is_null() {
        return f;
    }

    // May happen if fopen() failed, a log call ran before log_init(), the
    // directory does not exist, or the file is not writable. `strerror`
    // comes first: the prefix writer would clobber fopen's errno.
    // SAFETY: `strerror` returns a static string for any errno.
    let reason = unsafe { CStr::from_ptr(strerror(*__errno_location())).to_string_lossy() };
    let path = LOG_FILE_PATH.with(|p| {
        p.as_ref()
            .map_or_else(String::new, |p| p.to_string_lossy().into_owned())
    });
    // The trailing newline stands in for the eol=true of upstream's call.
    let msg = format!("failed to open $NVIM_LOG_FILE ({reason}): {path}\n\0");
    // SAFETY: `stderr` is open; `msg` is NUL-terminated and outlives the
    // call; the literals are NUL-terminated.
    let no_context = core::ptr::null::<c_char>();
    let here = c"open_log_file".as_ptr();
    let err = unsafe { stderr };
    if unsafe { log_write_prefix(err, LOGLVL_ERR, no_context, here, 234) } {
        unsafe { fputs(msg.as_ptr() as *const c_char, stderr) };
        unsafe { fflush(stderr) };
    }
    unsafe { stderr }
}

// ---------------------------------------------------------------------------
// The line prefix.

/// The three-letter tag a level prints as.
fn log_level_tag(log_level: c_int) -> *const c_char {
    match log_level {
        LOGLVL_DBG => c"DBG".as_ptr(),
        LOGLVL_INF => c"INF".as_ptr(),
        LOGLVL_WRN => c"WRN".as_ptr(),
        LOGLVL_ERR => c"ERR".as_ptr(),
        // Upstream asserts the range and would index past the table; a
        // compiled-out assert leaves a wild pointer, so answer with nothing.
        _ => c"".as_ptr(),
    }
}

/// Name of the Nvim instance that produced the log, as `%-10s` in every
/// prefix. `snprintf`-truncated to 31 bytes exactly as upstream's `char
/// name[32]` is.
static NAME: GlobalCell<[c_char; 32]> = GlobalCell::new([0; 32]);

/// Whether the instance name has to be (re)generated: as a UI client, so
/// that "ui" is in the name; when it was never set; or when it was last
/// built from the pid because there was no `v:servername` yet.
fn name_is_stale(ui: bool) -> bool {
    ui || NAME.with(|n| n[0] == 0 || n[0] == b'?' as c_char)
}

/// Rebuild [`NAME`] from the parent server (`$NVIM`), `v:servername`, or the
/// pid, in that order.
fn regen_name(ui: bool) {
    let mut parent_buf = [0 as c_char; 4096];
    // SAFETY: `parent_buf` is what its length says; `os_getenv_buf` and
    // `get_vim_var_str` both answer with NUL-terminated strings that outlive
    // the `snprintf` below (`parent` points into `parent_buf`, `serv` into
    // the vim variable).
    let parent = unsafe {
        path_tail(os_getenv_buf(
            ENV_NVIM.as_ptr(),
            parent_buf.as_mut_ptr(),
            parent_buf.len(),
        ))
    };
    let serv = unsafe { path_tail(get_vim_var_str(Vv::Servername)) };
    NAME.with_mut(|name| {
        let (n, len) = (name.as_mut_ptr(), name.len());
        if unsafe { *parent } != 0 {
            // "c/" = child of $NVIM.
            let fmt = if ui { c"ui/c/%s" } else { c"c/%s" };
            unsafe { snprintf(n, len, fmt.as_ptr(), parent) };
        } else if unsafe { *serv } != 0 {
            let fmt = if ui { c"ui/%s" } else { c"%s" };
            unsafe { snprintf(n, len, fmt.as_ptr(), serv) };
        } else {
            let who = if ui { c"ui" } else { c"?" };
            unsafe { snprintf(n, len, c"%s.%-5ld".as_ptr(), who.as_ptr(), os_get_pid()) };
        }
    });
}

/// `%Y-%m-%dT%H:%M:%S` of the current local time, plus the millisecond
/// fraction. `None` when the clock cannot be rendered at all.
fn log_timestamp() -> Option<([c_char; 20], c_int)> {
    let mut local_time = tm_zeroed();
    if !os_localtime(&mut local_time) {
        return None;
    }
    let mut date_time = [0 as c_char; 20];
    // SAFETY: `date_time` is what its length says and `local_time` is
    // initialised.
    let (into, cap) = (date_time.as_mut_ptr(), date_time.len());
    let (fmt, when) = (c"%Y-%m-%dT%H:%M:%S".as_ptr(), &raw mut local_time);
    let written = unsafe { strftime(into, cap, fmt, when) };
    if written == 0 {
        return None;
    }
    let mut curtime = uv_timeval64_t {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: `uv_gettimeofday` fills the value it is handed.
    let clock_read = unsafe { uv_gettimeofday(&raw mut curtime) } == 0;
    let millis = if clock_read {
        curtime.tv_usec / 1000
    } else {
        0
    };
    Some((date_time, millis))
}

/// The date/level/name/source-location head of a log line, up to where the
/// payload starts. Split out of upstream's `v_do_log_to_file` so that a
/// preformatted message ([`open_log_file`]'s fallback) can log without a
/// variadic hop.
///
/// # Safety
/// `log_file` is open; `context` and `func_name` are NUL-terminated or null.
unsafe fn log_write_prefix(
    log_file: *mut FILE,
    log_level: c_int,
    context: *const c_char,
    func_name: *const c_char,
    line_num: c_int,
) -> bool {
    debug_assert!(
        (LOGLVL_DBG..=LOGLVL_ERR).contains(&log_level),
        "log_level >= LOGLVL_DBG && log_level <= LOGLVL_ERR"
    );
    let Some((date_time, millis)) = log_timestamp() else {
        return false;
    };

    // Running as a UI client (--remote-ui).
    let ui = ui_client_channel_id.get() != 0;
    if name_is_stale(ui) {
        regen_name(ui);
    }

    // Without a source location the context stands in for it, and an absent
    // context prints as the "unknown location" marker instead of nothing.
    let located = line_num != -1 && !func_name.is_null();
    let ctx = if !context.is_null() {
        context
    } else if located {
        c"".as_ptr()
    } else {
        c"?:".as_ptr()
    };
    let tag = log_level_tag(log_level);
    let date = date_time.as_ptr();

    let rv = NAME.with(|name| {
        let name = name.as_ptr();
        // SAFETY: every argument outlives the call and matches its verb;
        // `name` is borrowed only for the duration of the write.
        if located {
            let fmt = c"%s %s.%03d %-10s %s%s:%d: ".as_ptr();
            let (at, line) = (func_name, line_num);
            unsafe { fprintf(log_file, fmt, tag, date, millis, name, ctx, at, line) }
        } else {
            let fmt = c"%s %s.%03d %-10s %s".as_ptr();
            unsafe { fprintf(log_file, fmt, tag, date, millis, name, ctx) }
        }
    });
    rv >= 0
}
