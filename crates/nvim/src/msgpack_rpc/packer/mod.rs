#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Serialisation of API objects to msgpack.
//!
//! The packer writes into a [`PackerBuffer`] — an owned block with a `flush`
//! hook the owner supplies (a socket write for RPC, growth for the string
//! buffer below). Room is not checked per byte: [`mpack_check_buffer`]
//! guarantees room for two whole items, and the header/scalar writers spend
//! at most one of those, so a caller only has to call it once per item. A
//! write past the block's end grows the block rather than overrunning it,
//! so a caller that miscounts costs an allocation, not memory safety.
//!
//! The encodings themselves live in [`mod@format`], which is pure and tested
//! against fixed byte sequences.

use core::ffi::{c_double, c_int, c_void};

use crate::lua::executor::release_luaref;
use crate::types::{
    Array, Handle, Integer, KeyValuePair, LuaRef, Object, int8_t, int64_t, size_t, uint32_t,
    uint64_t,
};

pub mod format;

pub type PackerBufferFlush = fn(&mut PackerBuffer);

/// A block the msgpack packer writes into, and the hook that makes room again
/// when it is nearly full.
///
/// The block is a `Vec<u8>` whose capacity is the window: `len` is how much
/// has been packed, `capacity - len` how much room is left. A detached
/// packer holds no block (capacity 0) until its owner [`adopt`](Self::adopt)s
/// one.
pub struct PackerBuffer {
    block: Vec<u8>,
    /// The channels an RPC packer's flush writes to.
    pub(crate) channels: Addressees,
    /// Whatever else the flush hook needs to find its owner: a `RemoteUI`,
    /// or nothing.
    pub anydata: *mut c_void,
    /// A second word for the same hook: the status the last flush returned.
    pub anyint: int64_t,
    pub packer_flush: Option<PackerBufferFlush>,
}

impl PackerBuffer {
    /// A packer over `block`, which is empty and whose capacity is the
    /// window.
    pub fn new(block: Vec<u8>, packer_flush: Option<PackerBufferFlush>) -> Self {
        debug_assert!(block.is_empty());
        Self {
            block,
            channels: Addressees::Nobody,
            anydata: ::core::ptr::null_mut(),
            anyint: 0,
            packer_flush,
        }
    }

    /// A packer holding no block at all, for an owner that allocates one
    /// lazily.
    pub const fn detached(anydata: *mut c_void, packer_flush: Option<PackerBufferFlush>) -> Self {
        Self {
            block: Vec::new(),
            channels: Addressees::Nobody,
            anydata,
            anyint: 0,
            packer_flush,
        }
    }

    /// Whether the packer holds a block at all.
    pub fn is_detached(&self) -> bool {
        self.block.capacity() == 0
    }

    /// Take `block` as the new window, discarding whatever the packer held.
    pub fn adopt(&mut self, block: Vec<u8>) {
        debug_assert!(block.is_empty());
        self.block = block;
    }

    /// Give the block up, packed bytes and all, leaving the packer detached.
    pub fn take(&mut self) -> Vec<u8> {
        core::mem::take(&mut self.block)
    }

    /// The bytes packed so far.
    pub fn bytes(&self) -> &[u8] {
        &self.block
    }

    /// How many bytes have been packed into the block.
    pub fn used(&self) -> usize {
        self.block.len()
    }

    /// How much room is left before the block has to be flushed.
    pub fn remaining(&self) -> usize {
        self.block.capacity() - self.block.len()
    }

    /// The whole block, packed or not.
    pub fn capacity(&self) -> usize {
        self.block.capacity()
    }

    /// Append `bytes`.
    #[inline(always)]
    pub fn push(&mut self, bytes: &[u8]) {
        self.block.extend_from_slice(bytes);
    }

    /// Append one byte.
    #[inline(always)]
    pub fn push_byte(&mut self, byte: u8) {
        self.block.push(byte);
    }

    /// Overwrite the 16-bit placeholder at `at`, most significant byte
    /// first: a header written before its length was known.
    pub fn patch_be16(&mut self, at: usize, value: uint32_t) {
        self.block[at..at + 2].copy_from_slice(&value.to_be_bytes()[2..]);
    }
}

