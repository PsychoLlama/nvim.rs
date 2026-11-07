//! The character-set tables a C caller reaches the decoders through.
//!
//! The unit specs look a set up with `vterm_lookup_encoding` and call its
//! `init` and `decode` slots directly; a state machine's slot holds one of
//! the same three tables, and the decoders behind them are
//! [`crate::vterm::encoding`]'s.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `vterm/abi/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::NonNull;
use core::slice;

use crate::os::libc::{size_t, uint32_t};
use crate::types::{VTermEncoding, VTermEncodingType};
use crate::vterm::encoding::{self, Charset, Codepoints};

// ------------------------------------------------------ the character sets

/// Each slot of a character-set table is a C function the specs call
/// directly; these bridge them to the decoders.
extern "C" fn init_utf8(_enc: *mut VTermEncoding, scratch: *mut c_void) {
    // SAFETY: the caller's scratch area, the 16 bytes a
    // `VTermEncodingInstance` carries.
    let data = unsafe { &mut *scratch.cast::<[u8; 16]>() };
    Charset::Utf8.init(data);
}

/// The C `decode` slot of `charset`, over the caller's arrays.
///
/// # Safety
///
/// `cp` must point at `cplen` writable codepoints, `bytes` at `bytelen`
/// readable bytes, `cpi` and `pos` at live scalars, and `scratch` -- for the
/// UTF-8 decoder, which keeps state -- at the 16-byte scratch area of an
/// instance `init` was run on.
#[allow(clippy::too_many_arguments)]
unsafe fn decode_bridge(
    charset: Charset,
    scratch: *mut c_void,
    cp: *mut uint32_t,
    cpi: *mut c_int,
    cplen: c_int,
    bytes: *const c_char,
    pos: *mut size_t,
    bytelen: size_t,
) {
    // SAFETY: the caller's output array.
    let out = unsafe { slice::from_raw_parts_mut(cp, usize::try_from(cplen).unwrap_or(0)) };
    // SAFETY: the caller's input run.
    let input = unsafe { slice::from_raw_parts(bytes.cast::<u8>(), bytelen) };
    // SAFETY: the caller's two scalars.
    let (len, mut at) = unsafe { (usize::try_from(*cpi).unwrap_or(0), *pos) };
    let mut stateless = [0u8; 16];
    let data = if charset == Charset::Utf8 {
        // SAFETY: the instance's scratch, for the decoder that keeps state.
        unsafe { &mut *scratch.cast::<[u8; 16]>() }
    } else {
        &mut stateless
    };
    let mut sink = Codepoints { out, len };
    charset.decode_into(data, input, &mut at, &mut sink);
    let written =
        c_int::try_from(sink.len).expect("no more codepoints than the caller's `int` room");
    // SAFETY: as above -- the same two scalars, written back.
    unsafe { (*cpi, *pos) = (written, at) };
}

macro_rules! decode_slot {
    ($name:ident, $charset:expr) => {
        /// # Safety
        ///
        /// As for `decode_bridge`.
        #[allow(clippy::too_many_arguments)]
        unsafe extern "C" fn $name(
            _enc: *mut VTermEncoding,
            scratch: *mut c_void,
            cp: *mut uint32_t,
            cpi: *mut c_int,
            cplen: c_int,
            bytes: *const c_char,
            pos: *mut size_t,
            bytelen: size_t,
        ) {
            // SAFETY: the caller's promise, forwarded.
            unsafe { decode_bridge($charset, scratch, cp, cpi, cplen, bytes, pos, bytelen) };
        }
    };
}

decode_slot!(decode_utf8, Charset::Utf8);
decode_slot!(decode_usascii, Charset::UsAscii);
decode_slot!(decode_dec_drawing, Charset::DecDrawing);

pub(crate) static ENCODING_UTF8: VTermEncoding = VTermEncoding {
    init: Some(init_utf8),
    decode: Some(decode_utf8),
};

pub(crate) static ENCODING_USASCII: VTermEncoding = VTermEncoding {
    init: None,
    decode: Some(decode_usascii),
};

pub(crate) static ENCODING_DEC_DRAWING: VTermEncoding = VTermEncoding {
    init: None,
    decode: Some(decode_dec_drawing),
};

/// The table of the set `designation` names, or null.
#[unsafe(no_mangle)]
pub extern "C" fn vterm_lookup_encoding(
    type_0: VTermEncodingType,
    designation: c_char,
) -> *mut VTermEncoding {
    encoding::lookup(type_0, designation.cast_unsigned())
        .map_or(core::ptr::null_mut(), NonNull::as_ptr)
}
