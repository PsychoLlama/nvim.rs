#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

//! Incremental msgpack-rpc decoding.
//!
//! The unpacker is fed whatever bytes have arrived and resumes wherever it
//! stopped, so it must never advance past a point it cannot recover from: the
//! read cursor is only committed once a stage has been decoded in full. See
//! [`protocol`] for the stages.
//!
//! Message bodies go through libmpack's tree walk ([`Parser`]), with
//! [`BodyWalk`] building owned `Object`s as the tokens arrive: each node of
//! the walk holds the object it is building, and hands it to its parent when
//! it is complete. `redraw` notifications from a UI server bypass that — a
//! `grid_line` event would otherwise allocate an object per screen cell —
//! and are decoded a token at a time straight into the shared line buffers.

use core::ffi::c_int;

use crate::api::private::dispatch::{NO_HANDLER, msgpack_rpc_get_handler_for};
use crate::memory::XString;
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_ERROR, MPACK_OK, Next, TokBuf, read_token};
use crate::mpack::object::{MPACK_NOMEM, Node, Parser};
use crate::mpack::token::{self, Kind, Tok};
use crate::msgpack_rpc::packer::{EXT_BUFFER, EXT_TABPAGE, EXT_WINDOW};
use crate::narrow::msgpack_uint_as_u32;
use crate::types::{
    ApiDict, ApiDispatchFn, Array, DictKey, Error, GridLineEvent, Integer, KeyValuePair,
    MessageType, MsgpackRpcRequestHandler, Object, String_0, UIClientHandler,
};
use crate::ui_client::handle_ui_client_redraw;

pub mod protocol;
pub mod redraw;

use protocol::{
    ARRAY as TOKEN_ARRAY, BIN as TOKEN_BIN, BOOLEAN as TOKEN_BOOLEAN, CHUNK as TOKEN_CHUNK,
    EXT as TOKEN_EXT, FLOAT as TOKEN_FLOAT, MAP as TOKEN_MAP, NIL as TOKEN_NIL, SINT as TOKEN_SINT,
    STR as TOKEN_STR, UINT as TOKEN_UINT,
};

pub const kMessageTypeRedrawEvent: MessageType = 3;
pub const kMessageTypeNotification: MessageType = 2;
pub const kMessageTypeResponse: MessageType = 1;
pub const kUnpackTypeStringArray: c_int = -1;

/// The most bytes an extension object's payload may take before it is
/// rejected: a msgpack uint is at most a tag plus eight bytes.
const EXT_PAYLOAD_MAX: usize = 9;

/// How many elements a container header may reserve room for up front.
/// The length is the peer's word, so a header claiming four billion items
/// grows its vector as the items arrive instead of asking for it all at once.
const RESERVE_MAX: usize = 1 << 16;

/// A channel's decoder: the walk in progress, the header it read, and the
/// message it is assembling.
pub struct Unpacker {
    pub(crate) parser: Parser<Building>,
    /// The header's own tokenizer, reset whenever a header has to wait for
    /// more bytes.
    reader: TokBuf,
    pub state: c_int,
    pub type_0: MessageType,
    pub request_id: u32,
    pub method_name_len: usize,
    pub handler: MsgpackRpcRequestHandler,
    pub error: Object,
    pub result: Object,
    pub unpack_error: Error,
    pub nevents: c_int,
    pub ncalls: c_int,
    pub ui_handler: UIClientHandler,
    pub grid_line_event: GridLineEvent,
    pub has_grid_line_event: bool,
}

impl Default for Unpacker {
    fn default() -> Self {
        Self::new()
    }
}

impl Unpacker {
    /// A decoder between messages.
    pub fn new() -> Self {
        Unpacker {
            parser: Parser::new(0),
            reader: TokBuf::new(),
            state: protocol::HEADER,
            type_0: 0,
            request_id: 0,
            method_name_len: 0,
            handler: NO_HANDLER,
            error: Object::Nil,
            result: Object::Nil,
            unpack_error: Error::none(),
            nevents: 0,
            ncalls: 0,
            ui_handler: UIClientHandler {
                name: core::ptr::null(),
                fn_0: None,
            },
            grid_line_event: GridLineEvent::default(),
            has_grid_line_event: false,
        }
    }

