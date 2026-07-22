use crate::src::nvim::eval::typval::tv_dict_to_env;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_disable_stdio_inheritance, uv_pipe_open, uv_signal_start, uv_signal_stop,
    uv_strerror,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::os::fs::os_set_cloexec;
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, _exit, cfsetispeed, cfsetospeed, close, dup, environ, execvp,
    fcntl, forkpty, ioctl, kill, killpg, ptsname, setsid, strerror, waitpid,
};
pub use crate::src::nvim::types::{
    Loop, LuaRef, MultiQueue, Proc, ProcType, PtyProc, RStream, ScopeType, Stream, VarLockStatus,
    __pid_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    cc_t, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T, int64_t, internal_proc_cb, loop_0,
    loop_0_children as C2Rust_Unnamed_11, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, size_t, speed_t, ssize_t, stream,
    stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_12, stream_write_cb, tcflag_t,
    termios, uint16_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
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
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, winsize,
    QUEUE,
};
extern "C" {
    fn signal(__sig: ::core::ffi::c_int, __handler: __sighandler_t) -> __sighandler_t;
    fn poll(
        __fds: *mut pollfd,
        __nfds: nfds_t,
        __timeout: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
}
pub type __sighandler_t = Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>;
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
pub type nfds_t = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pollfd {
    pub fd: ::core::ffi::c_int,
    pub events: ::core::ffi::c_short,
    pub revents: ::core::ffi::c_short,
}
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_NONBLOCK: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const F_GETFL: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const F_SETFL: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
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
            uv: C2Rust_Unnamed_12 {
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
                uv: C2Rust_Unnamed_12 {
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
                uv: C2Rust_Unnamed_12 {
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
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn pty_proc_spawn(mut ptyproc: *mut PtyProc) -> ::core::ffi::c_int {
    static termios_default: GlobalCell<termios> = GlobalCell::new(termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c_ispeed: 0,
        c_ospeed: 0,
    });
    if (*termios_default.ptr()).c_cflag == 0 {
        init_termios(termios_default.ptr());
    }
    let mut status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut proc: *mut Proc = ptyproc as *mut Proc;
    '_c2rust_label: {
        if (*proc).err.s.closed {
        } else {
            __assert_fail(
                b"proc->err.s.closed\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/pty_proc_unix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                181 as ::core::ffi::c_uint,
                b"int pty_proc_spawn(PtyProc *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    uv_signal_start(
        &raw mut (*(*proc).loop_0).children_watcher,
        Some(chld_handler as unsafe extern "C" fn(*mut uv_signal_t, ::core::ffi::c_int) -> ()),
        SIGCHLD,
    );
    (*ptyproc).winsize = winsize {
        ws_row: (*ptyproc).height as ::core::ffi::c_ushort,
        ws_col: (*ptyproc).width as ::core::ffi::c_ushort,
        ws_xpixel: 0 as ::core::ffi::c_ushort,
        ws_ypixel: 0 as ::core::ffi::c_ushort,
    };
    uv_disable_stdio_inheritance();
    let mut master: ::core::ffi::c_int = 0;
    let mut pid: ::core::ffi::c_int = forkpty(
        &raw mut master,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        termios_default.ptr(),
        &raw mut (*ptyproc).winsize,
    );
    if pid < 0 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"pty_proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
            190 as ::core::ffi::c_int,
            true_0 != 0,
            b"forkpty failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        return status;
    } else if pid == 0 as ::core::ffi::c_int {
        init_child(ptyproc);
    }
    let mut master_status_flags: ::core::ffi::c_int = fcntl(master, F_GETFL);
    if master_status_flags == -1 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"pty_proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
            200 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to get master descriptor status flags: %s\0".as_ptr()
                as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
    } else if fcntl(master, F_SETFL, master_status_flags | O_NONBLOCK) == -1 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"pty_proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
            205 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to make master descriptor non-blocking: %s\0".as_ptr()
                as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
    } else if os_set_cloexec(master) == -1 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"pty_proc_spawn\0".as_ptr() as *const ::core::ffi::c_char,
            212 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to set CLOEXEC on ptmx file descriptor\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    } else if !(!(*proc).in_0.closed && {
        status = set_duplicating_descriptor(master, &raw mut (*proc).in_0.uv.pipe);
        status != 0
    }) {
        if !(!(*proc).out.s.closed && {
            status = set_duplicating_descriptor(master, &raw mut (*proc).out.s.uv.pipe);
            status != 0
        }) {
            (*ptyproc).tty_fd = master;
            (*proc).pid = pid;
            return 0 as ::core::ffi::c_int;
        }
    }
    close(master);
    kill(pid as __pid_t, SIGKILL);
    waitpid(
        pid as __pid_t,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        0 as ::core::ffi::c_int,
    );
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_tty_name(
    mut ptyproc: *mut PtyProc,
) -> *const ::core::ffi::c_char {
    return ptsname((*ptyproc).tty_fd);
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_resize(
    mut ptyproc: *mut PtyProc,
    mut width: uint16_t,
    mut height: uint16_t,
) {
    (*ptyproc).winsize = winsize {
        ws_row: height as ::core::ffi::c_ushort,
        ws_col: width as ::core::ffi::c_ushort,
        ws_xpixel: 0 as ::core::ffi::c_ushort,
        ws_ypixel: 0 as ::core::ffi::c_ushort,
    };
    ioctl(
        (*ptyproc).tty_fd,
        TIOCSWINSZ as ::core::ffi::c_ulong,
        &raw mut (*ptyproc).winsize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_resume(mut ptyproc: *mut PtyProc) {
    killpg((*(ptyproc as *mut Proc)).pid as __pid_t, SIGCONT);
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_flush_master(mut ptyproc: *mut PtyProc) {
    let mut pollfd: pollfd = pollfd {
        fd: (*ptyproc).tty_fd,
        events: POLLIN as ::core::ffi::c_short,
        revents: 0,
    };
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        n = poll(&raw mut pollfd, 1 as nfds_t, 0 as ::core::ffi::c_int);
        if !(n < 0 as ::core::ffi::c_int && *__errno_location() == EINTR) {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_close(mut ptyproc: *mut PtyProc) {
    pty_proc_close_master(ptyproc);
    let mut proc: *mut Proc = ptyproc as *mut Proc;
    if (*proc).internal_close_cb.is_some() {
        (*proc)
            .internal_close_cb
            .expect("non-null function pointer")(proc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_close_master(mut ptyproc: *mut PtyProc) {
    if (*ptyproc).tty_fd >= 0 as ::core::ffi::c_int {
        close((*ptyproc).tty_fd);
        (*ptyproc).tty_fd = -1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_teardown(mut loop_0: *mut Loop) {
    uv_signal_stop(&raw mut (*loop_0).children_watcher);
}
unsafe extern "C" fn init_child(mut ptyproc: *mut PtyProc) -> ! {
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    setsid();
    signal(SIGCHLD, SIG_DFL);
    signal(SIGHUP, SIG_DFL);
    signal(SIGINT, SIG_DFL);
    signal(SIGQUIT, SIG_DFL);
    signal(SIGTERM, SIG_DFL);
    signal(SIGALRM, SIG_DFL);
    let mut proc: *mut Proc = ptyproc as *mut Proc;
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*proc).cwd.is_null() && {
        err = uv_chdir((*proc).cwd);
        err != 0 as ::core::ffi::c_int
    } {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"init_child\0".as_ptr() as *const ::core::ffi::c_char,
            318 as ::core::ffi::c_int,
            true_0 != 0,
            b"chdir(%s) failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
            (*proc).cwd,
            uv_strerror(err),
        );
        _exit(122 as ::core::ffi::c_int);
    }
    let mut prog: *const ::core::ffi::c_char = proc_get_exepath(proc);
    '_c2rust_label: {
        if !(*proc).env.is_null() {
        } else {
            __assert_fail(
                b"proc->env\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/pty_proc_unix.rs\0".as_ptr() as *const ::core::ffi::c_char,
                324 as ::core::ffi::c_uint,
                b"void init_child(PtyProc *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    environ = tv_dict_to_env((*proc).env);
    execvp(prog, (*proc).argv as *const *mut ::core::ffi::c_char);
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"init_child\0".as_ptr() as *const ::core::ffi::c_char,
        327 as ::core::ffi::c_int,
        true_0 != 0,
        b"execvp(%s) failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
        prog,
        strerror(*__errno_location()),
    );
    _exit(122 as ::core::ffi::c_int);
}
unsafe extern "C" fn init_termios(mut termios: *mut termios) {
    (*termios).c_iflag = (ICRNL | IXON) as tcflag_t;
    (*termios).c_oflag = (OPOST | ONLCR) as tcflag_t;
    (*termios).c_oflag |= TAB0 as tcflag_t;
    (*termios).c_cflag = (CS8 | CREAD) as tcflag_t;
    (*termios).c_lflag = (ISIG | ICANON | IEXTEN | ECHO | ECHOE | ECHOK) as tcflag_t;
    cfsetispeed(termios, 38400 as speed_t);
    cfsetospeed(termios, 38400 as speed_t);
    (*termios).c_iflag |= IUTF8 as tcflag_t;
    (*termios).c_oflag |= NL0 as tcflag_t;
    (*termios).c_oflag |= CR0 as tcflag_t;
    (*termios).c_oflag |= BS0 as tcflag_t;
    (*termios).c_oflag |= VT0 as tcflag_t;
    (*termios).c_oflag |= FF0 as tcflag_t;
    (*termios).c_lflag |= ECHOCTL as tcflag_t;
    (*termios).c_lflag |= ECHOKE as tcflag_t;
    (*termios).c_cc[VINTR as usize] =
        (0x1f as ::core::ffi::c_int & 'C' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VQUIT as usize] =
        (0x1f as ::core::ffi::c_int & '\\' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VERASE as usize] = 0x7f as cc_t;
    (*termios).c_cc[VKILL as usize] =
        (0x1f as ::core::ffi::c_int & 'U' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VEOF as usize] =
        (0x1f as ::core::ffi::c_int & 'D' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VEOL as usize] = _POSIX_VDISABLE as cc_t;
    (*termios).c_cc[VEOL2 as usize] = _POSIX_VDISABLE as cc_t;
    (*termios).c_cc[VSTART as usize] =
        (0x1f as ::core::ffi::c_int & 'Q' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VSTOP as usize] =
        (0x1f as ::core::ffi::c_int & 'S' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VSUSP as usize] =
        (0x1f as ::core::ffi::c_int & 'Z' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VREPRINT as usize] =
        (0x1f as ::core::ffi::c_int & 'R' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VWERASE as usize] =
        (0x1f as ::core::ffi::c_int & 'W' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VLNEXT as usize] =
        (0x1f as ::core::ffi::c_int & 'V' as ::core::ffi::c_int) as cc_t;
    (*termios).c_cc[VMIN as usize] = 1 as cc_t;
    (*termios).c_cc[VTIME as usize] = 0 as cc_t;
}
unsafe extern "C" fn set_duplicating_descriptor(
    mut fd: ::core::ffi::c_int,
    mut pipe: *mut uv_pipe_t,
) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fd_dup: ::core::ffi::c_int = dup(fd);
    if fd_dup < 0 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"set_duplicating_descriptor\0".as_ptr() as *const ::core::ffi::c_char,
            398 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to dup descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
            fd,
            strerror(*__errno_location()),
        );
        return status;
    }
    if os_set_cloexec(fd_dup) == -1 as ::core::ffi::c_int {
        status = -*__errno_location();
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"set_duplicating_descriptor\0".as_ptr() as *const ::core::ffi::c_char,
            404 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to set CLOEXEC on duplicate fd\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        status = uv_pipe_open(pipe, fd_dup as uv_file);
        if status != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"set_duplicating_descriptor\0".as_ptr() as *const ::core::ffi::c_char,
                411 as ::core::ffi::c_int,
                true_0 != 0,
                b"Failed to set pipe to descriptor %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
                fd_dup,
                uv_strerror(status),
            );
        } else {
            return status;
        }
    }
    close(fd_dup);
    return status;
}
unsafe extern "C" fn chld_handler(mut handle: *mut uv_signal_t, mut _signum: ::core::ffi::c_int) {
    let mut stat: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pid: ::core::ffi::c_int = 0;
    let mut loop_0: *mut Loop = (*(*handle).loop_0).data as *mut Loop;
    let mut i: size_t = 0 as size_t;
    while i < (*loop_0).children.size {
        let mut proc: *mut Proc = *(*loop_0).children.items.offset(i as isize);
        loop {
            pid = waitpid(
                (*proc).pid as __pid_t,
                &raw mut stat,
                WNOHANG | WUNTRACED | WCONTINUED,
            ) as ::core::ffi::c_int;
            if !(pid < 0 as ::core::ffi::c_int && *__errno_location() == EINTR) {
                break;
            }
        }
        if pid > 0 as ::core::ffi::c_int {
            if stat & 0xff as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                (*proc).state_cb.expect("non-null function pointer")(
                    proc,
                    true_0 != 0,
                    (*proc).data,
                );
            } else if stat == __W_CONTINUED {
                (*proc).state_cb.expect("non-null function pointer")(
                    proc,
                    false_0 != 0,
                    (*proc).data,
                );
            } else {
                if stat & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    (*proc).status =
                        (stat & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
                } else if ((stat & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_schar as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    > 0 as ::core::ffi::c_int
                {
                    (*proc).status =
                        128 as ::core::ffi::c_int + (stat & 0x7f as ::core::ffi::c_int);
                }
                (*proc).internal_exit_cb.expect("non-null function pointer")(proc);
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn pty_proc_init(
    mut loop_0: *mut Loop,
    mut data: *mut ::core::ffi::c_void,
) -> PtyProc {
    let mut rv: PtyProc = PtyProc {
        proc: proc {
            type_0: kProcTypeUv,
            loop_0: ::core::ptr::null_mut::<Loop>(),
            data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            pid: 0,
            status: 0,
            refcount: 0,
            exit_signal: 0,
            stopped_time: 0,
            cwd: ::core::ptr::null::<::core::ffi::c_char>(),
            argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            exepath: ::core::ptr::null::<::core::ffi::c_char>(),
            env: ::core::ptr::null_mut::<dict_T>(),
            in_0: Stream {
                closed: false,
                uv: C2Rust_Unnamed_12 {
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
            out: RStream {
                s: Stream {
                    closed: false,
                    uv: C2Rust_Unnamed_12 {
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
            err: RStream {
                s: Stream {
                    closed: false,
                    uv: C2Rust_Unnamed_12 {
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
            closed: false,
            detach: false,
            overlapped: false,
            fwd_err: false,
            stdio_noinherit: false,
            events: ::core::ptr::null_mut::<MultiQueue>(),
        },
        width: 0,
        height: 0,
        winsize: winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
        tty_fd: 0,
    };
    rv.proc = proc_init(loop_0, kProcTypePty, data);
    rv.width = 80 as uint16_t;
    rv.height = 24 as uint16_t;
    rv.tty_fd = -1 as ::core::ffi::c_int;
    return rv;
}
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const SIG_DFL: __sighandler_t = None;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SIGTERM: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SIGQUIT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const SIGKILL: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const SIGALRM: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const _POSIX_VDISABLE: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const SIGCONT: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const WNOHANG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WUNTRACED: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const WCONTINUED: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const __W_CONTINUED: ::core::ffi::c_int = 0xffff as ::core::ffi::c_int;
pub const TIOCSWINSZ: ::core::ffi::c_int = 0x5414 as ::core::ffi::c_int;
pub const VINTR: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const VQUIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const VERASE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const VKILL: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const VEOF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VSTART: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const VSTOP: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const VSUSP: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const VEOL: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const VREPRINT: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const VWERASE: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const VLNEXT: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const VEOL2: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const IXON: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const IUTF8: ::core::ffi::c_int = 0o40000 as ::core::ffi::c_int;
pub const CS8: ::core::ffi::c_int = 0o60 as ::core::ffi::c_int;
pub const CREAD: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const OPOST: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ONLCR: ::core::ffi::c_int = 0o4 as ::core::ffi::c_int;
pub const NL0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CR0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TAB0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const BS0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FF0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const VT0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ISIG: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ICANON: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const ECHO: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const ECHOE: ::core::ffi::c_int = 0o20 as ::core::ffi::c_int;
pub const ECHOK: ::core::ffi::c_int = 0o40 as ::core::ffi::c_int;
pub const ECHOCTL: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const ECHOKE: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const IEXTEN: ::core::ffi::c_int = 0o100000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const POLLIN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
