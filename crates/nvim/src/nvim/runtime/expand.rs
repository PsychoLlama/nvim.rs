//! Command-line completion for the runtime commands.
//!
//! `ExpandRTDir` completes a file name against a set of 'runtimepath'
//! subdirectories -- what `:colorscheme`, `:compiler`, `:runtime` and friends
//! offer -- and `ExpandPackAddDir` does the same for `:packadd` against
//! 'packpath'.  `expand_runtime_cmd` is `:runtime`'s own two-stage
//! completion, where the first word may be one of the `START`/`OPT`/`PACK`/
//! `ALL` qualifiers and everything after it is a path.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn ExpandRTDir_int(
    mut pat: *mut ::core::ffi::c_char,
    mut pat_len: size_t,
    mut flags: ::core::ffi::c_int,
    mut keep_ext: bool,
    mut gap: *mut garray_T,
    mut dirnames: *mut *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*dirnames.offset(i as isize)).is_null() {
            let buf_len: size_t = strlen(*dirnames.offset(i as isize))
                .wrapping_add(pat_len)
                .wrapping_add(64 as size_t);
            let mut buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
            let mut glob_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut expand_dirs: bool = false_0 != 0;
            snprintf(
                buf,
                buf_len,
                c"%s%s%s%s".as_ptr(),
                if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                    *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                } else {
                    c"".as_ptr()
                },
                if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                    c"/".as_ptr()
                } else {
                    c"".as_ptr()
                },
                pat,
                c"*.{vim,lua}".as_ptr(),
            );
            loop {
                if flags & DIP_NORTP as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    globpath(p_rtp.get(), buf, gap, glob_flags, expand_dirs);
                }
                if flags & DIP_START as ::core::ffi::c_int != 0 {
                    snprintf(
                        buf,
                        buf_len,
                        c"pack/*/start/*/%s%s%s%s".as_ptr(),
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                        } else {
                            c"".as_ptr()
                        },
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            c"/".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                        pat,
                        if expand_dirs as ::core::ffi::c_int != 0 {
                            c"*".as_ptr()
                        } else {
                            c"*.{vim,lua}".as_ptr()
                        },
                    );
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                    snprintf(
                        buf,
                        buf_len,
                        c"start/*/%s%s%s%s".as_ptr(),
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                        } else {
                            c"".as_ptr()
                        },
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            c"/".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                        pat,
                        if expand_dirs as ::core::ffi::c_int != 0 {
                            c"*".as_ptr()
                        } else {
                            c"*.{vim,lua}".as_ptr()
                        },
                    );
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                }
                if flags & DIP_OPT as ::core::ffi::c_int != 0 {
                    snprintf(
                        buf,
                        buf_len,
                        c"pack/*/opt/*/%s%s%s%s".as_ptr(),
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                        } else {
                            c"".as_ptr()
                        },
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            c"/".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                        pat,
                        if expand_dirs as ::core::ffi::c_int != 0 {
                            c"*".as_ptr()
                        } else {
                            c"*.{vim,lua}".as_ptr()
                        },
                    );
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                    snprintf(
                        buf,
                        buf_len,
                        c"opt/*/%s%s%s%s".as_ptr(),
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            *dirnames.offset(i as isize) as *const ::core::ffi::c_char
                        } else {
                            c"".as_ptr()
                        },
                        if **dirnames.offset(i as isize) as ::core::ffi::c_int != 0 {
                            c"/".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                        pat,
                        if expand_dirs as ::core::ffi::c_int != 0 {
                            c"*".as_ptr()
                        } else {
                            c"*.{vim,lua}".as_ptr()
                        },
                    );
                    globpath(p_pp.get(), buf, gap, glob_flags, expand_dirs);
                }
                if !(**dirnames.offset(i as isize) as ::core::ffi::c_int == NUL && !expand_dirs) {
                    break;
                }
                snprintf(buf, buf_len, c"%s*".as_ptr(), pat);
                glob_flags = WILD_ADD_SLASH;
                expand_dirs = true_0 != 0;
            }
            xfree(buf as *mut ::core::ffi::c_void);
            i += 1;
        }
        let mut pat_pathsep_cnt: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i_0: size_t = 0 as size_t;
        while i_0 < pat_len {
            if vim_ispathsep(*pat.add(i_0) as ::core::ffi::c_int) {
                pat_pathsep_cnt += 1;
            }
            i_0 = i_0.wrapping_add(1);
        }
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < (*gap).ga_len {
            let mut match_0: *mut ::core::ffi::c_char =
                *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(i_1 as isize);
            let mut s: *mut ::core::ffi::c_char = match_0;
            let mut e: *mut ::core::ffi::c_char = s.add(strlen(s));
            if e.offset_from(s) > 4_isize
                && !keep_ext
                && (strncasecmp(
                    e.offset(-(4 as ::core::ffi::c_int as isize)),
                    c".vim".as_ptr() as *mut ::core::ffi::c_char,
                    4 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
                    || strncasecmp(
                        e.offset(-(4 as ::core::ffi::c_int as isize)),
                        c".lua".as_ptr() as *mut ::core::ffi::c_char,
                        4 as ::core::ffi::c_int as size_t,
                    ) == 0 as ::core::ffi::c_int)
            {
                e = e.offset(-(4 as ::core::ffi::c_int as isize));
                *e = NUL as ::core::ffi::c_char;
            }
            let mut match_pathsep_cnt: ::core::ffi::c_int = if e > s
                && *e.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '/' as ::core::ffi::c_int
            {
                -1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            };
            s = e;
            while s > match_0 {
                if vim_ispathsep(*s as ::core::ffi::c_int) as ::core::ffi::c_int != 0 && {
                    match_pathsep_cnt += 1;
                    match_pathsep_cnt > pat_pathsep_cnt
                } {
                    break;
                }
                s = s.offset(
                    -((utf_head_off(match_0, s.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
            s = s.offset(1);
            if s != match_0 {
                debug_assert!(e.offset_from(s) + 1_isize >= 0_isize, "(e - s) + 1 >= 0");
                memmove(
                    match_0 as *mut ::core::ffi::c_void,
                    s as *const ::core::ffi::c_void,
                    (e.offset_from(s) as size_t).wrapping_add(1 as size_t),
                );
            }
            i_1 += 1;
        }
        if (*gap).ga_len <= 0 as ::core::ffi::c_int {
            return;
        }
        ga_remove_duplicate_strings(gap);
    }
}

pub unsafe extern "C" fn ExpandRTDir(
    mut pat: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
    mut dirnames: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        *num_file = 0 as ::core::ffi::c_int;
        *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        ExpandRTDir_int(pat, strlen(pat), flags, false_0 != 0, &raw mut ga, dirnames);
        if ga.ga_len <= 0 as ::core::ffi::c_int {
            return FAIL;
        }
        *file = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *num_file = ga.ga_len;
        return OK;
    }
}

pub unsafe extern "C" fn expand_runtime_cmd(
    mut pat: *mut ::core::ffi::c_char,
    mut numMatches: *mut ::core::ffi::c_int,
    mut matches: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        *numMatches = 0 as ::core::ffi::c_int;
        *matches = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        let pat_len: size_t = strlen(pat);
        let mut dirnames: [*mut ::core::ffi::c_char; 2] = [
            c"".as_ptr() as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ];
        ExpandRTDir_int(
            pat,
            pat_len,
            runtime_expand_flags.get(),
            true_0 != 0,
            &raw mut ga,
            &raw mut dirnames as *mut *mut ::core::ffi::c_char,
        );
        if runtime_expand_flags.get() == 0 as ::core::ffi::c_int {
            let mut where_values: [*mut ::core::ffi::c_char; 4] = [
                c"START".as_ptr() as *mut ::core::ffi::c_char,
                c"OPT".as_ptr() as *mut ::core::ffi::c_char,
                c"PACK".as_ptr() as *mut ::core::ffi::c_char,
                c"ALL".as_ptr() as *mut ::core::ffi::c_char,
            ];
            let mut i: size_t = 0 as size_t;
            while i < ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>()
                        .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                )
            {
                if strncmp(pat, where_values[i as usize], pat_len) == 0 as ::core::ffi::c_int {
                    ga_grow(&raw mut ga, 1 as ::core::ffi::c_int);
                    *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(ga.ga_len as isize) =
                        xstrdup(where_values[i as usize]);
                    ga.ga_len += 1;
                }
                i = i.wrapping_add(1);
            }
        }
        if ga.ga_len <= 0 as ::core::ffi::c_int {
            return FAIL;
        }
        *matches = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *numMatches = ga.ga_len;
        return OK;
    }
}

pub unsafe extern "C" fn ExpandPackAddDir(
    mut pat: *mut ::core::ffi::c_char,
    mut num_file: *mut ::core::ffi::c_int,
    mut file: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        *num_file = 0 as ::core::ffi::c_int;
        *file = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut pat_len: size_t = strlen(pat);
        ga_init(
            &raw mut ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
        let mut buflen: size_t = pat_len.wrapping_add(26 as size_t);
        let mut s: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
        snprintf(s, buflen, c"pack/*/opt/%s*".as_ptr(), pat);
        globpath(
            p_pp.get(),
            s,
            &raw mut ga,
            0 as ::core::ffi::c_int,
            true_0 != 0,
        );
        snprintf(s, buflen, c"opt/%s*".as_ptr(), pat);
        globpath(
            p_pp.get(),
            s,
            &raw mut ga,
            0 as ::core::ffi::c_int,
            true_0 != 0,
        );
        xfree(s as *mut ::core::ffi::c_void);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < ga.ga_len {
            let mut match_0: *mut ::core::ffi::c_char =
                *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            s = path_tail(match_0);
            memmove(
                match_0 as *mut ::core::ffi::c_void,
                s as *const ::core::ffi::c_void,
                strlen(s).wrapping_add(1 as size_t),
            );
            i += 1;
        }
        if ga.ga_len <= 0 as ::core::ffi::c_int {
            return FAIL;
        }
        ga_remove_duplicate_strings(&raw mut ga);
        *file = ga.ga_data as *mut *mut ::core::ffi::c_char;
        *num_file = ga.ga_len;
        return OK;
    }
}
