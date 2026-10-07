//! msgpack bytes into a `TypVal`: upstream's `mpack_parse()` walk with its
//! two typval callbacks, as one owned state machine.
//!
//! Tokens come off the safe tokenizer in [`crate::mpack::mpack_core`]; the
//! walk keeps an explicit stack of the containers still open, at most
//! [`MAX_DEPTH`] deep as upstream's parser is. Each node *owns* what it is
//! building -- a list, a map's decoded pairs, a string's bytes -- and hands
//! the finished value to the node below it when it closes, so nothing is ever
//! half-written into a slot, and a parse abandoned part-way simply drops what
//! it had.
//!
//! What a node becomes is upstream's: most values are finished as their token
//! arrives; `str`/`bin`/`ext` wait for their bytes, which arrive as chunks;
//! and a `map` waits for every pair, because whether it can be a `Dict` is
//! only knowable once every key has been decoded.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{
    create_special_dict, decode_create_map_special_dict, decode_owned_string, kMPExt, kMPInteger,
};
use crate::eval::encode::encode_list_write;
use crate::eval::typval::{ListRef, tv_dict_alloc, tv_list_alloc};
use crate::mpack::mpack_core::{MPACK_EOF, MPACK_ERROR, MPACK_OK, Step, empty_tokbuf, read_step};
use crate::mpack::token::{Kind, Tok, unpack_boolean, unpack_float, unpack_sint, unpack_uint};
use crate::types::{
    DictItem, TypVal, VarNumber, kBoolVarFalse, kBoolVarTrue, kListLenMayKnow, kSpecialVarNull,
    mpack_tokbuf_t,
};

/// How deep a value may nest, counting each string's bytes as one more
/// level: upstream's `MPACK_MAX_OBJECT_DEPTH`.
const MAX_DEPTH: usize = 32;

/// The status past `MPACK_ERROR`: the value nests deeper than [`MAX_DEPTH`].
pub(crate) const MPACK_NOMEM: c_int = MPACK_ERROR.cast_signed() + 1;

/// The most a container's header may reserve before its items arrive.
///
/// A header is only a claim, and a truncated or hostile one can claim four
/// billion items; the list grows past this as items really arrive.
const MAX_RESERVE: usize = 1 << 12;

/// One value still being built.
enum Node {
    Array {
        list: ListRef,
        len: u32,
    },
    /// The decoded `[key, value, key, value, ...]`, and whether the next
    /// value is a pair's key's value.
    Map {
        pairs: Vec<TypVal>,
        len: u32,
        key_visited: bool,
    },
    /// A `str`, `bin` or `ext`, its bytes arriving in chunks.
    Bytes {
        kind: Kind,
        ext_type: u32,
        bytes: Vec<u8>,
        len: u32,
    },
}

impl Node {
    /// Whether every child this node announced has arrived.
    fn is_complete(&self) -> bool {
        match self {
            Node::Array { list, len } => list.items().len() >= *len as usize,
            Node::Map { pairs, len, .. } => pairs.len() >= 2 * (*len as usize),
            Node::Bytes { bytes, len, .. } => bytes.len() >= *len as usize,
        }
    }

    /// The finished value.
    fn finish(self) -> TypVal {
        match self {
            Node::Array { list, .. } => TypVal::list(Some(list)),
            Node::Map { pairs, .. } => map_value(pairs),
            Node::Bytes {
                kind: Kind::Ext,
                ext_type,
                bytes,
                ..
            } => {
                // `{_TYPE: ext, _VAL: [type, [bytes…]]}`.  The payload goes
                // into a list of strings rather than a blob, as upstream's
                // TODO notes.
                let mut list = tv_list_alloc(2);
                list.push_number(VarNumber::from(ext_type));
                let mut payload = tv_list_alloc(kListLenMayKnow as isize);
                encode_list_write(&mut payload, &bytes);
                list.push_list(Some(payload));
                create_special_dict(kMPExt, TypVal::list(Some(list)))
            }
            Node::Bytes { bytes, .. } => decode_owned_string(bytes),
        }
    }
}

