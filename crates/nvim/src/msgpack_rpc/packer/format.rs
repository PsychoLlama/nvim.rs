#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! msgpack encodings for the scalars and container headers the API packer
//! emits.
//!
//! Everything here is pure byte production — nothing touches the output
//! buffer — so the encodings can be pinned to exact byte sequences.
//!
//! The editor does not aim for the shortest possible encoding: several of the
//! width choices below are one step wider than msgpack allows, and one of them
//! (`uint64`) is three steps wider. The exact boundaries are part of the wire
//! format other implementations have been reading for years, so they are
//! reproduced as they are rather than tightened.

/// msgpack format bytes and sizes, kept out of the flat namespace the
/// unit-test cdef generator collects top-level constants into.
mod tag {
    /// The longest sequence any encoder here produces: a one-byte tag plus an
    /// eight-byte payload. The packer guarantees this much room per item.
    pub(super) const ITEM_MAX: usize = 9;

    pub(super) const NIL: u8 = 0xc0;
    pub(super) const FALSE: u8 = 0xc2;
    pub(super) const BIN8: u8 = 0xc4;
    pub(super) const BIN16: u8 = 0xc5;
    pub(super) const BIN32: u8 = 0xc6;
    pub(super) const EXT8: u8 = 0xc7;
    pub(super) const EXT16: u8 = 0xc8;
    pub(super) const EXT32: u8 = 0xc9;
    pub(super) const FLOAT64: u8 = 0xcb;
    pub(super) const UINT8: u8 = 0xcc;
    pub(super) const UINT16: u8 = 0xcd;
    pub(super) const UINT32: u8 = 0xce;
    pub(super) const UINT64: u8 = 0xcf;
    pub(super) const INT8: u8 = 0xd0;
    pub(super) const INT16: u8 = 0xd1;
    pub(super) const INT32: u8 = 0xd2;
    pub(super) const INT64: u8 = 0xd3;
    pub(super) const FIXEXT1: u8 = 0xd4;
    pub(super) const FIXEXT2: u8 = 0xd5;
    pub(super) const STR8: u8 = 0xd9;
    pub(super) const STR16: u8 = 0xda;
    pub(super) const STR32: u8 = 0xdb;
    pub const ARRAY16: u8 = 0xdc;
    pub(super) const ARRAY32: u8 = 0xdd;
    pub(super) const MAP16: u8 = 0xde;
    pub(super) const MAP32: u8 = 0xdf;

    pub(super) const FIXSTR: u8 = 0xa0;
    pub(super) const FIXARRAY: u8 = 0x90;
    pub(super) const FIXMAP: u8 = 0x80;
}

use tag::ITEM_MAX;

/// One encoded msgpack item: a tag and however much of a payload fits beside
/// it. Never longer than [`ITEM_MAX`].
#[derive(PartialEq, Eq, Debug)]
pub struct Item {
    bytes: [u8; ITEM_MAX],
    len: usize,
}

impl Item {
    const fn empty() -> Self {
        Item {
            bytes: [0; ITEM_MAX],
            len: 0,
        }
    }

    fn of(bytes: &[u8]) -> Self {
        let mut item = Item::empty();
        item.push_all(bytes);
        item
    }

    fn push(&mut self, byte: u8) {
        self.bytes[self.len] = byte;
        self.len += 1;
    }

