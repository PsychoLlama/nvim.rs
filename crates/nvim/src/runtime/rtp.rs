//! Building the *default* 'runtimepath' at startup.
//!
//! `runtimepath_default` is pure string assembly: the XDG config and data
//! directories, each in its standard order, with `site` and `after`
//! variants, `$VIMRUNTIME`, and the library directory -- concatenated into
//! one comma-separated option value with every embedded comma and backslash
//! escaped.
//!
//! Everything but reading the environment and handing the result back as
//! `xmalloc`ed memory happens in [`RtpParts::build`], which is safe code over
//! byte slices and is what the tests at the bottom of this file exercise.
//! The C built the same string in two passes -- one to compute the exact
//! length, one to fill a buffer of it -- and asserted afterwards that the two
//! agreed; a growable `Vec` needs only the second, so the whole of
//! `compute_double_env_sep_len` and the arithmetic beside it is gone.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

use crate::types::{MAXPATHL, OK};
use core::ffi::{CStr, c_char};
use core::ptr;

/// The suffix under a data directory that holds installed-by-hand runtime
/// files, as opposed to the user's own.
const SITE: &[u8] = b"site";

/// The suffix whose entries load *after* everything else, which is why they
/// are appended in reverse.
const AFTER: &[u8] = b"after";

/// The strings the default 'runtimepath' is assembled from, once the
/// environment has been read.
///
/// A `NULL` from the environment and an empty value behave identically
/// everywhere below -- both contribute nothing -- so both arrive here as an
/// empty slice.
struct RtpParts<'a> {
    /// `$NVIM_APPNAME`, or `nvim`: the component appended to each *home*
    /// directory and to every entry of the two dirs lists.
    appname: &'a [u8],
    config_home: &'a [u8],
    /// `$XDG_CONFIG_DIRS`, [`ENV_SEPCHAR`]-separated.
    config_dirs: &'a [u8],
    data_home: &'a [u8],
    /// `$XDG_DATA_DIRS`, [`ENV_SEPCHAR`]-separated.
    data_dirs: &'a [u8],
    vimruntime: &'a [u8],
    libdir: &'a [u8],
}

impl RtpParts<'_> {
    /// The default 'runtimepath': comma-separated and NUL-terminated, or
    /// empty when nothing contributed a directory at all.
    ///
    /// The order is the contract -- config first, then data, then the
    /// runtime and library directories, then the same list again in reverse
    /// with `after` appended.
    fn build(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.push_dir(&mut out, self.config_home, true, &[]);
        self.push_list(&mut out, self.config_dirs, &[], Order::Forward);
        self.push_dir(&mut out, self.data_home, true, &[SITE]);
        self.push_list(&mut out, self.data_dirs, &[SITE], Order::Forward);
        self.push_dir(&mut out, self.vimruntime, false, &[]);
        self.push_dir(&mut out, self.libdir, false, &[]);
        self.push_list(&mut out, self.data_dirs, &[SITE, AFTER], Order::Reverse);
        self.push_dir(&mut out, self.data_home, true, &[SITE, AFTER]);
        self.push_list(&mut out, self.config_dirs, &[AFTER], Order::Reverse);
        self.push_dir(&mut out, self.config_home, true, &[AFTER]);
        if let Some(last) = out.last_mut() {
            // Overwrite the trailing comma rather than dropping it: the
            // option value is a C string and wants the terminator anyway.
            *last = 0;
        }
        out
    }

    /// Append one directory and its trailing comma.
    ///
    /// `named` distinguishes the two XDG *homes*, which are shared with
    /// other applications and so get the appname (and any suffixes) below
    /// them, from `$VIMRUNTIME` and the library directory, which are ours
    /// already and are taken verbatim.
    fn push_dir(&self, out: &mut Vec<u8>, dir: &[u8], named: bool, sufs: &[&[u8]]) {
        if dir.is_empty() {
            return;
        }
        push_comma_escaped(out, dir);
        if named {
            self.push_components(out, sufs);
        }
        out.push(b',');
    }

    /// Append every non-empty entry of an [`ENV_SEPCHAR`]-separated list,
    /// each with the appname and `sufs` below it.
    fn push_list(&self, out: &mut Vec<u8>, val: &[u8], sufs: &[&[u8]], order: Order) {
        for dir in entries(val, order) {
            push_comma_escaped(out, dir);
            self.push_components(out, sufs);
            out.push(b',');
        }
    }

    /// Append `/<appname>` and then each of `sufs`, separated by [`PATHSEP`].
    ///
    /// The leading separator is only written when the directory did not
    /// already end in one (C's `after_pathsep`, which on this platform is
    /// exactly "the previous byte is a [`PATHSEP`]").
    fn push_components(&self, out: &mut Vec<u8>, sufs: &[&[u8]]) {
        let sep = PATHSEP as u8;
        if out.last() != Some(&sep) {
            out.push(sep);
        }
        out.extend_from_slice(self.appname);
        for suf in sufs {
            out.push(sep);
            out.extend_from_slice(suf);
        }
    }
}

