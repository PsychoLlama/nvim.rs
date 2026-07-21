// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
pub type MultiQueue = multiqueue;
pub type Proc = proc;
pub type ProcType = ::core::ffi::c_uint;
pub type PutCallback =
    Option<unsafe extern "C" fn(*mut MultiQueue, *mut ::core::ffi::c_void) -> ()>;
pub type RStream = rstream;
pub type SignalWatcher = signal_watcher;
pub type SocketWatcher = socket_watcher;
pub type Stream = stream;
pub type TimeWatcher = time_watcher;
pub type WBuffer = wbuffer;
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
pub type internal_proc_cb = Option<unsafe extern "C" fn(*mut Proc) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_0 {
    pub uv: uv_loop_t,
    pub events: *mut MultiQueue,
    pub thread_events: *mut MultiQueue,
    pub fast_events: *mut MultiQueue,
    pub children: loop_0_children,
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
pub struct loop_0_children {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Proc,
}
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
pub type proc_exit_cb =
    Option<unsafe extern "C" fn(*mut Proc, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub type proc_state_cb =
    Option<unsafe extern "C" fn(*mut Proc, bool, *mut ::core::ffi::c_void) -> ()>;
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
pub type signal_cb = Option<
    unsafe extern "C" fn(*mut SignalWatcher, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
>;
pub type signal_close_cb =
    Option<unsafe extern "C" fn(*mut SignalWatcher, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct signal_watcher {
    pub uv: uv_signal_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: signal_cb,
    pub close_cb: signal_close_cb,
    pub events: *mut MultiQueue,
}
pub type socket_cb = Option<
    unsafe extern "C" fn(*mut SocketWatcher, ::core::ffi::c_int, *mut ::core::ffi::c_void) -> (),
>;
pub type socket_close_cb =
    Option<unsafe extern "C" fn(*mut SocketWatcher, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct socket_watcher {
    pub addr: [::core::ffi::c_char; 256],
    pub uv: socket_watcher_uv,
    pub stream: *mut uv_stream_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: socket_cb,
    pub close_cb: socket_close_cb,
    pub events: *mut MultiQueue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union socket_watcher_uv {
    pub tcp: socket_watcher_uv_tcp,
    pub pipe: socket_watcher_uv_pipe,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct socket_watcher_uv_pipe {
    pub handle: uv_pipe_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct socket_watcher_uv_tcp {
    pub handle: uv_tcp_t,
    pub addrinfo: *mut addrinfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stream {
    pub closed: bool,
    pub uv: stream_uv,
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
pub type stream_close_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void) -> ()>;
pub type stream_read_cb = Option<
    unsafe extern "C" fn(
        *mut RStream,
        *const ::core::ffi::c_char,
        size_t,
        *mut ::core::ffi::c_void,
        bool,
    ) -> size_t,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union stream_uv {
    pub pipe: uv_pipe_t,
    pub tcp: uv_tcp_t,
    pub idle: uv_idle_t,
}
pub type stream_write_cb =
    Option<unsafe extern "C" fn(*mut Stream, *mut ::core::ffi::c_void, ::core::ffi::c_int) -> ()>;
pub type time_cb = Option<unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct time_watcher {
    pub uv: uv_timer_t,
    pub data: *mut ::core::ffi::c_void,
    pub cb: time_cb,
    pub close_cb: time_cb,
    pub events: *mut MultiQueue,
    pub blockable: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbuffer {
    pub size: size_t,
    pub refcount: size_t,
    pub data: *mut ::core::ffi::c_char,
    pub cb: wbuffer_data_finalizer,
}
pub type wbuffer_data_finalizer = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
