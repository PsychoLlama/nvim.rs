//! Decoding at the RPC boundary, where the input is whatever a client sent.
//!
//! `test/unit/msgpack_spec.lua` covers two regressions in the streaming
//! `grid_line` decoder. These cover `unpack`, which every `nvim_*` call that
//! takes msgpack goes through, with an emphasis on inputs that are not valid
//! msgpack at all.

use std::ffi::{CStr, c_char, c_void};

use c2rust_neovim::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use c2rust_neovim::src::nvim::msgpack_rpc::unpacker::{
    Arena, Error, Object, kErrorTypeNone, unpack,
};
use c2rust_neovim::src::nvim::types::{
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, kObjectTypeTabpage, kObjectTypeWindow,
};

/// What one decode produced: the object plus whatever error was reported.
struct Decoded {
    object: Object,
    arena: Arena,
    error: Error,
}

impl Decoded {
    fn message(&self) -> Option<&str> {
        if self.error.type_0 == kErrorTypeNone {
            return None;
        }
        Some(unsafe { CStr::from_ptr(self.error.msg) }.to_str().unwrap())
    }
}

impl Drop for Decoded {
    fn drop(&mut self) {
        unsafe {
            arena_mem_free(arena_finish(&raw mut self.arena));
            if !self.error.msg.is_null() {
                c2rust_neovim::src::nvim::memory::xfree(self.error.msg.cast::<c_void>());
            }
        }
    }
}

fn decode(bytes: &[u8]) -> Decoded {
    let mut arena: Arena = ARENA_EMPTY;
    let mut error: Error = Error {
        type_0: kErrorTypeNone,
        msg: std::ptr::null_mut(),
    };
    let object = unsafe {
        unpack(
            bytes.as_ptr().cast::<c_char>(),
            bytes.len(),
            &raw mut arena,
            &raw mut error,
        )
    };
    Decoded {
        object,
        arena,
        error,
    }
}

fn text(object: &Object) -> Vec<u8> {
    assert_eq!(object.type_0, kObjectTypeString);
    let string = unsafe { object.data.string };
    unsafe { std::slice::from_raw_parts(string.data.cast::<u8>(), string.size) }.to_vec()
}