/// A msgpack unsigned integer as a `TypVal`.
///
/// Anything a `VarNumber` can hold is a plain number.  What it cannot is
/// split across a four-element `{_TYPE: integer, _VAL: [sign, hi, mid, lo]}`
/// list — one sign, then 2 + 31 + 31 bits — which is the same shape the
/// msgpack encoder reads back.
fn positive_integer_to_special_typval(val: u64) -> TypVal {
    if let Ok(number) = VarNumber::try_from(val) {
        return TypVal::Number(number);
    }
    let mut list = tv_list_alloc(4);
    for piece in [
        1,
        (val >> 62) & 0x3,
        (val >> 31) & 0x7fff_ffff,
        val & 0x7fff_ffff,
    ] {
        list.push_number(VarNumber::try_from(piece).expect("at most 31 bits"));
    }
    create_special_dict(kMPInteger, TypVal::list(Some(list)))
}

/// A decoded map: a `Dict` when every key is a non-empty string used once,
/// and the special map otherwise.
fn map_value(mut pairs: Vec<TypVal>) -> TypVal {
    if pairs
        .iter()
        .step_by(2)
        .all(|key| key.string_ref().is_some_and(|key| !key.is_empty()))
    {
        let mut dict = tv_dict_alloc();
        let mut added = 0;
        while added < pairs.len() {
            let mut item = DictItem::boxed(pairs[added].string_bytes());
            item.di_tv = pairs[added + 1].take();
            if let Err(mut refused) = dict.add_item(item) {
                // A duplicate key.  Hand every value already added back to
                // the pair it came from, for the special map below.
                pairs[added + 1] = refused.di_tv.take();
                for at in (0..added).step_by(2) {
                    let back = dict.edit().remove_key(pairs[at].string_bytes());
                    pairs[at + 1] = back.expect("a key just added").value_mut().take();
                }
                break;
            }
            added += 2;
        }
        if added == pairs.len() {
            return TypVal::dict(Some(dict));
        }
    }
    let count = pairs.len() / 2;
    let (special, mut list) = decode_create_map_special_dict(count.cast_signed());
    let mut pairs = pairs.into_iter();
    while let (Some(key), Some(value)) = (pairs.next(), pairs.next()) {
        let mut kv_pair = tv_list_alloc(2);
        kv_pair.push(key);
        kv_pair.push(value);
        list.push_list(Some(kv_pair));
    }
    special
}

/// A streaming msgpack-to-typval decoder: one object at a time, fed as many
/// slices as it takes.
pub(crate) struct MsgpackDecoder {
    tokbuf: mpack_tokbuf_t,
    stack: Vec<Node>,
}

impl MsgpackDecoder {
    pub(crate) fn new() -> Self {
        MsgpackDecoder {
            tokbuf: empty_tokbuf(),
            stack: Vec::new(),
        }
    }

    /// Hand a finished value to the node below it, or answer it as the
    /// whole object when there is none; then close every node that value
    /// completed.
    fn deliver(&mut self, mut value: TypVal) -> Option<TypVal> {
        loop {
            match self.stack.last_mut() {
                None => return Some(value),
                Some(Node::Array { list, .. }) => {
                    list.push(value);
                }
                Some(Node::Map {
                    pairs, key_visited, ..
                }) => {
                    pairs.push(value);
                    *key_visited = !*key_visited;
                }
                Some(Node::Bytes { .. }) => unreachable!("a string's children are its bytes"),
            }
            if !self.stack.last().is_some_and(Node::is_complete) {
                return None;
            }
            value = self.stack.pop().expect("the node just completed").finish();
        }
    }

