#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Serialisation of API objects to msgpack.
//!
//! The packer writes into a [`PackerBuffer`] — a window of memory with a `flush`
//! hook the owner supplies (a socket write for RPC, a `realloc` for the string
//! buffer below). Buffer space is not checked per byte: [`mpack_check_buffer`]
//! guarantees room for two whole items, and the header/scalar writers spend at
//! most one of those, so a caller only has to call it once per item.
//!
//! The encodings themselves live in [`format`], which is pure and tested
//! against fixed byte sequences.

use core::ffi::{c_char, c_double, c_int, c_void};

use crate::lua::executor::api_free_luaref;
use crate::memory::{xmalloc, xrealloc};
use crate::types::{
    Array, Handle, Integer, KeyValuePair, LuaRef, Object, String_0, int8_t, int64_t, size_t,
    uint32_t, uint64_t,
};

pub mod format;

pub type PackerBufferFlush = Option<unsafe fn(*mut PackerBuffer) -> ()>;

/// A window of memory the msgpack packer writes into, and the hook that
/// widens it again when the cursor reaches the end.
///
/// The three pointers are the invariant, and they are private because of it:
/// `start <= cursor <= end`, all three into one live allocation the owner
/// keeps alive for as long as the packer exists. Every write helper in
/// `msgpack_rpc::packer` is a *safe* `fn` that trusts that triple without
/// checking it, so a `PackerBuffer` assembled out of unrelated pointers would
/// make safe code write through them. Assembling one is therefore the unsafe
/// step ([`PackerBuffer::new`]); moving the cursor inside a window that is
/// already established is not.
pub struct PackerBuffer {
    start: *mut ::core::ffi::c_char,
    cursor: *mut ::core::ffi::c_char,
    end: *mut ::core::ffi::c_char,
    /// Whatever the flush hook needs to find its owner: a channel list, a
    /// `FileDescriptor`, a `RemoteUI`, or nothing.
    pub anydata: *mut ::core::ffi::c_void,
    /// A second word for the same hook -- an addressee count, or the status
    /// the last flush returned.
    pub anyint: int64_t,
    pub packer_flush: PackerBufferFlush,
}

impl PackerBuffer {
    /// A packer over the `capacity` bytes at `start`.
    ///
    /// # Safety
    ///
    /// `start` must point at `capacity` writable bytes the caller owns, and
    /// they must stay alive and unaliased until the packer is dropped or
    /// [`release`](Self::release)d. Nothing here checks the window: it is the
    /// bound every safe write below is measured against.
    pub unsafe fn new(
        start: *mut ::core::ffi::c_char,
        capacity: usize,
        packer_flush: PackerBufferFlush,
    ) -> Self {
        Self {
            start,
            cursor: start,
            end: start.wrapping_add(capacity),
            anydata: ::core::ptr::null_mut(),
            anyint: 0,
            packer_flush,
        }
    }

    /// A packer holding no window at all, for an owner that allocates its
    /// block lazily. Safe because the empty window bounds nothing: every
    /// write below refuses to run until [`adopt`](Self::adopt) gives it one.
    pub const fn detached(
        anydata: *mut ::core::ffi::c_void,
        packer_flush: PackerBufferFlush,
    ) -> Self {
        Self {
            start: ::core::ptr::null_mut(),
            cursor: ::core::ptr::null_mut(),
            end: ::core::ptr::null_mut(),
            anydata,
            anyint: 0,
            packer_flush,
        }
    }

    /// Whether the packer holds a window at all.
    pub fn is_detached(&self) -> bool {
        self.start.is_null()
    }

    /// Take the `capacity` bytes at `start` as the new window, discarding
    /// whatever the packer pointed at before.
    ///
    /// # Safety
    ///
    /// As [`new`](Self::new): the caller owns those bytes and keeps them
    /// alive, and anything the packer held before has been dealt with.
    pub unsafe fn adopt(&mut self, start: *mut ::core::ffi::c_char, capacity: usize) {
        self.start = start;
        self.cursor = start;
        self.end = start.wrapping_add(capacity);
    }

