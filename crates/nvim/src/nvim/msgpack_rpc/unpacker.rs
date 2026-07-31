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

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::src::mpack::conv::{
    mpack_unpack_boolean, mpack_unpack_float_fast, mpack_unpack_sint, mpack_unpack_uint,
};
use crate::src::mpack::mpack_core::{mpack_read, mpack_rtoken, mpack_tokbuf_init};
use crate::src::mpack::object::{mpack_parse, mpack_parser_init};
use crate::src::nvim::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::src::nvim::api::private::helpers::api_set_error;
use crate::src::nvim::grid::schar_from_buf;
use crate::src::nvim::main::{grid_line_buf_attr, grid_line_buf_char, grid_line_buf_size};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xrealloc, xstrdup,
};
use crate::src::nvim::os::libc::abort;
use crate::src::nvim::strings::arena_printf;
pub use crate::src::nvim::types::{
    AdditionalData, AdditionalDataBuilder, Arena, Array, Boolean, Dict, Error, ErrorType,
    FieldHashfn, GridLineEvent, Integer, KeySetLink, KeyValuePair, MessageType,
    MsgpackRpcRequestHandler, Object, ObjectType, OptKeySet, String_0, StringArray,
    UIClientHandler, Unpacker, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer,
    kObjectTypeDict, kObjectTypeFloat, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString,
    kObjectTypeTabpage, mpack_node_t, mpack_parser_t, mpack_tokbuf_t, mpack_token_t,
    mpack_uint32_t, object, object_data, schar_T, size_t, ssize_t, uint32_t,
};
use crate::src::nvim::ui_client::{
    handle_ui_client_redraw, ui_client_event_grid_line, ui_client_get_redraw_handler,
};

pub mod protocol;

use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
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
pub const MPACK_OK: c_uint = 0;
pub const MPACK_EOF: c_uint = 1;
pub const MPACK_ERROR: c_uint = 2;
pub const MPACK_NOMEM: c_int = 3;

/// Decodes one complete object from `data`.
///
/// Unlike the streaming path this needs the whole value up front; anything
/// left over is an error rather than the start of the next message.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpack(
    mut data: *const c_char,
    mut size: size_t,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    let mut unpacker: Unpacker = core::mem::zeroed();
    // `api_parse_enter` navigates back here through the parser's `data`
    // field, so every access below has to go through the same pointer it is
    // handed — writing to `unpacker` by name would strip the callback's
    // permission to read what it wrote.
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

    let message = if result == MPACK_NOMEM {
        c"object was too deep to unpack"
    } else if result == MPACK_EOF as c_int {
        c"incomplete msgpack string"
    } else if result == MPACK_ERROR as c_int {
        c"invalid msgpack string"
    } else if result == MPACK_OK as c_int && size != 0 {
        c"trailing data in msgpack string"
    } else {
        return (*p).result;
    };
    api_set_error(err, kErrorTypeException, message.as_ptr());
    (*p).result
}

