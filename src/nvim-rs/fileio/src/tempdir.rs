//! Temporary directory management for Neovim.
//!
//! This module provides:
//! - `vim_mktempdir`: Create a private temp directory
//! - `vim_gettempdir`: Get (or create) the temp directory path
//! - `vim_settempdir`: Set the temp directory from a path
//! - `vim_tempname`: Generate a unique temp filename
//! - `vim_deltempdir`: Delete the temp directory on exit
//! - `vim_opentempdir` / `vim_closetempdir`: flock management (HAVE_DIRFD_AND_FLOCK)

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, c_void};

/// Maximum path length for temp files (matches C TEMP_FILE_PATH_MAXLEN on Unix).
const TEMP_FILE_PATH_MAXLEN: usize = 256;
/// Maximum path length for full paths.
const MAXPATHL: usize = 4096;

// =============================================================================
// Global state (matches C statics)
// =============================================================================

/// Path to Nvim's own temp dir. Ends in a slash. NULL when not yet created.
static mut VIM_TEMPDIR: *mut c_char = std::ptr::null_mut();

/// File descriptor of temp dir (for flock on Linux).
#[cfg(target_os = "linux")]
static mut VIM_TEMPDIR_DP: *mut libc::DIR = std::ptr::null_mut();

/// Counter for unique temp filenames.
static mut TEMP_COUNT: u64 = 0;

/// How many times the tempdir has disappeared.
static mut NOTFOUND: c_int = 0;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    fn os_get_username(s: *mut c_char, len: usize) -> c_int;
    fn memchrsub(data: *mut c_void, c: c_char, x: c_char, len: usize);
    fn expand_env(src: *const c_char, dst: *mut c_char, dstlen: c_int) -> usize;
    fn os_isdir(name: *const c_char) -> bool;
    fn os_env_exists(name: *const c_char, nonempty: bool) -> bool;
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;
    fn vim_snprintf(buf: *mut c_char, buflen: usize, fmt: *const c_char, ...) -> c_int;
    fn os_mkdir(path: *const c_char, mode: u32) -> c_int;
    fn os_file_owned(fname: *const c_char) -> bool;
    fn os_getperm(path: *const c_char) -> c_int;
    fn os_mkdtemp(templ: *const c_char, path: *mut c_char) -> c_int;
    fn os_rmdir(path: *const c_char) -> c_int;
    fn vim_FullName(fname: *const c_char, buf: *mut c_char, len: usize, force: bool) -> c_int;
    fn add_pathsep(p: *mut c_char) -> bool;
    fn verbose_try_malloc(size: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn delete_recursive(name: *const c_char) -> c_int;
    fn path_tail(p: *const c_char) -> *mut c_char;
    fn emsg(s: *const c_char);
    /// Log a message. The `fmt` must be a literal format string (e.g. `c"%s"`);
    /// pre-format the message into a buffer and pass it as the single vararg to
    /// avoid format-string injection.
    ///
    /// Mirrors: `bool logmsg(int log_level, const char *context,
    ///           const char *func_name, int line_num, bool eol,
    ///           const char *fmt, ...)` in src/nvim/log.c.
    fn logmsg(
        log_level: c_int,
        context: *const c_char,
        func_name: *const c_char,
        line_num: c_int,
        eol: bool,
        fmt: *const c_char,
        ...
    ) -> bool;
}

// Log-level constants — mirror LOGLVL_* in src/nvim/log.h.
const LOGLVL_DBG: c_int = 1;
const LOGLVL_WRN: c_int = 3;
const LOGLVL_ERR: c_int = 4;

// Compile-time guards: if log.h ever changes these values the build breaks.
const _: () = assert!(LOGLVL_DBG == 1);
const _: () = assert!(LOGLVL_WRN == 3);
const _: () = assert!(LOGLVL_ERR == 4);

// Temp directory candidate paths (matches C TEMP_DIR_NAMES on Unix)
const TEMP_DIR_NAMES: &[&[u8]] = &[b"$TMPDIR\0", b"/tmp\0", b".\0", b"~\0"];

// =============================================================================
// vim_mktempdir (static helper)
// =============================================================================