    /// Decodes as far as `input` allows, advancing it past what was used.
    ///
    /// Returns whether a whole message — or, for a redraw batch, a whole
    /// event — is now available in the unpacker.
    pub fn advance(&mut self, input: &mut &[u8]) -> bool {
        debug_assert!(self.state >= 0);
        self.has_grid_line_event = false;

        if self.state == protocol::HEADER {
            if !self.parse_header(input) {
                return false;
            }
            // `handle_ui_client_redraw` is the handler the dispatch table
            // hands back for the `redraw` method.
            let is_redraw = self.handler.fn_0.is_some_and(|f| {
                core::ptr::fn_addr_eq(f, handle_ui_client_redraw as ApiDispatchFn)
            });
            if self.type_0 == kMessageTypeNotification && is_redraw {
                self.type_0 = kMessageTypeRedrawEvent;
                self.state = protocol::REDRAW_EVENTS;
            } else {
                self.state = if self.type_0 == kMessageTypeResponse {
                    protocol::RESPONSE_ERROR
                } else {
                    protocol::BODY
                };
            }
        }

        // A grid_line event decodes itself; every other body goes through
        // the tree walk below.
        let mut body_is_unpacked = false;
        if self.state >= protocol::REDRAW_EVENTS && self.state != protocol::REDRAW_ARGS_DONE {
            if !redraw::unpacker_parse_redraw(self, input) {
                return false;
            }
            if self.state == protocol::GRID_LINE_WRAP {
                self.has_grid_line_event = true;
                body_is_unpacked = true;
            } else {
                debug_assert!(self.state == protocol::REDRAW_ARGS);
                self.state = protocol::REDRAW_ARGS_DONE;
            }
        }

        loop {
            if !body_is_unpacked {
                let Unpacker { parser, result, .. } = self;
                let status = parser.parse(input, &mut BodyWalk { result });
                if status == MPACK_EOF.cast_signed() {
                    return false;
                }
                if status != MPACK_OK.cast_signed() {
                    self.unpack_error = Error::validation(c"failed to parse msgpack");
                    self.state = protocol::INVALID;
                    return false;
                }
            }
            body_is_unpacked = false;

            match self.state {
                protocol::RESPONSE_ERROR => {
                    self.error = self.result.take();
                    self.state = protocol::BODY;
                }
                protocol::BODY => {
                    self.state = protocol::HEADER;
                    return true;
                }
                protocol::REDRAW_ARGS_DONE | protocol::GRID_LINE_WRAP => {
                    self.ncalls -= 1;
                    self.state =
                        protocol::stage_after_redraw_call(self.state, self.ncalls, self.nevents);
                    return true;
                }
                _ => unreachable!("the unpacker lost track of its stage"),
            }
        }
    }

    /// Reads `[type, id?, method?, ...]` and leaves `input` on the body.
    ///
    /// The header is assumed to be small — around ten bytes plus the method
    /// name — so rather than tracking sub-states it refuses to advance the
    /// stream until the whole thing has arrived.
    fn parse_header(&mut self, input: &mut &[u8]) -> bool {
        debug_assert!(!self.unpack_error.is_set());
        let mut data = *input;
        match self.read_header(&mut data) {
            Ok(()) => {
                *input = data;
                true
            }
            Err(HeaderStop::Incomplete) => {
                // Recover by retrying from scratch once more data is
                // available.
                self.reader = TokBuf::new();
                false
            }
            Err(HeaderStop::Invalid) => {
                self.unpack_error = Error::validation(c"failed to decode msgpack");
                self.state = protocol::INVALID;
                false
            }
        }
    }

