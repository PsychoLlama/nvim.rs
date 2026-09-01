//! Reading entries out of a ShaDa file.
//!
//! Every entry on the wire is four things in a row: a type, a timestamp, a
//! byte length, and that many bytes of msgpack. The three integers are
//! unpacked one byte at a time by [`read_uint64`] so that nothing has to be
//! buffered or seeked over; the payload is then handed to whichever
//! `parse_*` below matches the type.
//!
//! The length is what makes an unknown entry skippable, and it is why an
//! entry this Nvim does not understand can still be copied through to the
//! file it writes: see `merge`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::{c_str, emsg_text};
use crate::semsg;
use crate::tr_c;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

use crate::msgpack_rpc::unpacker::{MPACK_EOF, MPACK_OK};
use crate::types::FieldHashfn;

use super::*;
use crate::os::cshim::gettext;

/// Report `E576`/`E886` about the file, naming a byte offset or a length.
///
/// Every message here ends in one variadic call; writing it once keeps the
/// unchecked part of reporting to these two functions.
fn shada_read_error(fmt: &'static CStr, at: uint64_t) {
    emsg_text(tr_c!(fmt, at));
}

/// As [`shada_read_error`], for the messages whose `%s` is a libuv error or
/// a fixed reason.
fn shada_read_error_why(fmt: &'static CStr, why: *const c_char) {
    // SAFETY: `why` is a NUL-terminated libuv message or a fixed reason.
    let why = unsafe { c_str(why) };
    emsg_text(tr_c!(fmt, why));
}

/// The largest entry type this Nvim knows. Anything above it is an unknown
/// entry, kept only to be written back out.
const SHADA_LAST_ENTRY: u64 = kSDItemChange as u64;

/// A msgpack cursor over the bytes of one entry.
///
/// The `unpack_*` helpers all take the remaining pointer and length by
/// reference and advance both; this keeps that pair together so the callers
/// below read as "take an array, take a string, take an integer".
pub(crate) struct Cursor {
    pub(crate) at: *const c_char,
    pub(crate) left: size_t,
}

impl Cursor {
    /// The element count of an array, or −1 if the next token is not one.
    pub(crate) fn array(&mut self) -> ssize_t {
        // SAFETY: `at`/`left` describe a live buffer of that many bytes, and
        // every helper here only ever advances inside it.
        unsafe { unpack_array(&raw mut self.at, &raw mut self.left) }
    }

    /// A string or binary token; `data` is null if the next token is neither.
    /// The result points into the entry's bytes, so it must be copied to
    /// outlive them.
    pub(crate) fn string(&mut self) -> String_0 {
        unsafe { unpack_string(&raw mut self.at, &raw mut self.left) }
    }

    /// An integer token, or `None` if the next token is not one.
    pub(crate) fn integer(&mut self) -> Option<Integer> {
        let mut value: Integer = 0;
        unsafe { unpack_integer(&raw mut self.at, &raw mut self.left, &raw mut value) }
            .then_some(value)
    }

    /// Step over one whole value, however deep. Answers an `MPACK_*` status.
    pub(crate) fn skip(&mut self) -> c_int {
        unsafe { unpack_skip(&raw mut self.at, &raw mut self.left) }
    }

    /// A map, into the keyset `into` describes. Keys the keyset does not
    /// name are pushed onto `extra` to be written back out unchanged.
    pub(crate) fn keydict(
        &mut self,
        into: *mut c_void,
        field: FieldHashfn,
        extra: &mut AdditionalDataBuilder,
        error: &mut *mut c_char,
    ) -> bool {
        unsafe {
            unpack_keydict(
                into,
                field,
                extra,
                &raw mut self.at,
                &raw mut self.left,
                error,
            )
        }
    }

    /// One value as a Vimscript value. Answers an `MPACK_*` status.
    pub(crate) fn typval(&mut self, into: *mut typval_T) -> c_int {
        unsafe { unpack_typval(&raw mut self.at, &raw mut self.left, into) }
    }
}

/// The bytes of one entry.
///
/// Small entries fit in the reader's own buffer and are read in place;
/// anything bigger gets a buffer of its own. That buffer stays `xmalloc`ed
/// rather than becoming a `Vec` because an unknown entry hands it straight
/// to `ShadaEntry`, which frees it with `xfree`.
struct Body {
    ptr: *mut c_char,
    len: size_t,
    /// Whether `ptr` is ours to release (or to give away).
    owned: bool,
}

