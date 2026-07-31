//! Surviving a crash: keeping the swap file current, and
//! finding the ones left behind.
//!
//! `ml_sync_all` and `ml_preserve` are the writing half — the timer and
//! `:preserve` making sure the swap file is worth recovering from.
//! `recover_names` is the reading half: which swap files exist for a given
//! file, for `:recover`, `swapfilelist()` and the ATTENTION message.
//!
//! `ml_recover` itself is still in the parent; it is one 1,059-line transpiled
//! function and moving it here would put this file over the line cap.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn recover_names(
    mut fname: *mut ::core::ffi::c_char,
    mut do_list: bool,
    mut ret_list: *mut list_T,
    mut nr: ::core::ffi::c_int,
    mut fname_out: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut num_names: ::core::ffi::c_int = 0;
        let mut names: [*mut ::core::ffi::c_char; 6] =
            [::core::ptr::null_mut::<::core::ffi::c_char>(); 6];
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut file_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut files: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut fname_res: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fname_buf: [::core::ffi::c_char; 4096] = [0; 4096];
        if !fname.is_null() {
            fname_res =
                if resolve_symlink(fname, &raw mut fname_buf as *mut ::core::ffi::c_char) == OK {
                    &raw mut fname_buf as *mut ::core::ffi::c_char
                } else {
                    fname
                };
        }
        msg_ext_skip_flush.set(true_0 != 0);
        if do_list {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            msg(
                gettext(b"Swap files found:\0".as_ptr() as *const ::core::ffi::c_char),
                0 as ::core::ffi::c_int,
            );
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        let mut dir_name: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        dir_name.data =
            xmalloc(strlen(p_dir.get()).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut dirp: *mut ::core::ffi::c_char = p_dir.get();
        while *dirp != 0 {
            dir_name.size = copy_option_part(
                &raw mut dirp,
                dir_name.data,
                31000 as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if *dir_name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *dir_name.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == NUL
            {
                if fname.is_null() {
                    names[0 as ::core::ffi::c_int as usize] = xmemdupz(
                        b"*.sw?\0".as_ptr() as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    names[1 as ::core::ffi::c_int as usize] = xmemdupz(
                        b".*.sw?\0".as_ptr() as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    names[2 as ::core::ffi::c_int as usize] = xmemdupz(
                        b".sw?\0".as_ptr() as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                    )
                        as *mut ::core::ffi::c_char;
                    num_names = 3 as ::core::ffi::c_int;
                } else {
                    num_names = recov_file_names(
                        &raw mut names as *mut *mut ::core::ffi::c_char,
                        fname_res,
                        true_0 != 0,
                    );
                }
            } else if fname.is_null() {
                names[0 as ::core::ffi::c_int as usize] = concat_fnames(
                    dir_name.data,
                    b"*.sw?\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0 != 0,
                )
                    as *mut ::core::ffi::c_char;
                names[1 as ::core::ffi::c_int as usize] = concat_fnames(
                    dir_name.data,
                    b".*.sw?\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0 != 0,
                )
                    as *mut ::core::ffi::c_char;
                names[2 as ::core::ffi::c_int as usize] = concat_fnames(
                    dir_name.data,
                    b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0 != 0,
                )
                    as *mut ::core::ffi::c_char;
                num_names = 3 as ::core::ffi::c_int;
            } else {
                p = dir_name.data.offset(dir_name.size as isize);
                if after_pathsep(dir_name.data, p) != 0
                    && dir_name.size > 1 as size_t
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    tail = make_percent_swname(dir_name.data, p, fname_res);
                } else {
                    tail = path_tail(fname_res);
                    tail = concat_fnames(dir_name.data, tail, true_0 != 0);
                }
                num_names = recov_file_names(
                    &raw mut names as *mut *mut ::core::ffi::c_char,
                    tail,
                    false_0 != 0,
                );
                xfree(tail as *mut ::core::ffi::c_void);
            }
            let mut num_files: ::core::ffi::c_int = 0;
            if num_names == 0 as ::core::ffi::c_int {
                num_files = 0 as ::core::ffi::c_int;
            } else if expand_wildcards(
                num_names,
                &raw mut names as *mut *mut ::core::ffi::c_char,
                &raw mut num_files,
                &raw mut files,
                EW_KEEPALL as ::core::ffi::c_int
                    | EW_FILE as ::core::ffi::c_int
                    | EW_SILENT as ::core::ffi::c_int,
            ) == FAIL
            {
                num_files = 0 as ::core::ffi::c_int;
            }
            if *dirp as ::core::ffi::c_int == NUL
                && file_count + num_files == 0 as ::core::ffi::c_int
                && !fname.is_null()
            {
                let mut swapname: *mut ::core::ffi::c_char = modname(
                    fname_res,
                    b".swp\0".as_ptr() as *const ::core::ffi::c_char,
                    true_0 != 0,
                );
                if !swapname.is_null() {
                    if os_path_exists(swapname) {
                        files = xmalloc(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                            as *mut *mut ::core::ffi::c_char;
                        *files.offset(0 as ::core::ffi::c_int as isize) = swapname;
                        swapname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        num_files = 1 as ::core::ffi::c_int;
                    }
                    xfree(swapname as *mut ::core::ffi::c_void);
                }
            }
            if !(*curbuf.get()).b_ml.ml_mfp.is_null()
                && {
                    p = (*(*curbuf.get()).b_ml.ml_mfp).mf_fname;
                    !p.is_null()
                }
                && ret_list.is_null()
            {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < num_files {
                    if path_full_compare(p, *files.offset(i as isize), true_0 != 0, false_0 != 0)
                        as ::core::ffi::c_uint
                        & kEqualFiles as ::core::ffi::c_int as ::core::ffi::c_uint
                        != 0
                    {
                        xfree(*files.offset(i as isize) as *mut ::core::ffi::c_void);
                        num_files -= 1;
                        if num_files == 0 as ::core::ffi::c_int {
                            xfree(files as *mut ::core::ffi::c_void);
                        } else {
                            while i < num_files {
                                *files.offset(i as isize) =
                                    *files.offset((i + 1 as ::core::ffi::c_int) as isize);
                                i += 1;
                            }
                        }
                    }
                    i += 1;
                }
            }
            if nr > 0 as ::core::ffi::c_int {
                file_count += num_files;
                if nr <= file_count {
                    *fname_out =
                        xstrdup(*files.offset(
                            (nr - 1 as ::core::ffi::c_int + num_files - file_count) as isize,
                        ));
                    dirp = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
                }
            } else if do_list {
                if *dir_name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                    && *dir_name.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == NUL
                {
                    if fname.is_null() {
                        msg_puts(gettext(
                            b"   In current directory:\n\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    } else {
                        msg_puts(gettext(
                            b"   Using specified name:\n\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                    }
                } else {
                    msg_puts(gettext(
                        b"   In directory \0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    msg_home_replace(dir_name.data);
                    msg_puts(b":\n\0".as_ptr() as *const ::core::ffi::c_char);
                }
                if num_files != 0 {
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < num_files {
                        file_count += 1;
                        msg_outnum(file_count);
                        msg_puts(b".    \0".as_ptr() as *const ::core::ffi::c_char);
                        msg_puts(path_tail(*files.offset(i_0 as isize)));
                        msg_putchar('\n' as ::core::ffi::c_int);
                        let mut msg_0: StringBuilder = KV_INITIAL_VALUE;
                        msg_0.capacity =
                            (1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
                        msg_0.items = xrealloc(
                            msg_0.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul(msg_0.capacity),
                        ) as *mut ::core::ffi::c_char;
                        swapfile_info(*files.offset(i_0 as isize), &raw mut msg_0);
                        let mut need_clear: bool = false_0 != 0;
                        msg_multiline(
                            String_0 {
                                data: msg_0.items,
                                size: msg_0.size,
                            },
                            0 as ::core::ffi::c_int,
                            false_0 != 0,
                            false_0 != 0,
                            &raw mut need_clear,
                        );
                        xfree(msg_0.items as *mut ::core::ffi::c_void);
                        msg_0.capacity = 0 as size_t;
                        msg_0.size = msg_0.capacity;
                        msg_0.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        i_0 += 1;
                    }
                } else {
                    msg_puts(gettext(
                        b"      -- none --\n\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                ui_flush();
            } else if !ret_list.is_null() {
                let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_1 < num_files {
                    let mut name: *mut ::core::ffi::c_char =
                        concat_fnames(dir_name.data, *files.offset(i_1 as isize), true_0 != 0);
                    tv_list_append_allocated_string(ret_list, name);
                    i_1 += 1;
                }
            } else {
                file_count += num_files;
            }
            let mut i_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_2 < num_names {
                xfree(names[i_2 as usize] as *mut ::core::ffi::c_void);
                i_2 += 1;
            }
            if num_files > 0 as ::core::ffi::c_int {
                FreeWild(num_files, files);
            }
        }
        msg_ext_skip_flush.set(false_0 != 0);
        xfree(dir_name.data as *mut ::core::ffi::c_void);
        return file_count;
    }
}

pub(crate) unsafe extern "C" fn recov_file_names(
    mut names: *mut *mut ::core::ffi::c_char,
    mut path: *mut ::core::ffi::c_char,
    mut prepend_dot: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut num_names: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if prepend_dot {
            *names.offset(num_names as isize) = modname(
                path,
                b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            );
            if (*names.offset(num_names as isize)).is_null() {
                return num_names;
            }
            num_names += 1;
        }
        *names.offset(num_names as isize) = concat_fnames(
            path,
            b".sw?\0".as_ptr() as *const ::core::ffi::c_char,
            false_0 != 0,
        );
        if num_names >= 1 as ::core::ffi::c_int {
            let mut p: *mut ::core::ffi::c_char =
                *names.offset((num_names - 1 as ::core::ffi::c_int) as isize);
            let mut i: ::core::ffi::c_int =
                strlen(*names.offset((num_names - 1 as ::core::ffi::c_int) as isize))
                    as ::core::ffi::c_int
                    - strlen(*names.offset(num_names as isize)) as ::core::ffi::c_int;
            if i > 0 as ::core::ffi::c_int {
                p = p.offset(i as isize);
            }
            if strcmp(p, *names.offset(num_names as isize)) != 0 as ::core::ffi::c_int {
                num_names += 1;
            } else {
                xfree(*names.offset(num_names as isize) as *mut ::core::ffi::c_void);
            }
        } else {
            num_names += 1;
        }
        return num_names;
    }
}

pub unsafe extern "C" fn ml_sync_all(
    mut check_file: ::core::ffi::c_int,
    mut check_char: ::core::ffi::c_int,
    mut do_fsync: bool,
) {
    unsafe {
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            if !((*buf).b_ml.ml_mfp.is_null() || (*(*buf).b_ml.ml_mfp).mf_fname.is_null()) {
                ml_flush_line(buf, false_0 != 0);
                ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
                if bufIsChanged(buf) as ::core::ffi::c_int != 0
                    && check_file != 0
                    && mf_need_trans((*buf).b_ml.ml_mfp) as ::core::ffi::c_int != 0
                    && !(*buf).b_ffname.is_null()
                {
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
                    if !os_fileinfo((*buf).b_ffname, &raw mut file_info)
                        || file_info.stat.st_mtim.tv_sec as int64_t != (*buf).b_mtime_read
                        || file_info.stat.st_mtim.tv_nsec as int64_t != (*buf).b_mtime_read_ns
                        || os_fileinfo_size(&raw mut file_info) != (*buf).b_orig_size
                    {
                        ml_preserve(buf, false_0 != 0, do_fsync);
                        did_check_timestamps.set(false_0 != 0);
                        need_check_timestamps.set(true_0 != 0);
                    }
                }
                if (*(*buf).b_ml.ml_mfp).mf_dirty as ::core::ffi::c_uint
                    == MF_DIRTY_YES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    mf_sync(
                        (*buf).b_ml.ml_mfp,
                        (if check_char != 0 {
                            MFS_STOP as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) | (if do_fsync as ::core::ffi::c_int != 0
                            && bufIsChanged(buf) as ::core::ffi::c_int != 0
                        {
                            MFS_FLUSH as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    );
                    if check_char != 0 && os_char_avail() as ::core::ffi::c_int != 0 {
                        break;
                    }
                }
            }
            buf = (*buf).b_next;
        }
    }
}

pub unsafe extern "C" fn ml_preserve(mut buf: *mut buf_T, mut message: bool, mut do_fsync: bool) {
    unsafe {
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        let mut got_int_save: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
        if mfp.is_null() || (*mfp).mf_fname.is_null() {
            if message {
                emsg(gettext(
                    b"E313: Cannot preserve, there is no swap file\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            return;
        }
        got_int.set(false_0 != 0);
        ml_flush_line(buf, false_0 != 0);
        ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
        let mut status: ::core::ffi::c_int = mf_sync(
            mfp,
            MFS_ALL as ::core::ffi::c_int
                | (if do_fsync as ::core::ffi::c_int != 0 {
                    MFS_FLUSH as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }),
        );
        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
        '_theend: {
            if mf_need_trans(mfp) as ::core::ffi::c_int != 0 && !got_int.get() {
                let mut lnum: linenr_T = 1 as linenr_T;
                while mf_need_trans(mfp) as ::core::ffi::c_int != 0
                    && lnum <= (*buf).b_ml.ml_line_count
                {
                    let mut hp: *mut bhdr_T =
                        ml_find_line(buf, lnum, ML_FIND as ::core::ffi::c_int);
                    if hp.is_null() {
                        status = FAIL;
                        break '_theend;
                    } else {
                        lnum = (*buf).b_ml.ml_locked_high + 1 as linenr_T;
                    }
                }
                ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
                if mf_sync(
                    mfp,
                    MFS_ALL as ::core::ffi::c_int
                        | (if do_fsync as ::core::ffi::c_int != 0 {
                            MFS_FLUSH as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                ) == FAIL
                {
                    status = FAIL;
                }
                (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
            }
        }
        got_int.set(got_int.get() as ::core::ffi::c_int | got_int_save != 0);
        if message {
            if status == OK {
                msg(
                    gettext(b"File preserved\0".as_ptr() as *const ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            } else {
                emsg(gettext(
                    b"E314: Preserve failed\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        }
    }
}
