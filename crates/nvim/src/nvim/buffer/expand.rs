//! Completing a buffer name -- `ExpandBufnames()`.
//!
//! The command-line completion side of the buffer list: match every listed
//! (or, with `!`, every) buffer against the pattern, either as a regexp or
//! with the fuzzy matcher, sort the results by score or by last-used time,
//! and return them as the completion candidates.  [`buflist_match`] and
//! [`fname_match`] are the per-buffer test it and `buflist_findpat` share,
//! and [`buflist_findnr`]/[`buflist_nr2name`] the number lookups.
//!
//! Original: `src/nvim/buffer.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::cmdexpand::cmdline_fuzzy_complete;
use crate::src::nvim::diff::diff_mode_buf;
use crate::src::nvim::fuzzy::{fuzzy_match_str, fuzzymatches_to_strmatches};
use crate::src::nvim::main::{buffer_handles, curbuf, curwin, firstbuf, p_fic, p_wic};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::libc::qsort;
use crate::src::nvim::regexp::{RE_MAGIC, vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::types::{buf_T, colnr_T, fuzmatch_str_T, regmatch_T, regprog_T, size_t};

pub unsafe extern "C" fn ExpandBufnames(
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut options: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut matches: *mut bufmatch_T = ::core::ptr::null_mut::<bufmatch_T>();
        let mut to_free: bool = false_0 != 0;
        *num_file = 0 as ::core::ffi::c_int;
        *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        if options & BUF_DIFF_FILTER as ::core::ffi::c_int != 0
            && (*curwin.get()).w_onebuf_opt.wo_diff == 0
        {
            return FAIL;
        }
        let fuzzy: bool = cmdline_fuzzy_complete(pat);
        let mut patc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fuzmatch: *mut fuzmatch_str_T = ::core::ptr::null_mut::<fuzmatch_str_T>();
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        if !fuzzy {
            if *pat as ::core::ffi::c_int == '^' as ::core::ffi::c_int
                && *pat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                patc = xstrdup(pat.offset(1 as ::core::ffi::c_int as isize));
                to_free = true_0 != 0;
            } else if *pat as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                patc = c"".as_ptr() as *mut ::core::ffi::c_char;
            } else {
                patc = pat;
            }
            regmatch.regprog = vim_regcomp(patc, RE_MAGIC);
        }
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut round: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while round <= 2 as ::core::ffi::c_int {
            count = 0 as ::core::ffi::c_int;
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                's_95: {
                    if (*buf).b_p_bl != 0 {
                        if options & BUF_DIFF_FILTER as ::core::ffi::c_int != 0 {
                            if buf == curbuf.get() || !diff_mode_buf(buf) {
                                break 's_95;
                            }
                        }
                        let mut p: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if !fuzzy {
                            if regmatch.regprog.is_null() {
                                if to_free {
                                    xfree(patc as *mut ::core::ffi::c_void);
                                }
                                return FAIL;
                            }
                            p = buflist_match(&raw mut regmatch, buf, p_wic.get() != 0);
                        } else {
                            p = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            score = fuzzy_match_str((*buf).b_sfname, pat);
                            if score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                                p = (*buf).b_sfname;
                            }
                            if p.is_null() {
                                score = fuzzy_match_str((*buf).b_ffname, pat);
                                if score != FUZZY_SCORE_NONE as ::core::ffi::c_int {
                                    p = (*buf).b_ffname;
                                }
                            }
                        }
                        if !p.is_null() {
                            if round == 1 as ::core::ffi::c_int {
                                count += 1;
                            } else {
                                if options & WILD_HOME_REPLACE as ::core::ffi::c_int != 0 {
                                    p = home_replace_save(buf, p);
                                } else {
                                    p = xstrdup(p);
                                }
                                if !fuzzy {
                                    if !matches.is_null() {
                                        (*matches.offset(count as isize)).buf = buf;
                                        (*matches.offset(count as isize)).match_0 = p;
                                        count += 1;
                                    } else {
                                        let c2rust_fresh3 = count;
                                        count = count + 1;
                                        let c2rust_lvalue_ptr =
                                            &raw mut *(*file).offset(c2rust_fresh3 as isize);
                                        *c2rust_lvalue_ptr = p;
                                    }
                                } else {
                                    (*fuzmatch.offset(count as isize)).idx = count;
                                    (*fuzmatch.offset(count as isize)).str = p;
                                    (*fuzmatch.offset(count as isize)).score = score;
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                buf = (*buf).b_next;
            }
            if count == 0 as ::core::ffi::c_int {
                break;
            }
            if round == 1 as ::core::ffi::c_int {
                if !fuzzy {
                    *file = xmalloc(
                        (count as size_t)
                            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
                    ) as *mut *mut ::core::ffi::c_char;
                    if options & WILD_BUFLASTUSED as ::core::ffi::c_int != 0 {
                        matches = xmalloc(
                            (count as size_t).wrapping_mul(::core::mem::size_of::<bufmatch_T>()),
                        ) as *mut bufmatch_T;
                    }
                } else {
                    fuzmatch = xmalloc(
                        (count as size_t).wrapping_mul(::core::mem::size_of::<fuzmatch_str_T>()),
                    ) as *mut fuzmatch_str_T;
                }
            }
            round += 1;
        }
        if !fuzzy {
            vim_regfree(regmatch.regprog);
            if to_free {
                xfree(patc as *mut ::core::ffi::c_void);
            }
        }
        if !fuzzy {
            if !matches.is_null() {
                if count > 1 as ::core::ffi::c_int {
                    qsort(
                        matches as *mut ::core::ffi::c_void,
                        count as size_t,
                        ::core::mem::size_of::<bufmatch_T>(),
                        Some(
                            buf_time_compare
                                as unsafe extern "C" fn(
                                    *const ::core::ffi::c_void,
                                    *const ::core::ffi::c_void,
                                )
                                    -> ::core::ffi::c_int,
                        ),
                    );
                }
                if (*matches.offset(0 as ::core::ffi::c_int as isize)).buf == curbuf.get() {
                    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    while i < count {
                        *(*file).offset((i - 1 as ::core::ffi::c_int) as isize) =
                            (*matches.offset(i as isize)).match_0;
                        i += 1;
                    }
                    *(*file).offset((count - 1 as ::core::ffi::c_int) as isize) =
                        (*matches.offset(0 as ::core::ffi::c_int as isize)).match_0;
                } else {
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < count {
                        *(*file).offset(i_0 as isize) = (*matches.offset(i_0 as isize)).match_0;
                        i_0 += 1;
                    }
                }
                xfree(matches as *mut ::core::ffi::c_void);
            }
        } else {
            fuzzymatches_to_strmatches(fuzmatch, file, count, false_0 != 0);
        }
        *num_file = count;
        return if count == 0 as ::core::ffi::c_int {
            FAIL
        } else {
            OK
        };
    }
}

