//! Deciding what to call the swap file, and what to do when
//! that name is taken.
//!
//! `findswapname` walks `'directory'` for a name nothing else is using. A name
//! that *is* using it means either a crash to recover from or another Nvim with
//! the file open, which is what `attention_message` and the `SwapExists`
//! autocommand (`do_swapexists`) exist to sort out.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ml_setname(mut buf: *mut buf_T) {
    unsafe {
        let mut success: bool = false_0 != 0;
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
            if p_uc.get() != 0 as OptInt
                && (*cmdmod.ptr()).cmod_flags & CMOD_NOSWAPFILE as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                ml_open_file(buf);
            }
            return;
        }
        let mut dirp: *mut ::core::ffi::c_char = p_dir.get();
        let mut found_existing_dir: bool = false_0 != 0;
        while *dirp as ::core::ffi::c_int != NUL {
            let mut fname: *mut ::core::ffi::c_char = findswapname(
                buf,
                &raw mut dirp,
                (*mfp).mf_fname,
                &raw mut found_existing_dir,
            );
            if dirp.is_null() {
                break;
            }
            if fname.is_null() {
                continue;
            }
            if path_fnamecmp(fname, (*mfp).mf_fname) == 0 as ::core::ffi::c_int {
                xfree(fname as *mut ::core::ffi::c_void);
                success = true_0 != 0;
                break;
            } else {
                if (*mfp).mf_fd >= 0 as ::core::ffi::c_int {
                    close((*mfp).mf_fd);
                    (*mfp).mf_fd = -1 as ::core::ffi::c_int;
                }
                if vim_rename((*mfp).mf_fname, fname) == 0 as ::core::ffi::c_int {
                    success = true_0 != 0;
                    mf_free_fnames(mfp);
                    mf_set_fnames(mfp, fname);
                    ml_upd_block0(buf, UB_SAME_DIR);
                    break;
                } else {
                    xfree(fname as *mut ::core::ffi::c_void);
                }
            }
        }
        if (*mfp).mf_fd == -1 as ::core::ffi::c_int {
            (*mfp).mf_fd = os_open((*mfp).mf_fname, O_RDWR, 0 as ::core::ffi::c_int);
            if (*mfp).mf_fd < 0 as ::core::ffi::c_int {
                emsg(gettext(
                    b"E301: Oops, lost the swap file!!!\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return;
            }
            os_set_cloexec((*mfp).mf_fd);
        }
        if !success {
            emsg(gettext(
                b"E302: Could not rename swap file\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
}

pub unsafe extern "C" fn make_percent_swname(
    mut dir: *mut ::core::ffi::c_char,
    mut dir_end: *mut ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut d: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut f: *mut ::core::ffi::c_char = fix_fname(if !name.is_null() {
            name
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        });
        if f.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut s: *mut ::core::ffi::c_char = xstrdup(f);
        d = s;
        while *d as ::core::ffi::c_int != NUL {
            if vim_ispathsep(*d as ::core::ffi::c_int) {
                *d = '%' as ::core::ffi::c_char;
            }
            d = d.offset(utfc_ptr2len(d) as isize);
        }
        *dir_end.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        d = concat_fnames(dir, s, true_0 != 0);
        xfree(s as *mut ::core::ffi::c_void);
        xfree(f as *mut ::core::ffi::c_void);
        return d;
    }
}

pub unsafe extern "C" fn resolve_symlink(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tmp: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut depth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if fname.is_null() {
            return FAIL;
        }
        xstrlcpy(
            &raw mut tmp as *mut ::core::ffi::c_char,
            fname,
            MAXPATHL as size_t,
        );
        loop {
            depth += 1;
            if depth == 100 as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E773: Symlink loop for \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    fname,
                );
                return FAIL;
            }
            let mut ret: ::core::ffi::c_int = readlink(
                &raw mut tmp as *mut ::core::ffi::c_char,
                buf,
                (MAXPATHL - 1 as ::core::ffi::c_int) as size_t,
            ) as ::core::ffi::c_int;
            if ret <= 0 as ::core::ffi::c_int {
                if *__errno_location() == EINVAL || *__errno_location() == ENOENT {
                    if depth == 1 as ::core::ffi::c_int {
                        return FAIL;
                    }
                    break;
                } else {
                    return FAIL;
                }
            } else {
                *buf.offset(ret as isize) = NUL as ::core::ffi::c_char;
                if path_is_absolute(buf) {
                    strcpy(&raw mut tmp as *mut ::core::ffi::c_char, buf);
                } else {
                    let mut tail: *mut ::core::ffi::c_char =
                        path_tail(&raw mut tmp as *mut ::core::ffi::c_char);
                    if strlen(tail).wrapping_add(strlen(buf)) >= MAXPATHL as size_t {
                        return FAIL;
                    }
                    strcpy(tail, buf);
                }
            }
        }
        return vim_FullName(
            &raw mut tmp as *mut ::core::ffi::c_char,
            buf,
            MAXPATHL as size_t,
            true_0 != 0,
        );
    }
}

