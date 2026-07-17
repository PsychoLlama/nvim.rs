extern "C" {
    pub type loop_0;
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn uv_write(
        req: *mut uv_write_t,
        handle: *mut uv_stream_t,
        bufs: *const uv_buf_t,
        nbufs: ::core::ffi::c_uint,
        cb: uv_write_cb,
    ) -> ::core::ffi::c_int;
    fn uv_fs_req_cleanup(req: *mut uv_fs_t);
    fn uv_fs_write(
        loop_0: *mut uv_loop_t,
        req: *mut uv_fs_t,
        file: uv_file,
        bufs: *const uv_buf_t,
        nbufs: ::core::ffi::c_uint,
        offset: int64_t,
        cb: uv_fs_cb,
    ) -> ::core::ffi::c_int;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn stream_init(
        loop_0: *mut Loop,
        stream: *mut Stream,
        fd: ::core::ffi::c_int,
        uvstream: *mut uv_stream_t,
    );
    fn stream_close_handle(stream: *mut Stream);
}
pub type size_t = usize;
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __mode_t = ::core::ffi::c_uint;
pub type __off_t = ::core::ffi::c_long;
pub type off_t = __off_t;
pub type ssize_t = isize;
pub type int64_t = i64;
pub type uint64_t = u64;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
pub type gid_t = __gid_t;
pub type mode_t = __mode_t;
pub type uid_t = __uid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::core::ffi::c_uint,
    pub __writers: ::core::ffi::c_uint,
    pub __wrphase_futex: ::core::ffi::c_uint,
    pub __writers_futex: ::core::ffi::c_uint,
    pub __pad3: ::core::ffi::c_uint,
    pub __pad4: ::core::ffi::c_uint,
    pub __cur_writer: ::core::ffi::c_int,
    pub __shared: ::core::ffi::c_int,
    pub __rwelision: ::core::ffi::c_schar,
    pub __pad1: [::core::ffi::c_uchar; 7],
    pub __pad2: ::core::ffi::c_ulong,
    pub __flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__work {
    pub work: Option<unsafe extern "C" fn(*mut uv__work) -> ()>,
    pub done: Option<unsafe extern "C" fn(*mut uv__work, ::core::ffi::c_int) -> ()>,
    pub loop_0: *mut uv_loop_s,
    pub wq: uv__queue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_loop_s {
    pub data: *mut ::core::ffi::c_void,
    pub active_handles: ::core::ffi::c_uint,
    pub handle_queue: uv__queue,
    pub active_reqs: C2Rust_Unnamed_4,
    pub internal_fields: *mut ::core::ffi::c_void,
    pub stop_flag: ::core::ffi::c_uint,
    pub flags: ::core::ffi::c_ulong,
    pub backend_fd: ::core::ffi::c_int,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub watchers: *mut *mut uv__io_t,
    pub nwatchers: ::core::ffi::c_uint,
    pub nfds: ::core::ffi::c_uint,
    pub wq: uv__queue,
    pub wq_mutex: uv_mutex_t,
    pub wq_async: uv_async_t,
    pub cloexec_lock: uv_rwlock_t,
    pub closing_handles: *mut uv_handle_t,
    pub process_handles: uv__queue,
    pub prepare_handles: uv__queue,
    pub check_handles: uv__queue,
    pub idle_handles: uv__queue,
    pub async_handles: uv__queue,
    pub async_unused: Option<unsafe extern "C" fn() -> ()>,
    pub async_io_watcher: uv__io_t,
    pub async_wfd: ::core::ffi::c_int,
    pub timer_heap: C2Rust_Unnamed_2,
    pub timer_counter: uint64_t,
    pub time: uint64_t,
    pub signal_pipefd: [::core::ffi::c_int; 2],
    pub signal_io_watcher: uv__io_t,
    pub child_watcher: uv_signal_t,
    pub emfile_fd: ::core::ffi::c_int,
    pub inotify_read_watcher: uv__io_t,
    pub inotify_watchers: *mut ::core::ffi::c_void,
    pub inotify_fd: ::core::ffi::c_int,
}
pub type uv__io_t = uv__io_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__io_s {
    pub cb: uv__io_cb,
    pub pending_queue: uv__queue,
    pub watcher_queue: uv__queue,
    pub pevents: ::core::ffi::c_uint,
    pub events: ::core::ffi::c_uint,
    pub fd: ::core::ffi::c_int,
}
pub type uv__io_cb =
    Option<unsafe extern "C" fn(*mut uv_loop_s, *mut uv__io_s, ::core::ffi::c_uint) -> ()>;
pub type uv_signal_t = uv_signal_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_signal_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_1,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub signal_cb: uv_signal_cb,
    pub signum: ::core::ffi::c_int,
    pub tree_entry: C2Rust_Unnamed,
    pub caught_signals: ::core::ffi::c_uint,
    pub dispatched_signals: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub rbe_left: *mut uv_signal_s,
    pub rbe_right: *mut uv_signal_s,
    pub rbe_parent: *mut uv_signal_s,
    pub rbe_color: ::core::ffi::c_int,
}
pub type uv_signal_cb = Option<unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()>;
pub type uv_handle_t = uv_handle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_handle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_0,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_close_cb = Option<unsafe extern "C" fn(*mut uv_handle_t) -> ()>;
pub type uv_handle_type = ::core::ffi::c_uint;
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
pub type uv_loop_t = uv_loop_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_1 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_2 {
    pub min: *mut ::core::ffi::c_void,
    pub nelts: ::core::ffi::c_uint,
}
pub type uv_rwlock_t = pthread_rwlock_t;
pub type uv_async_t = uv_async_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_async_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_3,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub async_cb: uv_async_cb,
    pub queue: uv__queue,
    pub pending: ::core::ffi::c_int,
}
pub type uv_async_cb = Option<unsafe extern "C" fn(*mut uv_async_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_3 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_mutex_t = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_4 {
    pub unused: *mut ::core::ffi::c_void,
    pub count: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_buf_t {
    pub base: *mut ::core::ffi::c_char,
    pub len: size_t,
}
pub type uv_file = ::core::ffi::c_int;
pub type uv_gid_t = gid_t;
pub type uv_uid_t = uid_t;
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
pub type uv_req_type = ::core::ffi::c_uint;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stream_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_6,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
pub type uv_connection_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> ()>;
pub type uv_stream_t = uv_stream_s;
pub type uv_shutdown_t = uv_shutdown_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_shutdown_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub handle: *mut uv_stream_t,
    pub cb: uv_shutdown_cb,
}
pub type uv_shutdown_cb =
    Option<unsafe extern "C" fn(*mut uv_shutdown_t, ::core::ffi::c_int) -> ()>;
pub type uv_connect_t = uv_connect_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_connect_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_connect_cb,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
}
pub type uv_connect_cb = Option<unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()>;
pub type uv_read_cb =
    Option<unsafe extern "C" fn(*mut uv_stream_t, ssize_t, *const uv_buf_t) -> ()>;