pub(crate) unsafe extern "C" fn buflist_match(
    mut rmp: *mut regmatch_T,
    mut buf: *mut buf_T,
    mut ignore_case: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut match_0: *mut ::core::ffi::c_char = fname_match(rmp, (*buf).b_sfname, ignore_case);
        if match_0.is_null() && !(*rmp).regprog.is_null() {
            match_0 = fname_match(rmp, (*buf).b_ffname, ignore_case);
        }
        return match_0;
    }
}

unsafe extern "C" fn fname_match(
    mut rmp: *mut regmatch_T,
    mut name: *mut ::core::ffi::c_char,
    mut ignore_case: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut match_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if name.is_null() || (*rmp).regprog.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*rmp).rm_ic = p_fic.get() != 0 || ignore_case as ::core::ffi::c_int != 0;
        if vim_regexec(rmp, name, 0 as colnr_T) {
            match_0 = name;
        } else if !(*rmp).regprog.is_null() {
            let mut p: *mut ::core::ffi::c_char =
                home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
            if vim_regexec(rmp, p, 0 as colnr_T) {
                match_0 = name;
            }
            xfree(p as *mut ::core::ffi::c_void);
        }
        return match_0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn buflist_findnr(mut nr: ::core::ffi::c_int) -> *mut buf_T {
    unsafe {
        if nr == 0 as ::core::ffi::c_int {
            nr = (*curwin.get()).w_alt_fnum;
        }
        return map_get_int_ptr_t(buffer_handles.ptr(), nr) as *mut buf_T;
    }
}

pub unsafe extern "C" fn buflist_nr2name(
    mut n: ::core::ffi::c_int,
    mut fullname: ::core::ffi::c_int,
    mut helptail: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut buf: *mut buf_T = buflist_findnr(n);
        if buf.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return home_replace_save(
            if helptail != 0 {
                buf
            } else {
                ::core::ptr::null_mut::<buf_T>()
            },
            if fullname != 0 {
                (*buf).b_ffname
            } else {
                (*buf).b_fname
            },
        );
    }
}
