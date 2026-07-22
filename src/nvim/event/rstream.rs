use crate::src::nvim::event::libuv::{
    uv_err_name, uv_fs_read, uv_fs_req_cleanup, uv_idle_start, uv_idle_stop, uv_read_start,
    uv_read_stop, uv_strerror,
};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::stream::{stream_close_handle, stream_init, stream_may_close};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::memory::{alloc_block, free_block};
use crate::src::nvim::os::libc::{__assert_fail, memmove};
pub use crate::src::nvim::types::{
    Event, Loop, MultiQueue, RStream, Stream, __gid_t, __mode_t, __off_t, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __uid_t, argv_callback, gid_t,
    int64_t, loop_0, mode_t, multiqueue, off_t, pthread_mutex_t, pthread_rwlock_t, rstream, size_t,
    ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_10,
    stream_write_cb, uid_t, uint64_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv__work,
    uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t,
    uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_fs_cb,
    uv_fs_s, uv_fs_t, uv_fs_type, uv_gid_t, uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_0,
    uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_9, uv_idle_t,
    uv_loop_s, uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2,
    uv_loop_t, uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_8, uv_pipe_t, uv_read_cb,
    uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1,
    uv_signal_t, uv_stat_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_6, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_7, uv_tcp_t, uv_timespec_t, uv_uid_t,
};
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;
pub const UV_FILE: uv_handle_type = 17;
pub const UV_SIGNAL: uv_handle_type = 16;
pub const UV_UDP: uv_handle_type = 15;
pub const UV_TTY: uv_handle_type = 14;
pub const UV_TIMER: uv_handle_type = 13;
pub const UV_TCP: uv_handle_type = 12;
pub const UV_STREAM: uv_handle_type = 11;
pub const UV_PROCESS: uv_handle_type = 10;
pub const UV_PREPARE: uv_handle_type = 9;
pub const UV_POLL: uv_handle_type = 8;
pub const UV_NAMED_PIPE: uv_handle_type = 7;
pub const UV_IDLE: uv_handle_type = 6;
pub const UV_HANDLE: uv_handle_type = 5;
pub const UV_FS_POLL: uv_handle_type = 4;
pub const UV_FS_EVENT: uv_handle_type = 3;
pub const UV_CHECK: uv_handle_type = 2;
pub const UV_ASYNC: uv_handle_type = 1;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed_5 = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed_5 = -8;
pub const UV_EUNATCH: C2Rust_Unnamed_5 = -49;
pub const UV_ENODATA: C2Rust_Unnamed_5 = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed_5 = -94;
pub const UV_EILSEQ: C2Rust_Unnamed_5 = -84;
pub const UV_EFTYPE: C2Rust_Unnamed_5 = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed_5 = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed_5 = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed_5 = -112;
pub const UV_EMLINK: C2Rust_Unnamed_5 = -31;
pub const UV_ENXIO: C2Rust_Unnamed_5 = -6;
pub const UV_EOF: C2Rust_Unnamed_5 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_5 = -4094;
pub const UV_EXDEV: C2Rust_Unnamed_5 = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed_5 = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed_5 = -110;
pub const UV_ESRCH: C2Rust_Unnamed_5 = -3;
pub const UV_ESPIPE: C2Rust_Unnamed_5 = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed_5 = -108;
pub const UV_EROFS: C2Rust_Unnamed_5 = -30;
pub const UV_ERANGE: C2Rust_Unnamed_5 = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed_5 = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed_5 = -93;
pub const UV_EPROTO: C2Rust_Unnamed_5 = -71;
pub const UV_EPIPE: C2Rust_Unnamed_5 = -32;
pub const UV_EPERM: C2Rust_Unnamed_5 = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed_5 = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed_5 = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed_5 = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed_5 = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed_5 = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed_5 = -107;
pub const UV_ENOSYS: C2Rust_Unnamed_5 = -38;
pub const UV_ENOSPC: C2Rust_Unnamed_5 = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed_5 = -92;
pub const UV_ENONET: C2Rust_Unnamed_5 = -64;
pub const UV_ENOMEM: C2Rust_Unnamed_5 = -12;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_ENODEV: C2Rust_Unnamed_5 = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed_5 = -105;
pub const UV_ENFILE: C2Rust_Unnamed_5 = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed_5 = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed_5 = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed_5 = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed_5 = -90;
pub const UV_EMFILE: C2Rust_Unnamed_5 = -24;
pub const UV_ELOOP: C2Rust_Unnamed_5 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_5 = -21;
pub const UV_EISCONN: C2Rust_Unnamed_5 = -106;
pub const UV_EIO: C2Rust_Unnamed_5 = -5;
pub const UV_EINVAL: C2Rust_Unnamed_5 = -22;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed_5 = -113;
pub const UV_EFBIG: C2Rust_Unnamed_5 = -27;
pub const UV_EFAULT: C2Rust_Unnamed_5 = -14;
pub const UV_EEXIST: C2Rust_Unnamed_5 = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed_5 = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed_5 = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed_5 = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed_5 = -103;
pub const UV_ECHARSET: C2Rust_Unnamed_5 = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed_5 = -125;
pub const UV_EBUSY: C2Rust_Unnamed_5 = -16;
pub const UV_EBADF: C2Rust_Unnamed_5 = -9;
pub const UV_EALREADY: C2Rust_Unnamed_5 = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed_5 = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed_5 = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed_5 = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed_5 = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed_5 = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed_5 = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed_5 = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed_5 = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed_5 = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed_5 = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed_5 = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed_5 = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed_5 = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed_5 = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed_5 = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed_5 = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed_5 = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed_5 = -98;
pub const UV_EACCES: C2Rust_Unnamed_5 = -13;
pub const UV_E2BIG: C2Rust_Unnamed_5 = -7;
pub const UV_REQ_TYPE_MAX: uv_req_type = 11;
pub const UV_RANDOM: uv_req_type = 10;
pub const UV_GETNAMEINFO: uv_req_type = 9;
pub const UV_GETADDRINFO: uv_req_type = 8;
pub const UV_WORK: uv_req_type = 7;
pub const UV_FS: uv_req_type = 6;
pub const UV_UDP_SEND: uv_req_type = 5;
pub const UV_SHUTDOWN: uv_req_type = 4;
pub const UV_WRITE: uv_req_type = 3;
pub const UV_CONNECT: uv_req_type = 2;
pub const UV_REQ: uv_req_type = 1;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_FS_LUTIME: uv_fs_type = 36;
pub const UV_FS_MKSTEMP: uv_fs_type = 35;
pub const UV_FS_STATFS: uv_fs_type = 34;
pub const UV_FS_CLOSEDIR: uv_fs_type = 33;
pub const UV_FS_READDIR: uv_fs_type = 32;
pub const UV_FS_OPENDIR: uv_fs_type = 31;
pub const UV_FS_LCHOWN: uv_fs_type = 30;
pub const UV_FS_COPYFILE: uv_fs_type = 29;
pub const UV_FS_REALPATH: uv_fs_type = 28;
pub const UV_FS_FCHOWN: uv_fs_type = 27;
pub const UV_FS_CHOWN: uv_fs_type = 26;
pub const UV_FS_READLINK: uv_fs_type = 25;
pub const UV_FS_SYMLINK: uv_fs_type = 24;
pub const UV_FS_LINK: uv_fs_type = 23;
pub const UV_FS_SCANDIR: uv_fs_type = 22;
pub const UV_FS_RENAME: uv_fs_type = 21;
pub const UV_FS_MKDTEMP: uv_fs_type = 20;
pub const UV_FS_MKDIR: uv_fs_type = 19;
pub const UV_FS_RMDIR: uv_fs_type = 18;
pub const UV_FS_UNLINK: uv_fs_type = 17;
pub const UV_FS_FDATASYNC: uv_fs_type = 16;
pub const UV_FS_FSYNC: uv_fs_type = 15;
pub const UV_FS_FCHMOD: uv_fs_type = 14;
pub const UV_FS_CHMOD: uv_fs_type = 13;
pub const UV_FS_ACCESS: uv_fs_type = 12;
pub const UV_FS_FUTIME: uv_fs_type = 11;
pub const UV_FS_UTIME: uv_fs_type = 10;
pub const UV_FS_FTRUNCATE: uv_fs_type = 9;
pub const UV_FS_FSTAT: uv_fs_type = 8;
pub const UV_FS_LSTAT: uv_fs_type = 7;
pub const UV_FS_STAT: uv_fs_type = 6;
pub const UV_FS_SENDFILE: uv_fs_type = 5;
pub const UV_FS_WRITE: uv_fs_type = 4;
pub const UV_FS_READ: uv_fs_type = 3;
pub const UV_FS_CLOSE: uv_fs_type = 2;
pub const UV_FS_OPEN: uv_fs_type = 1;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const UV_FS_UNKNOWN: uv_fs_type = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub unsafe extern "C" fn rstream_init_fd(
    mut loop_0: *mut Loop,
    mut stream: *mut RStream,
    mut fd: ::core::ffi::c_int,
) {
    stream_init(
        loop_0,
        &raw mut (*stream).s,
        fd,
        ::core::ptr::null_mut::<uv_stream_t>(),
    );
    rstream_init(stream);
}
pub unsafe extern "C" fn rstream_init(mut stream: *mut RStream) {
    (*stream).read_cb = None;
    (*stream).num_bytes = 0 as size_t;
    (*stream).buffer = alloc_block() as *mut ::core::ffi::c_char;
    (*stream).write_pos = (*stream).buffer;
    (*stream).read_pos = (*stream).write_pos;
    (*stream).s.close_cb =
        Some(rstream_close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
            as stream_close_cb;
    (*stream).s.close_cb_data = stream as *mut ::core::ffi::c_void;
}
pub unsafe extern "C" fn rstream_start_inner(mut stream: *mut RStream) {
    if !(*stream).s.uvstream.is_null() {
        uv_read_start(
            (*stream).s.uvstream,
            Some(alloc_cb as unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()),
            Some(read_cb as unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()),
        );
    } else {
        uv_idle_start(
            &raw mut (*stream).s.uv.idle,
            Some(fread_idle_cb as unsafe extern "C" fn(*mut uv_idle_t) -> ()),
        );
    };
}
pub unsafe extern "C" fn rstream_start(
    mut stream: *mut RStream,
    mut cb: stream_read_cb,
    mut data: *mut ::core::ffi::c_void,
) {
    (*stream).read_cb = cb;
    (*stream).s.cb_data = data;
    (*stream).want_read = true_0 != 0;
    if !(*stream).paused_full {
        rstream_start_inner(stream);
    }
}
pub unsafe extern "C" fn rstream_stop_inner(mut stream: *mut RStream) {
    if !(*stream).s.uvstream.is_null() {
        uv_read_stop((*stream).s.uvstream);
    } else {
        uv_idle_stop(&raw mut (*stream).s.uv.idle);
    };
}
pub unsafe extern "C" fn rstream_stop(mut stream: *mut RStream) {
    rstream_stop_inner(stream);
    (*stream).want_read = false_0 != 0;
}
unsafe extern "C" fn alloc_cb(
    mut handle: *mut uv_handle_t,
    mut _suggested: size_t,
    mut buf: *mut uv_buf_t,
) {
    let mut stream: *mut RStream = (*handle).data as *mut RStream;
    (*buf).base = (*stream).write_pos;
    (*buf).len = rstream_space(stream);
}
unsafe extern "C" fn read_cb(
    mut uvstream: *mut uv_stream_t,
    mut cnt: ssize_t,
    mut _buf: *const uv_buf_t,
) {
    let mut stream: *mut RStream = (*uvstream).data as *mut RStream;
    if cnt <= 0 as ssize_t {
        if cnt == UV_ENOBUFS as ::core::ffi::c_int as ssize_t || cnt == 0 as ssize_t {
            return;
        } else if cnt == UV_EOF as ::core::ffi::c_int as ssize_t
            && (*uvstream).type_0 as ::core::ffi::c_uint
                == UV_TTY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            invoke_read_cb(stream, true_0 != 0);
        } else {
            logmsg(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"read_cb\0".as_ptr() as *const ::core::ffi::c_char,
                122 as ::core::ffi::c_int,
                true_0 != 0,
                b"closing Stream (%p): %s (%s)\0".as_ptr() as *const ::core::ffi::c_char,
                stream as *mut ::core::ffi::c_void,
                uv_err_name(cnt as ::core::ffi::c_int),
                uv_strerror(cnt as ::core::ffi::c_int),
            );
            uv_read_stop(uvstream);
            invoke_read_cb(stream, true_0 != 0);
        }
        return;
    }
    let mut nread: size_t = cnt as size_t;
    (*stream).num_bytes = (*stream).num_bytes.wrapping_add(nread);
    (*stream).write_pos = (*stream).write_pos.offset(cnt as isize);
    invoke_read_cb(stream, false_0 != 0);
}
unsafe extern "C" fn rstream_space(mut stream: *mut RStream) -> size_t {
    return (*stream)
        .buffer
        .offset(ARENA_BLOCK_SIZE as isize)
        .offset_from((*stream).write_pos) as size_t;
}
unsafe extern "C" fn fread_idle_cb(mut handle: *mut uv_idle_t) {
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
    let mut stream: *mut RStream = (*handle).data as *mut RStream;
    (*stream).uvbuf.base = (*stream).write_pos;
    (*stream).uvbuf.len = rstream_space(stream);
    uv_fs_read(
        (*handle).loop_0,
        &raw mut req,
        (*stream).s.fd,
        &raw mut (*stream).uvbuf as *const uv_buf_t,
        1 as ::core::ffi::c_uint,
        (*stream).s.fpos,
        None,
    );
    uv_fs_req_cleanup(&raw mut req);
    if req.result <= 0 as ssize_t {
        uv_idle_stop(&raw mut (*stream).s.uv.idle);
        invoke_read_cb(stream, true_0 != 0);
        return;
    }
    (*stream).write_pos = (*stream).write_pos.offset(req.result as isize);
    (*stream).s.fpos =
        ((*stream).s.fpos as ::core::ffi::c_long + req.result as ::core::ffi::c_long) as int64_t;
    invoke_read_cb(stream, false_0 != 0);
}
unsafe extern "C" fn read_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut stream: *mut RStream = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut RStream;
    (*stream).pending_read = false_0 != 0;
    if (*stream).read_cb.is_some() {
        let mut available: size_t = rstream_available(stream);
        let mut consumed: size_t = (*stream).read_cb.expect("non-null function pointer")(
            stream,
            (*stream).read_pos,
            available,
            (*stream).s.cb_data,
            (*stream).did_eof,
        );
        '_c2rust_label: {
            if consumed <= available {
            } else {
                __assert_fail(
                    b"consumed <= available\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/event/rstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    182 as ::core::ffi::c_uint,
                    b"void read_event(void **)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        rstream_consume(stream, consumed);
    }
    (*stream).s.pending_reqs = (*stream).s.pending_reqs.wrapping_sub(1);
    if (*stream).s.closed as ::core::ffi::c_int != 0 && (*stream).s.pending_reqs == 0 {
        stream_close_handle(&raw mut (*stream).s);
    }
}
pub unsafe extern "C" fn rstream_available(mut stream: *mut RStream) -> size_t {
    return (*stream).write_pos.offset_from((*stream).read_pos) as size_t;
}
pub unsafe extern "C" fn rstream_consume(mut stream: *mut RStream, mut consumed: size_t) {
    (*stream).read_pos = (*stream).read_pos.offset(consumed as isize);
    let mut remaining: size_t = (*stream).write_pos.offset_from((*stream).read_pos) as size_t;
    if remaining > 0 as size_t && (*stream).read_pos > (*stream).buffer {
        memmove(
            (*stream).buffer as *mut ::core::ffi::c_void,
            (*stream).read_pos as *const ::core::ffi::c_void,
            remaining,
        );
        (*stream).read_pos = (*stream).buffer;
        (*stream).write_pos = (*stream).buffer.offset(remaining as isize);
    } else if remaining == 0 as size_t {
        (*stream).write_pos = (*stream).buffer;
        (*stream).read_pos = (*stream).write_pos;
    }
    if (*stream).want_read as ::core::ffi::c_int != 0
        && (*stream).paused_full as ::core::ffi::c_int != 0
        && rstream_space(stream) != 0
    {
        '_c2rust_label: {
            if (*stream).read_cb.is_some() {
            } else {
                __assert_fail(
                    b"stream->read_cb\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/event/rstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    210 as ::core::ffi::c_uint,
                    b"void rstream_consume(RStream *, size_t)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*stream).paused_full = false_0 != 0;
        rstream_start_inner(stream);
    }
}
unsafe extern "C" fn invoke_read_cb(mut stream: *mut RStream, mut eof: bool) {
    (*stream).did_eof = (*stream).did_eof as ::core::ffi::c_int | eof as ::core::ffi::c_int != 0;
    if rstream_space(stream) == 0 {
        rstream_stop_inner(stream);
        (*stream).paused_full = true_0 != 0;
    }
    if (*stream).pending_read {
        return;
    }
    (*stream).s.pending_reqs = (*stream).s.pending_reqs.wrapping_add(1);
    (*stream).pending_read = true_0 != 0;
    if !(*stream).s.events.is_null() {
        multiqueue_put_event(
            (*stream).s.events,
            Event {
                handler: Some(
                    read_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    stream as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    } else {
        let mut argv: [*mut ::core::ffi::c_void; 1] = [stream as *mut ::core::ffi::c_void];
        read_event(&raw mut argv as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn rstream_close_cb(mut s: *mut Stream, mut data: *mut ::core::ffi::c_void) {
    let mut stream: *mut RStream = data as *mut RStream;
    '_c2rust_label: {
        if !stream.is_null() && s == &raw mut (*stream).s {
        } else {
            __assert_fail(
                b"stream && s == &stream->s\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/rstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                239 as ::core::ffi::c_uint,
                b"void rstream_close_cb(Stream *, void *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if !(*stream).buffer.is_null() {
        free_block((*stream).buffer as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn rstream_may_close(mut stream: *mut RStream) {
    stream_may_close(&raw mut (*stream).s);
}
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
