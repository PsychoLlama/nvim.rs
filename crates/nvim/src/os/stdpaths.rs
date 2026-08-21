//! The XDG base directories, and the per-application subpaths nvim keeps
//! under them (`stdpath()`, `$NVIM_APPNAME`).
//!
//! # Boundary
//!
//! [`get_appname`] returns a pointer into the shared `NameBuff` scratch
//! buffer, as upstream does — the next call to it, or to anything else
//! that writes `NameBuff`, invalidates the result. Everything else here
//! returns freshly allocated C strings that the caller releases with
//! `xfree`.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::fileio::vim_gettempdir;
use crate::main::{IObuff, NameBuff};
use crate::memory::{xfree, xmemcpyz, xmemdupz, xstrdup};
use crate::os::env::{expand_env_save, os_env_exists, os_getenv, os_getenv_noalloc};
use crate::path::{concat_fnames_realloc, path_fnamecmp, path_is_absolute};
use crate::types::{IOSIZE, XDGVarType, size_t};
use core::ffi::{CStr, c_char};
use core::ptr;
use std::ffi::CString;

pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGDataDirs: XDGVarType = 6;

const PATHSEP: u8 = b'/';
/// Separator between entries of `$XDG_*_DIRS`.
const ENV_SEP: u8 = b':';

/// The environment variable behind each [`XDGVarType`].
const XDG_ENV_VARS: [&CStr; 7] = [
    c"XDG_CONFIG_HOME",
    c"XDG_DATA_HOME",
    c"XDG_CACHE_HOME",
    c"XDG_STATE_HOME",
    c"XDG_RUNTIME_DIR",
    c"XDG_CONFIG_DIRS",
    c"XDG_DATA_DIRS",
];

/// Value to fall back on when the environment variable is unset. Still
/// needs `~` expansion. `kXDGRuntimeDir` has none: `vim_mktempdir()`
/// decides it at startup.
const XDG_DEFAULTS: [Option<&CStr>; 7] = [
    Some(c"~/.config"),
    Some(c"~/.local/share"),
    Some(c"~/.cache"),
    Some(c"~/.local/state"),
    None,
    Some(c"/etc/xdg/"),
    Some(c"/usr/local/share/:/usr/share/"),
];

/// `$NVIM_APPNAME`, or "nvim" when unset.
///
/// The value lives in `NameBuff`, so the returned pointer is only good
/// until the next thing that writes there.
///
/// `namelike` additionally flattens path separators to `-`, for callers
/// that need a single name rather than a relative path. The substitution
/// runs over the whole buffer, past the terminator, exactly as upstream's
/// `memchrsub(NameBuff, ..., sizeof(NameBuff))` did.
pub fn get_appname(namelike: bool) -> *const c_char {
    // SAFETY: "noalloc" means it writes the value into `NameBuff` and
    // returns a pointer to it, or NULL when the variable is unset. No
    // borrow of `NameBuff` may be outstanding across the call.
    let is_set = unsafe { !os_getenv_noalloc(c"NVIM_APPNAME".as_ptr()).is_null() };
    const SLASH: c_char = b'/'.cast_signed();
    const BACKSLASH: c_char = b'\\'.cast_signed();
    const DASH: c_char = b'-'.cast_signed();
    NameBuff.with_mut(|buf| {
        if !is_set {
            for (slot, byte) in buf.iter_mut().zip(b"nvim\0") {
                *slot = byte.cast_signed();
            }
        }
        if namelike {
            for slot in buf.iter_mut() {
                if *slot == SLASH || *slot == BACKSLASH {
                    *slot = DASH;
                }
            }
        }
    });
    NameBuff.ptr().cast::<c_char>()
}

