use crate::src::nvim::autocmd::{
    EVENT_CURSORHOLD, EVENT_CURSORHOLDI, apply_autocmds, trigger_cursorhold,
};
use crate::src::nvim::event::libuv::uv_guess_handle;
use crate::src::nvim::event::r#loop::{loop_poll_events, process_events_until};
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_process_events, multiqueue_put_event,
};
use crate::src::nvim::event::rstream::{
    rstream_init_fd, rstream_may_close, rstream_start, rstream_stop,
};
use crate::src::nvim::getchar::{before_blocking, typebuf_changed};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{K_SPECIAL, trans_special};
use crate::src::nvim::log::{LOGLVL_DBG, logmsg};
use crate::src::nvim::main::{
    Columns, Rows, State, ch_before_blocking_events, ctrl_c_interrupts, curbuf, current_ui,
    did_cursorhold, do_profiling, getout, got_int, main_loop, mapped_ctrl_c, mouse_col, mouse_grid,
    mouse_row, p_mouset, p_ut, preserve_exit, silent_mode, typebuf_was_filled, used_stdin,
};
use crate::src::nvim::os::libc::{__assert_fail, gettext, memcpy, memmove, sscanf};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::profile::{prof_input_end, prof_input_start};
use crate::src::nvim::state::{MODE_INSERT, get_real_state};
use crate::src::nvim::types::libc::STDIN_FILENO;
use crate::src::nvim::types::{
    Event, MultiQueue, ProcType, RStream, String_0, TriState, event_T, int64_t, kFalse, kNone,
    kTrue, key_extra, rstream, size_t, ssize_t, stream, stream_uv as C2Rust_Unnamed_25, uint8_t,
    uint64_t, uv__io_t, uv__queue, uv_buf_t, uv_connect_t, uv_file, uv_handle_t, uv_handle_type,
    uv_loop_t, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_shutdown_t, uv_stream_t,
};
pub const UV_TTY: uv_handle_type = 14;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const kProcTypePty: ProcType = 1;
pub const KE_EVENT: key_extra = 102;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_X2RELEASE: key_extra = 94;
pub const KE_X2MOUSE: key_extra = 92;
pub const KE_X1MOUSE: key_extra = 89;
pub const KE_MOUSERIGHT: key_extra = 78;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_MIDDLEMOUSE: key_extra = 47;
pub const KE_LEFTMOUSE: key_extra = 44;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FSK_KEYCODE: C2Rust_Unnamed_27 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const KE_FILLER: ::core::ffi::c_int = 'X' as ::core::ffi::c_int;
pub const MOD_MASK_CTRL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MOD_MASK_2CLICK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MOD_MASK_3CLICK: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MOD_MASK_4CLICK: ::core::ffi::c_int = 0x60 as ::core::ffi::c_int;
pub const MAX_KEY_CODE_LEN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const READ_BUFFER_SIZE: ::core::ffi::c_int = 0xfff as ::core::ffi::c_int;
pub const INPUT_BUFFER_SIZE: ::core::ffi::c_int =
    READ_BUFFER_SIZE * 4 as ::core::ffi::c_int + MAX_KEY_CODE_LEN;
