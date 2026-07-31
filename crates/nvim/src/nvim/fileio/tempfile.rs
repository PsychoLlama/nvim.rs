//! The temporary directory, and the directory walks it needs.
//!
//! Nvim makes one private directory per process, under `$TMPDIR` or one of a
//! handful of fallbacks, and hands out names inside it; `vim_mktempdir` picks
//! the spot and `vim_tempname` numbers the files. `vim_opentempdir` keeps an
//! open handle on it so a `/tmp` cleaner cannot pull it out from under us, and
//! `vim_deltempdir` removes it at exit — which is what `delete_recursive` and
//! `readdir_core` are for, though `readdir()`/`delete()` in Vimscript reach
//! them too.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn vim_mktempdir() {
    unsafe {
        static temp_dirs: GlobalCell<[*const ::core::ffi::c_char; 4]> =
            GlobalCell::new(TEMP_DIR_NAMES);
        let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
        let mut path: [::core::ffi::c_char; 256] = [0; 256];
        let mut user: [::core::ffi::c_char; 40] = [
            0 as ::core::ffi::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        os_get_username(
            &raw mut user as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
        );
        memchrsub(
            &raw mut user as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '/' as ::core::ffi::c_char,
            '_' as ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
        );
        memchrsub(
            &raw mut user as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            '\\' as ::core::ffi::c_char,
            '_' as ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>(),
        );
        let mut umask_save: mode_t = umask(0o77 as __mode_t);
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 4]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            let mut tmplen: size_t = expand_env(
                (*temp_dirs.ptr())[i as usize] as *mut ::core::ffi::c_char,
                &raw mut tmp as *mut ::core::ffi::c_char,
                TEMP_FILE_PATH_MAXLEN - 64 as ::core::ffi::c_int,
            );
            if !os_isdir(&raw mut tmp as *mut ::core::ffi::c_char) {
                if strequal(
                    b"$TMPDIR\0".as_ptr() as *const ::core::ffi::c_char,
                    (*temp_dirs.ptr())[i as usize],
                ) {
                    if !os_env_exists(
                        b"TMPDIR\0".as_ptr() as *const ::core::ffi::c_char,
                        true_0 != 0,
                    ) {
                        logmsg(
                            LOGLVL_DBG,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                            3323 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"$TMPDIR is unset\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    } else {
                        logmsg(
                            LOGLVL_WRN,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                            3325 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"$TMPDIR tempdir not a directory (or does not exist): \"%s\"\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            &raw mut tmp as *mut ::core::ffi::c_char,
                        );
                    }
                }
            } else {
                if after_pathsep(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                ) == 0
                {
                    tmplen = tmplen.wrapping_add(vim_snprintf(
                        (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                        ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                        PATHSEPSTR.as_ptr(),
                    ) as size_t);
                    '_c2rust_label: {
                        if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                        } else {
                            __assert_fail(
                                b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                3334 as ::core::ffi::c_uint,
                                b"void vim_mktempdir(void)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
                tmplen = tmplen.wrapping_add(vim_snprintf(
                    (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                    b"nvim.%s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut user as *mut ::core::ffi::c_char,
                ) as size_t);
                '_c2rust_label_0: {
                    if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                    } else {
                        __assert_fail(
                            b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3338 as ::core::ffi::c_uint,
                            b"void vim_mktempdir(void)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                os_mkdir(&raw mut tmp as *mut ::core::ffi::c_char, 0o700 as int32_t);
                let mut owned: bool = os_file_owned(&raw mut tmp as *mut ::core::ffi::c_char);
                let mut isdir: bool = os_isdir(&raw mut tmp as *mut ::core::ffi::c_char);
                let mut perm: ::core::ffi::c_int =
                    os_getperm(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
                let mut valid: bool = isdir as ::core::ffi::c_int != 0
                    && owned as ::core::ffi::c_int != 0
                    && 0o700 as ::core::ffi::c_int == perm & 0o777 as ::core::ffi::c_int;
                if valid {
                    if after_pathsep(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                    ) == 0
                    {
                        tmplen = tmplen.wrapping_add(vim_snprintf(
                            (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                            ::core::mem::size_of::<[::core::ffi::c_char; 256]>()
                                .wrapping_sub(tmplen),
                            PATHSEPSTR.as_ptr(),
                        ) as size_t);
                        '_c2rust_label_1: {
                            if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                            } else {
                                __assert_fail(
                                    b"tmplen < sizeof(tmp)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                    3351 as ::core::ffi::c_uint,
                                    b"void vim_mktempdir(void)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                    }
                } else {
                    if !owned {
                        logmsg(
                            LOGLVL_ERR,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                            3355 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"tempdir root not owned by current user (%s): %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            &raw mut user as *mut ::core::ffi::c_char,
                            &raw mut tmp as *mut ::core::ffi::c_char,
                        );
                    } else if !isdir {
                        logmsg(
                            LOGLVL_ERR,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                            3357 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"tempdir root not a directory: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            &raw mut tmp as *mut ::core::ffi::c_char,
                        );
                    }
                    if 0o700 as ::core::ffi::c_int != perm & 0o777 as ::core::ffi::c_int {
                        logmsg(
                            LOGLVL_ERR,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                            3361 as ::core::ffi::c_int,
                            true_0 != 0,
                            b"tempdir root has invalid permissions (%o): %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            perm,
                            &raw mut tmp as *mut ::core::ffi::c_char,
                        );
                    }
                    tmplen = tmplen.wrapping_sub(strlen(&raw mut user as *mut ::core::ffi::c_char));
                    tmp[tmplen as usize] = NUL as ::core::ffi::c_char;
                }
                tmplen = tmplen.wrapping_add(vim_snprintf(
                    (&raw mut tmp as *mut ::core::ffi::c_char).offset(tmplen as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(tmplen),
                    b"XXXXXX\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
                '_c2rust_label_2: {
                    if tmplen < ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
                    } else {
                        __assert_fail(
                            b"tmplen < sizeof(tmp)\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/fileio.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3373 as ::core::ffi::c_uint,
                            b"void vim_mktempdir(void)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                let mut r: ::core::ffi::c_int = os_mkdtemp(
                    &raw mut tmp as *mut ::core::ffi::c_char,
                    &raw mut path as *mut ::core::ffi::c_char,
                );
                if r != 0 as ::core::ffi::c_int {
                    logmsg(
                        LOGLVL_WRN,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_mktempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3377 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"tempdir create failed: %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                        uv_strerror(r),
                        &raw mut tmp as *mut ::core::ffi::c_char,
                    );
                } else {
                    if vim_settempdir(&raw mut path as *mut ::core::ffi::c_char) {
                        break;
                    }
                    os_rmdir(&raw mut path as *mut ::core::ffi::c_char);
                }
            }
            i = i.wrapping_add(1);
        }
        umask(umask_save as __mode_t);
    }
}

pub unsafe extern "C" fn readdir_core(
    mut gap: *mut garray_T,
    mut path: *const ::core::ffi::c_char,
    mut context: *mut ::core::ffi::c_void,
    mut checkitem: CheckItem,
) -> ::core::ffi::c_int {
    unsafe {
        ga_init(
            gap,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            20 as ::core::ffi::c_int,
        );
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
        if !os_scandir(&raw mut dir, path) {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                path,
            );
            return FAIL;
        }
        loop {
            let mut p: *const ::core::ffi::c_char = os_scandir_next(&raw mut dir);
            if p.is_null() {
                break;
            }
            let mut ignore: bool = *p.offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                        && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == NUL);
            if !ignore && checkitem.is_some() {
                let mut r: varnumber_T = checkitem.expect("non-null function pointer")(context, p);
                if r < 0 as varnumber_T {
                    break;
                }
                if r == 0 as varnumber_T {
                    ignore = true_0 != 0;
                }
            }
            if !ignore {
                ga_grow(gap, 1 as ::core::ffi::c_int);
                let c2rust_fresh9 = (*gap).ga_len;
                (*gap).ga_len = (*gap).ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh9 as isize);
                *c2rust_lvalue_ptr = xstrdup(p);
            }
        }
        os_closedir(&raw mut dir);
        if (*gap).ga_len > 0 as ::core::ffi::c_int {
            sort_strings(
                (*gap).ga_data as *mut *mut ::core::ffi::c_char,
                (*gap).ga_len,
            );
        }
        return OK;
    }
}

pub unsafe extern "C" fn delete_recursive(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if os_isrealdir(name) {
            let mut exp: *mut ::core::ffi::c_char = xstrdup(name);
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            if readdir_core(&raw mut ga, exp, NULL, None) == OK {
                let mut len: ::core::ffi::c_int = snprintf(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    b"%s/\0".as_ptr() as *const ::core::ffi::c_char,
                    exp,
                );
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < ga.ga_len {
                    snprintf(
                        (NameBuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                        (MAXPATHL as size_t).wrapping_sub(len as size_t),
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        *(ga.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                    );
                    if delete_recursive(NameBuff.ptr() as *mut ::core::ffi::c_char)
                        != 0 as ::core::ffi::c_int
                    {
                        result = -1 as ::core::ffi::c_int;
                    }
                    i += 1;
                }
                ga_clear_strings(&raw mut ga);
                if os_rmdir(exp) != 0 as ::core::ffi::c_int {
                    result = -1 as ::core::ffi::c_int;
                }
            } else {
                result = -1 as ::core::ffi::c_int;
            }
            xfree(exp as *mut ::core::ffi::c_void);
        } else {
            result = if os_remove(name) == 0 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        return result;
    }
}

pub(crate) unsafe extern "C" fn vim_opentempdir() {
    unsafe {
        if !(*vim_tempdir_dp.ptr()).is_null() {
            return;
        }
        let mut dp: *mut DIR = opendir(vim_tempdir.get());
        if dp.is_null() {
            return;
        }
        vim_tempdir_dp.set(dp);
        flock(dirfd(vim_tempdir_dp.get()), LOCK_SH);
    }
}

pub(crate) unsafe extern "C" fn vim_closetempdir() {
    unsafe {
        if (*vim_tempdir_dp.ptr()).is_null() {
            return;
        }
        closedir(vim_tempdir_dp.get());
        vim_tempdir_dp.set(::core::ptr::null_mut::<DIR>());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_deltempdir() {
    unsafe {
        if (*vim_tempdir.ptr()).is_null() {
            return;
        }
        vim_closetempdir();
        *path_tail(vim_tempdir.get()).offset(-1 as ::core::ffi::c_int as isize) =
            NUL as ::core::ffi::c_char;
        delete_recursive(vim_tempdir.get());
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            vim_tempdir.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_gettempdir() -> *mut ::core::ffi::c_char {
    unsafe {
        static notfound: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        if (*vim_tempdir.ptr()).is_null() || !os_isdir(vim_tempdir.get()) {
            if !(*vim_tempdir.ptr()).is_null() {
                (*notfound.ptr()) += 1;
                if notfound.get() == 1 as ::core::ffi::c_int {
                    logmsg(
                        LOGLVL_ERR,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"vim_gettempdir\0".as_ptr() as *const ::core::ffi::c_char,
                        3534 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"tempdir disappeared (antivirus or broken cleanup job?): %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        vim_tempdir.get(),
                    );
                }
                if notfound.get() > 1 as ::core::ffi::c_int {
                    msg_schedule_semsg(
                        b"E5431: tempdir disappeared (%d times)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        notfound.get(),
                    );
                }
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    vim_tempdir.ptr() as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
            }
            vim_mktempdir();
        }
        return vim_tempdir.get();
    }
}

pub(crate) unsafe extern "C" fn vim_settempdir(mut tempdir: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut buf: *mut ::core::ffi::c_char =
            verbose_try_malloc((MAXPATHL + 2 as ::core::ffi::c_int) as size_t)
                as *mut ::core::ffi::c_char;
        if buf.is_null() {
            return false_0 != 0;
        }
        vim_FullName(tempdir, buf, MAXPATHL as size_t, false_0 != 0);
        let mut buflen: size_t = strlen(buf);
        if after_pathsep(buf, buf.offset(buflen as isize)) == 0 {
            strcpy(buf.offset(buflen as isize), PATHSEPSTR.as_ptr());
            buflen = (buflen as ::core::ffi::c_ulong).wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
                    as ::core::ffi::c_ulong,
            ) as size_t;
        }
        vim_tempdir
            .set(xmemdupz(buf as *const ::core::ffi::c_void, buflen) as *mut ::core::ffi::c_char);
        vim_opentempdir();
        xfree(buf as *mut ::core::ffi::c_void);
        return true_0 != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_tempname() -> *mut ::core::ffi::c_char {
    unsafe {
        static temp_count: GlobalCell<uint64_t> = GlobalCell::new(0);
        let mut tempdir: *mut ::core::ffi::c_char = vim_gettempdir();
        if tempdir.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut templ: [::core::ffi::c_char; 256] = [0; 256];
        let c2rust_fresh8 = temp_count.get();
        temp_count.set((*temp_count.ptr()).wrapping_add(1));
        let mut itmplen: ::core::ffi::c_int = snprintf(
            &raw mut templ as *mut ::core::ffi::c_char,
            TEMP_FILE_PATH_MAXLEN as size_t,
            b"%s%lu\0".as_ptr() as *const ::core::ffi::c_char,
            tempdir,
            c2rust_fresh8,
        );
        return xmemdupz(
            &raw mut templ as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            itmplen as size_t,
        ) as *mut ::core::ffi::c_char;
    }
}