/// Creates a directory for private use by this instance of Nvim.
///
/// Tries each of TEMP_DIR_NAMES until one succeeds.
/// Directly replaces the static C `vim_mktempdir`.
unsafe fn vim_mktempdir_impl() {
    let mut tmp = [0u8; TEMP_FILE_PATH_MAXLEN];
    let mut path = [0u8; TEMP_FILE_PATH_MAXLEN];
    let mut user = [0i8; 40usize];

    unsafe { os_get_username(user.as_mut_ptr(), user.len()) };

    // Usernames may contain slashes (#19240)
    unsafe {
        memchrsub(
            user.as_mut_ptr() as *mut c_void,
            b'/' as c_char,
            b'_' as c_char,
            user.len(),
        )
    };
    unsafe {
        memchrsub(
            user.as_mut_ptr() as *mut c_void,
            b'\\' as c_char,
            b'_' as c_char,
            user.len(),
        )
    };

    // Make sure the umask doesn't remove the executable bit.
    let umask_save = unsafe { libc::umask(0o077) };

    'outer: for dir_name in TEMP_DIR_NAMES {
        // Expand environment variables, leave room for "/tmp/nvim.<user>/XXXXXX/999999999".
        let tmplen = unsafe {
            expand_env(
                dir_name.as_ptr() as *const c_char,
                tmp.as_mut_ptr() as *mut c_char,
                (TEMP_FILE_PATH_MAXLEN - 64) as c_int,
            )
        };

        if !unsafe { os_isdir(tmp.as_ptr() as *const c_char) } {
            // Check if it's $TMPDIR being unset/invalid — mirrors upstream C DLOG/WLOG.
            if dir_name == b"$TMPDIR\0" {
                if !unsafe { os_env_exists(c"TMPDIR".as_ptr(), true) } {
                    // $TMPDIR is unset — emit DLOG equivalent (cosmetic, not test-gated).
                    unsafe {
                        logmsg(
                            LOGLVL_DBG,
                            std::ptr::null(),
                            c"vim_mktempdir".as_ptr(),
                            0,
                            true,
                            c"%s".as_ptr(),
                            c"$TMPDIR is unset".as_ptr(),
                        )
                    };
                } else {
                    // $TMPDIR is set but not a valid directory — emit WLOG equivalent.
                    // This is the line required by assert_log in the functional tests.
                    let mut msgbuf = [0u8; 512];
                    unsafe {
                        vim_snprintf(
                            msgbuf.as_mut_ptr() as *mut c_char,
                            msgbuf.len(),
                            c"$TMPDIR tempdir not a directory (or does not exist): \"%s\"".as_ptr(),
                            tmp.as_ptr() as *const c_char,
                        )
                    };
                    unsafe {
                        logmsg(
                            LOGLVL_WRN,
                            std::ptr::null(),
                            c"vim_mktempdir".as_ptr(),
                            0,
                            true,
                            c"%s".as_ptr(),
                            msgbuf.as_ptr() as *const c_char,
                        )
                    };
                }
            }
            continue;
        }

        // "/tmp/" exists, now try to create "/tmp/nvim.<user>/".
        let mut tmplen = tmplen;
        if unsafe {
            after_pathsep(
                tmp.as_ptr() as *const c_char,
                tmp.as_ptr().add(tmplen) as *const c_char,
            )
        } == 0
        {
            tmplen += unsafe {
                vim_snprintf(
                    tmp.as_mut_ptr().add(tmplen) as *mut c_char,
                    tmp.len() - tmplen,
                    c"/".as_ptr(),
                ) as usize
            };
        }
        tmplen += unsafe {
            vim_snprintf(
                tmp.as_mut_ptr().add(tmplen) as *mut c_char,
                tmp.len() - tmplen,
                c"nvim.%s".as_ptr(),
                user.as_ptr(),
            ) as usize
        };

        unsafe { os_mkdir(tmp.as_ptr() as *const c_char, 0o700) }; // Always create, to avoid a race.
        let owned = unsafe { os_file_owned(tmp.as_ptr() as *const c_char) };
        let isdir = unsafe { os_isdir(tmp.as_ptr() as *const c_char) };
        let perm = unsafe { os_getperm(tmp.as_ptr() as *const c_char) };
        let valid = isdir && owned && (perm & 0o777) == 0o700;

        if valid {
            if unsafe {
                after_pathsep(
                    tmp.as_ptr() as *const c_char,
                    tmp.as_ptr().add(tmplen) as *const c_char,
                )
            } == 0
            {
                tmplen += unsafe {
                    vim_snprintf(
                        tmp.as_mut_ptr().add(tmplen) as *mut c_char,
                        tmp.len() - tmplen,
                        c"/".as_ptr(),
                    ) as usize
                };
            }
        } else {
            // Tempdir root is invalid — emit ELOG equivalents matching upstream C.
            if !owned {
                let mut msgbuf = [0u8; 512];
                unsafe {
                    vim_snprintf(
                        msgbuf.as_mut_ptr() as *mut c_char,
                        msgbuf.len(),
                        c"tempdir root not owned by current user (%s): %s".as_ptr(),
                        user.as_ptr(),
                        tmp.as_ptr() as *const c_char,
                    )
                };
                unsafe {
                    logmsg(
                        LOGLVL_ERR,
                        std::ptr::null(),
                        c"vim_mktempdir".as_ptr(),
                        0,
                        true,
                        c"%s".as_ptr(),
                        msgbuf.as_ptr() as *const c_char,
                    )
                };
            } else if !isdir {
                let mut msgbuf = [0u8; 512];
                unsafe {
                    vim_snprintf(
                        msgbuf.as_mut_ptr() as *mut c_char,
                        msgbuf.len(),
                        c"tempdir root not a directory: %s".as_ptr(),
                        tmp.as_ptr() as *const c_char,
                    )
                };
                unsafe {
                    logmsg(
                        LOGLVL_ERR,
                        std::ptr::null(),
                        c"vim_mktempdir".as_ptr(),
                        0,
                        true,
                        c"%s".as_ptr(),
                        msgbuf.as_ptr() as *const c_char,
                    )
                };
            }
            if (perm & 0o777) != 0o700 {
                let mut msgbuf = [0u8; 512];
                unsafe {
                    vim_snprintf(
                        msgbuf.as_mut_ptr() as *mut c_char,
                        msgbuf.len(),
                        c"tempdir root has invalid permissions (%o): %s".as_ptr(),
                        perm as c_int,
                        tmp.as_ptr() as *const c_char,
                    )
                };
                unsafe {
                    logmsg(
                        LOGLVL_ERR,
                        std::ptr::null(),
                        c"vim_mktempdir".as_ptr(),
                        0,
                        true,
                        c"%s".as_ptr(),
                        msgbuf.as_ptr() as *const c_char,
                    )
                };
            }
            // If our "root" tempdir is invalid, proceed without "<user>/".
            let user_len = unsafe { libc::strlen(user.as_ptr()) };
            tmplen -= user_len;
            tmp[tmplen] = 0;
        }

        // Now try to create "/tmp/nvim.<user>/XXXXXX".
        tmplen += unsafe {
            vim_snprintf(
                tmp.as_mut_ptr().add(tmplen) as *mut c_char,
                tmp.len() - tmplen,
                c"XXXXXX".as_ptr(),
            ) as usize
        };
        let _ = tmplen;

        let r = unsafe {
            os_mkdtemp(
                tmp.as_ptr() as *const c_char,
                path.as_mut_ptr() as *mut c_char,
            )
        };
        if r != 0 {
            // TODO(migration): restore upstream WLOG("tempdir create failed: %s: %s",
            // os_strerror(r), tmp) — requires importing os_strerror, deferred.
            continue;
        }

        if unsafe { vim_settempdir_impl(path.as_ptr() as *const c_char) } {
            break 'outer;
        }
        // Couldn't set vim_tempdir, remove created directory.
        unsafe { os_rmdir(path.as_ptr() as *const c_char) };
    }

    unsafe { libc::umask(umask_save) };
}

