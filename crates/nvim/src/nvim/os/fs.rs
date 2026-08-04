use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_cwd, uv_exepath, uv_fs_access, uv_fs_chmod, uv_fs_chown, uv_fs_close,
    uv_fs_copyfile, uv_fs_fchown, uv_fs_fstat, uv_fs_fsync, uv_fs_lstat, uv_fs_mkdir,
    uv_fs_mkdtemp, uv_fs_open, uv_fs_realpath, uv_fs_rename, uv_fs_req_cleanup, uv_fs_rmdir,
    uv_fs_scandir, uv_fs_scandir_next, uv_fs_stat, uv_fs_unlink, uv_fs_utime, uv_strerror,
    uv_translate_sys_error,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::main::{e_mkdir, e_noname, g_stats, p_verbose, stdin_fd};
use crate::src::nvim::memory::{
    memcnt, xfree, xmalloc, xmemcpyz, xmemdupz, xstrchrnul, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{emsg, semsg, smsg, verbose_enter, verbose_leave};
use crate::src::nvim::os::env::os_getenv;
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, abort, dup, fcntl, fdopen, gettext, getuid, getxattr,
    listxattr, memset, read, readv, setxattr, strerror, strlen, write,
};
use crate::src::nvim::path::{
    FullName_save, append_path, dir_of_file_exists, get_past_head, gettail_dir, path_tail_with_sep,
    save_abs_path, vim_ispathsep,
};
use crate::src::nvim::types::libc::STDIN_FILENO;
use crate::src::nvim::types::{
    Directory, FILE, FileID, FileInfo, OptInt, int32_t, iovec, ptrdiff_t, size_t, ssize_t, uid_t,
    uint64_t, uv__queue, uv__work, uv_buf_t, uv_file, uv_fs_t, uv_fs_type, uv_gid_t, uv_loop_s,
    uv_loop_t, uv_req_type, uv_stat_t, uv_timespec_t, uv_uid_t, vim_acl_T,
};
use crate::src::nvim::ui::ui_call_chdir;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_EMLINK: C2Rust_Unnamed_5 = -31;
pub const UV_EOF: C2Rust_Unnamed_5 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_5 = -4094;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_ELOOP: C2Rust_Unnamed_5 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_5 = -21;
pub const UV_EINVAL: C2Rust_Unnamed_5 = -22;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
pub const UV_EEXIST: C2Rust_Unnamed_5 = -17;
pub const UV_EBADF: C2Rust_Unnamed_5 = -9;
pub const UV_EAGAIN: C2Rust_Unnamed_5 = -11;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const F_GETFD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const F_SETFD: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FD_CLOEXEC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_NORMAL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NODE_OTHER: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_xattr_erange: GlobalCell<[::core::ffi::c_char; 51]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 51], [::core::ffi::c_char; 51]>(
        *b"E1506: Buffer too small to copy xattr value or key\0",
    )
});
static e_xattr_e2big: GlobalCell<[::core::ffi::c_char; 84]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 84], [::core::ffi::c_char; 84]>(
        *b"E1508: Size of the extended attribute value is larger than the maximum size allowed\0",
    )
});
static e_xattr_other: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E1509: Error occurred when reading or writing extended attribute\0",
    )
});
static kLibuvSuccess: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_chdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if p_verbose.get() >= 5 as OptInt {
        verbose_enter();
        smsg(
            0 as ::core::ffi::c_int,
            b"chdir(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            path,
        );
        verbose_leave();
    }
    let mut err: ::core::ffi::c_int = uv_chdir(path);
    if err == 0 as ::core::ffi::c_int {
        ui_call_chdir(cstr_as_string(path));
    }
    return err;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_dirname(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut error_number: ::core::ffi::c_int = 0;
    error_number = uv_cwd(buf, &raw mut len);
    if error_number != kLibuvSuccess.get() {
        xstrlcpy(buf, uv_strerror(error_number), len);
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn os_isrealdir(mut name: *const ::core::ffi::c_char) -> bool {
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    if uv_fs_lstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    ) != kLibuvSuccess.get()
    {
        return false_0 != 0;
    }
    if request.statbuf.st_mode & __S_IFMT as uint64_t == 0o120000 as uint64_t {
        return false_0 != 0;
    }
    return request.statbuf.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_isdir(mut name: *const ::core::ffi::c_char) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    return mode & __S_IFMT as int32_t == 0o40000 as int32_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_nodetype(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
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
    if 0 as ::core::ffi::c_int != os_stat(name, &raw mut statbuf) {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t
        || statbuf.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t
    {
        return NODE_NORMAL;
    }
    if statbuf.st_mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
        return NODE_OTHER;
    }
    return NODE_WRITABLE;
}
pub unsafe extern "C" fn os_exepath(
    mut buffer: *mut ::core::ffi::c_char,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    return uv_exepath(buffer, size);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_can_exe(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
    mut use_path: bool,
) -> bool {
    if !use_path || gettail_dir(name) != name {
        return (use_path as ::core::ffi::c_int != 0 || gettail_dir(name) != name)
            && is_executable(name, abspath) as ::core::ffi::c_int != 0;
    }
    return is_executable_in_path(name, abspath);
}
unsafe extern "C" fn is_executable(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut mode: int32_t = os_getperm(name);
    if mode < 0 as int32_t {
        return false_0 != 0;
    }
    let mut r: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if mode & __S_IFMT as int32_t == 0o100000 as int32_t {
        let mut req: uv_fs_t = uv_fs_t {
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
        };
        r = uv_fs_access(
            ::core::ptr::null_mut::<uv_loop_t>(),
            &raw mut req,
            name,
            1 as ::core::ffi::c_int,
            None,
        );
        uv_fs_req_cleanup(&raw mut req);
    }
    let ok: bool = r == 0 as ::core::ffi::c_int;
    if ok as ::core::ffi::c_int != 0 && !abspath.is_null() {
        *abspath = save_abs_path(name);
    }
    return ok;
}
unsafe extern "C" fn is_executable_in_path(
    mut name: *const ::core::ffi::c_char,
    mut abspath: *mut *mut ::core::ffi::c_char,
) -> bool {
    let mut path_env: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    if path_env.is_null() {
        return false_0 != 0;
    }
    let mut path: *mut ::core::ffi::c_char = xstrdup(path_env);
    let bufsize: size_t = strlen(name)
        .wrapping_add(strlen(path))
        .wrapping_add(2 as size_t);
    let mut buf: *mut ::core::ffi::c_char = xmalloc(bufsize) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = path;
    let mut rv: bool = false_0 != 0;
    loop {
        let mut e: *mut ::core::ffi::c_char = xstrchrnul(p, ENV_SEPCHAR as ::core::ffi::c_char);
        xmemcpyz(
            buf as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            e.offset_from(p) as size_t,
        );
        append_path(buf, name, bufsize);
        if is_executable(buf, abspath) {
            rv = true_0 != 0;
            break;
        } else {
            if *e as ::core::ffi::c_int != ENV_SEPCHAR {
                break;
            }
            p = e.offset(1 as ::core::ffi::c_int as isize);
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(path as *mut ::core::ffi::c_void);
    xfree(path_env as *mut ::core::ffi::c_void);
    return rv;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_open(
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if path.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_open(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        flags,
        mode,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
pub unsafe extern "C" fn os_fopen(
    mut path: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut FILE {
    '_c2rust_label: {
        if !flags.is_null() && strlen(flags) > 0 as size_t && strlen(flags) <= 2 as size_t {
        } else {
            __assert_fail(
                b"flags != NULL && strlen(flags) > 0 && strlen(flags) <= 2\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                439 as ::core::ffi::c_uint,
                b"FILE *os_fopen(const char *, const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut iflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        || *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'b' as ::core::ffi::c_int
    {
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDONLY;
            }
            119 => {
                iflags = O_WRONLY | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_WRONLY | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    } else {
        '_c2rust_label_0: {
            if *flags.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '+' as ::core::ffi::c_int
            {
            } else {
                __assert_fail(
                    b"flags[1] == '+'\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    464 as ::core::ffi::c_uint,
                    b"FILE *os_fopen(const char *, const char *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        match *flags.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            114 => {
                iflags = O_RDWR;
            }
            119 => {
                iflags = O_RDWR | O_CREAT | O_TRUNC;
            }
            97 => {
                iflags = O_RDWR | O_CREAT | O_APPEND;
            }
            _ => {
                abort();
            }
        }
    }
    let mut fd: ::core::ffi::c_int = os_open(path, iflags, 0o666 as ::core::ffi::c_int);
    if fd < 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<FILE>();
    }
    return fdopen(fd, flags);
}
pub unsafe extern "C" fn os_set_cloexec(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut e: ::core::ffi::c_int = 0;
    let mut fdflags: ::core::ffi::c_int = fcntl(fd, F_GETFD);
    if fdflags < 0 as ::core::ffi::c_int {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_set_cloexec\0".as_ptr() as *const ::core::ffi::c_char,
            497 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to get flags on descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    if fdflags & FD_CLOEXEC == 0 as ::core::ffi::c_int
        && fcntl(fd, F_SETFD, fdflags | FD_CLOEXEC) == -1 as ::core::ffi::c_int
    {
        e = *__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_set_cloexec\0".as_ptr() as *const ::core::ffi::c_char,
            504 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to set CLOEXEC on descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
            fd,
            strerror(e),
        );
        *__errno_location() = e;
        return -1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_close(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_close(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_dup(fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0;
    loop {
        ret = dup(fd);
        if ret < 0 as ::core::ffi::c_int {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if error == UV_EINTR as ::core::ffi::c_int {
                continue;
            }
            return error;
        } else {
            return ret;
        }
    }
}
pub unsafe extern "C" fn os_open_stdin_fd() -> ::core::ffi::c_int {
    let mut stdin_dup_fd: ::core::ffi::c_int = 0;
    if stdin_fd.get() > 0 as ::core::ffi::c_int {
        stdin_dup_fd = stdin_fd.get();
    } else {
        stdin_dup_fd = os_dup(STDIN_FILENO);
    }
    return stdin_dup_fd;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_read(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    ret_buf: *mut ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    if ret_buf.is_null() {
        '_c2rust_label: {
            if size == 0 as size_t {
            } else {
                __assert_fail(
                    b"size == 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    588 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_read(const int, _Bool *const, char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return 0 as ptrdiff_t;
    }
    let mut read_bytes: size_t = 0 as size_t;
    while read_bytes != size {
        '_c2rust_label_0: {
            if size >= read_bytes {
            } else {
                __assert_fail(
                    b"size >= read_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    593 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_read(const int, _Bool *const, char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let cur_read_bytes: ptrdiff_t = read(
            fd,
            ret_buf.offset(read_bytes as isize) as *mut ::core::ffi::c_void,
            size.wrapping_sub(read_bytes),
        ) as ptrdiff_t;
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
        }
        if cur_read_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else {
            if cur_read_bytes != 0 as ptrdiff_t {
                continue;
            }
            *ret_eof = true_0 != 0;
            break;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_readv(
    fd: ::core::ffi::c_int,
    ret_eof: *mut bool,
    mut iov: *mut iovec,
    mut iov_size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    *ret_eof = false_0 != 0;
    let mut read_bytes: size_t = 0 as size_t;
    let mut toread: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < iov_size {
        '_c2rust_label: {
            if toread
                <= (18446744073709551615 as size_t).wrapping_sub((*iov.offset(i as isize)).iov_len)
            {
            } else {
                __assert_fail(
                    b"toread <= SIZE_MAX - iov[i].iov_len\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    642 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_readv(const int, _Bool *const, struct iovec *, size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        toread = toread.wrapping_add((*iov.offset(i as isize)).iov_len);
        i = i.wrapping_add(1);
    }
    while read_bytes < toread && iov_size != 0 && !*ret_eof {
        let mut cur_read_bytes: ptrdiff_t =
            readv(fd, iov, iov_size as ::core::ffi::c_int) as ptrdiff_t;
        if cur_read_bytes == 0 as ptrdiff_t {
            *ret_eof = true_0 != 0;
        }
        if cur_read_bytes > 0 as ptrdiff_t {
            read_bytes = read_bytes.wrapping_add(cur_read_bytes as size_t);
            while iov_size != 0 && cur_read_bytes != 0 {
                if cur_read_bytes < (*iov).iov_len as ptrdiff_t {
                    (*iov).iov_len = (*iov).iov_len.wrapping_sub(cur_read_bytes as size_t);
                    (*iov).iov_base = ((*iov).iov_base as *mut ::core::ffi::c_char)
                        .offset(cur_read_bytes as isize)
                        as *mut ::core::ffi::c_void;
                    cur_read_bytes = 0 as ptrdiff_t;
                } else {
                    cur_read_bytes -= (*iov).iov_len as ptrdiff_t;
                    iov_size = iov_size.wrapping_sub(1);
                    iov = iov.offset(1);
                }
            }
        } else {
            if cur_read_bytes >= 0 as ptrdiff_t {
                continue;
            }
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        }
    }
    return read_bytes as ptrdiff_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_write(
    fd: ::core::ffi::c_int,
    buf: *const ::core::ffi::c_char,
    size: size_t,
    non_blocking: bool,
) -> ptrdiff_t {
    if buf.is_null() {
        '_c2rust_label: {
            if size == 0 as size_t {
            } else {
                __assert_fail(
                    b"size == 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    691 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_write(const int, const char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return 0 as ptrdiff_t;
    }
    let mut written_bytes: size_t = 0 as size_t;
    while written_bytes != size {
        '_c2rust_label_0: {
            if size >= written_bytes {
            } else {
                __assert_fail(
                    b"size >= written_bytes\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/fs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    696 as ::core::ffi::c_uint,
                    b"ptrdiff_t os_write(const int, const char *const, const size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let cur_written_bytes: ptrdiff_t = write(
            fd,
            buf.offset(written_bytes as isize) as *const ::core::ffi::c_void,
            size.wrapping_sub(written_bytes),
        ) as ptrdiff_t;
        if cur_written_bytes > 0 as ptrdiff_t {
            written_bytes = written_bytes.wrapping_add(cur_written_bytes as size_t);
        }
        if cur_written_bytes < 0 as ptrdiff_t {
            let error: ::core::ffi::c_int = uv_translate_sys_error(*__errno_location());
            *__errno_location() = 0 as ::core::ffi::c_int;
            if non_blocking as ::core::ffi::c_int != 0 && error == UV_EAGAIN as ::core::ffi::c_int {
                break;
            }
            if error == UV_EINTR as ::core::ffi::c_int || error == UV_EAGAIN as ::core::ffi::c_int {
                continue;
            }
            return error as ptrdiff_t;
        } else if cur_written_bytes == 0 as ptrdiff_t {
            return UV_UNKNOWN as ::core::ffi::c_int as ptrdiff_t;
        }
    }
    return written_bytes as ptrdiff_t;
}
pub unsafe extern "C" fn os_copy(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_copyfile(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        new_path,
        flags,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
pub unsafe extern "C" fn os_fsync(mut fd: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_fsync(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    (*g_stats.ptr()).fsync += 1;
    return r;
}
unsafe extern "C" fn os_stat(
    mut name: *const ::core::ffi::c_char,
    mut statbuf: *mut uv_stat_t,
) -> ::core::ffi::c_int {
    if name.is_null() {
        return UV_EINVAL as ::core::ffi::c_int;
    }
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    let mut result: ::core::ffi::c_int = uv_fs_stat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    );
    if result == kLibuvSuccess.get() {
        *statbuf = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return result;
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
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_chmod(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        perm,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return if r == kLibuvSuccess.get() { OK } else { FAIL };
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
                            errmsg =
                                (e_xattr_e2big.ptr() as *const _) as *const ::core::ffi::c_char;
                            break '_error_exit;
                        }
                        ENOTSUP | EACCES | EPERM => {}
                        ERANGE => {
                            errmsg =
                                (e_xattr_erange.ptr() as *const _) as *const ::core::ffi::c_char;
                            break '_error_exit;
                        }
                        _ => {
                            errmsg =
                                (e_xattr_other.ptr() as *const _) as *const ::core::ffi::c_char;
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
            key = key.offset(keylen as isize);
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
pub unsafe extern "C" fn os_get_acl(mut _fname: *const ::core::ffi::c_char) -> vim_acl_T {
    let mut ret: vim_acl_T = NULL;
    return ret;
}
pub unsafe extern "C" fn os_set_acl(mut _fname: *const ::core::ffi::c_char, mut aclent: vim_acl_T) {
    if aclent.is_null() {
        return;
    }
}
pub unsafe extern "C" fn os_free_acl(mut aclent: vim_acl_T) {
    if aclent.is_null() {
        return;
    }
}
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
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_chown(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        owner,
        group,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fchown(
    mut fd: ::core::ffi::c_int,
    mut owner: uv_uid_t,
    mut group: uv_gid_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_fchown(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        fd as uv_file,
        owner,
        group,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_path_exists(mut path: *const ::core::ffi::c_char) -> bool {
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
    return os_stat(path, &raw mut statbuf) == kLibuvSuccess.get();
}
pub unsafe extern "C" fn os_file_settime(
    mut path: *const ::core::ffi::c_char,
    mut atime: ::core::ffi::c_double,
    mut mtime: ::core::ffi::c_double,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_utime(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        atime,
        mtime,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_readable(mut name: *const ::core::ffi::c_char) -> bool {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_access(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        4 as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r == 0 as ::core::ffi::c_int;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_file_is_writable(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_access(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        name,
        2 as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
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
pub unsafe extern "C" fn os_rename(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_rename(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        new_path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return if r == kLibuvSuccess.get() { OK } else { FAIL };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir(
    mut path: *const ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_mkdir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        mode as ::core::ffi::c_int,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir_recurse(
    dir: *const ::core::ffi::c_char,
    mut mode: int32_t,
    failed_dir: *mut *mut ::core::ffi::c_char,
    created: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let dirlen: size_t = strlen(dir);
    let curdir: *mut ::core::ffi::c_char =
        xmemdupz(dir as *const ::core::ffi::c_void, dirlen) as *mut ::core::ffi::c_char;
    let past_head: *mut ::core::ffi::c_char = get_past_head(curdir);
    let mut e: *mut ::core::ffi::c_char = curdir.offset(dirlen as isize);
    let real_end: *const ::core::ffi::c_char = e;
    let past_head_save: ::core::ffi::c_char = *past_head;
    while !os_isdir(curdir) {
        e = path_tail_with_sep(curdir);
        if e <= past_head {
            *past_head = NUL as ::core::ffi::c_char;
            break;
        } else {
            *e = NUL as ::core::ffi::c_char;
        }
    }
    while e != real_end as *mut ::core::ffi::c_char {
        if e > past_head {
            *e = PATHSEP as ::core::ffi::c_char;
        } else {
            *past_head = past_head_save;
        }
        let component_len: size_t = strlen(e);
        e = e.offset(component_len as isize);
        if e == real_end as *mut ::core::ffi::c_char
            && memcnt(
                e.offset(-(component_len as isize)) as *const ::core::ffi::c_void,
                PATHSEP as ::core::ffi::c_char,
                component_len,
            ) == component_len
        {
            break;
        }
        let mut ret: ::core::ffi::c_int = 0;
        ret = os_mkdir(curdir, mode);
        if ret != 0 as ::core::ffi::c_int {
            *failed_dir = curdir;
            return ret;
        } else if !created.is_null() && (*created).is_null() {
            *created = FullName_save(curdir, false_0 != 0);
        }
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_file_mkdir(
    mut fname: *mut ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    if !dir_of_file_exists(fname) {
        let mut tail: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
        let mut last_char: *mut ::core::ffi::c_char = tail
            .offset(strlen(tail) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
        if vim_ispathsep(*last_char as ::core::ffi::c_int) {
            emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_char = *tail;
        *tail = NUL as ::core::ffi::c_char;
        let mut r: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        r = os_mkdir_recurse(
            fname,
            mode,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if r < 0 as ::core::ffi::c_int {
            semsg(
                gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
                failed_dir,
                uv_strerror(r),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
        *tail = c;
        return r;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_mkdtemp(
    mut templ: *const ::core::ffi::c_char,
    mut path: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    let mut result: ::core::ffi::c_int = uv_fs_mkdtemp(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        templ,
        None,
    );
    if result == kLibuvSuccess.get() {
        xstrlcpy(path, request.path, TEMP_FILE_PATH_MAXLEN as size_t);
    }
    uv_fs_req_cleanup(&raw mut request);
    return result;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_rmdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_rmdir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
}
pub unsafe extern "C" fn os_scandir(
    mut dir: *mut Directory,
    mut path: *const ::core::ffi::c_char,
) -> bool {
    let mut r: ::core::ffi::c_int = uv_fs_scandir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut (*dir).request,
        path,
        0 as ::core::ffi::c_int,
        None,
    );
    if r < 0 as ::core::ffi::c_int {
        os_closedir(dir);
    }
    return r >= 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_scandir_next(mut dir: *mut Directory) -> *const ::core::ffi::c_char {
    let mut err: ::core::ffi::c_int =
        uv_fs_scandir_next(&raw mut (*dir).request, &raw mut (*dir).ent);
    return if err != UV_EOF as ::core::ffi::c_int {
        (*dir).ent.name
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn os_closedir(mut dir: *mut Directory) {
    uv_fs_req_cleanup(&raw mut (*dir).request);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_remove(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0;
    let mut req: uv_fs_t = uv_fs_t {
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
    };
    r = uv_fs_unlink(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut req,
        path,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    return r;
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
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    let mut ok: bool = uv_fs_lstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        path,
        None,
    ) == kLibuvSuccess.get();
    if ok {
        (*file_info).stat = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return ok;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_fileinfo_fd(
    mut file_descriptor: ::core::ffi::c_int,
    mut file_info: *mut FileInfo,
) -> bool {
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    memset(
        file_info as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<FileInfo>(),
    );
    let mut ok: bool = uv_fs_fstat(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        file_descriptor as uv_file,
        None,
    ) == kLibuvSuccess.get();
    if ok {
        (*file_info).stat = request.statbuf;
    }
    uv_fs_req_cleanup(&raw mut request);
    return ok;
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
pub unsafe extern "C" fn os_realpath(
    mut name: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> *mut ::core::ffi::c_char {
    let mut request: uv_fs_t = uv_fs_t {
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
    };
    let mut result: ::core::ffi::c_int = uv_fs_realpath(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut request,
        name,
        None,
    );
    if result == kLibuvSuccess.get() {
        if buf.is_null() {
            buf = xmalloc(len) as *mut ::core::ffi::c_char;
        }
        xstrlcpy(buf, request.ptr as *const ::core::ffi::c_char, len);
    }
    uv_fs_req_cleanup(&raw mut request);
    return if result == kLibuvSuccess.get() {
        buf
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENOTSUP: ::core::ffi::c_int = EOPNOTSUPP;
pub const EOPNOTSUPP: ::core::ffi::c_int = 95;
pub const EPERM: ::core::ffi::c_int = 1;
pub const E2BIG: ::core::ffi::c_int = 7;
pub const EACCES: ::core::ffi::c_int = 13;
pub const ERANGE: ::core::ffi::c_int = 34;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TEMP_FILE_PATH_MAXLEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
