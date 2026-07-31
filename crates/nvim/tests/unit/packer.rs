//! The msgpack encodings the API packer emits, pinned to exact bytes.
//!
//! `test/unit` has no packer spec — the wire format is only covered
//! indirectly, by the RPC functional tests. These fill that in: the pure
//! encoders below are checked against the msgpack specification's examples
//! plus the boundaries where this implementation deliberately differs from
//! it, and the object walker is checked end to end through a string buffer.

use std::ffi::c_char;

use c2rust_neovim::src::nvim::memory::xfree;
use c2rust_neovim::src::nvim::msgpack_rpc::packer::format;
use c2rust_neovim::src::nvim::msgpack_rpc::packer::{
    Array, Dict, KeyValuePair, Object, PackerBuffer, String_0, mpack_object, packer_string_buffer,
    packer_take_string,
};
use c2rust_neovim::src::nvim::types::{
    kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeFloat,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, kObjectTypeWindow, object_data,
};

#[test]
fn booleans_are_one_byte() {
    assert_eq!(format::boolean(false).bytes(), &[0xc2]);
    assert_eq!(format::boolean(true).bytes(), &[0xc3]);
}

#[test]
fn unsigned_widths() {
    assert_eq!(format::uint(0).bytes(), &[0x00]);
    assert_eq!(format::uint(0x7f).bytes(), &[0x7f]);
    assert_eq!(format::uint(0x80).bytes(), &[0xcc, 0x80]);
    assert_eq!(format::uint(0xff).bytes(), &[0xcc, 0xff]);
    assert_eq!(format::uint(0x100).bytes(), &[0xcd, 0x01, 0x00]);
    assert_eq!(format::uint(0xffff).bytes(), &[0xcd, 0xff, 0xff]);
    assert_eq!(
        format::uint(0x1_0000).bytes(),
        &[0xce, 0x00, 0x01, 0x00, 0x00]
    );
    assert_eq!(
        format::uint(u32::MAX).bytes(),
        &[0xce, 0xff, 0xff, 0xff, 0xff]
    );
}

/// The 64-bit form takes over at 0x0fff_ffff, four hex digits early, so
/// values that would fit a uint32 are written eight bytes wide. Preserved
/// deliberately: it is what every existing client has been decoding.
#[test]
fn unsigned_64_cuts_over_early() {
    assert_eq!(
        format::uint64(0xfff_ffff).bytes(),
        format::uint(0xfff_ffff).bytes()
    );
    assert_eq!(
        format::uint64(0x1000_0000).bytes(),
        &[0xcf, 0, 0, 0, 0, 0x10, 0, 0, 0]
    );
    assert_eq!(
        format::uint64(u64::MAX).bytes(),
        &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
    );
}