/// Who an RPC message goes to: almost always one channel, which needs no
/// allocation; a broadcast names them all.
#[derive(Default, Debug)]
pub(crate) enum Addressees {
    #[default]
    Nobody,
    One(uint64_t),
    Many(Vec<uint64_t>),
}

impl Addressees {
    /// The channel ids.
    pub(crate) fn as_slice(&self) -> &[uint64_t] {
        match self {
            Addressees::Nobody => &[],
            Addressees::One(id) => core::slice::from_ref(id),
            Addressees::Many(ids) => ids,
        }
    }
}

pub const LUA_NOREF: c_int = -2;

/// The most bytes one msgpack item's tag and inline payload can take.
pub const MPACK_ITEM_SIZE: c_int = 9;

/// Appends an encoded item.
///
/// The block must have [`MPACK_ITEM_SIZE`] bytes of room, which is what
/// [`mpack_check_buffer`] leaves behind.
#[inline(always)]
fn emit(packer: &mut PackerBuffer, bytes: &[u8]) {
    packer.push(bytes);
}

/// Hands the buffer back to its owner to make room, then resumes in
/// whatever block the owner set up.
fn flush(packer: &mut PackerBuffer) {
    let hook = packer.packer_flush.expect("packer has no flush hook");
    hook(packer);
}

/// The low 16 bits of `value`, most significant byte first.
#[inline]
pub fn mpack_be16(packer: &mut PackerBuffer, value: uint32_t) {
    emit(packer, &value.to_be_bytes()[2..]);
}

/// All 32 bits of `value`, most significant byte first.
#[inline]
pub fn mpack_be32(packer: &mut PackerBuffer, value: uint32_t) {
    emit(packer, &value.to_be_bytes());
}

#[inline]
pub fn mpack_uint(packer: &mut PackerBuffer, value: uint32_t) {
    emit(packer, format::uint(value).bytes());
}

#[inline]
pub fn mpack_uint64(packer: &mut PackerBuffer, value: uint64_t) {
    emit(packer, format::uint64(value).bytes());
}

#[inline]
pub fn mpack_integer(packer: &mut PackerBuffer, value: Integer) {
    emit(packer, format::integer(value).bytes());
}

#[inline]
pub fn mpack_float8(packer: &mut PackerBuffer, value: c_double) {
    emit(packer, format::float8(value).bytes());
}

#[inline]
pub fn mpack_bool(packer: &mut PackerBuffer, value: bool) {
    emit(packer, format::boolean(value).bytes());
}

#[inline]
pub fn mpack_nil(packer: &mut PackerBuffer) {
    emit(packer, &[format::NIL]);
}

#[inline]
pub fn mpack_array(packer: &mut PackerBuffer, len: uint32_t) {
    emit(packer, format::array_header(len).bytes());
}

#[inline]
pub fn mpack_map(packer: &mut PackerBuffer, len: uint32_t) {
    emit(packer, format::map_header(len).bytes());
}

/// Writes a 16-bit array header whose length is filled in later, and returns
/// the offset to fill it in at.
///
/// The UI event stream is built by appending to an array whose size is not
/// known until it is flushed, so the header goes out with a placeholder and is
/// overwritten in place with [`PackerBuffer::patch_be16`].
pub fn mpack_array_dyn16(packer: &mut PackerBuffer) -> usize {
    emit(packer, &[format::ARRAY16]);
    let pos = packer.used();
    // A recognisable placeholder rather than zero: a header left unpatched
    // shows up as a decoder error rather than as a silently empty array.
    mpack_be16(packer, 0xffef);
    pos
}

/// Writes a string that is known to fit a fixstr header.
///
/// Every caller passes a UI event or method name, all of which are well under
/// the 31-byte limit, so the width choice the general [`mpack_str`] makes is
/// an assertion here instead.
#[inline]
pub fn mpack_str_small(packer: &mut PackerBuffer, str: &[u8]) {
    emit(packer, format::fixstr_header(str.len()).bytes());
    emit(packer, str);
}

