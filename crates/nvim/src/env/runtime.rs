//! Where nvim's own directories are: `$VIM` and `$VIMRUNTIME`.
//!
//! [`getenv_resolved`] is the wrapper around the environment that makes the
//! runtime directory relocatable: when `$VIM` or `$VIMRUNTIME` is unset it
//! works backwards from `'helpfile'`, then from the executable's own path,
//! then from the compiled-in defaults, and publishes what it found.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::eval::vars::{vim_var_string, with_vim_var_str};
use crate::memory::{ThinCString, XString};
use crate::option::vars::{P_HF, p_hf};
use crate::os::env::{env_set, getenv_owned};
use crate::os::fs::dir_exists;
use crate::os::state::{didset_vim, didset_vimruntime};
use crate::path::MAXPATHL;
use crate::path::{
    after_pathsep, append_path, concat_fnames, path_fnamencmp, tail_index, tail_with_sep_index,
};
use crate::types::Vv;
use core::ffi::CStr;

/// Where `$VIM` falls back to.
pub(crate) const DEFAULT_VIM_DIR: &CStr = c"/usr/local/share/nvim";
/// Where `$VIMRUNTIME` falls back to, as the build configured it; empty when
/// it did not say.
pub(crate) const DEFAULT_VIMRUNTIME_DIR: &CStr = match CStr::from_bytes_with_nul(
    concat!(env!("NVIM_DEFAULT_VIMRUNTIME_DIR"), "\0").as_bytes(),
) {
    Ok(dir) => dir,
    Err(_) => panic!("NVIM_DEFAULT_VIMRUNTIME_DIR holds a NUL"),
};

/// The directory a runtime lives in, under `$VIM`.
const RUNTIME_DIRNAME: &[u8] = b"runtime";

/// `vimdir/runtime`, if `vimdir` is set and that directory exists.
fn runtime_dir_under(vimdir: &[u8]) -> Option<Vec<u8>> {
    if vimdir.is_empty() {
        return None;
    }
    let dir = concat_fnames(vimdir, RUNTIME_DIRNAME, true);
    dir_exists(dir.as_cstr()).then(|| dir.to_vec())
}

/// If `dirname + "/"` ends `path[..end]`, the index where `dirname` starts;
/// otherwise `end`.
///
/// With `path = /usr/local/share/nvim/runtime/doc/help.txt`, `end` at the
/// tail:
///
/// | `end` before | `dirname` | `end` after |
/// | --- | --- | --- |
/// | `help.txt` | `doc` | `doc/help.txt` |
/// | `doc/help.txt` | `runtime` | `runtime/doc/help.txt` |
/// | `runtime/doc/help.txt` | `vim74` | `runtime/doc/help.txt` |
fn remove_tail(path: &[u8], end: usize, dirname: &[u8]) -> usize {
    let len = dirname.len();
    let Some(start) = end.checked_sub(len + 1) else {
        return end;
    };
    if path_fnamencmp(&path[start..], dirname, len) == 0
        && (start == 0 || after_pathsep(path, start))
    {
        return start;
    }
    end
}

/// The install prefix nvim was run from: `v:progpath` with the executable and
/// its `bin/` removed, as a `MAXPATHL` buffer would hold it.
pub(crate) fn install_prefix() -> Vec<u8> {
    let progpath = vim_var_string(Vv::Progpath).unwrap_or_else(ThinCString::empty);
    let progpath = progpath.as_cstr().to_bytes();
    let mut prefix = progpath[..progpath.len().min(MAXPATHL as usize - 1)].to_vec();
    // Remove the trailing "nvim", then the trailing "bin/".
    prefix.truncate(tail_with_sep_index(&prefix));
    prefix.truncate(tail_index(&prefix));
    prefix
}

