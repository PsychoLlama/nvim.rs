#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Decoding a generated keyset out of raw msgpack, and the token-cursor
//! primitives it is built from.
//!
//! Nothing here is part of the streaming path: these read a value out of a
//! buffer the caller already holds whole, advancing a `(pointer, size)`
//! cursor as they go. The RPC transport never calls them — ShaDa does, for
//! every entry it reads — but they live beside [`super`] because they share
//! its token reader and its keyset layout.

use core::ffi::{c_char, c_int, c_void};

use crate::memory::{xrealloc, xstrdup};
use crate::mpack::mpack_core::mpack_rtoken;
use crate::mpack::object::{mpack_parse, mpack_parser_init};
use crate::strings::arena_printf;
use crate::types::{
    AdditionalData, AdditionalDataBuilder, Boolean, FieldHashfn, Integer, KeySetLink, OptKeySet,
    String_0, StringArray, mpack_parser_t, mpack_token_t, size_t, ssize_t, uint32_t,
};
use ::libc::abort;

use super::protocol;
use super::{
    TOKEN_ARRAY, TOKEN_BIN, TOKEN_MAP, TOKEN_STR, field_type, parse_nop, unpack_integer_token,
};

/// Reads a string or binary token, returning a borrow of the buffer rather
/// than a copy. An empty result means the next token was not one.
///
/// # Safety
/// `data`/`size` are a writable cursor over a live buffer.
pub unsafe fn unpack_string(data: *mut *const c_char, size: *mut size_t) -> String_0 {
    // SAFETY: the caller's cursor and the buffer it describes.
    let mut data2: *const c_char = unsafe { *data };
    let mut size2: size_t = unsafe { *size };
    let mut tok: mpack_token_t = unsafe { core::mem::zeroed() };
    if unsafe { mpack_rtoken(&raw mut data2, &raw mut size2, &raw mut tok) } != 0
        || tok.type_0 != TOKEN_STR && tok.type_0 != TOKEN_BIN
    {
        return String_0::NULL;
    }
    // Checked against the *original* size, so a token header that
    // consumed several bytes leaves that much slack. Upstream's bound;
    // the caller only ever reads within the buffer it owns.
    if unsafe { *size } < tok.length as size_t {
        return String_0::NULL;
    }
    unsafe { *data = data2.add(tok.length as usize) };
    unsafe { *size = size2 - tok.length as size_t };
    String_0::from_raw_parts(data2.cast_mut(), tok.length as size_t)
}

/// The length of the array that starts here, or -1 if this is not one.
///
/// # Safety
/// [`unpack_string`]'s contract.
pub unsafe fn unpack_array(data: *mut *const c_char, size: *mut size_t) -> ssize_t {
    // SAFETY: the caller's cursor.
    let tok = unsafe {
        let mut tok: mpack_token_t = core::mem::zeroed();
        if mpack_rtoken(data, size, &raw mut tok) != 0 {
            return -1;
        }
        tok
    };
    if tok.type_0 != TOKEN_ARRAY {
        return -1;
    }
    // A length past `ssize_t::MAX` is unreachable on any target the editor
    // builds for, and reads as "not an array" to every caller — which is what
    // upstream's `(ssize_t)` cast produced there too.
    ssize_t::try_from(tok.length).unwrap_or(-1)
}

/// # Safety
/// [`unpack_string`]'s contract, and `res` points at a writable `Integer`.
pub unsafe fn unpack_integer(
    data: *mut *const c_char,
    size: *mut size_t,
    res: *mut Integer,
) -> bool {
    // SAFETY: the caller's cursor.
    let tok = unsafe {
        let mut tok: mpack_token_t = core::mem::zeroed();
        if mpack_rtoken(data, size, &raw mut tok) != 0 {
            return false;
        }
        tok
    };
    match unpack_integer_token(tok) {
        // SAFETY: the caller's out-parameter.
        Some(value) => {
            unsafe { *res = value };
            true
        }
        None => false,
    }
}

/// Steps over one whole value without building anything from it.
///
/// # Safety
/// [`unpack_string`]'s contract.
pub unsafe fn unpack_skip(data: *mut *const c_char, size: *mut size_t) -> c_int {
    // SAFETY: the caller's cursor, and a parser that builds nothing.
    let mut parser: mpack_parser_t = unsafe { core::mem::zeroed() };
    unsafe { mpack_parser_init(&raw mut parser, 0) };
    unsafe {
        mpack_parse(
            &raw mut parser,
            data,
            size,
            Some(parse_nop),
            Some(parse_nop),
        )
    }
}