    /// One token: open a node for it, or finish a value with it. Answers the
    /// whole object once the token completes it.
    fn token(&mut self, tok: Tok) -> Result<Option<TypVal>, c_int> {
        // Every token takes a level while it is entered, a scalar too.
        if self.stack.len() == MAX_DEPTH {
            return Err(MPACK_NOMEM);
        }
        let len = tok.len;
        let value = match tok.kind {
            Some(Kind::Nil) => TypVal::Special(kSpecialVarNull),
            Some(Kind::Boolean) => TypVal::Bool(if unpack_boolean(&tok) {
                kBoolVarTrue
            } else {
                kBoolVarFalse
            }),
            Some(Kind::Sint) => TypVal::Number(unpack_sint(&tok)),
            Some(Kind::Uint) => positive_integer_to_special_typval(unpack_uint(&tok)),
            Some(Kind::Float) => TypVal::Float(unpack_float(&tok)),
            Some(Kind::Array) => {
                let reserve = (len as usize).min(MAX_RESERVE);
                let node = Node::Array {
                    list: tv_list_alloc(reserve.cast_signed()),
                    len,
                };
                return Ok(self.open(node));
            }
            Some(Kind::Map) => {
                let node = Node::Map {
                    pairs: Vec::with_capacity(2 * (len as usize).min(MAX_RESERVE)),
                    len,
                    key_visited: false,
                };
                return Ok(self.open(node));
            }
            Some(kind @ (Kind::Bin | Kind::Str | Kind::Ext)) => {
                // One more byte than the payload, for the string's NUL.
                let reserve = (len as usize).min(MAX_RESERVE << 4) + 1;
                let node = Node::Bytes {
                    kind,
                    ext_type: tok.lo,
                    bytes: Vec::with_capacity(reserve),
                    len,
                };
                return Ok(self.open(node));
            }
            Some(Kind::Chunk) | None => unreachable!("the tokenizer answers chunks apart"),
        };
        Ok(self.deliver(value))
    }

    /// Push `node`, or finish it at once when it announced no children.
    fn open(&mut self, node: Node) -> Option<TypVal> {
        if node.is_complete() {
            return self.deliver(node.finish());
        }
        self.stack.push(node);
        None
    }

    /// The next piece of the innermost string's bytes.
    fn chunk(&mut self, data: &[u8]) -> Result<Option<TypVal>, c_int> {
        // The chunk is a node of its own while it is copied in.
        if self.stack.len() == MAX_DEPTH {
            return Err(MPACK_NOMEM);
        }
        let Some(Node::Bytes { bytes, .. }) = self.stack.last_mut() else {
            unreachable!("a chunk follows a string header");
        };
        bytes.extend_from_slice(data);
        if !self.stack.last().is_some_and(Node::is_complete) {
            return Ok(None);
        }
        let value = self.stack.pop().expect("the node just completed").finish();
        Ok(self.deliver(value))
    }

    /// Decode as much of `data` as it takes to finish one object, advancing
    /// `data` past what was used.
    ///
    /// `Ok(Some(value))` is a whole object; `Ok(None)` (upstream's
    /// `MPACK_EOF`) says `data` ran out first, everything in it consumed, and
    /// the next call resumes where this one stopped. `Err` is `MPACK_ERROR`
    /// for bytes that are not msgpack and [`MPACK_NOMEM`] for a value nested
    /// too deep; the decoder is not to be fed again after either.
    pub(crate) fn feed(&mut self, data: &mut &[u8]) -> Result<Option<TypVal>, c_int> {
        while !data.is_empty() {
            let (step, used) = read_step(&mut self.tokbuf, data);
            let (taken, rest) = data.split_at(used);
            let done = match step {
                Step::Eof => None,
                Step::Error => return Err(MPACK_ERROR.cast_signed()),
                Step::Token(tok) => self.token(tok)?,
                Step::Chunk(_) => self.chunk(taken)?,
            };
            *data = rest;
            if done.is_some() {
                return Ok(done);
            }
        }
        Ok(None)
    }
}

/// Decode one complete msgpack object from `data` into `ret`.
///
/// `data` is advanced past the object.  Answers `MPACK_OK`, `MPACK_EOF` when
/// the bytes ran out mid-object, or an error status; on anything but
/// `MPACK_OK` the half-built value is dropped and `ret` is left
/// `VAR_UNKNOWN`.
pub(crate) fn unpack_typval(data: &mut &[u8], ret: &mut TypVal) -> c_int {
    ret.overwrite(TypVal::Unknown);
    match MsgpackDecoder::new().feed(data) {
        Ok(Some(value)) => {
            ret.overwrite(value);
            MPACK_OK.cast_signed()
        }
        Ok(None) => MPACK_EOF.cast_signed(),
        Err(status) => status,
    }
}