/// Which end of an [`ENV_SEPCHAR`]-separated list to start from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Order {
    Forward,
    Reverse,
}

/// The non-empty entries of an [`ENV_SEPCHAR`]-separated list.
///
/// This is `vim_env_iter`/`vim_env_iter_rev` without the pointer walk: both
/// skip a zero-length component, so both are a `split` with the empties
/// filtered out.
fn entries(val: &[u8], order: Order) -> impl Iterator<Item = &[u8]> {
    let mut fwd = val
        .split(|&b| b == ENV_SEPCHAR as u8)
        .filter(|d| !d.is_empty());
    let mut rev = fwd.clone();
    core::iter::from_fn(move || match order {
        Order::Forward => fwd.next(),
        Order::Reverse => rev.next_back(),
    })
}

/// Append `src` with every comma backslash-escaped.
///
/// 'runtimepath' is comma-separated and a directory may legitimately contain
/// one, so this is what keeps such a directory a single entry.
fn push_comma_escaped(out: &mut Vec<u8>, src: &[u8]) {
    for &byte in src {
        if byte == b',' {
            out.push(b'\\');
        }
        out.push(byte);
    }
}

/// A possibly-null C string as bytes, without its terminator.
///
/// # Safety
/// `s` is null or NUL-terminated, and stays valid for the borrow.
unsafe fn bytes_of<'a>(s: *const c_char) -> &'a [u8] {
    if s.is_null() {
        return &[];
    }
    // SAFETY: the caller's contract.
    unsafe { CStr::from_ptr(s) }.to_bytes()
}

/// Where nvim's Lua runtime files were installed.
///
/// The configured path when it exists, else `lib/nvim` beside the binary's
/// install prefix -- which is what makes a relocated or AppImage build find
/// its own files.
pub unsafe fn get_lib_dir() -> *mut c_char {
    // SAFETY: `default_lib_dir` is a NUL-terminated build-time constant, and
    // `exe_name` is `MAXPATHL` bytes for the two calls that fill it.
    unsafe {
        // TODO(bfredl): too fragile? Ideally default_lib_dir would be made
        // empty in an appimage build.
        if strlen(default_lib_dir.get()) != 0 && os_isdir(default_lib_dir.get()) {
            return xstrdup(default_lib_dir.get());
        }
        let mut exe_name = [0 as c_char; MAXPATHL as usize];
        vim_get_prefix_from_exepath(exe_name.as_mut_ptr());
        if append_path(
            exe_name.as_mut_ptr(),
            c"lib/nvim".as_ptr(),
            MAXPATHL as size_t,
        ) == OK
        {
            return xstrdup(exe_name.as_ptr());
        }
        ptr::null_mut()
    }
}

