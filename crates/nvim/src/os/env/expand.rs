//! `$VAR` and `~` expansion inside a path.
//!
//! [`expand_env_esc`] is the whole of it; the three shorter entry points are
//! its defaults. It writes into a caller-supplied buffer and, whenever
//! anything at all goes wrong, leaves what it has already copied — which is
//! why it is written as a copy loop with an early-out per name rather than as
//! a parser.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::charset::{vim_is_ident_char, vim_isfilec};
use crate::cmdexpand::{WildMode, WildOpts, expand_init, expand_one};
use crate::cstr;
use crate::eval::skip_expr;
use crate::os::users::os_get_userdir;
use crate::path::after_pathsep;
use crate::strings::{vim_strchr, vim_strsave_escaped};
use crate::types::{ExpandContext, MAXPATHL, expand_T};

/// [`expand_env`] into a newly allocated `MAXPATHL` buffer.
///
/// Not memory-efficient; the result is expected to be freed again soon.
///
/// # Safety
/// `src` must be a NUL-terminated string.
pub unsafe fn expand_env_save(src: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's contract.
    unsafe { expand_env_save_opt(src, false) }
}

/// [`expand_env_save`], but `one` treats `src` as a single file name, so only
/// a leading `~` is expanded.
///
/// # Safety
/// `src` must be a NUL-terminated string.
pub unsafe fn expand_env_save_opt(src: *mut c_char, one: bool) -> *mut c_char {
    // SAFETY: the caller's contract; `p` is `MAXPATHL` bytes, which is what
    // `expand_env_esc` is told.
    unsafe {
        let p = xmalloc(MAXPATHL as usize) as *mut c_char;
        expand_env_esc(src, p, MAXPATHL as c_int, false, one, ptr::null_mut());
        p
    }
}

/// Expand `$VAR` and a leading `~` in a path.
///
/// # Safety
/// `src` must be NUL-terminated and `dst` writable for `dstlen` bytes.
pub unsafe fn expand_env(src: *mut c_char, dst: *mut c_char, dstlen: c_int) -> size_t {
    // SAFETY: the caller's contract.
    unsafe { expand_env_esc(src, dst, dstlen, false, false, ptr::null_mut()) }
}

/// What one `$VAR`, `~`, or `~user` at `src` expands to.
struct Expansion {
    /// The expansion, or NULL when the name did not resolve.
    var: *mut c_char,
    /// Where in `src` to carry on from.
    tail: *const c_char,
    /// Whether `var` has to be freed.
    mustfree: bool,
}

/// Resolve the `$VAR` at `src`. The name is copied into `dst` first, because
/// `src` may be in read-only memory and a NUL has to go after it.
///
/// # Safety
/// `src` must be NUL-terminated and `dst` writable for `dstlen` bytes.
unsafe fn resolve_env_var(src: *const c_char, dst: *mut c_char, dstlen: c_int) -> Expansion {
    // SAFETY: the caller's contract; the copy is bounded by `dstlen - 1`.
    unsafe {
        let mut tail = src.add(1);
        let mut var = dst;
        let mut c = dstlen - 1;

        // Unix also has `${var-name}`, whose name may hold anything but '}'.
        // A `${VAR}` name may hold anything but '}'; a `$VAR` one is limited
        // to 'isident' characters — which is why the verification below asks
        // about `src[1]` rather than about `braced`: 'isident' may itself
        // contain '{'.
        let brace = *tail == b'{' as c_char;
        let braced = brace && !vim_is_ident_char('{' as c_int);
        if braced {
            tail = tail.add(1); // ignore '{'
            while c > 0 && *tail != 0 && *tail != b'}' as c_char {
                c -= 1;
                *var = *tail;
                var = var.add(1);
                tail = tail.add(1);
            }
        } else {
            while c > 0 && *tail != 0 && vim_is_ident_char(*tail as u8 as c_int) {
                c -= 1;
                *var = *tail;
                var = var.add(1);
                tail = tail.add(1);
            }
        }

        // A `${VAR}` that never reached its '}' is not a variable at all.
        if brace && *tail != b'}' as c_char {
            return Expansion {
                var: ptr::null_mut(),
                tail,
                mustfree: false,
            };
        }
        if brace {
            tail = tail.add(1);
        }
        *var = 0;
        Expansion {
            var: vim_getenv(dst),
            tail,
            mustfree: true,
        }
    }
}

/// Resolve the `~user` at `src`, falling back to shell expansion when the
/// user is not in the password database.
///
/// # Safety
/// As [`resolve_env_var`].
unsafe fn resolve_user_dir(src: *const c_char, dst: *mut c_char, dstlen: c_int) -> Expansion {
    // SAFETY: the caller's contract; the copy is bounded by `dstlen - 1`.
    unsafe {
        // Copy `~user` into `dst` so a NUL can go after it.
        let mut tail = src;
        let mut var = dst;
        let mut c = dstlen - 1;
        while c > 0
            && *tail != 0
            && vim_isfilec(*tail as u8 as c_int)
            && !vim_ispathsep(*tail as c_int)
        {
            c -= 1;
            *var = *tail;
            var = var.add(1);
            tail = tail.add(1);
        }
        *var = 0;

        let mut var = if *dst == 0 {
            ptr::null_mut()
        } else {
            os_get_userdir(dst.add(1))
        };
        if var.is_null() {
            // Not a known user: let the shell expand `~user`, which is slower
            // and may fail on an old /bin/sh.
            let mut xpc: expand_T = core::mem::zeroed();
            expand_init(&raw mut xpc);
            xpc.xp_context = ExpandContext::Files;
            var = expand_one(
                &raw mut xpc,
                dst,
                ptr::null_mut(),
                WildOpts::ADD_SLASH | WildOpts::SILENT,
                WildMode::ExpandFree,
            );
        }
        Expansion {
            var,
            tail,
            mustfree: true,
        }
    }
}