#[test]
fn decodes_scalars() {
    let nil = decode(&[0xc0]);
    assert_eq!(nil.object.type_0, kObjectTypeNil);
    assert_eq!(nil.message(), None);

    assert_eq!(decode(&[0xc3]).object.type_0, kObjectTypeBoolean);
    assert!(unsafe { decode(&[0xc3]).object.data.boolean });
    assert!(!unsafe { decode(&[0xc2]).object.data.boolean });

    assert_eq!(unsafe { decode(&[0x07]).object.data.integer }, 7);
    assert_eq!(unsafe { decode(&[0xff]).object.data.integer }, -1);
    assert_eq!(decode(&[0x07]).object.type_0, kObjectTypeInteger);

    let float = decode(&[0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(float.object.type_0, kObjectTypeFloat);
    assert_eq!(unsafe { float.object.data.floating }, 1.0);
}

#[test]
fn decodes_strings_and_containers() {
    let string = decode(&[0xa2, b'h', b'i']);
    assert_eq!(text(&string.object), b"hi");

    let array = decode(&[0x92, 0x01, 0x02]);
    assert_eq!(array.object.type_0, kObjectTypeArray);
    let items = unsafe { array.object.data.array };
    assert_eq!(items.size, 2);
    assert_eq!(unsafe { (*items.items.add(1)).data.integer }, 2);

    let dict = decode(&[0x81, 0xa1, b'a', 0x2a]);
    assert_eq!(dict.object.type_0, kObjectTypeDict);
    let entries = unsafe { dict.object.data.dict };
    assert_eq!(entries.size, 1);
    assert_eq!(unsafe { (*entries.items).key.size }, 1);
    assert_eq!(unsafe { (*entries.items).value.data.integer }, 42);
}

/// The API hands out NUL-terminated strings even though it carries the length
/// beside them, so the decoder allocates one byte more than it needs.
#[test]
fn decoded_strings_are_nul_terminated() {
    let string = decode(&[0xa3, b'a', b'b', b'c']);
    let raw = unsafe { string.object.data.string };
    assert_eq!(raw.size, 3);
    assert_eq!(unsafe { *raw.data.add(3) }, 0);
}

/// Handles arrive as extension objects whose type byte is the handle kind's
/// distance from `kObjectTypeBuffer`.
#[test]
fn decodes_handles() {
    let buffer = decode(&[0xd4, 0, 3]);
    assert_eq!(buffer.object.type_0, kObjectTypeBuffer);
    assert_eq!(unsafe { buffer.object.data.integer }, 3);

    let window = decode(&[0xc7, 3, 1, 0xcd, 0x03, 0xe8]);
    assert_eq!(window.object.type_0, kObjectTypeWindow);
    assert_eq!(unsafe { window.object.data.integer }, 1000);

    let tabpage = decode(&[0xd4, 2, 1]);
    assert_eq!(tabpage.object.type_0, kObjectTypeTabpage);
}

/// An extension the API does not define decodes as nil rather than failing
/// the whole message — as does one whose payload is not a plain integer.
#[test]
fn unknown_extensions_decode_as_nil() {
    let unknown_type = decode(&[0xd4, 9, 1]);
    assert_eq!(unknown_type.object.type_0, kObjectTypeNil);
    assert_eq!(unknown_type.message(), None);

    let negative_type = decode(&[0xd4, 0xff, 1]);
    assert_eq!(negative_type.object.type_0, kObjectTypeNil);

    // fixext1 carrying a nil, not an integer.
    let wrong_payload = decode(&[0xd4, 0, 0xc0]);
    assert_eq!(wrong_payload.object.type_0, kObjectTypeNil);

    // ext8 with a ten-byte payload: one more than the accumulator holds.
    let mut too_long = vec![0xc7, 10, 0];
    too_long.extend([0xcf, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(decode(&too_long).object.type_0, kObjectTypeNil);
}

#[test]
#[cfg_attr(miri, ignore = "api_set_error formats the message through vsnprintf")]
fn rejects_truncated_input() {
    assert_eq!(decode(&[]).message(), Some("incomplete msgpack string"));
    // An array header promising two elements, with one.
    assert_eq!(
        decode(&[0x92, 0x01]).message(),
        Some("incomplete msgpack string")
    );
    // A five-byte string header with three bytes behind it.
    assert_eq!(
        decode(&[0xa5, b'a', b'b', b'c']).message(),
        Some("incomplete msgpack string")
    );
}

#[test]
#[cfg_attr(miri, ignore = "api_set_error formats the message through vsnprintf")]
fn rejects_bytes_that_are_not_msgpack() {
    // 0xc1 is the one byte msgpack leaves undefined.
    assert_eq!(decode(&[0xc1]).message(), Some("invalid msgpack string"));
}

#[test]
#[cfg_attr(miri, ignore = "api_set_error formats the message through vsnprintf")]
fn rejects_trailing_data() {
    assert_eq!(
        decode(&[0xc0, 0xc0]).message(),
        Some("trailing data in msgpack string")
    );
}

/// The parse tree is a fixed-depth array, so nesting past it is refused
/// rather than recursed into.
#[test]
#[cfg_attr(miri, ignore = "api_set_error formats the message through vsnprintf")]
fn rejects_objects_that_nest_too_deep() {
    let shallow = vec![0x91_u8; 16];
    let mut ok = shallow.clone();
    ok.push(0xc0);
    assert_eq!(decode(&ok).message(), None);

    let mut deep = vec![0x91_u8; 4096];
    deep.push(0xc0);
    assert_eq!(
        decode(&deep).message(),
        Some("object was too deep to unpack")
    );
}

/// A container header may claim any length that fits in 32 bits. The decoder
/// allocates for it up front, so an unbacked claim has to fail on the missing
/// elements rather than on the allocation.
#[test]
#[cfg_attr(miri, ignore = "api_set_error formats the message through vsnprintf")]
fn rejects_a_container_longer_than_its_contents() {
    // array32 claiming 0x10000 elements, with none.
    assert_eq!(
        decode(&[0xdd, 0x00, 0x01, 0x00, 0x00]).message(),
        Some("incomplete msgpack string")
    );
    // map16 claiming 4096 pairs, with one.
    assert_eq!(
        decode(&[0xde, 0x10, 0x00, 0xa1, b'a', 0xc0]).message(),
        Some("incomplete msgpack string")
    );
}
