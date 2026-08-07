//! File metadata: permissions, ownership, timestamps and identity.
//!
//! Every answer here comes from libuv's `uv_fs_*` synchronous calls, which
//! is why each function is the same four lines around a `uv_fs_t`: fill the
//! request, read `req.result`, clean it up. `FileInfo` and `FileID` are the
//! two shapes upstream wraps `uv_stat_t` in — the first is the whole stat
//! buffer, the second only the (device, inode) pair that says two paths are
//! the same file.

#![allow(unsafe_op_in_unsafe_fn)]

use super::*;
pub(crate) unsafe extern "C" fn os_stat(
    mut name: *const ::core::ffi::c_char,
    mut statbuf: *mut uv_stat_t,
) -> ::core::ffi::c_int {
    if name.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    fs_request(
        |request| uv_fs_stat(::core::ptr::null_mut::<uv_loop_t>(), request, name, None),
        |result, request| {
            if result == kLibuvSuccess.get() {
                *statbuf = request.statbuf;
            }
            result
        },
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_getperm(mut name: *const ::core::ffi::c_char) -> int32_t {
    let mut statbuf: uv_stat_t = uv_stat_t {
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
    };
    let mut stat_result: ::core::ffi::c_int = os_stat(name, &raw mut statbuf);
    if stat_result == kLibuvSuccess.get() {
        return statbuf.st_mode as int32_t;
    }
    return stat_result as int32_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_setperm(
    name: *const ::core::ffi::c_char,
    mut perm: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    fs_ok(|req| uv_fs_chmod(::core::ptr::null_mut::<uv_loop_t>(), req, name, perm, None))
}
pub unsafe extern "C" fn os_copy_xattr(
    mut from_file: *const ::core::ffi::c_char,
    mut to_file: *const ::core::ffi::c_char,
) {
    if from_file.is_null() {
        return;
    }
    let mut size: ssize_t = listxattr(
        from_file as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        0 as size_t,
    );
    if size <= 0 as ssize_t {
        return;
    }
    let mut xattr_buf: *mut ::core::ffi::c_char =
        xmalloc(size as size_t) as *mut ::core::ffi::c_char;
    size = listxattr(from_file, xattr_buf, size as size_t);
    let mut tsize: ssize_t = size;
    *__errno_location() = 0 as ::core::ffi::c_int;
    let mut max_vallen: ssize_t = 0 as ssize_t;
    let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_error_exit: while round < 2 as ::core::ffi::c_int {
        let mut key: *mut ::core::ffi::c_char = xattr_buf;
        if round == 1 as ::core::ffi::c_int {
            size = tsize;
        }
        while size > 0 as ssize_t {
            let mut vallen: ssize_t = getxattr(
                from_file,
                key,
                val as *mut ::core::ffi::c_void,
                if round != 0 {
                    max_vallen as size_t
                } else {
                    0 as size_t
                },
            );
            if !(vallen >= 0 as ssize_t
                && round != 0
                && setxattr(
                    to_file,
                    key,
                    val as *const ::core::ffi::c_void,
                    vallen as size_t,
                    0 as ::core::ffi::c_int,
                ) == 0 as ::core::ffi::c_int)
            {
                if *__errno_location() != 0 {
                    match *__errno_location() {
                        E2BIG => {
                            errmsg = E_XATTR_E2BIG.as_ptr();
                            break '_error_exit;
                        }
                        ENOTSUP | EACCES | EPERM => {}
                        ERANGE => {
                            errmsg = E_XATTR_ERANGE.as_ptr();
                            break '_error_exit;
                        }
                        _ => {
                            errmsg = E_XATTR_OTHER.as_ptr();
                            break '_error_exit;
                        }
                    }
                }
            }
            if round == 0 as ::core::ffi::c_int && vallen > max_vallen {
                max_vallen = vallen;
            }
            let mut keylen: ssize_t = strlen(key) as ssize_t + 1 as ssize_t;
            size -= keylen;
            key = key.offset(keylen);
        }
        if round != 0 {
            break;
        }
        val = xmalloc((max_vallen as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        round += 1;
    }
    xfree(xattr_buf as *mut ::core::ffi::c_void);
    xfree(val as *mut ::core::ffi::c_void);
    if !errmsg.is_null() {
        emsg(gettext(errmsg));
    }
}
/// Access control lists, which nvim does not implement on any platform:
/// upstream's `os_get_acl` answers NULL and the other two are `if (aclent
/// == NULL) return;` followed by nothing. Kept because `fileio.c` calls all
/// three around every write.
pub unsafe extern "C" fn os_get_acl(_fname: *const ::core::ffi::c_char) -> vim_acl_T {
    NULL
}

pub unsafe extern "C" fn os_set_acl(_fname: *const ::core::ffi::c_char, _aclent: vim_acl_T) {}

pub unsafe extern "C" fn os_free_acl(_aclent: vim_acl_T) {}
pub unsafe extern "C" fn os_file_owned(mut fname: *const ::core::ffi::c_char) -> bool {
    let mut uid: uid_t = getuid();
    let mut finfo: FileInfo = FileInfo {
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
    let mut file_owned: bool = os_fileinfo(fname, &raw mut finfo) as ::core::ffi::c_int != 0
        && finfo.stat.st_uid == uid as uint64_t;
    let mut link_owned: bool = os_fileinfo_link(fname, &raw mut finfo) as ::core::ffi::c_int != 0
        && finfo.stat.st_uid == uid as uint64_t;
    return file_owned as ::core::ffi::c_int != 0 && link_owned as ::core::ffi::c_int != 0;
}
pub unsafe extern "C" fn os_chown(
    mut path: *const ::core::ffi::c_char,
    mut owner: uv_uid_t,
    mut group: uv_gid_t,
) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_chown(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            owner,
            group,
            None,
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fchown(
    mut fd: ::core::ffi::c_int,
    mut owner: uv_uid_t,
    mut group: uv_gid_t,
) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_fchown(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            fd as uv_file,
            owner,
            group,
            None,
        )
    })
}
pub unsafe extern "C" fn os_file_settime(
    mut path: *const ::core::ffi::c_char,
    mut atime: ::core::ffi::c_double,
    mut mtime: ::core::ffi::c_double,
) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_utime(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            atime,
            mtime,
            None,
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_readable(mut name: *const ::core::ffi::c_char) -> bool {
    fs_result(|req| uv_fs_access(::core::ptr::null_mut::<uv_loop_t>(), req, name, R_OK, None)) == 0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_writable(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let r =
        fs_result(|req| uv_fs_access(::core::ptr::null_mut::<uv_loop_t>(), req, name, W_OK, None));
    if r == 0 as ::core::ffi::c_int {
        return if os_isdir(name) as ::core::ffi::c_int != 0 {
            2 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
    return 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo(
    mut path: *const ::core::ffi::c_char,
    mut file_info: *mut FileInfo,
) -> bool {
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    return os_stat(path, &raw mut (*file_info).stat) == kLibuvSuccess.get();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_link(
    mut path: *const ::core::ffi::c_char,
    mut file_info: *mut FileInfo,
) -> bool {
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    if path.is_null() {
        return false_0 != 0;
    }
    fs_request(
        |request| uv_fs_lstat(::core::ptr::null_mut::<uv_loop_t>(), request, path, None),
        |result, request| {
            let ok = result == kLibuvSuccess.get();
            if ok {
                (*file_info).stat = request.statbuf;
            }
            ok
        },
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_fd(
    mut file_descriptor: ::core::ffi::c_int,
    mut file_info: *mut FileInfo,
) -> bool {
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    fs_request(
        |request| {
            uv_fs_fstat(
                ::core::ptr::null_mut::<uv_loop_t>(),
                request,
                file_descriptor as uv_file,
                None,
            )
        },
        |result, request| {
            let ok = result == kLibuvSuccess.get();
            if ok {
                (*file_info).stat = request.statbuf;
            }
            ok
        },
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_id_equal(
    mut file_info_1: *const FileInfo,
    mut file_info_2: *const FileInfo,
) -> bool {
    return (*file_info_1).stat.st_ino == (*file_info_2).stat.st_ino
        && (*file_info_1).stat.st_dev == (*file_info_2).stat.st_dev;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_id(mut file_info: *const FileInfo, mut file_id: *mut FileID) {
    (*file_id).inode = (*file_info).stat.st_ino;
    (*file_id).device_id = (*file_info).stat.st_dev;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_inode(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_ino;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_size(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_size;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_hardlinks(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_nlink;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_blocksize(mut file_info: *const FileInfo) -> uint64_t {
    return (*file_info).stat.st_blksize;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid(
    mut path: *const ::core::ffi::c_char,
    mut file_id: *mut FileID,
) -> bool {
    let mut statbuf: uv_stat_t = uv_stat_t {
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
    };
    if os_stat(path, &raw mut statbuf) == kLibuvSuccess.get() {
        (*file_id).inode = statbuf.st_ino;
        (*file_id).device_id = statbuf.st_dev;
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid_equal(
    mut file_id_1: *const FileID,
    mut file_id_2: *const FileID,
) -> bool {
    return (*file_id_1).inode == (*file_id_2).inode
        && (*file_id_1).device_id == (*file_id_2).device_id;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileid_equal_fileinfo(
    mut file_id: *const FileID,
    mut file_info: *const FileInfo,
) -> bool {
    return (*file_id).inode == (*file_info).stat.st_ino
        && (*file_id).device_id == (*file_info).stat.st_dev;
}
