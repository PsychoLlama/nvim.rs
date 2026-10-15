//! Building the *default* 'runtimepath' at startup.
//!
//! `runtimepath_default` is pure string assembly: the XDG config and data
//! directories, each in its standard order, with `site` and `after`
//! variants, `$VIMRUNTIME`, and the library directory -- concatenated into
//! one comma-separated option value with every embedded comma and backslash
//! escaped.
//!
//! Everything but reading the environment happens in [`RtpParts::build`],
//! which works over byte slices and is what the tests at the bottom of this
//! file exercise.
//! The C built the same string in two passes -- one to compute the exact
//! length, one to fill a buffer of it -- and asserted afterwards that the two
//! agreed; a growable `Vec` needs only the second, so the whole of
//! `compute_double_env_sep_len` and the arithmetic beside it is gone.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::memory::ThinCString;
use crate::os::env::{DEFAULT_LIB_DIR, vim_getenv_owned};
use crate::os::fs::dir_exists;
use crate::os::stdpaths::xdg_var;

use crate::types::MAXPATHL;

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
        let sep = u8::try_from(PATHSEP).expect("`PATHSEP` is an ASCII byte");
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
    let sep = u8::try_from(ENV_SEPCHAR).expect("`ENV_SEPCHAR` is an ASCII byte");
    let mut fwd = val.split(move |&b| b == sep).filter(|d| !d.is_empty());
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

/// An unset variable's bytes: none, as an empty one's.
fn bytes(value: &Option<ThinCString>) -> &[u8] {
    value.as_ref().map_or(&b""[..], ThinCString::as_bytes)
}

/// Where nvim's Lua runtime files were installed.
///
/// The configured path when it exists, else `lib/nvim` beside the binary's
/// install prefix -- which is what makes a relocated or AppImage build find
/// its own files.
pub fn get_lib_dir() -> Option<XString> {
    // TODO(bfredl): too fragile? Ideally default_lib_dir would be made
    // empty in an appimage build.
    if !DEFAULT_LIB_DIR.is_empty() && dir_exists(DEFAULT_LIB_DIR) {
        return Some(XString::from_cstr(DEFAULT_LIB_DIR));
    }
    let mut exe_name = vim_get_prefix_from_exepath();
    append_path(&mut exe_name, b"lib/nvim", MAXPATHL as usize).ok()?;
    Some(XString::from_vec(exe_name))
}

/// The startup value of 'runtimepath'.
///
/// `None` when nothing contributed a directory -- no XDG variables, no
/// `$VIMRUNTIME` and no library directory -- which is what the option code
/// reads as "leave the built-in default alone".
///
/// Reads the environment; must run on the main thread, like every other
/// reader of the XDG variables.
///
/// An unset variable and an empty one contribute the same nothing.
pub(crate) fn runtimepath_default(clean_arg: bool) -> Option<XString> {
    let appname = get_appname(false);
    // `--clean` starts from the packaged runtime only: no user config, no
    // user data.
    let data_home = if clean_arg {
        None
    } else {
        xdg_var(kXDGDataHome)
    };
    let config_home = if clean_arg {
        None
    } else {
        xdg_var(kXDGConfigHome)
    };
    let vimruntime = vim_getenv_owned(c"VIMRUNTIME");
    let libdir = get_lib_dir();
    let data_dirs = xdg_var(kXDGDataDirs);
    let config_dirs = xdg_var(kXDGConfigDirs);

    let mut rtp = RtpParts {
        appname: appname.to_bytes(),
        config_home: bytes(&config_home),
        config_dirs: bytes(&config_dirs),
        data_home: bytes(&data_home),
        data_dirs: bytes(&data_dirs),
        vimruntime: vimruntime.as_deref().unwrap_or_default(),
        libdir: libdir.as_deref().unwrap_or_default(),
    }
    .build();

    // The value ends in the terminator `build` wrote over the last comma.
    (rtp.pop() == Some(0)).then(|| XString::from_vec(rtp))
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