pub unsafe extern "C" fn makeswapname(
    mut fname: *mut ::core::ffi::c_char,
    mut _ffname: *mut ::core::ffi::c_char,
    mut _buf: *mut buf_T,
    mut dir_name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut fname_res: *mut ::core::ffi::c_char = fname;
        let mut fname_buf: [::core::ffi::c_char; 4096] = [0; 4096];
        if resolve_symlink(fname, &raw mut fname_buf as *mut ::core::ffi::c_char) == OK {
            fname_res = &raw mut fname_buf as *mut ::core::ffi::c_char;
        }
        let mut len: ::core::ffi::c_int = strlen(dir_name) as ::core::ffi::c_int;
        let mut s: *mut ::core::ffi::c_char = dir_name.offset(len as isize);
        if after_pathsep(dir_name, s) != 0
            && len > 1 as ::core::ffi::c_int
            && *s.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *s.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            let mut r: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            s = make_percent_swname(dir_name, s, fname_res);
            if !s.is_null() {
                r = modname(
                    s,
                    b".swp\0".as_ptr() as *const ::core::ffi::c_char,
                    false_0 != 0,
                );
                xfree(s as *mut ::core::ffi::c_void);
            }
            return r;
        }
        let mut r_0: *mut ::core::ffi::c_char = modname(
            fname_res,
            b".swp\0".as_ptr() as *const ::core::ffi::c_char,
            *dir_name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *dir_name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL,
        );
        if r_0.is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        s = get_file_in_dir(r_0, dir_name);
        xfree(r_0 as *mut ::core::ffi::c_void);
        return s;
    }
}

