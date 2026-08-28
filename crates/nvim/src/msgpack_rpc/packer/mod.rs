#![deny(unsafe_op_in_unsafe_fn)]
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
    Array, Dict, Integer, KeyValuePair, LuaRef, Object, ObjectType, PackerBuffer, String_0,
    handle_T, int8_t, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeFloat, kObjectTypeInteger, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString,
    kObjectTypeTabpage, kObjectTypeWindow, packer_buffer_t, size_t, uint32_t, uint64_t,
};

pub mod format;

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
    packer.endptr.addr() - packer.ptr.addr()
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
    emit(&mut packer.ptr, header.bytes());
    // SAFETY: the caller's bytes.
    unsafe { mpack_raw(str.data(), str.len(), packer) };
}

/// # Safety
/// `str` must describe `str.size` readable bytes at `str.data`.
pub unsafe fn mpack_bin(str: String_0, packer: &mut PackerBuffer) {
    let header = format::bin_header(str.len()).expect("blob too long for msgpack");
    emit(&mut packer.ptr, header.bytes());
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
        unsafe { packer.ptr.copy_from_nonoverlapping(data.add(pos), to_copy) };
        packer.ptr = unsafe { packer.ptr.add(to_copy) };
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
    emit(&mut packer.ptr, header.bytes());
    // SAFETY: the caller's bytes.
    unsafe { mpack_raw(buf, len, packer) };
}

/// A buffer, window or tabpage handle. The extension type is the object type's
/// distance from `kObjectTypeBuffer`, so the three are 0, 1 and 2.
pub fn mpack_handle(type_0: ObjectType, handle: handle_T, packer: &mut PackerBuffer) {
    let ext_type = type_0.wrapping_sub(kObjectTypeBuffer).to_le_bytes()[0].cast_signed();
    emit(&mut packer.ptr, format::handle(ext_type, handle).bytes());
}

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
    mpack_array(&mut packer.ptr, container_len(arr.size));
    if arr.size == 0 {
        return;
    }
    // The walk needs a container to come back to only when more than one
    // element is left after the first.
    let mut container = Object {
        type_0: kObjectTypeArray,
        data: crate::types::object_data { array: arr },
    };
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
        // SAFETY: `current` points at a live object for the whole walk; every
        // union read below is guarded by the `type_0` it belongs to.
        let type_0 = unsafe { (*current).type_0 };
        // Everything that is not a container writes itself here and moves on;
        // nil, and a luaref once it has been released, fall through to the
        // nil byte below.
        'packed: {
            match type_0 {
                kObjectTypeLuaRef => {
                    // SAFETY: as above. A released luaref packs as nil, and
                    // the slot is overwritten so a second pass cannot free it
                    // twice.
                    unsafe { api_free_luaref((*current).data.luaref) };
                    unsafe { (*current).data.luaref = LUA_NOREF as LuaRef };
                }
                kObjectTypeNil => {}
                kObjectTypeBoolean => {
                    // SAFETY: as above.
                    mpack_bool(&mut packer.ptr, unsafe { (*current).data.boolean });
                    break 'packed;
                }
                kObjectTypeInteger => {
                    // SAFETY: as above.
                    mpack_integer(&mut packer.ptr, unsafe { (*current).data.integer });
                    break 'packed;
                }
                kObjectTypeFloat => {
                    // SAFETY: as above.
                    mpack_float8(&mut packer.ptr, unsafe { (*current).data.floating });
                    break 'packed;
                }
                kObjectTypeString => {
                    // SAFETY: as above, and the string is live for the pack.
                    unsafe { mpack_str((*current).data.string, packer) };
                    break 'packed;
                }
                kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage => {
                    // SAFETY: as above; a handle is carried in the integer
                    // arm of the union.
                    let handle = crate::narrow::number_as_int(unsafe { (*current).data.integer });
                    mpack_handle(type_0, handle, packer);
                    break 'packed;
                }
                kObjectTypeArray | kObjectTypeDict => {
                    let is_array = type_0 == kObjectTypeArray;
                    // SAFETY: as above.
                    let size = if is_array {
                        unsafe { (*current).data.array.size }
                    } else {
                        unsafe { (*current).data.dict.size }
                    };
                    if is_array {
                        mpack_array(&mut packer.ptr, container_len(size));
                    } else {
                        mpack_map(&mut packer.ptr, container_len(size));
                    }
                    if size == 0 {
                        break 'packed;
                    }
                    if is_array && size == 1 {
                        // SAFETY: as above; a one-element array is entered
                        // without touching the stack.
                        current = unsafe { (*current).data.array.items };
                        continue 'walk;
                    }
                    if !container.is_null() {
                        stack.push((container, container_idx));
                    }
                    container = current;
                    container_idx = 0;
                    break 'packed;
                }
                _ => break 'packed,
            }
            mpack_nil(&mut packer.ptr);
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
        // below its size — the two assignments below restore that.
        if unsafe { (*container).type_0 } == kObjectTypeArray {
            let arr: Array = unsafe { (*container).data.array };
            current = unsafe { arr.items.add(container_idx) };
            container_idx += 1;
            if container_idx >= arr.size {
                container = core::ptr::null_mut();
            }
        } else {
            let dict: Dict = unsafe { (*container).data.dict };
            let entry: *mut KeyValuePair = unsafe { dict.items.add(container_idx) };
            container_idx += 1;
            mpack_check_buffer(packer);
            unsafe { mpack_str((*entry).key, packer) };
            current = unsafe { &raw mut (*entry).value };
            if container_idx >= dict.size {
                container = core::ptr::null_mut();
            }
        }
    }
}

/// A buffer that grows on the heap instead of flushing anywhere, for callers
/// that want the packed bytes rather than a stream.
pub fn packer_string_buffer() -> PackerBuffer {
    const INITIAL_SIZE: size_t = 64;
    let alloc = unsafe { xmalloc(INITIAL_SIZE) }.cast::<c_char>();
    packer_buffer_t {
        startptr: alloc,
        ptr: alloc,
        endptr: alloc.wrapping_add(INITIAL_SIZE),
        anydata: core::ptr::null_mut(),
        anyint: 0,
        packer_flush: Some(flush_string_buffer),
    }
}

/// # Safety
/// `buffer` points at a live buffer opened by [`packer_string_buffer`], whose
/// allocation this reallocates.
unsafe fn flush_string_buffer(buffer: *mut PackerBuffer) {
    // SAFETY: the caller's buffer, and an allocation only this hook resizes.
    let buffer = unsafe { &mut *buffer };
    let capacity = buffer.endptr.addr() - buffer.startptr.addr();
    let len = buffer.ptr.addr() - buffer.startptr.addr();
    let new_capacity = 2 * capacity;
    buffer.startptr =
        unsafe { xrealloc(buffer.startptr.cast::<c_void>(), new_capacity) }.cast::<c_char>();
    buffer.ptr = unsafe { buffer.startptr.add(len) };
    buffer.endptr = unsafe { buffer.startptr.add(new_capacity) };
}

/// Takes ownership of everything written to a [`packer_string_buffer`].
pub fn packer_take_string(buffer: &PackerBuffer) -> String_0 {
    String_0::from_raw_parts(buffer.startptr, buffer.ptr.addr() - buffer.startptr.addr())
}
