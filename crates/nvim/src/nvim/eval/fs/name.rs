//! Rewriting a file name as text -- `fnamemodify()` and the `:h` `:t` `:r`
//! `:e` `:p` `:~` `:.` `:s?` `:gs?` modifier language it shares with `%:h` and
//! friends on the command line.
//!
//! `modify_fname` is the whole modifier alphabet in one 331-line loop over a
//! `*mut char` cursor: it expands to a full path, strips head, tail, root and
//! extension, makes the name relative to the home directory or the current
//! one, and runs `:s` substitutions over the result, appending each stage to a
//! caller-owned buffer.  Nothing here touches the filesystem except `:p`, which
//! has to resolve the name to say whether it is a directory.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{MAXPATHL, NUL, VALID_HEAD, VALID_PATH, false_0, true_0};
use crate::src::nvim::eval::do_string_sub;
use crate::src::nvim::eval::typval::{tv_get_string_buf_chk, tv_get_string_chk};
use crate::src::nvim::mbyte::{utf_head_off, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmemdupz, xstrdup};
use crate::src::nvim::os::env::{expand_env_save, home_replace};
use crate::src::nvim::os::fs::{os_dirname, os_isdir};
use crate::src::nvim::os::libc::strlen;
use crate::src::nvim::path::{
    FullName_save, add_pathsep, after_pathsep, get_past_head, path_fnamencmp, path_tail,
    vim_isAbsName, vim_ispathsep,
};
use crate::src::nvim::strings::{vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::src::nvim::types::{EvalFuncData, VAR_STRING, buf_T, size_t, typval_T, uint8_t};

pub unsafe extern "C" fn modify_fname(
    mut src: *mut ::core::ffi::c_char,
    mut tilde_file: bool,
    mut usedlen: *mut size_t,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut fnamelen: *mut size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut valid: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut pbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut has_fullname: bool = false_0 != 0;
        let mut has_homerelative: bool = false_0 != 0;
        loop {
            if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'p' as ::core::ffi::c_int
            {
                has_fullname = true_0 != 0;
                valid |= VALID_PATH as ::core::ffi::c_int;
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
                if *(*fnamep).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '~' as ::core::ffi::c_int
                    && !(tilde_file as ::core::ffi::c_int != 0
                        && *(*fnamep).offset(1 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == NUL)
                {
                    *fnamep = expand_env_save(*fnamep);
                    xfree(*bufp as *mut ::core::ffi::c_void);
                    *bufp = *fnamep;
                    if (*fnamep).is_null() {
                        return -1 as ::core::ffi::c_int;
                    }
                }
                p = *fnamep;
                while *p as ::core::ffi::c_int != NUL {
                    if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                        && (*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL
                            || vim_ispathsep(
                                *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                            || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                                && (*p.offset(3 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL
                                    || vim_ispathsep(*p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0))
                    {
                        break;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
                if *p as ::core::ffi::c_int != NUL || !vim_isAbsName(*fnamep) {
                    *fnamep = FullName_save(*fnamep, *p as ::core::ffi::c_int != NUL);
                    xfree(*bufp as *mut ::core::ffi::c_void);
                    *bufp = *fnamep;
                    if (*fnamep).is_null() {
                        return -1 as ::core::ffi::c_int;
                    }
                }
                if os_isdir(*fnamep) {
                    *fnamep = xstrnsave(*fnamep, strlen(*fnamep).wrapping_add(2 as size_t));
                    xfree(*bufp as *mut ::core::ffi::c_void);
                    *bufp = *fnamep;
                    add_pathsep(*fnamep);
                }
            }
            c = 0;
            while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int && {
                c = *src.add((*usedlen).wrapping_add(1 as size_t)) as uint8_t as ::core::ffi::c_int;
                c == '.' as ::core::ffi::c_int
                    || c == '~' as ::core::ffi::c_int
                    || c == '8' as ::core::ffi::c_int
            } {
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
                if c == '8' as ::core::ffi::c_int {
                    continue;
                }
                pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                if !has_fullname && !has_homerelative {
                    if **fnamep as ::core::ffi::c_int == '~' as ::core::ffi::c_int {
                        pbuf = expand_env_save(*fnamep);
                        p = pbuf;
                    } else {
                        pbuf = FullName_save(*fnamep, false_0 != 0);
                        p = pbuf;
                    }
                } else {
                    p = *fnamep;
                }
                has_fullname = false_0 != 0;
                if !p.is_null() {
                    if c == '.' as ::core::ffi::c_int {
                        os_dirname(
                            &raw mut dirname as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                        );
                        if has_homerelative {
                            s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                            home_replace(
                                ::core::ptr::null::<buf_T>(),
                                s,
                                &raw mut dirname as *mut ::core::ffi::c_char,
                                MAXPATHL as size_t,
                                true_0 != 0,
                            );
                            xfree(s as *mut ::core::ffi::c_void);
                        }
                        let mut namelen: size_t =
                            strlen(&raw mut dirname as *mut ::core::ffi::c_char);
                        if path_fnamencmp(p, &raw mut dirname as *mut ::core::ffi::c_char, namelen)
                            == 0 as ::core::ffi::c_int
                        {
                            p = p.add(namelen);
                            if vim_ispathsep(*p as ::core::ffi::c_int) {
                                while *p as ::core::ffi::c_int != 0
                                    && vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                        != 0
                                {
                                    p = p.offset(1);
                                }
                                *fnamep = p;
                                if !pbuf.is_null() {
                                    xfree(*bufp as *mut ::core::ffi::c_void);
                                    *bufp = pbuf;
                                    pbuf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                }
                            }
                        }
                    } else {
                        home_replace(
                            ::core::ptr::null::<buf_T>(),
                            p,
                            &raw mut dirname as *mut ::core::ffi::c_char,
                            MAXPATHL as size_t,
                            true_0 != 0,
                        );
                        if *(&raw mut dirname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                            == '~' as ::core::ffi::c_int
                        {
                            s = xstrdup(&raw mut dirname as *mut ::core::ffi::c_char);
                            debug_assert!(!s.is_null(), "s != NULL");
                            *fnamep = s;
                            xfree(*bufp as *mut ::core::ffi::c_void);
                            *bufp = s;
                            has_homerelative = true_0 != 0;
                        }
                    }
                    xfree(pbuf as *mut ::core::ffi::c_void);
                }
            }
            tail = path_tail(*fnamep);
            *fnamelen = strlen(*fnamep);
            while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'h' as ::core::ffi::c_int
            {
                valid |= VALID_HEAD as ::core::ffi::c_int;
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
                s = get_past_head(*fnamep);
                while tail > s && after_pathsep(s, tail) != 0 {
                    tail = tail.offset(
                        -((utf_head_off(*fnamep, tail.offset(-(1 as ::core::ffi::c_int as isize)))
                            + 1 as ::core::ffi::c_int) as isize),
                    );
                }
                *fnamelen = tail.offset_from(*fnamep) as size_t;
                if *fnamelen == 0 as size_t {
                    xfree(*bufp as *mut ::core::ffi::c_void);
                    tail = xstrdup(c".".as_ptr());
                    *fnamep = tail;
                    *bufp = *fnamep;
                    *fnamelen = 1 as size_t;
                } else {
                    while tail > s && after_pathsep(s, tail) == 0 {
                        tail = tail.offset(
                            -((utf_head_off(
                                *fnamep,
                                tail.offset(-(1 as ::core::ffi::c_int as isize)),
                            ) + 1 as ::core::ffi::c_int) as isize),
                        );
                    }
                }
            }
            if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == '8' as ::core::ffi::c_int
            {
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
            }
            if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 't' as ::core::ffi::c_int
            {
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
                *fnamelen = (*fnamelen).wrapping_sub(tail.offset_from(*fnamep) as size_t);
                *fnamep = tail;
            }
            while *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && (*src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'e' as ::core::ffi::c_int
                    || *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                        == 'r' as ::core::ffi::c_int)
            {
                let is_second_e: bool = *fnamep > tail;
                if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'e' as ::core::ffi::c_int
                    && is_second_e as ::core::ffi::c_int != 0
                {
                    s = (*fnamep).offset(-(2 as ::core::ffi::c_int as isize));
                } else {
                    s = (*fnamep)
                        .add(*fnamelen)
                        .offset(-(1 as ::core::ffi::c_int as isize));
                }
                while s > tail {
                    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                    {
                        break;
                    }
                    s = s.offset(-1);
                }
                if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 'e' as ::core::ffi::c_int
                {
                    if s > tail || false && is_second_e as ::core::ffi::c_int != 0 && s == tail {
                        let mut newstart: *mut ::core::ffi::c_char =
                            s.offset(1 as ::core::ffi::c_int as isize);
                        let mut distance_stepped_back: size_t =
                            (*fnamep).offset_from(newstart) as size_t;
                        *fnamelen = (*fnamelen).wrapping_add(distance_stepped_back);
                        *fnamep = newstart;
                    } else if *fnamep <= tail {
                        *fnamelen = 0 as size_t;
                    }
                } else if s > (if tail > *fnamep { tail } else { *fnamep }) {
                    *fnamelen = s.offset_from(*fnamep) as size_t;
                }
                *usedlen = (*usedlen).wrapping_add(2 as size_t);
            }
            if !(*src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                && (*src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                    == 's' as ::core::ffi::c_int
                    || *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                        == 'g' as ::core::ffi::c_int
                        && *src.add((*usedlen).wrapping_add(2 as size_t)) as ::core::ffi::c_int
                            == 's' as ::core::ffi::c_int))
            {
                break;
            }
            let mut didit: bool = false_0 != 0;
            let mut flags: *mut ::core::ffi::c_char = c"".as_ptr() as *mut ::core::ffi::c_char;
            s = src.add(*usedlen).offset(2 as ::core::ffi::c_int as isize);
            if *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'g' as ::core::ffi::c_int
            {
                flags = c"g".as_ptr() as *mut ::core::ffi::c_char;
                s = s.offset(1);
            }
            let c2rust_fresh0 = s;
            s = s.offset(1);
            let mut sep: ::core::ffi::c_int = *c2rust_fresh0 as uint8_t as ::core::ffi::c_int;
            if sep == 0 {
                break;
            }
            p = vim_strchr(s, sep);
            if !p.is_null() {
                let pat: *mut ::core::ffi::c_char =
                    xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                        as *mut ::core::ffi::c_char;
                s = p.offset(1 as ::core::ffi::c_int as isize);
                p = vim_strchr(s, sep);
                if !p.is_null() {
                    let sub: *mut ::core::ffi::c_char =
                        xmemdupz(s as *const ::core::ffi::c_void, p.offset_from(s) as size_t)
                            as *mut ::core::ffi::c_char;
                    let str: *mut ::core::ffi::c_char =
                        xmemdupz(*fnamep as *const ::core::ffi::c_void, *fnamelen)
                            as *mut ::core::ffi::c_char;
                    *usedlen =
                        p.offset(1 as ::core::ffi::c_int as isize).offset_from(src) as size_t;
                    let mut slen: size_t = 0;
                    s = do_string_sub(
                        str,
                        *fnamelen,
                        pat,
                        sub,
                        ::core::ptr::null_mut::<typval_T>(),
                        flags,
                        &raw mut slen,
                    );
                    *fnamep = s;
                    *fnamelen = slen;
                    xfree(*bufp as *mut ::core::ffi::c_void);
                    *bufp = s;
                    didit = true_0 != 0;
                    xfree(sub as *mut ::core::ffi::c_void);
                    xfree(str as *mut ::core::ffi::c_void);
                }
                xfree(pat as *mut ::core::ffi::c_void);
            }
            if !didit {
                break;
            }
        }
        if *src.add(*usedlen) as ::core::ffi::c_int == ':' as ::core::ffi::c_int
            && *src.add((*usedlen).wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == 'S' as ::core::ffi::c_int
        {
            c = *(*fnamep).add(*fnamelen) as uint8_t as ::core::ffi::c_int;
            if c != NUL {
                *(*fnamep).add(*fnamelen) = NUL as ::core::ffi::c_char;
            }
            p = vim_strsave_shellescape(*fnamep, false_0 != 0, false_0 != 0);
            if c != NUL {
                *(*fnamep).add(*fnamelen) = c as ::core::ffi::c_char;
            }
            xfree(*bufp as *mut ::core::ffi::c_void);
            *fnamep = p;
            *bufp = *fnamep;
            *fnamelen = strlen(p);
            *usedlen = (*usedlen).wrapping_add(2 as size_t);
        }
        return valid;
    }
}

pub unsafe extern "C" fn f_fnamemodify(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut len: size_t = 0 as size_t;
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mods: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if mods.is_null() || fname.is_null() {
            fname = ::core::ptr::null::<::core::ffi::c_char>();
        } else {
            len = strlen(fname);
            if *mods as ::core::ffi::c_int != NUL {
                let mut usedlen: size_t = 0 as size_t;
                modify_fname(
                    mods as *mut ::core::ffi::c_char,
                    false_0 != 0,
                    &raw mut usedlen,
                    &raw mut fname as *mut *mut ::core::ffi::c_char,
                    &raw mut fbuf,
                    &raw mut len,
                );
            }
        }
        (*rettv).v_type = VAR_STRING;
        if fname.is_null() {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            (*rettv).vval.v_string =
                xmemdupz(fname as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        }
        xfree(fbuf as *mut ::core::ffi::c_void);
    }
}