    /// Give up the window without touching it -- for an owner that has just
    /// handed the block to someone else, or freed it.
    pub fn release(&mut self) {
        self.start = ::core::ptr::null_mut();
        self.cursor = ::core::ptr::null_mut();
        self.end = ::core::ptr::null_mut();
    }

    /// Re-point the window after the owner's flush hook has reallocated it.
    ///
    /// # Safety
    ///
    /// As [`new`](Self::new), with the extra promise that the first
    /// [`used`](Self::used) bytes of the new window carry what was packed so
    /// far: the cursor is kept at that offset.
    pub unsafe fn moved_to(&mut self, start: *mut ::core::ffi::c_char, capacity: usize) {
        let used = self.used();
        self.start = start;
        self.cursor = start.wrapping_add(used);
        self.end = start.wrapping_add(capacity);
    }

    /// The first byte of the window. Null when detached.
    pub fn start(&self) -> *mut ::core::ffi::c_char {
        self.start
    }

    /// Where the next byte goes.
    pub fn cursor(&self) -> *mut ::core::ffi::c_char {
        self.cursor
    }

    /// The cursor, for the write helpers that advance it as they emit.
    pub fn cursor_mut(&mut self) -> &mut *mut ::core::ffi::c_char {
        &mut self.cursor
    }

    /// Put the cursor at `at`, which must be inside the window.
    pub fn set_cursor(&mut self, at: *mut ::core::ffi::c_char) {
        debug_assert!(self.start.addr() <= at.addr() && at.addr() <= self.end.addr());
        self.cursor = at;
    }

    /// How many bytes have been packed into the window.
    pub fn used(&self) -> usize {
        self.cursor.addr() - self.start.addr()
    }

    /// How much room is left before the window has to be flushed.
    pub fn remaining(&self) -> usize {
        self.end.addr() - self.cursor.addr()
    }

    /// The whole window, packed or not.
    pub fn capacity(&self) -> usize {
        self.end.addr() - self.start.addr()
    }
}

pub const LUA_NOREF: c_int = -2;

/// The most bytes one msgpack item's tag and inline payload can take.
pub const MPACK_ITEM_SIZE: c_int = 9;

/// Copies an encoded item through the caller's write cursor.
///
/// The cursor must have [`MPACK_ITEM_SIZE`] bytes of room, which is what
/// [`mpack_check_buffer`] leaves behind.
fn emit(cursor: &mut *mut c_char, bytes: &[u8]) {
    unsafe { cursor.copy_from_nonoverlapping(bytes.as_ptr().cast::<c_char>(), bytes.len()) };
    *cursor = unsafe { cursor.add(bytes.len()) };
}

/// Hands the buffer back to its owner to make room, then resumes at whatever
/// window the owner set up.
fn flush(packer: &mut PackerBuffer) {
    let hook = packer.packer_flush.expect("packer has no flush hook");
    unsafe { hook(packer) };
}

/// The low 16 bits of `value`, most significant byte first.
pub fn mpack_be16(cursor: &mut *mut c_char, value: uint32_t) {
    emit(cursor, &value.to_be_bytes()[2..]);
}

/// All 32 bits of `value`, most significant byte first.
pub fn mpack_be32(cursor: &mut *mut c_char, value: uint32_t) {
    emit(cursor, &value.to_be_bytes());
}

pub fn mpack_uint(cursor: &mut *mut c_char, value: uint32_t) {
    emit(cursor, format::uint(value).bytes());
}

pub fn mpack_uint64(cursor: &mut *mut c_char, value: uint64_t) {
    emit(cursor, format::uint64(value).bytes());
}

pub fn mpack_integer(cursor: &mut *mut c_char, value: Integer) {
    emit(cursor, format::integer(value).bytes());
}

pub fn mpack_float8(cursor: &mut *mut c_char, value: c_double) {
    emit(cursor, format::float8(value).bytes());
}

