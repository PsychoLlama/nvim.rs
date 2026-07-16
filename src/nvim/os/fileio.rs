extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn alloc_block() -> *mut ::core::ffi::c_void;
    fn free_block(block: *mut ::core::ffi::c_void);
    fn os_open(
        path: *const ::core::ffi::c_char,
        flags: ::core::ffi::c_int,
        mode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn os_close(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_open_stdin_fd() -> ::core::ffi::c_int;
    fn os_read(
        fd: ::core::ffi::c_int,
        ret_eof: *mut bool,
        ret_buf: *mut ::core::ffi::c_char,
        size: size_t,
        non_blocking: bool,
    ) -> ptrdiff_t;
    fn os_readv(
        fd: ::core::ffi::c_int,
        ret_eof: *mut bool,
        iov: *mut iovec,
        iov_size: size_t,
        non_blocking: bool,
    ) -> ptrdiff_t;
    fn os_write(
        fd: ::core::ffi::c_int,
        buf: *const ::core::ffi::c_char,
        size: size_t,
        non_blocking: bool,
    ) -> ptrdiff_t;
    fn os_fsync(fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn os_file_mkdir(fname: *mut ::core::ffi::c_char, mode: int32_t) -> ::core::ffi::c_int;
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct iovec {
    pub iov_base: *mut ::core::ffi::c_void,
    pub iov_len: size_t,
}
pub type ptrdiff_t = isize;
pub type int32_t = i32;
pub type uint64_t = u64;
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed = -8;
pub const UV_EUNATCH: C2Rust_Unnamed = -49;
pub const UV_ENODATA: C2Rust_Unnamed = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed = -94;
pub const UV_EILSEQ: C2Rust_Unnamed = -84;
pub const UV_EFTYPE: C2Rust_Unnamed = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed = -112;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_ENXIO: C2Rust_Unnamed = -6;
pub const UV_EOF: C2Rust_Unnamed = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_EXDEV: C2Rust_Unnamed = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed = -110;
pub const UV_ESRCH: C2Rust_Unnamed = -3;
pub const UV_ESPIPE: C2Rust_Unnamed = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed = -108;
pub const UV_EROFS: C2Rust_Unnamed = -30;
pub const UV_ERANGE: C2Rust_Unnamed = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed = -93;
pub const UV_EPROTO: C2Rust_Unnamed = -71;
pub const UV_EPIPE: C2Rust_Unnamed = -32;
pub const UV_EPERM: C2Rust_Unnamed = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed = -107;
pub const UV_ENOSYS: C2Rust_Unnamed = -38;
pub const UV_ENOSPC: C2Rust_Unnamed = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed = -92;
pub const UV_ENONET: C2Rust_Unnamed = -64;
pub const UV_ENOMEM: C2Rust_Unnamed = -12;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENODEV: C2Rust_Unnamed = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ENFILE: C2Rust_Unnamed = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed = -90;
pub const UV_EMFILE: C2Rust_Unnamed = -24;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EISCONN: C2Rust_Unnamed = -106;
pub const UV_EIO: C2Rust_Unnamed = -5;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EINTR: C2Rust_Unnamed = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed = -113;
pub const UV_EFBIG: C2Rust_Unnamed = -27;
pub const UV_EFAULT: C2Rust_Unnamed = -14;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed = -103;
pub const UV_ECHARSET: C2Rust_Unnamed = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed = -125;
pub const UV_EBUSY: C2Rust_Unnamed = -16;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const UV_EALREADY: C2Rust_Unnamed = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed = -98;
pub const UV_EACCES: C2Rust_Unnamed = -13;
pub const UV_E2BIG: C2Rust_Unnamed = -7;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const kFileMkDir: C2Rust_Unnamed_0 = 256;
pub const kFileNonBlocking: C2Rust_Unnamed_0 = 128;
pub const kFileAppend: C2Rust_Unnamed_0 = 64;
pub const kFileTruncate: C2Rust_Unnamed_0 = 32;
pub const kFileCreateOnly: C2Rust_Unnamed_0 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_0 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_0 = 4;
pub const kFileCreate: C2Rust_Unnamed_0 = 2;
pub const kFileReadOnly: C2Rust_Unnamed_0 = 1;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn file_space(mut fp: *mut FileDescriptor) -> size_t {
    return (*fp)
        .buffer
        .offset(ARENA_BLOCK_SIZE as isize)
        .offset_from((*fp).write_pos) as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn file_open(
    ret_fp: *mut FileDescriptor,
    fname: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
    mode: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut os_open_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wr: TriState = kNone;
    if flags & kFileWriteOnly as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o1 as ::core::ffi::c_int;
        '_c2rust_label: {};
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileCreateOnly as ::core::ffi::c_int != 0 {
        os_open_flags |=
            0o100 as ::core::ffi::c_int | 0o200 as ::core::ffi::c_int | 0o1 as ::core::ffi::c_int;
        '_c2rust_label_0: {};
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileCreate as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o100 as ::core::ffi::c_int | 0o1 as ::core::ffi::c_int;
        '_c2rust_label_1: {
            if flags & kFileCreateOnly as ::core::ffi::c_int == 0 {
            } else {
                __assert_fail(
                    b"!(flags & kFileCreateOnly)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    59 as ::core::ffi::c_uint,
                    b"int file_open(FileDescriptor *const, const char *const, const int, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileTruncate as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o1000 as ::core::ffi::c_int | 0o1 as ::core::ffi::c_int;
        '_c2rust_label_2: {
            if flags & kFileCreateOnly as ::core::ffi::c_int == 0 {
            } else {
                __assert_fail(
                    b"!(flags & kFileCreateOnly)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    61 as ::core::ffi::c_uint,
                    b"int file_open(FileDescriptor *const, const char *const, const int, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileAppend as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o2000 as ::core::ffi::c_int | 0o1 as ::core::ffi::c_int;
        '_c2rust_label_3: {
            if flags & kFileCreateOnly as ::core::ffi::c_int == 0 {
            } else {
                __assert_fail(
                    b"!(flags & kFileCreateOnly)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    63 as ::core::ffi::c_uint,
                    b"int file_open(FileDescriptor *const, const char *const, const int, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileReadOnly as ::core::ffi::c_int != 0 {
        os_open_flags |= 0 as ::core::ffi::c_int;
        '_c2rust_label_4: {
            if wr as ::core::ffi::c_int != kTrue as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"wr != kTrue\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    64 as ::core::ffi::c_uint,
                    b"int file_open(FileDescriptor *const, const char *const, const int, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if kFalse as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kFalse;
        }
    }
    if flags & kFileNoSymlink as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o400000 as ::core::ffi::c_int;
        '_c2rust_label_5: {};
        if kNone as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kNone;
        }
    }
    if flags & kFileMkDir as ::core::ffi::c_int != 0 {
        os_open_flags |= 0o100 as ::core::ffi::c_int | 0o1 as ::core::ffi::c_int;
        '_c2rust_label_6: {
            if flags & kFileCreateOnly as ::core::ffi::c_int == 0 {
            } else {
                __assert_fail(
                    b"!(flags & kFileCreateOnly)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    67 as ::core::ffi::c_uint,
                    b"int file_open(FileDescriptor *const, const char *const, const int, const int)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if kTrue as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            wr = kTrue;
        }
    }
    if flags & kFileMkDir as ::core::ffi::c_int != 0 {
        let mut mkdir_ret: ::core::ffi::c_int =
            os_file_mkdir(fname as *mut ::core::ffi::c_char, 0o755 as int32_t);
        if mkdir_ret < 0 as ::core::ffi::c_int {
            return mkdir_ret;
        }
    }
    let fd: ::core::ffi::c_int = os_open(fname, os_open_flags, mode);
    if fd < 0 as ::core::ffi::c_int {
        return fd;
    }
    return file_open_fd(ret_fp, fd, flags);
}
#[no_mangle]
pub unsafe extern "C" fn file_open_fd(
    ret_fp: *mut FileDescriptor,
    fd: ::core::ffi::c_int,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    (*ret_fp).wr = flags
        & (kFileCreate as ::core::ffi::c_int
            | kFileCreateOnly as ::core::ffi::c_int
            | kFileTruncate as ::core::ffi::c_int
            | kFileAppend as ::core::ffi::c_int
            | kFileWriteOnly as ::core::ffi::c_int)
        != 0;
    (*ret_fp).non_blocking = flags & kFileNonBlocking as ::core::ffi::c_int != 0;
    '_c2rust_label: {
        if !(*ret_fp).wr || !(*ret_fp).non_blocking {
        } else {
            __assert_fail(
                b"!ret_fp->wr || !ret_fp->non_blocking\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                113 as ::core::ffi::c_uint,
                b"int file_open_fd(FileDescriptor *const, const int, const int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*ret_fp).fd = fd;
    (*ret_fp).eof = false_0 != 0;
    (*ret_fp).buffer = alloc_block() as *mut ::core::ffi::c_char;
    (*ret_fp).read_pos = (*ret_fp).buffer;
    (*ret_fp).write_pos = (*ret_fp).buffer;
    (*ret_fp).bytes_read = 0 as uint64_t;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_open_stdin(mut fp: *mut FileDescriptor) -> ::core::ffi::c_int {
    let mut error: ::core::ffi::c_int = file_open_fd(
        fp,
        os_open_stdin_fd(),
        kFileReadOnly as ::core::ffi::c_int | kFileNonBlocking as ::core::ffi::c_int,
    );
    if error != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"file_open_stdin\0".as_ptr() as *const ::core::ffi::c_char,
            129 as ::core::ffi::c_int,
            true_0 != 0,
            b"failed to open stdin: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(error),
        );
    }
    return error;
}
#[no_mangle]
pub unsafe extern "C" fn file_open_buffer(
    mut ret_fp: *mut FileDescriptor,
    mut data: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    (*ret_fp).wr = false_0 != 0;
    (*ret_fp).non_blocking = false_0 != 0;
    (*ret_fp).fd = -1 as ::core::ffi::c_int;
    (*ret_fp).eof = true_0 != 0;
    (*ret_fp).buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ret_fp).read_pos = data;
    (*ret_fp).write_pos = data.offset(len as isize);
    (*ret_fp).bytes_read = 0 as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> ::core::ffi::c_int {
    if (*fp).fd < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    let flush_error: ::core::ffi::c_int = if do_fsync as ::core::ffi::c_int != 0 {
        file_fsync(fp)
    } else {
        file_flush(fp)
    };
    let close_error: ::core::ffi::c_int = os_close((*fp).fd);
    free_block((*fp).buffer as *mut ::core::ffi::c_void);
    if close_error != 0 as ::core::ffi::c_int {
        return close_error;
    }
    return flush_error;
}
#[no_mangle]
pub unsafe extern "C" fn file_fsync(fp: *mut FileDescriptor) -> ::core::ffi::c_int {
    if !(*fp).wr {
        return 0 as ::core::ffi::c_int;
    }
    let flush_error: ::core::ffi::c_int = file_flush(fp);
    if flush_error != 0 as ::core::ffi::c_int {
        return flush_error;
    }
    let fsync_error: ::core::ffi::c_int = os_fsync((*fp).fd);
    if fsync_error != UV_EINVAL as ::core::ffi::c_int
        && fsync_error != UV_EROFS as ::core::ffi::c_int
        && fsync_error != UV_ENOTSUP as ::core::ffi::c_int
    {
        return fsync_error;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_flush(mut fp: *mut FileDescriptor) -> ::core::ffi::c_int {
    if !(*fp).wr {
        return 0 as ::core::ffi::c_int;
    }
    let mut to_write: ptrdiff_t = (*fp).write_pos.offset_from((*fp).read_pos);
    if to_write == 0 as ptrdiff_t {
        return 0 as ::core::ffi::c_int;
    }
    let wres: ptrdiff_t = os_write(
        (*fp).fd,
        (*fp).read_pos,
        to_write as size_t,
        (*fp).non_blocking,
    );
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).write_pos;
    if wres != to_write {
        return if wres >= 0 as ptrdiff_t {
            UV_EIO as ::core::ffi::c_int
        } else {
            wres as ::core::ffi::c_int
        };
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read(
    fp: *mut FileDescriptor,
    ret_buf: *mut ::core::ffi::c_char,
    size: size_t,
) -> ptrdiff_t {
    '_c2rust_label: {
        if !(*fp).wr {
        } else {
            __assert_fail(
                b"!fp->wr\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                230 as ::core::ffi::c_uint,
                b"ptrdiff_t file_read(FileDescriptor *const, char *const, const size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut from_buffer: size_t = if ((*fp).write_pos.offset_from((*fp).read_pos) as size_t) < size
    {
        (*fp).write_pos.offset_from((*fp).read_pos) as size_t
    } else {
        size
    };
    memcpy(
        ret_buf as *mut ::core::ffi::c_void,
        (*fp).read_pos as *const ::core::ffi::c_void,
        from_buffer,
    );
    let mut buf: *mut ::core::ffi::c_char = ret_buf.offset(from_buffer as isize);
    let mut read_remaining: size_t = size.wrapping_sub(from_buffer);
    if read_remaining == 0 {
        (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
            .wrapping_add(from_buffer as ::core::ffi::c_ulong)
            as uint64_t;
        (*fp).read_pos = (*fp).read_pos.offset(from_buffer as isize);
        return from_buffer as ptrdiff_t;
    }
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).write_pos;
    let mut called_read: bool = false_0 != 0;
    while read_remaining != 0 {
        if (*fp).eof as ::core::ffi::c_int != 0
            || called_read as ::core::ffi::c_int != 0
                && (*fp).non_blocking as ::core::ffi::c_int != 0
        {
            break;
        }
        let mut iov: [iovec; 2] = [
            iovec {
                iov_base: buf as *mut ::core::ffi::c_void,
                iov_len: read_remaining,
            },
            iovec {
                iov_base: (*fp).write_pos as *mut ::core::ffi::c_void,
                iov_len: ARENA_BLOCK_SIZE as size_t,
            },
        ];
        let r_ret: ptrdiff_t = os_readv(
            (*fp).fd,
            &raw mut (*fp).eof,
            &raw mut iov as *mut iovec,
            ::core::mem::size_of::<[iovec; 2]>()
                .wrapping_div(::core::mem::size_of::<iovec>())
                .wrapping_div(
                    (::core::mem::size_of::<[iovec; 2]>()
                        .wrapping_rem(::core::mem::size_of::<iovec>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            (*fp).non_blocking,
        );
        if r_ret > 0 as ptrdiff_t {
            if r_ret > read_remaining as ptrdiff_t {
                (*fp).write_pos = (*fp)
                    .write_pos
                    .offset((r_ret - read_remaining as ptrdiff_t) as size_t as isize);
                read_remaining = 0 as size_t;
            } else {
                buf = buf.offset(r_ret as isize);
                read_remaining = read_remaining.wrapping_sub(r_ret as size_t);
            }
        } else if r_ret < 0 as ptrdiff_t {
            return r_ret;
        }
        called_read = true_0 != 0;
    }
    (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
        .wrapping_add(size.wrapping_sub(read_remaining) as ::core::ffi::c_ulong)
        as uint64_t;
    return size.wrapping_sub(read_remaining) as ptrdiff_t;
}
#[no_mangle]
pub unsafe extern "C" fn file_try_read_buffered(
    fp: *mut FileDescriptor,
    size: size_t,
) -> *mut ::core::ffi::c_char {
    if (*fp).write_pos.offset_from((*fp).read_pos) as size_t >= size {
        let mut ret: *mut ::core::ffi::c_char = (*fp).read_pos;
        (*fp).read_pos = (*fp).read_pos.offset(size as isize);
        (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
            .wrapping_add(size as ::core::ffi::c_ulong) as uint64_t;
        return ret;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn file_write(
    fp: *mut FileDescriptor,
    buf: *const ::core::ffi::c_char,
    size: size_t,
) -> ptrdiff_t {
    '_c2rust_label: {
        if (*fp).wr {
        } else {
            __assert_fail(
                b"fp->wr\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                333 as ::core::ffi::c_uint,
                b"ptrdiff_t file_write(FileDescriptor *const, const char *const, const size_t)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if size < file_space(fp) {
        memcpy(
            (*fp).write_pos as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            size,
        );
        (*fp).write_pos = (*fp).write_pos.offset(size as isize);
        return size as ptrdiff_t;
    }
    let mut status: ::core::ffi::c_int = file_flush(fp);
    if status < 0 as ::core::ffi::c_int {
        return status as ptrdiff_t;
    }
    if size < ARENA_BLOCK_SIZE as size_t {
        memcpy(
            (*fp).write_pos as *mut ::core::ffi::c_void,
            buf as *const ::core::ffi::c_void,
            size,
        );
        (*fp).write_pos = (*fp).write_pos.offset(size as isize);
        return size as ptrdiff_t;
    }
    let wres: ptrdiff_t = os_write((*fp).fd, buf, size, (*fp).non_blocking);
    return if wres != size as ptrdiff_t && wres >= 0 as ptrdiff_t {
        UV_EIO as ::core::ffi::c_int as ptrdiff_t
    } else {
        wres
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_skip(fp: *mut FileDescriptor, size: size_t) -> ptrdiff_t {
    '_c2rust_label: {
        if !(*fp).wr {
        } else {
            __assert_fail(
                b"!fp->wr\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/os/fileio.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                365 as ::core::ffi::c_uint,
                b"ptrdiff_t file_skip(FileDescriptor *const, const size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut from_buffer: size_t = if ((*fp).write_pos.offset_from((*fp).read_pos) as size_t) < size
    {
        (*fp).write_pos.offset_from((*fp).read_pos) as size_t
    } else {
        size
    };
    let mut skip_remaining: size_t = size.wrapping_sub(from_buffer);
    if skip_remaining == 0 as size_t {
        (*fp).read_pos = (*fp).read_pos.offset(from_buffer as isize);
        (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
            .wrapping_add(from_buffer as ::core::ffi::c_ulong)
            as uint64_t;
        return from_buffer as ptrdiff_t;
    }
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).write_pos;
    let mut called_read: bool = false_0 != 0;
    while skip_remaining > 0 as size_t {
        if (*fp).eof as ::core::ffi::c_int != 0
            || called_read as ::core::ffi::c_int != 0
                && (*fp).non_blocking as ::core::ffi::c_int != 0
        {
            break;
        }
        let r_ret: ptrdiff_t = os_read(
            (*fp).fd,
            &raw mut (*fp).eof,
            (*fp).buffer,
            ARENA_BLOCK_SIZE as size_t,
            (*fp).non_blocking,
        );
        if r_ret < 0 as ptrdiff_t {
            return r_ret;
        } else if r_ret as size_t > skip_remaining {
            (*fp).read_pos = (*fp).buffer.offset(skip_remaining as isize);
            (*fp).write_pos = (*fp).buffer.offset(r_ret as isize);
            (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
                .wrapping_add(size as ::core::ffi::c_ulong)
                as uint64_t;
            return size as ptrdiff_t;
        }
        skip_remaining = skip_remaining.wrapping_sub(r_ret as size_t);
        called_read = true_0 != 0;
    }
    (*fp).bytes_read = ((*fp).bytes_read as ::core::ffi::c_ulong)
        .wrapping_add(size.wrapping_sub(skip_remaining) as ::core::ffi::c_ulong)
        as uint64_t;
    return size.wrapping_sub(skip_remaining) as ptrdiff_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
