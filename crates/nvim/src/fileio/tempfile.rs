//! The temporary directory, and the directory walks it needs.
//!
//! Nvim makes one private directory per process, under `$TMPDIR` or one of a
//! handful of fallbacks, and hands out names inside it; [`vim_mktempdir`]
//! picks the spot and [`vim_tempname`] numbers the files. Doing it that way
//! means the "does this name already exist?" question only has to be asked
//! once, when the directory is made, which is both faster and proof against
//! symlink attacks. [`vim_opentempdir`] keeps an open handle on the
//! directory so a `/tmp` cleaner cannot pull it out from under us, and
//! [`vim_deltempdir`] removes it at exit — which is what [`delete_recursive`]
//! and [`readdir_core`] are for, though `readdir()`/`delete()` in Vimscript
//! reach them too.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::msg_schedule_semsg;
use crate::os::env::{env_exists, expand_env_into};
use crate::os::fs::{dir_exists, getperm, os_mkdtemp, os_strerror, set_umask};
use crate::os::users::username_within;
use crate::smsg;
use crate::types::VarNumber;
use core::ffi::c_int;
use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;

use super::*;
use crate::os::fs::TEMP_FILE_PATH_MAXLEN;
use crate::types::{Failed, MAXPATHL};

/// Candidate homes for our private directory, tried in order.
const TEMP_DIR_NAMES: [&CStr; 4] = [c"$TMPDIR", c"/tmp", c".", c"~"];

/// Our temporary directory, always with a trailing path separator.
static VIM_TEMPDIR: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// An open handle on it, holding a shared `flock` so it is not auto-cleaned.
/// Closing the handle (dropping it) releases the lock.
static VIM_TEMPDIR_DP: GlobalCell<Option<File>> = GlobalCell::new(None);

/// `DLOG`/`WLOG`/`ELOG` from `log.h`.
///
/// The line numbers are the ones upstream's `__LINE__` produces in
/// `fileio.c`, so that moving this code does not move the log output.
macro_rules! log_at {
    ($level:expr, $line:literal, $($fmt:tt)*) => {
        $crate::logmsg!($level, c"vim_mktempdir", $line, $($fmt)*)
    };
}

/// A path being assembled a piece at a time.
///
/// Upstream builds these in a `char[TEMP_FILE_PATH_MAXLEN]` and asserts after
/// every append that it still fits; the bound is real, because `os_mkdtemp`
/// copied the template into a buffer of exactly that size. Overflowing it
/// panics here where upstream aborts on the assertion.
struct Template(Vec<u8>);

impl Template {
    fn new(start: &[u8]) -> Self {
        let mut tmp = Template(Vec::with_capacity(TEMP_FILE_PATH_MAXLEN as usize));
        tmp.push(start);
        tmp
    }

    /// The path as a string.
    fn as_cstr(&self) -> CString {
        CString::new(self.0.clone()).unwrap_or_default()
    }

    fn push(&mut self, bytes: &[u8]) {
        assert!(
            self.0.len() + bytes.len() < TEMP_FILE_PATH_MAXLEN as usize,
            "temp file path too long"
        );
        self.0.extend_from_slice(bytes);
    }

    /// Drop the last `n` bytes.
    fn shorten(&mut self, n: usize) {
        self.0.truncate(self.0.len() - n);
    }

    fn ends_with_sep(&self) -> bool {
        self.0.last() == Some(&b'/')
    }
}

