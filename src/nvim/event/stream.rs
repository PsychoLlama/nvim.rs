pub use crate::src::nvim::types::{
    Loop, LuaRef, MultiQueue, Proc, ProcType, RStream, ScopeType, Stream, VarLockStatus,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, dict_T,
    dictvar_S, hash_T, hashitem_T, hashtab_T, int64_t, internal_proc_cb, loop_0,
    loop_0_children as C2Rust_Unnamed_11, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, size_t, ssize_t, stream, stream_close_cb,
    stream_read_cb, stream_uv as C2Rust_Unnamed_12, stream_write_cb, uint64_t, uint8_t, uv__io_cb,
    uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_run_mode, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, QUEUE,
};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn uv_loop_init(loop_0: *mut uv_loop_t) -> ::core::ffi::c_int;
    fn uv_loop_close(loop_0: *mut uv_loop_t) -> ::core::ffi::c_int;
    fn uv_run(_: *mut uv_loop_t, mode: uv_run_mode) -> ::core::ffi::c_int;
    fn uv_close(handle: *mut uv_handle_t, close_cb_0: uv_close_cb);
    fn uv_stream_get_write_queue_size(stream: *const uv_stream_t) -> size_t;
    fn uv_stream_set_blocking(
        handle: *mut uv_stream_t,
        blocking: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_is_closing(handle: *const uv_handle_t) -> ::core::ffi::c_int;
    fn uv_guess_handle(file: uv_file) -> uv_handle_type;
    fn uv_pipe_init(
        _: *mut uv_loop_t,
        handle: *mut uv_pipe_t,
        ipc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_pipe_open(_: *mut uv_pipe_t, file: uv_file) -> ::core::ffi::c_int;
    fn uv_idle_init(_: *mut uv_loop_t, idle: *mut uv_idle_t) -> ::core::ffi::c_int;
    fn logmsg(
        log_level: ::core::ffi::c_int,
        context: *const ::core::ffi::c_char,
        func_name: *const ::core::ffi::c_char,
        line_num: ::core::ffi::c_int,
        eol: bool,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> bool;
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
pub unsafe extern "C" fn stream_set_blocking(
    mut fd: ::core::ffi::c_int,
    mut blocking: bool,
) -> ::core::ffi::c_int {
    let mut loop_0: uv_loop_t = uv_loop_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        active_handles: 0,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        active_reqs: C2Rust_Unnamed_4 {
            unused: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        internal_fields: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        stop_flag: 0,
        flags: 0,
        backend_fd: 0,
        pending_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watcher_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watchers: ::core::ptr::null_mut::<*mut uv__io_t>(),
        nwatchers: 0,
        nfds: 0,
        wq: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        wq_mutex: pthread_mutex_t {
            __data: __pthread_mutex_s {
                __lock: 0,
                __count: 0,
                __owner: 0,
                __nusers: 0,
                __kind: 0,
                __spins: 0,
                __elision: 0,
                __list: __pthread_list_t {
                    __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                    __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
                },
            },
        },
        wq_async: uv_async_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_3 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            async_cb: None,
            queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pending: 0,
        },
        cloexec_lock: pthread_rwlock_t {
            __data: __pthread_rwlock_arch_t {
                __readers: 0,
                __writers: 0,
                __wrphase_futex: 0,
                __writers_futex: 0,
                __pad3: 0,
                __pad4: 0,
                __cur_writer: 0,
                __shared: 0,
                __rwelision: 0,
                __pad1: [0; 7],
                __pad2: 0,
                __flags: 0,
            },
        },
        closing_handles: ::core::ptr::null_mut::<uv_handle_t>(),
        process_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        prepare_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        check_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        idle_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_unused: None,
        async_io_watcher: uv__io_t {
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
        async_wfd: 0,
        timer_heap: C2Rust_Unnamed_2 {
            min: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            nelts: 0,
        },
        timer_counter: 0,
        time: 0,
        signal_pipefd: [0; 2],
        signal_io_watcher: uv__io_t {
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
        child_watcher: uv_signal_t {
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: C2Rust_Unnamed_1 { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            signal_cb: None,
            signum: 0,
            tree_entry: C2Rust_Unnamed {
                rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_color: 0,
            },
            caught_signals: 0,
            dispatched_signals: 0,
        },
        emfile_fd: 0,
        inotify_read_watcher: uv__io_t {
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
        inotify_watchers: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        inotify_fd: 0,
    };
    let mut stream: uv_pipe_t = uv_pipe_t {
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
    };
    uv_loop_init(&raw mut loop_0);
    uv_pipe_init(&raw mut loop_0, &raw mut stream, 0 as ::core::ffi::c_int);
    uv_pipe_open(&raw mut stream, fd as uv_file);
    let mut retval: ::core::ffi::c_int = uv_stream_set_blocking(
        &raw mut stream as *mut uv_stream_t,
        blocking as ::core::ffi::c_int,
    );
    uv_close(&raw mut stream as *mut uv_handle_t, None);
    uv_run(&raw mut loop_0, UV_RUN_NOWAIT);
    uv_loop_close(&raw mut loop_0);
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn stream_init(
    mut loop_0: *mut Loop,
    mut stream: *mut Stream,
    mut fd: ::core::ffi::c_int,
    mut uvstream: *mut uv_stream_t,
) {
    '_c2rust_label: {
        if (if uvstream.is_null() {
            (fd >= 0 as ::core::ffi::c_int && !loop_0.is_null()) as ::core::ffi::c_int
        } else {
            (fd < 0 as ::core::ffi::c_int && loop_0.is_null()) as ::core::ffi::c_int
        }) != 0
        {
        } else {
            __assert_fail(
                b"uvstream == NULL ? fd >= 0 && loop != NULL : fd < 0 && loop == NULL\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/event/stream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                47 as ::core::ffi::c_uint,
                b"void stream_init(Loop *, Stream *, int, uv_stream_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*stream).uvstream = uvstream;
    if fd >= 0 as ::core::ffi::c_int {
        let mut type_0: uv_handle_type = uv_guess_handle(fd as uv_file);
        (*stream).fd = fd as uv_file;
        if type_0 as ::core::ffi::c_uint == UV_FILE as ::core::ffi::c_int as ::core::ffi::c_uint {
            uv_idle_init(&raw mut (*loop_0).uv, &raw mut (*stream).uv.idle);
            (*stream).uv.idle.data = stream as *mut ::core::ffi::c_void;
        } else {
            '_c2rust_label_0: {
                if type_0 as ::core::ffi::c_uint
                    == UV_NAMED_PIPE as ::core::ffi::c_int as ::core::ffi::c_uint
                    || type_0 as ::core::ffi::c_uint
                        == UV_TTY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"type == UV_NAMED_PIPE || type == UV_TTY\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/event/stream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        72 as ::core::ffi::c_uint,
                        b"void stream_init(Loop *, Stream *, int, uv_stream_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            uv_pipe_init(
                &raw mut (*loop_0).uv,
                &raw mut (*stream).uv.pipe,
                0 as ::core::ffi::c_int,
            );
            uv_pipe_open(&raw mut (*stream).uv.pipe, fd as uv_file);
            (*stream).uvstream = &raw mut (*stream).uv.pipe as *mut uv_stream_t;
        }
    }
    if !(*stream).uvstream.is_null() {
        (*(*stream).uvstream).data = stream as *mut ::core::ffi::c_void;
    }
    (*stream).fpos = 0 as int64_t;
    (*stream).internal_data = NULL;
    (*stream).curmem = 0 as size_t;
    (*stream).maxmem = 0 as size_t;
    (*stream).pending_reqs = 0 as size_t;
    (*stream).write_cb = None;
    (*stream).close_cb = None;
    (*stream).internal_close_cb = None;
    (*stream).closed = false_0 != 0;
    (*stream).events = ::core::ptr::null_mut::<MultiQueue>();
}
#[no_mangle]
pub unsafe extern "C" fn stream_may_close(mut stream: *mut Stream) {
    if (*stream).closed {
        return;
    }
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"stream_may_close\0".as_ptr() as *const ::core::ffi::c_char,
        101 as ::core::ffi::c_int,
        true_0 != 0,
        b"closing Stream: %p\0".as_ptr() as *const ::core::ffi::c_char,
        stream as *mut ::core::ffi::c_void,
    );
    (*stream).closed = true_0 != 0;
    if (*stream).pending_reqs == 0 {
        stream_close_handle(stream);
    }
}
#[no_mangle]
pub unsafe extern "C" fn stream_close_handle(mut stream: *mut Stream) {
    let mut handle: *mut uv_handle_t = ::core::ptr::null_mut::<uv_handle_t>();
    if !(*stream).uvstream.is_null() {
        if uv_stream_get_write_queue_size((*stream).uvstream) > 0 as size_t {
            logmsg(
                LOGLVL_WRN,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"stream_close_handle\0".as_ptr() as *const ::core::ffi::c_char,
                124 as ::core::ffi::c_int,
                true_0 != 0,
                b"closed Stream (%p) with %zu unwritten bytes\0".as_ptr()
                    as *const ::core::ffi::c_char,
                stream as *mut ::core::ffi::c_void,
                uv_stream_get_write_queue_size((*stream).uvstream),
            );
        }
        handle = (*stream).uvstream as *mut uv_handle_t;
    } else {
        handle = &raw mut (*stream).uv.idle as *mut uv_handle_t;
    }
    '_c2rust_label: {
        if !handle.is_null() {
        } else {
            __assert_fail(
                b"handle != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/stream.rs\0".as_ptr() as *const ::core::ffi::c_char,
                131 as ::core::ffi::c_uint,
                b"void stream_close_handle(Stream *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if (*stream).before_close_cb.is_some() {
        (*stream).pending_reqs = (*stream).pending_reqs.wrapping_add(1);
        (*stream)
            .before_close_cb
            .expect("non-null function pointer")(stream, (*stream).close_cb_data);
        (*stream).pending_reqs = (*stream).pending_reqs.wrapping_sub(1);
    }
    if uv_is_closing(handle) == 0 {
        uv_close(
            handle,
            Some(close_cb as unsafe extern "C" fn(*mut uv_handle_t) -> ()),
        );
    }
}
unsafe extern "C" fn close_cb(mut handle: *mut uv_handle_t) {
    let mut stream: *mut Stream = (*handle).data as *mut Stream;
    if !stream.is_null() && (*stream).close_cb.is_some() {
        (*stream).close_cb.expect("non-null function pointer")(stream, (*stream).close_cb_data);
    }
    if !stream.is_null() && (*stream).internal_close_cb.is_some() {
        (*stream)
            .internal_close_cb
            .expect("non-null function pointer")(stream, (*stream).internal_data);
    }
}
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
