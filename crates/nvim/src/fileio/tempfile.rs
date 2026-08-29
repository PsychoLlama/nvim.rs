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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::log::logmsg_c;
use crate::{msg_schedule_semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};

use super::*;
use crate::os::fs::TEMP_FILE_PATH_MAXLEN;
use crate::types::{FAIL, MAXPATHL, OK};
use ::libc::{DIR, closedir, dirfd, opendir};

/// Candidate homes for our private directory, tried in order.
const TEMP_DIR_NAMES: [&CStr; 4] = [c"$TMPDIR", c"/tmp", c".", c"~"];

/// `flock` shared lock.
const LOCK_SH: c_int = 1;

/// Our temporary directory, always with a trailing path separator.
static VIM_TEMPDIR: GlobalCell<Option<CString>> = GlobalCell::new(None);

/// An open handle on it, holding a shared `flock` so it is not auto-cleaned.
static VIM_TEMPDIR_DP: GlobalCell<*mut DIR> = GlobalCell::new(ptr::null_mut::<DIR>());

/// `DLOG`/`WLOG`/`ELOG` from `log.h`.
///
/// The line numbers are the ones upstream's `__LINE__` produces in
/// `fileio.c`, so that moving this code does not move the log output.
macro_rules! log_at {
    ($level:expr, $func:literal, $line:literal, $fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe { logmsg_c!(
            $level,
            core::ptr::null(),
            concat!($func, "\0").as_ptr().cast::<c_char>(),
            $line,
            true,
            concat!($fmt, "\0").as_ptr().cast::<c_char>(),
            $($arg,)*
        ) }
    };
}

/// A path being assembled a piece at a time, NUL-terminated throughout.
///
/// Upstream builds these in a `char[TEMP_FILE_PATH_MAXLEN]` and asserts after
/// every append that it still fits; the bound is real, because `os_mkdtemp`
/// copies the template into a buffer of exactly that size. Overflowing it
/// panics here where upstream aborts on the assertion.
struct Template {
    buf: [u8; TEMP_FILE_PATH_MAXLEN as usize],
    len: usize,
}

impl Template {
    fn new() -> Self {
        Template {
            buf: [0; TEMP_FILE_PATH_MAXLEN as usize],
            len: 0,
        }
    }

    fn as_ptr(&self) -> *const c_char {
        self.buf.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buf.as_mut_ptr().cast()
    }

    /// Adopt `len` bytes written into the buffer by someone else.
    fn set_len(&mut self, len: usize) {
        assert!(len < self.buf.len());
        self.len = len;
    }

    fn push(&mut self, bytes: &[u8]) {
        let end = self.len + bytes.len();
        assert!(end < self.buf.len(), "temp file path too long");
        self.buf[self.len..end].copy_from_slice(bytes);
        self.buf[end] = 0;
        self.len = end;
    }

    /// Drop the last `n` bytes.
    fn shorten(&mut self, n: usize) {
        self.len -= n;
        self.buf[self.len] = 0;
    }

    fn ends_with_sep(&self) -> bool {
        self.len > 0 && self.buf[self.len - 1] == b'/'
    }
}

