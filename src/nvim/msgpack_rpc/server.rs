extern "C" {
    pub type multiqueue;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_freeaddrinfo(ai: *mut addrinfo);
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn xstrlcpy(
        dst: *mut ::core::ffi::c_char,
        src: *const ::core::ffi::c_char,
        dsize: size_t,
    ) -> size_t;
    fn xstrdup(str: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn channel_from_connection(watcher: *mut SocketWatcher);
    fn socket_watcher_init(
        loop_0: *mut Loop,
        watcher: *mut SocketWatcher,
        endpoint: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn socket_watcher_start(
        watcher: *mut SocketWatcher,
        backlog: ::core::ffi::c_int,
        cb: socket_cb,
    ) -> ::core::ffi::c_int;
    fn socket_watcher_close(watcher: *mut SocketWatcher, cb: socket_close_cb);
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_grow(gap: *mut garray_T, n: ::core::ffi::c_int);
    fn get_vim_var_str(idx: VimVarIndex) -> *mut ::core::ffi::c_char;
    fn set_vim_var_string(idx: VimVarIndex, val: *const ::core::ffi::c_char, len: ptrdiff_t);
    static mut IObuff: [::core::ffi::c_char; 1025];
    static mut NameBuff: [::core::ffi::c_char; 4096];
    static mut main_loop: Loop;
    fn os_getenv(name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn os_env_exists(name: *const ::core::ffi::c_char, nonempty: bool) -> bool;
    fn os_unsetenv(name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn os_get_pid() -> int64_t;
    fn get_appname(namelike: bool) -> *const ::core::ffi::c_char;
    fn stdpaths_get_xdg_var(idx: XDGVarType) -> *mut ::core::ffi::c_char;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn fix_fname(fname: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
}
pub type __socklen_t = ::core::ffi::c_uint;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type size_t = usize;
pub type ssize_t = isize;
pub type ptrdiff_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
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
pub type socklen_t = __socklen_t;
pub type sa_family_t = ::core::ffi::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct addrinfo {
    pub ai_flags: ::core::ffi::c_int,
    pub ai_family: ::core::ffi::c_int,
    pub ai_socktype: ::core::ffi::c_int,
    pub ai_protocol: ::core::ffi::c_int,
    pub ai_addrlen: socklen_t,
    pub ai_addr: *mut sockaddr,
    pub ai_canonname: *mut ::core::ffi::c_char,
    pub ai_next: *mut addrinfo,
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
    pub u: C2Rust_Unnamed_5,
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
pub union C2Rust_Unnamed_5 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_6 {
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
    pub ipc: ::core::ffi::c_int,
    pub pipe_fname: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_7 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_pipe_t = uv_pipe_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timer_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_9,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub timer_cb: uv_timer_cb,
    pub node: C2Rust_Unnamed_8,
    pub timeout: uint64_t,
    pub repeat: uint64_t,
    pub start_id: uint64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_8 {
    pub heap: [*mut ::core::ffi::c_void; 3],
    pub queue: uv__queue,
}
pub type uv_timer_cb = Option<unsafe extern "C" fn(*mut uv_timer_t) -> ()>;
pub type uv_timer_t = uv_timer_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_idle_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_10,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub idle_cb: uv_idle_cb,
    pub queue: uv__queue,
}
pub type uv_idle_cb = Option<unsafe extern "C" fn(*mut uv_idle_t) -> ()>;
pub type uv_idle_t = uv_idle_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_10 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct garray_T {
    pub ga_len: ::core::ffi::c_int,
    pub ga_maxlen: ::core::ffi::c_int,
    pub ga_itemsize: ::core::ffi::c_int,
    pub ga_growsize: ::core::ffi::c_int,
    pub ga_data: *mut ::core::ffi::c_void,
}
pub type hash_T = size_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
pub type LuaRef = ::core::ffi::c_int;
pub type TriState = ::core::ffi::c_int;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub type dict_T = dictvar_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dictvar_S {
    pub dv_lock: VarLockStatus,
    pub dv_scope: ScopeType,
    pub dv_refcount: ::core::ffi::c_int,
    pub dv_copyID: ::core::ffi::c_int,
    pub dv_hashtab: hashtab_T,
    pub dv_copydict: *mut dict_T,
    pub dv_used_next: *mut dict_T,
    pub dv_used_prev: *mut dict_T,
    pub watchers: QUEUE,
    pub lua_table_ref: LuaRef,
}
pub type ScopeType = ::core::ffi::c_uint;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub type VarLockStatus = ::core::ffi::c_uint;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_11,
    pub children_watcher: uv_signal_t,
    pub children_kill_timer: uv_timer_t,
    pub poll_timer: uv_timer_t,
    pub exit_delay_timer: uv_timer_t,
    pub async_0: uv_async_t,
    pub mutex: uv_mutex_t,
    pub recursive: ::core::ffi::c_int,
    pub closing: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_11 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
pub type Proc = proc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct proc {
    pub type_0: ProcType,
    pub loop_0: *mut Loop,
    pub data: *mut ::core::ffi::c_void,
    pub pid: ::core::ffi::c_int,
    pub status: ::core::ffi::c_int,
    pub refcount: ::core::ffi::c_int,
    pub exit_signal: uint8_t,
    pub stopped_time: uint64_t,
    pub cwd: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub exepath: *const ::core::ffi::c_char,
    pub env: *mut dict_T,
    pub in_0: Stream,
    pub out: RStream,
    pub err: RStream,
    pub cb: proc_exit_cb,
    pub state_cb: proc_state_cb,
    pub internal_exit_cb: internal_proc_cb,
    pub internal_close_cb: internal_proc_cb,
    pub closed: bool,
    pub detach: bool,
    pub overlapped: bool,
    pub fwd_err: bool,
    pub stdio_noinherit: bool,
    pub events: *mut MultiQueue,
}
pub type MultiQueue = multiqueue;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct rstream {
    pub s: Stream,
    pub did_eof: bool,
    pub want_read: bool,
    pub pending_read: bool,
    pub paused_full: bool,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub uvbuf: uv_buf_t,
    pub read_cb: stream_read_cb,
    pub num_bytes: size_t,
}
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
pub type Stream = stream;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: C2Rust_Unnamed_12,
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
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct socket_watcher {
    pub addr: [::core::ffi::c_char; 256],
    pub uv: C2Rust_Unnamed_13,
    pub stream: *mut uv_stream_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: socket_cb,
    pub close_cb: socket_close_cb,
    pub events: *mut MultiQueue,
}
pub type socket_close_cb =
    Option<unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> ()>;
pub type SocketWatcher = socket_watcher;
pub type socket_cb = Option<
    unsafe extern "C" fn(*mut SocketWatcher, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_13 {
    pub tcp: C2Rust_Unnamed_15,
    pub pipe: C2Rust_Unnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_14 {
    pub handle: uv_pipe_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub handle: uv_tcp_t,
    pub addrinfo: *mut addrinfo,
}
pub type VimVarIndex = ::core::ffi::c_uint;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
pub type XDGVarType = ::core::ffi::c_int;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MAX_CONNECTIONS: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ENV_LISTEN: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(*b"NVIM_LISTEN_ADDRESS\0")
};
static mut watchers: garray_T = GA_EMPTY_INIT_VALUE;
#[no_mangle]
pub unsafe extern "C" fn server_init(mut listen_addr: *const ::core::ffi::c_char) -> bool {
    let mut ok: bool = true_0 != 0;
    let mut must_free: bool = false_0 != 0;
    let mut user_arg: TriState = kTrue;
    ga_init(
        &raw mut watchers,
        ::core::mem::size_of::<*mut SocketWatcher>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if listen_addr.is_null()
        || *listen_addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\0' as ::core::ffi::c_int
    {
        if os_env_exists(ENV_LISTEN.as_ptr(), true_0 != 0) {
            user_arg = kFalse;
            listen_addr = os_getenv(ENV_LISTEN.as_ptr());
        } else {
            user_arg = kNone;
            listen_addr = server_address_new(::core::ptr::null::<::core::ffi::c_char>());
        }
        must_free = true_0 != 0;
    }
    let mut rv: ::core::ffi::c_int = server_start(listen_addr);
    if os_env_exists(
        b"__NVIM_TEST_LOG\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    ) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_init\0".as_ptr() as *const ::core::ffi::c_char,
            58 as ::core::ffi::c_int,
            true_0 != 0,
            b"test log message\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if !(rv == 0 as ::core::ffi::c_int
        || user_arg as ::core::ffi::c_int == kNone as ::core::ffi::c_int)
    {
        snprintf(
            &raw mut IObuff as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            if user_arg as ::core::ffi::c_int == kTrue as ::core::ffi::c_int {
                b"Failed to --listen: %s: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"Failed $NVIM_LISTEN_ADDRESS: %s: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if rv < 0 as ::core::ffi::c_int {
                uv_strerror(rv)
            } else if rv == 1 as ::core::ffi::c_int {
                b"empty address\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"?\0".as_ptr() as *const ::core::ffi::c_char
            },
            listen_addr,
        );
        ok = false_0 != 0;
    }
    if os_env_exists(ENV_LISTEN.as_ptr(), false_0 != 0) {
        os_unsetenv(ENV_LISTEN.as_ptr());
    }
    if must_free {
        xfree(listen_addr as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void);
    }
    return ok;
}
unsafe extern "C" fn close_socket_watcher(mut watcher: *mut *mut SocketWatcher) {
    socket_watcher_close(
        *watcher,
        Some(
            free_server as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
}
unsafe extern "C" fn set_vservername(mut srvs: *mut garray_T) {
    let mut default_server: *mut ::core::ffi::c_char = if (*srvs).ga_len > 0 as ::core::ffi::c_int {
        &raw mut (**((*srvs).ga_data as *mut *mut SocketWatcher)
            .offset(0 as ::core::ffi::c_int as isize))
        .addr as *mut ::core::ffi::c_char
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    set_vim_var_string(VV_SEND_SERVER, default_server, -1 as ptrdiff_t);
}
#[no_mangle]
pub unsafe extern "C" fn server_teardown() {
    let mut _gap: *mut garray_T = &raw mut watchers;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut *mut SocketWatcher =
                ((*_gap).ga_data as *mut *mut SocketWatcher).offset(i as isize);
            close_socket_watcher(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
}
#[no_mangle]
pub unsafe extern "C" fn server_address_new(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    static mut count: uint32_t = 0 as uint32_t;
    let mut fmt: [::core::ffi::c_char; 256] = [0; 256];
    let mut dir: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(kXDGRuntimeDir);
    get_appname(true_0 != 0);
    let c2rust_fresh1 = count;
    count = count.wrapping_add(1);
    let mut r: ::core::ffi::c_int = snprintf(
        &raw mut fmt as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
        b"%s/%s.%lu.%u\0".as_ptr() as *const ::core::ffi::c_char,
        dir,
        if !name.is_null() {
            name
        } else {
            &raw mut NameBuff as *mut ::core::ffi::c_char as *const ::core::ffi::c_char
        },
        os_get_pid(),
        c2rust_fresh1,
    );
    xfree(dir as *mut ::core::ffi::c_void);
    if r as size_t >= ::core::mem::size_of::<[::core::ffi::c_char; 256]>() {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_address_new\0".as_ptr() as *const ::core::ffi::c_char,
            133 as ::core::ffi::c_int,
            true_0 != 0,
            b"truncated server address: %.40s...\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut fmt as *mut ::core::ffi::c_char,
        );
    }
    return xstrdup(&raw mut fmt as *mut ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn server_owns_pipe_address(mut address: *const ::core::ffi::c_char) -> bool {
    let mut result: bool = false_0 != 0;
    let mut path: *mut ::core::ffi::c_char = fix_fname(address);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < watchers.ga_len {
        let mut addr: *mut ::core::ffi::c_char = fix_fname(
            &raw mut (**(watchers.ga_data as *mut *mut SocketWatcher).offset(i as isize)).addr
                as *mut ::core::ffi::c_char,
        );
        result = strequal(path, addr);
        xfree(addr as *mut ::core::ffi::c_void);
        if result {
            break;
        }
        i += 1;
    }
    xfree(path as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn server_start(mut addr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if addr.is_null() || *addr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
            169 as ::core::ffi::c_int,
            true_0 != 0,
            b"Empty or NULL address\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 1 as ::core::ffi::c_int;
    }
    let mut isname: bool = strstr(addr, b":\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        && strstr(addr, b"/\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        && strstr(addr, b"\\\0".as_ptr() as *const ::core::ffi::c_char).is_null();
    let mut addr_gen: *mut ::core::ffi::c_char = if isname as ::core::ffi::c_int != 0 {
        server_address_new(addr)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut watcher: *mut SocketWatcher =
        xmalloc(::core::mem::size_of::<SocketWatcher>()) as *mut SocketWatcher;
    let mut result: ::core::ffi::c_int = socket_watcher_init(
        &raw mut main_loop,
        watcher,
        if isname as ::core::ffi::c_int != 0 {
            addr_gen as *const ::core::ffi::c_char
        } else {
            addr
        },
    );
    xfree(addr_gen as *mut ::core::ffi::c_void);
    if result < 0 as ::core::ffi::c_int {
        xfree(watcher as *mut ::core::ffi::c_void);
        return result;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < watchers.ga_len {
        if strcmp(
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
            &raw mut (**(watchers.ga_data as *mut *mut SocketWatcher).offset(i as isize)).addr
                as *mut ::core::ffi::c_char,
        ) == 0
        {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
                186 as ::core::ffi::c_int,
                true_0 != 0,
                b"Already listening on %s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
            );
            if (*(*watcher).stream).type_0 as ::core::ffi::c_uint
                == UV_TCP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
            }
            socket_watcher_close(
                watcher,
                Some(
                    free_server
                        as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
                ),
            );
            return 2 as ::core::ffi::c_int;
        }
        i += 1;
    }
    result = socket_watcher_start(
        watcher,
        MAX_CONNECTIONS,
        Some(
            connection_cb
                as unsafe extern "C" fn(
                    *mut SocketWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
    );
    if result < 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_start\0".as_ptr() as *const ::core::ffi::c_char,
            197 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to start server: %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(result),
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        );
        socket_watcher_close(
            watcher,
            Some(
                free_server
                    as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
        return result;
    }
    ga_grow(&raw mut watchers, 1 as ::core::ffi::c_int);
    let c2rust_fresh0 = watchers.ga_len;
    watchers.ga_len = watchers.ga_len + 1;
    let c2rust_lvalue_ptr =
        &raw mut *(watchers.ga_data as *mut *mut SocketWatcher).offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = watcher;
    if strlen(get_vim_var_str(VV_SEND_SERVER)) == 0 as size_t {
        set_vservername(&raw mut watchers);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_stop(
    mut endpoint: *const ::core::ffi::c_char,
    mut keep_vservername: bool,
) -> bool {
    let mut watcher: *mut SocketWatcher = ::core::ptr::null_mut::<SocketWatcher>();
    let mut watcher_found: bool = false_0 != 0;
    let mut addr: [::core::ffi::c_char; 256] = [0; 256];
    xstrlcpy(
        &raw mut addr as *mut ::core::ffi::c_char,
        endpoint,
        ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < watchers.ga_len {
        watcher = *(watchers.ga_data as *mut *mut SocketWatcher).offset(i as isize);
        if strcmp(
            &raw mut addr as *mut ::core::ffi::c_char,
            &raw mut (*watcher).addr as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            watcher_found = true_0 != 0;
            break;
        } else {
            i += 1;
        }
    }
    if !watcher_found {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"server_stop\0".as_ptr() as *const ::core::ffi::c_char,
            236 as ::core::ffi::c_int,
            true_0 != 0,
            b"Not listening on %s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut addr as *mut ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    socket_watcher_close(
        watcher,
        Some(
            free_server as unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
    if i != watchers.ga_len - 1 as ::core::ffi::c_int {
        *(watchers.ga_data as *mut *mut SocketWatcher).offset(i as isize) = *(watchers.ga_data
            as *mut *mut SocketWatcher)
            .offset((watchers.ga_len - 1 as ::core::ffi::c_int) as isize);
    }
    watchers.ga_len -= 1;
    if !keep_vservername
        && strequal(
            &raw mut addr as *mut ::core::ffi::c_char,
            get_vim_var_str(VV_SEND_SERVER),
        ) as ::core::ffi::c_int
            != 0
    {
        set_vservername(&raw mut watchers);
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn server_address_list(
    mut size: *mut size_t,
) -> *mut *mut ::core::ffi::c_char {
    *size = watchers.ga_len as size_t;
    if *size == 0 as size_t {
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    let mut addrs: *mut *mut ::core::ffi::c_char = xcalloc(
        watchers.ga_len as size_t,
        ::core::mem::size_of::<*const ::core::ffi::c_char>(),
    ) as *mut *mut ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < watchers.ga_len {
        *addrs.offset(i as isize) = xstrdup(
            &raw mut (**(watchers.ga_data as *mut *mut SocketWatcher).offset(i as isize)).addr
                as *mut ::core::ffi::c_char,
        );
        i += 1;
    }
    return addrs;
}
unsafe extern "C" fn connection_cb(
    mut watcher: *mut SocketWatcher,
    mut result: ::core::ffi::c_int,
    mut data: *mut ::core::ffi::c_void,
) {
    if result != 0 {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"connection_cb\0".as_ptr() as *const ::core::ffi::c_char,
            276 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to accept connection: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(result),
        );
        return;
    }
    channel_from_connection(watcher);
}
unsafe extern "C" fn free_server(
    mut watcher: *mut SocketWatcher,
    mut data: *mut ::core::ffi::c_void,
) {
    xfree(watcher as *mut ::core::ffi::c_void);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
