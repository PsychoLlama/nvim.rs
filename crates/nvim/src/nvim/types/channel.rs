#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use core::ffi::{c_char, c_int};

use super::*;

/// A channel's decoder. Zero-initialised by its owner, so every field has to
/// mean something sensible as all-zero bytes.
///
/// The layout is pinned: `test/unit/msgpack_spec.lua` allocates one with
/// `ffi.sizeof` and drives it by writing `read_ptr`/`read_size` directly.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub parser: mpack_parser_t,
    pub reader: mpack_tokbuf_t,
    pub read_ptr: *const c_char,
    pub read_size: size_t,
    /// Accumulates an extension object's payload, which arrives in chunks.
    pub ext_buf: [c_char; 9],
    pub state: c_int,
    pub type_0: MessageType,
    pub request_id: uint32_t,
    pub method_name_len: size_t,
    pub handler: MsgpackRpcRequestHandler,
    pub error: Object,
    pub result: Object,
    pub unpack_error: Error,
    pub arena: Arena,
    pub nevents: c_int,
    pub ncalls: c_int,
    pub ui_handler: UIClientHandler,
    pub grid_line_event: GridLineEvent,
    pub has_grid_line_event: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CallbackReader {
    pub cb: Callback,
    pub self_0: *mut dict_T,
    pub buffer: garray_T,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    pub type_0: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Channel {
    pub id: uint64_t,
    pub refcount: size_t,
    pub events: *mut MultiQueue,
    pub streamtype: ChannelStreamType,
    pub stream: Channel_stream,
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
pub struct ChannelCallFrame {
    pub request_id: uint32_t,
    pub returned: bool,
    pub errored: bool,
    pub result: Object,
    pub result_mem: ArenaMem,
}
pub type ChannelPart = ::core::ffi::c_uint;
pub type ChannelStdinMode = ::core::ffi::c_uint;
pub type ChannelStreamType = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub union Channel_stream {
    pub proc: Proc,
    pub uv: LibuvProc,
    pub pty: PtyProc,
    pub socket: RStream,
    pub stdio: StdioPair,
    pub err: StderrState,
    pub internal: InternalState,
}
pub type ClientType = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InternalState {
    pub cb: LuaRef,
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState {
    pub closed: bool,
    pub unpacker: *mut Unpacker,
    pub ui: *mut RemoteUI,
    pub next_request_id: uint32_t,
    pub call_stack: RpcState_call_stack,
    pub info: Dict,
    pub client_type: ClientType,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RpcState_call_stack {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ChannelCallFrame,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StderrState {
    pub closed: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StdioPair {
    pub in_0: RStream,
    pub out: Stream,
}