/// Expand `$VAR` and `~` in a path, with escaping.
///
/// `esc` backslash-escapes whitespace inside an expanded value, which
/// `:e ~/tt` needs when `$HOME` has a space in it. `one` treats `srcp` as a
/// single file name, so `~` after a space is *not* the start of a new name.
/// `prefix`, when not NULL, restarts name recognition after each occurrence
/// of it.
///
/// Answers the length written to `dst`, not counting the NUL. If anything
/// fails, no expansion is done and `dst` ends up equal to `src`.
///
/// # Safety
/// `srcp` must be NUL-terminated, `dst` writable for `dstlen` bytes, and
/// `prefix` NUL-terminated or NULL.
pub unsafe fn expand_env_esc(
    srcp: *const c_char,
    dst: *mut c_char,
    dstlen: c_int,
    esc: bool,
    one: bool,
    prefix: *mut c_char,
) -> size_t {
    // SAFETY: the caller's contract. Every write to `dst` is guarded by
    // `dstlen`, which counts down as the cursor advances, and `src` only ever
    // moves forward inside `srcp`.
    unsafe {
        let dst_start = dst;
        let mut dst = dst;
        // Leave one byte spare for a "\," escape.
        let mut dstlen = dstlen - 1;

        let prefix_len = if prefix.is_null() {
            0
        } else {
            cstr::bytes_at(prefix).len() as c_int
        };
        let mut src = skipwhite(srcp.cast_mut()).cast_const();
        // At the start of a name, which is what makes a `~` here mean the
        // home directory.
        let mut at_start = true;

        while *src != 0 && dstlen > 0 {
            // Skip over `=expr`, which is not a path at all.
            if *src == b'`' as c_char && *src.add(1) == b'=' as c_char {
                let var = src;
                src = src.add(2);
                let mut cursor = src.cast_mut();
                let _ = skip_expr(&raw mut cursor, ptr::null_mut());
                src = cursor;
                if *src == b'`' as c_char {
                    src = src.add(1);
                }
                let len = (src.offset_from(var) as usize).min(dstlen as usize);
                ptr::copy_nonoverlapping(var, dst, len);
                dst = dst.add(len);
                dstlen -= len as c_int;
                continue;
            }

            let mut copy_char = true;
            if *src == b'$' as c_char || (*src == b'~' as c_char && at_start) {
                let expansion = if *src != b'~' as c_char {
                    resolve_env_var(src, dst, dstlen)
                } else if *src.add(1) == 0
                    || vim_ispathsep(*src.add(1) as c_int)
                    || !vim_strchr(c" ,\t\n".as_ptr(), *src.add(1) as u8 as c_int).is_null()
                {
                    // The home directory itself.
                    Expansion {
                        var: homedir.get(),
                        tail: src.add(1),
                        mustfree: false,
                    }
                } else {
                    resolve_user_dir(src, dst, dstlen)
                };
                let Expansion {
                    mut var,
                    mut tail,
                    mut mustfree,
                } = expansion;

                // Whitespace inside the value has to be escaped, or the
                // caller's ' '-separated list would gain an entry.
                if esc && !var.is_null() && !strpbrk(var, c" \t".as_ptr()).is_null() {
                    let p = vim_strsave_escaped(var, c" \t".as_ptr());
                    if mustfree {
                        xfree(var.cast());
                    }
                    var = p;
                    mustfree = true;
                }

                if !var.is_null() && *var != 0 {
                    let c = cstr::bytes_at(var).len();
                    if c + cstr::bytes_at(tail).len() + 1 < dstlen as usize {
                        strcpy(dst, var);
                        dstlen -= c as c_int;
                        // If the value ends in a path separator and the tail
                        // starts with one, drop one of them.
                        if after_pathsep(dst, dst.add(c)) != 0 && vim_ispathsep(*tail as c_int) {
                            tail = tail.add(1);
                        }
                        dst = dst.add(c);
                        src = tail;
                        copy_char = false;
                    }
                }
                if mustfree {
                    xfree(var.cast());
                }
            }

            if copy_char {
                // Copy at least one byte, and work out whether the *next* one
                // starts a new name — which is what makes a `~` there mean
                // the home directory. Not when `one` is set, so `:edit foo ~
                // foo` leaves the middle one alone.
                at_start = false;
                if *src == b'\\' as c_char && *src.add(1) != 0 {
                    *dst = *src;
                    dst = dst.add(1);
                    src = src.add(1);
                    dstlen -= 1;
                } else if (*src == b' ' as c_char || *src == b',' as c_char) && !one {
                    at_start = true;
                }
                if dstlen > 0 {
                    *dst = *src;
                    dst = dst.add(1);
                    src = src.add(1);
                    dstlen -= 1;

                    if !prefix.is_null() && src.offset(-(prefix_len as isize)) >= srcp && {
                        let back = src.offset(-(prefix_len as isize));
                        cstr::prefix_eq(back, prefix, prefix_len as size_t)
                    } {
                        at_start = true;
                    }
                }
            }
        }
        *dst = 0;
        dst.offset_from(dst_start) as size_t
    }
}
