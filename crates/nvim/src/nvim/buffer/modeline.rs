//! Modelines -- `chk_modeline()` and `'modelines'`.
//!
//! [`do_modelines`] scans the first and last `'modelines'` lines of a buffer
//! for a `vim:`/`ex:` marker, and [`chk_modeline`] parses one: the
//! `set`-style option list, the `:`-terminated form, the version-guarded
//! `vim<800:` prefixes, and the `sandbox`/`'modelineexpr'` restrictions that
//! decide which options a file is allowed to set.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_isspace;
use crate::src::nvim::charset::{skipwhite, try_getdigits};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{curbuf, current_sctx, p_mls, secure};
use crate::src::nvim::memline::{ml_get, ml_get_len};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::option::do_set;
use crate::src::nvim::os::libc::{__ctype_b_loc, memmove, strncmp};
use crate::src::nvim::runtime::{estack_pop, estack_push};
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::types::{intmax_t, linenr_T, scid_T, sctx_T, size_t, uint8_t};
use crate::src::nvim::version::min_vim_version;

pub unsafe extern "C" fn do_modelines(mut flags: ::core::ffi::c_int) {
    unsafe {
        let mut lnum: linenr_T = 0;
        let mut nmlines: ::core::ffi::c_int = 0;
        static entered: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if (*curbuf.get()).b_p_ml == 0 || {
            nmlines = p_mls.get() as ::core::ffi::c_int;
            nmlines == 0 as ::core::ffi::c_int
        } {
            return;
        }
        if entered.get() != 0 {
            return;
        }
        (*entered.ptr()) += 1;
        lnum = 1 as ::core::ffi::c_int as linenr_T;
        while (*curbuf.get()).b_p_ml != 0
            && lnum <= (*curbuf.get()).b_ml.ml_line_count
            && lnum <= nmlines as linenr_T
        {
            if chk_modeline(lnum, flags) == FAIL {
                nmlines = 0 as ::core::ffi::c_int;
            }
            lnum += 1;
        }
        lnum = (*curbuf.get()).b_ml.ml_line_count;
        while (*curbuf.get()).b_p_ml != 0
            && lnum > 0 as linenr_T
            && lnum > nmlines as linenr_T
            && lnum > (*curbuf.get()).b_ml.ml_line_count - nmlines as linenr_T
        {
            if chk_modeline(lnum, flags) == FAIL {
                nmlines = 0 as ::core::ffi::c_int;
            }
            lnum -= 1;
        }
        (*entered.ptr()) -= 1;
    }
}

