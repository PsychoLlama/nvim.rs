#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Incremental msgpack-rpc decoding.
//!
//! The unpacker is fed whatever bytes have arrived and resumes wherever it
//! stopped, so it must never advance past a point it cannot recover from: the
//! read cursor is only committed once a stage has been decoded in full. See
//! [`protocol`] for the stages.
//!
//! Message bodies go through libmpack's tree parser, with [`api_parse_enter`]
//! building `Object`s in an arena as the tokens arrive. `redraw`
//! notifications from a UI server bypass that — a `grid_line` event would
//! otherwise allocate an object per screen cell — and are decoded a token at
//! a time straight into the shared line buffers.

use core::ffi::{c_char, c_int, c_void};

use crate::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::api::private::helpers::api_set_error;
use crate::memory::{ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free};
use crate::mpack::conv::{
    mpack_unpack_boolean, mpack_unpack_float_fast, mpack_unpack_sint, mpack_unpack_uint,
};
use crate::mpack::mpack_core::{mpack_read, mpack_rtoken, mpack_tokbuf_init};
use crate::mpack::object::{mpack_parse, mpack_parser_init};
use crate::narrow::msgpack_uint_as_u32;
use crate::types::{
    Arena, Array, Dict, Error, Integer, KeyValuePair, MessageType, Object, ObjectType, String_0,
    Unpacker, kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger,
    kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, mpack_node_t, mpack_parser_t,
    mpack_token_t, mpack_uint32_t, object_data, size_t,
};
use crate::ui_client::handle_ui_client_redraw;
use ::libc::abort;

pub mod keydict;
pub mod protocol;
pub mod redraw;

pub use keydict::{
    push_additional_data, unpack_array, unpack_integer, unpack_keydict, unpack_skip, unpack_string,
};

use redraw::unpacker_parse_redraw;

use protocol::{
    ARRAY as TOKEN_ARRAY, BIN as TOKEN_BIN, BOOLEAN as TOKEN_BOOLEAN, CHUNK as TOKEN_CHUNK,
    EXT as TOKEN_EXT, FLOAT as TOKEN_FLOAT, MAP as TOKEN_MAP, NIL as TOKEN_NIL, SINT as TOKEN_SINT,
    STR as TOKEN_STR, UINT as TOKEN_UINT,
};

pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kUnpackTypeStringArray: c_int = -1;

/// The value kinds a generated keyset field can hold. Three are `ObjectType`
/// values; the fourth is the keyset layer's own.
mod field_type {
    use core::ffi::c_int;

    pub const BOOLEAN: c_int = 1;
    pub const INTEGER: c_int = 2;
    pub const STRING: c_int = 4;
    pub const STRING_ARRAY: c_int = super::kUnpackTypeStringArray;
}

/// libmpack's parse results: ok, ran out of input, malformed, too deep.
///
/// `c_int` because that is what `mpack_parse` returns them in; upstream
/// declares the enum unsigned and then compares it against a signed result at
/// every site, which is the only reason the transpile had a cast here.
pub const MPACK_OK: c_int = 0;
pub const MPACK_EOF: c_int = 1;
pub const MPACK_ERROR: c_int = 2;
pub const MPACK_NOMEM: c_int = 3;

/// Decodes one complete object from `data`.
///
/// Unlike the streaming path this needs the whole value up front; anything
/// left over is an error rather than the start of the next message.
///
/// # Safety
/// `data` points at `size` readable bytes, and `arena`/`err` at a writable
/// `Arena` and `Error`.
pub unsafe extern "C" fn unpack(
    mut data: *const c_char,
    mut size: size_t,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    // SAFETY: the caller's buffer, arena and error slot. `api_parse_enter`
    // navigates back here through the parser's `data` field, so every access
    // below has to go through the same pointer it is handed — writing to
    // `unpacker` by name would strip the callback's permission to read what
    // it wrote.
    let (result, value) = unsafe {
        let mut unpacker: Unpacker = core::mem::zeroed();
        let p: *mut Unpacker = &raw mut unpacker;
        mpack_parser_init(&raw mut (*p).parser, 0);
        (*p).parser.data.p = p.cast::<c_void>();
        (*p).arena = *arena;

        let result = mpack_parse(
            &raw mut (*p).parser,
            &raw mut data,
            &raw mut size,
            Some(api_parse_enter),
            Some(parse_nop),
        );

        *arena = (*p).arena;
        (result, (*p).result)
    };

    let message = if result == MPACK_NOMEM {
        c"object was too deep to unpack"
    } else if result == MPACK_EOF {
        c"incomplete msgpack string"
    } else if result == MPACK_ERROR {
        c"invalid msgpack string"
    } else if result == MPACK_OK && size != 0 {
        c"trailing data in msgpack string"
    } else {
        return value;
    };
    // SAFETY: the caller's error slot, and a static message.
    unsafe { api_set_error(err, kErrorTypeException, message.as_ptr()) };
    value
}

