//! Shortening a list of names to the shortest ones that still differ.
//!
//! Command-line completion of file names shows the tails rather than whole
//! paths, and [`uniquefy_paths`] is what decides how much tail each entry
//! needs: the shortest suffix that no other entry shares, extended by whole
//! components until it is unique. [`path_shorten_fname`] and
//! [`shorten_dir_len`] are the simpler shortenings — relative to a directory,
//! and one letter per component — that `'shortmess'` and the status line
//! use.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn shorten_dir_len(
    mut str: *mut ::core::ffi::c_char,
    mut trim_len: ::core::ffi::c_int,
) {
    unsafe {
        let mut tail: *mut ::core::ffi::c_char = path_tail(str);
        let mut d: *mut ::core::ffi::c_char = str;
        let mut skip: bool = false_0 != 0;
        let mut dirchunk_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = str;
        loop {
            if s >= tail {
                let c2rust_fresh0 = d;
                d = d.offset(1);
                *c2rust_fresh0 = *s;
                if *s as ::core::ffi::c_int == NUL {
                    break;
                }
            } else if vim_ispathsep(*s as ::core::ffi::c_int) {
                let c2rust_fresh1 = d;
                d = d.offset(1);
                *c2rust_fresh1 = *s;
                skip = false_0 != 0;
                dirchunk_len = 0 as ::core::ffi::c_int;
            } else if !skip {
                let c2rust_fresh2 = d;
                d = d.offset(1);
                *c2rust_fresh2 = *s;
                if *s as ::core::ffi::c_int != '~' as ::core::ffi::c_int
                    && *s as ::core::ffi::c_int != '.' as ::core::ffi::c_int
                {
                    dirchunk_len += 1;
                    if dirchunk_len >= trim_len {
                        skip = true_0 != 0;
                    }
                }
                let mut l: ::core::ffi::c_int = utfc_ptr2len(s);
                loop {
                    l -= 1;
                    if l <= 0 as ::core::ffi::c_int {
                        break;
                    }
                    s = s.offset(1);
                    let c2rust_fresh3 = d;
                    d = d.offset(1);
                    *c2rust_fresh3 = *s;
                }
            }
            s = s.offset(1);
        }
    }
}

