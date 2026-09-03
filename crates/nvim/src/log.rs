//! The `$NVIM_LOG_FILE` log: where the file lives, how a line is framed, and
//! the lock and recursion guard that keep two lines from interleaving.
//!
//! [`logmsg_line`] writes one line: it takes the lock, opens the file,
//! renders the `LVL date.millis name func:line:` prefix and the caller's
//! payload into one buffer, and appends that buffer in a single write.
//! [`logmsg!`] / [`logmsg_tagged!`] are how the ~100 call sites across the
//! tree spell it — a `format_args!` template the compiler checks, not a
//! `printf` format the compiler cannot see.
//!
//! The file is chosen once, by [`log_init`], from `$NVIM_LOG_FILE` with two
//! fallbacks (`$XDG_STATE_HOME/nvim/nvim.log`, then `./nvim.log`) and a last
//! resort of stderr. Whichever wins is published back into the environment so
//! that child processes and `_core/log.lua` agree on it.
//!
//! # Why there is no `FILE *` here any more
//!
//! Upstream logs from libuv callbacks as well as from the editor proper, so
//! the lock is a real cross-thread lock and it has to be re-entrant: a log
//! call made from inside a log call must reach the recursion guard rather
//! than deadlock on the way to it. That was a `uv_mutex_t` reached by
//! address, which was this module's last raw pointer; [`ReentrantLock`] is
//! the same contract with the address kept private.
//!
//! Everything else followed from writing the line as bytes instead of
//! through `fprintf`: the prefix is one `write!` into a `Vec<u8>`, the
//! timestamp is `os_localtime`'s fields formatted rather than `strftime`'s,
//! the instance name is truncated to the 31 bytes `char name[32]` held
//! rather than by `snprintf`, and the file is a [`File`] opened for append
//! per line as the `fopen(…, "a")` per line already was. One write per line
//! also means a line is now all-or-nothing, where C could leave a prefix
//! behind when the payload failed.
//!
//! The one caller that genuinely needs a `FILE *` — `uv_print_all_handles`,
//! which takes one — lives in `event::loop` now, and reaches the log through
//! [`log_file_path`] and [`with_log_lock`].

#![forbid(unsafe_code)]

use crate::eval::vars::vim_var_bytes;
use crate::global_cell::GlobalCell;
use crate::main::{g_min_log_level, g_stats, ui_client_channel_id};
use crate::message_fmt::{msg_cstr, to_bytes};
use crate::msg_schedule_semsg;
use crate::os::env::{env_get_bounded, env_set, expand_env_into, os_get_pid};
use crate::os::fs::{MkdirFailure, dir_exists, mkdir_recurse};
use crate::os::stdpaths::{user_state_subpath, xdg_home};
use crate::os::time::{os_localtime, tm_zeroed};
use crate::path::tail_index;
use crate::types::{Vv, XDGVarType, int32_t};
/// `#[macro_export]` publishes at the crate root; this re-export lets callers
/// name the macro where the rest of the logging API lives, and brings it into
/// scope here ahead of its own textual definition.
pub(crate) use crate::{logmsg, logmsg_tagged};
use core::ffi::{CStr, c_int};
use std::cell::Cell;
use std::ffi::{CString, OsStr};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The levels [`logmsg_line`] takes, and 'verbose' compares against.
pub const LOGLVL_DBG: c_int = 1;
pub const LOGLVL_INF: c_int = 2;
pub const LOGLVL_WRN: c_int = 3;
pub const LOGLVL_ERR: c_int = 4;

const kXDGStateHome: XDGVarType = 3;
/// `MAXPATHL + 1`: what `log_file_path[]` is declared as upstream, and the
/// bound `expand_env`/`xstrlcpy` truncate the path to.
const LOG_PATH_SIZE: usize = 4096 + 1;
/// What `char name[32]` held: 31 bytes and a terminator.
const NAME_SIZE: usize = 31;
/// The `%-10s` the instance name is printed in.
const NAME_COLUMNS: usize = 10;
const ENV_LOGFILE: &CStr = c"NVIM_LOG_FILE";
const ENV_LOGFILE_REF: &CStr = c"$NVIM_LOG_FILE";
const ENV_LOGFILE_WANT: &CStr = c"__NVIM_LOG_FILE_WANT";
const ENV_NVIM: &CStr = c"NVIM";

/// The log file decided by [`log_path_init`], or `None` while logging still
/// falls back to stderr (before `log_init`, or when no candidate path was
/// writable).
static LOG_FILE_PATH: GlobalCell<Option<CString>> = GlobalCell::new(None);
static DID_LOG_INIT: GlobalCell<bool> = GlobalCell::new(false);