/// The `Object` a scalar token stands for, or `None` when the token opens a
/// container, a payload or a chunk instead.
///
/// Pure: a token is a value, and building a union is safe — only reading one
/// back is not.
pub(super) fn scalar_object(tok: mpack_token_t) -> Option<Object> {
    let (type_0, data) = match tok.type_0 {
        TOKEN_NIL => return Some(nil_object()),
        TOKEN_BOOLEAN => (
            kObjectTypeBoolean,
            object_data {
                boolean: mpack_unpack_boolean(tok),
            },
        ),
        TOKEN_SINT | TOKEN_UINT => (
            kObjectTypeInteger,
            object_data {
                integer: unpack_integer_token(tok).expect("token is an integer"),
            },
        ),
        TOKEN_FLOAT => (
            kObjectTypeFloat,
            object_data {
                floating: mpack_unpack_float_fast(tok),
            },
        ),
        _ => return None,
    };
    Some(Object { type_0, data })
}

/// An array `Object` over `capacity` slots that have already been allocated.
fn array_object(items: *mut Object, capacity: size_t) -> Object {
    Object {
        type_0: kObjectTypeArray,
        data: object_data {
            array: Array {
                size: capacity,
                capacity,
                items,
            },
        },
    }
}

/// A dict `Object` over `capacity` entries that have already been allocated.
fn dict_object(items: *mut KeyValuePair, capacity: size_t) -> Object {
    Object {
        type_0: kObjectTypeDict,
        data: object_data {
            dict: Dict {
                size: capacity,
                capacity,
                items,
            },
        },
    }
}

/// Where a node's value belongs: the slot to write it into, and — for a map
/// key that has not been read yet — the key slot beside it.
struct Destination {
    result: *mut Object,
    key_location: *mut String_0,
}

/// Resolves a node's destination from its parent.
///
/// # Safety
/// `parent` is null, or points at a live parse node whose `data[0]` holds the
/// object it is filling in.
unsafe fn destination(
    p: *mut Unpacker,
    parent: *mut mpack_node_t,
    tok: mpack_token_t,
) -> Destination {
    if parent.is_null() {
        // SAFETY: the root node's value is the unpacker's own result slot.
        return Destination {
            result: unsafe { &raw mut (*p).result },
            key_location: core::ptr::null_mut(),
        };
    }
    // SAFETY: the caller's parent node; each arm's union read is guarded by
    // the token type it belongs to.
    unsafe {
        match (*parent).tok.type_0 {
            TOKEN_ARRAY => {
                let obj: *mut Object = (*parent).data[0].p.cast::<Object>();
                Destination {
                    result: (*obj).data.array.items.add((*parent).pos),
                    key_location: core::ptr::null_mut(),
                }
            }
            TOKEN_MAP => {
                let obj: *mut Object = (*parent).data[0].p.cast::<Object>();
                let kv: *mut KeyValuePair = (*obj).data.dict.items.add((*parent).pos);
                let key_location = if (*parent).key_visited == 0 {
                    (*kv).key = String_0::NULL;
                    &raw mut (*kv).key
                } else {
                    core::ptr::null_mut()
                };
                Destination {
                    result: &raw mut (*kv).value,
                    key_location,
                }
            }
            // A string, blob or extension's only children are its chunks,
            // which write through the back-pointer instead of into a slot.
            TOKEN_STR | TOKEN_BIN | TOKEN_EXT => {
                debug_assert!(tok.type_0 == TOKEN_CHUNK);
                Destination {
                    result: core::ptr::null_mut(),
                    key_location: core::ptr::null_mut(),
                }
            }
            _ => abort(),
        }
    }
}

