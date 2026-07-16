extern "C" {
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn uv_strerror(err: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn uv_close(handle: *mut uv_handle_t, close_cb_0: uv_close_cb);
    fn uv_pipe(
        fds: *mut uv_file,
        read_flags: ::core::ffi::c_int,
        write_flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_pipe_open(_: *mut uv_pipe_t, file: uv_file) -> ::core::ffi::c_int;
    fn uv_spawn(
        loop_0: *mut uv_loop_t,
        handle: *mut uv_process_t,
        options: *const uv_process_options_t,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn tv_dict_to_env(denv: *mut dict_T) -> *mut *mut ::core::ffi::c_char;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn os_free_fullenv(env: *mut *mut ::core::ffi::c_char);
    static mut ui_client_forward_stdin: bool;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint64_t = u64;
pub type size_t = usize;
pub type ssize_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
}
pub type gid_t = __gid_t;
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
pub struct uv_process_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_11,
    pub next_closing: *mut uv_handle_t,
    pub flags: ::core::ffi::c_uint,
    pub exit_cb: uv_exit_cb,
    pub pid: ::core::ffi::c_int,
    pub queue: uv__queue,
    pub status: ::core::ffi::c_int,
}
pub type uv_exit_cb =
    Option<unsafe extern "C" fn(*mut uv_process_t, int64_t, ::core::ffi::c_int) -> ()>;
pub type uv_process_t = uv_process_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_11 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_stdio_flags = ::core::ffi::c_uint;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_stdio_container_s {
    pub flags: uv_stdio_flags,
    pub data: C2Rust_Unnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_12 {
    pub stream: *mut uv_stream_t,
    pub fd: ::core::ffi::c_int,
}
pub type uv_stdio_container_t = uv_stdio_container_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_process_options_s {
    pub exit_cb: uv_exit_cb,
    pub file: *const ::core::ffi::c_char,
    pub args: *mut *mut ::core::ffi::c_char,
    pub env: *mut *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_uint,
    pub stdio_count: ::core::ffi::c_int,
    pub stdio: *mut uv_stdio_container_t,
    pub uid: uv_uid_t,
    pub gid: uv_gid_t,
}
pub type uv_process_options_t = uv_process_options_s;
pub type uv_process_flags = ::core::ffi::c_uint;
pub const UV_PROCESS_WINDOWS_FILE_PATH_EXACT_NAME: uv_process_flags = 128;
pub const UV_PROCESS_WINDOWS_HIDE_GUI: uv_process_flags = 64;
pub const UV_PROCESS_WINDOWS_HIDE_CONSOLE: uv_process_flags = 32;
pub const UV_PROCESS_WINDOWS_HIDE: uv_process_flags = 16;
pub const UV_PROCESS_DETACHED: uv_process_flags = 8;
pub const UV_PROCESS_WINDOWS_VERBATIM_ARGUMENTS: uv_process_flags = 4;
pub const UV_PROCESS_SETGID: uv_process_flags = 2;
pub const UV_PROCESS_SETUID: uv_process_flags = 1;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: C2Rust_Unnamed_13,
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
pub struct C2Rust_Unnamed_13 {
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
    pub uv: C2Rust_Unnamed_14,
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
pub union C2Rust_Unnamed_14 {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
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
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 34] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_char; 34]>(
        *b"int libuv_proc_spawn(LibuvProc *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
unsafe extern "C" fn libuv_proc_stdio(
    mut uvproc: *mut LibuvProc,
    mut idx: ::core::ffi::c_int,
    mut parent_pipe: *mut uv_pipe_t,
    mut child_readable: bool,
    mut overlapped: bool,
    mut win_create_pipe: bool,
    mut to_close: *mut ::core::ffi::c_int,
) {
    let mut child_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pipe_pair: [uv_file; 2] = [0; 2];
    uv_pipe(
        &raw mut pipe_pair as *mut uv_file,
        if child_readable as ::core::ffi::c_int != 0 {
            child_flags
        } else {
            UV_NONBLOCK_PIPE as ::core::ffi::c_int
        },
        if child_readable as ::core::ffi::c_int != 0 {
            UV_NONBLOCK_PIPE as ::core::ffi::c_int
        } else {
            child_flags
        },
    );
    let mut child_fd: ::core::ffi::c_int = if child_readable as ::core::ffi::c_int != 0 {
        pipe_pair[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    } else {
        pipe_pair[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    };
    let mut parent_fd: ::core::ffi::c_int = if child_readable as ::core::ffi::c_int != 0 {
        pipe_pair[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    } else {
        pipe_pair[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
    };
    (*uvproc).uvstdio[idx as usize].flags = UV_INHERIT_FD;
    (*uvproc).uvstdio[idx as usize].data.fd = child_fd;
    *to_close.offset(idx as isize) = child_fd;
    uv_pipe_open(parent_pipe, parent_fd as uv_file);
}
#[no_mangle]
pub unsafe extern "C" fn libuv_proc_spawn(mut uvproc: *mut LibuvProc) -> ::core::ffi::c_int {
    let mut proc: *mut Proc = uvproc as *mut Proc;
    (*uvproc).uvopts.file = proc_get_exepath(proc);
    (*uvproc).uvopts.args = (*proc).argv;
    (*uvproc).uvopts.flags = UV_PROCESS_WINDOWS_HIDE as ::core::ffi::c_int as ::core::ffi::c_uint;
    (*uvproc).uvopts.flags |= UV_PROCESS_DETACHED as ::core::ffi::c_int as ::core::ffi::c_uint;
    (*uvproc).uvopts.exit_cb =
        Some(exit_cb as unsafe extern "C" fn(*mut uv_process_t, int64_t, ::core::ffi::c_int) -> ())
            as uv_exit_cb;
    (*uvproc).uvopts.cwd = (*proc).cwd;
    (*uvproc).uvopts.stdio = &raw mut (*uvproc).uvstdio as *mut uv_stdio_container_t;
    (*uvproc).uvopts.stdio_count = 3 as ::core::ffi::c_int;
    (*uvproc).uvstdio[0 as ::core::ffi::c_int as usize].flags = UV_IGNORE;
    (*uvproc).uvstdio[1 as ::core::ffi::c_int as usize].flags = UV_IGNORE;
    (*uvproc).uvstdio[2 as ::core::ffi::c_int as usize].flags = UV_IGNORE;
    if ui_client_forward_stdin {
        '_c2rust_label: {
            if 3 as ::core::ffi::c_int == 3 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"UI_CLIENT_STDIN_FD == 3\0".as_ptr() as *const ::core::ffi::c_char,
                    b"/home/overlord/projects/neovim/neovim/src/nvim/event/libuv_proc.c\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    106 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        (*uvproc).uvopts.stdio_count = 4 as ::core::ffi::c_int;
        (*uvproc).uvstdio[3 as ::core::ffi::c_int as usize].data.fd = 0 as ::core::ffi::c_int;
        (*uvproc).uvstdio[3 as ::core::ffi::c_int as usize].flags = UV_INHERIT_FD;
    }
    (*uvproc).uv.data = proc as *mut ::core::ffi::c_void;
    if !(*proc).env.is_null() {
        (*uvproc).uvopts.env = tv_dict_to_env((*proc).env);
    } else {
        (*uvproc).uvopts.env = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    let mut to_close: [::core::ffi::c_int; 3] = [
        -1 as ::core::ffi::c_int,
        -1 as ::core::ffi::c_int,
        -1 as ::core::ffi::c_int,
    ];
    if !(*proc).in_0.closed {
        libuv_proc_stdio(
            uvproc,
            0 as ::core::ffi::c_int,
            &raw mut (*proc).in_0.uv.pipe,
            true_0 != 0,
            (*proc).overlapped,
            (*proc).stdio_noinherit,
            &raw mut to_close as *mut ::core::ffi::c_int,
        );
    }
    if !(*proc).out.s.closed {
        libuv_proc_stdio(
            uvproc,
            1 as ::core::ffi::c_int,
            &raw mut (*proc).out.s.uv.pipe,
            false_0 != 0,
            (*proc).overlapped,
            true_0 != 0,
            &raw mut to_close as *mut ::core::ffi::c_int,
        );
    }
    if !(*proc).err.s.closed {
        libuv_proc_stdio(
            uvproc,
            2 as ::core::ffi::c_int,
            &raw mut (*proc).err.s.uv.pipe,
            false_0 != 0,
            (*proc).overlapped,
            (*proc).stdio_noinherit,
            &raw mut to_close as *mut ::core::ffi::c_int,
        );
    } else if (*proc).fwd_err {
        (*uvproc).uvstdio[2 as ::core::ffi::c_int as usize].flags = UV_INHERIT_FD;
        (*uvproc).uvstdio[2 as ::core::ffi::c_int as usize].data.fd = STDERR_FILENO;
    }
    let mut status: ::core::ffi::c_int = 0;
    status = uv_spawn(
        &raw mut (*(*proc).loop_0).uv,
        &raw mut (*uvproc).uv,
        &raw mut (*uvproc).uvopts,
    );
    if status != 0 {
        logmsg(
            LOGLVL_INF,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"libuv_proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
            141 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_spawn(%s) failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*uvproc).uvopts.file,
            uv_strerror(status),
        );
        if !(*uvproc).uvopts.env.is_null() {
            os_free_fullenv((*uvproc).uvopts.env);
        }
    } else {
        (*proc).pid = (*uvproc).uv.pid;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 3 as ::core::ffi::c_int {
        if to_close[i as usize] > -1 as ::core::ffi::c_int {
            close(to_close[i as usize]);
        }
        i += 1;
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn libuv_proc_close(mut uvproc: *mut LibuvProc) {
    uv_close(
        &raw mut (*uvproc).uv as *mut uv_handle_t,
        Some(close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
    );
}
unsafe extern "C" fn close_cb(mut handle: *mut uv_handle_t) {
    let mut proc: *mut Proc = (*handle).data as *mut Proc;
    if (*proc).internal_close_cb.is_some() {
        (*proc)
            .internal_close_cb
            .expect("non-null function pointer")(proc);
    }
    let mut uvproc: *mut LibuvProc = proc as *mut LibuvProc;
    if !(*uvproc).uvopts.env.is_null() {
        os_free_fullenv((*uvproc).uvopts.env);
    }
}
unsafe extern "C" fn exit_cb(
    mut handle: *mut uv_process_t,
    mut status: int64_t,
    mut term_signal: ::core::ffi::c_int,
) {
    let mut proc: *mut Proc = (*handle).data as *mut Proc;
    (*proc).status = if term_signal != 0 {
        128 as ::core::ffi::c_int + term_signal
    } else {
        status as ::core::ffi::c_int
    };
    (*proc).internal_exit_cb.expect("non-null function pointer")(proc);
}
#[no_mangle]
pub unsafe extern "C" fn libuv_proc_init(
    mut loop_0: *mut Loop,
    mut data: *mut ::core::ffi::c_void,
) -> LibuvProc {
    let mut rv: LibuvProc = LibuvProc {
        proc: proc_init(loop_0, kProcTypeUv, data),
        uv: uv_process_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_11 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            exit_cb: None,
            pid: 0,
            queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            status: 0,
        },
        uvopts: uv_process_options_t {
            exit_cb: None,
            file: ::core::ptr::null::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            env: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cwd: ::core::ptr::null::<::core::ffi::c_char>(),
            flags: 0,
            stdio_count: 0,
            stdio: ::core::ptr::null_mut::<uv_stdio_container_t>(),
            uid: 0,
            gid: 0,
        },
        uvstdio: [uv_stdio_container_t {
            flags: UV_IGNORE,
            data: C2Rust_Unnamed_12 {
                stream: ::core::ptr::null_mut::<uv_stream_t>(),
            },
        }; 4],
    };
    return rv;
}
#[inline]
unsafe extern "C" fn proc_init(
    mut loop_0: *mut Loop,
    mut type_0: ProcType,
    mut data: *mut ::core::ffi::c_void,
) -> Proc {
    return proc {
        type_0: type_0,
        loop_0: loop_0,
        data: data,
        pid: 0 as ::core::ffi::c_int,
        status: -1 as ::core::ffi::c_int,
        refcount: 0 as ::core::ffi::c_int,
        exit_signal: 0,
        stopped_time: 0 as uint64_t,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        exepath: ::core::ptr::null::<::core::ffi::c_char>(),
        env: ::core::ptr::null_mut::<dict_T>(),
        in_0: stream {
            closed: false_0 != 0,
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
                    u: C2Rust_Unnamed_7 { fd: 0 },
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
        out: rstream {
            s: stream {
                closed: false_0 != 0,
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
                        u: C2Rust_Unnamed_7 { fd: 0 },
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
                fd: STDOUT_FILENO,
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
        },
        err: rstream {
            s: stream {
                closed: false_0 != 0,
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
                        u: C2Rust_Unnamed_7 { fd: 0 },
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
                fd: STDERR_FILENO,
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
        },
        cb: None,
        state_cb: None,
        internal_exit_cb: None,
        internal_close_cb: None,
        closed: false_0 != 0,
        detach: false_0 != 0,
        overlapped: false,
        fwd_err: false_0 != 0,
        stdio_noinherit: false_0 != 0,
        events: ::core::ptr::null_mut::<MultiQueue>(),
    };
}
#[inline]
unsafe extern "C" fn proc_get_exepath(mut proc: *mut Proc) -> *const ::core::ffi::c_char {
    return if !(*proc).exepath.is_null() {
        (*proc).exepath
    } else {
        *(*proc).argv.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char
    };
}
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
