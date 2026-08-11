//! Canonicalising a path -- `resolve()`, `simplify()`, `pathshorten()`,
//! `glob2regpat()` and `isabsolutepath()`.
//!
//! These are the pure-ish string transforms over a path: `f_resolve` is the
//! only one that reads the filesystem, following a symlink chain (with its own
//! loop guard) until it reaches something that is not a link; `f_simplify`
//! collapses `.`/`..`/duplicate separators without looking at the disk;
//! `f_pathshorten` reduces every leading component to its first character;
//! `f_glob2regpat` translates a wildcard pattern into the regex the search
//! engine wants.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{MAXPATHL, NUL, NULL, false_0, true_0};
use crate::src::nvim::eval::typval::{tv_get_number, tv_get_string, tv_get_string_chk};
use crate::src::nvim::fileio::file_pat_to_reg_pat;
use crate::src::nvim::memory::{xfree, xmallocz, xrealloc, xstrdup, xstrlcat};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, memcpy, memmove, readlink, strlen};
use crate::src::nvim::path::{
    add_pathsep, after_pathsep, path_is_absolute, path_next_component, path_tail,
    path_tail_with_sep, shorten_dir_len, simplify_filename, vim_ispathsep,
};
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::{
    EvalFuncData, VAR_STRING, VAR_UNKNOWN, ptrdiff_t, size_t, typval_T, varnumber_T,
};

pub unsafe extern "C" fn f_glob2regpat(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let pat: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = if pat.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            file_pat_to_reg_pat(
                pat,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0,
            )
        };
    }
}

pub unsafe extern "C" fn f_isabsolutepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = path_is_absolute(tv_get_string(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        )) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_pathshorten(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut trim_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            trim_len = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize))
                as ::core::ffi::c_int;
            if trim_len < 1 as ::core::ffi::c_int {
                trim_len = 1 as ::core::ffi::c_int;
            }
        }
        (*rettv).v_type = VAR_STRING;
        let mut p: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if p.is_null() {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            (*rettv).vval.v_string = xstrdup(p);
            shorten_dir_len((*rettv).vval.v_string, trim_len);
        };
    }
}

