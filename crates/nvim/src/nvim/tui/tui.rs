use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::cursor_shape::shape_table;
use crate::src::nvim::event::libuv::{
    uv_chdir, uv_close, uv_is_closing, uv_loop_close, uv_loop_init, uv_pipe_init, uv_pipe_open,
    uv_run, uv_sleep, uv_strerror, uv_timer_init, uv_timer_start, uv_tty_reset_mode, uv_write,
};
use crate::src::nvim::event::r#loop::{
    loop_poll_events, loop_purge, loop_size, process_events_until,
};
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
    ARENA_EMPTY, arena_finish, arena_mem_free, arena_strdup, strequal, xcalloc, xfree, xrealloc,
    xstrdup,
};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
use crate::src::nvim::os::env::{os_getenv, os_getenv_noalloc};
use crate::src::nvim::os::input::os_isatty;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, fclose, fopen, fprintf, kill, memset, snprintf, sscanf, strlen, tcgetattr,
};
use crate::src::nvim::strings::kv_do_printf;
use crate::src::nvim::tui::input::{
    TermInput, tinput_destroy, tinput_init, tinput_start, tinput_stop,
};
use crate::src::nvim::tui::output::{
    TERMINFO_SEQ_LIMIT, flush_buf, out, out_cstr, out_fmt, out_raw, terminfo_out, terminfo_print,
    terminfo_print_num,
};
use crate::src::nvim::tui::quirks::{Terminal, TerminfoExt, augment_terminfo, patch_terminfo_bugs};
use crate::src::nvim::tui::terminfo::caps::{
    kTerm_carriage_return, kTerm_change_scroll_region, kTerm_clear_screen, kTerm_clr_eol,
    kTerm_clr_eos, kTerm_cursor_address, kTerm_cursor_down, kTerm_cursor_home, kTerm_cursor_left,
    kTerm_cursor_normal, kTerm_cursor_right, kTerm_cursor_up, kTerm_delete_line,
    kTerm_enter_blink_mode, kTerm_enter_bold_mode, kTerm_enter_ca_mode, kTerm_enter_dim_mode,
    kTerm_enter_italics_mode, kTerm_enter_reverse_mode, kTerm_enter_secure_mode,
    kTerm_enter_standout_mode, kTerm_enter_strikethrough_mode, kTerm_enter_underline_mode,
    kTerm_erase_chars, kTerm_exit_attribute_mode, kTerm_exit_ca_mode, kTerm_from_status_line,
    kTerm_insert_line, kTerm_keypad_local, kTerm_keypad_xmit, kTerm_parm_delete_line,
    kTerm_parm_down_cursor, kTerm_parm_insert_line, kTerm_parm_left_cursor,
    kTerm_parm_right_cursor, kTerm_parm_up_cursor, kTerm_reset_cursor_color,
    kTerm_reset_cursor_style, kTerm_set_a_background, kTerm_set_a_foreground, kTerm_set_attributes,
    kTerm_set_cursor_color, kTerm_set_cursor_style, kTerm_set_lr_margin, kTerm_set_rgb_background,
    kTerm_set_rgb_foreground, kTerm_set_underline_style, kTerm_to_status_line,
};
use crate::src::nvim::tui::terminfo::{
    terminfo_from_builtin, terminfo_from_database, terminfo_info_msg,
};
pub use crate::src::nvim::types::{
    __builtin_va_list, __gnuc_va_list, __off_t, __off64_t, __pid_t, __pthread_internal_list,
    __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __va_list_tag, _IO_FILE,
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, Arena, ArenaMem, Array, Boolean,
    CursorShape, Dict, FILE, Float, HlAttrs, Integer, KeyEncoding, KeyValuePair, LineFlags, Loop,
    LuaRef, MHPutStatus, MapHash, MultiQueue, Object, ObjectType, OptInt, Proc, ProcType, QUEUE,
    RStream, RgbValue, ScopeType, Set_cstr_t, SignalWatcher, Stream, String_0, StringBuilder,
    TPVAR, TermKey, TermKey_Terminfo_Getstr_Hook, TermMode, TermModeState, TerminfoEntry, UCell,
    UGrid, VarLockStatus, cc_t, consumed_blk, cstr_t, cursorentry_T, dict_T, dictvar_S, hash_T,
    hashitem_T, hashtab_T, int8_t, int16_t, int32_t, int64_t, internal_proc_cb, key_value_pair,
    loop_0, multiqueue, object, object_data as C2Rust_Unnamed_13, proc, proc_exit_cb,
    proc_state_cb, pthread_mutex_t, pthread_rwlock_t, queue, rstream, sattr_T, schar_T, signal_cb,
    signal_close_cb, signal_watcher, size_t, speed_t, ssize_t, stream, stream_close_cb,
    stream_read_cb, stream_uv as C2Rust_Unnamed_15, stream_write_cb, tcflag_t, termios, uint8_t,
    uint32_t, uint64_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb, uv_async_cb,
    uv_async_s, uv_async_s_u as C2Rust_Unnamed_3, uv_async_t, uv_buf_t, uv_close_cb, uv_connect_cb,
    uv_connect_s, uv_connect_t, uv_connection_cb, uv_file, uv_handle_s,
    uv_handle_s_u as C2Rust_Unnamed_0, uv_handle_t, uv_handle_type, uv_idle_cb, uv_idle_s,
    uv_idle_s_u as C2Rust_Unnamed_12, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_4, uv_loop_s_timer_heap as C2Rust_Unnamed_2, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_8, uv_pipe_t, uv_read_cb, uv_req_type,
    uv_run_mode, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t, uv_signal_cb,
    uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed, uv_signal_s_u as C2Rust_Unnamed_1,
    uv_signal_t, uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_6, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_7, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_10, uv_timer_s_u as C2Rust_Unnamed_11, uv_timer_t,
    uv_write_cb, uv_write_s, uv_write_t, va_list,
};
use crate::src::nvim::ui_client::{ui_client_attach, ui_client_detach, ui_client_set_size};
unsafe extern "C" {
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
}
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_EINTR: C2Rust_Unnamed_5 = -4;
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
pub const UV_RUN_DEFAULT: uv_run_mode = 0;
pub type uv_tty_mode_t = ::core::ffi::c_uint;
pub const UV_TTY_MODE_IO: uv_tty_mode_t = 2;
pub const UV_TTY_MODE_NORMAL: uv_tty_mode_t = 0;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HL_FG_INDEXED: C2Rust_Unnamed_16 = 4096;
pub const HL_BG_INDEXED: C2Rust_Unnamed_16 = 2048;
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
pub const kMHExisting: MHPutStatus = 0;
pub type ModeShape = ::core::ffi::c_uint;
pub const SHAPE_IDX_N: ModeShape = 0;
pub const SHAPE_VER: CursorShape = 2;
pub const SHAPE_HOR: CursorShape = 1;
pub const SHAPE_BLOCK: CursorShape = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const kLineFlagWrap: C2Rust_Unnamed_17 = 1;
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
    pub terminfo_ext: TerminfoExt,
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
pub struct C2Rust_Unnamed_19 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlAttrs,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub grapheme_clusters_theme_updates_resize_events: [u8; 1],
}
crate::bitfield_accessors! {
    impl C2Rust_Unnamed_20.grapheme_clusters_theme_updates_resize_events {
        0..=0 => grapheme_clusters, set_grapheme_clusters: bool;
        1..=1 => theme_updates, set_theme_updates: bool;
        2..=2 => resize_events, set_resize_events: bool;
    }
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
pub const kKeyEncodingXterm: KeyEncoding = 2;
pub const kKeyEncodingLegacy: KeyEncoding = 0;
pub const kTermModeResizeEvents: TermMode = 2048;
pub const kTermModeThemeUpdates: TermMode = 2031;
pub const kTermModeGraphemeClusters: TermMode = 2027;
pub const kTermModeSynchronizedOutput: TermMode = 2026;
pub const kTermModeBracketedPaste: TermMode = 2004;
pub const kTermModeMouseSGRExt: TermMode = 1006;
pub const kTermModeMouseAnyEvent: TermMode = 1003;
pub const kTermModeMouseButtonEvent: TermMode = 1002;
pub const kTermModeLeftAndRightMargins: TermMode = 69;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
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
static cursor_style_enabled: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static urls: GlobalCell<Set_cstr_t> = GlobalCell::new(SET_INIT);
pub unsafe fn tui_start(
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
    out_raw(tui, (&raw const buf).cast(), len as usize);
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
    out_raw(tui, (&raw const buf).cast(), len as usize);
}
pub unsafe fn tui_handle_term_mode(
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
    out(tui, b"\x1B[0m\x1B[4:3m\x1BP$qm\x1B\\");
    (*tui).print_attr_id = -1 as ::core::ffi::c_int;
}
pub unsafe fn tui_enable_extended_underline(mut tui: *mut TUIData) {
    if (*tui).ti.defs[kTerm_set_underline_style as usize].is_null() {
        (*tui).ti.defs[kTerm_set_underline_style as usize] = c"\x1b[4:%p1%dm".as_ptr();
    }
    (*tui).can_set_underline_color = true_0 != 0;
}
unsafe extern "C" fn tui_query_kitty_keyboard(mut tui: *mut TUIData) {
    (*tui).input.callbacks.primary_device_attr =
        Some(tui_set_key_encoding as unsafe extern "C" fn(*mut TUIData) -> ());
    out(tui, b"\x1B[?u\x1B[c");
}
pub unsafe extern "C" fn tui_set_key_encoding(tui: *mut TUIData) {
    let mut tui: *mut TUIData = tui.cast::<TUIData>();
    match (*tui).input.key_encoding as ::core::ffi::c_uint {
        1 => {
            out(tui, b"\x1B[>3u");
        }
        2 => {
            out(tui, b"\x1B[>4;2m");
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn tui_reset_key_encoding(mut tui: *mut TUIData) {
    match (*tui).input.key_encoding as ::core::ffi::c_uint {
        1 => {
            out(tui, b"\x1B[<u");
        }
        2 => {
            out(tui, b"\x1B[>4;0m");
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn tui_query_bg_color_noflush(mut tui: *mut TUIData) {
    out(tui, b"\x1B]11;?\x07\x1B[5n");
}
pub unsafe fn tui_query_bg_color(mut tui: *mut TUIData) {
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
    (*tui).terminfo_ext = TerminfoExt::default();
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
    let term_name: Option<&::core::ffi::CStr> =
        (!term.is_null()).then(|| ::core::ffi::CStr::from_ptr(term));
    if let Some(name) = term_name
        && let Some(entry) = terminfo_from_database(name, &raw mut (*tui).ti_arena)
    {
        (*tui).ti = entry;
        (*tui).term = arena_strdup(&raw mut (*tui).ti_arena, term);
        (*tui).terminfo_found_in_db = true_0 != 0;
    }
    if !(*tui).terminfo_found_in_db {
        let (builtin_name, entry) = terminfo_from_builtin(term_name);
        (*tui).ti = entry;
        (*tui).term = builtin_name.as_ptr().cast_mut();
    }
    let quirks = Terminal::identify(term_name);
    (*tui).screen_or_tmux = quirks.screen || quirks.tmux;
    (*tui).rgb = quirks.has_truecolor(&(*tui).ti);
    patch_terminfo_bugs(&mut (*tui).ti, &raw mut (*tui).ti_arena, &quirks);
    let augmented = augment_terminfo(&mut (*tui).ti, &quirks);
    (*tui).can_resize_screen = augmented.can_resize_screen;
    (*tui).can_set_title = augmented.can_set_title;
    (*tui).set_cursor_color_as_str = augmented.set_cursor_color_as_str;
    (*tui).terminfo_ext = augmented.ext;
    (*tui).input.key_encoding = augmented.key_encoding;
    if augmented.extended_underline {
        tui_enable_extended_underline(tui);
    }
    let nsterm = quirks.nsterm;
    let (screen, tmux) = (quirks.screen, quirks.tmux);
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
    (*tui).immediate_wrap_after_last_column = quirks.wraps_after_last_column;
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
    out_cstr(tui, (*tui).terminfo_ext.disable_focus_reporting);
    out(tui, b"\x1B[c");
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
    out_cstr(tui, (*tui).terminfo_ext.enable_focus_reporting);
    flush_buf(tui);
}
pub unsafe fn tui_stop(mut tui: *mut TUIData) {
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
        Some(tui_stop_cb as unsafe extern "C" fn(*mut TUIData) -> ());
    terminfo_disable(tui);
    process_events_until((*tui).loop_0, (*(*tui).loop_0).events, 1000, || {
        (*tui).stopped || (*tui).input.read_stream.did_eof
    });
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
unsafe extern "C" fn tui_stop_cb(tui: *mut TUIData) {
    let mut tui: *mut TUIData = tui.cast::<TUIData>();
    (*tui).stopped = true_0 != 0;
}
unsafe extern "C" fn tui_terminal_stop(mut tui: *mut TUIData) {
    tinput_stop(&raw mut (*tui).input);
    terminfo_stop(tui);
}
pub unsafe fn tui_is_stopped(mut tui: *mut TUIData) -> bool {
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
            terminfo_print(tui, kTerm_set_attributes, &mut params);
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
        out_cstr(tui, (*tui).terminfo_ext.enter_altfont_mode);
    }
    if strikethrough {
        terminfo_out(tui, kTerm_enter_strikethrough_mode);
    }
    if conceal {
        terminfo_out(tui, kTerm_enter_secure_mode);
    }
    if overline {
        out(tui, b"\x1B[53m");
    }
    if !(*tui).ti.defs[kTerm_set_underline_style as ::core::ffi::c_int as usize].is_null() {
        if undercurl {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                [
                    3 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ],
            );
        }
        if underdouble {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                [
                    2 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ],
            );
        }
        if underdotted {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                [
                    4 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ],
            );
        }
        if underdashed {
            terminfo_print_num(
                tui,
                kTerm_set_underline_style,
                [
                    5 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ],
            );
        }
    }
    if has_any_underline as ::core::ffi::c_int != 0
        && (*tui).can_set_underline_color as ::core::ffi::c_int != 0
    {
        let mut color: ::core::ffi::c_int = attrs.rgb_sp_color as ::core::ffi::c_int;
        if color != -1 as ::core::ffi::c_int {
            out_fmt(
                tui,
                format_args!(
                    "\x1b[58:2::{}:{}:{}m",
                    color >> 16 & 0xff,
                    color >> 8 & 0xff,
                    color & 0xff
                ),
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
                [
                    fg >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                    fg >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                    fg & 0xff as ::core::ffi::c_int,
                ],
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
                [fg, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
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
                [
                    bg >> 16 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                    bg >> 8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int,
                    bg & 0xff as ::core::ffi::c_int,
                ],
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
                [bg, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
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
            out_raw(tui, (*tui).urlbuf.items, (*tui).urlbuf.size);
        } else {
            out(tui, b"\x1B]8;;\x1B\\");
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
    out_raw(tui, buf, strlen(buf));
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
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < next {
        let cell: UCell = (*tui).grid.cell(row, col + i);
        if attrs_differ(
            tui,
            cell.attr as ::core::ffi::c_int,
            (*tui).print_attr_id,
            (*tui).rgb,
        ) {
            if (*tui).default_attr {
                return false_0 != 0;
            }
        }
        if schar_get_ascii(cell.data) as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        i += 1;
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
        out(tui, b"\x1B]8;;\x1B\\");
        (*tui).url = -1 as ::core::ffi::c_int;
        (*tui).print_attr_id = -1 as ::core::ffi::c_int;
    }
    if 0 as ::core::ffi::c_int == row && 0 as ::core::ffi::c_int == col {
        terminfo_out(tui, kTerm_cursor_home);
        (*grid).goto(row, col);
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
            (*grid).goto((*grid).row, 0 as ::core::ffi::c_int);
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
                        [n, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
                    );
                }
                (*grid).goto(row, col);
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
                        [n_0, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
                    );
                }
                (*grid).goto(row, col);
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
                        [n_1, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
                    );
                }
                (*grid).goto(row, col);
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
                        [n_2, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
                    );
                }
                (*grid).goto(row, col);
                return;
            }
        }
    }
    terminfo_print_num(
        tui,
        kTerm_cursor_address,
        [row, col, 0 as ::core::ffi::c_int],
    );
    (*grid).goto(row, col);
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
unsafe fn print_cell_at_pos(
    mut tui: *mut TUIData,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    cell: UCell,
    mut is_doublewidth: bool,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if (*grid).row == -1 as ::core::ffi::c_int && cell.data == NUL as schar_T {
        return;
    }
    cursor_goto(tui, row, col);
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    schar_get(&raw mut buf as *mut ::core::ffi::c_char, cell.data);
    let mut c: ::core::ffi::c_int = utf_ptr2char(&raw mut buf as *mut ::core::ffi::c_char);
    let mut is_ambiwidth: bool = utf_ambiguous_width(&raw mut buf as *mut ::core::ffi::c_char);
    if is_doublewidth as ::core::ffi::c_int != 0
        && (is_ambiwidth as ::core::ffi::c_int != 0 || utf_char2cells(c) == 1 as ::core::ffi::c_int)
    {
        is_ambiwidth = true_0 != 0;
        update_attrs(tui, cell.attr as ::core::ffi::c_int);
        print_spaces(tui, 2 as ::core::ffi::c_int);
        cursor_goto(tui, row, col);
    }
    print_cell(tui, &raw mut buf as *mut ::core::ffi::c_char, cell.attr);
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
            (*grid).goto(top, left);
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
                    [width, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int],
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
        [top, bot, 0 as ::core::ffi::c_int],
    );
    if left != 0 as ::core::ffi::c_int || right != (*tui).width - 1 as ::core::ffi::c_int {
        tui_set_term_mode(tui, kTermModeLeftAndRightMargins, true_0 != 0);
        terminfo_print_num(
            tui,
            kTerm_set_lr_margin,
            [left, right, 0 as ::core::ffi::c_int],
        );
    }
    (*grid).row = -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn reset_scroll_region(mut tui: *mut TUIData, mut fullwidth: bool) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    if (*tui).terminfo_ext.reset_scroll_region.is_some() {
        out_cstr(tui, (*tui).terminfo_ext.reset_scroll_region);
    } else {
        terminfo_print_num(
            tui,
            kTerm_change_scroll_region,
            [
                0 as ::core::ffi::c_int,
                (*tui).height - 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ],
        );
    }
    if !fullwidth {
        terminfo_print_num(
            tui,
            kTerm_set_lr_margin,
            [
                0 as ::core::ffi::c_int,
                (*tui).width - 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ],
        );
        tui_set_term_mode(tui, kTermModeLeftAndRightMargins, false_0 != 0);
    }
    (*grid).row = -1 as ::core::ffi::c_int;
}
pub unsafe fn tui_grid_resize(
    mut tui: *mut TUIData,
    mut _g: Integer,
    mut width: Integer,
    mut height: Integer,
) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    (*grid).resize(width as ::core::ffi::c_int, height as ::core::ffi::c_int);
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
        out_fmt(tui, format_args!("\x1b[8;{height};{width}t"));
    } else {
        (*tui).pending_resize_events = if (*tui).pending_resize_events > 0 as ::core::ffi::c_int {
            (*tui).pending_resize_events - 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        (*grid).row = -1 as ::core::ffi::c_int;
    };
}
pub unsafe fn tui_grid_clear(mut tui: *mut TUIData, mut _g: Integer) {
    let mut grid: *mut UGrid = &raw mut (*tui).grid;
    (*grid).clear();
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
pub unsafe fn tui_grid_cursor_goto(
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
pub unsafe fn tui_mode_info_set(
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
pub unsafe fn tui_update_menu(mut _tui: *mut TUIData) {}
pub unsafe fn tui_busy_start(mut tui: *mut TUIData) {
    (*tui).busy = true_0 != 0;
}
pub unsafe fn tui_busy_stop(mut tui: *mut TUIData) {
    (*tui).busy = false_0 != 0;
}
pub unsafe fn tui_mouse_on(mut tui: *mut TUIData) {
    if !(*tui).mouse_enabled {
        tui_set_term_mode(tui, kTermModeMouseButtonEvent, true_0 != 0);
        tui_set_term_mode(tui, kTermModeMouseSGRExt, true_0 != 0);
        if (*tui).mouse_move_enabled {
            tui_set_term_mode(tui, kTermModeMouseAnyEvent, true_0 != 0);
        }
        (*tui).mouse_enabled = true_0 != 0;
    }
}
pub unsafe fn tui_mouse_off(mut tui: *mut TUIData) {
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
            terminfo_print(tui, kTerm_set_cursor_color, &mut params);
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
        [
            shape
                + (c.blinkon == 0 as ::core::ffi::c_int || c.blinkoff == 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        ],
    );
}
pub unsafe fn tui_mode_change(mut tui: *mut TUIData, mut _mode: String_0, mut mode_idx: Integer) {
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
pub unsafe fn tui_grid_scroll(
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
    (*grid).scroll(top, bot, left, right, rows as ::core::ffi::c_int);
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
                    [
                        rows as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    ],
                );
            }
        } else if rows == -1 as Integer {
            terminfo_out(tui, kTerm_insert_line);
        } else {
            terminfo_print_num(
                tui,
                kTerm_parm_insert_line,
                [
                    -(rows as ::core::ffi::c_int),
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ],
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
pub unsafe fn tui_add_url(mut _tui: *mut TUIData, mut url: *const ::core::ffi::c_char) -> int32_t {
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
pub unsafe fn tui_hl_attr_define(
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
pub unsafe fn tui_bell(mut tui: *mut TUIData) {
    out(tui, b"\x07");
}
pub unsafe fn tui_visual_bell(mut tui: *mut TUIData) {
    if (*tui).screen_or_tmux {
        out(tui, b"\x1Bg");
    } else {
        out(tui, b"\x1B[?5h");
        flush_buf(tui);
        uv_sleep(100 as ::core::ffi::c_uint);
        out(tui, b"\x1B[?5l");
    }
    flush_buf(tui);
}
pub unsafe fn tui_default_colors_set(
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
pub unsafe fn tui_ui_send(mut tui: *mut TUIData, mut content: String_0) {
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
pub unsafe fn tui_flush(mut tui: *mut TUIData) {
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
            let mut clear_attr: ::core::ffi::c_int =
                (*grid).cell(row, r.right - 1 as ::core::ffi::c_int).attr as ::core::ffi::c_int;
            let mut clear_col: ::core::ffi::c_int = 0;
            clear_col = r.right;
            while clear_col > 0 as ::core::ffi::c_int {
                let cell: UCell = (*grid).cell(row, clear_col - 1 as ::core::ffi::c_int);
                if !(cell.data == ' ' as ::core::ffi::c_int as schar_T
                    && cell.attr == clear_attr as sattr_T)
                {
                    break;
                }
                clear_col -= 1;
            }
            let mut curcol: ::core::ffi::c_int = r.left;
            while curcol < clear_col {
                let cell_0: UCell = (*grid).cell(row, curcol);
                print_cell_at_pos(
                    tui,
                    row,
                    curcol,
                    cell_0,
                    curcol < clear_col - 1 as ::core::ffi::c_int
                        && (*grid).cell(row, curcol + 1 as ::core::ffi::c_int).data
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
    let mut str: String_0 = terminfo_info_msg(&(*tui).ti, (*tui).term, (*tui).terminfo_found_in_db);
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
pub unsafe fn tui_suspend(mut tui: *mut TUIData) {
    ui_client_detach();
    (*tui).mouse_enabled_save = (*tui).mouse_enabled;
    (*tui).input.callbacks.primary_device_attr =
        Some(tui_suspend_cb as unsafe extern "C" fn(*mut TUIData) -> ());
    terminfo_disable(tui);
}
unsafe extern "C" fn tui_suspend_cb(tui: *mut TUIData) {
    let mut tui: *mut TUIData = tui.cast::<TUIData>();
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
pub unsafe fn tui_set_title(mut tui: *mut TUIData, mut title: String_0) {
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
            out(tui, b"\x1B[22;0t");
            (*tui).title_enabled = true_0 != 0;
        }
        if ::core::mem::size_of::<[::core::ffi::c_char; 65535]>()
            .wrapping_sub((*tui).bufpos as usize)
            < title.size.wrapping_add(2 * TERMINFO_SEQ_LIMIT)
        {
            flush_buf(tui);
        }
        terminfo_out(tui, kTerm_to_status_line);
        out_raw(tui, title.data, title.size);
        terminfo_out(tui, kTerm_from_status_line);
    } else if (*tui).title_enabled {
        out(tui, b"\x1B[23;0t");
        (*tui).title_enabled = false_0 != 0;
    }
}
pub unsafe fn tui_set_icon(mut _tui: *mut TUIData, mut _icon: String_0) {}
pub unsafe fn tui_screenshot(mut tui: *mut TUIData, mut path: String_0) {
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
            let cell: UCell = (*grid).cell(i, j);
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
pub unsafe fn tui_option_set(mut tui: *mut TUIData, mut name: String_0, mut value: Object) {
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
pub unsafe fn tui_chdir(mut _tui: *mut TUIData, mut path: String_0) {
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
pub unsafe fn tui_raw_line(
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
        let mut cell: UCell = (*grid).cell(linerow as ::core::ffi::c_int, c as ::core::ffi::c_int);
        cell.data = *chunk.offset((c - startcol) as isize);
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
        cell.attr = *attrs.offset((c - startcol) as isize);
        (*grid).set_cell(linerow as ::core::ffi::c_int, c as ::core::ffi::c_int, cell);
        c += 1;
    }
    let mut curcol: ::core::ffi::c_int = startcol as ::core::ffi::c_int;
    while curcol < endcol as ::core::ffi::c_int {
        let cell_0: UCell = (*grid).cell(linerow as ::core::ffi::c_int, curcol);
        print_cell_at_pos(
            tui,
            linerow as ::core::ffi::c_int,
            curcol,
            cell_0,
            (curcol as Integer) < endcol - 1 as Integer
                && (*grid)
                    .cell(
                        linerow as ::core::ffi::c_int,
                        curcol + 1 as ::core::ffi::c_int,
                    )
                    .data
                    == '\0' as schar_T,
        );
        curcol += 1;
    }
    if clearcol > endcol {
        (*grid).clear_chunk(
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
            let mut size: ::core::ffi::c_int = if (*grid)
                .cell(
                    linerow as ::core::ffi::c_int,
                    (*grid).width - 1 as ::core::ffi::c_int,
                )
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
                (*grid).cell(linerow as ::core::ffi::c_int, (*grid).width - size),
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
pub unsafe fn tui_set_size(
    mut tui: *mut TUIData,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    (*tui).pending_resize_events += 1;
    (*tui).width = width;
    (*tui).height = height;
    ui_client_set_size(width, height);
}
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
                != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_char>(
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