/// Fills in one node of libmpack's parse tree as it is entered.
///
/// The node's parent says where the value belongs; strings and containers
/// leave a back-pointer in `node.data[0]` for their chunks to write through.
///
/// # Safety
/// libmpack's contract: `parser` is the parser this was registered with and
/// `node` the node it is entering.
unsafe extern "C-unwind" fn api_parse_enter(parser: *mut mpack_parser_t, node: *mut mpack_node_t) {
    /// The most bytes an extension object's payload may take before it is
    /// rejected: a msgpack uint is at most a tag plus eight bytes.
    const EXT_PAYLOAD_MAX: size_t = 9;

    // SAFETY: the parser's `data` is the unpacker that installed this
    // callback, and the root node's predecessor is the sentinel, whose `pos`
    // is all ones.
    let (p, parent, tok) = unsafe {
        let p: *mut Unpacker = (*parser).data.p.cast::<Unpacker>();
        let parent = node.sub(1);
        let parent = if (*parent).pos == size_t::MAX {
            core::ptr::null_mut()
        } else {
            parent
        };
        (p, parent, (*node).tok)
    };
    // SAFETY: as above.
    let Destination {
        result,
        key_location,
    } = unsafe { destination(p, parent, tok) };

    if let Some(object) = scalar_object(tok) {
        // SAFETY: a scalar always has a slot — only a chunk does not, and a
        // chunk is not a scalar.
        unsafe { *result = object };
        return;
    }

    match tok.type_0 {
        TOKEN_BIN | TOKEN_STR => {
            // One byte over length: the API hands out NUL-terminated strings
            // even though it carries the length beside them.
            let len = tok.length as size_t;
            // SAFETY: the arena hands back `len + 1` writable bytes, and the
            // node's back-pointer is where its chunks will write.
            unsafe {
                let mem = arena_alloc(&raw mut (*p).arena, len + 1, false).cast::<c_char>();
                *mem.add(len) = 0;
                let str = String_0::from_raw_parts(mem, len);
                if key_location.is_null() {
                    *result = Object {
                        type_0: kObjectTypeString,
                        data: object_data { string: str },
                    };
                } else {
                    *key_location = str;
                }
                (*node).data[0].p = str.data().cast::<c_void>();
            }
        }
        TOKEN_EXT => {
            // SAFETY: the node is live; its chunks assemble the payload and
            // then overwrite the slot this points at.
            unsafe { (*node).data[0].p = result.cast::<c_void>() };
        }
        TOKEN_CHUNK => {
            debug_assert!(!parent.is_null());
            // SAFETY: a chunk always has a parent, whose
            // `data[0]` is either the string being filled or the extension's
            // result slot; `tok.data.chunk_ptr` is `tok.length` readable
            // bytes of the input buffer.
            unsafe { copy_chunk(p, parent, tok, EXT_PAYLOAD_MAX) };
        }
        TOKEN_ARRAY => {
            let capacity = tok.length as size_t;
            // SAFETY: the arena hands back `capacity` zeroed `Object` slots.
            unsafe {
                let items = arena_alloc(&raw mut (*p).arena, size_of::<Object>() * capacity, true)
                    .cast::<Object>();
                *result = array_object(items, capacity);
                (*node).data[0].p = result.cast::<c_void>();
            }
        }
        TOKEN_MAP => {
            let capacity = tok.length as size_t;
            // SAFETY: the arena hands back `capacity` zeroed entries.
            unsafe {
                let items = arena_alloc(
                    &raw mut (*p).arena,
                    size_of::<KeyValuePair>() * capacity,
                    true,
                )
                .cast::<KeyValuePair>();
                *result = dict_object(items, capacity);
                (*node).data[0].p = result.cast::<c_void>();
            }
        }
        _ => {}
    }
}