/// Held across a whole log line, so that two threads' lines cannot
/// interleave. Upstream logs from libuv callbacks as well as from the editor
/// proper, which is what makes this a real cross-thread lock.
static LOG_LOCK: Mutex<()> = Mutex::new(());

// `LOGGING` is set while *this thread* is writing a log line, so a log call
// made from inside one is refused rather than deadlocking on the lock the
// outer call still holds. Upstream spells this as a global `bool` behind a
// re-entrant mutex, which comes to the same thing: another thread never reads
// it, because it is blocked on the lock until the flag is false again.
// `DID_RECURSION_MSG` keeps the refusal to one user-visible complaint per
// session.
thread_local! {
    static LOGGING: Cell<bool> = const { Cell::new(false) };
}
static DID_RECURSION_MSG: GlobalCell<bool> = GlobalCell::new(false);

/// Held for the length of one log line: the cross-thread lock and this
/// thread's "inside a line" flag, both released however the line ends.
struct Writing(#[expect(dead_code, reason = "held for its lock")] MutexGuard<'static, ()>);

impl Drop for Writing {
    fn drop(&mut self) {
        LOGGING.with(|inside| inside.set(false));
    }
}

/// Claim the log for one line, or `None` when this thread is already inside
/// one — which is the caller's cue to complain rather than write.
fn begin_line() -> Option<Writing> {
    if LOGGING.with(Cell::get) {
        return None;
    }
    let guard = LOG_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    LOGGING.with(|inside| inside.set(true));
    Some(Writing(guard))
}

// ---------------------------------------------------------------------------
// Choosing the log file.

/// `fname` as a path, or `None` when it is empty or holds an interior NUL —
/// the two cases `fopen` and `CString` respectively refuse.
fn as_path(fname: &[u8]) -> Option<&Path> {
    let usable = !fname.is_empty() && !fname.contains(&0);
    usable.then(|| Path::new(OsStr::from_bytes(fname)))
}

/// The log file open for appending, creating it if it does not exist.
fn append_to(path: &Path) -> io::Result<File> {
    OpenOptions::new().append(true).create(true).open(path)
}

/// Whether `fname` names something that can be appended to. Creates it if it
/// does not exist; an empty name never can.
fn log_try_create(fname: &[u8]) -> bool {
    as_path(fname).is_some_and(|path| append_to(path).is_ok())
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
    if let Ok(value) = CString::new(value) {
        env_set(name, &value);
    }
}

fn path_is_dir(path: &[u8]) -> bool {
    CString::new(path).is_ok_and(|path| dir_exists(&path))
}

/// Expand `$NVIM_LOG_FILE`. Returns the expansion and whether the user
/// actually set the variable (`expand_env` leaves the reference verbatim
/// when it is unset).
fn expand_log_file_var() -> (Vec<u8>, bool) {
    let expanded = expand_env_into(ENV_LOGFILE_REF, LOG_PATH_SIZE);
    let user_set = expanded != ENV_LOGFILE_REF.to_bytes();
    (expanded, user_set)
}

/// Make `$XDG_STATE_HOME` if it does not exist. Returns what failed, for the
/// warning [`log_path_init`] logs once the log file is open.
///
/// An unset state directory answers `None` rather than being tested: upstream
/// hands `get_xdg_home`'s null straight to `os_isdir`.
fn ensure_state_home() -> Option<MkdirFailure> {
    let home = xdg_home(kXDGStateHome)?;
    if dir_exists(&home) {
        return None;
    }
    mkdir_recurse(&home, 0o700 as int32_t).err()
}

/// The default log path, `$XDG_STATE_HOME/nvim/nvim.log`.
fn default_log_path() -> Vec<u8> {
    user_state_subpath(c"nvim.log").into_bytes()
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
        setenv_path(ENV_LOGFILE_WANT, &wanted);
    }

    // Making the directory comes before the path that lives in it, and its
    // failure can only be *logged* once a log file is open.
    let failure = ensure_state_home();

    let (mut path, mut overflowed) = fit_path(&default_log_path());
    if overflowed || !log_try_create(&path) {
        if !user_set {
            setenv_path(ENV_LOGFILE_WANT, &path);
        }
        (path, overflowed) = fit_path(b"nvim.log");
    }
    if overflowed || !log_try_create(&path) {
        LOG_FILE_PATH.set(None); // Fall back to stderr.
        return;
    }

    setenv_path(ENV_LOGFILE, &path);
    LOG_FILE_PATH.set(CString::new(path).ok());
    if let Some(failure) = failure {
        let dir = failure.dir.unwrap_or_default();
        let (dir, why) = (msg_cstr(&dir), msg_cstr(failure.why));
        logmsg!(
            LOGLVL_WRN,
            c"log_path_init",
            106,
            "Failed to create directory {dir} for writing logs: {why}"
        );
    }
}