    /// The header's tokens, in order. Every refusal is an error except
    /// running out of bytes; see [`HeaderStop`].
    fn read_header(&mut self, data: &mut &[u8]) -> Result<(), HeaderStop> {
        let reader = &mut self.reader;
        let tok = next_header_token(reader, data)?.0;
        if tok.kind != Some(Kind::Array) || tok.len < 3 || tok.len > 4 {
            return Err(HeaderStop::Invalid);
        }
        let array_length = tok.len as usize;

        let tok = next_header_token(reader, data)?.0;
        if tok.kind != Some(Kind::Uint) {
            return Err(HeaderStop::Invalid);
        }
        let message_type = msgpack_uint_as_u32(token::unpack_uint(&tok));
        if !protocol::header_shape_is_valid(array_length, message_type) {
            return Err(HeaderStop::Invalid);
        }
        self.type_0 = message_type.cast_signed();
        self.request_id = 0;

        if self.type_0 != kMessageTypeNotification {
            let tok = next_header_token(reader, data)?.0;
            if tok.kind != Some(Kind::Uint) {
                return Err(HeaderStop::Invalid);
            }
            self.request_id = msgpack_uint_as_u32(token::unpack_uint(&tok));
        }

        if self.type_0 != kMessageTypeResponse {
            let tok = next_header_token(reader, data)?.0;
            if !matches!(tok.kind, Some(Kind::Str | Kind::Bin))
                || tok.len > protocol::METHOD_NAME_MAX
            {
                return Err(HeaderStop::Invalid);
            }
            self.method_name_len = tok.len as usize;
            // An empty name is the header token itself, whose length is 0.
            let (mut got, mut name) = (tok.len, &[][..]);
            if self.method_name_len > 0 {
                let (chunk, bytes) = next_header_token(reader, data)?;
                debug_assert!(chunk.kind == Some(Kind::Chunk));
                (got, name) = (chunk.len, bytes);
            }
            if (got as usize) < self.method_name_len {
                return Err(HeaderStop::Incomplete);
            }
            // An unknown method leaves `handler.fn` null, which the dispatch
            // layer reports once the arguments have been read; why it did
            // not resolve waits in the unpacker's own error slot.
            self.handler = match msgpack_rpc_get_handler_for(name) {
                Ok(handler) => handler,
                Err(e) => {
                    self.unpack_error = e;
                    NO_HANDLER
                }
            };
        }
        Ok(())
    }
}

/// Why a header could not be read yet.
enum HeaderStop {
    /// Not all of it has arrived; it is read again from the start.
    Incomplete,
    /// It is not a msgpack-rpc header.
    Invalid,
}

/// The header's next token and, for a chunk, its bytes.
fn next_header_token<'a>(
    reader: &mut TokBuf,
    data: &mut &'a [u8],
) -> Result<(Tok, &'a [u8]), HeaderStop> {
    match reader.read(data) {
        Next::Token(tok) => Ok((tok, &[])),
        Next::Chunk(bytes) => Ok((chunk_token(bytes.len()), bytes)),
        Next::Eof => Err(HeaderStop::Incomplete),
        Next::Error => Err(HeaderStop::Invalid),
    }
}

fn chunk_token(len: usize) -> Tok {
    let len = u32::try_from(len).expect("a chunk is at most a token's length");
    Tok::new(Kind::Chunk, len, 0, 0)
}

/// Decodes one complete object from `data`.
///
/// Unlike the streaming path this needs the whole value up front; anything
/// left over is an error rather than the start of the next message.
pub fn unpack(data: &[u8]) -> Result<Object, Error> {
    let mut parser = Parser::<Building>::new(0);
    let mut value = Object::Nil;
    let mut rest = data;
    let result = parser.parse(&mut rest, &mut BodyWalk { result: &mut value });

    let message = if result == MPACK_NOMEM {
        c"object was too deep to unpack"
    } else if result == MPACK_EOF.cast_signed() {
        c"incomplete msgpack string"
    } else if result == MPACK_ERROR.cast_signed() {
        c"invalid msgpack string"
    } else if result == MPACK_OK.cast_signed() && !rest.is_empty() {
        c"trailing data in msgpack string"
    } else {
        return Ok(value);
    };
    Err(Error::exception(message))
}