/// Copies one chunk into whatever its parent is collecting.
///
/// A string or blob writes straight into the arena allocation its header
/// made; an extension accumulates into the unpacker's fixed payload buffer
/// and becomes a handle once the last chunk lands. A payload longer than
/// `ext_payload_max` decodes as nil rather than failing the message.
///
/// # Safety
/// [`api_parse_enter`]'s contract for a `CHUNK` node.
unsafe fn copy_chunk(
    p: *mut Unpacker,
    parent: *mut mpack_node_t,
    tok: mpack_token_t,
    ext_payload_max: size_t,
) {
    // SAFETY: the caller's node and chunk.
    unsafe {
        let len = tok.length as size_t;
        let chunk = tok.data.chunk_ptr;
        let pos = (*parent).pos;
        if (*parent).tok.type_0 == TOKEN_STR || (*parent).tok.type_0 == TOKEN_BIN {
            let data: *mut c_char = (*parent).data[0].p.cast::<c_char>();
            data.add(pos).copy_from_nonoverlapping(chunk, len);
            return;
        }
        let res: *mut Object = (*parent).data[0].p.cast::<Object>();
        if pos + len > ext_payload_max {
            *res = nil_object();
            return;
        }
        (&raw mut (*p).ext_buf)
            .cast::<c_char>()
            .add(pos)
            .copy_from_nonoverlapping(chunk, len);
        if pos + len >= (*parent).tok.length as size_t {
            *res = ext_object(p, (*parent).tok.data.ext_type, (*parent).tok.length);
        }
    }
}

fn nil_object() -> Object {
    Object {
        type_0: kObjectTypeNil,
        data: object_data { boolean: false },
    }
}

/// Turns a complete extension payload into the handle it stands for.
///
/// Only the three handle types are recognised; anything else — including a
/// payload that is not a plain unsigned integer — decodes as nil rather than
/// failing the message.
///
/// # Safety
/// `p` points at a live unpacker whose `ext_buf` holds `length` bytes.
unsafe fn ext_object(p: *mut Unpacker, ext_type: c_int, length: mpack_uint32_t) -> Object {
    // SAFETY: the caller's payload buffer, read as one token.
    let tok = unsafe {
        let mut buf: *const c_char = (&raw mut (*p).ext_buf).cast::<c_char>();
        let mut size = length as size_t;
        let mut tok: mpack_token_t = core::mem::zeroed();
        if mpack_rtoken(&raw mut buf, &raw mut size, &raw mut tok) != 0 {
            return nil_object();
        }
        tok
    };
    if tok.type_0 != TOKEN_UINT {
        return nil_object();
    }
    let handles = 0..=kObjectTypeTabpage.cast_signed() - kObjectTypeBuffer.cast_signed();
    if !handles.contains(&ext_type) {
        return nil_object();
    }
    Object {
        type_0: (ext_type + kObjectTypeBuffer.cast_signed()).cast_unsigned() as ObjectType,
        data: object_data {
            integer: mpack_unpack_uint(tok).cast_signed(),
        },
    }
}

/// # Safety
/// `p` points at writable `Unpacker`-sized storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_init(p: *mut Unpacker) {
    // SAFETY: the caller's storage.
    unsafe {
        mpack_parser_init(&raw mut (*p).parser, 0);
        (*p).parser.data.p = p.cast::<c_void>();
        mpack_tokbuf_init(&raw mut (*p).reader);
        (*p).unpack_error = Error {
            type_0: kErrorTypeNone,
            msg: core::ptr::null_mut(),
        };
        (*p).arena = ARENA_EMPTY;
        (*p).has_grid_line_event = false;
    }
}

/// # Safety
/// `p` points at an unpacker that has been through [`unpacker_init`] and is
/// about to be released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_teardown(p: *mut Unpacker) {
    // SAFETY: the arena is the unpacker's own.
    unsafe { arena_mem_free(arena_finish(&raw mut (*p).arena)) };
}