impl Body {
    /// Read `len` bytes of entry payload.
    unsafe fn read(sd_reader: *mut FileDescriptor, len: size_t) -> Result<Self, ShaDaReadResult> {
        let buffered = unsafe { file_try_read_buffered(sd_reader, len) };
        if !buffered.is_null() {
            return Ok(Body {
                ptr: buffered,
                len,
                owned: false,
            });
        }
        let ptr = unsafe { xmalloc(len) }.cast::<c_char>();
        let body = Body {
            ptr,
            len,
            owned: true,
        };
        match unsafe { fread_len(sd_reader, ptr, len) } {
            kSDReadStatusSuccess => Ok(body),
            other => Err(other),
        }
    }

    fn cursor(&self) -> Cursor {
        Cursor {
            at: self.ptr,
            left: self.len,
        }
    }

    /// The bytes, as something the entry can own and `xfree` later.
    fn into_owned(mut self) -> *mut c_char {
        let ptr = if self.owned {
            self.ptr
        } else {
            // SAFETY: `ptr` covers `len` readable bytes.
            unsafe { xmemdup(self.ptr.cast::<c_void>(), self.len).cast::<c_char>() }
        };
        self.owned = false;
        ptr
    }
}

impl Drop for Body {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: ours, from `xmalloc`, released exactly here.
            unsafe { xfree(self.ptr.cast::<c_void>()) };
        }
    }
}

/// Read exactly `length` bytes, complaining if the file is shorter.
pub(crate) unsafe fn fread_len(
    sd_reader: *mut FileDescriptor,
    buffer: *mut c_char,
    length: size_t,
) -> ShaDaReadResult {
    let read_bytes = unsafe { file_read(sd_reader, buffer, length) };
    if read_bytes < 0 {
        shada_read_error_why(c"E886: System error while reading ShaDa file: %s", unsafe {
            uv_strerror(read_bytes as c_int)
        });
        return kSDReadStatusReadError;
    }
    if read_bytes != length as ptrdiff_t {
        shada_read_error(c"E576: Error while reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier", length as uint64_t);
        return kSDReadStatusNotShaDa;
    }
    kSDReadStatusSuccess
}

/// Step over `offset` bytes, complaining if the file is shorter.
pub(crate) unsafe fn sd_reader_skip(
    sd_reader: *mut FileDescriptor,
    offset: size_t,
) -> ShaDaReadResult {
    let skip_bytes = unsafe { file_skip(sd_reader, offset) };
    if skip_bytes < 0 {
        shada_read_error_why(
            c"E886: System error while skipping in ShaDa file: %s",
            unsafe { uv_strerror(skip_bytes as c_int) },
        );
        return kSDReadStatusReadError;
    }
    if skip_bytes != offset as ptrdiff_t {
        assert!(skip_bytes < offset as ptrdiff_t, "skipped past end of file");
        if unsafe { file_eof(sd_reader) } {
            shada_read_error(c"E576: Reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier", offset as uint64_t);
        } else {
            shada_read_error_why(
                c"E886: System error while skipping in ShaDa file: %s",
                gettext(c"too few bytes read").as_ptr(),
            );
        }
        return kSDReadStatusNotShaDa;
    }
    kSDReadStatusSuccess
}