/// The value of environment variable `name`, or `None` when it is unset or
/// empty -- with nvim's own fallbacks for `$VIM` and `$VIMRUNTIME`, which
/// are what let the runtime directory be found relative to the binary.
/// A fallback found is published into the environment, so the next lookup
/// is fast and child processes see it too.
pub(crate) fn getenv_resolved(name: &CStr) -> Option<XString> {
    // `init_path()` runs before anything reaches here.
    debug_assert!(!with_vim_var_str(Vv::Progpath, CStr::is_empty));

    if let Some(value) = getenv_owned(name) {
        return Some(value);
    }
    let vimruntime = name == c"VIMRUNTIME";
    if !vimruntime && name != c"VIM" {
        return None;
    }

    // With $VIMRUNTIME unset, try $VIM/runtime and then $VIM -- but not
    // when the compiled-in default is set.
    let mut vim_path: Option<Vec<u8>> = None;
    if vimruntime
        && DEFAULT_VIMRUNTIME_DIR.is_empty()
        && let Some(vim) = getenv_owned(c"VIM")
    {
        vim_path = Some(runtime_dir_under(&vim).unwrap_or_else(|| vim.to_vec()));
    }

    // Still nothing: work backwards from 'helpfile' (unless it holds a
    // '$'), then from the executable's own path.
    if vim_path.is_none() {
        // Uninitialized (as it is this early in startup) is not the same as
        // empty: it rules 'helpfile' out, and the exe-relative path decides.
        let from_helpfile = !P_HF.is_uninit() && !p_hf(|hf| hf.to_bytes().contains(&b'$'));
        let path = if from_helpfile {
            Some(P_HF.get().to_vec())
        } else {
            // ../share/nvim/runtime, relative to the binary.
            let mut prefix = install_prefix();
            append_path(&mut prefix, b"share/nvim/runtime/", MAXPATHL as usize)
                .is_ok()
                .then_some(prefix)
        };
        if let Some(path) = path {
            // Drop the file name, then the directory that names what this
            // is: "doc/" for 'helpfile', "runtime/" when $VIM is wanted.
            let mut end = tail_index(&path);
            if from_helpfile {
                end = remove_tail(&path, end, b"doc");
            }
            if !vimruntime {
                end = remove_tail(&path, end, RUNTIME_DIRNAME);
            }
            // And the trailing path separator.
            if end > 0 && after_pathsep(&path, end) {
                end -= 1;
            }
            let dir = &path[..end];
            // Whatever came out has to be a directory.
            if dir_exists(XString::from_bytes(dir).as_cstr()) {
                vim_path = Some(dir.to_vec());
            }
        }
    }

    // Last resort: what the build compiled in.
    if vim_path.is_none() {
        if vimruntime && !DEFAULT_VIMRUNTIME_DIR.is_empty() {
            vim_path = Some(DEFAULT_VIMRUNTIME_DIR.to_bytes().to_vec());
        } else if !DEFAULT_VIM_DIR.is_empty() && vimruntime {
            vim_path = Some(
                runtime_dir_under(DEFAULT_VIM_DIR.to_bytes())
                    .unwrap_or_else(|| DEFAULT_VIM_DIR.to_bytes().to_vec()),
            );
        }
    }

    let vim_path = XString::from_vec(vim_path?);
    if vimruntime {
        env_set(c"VIMRUNTIME", vim_path.as_cstr());
        didset_vimruntime.set(true);
    } else {
        env_set(c"VIM", vim_path.as_cstr());
        didset_vim.set(true);
    }
    Some(vim_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_tail_steps_back_over_a_named_directory_only() {
        let path = b"/usr/local/share/nvim/runtime/doc/help.txt";
        let tail = tail_index(path);
        let doc = remove_tail(path, tail, b"doc");
        assert_eq!(&path[doc..], b"doc/help.txt");
        let runtime = remove_tail(path, doc, b"runtime");
        assert_eq!(&path[runtime..], b"runtime/doc/help.txt");
        assert_eq!(remove_tail(path, runtime, b"vim74"), runtime);
        // Too short to hold "doc/" at all.
        assert_eq!(remove_tail(b"oc/x", 3, b"doc"), 3);
    }
}
