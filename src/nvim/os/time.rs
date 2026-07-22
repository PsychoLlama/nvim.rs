use crate::src::nvim::event::libuv::{uv_err_name, uv_hrtime, uv_now, uv_sleep};
use crate::src::nvim::event::multiqueue::{multiqueue_empty, multiqueue_process_events};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{got_int, main_loop};
use crate::src::nvim::memory::{xstrlcat, xstrlcpy};
use crate::src::nvim::os::env::os_getenv_noalloc;
use crate::src::nvim::os::input::os_input_ready;
use crate::src::nvim::os::libc::{gettext, localtime_r, strftime, strncmp, strptime, time, tzset};
pub use crate::src::nvim::types::{
    Loop, LuaRef, MultiQueue, Proc, ProcType, RStream, ScopeType, Stream, Timestamp, VarLockStatus,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __time_t, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T, int32_t, int64_t, internal_proc_cb,
    loop_0, loop_0_children as C2Rust_Unnamed_11, multiqueue, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, size_t, ssize_t, stream, stream_close_cb,
    stream_read_cb, stream_uv as C2Rust_Unnamed_12, stream_write_cb, time_t, tm, uint64_t, uint8_t,
    uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s,
    uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_10, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_7, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_5, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_6, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, QUEUE,
};
extern "C" {
    fn uv_clock_gettime(clock_id: uv_clock_id, ts: *mut uv_timespec64_t) -> ::core::ffi::c_int;
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
pub type uv_clock_id = ::core::ffi::c_uint;
pub const UV_CLOCK_REALTIME: uv_clock_id = 1;
pub const UV_CLOCK_MONOTONIC: uv_clock_id = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_timespec64_t {
    pub tv_sec: int64_t,
    pub tv_nsec: int32_t,
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
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (INT_MAX as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn os_hrtime() -> uint64_t {
    return uv_hrtime();
}
#[no_mangle]
pub unsafe extern "C" fn os_realtime() -> int64_t {
    let mut ts: uv_timespec64_t = uv_timespec64_t {
        tv_sec: 0 as int64_t,
        tv_nsec: 0,
    };
    let mut error_number: ::core::ffi::c_int = 0;
    error_number = uv_clock_gettime(UV_CLOCK_REALTIME, &raw mut ts);
    if error_number != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_realtime\0".as_ptr() as *const ::core::ffi::c_char,
            48 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_clock_gettime failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            error_number,
            uv_err_name(error_number),
        );
        return 0 as int64_t;
    }
    return ts.tv_sec * 1000000000 as int64_t + ts.tv_nsec as int64_t;
}
#[no_mangle]
pub unsafe extern "C" fn os_now() -> uint64_t {
    return uv_now(&raw mut (*main_loop.ptr()).uv);
}
#[no_mangle]
pub unsafe extern "C" fn os_delay(mut ms: uint64_t, mut ignoreinput: bool) {
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_delay\0".as_ptr() as *const ::core::ffi::c_char,
        76 as ::core::ffi::c_int,
        true_0 != 0,
        b"%lu ms\0".as_ptr() as *const ::core::ffi::c_char,
        ms,
    );
    if ms > INT_MAX as uint64_t {
        ms = INT_MAX as uint64_t;
    }
    let mut remaining: int64_t = ms as ::core::ffi::c_int as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while if ignoreinput as ::core::ffi::c_int != 0 {
        got_int.get() as ::core::ffi::c_int
    } else {
        os_input_ready(::core::ptr::null_mut::<MultiQueue>()) as ::core::ffi::c_int
    } == 0
    {
        if !::core::ptr::null_mut::<::core::ffi::c_void>().is_null()
            && !multiqueue_empty(::core::ptr::null_mut::<MultiQueue>())
        {
            multiqueue_process_events(::core::ptr::null_mut::<MultiQueue>());
        } else {
            loop_poll_events(main_loop.ptr(), remaining);
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
}
#[no_mangle]
pub unsafe extern "C" fn os_sleep(mut ms: uint64_t) {
    if ms > UINT_MAX as uint64_t {
        ms = UINT_MAX as uint64_t;
    }
    uv_sleep(ms as ::core::ffi::c_uint);
}
static tz_cache: GlobalCell<[::core::ffi::c_char; 64]> = GlobalCell::new([0; 64]);
#[no_mangle]
pub unsafe extern "C" fn os_localtime_r(mut clock: *const time_t, mut result: *mut tm) -> *mut tm {
    let mut tz: *const ::core::ffi::c_char =
        os_getenv_noalloc(b"TZ\0".as_ptr() as *const ::core::ffi::c_char);
    if tz.is_null() {
        tz = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if strncmp(
        tz_cache.ptr() as *mut ::core::ffi::c_char,
        tz,
        ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(1 as size_t),
    ) != 0 as ::core::ffi::c_int
    {
        tzset();
        xstrlcpy(
            tz_cache.ptr() as *mut ::core::ffi::c_char,
            tz,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
        );
    }
    return localtime_r(clock, result);
}
#[no_mangle]
pub unsafe extern "C" fn os_localtime(mut result: *mut tm) -> *mut tm {
    let mut rawtime: time_t = time(::core::ptr::null_mut::<time_t>());
    return os_localtime_r(&raw mut rawtime, result);
}
#[no_mangle]
pub unsafe extern "C" fn os_ctime_r(
    mut clock: *const time_t,
    mut result: *mut ::core::ffi::c_char,
    mut result_len: size_t,
    mut add_newline: bool,
) -> *mut ::core::ffi::c_char {
    let mut clock_local: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut clock_local_ptr: *mut tm = os_localtime_r(clock, &raw mut clock_local);
    if clock_local_ptr.is_null() {
        xstrlcpy(
            result,
            gettext(b"(Invalid)\0".as_ptr() as *const ::core::ffi::c_char),
            result_len.wrapping_sub(1 as size_t),
        );
    } else if strftime(
        result,
        result_len.wrapping_sub(1 as size_t),
        gettext(b"%a %b %d %H:%M:%S %Y\0".as_ptr() as *const ::core::ffi::c_char),
        clock_local_ptr,
    ) == 0 as size_t
    {
        xstrlcpy(
            result,
            gettext(b"(Invalid)\0".as_ptr() as *const ::core::ffi::c_char),
            result_len.wrapping_sub(1 as size_t),
        );
    }
    if add_newline {
        xstrlcat(
            result,
            b"\n\0".as_ptr() as *const ::core::ffi::c_char,
            result_len,
        );
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn os_ctime(
    mut result: *mut ::core::ffi::c_char,
    mut result_len: size_t,
    mut add_newline: bool,
) -> *mut ::core::ffi::c_char {
    let mut rawtime: time_t = time(::core::ptr::null_mut::<time_t>());
    return os_ctime_r(&raw mut rawtime, result, result_len, add_newline);
}
#[no_mangle]
pub unsafe extern "C" fn os_strptime(
    mut str: *const ::core::ffi::c_char,
    mut format: *const ::core::ffi::c_char,
    mut tm: *mut tm,
) -> *mut ::core::ffi::c_char {
    return strptime(str, format, tm);
}
#[no_mangle]
pub unsafe extern "C" fn os_time() -> Timestamp {
    return time(::core::ptr::null_mut::<time_t>()) as Timestamp;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