/// Appends one unrecognised key's raw msgpack to a keyset's spillover.
///
/// The builder's bytes are an [`AdditionalData`] header followed by the
/// concatenated items, so the header is written on the first push and its
/// counters updated on every one.
///
/// # Safety
/// `ad` points at a live builder and `data` at `size` readable bytes.
pub unsafe fn push_additional_data(
    ad: *mut AdditionalDataBuilder,
    data: *const c_char,
    size: size_t,
) {
    // SAFETY: the caller's builder and bytes; `reserve` makes the room each
    // copy needs.
    if unsafe { (*ad).size } == 0 {
        let header = AdditionalData {
            nitems: 0,
            nbytes: 0,
            data: [],
        };
        unsafe { reserve(ad, size_of::<AdditionalData>()) };
        unsafe {
            (*ad).items.add((*ad).size).copy_from_nonoverlapping(
                (&raw const header).cast::<c_char>(),
                size_of::<AdditionalData>(),
            )
        };
        unsafe { (*ad).size += size_of::<AdditionalData>() };
    }

    let header: *mut AdditionalData = unsafe { (*ad).items }.cast::<AdditionalData>();
    unsafe { (*header).nitems += 1 };
    // One unrecognised item cannot be 4 GiB long: saturating is the safe
    // reading of upstream's bare `(uint32_t)`, and unreachable either way.
    unsafe { (*header).nbytes += uint32_t::try_from(size).unwrap_or(uint32_t::MAX) };

    if size > 0 {
        unsafe { reserve(ad, size) };
        unsafe {
            (*ad)
                .items
                .add((*ad).size)
                .copy_from_nonoverlapping(data, size)
        };
        unsafe { (*ad).size += size };
    }
}

/// Makes room for `extra` more bytes in a builder.
///
/// # Safety
/// `ad` points at a live builder whose `items` is either null or an
/// allocation of `capacity` bytes.
unsafe fn reserve(ad: *mut AdditionalDataBuilder, extra: size_t) {
    // SAFETY: the caller's builder.
    if unsafe { (*ad).capacity } >= unsafe { (*ad).size } + extra {
        return;
    }
    unsafe { (*ad).capacity = protocol::capacity_for((*ad).size + extra) };
    unsafe {
        (*ad).items = xrealloc((*ad).items.cast::<c_void>(), (*ad).capacity).cast::<c_char>()
    };
    assert!(!unsafe { (*ad).items }.is_null());
}

/// Decodes a msgpack map straight into a generated keyset struct.
///
/// Fields the keyset does not know about are skipped and, if the caller
/// supplied a builder, kept verbatim so they survive a round trip. `error` is
/// set to an owned message on failure.
///
/// # Safety
/// `retval` points at the keyset `hashy` describes, `ad` is null or a live
/// builder, `data`/`size` are a cursor over a live buffer, and `error` points
/// at a writable slot.
pub unsafe fn unpack_keydict(
    retval: *mut c_void,
    hashy: FieldHashfn,
    ad: *mut AdditionalDataBuilder,
    data: *mut *const c_char,
    size: *mut size_t,
    error: *mut *mut c_char,
) -> bool {
    let ks: *mut OptKeySet = retval.cast::<OptKeySet>();
    // SAFETY: the caller's cursor and error slot.
    let tok = unsafe {
        let mut tok: mpack_token_t = core::mem::zeroed();
        if mpack_rtoken(data, size, &raw mut tok) != 0 || tok.type_0 != TOKEN_MAP {
            *error = xstrdup(c"is not a dict".as_ptr());
            return false;
        }
        tok
    };

    for _ in 0..tok.length {
        // SAFETY: the caller's cursor, keyset and error slot; `key` borrows
        // the buffer the cursor walks.
        let item_start: *const c_char = unsafe { *data };
        let key = unsafe { unpack_string(data, size) };
        if key.data().is_null() {
            unsafe { *error = fail(c"has key value which is not a string", key) };
            return false;
        }
        if key.is_empty() {
            unsafe { *error = fail(c"has empty key", key) };
            return false;
        }

        let field: *const KeySetLink =
            unsafe { hashy.expect("keyset has no hash function")(key.data(), key.len()) };
        if field.is_null() {
            if unsafe { unpack_skip(data, size) } != 0 {
                return false;
            }
            if !ad.is_null() {
                unsafe { push_additional_data(ad, item_start, (*data).addr() - item_start.addr()) };
            }
            continue;
        }

        debug_assert!(unsafe { (*field).opt_index } >= 0);
        let flag = 1u64 << unsafe { (*field).opt_index };
        if unsafe { (*ks).is_set_ } & flag != 0 {
            unsafe { *error = xstrdup(c"duplicate key".as_ptr()) };
            return false;
        }
        unsafe { (*ks).is_set_ |= flag };

        let mem = unsafe { retval.cast::<c_char>().add((*field).ptr_off) };
        if let Err(message) = unsafe { unpack_field(mem, (*field).type_0, data, size) } {
            unsafe { *error = fail(message, key) };
            return false;
        }
    }
    true
}