pub unsafe extern "C" fn shorten_dir(mut str: *mut ::core::ffi::c_char) {
    unsafe {
        shorten_dir_len(str, 1 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn find_previous_pathsep(
    mut path: *mut ::core::ffi::c_char,
    mut psep: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if *psep > path && vim_ispathsep(**psep as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
            *psep = (*psep).offset(-1);
        }
        while *psep > path {
            if vim_ispathsep(**psep as ::core::ffi::c_int) {
                return OK;
            }
            *psep = (*psep).offset(
                -((utf_head_off(path, (*psep).offset(-(1 as ::core::ffi::c_int as isize)))
                    + 1 as ::core::ffi::c_int) as isize),
            );
        }
        return FAIL;
    }
}

pub(crate) unsafe extern "C" fn is_unique(
    mut maybe_unique: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
    mut i: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut candidate_len: size_t = strlen(maybe_unique);
        let mut other_paths: *mut *mut ::core::ffi::c_char =
            (*gap).ga_data as *mut *mut ::core::ffi::c_char;
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < (*gap).ga_len {
            if j != i {
                let mut other_path_len: size_t = strlen(*other_paths.offset(j as isize));
                if other_path_len >= candidate_len {
                    let mut rival: *mut ::core::ffi::c_char = (*other_paths.offset(j as isize))
                        .offset(other_path_len as isize)
                        .offset(-(candidate_len as isize));
                    if path_fnamecmp(maybe_unique, rival) == 0 as ::core::ffi::c_int
                        && (rival == *other_paths.offset(j as isize)
                            || vim_ispathsep(*rival.offset(-(1 as ::core::ffi::c_int as isize))
                                as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0)
                    {
                        return false_0 != 0;
                    }
                }
            }
            j += 1;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_path_cutoff(
    mut fname: *mut ::core::ffi::c_char,
    mut gap: *mut garray_T,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut maxlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut path_part: *mut *mut ::core::ffi::c_char =
            (*gap).ga_data as *mut *mut ::core::ffi::c_char;
        let mut cutoff: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while *fname.offset(j as isize) as ::core::ffi::c_int
                == *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
                && *fname.offset(j as isize) as ::core::ffi::c_int != NUL
                && *(*path_part.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != NUL
            {
                j += 1;
            }
            if j > maxlen {
                maxlen = j;
                cutoff = fname.offset(j as isize);
            }
            i += 1;
        }
        if !cutoff.is_null() {
            while vim_ispathsep(*cutoff as ::core::ffi::c_int) {
                cutoff = cutoff.offset(utfc_ptr2len(cutoff) as isize);
            }
        }
        return cutoff;
    }
}

pub(crate) unsafe extern "C" fn uniquefy_paths(
    mut gap: *mut garray_T,
    mut pattern: *mut ::core::ffi::c_char,
    mut path_option: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut fnames: *mut *mut ::core::ffi::c_char =
            (*gap).ga_data as *mut *mut ::core::ffi::c_char;
        let mut sort_again: bool = false_0 != 0;
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut path_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut in_curdir: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut short_name: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        ga_remove_duplicate_strings(gap);
        ga_init(
            &raw mut path_ga,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        let mut len: size_t = strlen(pattern);
        let mut file_pattern: *mut ::core::ffi::c_char =
            xmalloc(len.wrapping_add(2 as size_t)) as *mut ::core::ffi::c_char;
        *file_pattern.offset(0 as ::core::ffi::c_int as isize) = '*' as ::core::ffi::c_char;
        *file_pattern.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        strcpy(
            file_pattern.offset(1 as ::core::ffi::c_int as isize),
            pattern,
        );
        let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            file_pattern,
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        xfree(file_pattern as *mut ::core::ffi::c_void);
        if pat.is_null() {
            return;
        }
        regmatch.rm_ic = true_0 != 0;
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        xfree(pat as *mut ::core::ffi::c_void);
        if regmatch.regprog.is_null() {
            return;
        }
        let mut curdir: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        os_dirname(curdir, MAXPATHL as size_t);
        expand_path_option(curdir, path_option, &raw mut path_ga);
        in_curdir = xcalloc(
            (*gap).ga_len as size_t,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
        ) as *mut *mut ::core::ffi::c_char;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len && !got_int.get() {
            let mut path: *mut ::core::ffi::c_char = *fnames.offset(i as isize);
            let mut dir_end: *const ::core::ffi::c_char = gettail_dir(path);
            len = strlen(path);
            let mut is_in_curdir: bool =
                path_fnamencmp(curdir, path, dir_end.offset_from(path) as size_t)
                    == 0 as ::core::ffi::c_int
                    && *curdir.offset(dir_end.offset_from(path) as isize) as ::core::ffi::c_int
                        == NUL;
            if is_in_curdir {
                *in_curdir.offset(i as isize) =
                    xmemdupz(path as *const ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
            }
            let mut path_cutoff: *mut ::core::ffi::c_char = get_path_cutoff(path, &raw mut path_ga);
            if *pattern.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                && *pattern.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                && vim_ispathsep_nocolon(
                    *pattern.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                ) as ::core::ffi::c_int
                    != 0
                && !path_cutoff.is_null()
                && vim_regexec(&raw mut regmatch, path_cutoff, 0 as colnr_T) as ::core::ffi::c_int
                    != 0
                && is_unique(path_cutoff, gap, i) as ::core::ffi::c_int != 0
            {
                sort_again = true_0 != 0;
                memmove(
                    path as *mut ::core::ffi::c_void,
                    path_cutoff as *const ::core::ffi::c_void,
                    strlen(path_cutoff).wrapping_add(1 as size_t),
                );
            } else {
                let mut pathsep_p: *mut ::core::ffi::c_char = path
                    .offset(len as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize));
                while find_previous_pathsep(path, &raw mut pathsep_p) != 0 {
                    if !(vim_regexec(
                        &raw mut regmatch,
                        pathsep_p.offset(1 as ::core::ffi::c_int as isize),
                        0 as colnr_T,
                    ) as ::core::ffi::c_int
                        != 0
                        && is_unique(pathsep_p.offset(1 as ::core::ffi::c_int as isize), gap, i)
                            as ::core::ffi::c_int
                            != 0
                        && !path_cutoff.is_null()
                        && pathsep_p.offset(1 as ::core::ffi::c_int as isize) >= path_cutoff)
                    {
                        continue;
                    }
                    sort_again = true_0 != 0;
                    memmove(
                        path as *mut ::core::ffi::c_void,
                        pathsep_p.offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        (path
                            .offset(len as isize)
                            .offset_from(pathsep_p.offset(1 as ::core::ffi::c_int as isize))
                            as size_t)
                            .wrapping_add(1 as size_t),
                    );
                    break;
                }
            }
            if path_is_absolute(path) {
                short_name = path_shorten_fname(path, curdir);
                if !short_name.is_null()
                    && short_name > path.offset(1 as ::core::ffi::c_int as isize)
                {
                    vim_snprintf(
                        path,
                        MAXPATHL as size_t,
                        b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                        PATHSEPSTR.as_ptr(),
                        short_name,
                    );
                }
            }
            os_breakcheck();
            i += 1;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*gap).ga_len && !got_int.get() {
            let mut path_0: *mut ::core::ffi::c_char = *in_curdir.offset(i_0 as isize);
            if !path_0.is_null() {
                short_name = path_shorten_fname(path_0, curdir);
                if short_name.is_null() {
                    short_name = path_0;
                }
                if is_unique(short_name, gap, i_0) {
                    strcpy(*fnames.offset(i_0 as isize), short_name);
                } else {
                    let mut rel_pathsize: size_t = (1 as size_t)
                        .wrapping_add(
                            ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        )
                        .wrapping_add(strlen(short_name))
                        .wrapping_add(1 as size_t);
                    let mut rel_path: *mut ::core::ffi::c_char =
                        xmalloc(rel_pathsize) as *mut ::core::ffi::c_char;
                    vim_snprintf(
                        rel_path,
                        rel_pathsize,
                        b".%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                        PATHSEPSTR.as_ptr(),
                        short_name,
                    );
                    xfree(*fnames.offset(i_0 as isize) as *mut ::core::ffi::c_void);
                    *fnames.offset(i_0 as isize) = rel_path;
                    sort_again = true_0 != 0;
                    os_breakcheck();
                }
            }
            i_0 += 1;
        }
        xfree(curdir as *mut ::core::ffi::c_void);
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < (*gap).ga_len {
            xfree(*in_curdir.offset(i_1 as isize) as *mut ::core::ffi::c_void);
            i_1 += 1;
        }
        xfree(in_curdir as *mut ::core::ffi::c_void);
        ga_clear_strings(&raw mut path_ga);
        vim_regfree(regmatch.regprog);
        if sort_again {
            ga_remove_duplicate_strings(gap);
        }
    }
}

pub unsafe extern "C" fn gettail_dir(
    fname: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut dir_end: *const ::core::ffi::c_char = fname;
        let mut next_dir_end: *const ::core::ffi::c_char = fname;
        let mut look_for_sep: bool = true_0 != 0;
        let mut p: *const ::core::ffi::c_char = fname;
        while *p as ::core::ffi::c_int != NUL {
            if vim_ispathsep(*p as ::core::ffi::c_int) {
                if look_for_sep {
                    next_dir_end = p;
                    look_for_sep = false_0 != 0;
                }
            } else {
                if !look_for_sep {
                    dir_end = next_dir_end;
                }
                look_for_sep = true_0 != 0;
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
        return dir_end;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_try_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut dirname: *mut ::core::ffi::c_char =
            xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = full_path;
        if os_dirname(dirname, MAXPATHL as size_t) == OK {
            p = path_shorten_fname(full_path, dirname);
            if p.is_null() || *p as ::core::ffi::c_int == NUL {
                p = full_path;
            }
        }
        xfree(dirname as *mut ::core::ffi::c_void);
        return p;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_shorten_fname(
    mut full_path: *mut ::core::ffi::c_char,
    mut dir_name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if full_path.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        '_c2rust_label: {
            if !dir_name.is_null() {
            } else {
                __assert_fail(
                    b"dir_name != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2108 as ::core::ffi::c_uint,
                    b"char *path_shorten_fname(char *, char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut len: size_t = strlen(dir_name);
        if path_fnamencmp(dir_name, full_path, len) != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if len == path_head_length() as size_t && is_path_head(dir_name) as ::core::ffi::c_int != 0
        {
            return full_path.offset(len as isize);
        }
        let mut p: *mut ::core::ffi::c_char = full_path.offset(len as isize);
        if !vim_ispathsep(*p as ::core::ffi::c_int) {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        loop {
            p = p.offset(1);
            if !vim_ispathsep_nocolon(*p as ::core::ffi::c_int) {
                break;
            }
        }
        return p;
    }
}