/// Decide the log file. Called from startup, after `init_homedir` and
/// `set_init_1` — the path depends on both (#11501).
pub fn log_init() {
    log_path_init();
    DID_LOG_INIT.set(true);
}

/// The log file's path, or `None` when logging falls back to stderr.
pub(crate) fn log_file_path() -> Option<CString> {
    LOG_FILE_PATH.with(Clone::clone)
}

/// Run `f` with the log's lock held, so that what it writes cannot land in
/// the middle of a log line.
pub(crate) fn with_log_lock<T>(f: impl FnOnce() -> T) -> T {
    let _guard = LOG_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    f()
}

// ---------------------------------------------------------------------------
// Writing a line.

/// Where a log line goes: the file that was chosen, or stderr when there is
/// none or it could not be opened.
enum Sink {
    File(File),
    Stderr,
}

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Sink::File(file) => file.write(buf),
            Sink::Stderr => io::stderr().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Sink::File(file) => file.flush(),
            Sink::Stderr => io::stderr().flush(),
        }
    }
}

/// Open the log file for appending, or stderr when there is no usable one —
/// which is also reported, once per attempt, on stderr itself.
fn open_log_file() -> Sink {
    let path = log_file_path();
    let opened = path
        .as_ref()
        .and_then(|p| as_path(p.as_bytes()))
        .map(append_to);
    match opened {
        Some(Ok(file)) => return Sink::File(file),
        // May happen if the open failed, a log call ran before log_init(),
        // the directory does not exist, or the file is not writable.
        // Upstream reports `strerror(errno)` and cleared errno first, so the
        // "no path at all" case reads as a success.
        Some(Err(why)) => report_no_log_file(&why.to_string(), path),
        None => report_no_log_file("Success", path),
    }
    Sink::Stderr
}

/// The complaint `open_log_file` makes on stderr when it could not open the
/// log: an ordinary log line, written to stderr by hand because the thing
/// that would have written it is what failed.
fn report_no_log_file(reason: &str, path: Option<CString>) {
    let path = path.unwrap_or_default();
    let path = path.to_string_lossy();
    let mut sink = Sink::Stderr;
    let Some(mut line) = line_prefix(LOGLVL_ERR, None, Some(c"open_log_file"), 234) else {
        return;
    };
    // The trailing newline stands in for the eol=true of upstream's call.
    let _ = writeln!(line, "failed to open $NVIM_LOG_FILE ({reason}): {path}");
    let _ = sink.write_all(&line).and_then(|()| sink.flush());
}

/// Write one line to the log at `log_level`, tagged with
/// `context`/`func_name`/`line_num` and terminated by a newline when `eol`.
/// Answers whether the line landed.
///
/// `text` renders the payload; the prefix and it are written together, so a
/// line either lands whole or not at all.
///
/// **`text` runs only when the line is actually going to be written.** C
/// evaluated every variadic argument before `logmsg()` could decide, and the
/// macro that replaced it did the same on purpose; a closure lets the
/// arguments of a `LOGLVL_DBG` line cost nothing in a build that is not
/// logging at that level, which is what the per-message RPC trace wanted.
/// Nothing passed here has a side effect, so the only difference is the
/// cost — and the two tests that decide it, on `DID_LOG_INIT` and the
/// minimum level, come first and read one cell each.
///
/// The two tags are `'static` literals, which is the whole difference
/// between this and the `fprintf` it replaced: a log line no longer has a
/// raw pointer anywhere in it, so the call site needs no `unsafe`.
pub fn logmsg_line(
    log_level: c_int,
    context: Option<&'static CStr>,
    func_name: Option<&'static CStr>,
    line_num: c_int,
    eol: bool,
    text: impl FnOnce() -> String,
) -> bool {
    if !DID_LOG_INIT.get() {
        // set_init_1 may try logging before we are ready (#10183).
        g_stats.with_mut(|s| s.log_skip += 1);
        return false;
    }
    if log_level < g_min_log_level.get() {
        return false;
    }
    let Some(_writing) = begin_line() else {
        if !DID_RECURSION_MSG.get() {
            DID_RECURSION_MSG.set(true);
            let who = msg_cstr(func_name.or(context).unwrap_or(c""));
            msg_schedule_semsg!("E5430: {who}:{}: recursive log!", line_num);
        }
        g_stats.with_mut(|s| s.log_skip += 1);
        return false;
    };

    match line_prefix(log_level, context, func_name, line_num) {
        Some(mut line) => {
            line.extend_from_slice(&to_bytes(&text()));
            if eol {
                line.push(b'\n');
            }
            let mut sink = open_log_file();
            sink.write_all(&line).and_then(|()| sink.flush()).is_ok()
        }
        // The prefix is the head of the line; with none there is nothing to
        // append to, so the line is not written at all.
        None => false,
    }
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
/// `eol` is false where the payload carries its own terminator, so that the
/// line is not ended twice.
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

// ---------------------------------------------------------------------------
// The line prefix.

/// The three-letter tag a level prints as.
fn log_level_tag(log_level: c_int) -> &'static str {
    match log_level {
        LOGLVL_DBG => "DBG",
        LOGLVL_INF => "INF",
        LOGLVL_WRN => "WRN",
        LOGLVL_ERR => "ERR",
        // Upstream asserts the range and would index past the table; a
        // compiled-out assert leaves a wild pointer, so answer with nothing.
        _ => "",
    }
}