/// The type number a token carries, as [`protocol`] spells them.
pub(super) fn token_type(tok: &Tok) -> u32 {
    tok.kind.map_or(0, |kind| kind as u32)
}

/// The `Object` a scalar token stands for, or `None` when the token opens a
/// container, a payload or a chunk instead.
///
/// Pure: a token is a value.
pub(super) fn scalar_object(tok: Tok) -> Option<Object> {
    Some(match token_type(&tok) {
        TOKEN_NIL => Object::Nil,
        TOKEN_BOOLEAN => Object::Boolean(token::unpack_boolean(&tok)),
        TOKEN_SINT | TOKEN_UINT => {
            Object::Integer(unpack_integer_token(tok).expect("token is an integer"))
        }
        TOKEN_FLOAT => Object::Float(token::unpack_float(&tok)),
        _ => return None,
    })
}

/// What a node of the body walk is building.
#[derive(Default, Debug)]
pub(crate) enum Building {
    /// Nothing: a chunk, which writes into its parent, or a scalar, which
    /// was handed to its parent the moment it was read.
    #[default]
    Nothing,
    /// A container being filled.
    Value(Object),
    /// A `str` or `bin` that is a map's key, collected straight into one.
    Key(DictKey),
    /// A string value's bytes so far.
    Text(XString),
    /// An extension object: its payload so far, and the value it decodes
    /// to — nil until the last chunk lands, or for good once it overflows.
    /// Boxed: it is rare, and every node would otherwise carry its payload.
    Ext(Box<ExtPayload>),
}

/// An extension object being assembled; see [`Building::Ext`].
#[derive(Debug)]
pub(crate) struct ExtPayload {
    bytes: [u8; EXT_PAYLOAD_MAX],
    value: Object,
}

/// The body walk: every node builds its object in its own `data` and hands
/// it to its parent on the way out; the root's lands in `result`. A scalar
/// is handed over on the way in, as it is complete the moment it is read.
pub(crate) struct BodyWalk<'a> {
    result: &'a mut Object,
}

impl BodyWalk<'_> {
    /// Hand a finished value to `parent`, or make it the result when there
    /// is none. `is_key` says that the parent is a map expecting a key.
    #[inline]
    fn store(&mut self, parent: Option<&mut Node<Building>>, value: Building, is_key: bool) {
        let Some(parent) = parent else {
            if let Some(value) = finished(value) {
                *self.result = value;
            }
            return;
        };
        match &mut parent.data {
            Building::Value(Object::Array(items)) => {
                items.push(finished(value).unwrap_or(Object::Nil));
            }
            Building::Value(Object::Dict(entries)) => {
                if is_key {
                    // Only a string can be a key; anything else leaves the
                    // entry's key empty, as it always has.
                    let key = match value {
                        Building::Key(key) => key,
                        _ => DictKey::EMPTY,
                    };
                    entries.push(KeyValuePair {
                        key,
                        value: Object::Nil,
                    });
                } else if let Some(entry) = entries.last_mut() {
                    entry.value = finished(value).unwrap_or(Object::Nil);
                }
            }
            // A blob's only children are its chunks, which wrote themselves
            // in on the way down.
            _ => {}
        }
    }
}

