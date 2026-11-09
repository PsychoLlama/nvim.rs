#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! libtermkey's public types: the key record, the kinds of key and their
//! symbols, and what a read answers.

// These are libtermkey's types, Copyright (c) 2007-2011 Paul Evans, under
// the MIT license; the notice is reproduced in
// licenses/libtermkey-LICENSE.txt.

pub type TermKeyEvent = ::core::ffi::c_uint;
pub type TermKeyFormat = ::core::ffi::c_uint;
/// One key, as the reader hands it back.
#[derive(Copy, Clone, Default, Debug)]
pub struct TermKeyKey {
    /// Which kind of key this is (`TERMKEY_TYPE_*`), which is also what
    /// says how to read `code`.
    pub kind: TermKeyType,
    pub code: KeyCode,
    pub modifiers: ::core::ffi::c_int,
    pub event: TermKeyEvent,
    /// The key's character as UTF-8, NUL-terminated, for a unicode key.
    pub utf8: [u8; 7],
}

/// A key's four-byte code: a character's codepoint, a function key's
/// number, a symbol, a control string's serial number, or the packed
/// payload of a mouse, position or mode report -- which one is what the
/// key's `kind` says.
///
/// The scalar readings are one `c_int` and the payload is the same four
/// bytes in memory order, so every reading is defined whichever was
/// written; reading one the key is not in gives a stale value, never an
/// undefined one. This is libtermkey's `code` union with the layout spelled
/// out instead of overlaid.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct KeyCode(::core::ffi::c_int);

impl KeyCode {
    /// A codepoint, number or symbol.
    pub const fn scalar(value: ::core::ffi::c_int) -> Self {
        Self(value)
    }

    /// A report's packed payload.
    pub fn report(payload: [u8; 4]) -> Self {
        Self(::core::ffi::c_int::from_ne_bytes(payload))
    }

    /// The character a `TERMKEY_TYPE_UNICODE` key stands for.
    pub const fn codepoint(self) -> ::core::ffi::c_int {
        self.0
    }

    /// The number of a `TERMKEY_TYPE_FUNCTION` key, or the serial number of
    /// a control string or an unrecognised control sequence.
    pub const fn number(self) -> ::core::ffi::c_int {
        self.0
    }

    /// The symbol of a `TERMKEY_TYPE_KEYSYM` key.
    pub const fn sym(self) -> TermKeySym {
        self.0
    }

    /// The four packed bytes of a mouse, position or mode report.
    pub fn payload(self) -> [u8; 4] {
        self.0.to_ne_bytes()
    }
}

pub type TermKeyMouseEvent = ::core::ffi::c_uint;
pub type TermKeyResult = ::core::ffi::c_uint;
pub type TermKeySym = ::core::ffi::c_int;
pub type TermKeyType = ::core::ffi::c_int;
