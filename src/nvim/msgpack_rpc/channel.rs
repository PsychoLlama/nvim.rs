use crate::src::nvim::api::private::dispatch::{
    handle_nvim_get_mode, handle_nvim_paste, handle_nvim_ui_try_resize,
};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_dict, api_free_object, api_set_error, arena_string, copy_dict,
    cstr_as_string,
};
use crate::src::nvim::api::ui::{remote_ui_disconnect, remote_ui_flush_pending_data};
use crate::src::nvim::channel::channel_close;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::event::multiqueue::{
    event_create_oneshot, multiqueue_empty, multiqueue_new_child, multiqueue_process_events,
    multiqueue_put_event,
};
use crate::src::nvim::event::proc::exit_on_closed_chan;
use crate::src::nvim::event::r#loop::loop_poll_events;
use crate::src::nvim::event::rstream::rstream_start;
use crate::src::nvim::event::wstream::{
    wstream_new_buffer, wstream_release_wbuffer, wstream_write,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    ch_before_blocking_events, channels, main_loop, resize_events, ui_client_attached,
    ui_client_channel_id, ui_client_error_exit,
};
use crate::src::nvim::map::mh_get_uint64_t;
use crate::src::nvim::memory::{
    alloc_block, arena_mem_free, free_block, strequal, xcalloc, xfree, xmalloc, xrealloc,
};
use crate::src::nvim::message::semsg;
use crate::src::nvim::msgpack_rpc::packer::{
    mpack_integer, mpack_object, mpack_object_array, mpack_str,
};
use crate::src::nvim::os::input::input_blocking;
use crate::src::nvim::os::libc::{__assert_fail, abort, snprintf};
use crate::src::nvim::os::time::os_hrtime;
pub use crate::src::nvim::types::{
    ApiDispatchWrapper, Arena, ArenaMem, Array, BoolVarValue, Boolean, Callback, CallbackReader,
    CallbackType, Callback_data as C2Rust_Unnamed_0, ChannelCallFrame, ChannelPart,
    ChannelStreamType, ClientType, Dict, Error, ErrorType, Event, Float, GridLineEvent, Integer,
    InternalState, KeyValuePair, LibuvProc, Loop, LuaRef, MapHash, Map_uint64_t_ptr_t, MessageType,
    MsgpackRpcRequestHandler, MultiQueue, Object, ObjectType, PackerBuffer, PackerBufferFlush,
    Proc, ProcType, PtyProc, RStream, RemoteUI, ScopeDictDictItem, ScopeType, Set_uint64_t,
    SpecialVarValue, StderrState, StdioPair, Stream, String_0, Terminal, UIClientHandler,
    VarLockStatus, VarType, WBuffer, __gid_t, __pthread_internal_list, __pthread_list_t,
    __pthread_mutex_s, __pthread_rwlock_arch_t, __uid_t, argv_callback, blob_T, blobvar_S,
    consumed_blk, dict_T, dictvar_S, float_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_1,
    funccall_T, garray_T, gid_t, hash_T, hashitem_T, hashtab_T, int32_t, int64_t, internal_proc_cb,
    key_value_pair, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T,
    loop_0, loop_0_children as C2Rust_Unnamed_10, mpack_data_t, mpack_node_s, mpack_node_t,
    mpack_parser_t, mpack_sintmax_t, mpack_tokbuf_s, mpack_tokbuf_t, mpack_token_s,
    mpack_token_s_data as C2Rust_Unnamed_20, mpack_token_t, mpack_token_type_t, mpack_uint32_t,
    mpack_uintmax_t, mpack_value_s, mpack_value_t, multiqueue, object,
    object_data as C2Rust_Unnamed, packer_buffer_t, partial_S, partial_T, proc, proc_exit_cb,
    proc_state_cb, proftime_T, pthread_mutex_t, pthread_rwlock_t, ptr_t, queue, rstream, scid_T,
    sctx_T, size_t, ssize_t, stream, stream_close_cb, stream_read_cb,
    stream_uv as C2Rust_Unnamed_12, stream_write_cb, terminal, typval_T, typval_vval_union,
    ufunc_S, ufunc_T, uid_t, uint16_t, uint32_t, uint64_t, uint8_t, uv__io_cb, uv__io_s, uv__io_t,
    uv__queue, uv_alloc_cb, uv_async_cb, uv_async_s, uv_async_s_u as C2Rust_Unnamed_7, uv_async_t,
    uv_buf_t, uv_close_cb, uv_connect_cb, uv_connect_s, uv_connect_t, uv_connection_cb, uv_exit_cb,
    uv_file, uv_gid_t, uv_handle_s, uv_handle_s_u as C2Rust_Unnamed_2, uv_handle_t, uv_handle_type,
    uv_idle_cb, uv_idle_s, uv_idle_s_u as C2Rust_Unnamed_13, uv_idle_t, uv_loop_s,
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
    varnumber_T, wbuffer, wbuffer_data_finalizer, winsize, QUEUE,
};
use crate::src::nvim::ui_client::{ui_client_attach_to_restarted_server, ui_client_event_raw_line};
extern "C" {
    fn arena_finish(arena: *mut Arena) -> ArenaMem;
    fn channel_incref(chan: *mut Channel);
    fn channel_decref(chan: *mut Channel);
    fn channel_info_changed(chan: *mut Channel, new_chan: bool);
    fn unpacker_init(p: *mut Unpacker);
    fn unpacker_teardown(p: *mut Unpacker);
    fn unpacker_advance(p: *mut Unpacker) -> bool;
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
pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kMessageTypeRequest: MessageType = 0;
pub const kMessageTypeUnknown: MessageType = -1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed_16 = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed_16 = -8;
pub const UV_EUNATCH: C2Rust_Unnamed_16 = -49;
pub const UV_ENODATA: C2Rust_Unnamed_16 = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed_16 = -94;
pub const UV_EILSEQ: C2Rust_Unnamed_16 = -84;
pub const UV_EFTYPE: C2Rust_Unnamed_16 = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed_16 = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed_16 = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed_16 = -112;
pub const UV_EMLINK: C2Rust_Unnamed_16 = -31;
pub const UV_ENXIO: C2Rust_Unnamed_16 = -6;
pub const UV_EOF: C2Rust_Unnamed_16 = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed_16 = -4094;
pub const UV_EXDEV: C2Rust_Unnamed_16 = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed_16 = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed_16 = -110;
pub const UV_ESRCH: C2Rust_Unnamed_16 = -3;
pub const UV_ESPIPE: C2Rust_Unnamed_16 = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed_16 = -108;
pub const UV_EROFS: C2Rust_Unnamed_16 = -30;
pub const UV_ERANGE: C2Rust_Unnamed_16 = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed_16 = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed_16 = -93;
pub const UV_EPROTO: C2Rust_Unnamed_16 = -71;
pub const UV_EPIPE: C2Rust_Unnamed_16 = -32;
pub const UV_EPERM: C2Rust_Unnamed_16 = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed_16 = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed_16 = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed_16 = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed_16 = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed_16 = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed_16 = -107;
pub const UV_ENOSYS: C2Rust_Unnamed_16 = -38;
pub const UV_ENOSPC: C2Rust_Unnamed_16 = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed_16 = -92;
pub const UV_ENONET: C2Rust_Unnamed_16 = -64;
pub const UV_ENOMEM: C2Rust_Unnamed_16 = -12;
pub const UV_ENOENT: C2Rust_Unnamed_16 = -2;
pub const UV_ENODEV: C2Rust_Unnamed_16 = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed_16 = -105;
pub const UV_ENFILE: C2Rust_Unnamed_16 = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed_16 = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed_16 = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed_16 = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed_16 = -90;
pub const UV_EMFILE: C2Rust_Unnamed_16 = -24;
pub const UV_ELOOP: C2Rust_Unnamed_16 = -40;
pub const UV_EISDIR: C2Rust_Unnamed_16 = -21;
pub const UV_EISCONN: C2Rust_Unnamed_16 = -106;
pub const UV_EIO: C2Rust_Unnamed_16 = -5;
pub const UV_EINVAL: C2Rust_Unnamed_16 = -22;
pub const UV_EINTR: C2Rust_Unnamed_16 = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed_16 = -113;
pub const UV_EFBIG: C2Rust_Unnamed_16 = -27;
pub const UV_EFAULT: C2Rust_Unnamed_16 = -14;
pub const UV_EEXIST: C2Rust_Unnamed_16 = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed_16 = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed_16 = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed_16 = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed_16 = -103;
pub const UV_ECHARSET: C2Rust_Unnamed_16 = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed_16 = -125;
pub const UV_EBUSY: C2Rust_Unnamed_16 = -16;
pub const UV_EBADF: C2Rust_Unnamed_16 = -9;
pub const UV_EALREADY: C2Rust_Unnamed_16 = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed_16 = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed_16 = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed_16 = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed_16 = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed_16 = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed_16 = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed_16 = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed_16 = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed_16 = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed_16 = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed_16 = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed_16 = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed_16 = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed_16 = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed_16 = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed_16 = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed_16 = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed_16 = -98;
pub const UV_EACCES: C2Rust_Unnamed_16 = -13;
pub const UV_E2BIG: C2Rust_Unnamed_16 = -7;
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
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: C2Rust_Unnamed_21,
    pub is_rpc: bool,
    pub detach: bool,
    pub rpc: RpcState,
    pub term: *mut Terminal,
    pub on_data: CallbackReader,
    pub on_stderr: CallbackReader,
    pub on_exit: Callback,
    pub exit_status: ::core::ffi::c_int,
    pub callback_busy: bool,
    pub callback_scheduled: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: C2Rust_Unnamed_19,
    pub info: Dict,
    pub client_type: ClientType,
}
pub const kClientTypePlugin: ClientType = 4;
pub const kClientTypeHost: ClientType = 3;
pub const kClientTypeEmbedder: ClientType = 2;
pub const kClientTypeUi: ClientType = 1;
pub const kClientTypeMsgpackRpc: ClientType = 5;
pub const kClientTypeRemote: ClientType = 0;
pub const kClientTypeUnknown: ClientType = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_19 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ChannelCallFrame,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub parser: mpack_parser_t,
    pub reader: mpack_tokbuf_t,
    pub read_ptr: *const ::core::ffi::c_char,
    pub read_size: size_t,
    pub ext_buf: [::core::ffi::c_char; 9],
    pub state: ::core::ffi::c_int,
    pub type_0: MessageType,
    pub request_id: uint32_t,
    pub method_name_len: size_t,
    pub handler: MsgpackRpcRequestHandler,
    pub error: Object,
    pub result: Object,
    pub unpack_error: Error,
    pub arena: Arena,
    pub nevents: ::core::ffi::c_int,
    pub ncalls: ::core::ffi::c_int,
    pub ui_handler: UIClientHandler,
    pub grid_line_event: GridLineEvent,
    pub has_grid_line_event: bool,
}
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_21 {
    pub proc: Proc,
    pub uv: LibuvProc,
    pub pty: PtyProc,
    pub socket: RStream,
    pub stdio: StdioPair,
    pub err: StderrState,
    pub internal: InternalState,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RequestEvent {
    pub type_0: MessageType,
    pub channel: *mut Channel,
    pub handler: MsgpackRpcRequestHandler,
    pub args: Array,
    pub request_id: uint32_t,
    pub used_mem: Arena,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut Channel,
    pub init_array: [*mut Channel; 4],
}
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const ARENA_EMPTY: Arena = Arena {
    cur_blk: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    pos: 0 as size_t,
    size: 0 as size_t,
};
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
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
pub const LOGLVL_DBG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_INF: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn find_channel(mut id: uint64_t) -> *mut Channel {
    return map_get_uint64_t_ptr_t(channels.ptr(), id) as *mut Channel;
}
#[inline]
unsafe extern "C" fn channel_instream(mut chan: *mut Channel) -> *mut Stream {
    match (*chan).streamtype as ::core::ffi::c_uint {
        0 => return &raw mut (*chan).stream.proc.in_0,
        1 => return &raw mut (*chan).stream.socket.s,
        2 => return &raw mut (*chan).stream.stdio.out,
        4 | 3 => {
            abort();
        }
        _ => {}
    }
    abort();
}
#[inline]
unsafe extern "C" fn channel_outstream(mut chan: *mut Channel) -> *mut RStream {
    match (*chan).streamtype as ::core::ffi::c_uint {
        0 => return &raw mut (*chan).stream.proc.out,
        1 => return &raw mut (*chan).stream.socket,
        2 => return &raw mut (*chan).stream.stdio.in_0,
        4 | 3 => {
            abort();
        }
        _ => {}
    }
    abort();
}
pub const REQ: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"[request]  \0") };
pub const RES: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"[response] \0") };
pub const NOT: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"[notify]   \0") };
pub const ERR: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"[error]    \0") };
pub const SEND: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"->\0") };
pub const RECV: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"<-\0") };
unsafe extern "C" fn log_request(
    mut dir: *mut ::core::ffi::c_char,
    mut channel_id: uint64_t,
    mut req_id: uint32_t,
    mut name: *const ::core::ffi::c_char,
) {
    logmsg(
        LOGLVL_DBG,
        b"RPC: \0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ::core::ffi::c_int,
        false_0 != 0,
        b"%s %lu: %s id=%u: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        dir,
        channel_id,
        REQ.as_ptr(),
        req_id,
        name,
    );
}
unsafe extern "C" fn log_response(
    mut dir: *mut ::core::ffi::c_char,
    mut channel_id: uint64_t,
    mut kind: *mut ::core::ffi::c_char,
    mut req_id: uint32_t,
) {
    logmsg(
        LOGLVL_DBG,
        b"RPC: \0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ::core::ffi::c_int,
        false_0 != 0,
        b"%s %lu: %s id=%u\n\0".as_ptr() as *const ::core::ffi::c_char,
        dir,
        channel_id,
        kind,
        req_id,
    );
}
unsafe extern "C" fn log_notify(
    mut dir: *mut ::core::ffi::c_char,
    mut channel_id: uint64_t,
    mut name: *const ::core::ffi::c_char,
) {
    logmsg(
        LOGLVL_DBG,
        b"RPC: \0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        -1 as ::core::ffi::c_int,
        false_0 != 0,
        b"%s %lu: %s %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        dir,
        channel_id,
        NOT.as_ptr(),
        name,
    );
}
pub unsafe extern "C" fn rpc_init() {
    ch_before_blocking_events.set(multiqueue_new_child((*main_loop.ptr()).events));
}
#[no_mangle]
pub unsafe extern "C" fn rpc_start(mut channel: *mut Channel) {
    channel_incref(channel);
    (*channel).is_rpc = true_0 != 0;
    let mut rpc: *mut RpcState = &raw mut (*channel).rpc;
    (*rpc).closed = false_0 != 0;
    (*rpc).unpacker = xcalloc(1 as size_t, ::core::mem::size_of::<Unpacker>()) as *mut Unpacker;
    unpacker_init((*rpc).unpacker);
    (*rpc).next_request_id = 1 as uint32_t;
    (*rpc).info = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    (*rpc).call_stack.capacity = 0 as size_t;
    (*rpc).call_stack.size = (*rpc).call_stack.capacity;
    (*rpc).call_stack.items = ::core::ptr::null_mut::<*mut ChannelCallFrame>();
    if (*channel).streamtype as ::core::ffi::c_uint
        != kChannelStreamInternal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut out: *mut RStream = channel_outstream(channel);
        let mut in_0: *mut Stream = channel_instream(channel);
        logmsg(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"rpc_start\0".as_ptr() as *const ::core::ffi::c_char,
            93 as ::core::ffi::c_int,
            true_0 != 0,
            b"rpc ch %lu in-stream=%p out-stream=%p\0".as_ptr() as *const ::core::ffi::c_char,
            (*channel).id,
            in_0 as *mut ::core::ffi::c_void,
            out as *mut ::core::ffi::c_void,
        );
        rstream_start(
            out,
            Some(
                receive_msgpack
                    as unsafe extern "C" fn(
                        *mut RStream,
                        *const ::core::ffi::c_char,
                        size_t,
                        *mut ::core::ffi::c_void,
                        bool,
                    ) -> size_t,
            ),
            channel as *mut ::core::ffi::c_void,
        );
    }
}
unsafe extern "C" fn find_rpc_channel(mut id: uint64_t) -> *mut Channel {
    let mut chan: *mut Channel = find_channel(id);
    if chan.is_null() || !(*chan).is_rpc || (*chan).rpc.closed as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<Channel>();
    }
    return chan;
}
pub unsafe extern "C" fn rpc_send_event(
    mut id: uint64_t,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
) -> bool {
    let mut channel: *mut Channel = ::core::ptr::null_mut::<Channel>();
    if id != 0 && {
        channel = find_rpc_channel(id);
        channel.is_null()
    } {
        return false_0 != 0;
    }
    log_notify(
        SEND.as_ptr() as *mut ::core::ffi::c_char,
        if !channel.is_null() {
            (*channel).id
        } else {
            0 as uint64_t
        },
        name,
    );
    if !channel.is_null() {
        serialize_request(&raw mut channel, 1 as size_t, 0 as uint32_t, name, args);
    } else {
        broadcast_event(name, args);
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn rpc_send_call(
    mut id: uint64_t,
    mut method_name: *const ::core::ffi::c_char,
    mut args: Array,
    mut result_mem: *mut ArenaMem,
    mut err: *mut Error,
) -> Object {
    let mut channel: *mut Channel = ::core::ptr::null_mut::<Channel>();
    channel = find_rpc_channel(id);
    if channel.is_null() {
        api_set_error(
            err,
            kErrorTypeException,
            b"Invalid channel: %lu\0".as_ptr() as *const ::core::ffi::c_char,
            id,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    channel_incref(channel);
    let mut rpc: *mut RpcState = &raw mut (*channel).rpc;
    let c2rust_fresh21 = (*rpc).next_request_id;
    (*rpc).next_request_id = (*rpc).next_request_id.wrapping_add(1);
    let mut request_id: uint32_t = c2rust_fresh21;
    serialize_request(&raw mut channel, 1 as size_t, request_id, method_name, args);
    log_request(
        SEND.as_ptr() as *mut ::core::ffi::c_char,
        (*channel).id,
        request_id,
        method_name,
    );
    let mut frame: ChannelCallFrame = ChannelCallFrame {
        request_id: request_id,
        returned: false_0 != 0,
        errored: false_0 != 0,
        result: object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        result_mem: ::core::ptr::null_mut::<consumed_blk>(),
    };
    if (*rpc).call_stack.size == (*rpc).call_stack.capacity {
        (*rpc).call_stack.capacity = if (*rpc).call_stack.capacity != 0 {
            (*rpc).call_stack.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        (*rpc).call_stack.items = xrealloc(
            (*rpc).call_stack.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*mut ChannelCallFrame>()
                .wrapping_mul((*rpc).call_stack.capacity),
        ) as *mut *mut ChannelCallFrame;
    } else {
    };
    let c2rust_fresh22 = (*rpc).call_stack.size;
    (*rpc).call_stack.size = (*rpc).call_stack.size.wrapping_add(1);
    let c2rust_lvalue_ptr = &raw mut *(*rpc).call_stack.items.offset(c2rust_fresh22 as isize);
    *c2rust_lvalue_ptr = &raw mut frame;
    let mut remaining: int64_t = -1 as int64_t;
    let mut before: uint64_t = if remaining > 0 as int64_t {
        os_hrtime()
    } else {
        0 as uint64_t
    };
    while !(frame.returned as ::core::ffi::c_int != 0 || (*rpc).closed as ::core::ffi::c_int != 0) {
        if !(*channel).events.is_null() && !multiqueue_empty((*channel).events) {
            multiqueue_process_events((*channel).events);
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
    (*rpc).call_stack.size = (*rpc).call_stack.size.wrapping_sub(1);
    if !frame.returned {
        api_set_error(
            err,
            kErrorTypeException,
            b"Invalid channel: %lu\0".as_ptr() as *const ::core::ffi::c_char,
            id,
        );
        channel_decref(channel);
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    if frame.errored {
        if frame.result.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                frame.result.data.string.data,
            );
        } else if frame.result.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut array: Array = frame.result.data.array;
            if array.size == 2 as size_t
                && (*array.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                && ((*array.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .integer
                    == kErrorTypeException as ::core::ffi::c_int as Integer
                    || (*array.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .integer
                        == kErrorTypeValidation as ::core::ffi::c_int as Integer)
                && (*array.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                api_set_error(
                    err,
                    (*array.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .integer as ErrorType,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    (*array.items.offset(1 as ::core::ffi::c_int as isize))
                        .data
                        .string
                        .data,
                );
            } else {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    b"unknown error\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        } else {
            api_set_error(
                err,
                kErrorTypeException,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                b"unknown error\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        arena_mem_free(frame.result_mem);
        frame.result_mem = ::core::ptr::null_mut::<consumed_blk>();
    }
    channel_decref(channel);
    *result_mem = frame.result_mem;
    return if frame.errored as ::core::ffi::c_int != 0 {
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }
    } else {
        frame.result
    };
}
unsafe extern "C" fn receive_msgpack(
    mut stream: *mut RStream,
    mut rbuf: *const ::core::ffi::c_char,
    mut c: size_t,
    mut data: *mut ::core::ffi::c_void,
    mut eof: bool,
) -> size_t {
    let mut channel: *mut Channel = data as *mut Channel;
    channel_incref(channel);
    let mut consumed: size_t = 0 as size_t;
    logmsg(
        LOGLVL_DBG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"receive_msgpack\0".as_ptr() as *const ::core::ffi::c_char,
        211 as ::core::ffi::c_int,
        true_0 != 0,
        b"ch %lu: parsing %zu bytes from msgpack Stream: %p\0".as_ptr()
            as *const ::core::ffi::c_char,
        (*channel).id,
        c,
        stream as *mut ::core::ffi::c_void,
    );
    if c > 0 as size_t {
        let mut p: *mut Unpacker = (*channel).rpc.unpacker;
        (*p).read_ptr = rbuf;
        (*p).read_size = c;
        parse_msgpack(channel);
        if !((*p).state < 0 as ::core::ffi::c_int) {
            consumed = c.wrapping_sub((*p).read_size);
        }
    }
    if eof {
        let mut buf: [::core::ffi::c_char; 256] = [0; 256];
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"ch %lu was closed by the peer\0".as_ptr() as *const ::core::ffi::c_char,
            (*channel).id,
        );
        chan_close_on_err(
            channel,
            &raw mut buf as *mut ::core::ffi::c_char,
            LOGLVL_INF,
        );
    }
    channel_decref(channel);
    return consumed;
}
unsafe extern "C" fn find_call_frame(
    mut rpc: *mut RpcState,
    mut request_id: uint32_t,
) -> *mut ChannelCallFrame {
    let mut i: size_t = 0 as size_t;
    while i < (*rpc).call_stack.size {
        let mut frame: *mut ChannelCallFrame = *(*rpc).call_stack.items.offset(
            (*rpc)
                .call_stack
                .size
                .wrapping_sub(i)
                .wrapping_sub(1 as size_t) as isize,
        );
        if (*frame).request_id == request_id {
            return frame;
        }
        i = i.wrapping_add(1);
    }
    return ::core::ptr::null_mut::<ChannelCallFrame>();
}
unsafe extern "C" fn parse_msgpack(mut channel: *mut Channel) {
    let mut p: *mut Unpacker = (*channel).rpc.unpacker;
    while unpacker_advance(p) {
        if (*p).type_0 as ::core::ffi::c_int == kMessageTypeRedrawEvent as ::core::ffi::c_int {
            if ui_client_attached.get() {
                if (*p).has_grid_line_event {
                    ui_client_event_raw_line(&raw mut (*p).grid_line_event);
                    (*p).has_grid_line_event = false_0 != 0;
                } else if (*p).ui_handler.fn_0.is_some()
                    && (*p).result.type_0 as ::core::ffi::c_uint
                        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*p).ui_handler.fn_0.expect("non-null function pointer")(
                        (*p).result.data.array,
                    );
                }
            }
            arena_mem_free(arena_finish(&raw mut (*p).arena));
        } else if (*p).type_0 as ::core::ffi::c_int == kMessageTypeResponse as ::core::ffi::c_int {
            let mut frame: *mut ChannelCallFrame = if (*channel).rpc.client_type
                as ::core::ffi::c_int
                == kClientTypeMsgpackRpc as ::core::ffi::c_int
            {
                find_call_frame(&raw mut (*channel).rpc, (*p).request_id)
            } else {
                *(*channel).rpc.call_stack.items.offset(
                    (*channel)
                        .rpc
                        .call_stack
                        .size
                        .wrapping_sub(0 as size_t)
                        .wrapping_sub(1 as size_t) as isize,
                )
            };
            if frame.is_null() || (*p).request_id != (*frame).request_id {
                let mut buf: [::core::ffi::c_char; 256] = [0; 256];
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
                    b"ch %lu (type=%u) returned a response with an unknown request id %u. Ensure the client is properly synchronized\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    (*channel).id,
                    (*channel).rpc.client_type as ::core::ffi::c_uint,
                    (*p).request_id,
                );
                chan_close_on_err(
                    channel,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    LOGLVL_ERR,
                );
                return;
            }
            (*frame).returned = true_0 != 0;
            (*frame).errored = (*p).error.type_0 as ::core::ffi::c_uint
                != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint;
            if (*frame).errored {
                (*frame).result = (*p).error;
            } else {
                (*frame).result = (*p).result;
            }
            (*frame).result_mem = arena_finish(&raw mut (*p).arena);
            log_response(
                RECV.as_ptr() as *mut ::core::ffi::c_char,
                (*channel).id,
                (if (*frame).errored as ::core::ffi::c_int != 0 {
                    ERR.as_ptr()
                } else {
                    RES.as_ptr()
                }) as *mut ::core::ffi::c_char,
                (*p).request_id,
            );
        } else {
            if (*p).type_0 as ::core::ffi::c_int == kMessageTypeNotification as ::core::ffi::c_int {
                log_notify(
                    RECV.as_ptr() as *mut ::core::ffi::c_char,
                    (*channel).id,
                    (*p).handler.name,
                );
            } else {
                log_request(
                    RECV.as_ptr() as *mut ::core::ffi::c_char,
                    (*channel).id,
                    (*p).request_id,
                    (*p).handler.name,
                );
            }
            let mut res: Object = (*p).result;
            if (*p).result.type_0 as ::core::ffi::c_uint
                != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                chan_close_on_err(
                    channel,
                    b"msgpack-rpc request args must be an array\0".as_ptr()
                        as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    LOGLVL_ERR,
                );
                return;
            }
            let mut arg: Array = res.data.array;
            handle_request(channel, p, arg);
        }
    }
    if (*p).state < 0 as ::core::ffi::c_int {
        chan_close_on_err(channel, (*p).unpack_error.msg, LOGLVL_INF);
        api_clear_error(&raw mut (*p).unpack_error);
    }
}
unsafe extern "C" fn handle_request(
    mut channel: *mut Channel,
    mut p: *mut Unpacker,
    mut args: Array,
) {
    '_c2rust_label: {
        if (*p).type_0 as ::core::ffi::c_int == kMessageTypeRequest as ::core::ffi::c_int
            || (*p).type_0 as ::core::ffi::c_int == kMessageTypeNotification as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"p->type == kMessageTypeRequest || p->type == kMessageTypeNotification\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/msgpack_rpc/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                311 as ::core::ffi::c_uint,
                b"void handle_request(Channel *, Unpacker *, Array)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if (*p).handler.fn_0.is_none() {
        send_error(
            channel,
            (*p).handler,
            (*p).type_0,
            (*p).request_id,
            (*p).unpack_error.msg,
        );
        api_clear_error(&raw mut (*p).unpack_error);
        arena_mem_free(arena_finish(&raw mut (*p).arena));
        return;
    }
    let mut evdata: *mut RequestEvent =
        xmalloc(::core::mem::size_of::<RequestEvent>()) as *mut RequestEvent;
    (*evdata).type_0 = (*p).type_0;
    (*evdata).channel = channel;
    (*evdata).handler = (*p).handler;
    (*evdata).args = args;
    (*evdata).used_mem = (*p).arena;
    (*p).arena = ARENA_EMPTY;
    (*evdata).request_id = (*p).request_id;
    channel_incref(channel);
    if (*p).handler.fast {
        let mut is_get_mode: bool = (*p).handler.fn_0
            == Some(
                handle_nvim_get_mode
                    as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
            );
        if is_get_mode as ::core::ffi::c_int != 0 && !input_blocking() {
            multiqueue_put_event(
                ch_before_blocking_events.get(),
                Event {
                    handler: Some(
                        request_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        evdata as *mut ::core::ffi::c_void,
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
            request_event(&raw mut evdata as *mut *mut ::core::ffi::c_void);
        }
    } else {
        let mut is_resize: bool = (*p).handler.fn_0
            == Some(
                handle_nvim_ui_try_resize
                    as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
            );
        if is_resize {
            let mut ev: Event = event_create_oneshot(
                Event {
                    handler: Some(
                        request_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        evdata as *mut ::core::ffi::c_void,
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
                2 as ::core::ffi::c_int,
            );
            multiqueue_put_event((*channel).events, ev);
            multiqueue_put_event(resize_events.get(), ev);
        } else {
            multiqueue_put_event(
                (*channel).events,
                Event {
                    handler: Some(
                        request_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        evdata as *mut ::core::ffi::c_void,
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
            logmsg(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"handle_request\0".as_ptr() as *const ::core::ffi::c_char,
                347 as ::core::ffi::c_int,
                true_0 != 0,
                b"RPC: scheduled %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                (*p).method_name_len as ::core::ffi::c_int,
                (*p).handler.name,
            );
        }
    };
}
unsafe extern "C" fn request_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut result: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut e: *mut RequestEvent =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut RequestEvent;
    let mut channel: *mut Channel = (*e).channel;
    let mut handler: MsgpackRpcRequestHandler = (*e).handler;
    let mut error: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if !(*channel).rpc.closed {
        result = handler.fn_0.expect("non-null function pointer")(
            (*channel).id,
            (*e).args,
            &raw mut (*e).used_mem,
            &raw mut error,
        );
        if (*e).type_0 as ::core::ffi::c_int == kMessageTypeRequest as ::core::ffi::c_int
            || error.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
        {
            serialize_response(
                channel,
                (*e).handler,
                (*e).type_0,
                (*e).request_id,
                &raw mut error,
                &raw mut result,
            );
        }
        if handler.ret_alloc {
            api_free_object(result);
        }
    }
    arena_mem_free(arena_finish(&raw mut (*e).used_mem));
    channel_decref(channel);
    xfree(e as *mut ::core::ffi::c_void);
    api_clear_error(&raw mut error);
}
pub unsafe extern "C" fn rpc_write_raw(mut id: uint64_t, mut buffer: *mut WBuffer) -> bool {
    let mut channel: *mut Channel = find_rpc_channel(id);
    if channel.is_null() {
        wstream_release_wbuffer(buffer);
        return false_0 != 0;
    }
    return channel_write(channel, buffer);
}
unsafe extern "C" fn channel_write(mut channel: *mut Channel, mut buffer: *mut WBuffer) -> bool {
    let mut err: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*channel).rpc.closed {
        wstream_release_wbuffer(buffer);
        return false_0 != 0;
    }
    if (*channel).streamtype as ::core::ffi::c_uint
        == kChannelStreamInternal as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        channel_incref(channel);
        if !(*channel).events.is_null() {
            multiqueue_put_event(
                (*channel).events,
                Event {
                    handler: Some(
                        internal_read_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        channel as *mut ::core::ffi::c_void,
                        buffer as *mut ::core::ffi::c_void,
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
            let mut argv: [*mut ::core::ffi::c_void; 2] = [
                channel as *mut ::core::ffi::c_void,
                buffer as *mut ::core::ffi::c_void,
            ];
            internal_read_event(&raw mut argv as *mut *mut ::core::ffi::c_void);
        }
    } else {
        let mut in_0: *mut Stream = channel_instream(channel);
        err = wstream_write(in_0, buffer);
    }
    if err != 0 as ::core::ffi::c_int {
        let mut buf: [::core::ffi::c_char; 256] = [0; 256];
        snprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>(),
            b"ch %lu: stream write failed: %s. RPC canceled; closing channel\0".as_ptr()
                as *const ::core::ffi::c_char,
            (*channel).id,
            uv_strerror(err),
        );
        chan_close_on_err(
            channel,
            &raw mut buf as *mut ::core::ffi::c_char,
            if err == UV_EPIPE as ::core::ffi::c_int {
                LOGLVL_INF
            } else {
                LOGLVL_ERR
            },
        );
    }
    return err == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn internal_read_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut channel: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    let mut buffer: *mut WBuffer = *argv.offset(1 as ::core::ffi::c_int as isize) as *mut WBuffer;
    let mut p: *mut Unpacker = (*channel).rpc.unpacker;
    (*p).read_ptr = (*buffer).data;
    (*p).read_size = (*buffer).size;
    parse_msgpack(channel);
    if (*p).read_size != 0 {
        if !(*channel).rpc.closed {
            chan_close_on_err(
                channel,
                b"internal channel: internal error\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                LOGLVL_ERR,
            );
        }
    }
    channel_decref(channel);
    wstream_release_wbuffer(buffer);
}
unsafe extern "C" fn send_error(
    mut chan: *mut Channel,
    mut handler: MsgpackRpcRequestHandler,
    mut type_0: MessageType,
    mut id: uint32_t,
    mut err: *mut ::core::ffi::c_char,
) {
    let mut e: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    api_set_error(
        &raw mut e,
        kErrorTypeException,
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        err,
    );
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    serialize_response(
        chan,
        handler,
        type_0,
        id,
        &raw mut e,
        &raw mut c2rust_lvalue,
    );
    api_clear_error(&raw mut e);
}
unsafe extern "C" fn broadcast_event(mut name: *const ::core::ffi::c_char, mut args: Array) {
    let mut chans: C2Rust_Unnamed_22 = C2Rust_Unnamed_22 {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<*mut Channel>(),
        init_array: [::core::ptr::null_mut::<Channel>(); 4],
    };
    chans.capacity = ::core::mem::size_of::<[*mut Channel; 4]>()
        .wrapping_div(::core::mem::size_of::<*mut Channel>())
        .wrapping_div(
            (::core::mem::size_of::<[*mut Channel; 4]>()
                .wrapping_rem(::core::mem::size_of::<*mut Channel>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    chans.size = 0 as size_t;
    chans.items = &raw mut chans.init_array as *mut *mut Channel;
    let mut channel: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*channels.ptr()).set.h.n_keys {
        channel = *(*channels.ptr()).values.offset(__i as isize) as *mut Channel;
        if (*channel).is_rpc {
            if chans.size == chans.capacity {
                chans.capacity = if chans.capacity != 0 {
                    chans.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                chans.items = xrealloc(
                    chans.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<*mut Channel>().wrapping_mul(chans.capacity),
                ) as *mut *mut Channel;
            } else {
            };
            let c2rust_fresh20 = chans.size;
            chans.size = chans.size.wrapping_add(1);
            let c2rust_lvalue_ptr = &raw mut *chans.items.offset(c2rust_fresh20 as isize);
            *c2rust_lvalue_ptr = channel;
        }
        __i = __i.wrapping_add(1);
    }
    if chans.size != 0 {
        serialize_request(chans.items, chans.size, 0 as uint32_t, name, args);
    }
    if chans.items != &raw mut chans.init_array as *mut *mut Channel {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut chans.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpc_close(mut channel: *mut Channel) {
    if (*channel).rpc.closed {
        return;
    }
    (*channel).rpc.closed = true_0 != 0;
    multiqueue_put_event(
        (*main_loop.ptr()).fast_events,
        Event {
            handler: Some(
                rpc_close_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                channel as *mut ::core::ffi::c_void,
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
unsafe extern "C" fn rpc_close_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut channel: *mut Channel = *argv.offset(0 as ::core::ffi::c_int as isize) as *mut Channel;
    '_c2rust_label: {
        if !channel.is_null() {
        } else {
            __assert_fail(
                b"channel\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/msgpack_rpc/channel.rs\0".as_ptr() as *const ::core::ffi::c_char,
                493 as ::core::ffi::c_uint,
                b"void rpc_close_event(void **)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    channel_decref(channel);
    remote_ui_disconnect(
        (*channel).id,
        ::core::ptr::null_mut::<Error>(),
        false_0 != 0,
    );
    let mut is_ui_client: bool =
        ui_client_channel_id.get() != 0 && (*channel).id == ui_client_channel_id.get();
    if is_ui_client {
        ui_client_attach_to_restarted_server();
        if ui_client_channel_id.get() != (*channel).id {
            return;
        }
        if (*channel).streamtype as ::core::ffi::c_uint
            == kChannelStreamProc as ::core::ffi::c_int as ::core::ffi::c_uint
            && ui_client_error_exit.get() < 0 as ::core::ffi::c_int
        {
            return;
        }
        exit_on_closed_chan(0 as ::core::ffi::c_int);
    } else if (*channel).streamtype as ::core::ffi::c_uint
        == kChannelStreamStdio as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*channel).detach
    {
        exit_on_closed_chan(0 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn rpc_free(mut channel: *mut Channel) {
    unpacker_teardown((*channel).rpc.unpacker);
    xfree((*channel).rpc.unpacker as *mut ::core::ffi::c_void);
    xfree((*channel).rpc.call_stack.items as *mut ::core::ffi::c_void);
    (*channel).rpc.call_stack.capacity = 0 as size_t;
    (*channel).rpc.call_stack.size = (*channel).rpc.call_stack.capacity;
    (*channel).rpc.call_stack.items = ::core::ptr::null_mut::<*mut ChannelCallFrame>();
    api_free_dict((*channel).rpc.info);
}
unsafe extern "C" fn chan_close_on_err(
    mut channel: *mut Channel,
    mut msg: *mut ::core::ffi::c_char,
    mut loglevel: ::core::ffi::c_int,
) {
    let mut i: size_t = 0 as size_t;
    while i < (*channel).rpc.call_stack.size {
        let mut frame: *mut ChannelCallFrame = *(*channel).rpc.call_stack.items.offset(i as isize);
        if !(*frame).returned {
            (*frame).returned = true_0 != 0;
            (*frame).errored = true_0 != 0;
            (*frame).result = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        &raw mut (*(*channel).rpc.unpacker).arena,
                        cstr_as_string(msg),
                    ),
                },
            };
            (*frame).result_mem = arena_finish(&raw mut (*(*channel).rpc.unpacker).arena);
        }
        i = i.wrapping_add(1);
    }
    channel_close(
        (*channel).id,
        kChannelPartRpc,
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
    );
    logmsg(
        loglevel,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"chan_close_on_err\0".as_ptr() as *const ::core::ffi::c_char,
        545 as ::core::ffi::c_int,
        true_0 != 0,
        b"RPC: %s\0".as_ptr() as *const ::core::ffi::c_char,
        msg,
    );
}
unsafe extern "C" fn serialize_request(
    mut chans: *mut *mut Channel,
    mut nchans: size_t,
    mut request_id: uint32_t,
    mut method: *const ::core::ffi::c_char,
    mut args: Array,
) {
    let mut packer: PackerBuffer = PackerBuffer {
        startptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        endptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        anydata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        anyint: 0,
        packer_flush: None,
    };
    packer_buffer_init_channels(chans, nchans, &raw mut packer);
    mpack_array(
        &raw mut packer.ptr,
        (if request_id != 0 {
            4 as ::core::ffi::c_int
        } else {
            3 as ::core::ffi::c_int
        }) as uint32_t,
    );
    let c2rust_fresh19 = packer.ptr;
    packer.ptr = packer.ptr.offset(1);
    *c2rust_fresh19 = (if request_id != 0 {
        0 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    if request_id != 0 {
        mpack_uint(&raw mut packer.ptr, request_id);
    }
    mpack_str(cstr_as_string(method), &raw mut packer);
    mpack_object_array(args, &raw mut packer);
    packer_buffer_finish_channels(&raw mut packer);
}
pub unsafe extern "C" fn serialize_response(
    mut channel: *mut Channel,
    mut handler: MsgpackRpcRequestHandler,
    mut type_0: MessageType,
    mut response_id: uint32_t,
    mut err: *mut Error,
    mut arg: *mut Object,
) {
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
        && type_0 as ::core::ffi::c_int == kMessageTypeNotification as ::core::ffi::c_int
    {
        if handler.fn_0
            == Some(
                handle_nvim_paste
                    as unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
            )
        {
            semsg(
                b"paste: %s\0".as_ptr() as *const ::core::ffi::c_char,
                (*err).msg,
            );
            api_clear_error(err);
        } else {
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
            let c2rust_fresh0 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*err).type_0 as Integer,
                },
            };
            let c2rust_fresh1 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh1 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*err).msg),
                },
            };
            serialize_request(
                &raw mut channel,
                1 as size_t,
                0 as uint32_t,
                b"nvim_error_event\0".as_ptr() as *const ::core::ffi::c_char,
                args,
            );
        }
        return;
    }
    let mut packer: PackerBuffer = PackerBuffer {
        startptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        endptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        anydata: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        anyint: 0,
        packer_flush: None,
    };
    packer_buffer_init_channels(&raw mut channel, 1 as size_t, &raw mut packer);
    mpack_array(&raw mut packer.ptr, 4 as uint32_t);
    let c2rust_fresh2 = packer.ptr;
    packer.ptr = packer.ptr.offset(1);
    *c2rust_fresh2 = 1 as ::core::ffi::c_int as ::core::ffi::c_char;
    mpack_uint(&raw mut packer.ptr, response_id);
    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        mpack_array(&raw mut packer.ptr, 2 as uint32_t);
        mpack_integer(&raw mut packer.ptr, (*err).type_0 as Integer);
        mpack_str(cstr_as_string((*err).msg), &raw mut packer);
        let c2rust_fresh3 = packer.ptr;
        packer.ptr = packer.ptr.offset(1);
        *c2rust_fresh3 = 0xc0 as ::core::ffi::c_int as ::core::ffi::c_char;
    } else {
        let c2rust_fresh4 = packer.ptr;
        packer.ptr = packer.ptr.offset(1);
        *c2rust_fresh4 = 0xc0 as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_object(arg, &raw mut packer);
    }
    packer_buffer_finish_channels(&raw mut packer);
    log_response(
        SEND.as_ptr() as *mut ::core::ffi::c_char,
        (*channel).id,
        (if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            ERR.as_ptr()
        } else {
            RES.as_ptr()
        }) as *mut ::core::ffi::c_char,
        response_id,
    );
}
unsafe extern "C" fn packer_buffer_init_channels(
    mut chans: *mut *mut Channel,
    mut nchans: size_t,
    mut packer: *mut PackerBuffer,
) {
    let mut i: size_t = 0 as size_t;
    while i < nchans {
        let mut chan: *mut Channel = *chans.offset(i as isize);
        if !(*chan).rpc.ui.is_null()
            && (*(*chan).rpc.ui).incomplete_event as ::core::ffi::c_int != 0
        {
            remote_ui_flush_pending_data((*chan).rpc.ui);
        }
        i = i.wrapping_add(1);
    }
    (*packer).startptr = alloc_block() as *mut ::core::ffi::c_char;
    (*packer).ptr = (*packer).startptr;
    (*packer).endptr = (*packer).startptr.offset(ARENA_BLOCK_SIZE as isize);
    (*packer).packer_flush =
        Some(channel_flush_callback as unsafe extern "C" fn(*mut PackerBuffer) -> ())
            as PackerBufferFlush;
    (*packer).anydata = chans as *mut ::core::ffi::c_void;
    (*packer).anyint = nchans as int64_t;
}
unsafe extern "C" fn packer_buffer_finish_channels(mut packer: *mut PackerBuffer) {
    let mut len: size_t = (*packer).ptr.offset_from((*packer).startptr) as size_t;
    if len > 0 as size_t {
        let mut buf: *mut WBuffer = wstream_new_buffer(
            (*packer).startptr,
            len,
            (*packer).anyint as size_t,
            Some(free_block as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        );
        let mut chans: *mut *mut Channel = (*packer).anydata as *mut *mut Channel;
        let mut i: int64_t = 0 as int64_t;
        while i < (*packer).anyint {
            channel_write(*chans.offset(i as isize), buf);
            i += 1;
        }
    } else {
        free_block((*packer).startptr as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn channel_flush_callback(mut packer: *mut PackerBuffer) {
    packer_buffer_finish_channels(packer);
    packer_buffer_init_channels(
        (*packer).anydata as *mut *mut Channel,
        (*packer).anyint as size_t,
        packer,
    );
}
pub unsafe extern "C" fn rpc_set_client_info(mut id: uint64_t, mut info: Dict) {
    let mut chan: *mut Channel = find_rpc_channel(id);
    if chan.is_null() {
        abort();
    }
    api_free_dict((*chan).rpc.info);
    (*chan).rpc.info = info;
    let mut type_0: *const ::core::ffi::c_char =
        get_client_info(chan, b"type\0".as_ptr() as *const ::core::ffi::c_char);
    if type_0.is_null()
        || strequal(type_0, b"remote\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        (*chan).rpc.client_type = kClientTypeRemote;
    } else if strequal(
        type_0,
        b"msgpack-rpc\0".as_ptr() as *const ::core::ffi::c_char,
    ) {
        (*chan).rpc.client_type = kClientTypeMsgpackRpc;
    } else if strequal(type_0, b"ui\0".as_ptr() as *const ::core::ffi::c_char) {
        (*chan).rpc.client_type = kClientTypeUi;
    } else if strequal(type_0, b"embedder\0".as_ptr() as *const ::core::ffi::c_char) {
        (*chan).rpc.client_type = kClientTypeEmbedder;
    } else if strequal(type_0, b"host\0".as_ptr() as *const ::core::ffi::c_char) {
        (*chan).rpc.client_type = kClientTypeHost;
    } else if strequal(type_0, b"plugin\0".as_ptr() as *const ::core::ffi::c_char) {
        (*chan).rpc.client_type = kClientTypePlugin;
    } else {
        (*chan).rpc.client_type = kClientTypeUnknown;
    }
    channel_info_changed(chan, false_0 != 0);
}
pub unsafe extern "C" fn rpc_client_info(mut chan: *mut Channel) -> Dict {
    return copy_dict((*chan).rpc.info, ::core::ptr::null_mut::<Arena>());
}
#[no_mangle]
pub unsafe extern "C" fn get_client_info(
    mut chan: *mut Channel,
    mut key: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    if !(*chan).is_rpc {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    let mut info: Dict = (*chan).rpc.info;
    let mut i: size_t = 0 as size_t;
    while i < info.size {
        if strequal(key, (*info.items.offset(i as isize)).key.data) as ::core::ffi::c_int != 0
            && (*info.items.offset(i as isize)).value.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*info.items.offset(i as isize)).value.data.string.data;
        }
        i = i.wrapping_add(1);
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[inline]
unsafe extern "C" fn mpack_w2(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh12 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh12 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh13 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh13 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_w4(mut b: *mut *mut ::core::ffi::c_char, mut v: uint32_t) {
    let c2rust_fresh8 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh8 = (v >> 24 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh9 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh9 = (v >> 16 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh10 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh10 = (v >> 8 as ::core::ffi::c_int & 0xff as uint32_t) as ::core::ffi::c_char;
    let c2rust_fresh11 = *b;
    *b = (*b).offset(1);
    *c2rust_fresh11 = (v & 0xff as uint32_t) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn mpack_uint(mut buf: *mut *mut ::core::ffi::c_char, mut val: uint32_t) {
    if val > 0xffff as uint32_t {
        let c2rust_fresh14 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh14 = 0xce as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, val);
    } else if val > 0xff as uint32_t {
        let c2rust_fresh15 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh15 = 0xcd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, val);
    } else if val > 0x7f as uint32_t {
        let c2rust_fresh16 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh16 = 0xcc as ::core::ffi::c_int as ::core::ffi::c_char;
        let c2rust_fresh17 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh17 = val as ::core::ffi::c_char;
    } else {
        let c2rust_fresh18 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh18 = val as ::core::ffi::c_char;
    };
}
#[inline]
unsafe extern "C" fn mpack_array(mut buf: *mut *mut ::core::ffi::c_char, mut len: uint32_t) {
    if len < 0x10 as uint32_t {
        let c2rust_fresh5 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh5 = (0x90 as uint32_t | len) as ::core::ffi::c_char;
    } else if len < 0x10000 as uint32_t {
        let c2rust_fresh6 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh6 = 0xdc as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w2(buf, len);
    } else {
        let c2rust_fresh7 = *buf;
        *buf = (*buf).offset(1);
        *c2rust_fresh7 = 0xdd as ::core::ffi::c_int as ::core::ffi::c_char;
        mpack_w4(buf, len);
    };
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