pub unsafe extern "C" fn f_resolve(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut is_relative_to_current: bool = false_0 != 0;
        let mut has_trailing_pathsep: bool = false_0 != 0;
        let mut limit: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = xstrdup(fname);
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && vim_ispathsep(
                        *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
        {
            is_relative_to_current = true_0 != 0;
        }
        let mut len: ptrdiff_t = strlen(p) as ptrdiff_t;
        if len > 1 as ptrdiff_t && after_pathsep(p, p.offset(len as isize)) != 0 {
            has_trailing_pathsep = true_0 != 0;
            *p.offset((len - 1 as ptrdiff_t) as isize) = NUL as ::core::ffi::c_char;
        }
        let mut q: *mut ::core::ffi::c_char = path_next_component(p) as *mut ::core::ffi::c_char;
        let mut remain: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if *q as ::core::ffi::c_int != NUL {
            remain = xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)));
            *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        let buf: *mut ::core::ffi::c_char =
            xmallocz(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut cpy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        loop {
            loop {
                len = readlink(p, buf, MAXPATHL as size_t) as ptrdiff_t;
                if len <= 0 as ptrdiff_t {
                    break;
                }
                *buf.offset(len as isize) = NUL as ::core::ffi::c_char;
                let c2rust_fresh1 = limit;
                limit = limit - 1;
                if c2rust_fresh1 == 0 as ::core::ffi::c_int {
                    xfree(p as *mut ::core::ffi::c_void);
                    xfree(remain as *mut ::core::ffi::c_void);
                    emsg(gettext(c"E655: Too many symbolic links (cycle?)".as_ptr()));
                    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    xfree(buf as *mut ::core::ffi::c_void);
                    return;
                }
                if remain.is_null() && has_trailing_pathsep as ::core::ffi::c_int != 0 {
                    add_pathsep(buf);
                }
                q = path_next_component(
                    if vim_ispathsep(*buf as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                        buf.offset(1 as ::core::ffi::c_int as isize)
                    } else {
                        buf
                    },
                ) as *mut ::core::ffi::c_char;
                if *q as ::core::ffi::c_int != NUL {
                    cpy = remain;
                    remain = if !remain.is_null() {
                        concat_str(q.offset(-(1 as ::core::ffi::c_int as isize)), remain)
                    } else {
                        xstrdup(q.offset(-(1 as ::core::ffi::c_int as isize)))
                    };
                    xfree(cpy as *mut ::core::ffi::c_void);
                    *q.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
                }
                q = path_tail(p);
                if q > p && *q as ::core::ffi::c_int == NUL {
                    *p.offset(q.offset_from(p) - 1) = NUL as ::core::ffi::c_char;
                    q = path_tail(p);
                }
                if q > p && !path_is_absolute(buf) {
                    let p_len: size_t = strlen(p);
                    let buf_len: size_t = strlen(buf);
                    p = xrealloc(
                        p as *mut ::core::ffi::c_void,
                        p_len.wrapping_add(buf_len).wrapping_add(1 as size_t),
                    ) as *mut ::core::ffi::c_char;
                    memcpy(
                        path_tail(p) as *mut ::core::ffi::c_void,
                        buf as *const ::core::ffi::c_void,
                        buf_len.wrapping_add(1 as size_t),
                    );
                } else {
                    xfree(p as *mut ::core::ffi::c_void);
                    p = xstrdup(buf);
                }
            }
            if remain.is_null() {
                break;
            }
            q = path_next_component(remain.offset(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char;
            len = (q.offset_from(remain)
                - (*q as ::core::ffi::c_int != NUL) as ::core::ffi::c_int as isize)
                as ptrdiff_t;
            let p_len_0: size_t = strlen(p);
            cpy = xmallocz(p_len_0.wrapping_add(len as size_t)) as *mut ::core::ffi::c_char;
            memcpy(
                cpy as *mut ::core::ffi::c_void,
                p as *const ::core::ffi::c_void,
                p_len_0.wrapping_add(1 as size_t),
            );
            xstrlcat(
                cpy.add(p_len_0),
                remain,
                (len as size_t).wrapping_add(1 as size_t),
            );
            xfree(p as *mut ::core::ffi::c_void);
            p = cpy;
            if *q as ::core::ffi::c_int != NUL {
                memmove(
                    remain as *mut ::core::ffi::c_void,
                    q.offset(-(1 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void,
                    strlen(q.offset(-(1 as ::core::ffi::c_int as isize))).wrapping_add(1 as size_t),
                );
            } else {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut remain as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
            }
        }
        if !vim_ispathsep(*p as ::core::ffi::c_int) {
            if is_relative_to_current as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != NUL
                && !(*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                        || vim_ispathsep(
                            *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        ) as ::core::ffi::c_int
                            != 0
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                            && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == NUL
                                || vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0)))
            {
                cpy = concat_str(c"./".as_ptr(), p);
                xfree(p as *mut ::core::ffi::c_void);
                p = cpy;
            } else if !is_relative_to_current {
                q = p;
                while *q.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && vim_ispathsep(
                        *q.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                {
                    q = q.offset(2 as ::core::ffi::c_int as isize);
                }
                if q > p {
                    memmove(
                        p as *mut ::core::ffi::c_void,
                        p.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        strlen(p.offset(2 as ::core::ffi::c_int as isize))
                            .wrapping_add(1 as size_t),
                    );
                }
            }
        }
        if !has_trailing_pathsep {
            q = p.add(strlen(p));
            if after_pathsep(p, q) != 0 {
                *path_tail_with_sep(p) = NUL as ::core::ffi::c_char;
            }
        }
        (*rettv).vval.v_string = p;
        xfree(buf as *mut ::core::ffi::c_void);
        simplify_filename((*rettv).vval.v_string);
    }
}

pub unsafe extern "C" fn f_simplify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).vval.v_string = xstrdup(p);
        simplify_filename((*rettv).vval.v_string);
        (*rettv).v_type = VAR_STRING;
    }
}
