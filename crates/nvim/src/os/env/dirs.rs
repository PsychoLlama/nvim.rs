//! Where nvim's own directories are, and how to write a path back with `~`.
//!
//! [`vim_getenv`] is the wrapper around `$VIM`/`$VIMRUNTIME`/`$HOME` that
//! makes the runtime directory relocatable: when the variable is unset it
//! works backwards from `'helpfile'`, then from the executable's own path,
//! then from the compiled-in defaults. [`home_replace`] is the inverse
//! direction, turning the home directory back into a `~`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::fs::modify_fname;
use crate::eval::vars::get_vim_var_str;
use crate::main::{didset_vim, didset_vimruntime, p_hf};
use crate::memory::xmemrchr;
use crate::os::fs::os_isdir;
use crate::path::{after_pathsep, append_path, concat_fnames, path_fnamencmp, path_tail_with_sep};
use crate::strings::vim_strchr;
use crate::types::{MAXPATHL, Vv, buf_T};

/// The directory a runtime lives in, under `$VIM`.
const RUNTIME_DIRNAME: &CStr = c"runtime";

/// `vimdir/runtime` if it exists, newly allocated; NULL otherwise.
///
/// # Safety
/// `vimdir` must be NUL-terminated or NULL.
unsafe fn vim_runtime_dir(vimdir: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract.
    unsafe {
        if vimdir.is_null() || *vimdir == 0 {
            return ptr::null_mut();
        }
        let p = concat_fnames(vimdir, RUNTIME_DIRNAME.as_ptr(), true);
        if os_isdir(p) {
            return p;
        }
        xfree(p.cast());
        ptr::null_mut()
    }
}

/// If `dirname + "/"` precedes `pend` in `path`, answer the pointer to
/// `dirname + "/" + pend`; otherwise answer `pend`.
///
/// With `path = /usr/local/share/nvim/runtime/doc/help.txt`:
///
/// | `pend` | `dirname` | answer |
/// | --- | --- | --- |
/// | `help.txt` | `doc` | `doc/help.txt` |
/// | `doc/help.txt` | `runtime` | `runtime/doc/help.txt` |
/// | `runtime/doc/help.txt` | `vim74` | `runtime/doc/help.txt` |
///
/// # Safety
/// `pend` must be a suffix of the NUL-terminated `path`, and `dirname`
/// NUL-terminated.
unsafe fn remove_tail(path: *mut c_char, pend: *mut c_char, dirname: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract; `new_tail` is compared against `path`
    // before it is read through.
    unsafe {
        let len = strlen(dirname);
        let new_tail = pend.sub(len + 1);
        if new_tail >= path
            && path_fnamencmp(new_tail, dirname, len) == 0
            && (new_tail == path || after_pathsep(path, new_tail) != 0)
        {
            return new_tail;
        }
        pend
    }
}

/// Walk a `$PATH`-like `delim`-separated list.
///
/// `iter` is NULL on the first call and the answer of the previous one after
/// that; NULL comes back when there is nothing left. The environment must not
/// be modified during the walk.
///
/// # Safety
/// `val` must be NUL-terminated, `iter` NULL or an answer of this function
/// over the same `val`, and `dir`/`len` writable.
pub unsafe fn vim_env_iter(
    delim: c_char,
    val: *const c_char,
    iter: *const c_void,
    dir: *mut *const c_char,
    len: *mut size_t,
) -> *const c_void {
    // SAFETY: the caller's contract.
    unsafe {
        let varval = if iter.is_null() {
            val
        } else {
            iter as *const c_char
        };
        *dir = varval;
        let dirend = strchr(varval, delim as c_int);
        if dirend.is_null() {
            *len = strlen(varval);
            return ptr::null();
        }
        *len = dirend.offset_from(varval) as size_t;
        dirend.add(1).cast()
    }
}

/// [`vim_env_iter`], from the end backwards.
///
/// # Safety
/// As [`vim_env_iter`].
pub unsafe fn vim_env_iter_rev(
    delim: c_char,
    val: *const c_char,
    iter: *const c_void,
    dir: *mut *const c_char,
    len: *mut size_t,
) -> *const c_void {
    // SAFETY: the caller's contract.
    unsafe {
        let varend = if iter.is_null() {
            val.add(strlen(val)).sub(1)
        } else {
            iter as *const c_char
        };
        let varlen = varend.offset_from(val) as size_t + 1;
        let colon = xmemrchr(val.cast(), delim as u8, varlen) as *const c_char;
        if colon.is_null() {
            *len = varlen;
            *dir = val;
            return ptr::null();
        }
        *dir = colon.add(1);
        *len = varend.offset_from(colon) as size_t;
        colon.sub(1).cast()
    }
}