/// Whether `$NVIM_APPNAME` is usable: a name or a relative path, with no
/// way to escape the directory it names.
pub fn appname_is_valid() -> bool {
    let appname = get_appname(false);
    // SAFETY: `get_appname` returns `NameBuff`, which is NUL-terminated
    // and stays valid while this function runs.
    let name = unsafe {
        if path_is_absolute(appname) {
            return false;
        }
        CStr::from_ptr(appname).to_bytes()
    };
    // `path_is_absolute` does not call "/" absolute, hence the explicit
    // cases (upstream carries a TODO about that).
    !matches!(name, b"/" | b"\\" | b"." | b"..")
        && !contains(name, b"/..")
        && !contains(name, b"../")
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Drop repeated directories from a `sep`-separated search path, keeping
/// the first occurrence of each.
fn remove_duplicate_dirs(list: &CStr, sep: u8) -> Option<CString> {
    // `path_fnamecmp` rather than a byte compare: it honors
    // 'fileignorecase' and the platform's path-separator equivalences.
    dedup_dirs(list, sep, |a, b| {
        // SAFETY: both are NUL-terminated and outlive the call.
        unsafe { path_fnamecmp(a.as_ptr(), b.as_ptr()) == 0 }
    })
}

/// [`remove_duplicate_dirs`] with the "same directory" test injected.
///
/// `None` when nothing survives — the C built the result in a string
/// builder that stayed unallocated (NULL) in that case, and callers treat
/// NULL as "no such directory list".
fn dedup_dirs(list: &CStr, sep: u8, same: impl Fn(&CStr, &CStr) -> bool) -> Option<CString> {
    let mut kept: Vec<CString> = Vec::new();
    // Empty entries are skipped: the C tokenized with `strtok`, which
    // collapses runs of separators and ignores leading and trailing ones.
    for token in list.to_bytes().split(|&b| b == sep) {
        if token.is_empty() {
            continue;
        }
        let token = CString::new(token).expect("a CStr's bytes hold no NUL");
        if !kept.iter().any(|kept| same(kept, &token)) {
            kept.push(token);
        }
    }
    if kept.is_empty() {
        return None;
    }
    let joined = kept
        .iter()
        .map(|dir| dir.as_bytes())
        .collect::<Vec<_>>()
        .join(&[sep][..]);
    Some(CString::new(joined).expect("the parts hold no NUL"))
}

/// The value of an XDG base directory variable, or NULL when there is
/// none. The caller owns the result.
pub fn stdpaths_get_xdg_var(idx: XDGVarType) -> *mut c_char {
    let slot = usize::try_from(idx).expect("an XDGVarType is one of the seven");
    let env = XDG_ENV_VARS[slot];
    // SAFETY: every pointer handed out below is NUL-terminated and either
    // static or freshly allocated; `ret` is owned from here on.
    unsafe {
        let mut ret = os_getenv(env.as_ptr());
        if ret.is_null() {
            if os_env_exists(env.as_ptr(), false) {
                // Set but empty: `os_getenv` reports that as unset.
                ret = xstrdup(c"".as_ptr());
            } else if let Some(fallback) = XDG_DEFAULTS[slot] {
                ret = expand_env_save(fallback.as_ptr().cast_mut());
            } else if idx == kXDGRuntimeDir {
                // stdpath('run') is whatever vim_mktempdir() decided at
                // startup, minus its trailing slash.
                let tmpdir = vim_gettempdir();
                let tmpdir = if tmpdir.is_null() {
                    c"/tmp/".as_ptr()
                } else {
                    tmpdir
                };
                let len = CStr::from_ptr(tmpdir).to_bytes().len();
                ret = xmemdupz(tmpdir.cast(), len.saturating_sub(1)).cast::<c_char>();
            }
        }
        if ret.is_null() || (idx != kXDGDataDirs && idx != kXDGConfigDirs) {
            return ret;
        }
        let deduped = remove_duplicate_dirs(CStr::from_ptr(ret), ENV_SEP);
        xfree(ret.cast());
        deduped.map_or(ptr::null_mut(), CString::into_raw)
    }
}

/// `{xdg_directory}/$NVIM_APPNAME`, or NULL when the directory is unset.
/// The caller owns the result.
pub fn get_xdg_home(idx: XDGVarType) -> *mut c_char {
    let dir = stdpaths_get_xdg_var(idx);
    // SAFETY: `get_appname` returns NUL-terminated `NameBuff`; `IObuff` is
    // the scratch buffer this copy is sized against, and no borrow of it
    // is outstanding. `dir` is owned, and `concat_fnames_realloc` consumes
    // it.
    unsafe {
        let appname = get_appname(false);
        let appname_len = CStr::from_ptr(appname).to_bytes().len();
        // Windows appends "-data" to the data/state homes; the headroom is
        // asserted on every platform.
        let iosize = usize::try_from(IOSIZE).expect("the scratch buffer has a positive size");
        debug_assert!(appname_len < iosize - c"-data".count_bytes() - 1);
        if dir.is_null() {
            return dir;
        }
        xmemcpyz(IObuff.ptr().cast(), appname.cast(), appname_len);
        concat_fnames_realloc(dir, IObuff.ptr().cast::<c_char>(), true)
    }
}

/// `$XDG_CONFIG_HOME/$NVIM_APPNAME/{fname}`. The caller owns the result.
///
/// # Safety
///
/// `fname` is NUL-terminated.
pub unsafe fn stdpaths_user_conf_subpath(fname: *const c_char) -> *mut c_char {
    // SAFETY: the caller's name, and an owned directory `concat` consumes.
    unsafe { concat_fnames_realloc(get_xdg_home(kXDGConfigHome), fname, true) }
}

/// `$XDG_STATE_HOME/$NVIM_APPNAME/{fname}`, with `trailing_pathseps` path
/// separators appended and — when `escape_commas` — every comma
/// backslash-escaped, for the options that take comma-separated lists.
/// The caller owns the result.
///
/// # Safety
///
/// `fname` is NUL-terminated.
pub unsafe fn stdpaths_user_state_subpath(
    fname: *const c_char,
    trailing_pathseps: size_t,
    escape_commas: bool,
) -> *mut c_char {
    // SAFETY: the caller's name; `ret` is owned here and NUL-terminated,
    // and the borrow of it ends before it is freed.
    let ret = unsafe { concat_fnames_realloc(get_xdg_home(kXDGStateHome), fname, true) };
    let path = unsafe { CStr::from_ptr(ret) }.to_bytes();
    let commas = if escape_commas {
        path.iter().filter(|&&b| b == b',').count()
    } else {
        0
    };
    if commas == 0 && trailing_pathseps == 0 {
        return ret;
    }
    let mut out = Vec::with_capacity(path.len() + commas + trailing_pathseps + 1);
    for &byte in path {
        if escape_commas && byte == b',' {
            out.push(b'\\');
        }
        out.push(byte);
    }
    out.resize(out.len() + trailing_pathseps, PATHSEP);
    // SAFETY: `ret` is owned here and `path`'s borrow of it is over.
    unsafe { xfree(ret.cast()) };
    CString::new(out)
        .expect("a CStr's bytes hold no NUL")
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `dedup_dirs` with a plain byte comparison, so the test never
    /// reaches `path_fnamecmp` (and through it libc, which Miri cannot
    /// call).
    fn dedup(list: &CStr) -> Option<CString> {
        dedup_dirs(list, ENV_SEP, |a, b| a == b)
    }

    #[test]
    fn nothing_survives_an_empty_dir_list() {
        assert_eq!(dedup(c""), None);
        assert_eq!(dedup(c":::"), None);
    }

    #[test]
    fn separator_runs_collapse() {
        assert_eq!(dedup(c":/a::/b:").as_deref(), Some(c"/a:/b"));
    }

    #[test]
    fn the_first_occurrence_of_each_dir_wins() {
        assert_eq!(dedup(c"/a:/b:/a:/c:/b").as_deref(), Some(c"/a:/b:/c"));
    }

    #[test]
    fn appname_validity_rejects_traversal() {
        assert!(contains(b"a/../b", b"/.."));
        assert!(contains(b"../b", b"../"));
        assert!(!contains(b"a..b", b"/.."));
        assert!(!contains(b"", b"/.."));
    }
}