/// Creates a directory for private use by this instance of Nvim, trying each
/// of `TEMP_DIR_NAMES` until one succeeds.
///
/// Only done once; the same directory is used for all temp files.
fn vim_mktempdir() {
    // Usernames may contain slashes! #19240
    let user: Vec<u8> = username_within(40)
        .into_iter()
        .map(|b| if b == b'/' || b == b'\\' { b'_' } else { b })
        .collect();

    // Make sure the umask doesn't remove the executable bit. "repl" has
    // been reported to use "0177".
    let umask_save = set_umask(0o077);
    for root in TEMP_DIR_NAMES {
        // Leave room for "/tmp/nvim.<user>/XXXXXX/999999999".
        let room = TEMP_FILE_PATH_MAXLEN as usize - 63;
        let mut tmp = Template::new(&expand_env_into(root, room));

        if !dir_exists(&tmp.as_cstr()) {
            if root == c"$TMPDIR" {
                if !env_exists(c"TMPDIR", true) {
                    log_at!(LOGLVL_DBG, 3323, "$TMPDIR is unset");
                } else {
                    let path = tmp.as_cstr();
                    let at = msg_cstr(&path);
                    log_at!(
                        LOGLVL_WRN,
                        3325,
                        "$TMPDIR tempdir not a directory (or does not exist): \"{at}\""
                    );
                }
            }
            continue;
        }

        // "/tmp/" exists, now try to create "/tmp/nvim.<user>/".
        if !tmp.ends_with_sep() {
            tmp.push(b"/");
        }
        tmp.push(b"nvim.");
        tmp.push(&user);
        let path = tmp.as_cstr();
        os_mkdir(&path, 0o700); // Always create, to avoid a race.
        let owned = os_file_owned(&path);
        let isdir = dir_exists(&path);
        // XDG_RUNTIME_DIR must be owned by the user, mode 0700.
        let perm = getperm(&path);
        if isdir && owned && perm & 0o777 == 0o700 {
            if !tmp.ends_with_sep() {
                tmp.push(b"/");
            }
        } else {
            let at = msg_cstr(&path);
            if !owned {
                let who = msg_bytes(&user);
                log_at!(
                    LOGLVL_ERR,
                    3355,
                    "tempdir root not owned by current user ({who}): {at}"
                );
            } else if !isdir {
                log_at!(LOGLVL_ERR, 3357, "tempdir root not a directory: {at}");
            }
            if perm & 0o777 != 0o700 {
                log_at!(
                    LOGLVL_ERR,
                    3361,
                    "tempdir root has invalid permissions ({perm:o}): {at}"
                );
            }
            // If our "root" tempdir is invalid or fails, proceed without
            // "<user>/". Else user1 could break user2 by creating
            // "/tmp/nvim.user2/".
            tmp.shorten(user.len());
        }

        // Now try to create "/tmp/nvim.<user>/XXXXXX". "XXXXXX" is the
        // mkdtemp template, replaced with random alphanumeric characters.
        tmp.push(b"XXXXXX");
        let made = match os_mkdtemp(&tmp.as_cstr()) {
            Ok(made) => made,
            Err(r) => {
                let why = msg_cstr(os_strerror(r));
                let path = tmp.as_cstr();
                let at = msg_cstr(&path);
                log_at!(LOGLVL_WRN, 3377, "tempdir create failed: {why}: {at}");
                continue;
            }
        };

        if vim_settempdir(&made) {
            // Successfully created and set the temporary directory, so
            // stop trying.
            break;
        }
        // Couldn't set the temp dir to `made`, so remove what we made.
        os_rmdir(&made);
    }
    set_umask(umask_save);
}

/// Core part of the `readdir()` function: the names in `path`, sorted.
///
/// `checkitem` filters the entries; it returns zero to skip one and a
/// negative number to stop the walk.
///
/// @return  `Ok` for success, `Err` for failure.
pub fn readdir_core(
    path: &CStr,
    mut checkitem: Option<&mut dyn FnMut(&CStr) -> VarNumber>,
) -> Result<Vec<CString>, Failed> {
    let mut names = Vec::new();

    let mut dir = Directory::default();
    if !os_scandir(&mut dir, path) {
        let path = msg_cstr(path);
        smsg!(0, "E484: Can't open file {path}");
        return Err(Failed);
    }

    // Copied before the next step of the walk.
    let next = |dir: &mut Directory| os_scandir_next(dir).map(CString::from);
    while let Some(name) = next(&mut dir) {
        let bytes = name.as_bytes();
        let mut ignore = bytes == b"." || bytes == b"..";
        if !ignore && let Some(check) = checkitem.as_mut() {
            let r = check(&name);
            if r < 0 {
                break;
            }
            ignore = r == 0;
        }

        if !ignore {
            names.push(name);
        }
    }

    os_closedir(&mut dir);

    // Byte order, as `sort_strings`' `strcmp` gave.
    names.sort();
    Ok(names)
}

/// Delete `name` and everything in it, recursively.
///
/// @return  0 for success, -1 if some file was not deleted.
pub fn delete_recursive(name: &CStr) -> c_int {
    delete_tree(name.to_bytes())
}