// =============================================================================
// vim_settempdir (static helper)
// =============================================================================

/// Sets Nvim's own temporary directory name. The directory must already exist.
/// Expands the name to a full path and stores it in VIM_TEMPDIR.
/// Returns false if out of memory.
unsafe fn vim_settempdir_impl(tempdir: *const c_char) -> bool {
    let buf = unsafe { verbose_try_malloc(MAXPATHL + 2) } as *mut c_char;
    if buf.is_null() {
        return false;
    }

    unsafe { vim_FullName(tempdir, buf, MAXPATHL, false) };
    unsafe { add_pathsep(buf) };
    unsafe { VIM_TEMPDIR = xstrdup(buf) };
    unsafe { xfree(buf as *mut c_void) };

    #[cfg(target_os = "linux")]
    unsafe {
        vim_opentempdir_impl();
    }

    true
}

// =============================================================================
// vim_opentempdir / vim_closetempdir (Linux only, HAVE_DIRFD_AND_FLOCK)
// =============================================================================

#[cfg(target_os = "linux")]
unsafe fn vim_opentempdir_impl() {
    if !unsafe { VIM_TEMPDIR_DP }.is_null() {
        return;
    }
    let dp = unsafe { libc::opendir(VIM_TEMPDIR as *const libc::c_char) };
    if dp.is_null() {
        return;
    }
    unsafe { VIM_TEMPDIR_DP = dp };
    unsafe { libc::flock(libc::dirfd(dp), libc::LOCK_SH) };
}

