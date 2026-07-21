use crate::src::nvim::global_cell::SharedCell;
pub use crate::src::nvim::types::{
    Event, Loop, LuaRef, MultiQueue, Proc, ProcType, RStream, ScopeType, SocketWatcher, Stream,
    VarLockStatus, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s,
    __pthread_rwlock_arch_t, __socklen_t, addrinfo, argv_callback, dict_T, dictvar_S, hash_T,
    hashitem_T, hashtab_T, int64_t, internal_proc_cb, intmax_t, loop_0,
    loop_0_children as C2Rust_Unnamed_13, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, sa_family_t, size_t, sockaddr, socket_cb,
    socket_close_cb, socket_watcher, socket_watcher_uv as C2Rust_Unnamed_15,
    socket_watcher_uv_pipe as C2Rust_Unnamed_16, socket_watcher_uv_tcp as C2Rust_Unnamed_17,
    socklen_t, ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_14,
    stream_write_cb, uint16_t, uint32_t, uint64_t, uint8_t, uintptr_t, uv__io_cb, uv__io_s,
    uv__io_t, uv__queue, uv__work, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_4, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_1, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_12, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_5, uv_loop_s_timer_heap as C2Rust_Unnamed_3, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_9, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed_0, uv_signal_s_u as C2Rust_Unnamed_2, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_7, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_8, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_10, uv_timer_s_u as C2Rust_Unnamed_11, uv_timer_t, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_close(handle: *mut uv_handle_t, close_cb_0: uv_close_cb);
    fn uv_listen(
        stream: *mut uv_stream_t,
        backlog: ::core::ffi::c_int,
        cb: uv_connection_cb,
    ) -> ::core::ffi::c_int;
    fn uv_accept(server: *mut uv_stream_t, client: *mut uv_stream_t) -> ::core::ffi::c_int;
    fn uv_tcp_init(_: *mut uv_loop_t, handle: *mut uv_tcp_t) -> ::core::ffi::c_int;
    fn uv_tcp_nodelay(handle: *mut uv_tcp_t, enable: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn uv_tcp_bind(
        handle: *mut uv_tcp_t,
        addr: *const sockaddr,
        flags: ::core::ffi::c_uint,
    ) -> ::core::ffi::c_int;
    fn uv_tcp_getsockname(
        handle: *const uv_tcp_t,
        name: *mut sockaddr,
        namelen: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_tcp_connect(
        req: *mut uv_connect_t,
        handle: *mut uv_tcp_t,
        addr: *const sockaddr,
        cb: uv_connect_cb,
    ) -> ::core::ffi::c_int;
    fn uv_pipe_init(
        _: *mut uv_loop_t,
        handle: *mut uv_pipe_t,
        ipc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_pipe_bind(handle: *mut uv_pipe_t, name: *const ::core::ffi::c_char)
        -> ::core::ffi::c_int;
    fn uv_pipe_connect(
        req: *mut uv_connect_t,
        handle: *mut uv_pipe_t,
        name: *const ::core::ffi::c_char,
        cb: uv_connect_cb,
    );
    fn uv_getaddrinfo(
        loop_0: *mut uv_loop_t,
        req: *mut uv_getaddrinfo_t,
        getaddrinfo_cb: uv_getaddrinfo_cb,
        node: *const ::core::ffi::c_char,
        service: *const ::core::ffi::c_char,
        hints: *const addrinfo,
    ) -> ::core::ffi::c_int;
    fn uv_freeaddrinfo(ai: *mut addrinfo);
    fn ntohs(__netshort: uint16_t) -> uint16_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
    fn os_hrtime() -> uint64_t;
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn stream_init(
        loop_0: *mut Loop,
        stream: *mut Stream,
        fd: ::core::ffi::c_int,
        uvstream: *mut uv_stream_t,
    );
    fn stream_may_close(stream: *mut Stream);
    fn gettext(__msgid: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn try_getdigits(pp: *mut *mut ::core::ffi::c_char, nr: *mut intmax_t) -> bool;
    static main_loop: SharedCell<Loop>;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn os_path_exists(path: *const ::core::ffi::c_char) -> bool;
    fn os_remove(path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn path_tail(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
}
pub type __socket_type = ::core::ffi::c_uint;
pub const SOCK_NONBLOCK: __socket_type = 2048;
pub const SOCK_CLOEXEC: __socket_type = 524288;
pub const SOCK_PACKET: __socket_type = 10;
pub const SOCK_DCCP: __socket_type = 6;
pub const SOCK_SEQPACKET: __socket_type = 5;
pub const SOCK_RDM: __socket_type = 4;
pub const SOCK_RAW: __socket_type = 3;
pub const SOCK_DGRAM: __socket_type = 2;
pub const SOCK_STREAM: __socket_type = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_padding: [::core::ffi::c_char; 118],
    pub __ss_align: ::core::ffi::c_ulong,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: uint32_t,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in6_addr {
    pub __in6_u: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub __u6_addr8: [uint8_t; 16],
    pub __u6_addr16: [uint16_t; 8],
    pub __u6_addr32: [uint32_t; 4],
}
pub type in_port_t = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [::core::ffi::c_uchar; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type in_addr_t = uint32_t;
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
pub type C2Rust_Unnamed_6 = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed_6 = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed_6 = -8;
pub const UV_EUNATCH: C2Rust_Unnamed_6 = -49;
pub const UV_ENODATA: C2Rust_Unnamed_6 = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed_6 = -94;
pub const UV_EILSEQ: C2Rust_Unnamed_6 = -84;
pub const UV_EFTYPE: C2Rust_Unnamed_6 = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed_6 = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed_6 = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed_6 = -112;
pub const UV_EMLINK: C2Rust_Unnamed_6 = -31;
pub const UV_ENXIO: C2Rust_Unnamed_6 = -6;
pub const UV_EOF: C2Rust_Unnamed_6 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_6 = -4094;
pub const UV_EXDEV: C2Rust_Unnamed_6 = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed_6 = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed_6 = -110;
pub const UV_ESRCH: C2Rust_Unnamed_6 = -3;
pub const UV_ESPIPE: C2Rust_Unnamed_6 = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed_6 = -108;
pub const UV_EROFS: C2Rust_Unnamed_6 = -30;
pub const UV_ERANGE: C2Rust_Unnamed_6 = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed_6 = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed_6 = -93;
pub const UV_EPROTO: C2Rust_Unnamed_6 = -71;
pub const UV_EPIPE: C2Rust_Unnamed_6 = -32;
pub const UV_EPERM: C2Rust_Unnamed_6 = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed_6 = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed_6 = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed_6 = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed_6 = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed_6 = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed_6 = -107;
pub const UV_ENOSYS: C2Rust_Unnamed_6 = -38;
pub const UV_ENOSPC: C2Rust_Unnamed_6 = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed_6 = -92;
pub const UV_ENONET: C2Rust_Unnamed_6 = -64;
pub const UV_ENOMEM: C2Rust_Unnamed_6 = -12;
pub const UV_ENOENT: C2Rust_Unnamed_6 = -2;
pub const UV_ENODEV: C2Rust_Unnamed_6 = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed_6 = -105;
pub const UV_ENFILE: C2Rust_Unnamed_6 = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed_6 = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed_6 = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed_6 = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed_6 = -90;
pub const UV_EMFILE: C2Rust_Unnamed_6 = -24;
pub const UV_ELOOP: C2Rust_Unnamed_6 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_6 = -21;
pub const UV_EISCONN: C2Rust_Unnamed_6 = -106;
pub const UV_EIO: C2Rust_Unnamed_6 = -5;
pub const UV_EINVAL: C2Rust_Unnamed_6 = -22;
pub const UV_EINTR: C2Rust_Unnamed_6 = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed_6 = -113;
pub const UV_EFBIG: C2Rust_Unnamed_6 = -27;
pub const UV_EFAULT: C2Rust_Unnamed_6 = -14;
pub const UV_EEXIST: C2Rust_Unnamed_6 = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed_6 = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed_6 = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed_6 = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed_6 = -103;
pub const UV_ECHARSET: C2Rust_Unnamed_6 = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed_6 = -125;
pub const UV_EBUSY: C2Rust_Unnamed_6 = -16;
pub const UV_EBADF: C2Rust_Unnamed_6 = -9;
pub const UV_EALREADY: C2Rust_Unnamed_6 = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed_6 = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed_6 = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed_6 = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed_6 = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed_6 = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed_6 = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed_6 = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed_6 = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed_6 = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed_6 = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed_6 = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed_6 = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed_6 = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed_6 = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed_6 = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed_6 = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed_6 = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed_6 = -98;
pub const UV_EACCES: C2Rust_Unnamed_6 = -13;
pub const UV_E2BIG: C2Rust_Unnamed_6 = -7;
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
pub struct uv_getaddrinfo_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub loop_0: *mut uv_loop_t,
    pub work_req: uv__work,
    pub cb: uv_getaddrinfo_cb,
    pub hints: *mut addrinfo,
    pub hostname: *mut ::core::ffi::c_char,
    pub service: *mut ::core::ffi::c_char,
    pub addrinfo: *mut addrinfo,
    pub retcode: ::core::ffi::c_int,
}
pub type uv_getaddrinfo_cb =
    Option<unsafe extern "C" fn(*mut uv_getaddrinfo_t, ::core::ffi::c_int, *mut addrinfo) -> ()>;
pub type uv_getaddrinfo_t = uv_getaddrinfo_s;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
        *b"int socket_watcher_start(SocketWatcher *, int, socket_cb)\0",
    )
};
pub const UINT16_MAX: ::core::ffi::c_int = 65535 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const PF_UNSPEC: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PF_INET: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AF_UNSPEC: ::core::ffi::c_int = PF_UNSPEC;
pub const AF_INET: ::core::ffi::c_int = PF_INET;
pub const AI_NUMERICSERV: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn socket_address_tcp_host_end(
    mut address: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if address.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*address.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_uint
        >= 'A' as ::core::ffi::c_uint
        && *address.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_uint
            <= 'Z' as ::core::ffi::c_uint
        || *address.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_uint
            >= 'a' as ::core::ffi::c_uint
            && *address.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_uint
                <= 'z' as ::core::ffi::c_uint)
        && *address.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
        && (*address.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            || *address.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '/' as ::core::ffi::c_int)
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut colon: *mut ::core::ffi::c_char = strrchr(address, ':' as ::core::ffi::c_int);
    return if !colon.is_null() && colon != address {
        colon
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn socket_watcher_init(
    mut loop_0: *mut Loop,
    mut watcher: *mut SocketWatcher,
    mut endpoint: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    xstrlcpy(
        &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        endpoint,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
    );
    let mut addr: *mut ::core::ffi::c_char = &raw mut (*watcher).addr as *mut ::core::ffi::c_char;
    let mut host_end: *mut ::core::ffi::c_char = socket_address_tcp_host_end(addr);
    if !host_end.is_null() {
        *host_end = NUL as ::core::ffi::c_char;
        let mut port: *mut ::core::ffi::c_char = host_end.offset(1 as ::core::ffi::c_int as isize);
        let mut iport: intmax_t = 0;
        let mut c2rust_lvalue: *mut ::core::ffi::c_char = port;
        let mut ok: ::core::ffi::c_int =
            try_getdigits(&raw mut c2rust_lvalue, &raw mut iport) as ::core::ffi::c_int;
        if ok == 0 || iport < 0 as intmax_t || iport > UINT16_MAX as intmax_t {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"socket_watcher_init\0".as_ptr() as *const ::core::ffi::c_char,
                62 as ::core::ffi::c_int,
                true_0 != 0,
                b"Invalid port: %s\0".as_ptr() as *const ::core::ffi::c_char,
                port,
            );
            return UV_EINVAL as ::core::ffi::c_int;
        }
        if *port as ::core::ffi::c_int == NUL {
            port = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut request: uv_getaddrinfo_t = uv_getaddrinfo_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            type_0: UV_UNKNOWN_REQ,
            reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            work_req: uv__work {
                work: None,
                done: None,
                loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                wq: uv__queue {
                    next: ::core::ptr::null_mut::<uv__queue>(),
                    prev: ::core::ptr::null_mut::<uv__queue>(),
                },
            },
            cb: None,
            hints: ::core::ptr::null_mut::<addrinfo>(),
            hostname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            service: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            addrinfo: ::core::ptr::null_mut::<addrinfo>(),
            retcode: 0,
        };
        let mut c2rust_lvalue_0: addrinfo = addrinfo {
            ai_flags: 0,
            ai_family: AF_UNSPEC,
            ai_socktype: SOCK_STREAM as ::core::ffi::c_int,
            ai_protocol: 0,
            ai_addrlen: 0,
            ai_addr: ::core::ptr::null_mut::<sockaddr>(),
            ai_canonname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ai_next: ::core::ptr::null_mut::<addrinfo>(),
        };
        let mut retval: ::core::ffi::c_int = uv_getaddrinfo(
            &raw mut (*loop_0).uv,
            &raw mut request,
            None,
            addr,
            port,
            &raw mut c2rust_lvalue_0,
        );
        if retval != 0 as ::core::ffi::c_int {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"socket_watcher_init\0".as_ptr() as *const ::core::ffi::c_char,
                78 as ::core::ffi::c_int,
                true_0 != 0,
                b"Host lookup failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                endpoint,
            );
            return retval;
        }
        (*watcher).uv.tcp.addrinfo = request.addrinfo;
        uv_tcp_init(&raw mut (*loop_0).uv, &raw mut (*watcher).uv.tcp.handle);
        uv_tcp_nodelay(&raw mut (*watcher).uv.tcp.handle, true_0);
        (*watcher).stream = &raw mut (*watcher).uv.tcp.handle as *mut uv_stream_t;
    } else {
        uv_pipe_init(
            &raw mut (*loop_0).uv,
            &raw mut (*watcher).uv.pipe.handle,
            0 as ::core::ffi::c_int,
        );
        (*watcher).stream = &raw mut (*watcher).uv.pipe.handle as *mut uv_stream_t;
    }
    (*(*watcher).stream).data = watcher as *mut ::core::ffi::c_void;
    (*watcher).cb = None;
    (*watcher).close_cb = None;
    (*watcher).events = ::core::ptr::null_mut::<MultiQueue>();
    (*watcher).data = NULL;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn connect_close_cb(
    mut _stream: *mut Stream,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut closed: *mut bool = data as *mut bool;
    *closed = true_0 != 0;
}
unsafe extern "C" fn socket_alive(
    mut loop_0: *mut Loop,
    mut addr: *const ::core::ffi::c_char,
) -> bool {
    let mut stream: RStream = rstream {
        s: stream {
            closed: false,
            uv: C2Rust_Unnamed_14 {
                pipe: uv_pipe_t {
                    data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
                    type_0: UV_UNKNOWN_HANDLE,
                    close_cb: None,
                    handle_queue: uv__queue {
                        next: ::core::ptr::null_mut::<uv__queue>(),
                        prev: ::core::ptr::null_mut::<uv__queue>(),
                    },
                    u: C2Rust_Unnamed_9 { fd: 0 },
                    next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
                    flags: 0,
                    write_queue_size: 0,
                    alloc_cb: None,
                    read_cb: None,
                    connect_req: ::core::ptr::null_mut::<uv_connect_t>(),
                    shutdown_req: ::core::ptr::null_mut::<uv_shutdown_t>(),
                    io_watcher: uv__io_t {
                        cb: None,
                        pending_queue: uv__queue {
                            next: ::core::ptr::null_mut::<uv__queue>(),
                            prev: ::core::ptr::null_mut::<uv__queue>(),
                        },
                        watcher_queue: uv__queue {
                            next: ::core::ptr::null_mut::<uv__queue>(),
                            prev: ::core::ptr::null_mut::<uv__queue>(),
                        },
                        pevents: 0,
                        events: 0,
                        fd: 0,
                    },
                    write_queue: uv__queue {
                        next: ::core::ptr::null_mut::<uv__queue>(),
                        prev: ::core::ptr::null_mut::<uv__queue>(),
                    },
                    write_completed_queue: uv__queue {
                        next: ::core::ptr::null_mut::<uv__queue>(),
                        prev: ::core::ptr::null_mut::<uv__queue>(),
                    },
                    connection_cb: None,
                    delayed_error: 0,
                    accepted_fd: 0,
                    queued_fds: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ipc: 0,
                    pipe_fname: ::core::ptr::null::<::core::ffi::c_char>(),
                },
            },
            uvstream: ::core::ptr::null_mut::<uv_stream_t>(),
            fd: 0,
            fpos: 0,
            cb_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            before_close_cb: None,
            close_cb: None,
            internal_close_cb: None,
            close_cb_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            internal_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            pending_reqs: 0,
            events: ::core::ptr::null_mut::<MultiQueue>(),
            write_cb: None,
            curmem: 0,
            maxmem: 0,
        },
        did_eof: false,
        want_read: false,
        pending_read: false,
        paused_full: false,
        buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        uvbuf: uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        },
        read_cb: None,
        num_bytes: 0,
    };
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut connected: bool = socket_connect(
        loop_0,
        &raw mut stream,
        false_0 != 0,
        addr,
        500 as ::core::ffi::c_int,
        &raw mut error,
    );
    if !connected {
        return false_0 != 0;
    }
    let mut closed: bool = false_0 != 0;
    stream.s.internal_close_cb =
        Some(connect_close_cb as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ())
            as stream_close_cb;
    stream.s.internal_data = &raw mut closed as *mut ::core::ffi::c_void;
    stream_may_close(&raw mut stream.s);
    let mut remaining: int64_t = -1 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !closed {
        if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
            && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
        {
            multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
        } else {
            loop_poll_events(main_loop.ptr(), remaining);
        }
        if remaining == 0 as int64_t {
            break;
        }
        if remaining <= 0 as int64_t {
            continue;
        }
        let mut now: uint64_t = os_hrtime();
        remaining -= now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
        before = now;
        if remaining <= 0 as int64_t {
            break;
        }
    }
    return true_0 != 0;
}
unsafe extern "C" fn early_server_close_cb(mut handle: *mut uv_handle_t) {
    let mut closed: *mut bool = (*handle).data as *mut bool;
    *closed = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn socket_watcher_start(
    mut watcher: *mut SocketWatcher,
    mut backlog: ::core::ffi::c_int,
    mut cb: socket_cb,
) -> ::core::ffi::c_int {
    (*watcher).cb = cb;
    let mut result: ::core::ffi::c_int = UV_EINVAL as ::core::ffi::c_int;
    if (*(*watcher).stream).type_0 as ::core::ffi::c_uint
        == UV_TCP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut ai: *mut addrinfo = (*watcher).uv.tcp.addrinfo;
        while !ai.is_null() {
            result = uv_tcp_bind(
                &raw mut (*watcher).uv.tcp.handle,
                (*ai).ai_addr,
                0 as ::core::ffi::c_uint,
            );
            if result == 0 as ::core::ffi::c_int {
                result = uv_listen(
                    (*watcher).stream,
                    backlog,
                    Some(
                        connection_cb
                            as unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> (),
                    ),
                );
                if result == 0 as ::core::ffi::c_int {
                    let mut sas: sockaddr_storage = sockaddr_storage {
                        ss_family: 0,
                        __ss_padding: [0; 118],
                        __ss_align: 0,
                    };
                    let mut c2rust_lvalue: ::core::ffi::c_int =
                        ::core::mem::size_of::<sockaddr_storage>() as ::core::ffi::c_int;
                    uv_tcp_getsockname(
                        &raw mut (*watcher).uv.tcp.handle,
                        &raw mut sas as *mut sockaddr,
                        &raw mut c2rust_lvalue,
                    );
                    let mut port: uint16_t = (if sas.ss_family as ::core::ffi::c_int == AF_INET {
                        (*(&raw mut sas as *mut sockaddr_in)).sin_port as ::core::ffi::c_int
                    } else {
                        (*(&raw mut sas as *mut sockaddr_in6)).sin6_port as ::core::ffi::c_int
                    }) as uint16_t;
                    let mut len: size_t =
                        strlen(&raw mut (*watcher).addr as *mut ::core::ffi::c_char);
                    snprintf(
                        (&raw mut (*watcher).addr as *mut ::core::ffi::c_char).offset(len as isize),
                        ::core::mem::size_of::<[::core::ffi::c_char; 256]>().wrapping_sub(len),
                        b":%u\0".as_ptr() as *const ::core::ffi::c_char,
                        ntohs(port) as ::core::ffi::c_int,
                    );
                    break;
                }
            }
            ai = (*ai).ai_next;
        }
        uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
    } else {
        result = uv_pipe_bind(
            &raw mut (*watcher).uv.pipe.handle,
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        );
        if result == UV_EACCES as ::core::ffi::c_int
            || result == UV_EADDRINUSE as ::core::ffi::c_int
        {
            let mut loop_0: *mut Loop = (*(*(*watcher).stream).loop_0).data as *mut Loop;
            if !socket_alive(loop_0, &raw mut (*watcher).addr as *mut ::core::ffi::c_char) {
                logmsg(
                    LOGLVL_INF,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"socket_watcher_start\0".as_ptr() as *const ::core::ffi::c_char,
                    180 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"Removing stale socket: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
                );
                let mut rm_result: ::core::ffi::c_int =
                    os_remove(&raw mut (*watcher).addr as *mut ::core::ffi::c_char);
                if rm_result != 0 as ::core::ffi::c_int {
                    logmsg(
                        LOGLVL_WRN,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"socket_watcher_start\0".as_ptr() as *const ::core::ffi::c_char,
                        185 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"Failed to remove stale socket %s: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
                        uv_strerror(rm_result),
                    );
                } else {
                    let mut uv_loop: *mut uv_loop_t = (*watcher).uv.pipe.handle.loop_0;
                    let mut closed: bool = false_0 != 0;
                    (*watcher).uv.pipe.handle.data = &raw mut closed as *mut ::core::ffi::c_void;
                    uv_close(
                        &raw mut (*watcher).uv.pipe.handle as *mut uv_handle_t,
                        Some(early_server_close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
                    );
                    let mut remaining: int64_t = -1 as int64_t;
                    let mut before: uint64_t = if remaining > 0 as int64_t {
                        os_hrtime()
                    } else {
                        0 as uint64_t
                    };
                    while !closed {
                        if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
                            && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
                        {
                            multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
                        } else {
                            loop_poll_events(main_loop.ptr(), remaining);
                        }
                        if remaining == 0 as int64_t {
                            break;
                        }
                        if remaining <= 0 as int64_t {
                            continue;
                        }
                        let mut now: uint64_t = os_hrtime();
                        remaining -=
                            now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
                        before = now;
                        if remaining <= 0 as int64_t {
                            break;
                        }
                    }
                    uv_pipe_init(
                        uv_loop,
                        &raw mut (*watcher).uv.pipe.handle,
                        0 as ::core::ffi::c_int,
                    );
                    (*watcher).stream = &raw mut (*watcher).uv.pipe.handle as *mut uv_stream_t;
                    (*(*watcher).stream).data = watcher as *mut ::core::ffi::c_void;
                    result = uv_pipe_bind(
                        &raw mut (*watcher).uv.pipe.handle,
                        &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
                    );
                }
            } else {
                logmsg(
                    LOGLVL_ERR,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"socket_watcher_start\0".as_ptr() as *const ::core::ffi::c_char,
                    203 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"Socket already in use by another Nvim instance: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
                );
            }
        }
        if result == 0 as ::core::ffi::c_int {
            result = uv_listen(
                (*watcher).stream,
                backlog,
                Some(
                    connection_cb
                        as unsafe extern "C" fn(*mut uv_stream_t, ::core::ffi::c_int) -> (),
                ),
            );
        }
    }
    '_c2rust_label: {
        if result <= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result <= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/socket.rs\0".as_ptr() as *const ::core::ffi::c_char,
                212 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    if result < 0 as ::core::ffi::c_int {
        if result == UV_EACCES as ::core::ffi::c_int {
            *path_tail(&raw mut (*watcher).addr as *mut ::core::ffi::c_char) =
                NUL as ::core::ffi::c_char;
            if !os_path_exists(&raw mut (*watcher).addr as *mut ::core::ffi::c_char) {
                result = UV_ENOENT as ::core::ffi::c_int;
            }
        }
        return result;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn socket_watcher_accept(
    mut watcher: *mut SocketWatcher,
    mut stream: *mut RStream,
) -> ::core::ffi::c_int {
    let mut client: *mut uv_stream_t = ::core::ptr::null_mut::<uv_stream_t>();
    if (*(*watcher).stream).type_0 as ::core::ffi::c_uint
        == UV_TCP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        client = &raw mut (*stream).s.uv.tcp as *mut uv_stream_t;
        uv_tcp_init((*watcher).uv.tcp.handle.loop_0, client as *mut uv_tcp_t);
        uv_tcp_nodelay(client as *mut uv_tcp_t, true_0);
    } else {
        client = &raw mut (*stream).s.uv.pipe as *mut uv_stream_t;
        uv_pipe_init(
            (*watcher).uv.pipe.handle.loop_0,
            client as *mut uv_pipe_t,
            0 as ::core::ffi::c_int,
        );
    }
    let mut result: ::core::ffi::c_int = uv_accept((*watcher).stream, client);
    if result != 0 {
        uv_close(client as *mut uv_handle_t, None);
        return result;
    }
    stream_init(
        ::core::ptr::null_mut::<Loop>(),
        &raw mut (*stream).s,
        -1 as ::core::ffi::c_int,
        client,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn socket_watcher_close(
    mut watcher: *mut SocketWatcher,
    mut cb: socket_close_cb,
) {
    (*watcher).close_cb = cb;
    uv_close(
        (*watcher).stream as *mut uv_handle_t,
        Some(close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
    );
}
unsafe extern "C" fn connection_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut watcher: *mut SocketWatcher =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut SocketWatcher;
    let mut status: ::core::ffi::c_int = (*argv.offset(1 as ::core::ffi::c_int as isize))
        .expose_addr() as uintptr_t as ::core::ffi::c_int;
    (*watcher).cb.expect("non-null function pointer")(watcher, status, (*watcher).data);
}
unsafe extern "C" fn connection_cb(mut handle: *mut uv_stream_t, mut status: ::core::ffi::c_int) {
    let mut watcher: *mut SocketWatcher = (*handle).data as *mut SocketWatcher;
    if !(*watcher).events.is_null() {
        multiqueue_put_event(
            (*watcher).events,
            Event {
                handler: Some(
                    connection_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    watcher as *mut ::core::ffi::c_void,
                    ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                        status as uintptr_t as usize,
                    ),
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
        let mut argv: [*mut ::core::ffi::c_void; 2] = [
            watcher as *mut ::core::ffi::c_void,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(status as uintptr_t as usize),
        ];
        connection_event(&raw mut argv as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn close_cb(mut handle: *mut uv_handle_t) {
    let mut watcher: *mut SocketWatcher = (*handle).data as *mut SocketWatcher;
    if (*watcher).close_cb.is_some() {
        (*watcher).close_cb.expect("non-null function pointer")(watcher, (*watcher).data);
    }
}
unsafe extern "C" fn connect_cb(mut req: *mut uv_connect_t, mut status: ::core::ffi::c_int) {
    let mut ret_status: *mut ::core::ffi::c_int = (*req).data as *mut ::core::ffi::c_int;
    *ret_status = status;
    let mut handle: *mut uv_handle_t = (*req).handle as *mut uv_handle_t;
    if status != 0 as ::core::ffi::c_int {
        stream_may_close((*handle).data as *mut Stream);
    }
}
#[no_mangle]
pub unsafe extern "C" fn socket_connect(
    mut loop_0: *mut Loop,
    mut stream: *mut RStream,
    mut is_tcp: bool,
    mut address: *const ::core::ffi::c_char,
    mut timeout: ::core::ffi::c_int,
    mut error: *mut *const ::core::ffi::c_char,
) -> bool {
    let mut host_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut hints: addrinfo = addrinfo {
        ai_flags: 0,
        ai_family: 0,
        ai_socktype: 0,
        ai_protocol: 0,
        ai_addrlen: 0,
        ai_addr: ::core::ptr::null_mut::<sockaddr>(),
        ai_canonname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ai_next: ::core::ptr::null_mut::<addrinfo>(),
    };
    let mut retval: ::core::ffi::c_int = 0;
    let mut c2rust_current_block: u64;
    let mut success: bool = false_0 != 0;
    let mut closed: bool = false;
    let mut status: ::core::ffi::c_int = 0;
    let mut req: uv_connect_t = uv_connect_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        cb: None,
        handle: ::core::ptr::null_mut::<uv_stream_t>(),
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
    };
    req.data = &raw mut status as *mut ::core::ffi::c_void;
    let mut uv_stream: *mut uv_stream_t = ::core::ptr::null_mut::<uv_stream_t>();
    let mut tcp: *mut uv_tcp_t = &raw mut (*stream).s.uv.tcp;
    let mut addr_req: uv_getaddrinfo_t = uv_getaddrinfo_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        work_req: uv__work {
            work: None,
            done: None,
            loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
            wq: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
        },
        cb: None,
        hints: ::core::ptr::null_mut::<addrinfo>(),
        hostname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        service: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        addrinfo: ::core::ptr::null_mut::<addrinfo>(),
        retcode: 0,
    };
    addr_req.addrinfo = ::core::ptr::null_mut::<addrinfo>();
    let mut addrinfo: *const addrinfo = ::core::ptr::null::<addrinfo>();
    let mut addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if is_tcp {
        addr = xstrdup(address);
        host_end = strrchr(addr, ':' as ::core::ffi::c_int);
        if host_end.is_null() {
            *error =
                gettext(b"tcp address must be host:port\0".as_ptr() as *const ::core::ffi::c_char);
            c2rust_current_block = 8695800224913848308;
        } else {
            *host_end = NUL as ::core::ffi::c_char;
            hints = addrinfo {
                ai_flags: AI_NUMERICSERV,
                ai_family: AF_UNSPEC,
                ai_socktype: SOCK_STREAM as ::core::ffi::c_int,
                ai_protocol: 0,
                ai_addrlen: 0,
                ai_addr: ::core::ptr::null_mut::<sockaddr>(),
                ai_canonname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ai_next: ::core::ptr::null_mut::<addrinfo>(),
            };
            retval = uv_getaddrinfo(
                &raw mut (*loop_0).uv,
                &raw mut addr_req,
                None,
                addr,
                host_end.offset(1 as ::core::ffi::c_int as isize),
                &raw const hints,
            );
            if retval != 0 as ::core::ffi::c_int {
                *error = gettext(
                    b"failed to lookup host or port\0".as_ptr() as *const ::core::ffi::c_char
                );
                c2rust_current_block = 8695800224913848308;
            } else {
                addrinfo = addr_req.addrinfo;
                c2rust_current_block = 2648362272183767480;
            }
        }
    } else {
        let mut pipe: *mut uv_pipe_t = &raw mut (*stream).s.uv.pipe;
        uv_pipe_init(&raw mut (*loop_0).uv, pipe, 0 as ::core::ffi::c_int);
        uv_pipe_connect(
            &raw mut req,
            pipe,
            address,
            Some(connect_cb as unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> ()),
        );
        uv_stream = pipe as *mut uv_stream_t;
        c2rust_current_block = 2370887241019905314;
    }
    loop {
        match c2rust_current_block {
            2648362272183767480 => {
                uv_tcp_init(&raw mut (*loop_0).uv, tcp);
                uv_tcp_nodelay(tcp, true_0);
                uv_tcp_connect(
                    &raw mut req,
                    tcp,
                    (*addrinfo).ai_addr,
                    Some(
                        connect_cb
                            as unsafe extern "C" fn(*mut uv_connect_t, ::core::ffi::c_int) -> (),
                    ),
                );
                uv_stream = tcp as *mut uv_stream_t;
                c2rust_current_block = 2370887241019905314;
            }
            8695800224913848308 => {
                (*stream).s.internal_close_cb = None;
                break;
            }
            _ => {
                stream_init(
                    ::core::ptr::null_mut::<Loop>(),
                    &raw mut (*stream).s,
                    -1 as ::core::ffi::c_int,
                    uv_stream,
                );
                (*stream).s.internal_close_cb = Some(
                    connect_close_cb
                        as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> (),
                ) as stream_close_cb;
                (*stream).s.internal_data = &raw mut closed as *mut ::core::ffi::c_void;
                closed = false_0 != 0;
                status = 1 as ::core::ffi::c_int;
                let mut remaining: int64_t = timeout as int64_t;
                let mut before: uint64_t = if remaining > 0 as int64_t {
                    os_hrtime()
                } else {
                    0 as uint64_t
                };
                while status == 1 as ::core::ffi::c_int {
                    if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
                        && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
                    {
                        multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
                    } else {
                        loop_poll_events(main_loop.ptr(), remaining);
                    }
                    if remaining == 0 as int64_t {
                        break;
                    }
                    if remaining <= 0 as int64_t {
                        continue;
                    }
                    let mut now: uint64_t = os_hrtime();
                    remaining -=
                        now.wrapping_sub(before).wrapping_div(1000000 as uint64_t) as int64_t;
                    before = now;
                    if remaining <= 0 as int64_t {
                        break;
                    }
                }
                if status == 0 as ::core::ffi::c_int {
                    success = true_0 != 0;
                    c2rust_current_block = 8695800224913848308;
                } else {
                    stream_may_close(&raw mut (*stream).s);
                    let mut remaining_0: int64_t = -1 as int64_t;
                    let mut before_0: uint64_t = if remaining_0 > 0 as int64_t {
                        os_hrtime()
                    } else {
                        0 as uint64_t
                    };
                    while !closed {
                        if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
                            && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
                        {
                            multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
                        } else {
                            loop_poll_events(main_loop.ptr(), remaining_0);
                        }
                        if remaining_0 == 0 as int64_t {
                            break;
                        }
                        if remaining_0 <= 0 as int64_t {
                            continue;
                        }
                        let mut now_0: uint64_t = os_hrtime();
                        remaining_0 -= now_0
                            .wrapping_sub(before_0)
                            .wrapping_div(1000000 as uint64_t)
                            as int64_t;
                        before_0 = now_0;
                        if remaining_0 <= 0 as int64_t {
                            break;
                        }
                    }
                    if is_tcp as ::core::ffi::c_int != 0 && !(*addrinfo).ai_next.is_null() {
                        addrinfo = (*addrinfo).ai_next;
                        c2rust_current_block = 2648362272183767480;
                    } else {
                        *error =
                            gettext(b"connection refused\0".as_ptr() as *const ::core::ffi::c_char);
                        c2rust_current_block = 8695800224913848308;
                    }
                }
            }
        }
    }
    (*stream).s.internal_data = NULL;
    xfree(addr as *mut ::core::ffi::c_void);
    uv_freeaddrinfo(addr_req.addrinfo);
    return success;
}
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