    fn push_all(&mut self, bytes: &[u8]) {
        self.bytes[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

/// The nil object, and the only encoding that is a bare byte.
pub const NIL: u8 = tag::NIL;

pub fn boolean(value: bool) -> Item {
    Item::of(&[tag::FALSE | u8::from(value)])
}

/// An unsigned value up to 32 bits.
pub fn uint(value: u32) -> Item {
    if value > 0xffff {
        let mut item = Item::of(&[tag::UINT32]);
        item.push_all(&value.to_be_bytes());
        item
    } else if value > 0xff {
        let mut item = Item::of(&[tag::UINT16]);
        item.push_all(
            &u16::try_from(value)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else if value > 0x7f {
        Item::of(&[
            tag::UINT8,
            u8::try_from(value).expect("the msgpack width chosen above bounds this"),
        ])
    } else {
        Item::of(&[u8::try_from(value).expect("the msgpack width chosen above bounds this")])
    }
}

/// An unsigned value up to 64 bits.
///
/// The cutover to the 64-bit form is at 0x0fff_ffff rather than 0xffff_ffff,
/// so values in between are written eight bytes wide when four would do. That
/// is upstream's boundary (a `0xfffffff` literal one digit short of the 32-bit
/// maximum) and it is on the wire, not just in memory.
pub fn uint64(value: u64) -> Item {
    if value > 0xfff_ffff {
        let mut item = Item::of(&[tag::UINT64]);
        item.push_all(&value.to_be_bytes());
        item
    } else {
        uint(u32::try_from(value).expect("the msgpack width chosen above bounds this"))
    }
}

/// A signed value. Non-negative values go through [`uint64`], so they inherit
/// its early cutover.
pub fn integer(value: i64) -> Item {
    if value >= 0 {
        return uint64(value.cast_unsigned());
    }
    if value < -0x8000_0000 {
        let mut item = Item::of(&[tag::INT64]);
        item.push_all(&value.to_be_bytes());
        item
    } else if value < -0x8000 {
        let mut item = Item::of(&[tag::INT32]);
        item.push_all(
            &i32::try_from(value)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else if value < -0x80 {
        let mut item = Item::of(&[tag::INT16]);
        item.push_all(
            &i16::try_from(value)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else if value < -0x20 {
        Item::of(&[
            tag::INT8,
            i8::try_from(value)
                .expect("the msgpack width chosen above bounds this")
                .cast_unsigned(),
        ])
    } else {
        Item::of(&[i8::try_from(value)
            .expect("the msgpack width chosen above bounds this")
            .cast_unsigned()])
    }
}

/// Always the 64-bit form; the packer has no single-precision path.
pub fn float8(value: f64) -> Item {
    let mut item = Item::of(&[tag::FLOAT64]);
    item.push_all(&value.to_bits().to_be_bytes());
    item
}

pub fn array_header(len: u32) -> Item {
    if len < 0x10 {
        Item::of(&[
            tag::FIXARRAY | u8::try_from(len).expect("the msgpack width chosen above bounds this")
        ])
    } else if len < 0x10000 {
        let mut item = Item::of(&[ARRAY16]);
        item.push_all(
            &u16::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else {
        let mut item = Item::of(&[tag::ARRAY32]);
        item.push_all(&len.to_be_bytes());
        item
    }
}

pub fn map_header(len: u32) -> Item {
    if len < 0x10 {
        Item::of(&[
            tag::FIXMAP | u8::try_from(len).expect("the msgpack width chosen above bounds this")
        ])
    } else if len < 0x10000 {
        let mut item = Item::of(&[tag::MAP16]);
        item.push_all(
            &u16::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else {
        let mut item = Item::of(&[tag::MAP32]);
        item.push_all(&len.to_be_bytes());
        item
    }
}

/// The tag of a 16-bit array header. The UI event stream writes one with a
/// placeholder length and patches it once the batch is complete.
pub use tag::ARRAY16;

/// The one-byte header for a string of fewer than 32 bytes.
///
/// Panics above that: the callers that reach for it are writing UI event and
/// method names, all of which are fixed and short.
pub fn fixstr_header(len: usize) -> Item {
    assert!(len < 32, "not a fixstr");
    Item::of(
        &[tag::FIXSTR | u8::try_from(len).expect("the msgpack width chosen above bounds this")],
    )
}

/// The header for a string of `len` bytes, or `None` when no msgpack string
/// header can describe it.
///
/// Each width is chosen with a strict `<`, so a length that exactly saturates
/// one width (255, 65535) steps up to the next. Harmless, and preserved.
pub fn str_header(len: usize) -> Option<Item> {
    if len < 32 {
        Some(Item::of(&[tag::FIXSTR
            | u8::try_from(len)
                .expect("the msgpack width chosen above bounds this")]))
    } else if len < 0xff {
        Some(Item::of(&[
            tag::STR8,
            u8::try_from(len).expect("the msgpack width chosen above bounds this"),
        ]))
    } else if len < 0xffff {
        let mut item = Item::of(&[tag::STR16]);
        item.push_all(
            &u16::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        Some(item)
    } else if len < 0xffff_ffff {
        let mut item = Item::of(&[tag::STR32]);
        item.push_all(
            &u32::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        Some(item)
    } else {
        None
    }
}

/// The header for a binary blob of `len` bytes. Same saturation quirk as
/// [`str_header`].
pub fn bin_header(len: usize) -> Option<Item> {
    if len < 0xff {
        Some(Item::of(&[
            tag::BIN8,
            u8::try_from(len).expect("the msgpack width chosen above bounds this"),
        ]))
    } else if len < 0xffff {
        let mut item = Item::of(&[tag::BIN16]);
        item.push_all(
            &u16::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        Some(item)
    } else if len < 0xffff_ffff {
        let mut item = Item::of(&[tag::BIN32]);
        item.push_all(
            &u32::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        Some(item)
    } else {
        None
    }
}

/// The header for an extension payload of `len` bytes, including the type
/// byte.
///
/// Only the one- and two-byte fixed widths are used; msgpack's fixext4,
/// fixext8 and fixext16 are never emitted, so a four-byte payload takes the
/// ext8 form. Note the `<=` at the ext8 boundary where the string and binary
/// headers use `<`.
pub fn ext_header(len: usize, ext_type: i8) -> Option<Item> {
    let mut item = if len == 1 {
        Item::of(&[tag::FIXEXT1])
    } else if len == 2 {
        Item::of(&[tag::FIXEXT2])
    } else if len <= 0xff {
        Item::of(&[tag::EXT8])
    } else if len < 0xffff {
        let mut item = Item::of(&[tag::EXT16]);
        item.push_all(
            &u16::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else if len < 0xffff_ffff {
        let mut item = Item::of(&[tag::EXT32]);
        item.push_all(
            &u32::try_from(len)
                .expect("the msgpack width chosen above bounds this")
                .to_be_bytes(),
        );
        item
    } else {
        return None;
    };
    item.push(ext_type.cast_unsigned());
    Some(item)
}

/// A buffer, window or tabpage handle, as a complete extension object.
///
/// Small handles get a fixext1 carrying the raw byte — including the negative
/// ones, which is why the range starts below zero even though the wider form
/// asserts the handle is not negative. Anything larger is an ext8 wrapping the
/// handle's [`uint`] encoding, so a two-byte handle spends an ext8 header
/// where fixext2 would have done.
pub fn handle(ext_type: i8, value: i32) -> Item {
    if (-0x1f..=0x7f).contains(&value) {
        return Item::of(&[
            tag::FIXEXT1,
            ext_type.cast_unsigned(),
            i8::try_from(value)
                .expect("the msgpack width chosen above bounds this")
                .cast_unsigned(),
        ]);
    }
    assert!(value >= 0, "handles are allocated upward from zero");
    let payload = uint(value.cast_unsigned());
    let mut item = Item::of(&[
        tag::EXT8,
        u8::try_from(payload.len).expect("the msgpack width chosen above bounds this"),
        ext_type.cast_unsigned(),
    ]);
    item.push_all(payload.bytes());
    item
}

/// A stack that keeps its first two entries out of the heap.
///
/// The packer walks nested containers iteratively and needs somewhere to
/// remember the container it came from. Most objects it sends nest one or two
/// deep, and it runs per RPC message, so the common case has to stay free of
/// allocation — which is what upstream's inline-array vector bought.
pub struct SmallStack<T> {
    inline: [Option<T>; 2],
    inline_len: usize,
    spill: Vec<T>,
}

impl<T> Default for SmallStack<T> {
    fn default() -> Self {
        SmallStack {
            inline: [None, None],
            inline_len: 0,
            spill: Vec::new(),
        }
    }
}

impl<T> SmallStack<T> {
    pub fn push(&mut self, value: T) {
        if self.inline_len < self.inline.len() {
            self.inline[self.inline_len] = Some(value);
            self.inline_len += 1;
        } else {
            self.spill.push(value);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(value) = self.spill.pop() {
            return Some(value);
        }
        self.inline_len = self.inline_len.checked_sub(1)?;
        self.inline[self.inline_len].take()
    }

    /// Whether the stack has ever reached the heap. Only the tests care.
    #[cfg(test)]
    pub fn spilled(&self) -> bool {
        self.spill.capacity() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_keeps_two_entries_inline() {
        let mut stack = SmallStack::default();
        stack.push(1);
        stack.push(2);
        assert!(!stack.spilled());
        stack.push(3);
        assert!(stack.spilled());
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.pop(), None);
    }
}