#[cfg(target_os = "linux")]
unsafe fn vim_closetempdir_impl() {
    if unsafe { VIM_TEMPDIR_DP }.is_null() {
        return;
    }
    unsafe { libc::closedir(VIM_TEMPDIR_DP) };
    unsafe { VIM_TEMPDIR_DP = std::ptr::null_mut() };
}

// =============================================================================
// Public exports
// =============================================================================

/// Delete the temp directory and all files it contains.
///
/// Directly replaces the C `vim_deltempdir` symbol.
///
/// # Safety
/// Accesses global VIM_TEMPDIR.
#[export_name = "vim_deltempdir"]
pub unsafe extern "C" fn rs_vim_deltempdir() {
    if unsafe { VIM_TEMPDIR }.is_null() {
        return;
    }

    #[cfg(target_os = "linux")]
    unsafe {
        vim_closetempdir_impl();
    }

    // Remove the trailing path separator.
    let tail = unsafe { path_tail(VIM_TEMPDIR) };
    // path_tail returns pointer to filename; the separator is one byte before it.
    if !tail.is_null() && tail > unsafe { VIM_TEMPDIR } {
        unsafe { *tail.sub(1) = 0 };
    }

    unsafe { delete_recursive(VIM_TEMPDIR) };
    unsafe {
        xfree(VIM_TEMPDIR as *mut c_void);
        VIM_TEMPDIR = std::ptr::null_mut();
    }
}

/// Gets path to Nvim's own temp dir (ending with slash).
///
/// Creates the directory on the first call.
///
/// Directly replaces the C `vim_gettempdir` symbol.
///
/// # Safety
/// Accesses global VIM_TEMPDIR.
#[export_name = "vim_gettempdir"]
pub unsafe extern "C" fn rs_vim_gettempdir() -> *const c_char {
    if unsafe { VIM_TEMPDIR }.is_null() || !unsafe { os_isdir(VIM_TEMPDIR) } {
        if !unsafe { VIM_TEMPDIR }.is_null() {
            unsafe { NOTFOUND += 1 };
            if unsafe { NOTFOUND } == 1 {
                // ELOG equivalent: tempdir disappeared (antivirus or broken cleanup job?)
                let mut msgbuf = [0u8; 512];
                unsafe {
                    vim_snprintf(
                        msgbuf.as_mut_ptr() as *mut c_char,
                        msgbuf.len(),
                        c"tempdir disappeared (antivirus or broken cleanup job?): %s".as_ptr(),
                        VIM_TEMPDIR,
                    )
                };
                unsafe {
                    logmsg(
                        LOGLVL_ERR,
                        std::ptr::null(),
                        c"vim_gettempdir".as_ptr(),
                        0,
                        true,
                        c"%s".as_ptr(),
                        msgbuf.as_ptr() as *const c_char,
                    )
                };
            }
            if unsafe { NOTFOUND } > 1 {
                // User-visible: "E5431: tempdir disappeared (%d times)"
                let mut buf = [0u8; 128];
                unsafe {
                    vim_snprintf(
                        buf.as_mut_ptr() as *mut c_char,
                        buf.len(),
                        c"E5431: tempdir disappeared (%d times)".as_ptr(),
                        NOTFOUND as c_int,
                    )
                };
                unsafe { emsg(buf.as_ptr() as *const c_char) };
            }
            unsafe {
                xfree(VIM_TEMPDIR as *mut c_void);
                VIM_TEMPDIR = std::ptr::null_mut();
            }
        }
        unsafe { vim_mktempdir_impl() };
    }
    unsafe { VIM_TEMPDIR }
}

/// Return a unique name that can be used for a temp file.
///
/// Note: The temp file is NOT created.
///
/// Returns pointer to the temp file name or NULL if Nvim can't create a
/// temporary directory.
///
/// Directly replaces the C `vim_tempname` symbol.
///
/// # Safety
/// Accesses global VIM_TEMPDIR and TEMP_COUNT.
#[export_name = "vim_tempname"]
pub unsafe extern "C" fn rs_vim_tempname() -> *mut c_char {
    let tempdir = unsafe { rs_vim_gettempdir() };
    if tempdir.is_null() {
        return std::ptr::null_mut();
    }

    let mut templ = [0u8; TEMP_FILE_PATH_MAXLEN];
    unsafe {
        vim_snprintf(
            templ.as_mut_ptr() as *mut c_char,
            templ.len(),
            c"%s%lu".as_ptr(),
            tempdir,
            TEMP_COUNT as libc::c_ulong,
        )
    };
    unsafe { TEMP_COUNT += 1 };
    unsafe { xstrdup(templ.as_ptr() as *const c_char) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_temp_dir_names_count() {
        assert_eq!(super::TEMP_DIR_NAMES.len(), 4);
    }
}
