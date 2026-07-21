use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
extern "C" {
    pub type multiqueue;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn uv_unref(_: *mut uv_handle_t);
    fn uv_close(handle: *mut uv_handle_t, close_cb: uv_close_cb);
    fn uv_recv_buffer_size(
        handle: *mut uv_handle_t,
        value: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_pipe_init(
        _: *mut uv_loop_t,
        handle: *mut uv_pipe_t,
        ipc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_timer_start(
        handle: *mut uv_timer_t,
        cb: uv_timer_cb,
        timeout: uint64_t,
        repeat: uint64_t,
    ) -> ::core::ffi::c_int;
    fn uv_timer_stop(handle: *mut uv_timer_t) -> ::core::ffi::c_int;
    fn xrealloc(ptr: *mut ::core::ffi::c_void, size: size_t) -> *mut ::core::ffi::c_void;
    fn libuv_proc_spawn(uvproc: *mut LibuvProc) -> ::core::ffi::c_int;
    fn libuv_proc_close(uvproc: *mut LibuvProc);
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
    fn multiqueue_put_event(self_0: *mut MultiQueue, event: Event);
    fn multiqueue_process_events(self_0: *mut MultiQueue);
    fn multiqueue_empty(self_0: *mut MultiQueue) -> bool;
    fn os_hrtime() -> uint64_t;
    fn rstream_may_close(stream: *mut RStream);
    static exiting: GlobalCell<bool>;
    static got_int: GlobalCell<bool>;
    static main_loop: SharedCell<Loop>;
    fn os_exit(r: ::core::ffi::c_int) -> !;
    fn preserve_exit(errmsg: *const ::core::ffi::c_char) -> !;
    fn stream_init(
        loop_0: *mut Loop,
        stream: *mut Stream,
        fd: ::core::ffi::c_int,
        uvstream: *mut uv_stream_t,
    );
    fn stream_may_close(stream: *mut Stream);
    fn os_proc_tree_kill(pid: ::core::ffi::c_int, sig: ::core::ffi::c_int) -> bool;
    fn pty_proc_spawn(ptyproc: *mut PtyProc) -> ::core::ffi::c_int;
    fn pty_proc_flush_master(ptyproc: *mut PtyProc);
    fn pty_proc_close(ptyproc: *mut PtyProc);
    fn pty_proc_close_master(ptyproc: *mut PtyProc);
    fn pty_proc_teardown(loop_0: *mut Loop);
    fn shell_free_argv(argv: *mut *mut ::core::ffi::c_char);
    static ui_client_channel_id: GlobalCell<uint64_t>;
    static ui_client_exit_status: GlobalCell<::core::ffi::c_int>;
    fn loop_poll_events(loop_0: *mut Loop, ms: int64_t) -> bool;
}
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type uid_t = __uid_t;
pub type size_t = usize;
pub type ssize_t = isize;
pub type gid_t = __gid_t;
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
pub struct uv__queue {
    pub next: *mut uv__queue,
    pub prev: *mut uv__queue,
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
pub type Loop = loop_0;
pub type ProcType = ::core::ffi::c_uint;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LibuvProc {
    pub proc: Proc,
    pub uv: uv_process_t,
    pub uvopts: uv_process_options_t,
    pub uvstdio: [uv_stdio_container_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PtyProc {
    pub proc: Proc,
    pub width: uint16_t,
    pub height: uint16_t,
    pub winsize: winsize,
    pub tty_fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
}
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KILL_TIMEOUT_MS: ::core::ffi::c_int = 2000 as ::core::ffi::c_int;
static proc_is_tearing_down: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static exit_need_delay: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
#[no_mangle]
pub unsafe extern "C" fn proc_spawn(
    mut proc: *mut Proc,
    mut in_0: bool,
    mut out: bool,
    mut err: bool,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if !(err as ::core::ffi::c_int != 0 && (*proc).fwd_err as ::core::ffi::c_int != 0) {
        } else {
            __assert_fail(
                b"!(err && proc->fwd_err)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/proc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                46 as ::core::ffi::c_uint,
                b"int proc_spawn(Proc *, _Bool, _Bool, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if in_0 {
        uv_pipe_init(
            &raw mut (*(*proc).loop_0).uv,
            &raw mut (*proc).in_0.uv.pipe,
            0 as ::core::ffi::c_int,
        );
    } else {
        (*proc).in_0.closed = true_0 != 0;
    }
    if out {
        uv_pipe_init(
            &raw mut (*(*proc).loop_0).uv,
            &raw mut (*proc).out.s.uv.pipe,
            0 as ::core::ffi::c_int,
        );
    } else {
        (*proc).out.s.closed = true_0 != 0;
    }
    if err {
        uv_pipe_init(
            &raw mut (*(*proc).loop_0).uv,
            &raw mut (*proc).err.s.uv.pipe,
            0 as ::core::ffi::c_int,
        );
    } else {
        (*proc).err.s.closed = true_0 != 0;
    }
    let mut status: ::core::ffi::c_int = 0;
    match (*proc).type_0 as ::core::ffi::c_uint {
        0 => {
            status = libuv_proc_spawn(proc as *mut LibuvProc);
        }
        1 => {
            status = pty_proc_spawn(proc as *mut PtyProc);
        }
        _ => {}
    }
    if status != 0 {
        if in_0 {
            uv_close(&raw mut (*proc).in_0.uv.pipe as *mut uv_handle_t, None);
        }
        if out {
            uv_close(&raw mut (*proc).out.s.uv.pipe as *mut uv_handle_t, None);
        }
        if err {
            uv_close(&raw mut (*proc).err.s.uv.pipe as *mut uv_handle_t, None);
        }
        if (*proc).type_0 as ::core::ffi::c_uint
            == kProcTypeUv as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            uv_close(
                &raw mut (*(proc as *mut LibuvProc)).uv as *mut uv_handle_t,
                None,
            );
        } else {
            proc_close(proc);
        }
        proc_free(proc);
        (*proc).status = -1 as ::core::ffi::c_int;
        return status;
    }
    if in_0 {
        stream_init(
            ::core::ptr::null_mut::<Loop>(),
            &raw mut (*proc).in_0,
            -1 as ::core::ffi::c_int,
            &raw mut (*proc).in_0.uv.pipe as *mut uv_stream_t,
        );
        (*proc).in_0.internal_data = proc as *mut ::core::ffi::c_void;
        (*proc).in_0.internal_close_cb = Some(
            on_proc_stream_close
                as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> (),
        ) as stream_close_cb;
        (*proc).refcount += 1;
    }
    if out {
        stream_init(
            ::core::ptr::null_mut::<Loop>(),
            &raw mut (*proc).out.s,
            -1 as ::core::ffi::c_int,
            &raw mut (*proc).out.s.uv.pipe as *mut uv_stream_t,
        );
        (*proc).out.s.internal_data = proc as *mut ::core::ffi::c_void;
        (*proc).out.s.internal_close_cb = Some(
            on_proc_stream_close
                as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> (),
        ) as stream_close_cb;
        (*proc).refcount += 1;
    }
    if err {
        stream_init(
            ::core::ptr::null_mut::<Loop>(),
            &raw mut (*proc).err.s,
            -1 as ::core::ffi::c_int,
            &raw mut (*proc).err.s.uv.pipe as *mut uv_stream_t,
        );
        (*proc).err.s.internal_data = proc as *mut ::core::ffi::c_void;
        (*proc).err.s.internal_close_cb = Some(
            on_proc_stream_close
                as unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> (),
        ) as stream_close_cb;
        (*proc).refcount += 1;
    }
    (*proc).internal_exit_cb =
        Some(on_proc_exit as unsafe extern "C" fn(*mut Proc) -> ()) as internal_proc_cb;
    (*proc).internal_close_cb =
        Some(decref as unsafe extern "C" fn(*mut Proc) -> ()) as internal_proc_cb;
    (*proc).refcount += 1;
    if (*(*proc).loop_0).children.size == (*(*proc).loop_0).children.capacity {
        (*(*proc).loop_0).children.capacity = if (*(*proc).loop_0).children.capacity != 0 {
            (*(*proc).loop_0).children.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*(*proc).loop_0).children.items = xrealloc(
            (*(*proc).loop_0).children.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut Proc>().wrapping_mul((*(*proc).loop_0).children.capacity),
        ) as *mut *mut Proc;
    } else {
    };
    let c2rust_fresh0 = (*(*proc).loop_0).children.size;
    (*(*proc).loop_0).children.size = (*(*proc).loop_0).children.size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *(*(*proc).loop_0)
        .children
        .items
        .offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = proc;
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
        127 as ::core::ffi::c_int,
        true_0 != 0,
        b"new: pid=%d exepath=[%s]\0".as_ptr() as *const ::core::ffi::c_char,
        (*proc).pid,
        proc_get_exepath(proc),
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn proc_teardown(mut loop_0: *mut Loop) {
    proc_is_tearing_down.set(true_0 != 0);
    let mut i: size_t = 0 as size_t;
    while i < (*loop_0).children.size {
        let mut proc: *mut Proc = *(*loop_0).children.items.offset(i as isize);
        if (*proc).detach as ::core::ffi::c_int != 0
            || (*proc).type_0 as ::core::ffi::c_uint
                == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !(*loop_0).events.is_null() {
                multiqueue_put_event(
                    (*loop_0).events,
                    Event {
                        handler: Some(
                            proc_close_handles
                                as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                        ),
                        argv: [
                            proc as *mut ::core::ffi::c_void,
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
                let mut argv: [*mut ::core::ffi::c_void; 1] = [proc as *mut ::core::ffi::c_void];
                proc_close_handles(&raw mut argv as *mut *mut ::core::ffi::c_void);
            }
        } else {
            proc_stop(proc);
        }
        i = i.wrapping_add(1);
    }
    let mut remaining: int64_t = -1 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !((*loop_0).children.size == 0 as size_t
        && multiqueue_empty((*loop_0).events) as ::core::ffi::c_int != 0)
    {
        if !(*loop_0).events.is_null() && !multiqueue_empty((*loop_0).events) {
            multiqueue_process_events((*loop_0).events);
        } else {
            loop_poll_events(loop_0, remaining);
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
    pty_proc_teardown(loop_0);
}
#[no_mangle]
pub unsafe extern "C" fn proc_close_streams(mut proc: *mut Proc) {
    stream_may_close(&raw mut (*proc).in_0);
    rstream_may_close(&raw mut (*proc).out);
    rstream_may_close(&raw mut (*proc).err);
}
#[no_mangle]
pub unsafe extern "C" fn proc_wait(
    mut proc: *mut Proc,
    mut ms: ::core::ffi::c_int,
    mut events: *mut MultiQueue,
) -> ::core::ffi::c_int {
    if (*proc).refcount == 0 {
        let mut status: ::core::ffi::c_int = (*proc).status;
        if !(*proc).events.is_null() && !multiqueue_empty((*proc).events) {
            multiqueue_process_events((*proc).events);
        } else {
            loop_poll_events((*proc).loop_0, 0 as int64_t);
        }
        return status;
    }
    if events.is_null() {
        events = (*proc).events;
    }
    (*proc).refcount += 1;
    let mut remaining: int64_t = ms as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !(got_int.get() as ::core::ffi::c_int != 0 || (*proc).refcount == 1 as ::core::ffi::c_int)
    {
        if !events.is_null() && !multiqueue_empty(events) {
            multiqueue_process_events(events);
        } else {
            loop_poll_events((*proc).loop_0, remaining);
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
    if got_int.get() {
        got_int.set(false_0 != 0);
        proc_stop(proc);
        if ms == -1 as ::core::ffi::c_int {
            let mut remaining_0: int64_t = -1 as int64_t;
            let mut before_0: uint64_t = if remaining_0 > 0 as int64_t {
                os_hrtime()
            } else {
                0 as uint64_t
            };
            while (*proc).refcount != 1 as ::core::ffi::c_int {
                if !events.is_null() && !multiqueue_empty(events) {
                    multiqueue_process_events(events);
                } else {
                    loop_poll_events((*proc).loop_0, remaining_0);
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
                    .wrapping_div(1000000 as uint64_t) as int64_t;
                before_0 = now_0;
                if remaining_0 <= 0 as int64_t {
                    break;
                }
            }
        } else if !events.is_null() && !multiqueue_empty(events) {
            multiqueue_process_events(events);
        } else {
            loop_poll_events((*proc).loop_0, 0 as int64_t);
        }
        (*proc).status = -2 as ::core::ffi::c_int;
    }
    if (*proc).refcount == 1 as ::core::ffi::c_int {
        decref(proc);
        if !(*proc).events.is_null() {
            multiqueue_process_events((*proc).events);
        }
    } else {
        (*proc).refcount -= 1;
    }
    return (*proc).status;
}
#[no_mangle]
pub unsafe extern "C" fn proc_stop(mut proc: *mut Proc) {
    let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
    if exited as ::core::ffi::c_int != 0 || (*proc).stopped_time != 0 {
        return;
    }
    (*proc).stopped_time = os_hrtime();
    match (*proc).type_0 as ::core::ffi::c_uint {
        0 => {
            (*proc).exit_signal = SIGTERM as uint8_t;
            os_proc_tree_kill((*proc).pid, SIGTERM);
        }
        1 => {
            (*proc).exit_signal = SIGHUP as uint8_t;
            proc_close_streams(proc);
            pty_proc_close_master(proc as *mut PtyProc);
        }
        _ => {}
    }
    uv_timer_start(
        &raw mut (*(*proc).loop_0).children_kill_timer,
        Some(children_kill_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
        KILL_TIMEOUT_MS as uint64_t,
        0 as uint64_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn proc_free(mut proc: *mut Proc) {
    if !(*proc).argv.is_null() {
        shell_free_argv((*proc).argv);
        (*proc).argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
}
unsafe extern "C" fn children_kill_cb(mut handle: *mut uv_timer_t) {
    let mut loop_0: *mut Loop = (*(*handle).loop_0).data as *mut Loop;
    let mut i: size_t = 0 as size_t;
    while i < (*loop_0).children.size {
        let mut proc: *mut Proc = *(*loop_0).children.items.offset(i as isize);
        let mut exited: bool = (*proc).status >= 0 as ::core::ffi::c_int;
        if !(exited as ::core::ffi::c_int != 0 || (*proc).stopped_time == 0) {
            let mut term_sent: uint64_t =
                (UINT64_MAX as uint64_t == (*proc).stopped_time) as ::core::ffi::c_int as uint64_t;
            if kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*proc).type_0 as ::core::ffi::c_uint
                || term_sent != 0
            {
                (*proc).exit_signal = SIGKILL as uint8_t;
                os_proc_tree_kill((*proc).pid, SIGKILL);
            } else {
                (*proc).exit_signal = SIGTERM as uint8_t;
                os_proc_tree_kill((*proc).pid, SIGTERM);
                (*proc).stopped_time = UINT64_MAX as uint64_t;
                uv_timer_start(
                    &raw mut (*(*proc).loop_0).children_kill_timer,
                    Some(children_kill_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
                    KILL_TIMEOUT_MS as uint64_t,
                    0 as uint64_t,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn proc_close_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut proc: *mut Proc = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Proc;
    if (*proc).cb.is_some() {
        (*proc).cb.expect("non-null function pointer")(proc, (*proc).status, (*proc).data);
    } else {
        proc_free(proc);
    };
}
unsafe extern "C" fn decref(mut proc: *mut Proc) {
    (*proc).refcount -= 1;
    if (*proc).refcount != 0 as ::core::ffi::c_int {
        return;
    }
    let mut loop_0: *mut Loop = (*proc).loop_0;
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < (*loop_0).children.size {
        let mut current: *mut Proc = *(*loop_0).children.items.offset(i as isize);
        if current == proc {
            break;
        }
        i = i.wrapping_add(1);
    }
    '_c2rust_label: {
        if i < (*loop_0).children.size {
        } else {
            __assert_fail(
                b"i < kv_size(loop->children)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/proc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                305 as ::core::ffi::c_uint,
                b"void decref(Proc *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if i < (*loop_0).children.size.wrapping_sub(1 as size_t) {
        memmove(
            (*loop_0).children.items.offset(i as isize) as *mut ::core::ffi::c_void,
            (*loop_0)
                .children
                .items
                .offset(i.wrapping_add(1 as size_t) as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<*mut *mut Proc>().wrapping_mul(
                (*loop_0)
                    .children
                    .size
                    .wrapping_sub(i.wrapping_add(1 as size_t)),
            ),
        );
    }
    (*loop_0).children.size = (*loop_0).children.size.wrapping_sub(1);
    if !(*proc).events.is_null() {
        multiqueue_put_event(
            (*proc).events,
            Event {
                handler: Some(
                    proc_close_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    proc as *mut ::core::ffi::c_void,
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
        let mut argv: [*mut ::core::ffi::c_void; 1] = [proc as *mut ::core::ffi::c_void];
        proc_close_event(&raw mut argv as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn proc_close(mut proc: *mut Proc) {
    if proc_is_tearing_down.get() as ::core::ffi::c_int != 0
        && (*proc).closed as ::core::ffi::c_int != 0
        && ((*proc).detach as ::core::ffi::c_int != 0
            || (*proc).type_0 as ::core::ffi::c_uint
                == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return;
    }
    '_c2rust_label: {
        if !(*proc).closed {
        } else {
            __assert_fail(
                b"!proc->closed\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/proc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                321 as ::core::ffi::c_uint,
                b"void proc_close(Proc *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    (*proc).closed = true_0 != 0;
    if (*proc).detach {
        if (*proc).type_0 as ::core::ffi::c_uint
            == kProcTypeUv as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            uv_unref(&raw mut (*(proc as *mut LibuvProc)).uv as *mut uv_handle_t);
        }
    }
    match (*proc).type_0 as ::core::ffi::c_uint {
        0 => {
            libuv_proc_close(proc as *mut LibuvProc);
        }
        1 => {
            pty_proc_close(proc as *mut PtyProc);
        }
        _ => {}
    };
}
unsafe extern "C" fn flush_stream(mut proc: *mut Proc, mut stream: *mut RStream) {
    if stream.is_null() || (*stream).s.closed as ::core::ffi::c_int != 0 {
        return;
    }
    let mut max_bytes: size_t = SIZE_MAX as size_t;
    if (*proc).type_0 as ::core::ffi::c_uint
        != kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
        || proc_is_tearing_down.get() as ::core::ffi::c_int != 0
    {
        let mut system_buffer_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut err: ::core::ffi::c_int = uv_recv_buffer_size(
            &raw mut (*stream).s.uv as *mut uv_handle_t,
            &raw mut system_buffer_size,
        );
        if err != 0 as ::core::ffi::c_int {
            system_buffer_size = ARENA_BLOCK_SIZE;
        }
        max_bytes = (*stream)
            .num_bytes
            .wrapping_add(system_buffer_size as size_t);
    }
    while !(*stream).s.closed && (*stream).num_bytes < max_bytes {
        let mut num_bytes: size_t = (*stream).num_bytes;
        if (*proc).type_0 as ::core::ffi::c_uint
            == kProcTypePty as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*stream).did_eof
        {
            pty_proc_flush_master(proc as *mut PtyProc);
        }
        loop_poll_events((*proc).loop_0, 0 as int64_t);
        if !(*stream).s.events.is_null() {
            multiqueue_process_events((*stream).s.events);
        }
        if num_bytes != (*stream).num_bytes {
            continue;
        }
        if (*stream).read_cb.is_some() && !(*stream).did_eof {
            (*stream).read_cb.expect("non-null function pointer")(
                stream,
                (*stream).buffer,
                0 as size_t,
                (*stream).s.cb_data,
                true_0 != 0,
            );
        }
        break;
    }
}
unsafe extern "C" fn proc_close_handles(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut proc: *mut Proc = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Proc;
    (*exit_need_delay.ptr()) += 1;
    flush_stream(proc, &raw mut (*proc).out);
    flush_stream(proc, &raw mut (*proc).err);
    proc_close_streams(proc);
    proc_close(proc);
    (*exit_need_delay.ptr()) -= 1;
}
unsafe extern "C" fn exit_delay_cb(mut _handle: *mut uv_timer_t) {
    uv_timer_stop(&raw mut (*main_loop.ptr()).exit_delay_timer);
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        Event {
            handler: Some(exit_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()),
            argv: [
                (*main_loop.ptr()).exit_delay_timer.data,
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
}
unsafe extern "C" fn exit_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut status: ::core::ffi::c_int = (*argv.offset(0 as ::core::ffi::c_int as isize))
        .expose_addr() as intptr_t as ::core::ffi::c_int;
    if exit_need_delay.get() != 0 {
        (*main_loop.ptr()).exit_delay_timer.data = *argv.offset(0 as ::core::ffi::c_int as isize);
        uv_timer_start(
            &raw mut (*main_loop.ptr()).exit_delay_timer,
            Some(exit_delay_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
            0 as uint64_t,
            0 as uint64_t,
        );
        return;
    }
    if !exiting.get() {
        if ui_client_channel_id.get() != 0 {
            ui_client_exit_status.set(status);
            os_exit(status);
        } else {
            '_c2rust_label: {
                if status == 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"status == 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/event/proc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        431 as ::core::ffi::c_uint,
                        b"void exit_event(void **)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            preserve_exit(::core::ptr::null::<::core::ffi::c_char>());
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn exit_on_closed_chan(mut status: ::core::ffi::c_int) {
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"exit_on_closed_chan\0".as_ptr() as *const ::core::ffi::c_char,
        440 as ::core::ffi::c_int,
        true_0 != 0,
        b"self-exit triggered by closed RPC channel...\0".as_ptr() as *const ::core::ffi::c_char,
    );
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        Event {
            handler: Some(exit_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()),
            argv: [
                ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                    status as intptr_t as usize,
                ),
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
}
unsafe extern "C" fn on_proc_exit(mut proc: *mut Proc) {
    let mut loop_0: *mut Loop = (*proc).loop_0;
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"on_proc_exit\0".as_ptr() as *const ::core::ffi::c_char,
        447 as ::core::ffi::c_int,
        true_0 != 0,
        b"child exited: pid=%d status=%dlu\0".as_ptr() as *const ::core::ffi::c_char,
        (*proc).pid,
        (*proc).status,
    );
    let mut queue: *mut MultiQueue = if !(*proc).events.is_null() {
        (*proc).events
    } else {
        (*loop_0).events
    };
    if !queue.is_null() {
        multiqueue_put_event(
            queue,
            Event {
                handler: Some(
                    proc_close_handles as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    proc as *mut ::core::ffi::c_void,
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
        let mut argv: [*mut ::core::ffi::c_void; 1] = [proc as *mut ::core::ffi::c_void];
        proc_close_handles(&raw mut argv as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn on_proc_stream_close(
    mut _stream: *mut Stream,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut proc: *mut Proc = data as *mut Proc;
    decref(proc);
}
#[inline]
unsafe extern "C" fn proc_get_exepath(mut proc: *mut Proc) -> *const ::core::ffi::c_char {
    return if !(*proc).exepath.is_null() {
        (*proc).exepath
    } else {
        *(*proc).argv.offset(0 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char
    };
}
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SIGKILL: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