pub fn mpack_bool(cursor: &mut *mut c_char, value: bool) {
    emit(cursor, format::boolean(value).bytes());
}

pub fn mpack_nil(cursor: &mut *mut c_char) {
    emit(cursor, &[format::NIL]);
}

pub fn mpack_array(cursor: &mut *mut c_char, len: uint32_t) {
    emit(cursor, format::array_header(len).bytes());
}

pub fn mpack_map(cursor: &mut *mut c_char, len: uint32_t) {
    emit(cursor, format::map_header(len).bytes());
}

/// Writes a 16-bit array header whose length is filled in later, and returns
/// where to fill it in.
///
/// The UI event stream is built by appending to an array whose size is not
/// known until it is flushed, so the header goes out with a placeholder and is
/// overwritten in place with [`mpack_be16`].
pub fn mpack_array_dyn16(cursor: &mut *mut c_char) -> *mut c_char {
    emit(cursor, &[format::ARRAY16]);
    let pos = *cursor;
    // A recognisable placeholder rather than zero: a header left unpatched
    // shows up as a decoder error rather than as a silently empty array.
    mpack_be16(cursor, 0xffef);
    pos
}

/// Writes a string that is known to fit a fixstr header.
///
/// Every caller passes a UI event or method name, all of which are well under
/// the 31-byte limit, so the width choice the general [`mpack_str`] makes is
/// an assertion here instead. Taking the bytes as a slice keeps this a safe
/// `fn`; the callers hold their names as `(pointer, length)` inside an unsafe
/// body, where making the slice costs nothing.
pub fn mpack_str_small(cursor: &mut *mut c_char, str: &[u8]) {
    emit(cursor, format::fixstr_header(str.len()).bytes());
    emit(cursor, str);
}

/// How much room is left before the buffer has to be flushed.
pub fn mpack_remaining(packer: &PackerBuffer) -> size_t {
    packer.remaining()
}

/// Makes room for two more items, flushing if the buffer is nearly full.
///
/// Two rather than one because a dict entry writes its key and its value
/// between checks.
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

/// # Safety
/// `str` must describe `str.size` readable bytes at `str.data`.
pub unsafe fn mpack_str(str: String_0, packer: &mut PackerBuffer) {
    let header = format::str_header(str.len()).expect("string too long for msgpack");
    emit(packer.cursor_mut(), header.bytes());
    // SAFETY: the caller's bytes.
    unsafe { mpack_raw(str.data(), str.len(), packer) };
}

/// # Safety
/// `str` must describe `str.size` readable bytes at `str.data`.
pub unsafe fn mpack_bin(str: String_0, packer: &mut PackerBuffer) {
    let header = format::bin_header(str.len()).expect("blob too long for msgpack");
    emit(packer.cursor_mut(), header.bytes());
    // SAFETY: the caller's bytes.
    unsafe { mpack_raw(str.data(), str.len(), packer) };
}

/// Copies `len` opaque bytes, flushing as often as it takes.
///
/// Leaves the buffer ready for another item, so callers that follow a payload
/// with more structure do not need their own check.
///
/// # Safety
/// `data` must point at `len` readable bytes.
pub unsafe fn mpack_raw(data: *const c_char, len: size_t, packer: &mut PackerBuffer) {
    let mut pos: size_t = 0;
    while pos < len {
        let to_copy = (len - pos).min(mpack_remaining(packer));
        // SAFETY: `to_copy` is bounded by both what the caller still owes and
        // what the buffer window holds.
        unsafe {
            packer
                .cursor()
                .copy_from_nonoverlapping(data.add(pos), to_copy)
        };
        packer.set_cursor(unsafe { packer.cursor().add(to_copy) });
        pos += to_copy;
        if pos < len {
            flush(packer);
        }
    }
    mpack_check_buffer(packer);
}

