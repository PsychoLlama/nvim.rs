use crate::src::nvim::event::libuv::{
    uv_async_init, uv_async_send, uv_close, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_mutex_destroy, uv_mutex_init, uv_mutex_lock, uv_mutex_unlock, uv_run, uv_signal_init,
    uv_stop, uv_timer_init, uv_timer_start, uv_timer_stop,
};
use crate::src::nvim::event::multiqueue::{
    multiqueue_free, multiqueue_move_events, multiqueue_new, multiqueue_new_child,
    multiqueue_process_events, multiqueue_purge_events, multiqueue_put_event, multiqueue_size,
};
use crate::src::nvim::log::{log_uv_handles, logmsg};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::libc::abort;
use crate::src::nvim::os::time::os_hrtime;
pub use crate::src::nvim::types::{
    Event, Loop, LuaRef, MultiQueue, Proc, ProcType, PutCallback, RStream, ScopeType, Stream,
    VarLockStatus, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s,
    __pthread_rwlock_arch_t, argv_callback, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T,
    int64_t, internal_proc_cb, loop_0, loop_0_children as C2Rust_Unnamed_12, multiqueue, proc,
    proc_exit_cb, proc_state_cb, pthread_mutex_t, pthread_rwlock_t, queue, rstream, size_t,
    ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_13,
    stream_write_cb, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_11, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_8, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_run_mode, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_6, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_7, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_9, uv_timer_s_u as C2Rust_Unnamed_10, uv_timer_t, QUEUE,
};
extern "C" {
    fn uv_walk(loop_0: *mut uv_loop_t, walk_cb: uv_walk_cb, arg: *mut ::core::ffi::c_void);
}
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
pub const UV_RUN_NOWAIT: uv_run_mode = 2;
pub const UV_RUN_ONCE: uv_run_mode = 1;
pub const UV_RUN_DEFAULT: uv_run_mode = 0;
pub type uv_walk_cb =
    Option<unsafe extern "C" fn(*mut uv_handle_t, *mut ::core::ffi::c_void) -> ()>;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn loop_init(mut loop_0: *mut Loop, mut _data: *mut ::core::ffi::c_void) {
    uv_loop_init(&raw mut (*loop_0).uv);
    (*loop_0).recursive = 0 as ::core::ffi::c_int;
    (*loop_0).closing = false_0 != 0;
    (*loop_0).uv.data = loop_0 as *mut ::core::ffi::c_void;
    (*loop_0).children.capacity = 0 as size_t;
    (*loop_0).children.size = (*loop_0).children.capacity;
    (*loop_0).children.items = ::core::ptr::null_mut::<*mut Proc>();
    (*loop_0).events = multiqueue_new(
        Some(loop_on_put as unsafe extern "C" fn(*mut MultiQueue, *mut ::core::ffi::c_void) -> ()),
        loop_0 as *mut ::core::ffi::c_void,
    );
    (*loop_0).fast_events = multiqueue_new_child((*loop_0).events);
    (*loop_0).thread_events = multiqueue_new(None, NULL);
    uv_mutex_init(&raw mut (*loop_0).mutex);
    uv_async_init(
        &raw mut (*loop_0).uv,
        &raw mut (*loop_0).async_0,
        Some(async_cb as unsafe extern "C" fn(*mut uv_async_t) -> ()),
    );
    uv_signal_init(&raw mut (*loop_0).uv, &raw mut (*loop_0).children_watcher);
    uv_timer_init(
        &raw mut (*loop_0).uv,
        &raw mut (*loop_0).children_kill_timer,
    );
    uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*loop_0).poll_timer);
    uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*loop_0).exit_delay_timer);
    (*loop_0).poll_timer.data = xmalloc(::core::mem::size_of::<bool>());
}
unsafe extern "C" fn loop_uv_run(mut loop_0: *mut Loop, mut ms: int64_t) -> bool {
    let c2rust_fresh0 = (*loop_0).recursive;
    (*loop_0).recursive = (*loop_0).recursive + 1;
    if c2rust_fresh0 != 0 {
        abort();
    }
    let mut mode: uv_run_mode = UV_RUN_ONCE;
    let mut timeout_expired: *mut bool = (*loop_0).poll_timer.data as *mut bool;
    *timeout_expired = false_0 != 0;
    if ms > 0 as int64_t {
        uv_timer_start(
            &raw mut (*loop_0).poll_timer,
            Some(timer_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
            ms as uint64_t,
            ms as uint64_t,
        );
    } else if ms == 0 as int64_t {
        mode = UV_RUN_NOWAIT;
    }
    uv_run(&raw mut (*loop_0).uv, mode);
    if ms > 0 as int64_t {
        uv_timer_stop(&raw mut (*loop_0).poll_timer);
    }
    (*loop_0).recursive -= 1;
    return *timeout_expired;
}
#[no_mangle]
pub unsafe extern "C" fn loop_poll_events(mut loop_0: *mut Loop, mut ms: int64_t) -> bool {
    let mut timeout_expired: bool = loop_uv_run(loop_0, ms);
    multiqueue_process_events((*loop_0).fast_events);
    return timeout_expired;
}
#[no_mangle]
pub unsafe extern "C" fn loop_schedule_fast(mut loop_0: *mut Loop, mut event: Event) {
    uv_mutex_lock(&raw mut (*loop_0).mutex);
    multiqueue_put_event((*loop_0).thread_events, event);
    uv_async_send(&raw mut (*loop_0).async_0);
    uv_mutex_unlock(&raw mut (*loop_0).mutex);
}
#[no_mangle]
pub unsafe extern "C" fn loop_schedule_deferred(mut loop_0: *mut Loop, mut event: Event) {
    let mut eventp: *mut Event = xmalloc(::core::mem::size_of::<Event>()) as *mut Event;
    *eventp = event;
    loop_schedule_fast(
        loop_0,
        Event {
            handler: Some(
                loop_deferred_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                loop_0 as *mut ::core::ffi::c_void,
                eventp as *mut ::core::ffi::c_void,
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
unsafe extern "C" fn loop_deferred_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut loop_0: *mut Loop = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Loop;
    let mut eventp: *mut Event = *argv.offset(1 as ::core::ffi::c_int as isize) as *mut Event;
    multiqueue_put_event((*loop_0).events, *eventp);
    xfree(eventp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn loop_on_put(
    mut _queue: *mut MultiQueue,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut loop_0: *mut Loop = data as *mut Loop;
    if (*loop_0).recursive != 0 {
        uv_stop(&raw mut (*loop_0).uv);
    }
}
unsafe extern "C" fn loop_walk_cb(
    mut handle: *mut uv_handle_t,
    mut _arg: *mut ::core::ffi::c_void,
) {
    if uv_is_closing(handle) == 0 {
        uv_close(handle, None);
    }
}
#[no_mangle]
pub unsafe extern "C" fn loop_close(mut loop_0: *mut Loop, mut wait: bool) -> bool {
    let mut rv: bool = true_0 != 0;
    (*loop_0).closing = true_0 != 0;
    uv_mutex_destroy(&raw mut (*loop_0).mutex);
    uv_close(
        &raw mut (*loop_0).children_watcher as *mut uv_handle_t,
        None,
    );
    uv_close(
        &raw mut (*loop_0).children_kill_timer as *mut uv_handle_t,
        None,
    );
    uv_close(
        &raw mut (*loop_0).poll_timer as *mut uv_handle_t,
        Some(timer_close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
    );
    uv_close(
        &raw mut (*loop_0).exit_delay_timer as *mut uv_handle_t,
        None,
    );
    uv_close(&raw mut (*loop_0).async_0 as *mut uv_handle_t, None);
    let mut start: uint64_t = if wait as ::core::ffi::c_int != 0 {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    let mut didstop: bool = false_0 != 0;
    loop {
        uv_run(
            &raw mut (*loop_0).uv,
            (if didstop as ::core::ffi::c_int != 0 {
                UV_RUN_DEFAULT as ::core::ffi::c_int
            } else {
                UV_RUN_NOWAIT as ::core::ffi::c_int
            }) as uv_run_mode,
        );
        if uv_loop_close(&raw mut (*loop_0).uv) != UV_EBUSY as ::core::ffi::c_int || !wait {
            break;
        }
        let mut elapsed_s: uint64_t = os_hrtime()
            .wrapping_sub(start)
            .wrapping_div(1000000000 as uint64_t);
        if elapsed_s >= 2 as uint64_t {
            rv = false_0 != 0;
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"loop_close\0".as_ptr() as *const ::core::ffi::c_char,
                172 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_loop_close() hang?\0".as_ptr() as *const ::core::ffi::c_char,
            );
            log_uv_handles(&raw mut (*loop_0).uv as *mut ::core::ffi::c_void);
            break;
        } else if !didstop {
            uv_stop(&raw mut (*loop_0).uv);
            uv_walk(
                &raw mut (*loop_0).uv,
                Some(
                    loop_walk_cb
                        as unsafe extern "C" fn(*mut uv_handle_t, *mut ::core::ffi::c_void) -> (),
                ),
                NULL,
            );
            didstop = true_0 != 0;
        }
    }
    multiqueue_free((*loop_0).fast_events);
    multiqueue_free((*loop_0).thread_events);
    multiqueue_free((*loop_0).events);
    xfree((*loop_0).children.items as *mut ::core::ffi::c_void);
    (*loop_0).children.capacity = 0 as size_t;
    (*loop_0).children.size = (*loop_0).children.capacity;
    (*loop_0).children.items = ::core::ptr::null_mut::<*mut Proc>();
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn loop_purge(mut loop_0: *mut Loop) {
    uv_mutex_lock(&raw mut (*loop_0).mutex);
    multiqueue_purge_events((*loop_0).thread_events);
    multiqueue_purge_events((*loop_0).fast_events);
    uv_mutex_unlock(&raw mut (*loop_0).mutex);
}
#[no_mangle]
pub unsafe extern "C" fn loop_size(mut loop_0: *mut Loop) -> size_t {
    uv_mutex_lock(&raw mut (*loop_0).mutex);
    let mut rv: size_t = multiqueue_size((*loop_0).thread_events);
    uv_mutex_unlock(&raw mut (*loop_0).mutex);
    return rv;
}
unsafe extern "C" fn async_cb(mut handle: *mut uv_async_t) {
    let mut l: *mut Loop = (*(*handle).loop_0).data as *mut Loop;
    uv_mutex_lock(&raw mut (*l).mutex);
    multiqueue_move_events((*l).fast_events, (*l).thread_events);
    uv_mutex_unlock(&raw mut (*l).mutex);
}
unsafe extern "C" fn timer_cb(mut handle: *mut uv_timer_t) {
    let mut timeout_expired: *mut bool = (*handle).data as *mut bool;
    *timeout_expired = true_0 != 0;
}
unsafe extern "C" fn timer_close_cb(mut handle: *mut uv_handle_t) {
    xfree((*handle).data);
}
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