/// Reads `[type, id?, method?, ...]` and leaves the cursor on the body.
///
/// The header is assumed to be small — around ten bytes plus the method name
/// — so rather than tracking sub-states it refuses to advance the stream
/// until the whole thing has arrived.
///
/// # Safety
/// `p` points at a live unpacker whose `read_ptr`/`read_size` describe the
/// bytes that have arrived.
unsafe fn unpacker_parse_header(p: *mut Unpacker) -> bool {
    // SAFETY: the caller's unpacker. Nothing this function calls re-enters
    // it, so one borrow serves the whole body.
    let u = unsafe { &mut *p };
    debug_assert!(u.unpack_error.type_0 == kErrorTypeNone);
    let mut data: *const c_char = u.read_ptr;
    let mut size: size_t = u.read_size;
    // SAFETY: a token is a plain value with no invalid bit patterns.
    let mut tok: mpack_token_t = unsafe { core::mem::zeroed() };

    // `mpack_read` buffers a partial token across calls, so a failure other
    // than end-of-input leaves the reader unusable. The block below is the
    // only thing that touches the reader, which is why holding a raw pointer
    // to it beside `u` cannot alias anything else.
    let reader = &raw mut u.reader;
    let mut next =
        |tok: *mut mpack_token_t| unsafe { mpack_read(reader, &raw mut data, &raw mut size, tok) };

    let result = 'error: {
        let mut result = next(&raw mut tok);
        if result != 0 {
            break 'error result;
        }
        if tok.type_0 != TOKEN_ARRAY || tok.length < 3 || tok.length > 4 {
            break 'error result;
        }
        let array_length = tok.length as size_t;

        result = next(&raw mut tok);
        if result != 0 {
            break 'error result;
        }
        if tok.type_0 != TOKEN_UINT {
            break 'error result;
        }
        let message_type = msgpack_uint_as_u32(mpack_unpack_uint(tok));
        if !protocol::header_shape_is_valid(array_length, message_type) {
            break 'error result;
        }
        u.type_0 = message_type.cast_signed();
        u.request_id = 0;

        if u.type_0 != kMessageTypeNotification {
            result = next(&raw mut tok);
            if result != 0 {
                break 'error result;
            }
            if tok.type_0 != TOKEN_UINT {
                break 'error result;
            }
            u.request_id = msgpack_uint_as_u32(mpack_unpack_uint(tok));
        }

        if u.type_0 != kMessageTypeResponse {
            result = next(&raw mut tok);
            if result != 0 {
                break 'error result;
            }
            if tok.type_0 != TOKEN_STR && tok.type_0 != TOKEN_BIN
                || tok.length > protocol::METHOD_NAME_MAX
            {
                break 'error result;
            }
            u.method_name_len = tok.length as size_t;

            if u.method_name_len > 0 {
                result = next(&raw mut tok);
                if result != 0 {
                    break 'error result;
                }
                debug_assert!(tok.type_0 == TOKEN_CHUNK);
            }
            if (tok.length as size_t) < u.method_name_len {
                break 'error MPACK_EOF;
            }
            // An unknown method leaves `handler.fn` null, which the dispatch
            // layer reports once the arguments have been read.
            // SAFETY: the chunk is `tok.length` readable bytes of the input
            // buffer, and the error slot is the unpacker's own.
            u.handler = unsafe {
                let name = if tok.length != 0 {
                    tok.data.chunk_ptr
                } else {
                    c"".as_ptr()
                };
                msgpack_rpc_get_handler_for(name, tok.length as size_t, &raw mut u.unpack_error)
            };
        }

        u.read_ptr = data;
        u.read_size = size;
        return true;
    };

    if result == MPACK_EOF {
        // Recover by retrying from scratch once more data is available.
        // SAFETY: the reader is the unpacker's own.
        unsafe { mpack_tokbuf_init(&raw mut u.reader) };
    } else {
        // SAFETY: the error slot is the unpacker's own, and the message is a
        // static string.
        unsafe {
            api_set_error(
                &raw mut u.unpack_error,
                kErrorTypeValidation,
                c"failed to decode msgpack".as_ptr(),
            );
        }
        u.state = protocol::INVALID;
    }
    false
}