static read_stream: GlobalCell<RStream> = GlobalCell::new(rstream {
    s: stream {
        closed: true_0 != 0,
        uv: C2Rust_Unnamed_25 {
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
});
static input_buffer: GlobalCell<[::core::ffi::c_char; 16386]> = GlobalCell::new([0; 16386]);
static input_read_pos: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new((input_buffer.as_raw() as *const _) as *mut ::core::ffi::c_char);
static input_write_pos: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new((input_buffer.as_raw() as *const _) as *mut ::core::ffi::c_char);
static input_eof: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static blocking: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static cursorhold_time: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static cursorhold_tb_change_cnt: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn input_start() {
    if !(*read_stream.ptr()).s.closed {
        return;
    }
    used_stdin.set(true_0 != 0);
    rstream_init_fd(main_loop.ptr(), read_stream.ptr(), STDIN_FILENO);
    rstream_start(
        read_stream.ptr(),
        Some(
            input_read_cb
                as unsafe extern "C" fn(
                    *mut RStream,
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                    bool,
                ) -> size_t,
        ),
        NULL,
    );
}
pub unsafe extern "C" fn input_stop() {
    if (*read_stream.ptr()).s.closed {
        return;
    }
    rstream_stop(read_stream.ptr());
    rstream_may_close(read_stream.ptr());
}
unsafe extern "C" fn cursorhold_event(mut _argv: *mut *mut ::core::ffi::c_void) {
    let mut event: event_T = (if State.get() & MODE_INSERT != 0 {
        EVENT_CURSORHOLDI as ::core::ffi::c_int
    } else {
        EVENT_CURSORHOLD as ::core::ffi::c_int
    }) as event_T;
    apply_autocmds(
        event,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    did_cursorhold.set(true_0 != 0);
}
unsafe extern "C" fn create_cursorhold_event(mut events_enabled: bool) {
    '_c2rust_label: {
        if !events_enabled || multiqueue_empty((*main_loop.ptr()).events) as ::core::ffi::c_int != 0
        {
        } else {
            __assert_fail(
                b"!events_enabled || multiqueue_empty(main_loop.events)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                83 as ::core::ffi::c_uint,
                b"void create_cursorhold_event(_Bool)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        Event {
            handler: Some(
                cursorhold_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
unsafe extern "C" fn reset_cursorhold_wait(mut tb_change_cnt: ::core::ffi::c_int) {
    cursorhold_time.set(0 as ::core::ffi::c_int);
    cursorhold_tb_change_cnt.set(tb_change_cnt);
}
pub unsafe extern "C" fn input_get(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
    mut ms: ::core::ffi::c_int,
    mut tb_change_cnt: ::core::ffi::c_int,
    mut events: *mut MultiQueue,
) -> ::core::ffi::c_int {
    if tb_change_cnt != cursorhold_tb_change_cnt.get() {
        reset_cursorhold_wait(tb_change_cnt);
    }
    if maxlen != 0 && input_available() != 0 {
        reset_cursorhold_wait(tb_change_cnt);
        '_c2rust_label: {
            if maxlen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"maxlen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    129 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut to_read: size_t = if (maxlen as size_t) < input_available() {
            maxlen as size_t
        } else {
            input_available()
        };
        memcpy(
            buf as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            to_read,
        );
        input_read_pos.set((*input_read_pos.ptr()).offset(to_read as isize));
        '_c2rust_label_0: {
            if to_read <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"to_read <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    129 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return to_read as ::core::ffi::c_int;
    }
    if (mapped_ctrl_c.get() | (*curbuf.get()).b_mapped_ctrl_c) & get_real_state() != 0 {
        ctrl_c_interrupts.set(false_0 != 0);
    }
    let mut result: TriState = kFalse;
    if ms >= 0 as ::core::ffi::c_int {
        result = inbuf_poll(ms, events);
        if result as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        let mut wait_start: uint64_t = os_hrtime();
        cursorhold_time.set(
            if cursorhold_time.get() < p_ut.get() as ::core::ffi::c_int {
                cursorhold_time.get()
            } else {
                p_ut.get() as ::core::ffi::c_int
            },
        );
        result = inbuf_poll(
            p_ut.get() as ::core::ffi::c_int - cursorhold_time.get(),
            events,
        );
        if result as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
            if (*read_stream.ptr()).s.closed as ::core::ffi::c_int != 0
                && silent_mode.get() as ::core::ffi::c_int != 0
            {
                read_error_exit();
            }
            reset_cursorhold_wait(tb_change_cnt);
            if trigger_cursorhold() as ::core::ffi::c_int != 0 && !typebuf_changed(tb_change_cnt) {
                create_cursorhold_event(events == (*main_loop.ptr()).events);
            } else {
                before_blocking();
                result = inbuf_poll(-1 as ::core::ffi::c_int, events);
            }
        } else {
            (*cursorhold_time.ptr()) += os_hrtime()
                .wrapping_sub(wait_start)
                .wrapping_div(1000000 as uint64_t)
                as ::core::ffi::c_int;
        }
    }
    ctrl_c_interrupts.set(true_0 != 0);
    if typebuf_changed(tb_change_cnt) {
        return 0 as ::core::ffi::c_int;
    }
    if maxlen != 0 && input_available() != 0 {
        reset_cursorhold_wait(tb_change_cnt);
        '_c2rust_label_1: {
            if maxlen >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"maxlen >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    168 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut to_read_0: size_t = if (maxlen as size_t) < input_available() {
            maxlen as size_t
        } else {
            input_available()
        };
        memcpy(
            buf as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            to_read_0,
        );
        input_read_pos.set((*input_read_pos.ptr()).offset(to_read_0 as isize));
        '_c2rust_label_2: {
            if to_read_0 <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"to_read <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    168 as ::core::ffi::c_uint,
                    b"int input_get(uint8_t *, int, int, int, MultiQueue *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return to_read_0 as ::core::ffi::c_int;
    }
    if maxlen != 0 && pending_events(events) as ::core::ffi::c_int != 0 {
        return push_event_key(buf, maxlen);
    }
    if result as ::core::ffi::c_int == kNone as ::core::ffi::c_int && ms != 0 as ::core::ffi::c_int
    {
        read_error_exit();
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_char_avail() -> bool {
    return inbuf_poll(
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<MultiQueue>(),
    ) as ::core::ffi::c_int
        == kTrue as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_breakcheck() {
    if got_int.get() {
        return;
    }
    loop_poll_events(main_loop.ptr(), 0 as int64_t);
}
pub const BREAKCHECK_SKIP: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
static breakcheck_count: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub unsafe extern "C" fn line_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
pub unsafe extern "C" fn fast_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP * 10 as ::core::ffi::c_int {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
pub unsafe extern "C" fn veryfast_breakcheck() {
    (*breakcheck_count.ptr()) += 1;
    if breakcheck_count.get() >= BREAKCHECK_SKIP * 100 as ::core::ffi::c_int {
        breakcheck_count.set(0 as ::core::ffi::c_int);
        os_breakcheck();
    }
}
pub unsafe extern "C" fn os_isatty(mut fd: ::core::ffi::c_int) -> bool {
    return uv_guess_handle(fd as uv_file) as ::core::ffi::c_uint
        == UV_TTY as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub unsafe extern "C" fn input_available() -> size_t {
    return (*input_write_pos.ptr()).offset_from(input_read_pos.get()) as size_t;
}
unsafe extern "C" fn input_space() -> size_t {
    return (input_buffer.ptr() as *mut ::core::ffi::c_char)
        .offset(INPUT_BUFFER_SIZE as isize)
        .offset_from(input_write_pos.get()) as size_t;
}
pub unsafe extern "C" fn input_enqueue_raw(mut data: *const ::core::ffi::c_char, mut size: size_t) {
    if input_read_pos.get() > input_buffer.ptr() as *mut ::core::ffi::c_char {
        let mut available: size_t = input_available();
        memmove(
            input_buffer.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            input_read_pos.get() as *const ::core::ffi::c_void,
            available,
        );
        input_read_pos.set(input_buffer.ptr() as *mut ::core::ffi::c_char);
        input_write_pos
            .set((input_buffer.ptr() as *mut ::core::ffi::c_char).offset(available as isize));
    }
    let mut to_write: size_t = if size < input_space() {
        size
    } else {
        input_space()
    };
    memcpy(
        input_write_pos.get() as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        to_write,
    );
    input_write_pos.set((*input_write_pos.ptr()).offset(to_write as isize));
}
pub unsafe extern "C" fn input_enqueue(mut chan_id: uint64_t, mut keys: String_0) -> size_t {
    current_ui.set(chan_id);
    let mut ptr: *const ::core::ffi::c_char = keys.data;
    let mut end: *const ::core::ffi::c_char = ptr.offset(keys.size as isize);
    while input_space() >= 19 as size_t && ptr < end {
        let mut buf: [uint8_t; 19] = [
            0 as uint8_t,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut new_size: ::core::ffi::c_uint = trans_special(
            &raw mut ptr,
            end.offset_from(ptr) as size_t,
            &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
            FSK_KEYCODE as ::core::ffi::c_int,
            true_0 != 0,
            ::core::ptr::null_mut::<bool>(),
        );
        if new_size > 0 as ::core::ffi::c_uint {
            new_size = handle_mouse_event(&raw mut ptr, &raw mut buf as *mut uint8_t, new_size);
            if new_size > 0 as ::core::ffi::c_uint {
                input_enqueue_raw(
                    &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
                    new_size as size_t,
                );
            }
        } else if *ptr as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            let mut old_ptr: *const ::core::ffi::c_char = ptr;
            loop {
                ptr = ptr.offset(1);
                if !(ptr < end && *ptr as ::core::ffi::c_int != '>' as ::core::ffi::c_int) {
                    break;
                }
            }
            if *ptr as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
                ptr = old_ptr;
                break;
            } else {
                ptr = ptr.offset(1);
            }
        } else {
            if *ptr as uint8_t as ::core::ffi::c_int == K_SPECIAL {
                let mut c2rust_lvalue: uint8_t = K_SPECIAL as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
                let mut c2rust_lvalue_0: uint8_t = KS_SPECIAL as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue_0 as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
                let mut c2rust_lvalue_1: uint8_t = KE_FILLER as uint8_t;
                input_enqueue_raw(
                    &raw mut c2rust_lvalue_1 as *mut ::core::ffi::c_char,
                    1 as size_t,
                );
            } else {
                input_enqueue_raw(ptr, 1 as size_t);
            }
            ptr = ptr.offset(1);
        }
    }
    let mut rv: size_t = ptr.offset_from(keys.data) as size_t;
    process_ctrl_c();
    return rv;
}
unsafe extern "C" fn check_multiclick(
    mut code: ::core::ffi::c_int,
    mut grid: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut skip_event: *mut bool,
) -> uint8_t {
    static orig_num_clicks: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_code: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_grid: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_col: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_row: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    static orig_mouse_time: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
    if code >= KE_MOUSEDOWN as ::core::ffi::c_int && code <= KE_MOUSERIGHT as ::core::ffi::c_int {
        return 0 as uint8_t;
    }
    let mut no_move: bool =
        orig_mouse_grid.get() == grid && orig_mouse_col.get() == col && orig_mouse_row.get() == row;
    if code == KE_MOUSEMOVE as ::core::ffi::c_int {
        if no_move {
            *skip_event = true_0 != 0;
            return 0 as uint8_t;
        }
    } else if code == KE_LEFTMOUSE as ::core::ffi::c_int
        || code == KE_RIGHTMOUSE as ::core::ffi::c_int
        || code == KE_MIDDLEMOUSE as ::core::ffi::c_int
        || code == KE_X1MOUSE as ::core::ffi::c_int
        || code == KE_X2MOUSE as ::core::ffi::c_int
    {
        let mut mouse_time: uint64_t = os_hrtime();
        let mut timediff: uint64_t = mouse_time.wrapping_sub(orig_mouse_time.get());
        let mut mouset: uint64_t = (p_mouset.get() as uint64_t).wrapping_mul(1000000 as uint64_t);
        if code == orig_mouse_code.get()
            && no_move as ::core::ffi::c_int != 0
            && timediff < mouset
            && orig_num_clicks.get() != 4 as ::core::ffi::c_int
        {
            (*orig_num_clicks.ptr()) += 1;
        } else {
            orig_num_clicks.set(1 as ::core::ffi::c_int);
        }
        orig_mouse_code.set(code);
        orig_mouse_time.set(mouse_time);
    }
    orig_mouse_grid.set(grid);
    orig_mouse_col.set(col);
    orig_mouse_row.set(row);
    let mut modifiers: uint8_t = 0 as uint8_t;
    if code != KE_MOUSEMOVE as ::core::ffi::c_int {
        if orig_num_clicks.get() == 2 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_2CLICK) as uint8_t;
        } else if orig_num_clicks.get() == 3 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_3CLICK) as uint8_t;
        } else if orig_num_clicks.get() == 4 as ::core::ffi::c_int {
            modifiers = (modifiers as ::core::ffi::c_int | MOD_MASK_4CLICK) as uint8_t;
        }
    }
    return modifiers;
}
unsafe extern "C" fn handle_mouse_event(
    mut ptr: *mut *const ::core::ffi::c_char,
    mut buf: *mut uint8_t,
    mut bufsize: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut mouse_code: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut type_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if bufsize == 3 as ::core::ffi::c_uint {
        mouse_code = *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        type_0 = *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    } else if bufsize == 6 as ::core::ffi::c_uint {
        mouse_code = *buf.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        type_0 = *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
    }
    if type_0 != KS_EXTRA
        || !(mouse_code >= KE_LEFTMOUSE as ::core::ffi::c_int
            && mouse_code <= KE_RIGHTRELEASE as ::core::ffi::c_int
            || mouse_code >= KE_X1MOUSE as ::core::ffi::c_int
                && mouse_code <= KE_X2RELEASE as ::core::ffi::c_int
            || mouse_code >= KE_MOUSEDOWN as ::core::ffi::c_int
                && mouse_code <= KE_MOUSERIGHT as ::core::ffi::c_int
            || mouse_code == KE_MOUSEMOVE as ::core::ffi::c_int)
    {
        return bufsize;
    }
    let mut col: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut advance: ::core::ffi::c_int = 0;
    if sscanf(
        *ptr,
        b"<%d,%d>%n\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut col,
        &raw mut row,
        &raw mut advance,
    ) != EOF
        && advance != 0
    {
        if col >= 0 as ::core::ffi::c_int && row >= 0 as ::core::ffi::c_int {
            if col >= Columns.get() {
                col = Columns.get() - 1 as ::core::ffi::c_int;
            }
            if row >= Rows.get() {
                row = Rows.get() - 1 as ::core::ffi::c_int;
            }
            mouse_grid.set(0 as ::core::ffi::c_int);
            mouse_row.set(row);
            mouse_col.set(col);
        }
        *ptr = (*ptr).offset(advance as isize);
    }
    let mut skip_event: bool = false_0 != 0;
    let mut modifiers: uint8_t = check_multiclick(
        mouse_code,
        mouse_grid.get(),
        mouse_row.get(),
        mouse_col.get(),
        &raw mut skip_event,
    );
    if skip_event {
        return 0 as ::core::ffi::c_uint;
    }
    if modifiers != 0 {
        if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != KS_MODIFIER {
            memcpy(
                buf.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                buf as *const ::core::ffi::c_void,
                3 as size_t,
            );
            *buf.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
            *buf.offset(1 as ::core::ffi::c_int as isize) = KS_MODIFIER as uint8_t;
            *buf.offset(2 as ::core::ffi::c_int as isize) = modifiers;
            bufsize = bufsize.wrapping_add(3 as ::core::ffi::c_uint);
        } else {
            *buf.offset(2 as ::core::ffi::c_int as isize) =
                (*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    | modifiers as ::core::ffi::c_int) as uint8_t;
        }
    }
    return bufsize;
}
pub unsafe extern "C" fn input_enqueue_mouse(
    mut code: ::core::ffi::c_int,
    mut modifier: uint8_t,
    mut grid: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    let mut skip_event: bool = false_0 != 0;
    modifier = (modifier as ::core::ffi::c_int
        | check_multiclick(code, grid, row, col, &raw mut skip_event) as ::core::ffi::c_int)
        as uint8_t;
    if skip_event {
        return;
    }
    let mut buf: [uint8_t; 7] = [0; 7];
    let mut p: *mut uint8_t = &raw mut buf as *mut uint8_t;
    if modifier != 0 {
        *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
        *p.offset(1 as ::core::ffi::c_int as isize) = KS_MODIFIER as uint8_t;
        *p.offset(2 as ::core::ffi::c_int as isize) = modifier;
        p = p.offset(3 as ::core::ffi::c_int as isize);
    }
    *p.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as uint8_t;
    *p.offset(1 as ::core::ffi::c_int as isize) = KS_EXTRA as uint8_t;
    *p.offset(2 as ::core::ffi::c_int as isize) = code as uint8_t;
    mouse_grid.set(grid);
    mouse_row.set(row);
    mouse_col.set(col);
    let mut written: size_t =
        (3 as size_t).wrapping_add(p.offset_from(&raw mut buf as *mut uint8_t) as size_t);
    input_enqueue_raw(
        &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
        written,
    );
}
pub unsafe extern "C" fn input_blocking() -> bool {
    return blocking.get();
}
unsafe extern "C" fn inbuf_poll(
    mut ms: ::core::ffi::c_int,
    mut events: *mut MultiQueue,
) -> TriState {
    if os_input_ready(events) {
        return kTrue;
    }
    if do_profiling.get() == PROF_YES && ms != 0 {
        prof_input_start();
    }
    if (ms == -1 as ::core::ffi::c_int || ms > 0 as ::core::ffi::c_int)
        && events != (*main_loop.ptr()).events
        && !input_eof.get()
    {
        blocking.set(true_0 != 0);
        multiqueue_process_events(ch_before_blocking_events.get());
    }
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"inbuf_poll\0".as_ptr() as *const ::core::ffi::c_char,
        514 as ::core::ffi::c_int,
        true_0 != 0,
        b"blocking... events=%s\0".as_ptr() as *const ::core::ffi::c_char,
        if !events.is_null() {
            b"true\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"false\0".as_ptr() as *const ::core::ffi::c_char
        },
    );
    // Upstream polls with a NULL queue here, so the macro's "drain this queue
    // instead" branch is dead: `events` is only read by `os_input_ready`.
    process_events_until(
        main_loop.ptr(),
        ::core::ptr::null_mut(),
        ms as int64_t,
        || os_input_ready(events) || input_eof.get(),
    );
    blocking.set(false_0 != 0);
    if do_profiling.get() == PROF_YES && ms != 0 {
        prof_input_end();
    }
    if os_input_ready(events) {
        return kTrue;
    }
    return (if input_eof.get() as ::core::ffi::c_int != 0 {
        kNone as ::core::ffi::c_int
    } else {
        kFalse as ::core::ffi::c_int
    }) as TriState;
}
unsafe extern "C" fn input_read_cb(
    mut _stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut c: size_t,
    mut _data: *mut ::core::ffi::c_void,
    mut at_eof: bool,
) -> size_t {
    if at_eof {
        input_eof.set(true_0 != 0);
    }
    '_c2rust_label: {
        if input_space() >= c {
        } else {
            __assert_fail(
                b"input_space() >= c\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                534 as ::core::ffi::c_uint,
                b"size_t input_read_cb(RStream *, const char *, size_t, void *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    input_enqueue_raw(buf, c);
    return c;
}
unsafe extern "C" fn process_ctrl_c() {
    if !ctrl_c_interrupts.get() {
        return;
    }
    let mut available: size_t = input_available();
    let mut i: ssize_t = 0;
    i = available as ssize_t - 1 as ssize_t;
    while i >= 0 as ssize_t {
        let mut c: uint8_t = *(*input_read_pos.ptr()).offset(i as isize) as uint8_t;
        if c as ::core::ffi::c_int == Ctrl_C
            || c as ::core::ffi::c_int == 'C' as ::core::ffi::c_int
                && i >= 3 as ssize_t
                && *(*input_read_pos.ptr()).offset((i - 3 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == K_SPECIAL
                && *(*input_read_pos.ptr()).offset((i - 2 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == KS_MODIFIER
                && *(*input_read_pos.ptr()).offset((i - 1 as ssize_t) as isize) as uint8_t
                    as ::core::ffi::c_int
                    == MOD_MASK_CTRL
        {
            *(*input_read_pos.ptr()).offset(i as isize) = Ctrl_C as ::core::ffi::c_char;
            got_int.set(true_0 != 0);
            break;
        } else {
            i -= 1;
        }
    }
    if got_int.get() as ::core::ffi::c_int != 0 && i > 0 as ssize_t {
        input_read_pos.set((*input_read_pos.ptr()).offset(i as isize));
    }
}
unsafe extern "C" fn push_event_key(
    mut buf: *mut uint8_t,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    static key: GlobalCell<[uint8_t; 3]> = GlobalCell::new([
        K_SPECIAL as uint8_t,
        KS_EXTRA as uint8_t,
        KE_EVENT as ::core::ffi::c_int as uint8_t,
    ]);
    static key_idx: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut buf_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    loop {
        let c2rust_fresh0 = key_idx.get();
        key_idx.set(key_idx.get() + 1);
        let c2rust_fresh1 = buf_idx;
        buf_idx = buf_idx + 1;
        *buf.offset(c2rust_fresh1 as isize) = (*key.ptr())[c2rust_fresh0 as usize];
        (*key_idx.ptr()) %= 3 as ::core::ffi::c_int;
        if !(key_idx.get() > 0 as ::core::ffi::c_int && buf_idx < maxlen) {
            break;
        }
    }
    return buf_idx;
}
pub unsafe extern "C" fn os_input_ready(mut events: *mut MultiQueue) -> bool {
    return typebuf_was_filled.get() as ::core::ffi::c_int != 0
        || input_available() != 0
        || pending_events(events) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn read_error_exit() -> ! {
    if silent_mode.get() {
        getout(0 as ::core::ffi::c_int);
    }
    preserve_exit(gettext(
        b"Nvim: Error reading input, exiting...\n\0".as_ptr() as *const ::core::ffi::c_char,
    ));
}
unsafe extern "C" fn pending_events(mut events: *mut MultiQueue) -> bool {
    return !events.is_null() && !multiqueue_empty(events);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
