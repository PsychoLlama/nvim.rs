use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::cursor_shape::shape_table;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_close, uv_is_closing, uv_loop_close, uv_loop_init, uv_pipe_init, uv_pipe_open,
    uv_run, uv_sleep, uv_strerror, uv_timer_init, uv_timer_start, uv_tty_reset_mode, uv_write,
};
use crate::src::nvim::event::multiqueue::{multiqueue_empty, multiqueue_process_events};
use crate::src::nvim::event::r#loop::{loop_poll_events, loop_purge, loop_size};
use crate::src::nvim::event::signal::{
    signal_watcher_close, signal_watcher_init, signal_watcher_start, signal_watcher_stop,
};
use crate::src::nvim::event::stream::stream_set_blocking;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_cache_clear_if_full, schar_get, schar_get_ascii};
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    main_loop, nvim_testing, stdin_isatty, t_colors, ui_client_channel_id, ui_client_error_exit,
    ui_client_exit_status,
};
use crate::src::nvim::map::mh_put_cstr_t;
use crate::src::nvim::mbyte::{utf_ambiguous_width, utf_char2cells, utf_ptr2char};
use crate::src::nvim::memory::{
    arena_mem_free, strequal, xcalloc, xfree, xrealloc, xstrdup, xstrlcpy,
};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
use crate::src::nvim::os::env::{os_env_exists, os_getenv, os_getenv_noalloc};
use crate::src::nvim::os::input::os_isatty;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, fclose, fopen, fprintf, fwrite, kill, memcmp, memcpy, memset, snprintf,
    sscanf, strchr, strcmp, strlen, strstr, strtol, tcgetattr, vsnprintf,
};
use crate::src::nvim::os::time::os_hrtime;
use crate::src::nvim::strings::kv_do_printf;
use crate::src::nvim::tui::terminfo::{
    terminfo_fmt, terminfo_from_builtin, terminfo_from_database, terminfo_info_msg,
    terminfo_is_bsd_console, terminfo_is_term_family,
};
use crate::src::nvim::tui::ugrid::{
    ugrid_clear, ugrid_clear_chunk, ugrid_goto, ugrid_init, ugrid_resize, ugrid_scroll,
};
pub use crate::src::nvim::types::{
    Arena, ArenaMem, Array, Boolean, CursorShape, Dict, Float, HlAttrs, Integer, KeyEncoding,
    KeyValuePair, LineFlags, Loop, LuaRef, MHPutStatus, MapHash, MultiQueue, Object, ObjectType,
    OptInt, Proc, ProcType, RStream, RgbValue, ScopeType, Set_cstr_t, SignalWatcher, Stream,
    StringBuilder, String_0, TermKey, TermKey_Terminfo_Getstr_Hook, TermMode, TermModeState,
    TerminfoEntry, UCell, UGrid, VarLockStatus, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
    __builtin_va_list, __gnuc_va_list, __off64_t, __off_t, __pid_t, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __va_list_tag, cc_t,
    consumed_blk, cstr_t, cursorentry_T, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T, int16_t,
    int32_t, int64_t, int8_t, internal_proc_cb, key_value_pair, loop_0,
    loop_0_children as C2Rust_Unnamed_14, multiqueue, object, object_data as C2Rust_Unnamed_13,
    proc, proc_exit_cb, proc_state_cb, pthread_mutex_t, pthread_rwlock_t, queue, rstream, sattr_T,
    schar_T, signal_cb, signal_close_cb, signal_watcher, size_t, speed_t, ssize_t, stream,
    stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_15, stream_write_cb, tcflag_t,
    termios, uint32_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_12, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_8, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_run_mode, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_6, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_7, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_10, uv_timer_s_u as C2Rust_Unnamed_11, uv_timer_t,
    uv_write_cb, uv_write_s, uv_write_t, va_list, FILE, QUEUE, TPVAR, _IO_FILE,
};
use crate::src::nvim::ui_client::{ui_client_attach, ui_client_detach, ui_client_set_size};
use ::c2rust_bitfields;
extern "C" {
    fn uv_tty_init(
        _: *mut uv_loop_t,
        _: *mut uv_tty_t,
        fd: uv_file,
        readable: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn uv_tty_set_mode(_: *mut uv_tty_t, mode: uv_tty_mode_t) -> ::core::ffi::c_int;
    fn uv_tty_get_winsize(
        _: *mut uv_tty_t,
        width: *mut ::core::ffi::c_int,
        height: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn arena_memdupz(
        arena: *mut Arena,
        buf: *const ::core::ffi::c_char,
        size: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn arena_strdup(arena: *mut Arena, str: *const ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
    fn tinput_init(input: *mut TermInput, loop_0: *mut Loop, ti: *mut TerminfoEntry);
    fn tinput_destroy(input: *mut TermInput);
    fn tinput_start(input: *mut TermInput);
    fn tinput_stop(input: *mut TermInput);
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_tty_s {
    pub data: *mut ::core::ffi::c_void,
    pub loop_0: *mut uv_loop_t,
    pub type_0: uv_handle_type,
    pub close_cb: uv_close_cb,
    pub handle_queue: uv__queue,
    pub u: C2Rust_Unnamed_9,
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
    pub orig_termios: termios,
    pub mode: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_9 {
    pub fd: ::core::ffi::c_int,
    pub reserved: [*mut ::core::ffi::c_void; 4],
}
pub type uv_tty_t = uv_tty_s;
pub const UV_RUN_NOWAIT: uv_run_mode = 2;
pub const UV_RUN_ONCE: uv_run_mode = 1;
pub const UV_RUN_DEFAULT: uv_run_mode = 0;
pub type uv_tty_mode_t = ::core::ffi::c_uint;
pub const UV_TTY_MODE_RAW_VT: uv_tty_mode_t = 3;
pub const UV_TTY_MODE_IO: uv_tty_mode_t = 2;
pub const UV_TTY_MODE_RAW: uv_tty_mode_t = 1;
pub const UV_TTY_MODE_NORMAL: uv_tty_mode_t = 0;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HL_GLOBAL: C2Rust_Unnamed_16 = 16384;
pub const HL_DEFAULT: C2Rust_Unnamed_16 = 8192;
pub const HL_FG_INDEXED: C2Rust_Unnamed_16 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_16 = 2048;
pub const HL_NOCOMBINE: C2Rust_Unnamed_16 = 1024;
pub const HL_OVERLINE: C2Rust_Unnamed_16 = 131072;
pub const HL_CONCEALED: C2Rust_Unnamed_16 = 65536;
pub const HL_BLINK: C2Rust_Unnamed_16 = 32768;
pub const HL_DIM: C2Rust_Unnamed_16 = 512;
pub const HL_ALTFONT: C2Rust_Unnamed_16 = 256;
pub const HL_STRIKETHROUGH: C2Rust_Unnamed_16 = 128;
pub const HL_STANDOUT: C2Rust_Unnamed_16 = 64;
pub const HL_UNDERDASHED: C2Rust_Unnamed_16 = 40;
pub const HL_UNDERDOTTED: C2Rust_Unnamed_16 = 32;
pub const HL_UNDERDOUBLE: C2Rust_Unnamed_16 = 24;
pub const HL_UNDERCURL: C2Rust_Unnamed_16 = 16;
pub const HL_UNDERLINE: C2Rust_Unnamed_16 = 8;
pub const HL_UNDERLINE_MASK: C2Rust_Unnamed_16 = 56;
pub const HL_ITALIC: C2Rust_Unnamed_16 = 4;
pub const HL_BOLD: C2Rust_Unnamed_16 = 2;
pub const HL_INVERSE: C2Rust_Unnamed_16 = 1;
pub const kMHNewKeyRealloc: MHPutStatus = 2;
pub const kMHNewKeyDidFit: MHPutStatus = 1;
pub const kMHExisting: MHPutStatus = 0;
pub type ModeShape = ::core::ffi::c_uint;
pub const SHAPE_IDX_COUNT: ModeShape = 18;
pub const SHAPE_IDX_TERM: ModeShape = 17;
pub const SHAPE_IDX_SM: ModeShape = 16;
pub const SHAPE_IDX_MOREL: ModeShape = 15;
pub const SHAPE_IDX_MORE: ModeShape = 14;
pub const SHAPE_IDX_VDRAG: ModeShape = 13;
pub const SHAPE_IDX_VSEP: ModeShape = 12;
pub const SHAPE_IDX_SDRAG: ModeShape = 11;
pub const SHAPE_IDX_STATUS: ModeShape = 10;
pub const SHAPE_IDX_CLINE: ModeShape = 9;
pub const SHAPE_IDX_VE: ModeShape = 8;
pub const SHAPE_IDX_O: ModeShape = 7;
pub const SHAPE_IDX_CR: ModeShape = 6;
pub const SHAPE_IDX_CI: ModeShape = 5;
pub const SHAPE_IDX_C: ModeShape = 4;
pub const SHAPE_IDX_R: ModeShape = 3;
pub const SHAPE_IDX_I: ModeShape = 2;
pub const SHAPE_IDX_V: ModeShape = 1;
pub const SHAPE_IDX_N: ModeShape = 0;
pub const SHAPE_VER: CursorShape = 2;
pub const SHAPE_HOR: CursorShape = 1;
pub const SHAPE_BLOCK: CursorShape = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_17 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_17 = 1;
pub type TerminfoDef = ::core::ffi::c_uint;
pub const kTermCount: TerminfoDef = 49;
pub const kTerm_set_underline_style: TerminfoDef = 48;
pub const kTerm_reset_cursor_color: TerminfoDef = 47;
pub const kTerm_set_cursor_color: TerminfoDef = 46;
pub const kTerm_set_rgb_background: TerminfoDef = 45;
pub const kTerm_set_rgb_foreground: TerminfoDef = 44;
pub const kTerm_enter_strikethrough_mode: TerminfoDef = 43;
pub const kTerm_set_cursor_style: TerminfoDef = 42;
pub const kTerm_reset_cursor_style: TerminfoDef = 41;
pub const kTerm_to_status_line: TerminfoDef = 40;
pub const kTerm_set_lr_margin: TerminfoDef = 39;
pub const kTerm_set_attributes: TerminfoDef = 38;
pub const kTerm_set_a_foreground: TerminfoDef = 37;
pub const kTerm_set_a_background: TerminfoDef = 36;
pub const kTerm_parm_up_cursor: TerminfoDef = 35;
pub const kTerm_parm_right_cursor: TerminfoDef = 34;
pub const kTerm_parm_left_cursor: TerminfoDef = 33;
pub const kTerm_parm_insert_line: TerminfoDef = 32;
pub const kTerm_parm_down_cursor: TerminfoDef = 31;
pub const kTerm_parm_delete_line: TerminfoDef = 30;
pub const kTerm_keypad_xmit: TerminfoDef = 29;
pub const kTerm_keypad_local: TerminfoDef = 28;
pub const kTerm_insert_line: TerminfoDef = 27;
pub const kTerm_from_status_line: TerminfoDef = 26;
pub const kTerm_exit_ca_mode: TerminfoDef = 25;
pub const kTerm_exit_attribute_mode: TerminfoDef = 24;
pub const kTerm_erase_chars: TerminfoDef = 23;
pub const kTerm_enter_underline_mode: TerminfoDef = 22;
pub const kTerm_enter_standout_mode: TerminfoDef = 21;
pub const kTerm_enter_secure_mode: TerminfoDef = 20;
pub const kTerm_enter_reverse_mode: TerminfoDef = 19;
pub const kTerm_enter_italics_mode: TerminfoDef = 18;
pub const kTerm_enter_dim_mode: TerminfoDef = 17;
pub const kTerm_enter_ca_mode: TerminfoDef = 16;
pub const kTerm_enter_bold_mode: TerminfoDef = 15;
pub const kTerm_enter_blink_mode: TerminfoDef = 14;
pub const kTerm_delete_line: TerminfoDef = 13;
pub const kTerm_cursor_right: TerminfoDef = 12;
pub const kTerm_cursor_up: TerminfoDef = 11;
pub const kTerm_cursor_normal: TerminfoDef = 10;
pub const kTerm_cursor_home: TerminfoDef = 9;
pub const kTerm_cursor_left: TerminfoDef = 8;
pub const kTerm_cursor_invisible: TerminfoDef = 7;
pub const kTerm_cursor_down: TerminfoDef = 6;
pub const kTerm_cursor_address: TerminfoDef = 5;
pub const kTerm_clr_eos: TerminfoDef = 4;
pub const kTerm_clr_eol: TerminfoDef = 3;
pub const kTerm_clear_screen: TerminfoDef = 2;
pub const kTerm_change_scroll_region: TerminfoDef = 1;
pub const kTerm_carriage_return: TerminfoDef = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TUIData {
    pub loop_0: *mut Loop,
    pub buf: [::core::ffi::c_char; 65535],
    pub buf_to_flush: *mut ::core::ffi::c_char,
    pub bufpos: size_t,
    pub input: TermInput,
    pub write_loop: uv_loop_t,
    pub ti: TerminfoEntry,
    pub term: *mut ::core::ffi::c_char,
    pub output_handle: C2Rust_Unnamed_22,
    pub out_isatty: bool,
    pub winch_handle: SignalWatcher,
    pub startup_delay_timer: uv_timer_t,
    pub grid: UGrid,
    pub invalid_regions: C2Rust_Unnamed_21,
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub out_fd: ::core::ffi::c_int,
    pub pending_resize_events: ::core::ffi::c_int,
    pub terminfo_found_in_db: bool,
    pub can_change_scroll_region: bool,
    pub has_left_and_right_margin_mode: bool,
    pub has_sync_mode: bool,
    pub can_set_lr_margin: bool,
    pub can_scroll: bool,
    pub can_erase_chars: bool,
    pub immediate_wrap_after_last_column: bool,
    pub bce: bool,
    pub mouse_enabled: bool,
    pub mouse_move_enabled: bool,
    pub mouse_enabled_save: bool,
    pub title_enabled: bool,
    pub sync_output: bool,
    pub busy: bool,
    pub is_invisible: bool,
    pub want_invisible: bool,
    pub set_cursor_color_as_str: bool,
    pub cursor_has_color: bool,
    pub is_starting: bool,
    pub resize_events_enabled: bool,
    pub modes: C2Rust_Unnamed_20,
    pub screenshot: *mut FILE,
    pub cursor_shapes: [cursorentry_T; 18],
    pub clear_attrs: HlAttrs,
    pub attrs: C2Rust_Unnamed_19,
    pub print_attr_id: ::core::ffi::c_int,
    pub default_attr: bool,
    pub set_default_colors: bool,
    pub can_clear_attr: bool,
    pub showing_mode: ModeShape,
    pub verbose: Integer,
    pub terminfo_ext: C2Rust_Unnamed_18,
    pub can_set_title: bool,
    pub can_set_underline_color: bool,
    pub can_resize_screen: bool,
    pub stopped: bool,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub rgb: bool,
    pub screen_or_tmux: bool,
    pub url: ::core::ffi::c_int,
    pub urlbuf: StringBuilder,
    pub ti_arena: Arena,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_18 {
    pub enable_focus_reporting: *mut ::core::ffi::c_char,
    pub disable_focus_reporting: *mut ::core::ffi::c_char,
    pub reset_scroll_region: *mut ::core::ffi::c_char,
    pub enter_altfont_mode: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlAttrs,
}
#[derive(Copy, Clone, ::c2rust_bitfields::BitfieldStruct)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    #[bitfield(name = "grapheme_clusters", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "theme_updates", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "resize_events", ty = "bool", bits = "2..=2")]
    pub grapheme_clusters_theme_updates_resize_events: [u8; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_21 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Rect,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rect {
    pub top: ::core::ffi::c_int,
    pub bot: ::core::ffi::c_int,
    pub left: ::core::ffi::c_int,
    pub right: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_22 {
    pub tty: uv_tty_t,
    pub pipe: uv_pipe_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermInput {
    pub in_fd: ::core::ffi::c_int,
    pub paste: int8_t,
    pub ttimeout: bool,
    pub callbacks: TermInputCallbacks,
    pub key_encoding: KeyEncoding,
    pub ttimeoutlen: OptInt,
    pub tk: *mut TermKey,
    pub tk_ti_hook_fn: Option<TermKey_Terminfo_Getstr_Hook>,
    pub timer_handle: uv_timer_t,
    pub bg_query_timer: uv_timer_t,
    pub loop_0: *mut Loop,
    pub read_stream: RStream,
    pub tui_data: *mut TUIData,
    pub key_buffer: [::core::ffi::c_char; 4096],
    pub key_buffer_len: size_t,
}
pub const kKeyEncodingXterm: KeyEncoding = 2;
pub const kKeyEncodingKitty: KeyEncoding = 1;
pub const kKeyEncodingLegacy: KeyEncoding = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermInputCallbacks {
    pub primary_device_attr: Option<unsafe extern "C" fn(*mut TUIData) -> ()>,
}
pub const kTermModeResizeEvents: TermMode = 2048;
pub const kTermModeThemeUpdates: TermMode = 2031;
pub const kTermModeGraphemeClusters: TermMode = 2027;
pub const kTermModeSynchronizedOutput: TermMode = 2026;
pub const kTermModeBracketedPaste: TermMode = 2004;
pub const kTermModeMouseSGRExt: TermMode = 1006;
pub const kTermModeMouseAnyEvent: TermMode = 1003;
pub const kTermModeMouseButtonEvent: TermMode = 1002;
pub const kTermModeLeftAndRightMargins: TermMode = 69;
pub const kTermModePermanentlyReset: TermModeState = 4;
pub const kTermModePermanentlySet: TermModeState = 3;
pub const kTermModeReset: TermModeState = 2;
pub const kTermModeSet: TermModeState = 1;
pub const kTermModeNotRecognized: TermModeState = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_cstr_t = Set_cstr_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<cstr_t>(),
};
pub const NULL_STRING: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_WRN: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const DFLT_COLS: ::core::ffi::c_int = 80 as ::core::ffi::c_int;
pub const DFLT_ROWS: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const DEL: ::core::ffi::c_int = 0x7f as ::core::ffi::c_int;
pub const DEL_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x7F\0") };
pub const CTRL_H_STR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"\x08\0") };
pub const TOO_MANY_EVENTS: ::core::ffi::c_int = 1000000 as ::core::ffi::c_int;
pub const LINUXSET0C: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"\x1B[?0c\0") };
pub const LINUXSET1C: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"\x1B[?1c\0") };
static cursor_style_enabled: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const TERMINFO_SEQ_LIMIT: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
static urls: GlobalCell<Set_cstr_t> = GlobalCell::new(SET_INIT);
#[no_mangle]
pub unsafe extern "C" fn tui_start(
    mut tui_p: *mut *mut TUIData,
    mut width: *mut ::core::ffi::c_int,
    mut height: *mut ::core::ffi::c_int,
    mut term: *mut *mut ::core::ffi::c_char,
    mut rgb: *mut bool,
) {
    let mut tui: *mut TUIData =
        xcalloc(1 as size_t, ::core::mem::size_of::<TUIData>()) as *mut TUIData;
    (*tui).is_starting = true_0 != 0;
    (*tui).screenshot = ::core::ptr::null_mut::<FILE>();
    (*tui).stopped = false_0 != 0;
    (*tui).loop_0 = main_loop.ptr();
    (*tui).url = -1 as ::core::ffi::c_int;
    (*tui).invalid_regions.capacity = 0 as size_t;
    (*tui).invalid_regions.size = (*tui).invalid_regions.capacity;
    (*tui).invalid_regions.items = ::core::ptr::null_mut::<Rect>();
    (*tui).urlbuf.capacity = 0 as size_t;
    (*tui).urlbuf.size = (*tui).urlbuf.capacity;
    (*tui).urlbuf.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    signal_watcher_init(
        (*tui).loop_0,
        &raw mut (*tui).winch_handle,
        tui as *mut ::core::ffi::c_void,
    );
    signal_watcher_start(
        &raw mut (*tui).winch_handle,
        Some(
            sigwinch_cb
                as unsafe extern "C" fn(
                    *mut SignalWatcher,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        SIGWINCH,
    );
    if (*tui).attrs.size == (*tui).attrs.capacity {
        (*tui).attrs.capacity = if (*tui).attrs.capacity != 0 {
            (*tui).attrs.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*tui).attrs.items = xrealloc(
            (*tui).attrs.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HlAttrs>().wrapping_mul((*tui).attrs.capacity),
        ) as *mut HlAttrs;
    } else {
    };
    let c2rust_fresh0 = (*tui).attrs.size;
    (*tui).attrs.size = (*tui).attrs.size.wrapping_add(1);
    *(*tui).attrs.items.offset(c2rust_fresh0 as isize) = HlAttrs {
        rgb_ae_attr: 0 as int32_t,
        cterm_ae_attr: 0 as int32_t,
        rgb_fg_color: -1 as RgbValue,
        rgb_bg_color: -1 as RgbValue,
        rgb_sp_color: -1 as RgbValue,
        cterm_fg_color: 0 as int16_t,
        cterm_bg_color: 0 as int16_t,
        hl_blend: -1 as int32_t,
        url: -1 as int32_t,
    };
    (*tui).input.tk_ti_hook_fn = Some(
        tui_tk_ti_getstr
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                *const ::core::ffi::c_char,
                *mut ::core::ffi::c_void,
            ) -> *const ::core::ffi::c_char,
    ) as Option<TermKey_Terminfo_Getstr_Hook>;
    ugrid_init(&raw mut (*tui).grid);
    tui_terminal_start(tui);
    uv_timer_init(
        &raw mut (*(*tui).loop_0).uv,
        &raw mut (*tui).startup_delay_timer,
    );
    (*tui).startup_delay_timer.data = tui as *mut ::core::ffi::c_void;
    uv_timer_start(
        &raw mut (*tui).startup_delay_timer,
        Some(after_startup_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
        100 as uint64_t,
        0 as uint64_t,
    );
    *tui_p = tui;
    loop_poll_events(main_loop.ptr(), 1 as int64_t);
    *width = (*tui).width;
    *height = (*tui).height;
    *term = (*tui).term;
    *rgb = (*tui).rgb;
}
unsafe extern "C" fn tui_request_term_mode(mut tui: *mut TUIData, mut mode: TermMode) {
    let mut buf: [::core::ffi::c_char; 12] = [0; 12];
    let mut len: ::core::ffi::c_int = snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>(),
        b"\x1B[?%d$p\0".as_ptr() as *const ::core::ffi::c_char,
        mode as ::core::ffi::c_int,
    );
    '_c2rust_label: {
        if len > 0 as ::core::ffi::c_int
            && len < ::core::mem::size_of::<[::core::ffi::c_char; 12]>() as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"(len > 0) && (len < (int)sizeof(buf))\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                200 as ::core::ffi::c_uint,
                b"void tui_request_term_mode(TUIData *, TermMode)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    out(tui, &raw mut buf as *mut ::core::ffi::c_char, len as size_t);
}
unsafe extern "C" fn tui_set_term_mode(mut tui: *mut TUIData, mut mode: TermMode, mut set: bool) {
    let mut buf: [::core::ffi::c_char; 12] = [0; 12];
    let mut len: ::core::ffi::c_int = snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>(),
        b"\x1B[?%d%c\0".as_ptr() as *const ::core::ffi::c_char,
        mode as ::core::ffi::c_int,
        if set as ::core::ffi::c_int != 0 {
            'h' as ::core::ffi::c_int
        } else {
            'l' as ::core::ffi::c_int
        },
    );
    '_c2rust_label: {
        if len > 0 as ::core::ffi::c_int
            && len < ::core::mem::size_of::<[::core::ffi::c_char; 12]>() as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"(len > 0) && (len < (int)sizeof(buf))\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                210 as ::core::ffi::c_uint,
                b"void tui_set_term_mode(TUIData *, TermMode, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    out(tui, &raw mut buf as *mut ::core::ffi::c_char, len as size_t);
}
#[no_mangle]
pub unsafe extern "C" fn tui_handle_term_mode(
    mut tui: *mut TUIData,
    mut mode: TermMode,
    mut state: TermModeState,
) {
    let mut is_set: bool = false_0 != 0;
    's_137: {
        match state as ::core::ffi::c_uint {
            0 | 4 => {
                if !nvim_testing.get() {
                    logmsg(
                        LOGLVL_WRN,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"tui_handle_term_mode\0".as_ptr() as *const ::core::ffi::c_char,
                        226 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"TUI: terminal mode %d unavailable, state %d\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        mode as ::core::ffi::c_uint,
                        state as ::core::ffi::c_uint,
                    );
                }
                break 's_137;
            }
            3 | 1 => {
                is_set = true_0 != 0;
            }
            2 => {}
            _ => {
                break 's_137;
            }
        }
        if !nvim_testing.get() {
            logmsg(
                LOGLVL_WRN,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"tui_handle_term_mode\0".as_ptr() as *const ::core::ffi::c_char,
                239 as ::core::ffi::c_int,
                true_0 != 0,
                b"TUI: terminal mode %d detected, state %d\0".as_ptr()
                    as *const ::core::ffi::c_char,
                mode as ::core::ffi::c_uint,
                state as ::core::ffi::c_uint,
            );
        }
        match mode as ::core::ffi::c_uint {
            2026 => {
                (*tui).has_sync_mode = true_0 != 0;
            }
            2027 => {
                if !is_set {
                    tui_set_term_mode(tui, mode, true_0 != 0);
                    (*tui).modes.set_grapheme_clusters((true_0 != 0) as bool);
                }
            }
            2031 => {
                if !is_set {
                    tui_set_term_mode(tui, mode, true_0 != 0);
                    (*tui).modes.set_theme_updates((true_0 != 0) as bool);
                }
            }
            2048 => {
                if !is_set {
                    tui_set_term_mode(tui, mode, true_0 != 0);
                    (*tui).modes.set_resize_events((true_0 != 0) as bool);
                }
                (*tui).resize_events_enabled = true_0 != 0;
            }
            69 => {
                (*tui).has_left_and_right_margin_mode = true_0 != 0;
            }
            _ => {}
        }
    };
}
unsafe extern "C" fn tui_query_extended_underline(mut tui: *mut TUIData) {
    out(
        tui,
        b"\x1B[0m\x1B[4:3m\x1BP$qm\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 18]>().wrapping_sub(1 as size_t),
    );
    (*tui).print_attr_id = -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tui_enable_extended_underline(mut tui: *mut TUIData) {
    terminfo_set_if_empty(
        tui,
        kTerm_set_underline_style,
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    );
    (*tui).can_set_underline_color = true_0 != 0;
}
unsafe extern "C" fn tui_query_kitty_keyboard(mut tui: *mut TUIData) {
    (*tui).input.callbacks.primary_device_attr =
        Some(tui_set_key_encoding as unsafe extern "C" fn(*mut TUIData) -> ())
            as Option<unsafe extern "C" fn(*mut TUIData) -> ()>;
    out(
        tui,
        b"\x1B[?u\x1B[c\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_set_key_encoding(mut tui: *mut TUIData) {
    match (*tui).input.key_encoding as ::core::ffi::c_uint {
        1 => {
            out(
                tui,
                b"\x1B[>3u\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            );
        }
        2 => {
            out(
                tui,
                b"\x1B[>4;2m\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn tui_reset_key_encoding(mut tui: *mut TUIData) {
    match (*tui).input.key_encoding as ::core::ffi::c_uint {
        1 => {
            out(
                tui,
                b"\x1B[<u\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            );
        }
        2 => {
            out(
                tui,
                b"\x1B[>4;0m\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn tui_query_bg_color_noflush(mut tui: *mut TUIData) {
    out(
        tui,
        b"\x1B]11;?\x07\x1B[5n\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_query_bg_color(mut tui: *mut TUIData) {
    tui_query_bg_color_noflush(tui);
    flush_buf(tui);
}
unsafe extern "C" fn terminfo_start(mut tui: *mut TUIData) {
    (*tui).bufpos = 0 as size_t;
    (*tui).default_attr = false_0 != 0;
    (*tui).can_clear_attr = false_0 != 0;
    (*tui).is_invisible = true_0 != 0;
    (*tui).want_invisible = false_0 != 0;
    (*tui).busy = false_0 != 0;
    (*tui).set_cursor_color_as_str = false_0 != 0;
    (*tui).cursor_has_color = false_0 != 0;
    (*tui).resize_events_enabled = false_0 != 0;
    (*tui).modes.set_grapheme_clusters((false_0 != 0) as bool);
    (*tui).modes.set_resize_events((false_0 != 0) as bool);
    (*tui).modes.set_theme_updates((false_0 != 0) as bool);
    (*tui).showing_mode = SHAPE_IDX_N;
    (*tui).terminfo_ext.enable_focus_reporting = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tui).terminfo_ext.disable_focus_reporting = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tui).out_fd = STDOUT_FILENO;
    (*tui).out_isatty = os_isatty((*tui).out_fd);
    (*tui).input.tui_data = tui;
    (*tui).ti_arena = ARENA_EMPTY;
    '_c2rust_label: {
        if (*tui).term.is_null() {
        } else {
            __assert_fail(
                b"tui->term == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                384 as ::core::ffi::c_uint,
                b"void terminfo_start(TUIData *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut term: *mut ::core::ffi::c_char =
        os_getenv(b"TERM\0".as_ptr() as *const ::core::ffi::c_char);
    (*tui).terminfo_found_in_db = false_0 != 0;
    if !term.is_null() {
        if terminfo_from_database(&raw mut (*tui).ti, term, &raw mut (*tui).ti_arena) {
            (*tui).term = arena_strdup(&raw mut (*tui).ti_arena, term);
            (*tui).terminfo_found_in_db = true_0 != 0;
        }
    }
    if !(*tui).terminfo_found_in_db {
        let mut new: *const TerminfoEntry = terminfo_from_builtin(term, &raw mut (*tui).term);
        memcpy(
            &raw mut (*tui).ti as *mut ::core::ffi::c_void,
            new as *const ::core::ffi::c_void,
            ::core::mem::size_of::<TerminfoEntry>(),
        );
    }
    let mut colorterm: *mut ::core::ffi::c_char =
        os_getenv(b"COLORTERM\0".as_ptr() as *const ::core::ffi::c_char);
    let mut termprg: *mut ::core::ffi::c_char =
        os_getenv(b"TERM_PROGRAM\0".as_ptr() as *const ::core::ffi::c_char);
    let mut vte_version_env: *mut ::core::ffi::c_char =
        os_getenv(b"VTE_VERSION\0".as_ptr() as *const ::core::ffi::c_char);
    let mut konsolev_env: *mut ::core::ffi::c_char =
        os_getenv(b"KONSOLE_VERSION\0".as_ptr() as *const ::core::ffi::c_char);
    let mut term_program_version_env: *mut ::core::ffi::c_char =
        os_getenv(b"TERM_PROGRAM_VERSION\0".as_ptr() as *const ::core::ffi::c_char);
    let mut vtev: ::core::ffi::c_int = if !vte_version_env.is_null() {
        strtol(
            vte_version_env,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            10 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut iterm_env: bool = !termprg.is_null()
        && !strstr(
            termprg,
            b"iTerm.app\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null();
    let mut nsterm: bool = !termprg.is_null()
        && !strstr(
            termprg,
            b"Apple_Terminal\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null()
        || terminfo_is_term_family(term, b"nsterm\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0;
    let mut konsole: bool =
        terminfo_is_term_family(term, b"konsole\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || os_env_exists(
                b"KONSOLE_PROFILE_NAME\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0
            || os_env_exists(
                b"KONSOLE_DBUS_SESSION\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0;
    let mut konsolev: ::core::ffi::c_int = if !konsolev_env.is_null() {
        strtol(
            konsolev_env,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            10 as ::core::ffi::c_int,
        ) as ::core::ffi::c_int
    } else if konsole as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut wezterm: bool = strequal(termprg, b"WezTerm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut weztermv: *const ::core::ffi::c_char = if wezterm as ::core::ffi::c_int != 0 {
        term_program_version_env
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    let mut screen: bool =
        terminfo_is_term_family(term, b"screen\0".as_ptr() as *const ::core::ffi::c_char);
    let mut tmux: bool =
        terminfo_is_term_family(term, b"tmux\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || os_env_exists(
                b"TMUX\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0;
    (*tui).screen_or_tmux = screen as ::core::ffi::c_int != 0 || tmux as ::core::ffi::c_int != 0;
    (*tui).rgb = term_has_truecolor(tui, colorterm);
    patch_terminfo_bugs(tui, term, colorterm, vtev, konsolev, iterm_env, nsterm);
    augment_terminfo(tui, term, vtev, konsolev, weztermv, iterm_env, nsterm);
    (*tui).can_change_scroll_region =
        !(*tui).ti.defs[kTerm_change_scroll_region as ::core::ffi::c_int as usize].is_null();
    (*tui).can_set_lr_margin =
        !(*tui).ti.defs[kTerm_set_lr_margin as ::core::ffi::c_int as usize].is_null();
    (*tui).can_scroll = !(*tui).ti.defs[kTerm_delete_line as ::core::ffi::c_int as usize].is_null()
        && !(*tui).ti.defs[kTerm_parm_delete_line as ::core::ffi::c_int as usize].is_null()
        && !(*tui).ti.defs[kTerm_insert_line as ::core::ffi::c_int as usize].is_null()
        && !(*tui).ti.defs[kTerm_parm_insert_line as ::core::ffi::c_int as usize].is_null();
    (*tui).can_erase_chars =
        !(*tui).ti.defs[kTerm_erase_chars as ::core::ffi::c_int as usize].is_null();
    (*tui).immediate_wrap_after_last_column =
        terminfo_is_term_family(term, b"conemu\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || terminfo_is_term_family(term, b"cygwin\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"win32con\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"interix\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0;
    (*tui).bce = (*tui).ti.bce;
    t_colors.set((*tui).ti.max_colors);
    terminfo_out(tui, kTerm_enter_ca_mode);
    terminfo_out(tui, kTerm_keypad_xmit);
    terminfo_out(tui, kTerm_clear_screen);
    tui_set_term_mode(tui, kTermModeBracketedPaste, true_0 != 0);
    (*tui).has_left_and_right_margin_mode = false_0 != 0;
    (*tui).has_sync_mode = false_0 != 0;
    if !nsterm {
        tui_request_term_mode(tui, kTermModeLeftAndRightMargins);
        tui_request_term_mode(tui, kTermModeSynchronizedOutput);
        tui_request_term_mode(tui, kTermModeGraphemeClusters);
        tui_request_term_mode(tui, kTermModeThemeUpdates);
        tui_request_term_mode(tui, kTermModeResizeEvents);
    }
    if (*tui).ti.defs[kTerm_set_underline_style as ::core::ffi::c_int as usize].is_null()
        && !(screen as ::core::ffi::c_int != 0
            || tmux as ::core::ffi::c_int != 0
            || nsterm as ::core::ffi::c_int != 0)
    {
        tui_query_extended_underline(tui);
    }
    tui_query_kitty_keyboard(tui);
    tui_query_bg_color_noflush(tui);
    let mut ret: ::core::ffi::c_int = 0;
    uv_loop_init(&raw mut (*tui).write_loop);
    if (*tui).out_isatty {
        ret = uv_tty_init(
            &raw mut (*tui).write_loop,
            &raw mut (*tui).output_handle.tty,
            (*tui).out_fd as uv_file,
            0 as ::core::ffi::c_int,
        );
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"terminfo_start\0".as_ptr() as *const ::core::ffi::c_char,
                502 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_tty_init failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
        let mut retry_count: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        loop {
            ret = uv_tty_set_mode(&raw mut (*tui).output_handle.tty, UV_TTY_MODE_IO);
            if !(ret == UV_EINTR as ::core::ffi::c_int && retry_count > 0 as ::core::ffi::c_int) {
                break;
            }
            retry_count -= 1;
        }
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"terminfo_start\0".as_ptr() as *const ::core::ffi::c_char,
                513 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_tty_set_mode failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
    } else {
        ret = uv_pipe_init(
            &raw mut (*tui).write_loop,
            &raw mut (*tui).output_handle.pipe,
            0 as ::core::ffi::c_int,
        );
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"terminfo_start\0".as_ptr() as *const ::core::ffi::c_char,
                519 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_pipe_init failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
        ret = uv_pipe_open(&raw mut (*tui).output_handle.pipe, (*tui).out_fd as uv_file);
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"terminfo_start\0".as_ptr() as *const ::core::ffi::c_char,
                523 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_pipe_open failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
    }
    flush_buf(tui);
    xfree(term as *mut ::core::ffi::c_void);
    xfree(colorterm as *mut ::core::ffi::c_void);
    xfree(termprg as *mut ::core::ffi::c_void);
    xfree(vte_version_env as *mut ::core::ffi::c_void);
    xfree(konsolev_env as *mut ::core::ffi::c_void);
    xfree(term_program_version_env as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn terminfo_disable(mut tui: *mut TUIData) {
    if (*tui).modes.theme_updates() {
        tui_set_term_mode(tui, kTermModeThemeUpdates, false_0 != 0);
    }
    tui_mode_change(
        tui,
        NULL_STRING,
        SHAPE_IDX_N as ::core::ffi::c_int as Integer,
    );
    tui_mouse_off(tui);
    terminfo_out(tui, kTerm_exit_attribute_mode);
    terminfo_out(tui, kTerm_cursor_normal);
    terminfo_out(tui, kTerm_reset_cursor_style);
    terminfo_out(tui, kTerm_keypad_local);
    tui_reset_key_encoding(tui);
    if (*tui).modes.resize_events() {
        tui_set_term_mode(tui, kTermModeResizeEvents, false_0 != 0);
    }
    if (*tui).modes.grapheme_clusters() {
        tui_set_term_mode(tui, kTermModeGraphemeClusters, false_0 != 0);
    }
    tui_set_title(tui, NULL_STRING);
    if (*tui).cursor_has_color {
        terminfo_out(tui, kTerm_reset_cursor_color);
    }
    tui_set_term_mode(tui, kTermModeBracketedPaste, false_0 != 0);
    out_len(tui, (*tui).terminfo_ext.disable_focus_reporting);
    out(
        tui,
        b"\x1B[c\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
    );
    flush_buf(tui);
}
unsafe extern "C" fn terminfo_stop(mut tui: *mut TUIData) {
    if ui_client_exit_status.get() == 0 as ::core::ffi::c_int
        && ui_client_error_exit.get() > 0 as ::core::ffi::c_int
    {
        ui_client_exit_status.set(ui_client_error_exit.get());
    }
    if ui_client_exit_status.get()
        == (if ui_client_error_exit.get() > 0 as ::core::ffi::c_int {
            ui_client_error_exit.get()
        } else {
            0 as ::core::ffi::c_int
        })
    {
        cursor_goto(
            tui,
            (*tui).height - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        terminfo_out(tui, kTerm_exit_ca_mode);
    }
    flush_buf(tui);
    uv_tty_reset_mode();
    uv_close(&raw mut (*tui).output_handle as *mut uv_handle_t, None);
    uv_run(&raw mut (*tui).write_loop, UV_RUN_DEFAULT);
    if uv_loop_close(&raw mut (*tui).write_loop) != 0 {
        abort();
    }
    arena_mem_free(arena_finish(&raw mut (*tui).ti_arena));
    memset(
        &raw mut (*tui).ti as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<TerminfoEntry>(),
    );
    (*tui).term = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn tui_terminal_start(mut tui: *mut TUIData) {
    (*tui).print_attr_id = -1 as ::core::ffi::c_int;
    terminfo_start(tui);
    if (*tui).input.loop_0.is_null() {
        tinput_init(&raw mut (*tui).input, main_loop.ptr(), &raw mut (*tui).ti);
    }
    tui_guess_size(tui);
    tinput_start(&raw mut (*tui).input);
}
unsafe extern "C" fn after_startup_cb(mut handle: *mut uv_timer_t) {
    let mut tui: *mut TUIData = (*handle).data as *mut TUIData;
    tui_terminal_after_startup(tui);
}
unsafe extern "C" fn tui_terminal_after_startup(mut tui: *mut TUIData) {
    out_len(tui, (*tui).terminfo_ext.enable_focus_reporting);
    flush_buf(tui);
}
#[no_mangle]
pub unsafe extern "C" fn tui_stop(mut tui: *mut TUIData) {
    if uv_is_closing(&raw mut (*tui).output_handle as *mut uv_handle_t) != 0 {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_stop\0".as_ptr() as *const ::core::ffi::c_char,
            646 as ::core::ffi::c_int,
            true_0 != 0,
            b"TUI already stopped (race?)\0".as_ptr() as *const ::core::ffi::c_char,
        );
        (*tui).stopped = true_0 != 0;
        return;
    }
    (*tui).input.callbacks.primary_device_attr =
        Some(tui_stop_cb as unsafe extern "C" fn(*mut TUIData) -> ())
            as Option<unsafe extern "C" fn(*mut TUIData) -> ()>;
    terminfo_disable(tui);
    let mut remaining: int64_t = 1000 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !((*tui).stopped as ::core::ffi::c_int != 0
        || (*tui).input.read_stream.did_eof as ::core::ffi::c_int != 0)
    {
        if !(*(*tui).loop_0).events.is_null() && !multiqueue_empty((*(*tui).loop_0).events) {
            multiqueue_process_events((*(*tui).loop_0).events);
        } else {
            loop_poll_events((*tui).loop_0, remaining);
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
    if !(*tui).stopped && !(*tui).input.read_stream.did_eof {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_stop\0".as_ptr() as *const ::core::ffi::c_char,
            658 as ::core::ffi::c_int,
            true_0 != 0,
            b"TUI: timed out waiting for DA1 response\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    (*tui).stopped = true_0 != 0;
    tui_terminal_stop(tui);
    stream_set_blocking((*tui).input.in_fd, true_0 != 0);
    tinput_destroy(&raw mut (*tui).input);
    signal_watcher_stop(&raw mut (*tui).winch_handle);
    signal_watcher_close(&raw mut (*tui).winch_handle, None);
    uv_close(
        &raw mut (*tui).startup_delay_timer as *mut uv_handle_t,
        None,
    );
}
unsafe extern "C" fn tui_stop_cb(mut tui: *mut TUIData) {
    (*tui).stopped = true_0 != 0;
}
unsafe extern "C" fn tui_terminal_stop(mut tui: *mut TUIData) {
    tinput_stop(&raw mut (*tui).input);
    terminfo_stop(tui);
}
#[no_mangle]
pub unsafe extern "C" fn tui_is_stopped(mut tui: *mut TUIData) -> bool {
    return (*tui).stopped;
}
unsafe extern "C" fn sigwinch_cb(
    mut _watcher: *mut SignalWatcher,
    mut _signum: ::core::ffi::c_int,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut tui: *mut TUIData = cbdata as *mut TUIData;
    if tui_is_stopped(tui) as ::core::ffi::c_int != 0
        || (*tui).resize_events_enabled as ::core::ffi::c_int != 0
    {
        return;
    }
    tui_guess_size(tui);
}
unsafe extern "C" fn attrs_differ(
    mut tui: *mut TUIData,
    mut id1: ::core::ffi::c_int,
    mut id2: ::core::ffi::c_int,
    mut rgb: bool,
) -> bool {
    if id1 == id2 {
        return false_0 != 0;
    } else if id1 < 0 as ::core::ffi::c_int || id2 < 0 as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut a1: HlAttrs = *(*tui).attrs.items.offset(id1 as size_t as isize);
    let mut a2: HlAttrs = *(*tui).attrs.items.offset(id2 as size_t as isize);
    if a1.url != a2.url {
        return true_0 != 0;
    }
    if rgb {
        return a1.rgb_fg_color != a2.rgb_fg_color
            || a1.rgb_bg_color != a2.rgb_bg_color
            || a1.rgb_ae_attr != a2.rgb_ae_attr
            || a1.rgb_sp_color != a2.rgb_sp_color;
    } else {
        return a1.cterm_fg_color as ::core::ffi::c_int != a2.cterm_fg_color as ::core::ffi::c_int
            || a1.cterm_bg_color as ::core::ffi::c_int != a2.cterm_bg_color as ::core::ffi::c_int
            || a1.cterm_ae_attr != a2.cterm_ae_attr
            || a1.cterm_ae_attr & HL_UNDERLINE_MASK as ::core::ffi::c_int as int32_t != 0
                && a1.rgb_sp_color != a2.rgb_sp_color;
    };
}
unsafe extern "C" fn update_attrs(mut tui: *mut TUIData, mut attr_id: ::core::ffi::c_int) {
    if !attrs_differ(tui, attr_id, (*tui).print_attr_id, (*tui).rgb) {
        (*tui).print_attr_id = attr_id;
        return;
    }
    (*tui).print_attr_id = attr_id;
    let mut attrs: HlAttrs = *(*tui).attrs.items.offset(attr_id as size_t as isize);
    let mut attr: ::core::ffi::c_int = if (*tui).rgb as ::core::ffi::c_int != 0 {
        attrs.rgb_ae_attr as ::core::ffi::c_int
    } else {
        attrs.cterm_ae_attr as ::core::ffi::c_int
    };
    let mut bold: bool = attr & HL_BOLD as ::core::ffi::c_int != 0;
    let mut italic: bool = attr & HL_ITALIC as ::core::ffi::c_int != 0;
    let mut reverse: bool = attr & HL_INVERSE as ::core::ffi::c_int != 0;
    let mut standout: bool = attr & HL_STANDOUT as ::core::ffi::c_int != 0;
    let mut strikethrough: bool = attr & HL_STRIKETHROUGH as ::core::ffi::c_int != 0;
    let mut altfont: bool = attr & HL_ALTFONT as ::core::ffi::c_int != 0;
    let mut dim: bool = attr & HL_DIM as ::core::ffi::c_int != 0;
    let mut blink: bool = attr & HL_BLINK as ::core::ffi::c_int != 0;
    let mut conceal: bool = attr & HL_CONCEALED as ::core::ffi::c_int != 0;
    let mut overline: bool = attr & HL_OVERLINE as ::core::ffi::c_int != 0;
    let mut underline: bool = false;
    let mut undercurl: bool = false;
    let mut underdouble: bool = false;
    let mut underdotted: bool = false;
    let mut underdashed: bool = false;
    if !(*tui).ti.defs[kTerm_set_underline_style as ::core::ffi::c_int as usize].is_null() {
        let mut ul: ::core::ffi::c_int = attr & HL_UNDERLINE_MASK as ::core::ffi::c_int;
        underline = ul == HL_UNDERLINE as ::core::ffi::c_int;
        undercurl = ul == HL_UNDERCURL as ::core::ffi::c_int;
        underdouble = ul == HL_UNDERDOUBLE as ::core::ffi::c_int;
        underdashed = ul == HL_UNDERDASHED as ::core::ffi::c_int;
        underdotted = ul == HL_UNDERDOTTED as ::core::ffi::c_int;
    } else {
        underline = attr & HL_UNDERLINE_MASK as ::core::ffi::c_int != 0;
        undercurl = false_0 != 0;
        underdouble = false_0 != 0;
        underdotted = false_0 != 0;
        underdashed = false_0 != 0;
    }
    let mut has_any_underline: bool = undercurl as ::core::ffi::c_int != 0
        || underline as ::core::ffi::c_int != 0
        || underdouble as ::core::ffi::c_int != 0
        || underdotted as ::core::ffi::c_int != 0
        || underdashed as ::core::ffi::c_int != 0;
    if !(*tui).ti.defs[kTerm_set_attributes as ::core::ffi::c_int as usize].is_null() {
        if bold as ::core::ffi::c_int != 0
            || dim as ::core::ffi::c_int != 0
            || blink as ::core::ffi::c_int != 0
            || reverse as ::core::ffi::c_int != 0
            || underline as ::core::ffi::c_int != 0
            || standout as ::core::ffi::c_int != 0
        {
            let mut params: [TPVAR; 9] = [
                TPVAR {
                    num: 0 as ::core::ffi::c_long,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
            ];
            params[0 as ::core::ffi::c_int as usize].num = standout as ::core::ffi::c_long;
            params[1 as ::core::ffi::c_int as usize].num = underline as ::core::ffi::c_long;
            params[2 as ::core::ffi::c_int as usize].num = reverse as ::core::ffi::c_long;
            params[3 as ::core::ffi::c_int as usize].num = blink as ::core::ffi::c_long;
            params[4 as ::core::ffi::c_int as usize].num = dim as ::core::ffi::c_long;
            params[5 as ::core::ffi::c_int as usize].num = bold as ::core::ffi::c_long;
            params[6 as ::core::ffi::c_int as usize].num = 0 as ::core::ffi::c_long;
            params[7 as ::core::ffi::c_int as usize].num = 0 as ::core::ffi::c_long;
            params[8 as ::core::ffi::c_int as usize].num = 0 as ::core::ffi::c_long;
            terminfo_print(tui, kTerm_set_attributes, &raw mut params as *mut TPVAR);
        } else if !(*tui).default_attr {
            terminfo_out(tui, kTerm_exit_attribute_mode);
        }
    } else {
        if !(*tui).default_attr {
            terminfo_out(tui, kTerm_exit_attribute_mode);
        }
        if bold {
            terminfo_out(tui, kTerm_enter_bold_mode);
        }
        if underline {
            terminfo_out(tui, kTerm_enter_underline_mode);
        }
        if standout {
            terminfo_out(tui, kTerm_enter_standout_mode);
        }
        if reverse {
            terminfo_out(tui, kTerm_enter_reverse_mode);
        }
        if dim {
            terminfo_out(tui, kTerm_enter_dim_mode);
        }
        if blink {
            terminfo_out(tui, kTerm_enter_blink_mode);
        }
    }
    if italic {
        terminfo_out(tui, kTerm_enter_italics_mode);
    }
    if altfont {
        out_len(tui, (*tui).terminfo_ext.enter_altfont_mode);
    }
    if strikethrough {
        terminfo_out(tui, kTerm_enter_strikethrough_mode);
    }
    if conceal {
        terminfo_out(tui, kTerm_enter_secure_mode);
    }
    if overline {
        out(
            tui,
            b"\x1B[53m\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
    }
    if !(*tui).ti.defs[kTerm_set_underline_style as ::core::ffi::c_int as usize].is_null() {
        if undercurl {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                3 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if underdouble {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                2 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if underdotted {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                4 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if underdashed {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                5 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
    }
    if has_any_underline as ::core::ffi::c_int != 0
        && (*tui).can_set_underline_color as ::core::ffi::c_int != 0
    {
        let mut color: ::core::ffi::c_int = attrs.rgb_sp_color as ::core::ffi::c_int;
        if color != -1 as ::core::ffi::c_int {
            out_printf(
                tui,
                128 as size_t,
                b"\x1B[58:2::%d:%d:%dm\0".as_ptr() as *const ::core::ffi::c_char,
                color >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                color >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                color & 0xff as ::core::ffi::c_int,
            );
        }
    }
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    if (*tui).rgb as ::core::ffi::c_int != 0 && attr & HL_FG_INDEXED as ::core::ffi::c_int == 0 {
        fg = (if attrs.rgb_fg_color != -1 as RgbValue {
            attrs.rgb_fg_color
        } else {
            (*tui).clear_attrs.rgb_fg_color
        }) as ::core::ffi::c_int;
        if fg != -1 as ::core::ffi::c_int {
            terminfo_print_num(
                tui,
                kTerm_set_rgb_foreground,
                fg >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                fg >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                fg & 0xff as ::core::ffi::c_int,
            );
        }
    } else {
        fg = if attrs.cterm_fg_color as ::core::ffi::c_int != 0 {
            attrs.cterm_fg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            (*tui).clear_attrs.cterm_fg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        };
        if fg != -1 as ::core::ffi::c_int {
            terminfo_print_num(
                tui,
                kTerm_set_a_foreground,
                fg,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
    }
    if (*tui).rgb as ::core::ffi::c_int != 0 && attr & HL_BG_INDEXED as ::core::ffi::c_int == 0 {
        bg = (if attrs.rgb_bg_color != -1 as RgbValue {
            attrs.rgb_bg_color
        } else {
            (*tui).clear_attrs.rgb_bg_color
        }) as ::core::ffi::c_int;
        if bg != -1 as ::core::ffi::c_int {
            terminfo_print_num(
                tui,
                kTerm_set_rgb_background,
                bg >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                bg >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                bg & 0xff as ::core::ffi::c_int,
            );
        }
    } else {
        bg = if attrs.cterm_bg_color as ::core::ffi::c_int != 0 {
            attrs.cterm_bg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            (*tui).clear_attrs.cterm_bg_color as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        };
        if bg != -1 as ::core::ffi::c_int {
            terminfo_print_num(
                tui,
                kTerm_set_a_background,
                bg,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
    }
    if (*tui).url as int32_t != attrs.url {
        if attrs.url >= 0 as int32_t {
            let mut url: *const ::core::ffi::c_char =
                *(*urls.ptr()).keys.offset(attrs.url as isize) as *const ::core::ffi::c_char;
            (*tui).urlbuf.size = 0 as size_t;
            let id: uint64_t =
                (0xe1ea0000 as uint32_t).wrapping_add(attrs.url as uint32_t) as uint64_t;
            kv_do_printf(
                &raw mut (*tui).urlbuf,
                b"\x1B]8;id=%lu;%s\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
                id,
                url,
            );
            out(tui, (*tui).urlbuf.items, (*tui).urlbuf.size);
        } else {
            out(
                tui,
                b"\x1B]8;;\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
        }
        (*tui).url = attrs.url as ::core::ffi::c_int;
    }
    (*tui).default_attr = fg == -1 as ::core::ffi::c_int
        && bg == -1 as ::core::ffi::c_int
        && !bold
        && !dim
        && !blink
        && !conceal
        && !overline
        && !italic
        && !has_any_underline
        && !reverse
        && !standout
        && !strikethrough;
    (*tui).can_clear_attr = !reverse
        && !standout
        && !dim
        && !blink
        && !conceal
        && !overline
        && !has_any_underline
        && !strikethrough
        && ((*tui).bce as ::core::ffi::c_int != 0 || bg == -1 as ::core::ffi::c_int);
}
unsafe extern "C" fn final_column_wrap(mut tui: *mut TUIData) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if (*grid).row != -1 as ::core::ffi::c_int && (*grid).col == (*tui).width {
        (*grid).col = 0 as ::core::ffi::c_int;
        if (*grid).row
            < (if (*tui).height < (*grid).height - 1 as ::core::ffi::c_int {
                (*tui).height
            } else {
                (*grid).height - 1 as ::core::ffi::c_int
            })
        {
            (*grid).row += 1;
        }
    }
}
unsafe extern "C" fn print_cell(
    mut tui: *mut TUIData,
    mut buf: *mut ::core::ffi::c_char,
    mut attr: sattr_T,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if !(*tui).immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
    update_attrs(tui, attr as ::core::ffi::c_int);
    out(tui, buf, strlen(buf));
    (*grid).col += 1;
    if (*tui).immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
}
unsafe extern "C" fn cheap_to_print(
    mut tui: *mut TUIData,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut next: ::core::ffi::c_int,
) -> bool {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    let mut cell: *mut UCell = (*(*grid).cells.offset(row as isize)).offset(col as isize);
    while next != 0 {
        next -= 1;
        if attrs_differ(
            tui,
            (*cell).attr as ::core::ffi::c_int,
            (*tui).print_attr_id,
            (*tui).rgb,
        ) {
            if (*tui).default_attr {
                return false_0 != 0;
            }
        }
        if schar_get_ascii((*cell).data) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        cell = cell.offset(1);
    }
    return true_0 != 0;
}
unsafe extern "C" fn cursor_goto(
    mut tui: *mut TUIData,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if row == (*grid).row && col == (*grid).col {
        return;
    }
    if (*tui).url >= 0 as ::core::ffi::c_int {
        out(
            tui,
            b"\x1B]8;;\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        );
        (*tui).url = -1 as ::core::ffi::c_int;
        (*tui).print_attr_id = -1 as ::core::ffi::c_int;
    }
    if 0 as ::core::ffi::c_int == row && 0 as ::core::ffi::c_int == col {
        terminfo_out(tui, kTerm_cursor_home);
        ugrid_goto(grid, row, col);
        return;
    }
    if (*grid).row != -1 as ::core::ffi::c_int {
        if if 0 as ::core::ffi::c_int == col {
            (col != (*grid).col) as ::core::ffi::c_int
        } else if row != (*grid).row {
            false_0
        } else if 1 as ::core::ffi::c_int == col {
            ((2 as ::core::ffi::c_int) < (*grid).col
                && cheap_to_print(tui, (*grid).row, 0 as ::core::ffi::c_int, col)
                    as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
        } else if 2 as ::core::ffi::c_int == col {
            ((5 as ::core::ffi::c_int) < (*grid).col
                && cheap_to_print(tui, (*grid).row, 0 as ::core::ffi::c_int, col)
                    as ::core::ffi::c_int
                    != 0) as ::core::ffi::c_int
        } else {
            false_0
        } != 0
        {
            terminfo_out(tui, kTerm_carriage_return);
            ugrid_goto(grid, (*grid).row, 0 as ::core::ffi::c_int);
        }
        if row == (*grid).row {
            if col < (*grid).col
                && ((*tui).immediate_wrap_after_last_column as ::core::ffi::c_int != 0
                    || (*grid).col < (*tui).width)
            {
                let mut n: ::core::ffi::c_int = (*grid).col - col;
                if n <= 4 as ::core::ffi::c_int {
                    loop {
                        let c2rust_fresh1 = n;
                        n = n - 1;
                        if c2rust_fresh1 == 0 {
                            break;
                        }
                        terminfo_out(tui, kTerm_cursor_left);
                    }
                } else {
                    terminfo_print_num(
                        tui,
                        kTerm_parm_left_cursor,
                        n,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                ugrid_goto(grid, row, col);
                return;
            } else if col > (*grid).col {
                let mut n_0: ::core::ffi::c_int = col - (*grid).col;
                if n_0 <= 2 as ::core::ffi::c_int {
                    loop {
                        let c2rust_fresh2 = n_0;
                        n_0 = n_0 - 1;
                        if c2rust_fresh2 == 0 {
                            break;
                        }
                        terminfo_out(tui, kTerm_cursor_right);
                    }
                } else {
                    terminfo_print_num(
                        tui,
                        kTerm_parm_right_cursor,
                        n_0,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                ugrid_goto(grid, row, col);
                return;
            }
        }
        if col == (*grid).col {
            if row > (*grid).row {
                let mut n_1: ::core::ffi::c_int = row - (*grid).row;
                if n_1 <= 4 as ::core::ffi::c_int {
                    loop {
                        let c2rust_fresh3 = n_1;
                        n_1 = n_1 - 1;
                        if c2rust_fresh3 == 0 {
                            break;
                        }
                        terminfo_out(tui, kTerm_cursor_down);
                    }
                } else {
                    terminfo_print_num(
                        tui,
                        kTerm_parm_down_cursor,
                        n_1,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                ugrid_goto(grid, row, col);
                return;
            } else if row < (*grid).row {
                let mut n_2: ::core::ffi::c_int = (*grid).row - row;
                if n_2 <= 2 as ::core::ffi::c_int {
                    loop {
                        let c2rust_fresh4 = n_2;
                        n_2 = n_2 - 1;
                        if c2rust_fresh4 == 0 {
                            break;
                        }
                        terminfo_out(tui, kTerm_cursor_up);
                    }
                } else {
                    terminfo_print_num(
                        tui,
                        kTerm_parm_up_cursor,
                        n_2,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                ugrid_goto(grid, row, col);
                return;
            }
        }
    }
    terminfo_print_num(tui, kTerm_cursor_address, row, col, 0 as ::core::ffi::c_int);
    ugrid_goto(grid, row, col);
}
unsafe extern "C" fn print_spaces(mut tui: *mut TUIData, mut width: ::core::ffi::c_int) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    let mut left: size_t = width as size_t;
    loop {
        let mut buf_fit: size_t = if left
            < ::core::mem::size_of::<[::core::ffi::c_char; 65535]>()
                .wrapping_sub((*tui).bufpos as usize)
        {
            left
        } else {
            ::core::mem::size_of::<[::core::ffi::c_char; 65535]>().wrapping_sub((*tui).bufpos)
        };
        memset(
            (&raw mut (*tui).buf as *mut ::core::ffi::c_char).offset((*tui).bufpos as isize)
                as *mut ::core::ffi::c_void,
            ' ' as ::core::ffi::c_int,
            buf_fit,
        );
        (*tui).bufpos = (*tui).bufpos.wrapping_add(buf_fit);
        left = left.wrapping_sub(buf_fit);
        if left == 0 as size_t {
            break;
        }
        flush_buf(tui);
    }
    (*grid).col += width;
    if (*tui).immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
}
unsafe extern "C" fn print_cell_at_pos(
    mut tui: *mut TUIData,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut cell: *mut UCell,
    mut is_doublewidth: bool,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if (*grid).row == -1 as ::core::ffi::c_int && (*cell).data == NUL as schar_T {
        return;
    }
    cursor_goto(tui, row, col);
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut buf as *mut ::core::ffi::c_char, (*cell).data);
    let mut c: ::core::ffi::c_int = utf_ptr2char(&raw mut buf as *mut ::core::ffi::c_char);
    let mut is_ambiwidth: bool = utf_ambiguous_width(&raw mut buf as *mut ::core::ffi::c_char);
    if is_doublewidth as ::core::ffi::c_int != 0
        && (is_ambiwidth as ::core::ffi::c_int != 0 || utf_char2cells(c) == 1 as ::core::ffi::c_int)
    {
        is_ambiwidth = true_0 != 0;
        update_attrs(tui, (*cell).attr as ::core::ffi::c_int);
        print_spaces(tui, 2 as ::core::ffi::c_int);
        cursor_goto(tui, row, col);
    }
    print_cell(tui, &raw mut buf as *mut ::core::ffi::c_char, (*cell).attr);
    if is_ambiwidth {
        (*grid).row = -1 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn clear_region(
    mut tui: *mut TUIData,
    mut top: ::core::ffi::c_int,
    mut bot: ::core::ffi::c_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut attr_id: ::core::ffi::c_int,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if (*tui).set_default_colors {
        update_attrs(tui, attr_id);
    } else {
        terminfo_out(tui, kTerm_exit_attribute_mode);
    }
    if (*tui).can_clear_attr as ::core::ffi::c_int != 0
        && left == 0 as ::core::ffi::c_int
        && right == (*tui).width
        && bot == (*tui).height
    {
        if top == 0 as ::core::ffi::c_int {
            terminfo_out(tui, kTerm_clear_screen);
            ugrid_goto(grid, top, left);
        } else {
            cursor_goto(tui, top, 0 as ::core::ffi::c_int);
            terminfo_out(tui, kTerm_clr_eos);
        }
    } else {
        let mut width: ::core::ffi::c_int = right - left;
        let mut row: ::core::ffi::c_int = top;
        while row < bot {
            cursor_goto(tui, row, left);
            if (*tui).can_clear_attr as ::core::ffi::c_int != 0 && right == (*tui).width {
                terminfo_out(tui, kTerm_clr_eol);
            } else if (*tui).can_erase_chars as ::core::ffi::c_int != 0
                && (*tui).can_clear_attr as ::core::ffi::c_int != 0
                && width >= 5 as ::core::ffi::c_int
            {
                terminfo_print_num(
                    tui,
                    kTerm_erase_chars,
                    width,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            } else {
                print_spaces(tui, width);
            }
            row += 1;
        }
    };
}
unsafe extern "C" fn set_scroll_region(
    mut tui: *mut TUIData,
    mut top: ::core::ffi::c_int,
    mut bot: ::core::ffi::c_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    terminfo_print_num(
        tui,
        kTerm_change_scroll_region,
        top,
        bot,
        0 as ::core::ffi::c_int,
    );
    if left != 0 as ::core::ffi::c_int || right != (*tui).width - 1 as ::core::ffi::c_int {
        tui_set_term_mode(tui, kTermModeLeftAndRightMargins, true_0 != 0);
        terminfo_print_num(
            tui,
            kTerm_set_lr_margin,
            left,
            right,
            0 as ::core::ffi::c_int,
        );
    }
    (*grid).row = -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn reset_scroll_region(mut tui: *mut TUIData, mut fullwidth: bool) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if !(*tui).terminfo_ext.reset_scroll_region.is_null() {
        out_len(tui, (*tui).terminfo_ext.reset_scroll_region);
    } else {
        terminfo_print_num(
            tui,
            kTerm_change_scroll_region,
            0 as ::core::ffi::c_int,
            (*tui).height - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    if !fullwidth {
        terminfo_print_num(
            tui,
            kTerm_set_lr_margin,
            0 as ::core::ffi::c_int,
            (*tui).width - 1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        tui_set_term_mode(tui, kTermModeLeftAndRightMargins, false_0 != 0);
    }
    (*grid).row = -1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tui_grid_resize(
    mut tui: *mut TUIData,
    mut _g: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    ugrid_resize(
        grid,
        width as ::core::ffi::c_int,
        height as ::core::ffi::c_int,
    );
    let mut i: size_t = 0 as size_t;
    while i < (*tui).invalid_regions.size {
        let mut r: *mut Rect = (*tui).invalid_regions.items.offset(i as isize);
        (*r).bot = if (*r).bot < (*grid).height {
            (*r).bot
        } else {
            (*grid).height
        };
        (*r).right = if (*r).right < (*grid).width {
            (*r).right
        } else {
            (*grid).width
        };
        i = i.wrapping_add(1);
    }
    if (*tui).pending_resize_events == 0 as ::core::ffi::c_int && !(*tui).is_starting {
        out_printf(
            tui,
            64 as size_t,
            b"\x1B[8;%d;%dt\0".as_ptr() as *const ::core::ffi::c_char,
            height as ::core::ffi::c_int,
            width as ::core::ffi::c_int,
        );
    } else {
        (*tui).pending_resize_events = if (*tui).pending_resize_events > 0 as ::core::ffi::c_int {
            (*tui).pending_resize_events - 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        (*grid).row = -1 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn tui_grid_clear(mut tui: *mut TUIData, mut _g: Integer) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    ugrid_clear(grid);
    schar_cache_clear_if_full();
    (*tui).invalid_regions.size = 0 as size_t;
    clear_region(
        tui,
        0 as ::core::ffi::c_int,
        (*tui).height,
        0 as ::core::ffi::c_int,
        (*tui).width,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_grid_cursor_goto(
    mut tui: *mut TUIData,
    mut _grid: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    (*tui).row = row as ::core::ffi::c_int;
    (*tui).col = col as ::core::ffi::c_int;
}
unsafe extern "C" fn tui_cursor_decode_shape(
    mut shape_str: *const ::core::ffi::c_char,
) -> CursorShape {
    let mut shape: CursorShape = SHAPE_BLOCK;
    if strequal(shape_str, b"block\0".as_ptr() as *const ::core::ffi::c_char) {
        shape = SHAPE_BLOCK;
    } else if strequal(
        shape_str,
        b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        shape = SHAPE_VER;
    } else if strequal(
        shape_str,
        b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        shape = SHAPE_HOR;
    } else {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_cursor_decode_shape\0".as_ptr() as *const ::core::ffi::c_char,
            1281 as ::core::ffi::c_int,
            true_0 != 0,
            b"Unknown shape value '%s'\0".as_ptr() as *const ::core::ffi::c_char,
            shape_str,
        );
        shape = SHAPE_BLOCK;
    }
    return shape;
}
unsafe extern "C" fn tui_cursor_reset_style(mut tui: *mut TUIData) {
    terminfo_out(tui, kTerm_reset_cursor_style);
}
unsafe extern "C" fn decode_cursor_entry(mut args: Dict) -> cursorentry_T {
    let mut r: cursorentry_T = (*shape_table.ptr())[0 as ::core::ffi::c_int as usize];
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        let mut key: *mut ::core::ffi::c_char = (*args.items.offset(i as isize)).key.data;
        let mut value: Object = (*args.items.offset(i as isize)).value;
        if strequal(
            key,
            b"cursor_shape\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            r.shape =
                tui_cursor_decode_shape((*args.items.offset(i as isize)).value.data.string.data);
        } else if strequal(key, b"blinkon\0".as_ptr() as *const ::core::ffi::c_char) {
            r.blinkon = value.data.integer as ::core::ffi::c_int;
        } else if strequal(key, b"blinkoff\0".as_ptr() as *const ::core::ffi::c_char) {
            r.blinkoff = value.data.integer as ::core::ffi::c_int;
        } else if strequal(key, b"attr_id\0".as_ptr() as *const ::core::ffi::c_char) {
            r.id = value.data.integer as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn tui_mode_info_set(
    mut tui: *mut TUIData,
    mut guicursor_enabled: bool,
    mut args: Array,
) {
    cursor_style_enabled.set(guicursor_enabled);
    if !guicursor_enabled {
        tui_cursor_reset_style(tui);
        return;
    }
    '_c2rust_label: {
        if args.size != 0 {
        } else {
            __assert_fail(
                b"args.size\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1327 as ::core::ffi::c_uint,
                b"void tui_mode_info_set(TUIData *, _Bool, Array)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        '_c2rust_label_0: {
            if (*args.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
                == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"args.items[i].type == kObjectTypeDict\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1331 as ::core::ffi::c_uint,
                    b"void tui_mode_info_set(TUIData *, _Bool, Array)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut r: cursorentry_T = decode_cursor_entry((*args.items.offset(i as isize)).data.dict);
        (*tui).cursor_shapes[i as usize] = r;
        i = i.wrapping_add(1);
    }
    tui_set_mode(tui, (*tui).showing_mode);
}
#[no_mangle]
pub unsafe extern "C" fn tui_update_menu(mut _tui: *mut TUIData) {}
#[no_mangle]
pub unsafe extern "C" fn tui_busy_start(mut tui: *mut TUIData) {
    (*tui).busy = true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn tui_busy_stop(mut tui: *mut TUIData) {
    (*tui).busy = false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn tui_mouse_on(mut tui: *mut TUIData) {
    if !(*tui).mouse_enabled {
        tui_set_term_mode(tui, kTermModeMouseButtonEvent, true_0 != 0);
        tui_set_term_mode(tui, kTermModeMouseSGRExt, true_0 != 0);
        if (*tui).mouse_move_enabled {
            tui_set_term_mode(tui, kTermModeMouseAnyEvent, true_0 != 0);
        }
        (*tui).mouse_enabled = true_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tui_mouse_off(mut tui: *mut TUIData) {
    if (*tui).mouse_enabled {
        if (*tui).mouse_move_enabled {
            tui_set_term_mode(tui, kTermModeMouseAnyEvent, false_0 != 0);
        }
        tui_set_term_mode(tui, kTermModeMouseButtonEvent, false_0 != 0);
        tui_set_term_mode(tui, kTermModeMouseSGRExt, false_0 != 0);
        (*tui).mouse_enabled = false_0 != 0;
    }
}
unsafe extern "C" fn tui_set_mode(mut tui: *mut TUIData, mut mode: ModeShape) {
    if !cursor_style_enabled.get() {
        tui_cursor_reset_style(tui);
        return;
    }
    let mut c: cursorentry_T = (*tui).cursor_shapes[mode as usize];
    if c.id != 0 as ::core::ffi::c_int
        && c.id < (*tui).attrs.size as ::core::ffi::c_int
        && (*tui).rgb as ::core::ffi::c_int != 0
    {
        let mut aep: HlAttrs = *(*tui).attrs.items.offset(c.id as isize);
        (*tui).want_invisible = aep.hl_blend == 100 as int32_t;
        if !(*tui).want_invisible
            && aep.rgb_ae_attr & HL_INVERSE as ::core::ffi::c_int as int32_t != 0
        {
            terminfo_out(tui, kTerm_reset_cursor_color);
        } else if !(*tui).want_invisible && aep.rgb_bg_color >= 0 as RgbValue {
            let mut params: [TPVAR; 9] = [
                TPVAR {
                    num: 0 as ::core::ffi::c_long,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                TPVAR {
                    num: 0,
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
            ];
            let mut hexbuf: [::core::ffi::c_char; 8] = [0; 8];
            if (*tui).set_cursor_color_as_str {
                snprintf(
                    &raw mut hexbuf as *mut ::core::ffi::c_char,
                    (7 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t,
                    b"#%06x\0".as_ptr() as *const ::core::ffi::c_char,
                    aep.rgb_bg_color,
                );
                params[0 as ::core::ffi::c_int as usize].string =
                    &raw mut hexbuf as *mut ::core::ffi::c_char;
            } else {
                params[0 as ::core::ffi::c_int as usize].num =
                    aep.rgb_bg_color as ::core::ffi::c_long;
            }
            terminfo_print(tui, kTerm_set_cursor_color, &raw mut params as *mut TPVAR);
            (*tui).cursor_has_color = true_0 != 0;
        }
    } else if c.id == 0 as ::core::ffi::c_int
        && ((*tui).want_invisible as ::core::ffi::c_int != 0
            || (*tui).cursor_has_color as ::core::ffi::c_int != 0)
    {
        (*tui).want_invisible = false_0 != 0;
        (*tui).cursor_has_color = false_0 != 0;
        terminfo_out(tui, kTerm_reset_cursor_color);
    }
    let mut shape: ::core::ffi::c_int = 0;
    match c.shape as ::core::ffi::c_uint {
        0 => {
            shape = 1 as ::core::ffi::c_int;
        }
        1 => {
            shape = 3 as ::core::ffi::c_int;
        }
        2 => {
            shape = 5 as ::core::ffi::c_int;
        }
        _ => {}
    }
    terminfo_print_num(
        tui,
        kTerm_set_cursor_style,
        shape
            + (c.blinkon == 0 as ::core::ffi::c_int || c.blinkoff == 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_mode_change(
    mut tui: *mut TUIData,
    mut _mode: String_0,
    mut mode_idx: Integer,
) {
    if (*tui).out_isatty as ::core::ffi::c_int != 0
        && (*tui).is_starting as ::core::ffi::c_int != 0
        && !stdin_isatty.get()
    {
        let mut ret: ::core::ffi::c_int =
            uv_tty_set_mode(&raw mut (*tui).output_handle.tty, UV_TTY_MODE_NORMAL);
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"tui_mode_change\0".as_ptr() as *const ::core::ffi::c_char,
                1436 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_tty_set_mode failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
        ret = uv_tty_set_mode(&raw mut (*tui).output_handle.tty, UV_TTY_MODE_IO);
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"tui_mode_change\0".as_ptr() as *const ::core::ffi::c_char,
                1440 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_tty_set_mode failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
    }
    tui_set_mode(tui, mode_idx as ModeShape);
    if (*tui).is_starting {
        if (*tui).verbose >= 3 as Integer {
            show_verbose_terminfo(tui);
        }
    }
    (*tui).is_starting = false_0 != 0;
    (*tui).showing_mode = mode_idx as ModeShape;
}
#[no_mangle]
pub unsafe extern "C" fn tui_grid_scroll(
    mut tui: *mut TUIData,
    mut _g: Integer,
    mut startrow: Integer,
    mut endrow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut rows: Integer,
    mut _cols: Integer,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    let mut top: ::core::ffi::c_int = startrow as ::core::ffi::c_int;
    let mut bot: ::core::ffi::c_int = endrow as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    let mut left: ::core::ffi::c_int = startcol as ::core::ffi::c_int;
    let mut right: ::core::ffi::c_int = endcol as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    let mut fullwidth: bool =
        left == 0 as ::core::ffi::c_int && right == (*tui).width - 1 as ::core::ffi::c_int;
    let mut full_screen_scroll: bool = fullwidth as ::core::ffi::c_int != 0
        && top == 0 as ::core::ffi::c_int
        && bot == (*tui).height - 1 as ::core::ffi::c_int;
    ugrid_scroll(grid, top, bot, left, right, rows as ::core::ffi::c_int);
    let mut has_lr_margins: bool = (*tui).has_left_and_right_margin_mode as ::core::ffi::c_int != 0
        && (*tui).can_set_lr_margin as ::core::ffi::c_int != 0;
    let mut can_scroll: bool = (*tui).can_scroll as ::core::ffi::c_int != 0
        && (full_screen_scroll as ::core::ffi::c_int != 0
            || (*tui).can_change_scroll_region as ::core::ffi::c_int != 0
                && (left == 0 as ::core::ffi::c_int
                    && right == (*tui).width - 1 as ::core::ffi::c_int
                    || has_lr_margins as ::core::ffi::c_int != 0));
    if can_scroll {
        if !full_screen_scroll {
            set_scroll_region(tui, top, bot, left, right);
        }
        cursor_goto(tui, top, left);
        update_attrs(tui, 0 as ::core::ffi::c_int);
        if rows > 0 as Integer {
            if rows == 1 as Integer {
                terminfo_out(tui, kTerm_delete_line);
            } else {
                terminfo_print_num(
                    tui,
                    kTerm_parm_delete_line,
                    rows as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            }
        } else if rows == -1 as Integer {
            terminfo_out(tui, kTerm_insert_line);
        } else {
            terminfo_print_num(
                tui,
                kTerm_parm_insert_line,
                -(rows as ::core::ffi::c_int),
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if !full_screen_scroll {
            reset_scroll_region(tui, fullwidth);
        }
    } else {
        if rows > 0 as Integer {
            endrow = endrow - rows;
        } else {
            startrow = startrow - rows;
        }
        invalidate(
            tui,
            startrow as ::core::ffi::c_int,
            endrow as ::core::ffi::c_int,
            startcol as ::core::ffi::c_int,
            endcol as ::core::ffi::c_int,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn tui_add_url(
    mut _tui: *mut TUIData,
    mut url: *const ::core::ffi::c_char,
) -> int32_t {
    if url.is_null() {
        return -1 as int32_t;
    }
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = mh_put_cstr_t(urls.ptr(), url as cstr_t, &raw mut status);
    if status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint {
        *(*urls.ptr()).keys.offset(k as isize) = xstrdup(url) as cstr_t;
    }
    return k as int32_t;
}
#[no_mangle]
pub unsafe extern "C" fn tui_hl_attr_define(
    mut tui: *mut TUIData,
    mut id: Integer,
    mut attrs: HlAttrs,
    mut cterm_attrs: HlAttrs,
    mut _info: Array,
) {
    attrs.cterm_ae_attr = cterm_attrs.cterm_ae_attr;
    attrs.cterm_fg_color = cterm_attrs.cterm_fg_color;
    attrs.cterm_bg_color = cterm_attrs.cterm_bg_color;
    if (*tui).attrs.capacity <= id as size_t {
        (*tui).attrs.size = (id as size_t).wrapping_add(1 as size_t);
        (*tui).attrs.capacity = (*tui).attrs.size;
        (*tui).attrs.capacity = (*tui).attrs.capacity.wrapping_sub(1);
        (*tui).attrs.capacity |= (*tui).attrs.capacity >> 1 as ::core::ffi::c_int;
        (*tui).attrs.capacity |= (*tui).attrs.capacity >> 2 as ::core::ffi::c_int;
        (*tui).attrs.capacity |= (*tui).attrs.capacity >> 4 as ::core::ffi::c_int;
        (*tui).attrs.capacity |= (*tui).attrs.capacity >> 8 as ::core::ffi::c_int;
        (*tui).attrs.capacity |= (*tui).attrs.capacity >> 16 as ::core::ffi::c_int;
        (*tui).attrs.capacity = (*tui).attrs.capacity.wrapping_add(1);
        (*tui).attrs.capacity = (*tui).attrs.capacity;
        (*tui).attrs.items = xrealloc(
            (*tui).attrs.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HlAttrs>().wrapping_mul((*tui).attrs.capacity),
        ) as *mut HlAttrs;
    } else {
        if (*tui).attrs.size <= id as size_t {
            (*tui).attrs.size = (id as size_t).wrapping_add(1 as size_t);
        } else {
        };
    };
    *(*tui).attrs.items.offset(id as size_t as isize) = attrs;
}
#[no_mangle]
pub unsafe extern "C" fn tui_bell(mut tui: *mut TUIData) {
    out(
        tui,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_visual_bell(mut tui: *mut TUIData) {
    if (*tui).screen_or_tmux {
        out(
            tui,
            b"\x1Bg\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    } else {
        out(
            tui,
            b"\x1B[?5h\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        flush_buf(tui);
        uv_sleep(100 as ::core::ffi::c_uint);
        out(
            tui,
            b"\x1B[?5l\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
    }
    flush_buf(tui);
}
#[no_mangle]
pub unsafe extern "C" fn tui_default_colors_set(
    mut tui: *mut TUIData,
    mut rgb_fg: Integer,
    mut rgb_bg: Integer,
    mut rgb_sp: Integer,
    mut cterm_fg: Integer,
    mut cterm_bg: Integer,
) {
    (*tui).clear_attrs.rgb_fg_color = rgb_fg as RgbValue;
    (*tui).clear_attrs.rgb_bg_color = rgb_bg as RgbValue;
    (*tui).clear_attrs.rgb_sp_color = rgb_sp as RgbValue;
    (*tui).clear_attrs.cterm_fg_color = cterm_fg as int16_t;
    (*tui).clear_attrs.cterm_bg_color = cterm_bg as int16_t;
    (*tui).print_attr_id = -1 as ::core::ffi::c_int;
    (*tui).set_default_colors = true_0 != 0;
    invalidate(
        tui,
        0 as ::core::ffi::c_int,
        (*tui).grid.height,
        0 as ::core::ffi::c_int,
        (*tui).grid.width,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tui_ui_send(mut tui: *mut TUIData, mut content: String_0) {
    let mut req: uv_write_t = uv_write_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        cb: None,
        send_handle: ::core::ptr::null_mut::<uv_stream_t>(),
        handle: ::core::ptr::null_mut::<uv_stream_t>(),
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        write_index: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        nbufs: 0,
        error: 0,
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut buf: uv_buf_t = uv_buf_t {
        base: content.data,
        len: content.size,
    };
    let mut ret: ::core::ffi::c_int = uv_write(
        &raw mut req,
        &raw mut (*tui).output_handle as *mut uv_stream_t,
        &raw mut buf as *const uv_buf_t,
        1 as ::core::ffi::c_uint,
        None,
    );
    if ret != 0 {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            1583 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_write failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
            uv_strerror(ret),
        );
    }
    uv_run(&raw mut (*tui).write_loop, UV_RUN_DEFAULT);
}
#[no_mangle]
pub unsafe extern "C" fn tui_flush(mut tui: *mut TUIData) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    let mut nrevents: size_t = loop_size((*tui).loop_0);
    if nrevents > TOO_MANY_EVENTS as size_t {
        logmsg(
            LOGLVL_WRN,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_flush\0".as_ptr() as *const ::core::ffi::c_char,
            1597 as ::core::ffi::c_int,
            true_0 != 0,
            b"TUI event-queue flooded (thread_events=%zu); purging\0".as_ptr()
                as *const ::core::ffi::c_char,
            nrevents,
        );
        loop_purge((*tui).loop_0);
        tui_busy_stop(tui);
    }
    while (*tui).invalid_regions.size != 0 {
        (*tui).invalid_regions.size = (*tui).invalid_regions.size.wrapping_sub(1);
        let mut r: Rect = *(*tui)
            .invalid_regions
            .items
            .offset((*tui).invalid_regions.size as isize);
        '_c2rust_label: {
            if r.bot <= (*grid).height && r.right <= (*grid).width {
            } else {
                __assert_fail(
                    b"r.bot <= grid->height && r.right <= grid->width\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1609 as ::core::ffi::c_uint,
                    b"void tui_flush(TUIData *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut row: ::core::ffi::c_int = r.top;
        while row < r.bot {
            let mut clear_attr: ::core::ffi::c_int = (*(*(*grid).cells.offset(row as isize))
                .offset((r.right - 1 as ::core::ffi::c_int) as isize))
            .attr as ::core::ffi::c_int;
            let mut clear_col: ::core::ffi::c_int = 0;
            clear_col = r.right;
            while clear_col > 0 as ::core::ffi::c_int {
                let mut cell: *mut UCell = (*(*grid).cells.offset(row as isize))
                    .offset((clear_col - 1 as ::core::ffi::c_int) as isize);
                if !((*cell).data == ' ' as ::core::ffi::c_int as schar_T
                    && (*cell).attr == clear_attr as sattr_T)
                {
                    break;
                }
                clear_col -= 1;
            }
            let mut row_cells: *mut UCell = *(*grid).cells.offset(row as isize);
            let mut curcol: ::core::ffi::c_int = r.left;
            while curcol < clear_col {
                let mut cell_0: *mut UCell = row_cells.offset(curcol as isize);
                print_cell_at_pos(
                    tui,
                    row,
                    curcol,
                    cell_0,
                    curcol < clear_col - 1 as ::core::ffi::c_int
                        && (*cell_0.offset(1 as ::core::ffi::c_int as isize)).data
                            == '\0' as schar_T,
                );
                curcol += 1;
            }
            if clear_col < r.right {
                clear_region(
                    tui,
                    row,
                    row + 1 as ::core::ffi::c_int,
                    clear_col,
                    r.right,
                    clear_attr,
                );
            }
            row += 1;
        }
    }
    cursor_goto(tui, (*tui).row, (*tui).col);
    flush_buf(tui);
}
unsafe extern "C" fn show_verbose_terminfo(mut tui: *mut TUIData) {
    let mut chunks: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut chunks__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 3];
    chunks.capacity = 3 as size_t;
    chunks.items = &raw mut chunks__items as *mut Object;
    let mut title: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut title__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 2];
    title.capacity = 2 as size_t;
    title.items = &raw mut title__items as *mut Object;
    let c2rust_fresh5 = title.size;
    title.size = title.size.wrapping_add(1);
    *title.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(
                b"\n\n--- Terminal info --- {{{\n\0".as_ptr() as *const ::core::ffi::c_char
            ),
        },
    };
    let c2rust_fresh6 = title.size;
    title.size = title.size.wrapping_add(1);
    *title.items.offset(c2rust_fresh6 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(b"Title\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let c2rust_fresh7 = chunks.size;
    chunks.size = chunks.size.wrapping_add(1);
    *chunks.items.offset(c2rust_fresh7 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_13 { array: title },
    };
    let mut info: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut info__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 1];
    info.capacity = 1 as size_t;
    info.items = &raw mut info__items as *mut Object;
    let mut str: String_0 =
        terminfo_info_msg(&raw mut (*tui).ti, (*tui).term, (*tui).terminfo_found_in_db);
    let c2rust_fresh8 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh8 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 { string: str },
    };
    let c2rust_fresh9 = chunks.size;
    chunks.size = chunks.size.wrapping_add(1);
    *chunks.items.offset(c2rust_fresh9 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_13 { array: info },
    };
    let mut end_fold: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut end_fold__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 2];
    end_fold.capacity = 2 as size_t;
    end_fold.items = &raw mut end_fold__items as *mut Object;
    let c2rust_fresh10 = end_fold.size;
    end_fold.size = end_fold.size.wrapping_add(1);
    *end_fold.items.offset(c2rust_fresh10 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(b"}}}\n\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let c2rust_fresh11 = end_fold.size;
    end_fold.size = end_fold.size.wrapping_add(1);
    *end_fold.items.offset(c2rust_fresh11 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_13 {
            string: cstr_as_string(b"Title\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let c2rust_fresh12 = chunks.size;
    chunks.size = chunks.size.wrapping_add(1);
    *chunks.items.offset(c2rust_fresh12 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_13 { array: end_fold },
    };
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_13 { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh13 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh13 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed_13 { array: chunks },
    };
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed_13 { boolean: true },
    };
    let mut opts: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut opts__items: [KeyValuePair; 1] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_13 { boolean: false },
        },
    }; 1];
    opts.capacity = 1 as size_t;
    opts.items = &raw mut opts__items as *mut KeyValuePair;
    let c2rust_fresh15 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh15 as isize) = key_value_pair {
        key: cstr_as_string(b"verbose\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed_13 { boolean: true },
        },
    };
    let c2rust_fresh16 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh16 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed_13 { dict: opts },
    };
    rpc_send_event(
        ui_client_channel_id.get(),
        b"nvim_echo\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
    xfree(str.data as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tui_suspend(mut tui: *mut TUIData) {
    ui_client_detach();
    (*tui).mouse_enabled_save = (*tui).mouse_enabled;
    (*tui).input.callbacks.primary_device_attr =
        Some(tui_suspend_cb as unsafe extern "C" fn(*mut TUIData) -> ())
            as Option<unsafe extern "C" fn(*mut TUIData) -> ()>;
    terminfo_disable(tui);
}
unsafe extern "C" fn tui_suspend_cb(mut tui: *mut TUIData) {
    tui_terminal_stop(tui);
    stream_set_blocking((*tui).input.in_fd, true_0 != 0);
    kill(0 as __pid_t, SIGSTOP);
    tui_terminal_start(tui);
    tui_terminal_after_startup(tui);
    if (*tui).mouse_enabled_save {
        tui_mouse_on(tui);
    }
    stream_set_blocking((*tui).input.in_fd, false_0 != 0);
    ui_client_attach((*tui).width, (*tui).height, (*tui).term, (*tui).rgb);
}
#[no_mangle]
pub unsafe extern "C" fn tui_set_title(mut tui: *mut TUIData, mut title: String_0) {
    if !(*tui).can_set_title {
        return;
    }
    let mut too_long: bool = title.size > 4096 as size_t;
    if too_long {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_set_title\0".as_ptr() as *const ::core::ffi::c_char,
            1703 as ::core::ffi::c_int,
            true_0 != 0,
            b"set_title: title string too long!\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if title.size > 0 as size_t && !too_long {
        if !(*tui).title_enabled {
            out(
                tui,
                b"\x1B[22;0t\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
            (*tui).title_enabled = true_0 != 0;
        }
        if ::core::mem::size_of::<[::core::ffi::c_char; 65535]>()
            .wrapping_sub((*tui).bufpos as usize)
            < title
                .size
                .wrapping_add((2 as ::core::ffi::c_int * TERMINFO_SEQ_LIMIT) as size_t)
        {
            flush_buf(tui);
        }
        terminfo_out(tui, kTerm_to_status_line);
        out(tui, title.data, title.size);
        terminfo_out(tui, kTerm_from_status_line);
    } else if (*tui).title_enabled {
        out(
            tui,
            b"\x1B[23;0t\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        );
        (*tui).title_enabled = false_0 != 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tui_set_icon(mut _tui: *mut TUIData, mut _icon: String_0) {}
#[no_mangle]
pub unsafe extern "C" fn tui_screenshot(mut tui: *mut TUIData, mut path: String_0) {
    let mut f: *mut FILE =
        fopen(path.data, b"w\0".as_ptr() as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        return;
    }
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    flush_buf(tui);
    (*grid).row = 0 as ::core::ffi::c_int;
    (*grid).col = 0 as ::core::ffi::c_int;
    (*tui).screenshot = f;
    fprintf(
        f,
        b"%d,%d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*grid).height,
        (*grid).width,
    );
    terminfo_out(tui, kTerm_clear_screen);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*grid).height {
        cursor_goto(tui, i, 0 as ::core::ffi::c_int);
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < (*grid).width {
            let mut cell: UCell = *(*(*grid).cells.offset(i as isize)).offset(j as isize);
            let mut buf: [::core::ffi::c_char; 32] = [0; 32];
            schar_get(&raw mut buf as *mut ::core::ffi::c_char, cell.data);
            print_cell(tui, &raw mut buf as *mut ::core::ffi::c_char, cell.attr);
            j += 1;
        }
        i += 1;
    }
    flush_buf(tui);
    (*tui).screenshot = ::core::ptr::null_mut::<FILE>();
    fclose(f);
}
#[no_mangle]
pub unsafe extern "C" fn tui_option_set(
    mut tui: *mut TUIData,
    mut name: String_0,
    mut value: Object,
) {
    if strequal(
        name.data,
        b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if (*tui).mouse_move_enabled as ::core::ffi::c_int
            != value.data.boolean as ::core::ffi::c_int
        {
            if (*tui).mouse_enabled {
                tui_mouse_off(tui);
                (*tui).mouse_move_enabled = value.data.boolean as bool;
                tui_mouse_on(tui);
            } else {
                (*tui).mouse_move_enabled = value.data.boolean as bool;
            }
        }
    } else if strequal(
        name.data,
        b"termguicolors\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*tui).rgb = value.data.boolean as bool;
        (*tui).print_attr_id = -1 as ::core::ffi::c_int;
        invalidate(
            tui,
            0 as ::core::ffi::c_int,
            (*tui).grid.height,
            0 as ::core::ffi::c_int,
            (*tui).grid.width,
        );
        if ui_client_channel_id.get() != 0 {
            let mut args: Array = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            let mut args__items: [Object; 2] = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_13 { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            let c2rust_fresh18 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh18 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_13 {
                    string: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
                },
            };
            let c2rust_fresh19 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh19 as isize) = object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_13 {
                    boolean: value.data.boolean,
                },
            };
            rpc_send_event(
                ui_client_channel_id.get(),
                b"nvim_ui_set_option\0".as_ptr() as *const ::core::ffi::c_char,
                args,
            );
        }
    } else if strequal(
        name.data,
        b"ttimeout\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*tui).input.ttimeout = value.data.boolean as bool;
    } else if strequal(
        name.data,
        b"ttimeoutlen\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*tui).input.ttimeoutlen = value.data.integer;
    } else if strequal(
        name.data,
        b"verbose\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*tui).verbose = value.data.integer;
    } else if strequal(
        name.data,
        b"termsync\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*tui).sync_output = value.data.boolean as bool;
    }
}
#[no_mangle]
pub unsafe extern "C" fn tui_chdir(mut _tui: *mut TUIData, mut path: String_0) {
    let mut err: ::core::ffi::c_int = uv_chdir(path.data);
    if err != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_chdir\0".as_ptr() as *const ::core::ffi::c_char,
            1799 as ::core::ffi::c_int,
            true_0 != 0,
            b"Failed to chdir to %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            path.data,
            uv_strerror(err),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn tui_raw_line(
    mut tui: *mut TUIData,
    mut _g: Integer,
    mut linerow: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    let mut c: Integer = startcol;
    while c < endcol {
        (*(*(*grid).cells.offset(linerow as isize)).offset(c as isize)).data =
            *chunk.offset((c - startcol) as isize);
        '_c2rust_label: {
            if (*attrs.offset((c - startcol) as isize) as size_t) < (*tui).attrs.size {
            } else {
                __assert_fail(
                    b"(size_t)attrs[c - startcol] < kv_size(tui->attrs)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/tui/tui.rs\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    1810 as ::core::ffi::c_uint,
                    b"void tui_raw_line(TUIData *, Integer, Integer, Integer, Integer, Integer, Integer, LineFlags, const schar_T *, const sattr_T *)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        (*(*(*grid).cells.offset(linerow as isize)).offset(c as isize)).attr =
            *attrs.offset((c - startcol) as isize);
        c += 1;
    }
    let mut row_cells: *mut UCell = *(*grid).cells.offset(linerow as ::core::ffi::c_int as isize);
    let mut curcol: ::core::ffi::c_int = startcol as ::core::ffi::c_int;
    while curcol < endcol as ::core::ffi::c_int {
        let mut cell: *mut UCell = row_cells.offset(curcol as isize);
        print_cell_at_pos(
            tui,
            linerow as ::core::ffi::c_int,
            curcol,
            cell,
            (curcol as Integer) < endcol - 1 as Integer
                && (*cell.offset(1 as ::core::ffi::c_int as isize)).data == '\0' as schar_T,
        );
        curcol += 1;
    }
    if clearcol > endcol {
        ugrid_clear_chunk(
            grid,
            linerow as ::core::ffi::c_int,
            endcol as ::core::ffi::c_int,
            clearcol as ::core::ffi::c_int,
            clearattr as sattr_T,
        );
        clear_region(
            tui,
            linerow as ::core::ffi::c_int,
            linerow as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            endcol as ::core::ffi::c_int,
            clearcol as ::core::ffi::c_int,
            clearattr as ::core::ffi::c_int,
        );
    }
    if flags as ::core::ffi::c_int & kLineFlagWrap as ::core::ffi::c_int != 0
        && (*tui).width == (*grid).width
        && (linerow + 1 as Integer) < (*grid).height as Integer
    {
        if endcol != (*grid).width as Integer {
            let mut size: ::core::ffi::c_int = if (*(*(*grid).cells.offset(linerow as isize))
                .offset(((*grid).width - 1 as ::core::ffi::c_int) as isize))
            .data
                == NUL as schar_T
            {
                2 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            };
            print_cell_at_pos(
                tui,
                linerow as ::core::ffi::c_int,
                (*grid).width - size,
                (*(*grid).cells.offset(linerow as isize)).offset(((*grid).width - size) as isize),
                size == 2 as ::core::ffi::c_int,
            );
        }
        final_column_wrap(tui);
    }
}
unsafe extern "C" fn invalidate(
    mut tui: *mut TUIData,
    mut top: ::core::ffi::c_int,
    mut bot: ::core::ffi::c_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
) {
    let mut intersects: *mut Rect = ::core::ptr::null_mut::<Rect>();
    let mut i: size_t = 0 as size_t;
    while i < (*tui).invalid_regions.size {
        let mut r: *mut Rect = (*tui).invalid_regions.items.offset(i as isize);
        if !(top > (*r).bot || bot < (*r).top) && !(left > (*r).right || right < (*r).left) {
            intersects = r;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if !intersects.is_null() {
        (*intersects).top = if top < (*intersects).top {
            top
        } else {
            (*intersects).top
        };
        (*intersects).bot = if bot > (*intersects).bot {
            bot
        } else {
            (*intersects).bot
        };
        (*intersects).left = if left < (*intersects).left {
            left
        } else {
            (*intersects).left
        };
        (*intersects).right = if right > (*intersects).right {
            right
        } else {
            (*intersects).right
        };
    } else {
        if (*tui).invalid_regions.size == (*tui).invalid_regions.capacity {
            (*tui).invalid_regions.capacity = if (*tui).invalid_regions.capacity != 0 {
                (*tui).invalid_regions.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*tui).invalid_regions.items = xrealloc(
                (*tui).invalid_regions.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Rect>().wrapping_mul((*tui).invalid_regions.capacity),
            ) as *mut Rect;
        } else {
        };
        let c2rust_fresh17 = (*tui).invalid_regions.size;
        (*tui).invalid_regions.size = (*tui).invalid_regions.size.wrapping_add(1);
        *(*tui).invalid_regions.items.offset(c2rust_fresh17 as isize) = Rect {
            top: top,
            bot: bot,
            left: left,
            right: right,
        };
    };
}
#[no_mangle]
pub unsafe extern "C" fn tui_set_size(
    mut tui: *mut TUIData,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    (*tui).pending_resize_events += 1;
    (*tui).width = width;
    (*tui).height = height;
    ui_client_set_size(width, height);
}
#[no_mangle]
pub unsafe extern "C" fn tui_guess_size(mut tui: *mut TUIData) {
    let mut val: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut advance: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lines: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut columns: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !((*tui).out_isatty as ::core::ffi::c_int != 0
        && uv_tty_get_winsize(
            &raw mut (*tui).output_handle.tty,
            &raw mut width,
            &raw mut height,
        ) == 0)
    {
        val = ::core::ptr::null::<::core::ffi::c_char>();
        advance = 0;
        val = os_getenv_noalloc(b"LINES\0".as_ptr() as *const ::core::ffi::c_char);
        if !(!val.is_null()
            && sscanf(
                val,
                b"%d%n\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut height,
                &raw mut advance,
            ) != EOF
            && advance != 0
            && {
                val = os_getenv_noalloc(b"COLUMNS\0".as_ptr() as *const ::core::ffi::c_char);
                !val.is_null()
            }
            && sscanf(
                val,
                b"%d%n\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut width,
                &raw mut advance,
            ) != EOF
            && advance != 0)
        {
            height = (*tui).ti.lines;
            width = (*tui).ti.columns;
        }
    }
    if width <= 0 as ::core::ffi::c_int || height <= 0 as ::core::ffi::c_int {
        width = DFLT_COLS;
        height = DFLT_ROWS;
    }
    tui_set_size(tui, width, height);
    xfree(lines as *mut ::core::ffi::c_void);
    xfree(columns as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn out(
    mut tui: *mut TUIData,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut available: size_t =
        ::core::mem::size_of::<[::core::ffi::c_char; 65535]>().wrapping_sub((*tui).bufpos);
    if len > available {
        flush_buf(tui);
        if len > ::core::mem::size_of::<[::core::ffi::c_char; 65535]>() {
            (*tui).buf_to_flush = str as *mut ::core::ffi::c_char;
            (*tui).bufpos = len;
            flush_buf(tui);
            return;
        }
    }
    memcpy(
        (&raw mut (*tui).buf as *mut ::core::ffi::c_char).offset((*tui).bufpos as isize)
            as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    (*tui).bufpos = (*tui).bufpos.wrapping_add(len);
}
unsafe extern "C" fn out_len(mut tui: *mut TUIData, mut str: *const ::core::ffi::c_char) {
    if !str.is_null() {
        out(tui, str, strlen(str));
    }
}
#[no_mangle]
pub unsafe extern "C" fn out_printf(
    mut tui: *mut TUIData,
    mut limit: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    '_c2rust_label: {
        if limit <= ::core::mem::size_of::<[::core::ffi::c_char; 65535]>() {
        } else {
            __assert_fail(
                b"limit <= sizeof(tui->buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/tui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1951 as ::core::ffi::c_uint,
                b"void out_printf(TUIData *, size_t, const char *, ...)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut available: size_t =
        ::core::mem::size_of::<[::core::ffi::c_char; 65535]>().wrapping_sub((*tui).bufpos);
    if available < limit {
        flush_buf(tui);
    }
    let mut ap: ::core::ffi::VaListImpl;
    ap = c2rust_args.clone();
    let mut printed: ::core::ffi::c_int = vsnprintf(
        (&raw mut (*tui).buf as *mut ::core::ffi::c_char).offset((*tui).bufpos as isize),
        limit,
        fmt,
        ap.as_va_list(),
    );
    if printed > 0 as ::core::ffi::c_int {
        (*tui).bufpos = (*tui).bufpos.wrapping_add(printed as size_t);
    }
}
unsafe extern "C" fn terminfo_out(mut tui: *mut TUIData, mut what: TerminfoDef) {
    let mut null_params: [TPVAR; 9] = [
        TPVAR {
            num: 0 as ::core::ffi::c_long,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
    ];
    terminfo_print(tui, what, &raw mut null_params as *mut TPVAR);
}
unsafe extern "C" fn terminfo_print_num(
    mut tui: *mut TUIData,
    mut what: TerminfoDef,
    mut num1: ::core::ffi::c_int,
    mut num2: ::core::ffi::c_int,
    mut num3: ::core::ffi::c_int,
) {
    let mut params: [TPVAR; 9] = [
        TPVAR {
            num: 0 as ::core::ffi::c_long,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
    ];
    params[0 as ::core::ffi::c_int as usize].num = num1 as ::core::ffi::c_long;
    params[1 as ::core::ffi::c_int as usize].num = num2 as ::core::ffi::c_long;
    params[2 as ::core::ffi::c_int as usize].num = num3 as ::core::ffi::c_long;
    terminfo_print(tui, what, &raw mut params as *mut TPVAR);
}
unsafe extern "C" fn terminfo_print(
    mut tui: *mut TUIData,
    mut what: TerminfoDef,
    mut params: *mut TPVAR,
) {
    if what as ::core::ffi::c_uint >= kTermCount as ::core::ffi::c_int as ::core::ffi::c_uint {
        abort();
    }
    let mut str: *const ::core::ffi::c_char = (*tui).ti.defs[what as usize];
    if str.is_null() || *str as ::core::ffi::c_int == NUL {
        return;
    }
    if ::core::mem::size_of::<[::core::ffi::c_char; 65535]>().wrapping_sub((*tui).bufpos as usize)
        > TERMINFO_SEQ_LIMIT as usize
    {
        let mut copy_params: [TPVAR; 9] = [TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        }; 9];
        memcpy(
            &raw mut copy_params as *mut TPVAR as *mut ::core::ffi::c_void,
            params as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[TPVAR; 9]>(),
        );
        let mut len: size_t = terminfo_fmt(
            (&raw mut (*tui).buf as *mut ::core::ffi::c_char).offset((*tui).bufpos as isize),
            (&raw mut (*tui).buf as *mut ::core::ffi::c_char)
                .offset(::core::mem::size_of::<[::core::ffi::c_char; 65535]>() as isize),
            str,
            &raw mut copy_params as *mut TPVAR,
        );
        if len > 0 as size_t {
            (*tui).bufpos = (*tui).bufpos.wrapping_add(len);
            return;
        }
    }
    flush_buf(tui);
    let mut len_0: size_t = terminfo_fmt(
        (&raw mut (*tui).buf as *mut ::core::ffi::c_char).offset((*tui).bufpos as isize),
        (&raw mut (*tui).buf as *mut ::core::ffi::c_char)
            .offset(::core::mem::size_of::<[::core::ffi::c_char; 65535]>() as isize),
        str,
        params as *mut TPVAR,
    );
    if len_0 > 0 as size_t {
        (*tui).bufpos = (*tui).bufpos.wrapping_add(len_0);
    }
}
unsafe extern "C" fn terminfo_set_if_empty(
    mut tui: *mut TUIData,
    mut str: TerminfoDef,
    mut val: *const ::core::ffi::c_char,
) {
    if (*tui).ti.defs[str as usize].is_null() {
        (*tui).ti.defs[str as usize] = val;
    }
}
unsafe extern "C" fn terminfo_set_str(
    mut tui: *mut TUIData,
    mut str: TerminfoDef,
    mut val: *const ::core::ffi::c_char,
) {
    (*tui).ti.defs[str as usize] = val;
}
unsafe extern "C" fn term_has_truecolor(
    mut tui: *mut TUIData,
    mut colorterm: *const ::core::ffi::c_char,
) -> bool {
    if strequal(
        colorterm,
        b"truecolor\0".as_ptr() as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != 0
        || strequal(colorterm, b"24bit\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        return true_0 != 0;
    }
    if (*tui).ti.has_Tc_or_RGB {
        return true_0 != 0;
    }
    let mut setrgbf: bool =
        !(*tui).ti.defs[kTerm_set_rgb_foreground as ::core::ffi::c_int as usize].is_null();
    let mut setrgbb: bool =
        !(*tui).ti.defs[kTerm_set_rgb_background as ::core::ffi::c_int as usize].is_null();
    return setrgbf as ::core::ffi::c_int != 0 && setrgbb as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn patch_terminfo_bugs(
    mut tui: *mut TUIData,
    mut term: *const ::core::ffi::c_char,
    mut colorterm: *const ::core::ffi::c_char,
    mut vte_version: ::core::ffi::c_int,
    mut konsolev: ::core::ffi::c_int,
    mut iterm_env: bool,
    mut nsterm: bool,
) {
    let mut xterm_version: *mut ::core::ffi::c_char =
        os_getenv(b"XTERM_VERSION\0".as_ptr() as *const ::core::ffi::c_char);
    let mut xterm: bool =
        terminfo_is_term_family(term, b"xterm\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || nsterm as ::core::ffi::c_int != 0;
    let mut hterm: bool =
        terminfo_is_term_family(term, b"hterm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut kitty: bool = terminfo_is_term_family(
        term,
        b"xterm-kitty\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut linuxvt: bool =
        terminfo_is_term_family(term, b"linux\0".as_ptr() as *const ::core::ffi::c_char);
    let mut bsdvt: bool = terminfo_is_bsd_console(term);
    let mut rxvt: bool =
        terminfo_is_term_family(term, b"rxvt\0".as_ptr() as *const ::core::ffi::c_char);
    let mut teraterm: bool =
        terminfo_is_term_family(term, b"teraterm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut putty: bool =
        terminfo_is_term_family(term, b"putty\0".as_ptr() as *const ::core::ffi::c_char);
    let mut screen: bool =
        terminfo_is_term_family(term, b"screen\0".as_ptr() as *const ::core::ffi::c_char);
    let mut tmux: bool =
        terminfo_is_term_family(term, b"tmux\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || os_env_exists(
                b"TMUX\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0;
    let mut st: bool =
        terminfo_is_term_family(term, b"st\0".as_ptr() as *const ::core::ffi::c_char);
    let mut gnome: bool =
        terminfo_is_term_family(term, b"gnome\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || terminfo_is_term_family(term, b"vte\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0;
    let mut iterm: bool =
        terminfo_is_term_family(term, b"iterm\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || terminfo_is_term_family(term, b"iterm2\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"iTerm.app\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"iTerm2.app\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0;
    let mut alacritty: bool =
        terminfo_is_term_family(term, b"alacritty\0".as_ptr() as *const ::core::ffi::c_char);
    let mut foot: bool =
        terminfo_is_term_family(term, b"foot\0".as_ptr() as *const ::core::ffi::c_char);
    let mut iterm_pretending_xterm: bool =
        xterm as ::core::ffi::c_int != 0 && iterm_env as ::core::ffi::c_int != 0;
    let mut gnome_pretending_xterm: bool = xterm as ::core::ffi::c_int != 0
        && !colorterm.is_null()
        && !strstr(
            colorterm,
            b"gnome-terminal\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null();
    let mut mate_pretending_xterm: bool = xterm as ::core::ffi::c_int != 0
        && !colorterm.is_null()
        && !strstr(
            colorterm,
            b"mate-terminal\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null();
    let mut true_xterm: bool =
        xterm as ::core::ffi::c_int != 0 && !xterm_version.is_null() && !bsdvt;
    let mut cygwin: bool =
        terminfo_is_term_family(term, b"cygwin\0".as_ptr() as *const ::core::ffi::c_char);
    let mut ghostty: bool = terminfo_is_term_family(
        term,
        b"xterm-ghostty\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut fix_normal: *const ::core::ffi::c_char =
        (*tui).ti.defs[kTerm_cursor_normal as ::core::ffi::c_int as usize];
    if !fix_normal.is_null() {
        if strlen(fix_normal)
            >= ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
            && 0 as ::core::ffi::c_int
                == memcmp(
                    fix_normal as *const ::core::ffi::c_void,
                    b"\x1B[?12l\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                )
        {
            fix_normal = fix_normal.offset(
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as usize)
                    as isize,
            );
            terminfo_set_str(tui, kTerm_cursor_normal, fix_normal);
        }
        if linuxvt as ::core::ffi::c_int != 0
            && strlen(fix_normal)
                >= ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
            && memcmp(
                strchr(fix_normal, 0 as ::core::ffi::c_int).offset(
                    -(::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                        as isize),
                ) as *const ::core::ffi::c_void,
                LINUXSET0C.as_ptr() as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            ) == 0
        {
            let mut new_normal: *mut ::core::ffi::c_char = arena_memdupz(
                &raw mut (*tui).ti_arena,
                fix_normal,
                strlen(fix_normal).wrapping_sub(
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                ),
            );
            terminfo_set_str(tui, kTerm_cursor_normal, new_normal);
        }
    }
    let mut fix_invisible: *const ::core::ffi::c_char =
        (*tui).ti.defs[kTerm_cursor_invisible as ::core::ffi::c_int as usize];
    if !fix_invisible.is_null() {
        if linuxvt as ::core::ffi::c_int != 0
            && strlen(fix_invisible)
                >= ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
            && memcmp(
                strchr(fix_invisible, 0 as ::core::ffi::c_int).offset(
                    -(::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                        as isize),
                ) as *const ::core::ffi::c_void,
                LINUXSET1C.as_ptr() as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            ) == 0
        {
            let mut new_invisible: *mut ::core::ffi::c_char = arena_memdupz(
                &raw mut (*tui).ti_arena,
                fix_invisible,
                strlen(fix_invisible).wrapping_sub(
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                ),
            );
            terminfo_set_str(tui, kTerm_cursor_invisible, new_invisible);
        }
    }
    if tmux as ::core::ffi::c_int != 0
        || screen as ::core::ffi::c_int != 0
        || kitty as ::core::ffi::c_int != 0
    {
        (*tui).ti.bce = false_0 != 0;
    }
    if xterm as ::core::ffi::c_int != 0 || hterm as ::core::ffi::c_int != 0 {
        if !hterm {
            terminfo_set_if_empty(
                tui,
                kTerm_to_status_line,
                b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
            );
            terminfo_set_if_empty(
                tui,
                kTerm_from_status_line,
                b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        terminfo_set_if_empty(
            tui,
            kTerm_enter_italics_mode,
            b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_set_lr_margin,
            b"\x1B[%i%p1%d;%p2%ds\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if rxvt {
        terminfo_set_if_empty(
            tui,
            kTerm_enter_italics_mode,
            b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_to_status_line,
            b"\x1B]2\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_from_status_line,
            b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_str(
            tui,
            kTerm_enter_ca_mode,
            b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_str(
            tui,
            kTerm_exit_ca_mode,
            b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if screen {
        terminfo_set_if_empty(
            tui,
            kTerm_to_status_line,
            b"\x1B_\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_from_status_line,
            b"\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if tmux {
        terminfo_set_if_empty(
            tui,
            kTerm_to_status_line,
            b"\x1B_\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_from_status_line,
            b"\x1B\\\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_enter_italics_mode,
            b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if terminfo_is_term_family(term, b"interix\0".as_ptr() as *const ::core::ffi::c_char) {
        terminfo_set_if_empty(
            tui,
            kTerm_carriage_return,
            b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if linuxvt {
        terminfo_set_if_empty(
            tui,
            kTerm_parm_up_cursor,
            b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_parm_down_cursor,
            b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_parm_right_cursor,
            b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_if_empty(
            tui,
            kTerm_parm_left_cursor,
            b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if !putty {
        if iterm {
            terminfo_set_str(
                tui,
                kTerm_enter_ca_mode,
                b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
            );
            terminfo_set_str(
                tui,
                kTerm_exit_ca_mode,
                b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
            );
            terminfo_set_if_empty(
                tui,
                kTerm_enter_italics_mode,
                b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            let _ = st;
        }
    }
    if (*tui).ti.max_colors < 256 as ::core::ffi::c_int {
        if true_xterm as ::core::ffi::c_int != 0
            || iterm as ::core::ffi::c_int != 0
            || iterm_pretending_xterm as ::core::ffi::c_int != 0
        {
            (*tui).ti.max_colors = 256 as ::core::ffi::c_int;
            terminfo_set_str(tui, kTerm_set_a_foreground, XTERM_SETAF_256_COLON.as_ptr());
            terminfo_set_str(tui, kTerm_set_a_background, XTERM_SETAB_256_COLON.as_ptr());
        } else if konsolev != 0
            || xterm as ::core::ffi::c_int != 0
            || gnome as ::core::ffi::c_int != 0
            || rxvt as ::core::ffi::c_int != 0
            || st as ::core::ffi::c_int != 0
            || putty as ::core::ffi::c_int != 0
            || linuxvt as ::core::ffi::c_int != 0
            || mate_pretending_xterm as ::core::ffi::c_int != 0
            || gnome_pretending_xterm as ::core::ffi::c_int != 0
            || tmux as ::core::ffi::c_int != 0
            || !colorterm.is_null()
                && !strstr(colorterm, b"256\0".as_ptr() as *const ::core::ffi::c_char).is_null()
            || !term.is_null()
                && !strstr(term, b"256\0".as_ptr() as *const ::core::ffi::c_char).is_null()
        {
            (*tui).ti.max_colors = 256 as ::core::ffi::c_int;
            terminfo_set_str(tui, kTerm_set_a_foreground, XTERM_SETAF_256.as_ptr());
            terminfo_set_str(tui, kTerm_set_a_background, XTERM_SETAB_256.as_ptr());
        }
    }
    if (*tui).ti.max_colors < 16 as ::core::ffi::c_int {
        if !colorterm.is_null() {
            (*tui).ti.max_colors = 16 as ::core::ffi::c_int;
            terminfo_set_if_empty(tui, kTerm_set_a_foreground, XTERM_SETAF_16.as_ptr());
            terminfo_set_if_empty(tui, kTerm_set_a_background, XTERM_SETAB_16.as_ptr());
        }
    }
    if st as ::core::ffi::c_int != 0
        || vte_version != 0 as ::core::ffi::c_int && vte_version < 3900 as ::core::ffi::c_int
        || konsolev != 0
    {
        (*tui).ti.defs[kTerm_reset_cursor_style as ::core::ffi::c_int as usize] =
            ::core::ptr::null::<::core::ffi::c_char>();
    }
    if !bsdvt
        && (xterm as ::core::ffi::c_int != 0
            || putty as ::core::ffi::c_int != 0
            || hterm as ::core::ffi::c_int != 0
            || vte_version != 0
            || konsolev != 0
            || tmux as ::core::ffi::c_int != 0
            || screen as ::core::ffi::c_int != 0
            || st as ::core::ffi::c_int != 0
            || rxvt as ::core::ffi::c_int != 0
            || iterm as ::core::ffi::c_int != 0
            || iterm_pretending_xterm as ::core::ffi::c_int != 0
            || teraterm as ::core::ffi::c_int != 0
            || alacritty as ::core::ffi::c_int != 0
            || cygwin as ::core::ffi::c_int != 0
            || foot as ::core::ffi::c_int != 0
            || kitty as ::core::ffi::c_int != 0
            || ghostty as ::core::ffi::c_int != 0
            || linuxvt as ::core::ffi::c_int != 0
                && (!xterm_version.is_null() || !colorterm.is_null()))
    {
        terminfo_set_str(
            tui,
            kTerm_set_cursor_style,
            b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_str(
            tui,
            kTerm_reset_cursor_style,
            b"\x1B[0 q\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if linuxvt {
        terminfo_set_str(
            tui,
            kTerm_set_cursor_style,
            b"\x1B[?%?%p1%{2}%<%t%{8}%e%p1%{2}%=%t%{112}%e%p1%{3}%=%t%{4}%e%p1%{4}%=%t%{4}%e%p1%{5}%=%t%{2}%e%p1%{6}%=%t%{2}%e%{0}%;%dc\0"
                .as_ptr() as *const ::core::ffi::c_char,
        );
        terminfo_set_str(
            tui,
            kTerm_reset_cursor_style,
            b"\x1B[?c\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    xfree(xterm_version as *mut ::core::ffi::c_void);
}
pub const XTERM_SETAF_256_COLON: [::core::ffi::c_char; 63] = unsafe {
    ::core::mem::transmute::<[u8; 63], [::core::ffi::c_char; 63]>(
        *b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38:5:%p1%d%;m\0",
    )
};
pub const XTERM_SETAB_256_COLON: [::core::ffi::c_char; 64] = unsafe {
    ::core::mem::transmute::<[u8; 64], [::core::ffi::c_char; 64]>(
        *b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48:5:%p1%d%;m\0",
    )
};
pub const XTERM_SETAF_256: [::core::ffi::c_char; 63] = unsafe {
    ::core::mem::transmute::<[u8; 63], [::core::ffi::c_char; 63]>(
        *b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0",
    )
};
pub const XTERM_SETAB_256: [::core::ffi::c_char; 64] = unsafe {
    ::core::mem::transmute::<[u8; 64], [::core::ffi::c_char; 64]>(
        *b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0",
    )
};
pub const XTERM_SETAF_16: [::core::ffi::c_char; 55] = unsafe {
    ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
        *b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e39%;m\0",
    )
};
pub const XTERM_SETAB_16: [::core::ffi::c_char; 56] = unsafe {
    ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
        *b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e39%;m\0",
    )
};
unsafe extern "C" fn augment_terminfo(
    mut tui: *mut TUIData,
    mut term: *const ::core::ffi::c_char,
    mut vte_version: ::core::ffi::c_int,
    mut konsolev: ::core::ffi::c_int,
    mut weztermv: *const ::core::ffi::c_char,
    mut iterm_env: bool,
    mut nsterm: bool,
) {
    let mut xterm_version: *mut ::core::ffi::c_char =
        os_getenv(b"XTERM_VERSION\0".as_ptr() as *const ::core::ffi::c_char);
    let mut xterm: bool =
        terminfo_is_term_family(term, b"xterm\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || nsterm as ::core::ffi::c_int != 0;
    let mut hterm: bool =
        terminfo_is_term_family(term, b"hterm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut bsdvt: bool = terminfo_is_bsd_console(term);
    let mut dtterm: bool =
        terminfo_is_term_family(term, b"dtterm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut rxvt: bool =
        terminfo_is_term_family(term, b"rxvt\0".as_ptr() as *const ::core::ffi::c_char);
    let mut teraterm: bool =
        terminfo_is_term_family(term, b"teraterm\0".as_ptr() as *const ::core::ffi::c_char);
    let mut putty: bool =
        terminfo_is_term_family(term, b"putty\0".as_ptr() as *const ::core::ffi::c_char);
    let mut screen: bool =
        terminfo_is_term_family(term, b"screen\0".as_ptr() as *const ::core::ffi::c_char);
    let mut tmux: bool =
        terminfo_is_term_family(term, b"tmux\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || os_env_exists(
                b"TMUX\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as ::core::ffi::c_int
                != 0;
    let mut st: bool =
        terminfo_is_term_family(term, b"st\0".as_ptr() as *const ::core::ffi::c_char);
    let mut iterm: bool =
        terminfo_is_term_family(term, b"iterm\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
            || terminfo_is_term_family(term, b"iterm2\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"iTerm.app\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            || terminfo_is_term_family(term, b"iTerm2.app\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0;
    let mut alacritty: bool =
        terminfo_is_term_family(term, b"alacritty\0".as_ptr() as *const ::core::ffi::c_char);
    let mut kitty: bool = terminfo_is_term_family(
        term,
        b"xterm-kitty\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut iterm_pretending_xterm: bool =
        xterm as ::core::ffi::c_int != 0 && iterm_env as ::core::ffi::c_int != 0;
    let mut true_xterm: bool =
        xterm as ::core::ffi::c_int != 0 && !xterm_version.is_null() && !bsdvt;
    if dtterm as ::core::ffi::c_int != 0
        || xterm as ::core::ffi::c_int != 0
        || konsolev != 0
        || teraterm as ::core::ffi::c_int != 0
        || rxvt as ::core::ffi::c_int != 0
    {
        (*tui).can_resize_screen = true_0 != 0;
    }
    if putty as ::core::ffi::c_int != 0
        || xterm as ::core::ffi::c_int != 0
        || hterm as ::core::ffi::c_int != 0
        || rxvt as ::core::ffi::c_int != 0
    {
        (*tui).terminfo_ext.reset_scroll_region =
            b"\x1B[r\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    (*tui).terminfo_ext.enter_altfont_mode =
        b"\x1B[11m\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    let mut has_colon_rgb: bool = !tmux
        && !screen
        && vte_version == 0
        && (iterm as ::core::ffi::c_int != 0
            || iterm_pretending_xterm as ::core::ffi::c_int != 0
            || true_xterm as ::core::ffi::c_int != 0);
    if (*tui).ti.defs[kTerm_set_rgb_foreground as ::core::ffi::c_int as usize].is_null() {
        if has_colon_rgb {
            (*tui).ti.defs[kTerm_set_rgb_foreground as ::core::ffi::c_int as usize] =
                b"\x1B[38:2:%p1%d:%p2%d:%p3%dm\0".as_ptr() as *const ::core::ffi::c_char;
        } else {
            (*tui).ti.defs[kTerm_set_rgb_foreground as ::core::ffi::c_int as usize] =
                b"\x1B[38;2;%p1%d;%p2%d;%p3%dm\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
    if (*tui).ti.defs[kTerm_set_rgb_background as ::core::ffi::c_int as usize].is_null() {
        if has_colon_rgb {
            (*tui).ti.defs[kTerm_set_rgb_background as ::core::ffi::c_int as usize] =
                b"\x1B[48:2:%p1%d:%p2%d:%p3%dm\0".as_ptr() as *const ::core::ffi::c_char;
        } else {
            (*tui).ti.defs[kTerm_set_rgb_background as ::core::ffi::c_int as usize] =
                b"\x1B[48;2;%p1%d;%p2%d;%p3%dm\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
    if (*tui).ti.defs[kTerm_set_cursor_color as ::core::ffi::c_int as usize].is_null() {
        if iterm as ::core::ffi::c_int != 0 || iterm_pretending_xterm as ::core::ffi::c_int != 0 {
            (*tui).ti.defs[kTerm_set_cursor_color as ::core::ffi::c_int as usize] =
                if tmux as ::core::ffi::c_int != 0 {
                    b"\x1BPtmux;\x1B\x1B]Pl%p1%06x\x1B\\\x1B\\\0".as_ptr()
                        as *const ::core::ffi::c_char
                } else {
                    b"\x1B]Pl%p1%06x\x1B\\\0".as_ptr() as *const ::core::ffi::c_char
                };
        } else if (xterm as ::core::ffi::c_int != 0
            || hterm as ::core::ffi::c_int != 0
            || rxvt as ::core::ffi::c_int != 0
            || tmux as ::core::ffi::c_int != 0
            || alacritty as ::core::ffi::c_int != 0
            || st as ::core::ffi::c_int != 0)
            && (vte_version == 0 as ::core::ffi::c_int || vte_version >= 3900 as ::core::ffi::c_int)
        {
            (*tui).ti.defs[kTerm_set_cursor_color as ::core::ffi::c_int as usize] =
                b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
    if !(*tui).ti.defs[kTerm_set_cursor_color as ::core::ffi::c_int as usize].is_null() {
        (*tui).set_cursor_color_as_str = !strstr(
            (*tui).ti.defs[kTerm_set_cursor_color as ::core::ffi::c_int as usize],
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        )
        .is_null();
        terminfo_set_if_empty(
            tui,
            kTerm_reset_cursor_color,
            b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if !(*tui).ti.defs[kTerm_to_status_line as ::core::ffi::c_int as usize].is_null()
        && !(*tui).ti.defs[kTerm_from_status_line as ::core::ffi::c_int as usize].is_null()
    {
        (*tui).can_set_title = true_0 != 0;
    }
    (*tui).terminfo_ext.enable_focus_reporting = (if rxvt as ::core::ffi::c_int != 0 {
        b"\x1B[?1004h\x1B]777;focus;on\x07\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"\x1B[?1004h\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    (*tui).terminfo_ext.disable_focus_reporting = (if rxvt as ::core::ffi::c_int != 0 {
        b"\x1B[?1004l\x1B]777;focus;off\x07\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"\x1B[?1004l\0".as_ptr() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    if (*tui).ti.defs[kTerm_set_underline_style as ::core::ffi::c_int as usize].is_null() {
        if vte_version >= 5102 as ::core::ffi::c_int
            || konsolev >= 221170 as ::core::ffi::c_int
            || (*tui).ti.Su as ::core::ffi::c_int != 0
            || !weztermv.is_null()
                && strcmp(
                    weztermv,
                    b"20210203-095643\0".as_ptr() as *const ::core::ffi::c_char,
                ) > 0 as ::core::ffi::c_int
        {
            tui_enable_extended_underline(tui);
        }
    } else {
        tui_enable_extended_underline(tui);
    }
    if kitty as ::core::ffi::c_int != 0
        || vte_version != 0 as ::core::ffi::c_int && vte_version < 5400 as ::core::ffi::c_int
    {
        (*tui).input.key_encoding = kKeyEncodingLegacy;
    } else {
        (*tui).input.key_encoding = kKeyEncodingXterm;
    }
    xfree(xterm_version as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn should_invisible(mut tui: *mut TUIData) -> bool {
    return (*tui).busy as ::core::ffi::c_int != 0
        || (*tui).want_invisible as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn flush_buf_start(
    mut tui: *mut TUIData,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> size_t {
    if (*tui).sync_output as ::core::ffi::c_int != 0
        && (*tui).has_sync_mode as ::core::ffi::c_int != 0
    {
        return xstrlcpy(
            buf,
            b"\x1B[?2026h\0".as_ptr() as *const ::core::ffi::c_char,
            len,
        );
    } else if !(*tui).is_invisible {
        (*tui).is_invisible = true_0 != 0;
        let mut null_params: [TPVAR; 9] = [
            TPVAR {
                num: 0 as ::core::ffi::c_long,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            TPVAR {
                num: 0,
                string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
        ];
        let mut str: *const ::core::ffi::c_char =
            (*tui).ti.defs[kTerm_cursor_invisible as ::core::ffi::c_int as usize];
        if !str.is_null() {
            return terminfo_fmt(
                buf,
                buf.offset(len as isize),
                str,
                &raw mut null_params as *mut TPVAR,
            );
        }
    }
    return 0 as size_t;
}
unsafe extern "C" fn flush_buf_end(
    mut tui: *mut TUIData,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> size_t {
    let mut offset: size_t = 0 as size_t;
    if (*tui).sync_output as ::core::ffi::c_int != 0
        && (*tui).has_sync_mode as ::core::ffi::c_int != 0
    {
        memcpy(
            buf as *mut ::core::ffi::c_void,
            SYNC_END.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>(),
        );
        offset = (offset as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*tui).is_invisible as ::core::ffi::c_int != 0 && !should_invisible(tui) {
        str = (*tui).ti.defs[kTerm_cursor_normal as ::core::ffi::c_int as usize];
        (*tui).is_invisible = false_0 != 0;
    } else if !(*tui).is_invisible && should_invisible(tui) as ::core::ffi::c_int != 0 {
        str = (*tui).ti.defs[kTerm_cursor_invisible as ::core::ffi::c_int as usize];
        (*tui).is_invisible = true_0 != 0;
    }
    let mut null_params: [TPVAR; 9] = [
        TPVAR {
            num: 0 as ::core::ffi::c_long,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        TPVAR {
            num: 0,
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
    ];
    if !str.is_null() {
        offset = offset.wrapping_add(terminfo_fmt(
            buf.offset(offset as isize),
            buf.offset(len as isize),
            str,
            &raw mut null_params as *mut TPVAR,
        ));
    }
    return offset;
}
pub const SYNC_END: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"\x1B[?2026l\0") };
unsafe extern "C" fn flush_buf(mut tui: *mut TUIData) {
    let mut req: uv_write_t = uv_write_t {
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        type_0: UV_UNKNOWN_REQ,
        reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
        cb: None,
        send_handle: ::core::ptr::null_mut::<uv_stream_t>(),
        handle: ::core::ptr::null_mut::<uv_stream_t>(),
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        write_index: 0,
        bufs: ::core::ptr::null_mut::<uv_buf_t>(),
        nbufs: 0,
        error: 0,
        bufsml: [uv_buf_t {
            base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            len: 0,
        }; 4],
    };
    let mut bufs: [uv_buf_t; 3] = [uv_buf_t {
        base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    }; 3];
    let mut pre: [::core::ffi::c_char; 32] = [0; 32];
    let mut post: [::core::ffi::c_char; 32] = [0; 32];
    if (*tui).bufpos <= 0 as size_t
        && (*tui).is_invisible as ::core::ffi::c_int == should_invisible(tui) as ::core::ffi::c_int
    {
        return;
    }
    bufs[0 as ::core::ffi::c_int as usize].base = &raw mut pre as *mut ::core::ffi::c_char;
    bufs[0 as ::core::ffi::c_int as usize].len = flush_buf_start(
        tui,
        &raw mut pre as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
    );
    bufs[1 as ::core::ffi::c_int as usize].base = if !(*tui).buf_to_flush.is_null() {
        (*tui).buf_to_flush
    } else {
        &raw mut (*tui).buf as *mut ::core::ffi::c_char
    };
    bufs[1 as ::core::ffi::c_int as usize].len = (*tui).bufpos;
    bufs[2 as ::core::ffi::c_int as usize].base = &raw mut post as *mut ::core::ffi::c_char;
    bufs[2 as ::core::ffi::c_int as usize].len = flush_buf_end(
        tui,
        &raw mut post as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
    );
    if !(*tui).screenshot.is_null() {
        let mut i: size_t = 0 as size_t;
        while i < ::core::mem::size_of::<[uv_buf_t; 3]>()
            .wrapping_div(::core::mem::size_of::<uv_buf_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uv_buf_t; 3]>()
                    .wrapping_rem(::core::mem::size_of::<uv_buf_t>())
                    == 0) as ::core::ffi::c_int as usize,
            )
        {
            fwrite(
                bufs[i as usize].base as *const ::core::ffi::c_void,
                bufs[i as usize].len,
                1 as size_t,
                (*tui).screenshot,
            );
            i = i.wrapping_add(1);
        }
    } else {
        let mut ret: ::core::ffi::c_int = uv_write(
            &raw mut req,
            &raw mut (*tui).output_handle as *mut uv_stream_t,
            &raw mut bufs as *mut uv_buf_t as *const uv_buf_t,
            ::core::mem::size_of::<[uv_buf_t; 3]>()
                .wrapping_div(::core::mem::size_of::<uv_buf_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uv_buf_t; 3]>()
                        .wrapping_rem(::core::mem::size_of::<uv_buf_t>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_uint,
            None,
        );
        if ret != 0 {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"flush_buf\0".as_ptr() as *const ::core::ffi::c_char,
                2534 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_write failed: %s\0".as_ptr() as *const ::core::ffi::c_char,
                uv_strerror(ret),
            );
        }
        uv_run(&raw mut (*tui).write_loop, UV_RUN_DEFAULT);
    }
    (*tui).buf_to_flush = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*tui).bufpos = 0 as size_t;
}
unsafe extern "C" fn tui_get_stty_erase(mut input: *mut TermInput) -> *const ::core::ffi::c_char {
    static stty_erase: GlobalCell<[::core::ffi::c_char; 2]> =
        GlobalCell::new([0 as ::core::ffi::c_char, 0]);
    let mut t: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c_ispeed: 0,
        c_ospeed: 0,
    };
    if tcgetattr((*input).in_fd, &raw mut t) != -1 as ::core::ffi::c_int {
        (*stty_erase.ptr())[0 as ::core::ffi::c_int as usize] =
            t.c_cc[VERASE as usize] as ::core::ffi::c_char;
        (*stty_erase.ptr())[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_get_stty_erase\0".as_ptr() as *const ::core::ffi::c_char,
            2557 as ::core::ffi::c_int,
            true_0 != 0,
            b"stty/termios:erase=%s\0".as_ptr() as *const ::core::ffi::c_char,
            stty_erase.ptr() as *mut ::core::ffi::c_char,
        );
    }
    return stty_erase.ptr() as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn tui_tk_ti_getstr(
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut data: *mut ::core::ffi::c_void,
) -> *const ::core::ffi::c_char {
    let mut input: *mut TermInput = data as *mut TermInput;
    static stty_erase: GlobalCell<*const ::core::ffi::c_char> =
        GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
    if (*stty_erase.ptr()).is_null() {
        stty_erase.set(tui_get_stty_erase(input));
    }
    if strequal(
        name,
        b"key_backspace\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_tk_ti_getstr\0".as_ptr() as *const ::core::ffi::c_char,
            2582 as ::core::ffi::c_int,
            true_0 != 0,
            b"libtermkey:kbs=%s\0".as_ptr() as *const ::core::ffi::c_char,
            value,
        );
        if *(*stty_erase.ptr()).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            return stty_erase.get();
        }
    } else if strequal(name, b"key_dc\0".as_ptr() as *const ::core::ffi::c_char) {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_tk_ti_getstr\0".as_ptr() as *const ::core::ffi::c_char,
            2587 as ::core::ffi::c_int,
            true_0 != 0,
            b"libtermkey:kdch1=%s\0".as_ptr() as *const ::core::ffi::c_char,
            value,
        );
        if !value.is_null()
            && value
                != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_char>(
                    -1 as ::core::ffi::c_int as usize,
                ) as *const ::core::ffi::c_char
            && strequal(stty_erase.get(), value) as ::core::ffi::c_int != 0
        {
            return if *(*stty_erase.ptr()).offset(0 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == DEL
            {
                CTRL_H_STR.as_ptr()
            } else {
                DEL_STR.as_ptr()
            };
        }
    } else if strequal(name, b"key_mouse\0".as_ptr() as *const ::core::ffi::c_char) {
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"tui_tk_ti_getstr\0".as_ptr() as *const ::core::ffi::c_char,
            2593 as ::core::ffi::c_int,
            true_0 != 0,
            b"libtermkey:kmous=%s\0".as_ptr() as *const ::core::ffi::c_char,
            value,
        );
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return value;
}
pub const SIGSTOP: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const SIGWINCH: ::core::ffi::c_int = 28 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const VERASE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
