use crate::src::nvim::event::libuv::{uv_close, uv_signal_init, uv_signal_start, uv_signal_stop};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
pub use crate::src::nvim::types::{
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    argv_callback, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T, int64_t, internal_proc_cb,
    loop_0, loop_0_children as C2Rust_Unnamed_11, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, signal_cb, signal_close_cb, signal_watcher,
    size_t, ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_12,
    stream_write_cb, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, Event, Loop,
    LuaRef, MultiQueue, Proc, ProcType, RStream, ScopeType, SignalWatcher, Stream, VarLockStatus,
    QUEUE,
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
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe extern "C" fn signal_watcher_init(
    mut loop_0: *mut Loop,
    mut watcher: *mut SignalWatcher,
    mut data: *mut ::core::ffi::c_void,
) {
    uv_signal_init(&raw mut (*loop_0).uv, &raw mut (*watcher).uv);
    (*watcher).uv.data = watcher as *mut ::core::ffi::c_void;
    (*watcher).data = data;
    (*watcher).cb = None;
    (*watcher).events = (*loop_0).fast_events;
}
pub unsafe extern "C" fn signal_watcher_start(
    mut watcher: *mut SignalWatcher,
    mut cb: signal_cb,
    mut signum: ::core::ffi::c_int,
) {
    (*watcher).cb = cb;
    uv_signal_start(
        &raw mut (*watcher).uv,
        Some(signal_watcher_cb as unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()),
        signum,
    );
}
pub unsafe extern "C" fn signal_watcher_stop(mut watcher: *mut SignalWatcher) {
    uv_signal_stop(&raw mut (*watcher).uv);
}
pub unsafe extern "C" fn signal_watcher_close(
    mut watcher: *mut SignalWatcher,
    mut cb: signal_close_cb,
) {
    (*watcher).close_cb = cb;
    uv_close(
        &raw mut (*watcher).uv as *mut uv_handle_t,
        Some(close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
    );
}
unsafe extern "C" fn signal_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut watcher: *mut SignalWatcher =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut SignalWatcher;
    (*watcher).cb.expect("non-null function pointer")(
        watcher,
        (*watcher).uv.signum,
        (*watcher).data,
    );
}
unsafe extern "C" fn signal_watcher_cb(
    mut handle: *mut uv_signal_t,
    mut _signum: ::core::ffi::c_int,
) {
    let mut watcher: *mut SignalWatcher = (*handle).data as *mut SignalWatcher;
    if !(*watcher).events.is_null() {
        multiqueue_put_event(
            (*watcher).events,
            Event {
                handler: Some(
                    signal_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    watcher as *mut ::core::ffi::c_void,
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
        let mut argv: [*mut ::core::ffi::c_void; 1] = [watcher as *mut ::core::ffi::c_void];
        signal_event(&raw mut argv as *mut *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn close_cb(mut handle: *mut uv_handle_t) {
    let mut watcher: *mut SignalWatcher = (*handle).data as *mut SignalWatcher;
    if (*watcher).close_cb.is_some() {
        (*watcher).close_cb.expect("non-null function pointer")(watcher, (*watcher).data);
    }
}