/// [`delete_recursive`] on a path that has not been through C yet.
///
/// Upstream assembles the child paths in the shared `NameBuff`, which works
/// only because each level of the recursion rewrites the prefix it shares
/// with its caller. This carries its own buffer instead, which also lifts
/// the `MAXPATHL` limit on how deep a tree can be deleted.
fn delete_tree(name: &[u8]) -> c_int {
    let path = CString::new(name).unwrap_or_default();
    if !os_isrealdir(&path) {
        // Delete symlink only.
        return if os_remove(&path) == 0 { 0 } else { -1 };
    }

    let Ok(names) = readdir_core(&path, None) else {
        return -1;
    };

    let mut result = 0;
    let mut child = name.to_vec();
    child.push(b'/');
    let stem = child.len();
    for entry in &names {
        child.truncate(stem);
        child.extend_from_slice(entry.to_bytes());
        if delete_tree(&child) != 0 {
            // Remember the failure but continue deleting any further
            // entries.
            result = -1;
        }
    }
    if os_rmdir(&path) != 0 {
        result = -1;
    }
    result
}

/// Open the temporary directory and take a file lock, so that it is not
/// auto-cleaned while we are using it.
fn vim_opentempdir() {
    if VIM_TEMPDIR_DP.with(Option::is_some) {
        return;
    }
    let dir = VIM_TEMPDIR.with(|dir| {
        let path = OsStr::from_bytes(dir.as_ref()?.to_bytes());
        File::open(path).ok()
    });
    let Some(dir) = dir else {
        return;
    };
    // Upstream ignores a failed `flock` too: the lock only defends against
    // a cleaner, and the directory is usable without it.
    let _ = dir.lock_shared();
    VIM_TEMPDIR_DP.set(Some(dir));
}

/// Close the temporary directory, which releases the file lock.
fn vim_closetempdir() {
    drop(VIM_TEMPDIR_DP.take());
}

/// Delete the temp directory and all files it contains.
pub fn vim_deltempdir() {
    let Some(dir) = VIM_TEMPDIR.with_mut(|dir| dir.take()) else {
        return;
    };
    vim_closetempdir();
    // Remove the trailing path separator, which is always there.
    let dir = dir.to_bytes();
    delete_tree(dir.strip_suffix(b"/").unwrap_or(dir));
}

/// Gets the path to Nvim's own temp dir, ending with a slash.
///
/// Creates the directory on the first call. `None` when it cannot be made.
pub fn vim_gettempdir() -> Option<CString> {
    static NOTFOUND: GlobalCell<c_int> = GlobalCell::new(0);
    let usable = VIM_TEMPDIR.with(|dir| dir.as_deref().is_some_and(dir_exists));
    if !usable {
        if let Some(gone) = VIM_TEMPDIR.with_mut(|dir| dir.take()) {
            let notfound = NOTFOUND.get() + 1;
            NOTFOUND.set(notfound);
            if notfound == 1 {
                let gone = msg_cstr(&gone);
                crate::logmsg!(
                    LOGLVL_ERR,
                    c"vim_gettempdir",
                    3534,
                    "tempdir disappeared (antivirus or broken cleanup job?): {gone}"
                );
            }
            if notfound > 1 {
                msg_schedule_semsg!("E5431: tempdir disappeared ({} times)", notfound);
            }
        }
        vim_mktempdir();
    }
    VIM_TEMPDIR.with(Clone::clone)
}

/// Sets Nvim's own temporary directory name to `tempdir`, which must already
/// exist. The name is expanded to a full path first, so that a later `:cd`
/// cannot confuse us, and a trailing path separator is added.
///
/// @return  false if we run out of memory.
fn vim_settempdir(tempdir: &CStr) -> bool {
    let mut full = vim_full_name(tempdir, MAXPATHL as usize, false).unwrap_or_else(|name| name);

    if !full.ends_with(b"/") {
        full.push(b'/');
    }
    VIM_TEMPDIR.set(CString::new(full).ok());
    vim_opentempdir();
    true
}

/// A unique name that can be used for a temp file: `None` when Nvim can't
/// create its temporary directory.
///
/// The file is NOT created. There is no need to check whether it already
/// exists, because we own the directory and nobody else creates files in it.
pub fn temp_name() -> Option<XString> {
    /// Temp filename counter.
    static TEMP_COUNT: GlobalCell<u64> = GlobalCell::new(0);
    let tempdir = vim_gettempdir()?;
    let count = TEMP_COUNT.get();
    TEMP_COUNT.set(count.wrapping_add(1));
    // Upstream formats this into a `char[TEMP_FILE_PATH_MAXLEN]` and then
    // copies out the length `snprintf` *would* have needed, which reads
    // off the end of the buffer once the temp dir gets long enough.
    let mut name = tempdir.into_bytes();
    name.extend_from_slice(count.to_string().as_bytes());
    Some(XString::from_vec(name))
}