pub type uv_alloc_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, size_t, *mut uv_buf_t) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tcp_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_7,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tcp_t = uv_tcp_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_pipe_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_8,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub write_queue_size: size_t,
    pub alloc_cb: uv_alloc_cb,
    pub read_cb: uv_read_cb,
    pub connect_req: *mut uv_connect_t,
    pub shutdown_req: *mut uv_shutdown_t,
    pub io_watcher: uv__io_t,
    pub write_queue: uv__queue,
    pub write_completed_queue: uv__queue,
    pub connection_cb: uv_connection_cb,
    pub delayed_error: ::core::ffi::c_int,
    pub accepted_fd: ::core::ffi::c_int,
    pub queued_fds: *mut ::core::ffi::c_void,
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_9,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_write_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub cb: uv_write_cb,
    pub send_handle: *mut uv_stream_t,
    pub handle: *mut uv_stream_t,
    pub queue: uv__queue,
    pub write_index: ::core::ffi::c_uint,
    pub bufs: *mut uv_buf_t,
    pub nbufs: ::core::ffi::c_uint,
    pub error: ::core::ffi::c_int,
    pub bufsml: [uv_buf_t; 4],
}
pub type uv_write_cb = Option<unsafe extern "C" fn(*mut uv_write_t, ::core::ffi::c_int) -> ()>;
pub type uv_write_t = uv_write_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_fs_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub fs_type: uv_fs_type,
    pub loop_0: *mut uv_loop_t,
    pub cb: uv_fs_cb,
    pub result: ssize_t,
    pub ptr: *mut ::core::ffi::c_void,
    pub path: *const ::core::ffi::c_char,
    pub statbuf: uv_stat_t,
    pub new_path: *const ::core::ffi::c_char,
    pub file: uv_file,
    pub flags: ::core::ffi::c_int,
    pub mode: mode_t,
    pub nbufs: ::core::ffi::c_uint,
    pub bufs: *mut uv_buf_t,
    pub off: off_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
    pub atime: ::core::ffi::c_double,
    pub mtime: ::core::ffi::c_double,
    pub work_req: uv__work,
    pub bufsml: [uv_buf_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stat_t {
    pub st_dev: uint64_t,
    pub st_mode: uint64_t,
    pub st_nlink: uint64_t,
    pub st_uid: uint64_t,
    pub st_gid: uint64_t,
    pub st_rdev: uint64_t,
    pub st_ino: uint64_t,
    pub st_size: uint64_t,
    pub st_blksize: uint64_t,
    pub st_blocks: uint64_t,
    pub st_flags: uint64_t,
    pub st_gen: uint64_t,
    pub st_atim: uv_timespec_t,
    pub st_mtim: uv_timespec_t,
    pub st_ctim: uv_timespec_t,
    pub st_birthtim: uv_timespec_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec_t {
    pub tv_sec: ::core::ffi::c_long,
    pub tv_nsec: ::core::ffi::c_long,
}
pub type uv_fs_cb = Option<unsafe extern "C" fn(*mut uv_fs_t) -> ()>;
pub type uv_fs_t = uv_fs_s;
pub type uv_fs_type = ::core::ffi::c_int;
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
pub type Loop = loop_0;
pub type MultiQueue = multiqueue;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbuffer {
    pub size: size_t,
    pub refcount: size_t,
    pub data: *mut ::core::ffi::c_char,
    pub cb: wbuffer_data_finalizer,
}
pub type wbuffer_data_finalizer = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type WBuffer = wbuffer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_10,
    pub uvstream: *mut uv_stream_t,
    pub fd: uv_file,
    pub fpos: int64_t,
    pub cb_data: *mut ::core::ffi::c_void,
    pub before_close_cb: stream_close_cb,
    pub close_cb: stream_close_cb,
    pub internal_close_cb: stream_close_cb,
    pub close_cb_data: *mut ::core::ffi::c_void,
    pub internal_data: *mut ::core::ffi::c_void,
    pub pending_reqs: size_t,
    pub events: *mut MultiQueue,
    pub write_cb: stream_write_cb,
    pub curmem: size_t,
    pub maxmem: size_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type Stream = stream;
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct WRequest {
    pub stream: *mut Stream,
    pub buffer: *mut WBuffer,
    pub uv_req: uv_write_t,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"int wstream_write(Stream *, WBuffer *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXMEM: ::core::ffi::c_int =
    1024 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int * 2000 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn wstream_init_fd(
    mut loop_0: *mut Loop,
    mut stream: *mut Stream,
    mut fd: ::core::ffi::c_int,
    mut maxmem: size_t,
) {
    stream_init(loop_0, stream, fd, ::core::ptr::null_mut::<uv_stream_t>());
    wstream_init(stream, maxmem);
}
#[no_mangle]
pub unsafe extern "C" fn wstream_init_stream(
    mut stream: *mut Stream,
    mut uvstream: *mut uv_stream_t,
    mut maxmem: size_t,
) {
    stream_init(
        ::core::ptr::null_mut::<Loop>(),
        stream,
        -1 as ::core::ffi::c_int,
        uvstream,
    );
    wstream_init(stream, maxmem);
}
#[no_mangle]
pub unsafe extern "C" fn wstream_init(mut stream: *mut Stream, mut maxmem: size_t) {
    (*stream).maxmem = if maxmem != 0 {
        maxmem
    } else {
        DEFAULT_MAXMEM as size_t
    };
}
#[no_mangle]
pub unsafe extern "C" fn wstream_set_write_cb(
    mut stream: *mut Stream,
    mut cb: stream_write_cb,
    mut data: *mut ::core::ffi::c_void,
) {
    (*stream).write_cb = cb;
    (*stream).cb_data = data;
}
#[no_mangle]
pub unsafe extern "C" fn wstream_write(
    mut stream: *mut Stream,
    mut buffer: *mut WBuffer,
) -> ::core::ffi::c_int {
    let mut data: *mut WRequest = ::core::ptr::null_mut::<WRequest>();
    '_c2rust_label: {
        if (*stream).maxmem != 0 {
        } else {
            __assert_fail(
                b"stream->maxmem\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/wstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                70 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    '_c2rust_label_0: {
        if !(*stream).closed {
        } else {
            __assert_fail(
                b"!stream->closed\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/wstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                72 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut uvbuf: uv_buf_t = uv_buf_t {
        base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    };
    uvbuf.base = (*buffer).data;
    uvbuf.len = (*buffer).size;
    if (*stream).uvstream.is_null() {
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
        err = uv_fs_write(
            (*stream).uv.idle.loop_0,
            &raw mut req,
            (*stream).fd,
            &raw mut uvbuf as *const uv_buf_t,
            1 as ::core::ffi::c_uint,
            (*stream).fpos,
            None,
        );
        uv_fs_req_cleanup(&raw mut req);
        wstream_release_wbuffer(buffer);
        '_c2rust_label_1: {
            if (*stream).write_cb.is_none() {
            } else {
                __assert_fail(
                    b"stream->write_cb == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/event/wstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    89 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        (*stream).fpos = ((*stream).fpos as ::core::ffi::c_long
            + (if req.result > 0 as ssize_t {
                req.result
            } else {
                0 as ssize_t
            }) as ::core::ffi::c_long) as int64_t;
        return if req.result > 0 as ssize_t {
            0 as ::core::ffi::c_int
        } else if err != 0 as ::core::ffi::c_int {
            err
        } else {
            UV_UNKNOWN as ::core::ffi::c_int
        };
    }
    if (*stream).curmem > (*stream).maxmem {
        err = UV_ENOMEM as ::core::ffi::c_int;
    } else {
        (*stream).curmem = (*stream).curmem.wrapping_add((*buffer).size);
        data = xmalloc(::core::mem::size_of::<WRequest>()) as *mut WRequest;
        (*data).stream = stream;
        (*data).buffer = buffer;
        (*data).uv_req.data = data as *mut ::core::ffi::c_void;
        err = uv_write(
            &raw mut (*data).uv_req,
            (*stream).uvstream,
            &raw mut uvbuf as *const uv_buf_t,
            1 as ::core::ffi::c_uint,
            Some(write_cb as unsafe extern "C" fn(*mut uv_write_t, ::core::ffi::c_int) -> ()),
        );
        if err != 0 as ::core::ffi::c_int {
            xfree(data as *mut ::core::ffi::c_void);
        } else {
            (*stream).pending_reqs = (*stream).pending_reqs.wrapping_add(1);
            '_c2rust_label_2: {
                if err == 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"err == 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/event/wstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        113 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            return 0 as ::core::ffi::c_int;
        }
    }
    wstream_release_wbuffer(buffer);
    '_c2rust_label_3: {
        if err != 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"err != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/wstream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                118 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    return err;
}
#[no_mangle]
pub unsafe extern "C" fn wstream_new_buffer(
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
    mut refcount: size_t,
    mut cb: wbuffer_data_finalizer,
) -> *mut WBuffer {
    let mut rv: *mut WBuffer = xmalloc(::core::mem::size_of::<WBuffer>()) as *mut WBuffer;
    (*rv).size = size;
    (*rv).refcount = refcount;
    (*rv).cb = cb;
    (*rv).data = data;
    return rv;
}
unsafe extern "C" fn write_cb(mut req: *mut uv_write_t, mut status: ::core::ffi::c_int) {
    let mut data: *mut WRequest = (*req).data as *mut WRequest;
    (*(*data).stream).curmem = (*(*data).stream)
        .curmem
        .wrapping_sub((*(*data).buffer).size);
    wstream_release_wbuffer((*data).buffer);
    if (*(*data).stream).write_cb.is_some() {
        (*(*data).stream)
            .write_cb
            .expect("non-null function pointer")(
            (*data).stream, (*(*data).stream).cb_data, status
        );
    }
    (*(*data).stream).pending_reqs = (*(*data).stream).pending_reqs.wrapping_sub(1);
    if (*(*data).stream).closed as ::core::ffi::c_int != 0
        && (*(*data).stream).pending_reqs == 0 as size_t
    {
        stream_close_handle((*data).stream);
    }
    xfree(data as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wstream_release_wbuffer(mut buffer: *mut WBuffer) {
    (*buffer).refcount = (*buffer).refcount.wrapping_sub(1);
    if (*buffer).refcount == 0 {
        if (*buffer).cb.is_some() {
            (*buffer).cb.expect("non-null function pointer")(
                (*buffer).data as *mut ::core::ffi::c_void,
            );
        }
        xfree(buffer as *mut ::core::ffi::c_void);
    }
}