/// The install prefix nvim was run from: `v:progpath` with the executable and
/// its `bin/` removed.
///
/// # Safety
/// `exe_name` must be writable for `MAXPATHL` bytes.
pub unsafe fn vim_get_prefix_from_exepath(exe_name: *mut c_char) {
    // SAFETY: the caller's contract; `path_tail*` answer pointers inside the
    // buffer they are given.
    unsafe {
        xstrlcpy(exe_name, get_vim_var_str(Vv::Progpath), MAXPATHL as usize);
        // Remove the trailing "nvim", then the trailing "bin/".
        *path_tail_with_sep(exe_name) = 0;
        *path_tail(exe_name) = 0;
    }
}

/// `getenv()` with nvim's special handling of `$VIM` and `$VIMRUNTIME`, which
/// is what lets the runtime directory be found relative to the binary.
///
/// The result is newly allocated, or NULL.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn vim_getenv(name: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract. Everything below is a NUL-terminated
    // path; `exe_name` is a local whose address never escapes, which the
    // `vim_path != exe_name` assertion at the end of the block is upstream's
    // way of saying.
    unsafe {
        // `init_path()` runs before anything reaches here.
        debug_assert!(*get_vim_var_str(Vv::Progpath) != 0);

        let kos_env_path = os_getenv(name);
        if !kos_env_path.is_null() {
            return kos_env_path;
        }

        let vimruntime = strcmp(name, c"VIMRUNTIME".as_ptr()) == 0;
        if !vimruntime && strcmp(name, c"VIM".as_ptr()) != 0 {
            return ptr::null_mut();
        }

        // With $VIMRUNTIME unset, try $VIM/runtime and then $VIM — but not
        // when the compiled-in default is set.
        let mut vim_path: *mut c_char = ptr::null_mut();
        if vimruntime && *default_vimruntime_dir.get() == 0 {
            let vim = os_getenv(c"VIM".as_ptr());
            if !vim.is_null() {
                vim_path = vim_runtime_dir(vim);
                if vim_path.is_null() {
                    vim_path = vim;
                } else {
                    xfree(vim.cast());
                }
            }
        }

        // Still nothing: work backwards from 'helpfile' (unless it holds a
        // '$'), then from the executable's own path.
        let mut exe_name: [c_char; MAXPATHL as usize] = [0; MAXPATHL as usize];
        if vim_path.is_null() {
            let from_helpfile =
                !p_hf.get().is_null() && vim_strchr(p_hf.get(), '$' as c_int).is_null();
            if from_helpfile {
                vim_path = p_hf.get();
            } else {
                // ../share/nvim/runtime, relative to the binary.
                vim_get_prefix_from_exepath(exe_name.as_mut_ptr());
                if append_path(
                    exe_name.as_mut_ptr(),
                    c"share/nvim/runtime/".as_ptr(),
                    MAXPATHL as usize,
                )
                .is_ok()
                {
                    vim_path = exe_name.as_mut_ptr();
                }
            }

            if !vim_path.is_null() {
                // Drop the file name, then the directory that names what this
                // is: "doc/" for 'helpfile', "runtime/" when $VIM is wanted.
                let mut vim_path_end = path_tail(vim_path);
                if from_helpfile {
                    vim_path_end = remove_tail(vim_path, vim_path_end, c"doc".as_ptr());
                }
                if !vimruntime {
                    vim_path_end = remove_tail(vim_path, vim_path_end, RUNTIME_DIRNAME.as_ptr());
                }
                // And the trailing path separator.
                if vim_path_end > vim_path && after_pathsep(vim_path, vim_path_end) != 0 {
                    vim_path_end = vim_path_end.sub(1);
                }
                debug_assert!(vim_path_end >= vim_path);
                vim_path = xmemdupz(
                    vim_path.cast(),
                    vim_path_end.offset_from(vim_path) as size_t,
                ) as *mut c_char;

                // Whatever came out has to be a directory.
                if !os_isdir(vim_path) {
                    xfree(vim_path.cast());
                    vim_path = ptr::null_mut();
                }
            }
        }

        // Last resort: what `pathdef` compiled in.
        if vim_path.is_null() {
            if vimruntime && *default_vimruntime_dir.get() != 0 {
                vim_path = xstrdup(default_vimruntime_dir.get());
            } else if *default_vim_dir.get() != 0 {
                if vimruntime {
                    vim_path = vim_runtime_dir(default_vim_dir.get());
                }
                if vim_path.is_null() && vimruntime {
                    vim_path = xstrdup(default_vim_dir.get());
                }
            }
        }

        // Publish it, so the next lookup is fast and other processes (Perl,
        // say) see it too.
        if !vim_path.is_null() {
            if vimruntime {
                os_setenv(c"VIMRUNTIME".as_ptr(), vim_path, 1);
                didset_vimruntime.set(true);
            } else {
                os_setenv(c"VIM".as_ptr(), vim_path, 1);
                didset_vim.set(true);
            }
        }
        vim_path
    }
}