/// The startup value of 'runtimepath'.
///
/// Answers null when nothing contributed a directory -- no XDG variables, no
/// `$VIMRUNTIME` and no library directory -- which is what the option code
/// reads as "leave the built-in default alone".
///
/// # Safety
/// Reads the environment; must run on the main thread, like every other
/// caller of `stdpaths_get_xdg_var`.
pub unsafe fn runtimepath_default(clean_arg: bool) -> *mut c_char {
    // SAFETY: every pointer below is either null or an owned NUL-terminated
    // string, freed once at the end and not borrowed past that point.
    unsafe {
        // `--clean` starts from the packaged runtime only: no user config,
        // no user data.
        let data_home = if clean_arg {
            ptr::null_mut()
        } else {
            stdpaths_get_xdg_var(kXDGDataHome)
        };
        let config_home = if clean_arg {
            ptr::null_mut()
        } else {
            stdpaths_get_xdg_var(kXDGConfigHome)
        };
        let vimruntime = vim_getenv(c"VIMRUNTIME".as_ptr());
        let libdir = get_lib_dir();
        let data_dirs = stdpaths_get_xdg_var(kXDGDataDirs);
        let config_dirs = stdpaths_get_xdg_var(kXDGConfigDirs);

        let rtp = RtpParts {
            appname: bytes_of(get_appname(false)),
            config_home: bytes_of(config_home),
            config_dirs: bytes_of(config_dirs),
            data_home: bytes_of(data_home),
            data_dirs: bytes_of(data_dirs),
            vimruntime: bytes_of(vimruntime),
            libdir: bytes_of(libdir),
        }
        .build();

        xfree(data_dirs.cast());
        xfree(config_dirs.cast());
        xfree(data_home.cast());
        xfree(config_home.cast());
        xfree(vimruntime.cast());
        xfree(libdir.cast());

        if rtp.is_empty() {
            return ptr::null_mut();
        }
        let out = xmalloc(rtp.len()) as *mut c_char;
        ptr::copy_nonoverlapping(rtp.as_ptr(), out.cast(), rtp.len());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The builder's answer as a string, with the NUL dropped.
    fn build(parts: &RtpParts) -> String {
        let mut out = parts.build();
        assert_eq!(out.pop(), Some(0));
        String::from_utf8(out).unwrap()
    }

    fn parts<'a>() -> RtpParts<'a> {
        RtpParts {
            appname: b"nvim",
            config_home: b"",
            config_dirs: b"",
            data_home: b"",
            data_dirs: b"",
            vimruntime: b"",
            libdir: b"",
        }
    }

    #[test]
    fn nothing_set_builds_nothing() {
        assert!(
            RtpParts {
                appname: b"nvim",
                ..parts()
            }
            .build()
            .is_empty()
        );
    }

    #[test]
    fn homes_come_first_and_after_comes_last() {
        let got = build(&RtpParts {
            config_home: b"/c",
            data_home: b"/d",
            vimruntime: b"/rt",
            libdir: b"/lib",
            ..parts()
        });
        assert_eq!(
            got,
            "/c/nvim,/d/nvim/site,/rt,/lib,/d/nvim/site/after,/c/nvim/after"
        );
    }

    #[test]
    fn dirs_lists_appear_twice_and_the_after_half_is_reversed() {
        let got = build(&RtpParts {
            config_dirs: b"/e1:/e2",
            data_dirs: b"/s1:/s2",
            ..parts()
        });
        assert_eq!(
            got,
            "/e1/nvim,/e2/nvim,/s1/nvim/site,/s2/nvim/site,\
             /s2/nvim/site/after,/s1/nvim/site/after,/e2/nvim/after,/e1/nvim/after"
        );
    }

    #[test]
    fn empty_list_entries_are_skipped() {
        let got = build(&RtpParts {
            config_dirs: b":/e1::/e2:",
            ..parts()
        });
        assert_eq!(got, "/e1/nvim,/e2/nvim,/e2/nvim/after,/e1/nvim/after");
    }

    #[test]
    fn commas_in_a_directory_are_escaped() {
        let got = build(&RtpParts {
            config_home: b"/co,mma",
            ..parts()
        });
        assert_eq!(got, "/co\\,mma/nvim,/co\\,mma/nvim/after");
    }

    #[test]
    fn a_trailing_separator_is_not_doubled() {
        let got = build(&RtpParts {
            config_home: b"/c/",
            config_dirs: b"/e/",
            ..parts()
        });
        assert_eq!(got, "/c/nvim,/e/nvim,/e/nvim/after,/c/nvim/after");
    }

    #[test]
    fn the_appname_is_what_names_every_home_component() {
        let got = build(&RtpParts {
            appname: b"probe",
            config_home: b"/c",
            data_dirs: b"/s",
            ..parts()
        });
        assert_eq!(
            got,
            "/c/probe,/s/probe/site,/s/probe/site/after,/c/probe/after"
        );
    }

    #[test]
    fn the_runtime_and_library_directories_take_no_components() {
        let got = build(&RtpParts {
            vimruntime: b"/rt/",
            libdir: b"/li,b",
            ..parts()
        });
        assert_eq!(got, "/rt/,/li\\,b");
    }
}
