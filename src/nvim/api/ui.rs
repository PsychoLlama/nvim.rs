use crate::src::nvim::api::private::helpers::{
    api_set_error, api_typename, arena_array, arena_dict, cstr_as_string, string_to_cstr,
};
use crate::src::nvim::api::private::validate::{api_err_exp, api_err_invalid};
use crate::src::nvim::autocmd::{do_autocmd_focusgained, may_trigger_vim_suspend_resume};
use crate::src::nvim::event::multiqueue::{multiqueue_empty, multiqueue_process_events};
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::wstream::wstream_new_buffer;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{schar_get, schar_get_adv};
use crate::src::nvim::highlight::{hl_get_url, hlattrs2dict, syn_attr2entry};
use crate::src::nvim::main::{
    channels, current_ui, main_loop, noargs, p_bg, starting, stdin_fd, stdin_isatty, stdout_isatty,
    t_colors, ui_ext_names, Columns,
};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::mbyte::utf_ambiguous_width;
use crate::src::nvim::memory::{
    alloc_block, arena_finish, arena_mem_free, free_block, strequal, xcalloc, xfree, ARENA_EMPTY,
};
use crate::src::nvim::msgpack_rpc::channel::rpc_write_raw;
use crate::src::nvim::msgpack_rpc::packer::mpack_object_array;
use crate::src::nvim::option::set_tty_option;
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memset, strlen};
use crate::src::nvim::os::time::os_hrtime;
pub use crate::src::nvim::types::{
    __gid_t, __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    __uid_t, blob_T, blobvar_S, consumed_blk, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed_1, funccall_T, garray_T, gid_t, handle_T, hash_T,
    hashitem_T, hashtab_T, int16_t, int32_t, int64_t, internal_proc_cb, key_value_pair, linenr_T,
    list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, loop_0,
    loop_0_children as C2Rust_Unnamed_10, multiqueue, object, object_data as C2Rust_Unnamed,
    packer_buffer_t, partial_S, partial_T, proc, proc_exit_cb, proc_state_cb, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, queue, rstream, sattr_T, schar_T, scid_T, sctx_T,
    size_t, ssize_t, stream, stream_close_cb, stream_read_cb, stream_uv as C2Rust_Unnamed_12,
    stream_write_cb, terminal, typval_T, typval_vval_union, ufunc_S, ufunc_T, uid_t, uint16_t,
    uint32_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t, uv__queue, uv_alloc_cb,
    uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_7, uv_async_t, uv_buf_t, uv_close_cb,
    uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb, uv_file, uv_gid_t,
    uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_2, uv_handle_t, uv_handle_type, uv_idle_cb,
    uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_13, uv_idle_t, uv_loop_s,
    uv_loop_s_active_reqs as C2Rust_Unnamed_6, uv_loop_s_timer_heap as C2Rust_Unnamed_5, uv_loop_t,
    uv_mutex_t, uv_pipe_s, uv_pipe_s_u as C2Rust_Unnamed_15, uv_pipe_t, uv_process_options_s,
    uv_process_options_t, uv_process_s, uv_process_s_u as C2Rust_Unnamed_17, uv_process_t,
    uv_read_cb, uv_req_type, uv_rwlock_t, uv_shutdown_cb, uv_shutdown_s, uv_shutdown_t,
    uv_signal_cb, uv_signal_s, uv_signal_s_tree_entry as C2Rust_Unnamed_3,
    uv_signal_s_u as C2Rust_Unnamed_4, uv_signal_t, uv_stdio_container_s,
    uv_stdio_container_s_data as C2Rust_Unnamed_18, uv_stdio_container_t, uv_stdio_flags,
    uv_stream_s, uv_stream_s_u as C2Rust_Unnamed_11, uv_stream_t, uv_tcp_s,
    uv_tcp_s_u as C2Rust_Unnamed_14, uv_tcp_t, uv_timer_cb, uv_timer_s,
    uv_timer_s_node as C2Rust_Unnamed_8, uv_timer_s_u as C2Rust_Unnamed_9, uv_timer_t, uv_uid_t,
    varnumber_T, wbuffer, wbuffer_data_finalizer, winsize, Arena, ArenaMem, Array, BoolVarValue,
    Boolean, Callback, CallbackReader, CallbackType, Callback_data as C2Rust_Unnamed_0, Channel,
    ChannelCallFrame, ChannelStreamType, Channel_stream as C2Rust_Unnamed_21, ClientType, Dict,
    Error, ErrorType, Float, HlAttrs, Integer, InternalState, KeyValuePair, LibuvProc, LineFlags,
    Loop, LuaRef, MapHash, Map_uint64_t_ptr_t, MultiQueue, Object, ObjectType, PackerBuffer,
    PackerBufferFlush, Proc, ProcType, PtyProc, RStream, RemoteUI, RgbValue, RpcState,
    RpcState_call_stack as C2Rust_Unnamed_20, ScopeDictDictItem, ScopeType, Set_uint64_t,
    SpecialVarValue, StderrState, StdioPair, Stream, String_0, Terminal, UIExtension, Unpacker,
    VarLockStatus, VarType, WBuffer, Window, QUEUE,
};
use crate::src::nvim::ui::{
    ui_active, ui_attach_impl, ui_call_ui_send, ui_can_attach_more, ui_detach_impl, ui_grid_resize,
    ui_refresh, ui_set_ext_option,
};
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
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const HLATTRS_DICT_SIZE: C2Rust_Unnamed_16 = 24;
pub const UV_OVERLAPPED_PIPE: uv_stdio_flags = 64;
pub const UV_NONBLOCK_PIPE: uv_stdio_flags = 64;
pub const UV_WRITABLE_PIPE: uv_stdio_flags = 32;
pub const UV_READABLE_PIPE: uv_stdio_flags = 16;
pub const UV_INHERIT_STREAM: uv_stdio_flags = 4;
pub const UV_INHERIT_FD: uv_stdio_flags = 2;
pub const UV_CREATE_PIPE: uv_stdio_flags = 1;
pub const UV_IGNORE: uv_stdio_flags = 0;
pub const kUIExtCount: UIExtension = 10;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUITermColors: UIExtension = 8;
pub const kUIHlState: UIExtension = 7;
pub const kUIMultigrid: UIExtension = 6;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMessages: UIExtension = 4;
pub const kUIWildmenu: UIExtension = 3;
pub const kUITabline: UIExtension = 2;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUICmdline: UIExtension = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kLineFlagInvalid: C2Rust_Unnamed_19 = 2;
pub const kLineFlagWrap: C2Rust_Unnamed_19 = 1;
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kChannelStreamStderr: ChannelStreamType = 3;
pub const kChannelStreamStdio: ChannelStreamType = 2;
pub const kChannelStreamSocket: ChannelStreamType = 1;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
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
pub const SET_INIT: Set_uint64_t = Set_uint64_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint64_t>(),
};
pub const MAP_INIT: Map_uint64_t_ptr_t = Map_uint64_t_ptr_t {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<ptr_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn set_has_uint64_t(mut set: *mut Set_uint64_t, mut key: uint64_t) -> bool {
    return mh_get_uint64_t(set, key) != MH_TOMBSTONE as uint32_t;
}
#[inline]
unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const UI_BUF_SIZE: ::core::ffi::c_int = ARENA_BLOCK_SIZE;
pub const EVENT_BUF_SIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
static connected_uis: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(MAP_INIT);
unsafe extern "C" fn get_ui_or_err(mut chan_id: uint64_t, mut err: *mut Error) -> *mut RemoteUI {
    let mut ui: *mut RemoteUI =
        map_get_uint64_t_ptr_t(connected_uis.ptr(), chan_id) as *mut RemoteUI;
    if ui.is_null() && !err.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"UI not attached to channel: %ld\0".as_ptr() as *const ::core::ffi::c_char,
            chan_id,
        );
    }
    return ui;
}
unsafe extern "C" fn mpack_array_dyn16(
    mut buf: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let c2rust_fresh4 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh4 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut pos: *mut ::core::ffi::c_char = *buf;
    mpack_w2(buf, 0xffef as uint32_t);
    return pos;
}
unsafe extern "C" fn mpack_str_small(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    '_c2rust_label: {
        if len < 0x20 as size_t {
        } else {
            __assert_fail(
                b"len < 0x20\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                71 as ::core::ffi::c_uint,
                b"void mpack_str_small(char **, const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let c2rust_fresh3 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh3 = (0xa0 as size_t | len) as ::core::ffi::c_char;
    memcpy(
        *buf as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    *buf = (*buf).offset(len as isize);
}
unsafe extern "C" fn remote_ui_destroy(mut ui: *mut RemoteUI) {
    xfree((*ui).packer.startptr as *mut ::core::ffi::c_void);
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*ui).term_name as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    xfree(ui as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn remote_ui_disconnect(
    mut channel_id: uint64_t,
    mut err: *mut Error,
    mut send_error_exit: bool,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if send_error_exit {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh0 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        push_call(
            ui,
            b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        ui_flush_buf(ui, false_0 != 0);
    }
    map_del_uint64_t_ptr_t(
        connected_uis.ptr(),
        channel_id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    ui_detach_impl(ui, channel_id);
    let mut chan: *mut Channel = find_channel(channel_id);
    if !chan.is_null() && (*chan).rpc.ui == ui {
        (*chan).rpc.ui = ::core::ptr::null_mut::<RemoteUI>();
    }
    remote_ui_destroy(ui);
}
pub unsafe extern "C" fn remote_ui_wait_for_attach() {
    let mut remaining: int64_t = -1 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while ui_active() == 0 {
        if !(*main_loop.ptr()).events.is_null() && !multiqueue_empty((*main_loop.ptr()).events) {
            multiqueue_process_events((*main_loop.ptr()).events);
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
pub unsafe extern "C" fn nvim_ui_attach(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut options: Dict,
    mut err: *mut Error,
) {
    if set_has_uint64_t(&raw mut (*connected_uis.ptr()).set, channel_id) {
        api_set_error(
            err,
            kErrorTypeException,
            b"UI already attached to channel: %ld\0".as_ptr() as *const ::core::ffi::c_char,
            channel_id,
        );
        return;
    }
    if !ui_can_attach_more() {
        api_set_error(
            err,
            kErrorTypeException,
            b"Maximum UI count reached\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if width <= 0 as Integer || height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0 and height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    let mut ui: *mut RemoteUI =
        xcalloc(1 as size_t, ::core::mem::size_of::<RemoteUI>()) as *mut RemoteUI;
    (*ui).channel_id = channel_id;
    (*ui).width = width as ::core::ffi::c_int;
    (*ui).height = height as ::core::ffi::c_int;
    (*ui).pum_row = -1.0f64;
    (*ui).pum_col = -1.0f64;
    (*ui).rgb = true_0 != 0;
    memset(
        &raw mut (*ui).ui_ext as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[bool; 10]>(),
    );
    let mut i: size_t = 0 as size_t;
    while i < options.size {
        ui_set_option(
            ui,
            true_0 != 0,
            (*options.items.offset(i as isize)).key,
            (*options.items.offset(i as isize)).value,
            err,
        );
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            xfree(ui as *mut ::core::ffi::c_void);
            return;
        }
        i = i.wrapping_add(1);
    }
    if (*ui).ui_ext[kUIHlState as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
        || (*ui).ui_ext[kUIMultigrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0
    {
        (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] = true_0 != 0;
    }
    if (*ui).ui_ext[kUIMessages as ::core::ffi::c_int as usize] {
        (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] = true_0 != 0;
        (*ui).ui_ext[kUICmdline as ::core::ffi::c_int as usize] = true_0 != 0;
    }
    (*ui).cur_event = ::core::ptr::null::<::core::ffi::c_char>();
    (*ui).hl_id = 0 as ::core::ffi::c_int;
    (*ui).client_col = -1 as Integer;
    (*ui).nevents_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).nevents = 0 as uint32_t;
    (*ui).flushed_events = false_0 != 0;
    (*ui).incomplete_event = false_0 != 0;
    (*ui).ncalls_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).ncalls = 0 as uint32_t;
    (*ui).ncells_pending = 0 as size_t;
    (*ui).packer = packer_buffer_t {
        startptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        endptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        anydata: ui as *mut ::core::ffi::c_void,
        anyint: 0,
        packer_flush: Some(ui_flush_callback as unsafe extern "C" fn(*mut PackerBuffer) -> ()),
    };
    (*ui).wildmenu_active = false_0 != 0;
    map_put_uint64_t_ptr_t(connected_uis.ptr(), channel_id, ui as ptr_t);
    current_ui.set(channel_id);
    ui_attach_impl(ui, channel_id);
    let mut chan: *mut Channel = find_channel(channel_id);
    if !chan.is_null() {
        (*chan).rpc.ui = ui;
    }
    may_trigger_vim_suspend_resume(false_0 != 0);
}
pub unsafe extern "C" fn ui_attach(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut enable_rgb: Boolean,
    mut err: *mut Error,
) {
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
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 1];
    opts.capacity = 1 as size_t;
    opts.items = &raw mut opts__items as *mut KeyValuePair;
    let c2rust_fresh17 = opts.size;
    opts.size = opts.size.wrapping_add(1);
    *opts.items.offset(c2rust_fresh17 as isize) = key_value_pair {
        key: cstr_as_string(b"rgb\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeBoolean,
            data: C2Rust_Unnamed {
                boolean: enable_rgb,
            },
        },
    };
    nvim_ui_attach(channel_id, width, height, opts, err);
}
pub unsafe extern "C" fn nvim_ui_set_focus(
    mut channel_id: uint64_t,
    mut gained: Boolean,
    mut error: *mut Error,
) {
    if get_ui_or_err(channel_id, error).is_null() {
        return;
    }
    if gained {
        current_ui.set(channel_id);
        may_trigger_vim_suspend_resume(false_0 != 0);
    }
    do_autocmd_focusgained(gained);
}
pub unsafe extern "C" fn nvim_ui_detach(mut channel_id: uint64_t, mut err: *mut Error) {
    remote_ui_disconnect(channel_id, err, false_0 != 0);
}
pub unsafe extern "C" fn remote_ui_connect(
    mut channel_id: uint64_t,
    mut server_addr: *mut ::core::ffi::c_char,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh18 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh18 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(server_addr),
        },
    };
    push_call(
        ui,
        b"connect\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_stop(mut _ui: *mut RemoteUI) {}
pub unsafe extern "C" fn nvim_ui_try_resize(
    mut channel_id: uint64_t,
    mut width: Integer,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if width <= 0 as Integer || height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0 and height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).width = width as ::core::ffi::c_int;
    (*ui).height = height as ::core::ffi::c_int;
    ui_refresh();
}
pub unsafe extern "C" fn nvim_ui_set_option(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut value: Object,
    mut error: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, error);
    if ui.is_null() {
        return;
    }
    ui_set_option(ui, false_0 != 0, name, value, error);
}
unsafe extern "C" fn ui_set_option(
    mut ui: *mut RemoteUI,
    mut init: bool,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    if strequal(
        name.data,
        b"override\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"override\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        (*ui).override_0 = value.data.boolean as bool;
        return;
    }
    if strequal(name.data, b"rgb\0".as_ptr() as *const ::core::ffi::c_char) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"rgb\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        (*ui).rgb = value.data.boolean as bool;
        if !init && !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            ui_refresh();
        }
        return;
    }
    if strequal(
        name.data,
        b"term_name\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"term_name\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeString),
                api_typename(value.type_0),
            );
            return;
        }
        set_tty_option(
            b"term\0".as_ptr() as *const ::core::ffi::c_char,
            string_to_cstr(value.data.string),
        );
        (*ui).term_name = string_to_cstr(value.data.string);
        return;
    }
    if strequal(
        name.data,
        b"term_colors\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"term_colors\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeInteger),
                api_typename(value.type_0),
            );
            return;
        }
        t_colors.set(value.data.integer as ::core::ffi::c_int);
        (*ui).term_colors = value.data.integer as ::core::ffi::c_int;
        return;
    }
    if strequal(
        name.data,
        b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeInteger),
                api_typename(value.type_0),
            );
            return;
        }
        if !(value.data.integer >= 0 as Integer) {
            api_err_invalid(
                err,
                b"stdin_fd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
                value.data.integer as int64_t,
                false_0 != 0,
            );
            return;
        }
        if !(starting.get() == 2 as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeValidation,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"stdin_fd can only be used with first attached UI\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            return;
        }
        stdin_fd.set(value.data.integer as ::core::ffi::c_int);
        return;
    }
    if strequal(
        name.data,
        b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdin_tty\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        if (*ui).channel_id == CHAN_STDIO as uint64_t {
            stdin_isatty.set(value.data.boolean as bool);
        }
        (*ui).stdin_tty = value.data.boolean as bool;
        return;
    }
    if strequal(
        name.data,
        b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        if kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            != value.type_0 as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                b"stdout_tty\0".as_ptr() as *const ::core::ffi::c_char,
                api_typename(kObjectTypeBoolean),
                api_typename(value.type_0),
            );
            return;
        }
        if (*ui).channel_id == CHAN_STDIO as uint64_t {
            stdout_isatty.set(value.data.boolean as bool);
        }
        (*ui).stdout_tty = value.data.boolean as bool;
        return;
    }
    let mut is_popupmenu: bool = strequal(
        name.data,
        b"popupmenu_external\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut i: UIExtension = kUICmdline;
    while (i as ::core::ffi::c_uint) < kUIExtCount as ::core::ffi::c_int as ::core::ffi::c_uint {
        if strequal(
            name.data,
            *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(i as isize),
        ) as ::core::ffi::c_int
            != 0
            || i as ::core::ffi::c_uint == kUIPopupmenu as ::core::ffi::c_int as ::core::ffi::c_uint
                && is_popupmenu as ::core::ffi::c_int != 0
        {
            if !(value.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                api_err_exp(
                    err,
                    name.data,
                    b"Boolean\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(value.type_0),
                );
                return;
            }
            let mut boolval: bool = value.data.boolean as bool;
            if !init
                && i as ::core::ffi::c_uint
                    == kUILinegrid as ::core::ffi::c_int as ::core::ffi::c_uint
                && boolval as ::core::ffi::c_int != (*ui).ui_ext[i as usize] as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"ext_linegrid option cannot be changed\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
            (*ui).ui_ext[i as usize] = boolval;
            if !init {
                ui_set_ext_option(ui, i, boolval);
            }
            return;
        }
        i += 1;
    }
    if true {
        api_err_invalid(
            err,
            b"UI option\0".as_ptr() as *const ::core::ffi::c_char,
            name.data,
            0 as int64_t,
            true_0 != 0,
        );
        return;
    }
}
pub unsafe extern "C" fn nvim_ui_try_resize_grid(
    mut channel_id: uint64_t,
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
    mut err: *mut Error,
) {
    if get_ui_or_err(channel_id, err).is_null() {
        return;
    }
    if grid == DEFAULT_GRID_HANDLE as Integer {
        nvim_ui_try_resize(channel_id, width, height, err);
    } else {
        ui_grid_resize(
            grid as handle_T,
            width as ::core::ffi::c_int,
            height as ::core::ffi::c_int,
            err,
        );
    };
}
pub unsafe extern "C" fn nvim_ui_pum_set_height(
    mut channel_id: uint64_t,
    mut height: Integer,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if height <= 0 as Integer {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected pum height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize] {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"UI must support the ext_popupmenu option\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).pum_nlines = height as ::core::ffi::c_int;
}
pub unsafe extern "C" fn nvim_ui_pum_set_bounds(
    mut channel_id: uint64_t,
    mut width: Float,
    mut height: Float,
    mut row: Float,
    mut col: Float,
    mut err: *mut Error,
) {
    let mut ui: *mut RemoteUI = get_ui_or_err(channel_id, err);
    if ui.is_null() {
        return;
    }
    if !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize] {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"UI must support the ext_popupmenu option\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if width <= 0 as ::core::ffi::c_int as Float {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected width > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    } else if height <= 0 as ::core::ffi::c_int as Float {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Expected height > 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    (*ui).pum_row = row;
    (*ui).pum_col = col;
    (*ui).pum_width = width;
    (*ui).pum_height = height;
    (*ui).pum_pos = true_0 != 0;
}
unsafe extern "C" fn flush_event(mut ui: *mut RemoteUI) {
    if !(*ui).cur_event.is_null() {
        mpack_w2(
            &raw mut (*ui).ncalls_pos,
            (1 as uint32_t).wrapping_add((*ui).ncalls),
        );
        (*ui).cur_event = ::core::ptr::null::<::core::ffi::c_char>();
        (*ui).ncalls_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*ui).ncalls = 0 as uint32_t;
    }
}
unsafe extern "C" fn ui_alloc_buf(mut ui: *mut RemoteUI) {
    (*ui).packer.startptr = alloc_block() as *mut ::core::ffi::c_char;
    (*ui).packer.ptr = (*ui).packer.startptr;
    (*ui).packer.endptr = (*ui).packer.startptr.offset(UI_BUF_SIZE as isize);
}
unsafe extern "C" fn prepare_call(mut ui: *mut RemoteUI, mut name: *const ::core::ffi::c_char) {
    if !(*ui).packer.startptr.is_null()
        && ((*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t
            > (UI_BUF_SIZE - EVENT_BUF_SIZE) as size_t
            || (*ui).ncells_pending >= 500 as size_t)
    {
        ui_flush_buf(ui, false_0 != 0);
    }
    if (*ui).packer.startptr.is_null() {
        ui_alloc_buf(ui);
    }
    if (*ui).cur_event.is_null() || !strequal((*ui).cur_event, name) {
        let mut buf: *mut *mut ::core::ffi::c_char = &raw mut (*ui).packer.ptr;
        if (*ui).nevents_pos.is_null() {
            mpack_array(buf, 3 as uint32_t);
            mpack_uint(buf, 2 as uint32_t);
            mpack_str_small(
                buf,
                b"redraw\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            );
            (*ui).nevents_pos = mpack_array_dyn16(buf);
            '_c2rust_label: {
                if (*ui).cur_event.is_null() {
                } else {
                    __assert_fail(
                        b"ui->cur_event == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/ui.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        549 as ::core::ffi::c_uint,
                        b"void prepare_call(RemoteUI *, const char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
        }
        flush_event(ui);
        (*ui).cur_event = name;
        (*ui).ncalls_pos = mpack_array_dyn16(buf);
        mpack_str_small(buf, name, strlen(name));
        (*ui).nevents = (*ui).nevents.wrapping_add(1);
        (*ui).ncalls = 1 as uint32_t;
    } else {
        (*ui).ncalls = (*ui).ncalls.wrapping_add(1);
    };
}
unsafe extern "C" fn push_call(
    mut ui: *mut RemoteUI,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
) {
    prepare_call(ui, name);
    mpack_object_array(args, &raw mut (*ui).packer);
}
unsafe extern "C" fn ui_flush_callback(mut packer: *mut PackerBuffer) {
    let mut ui: *mut RemoteUI = (*packer).anydata as *mut RemoteUI;
    ui_flush_buf(ui, true_0 != 0);
    ui_alloc_buf(ui);
}
pub unsafe extern "C" fn remote_ui_grid_clear(mut ui: *mut RemoteUI, mut grid: Integer) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let c2rust_fresh19 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh19 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
    }
    let mut name: *const ::core::ffi::c_char =
        if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0 {
            b"grid_clear\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"clear\0".as_ptr() as *const ::core::ffi::c_char
        };
    push_call(ui, name, args);
}
pub unsafe extern "C" fn remote_ui_grid_resize(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut width: Integer,
    mut height: Integer,
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
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let c2rust_fresh20 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh20 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
    } else {
        (*ui).client_col = -1 as Integer;
    }
    let c2rust_fresh21 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh21 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: width },
    };
    let c2rust_fresh22 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh22 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: height },
    };
    let mut name: *const ::core::ffi::c_char =
        if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0 {
            b"grid_resize\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"resize\0".as_ptr() as *const ::core::ffi::c_char
        };
    push_call(ui, name, args);
}
pub unsafe extern "C" fn remote_ui_grid_scroll(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut top: Integer,
    mut bot: Integer,
    mut left: Integer,
    mut right: Integer,
    mut rows: Integer,
    mut cols: Integer,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 7] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 7];
        args.capacity = 7 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh23 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh23 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
        let c2rust_fresh24 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh24 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: top },
        };
        let c2rust_fresh25 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh25 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: bot },
        };
        let c2rust_fresh26 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh26 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: left },
        };
        let c2rust_fresh27 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh27 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: right },
        };
        let c2rust_fresh28 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh28 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: rows },
        };
        let c2rust_fresh29 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh29 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: cols },
        };
        push_call(
            ui,
            b"grid_scroll\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    } else {
        let mut args_0: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items_0: [Object; 4] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 4];
        args_0.capacity = 4 as size_t;
        args_0.items = &raw mut args__items_0 as *mut Object;
        let c2rust_fresh30 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh30 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: top },
        };
        let c2rust_fresh31 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh31 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: bot - 1 as Integer,
            },
        };
        let c2rust_fresh32 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh32 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: left },
        };
        let c2rust_fresh33 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh33 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: right - 1 as Integer,
            },
        };
        push_call(
            ui,
            b"set_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
        args_0.size = 0 as size_t;
        let c2rust_fresh34 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh34 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: rows },
        };
        push_call(
            ui,
            b"scroll\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
        args_0.size = 0 as size_t;
        let c2rust_fresh35 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh35 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh36 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh36 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ((*ui).height - 1 as ::core::ffi::c_int) as Integer,
            },
        };
        let c2rust_fresh37 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh37 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: 0 as Integer,
            },
        };
        let c2rust_fresh38 = args_0.size;
        args_0.size = args_0.size.wrapping_add(1);
        *args_0.items.offset(c2rust_fresh38 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: ((*ui).width - 1 as ::core::ffi::c_int) as Integer,
            },
        };
        push_call(
            ui,
            b"set_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
            args_0,
        );
    };
}
pub unsafe extern "C" fn remote_ui_default_colors_set(
    mut ui: *mut RemoteUI,
    mut rgb_fg: Integer,
    mut rgb_bg: Integer,
    mut rgb_sp: Integer,
    mut cterm_fg: Integer,
    mut cterm_bg: Integer,
) {
    if !(*ui).ui_ext[kUITermColors as ::core::ffi::c_int as usize] {
        let mut dark_: bool = *p_bg.get() as ::core::ffi::c_int == 'd' as ::core::ffi::c_int;
        rgb_fg = if rgb_fg != -1 as Integer {
            rgb_fg
        } else {
            (if dark_ as ::core::ffi::c_int != 0 {
                0xffffff as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as Integer
        };
        rgb_bg = if rgb_bg != -1 as Integer {
            rgb_bg
        } else {
            (if dark_ as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                0xffffff as ::core::ffi::c_int
            }) as Integer
        };
        rgb_sp = if rgb_sp != -1 as Integer {
            rgb_sp
        } else {
            0xff0000 as Integer
        };
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 5] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 5];
    args.capacity = 5 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh39 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh39 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_fg },
    };
    let c2rust_fresh40 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh40 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_bg },
    };
    let c2rust_fresh41 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh41 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: rgb_sp },
    };
    let c2rust_fresh42 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh42 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: cterm_fg },
    };
    let c2rust_fresh43 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh43 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: cterm_bg },
    };
    push_call(
        ui,
        b"default_colors_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
    if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        args.size = 0 as size_t;
        let c2rust_fresh44 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh44 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_fg
                } else {
                    cterm_fg - 1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_fg\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        args.size = 0 as size_t;
        let c2rust_fresh45 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh45 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_bg
                } else {
                    cterm_bg - 1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_bg\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
        args.size = 0 as size_t;
        let c2rust_fresh46 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh46 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed {
                integer: if (*ui).rgb as ::core::ffi::c_int != 0 {
                    rgb_sp
                } else {
                    -1 as Integer
                },
            },
        };
        push_call(
            ui,
            b"update_sp\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    }
}
pub unsafe extern "C" fn remote_ui_hl_attr_define(
    mut ui: *mut RemoteUI,
    mut id: Integer,
    mut rgb_attrs: HlAttrs,
    mut _cterm_attrs: HlAttrs,
    mut info: Array,
) {
    if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 4] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 4];
    args.capacity = 4 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh47 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh47 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: id },
    };
    let mut rgb: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut rgb__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    rgb.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    rgb.items = &raw mut rgb__items as *mut KeyValuePair;
    let mut cterm: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut cterm__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    cterm.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    cterm.items = &raw mut cterm__items as *mut KeyValuePair;
    hlattrs2dict(
        &raw mut rgb,
        ::core::ptr::null_mut::<Dict>(),
        rgb_attrs,
        true_0 != 0,
        false_0 != 0,
    );
    hlattrs2dict(
        &raw mut cterm,
        ::core::ptr::null_mut::<Dict>(),
        rgb_attrs,
        false_0 != 0,
        false_0 != 0,
    );
    if rgb_attrs.url >= 0 as int32_t {
        let mut url: *const ::core::ffi::c_char = hl_get_url(rgb_attrs.url as uint32_t);
        let c2rust_fresh48 = rgb.size;
        rgb.size = rgb.size.wrapping_add(1);
        *rgb.items.offset(c2rust_fresh48 as isize) = key_value_pair {
            key: cstr_as_string(b"url\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(url),
                },
            },
        };
    }
    let c2rust_fresh49 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh49 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: rgb },
    };
    let c2rust_fresh50 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh50 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: cterm },
    };
    if (*ui).ui_ext[kUIHlState as ::core::ffi::c_int as usize] {
        let c2rust_fresh51 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh51 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: info },
        };
    } else {
        let c2rust_fresh52 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh52 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                },
            },
        };
    }
    push_call(
        ui,
        b"hl_attr_define\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_highlight_set(
    mut ui: *mut RemoteUI,
    mut id: ::core::ffi::c_int,
) {
    if (*ui).hl_id == id {
        return;
    }
    (*ui).hl_id = id;
    let mut dict: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut dict__items: [KeyValuePair; 24] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 24];
    dict.capacity = HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t;
    dict.items = &raw mut dict__items as *mut KeyValuePair;
    hlattrs2dict(
        &raw mut dict,
        ::core::ptr::null_mut::<Dict>(),
        syn_attr2entry(id),
        (*ui).rgb,
        false_0 != 0,
    );
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh53 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh53 as isize) = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: dict },
    };
    push_call(
        ui,
        b"highlight_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_grid_cursor_goto(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
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
        let c2rust_fresh54 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh54 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: grid },
        };
        let c2rust_fresh55 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh55 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: row },
        };
        let c2rust_fresh56 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh56 as isize) = object {
            type_0: kObjectTypeInteger,
            data: C2Rust_Unnamed { integer: col },
        };
        push_call(
            ui,
            b"grid_cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
            args,
        );
    } else {
        (*ui).cursor_row = row;
        (*ui).cursor_col = col;
        remote_ui_cursor_goto(ui, row, col);
    };
}
pub unsafe extern "C" fn remote_ui_cursor_goto(
    mut ui: *mut RemoteUI,
    mut row: Integer,
    mut col: Integer,
) {
    if (*ui).client_row == row && (*ui).client_col == col {
        return;
    }
    (*ui).client_row = row;
    (*ui).client_col = col;
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
    let c2rust_fresh57 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh57 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh58 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh58 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: col },
    };
    push_call(
        ui,
        b"cursor_goto\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_put(
    mut ui: *mut RemoteUI,
    mut cell: *const ::core::ffi::c_char,
) {
    (*ui).client_col += 1;
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh59 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh59 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed {
            string: cstr_as_string(cell),
        },
    };
    push_call(ui, b"put\0".as_ptr() as *const ::core::ffi::c_char, args);
}
pub unsafe extern "C" fn remote_ui_raw_line(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut startcol: Integer,
    mut endcol: Integer,
    mut clearcol: Integer,
    mut clearattr: Integer,
    mut flags: LineFlags,
    mut chunk: *const schar_T,
    mut attrs: *const sattr_T,
) {
    if (*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
        prepare_call(ui, b"grid_line\0".as_ptr() as *const ::core::ffi::c_char);
        let mut buf: *mut *mut ::core::ffi::c_char = &raw mut (*ui).packer.ptr;
        mpack_array(buf, 5 as uint32_t);
        mpack_uint(buf, grid as uint32_t);
        mpack_uint(buf, row as uint32_t);
        mpack_uint(buf, startcol as uint32_t);
        let mut lenpos: *mut ::core::ffi::c_char = mpack_array_dyn16(buf);
        let mut repeat: uint32_t = 0 as uint32_t;
        let mut ncells: size_t = (endcol - startcol) as size_t;
        let mut last_hl: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut nelem: uint32_t = 0 as uint32_t;
        let mut was_space: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < ncells {
            repeat = repeat.wrapping_add(1);
            if i == ncells.wrapping_sub(1 as size_t)
                || *attrs.offset(i as isize) != *attrs.offset(i.wrapping_add(1 as size_t) as isize)
                || *chunk.offset(i as isize) != *chunk.offset(i.wrapping_add(1 as size_t) as isize)
            {
                if (UI_BUF_SIZE as size_t)
                    .wrapping_sub((*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t)
                    < (2 as ::core::ffi::c_int
                        * (1 as ::core::ffi::c_int
                            + 2 as ::core::ffi::c_int
                            + MAX_SCHAR_SIZE
                            + 5 as ::core::ffi::c_int
                            + 5 as ::core::ffi::c_int)
                        + 1 as ::core::ffi::c_int) as size_t
                    || (*ui).ncells_pending >= 500 as size_t
                {
                    if was_space {
                        nelem = nelem.wrapping_add(1);
                        (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(1 as size_t);
                        mpack_array(buf, 3 as uint32_t);
                        mpack_str_small(
                            buf,
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        );
                        mpack_uint(buf, clearattr as uint32_t);
                        mpack_uint(buf, 0 as uint32_t);
                    }
                    mpack_w2(&raw mut lenpos, nelem);
                    mpack_bool(buf, false_0 != 0);
                    ui_flush_buf(ui, false_0 != 0);
                    prepare_call(ui, b"grid_line\0".as_ptr() as *const ::core::ffi::c_char);
                    mpack_array(buf, 5 as uint32_t);
                    mpack_uint(buf, grid as uint32_t);
                    mpack_uint(buf, row as uint32_t);
                    mpack_uint(
                        buf,
                        (startcol as uint32_t)
                            .wrapping_add(i as uint32_t)
                            .wrapping_sub(repeat)
                            .wrapping_add(1 as uint32_t),
                    );
                    lenpos = mpack_array_dyn16(buf);
                    nelem = 0 as uint32_t;
                    last_hl = -1 as ::core::ffi::c_int;
                }
                let mut csize: uint32_t = (if repeat > 1 as uint32_t {
                    3 as ::core::ffi::c_int
                } else if *attrs.offset(i as isize) != last_hl as sattr_T {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as uint32_t;
                nelem = nelem.wrapping_add(1);
                mpack_array(buf, csize);
                let c2rust_fresh60 = *buf;
                *buf = (*buf).offset(1);
                let mut size_byte: *mut ::core::ffi::c_char = c2rust_fresh60;
                let mut len: size_t = schar_get_adv(buf, *chunk.offset(i as isize));
                *size_byte = (0xa0 as size_t | len) as ::core::ffi::c_char;
                if csize >= 2 as uint32_t {
                    mpack_uint(buf, *attrs.offset(i as isize) as uint32_t);
                    if csize >= 3 as uint32_t {
                        mpack_uint(buf, repeat);
                    }
                }
                (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(
                    (if repeat < 2 as uint32_t {
                        repeat
                    } else {
                        2 as uint32_t
                    }) as size_t,
                );
                last_hl = *attrs.offset(i as isize) as ::core::ffi::c_int;
                repeat = 0 as uint32_t;
                was_space = *chunk.offset(i as isize) == ' ' as ::core::ffi::c_int as schar_T;
            }
            i = i.wrapping_add(1);
        }
        if endcol < clearcol || was_space as ::core::ffi::c_int != 0 {
            nelem = nelem.wrapping_add(1);
            (*ui).ncells_pending = (*ui).ncells_pending.wrapping_add(1 as size_t);
            mpack_array(buf, 3 as uint32_t);
            mpack_str_small(
                buf,
                b" \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
            mpack_uint(buf, clearattr as uint32_t);
            mpack_uint(buf, (clearcol - endcol) as uint32_t);
        }
        mpack_w2(&raw mut lenpos, nelem);
        mpack_bool(
            buf,
            flags as ::core::ffi::c_int & kLineFlagWrap as ::core::ffi::c_int != 0,
        );
    } else {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as Integer) < endcol - startcol {
            remote_ui_cursor_goto(ui, row, startcol + i_0 as Integer);
            remote_ui_highlight_set(ui, *attrs.offset(i_0 as isize) as ::core::ffi::c_int);
            let mut sc_buf: [::core::ffi::c_char; 32] = [0; 32];
            schar_get(
                &raw mut sc_buf as *mut ::core::ffi::c_char,
                *chunk.offset(i_0 as isize),
            );
            remote_ui_put(ui, &raw mut sc_buf as *mut ::core::ffi::c_char);
            if utf_ambiguous_width(&raw mut sc_buf as *mut ::core::ffi::c_char) {
                (*ui).client_col = -1 as Integer;
            }
            i_0 += 1;
        }
        if endcol < clearcol {
            remote_ui_cursor_goto(ui, row, endcol);
            remote_ui_highlight_set(ui, clearattr as ::core::ffi::c_int);
            if clearattr == 0 as Integer && clearcol == Columns.get() as Integer {
                let mut args: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                push_call(
                    ui,
                    b"eol_clear\0".as_ptr() as *const ::core::ffi::c_char,
                    args,
                );
            } else {
                let mut c: Integer = endcol;
                while c < clearcol {
                    remote_ui_put(ui, b" \0".as_ptr() as *const ::core::ffi::c_char);
                    c += 1;
                }
            }
        }
    };
}
unsafe extern "C" fn ui_flush_buf(mut ui: *mut RemoteUI, mut incomplete_event: bool) {
    if (*ui).packer.startptr.is_null()
        || (*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t == 0
    {
        return;
    }
    (*ui).incomplete_event = incomplete_event;
    flush_event(ui);
    if !(*ui).nevents_pos.is_null() {
        mpack_w2(&raw mut (*ui).nevents_pos, (*ui).nevents);
        (*ui).nevents = 0 as uint32_t;
        (*ui).nevents_pos = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut WBuffer = wstream_new_buffer(
        (*ui).packer.startptr,
        (*ui).packer.ptr.offset_from((*ui).packer.startptr) as size_t,
        1 as size_t,
        Some(free_block as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    rpc_write_raw((*ui).channel_id, buf);
    (*ui).packer.startptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).packer.ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ui).flushed_events = true_0 != 0;
    (*ui).ncells_pending = 0 as size_t;
}
pub unsafe extern "C" fn remote_ui_flush(mut ui: *mut RemoteUI) {
    if (*ui).nevents > 0 as uint32_t || (*ui).flushed_events as ::core::ffi::c_int != 0 {
        if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            remote_ui_cursor_goto(ui, (*ui).cursor_row, (*ui).cursor_col);
        }
        push_call(
            ui,
            b"flush\0".as_ptr() as *const ::core::ffi::c_char,
            Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            },
        );
        ui_flush_buf(ui, false_0 != 0);
        (*ui).flushed_events = false_0 != 0;
    }
}
pub unsafe extern "C" fn remote_ui_ui_send(mut ui: *mut RemoteUI, mut content: String_0) {
    if !(*ui).stdout_tty {
        return;
    }
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh62 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh62 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: content },
    };
    push_call(
        ui,
        b"ui_send\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_flush_pending_data(mut ui: *mut RemoteUI) {
    ui_flush_buf(ui, false_0 != 0);
}
unsafe extern "C" fn translate_contents(
    mut ui: *mut RemoteUI,
    mut contents: Array,
    mut arena: *mut Arena,
) -> Array {
    let mut new_contents: Array = arena_array(arena, contents.size);
    let mut i: size_t = 0 as size_t;
    while i < contents.size {
        let mut item: Array = (*contents.items.offset(i as isize)).data.array;
        let mut new_item: Array = arena_array(arena, 2 as size_t);
        let mut attr: ::core::ffi::c_int = (*item.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as ::core::ffi::c_int;
        if attr != 0 {
            let mut rgb_attrs: Dict =
                arena_dict(arena, HLATTRS_DICT_SIZE as ::core::ffi::c_int as size_t);
            hlattrs2dict(
                &raw mut rgb_attrs,
                ::core::ptr::null_mut::<Dict>(),
                syn_attr2entry(attr),
                (*ui).rgb,
                false_0 != 0,
            );
            let c2rust_fresh70 = new_item.size;
            new_item.size = new_item.size.wrapping_add(1);
            *new_item.items.offset(c2rust_fresh70 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: rgb_attrs },
            };
        } else {
            let c2rust_fresh71 = new_item.size;
            new_item.size = new_item.size.wrapping_add(1);
            *new_item.items.offset(c2rust_fresh71 as isize) = object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed {
                    dict: Dict {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<KeyValuePair>(),
                    },
                },
            };
        }
        let c2rust_fresh72 = new_item.size;
        new_item.size = new_item.size.wrapping_add(1);
        *new_item.items.offset(c2rust_fresh72 as isize) =
            *item.items.offset(1 as ::core::ffi::c_int as isize);
        let c2rust_fresh73 = new_contents.size;
        new_contents.size = new_contents.size.wrapping_add(1);
        *new_contents.items.offset(c2rust_fresh73 as isize) = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed { array: new_item },
        };
        i = i.wrapping_add(1);
    }
    return new_contents;
}
unsafe extern "C" fn translate_firstarg(
    mut ui: *mut RemoteUI,
    mut args: Array,
    mut arena: *mut Arena,
) -> Array {
    let mut new_args: Array = arena_array(arena, args.size);
    let mut contents: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .array;
    let c2rust_fresh68 = new_args.size;
    new_args.size = new_args.size.wrapping_add(1);
    *new_args.items.offset(c2rust_fresh68 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: translate_contents(ui, contents, arena),
        },
    };
    let mut i: size_t = 1 as size_t;
    while i < args.size {
        let c2rust_fresh69 = new_args.size;
        new_args.size = new_args.size.wrapping_add(1);
        *new_args.items.offset(c2rust_fresh69 as isize) = *args.items.offset(i as isize);
        i = i.wrapping_add(1);
    }
    return new_args;
}
pub unsafe extern "C" fn remote_ui_event(
    mut ui: *mut RemoteUI,
    mut name: *mut ::core::ffi::c_char,
    mut args: Array,
) {
    let mut arena: Arena = ARENA_EMPTY;
    '_free_ret: {
        if !(*ui).ui_ext[kUILinegrid as ::core::ffi::c_int as usize] {
            if strequal(
                name,
                b"cmdline_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut new_args: Array = translate_firstarg(ui, args, &raw mut arena);
                push_call(ui, name, new_args);
                break '_free_ret;
            } else if strequal(
                name,
                b"cmdline_block_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut block: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .array;
                let mut new_block: Array = arena_array(&raw mut arena, block.size);
                let mut i: size_t = 0 as size_t;
                while i < block.size {
                    let c2rust_fresh63 = new_block.size;
                    new_block.size = new_block.size.wrapping_add(1);
                    *new_block.items.offset(c2rust_fresh63 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed {
                            array: translate_contents(
                                ui,
                                (*block.items.offset(i as isize)).data.array,
                                &raw mut arena,
                            ),
                        },
                    };
                    i = i.wrapping_add(1);
                }
                let mut new_args_0: Array = Array {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<Object>(),
                };
                let mut new_args__items: [Object; 1] = [Object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }; 1];
                new_args_0.capacity = 1 as size_t;
                new_args_0.items = &raw mut new_args__items as *mut Object;
                let c2rust_fresh64 = new_args_0.size;
                new_args_0.size = new_args_0.size.wrapping_add(1);
                *new_args_0.items.offset(c2rust_fresh64 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: new_block },
                };
                push_call(ui, name, new_args_0);
                break '_free_ret;
            } else if strequal(
                name,
                b"cmdline_block_append\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                let mut new_args_1: Array = translate_firstarg(ui, args, &raw mut arena);
                push_call(ui, name, new_args_1);
                break '_free_ret;
            }
        }
        if (*ui).ui_ext[kUIWildmenu as ::core::ffi::c_int as usize] {
            if strequal(
                name,
                b"popupmenu_show\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                (*ui).wildmenu_active = (*args.items.offset(4 as ::core::ffi::c_int as isize))
                    .data
                    .integer
                    == -1 as Integer
                    || !(*ui).ui_ext[kUIPopupmenu as ::core::ffi::c_int as usize];
                if (*ui).wildmenu_active {
                    let mut items: Array = (*args.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .array;
                    let mut new_items: Array = arena_array(&raw mut arena, items.size);
                    let mut i_0: size_t = 0 as size_t;
                    while i_0 < items.size {
                        let c2rust_fresh65 = new_items.size;
                        new_items.size = new_items.size.wrapping_add(1);
                        *new_items.items.offset(c2rust_fresh65 as isize) =
                            *(*items.items.offset(i_0 as isize))
                                .data
                                .array
                                .items
                                .offset(0 as ::core::ffi::c_int as isize);
                        i_0 = i_0.wrapping_add(1);
                    }
                    let mut new_args_2: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut new_args__items_0: [Object; 1] = [Object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    }; 1];
                    new_args_2.capacity = 1 as size_t;
                    new_args_2.items = &raw mut new_args__items_0 as *mut Object;
                    let c2rust_fresh66 = new_args_2.size;
                    new_args_2.size = new_args_2.size.wrapping_add(1);
                    *new_args_2.items.offset(c2rust_fresh66 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: new_items },
                    };
                    push_call(
                        ui,
                        b"wildmenu_show\0".as_ptr() as *const ::core::ffi::c_char,
                        new_args_2,
                    );
                    if (*args.items.offset(1 as ::core::ffi::c_int as isize))
                        .data
                        .integer
                        != -1 as Integer
                    {
                        new_args_2.size = 0 as size_t;
                        let c2rust_fresh67 = new_args_2.size;
                        new_args_2.size = new_args_2.size.wrapping_add(1);
                        *new_args_2.items.offset(c2rust_fresh67 as isize) =
                            *args.items.offset(1 as ::core::ffi::c_int as isize);
                        push_call(
                            ui,
                            b"wildmenu_select\0".as_ptr() as *const ::core::ffi::c_char,
                            new_args_2,
                        );
                    }
                    break '_free_ret;
                }
            } else if strequal(
                name,
                b"popupmenu_select\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                if (*ui).wildmenu_active {
                    name = b"wildmenu_select\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
            } else if strequal(
                name,
                b"popupmenu_hide\0".as_ptr() as *const ::core::ffi::c_char,
            ) {
                if (*ui).wildmenu_active {
                    name = b"wildmenu_hide\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
            }
        }
        push_call(ui, name, args);
        return;
    }
    arena_mem_free(arena_finish(&raw mut arena));
}
pub unsafe extern "C" fn nvim_ui_send(
    mut _channel_id: uint64_t,
    mut content: String_0,
    mut _err: *mut Error,
) {
    ui_call_ui_send(content);
}
pub unsafe extern "C" fn remote_ui_mode_info_set(
    mut ui: *mut RemoteUI,
    mut enabled: Boolean,
    mut cursor_styles: Array,
) {
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
    let c2rust_fresh74 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh74 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed { boolean: enabled },
    };
    let c2rust_fresh75 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh75 as isize) = object {
        type_0: kObjectTypeArray,
        data: C2Rust_Unnamed {
            array: cursor_styles,
        },
    };
    push_call(
        ui,
        b"mode_info_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_update_menu(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"update_menu\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_busy_start(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"busy_start\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_busy_stop(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"busy_stop\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_mouse_on(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"mouse_on\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_mouse_off(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"mouse_off\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_mode_change(
    mut ui: *mut RemoteUI,
    mut mode: String_0,
    mut mode_idx: Integer,
) {
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
    let c2rust_fresh76 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh76 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: mode },
    };
    let c2rust_fresh77 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh77 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: mode_idx },
    };
    push_call(
        ui,
        b"mode_change\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_bell(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"bell\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_visual_bell(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"visual_bell\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_suspend(mut ui: *mut RemoteUI) {
    push_call(
        ui,
        b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
        noargs.get(),
    );
}
pub unsafe extern "C" fn remote_ui_set_title(mut ui: *mut RemoteUI, mut title: String_0) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh78 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh78 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: title },
    };
    push_call(
        ui,
        b"set_title\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_set_icon(mut ui: *mut RemoteUI, mut icon: String_0) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh79 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh79 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: icon },
    };
    push_call(
        ui,
        b"set_icon\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_screenshot(mut ui: *mut RemoteUI, mut path: String_0) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh80 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh80 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: path },
    };
    push_call(
        ui,
        b"screenshot\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_option_set(
    mut ui: *mut RemoteUI,
    mut name: String_0,
    mut value: Object,
) {
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
    let c2rust_fresh81 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh81 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: name },
    };
    let c2rust_fresh82 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh82 as isize) = value;
    push_call(
        ui,
        b"option_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_chdir(mut ui: *mut RemoteUI, mut path: String_0) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh83 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh83 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: path },
    };
    push_call(ui, b"chdir\0".as_ptr() as *const ::core::ffi::c_char, args);
}
pub unsafe extern "C" fn remote_ui_hl_group_set(
    mut ui: *mut RemoteUI,
    mut name: String_0,
    mut id: Integer,
) {
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
    let c2rust_fresh84 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh84 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: name },
    };
    let c2rust_fresh85 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh85 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: id },
    };
    push_call(
        ui,
        b"hl_group_set\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_msg_set_pos(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut row: Integer,
    mut scrolled: Boolean,
    mut sep_char: String_0,
    mut zindex: Integer,
    mut compindex: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh86 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh86 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh87 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh87 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: row },
    };
    let c2rust_fresh88 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh88 as isize) = object {
        type_0: kObjectTypeBoolean,
        data: C2Rust_Unnamed { boolean: scrolled },
    };
    let c2rust_fresh89 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh89 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed { string: sep_char },
    };
    let c2rust_fresh90 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh90 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: zindex },
    };
    let c2rust_fresh91 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh91 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: compindex },
    };
    push_call(
        ui,
        b"msg_set_pos\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_win_viewport(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut win: Window,
    mut topline: Integer,
    mut botline: Integer,
    mut curline: Integer,
    mut curcol: Integer,
    mut line_count: Integer,
    mut scroll_delta: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 8] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 8];
    args.capacity = 8 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh92 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh92 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh93 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh93 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed {
            integer: win as Integer,
        },
    };
    let c2rust_fresh94 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh94 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: topline },
    };
    let c2rust_fresh95 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh95 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: botline },
    };
    let c2rust_fresh96 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh96 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: curline },
    };
    let c2rust_fresh97 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh97 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: curcol },
    };
    let c2rust_fresh98 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh98 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: line_count,
        },
    };
    let c2rust_fresh99 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh99 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed {
            integer: scroll_delta,
        },
    };
    push_call(
        ui,
        b"win_viewport\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_win_viewport_margins(
    mut ui: *mut RemoteUI,
    mut grid: Integer,
    mut win: Window,
    mut top: Integer,
    mut bottom: Integer,
    mut left: Integer,
    mut right: Integer,
) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 6] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 6];
    args.capacity = 6 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh100 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh100 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: grid },
    };
    let c2rust_fresh101 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh101 as isize) = object {
        type_0: kObjectTypeWindow,
        data: C2Rust_Unnamed {
            integer: win as Integer,
        },
    };
    let c2rust_fresh102 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh102 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: top },
    };
    let c2rust_fresh103 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh103 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: bottom },
    };
    let c2rust_fresh104 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh104 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: left },
    };
    let c2rust_fresh105 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh105 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: right },
    };
    push_call(
        ui,
        b"win_viewport_margins\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub unsafe extern "C" fn remote_ui_error_exit(mut ui: *mut RemoteUI, mut status: Integer) {
    let mut args: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let c2rust_fresh106 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh106 as isize) = object {
        type_0: kObjectTypeInteger,
        data: C2Rust_Unnamed { integer: status },
    };
    push_call(
        ui,
        b"error_exit\0".as_ptr() as *const ::core::ffi::c_char,
        args,
    );
}
pub const CHAN_STDIO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh1 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh1 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh2 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh2 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh10 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh10 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh11 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh11 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh12 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh12 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh13 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh13 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_uint(mut buf: *mut *mut ::core::ffi::c_char, mut val: uint32_t) {
    if val > 0xffff as uint32_t {
        let c2rust_fresh5 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh5 = 0xce as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, val);
    } else if val > 0xff as uint32_t {
        let c2rust_fresh6 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh6 = 0xcd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, val);
    } else if val > 0x7f as uint32_t {
        let c2rust_fresh7 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh7 = 0xcc as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh8 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh8 = val as ::core::ffi::c_char;
    } else {
        let c2rust_fresh9 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh9 = val as ::core::ffi::c_char;
    };
}
#[inline]
unsafe extern "C" fn mpack_bool(mut buf: *mut *mut ::core::ffi::c_char, mut val: bool) {
    let c2rust_fresh61 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh61 = (0xc2 as ::core::ffi::c_int
        | (if val as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        })) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh14 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh14 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh15 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh15 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh16 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh16 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
