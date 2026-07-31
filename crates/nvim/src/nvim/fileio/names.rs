//! Turning one file name into another.
//!
//! `modname` builds the "same name with a different extension" that backups,
//! swap files and `:make` want, honouring `'shortname'` and the 8.3-ish limits
//! that `BASENAMELEN` still encodes. `vim_rename` and `vim_copyfile` move a
//! file, falling back from `rename` to a copy when the two paths are on
//! different filesystems. `file_pat_to_reg_pat` compiles a shell-style file
//! pattern into a regexp, which `match_file_pat`/`match_file_list` then run
//! against a name — that is how `'wildignore'`, `'backupskip'` and autocommand
//! patterns are matched.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn shorten_buf_fname(
    mut buf: *mut buf_T,
    mut dirname: *mut ::core::ffi::c_char,
    mut force: ::core::ffi::c_int,
) {
    unsafe {
        if !(*buf).b_fname.is_null()
            && !bt_nofilename(buf)
            && path_with_url((*buf).b_fname) == 0
            && (force != 0
                || (*buf).b_sfname.is_null()
                || path_is_absolute((*buf).b_sfname) as ::core::ffi::c_int != 0)
        {
            if (*buf).b_sfname != (*buf).b_ffname {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut (*buf).b_sfname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
            }
            let mut p: *mut ::core::ffi::c_char = path_shorten_fname((*buf).b_ffname, dirname);
            if !p.is_null() {
                (*buf).b_sfname = xstrdup(p);
                (*buf).b_fname = (*buf).b_sfname;
            }
            if p.is_null() {
                (*buf).b_fname = (*buf).b_ffname;
            }
        }
    }
}

pub unsafe extern "C" fn shorten_fnames(mut force: ::core::ffi::c_int) {
    unsafe {
        let mut dirname: [::core::ffi::c_char; 4096] = [0; 4096];
        os_dirname(
            &raw mut dirname as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
        );
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            shorten_buf_fname(buf, &raw mut dirname as *mut ::core::ffi::c_char, force);
            mf_fullname((*buf).b_ml.ml_mfp);
            buf = (*buf).b_next;
        }
        status_redraw_all();
        redraw_tabline.set(true_0 != 0);
    }
}