pub unsafe extern "C" fn get_file_in_dir(
    mut fname: *mut ::core::ffi::c_char,
    mut dname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tail: *mut ::core::ffi::c_char = path_tail(fname);
        if *dname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *dname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            retval = xstrdup(fname);
        } else if *dname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && vim_ispathsep(*dname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            if tail == fname {
                retval = concat_fnames(
                    dname.offset(2 as ::core::ffi::c_int as isize),
                    tail,
                    true_0 != 0,
                );
            } else {
                let mut save_char: ::core::ffi::c_char = *tail;
                *tail = NUL as ::core::ffi::c_char;
                let mut t: *mut ::core::ffi::c_char = concat_fnames(
                    fname,
                    dname.offset(2 as ::core::ffi::c_int as isize),
                    true_0 != 0,
                );
                *tail = save_char;
                retval = concat_fnames(t, tail, true_0 != 0);
                xfree(t as *mut ::core::ffi::c_void);
            }
        } else {
            retval = concat_fnames(dname, tail, true_0 != 0);
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn attention_message(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fhname: *mut ::core::ffi::c_char,
    mut msg_0: *mut StringBuilder,
) {
    unsafe {
        '_c2rust_label: {
            if !(*buf).b_fname.is_null() {
            } else {
                __assert_fail(
                    b"buf->b_fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3379 as ::core::ffi::c_uint,
                    b"void attention_message(buf_T *, char *, char *, StringBuilder *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        emsg(gettext(
            b"E325: ATTENTION\0".as_ptr() as *const ::core::ffi::c_char
        ));
        kv_do_printf(
            msg_0,
            gettext(b"Found a swap file by the name \"\0".as_ptr() as *const ::core::ffi::c_char),
        );
        kv_do_printf(
            msg_0,
            b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
            fhname,
        );
        let swap_mtime: time_t = swapfile_info(fname, msg_0);
        kv_do_printf(
            msg_0,
            gettext(b"While opening file \"\0".as_ptr() as *const ::core::ffi::c_char),
        );
        kv_do_printf(
            msg_0,
            b"%s\"\n\0".as_ptr() as *const ::core::ffi::c_char,
            (*buf).b_fname,
        );
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
        if !os_fileinfo((*buf).b_fname, &raw mut file_info) {
            kv_do_printf(
                msg_0,
                gettext(b"      CANNOT BE FOUND\0".as_ptr() as *const ::core::ffi::c_char),
            );
        } else {
            kv_do_printf(
                msg_0,
                gettext(b"             dated: \0".as_ptr() as *const ::core::ffi::c_char),
            );
            let mut x: time_t = file_info.stat.st_mtim.tv_sec as time_t;
            let mut ctime_buf: [::core::ffi::c_char; 50] = [0; 50];
            kv_do_printf(
                msg_0,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                os_ctime_r(x, &mut ctime_buf, true),
            );
            if swap_mtime != 0 as time_t && x > swap_mtime {
                kv_do_printf(
                    msg_0,
                    gettext(
                        b"      NEWER than swap file!\n\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                );
            }
        }
        kv_do_printf(
        msg_0,
        gettext(
            b"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n\0"
                .as_ptr() as *const ::core::ffi::c_char,
        ),
    );
        kv_do_printf(
            msg_0,
            gettext(b"(2) An edit session for this file crashed.\n\0".as_ptr()
                as *const ::core::ffi::c_char),
        );
        kv_do_printf(
            msg_0,
            gettext(
                b"    If this is the case, use \":recover\" or \"nvim -r \0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        kv_do_printf(
            msg_0,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*buf).b_fname,
        );
        kv_do_printf(
            msg_0,
            gettext(
                b"\"\n    to recover the changes (see \":help recovery\").\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        kv_do_printf(
            msg_0,
            gettext(
                b"    If you did this already, delete the swap file \"\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        kv_do_printf(msg_0, b"%s\0".as_ptr() as *const ::core::ffi::c_char, fname);
        kv_do_printf(
            msg_0,
            gettext(b"\"\n    to avoid this message.\n\0".as_ptr() as *const ::core::ffi::c_char),
        );
    }
}

pub(crate) unsafe extern "C" fn do_swapexists(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
) -> sea_choice_T {
    unsafe {
        set_vim_var_string(VV_SWAPNAME, fname, -1 as ptrdiff_t);
        set_vim_var_string(
            VV_SWAPCHOICE,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        (*allbuf_lock.ptr()) += 1;
        apply_autocmds(
            EVENT_SWAPEXISTS,
            (*buf).b_fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        (*allbuf_lock.ptr()) -= 1;
        set_vim_var_string(
            VV_SWAPNAME,
            ::core::ptr::null::<::core::ffi::c_char>(),
            -1 as ptrdiff_t,
        );
        match *get_vim_var_str(VV_SWAPCHOICE) as ::core::ffi::c_int {
            111 => return SEA_CHOICE_READONLY,
            101 => return SEA_CHOICE_EDIT,
            114 => return SEA_CHOICE_RECOVER,
            100 => return SEA_CHOICE_DELETE,
            113 => return SEA_CHOICE_QUIT,
            97 => return SEA_CHOICE_ABORT,
            _ => {}
        }
        return SEA_CHOICE_NONE;
    }
}

pub(crate) unsafe extern "C" fn findswapname(
    mut buf: *mut buf_T,
    mut dirp: *mut *mut ::core::ffi::c_char,
    mut old_fname: *mut ::core::ffi::c_char,
    mut found_existing_dir: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut buf_fname: *mut ::core::ffi::c_char = (*buf).b_fname;
        let dir_len: size_t = strlen(*dirp).wrapping_add(1 as size_t);
        let mut dir_name: *mut ::core::ffi::c_char = xmalloc(dir_len) as *mut ::core::ffi::c_char;
        copy_option_part(
            dirp,
            dir_name,
            dir_len,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut fname: *mut ::core::ffi::c_char =
            makeswapname(buf_fname, (*buf).b_ffname, buf, dir_name);
        loop {
            let mut n: size_t = 0;
            if fname.is_null() {
                break;
            }
            n = strlen(fname);
            if n == 0 as size_t {
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut fname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL_0;
                let _ = *ptr_;
                break;
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
                let mut file_or_link_found: bool = os_fileinfo_link(fname, &raw mut file_info);
                if !file_or_link_found {
                    break;
                }
                if !old_fname.is_null()
                    && path_fnamecmp(fname, old_fname) == 0 as ::core::ffi::c_int
                {
                    break;
                }
                if *fname.offset(n.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    == 'w' as ::core::ffi::c_int
                    && *fname.offset(n.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == 'p' as ::core::ffi::c_int
                {
                    if !recoverymode.get()
                        && !buf_fname.is_null()
                        && !(*buf).b_help
                        && (*buf).b_flags & BF_DUMMY == 0
                    {
                        let mut fd: ::core::ffi::c_int = 0;
                        let mut b0: ZeroBlock = ZeroBlock {
                            b0_id: [0; 2],
                            b0_version: [0; 10],
                            b0_page_size: [0; 4],
                            b0_mtime: [0; 4],
                            b0_ino: [0; 4],
                            b0_pid: [0; 4],
                            b0_uname: [0; 40],
                            b0_hname: [0; 40],
                            b0_fname: [0; 900],
                            b0_magic_long: 0,
                            b0_magic_int: 0,
                            b0_magic_short: 0,
                            b0_magic_char: 0,
                        };
                        let mut differ: bool = false_0 != 0;
                        fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
                        if fd >= 0 as ::core::ffi::c_int {
                            if read_eintr(
                                fd,
                                &raw mut b0 as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<ZeroBlock>(),
                            ) as usize
                                == ::core::mem::size_of::<ZeroBlock>()
                            {
                                proc_running.set(swapfile_proc_running(&raw mut b0, fname));
                                if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                                    - 2 as ::core::ffi::c_int)
                                    as usize]
                                    as ::core::ffi::c_int
                                    & B0_SAME_DIR
                                    != 0
                                {
                                    if path_fnamecmp(
                                        path_tail((*buf).b_ffname),
                                        path_tail(&raw mut b0.b0_fname as *mut ::core::ffi::c_char),
                                    ) != 0 as ::core::ffi::c_int
                                        || !same_directory(fname, (*buf).b_ffname)
                                    {
                                        expand_env(
                                            &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                                            MAXPATHL,
                                        );
                                        if fnamecmp_ino(
                                            (*buf).b_ffname,
                                            NameBuff.ptr() as *mut ::core::ffi::c_char,
                                            char_to_long(
                                                &raw mut b0.b0_ino as *mut ::core::ffi::c_char,
                                            ),
                                        ) {
                                            differ = true_0 != 0;
                                        }
                                    }
                                } else {
                                    expand_env(
                                        &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        MAXPATHL,
                                    );
                                    if fnamecmp_ino(
                                        (*buf).b_ffname,
                                        NameBuff.ptr() as *mut ::core::ffi::c_char,
                                        char_to_long(
                                            &raw mut b0.b0_ino as *mut ::core::ffi::c_char,
                                        ),
                                    ) {
                                        differ = true_0 != 0;
                                    }
                                }
                            }
                            close(fd);
                        }
                        if !differ
                            && (*curbuf.get()).b_flags & BF_RECOVERED == 0
                            && vim_strchr(p_shm.get(), SHM_ATTENTION as ::core::ffi::c_int)
                                .is_null()
                        {
                            let mut choice: sea_choice_T = SEA_CHOICE_NONE;
                            if os_path_exists((*buf).b_fname) as ::core::ffi::c_int != 0
                                && swapfile_unchanged(fname) as ::core::ffi::c_int != 0
                            {
                                choice = SEA_CHOICE_DELETE;
                                if p_verbose.get() > 0 as OptInt {
                                    verb_msg(gettext(
                                        b"Found a swap file that is not useful, deleting it\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ));
                                }
                            }
                            if choice as ::core::ffi::c_uint
                                == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                && swap_exists_action.get() != SEA_NONE
                                && has_autocmd(EVENT_SWAPEXISTS, buf_fname, buf)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                choice = do_swapexists(buf, fname);
                            }
                            if choice as ::core::ffi::c_uint
                                == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                && swap_exists_action.get() == SEA_READONLY
                            {
                                choice = SEA_CHOICE_READONLY;
                            }
                            proc_running.set(0 as ::core::ffi::c_int);
                            if choice as ::core::ffi::c_uint
                                == SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                (*no_wait_return.ptr()) += 1;
                                let mut msg_0: StringBuilder = KV_INITIAL_VALUE;
                                msg_0.capacity = (1024 as ::core::ffi::c_int
                                    + 1 as ::core::ffi::c_int)
                                    as size_t;
                                msg_0.items = xrealloc(
                                    msg_0.items as *mut ::core::ffi::c_void,
                                    ::core::mem::size_of::<::core::ffi::c_char>()
                                        .wrapping_mul(msg_0.capacity),
                                )
                                    as *mut ::core::ffi::c_char;
                                let mut fhname: *mut ::core::ffi::c_char =
                                    home_replace_save(::core::ptr::null_mut::<buf_T>(), fname);
                                attention_message(buf, fname, fhname, &raw mut msg_0);
                                got_int.set(false_0 != 0);
                                flush_buffers(FLUSH_TYPEAHEAD);
                                if swap_exists_action.get() != SEA_NONE {
                                    kv_do_printf(
                                        &raw mut msg_0,
                                        gettext(b"Swap file \"\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    kv_do_printf(
                                        &raw mut msg_0,
                                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        fhname,
                                    );
                                    kv_do_printf(
                                        &raw mut msg_0,
                                        gettext(b"\" already exists!\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    let mut run_but: *mut ::core::ffi::c_char = gettext(
                                        b"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                    let mut but: *mut ::core::ffi::c_char = gettext(
                                    b"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                                    choice = do_dialog(
                                        VIM_WARNING as ::core::ffi::c_int,
                                        gettext(b"VIM - ATTENTION\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        msg_0.items,
                                        if proc_running.get() != 0 {
                                            run_but
                                        } else {
                                            but
                                        },
                                        1 as ::core::ffi::c_int,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        false_0,
                                    ) as sea_choice_T;
                                    choice = (choice as ::core::ffi::c_uint).wrapping_add(
                                        (proc_running.get() != 0
                                            && choice as ::core::ffi::c_uint
                                                >= 4 as ::core::ffi::c_uint)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_uint,
                                    ) as sea_choice_T;
                                    msg_reset_scroll();
                                } else {
                                    let mut need_clear: bool = false_0 != 0;
                                    msg_ext_set_kind(
                                        b"wmsg\0".as_ptr() as *const ::core::ffi::c_char
                                    );
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
                                }
                                (*no_wait_return.ptr()) -= 1;
                                xfree(msg_0.items as *mut ::core::ffi::c_void);
                                msg_0.capacity = 0 as size_t;
                                msg_0.size = msg_0.capacity;
                                msg_0.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                xfree(fhname as *mut ::core::ffi::c_void);
                            }
                            match choice as ::core::ffi::c_uint {
                                1 => {
                                    (*buf).b_p_ro = true_0;
                                }
                                3 => {
                                    swap_exists_action.set(SEA_RECOVER);
                                }
                                4 => {
                                    os_remove(fname);
                                }
                                5 => {
                                    swap_exists_action.set(SEA_QUIT);
                                }
                                6 => {
                                    swap_exists_action.set(SEA_QUIT);
                                    got_int.set(true_0 != 0);
                                }
                                0 => {
                                    msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                                    if msg_silent.get() == 0 as ::core::ffi::c_int {
                                        need_wait_return.set(true_0 != 0);
                                    }
                                }
                                2 | _ => {}
                            }
                            if choice as ::core::ffi::c_uint
                                != SEA_CHOICE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                && !os_path_exists(fname)
                            {
                                break;
                            }
                        }
                    }
                }
                if *fname.offset(n.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == 'a' as ::core::ffi::c_int
                {
                    if *fname.offset(n.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                        == 'a' as ::core::ffi::c_int
                    {
                        emsg(gettext(b"E326: Too many swap files found\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        let mut ptr__0: *mut *mut ::core::ffi::c_void =
                            &raw mut fname as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__0);
                        *ptr__0 = NULL_0;
                        let _ = *ptr__0;
                        break;
                    } else {
                        *fname.offset(n.wrapping_sub(2 as size_t) as isize) -= 1;
                        *fname.offset(n.wrapping_sub(1 as size_t) as isize) =
                            ('z' as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                                as ::core::ffi::c_char;
                    }
                }
                *fname.offset(n.wrapping_sub(1 as size_t) as isize) -= 1;
            }
        }
        if os_isdir(dir_name) {
            *found_existing_dir = true_0 != 0;
        } else if !*found_existing_dir && **dirp as ::core::ffi::c_int == NUL {
            let mut ret: ::core::ffi::c_int = 0;
            let mut failed_dir: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            ret = os_mkdir_recurse(
                dir_name,
                0o755 as int32_t,
                &raw mut failed_dir,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            if ret != 0 as ::core::ffi::c_int {
                semsg(
                gettext(
                    b"E303: Unable to create directory \"%s\" for swap file, recovery impossible: %s\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                ),
                failed_dir,
                uv_strerror(ret),
            );
                xfree(failed_dir as *mut ::core::ffi::c_void);
            }
        }
        xfree(dir_name as *mut ::core::ffi::c_void);
        return fname;
    }
}