/// Fills in one node of libmpack's parse tree as it is entered.
///
/// The node's parent says where the value belongs; strings and containers
/// leave a back-pointer in `node.data[0]` for their chunks to write through.
unsafe extern "C-unwind" fn api_parse_enter(parser: *mut mpack_parser_t, node: *mut mpack_node_t) {
    /// The most bytes an extension object's payload may take before it is
    /// rejected: a msgpack uint is at most a tag plus eight bytes.
    const EXT_PAYLOAD_MAX: size_t = 9;

    let p: *mut Unpacker = (*parser).data.p.cast::<Unpacker>();
    let mut result: *mut Object = core::ptr::null_mut();
    let mut key_location: *mut String_0 = core::ptr::null_mut();

    // The root node's predecessor is the sentinel, whose `pos` is all ones.
    let parent = node.sub(1);
    let parent = if (*parent).pos == size_t::MAX {
        core::ptr::null_mut()
    } else {
        parent
    };

    if parent.is_null() {
        result = &raw mut (*p).result;
    } else {
        match (*parent).tok.type_0 {
            TOKEN_ARRAY => {
                let obj: *mut Object = (*parent).data[0].p.cast::<Object>();
                result = (*obj).data.array.items.add((*parent).pos);
            }
            TOKEN_MAP => {
                let obj: *mut Object = (*parent).data[0].p.cast::<Object>();
                let kv: *mut KeyValuePair = (*obj).data.dict.items.add((*parent).pos);
                if (*parent).key_visited == 0 {
                    (*kv).key = String_0 {
                        data: core::ptr::null_mut(),
                        size: 0,
                    };
                    key_location = &raw mut (*kv).key;
                }
                result = &raw mut (*kv).value;
            }
            TOKEN_STR | TOKEN_BIN | TOKEN_EXT => {
                assert!((*node).tok.type_0 == TOKEN_CHUNK);
            }
            _ => abort(),
        }
    }

    match (*node).tok.type_0 {
        TOKEN_NIL => *result = nil_object(),
        TOKEN_BOOLEAN => {
            *result = Object {
                type_0: kObjectTypeBoolean,
                data: object_data {
                    boolean: mpack_unpack_boolean((*node).tok),
                },
            };
        }
        TOKEN_SINT | TOKEN_UINT => {
            *result = Object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: unpack_integer_token((*node).tok).expect("token is an integer"),
                },
            };
        }
        TOKEN_FLOAT => {
            *result = Object {
                type_0: kObjectTypeFloat,
                data: object_data {
                    floating: mpack_unpack_float_fast((*node).tok),
                },
            };
        }
        TOKEN_BIN | TOKEN_STR => {
            // One byte over length: the API hands out NUL-terminated strings
            // even though it carries the length beside them.
            let len = (*node).tok.length as size_t;
            let mem = arena_alloc(&raw mut (*p).arena, len + 1, false).cast::<c_char>();
            *mem.add(len) = 0;
            let str = String_0 {
                data: mem,
                size: len,
            };
            if key_location.is_null() {
                *result = Object {
                    type_0: kObjectTypeString,
                    data: object_data { string: str },
                };
            } else {
                *key_location = str;
            }
            (*node).data[0].p = str.data.cast::<c_void>();
        }
        TOKEN_EXT => {
            (*node).data[0].p = result.cast::<c_void>();
        }
        TOKEN_CHUNK => {
            assert!(!parent.is_null());
            let len = (*node).tok.length as size_t;
            let chunk = (*node).tok.data.chunk_ptr;
            if (*parent).tok.type_0 == TOKEN_STR || (*parent).tok.type_0 == TOKEN_BIN {
                let data: *mut c_char = (*parent).data[0].p.cast::<c_char>();
                data.add((*parent).pos).copy_from_nonoverlapping(chunk, len);
            } else {
                let res: *mut Object = (*parent).data[0].p.cast::<Object>();
                if (*parent).pos + len > EXT_PAYLOAD_MAX {
                    *res = nil_object();
                } else {
                    (&raw mut (*p).ext_buf)
                        .cast::<c_char>()
                        .add((*parent).pos)
                        .copy_from_nonoverlapping(chunk, len);
                    if (*parent).pos + len >= (*parent).tok.length as size_t {
                        *res = ext_object(p, (*parent).tok.data.ext_type, (*parent).tok.length);
                    }
                }
            }
        }
        TOKEN_ARRAY => {
            let capacity = (*node).tok.length as size_t;
            let items = arena_alloc(&raw mut (*p).arena, size_of::<Object>() * capacity, true)
                .cast::<Object>();
            *result = Object {
                type_0: kObjectTypeArray,
                data: object_data {
                    array: Array {
                        size: capacity,
                        capacity,
                        items,
                    },
                },
            };
            (*node).data[0].p = result.cast::<c_void>();
        }
        TOKEN_MAP => {
            let capacity = (*node).tok.length as size_t;
            let items = arena_alloc(
                &raw mut (*p).arena,
                size_of::<KeyValuePair>() * capacity,
                true,
            )
            .cast::<KeyValuePair>();
            *result = Object {
                type_0: kObjectTypeDict,
                data: object_data {
                    dict: Dict {
                        size: capacity,
                        capacity,
                        items,
                    },
                },
            };
            (*node).data[0].p = result.cast::<c_void>();
        }
        _ => {}
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
unsafe fn ext_object(p: *mut Unpacker, ext_type: c_int, length: mpack_uint32_t) -> Object {
    let mut buf: *const c_char = (&raw mut (*p).ext_buf).cast::<c_char>();
    let mut size = length as size_t;
    let mut tok: mpack_token_t = core::mem::zeroed();
    if mpack_rtoken(&raw mut buf, &raw mut size, &raw mut tok) != 0 || tok.type_0 != TOKEN_UINT {
        return nil_object();
    }
    if !(0..=kObjectTypeTabpage as c_int - kObjectTypeBuffer as c_int).contains(&ext_type) {
        return nil_object();
    }
    Object {
        type_0: (ext_type + kObjectTypeBuffer as c_int) as ObjectType,
        data: object_data {
            integer: mpack_unpack_uint(tok) as Integer,
        },
    }
}