impl crate::mpack::object::Decode<Building> for BodyWalk<'_> {
    #[inline]
    fn enter(&mut self, tok: Tok, parents: &mut [Node<Building>], chunk: &[u8]) -> Building {
        let parent = parents.last_mut();
        // Whether the parent is a map that is about to be handed a key: on
        // the way in, before the pop has flipped its phase.
        let is_key = parent
            .as_ref()
            .is_some_and(|parent| token_type(&parent.tok) == TOKEN_MAP && !parent.key_visited);
        if let Some(object) = scalar_object(tok) {
            self.store(parent, Building::Value(object), is_key);
            return Building::Nothing;
        }
        let len = tok.len as usize;
        match token_type(&tok) {
            // A key goes straight into its slot's own storage; a value
            // collects into a string the API will hand out terminated.
            TOKEN_BIN | TOKEN_STR if is_key => Building::Key(DictKey::zeroed(len)),
            TOKEN_BIN | TOKEN_STR => Building::Text(XString::with_capacity(len)),
            TOKEN_EXT => Building::Ext(Box::new(ExtPayload {
                bytes: [0; EXT_PAYLOAD_MAX],
                value: Object::Nil,
            })),
            TOKEN_ARRAY => {
                Building::Value(Object::array(Array::with_capacity(len.min(RESERVE_MAX))))
            }
            TOKEN_MAP => {
                Building::Value(Object::dict(ApiDict::with_capacity(len.min(RESERVE_MAX))))
            }
            TOKEN_CHUNK => {
                copy_chunk(parent.expect("a chunk always has a parent"), chunk);
                Building::Nothing
            }
            _ => Building::Nothing,
        }
    }

    #[inline]
    fn exit(&mut self, node: Node<Building>, parents: &mut [Node<Building>]) {
        if matches!(node.data, Building::Nothing) {
            return;
        }
        let parent = parents.last_mut();
        // The pop has already flipped the phase: a map that now expects a
        // value has just been handed its key.
        let is_key = parent.as_ref().is_some_and(|parent| parent.key_visited);
        self.store(parent, node.data, is_key);
    }
}

/// The object a completed node built, if it built one.
#[inline]
fn finished(data: Building) -> Option<Object> {
    match data {
        Building::Value(value) => Some(value),
        Building::Ext(ext) => Some(ext.value),
        Building::Text(text) => Some(Object::string(String_0::from_xstring(text))),
        Building::Key(_) | Building::Nothing => None,
    }
}

/// Copies one chunk into whatever its parent is collecting.
///
/// A string or blob appends to its bytes; an extension accumulates into its
/// fixed payload buffer and becomes a handle once the last chunk lands. A
/// payload longer than [`EXT_PAYLOAD_MAX`] decodes as nil rather than
/// failing the message.
fn copy_chunk(parent: &mut Node<Building>, chunk: &[u8]) {
    let pos = parent.pos;
    let total = parent.tok.len as usize;
    let ext_type = parent.tok.lo.cast_signed();
    match &mut parent.data {
        Building::Text(text) => text.push_bytes(chunk),
        Building::Key(key) => key.bytes_mut()[pos..pos + chunk.len()].copy_from_slice(chunk),
        Building::Ext(ext) => {
            if pos + chunk.len() > EXT_PAYLOAD_MAX {
                ext.value = Object::Nil;
                return;
            }
            ext.bytes[pos..pos + chunk.len()].copy_from_slice(chunk);
            if pos + chunk.len() >= total {
                ext.value = ext_object(&ext.bytes[..total], ext_type);
            }
        }
        _ => unreachable!("a chunk's parent is a string, a blob or an extension"),
    }
}

/// Turns a complete extension payload into the handle it stands for.
///
/// Only the three handle types are recognised; anything else — including a
/// payload that is not a plain unsigned integer — decodes as nil rather than
/// failing the message.
fn ext_object(payload: &[u8], ext_type: c_int) -> Object {
    let mut rest = payload;
    let (status, tok) = read_token(&mut rest);
    if status != MPACK_OK || token_type(&tok) != TOKEN_UINT {
        return Object::Nil;
    }
    let handle = token::unpack_uint(&tok).cast_signed();
    match i8::try_from(ext_type) {
        Ok(EXT_BUFFER) => Object::Buffer(handle),
        Ok(EXT_WINDOW) => Object::Window(handle),
        Ok(EXT_TABPAGE) => Object::Tabpage(handle),
        _ => Object::Nil,
    }
}

/// The value of an integer token, whichever sign it was encoded with.
pub(super) fn unpack_integer_token(tok: Tok) -> Option<Integer> {
    match token_type(&tok) {
        TOKEN_UINT => Some(token::unpack_uint(&tok).cast_signed()),
        TOKEN_SINT => Some(token::unpack_sint(&tok)),
        _ => None,
    }
}
