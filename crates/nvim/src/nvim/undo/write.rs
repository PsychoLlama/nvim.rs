//! `u_write_undo`: writing a buffer's undo tree out to its undo file.

use super::file::*;
use super::format::*;
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn u_write_undo(
    name: *const c_char,
    forceit: bool,
    buf: *mut buf_T,
    hash: *mut uint8_t,
) {
    let mut mark: c_int = 0;
    let mut uhp: *mut u_header_T = ptr::null_mut();
    let mut fd_0: c_int = 0;
    let mut file_info_old: FileInfo = FileInfo {
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
    let mut bi: bufinfo_T = bufinfo_T {
        bi_buf: ptr::null_mut(),
        bi_fp: ptr::null_mut(),
    };
    let mut file_name: *mut c_char = ptr::null_mut();
    let mut fp: *mut FILE = ptr::null_mut();
    let mut write_ok: bool = false;
    if name.is_null() {
        file_name = u_get_undo_file_name((*buf).b_ffname, false);
        if file_name.is_null() {
            if p_verbose.get() > 0 as OptInt {
                verbose_enter();
                smsg(
                    0,
                    c"%s".as_ptr(),
                    gettext(c"Cannot write undo file in any directory in 'undodir'".as_ptr()),
                );
                verbose_leave();
            }
            return;
        }
    } else {
        file_name = name as *mut c_char;
    }
    let mut perm: c_int = 0o600;
    if !(*buf).b_ffname.is_null() {
        perm = os_getperm((*buf).b_ffname) as c_int;
        if perm < 0 {
            perm = 0o600;
        }
    }
    perm = perm & 0o666;
    '_theend: {
        if os_path_exists(file_name) {
            if name.is_null() || !forceit {
                let mut fd: c_int = os_open(file_name, O_RDONLY, 0);
                if fd < 0 {
                    if !name.is_null() || p_verbose.get() > 0 as OptInt {
                        if name.is_null() {
                            verbose_enter();
                        }
                        smsg(
                            0,
                            gettext(c"Will not overwrite with undo file, cannot read: %s".as_ptr()),
                            file_name,
                        );
                        if name.is_null() {
                            verbose_leave();
                        }
                    }
                    break '_theend;
                } else {
                    let mut mbuf: [c_char; 9] = [0; 9];
                    let mut len: ssize_t = read_eintr(
                        fd,
                        &raw mut mbuf as *mut c_char as *mut c_void,
                        UF_START_MAGIC_LEN as size_t,
                    );
                    close(fd);
                    if len < UF_START_MAGIC_LEN as ssize_t
                        || memcmp(
                            &raw mut mbuf as *mut c_char as *const c_void,
                            UF_START_MAGIC.as_ptr() as *const c_void,
                            UF_START_MAGIC_LEN as size_t,
                        ) != 0
                    {
                        if !name.is_null() || p_verbose.get() > 0 as OptInt {
                            if name.is_null() {
                                verbose_enter();
                            }
                            smsg(
                                0,
                                gettext(
                                    c"Will not overwrite, this is not an undo file: %s".as_ptr(),
                                ),
                                file_name,
                            );
                            if name.is_null() {
                                verbose_leave();
                            }
                        }
                        break '_theend;
                    }
                }
            }
            os_remove(file_name);
        }
        if (*buf).b_u_numhead == 0 && (*buf).b_u_line_ptr.is_null() {
            if p_verbose.get() > 0 as OptInt {
                verb_msg(gettext(
                    c"Skipping undo file write, nothing to undo".as_ptr(),
                ));
            }
        } else {
            fd_0 = os_open(file_name, O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW, perm);
            if fd_0 < 0 {
                semsg(
                    gettext(c"E828: Cannot open undo file for writing: %s".as_ptr()),
                    file_name,
                );
            } else {
                os_setperm(file_name, perm);
                if p_verbose.get() > 0 as OptInt {
                    verbose_enter();
                    smsg(0, gettext(c"Writing undo file: %s".as_ptr()), file_name);
                    verbose_leave();
                }
                file_info_old = FileInfo {
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
                file_info_new = FileInfo {
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
                if !(*buf).b_ffname.is_null()
                    && os_fileinfo((*buf).b_ffname, &raw mut file_info_old) as c_int != 0
                    && os_fileinfo(file_name, &raw mut file_info_new)
                    && file_info_old.stat.st_gid != file_info_new.stat.st_gid
                    && os_fchown(
                        fd_0,
                        u32::MAX as uv_uid_t,
                        file_info_old.stat.st_gid as uv_gid_t,
                    ) != 0
                {
                    os_setperm(file_name, perm & 0o707 | (perm & 0o7) << 3);
                }
                fp = fdopen(fd_0, c"w".as_ptr());
                if fp.is_null() {
                    semsg(
                        gettext(c"E828: Cannot open undo file for writing: %s".as_ptr()),
                        file_name,
                    );
                    close(fd_0);
                    os_remove(file_name);
                } else {
                    u_sync(true);
                    bi = bufinfo_T {
                        bi_buf: buf,
                        bi_fp: fp,
                    };
                    '_write_error: {
                        if serialize_header(&raw mut bi, hash) {
                            (*lastmark.ptr()) += 1;
                            mark = lastmark.get();
                            uhp = (*buf).b_u_oldhead;
                            while !uhp.is_null() {
                                if (*uhp).uh_walk != mark {
                                    (*uhp).uh_walk = mark;
                                    if !serialize_uhp(&raw mut bi, uhp) {
                                        break '_write_error;
                                    }
                                }
                                if !(*uhp).uh_prev.ptr.is_null()
                                    && (*(*uhp).uh_prev.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_prev.ptr;
                                } else if !(*uhp).uh_alt_next.ptr.is_null()
                                    && (*(*uhp).uh_alt_next.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_alt_next.ptr;
                                } else if !(*uhp).uh_next.ptr.is_null()
                                    && (*uhp).uh_alt_prev.ptr.is_null()
                                    && (*(*uhp).uh_next.ptr).uh_walk != mark
                                {
                                    uhp = (*uhp).uh_next.ptr;
                                } else if !(*uhp).uh_alt_prev.ptr.is_null() {
                                    uhp = (*uhp).uh_alt_prev.ptr;
                                } else {
                                    uhp = (*uhp).uh_next.ptr;
                                }
                            }
                            if undo_write_bytes(
                                &raw mut bi,
                                UF_HEADER_END_MAGIC as uintmax_t,
                                2 as size_t,
                            ) {
                                write_ok = true;
                            }
                            if (if (*buf).b_p_fs >= 0 {
                                (*buf).b_p_fs
                            } else {
                                p_fs.get()
                            }) != 0
                                && fflush(fp) == 0
                                && os_fsync(fd_0) != 0
                            {
                                write_ok = false;
                            }
                        }
                    }
                    fclose(fp);
                    if !write_ok {
                        semsg(
                            gettext(c"E829: Write error in undo file: %s".as_ptr()),
                            file_name,
                        );
                    }
                    if !(*buf).b_ffname.is_null() {
                        let mut acl: vim_acl_T = os_get_acl((*buf).b_ffname);
                        os_set_acl(file_name, acl);
                        os_free_acl(acl);
                    }
                }
            }
        }
    }
    if file_name != name as *mut c_char {
        xfree(file_name as *mut c_void);
    }
}