unsafe extern "C-unwind" fn api_parse_exit(_parser: *mut mpack_parser_t, _node: *mut mpack_node_t) {
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_init(p: *mut Unpacker) {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_teardown(p: *mut Unpacker) {
    arena_mem_free(arena_finish(&raw mut (*p).arena));
}

/// Reads `[type, id?, method?, ...]` and leaves the cursor on the body.
///
/// The header is assumed to be small — around ten bytes plus the method name
/// — so rather than tracking sub-states it refuses to advance the stream
/// until the whole thing has arrived.
unsafe fn unpacker_parse_header(p: *mut Unpacker) -> bool {
    assert!((*p).unpack_error.type_0 == kErrorTypeNone);
    let mut data: *const c_char = (*p).read_ptr;
    let mut size: size_t = (*p).read_size;
    let mut tok: mpack_token_t = core::mem::zeroed();

    // `mpack_read` buffers a partial token across calls, so a failure other
    // than end-of-input leaves the reader unusable.
    let reader = &raw mut (*p).reader;
    let mut next = |tok: *mut mpack_token_t| mpack_read(reader, &raw mut data, &raw mut size, tok);

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
        let message_type = mpack_unpack_uint(tok) as uint32_t;
        if !protocol::header_shape_is_valid(array_length, message_type) {
            break 'error result;
        }
        (*p).type_0 = message_type as MessageType;
        (*p).request_id = 0;

        if (*p).type_0 != kMessageTypeNotification {
            result = next(&raw mut tok);
            if result != 0 {
                break 'error result;
            }
            if tok.type_0 != TOKEN_UINT {
                break 'error result;
            }
            (*p).request_id = mpack_unpack_uint(tok) as uint32_t;
        }

        if (*p).type_0 != kMessageTypeResponse {
            result = next(&raw mut tok);
            if result != 0 {
                break 'error result;
            }
            if tok.type_0 != TOKEN_STR && tok.type_0 != TOKEN_BIN
                || tok.length > protocol::METHOD_NAME_MAX
            {
                break 'error result;
            }
            (*p).method_name_len = tok.length as size_t;

            if (*p).method_name_len > 0 {
                result = next(&raw mut tok);
                if result != 0 {
                    break 'error result;
                }
                assert!(tok.type_0 == TOKEN_CHUNK);
            }
            if (tok.length as size_t) < (*p).method_name_len {
                break 'error MPACK_EOF as c_int;
            }
            // An unknown method leaves `handler.fn` null, which the dispatch
            // layer reports once the arguments have been read.
            (*p).handler = msgpack_rpc_get_handler_for(
                if tok.length != 0 {
                    tok.data.chunk_ptr
                } else {
                    c"".as_ptr()
                },
                tok.length as size_t,
                &raw mut (*p).unpack_error,
            );
        }

        (*p).read_ptr = data;
        (*p).read_size = size;
        return true;
    };

    if result == MPACK_EOF as c_int {
        // Recover by retrying from scratch once more data is available.
        mpack_tokbuf_init(&raw mut (*p).reader);
    } else {
        api_set_error(
            &raw mut (*p).unpack_error,
            kErrorTypeValidation,
            c"failed to decode msgpack".as_ptr(),
        );
        (*p).state = protocol::INVALID;
    }
    false
}

/// Decodes as far as the buffered bytes allow.
///
/// Returns whether a whole message — or, for a redraw batch, a whole event —
/// is now available in the unpacker.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unpacker_advance(p: *mut Unpacker) -> bool {
    assert!((*p).state >= 0);
    (*p).has_grid_line_event = false;

    if (*p).state == protocol::HEADER {
        if !unpacker_parse_header(p) {
            return false;
        }
        let is_redraw = (*p).handler.fn_0.is_some_and(|f| {
            core::ptr::fn_addr_eq(
                f,
                handle_ui_client_redraw
                    as unsafe extern "C" fn(u64, Array, *mut Arena, *mut Error) -> Object,
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

    // A grid_line event decodes itself; every other body goes through the
    // tree parser below.
    let mut body_is_unpacked = false;
    if (*p).state >= protocol::REDRAW_EVENTS && (*p).state != protocol::REDRAW_ARGS_DONE {
        if !unpacker_parse_redraw(p) {
            return false;
        }
        if (*p).state == protocol::GRID_LINE_WRAP {
            (*p).has_grid_line_event = true;
            body_is_unpacked = true;
        } else {
            assert!((*p).state == protocol::REDRAW_ARGS);
            (*p).arena = ARENA_EMPTY;
            (*p).state = protocol::REDRAW_ARGS_DONE;
        }
    }

    loop {
        if !body_is_unpacked {
            let result = mpack_parse(
                &raw mut (*p).parser,
                &raw mut (*p).read_ptr,
                &raw mut (*p).read_size,
                Some(api_parse_enter),
                Some(parse_nop),
            );
            if result == MPACK_EOF as c_int {
                return false;
            }
            if result != MPACK_OK as c_int {
                api_set_error(
                    &raw mut (*p).unpack_error,
                    kErrorTypeValidation,
                    c"failed to parse msgpack".as_ptr(),
                );
                (*p).state = protocol::INVALID;
                return false;
            }
        }
        body_is_unpacked = false;

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

/// Why a redraw parse stopped short.
enum Halt {
    /// Not all of this stage has arrived. The read cursor stays where it was
    /// last committed, so the stage restarts when more bytes turn up.
    Incomplete,
    /// The stream is not a well-formed redraw batch.
    Invalid,
}

/// The bytes still to be read, as libmpack's token reader wants them.
struct Cursor {
    data: *const c_char,
    size: size_t,
}

impl Cursor {
    /// Reads one token and checks it is the kind this position calls for.
    unsafe fn next(
        &mut self,
        expected: crate::src::nvim::types::mpack_token_type_t,
    ) -> Result<mpack_token_t, Halt> {
        let mut tok: mpack_token_t = core::mem::zeroed();
        let result = mpack_rtoken(&raw mut self.data, &raw mut self.size, &raw mut tok);
        if result == MPACK_EOF as c_int {
            return Err(Halt::Incomplete);
        }
        if result != 0 || !protocol::token_matches(expected, tok.type_0) {
            return Err(Halt::Invalid);
        }
        Ok(tok)
    }

    /// Consumes `len` bytes of payload, handing back where they started.
    fn take(&mut self, len: size_t) -> *const c_char {
        let taken = self.data;
        self.data = self.data.wrapping_add(len);
        self.size -= len;
        taken
    }
}

unsafe fn unpacker_parse_redraw(p: *mut Unpacker) -> bool {
    let mut cursor = Cursor {
        data: (*p).read_ptr,
        size: (*p).read_size,
    };
    match parse_redraw(p, &mut cursor) {
        Ok(done) => done,
        Err(Halt::Incomplete) => false,
        Err(Halt::Invalid) => {
            (*p).state = protocol::INVALID;
            false
        }
    }
}

/// Decodes `[[name, [args], ...], ...]`, one event at a time.
///
/// Each stage falls through into the next, and the read cursor is committed
/// wherever a stage boundary is crossed — so an event that arrives in pieces
/// resumes at the last boundary rather than at the start of the batch.
unsafe fn parse_redraw(p: *mut Unpacker, cursor: &mut Cursor) -> Result<bool, Halt> {
    let g: *mut GridLineEvent = &raw mut (*p).grid_line_event;
    let mut stage = (*p).state;

    if stage == protocol::REDRAW_ARGS {
        return Ok(true);
    }
    // `REDRAW_ARGS_DONE` belongs to the tree parser, not to this one, and its
    // caller filters it out; anything else means the machine lost track.
    let known = protocol::REDRAW_EVENTS..=protocol::GRID_LINE_WRAP;
    if stage == protocol::REDRAW_ARGS_DONE || !known.contains(&stage) {
        abort();
    }

    if stage == protocol::REDRAW_EVENTS {
        (*p).nevents = cursor.next(TOKEN_ARRAY)?.length as c_int;
        stage = protocol::REDRAW_CALL;
    }

    if stage == protocol::REDRAW_CALL {
        (*p).ncalls = cursor.next(TOKEN_ARRAY)?.length as c_int;
        let had_calls = (*p).ncalls;
        (*p).ncalls -= 1;
        if had_calls == 0 {
            return Err(Halt::Invalid);
        }

        let tok = cursor.next(TOKEN_STR)?;
        if tok.length as size_t > cursor.size {
            return Err(Halt::Incomplete);
        }
        (*p).ui_handler =
            ui_client_get_redraw_handler(cursor.data, tok.length as size_t, core::ptr::null_mut());
        cursor.take(tok.length as size_t);

        (*p).nevents -= 1;
        (*p).read_ptr = cursor.data;
        (*p).read_size = cursor.size;

        let is_grid_line = (*p).ui_handler.fn_0.is_some_and(|f| {
            core::ptr::fn_addr_eq(f, ui_client_event_grid_line as unsafe extern "C" fn(Array))
        });
        if !is_grid_line {
            (*p).state = protocol::REDRAW_ARGS;
            return Ok(true);
        }
        (*p).state = protocol::GRID_LINE_EVENT;
        (*p).arena = ARENA_EMPTY;
        stage = protocol::GRID_LINE_EVENT;
    }

    if stage == protocol::GRID_LINE_EVENT {
        // [grid, row, startcol, [cells], wrap]
        if cursor.next(TOKEN_ARRAY)?.length != 5 {
            return Err(Halt::Invalid);
        }
        for slot in 0..3 {
            (*g).args[slot] = cursor.next(TOKEN_UINT)?.data.value.lo as c_int;
        }
        (*g).ncells = cursor.next(TOKEN_ARRAY)?.length as c_int;
        (*g).icell = 0;
        (*g).coloff = 0;
        (*g).cur_attr = -1;
        (*p).read_ptr = cursor.data;
        (*p).read_size = cursor.size;
        (*p).state = protocol::GRID_LINE_CELLS;
        stage = protocol::GRID_LINE_CELLS;
    }

    if stage == protocol::GRID_LINE_CELLS {
        while (*g).icell != (*g).ncells {
            parse_grid_line_cell(g, cursor)?;
            (*p).read_ptr = cursor.data;
            (*p).read_size = cursor.size;
            (*g).icell += 1;
        }
        (*p).state = protocol::GRID_LINE_WRAP;
    }

    (*g).wrap = mpack_unpack_boolean(cursor.next(TOKEN_BOOLEAN)?);
    (*p).read_ptr = cursor.data;
    (*p).read_size = cursor.size;
    Ok(true)
}

/// Decodes `[text, attr?, repeat?]` into the shared line buffers.
///
/// `attr` persists across cells that omit it, which is what makes the wire
/// form compact. A run of spaces at the end of the line is not written at
/// all: it becomes the event's `clear_width`.
unsafe fn parse_grid_line_cell(g: *mut GridLineEvent, cursor: &mut Cursor) -> Result<(), Halt> {
    let arity = cursor.next(TOKEN_ARRAY)?.length as c_int;
    if !(1..=3).contains(&arity) {
        return Err(Halt::Invalid);
    }

    let tok = cursor.next(TOKEN_STR)?;
    if tok.length as size_t > cursor.size {
        return Err(Halt::Incomplete);
    }
    let cell_len = tok.length as size_t;
    let cell = cursor.take(cell_len);

    if arity >= 2 {
        (*g).cur_attr = cursor.next(TOKEN_SINT)?.data.value.lo as c_int;
    }
    let repeat = if arity >= 3 {
        cursor.next(TOKEN_UINT)?.data.value.lo as c_int
    } else {
        1
    };

    (*g).clear_width = 0;
    let cell_bytes = core::slice::from_raw_parts(cell.cast::<u8>(), cell_len);
    if protocol::is_clear_run((*g).icell == (*g).ncells - 1, cell_bytes, repeat) {
        (*g).clear_width = repeat;
        return Ok(());
    }

    let sc: schar_T = schar_from_buf(cell, cell_len);
    for _ in 0..repeat {
        if (*g).coloff >= grid_line_buf_size.get() as c_int {
            return Err(Halt::Invalid);
        }
        *(*grid_line_buf_char.ptr()).add((*g).coloff as usize) = sc;
        *(*grid_line_buf_attr.ptr()).add((*g).coloff as usize) = (*g).cur_attr as _;
        (*g).coloff += 1;
    }
    Ok(())
}

/// Reads a string or binary token, returning a borrow of the buffer rather
/// than a copy. An empty result means the next token was not one.
pub unsafe fn unpack_string(data: *mut *const c_char, size: *mut size_t) -> String_0 {
    const EMPTY: String_0 = String_0 {
        data: core::ptr::null_mut(),
        size: 0,
    };
    let mut data2: *const c_char = *data;
    let mut size2: size_t = *size;
    let mut tok: mpack_token_t = core::mem::zeroed();
    if mpack_rtoken(&raw mut data2, &raw mut size2, &raw mut tok) != 0
        || tok.type_0 != TOKEN_STR && tok.type_0 != TOKEN_BIN
    {
        return EMPTY;
    }
    // Checked against the *original* size, so a token header that consumed
    // several bytes leaves that much slack. Upstream's bound; the caller only
    // ever reads within the buffer it owns.
    if *size < tok.length as size_t {
        return EMPTY;
    }
    *data = data2.add(tok.length as usize);
    *size = size2 - tok.length as size_t;
    String_0 {
        data: data2 as *mut c_char,
        size: tok.length as size_t,
    }
}

/// The length of the array that starts here, or -1 if this is not one.
pub unsafe fn unpack_array(data: *mut *const c_char, size: *mut size_t) -> ssize_t {
    let mut tok: mpack_token_t = core::mem::zeroed();
    if mpack_rtoken(data, size, &raw mut tok) != 0 || tok.type_0 != TOKEN_ARRAY {
        return -1;
    }
    tok.length as ssize_t
}

pub unsafe fn unpack_integer(
    data: *mut *const c_char,
    size: *mut size_t,
    res: *mut Integer,
) -> bool {
    let mut tok: mpack_token_t = core::mem::zeroed();
    if mpack_rtoken(data, size, &raw mut tok) != 0 {
        return false;
    }
    match unpack_integer_token(tok) {
        Some(value) => {
            *res = value;
            true
        }
        None => false,
    }
}

/// The value of an integer token, whichever sign it was encoded with.
fn unpack_integer_token(tok: mpack_token_t) -> Option<Integer> {
    // The decoders only read the token they are handed.
    unsafe {
        if tok.type_0 == TOKEN_UINT {
            Some(mpack_unpack_uint(tok) as Integer)
        } else if tok.type_0 == TOKEN_SINT {
            Some(mpack_unpack_sint(tok) as Integer)
        } else {
            None
        }
    }
}

/// libmpack calls this on every node it leaves, and on both edges when a
/// value is being skipped. Nothing here needs either.
unsafe extern "C-unwind" fn parse_nop(_parser: *mut mpack_parser_t, _node: *mut mpack_node_t) {}

/// Steps over one whole value without building anything from it.
pub unsafe fn unpack_skip(data: *mut *const c_char, size: *mut size_t) -> c_int {
    let mut parser: mpack_parser_t = core::mem::zeroed();
    mpack_parser_init(&raw mut parser, 0);
    mpack_parse(
        &raw mut parser,
        data,
        size,
        Some(parse_nop),
        Some(parse_nop),
    )
}

/// Appends one unrecognised key's raw msgpack to a keyset's spillover.
///
/// The builder's bytes are an [`AdditionalData`] header followed by the
/// concatenated items, so the header is written on the first push and its
/// counters updated on every one.
pub unsafe fn push_additional_data(
    ad: *mut AdditionalDataBuilder,
    data: *const c_char,
    size: size_t,
) {
    if (*ad).size == 0 {
        let header = AdditionalData {
            nitems: 0,
            nbytes: 0,
            data: [],
        };
        reserve(ad, size_of::<AdditionalData>());
        (*ad).items.add((*ad).size).copy_from_nonoverlapping(
            (&raw const header).cast::<c_char>(),
            size_of::<AdditionalData>(),
        );
        (*ad).size += size_of::<AdditionalData>();
    }

    let header: *mut AdditionalData = (*ad).items.cast::<AdditionalData>();
    (*header).nitems += 1;
    (*header).nbytes += size as uint32_t;

    if size > 0 {
        reserve(ad, size);
        (*ad)
            .items
            .add((*ad).size)
            .copy_from_nonoverlapping(data, size);
        (*ad).size += size;
    }
}

/// Makes room for `extra` more bytes in a builder.
unsafe fn reserve(ad: *mut AdditionalDataBuilder, extra: size_t) {
    if (*ad).capacity >= (*ad).size + extra {
        return;
    }
    (*ad).capacity = protocol::capacity_for((*ad).size + extra);
    (*ad).items = xrealloc((*ad).items.cast::<c_void>(), (*ad).capacity).cast::<c_char>();
    assert!(!(*ad).items.is_null());
}

/// Decodes a msgpack map straight into a generated keyset struct.
///
/// Fields the keyset does not know about are skipped and, if the caller
/// supplied a builder, kept verbatim so they survive a round trip. `error` is
/// set to an owned message on failure.
pub unsafe fn unpack_keydict(
    retval: *mut c_void,
    hashy: FieldHashfn,
    ad: *mut AdditionalDataBuilder,
    data: *mut *const c_char,
    size: *mut size_t,
    error: *mut *mut c_char,
) -> bool {
    let ks: *mut OptKeySet = retval.cast::<OptKeySet>();
    let mut tok: mpack_token_t = core::mem::zeroed();
    if mpack_rtoken(data, size, &raw mut tok) != 0 || tok.type_0 != TOKEN_MAP {
        *error = xstrdup(c"is not a dict".as_ptr());
        return false;
    }

    for _ in 0..tok.length {
        let item_start: *const c_char = *data;
        let key = unpack_string(data, size);
        if key.data.is_null() {
            *error = fail(c"has key value which is not a string", key);
            return false;
        }
        if key.size == 0 {
            *error = fail(c"has empty key", key);
            return false;
        }

        let field: *mut KeySetLink =
            hashy.expect("keyset has no hash function")(key.data, key.size);
        if field.is_null() {
            if unpack_skip(data, size) != 0 {
                return false;
            }
            if !ad.is_null() {
                push_additional_data(ad, item_start, (*data).addr() - item_start.addr());
            }
            continue;
        }

        assert!((*field).opt_index >= 0);
        let flag = 1u64 << (*field).opt_index;
        if (*ks).is_set_ & flag != 0 {
            *error = xstrdup(c"duplicate key".as_ptr());
            return false;
        }
        (*ks).is_set_ |= flag;

        let mem = retval.cast::<c_char>().offset((*field).ptr_off as isize);
        match (*field).type_0 {
            field_type::BOOLEAN => {
                // Read straight off the wire: both boolean encodings differ
                // only in their low bit.
                if *size == 0 || **data as c_int & 0xfe != 0xc2 {
                    *error = fail(c"has %.*s key value which is not a boolean", key);
                    return false;
                }
                *mem.cast::<Boolean>() = **data as c_int & 0x1 != 0;
                *data = (*data).add(1);
                *size -= 1;
            }
            field_type::INTEGER => {
                if !unpack_integer(data, size, mem.cast::<Integer>()) {
                    *error = fail(c"has %.*s key value which is not an integer", key);
                    return false;
                }
            }
            field_type::STRING => {
                let val = unpack_string(data, size);
                if val.data.is_null() {
                    *error = fail(c"has %.*s key value which is not a binary", key);
                    return false;
                }
                *mem.cast::<String_0>() = val;
            }
            field_type::STRING_ARRAY => {
                let len = unpack_array(data, size);
                if len < 0 {
                    *error = fail(c"has %.*s key with non-array value", key);
                    return false;
                }
                let a: *mut StringArray = mem.cast::<StringArray>();
                if (*a).capacity < (*a).size + len as size_t {
                    (*a).capacity = protocol::capacity_for((*a).size + len as size_t);
                    (*a).items = xrealloc(
                        (*a).items.cast::<c_void>(),
                        size_of::<String_0>() * (*a).capacity,
                    )
                    .cast::<String_0>();
                }
                for _ in 0..len {
                    let item = unpack_string(data, size);
                    if item.data.is_null() {
                        *error = fail(c"has %.*s array with non-binary value", key);
                        return false;
                    }
                    if (*a).size == (*a).capacity {
                        (*a).capacity = protocol::grown_capacity((*a).capacity);
                        (*a).items = xrealloc(
                            (*a).items.cast::<c_void>(),
                            size_of::<String_0>() * (*a).capacity,
                        )
                        .cast::<String_0>();
                    }
                    *(*a).items.add((*a).size) = item;
                    (*a).size += 1;
                }
            }
            _ => abort(),
        }
    }
    true
}

/// An owned copy of one of `unpack_keydict`'s complaints. Messages that name
/// the offending key spell it `%.*s`; the extra arguments are harmless to the
/// ones that do not.
unsafe fn fail(message: &core::ffi::CStr, key: String_0) -> *mut c_char {
    arena_printf(
        core::ptr::null_mut(),
        message.as_ptr(),
        key.size as c_int,
        key.data,
    )
    .data
}
