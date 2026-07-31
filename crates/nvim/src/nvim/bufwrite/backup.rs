//! The backup file, and deciding whether the write may happen
//! at all.
//!
//! `get_fileinfo` is the pre-flight: does the target exist, is it a regular
//! file, is it writable, and has it changed on disk since we read it.
//! `buf_write_make_backup` then makes the backup that `'backup'`,
//! `'writebackup'` and `'patchmode'` ask for — either by copying the original
//! aside or by renaming it out of the way, which `'backupcopy'` chooses
//! between. `buf_get_backup_name` is where `'backupdir'` and `'backupext'`
//! turn into a path.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn check_mtime(
    mut buf: *mut buf_T,
    mut file_info: *mut FileInfo,
) -> ::core::ffi::c_int {
    unsafe {
        if (*buf).b_mtime_read != 0 as int64_t
            && time_differs(file_info, (*buf).b_mtime_read, (*buf).b_mtime_read_ns)
                as ::core::ffi::c_int
                != 0
        {
            msg_scroll.set(true_0);
            msg_silent.set(0 as ::core::ffi::c_int);
            msg(
                gettext(
                    b"WARNING: The file has been changed since reading it!!!\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                HLF_E as ::core::ffi::c_int,
            );
            if ask_yesno(gettext(
                b"Do you really want to write to it\0".as_ptr() as *const ::core::ffi::c_char
            )) == 'n' as ::core::ffi::c_int
            {
                return FAIL;
            }
            msg_scroll.set(false_0);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_fileinfo_os(
    mut fname: *mut ::core::ffi::c_char,
    mut file_info_old: *mut FileInfo,
    mut _overwriting: bool,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    unsafe {
        *perm = -1 as ::core::ffi::c_int;
        if !os_fileinfo(fname, file_info_old) {
            *newfile = true_0 != 0;
        } else {
            *perm = (*file_info_old).stat.st_mode as ::core::ffi::c_int;
            if !((*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t) {
                if (*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
                    *err = set_err_num(
                        b"E502\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"is a directory\0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    return FAIL;
                }
                if os_nodetype(fname) != NODE_WRITABLE {
                    *err = set_err_num(
                        b"E503\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"is not a file or writable device\0".as_ptr()
                            as *const ::core::ffi::c_char),
                    );
                    return FAIL;
                }
                *device = true_0 != 0;
                *newfile = true_0 != 0;
                *perm = -1 as ::core::ffi::c_int;
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn get_fileinfo(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut overwriting: bool,
    mut forceit: bool,
    mut file_info_old: *mut FileInfo,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut readonly: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    unsafe {
        if get_fileinfo_os(
            fname,
            file_info_old,
            overwriting,
            perm,
            device,
            newfile,
            err,
        ) == FAIL
        {
            return FAIL;
        }
        *readonly = false_0 != 0;
        if !*device && !*newfile {
            *readonly = os_file_is_writable(fname) == 0;
            if !forceit && *readonly as ::core::ffi::c_int != 0 {
                if !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null() {
                    *err = set_err_num(
                        b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(err_readonly.get()),
                    );
                } else {
                    *err = set_err_num(
                        b"E505\0".as_ptr() as *const ::core::ffi::c_char,
                        gettext(b"is read-only (add ! to override)\0".as_ptr()
                            as *const ::core::ffi::c_char),
                    );
                }
                return FAIL;
            }
            if overwriting as ::core::ffi::c_int != 0 && !forceit {
                let mut retval: ::core::ffi::c_int = check_mtime(buf, file_info_old);
                if retval == FAIL {
                    return FAIL;
                }
            }
        }
        return OK;
    }
}

pub unsafe extern "C" fn buf_get_backup_name(
    mut fname: *mut ::core::ffi::c_char,
    mut dirp: *mut *mut ::core::ffi::c_char,
    mut no_prepend_dot: bool,
    mut backup_ext: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut backup: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dir_len: size_t = copy_option_part(
            dirp,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut p: *mut ::core::ffi::c_char =
            (IObuff.ptr() as *mut ::core::ffi::c_char).offset(dir_len as isize);
        if **dirp as ::core::ffi::c_int == NUL
            && !os_isdir(IObuff.ptr() as *mut ::core::ffi::c_char)
        {
            let mut ret: ::core::ffi::c_int = 0;
            let mut failed_dir: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            ret = os_mkdir_recurse(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                0o755 as int32_t,
                &raw mut failed_dir,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            if ret != 0 as ::core::ffi::c_int {
                semsg(
                    gettext(
                        b"E303: Unable to create directory \"%s\" for backup file: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    failed_dir,
                    uv_strerror(ret),
                );
                xfree(failed_dir as *mut ::core::ffi::c_void);
            }
        }
        if dir_len > 1 as size_t
            && after_pathsep(IObuff.ptr() as *mut ::core::ffi::c_char, p) != 0
            && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        {
            p = make_percent_swname(IObuff.ptr() as *mut ::core::ffi::c_char, p, fname);
            if !p.is_null() {
                backup = modname(p, backup_ext, no_prepend_dot);
                xfree(p as *mut ::core::ffi::c_void);
            }
        }
        if backup.is_null() {
            let mut rootname: *mut ::core::ffi::c_char =
                get_file_in_dir(fname, IObuff.ptr() as *mut ::core::ffi::c_char);
            if !rootname.is_null() {
                backup = modname(rootname, backup_ext, no_prepend_dot);
                xfree(rootname as *mut ::core::ffi::c_void);
            }
        }
        return backup;
    }
}

pub(crate) unsafe extern "C" fn buf_write_make_backup(
    mut fname: *mut ::core::ffi::c_char,
    mut append: bool,
    mut file_info_old: *mut FileInfo,
    mut acl: vim_acl_T,
    mut perm: ::core::ffi::c_int,
    mut bkc: ::core::ffi::c_uint,
    mut file_readonly: bool,
    mut forceit: bool,
    mut backup_copyp: *mut bool,
    mut backupp: *mut *mut ::core::ffi::c_char,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    unsafe {
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
        let no_prepend_dot: bool = false_0 != 0;
        if bkc & kOptBkcFlagYes as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            || append as ::core::ffi::c_int != 0
        {
            *backup_copyp = true_0 != 0;
        } else if bkc & kOptBkcFlagAuto as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
            if os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
                || !os_fileinfo_link(fname, &raw mut file_info)
                || !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
            {
                *backup_copyp = true_0 != 0;
            } else {
                let mut dirlen: size_t = path_tail(fname).offset_from(fname) as size_t;
                '_c2rust_label: {
                    if dirlen < 4096 as size_t {
                    } else {
                        __assert_fail(
                            b"dirlen < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/bufwrite.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            743 as ::core::ffi::c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                let mut tmp_fname: [::core::ffi::c_char; 4096] = [0; 4096];
                xmemcpyz(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    fname as *const ::core::ffi::c_void,
                    dirlen,
                );
                let mut i: ::core::ffi::c_int = 4913 as ::core::ffi::c_int;
                loop {
                    snprintf(
                        (&raw mut tmp_fname as *mut ::core::ffi::c_char).offset(dirlen as isize),
                        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>().wrapping_sub(dirlen),
                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                    );
                    if !os_fileinfo_link(
                        &raw mut tmp_fname as *mut ::core::ffi::c_char,
                        &raw mut file_info,
                    ) {
                        break;
                    }
                    i += 123 as ::core::ffi::c_int;
                }
                let mut fd: ::core::ffi::c_int = os_open(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char,
                    O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW,
                    perm,
                );
                if fd < 0 as ::core::ffi::c_int {
                    *backup_copyp = true_0 != 0;
                } else {
                    os_fchown(
                        fd,
                        (*file_info_old).stat.st_uid as uv_uid_t,
                        (*file_info_old).stat.st_gid as uv_gid_t,
                    );
                    if !os_fileinfo(
                        &raw mut tmp_fname as *mut ::core::ffi::c_char,
                        &raw mut file_info,
                    ) || file_info.stat.st_uid != (*file_info_old).stat.st_uid
                        || file_info.stat.st_gid != (*file_info_old).stat.st_gid
                        || file_info.stat.st_mode as ::core::ffi::c_int != perm
                    {
                        *backup_copyp = true_0 != 0;
                    }
                    close(fd);
                    os_remove(&raw mut tmp_fname as *mut ::core::ffi::c_char);
                }
            }
        }
        if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            || bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        {
            let mut file_info_link_ok: bool = os_fileinfo_link(fname, &raw mut file_info);
            if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                && file_info_link_ok as ::core::ffi::c_int != 0
                && !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
            {
                *backup_copyp = false_0 != 0;
            }
            if bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
                && os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
                && (!file_info_link_ok
                    || os_fileinfo_id_equal(&raw mut file_info, file_info_old)
                        as ::core::ffi::c_int
                        != 0)
            {
                *backup_copyp = false_0 != 0;
            }
        }
        let mut backup_ext: *mut ::core::ffi::c_char = (if *p_bex.get() as ::core::ffi::c_int == NUL
        {
            b".bak\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            p_bex.get() as *const ::core::ffi::c_char
        }) as *mut ::core::ffi::c_char;
        if *backup_copyp {
            let mut some_error: bool = false_0 != 0;
            let mut dirp: *mut ::core::ffi::c_char = p_bdir.get();
            while *dirp != 0 {
                *backupp = buf_get_backup_name(fname, &raw mut dirp, no_prepend_dot, backup_ext);
                if (*backupp).is_null() {
                    some_error = true_0 != 0;
                    break;
                } else {
                    let mut file_info_new: FileInfo = FileInfo {
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
                    if os_fileinfo(*backupp, &raw mut file_info_new) {
                        if os_fileinfo_id_equal(&raw mut file_info_new, file_info_old) {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                backupp as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            let _ = *ptr_;
                        } else if p_bk.get() == 0 {
                            let mut wp: *mut ::core::ffi::c_char = (*backupp)
                                .offset(strlen(*backupp) as isize)
                                .offset(-(1 as ::core::ffi::c_int as isize))
                                .offset(-(strlen(backup_ext) as isize));
                            wp = if wp > *backupp { wp } else { *backupp };
                            *wp = 'z' as ::core::ffi::c_char;
                            while *wp as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                                && os_fileinfo(*backupp, &raw mut file_info_new)
                                    as ::core::ffi::c_int
                                    != 0
                            {
                                *wp -= 1;
                            }
                            if *wp as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                    backupp as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr__0);
                                *ptr__0 = NULL;
                                let _ = *ptr__0;
                            }
                        }
                    }
                    if (*backupp).is_null() {
                        continue;
                    }
                    os_remove(*backupp);
                    if os_copy(fname, *backupp, UV_FS_COPYFILE_FICLONE) != 0 as ::core::ffi::c_int {
                        *err = set_err(gettext(
                            b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        let mut ptr__1: *mut *mut ::core::ffi::c_void =
                            backupp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__1);
                        *ptr__1 = NULL;
                        let _ = *ptr__1;
                        *backupp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        os_setperm(*backupp, perm & 0o777 as ::core::ffi::c_int);
                        if file_info_new.stat.st_gid != (*file_info_old).stat.st_gid
                            && os_chown(
                                *backupp,
                                -1 as ::core::ffi::c_int as uv_uid_t,
                                (*file_info_old).stat.st_gid as uv_gid_t,
                            ) != 0 as ::core::ffi::c_int
                        {
                            os_setperm(
                                *backupp,
                                perm & 0o707 as ::core::ffi::c_int
                                    | (perm & 0o7 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int,
                            );
                        }
                        os_file_settime(
                            *backupp,
                            (*file_info_old).stat.st_atim.tv_sec as ::core::ffi::c_double,
                            (*file_info_old).stat.st_mtim.tv_sec as ::core::ffi::c_double,
                        );
                        os_set_acl(*backupp, acl);
                        os_copy_xattr(fname, *backupp);
                        *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
                        break;
                    }
                }
            }
            if (*backupp).is_null() && (*err).msg.is_null() {
                *err = set_err(gettext(
                    b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            if (some_error as ::core::ffi::c_int != 0 || !(*err).msg.is_null()) && !forceit {
                return FAIL;
            }
            *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
        } else {
            if file_readonly as ::core::ffi::c_int != 0
                && !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
            {
                *err = set_err_num(
                    b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(err_readonly.get()),
                );
                return FAIL;
            }
            let mut dirp_0: *mut ::core::ffi::c_char = p_bdir.get();
            while *dirp_0 != 0 {
                *backupp = buf_get_backup_name(fname, &raw mut dirp_0, no_prepend_dot, backup_ext);
                if !(*backupp).is_null() {
                    if p_bk.get() == 0 && os_path_exists(*backupp) as ::core::ffi::c_int != 0 {
                        let mut p: *mut ::core::ffi::c_char = (*backupp)
                            .offset(strlen(*backupp) as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize))
                            .offset(-(strlen(backup_ext) as isize));
                        p = if p > *backupp { p } else { *backupp };
                        *p = 'z' as ::core::ffi::c_char;
                        while *p as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                            && os_path_exists(*backupp) as ::core::ffi::c_int != 0
                        {
                            *p -= 1;
                        }
                        if *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                            let mut ptr__2: *mut *mut ::core::ffi::c_void =
                                backupp as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__2);
                            *ptr__2 = NULL;
                            let _ = *ptr__2;
                        }
                    }
                }
                if (*backupp).is_null() {
                    continue;
                }
                if vim_rename(fname, *backupp) == 0 as ::core::ffi::c_int {
                    break;
                }
                let mut ptr__3: *mut *mut ::core::ffi::c_void =
                    backupp as *mut *mut ::core::ffi::c_void;
                xfree(*ptr__3);
                *ptr__3 = NULL;
                let _ = *ptr__3;
            }
            if (*backupp).is_null() && !forceit {
                *err = set_err(gettext(
                    b"E510: Can't make backup file (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        }
        return OK;
    }
}