/// Creates a directory for private use by this instance of Nvim, trying each
/// of `TEMP_DIR_NAMES` until one succeeds.
///
/// Only done once; the same directory is used for all temp files.
unsafe fn vim_mktempdir() {
    let mut user = [0u8; 40];
    unsafe { os_get_username(user.as_mut_ptr().cast(), user.len()) };
    // Usernames may contain slashes! #19240
    let data = user.as_mut_ptr().cast::<c_void>();
    unsafe { memchrsub(data, b'/' as c_char, b'_' as c_char, user.len()) };
    unsafe { memchrsub(data, b'\\' as c_char, b'_' as c_char, user.len()) };
    let user = cstr::in_bytes(&user).to_bytes();

    // Make sure the umask doesn't remove the executable bit. "repl" has
    // been reported to use "0177".
    let umask_save = unsafe { umask(0o077) };
    for root in TEMP_DIR_NAMES {
        let mut tmp = Template::new();
        // Leave room for "/tmp/nvim.<user>/XXXXXX/999999999".
        let at = tmp.as_mut_ptr();
        let from = root.as_ptr().cast_mut();
        tmp.set_len(unsafe { expand_env(from, at, TEMP_FILE_PATH_MAXLEN - 64) });

        if !unsafe { os_isdir(tmp.as_ptr()) } {
            if root == c"$TMPDIR" {
                if !unsafe { os_env_exists(c"TMPDIR".as_ptr(), true) } {
                    log_at!(LOGLVL_DBG, "vim_mktempdir", 3323, "$TMPDIR is unset");
                } else {
                    log_at!(
                        LOGLVL_WRN,
                        "vim_mktempdir",
                        3325,
                        "$TMPDIR tempdir not a directory (or does not exist): \"%s\"",
                        tmp.as_ptr(),
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
        tmp.push(user);
        unsafe { os_mkdir(tmp.as_ptr(), 0o700) }; // Always create, to avoid a race.
        let owned = unsafe { os_file_owned(tmp.as_ptr()) };
        let isdir = unsafe { os_isdir(tmp.as_ptr()) };
        // XDG_RUNTIME_DIR must be owned by the user, mode 0700.
        let perm = unsafe { os_getperm(tmp.as_ptr()) } as c_int;
        if isdir && owned && perm & 0o777 == 0o700 {
            if !tmp.ends_with_sep() {
                tmp.push(b"/");
            }
        } else {
            if !owned {
                log_at!(
                    LOGLVL_ERR,
                    "vim_mktempdir",
                    3355,
                    "tempdir root not owned by current user (%s): %s",
                    user.as_ptr(),
                    tmp.as_ptr(),
                );
            } else if !isdir {
                log_at!(
                    LOGLVL_ERR,
                    "vim_mktempdir",
                    3357,
                    "tempdir root not a directory: %s",
                    tmp.as_ptr(),
                );
            }
            if perm & 0o777 != 0o700 {
                log_at!(
                    LOGLVL_ERR,
                    "vim_mktempdir",
                    3361,
                    "tempdir root has invalid permissions (%o): %s",
                    perm,
                    tmp.as_ptr(),
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
        let mut path = Template::new();
        let r = unsafe { os_mkdtemp(tmp.as_ptr(), path.as_mut_ptr()) };
        if r != 0 {
            log_at!(
                LOGLVL_WRN,
                "vim_mktempdir",
                3377,
                "tempdir create failed: %s: %s",
                uv_strerror(r),
                tmp.as_ptr(),
            );
            continue;
        }

        if unsafe { vim_settempdir(path.as_ptr()) } {
            // Successfully created and set the temporary directory, so
            // stop trying.
            break;
        }
        // Couldn't set the temp dir to `path`, so remove what we made.
        unsafe { os_rmdir(path.as_ptr()) };
    }
    unsafe { umask(umask_save) };
}

/// Core part of the `readdir()` function: list `path` into `gap`.
///
/// `checkitem` filters the entries; it returns zero to skip one and a
/// negative number to stop the walk.
///
/// @return  OK for success, FAIL for failure.
pub unsafe fn readdir_core(
    gap: *mut garray_T,
    path: *const c_char,
    context: *mut c_void,
    checkitem: CheckItem,
) -> c_int {
    unsafe { ga_init(gap, size_of::<*mut c_char>() as c_int, 20) };

    let mut dir = Directory::default();
    if !unsafe { os_scandir(&raw mut dir, path) } {
        unsafe { smsg_c!(0, gettext(e_notopen).as_ptr(), path) };
        return FAIL;
    }

    loop {
        let p = unsafe { os_scandir_next(&raw mut dir) };
        if p.is_null() {
            break;
        }

        let name = unsafe { CStr::from_ptr(p) }.to_bytes();
        let mut ignore = name == b"." || name == b"..";
        if !ignore && let Some(check) = checkitem {
            let r = unsafe { check(context, p) };
            if r < 0 {
                break;
            }
            ignore = r == 0;
        }

        if !ignore {
            unsafe { ga_grow(gap, 1) };
            let at = unsafe { (*gap).ga_len };
            unsafe { (*gap).ga_len += 1 };
            unsafe { *((*gap).ga_data as *mut *mut c_char).add(at as usize) = xstrdup(p) };
        }
    }

    unsafe { os_closedir(&raw mut dir) };

    if unsafe { (*gap).ga_len } > 0 {
        unsafe { sort_strings((*gap).ga_data as *mut *mut c_char, (*gap).ga_len) };
    }

    OK
}

/// Delete `name` and everything in it, recursively.
///
/// @return  0 for success, -1 if some file was not deleted.
pub unsafe fn delete_recursive(name: *const c_char) -> c_int {
    unsafe { delete_tree(CStr::from_ptr(name).to_bytes()) }
}

/// [`delete_recursive`] on a path that has not been through C yet.
///
/// Upstream assembles the child paths in the shared `NameBuff`, which works
/// only because each level of the recursion rewrites the prefix it shares
/// with its caller. This carries its own buffer instead, which also lifts
/// the `MAXPATHL` limit on how deep a tree can be deleted.
unsafe fn delete_tree(name: &[u8]) -> c_int {
    let path = CString::new(name).unwrap_or_default();
    if !unsafe { os_isrealdir(path.as_ptr()) } {
        // Delete symlink only.
        return if unsafe { os_remove(path.as_ptr()) } == 0 {
            0
        } else {
            -1
        };
    }

    let mut ga = garray_T::default();
    if unsafe { readdir_core(&raw mut ga, path.as_ptr(), ptr::null_mut(), None) } != OK {
        return -1;
    }

    let mut result = 0;
    let mut child = name.to_vec();
    child.push(b'/');
    let stem = child.len();
    for at in 0..ga.ga_len as usize {
        child.truncate(stem);
        let entry = unsafe { *(ga.ga_data as *mut *mut c_char).add(at) };
        child.extend_from_slice(unsafe { CStr::from_ptr(entry) }.to_bytes());
        if unsafe { delete_tree(&child) } != 0 {
            // Remember the failure but continue deleting any further
            // entries.
            result = -1;
        }
    }
    unsafe { ga_clear_strings(&raw mut ga) };
    if unsafe { os_rmdir(path.as_ptr()) } != 0 {
        result = -1;
    }
    result
}

/// Open the temporary directory and take a file lock, so that it is not
/// auto-cleaned while we are using it.
unsafe fn vim_opentempdir() {
    if !VIM_TEMPDIR_DP.get().is_null() {
        return;
    }
    let dp = VIM_TEMPDIR.with(|dir| match dir {
        Some(dir) => unsafe { opendir(dir.as_ptr()) },
        None => ptr::null_mut(),
    });
    if dp.is_null() {
        return;
    }
    VIM_TEMPDIR_DP.set(dp);
    unsafe { flock(dirfd(dp), LOCK_SH) };
}

/// Close the temporary directory, which releases the file lock.
unsafe fn vim_closetempdir() {
    let dp = VIM_TEMPDIR_DP.get();
    if !dp.is_null() {
        unsafe { closedir(dp) };
        VIM_TEMPDIR_DP.set(ptr::null_mut());
    }
}

/// Delete the temp directory and all files it contains.
pub unsafe fn vim_deltempdir() {
    let Some(dir) = VIM_TEMPDIR.with_mut(|dir| dir.take()) else {
        return;
    };
    unsafe { vim_closetempdir() };
    // Remove the trailing path separator, which is always there.
    let dir = dir.to_bytes();
    unsafe { delete_tree(dir.strip_suffix(b"/").unwrap_or(dir)) };
}

/// Gets the path to Nvim's own temp dir, ending with a slash.
///
/// Creates the directory on the first call.
pub unsafe fn vim_gettempdir() -> *mut c_char {
    static NOTFOUND: GlobalCell<c_int> = GlobalCell::new(0);
    let usable = VIM_TEMPDIR.with(|dir| {
        dir.as_ref()
            .is_some_and(|dir| unsafe { os_isdir(dir.as_ptr()) })
    });
    if !usable {
        if let Some(gone) = VIM_TEMPDIR.with_mut(|dir| dir.take()) {
            let notfound = NOTFOUND.get() + 1;
            NOTFOUND.set(notfound);
            if notfound == 1 {
                log_at!(
                    LOGLVL_ERR,
                    "vim_gettempdir",
                    3534,
                    "tempdir disappeared (antivirus or broken cleanup job?): %s",
                    gone.as_ptr(),
                );
            }
            if notfound > 1 {
                let fmt = c"E5431: tempdir disappeared (%d times)".as_ptr();
                unsafe { msg_schedule_semsg_c!(fmt, notfound) };
            }
        }
        unsafe { vim_mktempdir() };
    }
    VIM_TEMPDIR.with(|dir| match dir {
        Some(dir) => dir.as_ptr().cast_mut(),
        None => ptr::null_mut(),
    })
}

/// Sets Nvim's own temporary directory name to `tempdir`, which must already
/// exist. The name is expanded to a full path first, so that a later `:cd`
/// cannot confuse us, and a trailing path separator is added.
///
/// @return  false if we run out of memory.
unsafe fn vim_settempdir(tempdir: *const c_char) -> bool {
    // Not `xmalloc`: running out of memory here is survivable, we just
    // fall through to the next candidate directory.
    let buf = unsafe { verbose_try_malloc(MAXPATHL as usize + 2) }.cast::<c_char>();
    if buf.is_null() {
        return false;
    }
    unsafe { vim_full_name(tempdir, buf, MAXPATHL as size_t, false) };
    let mut full = unsafe { CStr::from_ptr(buf) }.to_bytes().to_vec();
    unsafe { xfree(buf.cast()) };

    if !full.ends_with(b"/") {
        full.push(b'/');
    }
    VIM_TEMPDIR.set(CString::new(full).ok());
    unsafe { vim_opentempdir() };
    true
}

/// Return a unique name that can be used for a temp file.
///
/// The file is NOT created. There is no need to check whether it already
/// exists, because we own the directory and nobody else creates files in it.
///
/// @return  the name, or NULL if Nvim can't create its temporary directory.
pub unsafe fn vim_tempname() -> *mut c_char {
    /// Temp filename counter.
    static TEMP_COUNT: GlobalCell<u64> = GlobalCell::new(0);
    let tempdir = unsafe { vim_gettempdir() };
    if tempdir.is_null() {
        return ptr::null_mut();
    }
    let count = TEMP_COUNT.get();
    TEMP_COUNT.set(count.wrapping_add(1));
    // Upstream formats this into a `char[TEMP_FILE_PATH_MAXLEN]` and then
    // copies out the length `snprintf` *would* have needed, which reads
    // off the end of the buffer once the temp dir gets long enough.
    let mut name = unsafe { CStr::from_ptr(tempdir) }.to_bytes().to_vec();
    name.extend_from_slice(count.to_string().as_bytes());
    unsafe { xmemdupz(name.as_ptr().cast(), name.len()) }.cast()
}