/// How much room is left before the buffer has to be flushed.
#[inline]
pub fn mpack_remaining(packer: &PackerBuffer) -> size_t {
    packer.remaining()
}

/// Makes room for two more items, flushing if the buffer is nearly full.
///
/// Two rather than one because a dict entry writes its key and its value
/// between checks.
#[inline]
pub fn mpack_check_buffer(packer: &mut PackerBuffer) {
    if mpack_remaining(packer) < 2 * MPACK_ITEM_SIZE as size_t {
        flush(packer);
    }
}

/// A container's element count as msgpack carries it.
///
/// The panic is unreachable: 2^32 `Object`s is 128 GiB of arena. It matches
/// the `expect` the string and extension headers already carry for the same
/// reason.
fn container_len(size: size_t) -> uint32_t {
    uint32_t::try_from(size).expect("container too long for msgpack")
}

/// Packs `bytes` as a msgpack string.
pub fn mpack_str(bytes: &[u8], packer: &mut PackerBuffer) {
    let header = format::str_header(bytes.len()).expect("string too long for msgpack");
    emit(packer, header.bytes());
    mpack_raw(bytes, packer);
}

/// Packs `bytes` as a msgpack binary.
pub fn mpack_bin(bytes: &[u8], packer: &mut PackerBuffer) {
    let header = format::bin_header(bytes.len()).expect("blob too long for msgpack");
    emit(packer, header.bytes());
    mpack_raw(bytes, packer);
}

/// Copies opaque bytes, flushing as often as it takes.
///
/// Leaves the buffer ready for another item, so callers that follow a payload
/// with more structure do not need their own check.
#[inline]
pub fn mpack_raw(data: &[u8], packer: &mut PackerBuffer) {
    // The common case: it fits.
    if data.len() <= mpack_remaining(packer) {
        packer.push(data);
        mpack_check_buffer(packer);
        return;
    }
    let mut rest = data;
    while !rest.is_empty() {
        let (now, later) = rest.split_at(rest.len().min(mpack_remaining(packer)));
        packer.push(now);
        rest = later;
        if !rest.is_empty() {
            flush(packer);
        }
    }
    mpack_check_buffer(packer);
}

/// An extension object: the header, the type byte, then the raw bytes.
pub fn mpack_ext(bytes: &[u8], ext_type: int8_t, packer: &mut PackerBuffer) {
    let header = format::ext_header(bytes.len(), ext_type).expect("extension too long for msgpack");
    emit(packer, header.bytes());
    mpack_raw(bytes, packer);
}

/// A buffer, window or tabpage handle. `ext_type` is the msgpack extension
/// type the wire gives that kind of handle: the variant's distance from
/// [`Object::Buffer`], so the three are 0, 1 and 2.
#[inline]
pub fn mpack_handle(ext_type: int8_t, handle: Handle, packer: &mut PackerBuffer) {
    emit(packer, format::handle(ext_type, handle).bytes());
}

/// The three extension types, spelled once so the packer's arms and the
/// unpacker's agree.
pub(crate) const EXT_BUFFER: int8_t = 0;
pub(crate) const EXT_WINDOW: int8_t = 1;
pub(crate) const EXT_TABPAGE: int8_t = 2;

/// Packs `obj`. A luaref inside it is released and packs as nil.
pub fn mpack_object(obj: &mut Object, packer: &mut PackerBuffer) {
    mpack_object_inner(obj, packer);
}

/// Packs an array's elements without wrapping them in another object.
pub fn mpack_object_array(arr: &mut Array, packer: &mut PackerBuffer) {
    mpack_array(packer, container_len(arr.len()));
    for item in arr.iter_mut() {
        mpack_object_inner(item, packer);
    }
}

/// A container whose remaining elements the walk will come back for.
enum Resume<'a> {
    Array(core::slice::IterMut<'a, Object>),
    Dict(core::slice::IterMut<'a, KeyValuePair>),
}

impl Resume<'_> {
    fn is_exhausted(&self) -> bool {
        match self {
            Resume::Array(items) => items.len() == 0,
            Resume::Dict(entries) => entries.len() == 0,
        }
    }
}