#[test]
fn signed_widths() {
    // Non-negative values go through the unsigned encoder.
    assert_eq!(format::integer(0).bytes(), &[0x00]);
    assert_eq!(format::integer(127).bytes(), &[0x7f]);

    assert_eq!(format::integer(-1).bytes(), &[0xff]);
    assert_eq!(format::integer(-0x20).bytes(), &[0xe0]);
    assert_eq!(format::integer(-0x21).bytes(), &[0xd0, 0xdf]);
    assert_eq!(format::integer(-0x80).bytes(), &[0xd0, 0x80]);
    assert_eq!(format::integer(-0x81).bytes(), &[0xd1, 0xff, 0x7f]);
    assert_eq!(format::integer(-0x8000).bytes(), &[0xd1, 0x80, 0x00]);
    assert_eq!(
        format::integer(-0x8001).bytes(),
        &[0xd2, 0xff, 0xff, 0x7f, 0xff]
    );
    assert_eq!(
        format::integer(-0x8000_0000).bytes(),
        &[0xd2, 0x80, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        format::integer(-0x8000_0001).bytes(),
        &[0xd3, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff, 0xff, 0xff]
    );
    assert_eq!(
        format::integer(i64::MIN).bytes(),
        &[0xd3, 0x80, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn floats_are_always_double_precision() {
    assert_eq!(
        format::float8(1.0).bytes(),
        &[0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(format::float8(0.0).bytes(), &[0xcb, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        format::float8(-0.0).bytes(),
        &[0xcb, 0x80, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        format::float8(f64::INFINITY).bytes(),
        &[0xcb, 0x7f, 0xf0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn container_headers() {
    assert_eq!(format::array_header(0).bytes(), &[0x90]);
    assert_eq!(format::array_header(15).bytes(), &[0x9f]);
    assert_eq!(format::array_header(16).bytes(), &[0xdc, 0x00, 0x10]);
    assert_eq!(format::array_header(0xffff).bytes(), &[0xdc, 0xff, 0xff]);
    assert_eq!(
        format::array_header(0x1_0000).bytes(),
        &[0xdd, 0x00, 0x01, 0x00, 0x00]
    );

    assert_eq!(format::map_header(0).bytes(), &[0x80]);
    assert_eq!(format::map_header(15).bytes(), &[0x8f]);
    assert_eq!(format::map_header(16).bytes(), &[0xde, 0x00, 0x10]);
    assert_eq!(
        format::map_header(0x1_0000).bytes(),
        &[0xdf, 0x00, 0x01, 0x00, 0x00]
    );
}

/// Each width is chosen with a strict `<`, so the length that exactly fills
/// one width steps up to the next. 255 bytes could be a str8 and 65535 a
/// str16; upstream spends the wider header on both.
#[test]
fn string_headers_step_up_one_early() {
    assert_eq!(format::str_header(0).unwrap().bytes(), &[0xa0]);
    assert_eq!(format::str_header(31).unwrap().bytes(), &[0xbf]);
    assert_eq!(format::str_header(32).unwrap().bytes(), &[0xd9, 32]);
    assert_eq!(format::str_header(254).unwrap().bytes(), &[0xd9, 254]);
    assert_eq!(
        format::str_header(255).unwrap().bytes(),
        &[0xda, 0x00, 0xff]
    );
    assert_eq!(
        format::str_header(0xffff).unwrap().bytes(),
        &[0xdb, 0x00, 0x00, 0xff, 0xff]
    );

    assert_eq!(format::bin_header(0).unwrap().bytes(), &[0xc4, 0]);
    assert_eq!(format::bin_header(254).unwrap().bytes(), &[0xc4, 254]);
    assert_eq!(
        format::bin_header(255).unwrap().bytes(),
        &[0xc5, 0x00, 0xff]
    );
    assert_eq!(
        format::bin_header(0xffff).unwrap().bytes(),
        &[0xc6, 0x00, 0x00, 0xff, 0xff]
    );

    assert!(format::str_header(0xffff_ffff).is_none());
    assert!(format::bin_header(0xffff_ffff).is_none());
}

/// Only fixext1 and fixext2 are used; a four-byte payload takes ext8 rather
/// than msgpack's fixext4. The ext8 boundary is `<=` where the string headers
/// use `<`, so a 255-byte payload does fit ext8.
#[test]
fn extension_headers_skip_the_wider_fixed_forms() {
    assert_eq!(format::ext_header(1, 0).unwrap().bytes(), &[0xd4, 0]);
    assert_eq!(format::ext_header(2, 1).unwrap().bytes(), &[0xd5, 1]);
    assert_eq!(format::ext_header(4, 2).unwrap().bytes(), &[0xc7, 2]);
    assert_eq!(format::ext_header(16, -1).unwrap().bytes(), &[0xc7, 0xff]);
    assert_eq!(format::ext_header(255, 0).unwrap().bytes(), &[0xc7, 0]);
    assert_eq!(
        format::ext_header(256, 0).unwrap().bytes(),
        &[0xc8, 0x01, 0x00, 0]
    );
    assert!(format::ext_header(0xffff_ffff, 0).is_none());
}

/// Handles are extension objects whose type byte is the object type's
/// distance from `kObjectTypeBuffer`. Small ones are a fixext1 carrying the
/// raw byte; larger ones wrap the handle's own unsigned encoding, so they
/// spend an ext8 header even at two bytes.
#[test]
fn handles() {
    assert_eq!(format::handle(0, 1).bytes(), &[0xd4, 0, 1]);
    assert_eq!(format::handle(1, 0x7f).bytes(), &[0xd4, 1, 0x7f]);
    assert_eq!(format::handle(2, -1).bytes(), &[0xd4, 2, 0xff]);
    assert_eq!(format::handle(0, 0x80).bytes(), &[0xc7, 2, 0, 0xcc, 0x80]);
    assert_eq!(
        format::handle(0, 0x1_0000).bytes(),
        &[0xc7, 5, 0, 0xce, 0x00, 0x01, 0x00, 0x00]
    );
}

#[test]
#[should_panic(expected = "handles are allocated upward")]
fn a_large_negative_handle_is_rejected() {
    format::handle(0, -0x20);
}

/// Packs one object through a heap-backed buffer and returns the bytes.
fn pack(object: &mut Object) -> Vec<u8> {
    let mut buffer: PackerBuffer = packer_string_buffer();
    unsafe { mpack_object(object, &mut buffer) };
    let packed: String_0 = packer_take_string(&buffer);
    let bytes =
        unsafe { std::slice::from_raw_parts(packed.data.cast::<u8>(), packed.size) }.to_vec();
    unsafe { xfree(packed.data.cast()) };
    bytes
}

fn scalar(type_0: u32, data: object_data) -> Object {
    Object { type_0, data }
}

fn string(text: &mut [u8]) -> Object {
    scalar(
        kObjectTypeString,
        object_data {
            string: String_0 {
                data: text.as_mut_ptr().cast::<c_char>(),
                size: text.len(),
            },
        },
    )
}

fn array(items: &mut [Object]) -> Object {
    scalar(
        kObjectTypeArray,
        object_data {
            array: Array {
                size: items.len(),
                capacity: items.len(),
                items: items.as_mut_ptr(),
            },
        },
    )
}

fn dict(items: &mut [KeyValuePair]) -> Object {
    scalar(
        kObjectTypeDict,
        object_data {
            dict: Dict {
                size: items.len(),
                capacity: items.len(),
                items: items.as_mut_ptr(),
            },
        },
    )
}

#[test]
fn packs_scalars() {
    assert_eq!(
        pack(&mut scalar(kObjectTypeNil, object_data { integer: 0 })),
        [0xc0]
    );
    assert_eq!(
        pack(&mut scalar(
            kObjectTypeBoolean,
            object_data { boolean: true }
        )),
        [0xc3]
    );
    assert_eq!(
        pack(&mut scalar(kObjectTypeInteger, object_data { integer: -3 })),
        [0xfd]
    );
    assert_eq!(
        pack(&mut scalar(kObjectTypeFloat, object_data { floating: 1.0 })),
        [0xcb, 0x3f, 0xf0, 0, 0, 0, 0, 0, 0]
    );
    let mut hi = *b"hi";
    assert_eq!(pack(&mut string(&mut hi)), [0xa2, b'h', b'i']);
    assert_eq!(
        pack(&mut scalar(kObjectTypeBuffer, object_data { integer: 3 })),
        [0xd4, 0, 3]
    );
    assert_eq!(
        pack(&mut scalar(
            kObjectTypeWindow,
            object_data { integer: 1000 }
        )),
        [0xc7, 3, 1, 0xcd, 0x03, 0xe8]
    );
}

#[test]
fn packs_empty_containers() {
    assert_eq!(pack(&mut array(&mut [])), [0x90]);
    assert_eq!(pack(&mut dict(&mut [])), [0x80]);
}

/// The walker enters a one-element array without remembering anything, so
/// arbitrarily deep single-element nesting costs no stack at all.
#[test]
fn packs_deeply_nested_single_element_arrays() {
    const DEPTH: usize = 8;
    let mut levels: Vec<Vec<Object>> = (0..DEPTH)
        .map(|_| vec![scalar(kObjectTypeInteger, object_data { integer: 7 })])
        .collect();
    for level in 1..DEPTH {
        let (below, above) = levels.split_at_mut(level);
        above[0][0] = array(&mut below[level - 1][..]);
    }
    let mut root = array(&mut levels[DEPTH - 1][..]);

    let mut expected = vec![0x91_u8; DEPTH];
    expected.push(0x07);
    assert_eq!(pack(&mut root), expected);
}

#[test]
fn packs_a_nested_object() {
    // { "a": [1, 2], "b": { "c": true } }
    let mut inner_key = *b"c";
    let mut inner = [KeyValuePair {
        key: String_0 {
            data: inner_key.as_mut_ptr().cast::<c_char>(),
            size: 1,
        },
        value: scalar(kObjectTypeBoolean, object_data { boolean: true }),
    }];
    let mut list = [
        scalar(kObjectTypeInteger, object_data { integer: 1 }),
        scalar(kObjectTypeInteger, object_data { integer: 2 }),
    ];
    let mut a = *b"a";
    let mut b = *b"b";
    let mut entries = [
        KeyValuePair {
            key: String_0 {
                data: a.as_mut_ptr().cast::<c_char>(),
                size: 1,
            },
            value: array(&mut list),
        },
        KeyValuePair {
            key: String_0 {
                data: b.as_mut_ptr().cast::<c_char>(),
                size: 1,
            },
            value: dict(&mut inner),
        },
    ];
    assert_eq!(
        pack(&mut dict(&mut entries)),
        [
            0x82, // map of 2
            0xa1, b'a', 0x92, 0x01, 0x02, // "a" -> [1, 2]
            0xa1, b'b', 0x81, 0xa1, b'c', 0xc3, // "b" -> {"c": true}
        ]
    );
}

/// Longer than the 64 bytes the string buffer starts with, so the flush hook
/// has to grow it mid-object — twice for the payload and again between items.
#[test]
fn grows_the_buffer_across_a_long_payload() {
    let mut text = vec![b'x'; 300];
    let mut items = [
        string(&mut text),
        scalar(kObjectTypeInteger, object_data { integer: 42 }),
    ];
    let packed = pack(&mut array(&mut items));
    assert_eq!(&packed[..4], &[0x92, 0xda, 0x01, 0x2c]);
    assert!(packed[4..304].iter().all(|&byte| byte == b'x'));
    assert_eq!(&packed[304..], &[42]);
    assert_eq!(packed.len(), 305);
}