/// An extension object: the header, the type byte, then `len` raw bytes.
///
/// # Safety
/// `buf` must point at `len` readable bytes.
pub unsafe fn mpack_ext(
    buf: *mut c_char,
    len: size_t,
    ext_type: int8_t,
    packer: &mut PackerBuffer,
) {
    let header = format::ext_header(len, ext_type).expect("extension too long for msgpack");
    emit(packer.cursor_mut(), header.bytes());
    // SAFETY: the caller's bytes.
    unsafe { mpack_raw(buf, len, packer) };
}

/// A buffer, window or tabpage handle. `ext_type` is the msgpack extension
/// type the wire gives that kind of handle: the variant's distance from
/// [`Object::Buffer`], so the three are 0, 1 and 2.
pub fn mpack_handle(ext_type: int8_t, handle: Handle, packer: &mut PackerBuffer) {
    emit(
        packer.cursor_mut(),
        format::handle(ext_type, handle).bytes(),
    );
}

/// The three extension types, spelled once so the packer's arms and the
/// unpacker's agree.
pub(crate) const EXT_BUFFER: int8_t = 0;
pub(crate) const EXT_WINDOW: int8_t = 1;
pub(crate) const EXT_TABPAGE: int8_t = 2;

/// # Safety
/// `obj` must point at a live object; its contents are traversed.
pub unsafe fn mpack_object(obj: *mut Object, packer: &mut PackerBuffer) {
    // SAFETY: the caller's object.
    unsafe { mpack_object_inner(obj, core::ptr::null_mut(), 0, packer) };
}

/// Packs an array's elements without wrapping them in another object.
///
/// # Safety
/// `arr` must describe `arr.size` live objects at `arr.items`.
pub unsafe fn mpack_object_array(arr: Array, packer: &mut PackerBuffer) {
    mpack_array(packer.cursor_mut(), container_len(arr.size));
    if arr.size == 0 {
        return;
    }
    // The walk needs a container to come back to only when more than one
    // element is left after the first.
    let mut container = Object::Array(arr);
    let resume = if arr.size > 1 {
        &raw mut container
    } else {
        core::ptr::null_mut()
    };
    // SAFETY: the caller's elements, and a container that lives to the end of
    // the walk below.
    unsafe { mpack_object_inner(arr.items, resume, 1, packer) };
}