/// Decodes as far as the buffered bytes allow.
///
/// Returns whether a whole message — or, for a redraw batch, a whole event —
/// is now available in the unpacker.
///
/// # Safety
/// `p` points at a live unpacker whose `read_ptr`/`read_size` describe the
/// bytes that have arrived.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_advance(p: *mut Unpacker) -> bool {
    // The tree parser below re-enters the unpacker through the parser's own
    // `data` pointer, so this body keeps raw-pointer discipline throughout
    // rather than holding a reference across `mpack_parse`.
    // SAFETY: the caller's unpacker.
    unsafe {
        debug_assert!((*p).state >= 0);
        (*p).has_grid_line_event = false;
    }

    // SAFETY: as above.
    if unsafe { (*p).state } == protocol::HEADER {
        // SAFETY: as above.
        if !unsafe { unpacker_parse_header(p) } {
            return false;
        }
        // SAFETY: as above. `handle_ui_client_redraw` is the handler the
        // dispatch table hands back for the `redraw` method.
        unsafe {
            let is_redraw = (*p).handler.fn_0.is_some_and(|f| {
                core::ptr::fn_addr_eq(
                    f,
                    handle_ui_client_redraw
                        as unsafe fn(u64, Array, *mut Arena, *mut Error) -> Object,
                )
            });
            if (*p).type_0 == kMessageTypeNotification && is_redraw {
                (*p).type_0 = kMessageTypeRedrawEvent;
                (*p).state = protocol::REDRAW_EVENTS;
            } else {
                (*p).state = if (*p).type_0 == kMessageTypeResponse {
                    protocol::RESPONSE_ERROR
                } else {
                    protocol::BODY
                };
                (*p).arena = ARENA_EMPTY;
            }
        }
    }

    // A grid_line event decodes itself; every other body goes through the
    // tree parser below.
    let mut body_is_unpacked = false;
    // SAFETY: the caller's unpacker.
    let in_redraw = unsafe {
        (*p).state >= protocol::REDRAW_EVENTS && (*p).state != protocol::REDRAW_ARGS_DONE
    };
    if in_redraw {
        // SAFETY: as above.
        if !unsafe { unpacker_parse_redraw(p) } {
            return false;
        }
        // SAFETY: as above.
        unsafe {
            if (*p).state == protocol::GRID_LINE_WRAP {
                (*p).has_grid_line_event = true;
                body_is_unpacked = true;
            } else {
                debug_assert!((*p).state == protocol::REDRAW_ARGS);
                (*p).arena = ARENA_EMPTY;
                (*p).state = protocol::REDRAW_ARGS_DONE;
            }
        }
    }

    loop {
        if !body_is_unpacked {
            // SAFETY: the caller's unpacker and the bytes it points at; the
            // callback navigates back to it through `parser.data`.
            let result = unsafe {
                mpack_parse(
                    &raw mut (*p).parser,
                    &raw mut (*p).read_ptr,
                    &raw mut (*p).read_size,
                    Some(api_parse_enter),
                    Some(parse_nop),
                )
            };
            if result == MPACK_EOF {
                return false;
            }
            if result != MPACK_OK {
                // SAFETY: the error slot is the unpacker's own, and the
                // message is a static string.
                unsafe {
                    api_set_error(
                        &raw mut (*p).unpack_error,
                        kErrorTypeValidation,
                        c"failed to parse msgpack".as_ptr(),
                    );
                    (*p).state = protocol::INVALID;
                }
                return false;
            }
        }
        body_is_unpacked = false;

        // SAFETY: the caller's unpacker.
        unsafe {
            match (*p).state {
                protocol::RESPONSE_ERROR => {
                    (*p).error = (*p).result;
                    (*p).state = protocol::BODY;
                }
                protocol::BODY => {
                    (*p).state = protocol::HEADER;
                    return true;
                }
                protocol::REDRAW_ARGS_DONE | protocol::GRID_LINE_WRAP => {
                    (*p).ncalls -= 1;
                    (*p).state =
                        protocol::stage_after_redraw_call((*p).state, (*p).ncalls, (*p).nevents);
                    return true;
                }
                _ => abort(),
            }
        }
    }
}

/// The value of an integer token, whichever sign it was encoded with.
pub(super) fn unpack_integer_token(tok: mpack_token_t) -> Option<Integer> {
    if tok.type_0 == TOKEN_UINT {
        Some(mpack_unpack_uint(tok).cast_signed())
    } else if tok.type_0 == TOKEN_SINT {
        Some(mpack_unpack_sint(tok))
    } else {
        None
    }
}

/// libmpack calls this on every node it leaves, and on both edges when a
/// value is being skipped. Nothing here needs either.
///
/// # Safety
/// libmpack's contract; neither argument is touched.
pub(super) unsafe extern "C-unwind" fn parse_nop(
    _parser: *mut mpack_parser_t,
    _node: *mut mpack_node_t,
) {
}
