//! Decoding at the RPC boundary, where the input is whatever a client sent.
//!
//! `test/unit/msgpack_spec.lua` covers two regressions in the streaming
//! `grid_line` decoder. These cover `unpack`, which every `nvim_*` call that
//! takes msgpack goes through, with an emphasis on inputs that are not valid
//! msgpack at all.

use std::ffi::c_char;

use neovim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};
use neovim::msgpack_rpc::unpacker::unpack;
use neovim::types::{Arena, Error, Object};

/// What one decode produced: the object plus whatever error was reported.
///
/// The error owns its message and releases it when the `Decoded` drops, so
/// the only thing this has to free by hand is the arena.
struct Decoded {
    object: Object,
    arena: Arena,
    error: Error,
}

impl Decoded {
    /// The reported message, or `None` for a decode that reported nothing.
    ///
    /// `Error`'s `Display` -- what this crate reaches it through from
    /// outside -- is the message and nothing else, so an unset error and a
    /// set one are told apart by whether there is any text. Every message
    /// the decoder produces has some.
    fn message(&self) -> Option<String> {
        let text = self.error.to_string();
        (!text.is_empty()).then_some(text)
    }
}

impl Drop for Decoded {
    fn drop(&mut self) {
        unsafe {
            arena_mem_free(arena_finish(&raw mut self.arena));
        }
    }
}

fn decode(bytes: &[u8]) -> Decoded {
    let mut arena: Arena = ARENA_EMPTY;
    let mut error = Error::default();
    let object = unsafe {
        unpack(
            bytes.as_ptr().cast::<c_char>(),
            bytes.len(),
            &raw mut arena,
            &mut error,
        )
    };
    Decoded {
        object,
        arena,
        error,
    }
}

fn text(object: &Object) -> Vec<u8> {
    let string = object.as_string().expect("decoded a string");
    unsafe { string.as_bytes() }.to_vec()
}

#[test]
fn decodes_scalars() {
    let nil = decode(&[0xc0]);
    assert!(nil.object.is_nil());
    assert_eq!(nil.message().as_deref(), None);

    assert_eq!(decode(&[0xc3]).object.as_boolean(), Some(true));
    assert_eq!(decode(&[0xc2]).object.as_boolean(), Some(false));

    assert_eq!(decode(&[0x07]).object.as_integer(), Some(7));
    assert_eq!(decode(&[0xff]).object.as_integer(), Some(-1));

    let float = decode(&[0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(float.object.as_float(), Some(1.0));
}

#[test]
fn decodes_strings_and_containers() {
    let string = decode(&[0xa2, b'h', b'i']);
    assert_eq!(text(&string.object), b"hi");

    let array = decode(&[0x92, 0x01, 0x02]);
    let items = array.object.as_array().expect("decoded an array");
    assert_eq!(items.size, 2);
    assert_eq!(unsafe { *items.items.add(1) }.as_integer(), Some(2));

    let dict = decode(&[0x81, 0xa1, b'a', 0x2a]);
    let entries = dict.object.as_dict().expect("decoded a dict");
    assert_eq!(entries.size, 1);
    assert_eq!(unsafe { (*entries.items).key.len() }, 1);
    assert_eq!(unsafe { (*entries.items).value }.as_integer(), Some(42));
}

/// The API hands out NUL-terminated strings even though it carries the length
/// beside them, so the decoder allocates one byte more than it needs.
#[test]
fn decoded_strings_are_nul_terminated() {
    let string = decode(&[0xa3, b'a', b'b', b'c']);
    let raw = string.object.as_string().expect("decoded a string");
    assert_eq!(raw.len(), 3);
    assert_eq!(unsafe { *raw.data().add(3) }, 0);
}

/// Handles arrive as extension objects whose type byte is the handle kind's
/// distance from `kObjectTypeBuffer`.
#[test]
fn decodes_handles() {
    assert!(matches!(decode(&[0xd4, 0, 3]).object, Object::Buffer(3)));
    let window = decode(&[0xc7, 3, 1, 0xcd, 0x03, 0xe8]);
    assert!(matches!(window.object, Object::Window(1000)));
    assert!(matches!(decode(&[0xd4, 2, 1]).object, Object::Tabpage(1)));
}

/// An extension the API does not define decodes as nil rather than failing
/// the whole message — as does one whose payload is not a plain integer.
#[test]
fn unknown_extensions_decode_as_nil() {
    let unknown_type = decode(&[0xd4, 9, 1]);
    assert!(unknown_type.object.is_nil());
    assert_eq!(unknown_type.message().as_deref(), None);

    assert!(decode(&[0xd4, 0xff, 1]).object.is_nil());

    // fixext1 carrying a nil, not an integer.
    assert!(decode(&[0xd4, 0, 0xc0]).object.is_nil());

    // ext8 with a ten-byte payload: one more than the accumulator holds.
    let mut too_long = vec![0xc7, 10, 0];
    too_long.extend([0xcf, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert!(decode(&too_long).object.is_nil());
}

#[test]
fn rejects_truncated_input() {
    assert_eq!(
        decode(&[]).message().as_deref(),
        Some("incomplete msgpack string")
    );
    // An array header promising two elements, with one.
    assert_eq!(
        decode(&[0x92, 0x01]).message().as_deref(),
        Some("incomplete msgpack string")
    );
    // A five-byte string header with three bytes behind it.
    assert_eq!(
        decode(&[0xa5, b'a', b'b', b'c']).message().as_deref(),
        Some("incomplete msgpack string")
    );
}

#[test]
fn rejects_bytes_that_are_not_msgpack() {
    // 0xc1 is the one byte msgpack leaves undefined.
    assert_eq!(
        decode(&[0xc1]).message().as_deref(),
        Some("invalid msgpack string")
    );
}

#[test]
fn rejects_trailing_data() {
    assert_eq!(
        decode(&[0xc0, 0xc0]).message().as_deref(),
        Some("trailing data in msgpack string")
    );
}

/// The parse tree is a fixed-depth array, so nesting past it is refused
/// rather than recursed into.
#[test]
fn rejects_objects_that_nest_too_deep() {
    let shallow = vec![0x91_u8; 16];
    let mut ok = shallow.clone();
    ok.push(0xc0);
    assert_eq!(decode(&ok).message().as_deref(), None);

    let mut deep = vec![0x91_u8; 4096];
    deep.push(0xc0);
    assert_eq!(
        decode(&deep).message().as_deref(),
        Some("object was too deep to unpack")
    );
}

/// A container header may claim any length that fits in 32 bits. The decoder
/// allocates for it up front, so an unbacked claim has to fail on the missing
/// elements rather than on the allocation.
#[test]
fn rejects_a_container_longer_than_its_contents() {
    // array32 claiming 0x10000 elements, with none.
    assert_eq!(
        decode(&[0xdd, 0x00, 0x01, 0x00, 0x00]).message().as_deref(),
        Some("incomplete msgpack string")
    );
    // map16 claiming 4096 pairs, with one.
    assert_eq!(
        decode(&[0xde, 0x10, 0x00, 0xa1, b'a', 0xc0])
            .message()
            .as_deref(),
        Some("incomplete msgpack string")
    );
}