unsafe extern "C" fn chk_modeline(
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut retval: ::core::ffi::c_int = OK;
        let mut prev: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = ml_get(lnum);
        let mut line_end: *mut ::core::ffi::c_char = s.offset(ml_get_len(lnum) as isize);
        's_91: while *s as ::core::ffi::c_int != NUL {
            's_24: {
                if prev == -1 as ::core::ffi::c_int
                    || ascii_isspace(prev) as ::core::ffi::c_int != 0
                {
                    if prev != -1 as ::core::ffi::c_int
                        && strncmp(s, c"ex:".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int
                        || strncmp(s, c"vi:".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int
                    {
                        break 's_91;
                    }
                    if (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'v' as ::core::ffi::c_int
                        || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'V' as ::core::ffi::c_int)
                        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'i' as ::core::ffi::c_int
                        && *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'm' as ::core::ffi::c_int
                    {
                        if *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '<' as ::core::ffi::c_int
                            || *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '=' as ::core::ffi::c_int
                            || *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '>' as ::core::ffi::c_int
                        {
                            e = s.offset(4 as ::core::ffi::c_int as isize);
                        } else {
                            e = s.offset(3 as ::core::ffi::c_int as isize);
                        }
                        let mut vers: intmax_t = 0;
                        if !try_getdigits(&raw mut e, &raw mut vers) {
                            break 's_24;
                        } else {
                            let vim_version: ::core::ffi::c_int = min_vim_version();
                            if *e as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                                && (*s.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    != 'V' as ::core::ffi::c_int
                                    || strncmp(
                                        skipwhite(e.offset(1 as ::core::ffi::c_int as isize)),
                                        c"set".as_ptr(),
                                        3 as size_t,
                                    ) == 0 as ::core::ffi::c_int)
                                && (*s.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == ':' as ::core::ffi::c_int
                                    || vim_version as intmax_t >= vers
                                        && *(*__ctype_b_loc()).offset(
                                            *s.offset(3 as ::core::ffi::c_int as isize) as uint8_t
                                                as ::core::ffi::c_int
                                                as isize,
                                        )
                                            as ::core::ffi::c_int
                                            & _ISdigit as ::core::ffi::c_int
                                                as ::core::ffi::c_ushort
                                                as ::core::ffi::c_int
                                            != 0
                                    || (vim_version as intmax_t) < vers
                                        && *s.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '<' as ::core::ffi::c_int
                                    || vim_version as intmax_t > vers
                                        && *s.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '>' as ::core::ffi::c_int
                                    || vim_version as intmax_t == vers
                                        && *s.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '=' as ::core::ffi::c_int)
                            {
                                break 's_91;
                            }
                        }
                    }
                }
                prev = *s as uint8_t as ::core::ffi::c_int;
            }
            s = s.offset(1);
        }
        if *s == 0 {
            return retval;
        }
        loop {
            s = s.offset(1);
            if *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == ':' as ::core::ffi::c_int
            {
                break;
            }
        }
        let mut len: size_t = line_end.offset_from(s) as size_t;
        let mut linecopy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        linecopy = xstrnsave(s, len);
        s = linecopy;
        line_end = s.add(len);
        estack_push(
            ETYPE_MODELINE,
            c"modelines".as_ptr() as *mut ::core::ffi::c_char,
            lnum,
        );
        let mut end: bool = false_0 != 0;
        while end as ::core::ffi::c_int == false_0 {
            s = skipwhite(s);
            if *s as ::core::ffi::c_int == NUL {
                break;
            }
            e = s;
            while *e as ::core::ffi::c_int != ':' as ::core::ffi::c_int
                && *e as ::core::ffi::c_int != NUL
            {
                if *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *e.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ':' as ::core::ffi::c_int
                {
                    memmove(
                        e as *mut ::core::ffi::c_void,
                        e.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        (line_end.offset_from(e.offset(1 as ::core::ffi::c_int as isize))
                            as size_t)
                            .wrapping_add(1 as size_t),
                    );
                    line_end = line_end.offset(-1);
                }
                e = e.offset(1);
            }
            if *e as ::core::ffi::c_int == NUL {
                end = true_0 != 0;
            }
            if strncmp(s, c"set ".as_ptr(), 4 as size_t) == 0 as ::core::ffi::c_int
                || strncmp(s, c"se ".as_ptr(), 3 as size_t) == 0 as ::core::ffi::c_int
            {
                if *e as ::core::ffi::c_int != ':' as ::core::ffi::c_int {
                    break;
                } else {
                    end = true_0 != 0;
                    s = s.offset(
                        (if *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                        {
                            3 as ::core::ffi::c_int
                        } else {
                            4 as ::core::ffi::c_int
                        }) as isize,
                    );
                }
            }
            *e = NUL as ::core::ffi::c_char;
            if *s as ::core::ffi::c_int != NUL {
                let secure_save: ::core::ffi::c_int = secure.get();
                let save_current_sctx: sctx_T = current_sctx.get();
                (*current_sctx.ptr()).sc_sid = SID_MODELINE as scid_T;
                (*current_sctx.ptr()).sc_seq = 0 as ::core::ffi::c_int;
                (*current_sctx.ptr()).sc_lnum = lnum;
                secure.set(1 as ::core::ffi::c_int);
                retval = do_set(
                    s,
                    OPT_MODELINE as ::core::ffi::c_int | OPT_LOCAL as ::core::ffi::c_int | flags,
                );
                secure.set(secure_save);
                current_sctx.set(save_current_sctx);
                if retval == FAIL {
                    break;
                }
            }
            s = if e == line_end {
                e
            } else {
                e.offset(1 as ::core::ffi::c_int as isize)
            };
        }
        estack_pop();
        xfree(linecopy as *mut ::core::ffi::c_void);
        return retval;
    }
}
