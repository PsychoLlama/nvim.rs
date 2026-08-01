//! Expanding one wildcard pattern against the file system.
//!
//! [`do_path_expand`] is the recursive walk: it takes the pattern apart one
//! component at a time, turns the component into a regexp, and reads each
//! directory that the components before it matched, recursing on `**` to the
//! depth the pattern asks for. [`addfile`] is what decides whether a name the
//! walk found belongs in the answer, and [`match_suffix`] ranks the results
//! by `'suffixes'`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn path_has_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    unsafe {
        while *p != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            } else {
                let mut wildcards: *const ::core::ffi::c_char =
                    b"*?[{`'$\0".as_ptr() as *const ::core::ffi::c_char;
                if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null()
                    || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '~' as ::core::ffi::c_int
                        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    return true_0 != 0;
                }
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn pstrcmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        return pathcmp(
            *(a as *mut *mut ::core::ffi::c_char),
            *(b as *mut *mut ::core::ffi::c_char),
            -1 as ::core::ffi::c_int,
        );
    }
}

pub unsafe extern "C" fn path_has_exp_wildcard(mut p: *const ::core::ffi::c_char) -> bool {
    unsafe {
        while *p as ::core::ffi::c_int != NUL {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            } else {
                let mut wildcards: *const ::core::ffi::c_char =
                    b"*?[{\0".as_ptr() as *const ::core::ffi::c_char;
                if !vim_strchr(wildcards, *p as uint8_t as ::core::ffi::c_int).is_null() {
                    return true_0 != 0;
                }
            }
            p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> size_t {
    unsafe {
        return do_path_expand(gap, path, 0 as size_t, flags, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn scandir_next_with_dots(
    mut dir: *mut Directory,
) -> *const ::core::ffi::c_char {
    unsafe {
        static count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if dir.is_null() {
            count.set(0 as ::core::ffi::c_int);
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        (*count.ptr()) += 1 as ::core::ffi::c_int;
        if count.get() == 1 as ::core::ffi::c_int || count.get() == 2 as ::core::ffi::c_int {
            return if count.get() == 1 as ::core::ffi::c_int {
                b".\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"..\0".as_ptr() as *const ::core::ffi::c_char
            };
        }
        return os_scandir_next(dir);
    }
}

pub(crate) unsafe extern "C" fn do_path_expand(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut wildoff: size_t,
    mut flags: ::core::ffi::c_int,
    mut didstar: bool,
) -> size_t {
    unsafe {
        let mut start_len: ::core::ffi::c_int = (*gap).ga_len;
        let mut starstar: bool = false_0 != 0;
        static stardepth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if stardepth.get() > 0 as ::core::ffi::c_int
            && flags & EW_NOBREAK as ::core::ffi::c_int == 0
        {
            os_breakcheck();
            if got_int.get() {
                return 0 as size_t;
            }
        }
        let buflen: size_t = strlen(path).wrapping_add(MAXPATHL as size_t);
        let mut buf: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
        let mut p: *mut ::core::ffi::c_char = buf;
        let mut s: *mut ::core::ffi::c_char = buf;
        let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut path_end: *const ::core::ffi::c_char = path;
        while *path_end as ::core::ffi::c_int != NUL {
            if path_end >= path.offset(wildoff as isize)
                && rem_backslash(path_end) as ::core::ffi::c_int != 0
            {
                let c2rust_fresh5 = path_end;
                path_end = path_end.offset(1);
                let c2rust_fresh6 = p;
                p = p.offset(1);
                *c2rust_fresh6 = *c2rust_fresh5;
            } else if vim_ispathsep_nocolon(*path_end as ::core::ffi::c_int) {
                if !e.is_null() {
                    break;
                }
                s = p.offset(1 as ::core::ffi::c_int as isize);
            } else if path_end >= path.offset(wildoff as isize)
                && (!vim_strchr(
                    b"*?[{~$\0".as_ptr() as *const ::core::ffi::c_char,
                    *path_end as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                    || p_fic.get() == 0
                        && flags & EW_ICASE as ::core::ffi::c_int != 0
                        && mb_isalpha(utf_ptr2char(path_end)) as ::core::ffi::c_int != 0)
            {
                e = p;
            }
            let mut charlen: ::core::ffi::c_int = utfc_ptr2len(path_end);
            memcpy(
                p as *mut ::core::ffi::c_void,
                path_end as *const ::core::ffi::c_void,
                charlen as size_t,
            );
            p = p.offset(charlen as isize);
            path_end = path_end.offset(charlen as isize);
        }
        e = p;
        *e = NUL as ::core::ffi::c_char;
        p = buf.offset(wildoff as isize);
        while p < s {
            if rem_backslash(p) {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
                e = e.offset(-1);
                s = s.offset(-1);
            }
            p = p.offset(1);
        }
        p = s;
        while p < e {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
            {
                starstar = true_0 != 0;
            }
            p = p.offset(1);
        }
        let mut starts_with_dot: ::core::ffi::c_int =
            (*s as ::core::ffi::c_int == '.' as ::core::ffi::c_int) as ::core::ffi::c_int;
        let mut pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
            s,
            e,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0,
        );
        if pat.is_null() {
            xfree(buf as *mut ::core::ffi::c_void);
            return 0 as size_t;
        }
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        regmatch.rm_ic = flags & EW_ICASE as ::core::ffi::c_int != 0 || p_fic.get() != 0;
        if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
            (*emsg_silent.ptr()) += 1;
        }
        let mut nobreak: bool = flags & EW_NOBREAK as ::core::ffi::c_int != 0;
        regmatch.regprog = vim_regcomp(
            pat,
            RE_MAGIC
                | (if nobreak as ::core::ffi::c_int != 0 {
                    RE_NOBREAK
                } else {
                    0 as ::core::ffi::c_int
                }),
        );
        if flags & (EW_NOERROR as ::core::ffi::c_int | EW_NOTWILD as ::core::ffi::c_int) != 0 {
            (*emsg_silent.ptr()) -= 1;
        }
        xfree(pat as *mut ::core::ffi::c_void);
        if regmatch.regprog.is_null()
            && flags & EW_NOTWILD as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            xfree(buf as *mut ::core::ffi::c_void);
            return 0 as size_t;
        }
        let mut len: size_t = s.offset_from(buf) as size_t;
        if !didstar
            && stardepth.get() < 100 as ::core::ffi::c_int
            && starstar as ::core::ffi::c_int != 0
            && e.offset_from(s) == 2 as isize
            && *path_end as ::core::ffi::c_int == '/' as ::core::ffi::c_int
        {
            vim_snprintf(
                s,
                buflen.wrapping_sub(len),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                path_end.offset(1 as ::core::ffi::c_int as isize),
            );
            (*stardepth.ptr()) += 1;
            do_path_expand(gap, buf, len, flags, true_0 != 0);
            (*stardepth.ptr()) -= 1;
        }
        *s = NUL as ::core::ffi::c_char;
        let mut dir: Directory = Directory {
            request: uv_fs_t {
                data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                type_0: UV_UNKNOWN_REQ,
                reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
                fs_type: UV_FS_CUSTOM,
                loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
                cb: None,
                result: 0,
                ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                path: ::core::ptr::null::<::core::ffi::c_char>(),
                statbuf: uv_stat_t {
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
                new_path: ::core::ptr::null::<::core::ffi::c_char>(),
                file: 0,
                flags: 0,
                mode: 0,
                nbufs: 0,
                bufs: ::core::ptr::null_mut::<uv_buf_t>(),
                off: 0,
                uid: 0,
                gid: 0,
                atime: 0.,
                mtime: 0.,
                work_req: uv__work {
                    work: None,
                    done: None,
                    loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                    wq: uv__queue {
                        next: ::core::ptr::null_mut::<uv__queue>(),
                        prev: ::core::ptr::null_mut::<uv__queue>(),
                    },
                },
                bufsml: [uv_buf_t {
                    base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    len: 0,
                }; 4],
            },
            ent: uv_dirent_t {
                name: ::core::ptr::null::<::core::ffi::c_char>(),
                type_0: UV_DIRENT_UNKNOWN,
            },
        };
        let mut dirpath: *mut ::core::ffi::c_char = (if *buf as ::core::ffi::c_int == NUL {
            b".\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            buf as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        if os_file_is_readable(dirpath) as ::core::ffi::c_int != 0
            && os_scandir(&raw mut dir, dirpath) as ::core::ffi::c_int != 0
        {
            let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            scandir_next_with_dots(::core::ptr::null_mut::<Directory>());
            while !got_int.get() && {
                name = scandir_next_with_dots(&raw mut dir);
                !name.is_null()
            } {
                len = s.offset_from(buf) as size_t;
                if !((*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '.' as ::core::ffi::c_int
                    || starts_with_dot != 0
                    || flags & EW_DODOT as ::core::ffi::c_int != 0
                        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != NUL
                        && (*name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '.' as ::core::ffi::c_int
                            || *name.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                != NUL))
                    && (!regmatch.regprog.is_null()
                        && vim_regexec(&raw mut regmatch, name, 0 as colnr_T)
                            as ::core::ffi::c_int
                            != 0
                        || flags & EW_NOTWILD as ::core::ffi::c_int != 0
                            && path_fnamencmp(
                                path.offset(len as isize),
                                name,
                                e.offset_from(s) as size_t,
                            ) == 0 as ::core::ffi::c_int))
                {
                    continue;
                }
                len = len.wrapping_add(vim_snprintf(
                    s,
                    buflen.wrapping_sub(len),
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                ) as size_t);
                if len.wrapping_add(1 as size_t) >= buflen {
                    continue;
                }
                if starstar as ::core::ffi::c_int != 0
                    && stardepth.get() < 100 as ::core::ffi::c_int
                {
                    vim_snprintf(
                        buf.offset(len as isize),
                        buflen.wrapping_sub(len),
                        b"/**%s\0".as_ptr() as *const ::core::ffi::c_char,
                        path_end,
                    );
                    (*stardepth.ptr()) += 1;
                    do_path_expand(gap, buf, len.wrapping_add(1 as size_t), flags, true_0 != 0);
                    (*stardepth.ptr()) -= 1;
                }
                vim_snprintf(
                    buf.offset(len as isize),
                    buflen.wrapping_sub(len),
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    path_end,
                );
                if path_has_exp_wildcard(path_end) {
                    if stardepth.get() < 100 as ::core::ffi::c_int {
                        (*stardepth.ptr()) += 1;
                        do_path_expand(
                            gap,
                            buf,
                            len.wrapping_add(1 as size_t),
                            flags,
                            false_0 != 0,
                        );
                        (*stardepth.ptr()) -= 1;
                    }
                } else {
                    let mut file_info: FileInfo = FileInfo {
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
                    if *path_end as ::core::ffi::c_int != NUL {
                        backslash_halve(
                            buf.offset(len as isize)
                                .offset(1 as ::core::ffi::c_int as isize),
                        );
                    }
                    if if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
                        os_fileinfo_link(buf, &raw mut file_info) as ::core::ffi::c_int
                    } else {
                        os_path_exists(buf) as ::core::ffi::c_int
                    } != 0
                    {
                        addfile(gap, buf, flags);
                    }
                }
            }
            os_closedir(&raw mut dir);
        }
        xfree(buf as *mut ::core::ffi::c_void);
        vim_regfree(regmatch.regprog);
        let mut matches: size_t = ((*gap).ga_len - start_len) as size_t;
        if matches > 0 as size_t && !got_int.get() {
            qsort(
                ((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(start_len as isize)
                    as *mut ::core::ffi::c_void,
                matches,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>(),
                Some(
                    pstrcmp
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
        }
        return matches;
    }
}

pub(crate) unsafe extern "C" fn has_special_wildchar(
    mut p: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    unsafe {
        while *p != 0 {
            if *p as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
            {
                break;
            }
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\r' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\n' as ::core::ffi::c_int
            {
                p = p.offset(1);
            } else if !vim_strchr(
                SPECIAL_WILDCHAR.as_ptr(),
                *p as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            {
                if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                    && flags & EW_NOTFOUND as ::core::ffi::c_int == 0)
                {
                    if !(*p as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        && vim_strchr(p, '}' as ::core::ffi::c_int).is_null())
                    {
                        if !((*p as ::core::ffi::c_int == '`' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int)
                            && vim_strchr(p, *p as uint8_t as ::core::ffi::c_int).is_null())
                        {
                            return true_0 != 0;
                        }
                    }
                }
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn addfile(
    mut gap: *mut garray_T,
    mut f: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        let mut isdir: bool = false;
        let mut file_info: FileInfo = FileInfo {
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
        if flags & EW_NOTFOUND as ::core::ffi::c_int == 0
            && (if flags & EW_ALLLINKS as ::core::ffi::c_int != 0 {
                !os_fileinfo_link(f, &raw mut file_info) as ::core::ffi::c_int
            } else {
                !os_path_exists(f) as ::core::ffi::c_int
            }) != 0
        {
            return;
        }
        isdir = os_isdir(f);
        if isdir as ::core::ffi::c_int != 0 && flags & EW_DIR as ::core::ffi::c_int == 0
            || !isdir && flags & EW_FILE as ::core::ffi::c_int == 0
        {
            return;
        }
        if !isdir
            && flags & EW_EXEC as ::core::ffi::c_int != 0
            && !os_can_exe(
                f,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                flags & EW_SHELLCMD as ::core::ffi::c_int == 0,
            )
        {
            return;
        }
        let mut p: *mut ::core::ffi::c_char = xmalloc(
            strlen(f)
                .wrapping_add(1 as size_t)
                .wrapping_add(isdir as size_t),
        ) as *mut ::core::ffi::c_char;
        strcpy(p, f);
        if isdir as ::core::ffi::c_int != 0 && flags & EW_ADDSLASH as ::core::ffi::c_int != 0 {
            add_pathsep(p);
        }
        ga_grow(gap, 1 as ::core::ffi::c_int);
        *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset((*gap).ga_len as isize) = p;
        (*gap).ga_len += 1;
    }
}

pub unsafe extern "C" fn match_suffix(mut fname: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut suf_buf: [::core::ffi::c_char; 30] = [0; 30];
        let mut fnamelen: size_t = strlen(fname);
        let mut setsuflen: size_t = 0 as size_t;
        let mut setsuf: *mut ::core::ffi::c_char = p_su.get();
        while *setsuf != 0 {
            setsuflen = copy_option_part(
                &raw mut setsuf,
                &raw mut suf_buf as *mut ::core::ffi::c_char,
                MAXSUFLEN as size_t,
                b".,\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if setsuflen == 0 as size_t {
                let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
                if !vim_strchr(tail, '.' as ::core::ffi::c_int).is_null() {
                    continue;
                }
                setsuflen = 1 as size_t;
                break;
            } else {
                if fnamelen >= setsuflen
                    && path_fnamencmp(
                        &raw mut suf_buf as *mut ::core::ffi::c_char,
                        fname
                            .offset(fnamelen as isize)
                            .offset(-(setsuflen as isize)),
                        setsuflen,
                    ) == 0 as ::core::ffi::c_int
                {
                    break;
                }
                setsuflen = 0 as size_t;
            }
        }
        return setsuflen != 0 as size_t;
    }
}