/// Decodes one keyset field's value into the slot at `mem`.
///
/// The error is the complaint's format string; the caller turns it into an
/// owned message naming the key.
///
/// # Safety
/// [`unpack_keydict`]'s contract, and `mem` points at storage of the kind
/// `type_0` names.
unsafe fn unpack_field(
    mem: *mut c_char,
    type_0: c_int,
    data: *mut *const c_char,
    size: *mut size_t,
) -> Result<(), &'static core::ffi::CStr> {
    // SAFETY: the caller's slot and cursor; each arm writes the kind
    // `type_0` names.
    match type_0 {
        field_type::BOOLEAN => {
            // Read straight off the wire: both boolean encodings differ
            // only in their low bit.
            if unsafe { *size } == 0 || c_int::from(unsafe { **data }) & 0xfe != 0xc2 {
                return Err(c"has %.*s key value which is not a boolean");
            }
            unsafe { *mem.cast::<Boolean>() = c_int::from(**data) & 0x1 != 0 };
            unsafe { *data = (*data).add(1) };
            unsafe { *size -= 1 };
        }
        field_type::INTEGER => {
            if !unsafe { unpack_integer(data, size, mem.cast::<Integer>()) } {
                return Err(c"has %.*s key value which is not an integer");
            }
        }
        field_type::STRING => {
            let val = unsafe { unpack_string(data, size) };
            if val.data().is_null() {
                return Err(c"has %.*s key value which is not a binary");
            }
            unsafe { *mem.cast::<String_0>() = val };
        }
        field_type::STRING_ARRAY => {
            let len = unsafe { unpack_array(data, size) };
            if len < 0 {
                return Err(c"has %.*s key with non-array value");
            }
            return unsafe {
                unpack_string_array(mem.cast::<StringArray>(), len.cast_unsigned(), data, size)
            };
        }
        _ => unsafe { abort() },
    }
    Ok(())
}

/// Appends `len` strings to the hand-rolled vector at `a`.
///
/// # Safety
/// [`unpack_field`]'s contract for a string-array field.
unsafe fn unpack_string_array(
    a: *mut StringArray,
    len: size_t,
    data: *mut *const c_char,
    size: *mut size_t,
) -> Result<(), &'static core::ffi::CStr> {
    // SAFETY: the caller's vector and cursor; every growth reallocates to the
    // capacity it then writes within.
    if unsafe { (*a).capacity } < unsafe { (*a).size } + len {
        unsafe { (*a).capacity = protocol::capacity_for((*a).size + len) };
        unsafe {
            (*a).items = xrealloc(
                (*a).items.cast::<c_void>(),
                size_of::<String_0>() * (*a).capacity,
            )
            .cast::<String_0>()
        };
    }
    for _ in 0..len {
        let item = unsafe { unpack_string(data, size) };
        if item.data().is_null() {
            return Err(c"has %.*s array with non-binary value");
        }
        if unsafe { (*a).size } == unsafe { (*a).capacity } {
            unsafe { (*a).capacity = protocol::grown_capacity((*a).capacity) };
            unsafe {
                (*a).items = xrealloc(
                    (*a).items.cast::<c_void>(),
                    size_of::<String_0>() * (*a).capacity,
                )
                .cast::<String_0>()
            };
        }
        unsafe { *(*a).items.add((*a).size) = item };
        unsafe { (*a).size += 1 };
    }
    Ok(())
}

/// An owned copy of one of `unpack_keydict`'s complaints. Messages that name
/// the offending key spell it `%.*s`; the extra arguments are harmless to the
/// ones that do not.
///
/// # Safety
/// `key` describes `key.size` readable bytes.
unsafe fn fail(message: &core::ffi::CStr, key: String_0) -> *mut c_char {
    // SAFETY: the caller's key, and verbs that match it.
    unsafe {
        arena_printf(
            core::ptr::null_mut(),
            message.as_ptr(),
            // `%.*s`'s precision, which upstream writes as `(int)key.size`.
            // A 2 GiB keydict key cannot reach here — it would have to be a
            // ShaDa entry that big — and if one did, saturating to
            // `c_int::MAX` would print two gigabytes of it into an error
            // message. `len_as_int` says so instead.
            crate::narrow::len_as_int(key.len()),
            key.data(),
        )
    }
    .data()
}
