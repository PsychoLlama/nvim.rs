//! The XDG base directories, and the per-application subpaths nvim keeps
//! under them (`stdpath()`, `$NVIM_APPNAME`).
//!
//! Every answer is an owned string.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::env::XDGVarType;
use crate::env::expand_env;
use crate::fileio::vim_gettempdir;
use crate::memory::{ThinCString, XString};
use crate::os::env::{env_exists, env_get_bounded, getenv_owned};
use crate::path::{concat_fnames, path_fnamecmp, path_is_absolute};
use crate::types::IOSIZE;
use core::ffi::CStr;
use std::ffi::CString;

pub(crate) const kXDGConfigHome: XDGVarType = 0;
pub(crate) const kXDGStateHome: XDGVarType = 3;
pub(crate) const kXDGRuntimeDir: XDGVarType = 4;
pub(crate) const kXDGConfigDirs: XDGVarType = 5;
pub(crate) const kXDGDataDirs: XDGVarType = 6;

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

/// `$NVIM_APPNAME`, or "nvim" when unset, as a string the caller owns.
///
/// Upstream stages it in the shared `NameBuff` and answers a pointer there,
/// so the answer was only good until the next path anything in the editor
/// formatted -- which is why [`get_xdg_home`] copied it into `IObuff`
/// before building a path out of it, and why two of the four callers
/// document how long they may hold it. Owning it retires all of that.
///
/// `namelike` additionally flattens path separators to `-`, for callers
/// that need a single name rather than a relative path.
pub(crate) fn get_appname(namelike: bool) -> CString {
    let mut name = env_get_bounded(c"NVIM_APPNAME");
    if name.is_empty() {
        name = b"nvim".to_vec();
    }
    if namelike {
        for byte in &mut name {
            if *byte == b'/' || *byte == b'\\' {
                *byte = b'-';
            }
        }
    }
    cstr::owned(&name)
}

/// Whether `$NVIM_APPNAME` is usable: a name or a relative path, with no
/// way to escape the directory it names.
pub(crate) fn appname_is_valid() -> bool {
    let appname = get_appname(false);
    if path_is_absolute(&appname) {
        return false;
    }
    let name = appname.to_bytes();
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
        path_fnamecmp(a.to_bytes(), b.to_bytes()) == 0
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

/// The value of an XDG base directory variable, or `None` when there is
/// none.
pub(crate) fn xdg_var(idx: XDGVarType) -> Option<ThinCString> {
    let slot = usize::try_from(idx).expect("an XDGVarType is one of the seven");
    let env = XDG_ENV_VARS[slot];
    let value = match getenv_owned(env) {
        Some(value) => value.to_vec(),
        // Set but empty: `getenv_owned` reports that as unset.
        None if env_exists(env, false) => Vec::new(),
        None => match XDG_DEFAULTS[slot] {
            Some(fallback) => expand_env(fallback, false).to_vec(),
            // stdpath('run') is whatever vim_mktempdir() decided at
            // startup, minus its trailing slash.
            None if idx == kXDGRuntimeDir => {
                let tmpdir = vim_gettempdir();
                let tmpdir = tmpdir.as_deref().unwrap_or(c"/tmp/").to_bytes();
                tmpdir[..tmpdir.len().saturating_sub(1)].to_vec()
            }
            None => return None,
        },
    };
    if idx != kXDGDataDirs && idx != kXDGConfigDirs {
        return Some(ThinCString::from_vec(value));
    }
    remove_duplicate_dirs(&cstr::owned(&value), ENV_SEP).map(ThinCString::from)
}

/// `{xdg_directory}/$NVIM_APPNAME`, or `None` when the directory is unset.
pub(crate) fn xdg_home(idx: XDGVarType) -> Option<XString> {
    let dir = xdg_var(idx);
    let appname = get_appname(false);
    // Windows appends "-data" to the data/state homes; the headroom upstream
    // needed for that in `IObuff` is asserted on every platform.
    let iosize = usize::try_from(IOSIZE).expect("the scratch buffer has a positive size");
    debug_assert!(appname.count_bytes() < iosize - c"-data".count_bytes() - 1);
    Some(concat_fnames(
        dir?.as_cstr().to_bytes(),
        appname.to_bytes(),
        true,
    ))
}

/// `$XDG_CONFIG_HOME/$NVIM_APPNAME/{fname}`.
pub(crate) fn user_config_path(fname: &[u8]) -> XString {
    let home = xdg_home(kXDGConfigHome).expect("the config home has a default");
    concat_fnames(&home, fname, true)
}

/// `$XDG_STATE_HOME/$NVIM_APPNAME/{fname}`, with `trailing_pathseps` path
/// separators appended and -- when `escape_commas` -- every comma
/// backslash-escaped, for the options that take comma-separated lists.
pub(crate) fn state_subpath(
    fname: &CStr,
    trailing_pathseps: usize,
    escape_commas: bool,
) -> XString {
    let home = xdg_home(kXDGStateHome).expect("the state home has a default");
    let path = concat_fnames(&home, fname.to_bytes(), true);
    let mut out = XString::with_capacity(path.len() + trailing_pathseps);
    for &byte in path.iter() {
        if escape_commas && byte == b',' {
            out.push_byte(b'\\');
        }
        out.push_byte(byte);
    }
    for _ in 0..trailing_pathseps {
        out.push_byte(PATHSEP);
    }
    out
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