pub unsafe extern "C" fn modname(
    mut fname: *const ::core::ffi::c_char,
    mut ext: *const ::core::ffi::c_char,
    mut prepend_dot: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fnamelen: size_t = 0;
        let mut extlen: size_t = strlen(ext);
        if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
            retval = xmalloc(
                (MAXPATHL as size_t)
                    .wrapping_add(extlen)
                    .wrapping_add(3 as size_t),
            ) as *mut ::core::ffi::c_char;
            if os_dirname(retval, MAXPATHL as size_t) == FAIL || strlen(retval) == 0 as size_t {
                xfree(retval as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            add_pathsep(retval);
            fnamelen = strlen(retval);
            prepend_dot = false_0 != 0;
        } else {
            fnamelen = strlen(fname);
            retval = xmalloc(fnamelen.wrapping_add(extlen).wrapping_add(3 as size_t))
                as *mut ::core::ffi::c_char;
            strcpy(retval, fname);
        }
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        ptr = retval.offset(fnamelen as isize);
        while ptr > retval {
            if vim_ispathsep(*ptr as ::core::ffi::c_int) {
                ptr = ptr.offset(1);
                break;
            } else {
                ptr = ptr.offset(
                    -((utf_head_off(retval, ptr.offset(-(1 as ::core::ffi::c_int as isize)))
                        + 1 as ::core::ffi::c_int) as isize),
                );
            }
        }
        let mut ptrlen: size_t = fnamelen.wrapping_sub(ptr.offset_from(retval) as size_t);
        if ptrlen > BASENAMELEN as ::core::ffi::c_uint as size_t {
            ptrlen = BASENAMELEN as size_t;
            *ptr.offset(ptrlen as isize) = NUL as ::core::ffi::c_char;
        }
        let mut s: *mut ::core::ffi::c_char = ptr.offset(ptrlen as isize);
        strcpy(s, ext);
        let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if prepend_dot as ::core::ffi::c_int != 0 && {
            e = path_tail(retval);
            *e as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        } {
            memmove(
                e.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                e as *const ::core::ffi::c_void,
                fnamelen
                    .wrapping_add(extlen)
                    .wrapping_sub(e.offset_from(retval) as size_t)
                    .wrapping_add(1 as size_t),
            );
            *e = '.' as ::core::ffi::c_char;
        }
        if !fname.is_null() && strcmp(fname, retval) == 0 as ::core::ffi::c_int {
            loop {
                s = s.offset(-1);
                if s < ptr {
                    break;
                }
                if *s as ::core::ffi::c_int == '_' as ::core::ffi::c_int {
                    continue;
                }
                *s = '_' as ::core::ffi::c_char;
                break;
            }
            if s < ptr {
                *ptr = 'v' as ::core::ffi::c_char;
            }
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn rename_with_tmp(
    from: *const ::core::ffi::c_char,
    to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if strlen(from) >= (MAXPATHL - 5 as ::core::ffi::c_int) as size_t {
            return -1 as ::core::ffi::c_int;
        }
        let mut tempname: [::core::ffi::c_char; 4097] = [0; 4097];
        strcpy(
            &raw mut tempname as *mut ::core::ffi::c_char,
            from as *mut ::core::ffi::c_char,
        );
        let mut n: ::core::ffi::c_int = 123 as ::core::ffi::c_int;
        while n < 99999 as ::core::ffi::c_int {
            let mut tail: *mut ::core::ffi::c_char =
                path_tail(&raw mut tempname as *mut ::core::ffi::c_char);
            snprintf(
                tail,
                ((MAXPATHL + 1 as ::core::ffi::c_int) as isize
                    - tail.offset_from(&raw mut tempname as *mut ::core::ffi::c_char))
                    as size_t,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                n,
            );
            if !os_path_exists(&raw mut tempname as *mut ::core::ffi::c_char) {
                if os_rename(from, &raw mut tempname as *mut ::core::ffi::c_char) == OK {
                    if os_rename(&raw mut tempname as *mut ::core::ffi::c_char, to) == OK {
                        return 0 as ::core::ffi::c_int;
                    }
                    os_rename(&raw mut tempname as *mut ::core::ffi::c_char, from);
                    return -1 as ::core::ffi::c_int;
                }
                return -1 as ::core::ffi::c_int;
            }
            n += 1;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_rename(
    mut from: *const ::core::ffi::c_char,
    mut to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut use_tmp_file: bool = false_0 != 0;
        if path_fnamecmp(from, to) == 0 as ::core::ffi::c_int {
            if p_fic.get() != 0 && strcmp(path_tail(from), path_tail(to)) != 0 as ::core::ffi::c_int
            {
                use_tmp_file = true_0 != 0;
            } else {
                return 0 as ::core::ffi::c_int;
            }
        }
        let mut from_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if !os_fileinfo(from, &raw mut from_info) {
            return -1 as ::core::ffi::c_int;
        }
        let mut to_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if os_fileinfo(to, &raw mut to_info) as ::core::ffi::c_int != 0
            && os_fileinfo_id_equal(&raw mut from_info, &raw mut to_info) as ::core::ffi::c_int != 0
        {
            use_tmp_file = true_0 != 0;
        }
        if use_tmp_file {
            return rename_with_tmp(from, to);
        }
        os_remove(to);
        if os_rename(from, to) == OK {
            return 0 as ::core::ffi::c_int;
        }
        let mut ret: ::core::ffi::c_int = vim_copyfile(from, to);
        if ret != OK {
            return -1 as ::core::ffi::c_int;
        }
        if os_fileinfo(from, &raw mut from_info) {
            os_remove(from);
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_copyfile(
    mut from: *const ::core::ffi::c_char,
    mut to: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut errmsg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut from_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if os_fileinfo_link(from, &raw mut from_info) as ::core::ffi::c_int != 0
            && from_info.stat.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t
        {
            let mut ret: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut linkbuf: [::core::ffi::c_char; 4097] = [0; 4097];
            let mut len: ssize_t = readlink(
                from,
                &raw mut linkbuf as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
            );
            if len > 0 as ssize_t {
                linkbuf[len as usize] = NUL as ::core::ffi::c_char;
                ret = symlink(&raw mut linkbuf as *mut ::core::ffi::c_char, to);
            }
            return if ret == 0 as ::core::ffi::c_int {
                OK
            } else {
                FAIL
            };
        }
        let mut acl: vim_acl_T = os_get_acl(from);
        if os_copy(from, to, UV_FS_COPYFILE_EXCL) != 0 as ::core::ffi::c_int {
            os_free_acl(acl);
            return FAIL;
        }
        os_set_acl(to, acl);
        os_free_acl(acl);
        if !errmsg.is_null() {
            semsg(errmsg, to);
            return FAIL;
        }
        return OK;
    }
}

pub unsafe extern "C" fn match_file_pat(
    mut pattern: *mut ::core::ffi::c_char,
    mut prog: *mut *mut regprog_T,
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut tail: *mut ::core::ffi::c_char,
    mut allow_dirs: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut result: bool = false_0 != 0;
        regmatch.rm_ic = p_fic.get() != 0;
        regmatch.regprog = if !prog.is_null() {
            *prog
        } else {
            vim_regcomp(pattern, RE_MAGIC)
        };
        if !regmatch.regprog.is_null()
            && (allow_dirs != 0
                && (vim_regexec(&raw mut regmatch, fname, 0 as colnr_T) as ::core::ffi::c_int != 0
                    || !sfname.is_null()
                        && vim_regexec(&raw mut regmatch, sfname, 0 as colnr_T)
                            as ::core::ffi::c_int
                            != 0)
                || allow_dirs == 0
                    && vim_regexec(&raw mut regmatch, tail, 0 as colnr_T) as ::core::ffi::c_int
                        != 0)
        {
            result = true_0 != 0;
        }
        if !prog.is_null() {
            *prog = regmatch.regprog;
        } else {
            vim_regfree(regmatch.regprog);
        }
        return result;
    }
}

pub unsafe extern "C" fn match_file_list(
    mut list: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut ffname: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut tail: *mut ::core::ffi::c_char = path_tail(sfname);
        let mut p: *mut ::core::ffi::c_char = list;
        while *p != 0 {
            let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
            copy_option_part(
                &raw mut p,
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            let mut allow_dirs: ::core::ffi::c_char = 0;
            let mut regpat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                &raw mut allow_dirs,
                false_0,
            );
            if regpat.is_null() {
                break;
            }
            let mut match_0: bool = match_file_pat(
                regpat,
                ::core::ptr::null_mut::<*mut regprog_T>(),
                ffname,
                sfname,
                tail,
                allow_dirs as ::core::ffi::c_int,
            );
            xfree(regpat as *mut ::core::ffi::c_void);
            if match_0 {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_pat_to_reg_pat(
    mut pat: *const ::core::ffi::c_char,
    mut pat_end: *const ::core::ffi::c_char,
    mut allow_dirs: *mut ::core::ffi::c_char,
    mut no_bslash: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if !allow_dirs.is_null() {
            *allow_dirs = false_0 as ::core::ffi::c_char;
        }
        if pat_end.is_null() {
            pat_end = pat.offset(strlen(pat) as isize);
        }
        if pat_end == pat {
            return xstrdup(b"^$\0".as_ptr() as *const ::core::ffi::c_char);
        }
        let mut size: size_t = 2 as size_t;
        let mut p: *const ::core::ffi::c_char = pat;
        while p < pat_end {
            match *p as ::core::ffi::c_int {
                42 | 46 | 44 | 123 | 125 | 126 => {
                    size = size.wrapping_add(2 as size_t);
                }
                _ => {
                    size = size.wrapping_add(1);
                }
            }
            p = p.offset(1);
        }
        let mut reg_pat: *mut ::core::ffi::c_char =
            xmalloc(size.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut i: size_t = 0 as size_t;
        if *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '*' as ::core::ffi::c_int
        {
            while *pat.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                && pat < pat_end.offset(-(1 as ::core::ffi::c_int as isize))
            {
                pat = pat.offset(1);
            }
        } else {
            let c2rust_fresh10 = i;
            i = i.wrapping_add(1);
            *reg_pat.offset(c2rust_fresh10 as isize) = '^' as ::core::ffi::c_char;
        }
        let mut endp: *const ::core::ffi::c_char =
            pat_end.offset(-(1 as ::core::ffi::c_int as isize));
        let mut add_dollar: bool = true_0 != 0;
        if endp >= pat && *endp as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
            while endp.offset_from(pat) > 0 as isize
                && *endp as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            {
                endp = endp.offset(-1);
            }
            add_dollar = false_0 != 0;
        }
        let mut nested: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p_0: *const ::core::ffi::c_char = pat;
        while *p_0 as ::core::ffi::c_int != 0 && nested >= 0 as ::core::ffi::c_int && p_0 <= endp {
            match *p_0 as ::core::ffi::c_int {
                42 => {
                    let c2rust_fresh11 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh11 as isize) = '.' as ::core::ffi::c_char;
                    let c2rust_fresh12 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh12 as isize) = '*' as ::core::ffi::c_char;
                    while *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '*' as ::core::ffi::c_int
                    {
                        p_0 = p_0.offset(1);
                    }
                }
                46 | 126 => {
                    let c2rust_fresh13 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh13 as isize) = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh14 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh14 as isize) = *p_0;
                }
                63 => {
                    let c2rust_fresh15 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh15 as isize) = '.' as ::core::ffi::c_char;
                }
                92 => {
                    if *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                        p_0 = p_0.offset(1);
                        if *p_0 as ::core::ffi::c_int == '?' as ::core::ffi::c_int
                            && (BACKSLASH_IN_FILENAME_BOOL == 0 || no_bslash != 0)
                        {
                            let c2rust_fresh16 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh16 as isize) = '?' as ::core::ffi::c_char;
                        } else if *p_0 as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                            || *p_0 as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                            || *p_0 as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                            || ascii_isspace(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                            || *p_0 as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                            || *p_0 as ::core::ffi::c_int == '}' as ::core::ffi::c_int
                        {
                            let c2rust_fresh17 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh17 as isize) = *p_0;
                        } else if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                            && *p_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
                            && *p_0.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '{' as ::core::ffi::c_int
                        {
                            let c2rust_fresh18 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh18 as isize) = '\\' as ::core::ffi::c_char;
                            let c2rust_fresh19 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh19 as isize) = '{' as ::core::ffi::c_char;
                            p_0 = p_0.offset(2 as ::core::ffi::c_int as isize);
                        } else {
                            if !allow_dirs.is_null()
                                && vim_ispathsep(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int
                                    != 0
                                && (BACKSLASH_IN_FILENAME_BOOL == 0
                                    || (no_bslash == 0
                                        || *p_0 as ::core::ffi::c_int
                                            != '\\' as ::core::ffi::c_int))
                            {
                                *allow_dirs = true_0 as ::core::ffi::c_char;
                            }
                            let c2rust_fresh20 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh20 as isize) = '\\' as ::core::ffi::c_char;
                            let c2rust_fresh21 = i;
                            i = i.wrapping_add(1);
                            *reg_pat.offset(c2rust_fresh21 as isize) = *p_0;
                        }
                    }
                }
                123 => {
                    let c2rust_fresh22 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh22 as isize) = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh23 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh23 as isize) = '(' as ::core::ffi::c_char;
                    nested += 1;
                }
                125 => {
                    let c2rust_fresh24 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh24 as isize) = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh25 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh25 as isize) = ')' as ::core::ffi::c_char;
                    nested -= 1;
                }
                44 => {
                    if nested != 0 {
                        let c2rust_fresh26 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh26 as isize) = '\\' as ::core::ffi::c_char;
                        let c2rust_fresh27 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh27 as isize) = '|' as ::core::ffi::c_char;
                    } else {
                        let c2rust_fresh28 = i;
                        i = i.wrapping_add(1);
                        *reg_pat.offset(c2rust_fresh28 as isize) = ',' as ::core::ffi::c_char;
                    }
                }
                _ => {
                    if !allow_dirs.is_null()
                        && vim_ispathsep(*p_0 as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    {
                        *allow_dirs = true_0 as ::core::ffi::c_char;
                    }
                    let c2rust_fresh29 = i;
                    i = i.wrapping_add(1);
                    *reg_pat.offset(c2rust_fresh29 as isize) = *p_0;
                }
            }
            p_0 = p_0.offset(1);
        }
        if add_dollar {
            let c2rust_fresh30 = i;
            i = i.wrapping_add(1);
            *reg_pat.offset(c2rust_fresh30 as isize) = '$' as ::core::ffi::c_char;
        }
        *reg_pat.offset(i as isize) = NUL as ::core::ffi::c_char;
        if nested != 0 as ::core::ffi::c_int {
            if nested < 0 as ::core::ffi::c_int {
                emsg(gettext(
                    b"E219: Missing {.\0".as_ptr() as *const ::core::ffi::c_char
                ));
            } else {
                emsg(gettext(
                    b"E220: Missing }.\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut reg_pat as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        return reg_pat;
    }
}