/// Walks `root` and everything below it, iteratively.
///
/// `container` is where to resume once the current object is done; deeper
/// containers are remembered on a stack that stays off the heap for the
/// first two levels. A container whose last element is being walked is not
/// remembered at all, which is what keeps deeply nested one-element arrays
/// from growing the stack.
fn mpack_object_inner(root: &mut Object, packer: &mut PackerBuffer) {
    let mut stack: format::SmallStack<Resume<'_>> = format::SmallStack::default();
    let mut container: Option<Resume<'_>> = None;
    let mut current = root;
    loop {
        mpack_check_buffer(packer);
        // Everything that is not a container writes itself here and moves on;
        // nil, and a luaref once it has been released, write the nil byte.
        let opened = match current {
            Object::LuaRef(reference) => {
                // A released luaref packs as nil, and the slot is overwritten
                // so a second pass cannot free it twice.
                release_luaref(*reference);
                *reference = LUA_NOREF as LuaRef;
                mpack_nil(packer);
                None
            }
            Object::Nil => {
                mpack_nil(packer);
                None
            }
            Object::Boolean(value) => {
                mpack_bool(packer, *value);
                None
            }
            Object::Integer(value) => {
                mpack_integer(packer, *value);
                None
            }
            Object::Float(value) => {
                mpack_float8(packer, *value);
                None
            }
            Object::String(value) => {
                mpack_str(value.as_bytes(), packer);
                None
            }
            Object::Buffer(handle) => {
                mpack_handle(EXT_BUFFER, crate::narrow::number_as_int(*handle), packer);
                None
            }
            Object::Window(handle) => {
                mpack_handle(EXT_WINDOW, crate::narrow::number_as_int(*handle), packer);
                None
            }
            Object::Tabpage(handle) => {
                mpack_handle(EXT_TABPAGE, crate::narrow::number_as_int(*handle), packer);
                None
            }
            Object::Array(array) => {
                mpack_array(packer, container_len(array.len()));
                (!array.is_empty()).then(|| Resume::Array(array.iter_mut()))
            }
            Object::Dict(dict) => {
                mpack_map(packer, container_len(dict.len()));
                (!dict.is_empty()).then(|| Resume::Dict(dict.iter_mut()))
            }
        };
        if let Some(opened) = opened {
            if let Some(parent) = container.take()
                && !parent.is_exhausted()
            {
                stack.push(parent);
            }
            container = Some(opened);
        }

        // The next element of the innermost unfinished container.
        current = loop {
            let Some(resume) = container.as_mut() else {
                match stack.pop() {
                    Some(parent) => {
                        container = Some(parent);
                        continue;
                    }
                    None => return,
                }
            };
            match resume {
                Resume::Array(items) => match items.next() {
                    Some(item) => break item,
                    None => container = None,
                },
                Resume::Dict(entries) => match entries.next() {
                    Some(entry) => {
                        mpack_check_buffer(packer);
                        mpack_str(entry.key.bytes(), packer);
                        break &mut entry.value;
                    }
                    None => container = None,
                },
            }
        };
    }
}

/// The initial window of a [`packer_string_buffer`].
const STRING_BUFFER_INITIAL: size_t = 64;

/// A buffer that grows on the heap instead of flushing anywhere, for callers
/// that want the packed bytes rather than a stream.
pub fn packer_string_buffer() -> PackerBuffer {
    PackerBuffer::new(
        Vec::with_capacity(STRING_BUFFER_INITIAL),
        Some(flush_string_buffer),
    )
}

/// Doubles a [`packer_string_buffer`]'s block, keeping what was packed.
fn flush_string_buffer(buffer: &mut PackerBuffer) {
    let grown = 2 * buffer.capacity();
    let len = buffer.used();
    buffer.block.reserve_exact(grown - len);
}

/// Pack with `pack` into a fresh [`packer_string_buffer`] and answer the
/// bytes, which keep the buffer's own allocation.
pub fn pack_to_bytes(pack: impl FnOnce(&mut PackerBuffer)) -> Vec<u8> {
    let mut packer = packer_string_buffer();
    pack(&mut packer);
    packer.take()
}
