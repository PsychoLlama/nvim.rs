use crate::src::nvim::api::private::dispatch::KeyDict_highlight_get_field;
use crate::src::nvim::api::private::helpers::{
    api_dict_to_keydict, api_free_array, api_metadata, api_set_error, copy_array, cstr_as_string,
};
use crate::src::nvim::channel::{channel_connect, channel_job_start};
use crate::src::nvim::event::multiqueue::{
    multiqueue_empty, multiqueue_process_events, multiqueue_put_event,
};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::socket::socket_address_tcp_host_end;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight::dict2hlattrs;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    grid_line_buf_attr, grid_line_buf_char, grid_line_buf_size, main_loop, os_exit, stderr_isatty,
    stdin_isatty, stdout_isatty, t_colors, time_fd, ui_client_attached, ui_client_channel_id,
    ui_client_error_exit, ui_client_exit_status, ui_client_forward_stdin,
};
use crate::src::nvim::memory::{strequal, xfree, xmalloc, xmemdupz, xstrdup};
use crate::src::nvim::msgpack_rpc::channel::rpc_send_event;
use crate::src::nvim::os::env::{os_env_exists, os_get_pid};
use crate::src::nvim::os::libc::{__assert_fail, abort, close, dup, memcmp};
use crate::src::nvim::profile::{time_finish, time_msg};
pub use crate::src::nvim::types::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __gid_t, __off64_t, __off_t,
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t, __uid_t,
    argv_callback, blob_T, blobvar_S, consumed_blk, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed_1, funccall_T, garray_T, gid_t, hash_T, hashitem_T,
    hashtab_T, int16_t, int32_t, int64_t, internal_proc_cb, key_value_pair, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, loop_0,
    loop_0_children as C2Rust_Unnamed_10, multiqueue, object, object_data as C2Rust_Unnamed,
    packer_buffer_t, partial_S, partial_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, queue, rstream, sattr_T, schar_T, scid_T, sctx_T, size_t,
    ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_12,
    stream_write_cb, terminal, typval_T, typval_vval_union, ufunc_S, ufunc_T, uid_t, uint16_t,
    uint32_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_7, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_2, uv_handle_t, uv_handle_type, uv_idle_cb,
    uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_13, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_6, uv_loop_s_timer_heap as C2Rust_Unnamed_5, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_15, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_16, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_3,
    uv_signal_s_u as C2Rust_Unnamed_4, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_17, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_11, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_14, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, uv_uid_t,
    varnumber_T, winsize, Arena, ArenaMem, Array, BoolVarValue, Boolean, Callback, CallbackReader,
    CallbackType, Callback_data as C2Rust_Unnamed_0, Channel, ChannelCallFrame, ChannelStdinMode,
    ChannelStreamType, Channel_stream as C2Rust_Unnamed_20, ClientType, Dict, Error, ErrorType,
    Event, FieldHashfn, Float, GridLineEvent, HLGroupID, HlAttrs, Integer, InternalState,
    KeyDict_highlight, KeySetLink, KeyValuePair, LibuvProc, LineFlags, Loop, LuaRef, MultiQueue,
    Object, ObjectType, OptionalKeys, PackerBuffer, PackerBufferFlush, Proc, ProcType, PtyProc,
    RStream, RemoteUI, RgbValue, RpcState, RpcState_call_stack as C2Rust_Unnamed_19,
    ScopeDictDictItem, ScopeType, SpecialVarValue, StderrState, StdioPair, Stream, String_0,
    TUIData, Terminal, UIClientHandler, Unpacker, VarLockStatus, VarType, _IO_FILE, FILE, QUEUE,
};
extern "C" {
    fn tui_start(
        tui_p: *mut *mut TUIData,
        width: *mut ::core::ffi::c_int,
        height: *mut ::core::ffi::c_int,
        term: *mut *mut ::core::ffi::c_char,
        rgb: *mut bool,
    );
    fn tui_stop(tui_0: *mut TUIData);
    fn tui_is_stopped(tui_0: *mut TUIData) -> bool;
    fn tui_grid_resize(tui_0: *mut TUIData, g: Integer, width: Integer, height: Integer);
    fn tui_grid_clear(tui_0: *mut TUIData, g: Integer);
    fn tui_grid_cursor_goto(tui_0: *mut TUIData, grid: Integer, row: Integer, col: Integer);
    fn tui_mode_info_set(tui_0: *mut TUIData, guicursor_enabled: bool, args: Array);
    fn tui_update_menu(tui_0: *mut TUIData);
    fn tui_busy_start(tui_0: *mut TUIData);
    fn tui_busy_stop(tui_0: *mut TUIData);
    fn tui_mouse_on(tui_0: *mut TUIData);
    fn tui_mouse_off(tui_0: *mut TUIData);
    fn tui_mode_change(tui_0: *mut TUIData, mode: String_0, mode_idx: Integer);
    fn tui_grid_scroll(
        tui_0: *mut TUIData,
        g: Integer,
        startrow: Integer,
        endrow: Integer,
        startcol: Integer,
        endcol: Integer,
        rows: Integer,
        cols: Integer,
    );
    fn tui_add_url(tui_0: *mut TUIData, url: *const ::core::ffi::c_char) -> int32_t;
    fn tui_hl_attr_define(
        tui_0: *mut TUIData,
        id: Integer,
        attrs: HlAttrs,
        cterm_attrs: HlAttrs,
        info: Array,
    );
    fn tui_bell(tui_0: *mut TUIData);
    fn tui_visual_bell(tui_0: *mut TUIData);
    fn tui_default_colors_set(
        tui_0: *mut TUIData,
        rgb_fg: Integer,
        rgb_bg: Integer,
        rgb_sp: Integer,
        cterm_fg: Integer,
        cterm_bg: Integer,
    );
    fn tui_ui_send(tui_0: *mut TUIData, content: String_0);
    fn tui_flush(tui_0: *mut TUIData);
    fn tui_suspend(tui_0: *mut TUIData);
    fn tui_set_title(tui_0: *mut TUIData, title: String_0);
    fn tui_set_icon(tui_0: *mut TUIData, icon: String_0);
    fn tui_screenshot(tui_0: *mut TUIData, path: String_0);
    fn tui_option_set(tui_0: *mut TUIData, name: String_0, value: Object);
    fn tui_chdir(tui_0: *mut TUIData, path: String_0);
    fn tui_raw_line(
        tui_0: *mut TUIData,
        g: Integer,
        linerow: Integer,
        startcol: Integer,
        endcol: Integer,
        clearcol: Integer,
        clearattr: Integer,
        flags: LineFlags,
        chunk: *const schar_T,
        attrs: *const sattr_T,
    );
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
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
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
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
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelStdinNull: ChannelStdinMode = 1;
pub const kChannelStdinPipe: ChannelStdinMode = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_18 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_18 = 1;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"void ui_client_attach(int, int, char *, _Bool)\0",
    )
};
pub const UINT64_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const HLATTRS_INIT: HlAttrs = HlAttrs {
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
pub const KEYDICT_INIT: KeyDict_highlight = KeyDict_highlight {
    is_set__highlight_: 0 as OptionalKeys,
    altfont: false,
    blink: false,
    bold: false,
    conceal: false,
    dim: false,
    italic: false,
    nocombine: false,
    overline: false,
    reverse: false,
    standout: false,
    strikethrough: false,
    undercurl: false,
    underdashed: false,
    underdotted: false,
    underdouble: false,
    underline: false,
    default_: false,
    cterm: Dict {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    },
    foreground: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    fg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    background: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    bg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    ctermfg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    ctermbg: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    special: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    sp: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    link: 0,
    link_global: 0,
    fallback: false,
    blend: 0,
    fg_indexed: false,
    bg_indexed: false,
    force: false,
    update: false,
    url: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
};
pub const KEYSET_OPTIDX_highlight__url: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
static tui: GlobalCell<*mut TUIData> = GlobalCell::new(::core::ptr::null_mut::<TUIData>());
static tui_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static tui_height: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static tui_term: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
static tui_rgb: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn ui_client_start_server(
    mut exepath: *const ::core::ffi::c_char,
    mut argc: size_t,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> uint64_t {
    let mut args: *mut *mut ::core::ffi::c_char = xmalloc(
        (2 as size_t)
            .wrapping_add(argc)
            .wrapping_mul(::core::mem::size_of::<*mut ::core::ffi::c_char>()),
    ) as *mut *mut ::core::ffi::c_char;
    let mut args_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let c2rust_fresh0 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr = &raw mut *args.offset(c2rust_fresh0 as isize);
    *c2rust_lvalue_ptr = xstrdup(*argv.offset(0 as ::core::ffi::c_int as isize));
    let c2rust_fresh1 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr_0 = &raw mut *args.offset(c2rust_fresh1 as isize);
    *c2rust_lvalue_ptr_0 = xstrdup(b"--embed\0".as_ptr() as *const ::core::ffi::c_char);
    let mut i: size_t = 1 as size_t;
    while i < argc {
        let c2rust_fresh2 = args_idx;
        args_idx = args_idx + 1;
        let c2rust_lvalue_ptr_1 = &raw mut *args.offset(c2rust_fresh2 as isize);
        *c2rust_lvalue_ptr_1 = xstrdup(*argv.offset(i as isize));
        i = i.wrapping_add(1);
    }
    let c2rust_fresh3 = args_idx;
    args_idx = args_idx + 1;
    let c2rust_lvalue_ptr_2 = &raw mut *args.offset(c2rust_fresh3 as isize);
    *c2rust_lvalue_ptr_2 = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut on_err: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_0 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    on_err.fwd_err = true_0 != 0;
    let mut detach: bool = true_0 != 0;
    let mut exit_status: varnumber_T = 0;
    let mut channel: *mut Channel = channel_job_start(
        args,
        exepath,
        CallbackReader {
            cb: Callback {
                data: C2Rust_Unnamed_0 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            },
            self_0: ::core::ptr::null_mut::<dict_T>(),
            buffer: GA_EMPTY_INIT_VALUE,
            eof: false,
            buffered: false_0 != 0,
            fwd_err: false_0 != 0,
            type_0: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        on_err,
        Callback {
            data: C2Rust_Unnamed_0 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        false_0 != 0,
        true_0 != 0,
        true_0 != 0,
        detach,
        kChannelStdinPipe,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as uint16_t,
        0 as uint16_t,
        ::core::ptr::null_mut::<dict_T>(),
        &raw mut exit_status,
    );
    if channel.is_null() {
        return 0 as uint64_t;
    }
    if ui_client_forward_stdin.get() {
        close(0 as ::core::ffi::c_int);
        dup(if stderr_isatty.get() as ::core::ffi::c_int != 0 {
            STDERR_FILENO
        } else {
            STDOUT_FILENO
        });
    }
    return (*channel).id;
}
pub unsafe extern "C" fn ui_client_attach(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
    mut term: *mut ::core::ffi::c_char,
    mut rgb: bool,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 3] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 3];
    args.capacity = 3 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh4 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh4 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: width as Integer,
        },
    };
    let c2rust_fresh5 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh5 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: height as Integer,
        },
    };
    let mut opts: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut opts__items: [KeyValuePair; 9] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 9];
    opts.capacity = 9 as size_t;
    opts.items = &raw mut opts__items as *mut KeyValuePair;
    let c2rust_fresh6 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh6 as isize) = key_value_pair {
        key: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: rgb },
        },
    };
    let c2rust_fresh7 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh7 as isize) = key_value_pair {
        key: cstr_as_string(b"ext_linegrid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: true },
        },
    };
    let c2rust_fresh8 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh8 as isize) = key_value_pair {
        key: cstr_as_string(b"ext_termcolors\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed { boolean: true },
        },
    };
    if !term.is_null() {
        let c2rust_fresh9 = opts.size;
        opts.size = opts.size.wrapping_add(1);
        *opts.items.offset(c2rust_fresh9 as isize) = key_value_pair {
            key: cstr_as_string(b"term_name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(term),
                },
            },
        };
    }
    let c2rust_fresh10 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh10 as isize) = key_value_pair {
        key: cstr_as_string(b"term_colors\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: t_colors.get() as Integer,
            },
        },
    };
    let c2rust_fresh11 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: stdin_isatty.get(),
            },
        },
    };
    let c2rust_fresh12 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh12 as isize) = key_value_pair {
        key: cstr_as_string(b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: stdout_isatty.get(),
            },
        },
    };
    if ui_client_forward_stdin.get() {
        let c2rust_fresh13 = opts.size;
        opts.size = opts.size.wrapping_add(1);
        *opts.items.offset(c2rust_fresh13 as isize) = key_value_pair {
            key: cstr_as_string(b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: 3 as Integer,
                },
            },
        };
        ui_client_forward_stdin.set(false_0 != 0);
    }
    let c2rust_fresh14 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh14 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: opts },
    };
    rpc_send_event(
        ui_client_channel_id.get(),
        b"nvim_ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
    ui_client_attached.set(true_0 != 0);
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"nvim_ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
    let mut args2: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args2__items: [Object; 5] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 5];
    args2.capacity = 5 as size_t;
    args2.items = &raw mut args2__items as *mut Object;
    let c2rust_fresh15 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh15 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(b"nvim-tui\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let mut m: Object = api_metadata();
    let mut version: Dict = Dict {
        size: 0 as size_t,
        capacity: 0,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    '_c2rust_label: {
        if m.data.dict.size > 0 as size_t {
        } else {
            __assert_fail(
                b"m.data.dict.size > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/ui_client.rs\0".as_ptr() as *const ::core::ffi::c_char,
                123 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    let mut i: size_t = 0 as size_t;
    while i < m.data.dict.size {
        if strequal(
            (*m.data.dict.items.offset(i as isize)).key.data,
            b"version\0".as_ptr() as *const ::core::ffi::c_char,
        ) {
            version = (*m.data.dict.items.offset(i as isize)).value.data.dict;
            break;
        } else {
            if i.wrapping_add(1 as size_t) == m.data.dict.size {
                abort();
            }
            i = i.wrapping_add(1);
        }
    }
    let c2rust_fresh16 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh16 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: version },
    };
    let c2rust_fresh17 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh17 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(b"ui\0".as_ptr() as *const ::core::ffi::c_char),
        },
    };
    let c2rust_fresh18 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh18 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        },
    };
    let mut info: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut info__items: [KeyValuePair; 9] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 9];
    info.capacity = 9 as size_t;
    info.items = &raw mut info__items as *mut KeyValuePair;
    let c2rust_fresh19 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh19 as isize) = key_value_pair {
        key: cstr_as_string(b"website\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(
                    b"https://neovim.io\0".as_ptr() as *const ::core::ffi::c_char
                ),
            },
        },
    };
    let c2rust_fresh20 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh20 as isize) = key_value_pair {
        key: cstr_as_string(b"license\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_as_string(b"Apache 2\0".as_ptr() as *const ::core::ffi::c_char),
            },
        },
    };
    let c2rust_fresh21 = info.size;
    info.size = info.size.wrapping_add(1);
    *info.items.offset(c2rust_fresh21 as isize) = key_value_pair {
        key: cstr_as_string(b"pid\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: os_get_pid(),
            },
        },
    };
    let c2rust_fresh22 = args2.size;
    args2.size = args2.size.wrapping_add(1);
    *args2.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: info },
    };
    rpc_send_event(
        ui_client_channel_id.get(),
        b"nvim_set_client_info\0".as_ptr() as *const ::core::ffi::c_char,
        args2,
    );
    if !(*time_fd.ptr()).is_null() {
        time_msg(
            b"nvim_set_client_info\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<proftime_T>(),
        );
    }
}
pub unsafe extern "C" fn ui_client_detach() {
    rpc_send_event(
        ui_client_channel_id.get(),
        b"nvim_ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
        Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        },
    );
    ui_client_attached.set(false_0 != 0);
}
pub unsafe extern "C" fn ui_client_run() -> ! {
    tui_start(
        tui.ptr(),
        tui_width.ptr(),
        tui_height.ptr(),
        tui_term.ptr(),
        tui_rgb.ptr(),
    );
    ui_client_attach(
        tui_width.get(),
        tui_height.get(),
        tui_term.get(),
        tui_rgb.get(),
    );
    if os_env_exists(
        b"__NVIM_TEST_LOG\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
    ) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_run\0".as_ptr() as *const ::core::ffi::c_char,
            163 as ::core::ffi::c_int,
            true_0 != 0,
            b"test log message\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    time_finish();
    loop {
        if !(*main_loop.ptr()).events.is_null() && !multiqueue_empty((*main_loop.ptr()).events) {
            multiqueue_process_events((*main_loop.ptr()).events);
        } else {
            loop_poll_events(main_loop.ptr(), -1 as int64_t);
        }
    }
}
pub unsafe extern "C" fn ui_client_stop() {
    ui_client_attached.set(false_0 != 0);
    if !tui_is_stopped(tui.get()) {
        tui_stop(tui.get());
    }
}
pub unsafe extern "C" fn ui_client_set_size(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    if ui_client_attached.get() {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 2] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        args.capacity = 2 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh23 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh23 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: width as Integer,
            },
        };
        let c2rust_fresh24 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh24 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: height as Integer,
            },
        };
        rpc_send_event(
            ui_client_channel_id.get(),
            b"nvim_ui_try_resize\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    }
    tui_width.set(width);
    tui_height.set(height);
}
pub unsafe extern "C" fn ui_client_get_redraw_handler(
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
    mut _error: *mut Error,
) -> UIClientHandler {
    let mut hash: ::core::ffi::c_int = ui_client_handler_hash(name, name_len);
    if hash < 0 as ::core::ffi::c_int {
        return UIClientHandler {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            fn_0: None,
        };
    }
    return (*event_handlers.ptr())[hash as usize];
}
pub unsafe extern "C" fn handle_ui_client_redraw(
    mut _channel_id: uint64_t,
    mut _args: Array,
    mut _arena: *mut Arena,
    mut error: *mut Error,
) -> Object {
    api_set_error(
        error,
        kErrorTypeValidation,
        b"'redraw' cannot be sent as a request\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
}
unsafe extern "C" fn ui_client_dict2hlattrs(mut d: Dict, mut rgb: bool) -> HlAttrs {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut dict: KeyDict_highlight = KEYDICT_INIT;
    if !api_dict_to_keydict(
        &raw mut dict as *mut ::core::ffi::c_void,
        Some(
            KeyDict_highlight_get_field
                as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
        ),
        d,
        &raw mut err,
    ) {
        return HLATTRS_INIT;
    }
    let mut attrs: HlAttrs = dict2hlattrs(
        &raw mut dict,
        rgb,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<HlAttrs>(),
        &raw mut err,
    );
    if dict.is_set__highlight_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_highlight__url
        != 0 as ::core::ffi::c_ulonglong
    {
        attrs.url = tui_add_url(tui.get(), dict.url.data);
    }
    return attrs;
}
pub unsafe extern "C" fn ui_client_event_grid_resize(mut args: Array) {
    if args.size < 3 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_resize\0".as_ptr() as *const ::core::ffi::c_char,
            241 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_resize'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut grid: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut width: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut height: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_resize(tui.get(), grid, width, height);
    if grid_line_buf_size.get() < width as size_t {
        xfree(grid_line_buf_char.get() as *mut ::core::ffi::c_void);
        xfree(grid_line_buf_attr.get() as *mut ::core::ffi::c_void);
        grid_line_buf_size.set(width as size_t);
        grid_line_buf_char.set(xmalloc(
            (*grid_line_buf_size.ptr()).wrapping_mul(::core::mem::size_of::<schar_T>()),
        ) as *mut schar_T);
        grid_line_buf_attr.set(xmalloc(
            (*grid_line_buf_size.ptr()).wrapping_mul(::core::mem::size_of::<sattr_T>()),
        ) as *mut sattr_T);
    }
}
pub unsafe extern "C" fn ui_client_event_grid_line(mut _args: Array) -> ! {
    abort();
}
pub unsafe extern "C" fn ui_client_event_raw_line(mut g: *mut GridLineEvent) {
    let mut grid: ::core::ffi::c_int = (*g).args[0 as ::core::ffi::c_int as usize];
    let mut row: ::core::ffi::c_int = (*g).args[1 as ::core::ffi::c_int as usize];
    let mut startcol: ::core::ffi::c_int = (*g).args[2 as ::core::ffi::c_int as usize];
    let mut endcol: Integer = (startcol + (*g).coloff) as Integer;
    let mut clearcol: Integer = endcol + (*g).clear_width as Integer;
    let mut lineflags: LineFlags = if (*g).wrap as ::core::ffi::c_int != 0 {
        kLineFlagWrap as ::core::ffi::c_int
    } else {
        0 as LineFlags
    };
    tui_raw_line(
        tui.get(),
        grid as Integer,
        row as Integer,
        startcol as Integer,
        endcol,
        clearcol,
        (*g).cur_attr as Integer,
        lineflags,
        grid_line_buf_char.get() as *const schar_T,
        grid_line_buf_attr.get(),
    );
}
pub unsafe extern "C" fn ui_client_event_connect(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_connect\0".as_ptr() as *const ::core::ffi::c_char,
            282 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling UI event 'connect'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut s: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut server_addr: *mut ::core::ffi::c_char =
        xmemdupz(s.data as *const ::core::ffi::c_void, s.size) as *mut ::core::ffi::c_char;
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        Event {
            handler: Some(
                channel_connect_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                server_addr as *mut ::core::ffi::c_void,
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
    ui_client_channel_id.set(UINT64_MAX as uint64_t);
}
unsafe extern "C" fn channel_connect_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut server_addr: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    let mut err: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
    let mut is_tcp: bool = !socket_address_tcp_host_end(server_addr).is_null();
    let mut on_data: CallbackReader = CallbackReader {
        cb: Callback {
            data: C2Rust_Unnamed_0 {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        self_0: ::core::ptr::null_mut::<dict_T>(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false_0 != 0,
        fwd_err: false_0 != 0,
        type_0: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut chan: uint64_t = channel_connect(
        is_tcp,
        server_addr,
        true_0 != 0,
        on_data,
        50 as ::core::ffi::c_int,
        &raw mut err,
    );
    if !strequal(err, b"\0".as_ptr() as *const ::core::ffi::c_char) {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"channel_connect_event\0".as_ptr() as *const ::core::ffi::c_char,
            303 as ::core::ffi::c_int,
            true_0 != 0,
            b"Cannot connect to server %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
            server_addr,
            err,
        );
        xfree(server_addr as *mut ::core::ffi::c_void);
        ui_client_exit_status.set(1 as ::core::ffi::c_int);
        os_exit(1 as ::core::ffi::c_int);
    }
    ui_client_channel_id.set(chan);
    ui_client_attach(
        tui_width.get(),
        tui_height.get(),
        tui_term.get(),
        tui_rgb.get(),
    );
    logmsg(
        LOGLVL_INF,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"channel_connect_event\0".as_ptr() as *const ::core::ffi::c_char,
        312 as ::core::ffi::c_int,
        true_0 != 0,
        b"Connected to server %s on channel %ld\0".as_ptr() as *const ::core::ffi::c_char,
        server_addr,
        chan,
    );
    xfree(server_addr as *mut ::core::ffi::c_void);
}
static restart_args: GlobalCell<Array> = GlobalCell::new(Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
});
static restart_pending: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub unsafe extern "C" fn ui_client_event_restart(mut args: Array) {
    api_free_array(restart_args.get());
    restart_args.set(copy_array(args, ::core::ptr::null_mut::<Arena>()));
    restart_pending.set(true_0 != 0);
}
pub unsafe extern "C" fn ui_client_attach_to_restarted_server() {
    let mut listen_addr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_tcp: bool = false;
    let mut err: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut chan_id: uint64_t = 0;
    if !restart_pending.get() {
        return;
    }
    restart_pending.set(false_0 != 0);
    if (*restart_args.ptr()).size < 1 as size_t
        || (*(*restart_args.ptr())
            .items
            .offset(0 as ::core::ffi::c_int as isize))
        .type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
            343 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'restart'\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        listen_addr = (*(*restart_args.ptr())
            .items
            .offset(0 as ::core::ffi::c_int as isize))
        .data
        .string
        .data;
        is_tcp = !socket_address_tcp_host_end(listen_addr).is_null();
        err = b"\0".as_ptr() as *const ::core::ffi::c_char;
        chan_id = channel_connect(
            is_tcp,
            listen_addr,
            true_0 != 0,
            CallbackReader {
                cb: Callback {
                    data: C2Rust_Unnamed_0 {
                        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    },
                    type_0: kCallbackNone,
                },
                self_0: ::core::ptr::null_mut::<dict_T>(),
                buffer: GA_EMPTY_INIT_VALUE,
                eof: false,
                buffered: false_0 != 0,
                fwd_err: false_0 != 0,
                type_0: ::core::ptr::null::<::core::ffi::c_char>(),
            },
            50 as ::core::ffi::c_int,
            &raw mut err,
        );
        if !strequal(err, b"\0".as_ptr() as *const ::core::ffi::c_char) {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
                353 as ::core::ffi::c_int,
                true_0 != 0,
                b"cannot connect to server %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                listen_addr,
                err,
            );
        } else {
            ui_client_channel_id.set(chan_id);
            ui_client_attach(
                tui_width.get(),
                tui_height.get(),
                tui_term.get(),
                tui_rgb.get(),
            );
            logmsg(
                LOGLVL_INF,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"ui_client_attach_to_restarted_server\0".as_ptr() as *const ::core::ffi::c_char,
                361 as ::core::ffi::c_int,
                true_0 != 0,
                b"restarted server address=%s id=%ld\0".as_ptr() as *const ::core::ffi::c_char,
                listen_addr,
                chan_id,
            );
        }
    }
    api_free_array(restart_args.get());
    restart_args.set(Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    });
}
pub unsafe extern "C" fn ui_client_event_error_exit(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            372 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'error_exit'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    ui_client_error_exit.set(
        (*args.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn ui_client_event_mode_info_set(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
            6 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'mode_info_set'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Boolean = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .boolean;
    let mut arg_2: Array = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .array;
    tui_mode_info_set(tui.get(), arg_1 as bool, arg_2);
}
pub unsafe extern "C" fn ui_client_event_update_menu(mut _args: Array) {
    tui_update_menu(tui.get());
}
pub unsafe extern "C" fn ui_client_event_busy_start(mut _args: Array) {
    tui_busy_start(tui.get());
}
pub unsafe extern "C" fn ui_client_event_busy_stop(mut _args: Array) {
    tui_busy_stop(tui.get());
}
pub unsafe extern "C" fn ui_client_event_mouse_on(mut _args: Array) {
    tui_mouse_on(tui.get());
}
pub unsafe extern "C" fn ui_client_event_mouse_off(mut _args: Array) {
    tui_mouse_off(tui.get());
}
pub unsafe extern "C" fn ui_client_event_mode_change(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_mode_change\0".as_ptr() as *const ::core::ffi::c_char,
            44 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'mode_change'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_mode_change(tui.get(), arg_1, arg_2);
}
pub unsafe extern "C" fn ui_client_event_bell(mut _args: Array) {
    tui_bell(tui.get());
}
pub unsafe extern "C" fn ui_client_event_visual_bell(mut _args: Array) {
    tui_visual_bell(tui.get());
}
pub unsafe extern "C" fn ui_client_event_flush(mut _args: Array) {
    tui_flush(tui.get());
}
pub unsafe extern "C" fn ui_client_event_suspend(mut _args: Array) {
    tui_suspend(tui.get());
}
pub unsafe extern "C" fn ui_client_event_set_title(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_set_title\0".as_ptr() as *const ::core::ffi::c_char,
            76 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'set_title'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_set_title(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_set_icon(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_set_icon\0".as_ptr() as *const ::core::ffi::c_char,
            87 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'set_icon'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_set_icon(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_screenshot(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_screenshot\0".as_ptr() as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'screenshot'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_screenshot(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_option_set(mut args: Array) {
    if args.size < 2 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_option_set\0".as_ptr() as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'option_set'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    let mut arg_2: Object = *args.items.offset(1 as ::core::ffi::c_int as isize);
    tui_option_set(tui.get(), arg_1, arg_2);
}
pub unsafe extern "C" fn ui_client_event_chdir(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_chdir\0".as_ptr() as *const ::core::ffi::c_char,
            121 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'chdir'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_chdir(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_ui_send(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            132 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'ui_send'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: String_0 = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .string;
    tui_ui_send(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_default_colors_set(mut args: Array) {
    if args.size < 5 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(4 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
            147 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'default_colors_set'\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_4: Integer = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_5: Integer = (*args.items.offset(4 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_default_colors_set(tui.get(), arg_1, arg_2, arg_3, arg_4, arg_5);
}
pub unsafe extern "C" fn ui_client_event_hl_attr_define(mut args: Array) {
    if args.size < 4 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
            165 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'hl_attr_define'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: HlAttrs = ui_client_dict2hlattrs(
        (*args.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .dict,
        true_0 != 0,
    );
    let mut arg_3: HlAttrs = ui_client_dict2hlattrs(
        (*args.items.offset(2 as ::core::ffi::c_int as isize))
            .data
            .dict,
        false_0 != 0,
    );
    let mut arg_4: Array = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .array;
    tui_hl_attr_define(tui.get(), arg_1, arg_2, arg_3, arg_4);
}
pub unsafe extern "C" fn ui_client_event_grid_clear(mut args: Array) {
    if args.size < 1 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_clear\0".as_ptr() as *const ::core::ffi::c_char,
            179 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_clear'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_clear(tui.get(), arg_1);
}
pub unsafe extern "C" fn ui_client_event_grid_cursor_goto(mut args: Array) {
    if args.size < 3 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            192 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_cursor_goto'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_cursor_goto(tui.get(), arg_1, arg_2, arg_3);
}
pub unsafe extern "C" fn ui_client_event_grid_scroll(mut args: Array) {
    if args.size < 7 as size_t
        || (*args.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(2 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(3 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(4 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(5 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*args.items.offset(6 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"ui_client_event_grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            211 as ::core::ffi::c_int,
            true_0 != 0,
            b"Error handling ui event 'grid_scroll'\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut arg_1: Integer = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_2: Integer = (*args.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_3: Integer = (*args.items.offset(2 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_4: Integer = (*args.items.offset(3 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_5: Integer = (*args.items.offset(4 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_6: Integer = (*args.items.offset(5 as ::core::ffi::c_int as isize))
        .data
        .integer;
    let mut arg_7: Integer = (*args.items.offset(6 as ::core::ffi::c_int as isize))
        .data
        .integer;
    tui_grid_scroll(tui.get(), arg_1, arg_2, arg_3, arg_4, arg_5, arg_6, arg_7);
}
static event_handlers: GlobalCell<[UIClientHandler; 27]> = GlobalCell::new(unsafe {
    [
        UIClientHandler {
            name: b"bell\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_bell as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"chdir\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_chdir as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"flush\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_flush as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"connect\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_connect as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"restart\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_restart as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_suspend as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"ui_send\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_ui_send as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mouse_on\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mouse_on as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"set_icon\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_set_icon as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"busy_stop\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_busy_stop as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_line\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: ::core::mem::transmute::<
                Option<unsafe extern "C" fn(Array) -> !>,
                Option<unsafe extern "C" fn(Array) -> ()>,
            >(Some(
                ui_client_event_grid_line as unsafe extern "C" fn(Array) -> !,
            )),
        },
        UIClientHandler {
            name: b"mouse_off\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mouse_off as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"set_title\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_set_title as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"busy_start\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_busy_start as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_error_exit as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_clear\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_clear as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"option_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_option_set as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"screenshot\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_screenshot as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mode_change\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mode_change as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"update_menu\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_update_menu as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"visual_bell\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_visual_bell as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_resize as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_scroll as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_mode_info_set as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_hl_attr_define as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_grid_cursor_goto as unsafe extern "C" fn(Array) -> ()),
        },
        UIClientHandler {
            name: b"default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
            fn_0: Some(ui_client_event_default_colors_set as unsafe extern "C" fn(Array) -> ()),
        },
    ]
});
pub unsafe extern "C" fn ui_client_handler_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
        }
        5 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 1 as ::core::ffi::c_int;
            }
            102 => {
                low = 2 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 3 as ::core::ffi::c_int;
            }
            114 => {
                low = 4 as ::core::ffi::c_int;
            }
            115 => {
                low = 5 as ::core::ffi::c_int;
            }
            117 => {
                low = 6 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            109 => {
                low = 7 as ::core::ffi::c_int;
            }
            115 => {
                low = 8 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 9 as ::core::ffi::c_int;
            }
            103 => {
                low = 10 as ::core::ffi::c_int;
            }
            109 => {
                low = 11 as ::core::ffi::c_int;
            }
            115 => {
                low = 12 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            98 => {
                low = 13 as ::core::ffi::c_int;
            }
            101 => {
                low = 14 as ::core::ffi::c_int;
            }
            103 => {
                low = 15 as ::core::ffi::c_int;
            }
            111 => {
                low = 16 as ::core::ffi::c_int;
            }
            115 => {
                low = 17 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            99 => {
                low = 18 as ::core::ffi::c_int;
            }
            101 => {
                low = 19 as ::core::ffi::c_int;
            }
            108 => {
                low = 20 as ::core::ffi::c_int;
            }
            114 => {
                low = 21 as ::core::ffi::c_int;
            }
            115 => {
                low = 22 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => {
            low = 23 as ::core::ffi::c_int;
        }
        14 => {
            low = 24 as ::core::ffi::c_int;
        }
        16 => {
            low = 25 as ::core::ffi::c_int;
        }
        18 => {
            low = 26 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if low < 0 as ::core::ffi::c_int
        || memcmp(
            str as *const ::core::ffi::c_void,
            (*event_handlers.ptr())[low as usize].name as *const ::core::ffi::c_void,
            len,
        ) != 0
    {
        return -1 as ::core::ffi::c_int;
    }
    return low;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
