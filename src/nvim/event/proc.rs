use crate::src::nvim::event::libuv::{
    uv_close, uv_pipe_init, uv_recv_buffer_size, uv_timer_start, uv_timer_stop, uv_unref,
};
use crate::src::nvim::event::libuv_proc::{libuv_proc_close, libuv_proc_spawn};
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_process_events, multiqueue_put_event,
};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::rstream::rstream_may_close;
use crate::src::nvim::event::stream::{stream_init, stream_may_close};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    exiting, got_int, main_loop, os_exit, preserve_exit, ui_client_channel_id,
    ui_client_exit_status,
};
use crate::src::nvim::memory::xrealloc;
use crate::src::nvim::os::libc::{__assert_fail, memmove};
use crate::src::nvim::os::proc::os_proc_tree_kill;
use crate::src::nvim::os::pty_proc_unix::{
    pty_proc_close, pty_proc_close_master, pty_proc_flush_master, pty_proc_spawn, pty_proc_teardown,
};
use crate::src::nvim::os::shell::shell_free_argv;
use crate::src::nvim::os::time::os_hrtime;
pub use crate::src::nvim::types::{
    Event, LibuvProc, Loop, LuaRef, MultiQueue, Proc, ProcType, PtyProc, RStream, ScopeType,
    Stream, VarLockStatus, __gid_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s,
    __pthread_rwlock_arch_t, __uid_t, argv_callback, dict_T, dictvar_S, gid_t, hash_T, hashitem_T,
    hashtab_T, int64_t, internal_proc_cb, intptr_t, loop_0, loop_0_children as C2Rust_Unnamed_13,
    multiqueue, proc, proc_exit_cb, proc_state_cb, pthread_mutex_t, pthread_rwlock_t, queue,
    rstream, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_14, stream_write_cb, uid_t, uint16_t, uint64_t, uint8_t, uv__io_cb,
    uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_11, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed,
    uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_12, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, uv_uid_t,
    winsize, QUEUE,
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
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
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
pub unsafe extern "C" fn proc_close_streams(mut proc: *mut Proc) {
    stream_may_close(&raw mut (*proc).in_0);
    rstream_may_close(&raw mut (*proc).out);
    rstream_may_close(&raw mut (*proc).err);
}
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