/// Read one unsigned integer, one byte at a time.
///
/// Unlike msgpack's own reader this consumes *exactly* the bytes the value
/// needs, which is what lets the entry header be read without a buffer or a
/// seek. One byte is always consumed, even when it turns out to be wrong.
///
/// `Err(kSDReadStatusFinished)` means end of file, and is only produced when
/// `allow_eof` — i.e. for the first of the three header integers.
unsafe fn read_uint64(
    sd_reader: *mut FileDescriptor,
    allow_eof: bool,
) -> Result<uint64_t, ShaDaReadResult> {
    let fpos = unsafe { (*sd_reader).bytes_read };
    let mut first: uint8_t = 0;
    let read_bytes = unsafe { file_read(sd_reader, (&raw mut first).cast::<c_char>(), 1) };
    if read_bytes < 0 {
        shada_read_error_why(
            c"E886: System error while reading integer from ShaDa file: %s",
            unsafe { uv_strerror(read_bytes as c_int) },
        );
        return Err(kSDReadStatusReadError);
    }
    if read_bytes == 0 {
        if allow_eof && unsafe { file_eof(sd_reader) } {
            return Err(kSDReadStatusFinished);
        }
        shada_read_error(c"E576: Error while reading ShaDa file: expected positive integer at position %lu, but got nothing", fpos);
        return Err(kSDReadStatusNotShaDa);
    }

    // A positive fixint is its own value; the four uint formats say how
    // many big-endian bytes follow.
    if first & 0x80 == 0 {
        return Ok(first as uint64_t);
    }
    let length: usize = match first {
        0xcc => 1,
        0xcd => 2,
        0xce => 4,
        0xcf => 8,
        _ => {
            shada_read_error(
                c"E576: Error while reading ShaDa file: expected positive integer at position %lu",
                fpos,
            );
            return Err(kSDReadStatusNotShaDa);
        }
    };
    let mut bytes = [0u8; 8];
    match unsafe {
        fread_len(
            sd_reader,
            (&raw mut bytes[8 - length]).cast::<c_char>(),
            length,
        )
    } {
        kSDReadStatusSuccess => Ok(uint64_t::from_be_bytes(bytes)),
        other => Err(other),
    }
}

/// What the msgpack parser made of an entry's payload.
pub(crate) unsafe fn shada_check_status(
    initial_fpos: uintmax_t,
    status: c_int,
    remaining: size_t,
) -> ShaDaReadResult {
    if status == MPACK_OK {
        if remaining != 0 {
            shada_read_error(
                c"E576: Failed to parse ShaDa file: extra bytes in msgpack string at position %lu",
                initial_fpos as uint64_t,
            );
            return kSDReadStatusNotShaDa;
        }
        return kSDReadStatusSuccess;
    }
    let at = initial_fpos as uint64_t;
    if status == MPACK_EOF {
        semsg!("E576: Failed to parse ShaDa file: incomplete msgpack string at position {at}");
    } else {
        semsg!("E576: Failed to parse ShaDa file due to a msgpack parser error at position {at}");
    }
    kSDReadStatusNotShaDa
}

/// The type, timestamp and length that head every entry.
pub(crate) struct Header {
    /// The raw type: not yet known to be a [`ShadaEntryType`].
    pub(crate) type_u64: uint64_t,
    timestamp: uint64_t,
    length: size_t,
    /// Where in the file the entry started, for the error messages.
    pub(crate) fpos: uint64_t,
}

unsafe fn read_header(sd_reader: *mut FileDescriptor) -> Result<Header, ShaDaReadResult> {
    let fpos = unsafe { (*sd_reader).bytes_read };
    let type_u64 = unsafe { read_uint64(sd_reader, true) }?;
    let timestamp = unsafe { read_uint64(sd_reader, false) }?;
    let length_u64 = unsafe { read_uint64(sd_reader, false) }?;

    if length_u64 > PTRDIFF_MAX as uint64_t {
        shada_read_error(c"E576: Error while reading ShaDa file: there is an item at position %lu that is stated to be too long", fpos);
        return Err(kSDReadStatusNotShaDa);
    }
    if type_u64 == 0 {
        // `kSDItemUnknown` cannot get this far — it is −1, which
        // `read_uint64` rejects — but `kSDItemMissing` can, and it would
        // otherwise be skipped silently because `1 << 0` is never in the
        // flags.
        shada_read_error(c"E576: Error while reading ShaDa file: there is an item at position %lu that must not be there: Missing items are for internal uses only", fpos);
        return Err(kSDReadStatusNotShaDa);
    }
    Ok(Header {
        type_u64,
        timestamp,
        length: length_u64 as size_t,
        fpos,
    })
}

/// Whether the caller asked for this entry type at this size.
fn wanted(header: &Header, flags: c_uint, max_kbyte: size_t) -> bool {
    let by_type = if header.type_u64 > SHADA_LAST_ENTRY {
        flags & kSDReadUnknown as c_uint != 0
    } else {
        (1u32 << header.type_u64) & flags != 0
    };
    by_type && (max_kbyte == 0 || header.length <= max_kbyte.wrapping_mul(1024))
}