/// Name of the Nvim instance that produced the log, as `%-10s` in every
/// prefix. Truncated to 31 bytes exactly as upstream's `char name[32]` is.
static NAME: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// Whether the instance name has to be (re)generated: as a UI client, so
/// that "ui" is in the name; when it was never set; or when it was last
/// built from the pid because there was no `v:servername` yet.
fn name_is_stale(ui: bool) -> bool {
    ui || NAME.with(|n| n.first().is_none_or(|&b| b == b'?'))
}

/// Rebuild [`NAME`] from the parent server (`$NVIM`), `v:servername`, or the
/// pid, in that order.
fn regen_name(ui: bool) {
    let tail = |s: &[u8]| s[tail_index(s)..].to_vec();
    let parent = tail(&env_get_bounded(ENV_NVIM));
    let servername = tail(&vim_var_bytes(Vv::Servername));
    let mut name = if !parent.is_empty() {
        // "c/" = child of $NVIM.
        [if ui { &b"ui/c/"[..] } else { b"c/" }, &parent].concat()
    } else if !servername.is_empty() {
        [if ui { &b"ui/"[..] } else { b"" }, &servername].concat()
    } else {
        // `%-5ld`: the pid left-justified in five columns.
        let who = if ui { "ui" } else { "?" };
        format!("{who}.{:<5}", os_get_pid()).into_bytes()
    };
    name.truncate(NAME_SIZE);
    NAME.set(name);
}

/// `%Y-%m-%dT%H:%M:%S` of the current local time, plus the millisecond
/// fraction. `None` when the clock cannot be read at all.
fn log_timestamp() -> Option<(String, u32)> {
    let mut local_time = tm_zeroed();
    if !os_localtime(&mut local_time) {
        return None;
    }
    let date_time = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        local_time.tm_year + 1900,
        local_time.tm_mon + 1,
        local_time.tm_mday,
        local_time.tm_hour,
        local_time.tm_min,
        local_time.tm_sec,
    );
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.subsec_millis());
    Some((date_time, millis))
}

/// The date/level/name/source-location head of a log line, up to where the
/// payload starts, or `None` when the clock could not be read — which is the
/// one thing that stops a line from being written at all.
fn line_prefix(
    log_level: c_int,
    context: Option<&CStr>,
    func_name: Option<&CStr>,
    line_num: c_int,
) -> Option<Vec<u8>> {
    debug_assert!(
        (LOGLVL_DBG..=LOGLVL_ERR).contains(&log_level),
        "log_level >= LOGLVL_DBG && log_level <= LOGLVL_ERR"
    );
    let (date_time, millis) = log_timestamp()?;

    // Running as a UI client (--remote-ui).
    let ui = ui_client_channel_id.get() != 0;
    if name_is_stale(ui) {
        regen_name(ui);
    }

    // Without a source location the context stands in for it, and an absent
    // context prints as the "unknown location" marker instead of nothing.
    let located = line_num != -1 && func_name.is_some();
    let ctx: &[u8] = match context {
        Some(context) => context.to_bytes(),
        None if located => b"",
        None => b"?:",
    };

    let mut prefix = Vec::with_capacity(64);
    let tag = log_level_tag(log_level);
    let _ = write!(prefix, "{tag} {date_time}.{millis:03} ");
    NAME.with(|name| {
        // `%-10s`: padded out to ten columns, and never cut down to them.
        let padding = NAME_COLUMNS.saturating_sub(name.len());
        prefix.extend_from_slice(name);
        prefix.resize(prefix.len() + padding, b' ');
    });
    prefix.push(b' ');
    prefix.extend_from_slice(ctx);
    if let Some(func_name) = func_name.filter(|_| located) {
        prefix.extend_from_slice(func_name.to_bytes());
        let _ = write!(prefix, ":{line_num}: ");
    }
    Some(prefix)
}