/// Walks `current` and everything below it, iteratively.
///
/// `container`/`container_idx` name where to resume once `current` is done;
/// deeper containers are remembered on a stack that stays off the heap for the
/// first two levels. A single-element array is entered without touching the
/// stack at all, which is what keeps deeply nested one-element arrays from
/// growing it.
///
/// # Safety
/// `current` and `container` must point at live objects.
pub unsafe fn mpack_object_inner(
    mut current: *mut Object,
    mut container: *mut Object,
    mut container_idx: size_t,
    packer: &mut PackerBuffer,
) {
    let mut stack: format::SmallStack<(*mut Object, size_t)> = format::SmallStack::default();
    'walk: loop {
        mpack_check_buffer(packer);
        // SAFETY: `current` points at a live object for the whole walk, and
        // nothing else holds a reference to it while this borrow lasts.
        let obj = unsafe { &mut *current };
        // Everything that is not a container writes itself here and moves on;
        // nil, and a luaref once it has been released, fall through to the
        // nil byte below.
        'packed: {
            match obj {
                Object::LuaRef(reference) => {
                    // A released luaref packs as nil, and the slot is
                    // overwritten so a second pass cannot free it twice.
                    // SAFETY: a registry index, not a pointer.
                    unsafe { api_free_luaref(*reference) };
                    *reference = LUA_NOREF as LuaRef;
                }
                Object::Nil => {}
                Object::Boolean(value) => {
                    mpack_bool(packer.cursor_mut(), *value);
                    break 'packed;
                }
                Object::Integer(value) => {
                    mpack_integer(packer.cursor_mut(), *value);
                    break 'packed;
                }
                Object::Float(value) => {
                    mpack_float8(packer.cursor_mut(), *value);
                    break 'packed;
                }
                Object::String(value) => {
                    // SAFETY: the string is live for the pack.
                    unsafe { mpack_str(*value, packer) };
                    break 'packed;
                }
                Object::Buffer(handle) => {
                    mpack_handle(EXT_BUFFER, crate::narrow::number_as_int(*handle), packer);
                    break 'packed;
                }
                Object::Window(handle) => {
                    mpack_handle(EXT_WINDOW, crate::narrow::number_as_int(*handle), packer);
                    break 'packed;
                }
                Object::Tabpage(handle) => {
                    mpack_handle(EXT_TABPAGE, crate::narrow::number_as_int(*handle), packer);
                    break 'packed;
                }
                Object::Array(array) => {
                    let (size, items) = (array.size, array.items);
                    mpack_array(packer.cursor_mut(), container_len(size));
                    if size == 0 {
                        break 'packed;
                    }
                    if size == 1 {
                        // A one-element array is entered without touching the
                        // stack.
                        current = items;
                        continue 'walk;
                    }
                    if !container.is_null() {
                        stack.push((container, container_idx));
                    }
                    container = current;
                    container_idx = 0;
                    break 'packed;
                }
                Object::Dict(dict) => {
                    let size = dict.size;
                    mpack_map(packer.cursor_mut(), container_len(size));
                    if size == 0 {
                        break 'packed;
                    }
                    if !container.is_null() {
                        stack.push((container, container_idx));
                    }
                    container = current;
                    container_idx = 0;
                    break 'packed;
                }
            }
            mpack_nil(packer.cursor_mut());
        }

        if container.is_null() {
            match stack.pop() {
                Some((resume, idx)) => {
                    container = resume;
                    container_idx = idx;
                }
                None => break,
            }
        }

        // SAFETY: `container` is a live array or dict, and `container_idx` is
        // below its size -- the two assignments below restore that.
        match unsafe { *container } {
            Object::Array(arr) => {
                current = unsafe { arr.items.add(container_idx) };
                container_idx += 1;
                if container_idx >= arr.size {
                    container = core::ptr::null_mut();
                }
            }
            Object::Dict(dict) => {
                let entry: *mut KeyValuePair = unsafe { dict.items.add(container_idx) };
                container_idx += 1;
                mpack_check_buffer(packer);
                unsafe { mpack_str((*entry).key, packer) };
                current = unsafe { &raw mut (*entry).value };
                if container_idx >= dict.size {
                    container = core::ptr::null_mut();
                }
            }
            _ => unreachable!("only an array or a dict is left to resume"),
        }
    }
}

/// A buffer that grows on the heap instead of flushing anywhere, for callers
/// that want the packed bytes rather than a stream.
pub fn packer_string_buffer() -> PackerBuffer {
    const INITIAL_SIZE: size_t = 64;
    let alloc = unsafe { xmalloc(INITIAL_SIZE) }.cast::<c_char>();
    // SAFETY: `xmalloc` never returns null and hands back `INITIAL_SIZE`
    // bytes nothing else names; `flush_string_buffer` owns them from here
    // and `packer_take_string` gives them away.
    unsafe { PackerBuffer::new(alloc, INITIAL_SIZE, Some(flush_string_buffer)) }
}

/// # Safety
/// `buffer` points at a live buffer opened by [`packer_string_buffer`], whose
/// allocation this reallocates.
unsafe fn flush_string_buffer(buffer: *mut PackerBuffer) {
    // SAFETY: the caller's buffer, and an allocation only this hook resizes.
    let buffer = unsafe { &mut *buffer };
    let new_capacity = 2 * buffer.capacity();
    let grown = unsafe { xrealloc(buffer.start().cast::<c_void>(), new_capacity) }.cast::<c_char>();
    // SAFETY: `xrealloc` moved the packed bytes into a block of
    // `new_capacity`, which the same hook goes on owning.
    unsafe { buffer.moved_to(grown, new_capacity) };
}

/// Takes ownership of everything written to a [`packer_string_buffer`].
pub fn packer_take_string(buffer: &PackerBuffer) -> String_0 {
    String_0::from_raw_parts(buffer.start(), buffer.used())
}