/// What to do with an entry once its header has been read.
enum Disposition {
    /// Parse it into the caller's entry.
    Parse,
    /// Skip its bytes without looking at them.
    Skip,
    /// Parse it only far enough to prove the file is msgpack at all, then
    /// throw it away. This is the "is this a ShaDa file?" check: the first
    /// entry of a real one is a header, so an unknown first entry (or a
    /// literal `\n`, i.e. a Vim viminfo file) is most likely not ShaDa, and
    /// a non-msgpack payload proves it.
    VerifyOnly,
}

fn disposition(header: &Header, flags: c_uint, max_kbyte: size_t) -> Disposition {
    if wanted(header, flags, max_kbyte) {
        Disposition::Parse
    } else if header.fpos == 0
        && (header.type_u64 == '\n' as uint64_t || header.type_u64 > SHADA_LAST_ENTRY)
    {
        Disposition::VerifyOnly
    } else {
        Disposition::Skip
    }
}

/// A parse failure. The entry is freed and reported as missing.
pub(crate) struct Malformed;

/// Read the next entry into `entry`.
///
/// `flags` says which types the caller wants (see the `kSDRead*` values);
/// anything else is skipped. `max_kbyte`, if non-zero, skips entries longer
/// than that many kilobytes.
pub(crate) unsafe fn shada_read_next_item(
    sd_reader: *mut FileDescriptor,
    entry: *mut ShadaEntry,
    flags: c_uint,
    max_kbyte: size_t,
) -> ShaDaReadResult {
    loop {
        // Start from an empty entry, so that an early failure still
        // leaves one that is safe to free.
        unsafe { entry.write(ShadaEntry::MISSING) };
        if unsafe { file_eof(sd_reader) } {
            return kSDReadStatusFinished;
        }

        let header = match unsafe { read_header(sd_reader) } {
            Ok(header) => header,
            Err(status) => return status,
        };
        unsafe { (*entry).timestamp = header.timestamp };
        // Everything the parsers allocate belongs to the entry.
        unsafe { (*entry).can_free_entry = true };

        let disposition = disposition(&header, flags, max_kbyte);
        if let Disposition::Skip = disposition {
            match unsafe { sd_reader_skip(sd_reader, header.length) } {
                kSDReadStatusSuccess => continue,
                status => return status,
            }
        }

        let parse_pos = unsafe { (*sd_reader).bytes_read };
        let body = match unsafe { Body::read(sd_reader, header.length) } {
            Ok(body) => body,
            Err(status) => return status,
        };
        let mut cursor = body.cursor();

        if let Disposition::VerifyOnly = disposition {
            let status = cursor.skip();
            match unsafe { shada_check_status(parse_pos, status, cursor.left) } {
                kSDReadStatusSuccess => continue,
                status => return status,
            }
        }

        if header.type_u64 > SHADA_LAST_ENTRY {
            return unsafe { read_unknown(entry, &header, body, parse_pos, cursor) };
        }

        match unsafe { parse_known(entry, &header, &mut cursor) } {
            Ok(()) => return kSDReadStatusSuccess,
            Err(Malformed) => {
                // `parse_known` set the entry's kind before parsing into
                // it, so whatever it managed to build is freed here.
                unsafe { shada_free_shada_entry(entry) };
                unsafe { (*entry).data = ShadaEntryData::Missing };
                return kSDReadStatusMalformed;
            }
        }
    }
}

/// An entry of a type this Nvim does not know: kept whole, so that writing
/// the file back out does not lose it.
unsafe fn read_unknown(
    entry: *mut ShadaEntry,
    header: &Header,
    body: Body,
    parse_pos: uint64_t,
    mut cursor: Cursor,
) -> ShaDaReadResult {
    let item = unknown_item {
        type_0: header.type_u64,
        contents: core::ptr::null_mut(),
        size: header.length,
    };
    unsafe { (*entry).data = ShadaEntryData::Unknown(item) };
    // As above: a strange *first* entry has to be proved to be msgpack
    // before the file is believed to be ShaDa at all.
    if header.fpos == 0 {
        let status = cursor.skip();
        let checked = unsafe { shada_check_status(parse_pos, status, cursor.left) };
        if checked != kSDReadStatusSuccess {
            unsafe { (*entry).data = ShadaEntryData::Missing };
            return checked;
        }
    }
    unsafe { (*entry).data.unknown_mut().contents = body.into_owned() };
    kSDReadStatusSuccess
}
