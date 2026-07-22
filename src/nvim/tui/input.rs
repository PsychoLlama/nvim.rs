use crate::src::nvim::event::libuv::{
    uv_close, uv_timer_get_due_in, uv_timer_init, uv_timer_start, uv_timer_stop,
};
use crate::src::nvim::event::r#loop::loop_schedule_fast;
use crate::src::nvim::event::rstream::{
    rstream_available, rstream_consume, rstream_init_fd, rstream_may_close, rstream_start,
    rstream_stop,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{main_loop, os_exit, p_ttimeout, p_ttm, ui_client_channel_id};
use crate::src::nvim::map::{map_put_ref_int_ptr_t, mh_get_int};
use crate::src::nvim::memory::{strnequal, xfree, xrealloc};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
use crate::src::nvim::os::libc::{__assert_fail, abort, memcmp, memcpy, memmove, snprintf, strlen};
use crate::src::nvim::strings::kv_do_printf;
use crate::src::nvim::tui::termkey::driver_csi::{
    termkey_interpret_csi, termkey_interpret_csi_param, termkey_interpret_modereport,
    termkey_interpret_mouse,
};
use crate::src::nvim::tui::termkey::termkey::{
    termkey_destroy, termkey_get_buffer_remaining, termkey_get_buffer_size, termkey_get_canonflags,
    termkey_getkey, termkey_getkey_force, termkey_hook_terminfo_getstr, termkey_interpret_string,
    termkey_new_abstract, termkey_push_bytes, termkey_set_buffer_size, termkey_set_canonflags,
    termkey_start, termkey_strfkey,
};
pub use crate::src::nvim::types::{
    Array, Boolean, Dict, Event, Float, Integer, KeyEncoding, KeyValuePair, Loop, LuaRef, MapHash,
    Map_int_ptr_t, MultiQueue, Object, ObjectType, OptInt, Proc, ProcType, RStream, ScopeType,
    Set_int, Stream, StringBuilder, String_0, TUIData, TermKey, TermKeyCsiParam, TermKeyEvent,
    TermKeyFormat, TermKeyKey, TermKeyKey_code as C2Rust_Unnamed_18, TermKeyMouseEvent,
    TermKeyResult, TermKeySym, TermKeyType, TermKey_Terminfo_Getstr_Hook, TermMode, TermModeState,
    TerminfoEntry, VarLockStatus, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s,
    __pthread_rwlock_arch_t, argv_callback, dict_T, dictvar_S, hash_T, hashitem_T, hashtab_T,
    int64_t, int8_t, internal_proc_cb, key_value_pair, loop_0, loop_0_children as C2Rust_Unnamed_8,
    multiqueue, object, object_data as C2Rust_Unnamed, proc, proc_exit_cb, proc_state_cb,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, queue, rstream, size_t, ssize_t, stream,
    stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_10, stream_write_cb, uint32_t,
    uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb,
    uv_async_s, uv_async_s_u as C2Rust_Unnamed_5, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_11, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_3, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_13, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb, uv_signal_s,
    uv_signal_s_tree_entry as C2Rust_Unnamed_1, uv_signal_s_u as C2Rust_Unnamed_2, uv_signal_t,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_9, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_12, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_6, uv_timer_s_u as C2Rust_Unnamed_7, uv_timer_t, QUEUE,
};
extern "C" {
    fn tui_handle_term_mode(tui: *mut TUIData, mode: TermMode, state: TermModeState);
    fn tui_enable_extended_underline(tui: *mut TUIData);
    fn tui_query_bg_color(tui: *mut TUIData);
    fn tui_set_size(tui: *mut TUIData, width: ::core::ffi::c_int, height: ::core::ffi::c_int);
}
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
pub const kProcTypePty: ProcType = 1;
pub const kProcTypeUv: ProcType = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const KITTY_KEY_ISO_LEVEL5_SHIFT: C2Rust_Unnamed_14 = 57454;
pub const KITTY_KEY_ISO_LEVEL3_SHIFT: C2Rust_Unnamed_14 = 57453;
pub const KITTY_KEY_RIGHT_META: C2Rust_Unnamed_14 = 57452;
pub const KITTY_KEY_RIGHT_HYPER: C2Rust_Unnamed_14 = 57451;
pub const KITTY_KEY_RIGHT_SUPER: C2Rust_Unnamed_14 = 57450;
pub const KITTY_KEY_RIGHT_ALT: C2Rust_Unnamed_14 = 57449;
pub const KITTY_KEY_RIGHT_CONTROL: C2Rust_Unnamed_14 = 57448;
pub const KITTY_KEY_RIGHT_SHIFT: C2Rust_Unnamed_14 = 57447;
pub const KITTY_KEY_LEFT_META: C2Rust_Unnamed_14 = 57446;
pub const KITTY_KEY_LEFT_HYPER: C2Rust_Unnamed_14 = 57445;
pub const KITTY_KEY_LEFT_SUPER: C2Rust_Unnamed_14 = 57444;
pub const KITTY_KEY_LEFT_ALT: C2Rust_Unnamed_14 = 57443;
pub const KITTY_KEY_LEFT_CONTROL: C2Rust_Unnamed_14 = 57442;
pub const KITTY_KEY_LEFT_SHIFT: C2Rust_Unnamed_14 = 57441;
pub const KITTY_KEY_MUTE_VOLUME: C2Rust_Unnamed_14 = 57440;
pub const KITTY_KEY_RAISE_VOLUME: C2Rust_Unnamed_14 = 57439;
pub const KITTY_KEY_LOWER_VOLUME: C2Rust_Unnamed_14 = 57438;
pub const KITTY_KEY_MEDIA_RECORD: C2Rust_Unnamed_14 = 57437;
pub const KITTY_KEY_MEDIA_TRACK_PREVIOUS: C2Rust_Unnamed_14 = 57436;
pub const KITTY_KEY_MEDIA_TRACK_NEXT: C2Rust_Unnamed_14 = 57435;
pub const KITTY_KEY_MEDIA_REWIND: C2Rust_Unnamed_14 = 57434;
pub const KITTY_KEY_MEDIA_FAST_FORWARD: C2Rust_Unnamed_14 = 57433;
pub const KITTY_KEY_MEDIA_STOP: C2Rust_Unnamed_14 = 57432;
pub const KITTY_KEY_MEDIA_REVERSE: C2Rust_Unnamed_14 = 57431;
pub const KITTY_KEY_MEDIA_PLAY_PAUSE: C2Rust_Unnamed_14 = 57430;
pub const KITTY_KEY_MEDIA_PAUSE: C2Rust_Unnamed_14 = 57429;
pub const KITTY_KEY_MEDIA_PLAY: C2Rust_Unnamed_14 = 57428;
pub const KITTY_KEY_KP_BEGIN: C2Rust_Unnamed_14 = 57427;
pub const KITTY_KEY_KP_DELETE: C2Rust_Unnamed_14 = 57426;
pub const KITTY_KEY_KP_INSERT: C2Rust_Unnamed_14 = 57425;
pub const KITTY_KEY_KP_END: C2Rust_Unnamed_14 = 57424;
pub const KITTY_KEY_KP_HOME: C2Rust_Unnamed_14 = 57423;
pub const KITTY_KEY_KP_PAGE_DOWN: C2Rust_Unnamed_14 = 57422;
pub const KITTY_KEY_KP_PAGE_UP: C2Rust_Unnamed_14 = 57421;
pub const KITTY_KEY_KP_DOWN: C2Rust_Unnamed_14 = 57420;
pub const KITTY_KEY_KP_UP: C2Rust_Unnamed_14 = 57419;
pub const KITTY_KEY_KP_RIGHT: C2Rust_Unnamed_14 = 57418;
pub const KITTY_KEY_KP_LEFT: C2Rust_Unnamed_14 = 57417;
pub const KITTY_KEY_KP_SEPARATOR: C2Rust_Unnamed_14 = 57416;
pub const KITTY_KEY_KP_EQUAL: C2Rust_Unnamed_14 = 57415;
pub const KITTY_KEY_KP_ENTER: C2Rust_Unnamed_14 = 57414;
pub const KITTY_KEY_KP_ADD: C2Rust_Unnamed_14 = 57413;
pub const KITTY_KEY_KP_SUBTRACT: C2Rust_Unnamed_14 = 57412;
pub const KITTY_KEY_KP_MULTIPLY: C2Rust_Unnamed_14 = 57411;
pub const KITTY_KEY_KP_DIVIDE: C2Rust_Unnamed_14 = 57410;
pub const KITTY_KEY_KP_DECIMAL: C2Rust_Unnamed_14 = 57409;
pub const KITTY_KEY_KP_9: C2Rust_Unnamed_14 = 57408;
pub const KITTY_KEY_KP_8: C2Rust_Unnamed_14 = 57407;
pub const KITTY_KEY_KP_7: C2Rust_Unnamed_14 = 57406;
pub const KITTY_KEY_KP_6: C2Rust_Unnamed_14 = 57405;
pub const KITTY_KEY_KP_5: C2Rust_Unnamed_14 = 57404;
pub const KITTY_KEY_KP_4: C2Rust_Unnamed_14 = 57403;
pub const KITTY_KEY_KP_3: C2Rust_Unnamed_14 = 57402;
pub const KITTY_KEY_KP_2: C2Rust_Unnamed_14 = 57401;
pub const KITTY_KEY_KP_1: C2Rust_Unnamed_14 = 57400;
pub const KITTY_KEY_KP_0: C2Rust_Unnamed_14 = 57399;
pub const KITTY_KEY_F35: C2Rust_Unnamed_14 = 57398;
pub const KITTY_KEY_F34: C2Rust_Unnamed_14 = 57397;
pub const KITTY_KEY_F33: C2Rust_Unnamed_14 = 57396;
pub const KITTY_KEY_F32: C2Rust_Unnamed_14 = 57395;
pub const KITTY_KEY_F31: C2Rust_Unnamed_14 = 57394;
pub const KITTY_KEY_F30: C2Rust_Unnamed_14 = 57393;
pub const KITTY_KEY_F29: C2Rust_Unnamed_14 = 57392;
pub const KITTY_KEY_F28: C2Rust_Unnamed_14 = 57391;
pub const KITTY_KEY_F27: C2Rust_Unnamed_14 = 57390;
pub const KITTY_KEY_F26: C2Rust_Unnamed_14 = 57389;
pub const KITTY_KEY_F25: C2Rust_Unnamed_14 = 57388;
pub const KITTY_KEY_F24: C2Rust_Unnamed_14 = 57387;
pub const KITTY_KEY_F23: C2Rust_Unnamed_14 = 57386;
pub const KITTY_KEY_F22: C2Rust_Unnamed_14 = 57385;
pub const KITTY_KEY_F21: C2Rust_Unnamed_14 = 57384;
pub const KITTY_KEY_F20: C2Rust_Unnamed_14 = 57383;
pub const KITTY_KEY_F19: C2Rust_Unnamed_14 = 57382;
pub const KITTY_KEY_F18: C2Rust_Unnamed_14 = 57381;
pub const KITTY_KEY_F17: C2Rust_Unnamed_14 = 57380;
pub const KITTY_KEY_F16: C2Rust_Unnamed_14 = 57379;
pub const KITTY_KEY_F15: C2Rust_Unnamed_14 = 57378;
pub const KITTY_KEY_F14: C2Rust_Unnamed_14 = 57377;
pub const KITTY_KEY_F13: C2Rust_Unnamed_14 = 57376;
pub const KITTY_KEY_F12: C2Rust_Unnamed_14 = 57375;
pub const KITTY_KEY_F11: C2Rust_Unnamed_14 = 57374;
pub const KITTY_KEY_F10: C2Rust_Unnamed_14 = 57373;
pub const KITTY_KEY_F9: C2Rust_Unnamed_14 = 57372;
pub const KITTY_KEY_F8: C2Rust_Unnamed_14 = 57371;
pub const KITTY_KEY_F7: C2Rust_Unnamed_14 = 57370;
pub const KITTY_KEY_F6: C2Rust_Unnamed_14 = 57369;
pub const KITTY_KEY_F5: C2Rust_Unnamed_14 = 57368;
pub const KITTY_KEY_F4: C2Rust_Unnamed_14 = 57367;
pub const KITTY_KEY_F3: C2Rust_Unnamed_14 = 57366;
pub const KITTY_KEY_F2: C2Rust_Unnamed_14 = 57365;
pub const KITTY_KEY_F1: C2Rust_Unnamed_14 = 57364;
pub const KITTY_KEY_MENU: C2Rust_Unnamed_14 = 57363;
pub const KITTY_KEY_PAUSE: C2Rust_Unnamed_14 = 57362;
pub const KITTY_KEY_PRINT_SCREEN: C2Rust_Unnamed_14 = 57361;
pub const KITTY_KEY_NUM_LOCK: C2Rust_Unnamed_14 = 57360;
pub const KITTY_KEY_SCROLL_LOCK: C2Rust_Unnamed_14 = 57359;
pub const KITTY_KEY_CAPS_LOCK: C2Rust_Unnamed_14 = 57358;
pub const KITTY_KEY_END: C2Rust_Unnamed_14 = 57357;
pub const KITTY_KEY_HOME: C2Rust_Unnamed_14 = 57356;
pub const KITTY_KEY_PAGE_DOWN: C2Rust_Unnamed_14 = 57355;
pub const KITTY_KEY_PAGE_UP: C2Rust_Unnamed_14 = 57354;
pub const KITTY_KEY_DOWN: C2Rust_Unnamed_14 = 57353;
pub const KITTY_KEY_UP: C2Rust_Unnamed_14 = 57352;
pub const KITTY_KEY_RIGHT: C2Rust_Unnamed_14 = 57351;
pub const KITTY_KEY_LEFT: C2Rust_Unnamed_14 = 57350;
pub const KITTY_KEY_DELETE: C2Rust_Unnamed_14 = 57349;
pub const KITTY_KEY_INSERT: C2Rust_Unnamed_14 = 57348;
pub const KITTY_KEY_BACKSPACE: C2Rust_Unnamed_14 = 57347;
pub const KITTY_KEY_TAB: C2Rust_Unnamed_14 = 57346;
pub const KITTY_KEY_ENTER: C2Rust_Unnamed_14 = 57345;
pub const KITTY_KEY_ESCAPE: C2Rust_Unnamed_14 = 57344;
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
pub const TERMKEY_RES_ERROR: TermKeyResult = 4;
pub const TERMKEY_RES_AGAIN: TermKeyResult = 3;
pub const TERMKEY_RES_EOF: TermKeyResult = 2;
pub const TERMKEY_RES_KEY: TermKeyResult = 1;
pub const TERMKEY_RES_NONE: TermKeyResult = 0;
pub const TERMKEY_N_SYMS: TermKeySym = 60;
pub const TERMKEY_SYM_KPEQUALS: TermKeySym = 59;
pub const TERMKEY_SYM_KPPERIOD: TermKeySym = 58;
pub const TERMKEY_SYM_KPCOMMA: TermKeySym = 57;
pub const TERMKEY_SYM_KPDIV: TermKeySym = 56;
pub const TERMKEY_SYM_KPMULT: TermKeySym = 55;
pub const TERMKEY_SYM_KPMINUS: TermKeySym = 54;
pub const TERMKEY_SYM_KPPLUS: TermKeySym = 53;
pub const TERMKEY_SYM_KPENTER: TermKeySym = 52;
pub const TERMKEY_SYM_KP9: TermKeySym = 51;
pub const TERMKEY_SYM_KP8: TermKeySym = 50;
pub const TERMKEY_SYM_KP7: TermKeySym = 49;
pub const TERMKEY_SYM_KP6: TermKeySym = 48;
pub const TERMKEY_SYM_KP5: TermKeySym = 47;
pub const TERMKEY_SYM_KP4: TermKeySym = 46;
pub const TERMKEY_SYM_KP3: TermKeySym = 45;
pub const TERMKEY_SYM_KP2: TermKeySym = 44;
pub const TERMKEY_SYM_KP1: TermKeySym = 43;
pub const TERMKEY_SYM_KP0: TermKeySym = 42;
pub const TERMKEY_SYM_UNDO: TermKeySym = 41;
pub const TERMKEY_SYM_SUSPEND: TermKeySym = 40;
pub const TERMKEY_SYM_SAVE: TermKeySym = 39;
pub const TERMKEY_SYM_RESUME: TermKeySym = 38;
pub const TERMKEY_SYM_RESTART: TermKeySym = 37;
pub const TERMKEY_SYM_REPLACE: TermKeySym = 36;
pub const TERMKEY_SYM_REFRESH: TermKeySym = 35;
pub const TERMKEY_SYM_REFERENCE: TermKeySym = 34;
pub const TERMKEY_SYM_REDO: TermKeySym = 33;
pub const TERMKEY_SYM_PRINT: TermKeySym = 32;
pub const TERMKEY_SYM_OPTIONS: TermKeySym = 31;
pub const TERMKEY_SYM_OPEN: TermKeySym = 30;
pub const TERMKEY_SYM_MOVE: TermKeySym = 29;
pub const TERMKEY_SYM_MESSAGE: TermKeySym = 28;
pub const TERMKEY_SYM_MARK: TermKeySym = 27;
pub const TERMKEY_SYM_HELP: TermKeySym = 26;
pub const TERMKEY_SYM_EXIT: TermKeySym = 25;
pub const TERMKEY_SYM_COPY: TermKeySym = 24;
pub const TERMKEY_SYM_COMMAND: TermKeySym = 23;
pub const TERMKEY_SYM_CLOSE: TermKeySym = 22;
pub const TERMKEY_SYM_CLEAR: TermKeySym = 21;
pub const TERMKEY_SYM_CANCEL: TermKeySym = 20;
pub const TERMKEY_SYM_END: TermKeySym = 19;
pub const TERMKEY_SYM_HOME: TermKeySym = 18;
pub const TERMKEY_SYM_PAGEDOWN: TermKeySym = 17;
pub const TERMKEY_SYM_PAGEUP: TermKeySym = 16;
pub const TERMKEY_SYM_SELECT: TermKeySym = 15;
pub const TERMKEY_SYM_DELETE: TermKeySym = 14;
pub const TERMKEY_SYM_INSERT: TermKeySym = 13;
pub const TERMKEY_SYM_FIND: TermKeySym = 12;
pub const TERMKEY_SYM_BEGIN: TermKeySym = 11;
pub const TERMKEY_SYM_RIGHT: TermKeySym = 10;
pub const TERMKEY_SYM_LEFT: TermKeySym = 9;
pub const TERMKEY_SYM_DOWN: TermKeySym = 8;
pub const TERMKEY_SYM_UP: TermKeySym = 7;
pub const TERMKEY_SYM_DEL: TermKeySym = 6;
pub const TERMKEY_SYM_SPACE: TermKeySym = 5;
pub const TERMKEY_SYM_ESCAPE: TermKeySym = 4;
pub const TERMKEY_SYM_ENTER: TermKeySym = 3;
pub const TERMKEY_SYM_TAB: TermKeySym = 2;
pub const TERMKEY_SYM_BACKSPACE: TermKeySym = 1;
pub const TERMKEY_SYM_NONE: TermKeySym = 0;
pub const TERMKEY_SYM_UNKNOWN: TermKeySym = -1;
pub const TERMKEY_TYPE_UNKNOWN_CSI: TermKeyType = -1;
pub const TERMKEY_TYPE_APC: TermKeyType = 8;
pub const TERMKEY_TYPE_OSC: TermKeyType = 7;
pub const TERMKEY_TYPE_DCS: TermKeyType = 6;
pub const TERMKEY_TYPE_MODEREPORT: TermKeyType = 5;
pub const TERMKEY_TYPE_POSITION: TermKeyType = 4;
pub const TERMKEY_TYPE_MOUSE: TermKeyType = 3;
pub const TERMKEY_TYPE_KEYSYM: TermKeyType = 2;
pub const TERMKEY_TYPE_FUNCTION: TermKeyType = 1;
pub const TERMKEY_TYPE_UNICODE: TermKeyType = 0;
pub const TERMKEY_MOUSE_RELEASE: TermKeyMouseEvent = 3;
pub const TERMKEY_MOUSE_DRAG: TermKeyMouseEvent = 2;
pub const TERMKEY_MOUSE_PRESS: TermKeyMouseEvent = 1;
pub const TERMKEY_MOUSE_UNKNOWN: TermKeyMouseEvent = 0;
pub const TERMKEY_EVENT_RELEASE: TermKeyEvent = 3;
pub const TERMKEY_EVENT_REPEAT: TermKeyEvent = 2;
pub const TERMKEY_EVENT_PRESS: TermKeyEvent = 1;
pub const TERMKEY_EVENT_UNKNOWN: TermKeyEvent = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const TERMKEY_KEYMOD_CTRL: C2Rust_Unnamed_15 = 4;
pub const TERMKEY_KEYMOD_ALT: C2Rust_Unnamed_15 = 2;
pub const TERMKEY_KEYMOD_SHIFT: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const TERMKEY_FLAG_KEEPC0: C2Rust_Unnamed_16 = 512;
pub const TERMKEY_FLAG_NOSTART: C2Rust_Unnamed_16 = 256;
pub const TERMKEY_FLAG_EINTR: C2Rust_Unnamed_16 = 128;
pub const TERMKEY_FLAG_CTRLC: C2Rust_Unnamed_16 = 64;
pub const TERMKEY_FLAG_SPACESYMBOL: C2Rust_Unnamed_16 = 32;
pub const TERMKEY_FLAG_NOTERMIOS: C2Rust_Unnamed_16 = 16;
pub const TERMKEY_FLAG_UTF8: C2Rust_Unnamed_16 = 8;
pub const TERMKEY_FLAG_RAW: C2Rust_Unnamed_16 = 4;
pub const TERMKEY_FLAG_CONVERTKP: C2Rust_Unnamed_16 = 2;
pub const TERMKEY_FLAG_NOINTERPRET: C2Rust_Unnamed_16 = 1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const TERMKEY_CANON_DELBS: C2Rust_Unnamed_17 = 2;
pub const TERMKEY_CANON_SPACESYMBOL: C2Rust_Unnamed_17 = 1;
pub const TERMKEY_FORMAT_MOUSE_POS: TermKeyFormat = 256;
pub const TERMKEY_FORMAT_LOWERSPACE: TermKeyFormat = 64;
pub const TERMKEY_FORMAT_LOWERMOD: TermKeyFormat = 32;
pub const TERMKEY_FORMAT_SPACEMOD: TermKeyFormat = 16;
pub const TERMKEY_FORMAT_WRAPBRACKET: TermKeyFormat = 8;
pub const TERMKEY_FORMAT_ALTISMETA: TermKeyFormat = 4;
pub const TERMKEY_FORMAT_CARETCTRL: TermKeyFormat = 2;
pub const TERMKEY_FORMAT_LONGMOD: TermKeyFormat = 1;
pub const kKeyEncodingXterm: KeyEncoding = 2;
pub const kKeyEncodingKitty: KeyEncoding = 1;
pub const kKeyEncodingLegacy: KeyEncoding = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermInputCallbacks {
    pub primary_device_attr: Option<unsafe extern "C" fn(*mut TUIData) -> ()>,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct kitty_key_map_entry {
    pub key: ::core::ffi::c_int,
    pub name: *const ::core::ffi::c_char,
}
pub const KEYMOD_META: C2Rust_Unnamed_19 = 32;
pub const KEYMOD_SUPER: C2Rust_Unnamed_19 = 8;
pub const KEYMOD_RECOGNIZED: C2Rust_Unnamed_19 = 47;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_int = Set_int {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MAP_INIT: Map_int_ptr_t = Map_int_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_int(mut set: *mut Set_int, mut key: ::core::ffi::c_int) -> bool {
    return mh_get_int(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const INPUT_BUFFER_SIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
static kitty_key_map_entry: GlobalCell<[kitty_key_map_entry; 77]> = GlobalCell::new([
    kitty_key_map_entry {
        key: KITTY_KEY_ESCAPE as ::core::ffi::c_int,
        name: b"Esc\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_ENTER as ::core::ffi::c_int,
        name: b"CR\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_TAB as ::core::ffi::c_int,
        name: b"Tab\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_BACKSPACE as ::core::ffi::c_int,
        name: b"BS\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_INSERT as ::core::ffi::c_int,
        name: b"Insert\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_DELETE as ::core::ffi::c_int,
        name: b"Del\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_LEFT as ::core::ffi::c_int,
        name: b"Left\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_RIGHT as ::core::ffi::c_int,
        name: b"Right\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_UP as ::core::ffi::c_int,
        name: b"Up\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_DOWN as ::core::ffi::c_int,
        name: b"Down\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_PAGE_UP as ::core::ffi::c_int,
        name: b"PageUp\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_PAGE_DOWN as ::core::ffi::c_int,
        name: b"PageDown\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_HOME as ::core::ffi::c_int,
        name: b"Home\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_END as ::core::ffi::c_int,
        name: b"End\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F1 as ::core::ffi::c_int,
        name: b"F1\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F2 as ::core::ffi::c_int,
        name: b"F2\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F3 as ::core::ffi::c_int,
        name: b"F3\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F4 as ::core::ffi::c_int,
        name: b"F4\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F5 as ::core::ffi::c_int,
        name: b"F5\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F6 as ::core::ffi::c_int,
        name: b"F6\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F7 as ::core::ffi::c_int,
        name: b"F7\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F8 as ::core::ffi::c_int,
        name: b"F8\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F9 as ::core::ffi::c_int,
        name: b"F9\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F10 as ::core::ffi::c_int,
        name: b"F10\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F11 as ::core::ffi::c_int,
        name: b"F11\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F12 as ::core::ffi::c_int,
        name: b"F12\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F13 as ::core::ffi::c_int,
        name: b"F13\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F14 as ::core::ffi::c_int,
        name: b"F14\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F15 as ::core::ffi::c_int,
        name: b"F15\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F16 as ::core::ffi::c_int,
        name: b"F16\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F17 as ::core::ffi::c_int,
        name: b"F17\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F18 as ::core::ffi::c_int,
        name: b"F18\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F19 as ::core::ffi::c_int,
        name: b"F19\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F20 as ::core::ffi::c_int,
        name: b"F20\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F21 as ::core::ffi::c_int,
        name: b"F21\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F22 as ::core::ffi::c_int,
        name: b"F22\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F23 as ::core::ffi::c_int,
        name: b"F23\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F24 as ::core::ffi::c_int,
        name: b"F24\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F25 as ::core::ffi::c_int,
        name: b"F25\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F26 as ::core::ffi::c_int,
        name: b"F26\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F27 as ::core::ffi::c_int,
        name: b"F27\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F28 as ::core::ffi::c_int,
        name: b"F28\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F29 as ::core::ffi::c_int,
        name: b"F29\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F30 as ::core::ffi::c_int,
        name: b"F30\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F31 as ::core::ffi::c_int,
        name: b"F31\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F32 as ::core::ffi::c_int,
        name: b"F32\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F33 as ::core::ffi::c_int,
        name: b"F33\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F34 as ::core::ffi::c_int,
        name: b"F34\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_F35 as ::core::ffi::c_int,
        name: b"F35\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_0 as ::core::ffi::c_int,
        name: b"k0\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_1 as ::core::ffi::c_int,
        name: b"k1\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_2 as ::core::ffi::c_int,
        name: b"k2\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_3 as ::core::ffi::c_int,
        name: b"k3\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_4 as ::core::ffi::c_int,
        name: b"k4\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_5 as ::core::ffi::c_int,
        name: b"k5\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_6 as ::core::ffi::c_int,
        name: b"k6\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_7 as ::core::ffi::c_int,
        name: b"k7\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_8 as ::core::ffi::c_int,
        name: b"k8\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_9 as ::core::ffi::c_int,
        name: b"k9\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_DECIMAL as ::core::ffi::c_int,
        name: b"kPoint\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_DIVIDE as ::core::ffi::c_int,
        name: b"kDivide\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_MULTIPLY as ::core::ffi::c_int,
        name: b"kMultiply\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_SUBTRACT as ::core::ffi::c_int,
        name: b"kMinus\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_ADD as ::core::ffi::c_int,
        name: b"kPlus\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_ENTER as ::core::ffi::c_int,
        name: b"kEnter\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_EQUAL as ::core::ffi::c_int,
        name: b"kEqual\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_LEFT as ::core::ffi::c_int,
        name: b"kLeft\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_RIGHT as ::core::ffi::c_int,
        name: b"kRight\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_UP as ::core::ffi::c_int,
        name: b"kUp\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_DOWN as ::core::ffi::c_int,
        name: b"kDown\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_PAGE_UP as ::core::ffi::c_int,
        name: b"kPageUp\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_PAGE_DOWN as ::core::ffi::c_int,
        name: b"kPageDown\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_HOME as ::core::ffi::c_int,
        name: b"kHome\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_END as ::core::ffi::c_int,
        name: b"kEnd\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_INSERT as ::core::ffi::c_int,
        name: b"kInsert\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_DELETE as ::core::ffi::c_int,
        name: b"kDel\0".as_ptr() as *const ::core::ffi::c_char,
    },
    kitty_key_map_entry {
        key: KITTY_KEY_KP_BEGIN as ::core::ffi::c_int,
        name: b"kOrigin\0".as_ptr() as *const ::core::ffi::c_char,
    },
]);
static kitty_key_map: GlobalCell<Map_int_ptr_t> = GlobalCell::new(MAP_INIT);
#[no_mangle]
pub unsafe extern "C" fn tinput_init(
    mut input: *mut TermInput,
    mut loop_0: *mut Loop,
    mut ti: *mut TerminfoEntry,
) {
    '_c2rust_label: {
        if (*input).loop_0.is_null() {
        } else {
            __assert_fail(
                b"input->loop == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                128 as ::core::ffi::c_uint,
                b"void tinput_init(TermInput *, Loop *, TerminfoEntry *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*input).loop_0 = loop_0;
    (*input).paste = 0 as int8_t;
    (*input).in_fd = STDIN_FILENO;
    (*input).ttimeout = p_ttimeout.get() != 0;
    (*input).ttimeoutlen = p_ttm.get();
    rstream_init_fd(loop_0, &raw mut (*input).read_stream, (*input).in_fd);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[kitty_key_map_entry; 77]>()
        .wrapping_div(::core::mem::size_of::<kitty_key_map_entry>())
        .wrapping_div(
            (::core::mem::size_of::<[kitty_key_map_entry; 77]>()
                .wrapping_rem(::core::mem::size_of::<kitty_key_map_entry>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        map_put_int_ptr_t(
            kitty_key_map.ptr(),
            (*kitty_key_map_entry.ptr())[i as usize].key,
            (*kitty_key_map_entry.ptr())[i as usize].name as ptr_t,
        );
        i = i.wrapping_add(1);
    }
    (*input).tk = termkey_new_abstract(
        ti,
        TERMKEY_FLAG_UTF8 as ::core::ffi::c_int
            | TERMKEY_FLAG_NOSTART as ::core::ffi::c_int
            | TERMKEY_FLAG_KEEPC0 as ::core::ffi::c_int,
    );
    termkey_set_buffer_size((*input).tk, INPUT_BUFFER_SIZE as size_t);
    termkey_hook_terminfo_getstr(
        (*input).tk,
        (*input).tk_ti_hook_fn,
        input as *mut ::core::ffi::c_void,
    );
    termkey_start((*input).tk);
    let mut curflags: ::core::ffi::c_int = termkey_get_canonflags((*input).tk);
    termkey_set_canonflags(
        (*input).tk,
        curflags | TERMKEY_CANON_DELBS as ::core::ffi::c_int,
    );
    uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*input).timer_handle);
    (*input).timer_handle.data = input as *mut ::core::ffi::c_void;
    uv_timer_init(&raw mut (*loop_0).uv, &raw mut (*input).bg_query_timer);
    (*input).bg_query_timer.data = input as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn tinput_destroy(mut input: *mut TermInput) {
    xfree((*kitty_key_map.ptr()).set.keys as *mut ::core::ffi::c_void);
    xfree((*kitty_key_map.ptr()).set.h.hash as *mut ::core::ffi::c_void);
    (*kitty_key_map.ptr()).set = SET_INIT;
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*kitty_key_map.ptr()).values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    uv_close(&raw mut (*input).timer_handle as *mut uv_handle_t, None);
    uv_close(&raw mut (*input).bg_query_timer as *mut uv_handle_t, None);
    rstream_may_close(&raw mut (*input).read_stream);
    termkey_destroy((*input).tk);
    (*input).loop_0 = ::core::ptr::null_mut::<Loop>();
}
#[no_mangle]
pub unsafe extern "C" fn tinput_start(mut input: *mut TermInput) {
    rstream_start(
        &raw mut (*input).read_stream,
        Some(
            tinput_read_cb
                as unsafe extern "C" fn(
                    *mut RStream,
                    *const ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                    bool,
                ) -> size_t,
        ),
        input as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tinput_stop(mut input: *mut TermInput) {
    rstream_stop(&raw mut (*input).read_stream);
    uv_timer_stop(&raw mut (*input).timer_handle);
    uv_timer_stop(&raw mut (*input).bg_query_timer);
}
unsafe extern "C" fn tinput_done_event(mut _argv: *mut *mut ::core::ffi::c_void) -> ! {
    os_exit(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn tinput_flush(mut input: *mut TermInput) {
    let mut keys: String_0 = String_0 {
        data: &raw mut (*input).key_buffer as *mut ::core::ffi::c_char,
        size: (*input).key_buffer_len,
    };
    if (*input).paste != 0 {
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 3] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 3];
        args.capacity = 3 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh0 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: keys },
        };
        let c2rust_fresh1 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh1 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: true },
        };
        let c2rust_fresh2 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh2 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: (*input).paste as Integer,
            },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_paste\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        if (*input).paste as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
            (*input).paste = 2 as int8_t;
        }
    } else if (*input).key_buffer_len > 0 as size_t {
        let mut args_0: Array = ARRAY_DICT_INIT;
        let mut args__items_0: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args_0.capacity = 1 as size_t;
        args_0.items = &raw mut args__items_0 as *mut Object;
        let c2rust_fresh3 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh3 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: keys },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_input\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
    }
    (*input).key_buffer_len = 0 as size_t;
}
unsafe extern "C" fn tinput_enqueue(
    mut input: *mut TermInput,
    mut buf: *const ::core::ffi::c_char,
    mut size: size_t,
) {
    if (*input).key_buffer_len > (KEY_BUFFER_SIZE as size_t).wrapping_sub(size) {
        tinput_flush(input);
    }
    let mut to_copy: size_t = if size < (0x1000 as size_t).wrapping_sub((*input).key_buffer_len) {
        size
    } else {
        (0x1000 as size_t).wrapping_sub((*input).key_buffer_len)
    };
    memcpy(
        (&raw mut (*input).key_buffer as *mut ::core::ffi::c_char)
            .offset((*input).key_buffer_len as isize) as *mut ::core::ffi::c_void,
        buf as *const ::core::ffi::c_void,
        to_copy,
    );
    (*input).key_buffer_len = (*input).key_buffer_len.wrapping_add(to_copy);
}
unsafe extern "C" fn handle_termkey_modifiers(
    mut key: *mut TermKeyKey,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) -> size_t {
    let mut len: size_t = 0 as size_t;
    if (*key).modifiers & TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int != 0 {
        len = len.wrapping_add(snprintf(
            buf.offset(len as isize),
            buflen.wrapping_sub(len),
            b"S-\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    if (*key).modifiers & TERMKEY_KEYMOD_ALT as ::core::ffi::c_int != 0 {
        len = len.wrapping_add(snprintf(
            buf.offset(len as isize),
            buflen.wrapping_sub(len),
            b"A-\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    if (*key).modifiers & TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int != 0 {
        len = len.wrapping_add(snprintf(
            buf.offset(len as isize),
            buflen.wrapping_sub(len),
            b"C-\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    '_c2rust_label: {
        if len < buflen {
        } else {
            __assert_fail(
                b"len < buflen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                240 as ::core::ffi::c_uint,
                b"size_t handle_termkey_modifiers(TermKeyKey *, char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return len;
}
unsafe extern "C" fn handle_more_modifiers(
    mut key: *mut TermKeyKey,
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) -> size_t {
    let mut len: size_t = 0 as size_t;
    if (*key).modifiers & KEYMOD_SUPER as ::core::ffi::c_int != 0 {
        len = len.wrapping_add(snprintf(
            buf.offset(len as isize),
            buflen.wrapping_sub(len),
            b"D-\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    if (*key).modifiers & KEYMOD_META as ::core::ffi::c_int != 0 {
        len = len.wrapping_add(snprintf(
            buf.offset(len as isize),
            buflen.wrapping_sub(len),
            b"T-\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    '_c2rust_label: {
        if len < buflen {
        } else {
            __assert_fail(
                b"len < buflen\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                272 as ::core::ffi::c_uint,
                b"size_t handle_more_modifiers(TermKeyKey *, char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return len;
}
unsafe extern "C" fn handle_kitty_key_protocol(
    mut input: *mut TermInput,
    mut key: *mut TermKeyKey,
) {
    let mut name: *const ::core::ffi::c_char =
        map_get_int_ptr_t(kitty_key_map.ptr(), (*key).code.codepoint) as *const ::core::ffi::c_char;
    if !name.is_null() {
        let mut buf: [::core::ffi::c_char; 64] = [0; 64];
        let mut len: size_t = 0 as size_t;
        let c2rust_fresh13 = len;
        len = len.wrapping_add(1);
        buf[c2rust_fresh13 as usize] = '<' as ::core::ffi::c_char;
        len = len.wrapping_add(handle_termkey_modifiers(
            key,
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
        ));
        len = len.wrapping_add(handle_more_modifiers(
            key,
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
        ));
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"%s>\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        ) as size_t);
        '_c2rust_label: {
            if len < ::core::mem::size_of::<[::core::ffi::c_char; 64]>() {
            } else {
                __assert_fail(
                    b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    286 as ::core::ffi::c_uint,
                    b"void handle_kitty_key_protocol(TermInput *, TermKeyKey *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        tinput_enqueue(input, &raw mut buf as *mut ::core::ffi::c_char, len);
    }
}
unsafe extern "C" fn forward_simple_utf8(mut input: *mut TermInput, mut key: *mut TermKeyKey) {
    let mut len: size_t = 0 as size_t;
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    let mut ptr: *mut ::core::ffi::c_char = &raw mut (*key).utf8 as *mut ::core::ffi::c_char;
    if (*key).code.codepoint >= 0xe000 as ::core::ffi::c_int
        && (*key).code.codepoint <= 0xf8ff as ::core::ffi::c_int
        && set_has_int(&raw mut (*kitty_key_map.ptr()).set, (*key).code.codepoint)
            as ::core::ffi::c_int
            != 0
    {
        handle_kitty_key_protocol(input, key);
        return;
    }
    while *ptr != 0 {
        if *ptr as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            len = len.wrapping_add(snprintf(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                b"<lt>\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t);
        } else {
            let c2rust_fresh14 = len;
            len = len.wrapping_add(1);
            buf[c2rust_fresh14 as usize] = *ptr;
        }
        '_c2rust_label: {
            if len < ::core::mem::size_of::<[::core::ffi::c_char; 64]>() {
            } else {
                __assert_fail(
                    b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    308 as ::core::ffi::c_uint,
                    b"void forward_simple_utf8(TermInput *, TermKeyKey *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ptr = ptr.offset(1);
    }
    tinput_enqueue(input, &raw mut buf as *mut ::core::ffi::c_char, len);
}
unsafe extern "C" fn forward_modified_utf8(mut input: *mut TermInput, mut key: *mut TermKeyKey) {
    let mut len: size_t = 0;
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_KEYSYM as ::core::ffi::c_int
        && (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_SUSPEND as ::core::ffi::c_int
    {
        len = snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            b"<C-Z>\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t;
    } else if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_UNICODE as ::core::ffi::c_int {
        len = termkey_strfkey(
            (*input).tk,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            key,
            (TERMKEY_FORMAT_ALTISMETA as ::core::ffi::c_int
                | TERMKEY_FORMAT_WRAPBRACKET as ::core::ffi::c_int) as TermKeyFormat,
        );
    } else {
        '_c2rust_label: {
            if (*key).modifiers != 0 {
            } else {
                __assert_fail(
                    b"key->modifiers\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    326 as ::core::ffi::c_uint,
                    b"void forward_modified_utf8(TermInput *, TermKeyKey *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*key).code.codepoint >= 0xe000 as ::core::ffi::c_int
            && (*key).code.codepoint <= 0xf8ff as ::core::ffi::c_int
            && set_has_int(&raw mut (*kitty_key_map.ptr()).set, (*key).code.codepoint)
                as ::core::ffi::c_int
                != 0
        {
            handle_kitty_key_protocol(input, key);
            return;
        }
        len = termkey_strfkey(
            (*input).tk,
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            key,
            (TERMKEY_FORMAT_ALTISMETA as ::core::ffi::c_int
                | TERMKEY_FORMAT_WRAPBRACKET as ::core::ffi::c_int) as TermKeyFormat,
        );
        if (*key).modifiers & TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int != 0
            && (*key).modifiers & TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int == 0
            && ((*key).code.codepoint as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && (*key).code.codepoint as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
        {
            '_c2rust_label_0: {
                if len.wrapping_add(2 as size_t)
                    < ::core::mem::size_of::<[::core::ffi::c_char; 64]>()
                {
                } else {
                    __assert_fail(
                        b"len + 2 < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        339 as ::core::ffi::c_uint,
                        b"void forward_modified_utf8(TermInput *, TermKeyKey *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memmove(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_void,
                (&raw mut buf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                len.wrapping_sub(1 as size_t),
            );
            buf[1 as ::core::ffi::c_int as usize] = 'S' as ::core::ffi::c_char;
            buf[2 as ::core::ffi::c_int as usize] = '-' as ::core::ffi::c_char;
            len = len.wrapping_add(2 as size_t);
        }
    }
    let mut more_buf: [::core::ffi::c_char; 25] = [0; 25];
    let mut more_len: size_t = handle_more_modifiers(
        key,
        &raw mut more_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 25]>(),
    );
    if more_len > 0 as size_t {
        '_c2rust_label_1: {
            if len.wrapping_add(more_len) < ::core::mem::size_of::<[::core::ffi::c_char; 64]>() {
            } else {
                __assert_fail(
                    b"len + more_len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    351 as ::core::ffi::c_uint,
                    b"void forward_modified_utf8(TermInput *, TermKeyKey *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        memmove(
            (&raw mut buf as *mut ::core::ffi::c_char)
                .offset(1 as ::core::ffi::c_int as isize)
                .offset(more_len as isize) as *mut ::core::ffi::c_void,
            (&raw mut buf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            len.wrapping_sub(1 as size_t),
        );
        memcpy(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize)
                as *mut ::core::ffi::c_void,
            &raw mut more_buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            more_len,
        );
        len = len.wrapping_add(more_len);
    }
    '_c2rust_label_2: {
        if len < ::core::mem::size_of::<[::core::ffi::c_char; 64]>() {
        } else {
            __assert_fail(
                b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                357 as ::core::ffi::c_uint,
                b"void forward_modified_utf8(TermInput *, TermKeyKey *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tinput_enqueue(input, &raw mut buf as *mut ::core::ffi::c_char, len);
}
unsafe extern "C" fn forward_mouse_event(mut input: *mut TermInput, mut key: *mut TermKeyKey) {
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    let mut len: size_t = 0 as size_t;
    let mut button: ::core::ffi::c_int = 0;
    let mut row: ::core::ffi::c_int = 0;
    let mut col: ::core::ffi::c_int = 0;
    static last_pressed_button: GlobalCell<::core::ffi::c_int> =
        GlobalCell::new(0 as ::core::ffi::c_int);
    let mut ev: TermKeyMouseEvent = TERMKEY_MOUSE_UNKNOWN;
    termkey_interpret_mouse(
        (*input).tk,
        key,
        &raw mut ev,
        &raw mut button,
        &raw mut row,
        &raw mut col,
    );
    if (ev as ::core::ffi::c_uint
        == TERMKEY_MOUSE_RELEASE as ::core::ffi::c_int as ::core::ffi::c_uint
        || ev as ::core::ffi::c_uint
            == TERMKEY_MOUSE_DRAG as ::core::ffi::c_int as ::core::ffi::c_uint)
        && button == 0 as ::core::ffi::c_int
    {
        button = last_pressed_button.get();
    }
    if button == 0 as ::core::ffi::c_int
        && ev as ::core::ffi::c_uint
            != TERMKEY_MOUSE_RELEASE as ::core::ffi::c_int as ::core::ffi::c_uint
        || ev as ::core::ffi::c_uint
            != TERMKEY_MOUSE_PRESS as ::core::ffi::c_int as ::core::ffi::c_uint
            && ev as ::core::ffi::c_uint
                != TERMKEY_MOUSE_DRAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && ev as ::core::ffi::c_uint
                != TERMKEY_MOUSE_RELEASE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    row -= 1;
    col -= 1;
    let c2rust_fresh12 = len;
    len = len.wrapping_add(1);
    buf[c2rust_fresh12 as usize] = '<' as ::core::ffi::c_char;
    len = len.wrapping_add(handle_termkey_modifiers(
        key,
        (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
        ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
    ));
    if button == 1 as ::core::ffi::c_int {
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"Left\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    } else if button == 2 as ::core::ffi::c_int {
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"Middle\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    } else if button == 3 as ::core::ffi::c_int {
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"Right\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    } else if button == 8 as ::core::ffi::c_int {
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"X1\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    } else if button == 9 as ::core::ffi::c_int {
        len = len.wrapping_add(snprintf(
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
            b"X2\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t);
    }
    match ev as ::core::ffi::c_uint {
        1 => {
            if button == 4 as ::core::ffi::c_int {
                len = len.wrapping_add(snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                    b"ScrollWheelUp\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
            } else if button == 5 as ::core::ffi::c_int {
                len = len.wrapping_add(snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                    b"ScrollWheelDown\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
            } else if button == 6 as ::core::ffi::c_int {
                len = len.wrapping_add(snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                    b"ScrollWheelLeft\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
            } else if button == 7 as ::core::ffi::c_int {
                len = len.wrapping_add(snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                    b"ScrollWheelRight\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
            } else {
                len = len.wrapping_add(snprintf(
                    (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                    b"Mouse\0".as_ptr() as *const ::core::ffi::c_char,
                ) as size_t);
                last_pressed_button.set(button);
            }
        }
        2 => {
            len = len.wrapping_add(snprintf(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                b"Drag\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t);
        }
        3 => {
            len = len.wrapping_add(snprintf(
                (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
                if button != 0 {
                    b"Release\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"MouseMove\0".as_ptr() as *const ::core::ffi::c_char
                },
            ) as size_t);
            last_pressed_button.set(0 as ::core::ffi::c_int);
        }
        0 => {
            abort();
        }
        _ => {}
    }
    len = len.wrapping_add(snprintf(
        (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize),
        ::core::mem::size_of::<[::core::ffi::c_char; 64]>().wrapping_sub(len),
        b"><%d,%d>\0".as_ptr() as *const ::core::ffi::c_char,
        col,
        row,
    ) as size_t);
    '_c2rust_label: {
        if len < ::core::mem::size_of::<[::core::ffi::c_char; 64]>() {
        } else {
            __assert_fail(
                b"len < sizeof(buf)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                430 as ::core::ffi::c_uint,
                b"void forward_mouse_event(TermInput *, TermKeyKey *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tinput_enqueue(input, &raw mut buf as *mut ::core::ffi::c_char, len);
}
unsafe extern "C" fn tk_getkey(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut force: bool,
) -> TermKeyResult {
    return (if force as ::core::ffi::c_int != 0 {
        termkey_getkey_force(tk, key) as ::core::ffi::c_uint
    } else {
        termkey_getkey(tk, key) as ::core::ffi::c_uint
    }) as TermKeyResult;
}
unsafe extern "C" fn tk_getkeys(mut input: *mut TermInput, mut force: bool) {
    let mut key: TermKeyKey = TermKeyKey {
        type_0: TERMKEY_TYPE_UNICODE,
        code: C2Rust_Unnamed_18 { codepoint: 0 },
        modifiers: 0,
        event: TERMKEY_EVENT_UNKNOWN,
        utf8: [0; 7],
    };
    let mut result: TermKeyResult = TERMKEY_RES_NONE;
    loop {
        result = tk_getkey((*input).tk, &raw mut key, force);
        if result as ::core::ffi::c_uint
            != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        match key.event as ::core::ffi::c_uint {
            1 | 2 => {}
            _ => {
                continue;
            }
        }
        if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNICODE as ::core::ffi::c_int
            && key.modifiers & KEYMOD_RECOGNIZED as ::core::ffi::c_int == 0
        {
            forward_simple_utf8(input, &raw mut key);
        } else if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNICODE as ::core::ffi::c_int
            || key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_FUNCTION as ::core::ffi::c_int
            || key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_KEYSYM as ::core::ffi::c_int
        {
            forward_modified_utf8(input, &raw mut key);
        } else if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_MOUSE as ::core::ffi::c_int {
            forward_mouse_event(input, &raw mut key);
        } else if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_MODEREPORT as ::core::ffi::c_int
        {
            handle_modereport(input, &raw mut key);
        } else if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNKNOWN_CSI as ::core::ffi::c_int
        {
            handle_unknown_csi(input, &raw mut key);
        } else if key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_OSC as ::core::ffi::c_int
            || key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_DCS as ::core::ffi::c_int
            || key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_APC as ::core::ffi::c_int
        {
            handle_term_response(input, &raw mut key);
        }
    }
    if result as ::core::ffi::c_uint
        != TERMKEY_RES_AGAIN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*input).ttimeout as ::core::ffi::c_int != 0 && (*input).ttimeoutlen >= 0 as OptInt {
        uv_timer_stop(&raw mut (*input).timer_handle);
        uv_timer_start(
            &raw mut (*input).timer_handle,
            Some(tinput_timer_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
            (*input).ttimeoutlen as uint64_t,
            0 as uint64_t,
        );
    } else {
        tk_getkeys(input, true_0 != 0);
    };
}
unsafe extern "C" fn tinput_timer_cb(mut handle: *mut uv_timer_t) {
    let mut input: *mut TermInput = (*handle).data as *mut TermInput;
    let mut size: size_t = rstream_available(&raw mut (*input).read_stream);
    if size != 0 {
        let mut consumed: size_t =
            handle_raw_buffer(input, true_0 != 0, (*input).read_stream.read_pos, size);
        rstream_consume(&raw mut (*input).read_stream, consumed);
    }
    tk_getkeys(input, true_0 != 0);
    tinput_flush(input);
}
unsafe extern "C" fn bg_query_timer_cb(mut handle: *mut uv_timer_t) {
    let mut input: *mut TermInput = (*handle).data as *mut TermInput;
    tui_query_bg_color((*input).tui_data);
}
unsafe extern "C" fn handle_focus_event(
    mut _input: *mut TermInput,
    mut ptr: *const ::core::ffi::c_char,
    mut size: size_t,
) -> size_t {
    if size >= 3 as size_t
        && (memcmp(
            ptr as *const ::core::ffi::c_void,
            b"\x1B[I\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            3 as size_t,
        ) == 0
            || memcmp(
                ptr as *const ::core::ffi::c_void,
                b"\x1B[O\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                3 as size_t,
            ) == 0)
    {
        let mut focus_gained: bool = *ptr.offset(2 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            == 'I' as ::core::ffi::c_int;
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh15 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh15 as isize) = object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: focus_gained,
            },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_ui_set_focus\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        return 3 as size_t;
    }
    return 0 as size_t;
}
pub const START_PASTE: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"\x1B[200~\0") };
pub const END_PASTE: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"\x1B[201~\0") };
unsafe extern "C" fn handle_bracketed_paste(
    mut input: *mut TermInput,
    mut ptr: *const ::core::ffi::c_char,
    mut size: size_t,
    mut incomplete: *mut bool,
) -> size_t {
    if size >= 6 as size_t
        && (memcmp(
            ptr as *const ::core::ffi::c_void,
            START_PASTE.as_ptr() as *const ::core::ffi::c_void,
            6 as size_t,
        ) == 0
            || memcmp(
                ptr as *const ::core::ffi::c_void,
                END_PASTE.as_ptr() as *const ::core::ffi::c_void,
                6 as size_t,
            ) == 0)
    {
        let mut enable: bool = *ptr.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '0' as ::core::ffi::c_int;
        if (*input).paste as ::core::ffi::c_int != 0 && enable as ::core::ffi::c_int != 0 {
            return 0 as size_t;
        }
        if ((*input).paste != 0) as ::core::ffi::c_int == enable as ::core::ffi::c_int {
            return 6 as size_t;
        }
        if enable {
            tinput_flush(input);
            (*input).paste = 1 as int8_t;
        } else if (*input).paste != 0 {
            (*input).paste = (if (*input).paste as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                3 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            }) as int8_t;
            tinput_flush(input);
            (*input).paste = 0 as int8_t;
        }
        return 6 as size_t;
    } else if size < 6 as size_t
        && (memcmp(
            ptr as *const ::core::ffi::c_void,
            START_PASTE.as_ptr() as *const ::core::ffi::c_void,
            size,
        ) == 0
            || memcmp(
                ptr as *const ::core::ffi::c_void,
                END_PASTE.as_ptr() as *const ::core::ffi::c_void,
                size,
            ) == 0)
    {
        *incomplete = true_0 != 0;
        return 0 as size_t;
    }
    return 0 as size_t;
}
unsafe extern "C" fn handle_term_response(mut input: *mut TermInput, mut key: *const TermKeyKey) {
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if termkey_interpret_string((*input).tk, key, &raw mut str) as ::core::ffi::c_uint
        == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        '_c2rust_label: {
            if !str.is_null() {
            } else {
                __assert_fail(
                    b"str != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    579 as ::core::ffi::c_uint,
                    b"void handle_term_response(TermInput *, const TermKeyKey *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_DCS as ::core::ffi::c_int
            && (strnequal(
                str,
                b"1$r4:3m\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            ) as ::core::ffi::c_int
                != 0
                || strnequal(
                    str,
                    b"1$r0;4:3m\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                ) as ::core::ffi::c_int
                    != 0)
        {
            tui_enable_extended_underline((*input).tui_data);
        }
        let mut args: Array = ARRAY_DICT_INIT;
        let mut args__items: [Object; 2] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        args.capacity = 2 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh4 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh4 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: b"termresponse\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 13]>()
                        .wrapping_sub(1 as size_t),
                },
            },
        };
        let mut response: StringBuilder = StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        match (*key).type_0 as ::core::ffi::c_int {
            7 => {
                kv_do_printf(
                    &raw mut response,
                    b"\x1B]%s\0".as_ptr() as *const ::core::ffi::c_char,
                    str,
                );
            }
            6 => {
                kv_do_printf(
                    &raw mut response,
                    b"\x1BP%s\0".as_ptr() as *const ::core::ffi::c_char,
                    str,
                );
            }
            8 => {
                kv_do_printf(
                    &raw mut response,
                    b"\x1B_%s\0".as_ptr() as *const ::core::ffi::c_char,
                    str,
                );
            }
            _ => {
                unreachable!();
            }
        }
        let c2rust_fresh5 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh5 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: response.items,
                    size: response.size,
                },
            },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_ui_term_event\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        xfree(response.items as *mut ::core::ffi::c_void);
        response.capacity = 0 as size_t;
        response.size = response.capacity;
        response.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}
unsafe extern "C" fn handle_primary_device_attr(
    mut input: *mut TermInput,
    mut params: *mut TermKeyCsiParam,
    mut nparams: size_t,
) {
    if (*input).callbacks.primary_device_attr.is_some() {
        let mut cb_save: Option<unsafe extern "C" fn(*mut TUIData) -> ()> =
            (*input).callbacks.primary_device_attr;
        (*input).callbacks.primary_device_attr = None;
        cb_save.expect("non-null function pointer")((*input).tui_data);
    }
    if nparams == 0 as size_t {
        return;
    }
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 2] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 2];
    args.capacity = 2 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh8 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh8 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: String_0 {
                data: b"termresponse\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            },
        },
    };
    let mut response: StringBuilder = StringBuilder {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if strlen(b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char) > 0 as size_t {
        if response.capacity
            < response
                .size
                .wrapping_add(strlen(b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char))
        {
            response.capacity = response
                .size
                .wrapping_add(strlen(b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char));
            response.capacity = response.capacity.wrapping_sub(1);
            response.capacity |= response.capacity >> 1 as ::core::ffi::c_int;
            response.capacity |= response.capacity >> 2 as ::core::ffi::c_int;
            response.capacity |= response.capacity >> 4 as ::core::ffi::c_int;
            response.capacity |= response.capacity >> 8 as ::core::ffi::c_int;
            response.capacity |= response.capacity >> 16 as ::core::ffi::c_int;
            response.capacity = response.capacity.wrapping_add(1);
            response.capacity = response.capacity;
            response.items = xrealloc(
                response.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(response.capacity),
            ) as *mut ::core::ffi::c_char;
        }
        '_c2rust_label: {
            if !response.items.is_null() {
            } else {
                __assert_fail(
                    b"(response).items\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    637 as ::core::ffi::c_uint,
                    b"void handle_primary_device_attr(TermInput *, TermKeyCsiParam *, size_t)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        memcpy(
            response.items.offset(response.size as isize) as *mut ::core::ffi::c_void,
            b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>()
                .wrapping_mul(strlen(b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char)),
        );
        response.size = response
            .size
            .wrapping_add(strlen(b"\x1B[?\0".as_ptr() as *const ::core::ffi::c_char));
    }
    let mut i: size_t = 0 as size_t;
    '_out: {
        while i < nparams {
            let mut arg: ::core::ffi::c_int = 0;
            if termkey_interpret_csi_param(
                *params.offset(i as isize),
                &raw mut arg,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break '_out;
            }
            kv_do_printf(
                &raw mut response,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                arg,
            );
            if i < nparams.wrapping_sub(1 as size_t) {
                if response.size == response.capacity {
                    response.capacity = if response.capacity != 0 {
                        response.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    response.items = xrealloc(
                        response.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(response.capacity),
                    ) as *mut ::core::ffi::c_char;
                } else {
                };
                let c2rust_fresh9 = response.size;
                response.size = response.size.wrapping_add(1);
                *response.items.offset(c2rust_fresh9 as isize) = ';' as ::core::ffi::c_char;
            }
            i = i.wrapping_add(1);
        }
        if response.size == response.capacity {
            response.capacity = if response.capacity != 0 {
                response.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            response.items = xrealloc(
                response.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(response.capacity),
            ) as *mut ::core::ffi::c_char;
        } else {
        };
        let c2rust_fresh10 = response.size;
        response.size = response.size.wrapping_add(1);
        *response.items.offset(c2rust_fresh10 as isize) = 'c' as ::core::ffi::c_char;
        let c2rust_fresh11 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh11 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: String_0 {
                    data: response.items,
                    size: response.size,
                },
            },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_ui_term_event\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    }
    xfree(response.items as *mut ::core::ffi::c_void);
    response.capacity = 0 as size_t;
    response.size = response.capacity;
    response.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn handle_modereport(mut input: *mut TermInput, mut key: *const TermKeyKey) {
    let mut initial: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    let mut value: ::core::ffi::c_int = 0;
    if termkey_interpret_modereport(
        (*input).tk,
        key,
        &raw mut initial,
        &raw mut mode,
        &raw mut value,
    ) as ::core::ffi::c_uint
        == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tui_handle_term_mode((*input).tui_data, mode as TermMode, value as TermModeState);
    }
}
unsafe extern "C" fn handle_unknown_csi(mut input: *mut TermInput, mut key: *const TermKeyKey) {
    let mut params: [TermKeyCsiParam; 16] = [TermKeyCsiParam {
        param: ::core::ptr::null::<::core::ffi::c_uchar>(),
        length: 0,
    }; 16];
    let mut nparams: size_t = 16 as size_t;
    let mut cmd: ::core::ffi::c_uint = 0;
    if termkey_interpret_csi(
        (*input).tk,
        key,
        &raw mut params as *mut TermKeyCsiParam,
        &raw mut nparams,
        &raw mut cmd,
    ) as ::core::ffi::c_uint
        != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    let mut _intermediate: uint8_t =
        (cmd >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint) as uint8_t;
    let mut initial: uint8_t =
        (cmd >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_uint) as uint8_t;
    let mut command: uint8_t = (cmd & 0xff as ::core::ffi::c_uint) as uint8_t;
    match command as ::core::ffi::c_int {
        117 => match initial as ::core::ffi::c_int {
            63 => {
                (*input).key_encoding = kKeyEncodingKitty;
            }
            _ => {}
        },
        99 => match initial as ::core::ffi::c_int {
            63 => {
                handle_primary_device_attr(input, &raw mut params as *mut TermKeyCsiParam, nparams);
            }
            _ => {}
        },
        116 => {
            if nparams == 5 as size_t {
                let mut args: [::core::ffi::c_int; 3] = [0; 3];
                let mut i: size_t = 0 as size_t;
                while i < ::core::mem::size_of::<[::core::ffi::c_int; 3]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_int; 3]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                            == 0) as ::core::ffi::c_int as usize,
                    )
                {
                    if termkey_interpret_csi_param(
                        params[i as usize],
                        (&raw mut args as *mut ::core::ffi::c_int).offset(i as isize),
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ::core::ptr::null_mut::<size_t>(),
                    ) as ::core::ffi::c_uint
                        != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return;
                    }
                    i = i.wrapping_add(1);
                }
                if args[0 as ::core::ffi::c_int as usize] == 48 as ::core::ffi::c_int {
                    let mut height_chars: ::core::ffi::c_int =
                        args[1 as ::core::ffi::c_int as usize];
                    let mut width_chars: ::core::ffi::c_int =
                        args[2 as ::core::ffi::c_int as usize];
                    tui_set_size((*input).tui_data, width_chars, height_chars);
                }
            }
        }
        110 => {
            if nparams == 1 as size_t {
                let mut arg: ::core::ffi::c_int = 0;
                if termkey_interpret_csi_param(
                    params[0 as ::core::ffi::c_int as usize],
                    &raw mut arg,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<size_t>(),
                ) as ::core::ffi::c_uint
                    != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return;
                }
                let mut args_0: Array = ARRAY_DICT_INIT;
                let mut args__items: [Object; 2] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }; 2];
                args_0.capacity = 2 as size_t;
                args_0.items = &raw mut args__items as *mut Object;
                let c2rust_fresh6 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.offset(c2rust_fresh6 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: b"termresponse\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 13]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                };
                let mut response: StringBuilder = StringBuilder {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                };
                kv_do_printf(
                    &raw mut response,
                    b"\x1B[%dn\0".as_ptr() as *const ::core::ffi::c_char,
                    arg,
                );
                let c2rust_fresh7 = args_0.size;
                args_0.size = args_0.size.wrapping_add(1);
                *args_0.items.offset(c2rust_fresh7 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: response.items,
                            size: response.size,
                        },
                    },
                };
                rpc_send_event(
                    ui_client_channel_id.get(),
                    b"nvim_ui_term_event\0".as_ptr() as *const ::core::ffi::c_char,
                    args_0,
                );
                xfree(response.items as *mut ::core::ffi::c_void);
                response.capacity = 0 as size_t;
                response.size = response.capacity;
                response.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else if nparams == 2 as size_t {
                let mut args_1: [::core::ffi::c_int; 2] = [0; 2];
                let mut i_0: size_t = 0 as size_t;
                while i_0
                    < ::core::mem::size_of::<[::core::ffi::c_int; 2]>()
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>())
                        .wrapping_div(
                            (::core::mem::size_of::<[::core::ffi::c_int; 2]>()
                                .wrapping_rem(::core::mem::size_of::<::core::ffi::c_int>())
                                == 0) as ::core::ffi::c_int as usize,
                        )
                {
                    if termkey_interpret_csi_param(
                        params[i_0 as usize],
                        (&raw mut args_1 as *mut ::core::ffi::c_int).offset(i_0 as isize),
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ::core::ptr::null_mut::<size_t>(),
                    ) as ::core::ffi::c_uint
                        != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return;
                    }
                    i_0 = i_0.wrapping_add(1);
                }
                if args_1[0 as ::core::ffi::c_int as usize] == 997 as ::core::ffi::c_int {
                    if uv_timer_get_due_in(&raw mut (*input).bg_query_timer) > 0 as uint64_t {
                        return;
                    }
                    uv_timer_start(
                        &raw mut (*input).bg_query_timer,
                        Some(bg_query_timer_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
                        100 as uint64_t,
                        0 as uint64_t,
                    );
                }
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn handle_raw_buffer(
    mut input: *mut TermInput,
    mut force: bool,
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
) -> size_t {
    let mut ptr: *const ::core::ffi::c_char = data;
    loop {
        's_4: {
            if !force {
                let mut consumed: size_t = handle_focus_event(input, ptr, size);
                if consumed != 0 {
                    ptr = ptr.offset(consumed as isize);
                    size = size.wrapping_sub(consumed);
                    break 's_4;
                } else {
                    let mut incomplete: bool = false_0 != 0;
                    consumed = handle_bracketed_paste(input, ptr, size, &raw mut incomplete);
                    if incomplete {
                        '_c2rust_label: {
                            if consumed == 0 as size_t {
                            } else {
                                __assert_fail(
                                    b"consumed == 0\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/tui/input.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    799 as ::core::ffi::c_uint,
                                    b"size_t handle_raw_buffer(TermInput *, _Bool, const char *, size_t)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        return ptr.offset_from(data) as size_t;
                    } else if consumed != 0 {
                        ptr = ptr.offset(consumed as isize);
                        size = size.wrapping_sub(consumed);
                        break 's_4;
                    }
                }
            }
            let mut count: size_t = 0 as size_t;
            let mut i: size_t = 0 as size_t;
            while i < size {
                count = i.wrapping_add(1 as size_t);
                if *ptr.offset(i as isize) as ::core::ffi::c_int == '\u{1b}' as ::core::ffi::c_int
                    && count > 1 as size_t
                {
                    count = count.wrapping_sub(1);
                    break;
                } else {
                    i = i.wrapping_add(1);
                }
            }
            if (*input).paste != 0 {
                tinput_enqueue(input, ptr, count);
                ptr = ptr.offset(count as isize);
                size = size.wrapping_sub(count);
            } else {
                let to_use: size_t = if count < size { count } else { size };
                if to_use > termkey_get_buffer_remaining((*input).tk) {
                    let delta: size_t =
                        to_use.wrapping_sub(termkey_get_buffer_remaining((*input).tk));
                    let bufsize: size_t = termkey_get_buffer_size((*input).tk);
                    if termkey_set_buffer_size(
                        (*input).tk,
                        if bufsize.wrapping_add(delta) > bufsize.wrapping_mul(2 as size_t) {
                            bufsize.wrapping_add(delta)
                        } else {
                            bufsize.wrapping_mul(2 as size_t)
                        },
                    ) == 0
                    {
                        abort();
                    }
                }
                let mut consumed_0: size_t = termkey_push_bytes((*input).tk, ptr, to_use);
                '_c2rust_label_0: {
                    if consumed_0 <= to_use {
                    } else {
                        __assert_fail(
                            b"consumed <= to_use\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/tui/input.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            849 as ::core::ffi::c_uint,
                            b"size_t handle_raw_buffer(TermInput *, _Bool, const char *, size_t)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                ptr = ptr.offset(consumed_0 as isize);
                size = size.wrapping_sub(consumed_0);
                tk_getkeys(input, false_0 != 0);
            }
        }
        if size == 0 {
            break;
        }
    }
    let tk_size: size_t = termkey_get_buffer_size((*input).tk);
    let tk_remaining: size_t = termkey_get_buffer_remaining((*input).tk);
    let tk_count: size_t = tk_size.wrapping_sub(tk_remaining);
    if tk_count < INPUT_BUFFER_SIZE as size_t && tk_size > INPUT_BUFFER_SIZE as size_t {
        if termkey_set_buffer_size((*input).tk, INPUT_BUFFER_SIZE as size_t) == 0 {
            abort();
        }
    }
    return ptr.offset_from(data) as size_t;
}
unsafe extern "C" fn tinput_read_cb(
    mut _stream: *mut RStream,
    mut buf: *const ::core::ffi::c_char,
    mut count_: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut input: *mut TermInput = data as *mut TermInput;
    let mut consumed: size_t = handle_raw_buffer(input, false_0 != 0, buf, count_);
    tinput_flush(input);
    if eof {
        loop_schedule_fast(
            main_loop.ptr(),
            Event {
                handler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> !>,
                    argv_callback,
                >(Some(
                    tinput_done_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> !,
                )),
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
        return consumed;
    }
    if consumed < count_ {
        let mut ms: int64_t = if (*input).ttimeout as ::core::ffi::c_int != 0 {
            if (*input).ttimeoutlen >= 0 as OptInt {
                (*input).ttimeoutlen as int64_t
            } else {
                0 as int64_t
            }
        } else {
            0 as int64_t
        };
        uv_timer_stop(&raw mut (*input).timer_handle);
        uv_timer_start(
            &raw mut (*input).timer_handle,
            Some(tinput_timer_cb as unsafe extern "C" fn(*mut uv_timer_t) -> ()),
            ms as uint32_t as uint64_t,
            0 as uint64_t,
        );
    }
    return consumed;
}
pub const KEY_BUFFER_SIZE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