/// Replace the home directory with `~` in each file name of `src`.
///
/// `buf`, when not NULL, is checked for being a help buffer — in which case
/// the path is dropped entirely and `one` is ignored. `one` treats `src` as
/// a single file name rather than a space/comma separated list.
///
/// Answers the length written to `dst`, not counting the NUL. If anything but
/// running out of space fails, `dst` ends up equal to `src`.
///
/// # Safety
/// `dst` must be writable for `dstlen` bytes; `src` NUL-terminated or NULL;
/// `buf` a live buffer or NULL.
pub unsafe fn home_replace(
    buf: *const buf_T,
    src: *const c_char,
    dst: *mut c_char,
    dstlen: size_t,
    one: bool,
) -> size_t {
    // SAFETY: the caller's contract. Every write to `dst` decrements
    // `dstlen`, and both loops stop when it reaches zero.
    unsafe {
        if src.is_null() {
            *dst = 0;
            return 0;
        }
        if !buf.is_null() && (*buf).b_help {
            let dlen = xstrlcpy(dst, path_tail(src), dstlen);
            return dlen.min(dstlen - 1);
        }

        // Both the *value* of $HOME and the resolved home directory count.
        let dirlen = if homedir.get().is_null() {
            0
        } else {
            strlen(homedir.get())
        };

        let homedir_env = os_getenv(c"HOME".as_ptr());
        let mut homedir_env_mod = homedir_env;
        let mut must_free = false;
        if !homedir_env_mod.is_null() && *homedir_env_mod == b'~' as c_char {
            // A $HOME that is itself relative to a home directory.
            must_free = true;
            let mut usedlen: size_t = 0;
            let mut flen = strlen(homedir_env_mod);
            let mut fbuf: *mut c_char = ptr::null_mut();
            modify_fname(
                c":p".as_ptr().cast_mut(),
                false,
                &raw mut usedlen,
                &raw mut homedir_env_mod,
                &raw mut fbuf,
                &raw mut flen,
            );
            flen = strlen(homedir_env_mod);
            debug_assert!(homedir_env_mod != homedir_env);
            if vim_ispathsep(*homedir_env_mod.add(flen - 1) as c_int) {
                // Drop the '/' that gets added to a directory.
                *homedir_env_mod.add(flen - 1) = 0;
            }
        }
        let envlen = if homedir_env_mod.is_null() {
            0
        } else {
            strlen(homedir_env_mod)
        };

        let mut src = if one { src } else { skipwhite(src.cast_mut()) };
        let mut dstlen = dstlen;
        let mut dst_p = dst;
        while *src != 0 && dstlen > 0 {
            // At the start of a file name. Either home directory may match,
            // but only up to a path separator: with a home of "/home/piet",
            // "/home/pieter/bla" must not come back as "~er/bla".
            let mut p = homedir.get();
            let mut len = dirlen;
            loop {
                if len != 0
                    && path_fnamencmp(src, p, len) == 0
                    && (vim_ispathsep(*src.add(len) as c_int)
                        || (!one
                            && (*src.add(len) == b',' as c_char
                                || *src.add(len) == b' ' as c_char))
                        || *src.add(len) == 0)
                {
                    src = src.add(len);
                    dstlen -= 1;
                    if dstlen > 0 {
                        *dst_p = b'~' as c_char;
                        dst_p = dst_p.add(1);
                    }
                    // No separator goes into `dst`: the caller wants the
                    // directory name without one.
                    break;
                }
                if p == homedir_env_mod {
                    break;
                }
                p = homedir_env_mod;
                len = envlen;
            }
            if dstlen == 0 {
                break;
            }

            // Copy the name, up to the separator unless `one`.
            while *src != 0 && (one || (*src != b',' as c_char && *src != b' ' as c_char)) {
                dstlen -= 1;
                if dstlen == 0 {
                    break;
                }
                *dst_p = *src;
                dst_p = dst_p.add(1);
                src = src.add(1);
            }
            if dstlen == 0 {
                break;
            }
            // And the separator itself.
            while *src == b' ' as c_char || *src == b',' as c_char {
                dstlen -= 1;
                if dstlen == 0 {
                    break;
                }
                *dst_p = *src;
                dst_p = dst_p.add(1);
                src = src.add(1);
            }
            if dstlen == 0 {
                break;
            }
        }
        // Running out of space just truncates.
        *dst_p = 0;

        xfree(homedir_env.cast());
        if must_free {
            xfree(homedir_env_mod.cast());
        }
        dst_p.offset_from(dst) as size_t
    }
}

/// [`home_replace`] into newly allocated memory.
///
/// # Safety
/// `src` must be NUL-terminated or NULL, `buf` live or NULL.
pub unsafe fn home_replace_save(buf: *mut buf_T, src: *const c_char) -> *mut c_char {
    // SAFETY: the caller's contract; the buffer is sized for the source plus
    // "~/" and the NUL.
    unsafe {
        let len = 3 + if src.is_null() { 0 } else { strlen(src) };
        let dst = xmalloc(len) as *mut c_char;
        home_replace(buf, src, dst, len, true);
        dst
    }
}
