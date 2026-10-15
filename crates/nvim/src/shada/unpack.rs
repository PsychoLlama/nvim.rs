//! Reading entries out of a ShaDa file.
//!
//! Every entry on the wire is four things in a row: a type, a timestamp, a
//! byte length, and that many bytes of msgpack. The three integers are
//! unpacked one byte at a time by [`read_uint64`] so that nothing has to be
//! buffered or seeked over; the payload is then handed to `parse`, which
//! walks it with a [`Cursor`].
//!
//! The length is what makes an unknown entry skippable, and it is why an
//! entry this Nvim does not understand can still be copied through to the
//! file it writes: see `merge`.

#![forbid(unsafe_code)]

use crate::api::private::keyset::{KeySet, Slot};
use crate::eval::decode::unpack_typval;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::mpack::token::{self, Kind, Read, Tok};
use crate::os::cshim::gettext;
use crate::os::fileio::ReadFile;
use crate::os::fs::os_strerror;
use crate::types::{Integer, String_0, StringArray, TypVal};
use crate::{semsg, tr_c};
use core::ffi::{CStr, c_int};
use std::borrow::Cow;
use std::ffi::CString;

use super::parse::parse_known;
use super::{ReadKinds, ReadStatus, ShadaEntry, ShadaEntryData, ShadaExtra, ShadaUnknownItem};
use super::{WANT_UNKNOWN, kind};

/// Report `E576`/`E886` about the file, naming a byte offset or a length.
fn shada_read_error(fmt: &'static CStr, at: u64) {
    emsg_text(tr_c!(fmt, at));
}

/// As [`shada_read_error`], for the messages whose `%s` is a libuv error or
/// a fixed reason.
fn shada_read_error_why(fmt: &'static CStr, why: &CStr) {
    let why = msg_cstr(why);
    emsg_text(tr_c!(fmt, why));
}

/// What `mpack_parse` made of a value it was asked to step over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Parsed {
    /// The whole value was there.
    Ok,
    /// The bytes ran out part way through it.
    Eof,
    /// A byte that is not msgpack, or a value nested deeper than the parser
    /// goes.
    Error,
}

/// How deep `mpack_parse` nests before it gives up: every token takes a
/// frame while it is open, a string's chunk included.
const SKIP_FRAMES: usize = 32;

/// One open frame of [`Cursor::skip`]'s walk: a token, and how many of its
/// children (items, pairs or bytes) have been seen.
struct Frame {
    kind: Kind,
    len: u64,
    seen: u64,
    /// For a map: whether the next child closes a pair.
    key_seen: bool,
}

/// A msgpack cursor over the bytes of one entry.
pub(crate) struct Cursor<'a> {
    rest: &'a [u8],
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Cursor<'a> {
        Cursor { rest: bytes }
    }

    /// What has not been read yet.
    pub(crate) fn rest(&self) -> &'a [u8] {
        self.rest
    }

    /// How many bytes of the entry are still unread.
    pub(crate) fn left(&self) -> usize {
        self.rest.len()
    }

    /// One whole token, consumed. A type byte whose payload is short, or a
    /// byte that is not msgpack, is consumed and answers `None`, as
    /// `mpack_rtoken` steps over it; at the end nothing is consumed.
    fn token(&mut self) -> Option<Tok> {
        match token::decode_token(self.rest) {
            Read::Done { tok, used } => {
                self.rest = &self.rest[used..];
                Some(tok)
            }
            Read::Partial { .. } | Read::Invalid => {
                self.rest = &self.rest[1..];
                None
            }
            Read::Empty => None,
        }
    }

    /// The element count of an array, or `None` if the next token is not one.
    pub(crate) fn array(&mut self) -> Option<usize> {
        let tok = self.token()?;
        (tok.kind == Some(Kind::Array)).then_some(tok.len as usize)
    }

    /// A string or binary token's bytes, or `None` (consuming nothing) if
    /// the next token is neither, or its bytes are not all there.
    pub(crate) fn string(&mut self) -> Option<&'a [u8]> {
        let Read::Done { tok, used } = token::decode_token(self.rest) else {
            return None;
        };
        if tok.kind != Some(Kind::Str) && tok.kind != Some(Kind::Bin) {
            return None;
        }
        let body = &self.rest[used..];
        let len = tok.len as usize;
        if body.len() < len {
            return None;
        }
        self.rest = &body[len..];
        Some(&body[..len])
    }

    /// An integer token, or `None` if the next token is not one.
    pub(crate) fn integer(&mut self) -> Option<Integer> {
        let tok = self.token()?;
        match tok.kind {
            Some(Kind::Uint) => Some(token::unpack_uint(&tok).cast_signed()),
            Some(Kind::Sint) => Some(token::unpack_sint(&tok)),
            _ => None,
        }
    }

    /// Step over one whole value, however deep, as `mpack_parse` with a
    /// parser that builds nothing does. On [`Parsed::Error`] the cursor is
    /// left before the token that failed; on [`Parsed::Eof`] it has
    /// consumed everything.
    pub(crate) fn skip(&mut self) -> Parsed {
        if self.rest.is_empty() {
            return Parsed::Eof;
        }
        let mut stack: Vec<Frame> = Vec::new();
        // The bytes of a string, binary or extension still to come, read
        // as chunks of whatever is there.
        let mut passthrough = 0usize;
        loop {
            if self.rest.is_empty() {
                return Parsed::Eof;
            }
            let before = self.rest;
            let (kind, len) = if passthrough > 0 {
                let chunk = passthrough.min(self.rest.len());
                self.rest = &self.rest[chunk..];
                passthrough -= chunk;
                (Kind::Chunk, chunk as u64)
            } else {
                match token::decode_token(self.rest) {
                    Read::Done { tok, used } => {
                        self.rest = &self.rest[used..];
                        let kind = tok.kind.expect("a decoded token has a kind");
                        if kind > Kind::Map {
                            passthrough = tok.len as usize;
                        }
                        (kind, u64::from(tok.len))
                    }
                    // `mpack_read` keeps the partial token and asks for more,
                    // which there is not.
                    Read::Partial { .. } => {
                        self.rest = &[];
                        return Parsed::Eof;
                    }
                    Read::Empty => unreachable!("the cursor is not empty"),
                    Read::Invalid => {
                        self.rest = before;
                        return Parsed::Error;
                    }
                }
            };
            if stack.len() == SKIP_FRAMES {
                self.rest = before;
                return Parsed::Error;
            }
            stack.push(Frame {
                kind,
                len,
                seen: 0,
                key_seen: false,
            });
            // Close every frame that has all its children.
            loop {
                let top = stack.last().expect("a frame was just pushed");
                if top.kind > Kind::Chunk && top.seen < top.len {
                    break;
                }
                let done = stack.pop().expect("the frame just looked at");
                let Some(parent) = stack.last_mut() else {
                    return Parsed::Ok;
                };
                if done.kind == Kind::Chunk {
                    parent.seen += done.len;
                } else if parent.kind == Kind::Map {
                    if parent.key_seen {
                        parent.seen += 1;
                    }
                    parent.key_seen = !parent.key_seen;
                } else {
                    parent.seen += 1;
                }
            }
        }
    }

    /// One value as a Vimscript value. Answers an `MPACK_*` status.
    pub(crate) fn typval(&mut self, into: &mut TypVal) -> c_int {
        let mut data = self.rest;
        let status = unpack_typval(&mut data, into);
        self.rest = data;
        status
    }

    /// A map, into the keyset `into`. Keys the keyset does not name are
    /// kept in `extra`, to be written back out unchanged.
    ///
    /// The error is the reason the map was refused, or `None` when a value
    /// under an unknown key was not msgpack at all -- which upstream reports
    /// as a null `%s`.
    pub(crate) fn keydict(
        &mut self,
        into: &mut dyn KeySet,
        extra: &mut Option<ShadaExtra>,
    ) -> Result<(), Option<CString>> {
        let Some(tok) = self.token().filter(|tok| tok.kind == Some(Kind::Map)) else {
            return Err(Some(c"is not a dict".to_owned()));
        };

        // The fields filled so far, for the duplicate-key check: a keyset
        // the caller pre-filled (a search pattern arrives holding its
        // defaults) is already `Some` before the map is read at all.
        let mut filled: Vec<usize> = Vec::new();
        for _ in 0..tok.len {
            let item_start = self.rest;
            let Some(key) = self.string() else {
                return Err(Some(complaint(b"has key value which is not a string", b"")));
            };
            if key.is_empty() {
                return Err(Some(complaint(b"has empty key", key)));
            }
            let Some(field) = into.find(key) else {
                if self.skip() != Parsed::Ok {
                    return Err(None);
                }
                let used = item_start.len() - self.rest.len();
                ShadaExtra::push(extra, &item_start[..used]);
                continue;
            };
            if filled.contains(&field) {
                return Err(Some(c"duplicate key".to_owned()));
            }
            filled.push(field);
            if let Err(message) = self.field(into.slot(field)) {
                return Err(Some(complaint(message, key)));
            }
        }
        Ok(())
    }

    /// One keyset field's value into `slot`. The error is the complaint,
    /// whose `%.*s` the caller fills with the key.
    fn field(&mut self, slot: Slot<'_>) -> Result<(), &'static [u8]> {
        match slot {
            Slot::Boolean(slot) => {
                // Read straight off the wire: both boolean encodings differ
                // only in their low bit.
                let Some(&byte) = self.rest.first().filter(|&&b| b & 0xfe == 0xc2) else {
                    return Err(b"has %.*s key value which is not a boolean");
                };
                *slot = Some(byte & 1 != 0);
                self.rest = &self.rest[1..];
            }
            Slot::Integer(slot) => {
                let Some(number) = self.integer() else {
                    return Err(b"has %.*s key value which is not an integer");
                };
                *slot = Some(number);
            }
            Slot::String(slot) => {
                let Some(val) = self.string() else {
                    return Err(b"has %.*s key value which is not a binary");
                };
                *slot = Some(String_0::from_bytes(val));
            }
            Slot::StringArray(slot) => {
                let Some(len) = self.array() else {
                    return Err(b"has %.*s key with non-array value");
                };
                // Built beside the slot and stored whole, a partial one too:
                // the field is unset when this reaches it.
                let mut array = StringArray::EMPTY;
                let mut read = Ok(());
                for _ in 0..len {
                    let Some(item) = self.string() else {
                        read = Err(&b"has %.*s array with non-binary value"[..]);
                        break;
                    };
                    array.push(String_0::from_bytes(item));
                }
                *slot = Some(array);
                return read;
            }
            _ => unreachable!("ShaDa's keysets hold booleans, integers and strings"),
        }
        Ok(())
    }
}

/// One of [`Cursor::keydict`]'s complaints, with its `%.*s` (if it has one)
/// the key -- as `printf` puts it, up to the key's first NUL.
fn complaint(message: &[u8], key: &[u8]) -> CString {
    let key = key.split(|&b| b == 0).next().unwrap_or(&[]);
    let mut text = Vec::with_capacity(message.len() + key.len());
    match message.windows(4).position(|w| w == b"%.*s") {
        Some(at) => {
            text.extend_from_slice(&message[..at]);
            text.extend_from_slice(key);
            text.extend_from_slice(&message[at + 4..]);
        }
        None => text.extend_from_slice(message),
    }
    CString::new(text).expect("a complaint with no NUL")
}

/// Where the entries come from: a file, or a ShaDa string the context stack
/// holds in memory.
pub(crate) enum Source<'a> {
    File(ReadFile),
    Bytes { data: &'a [u8], read: usize },
}

impl<'a> Source<'a> {
    /// Whether everything has been read.
    fn at_eof(&self) -> bool {
        match self {
            Source::File(file) => file.at_eof(),
            Source::Bytes { data, read } => *read == data.len(),
        }
    }

    /// How many bytes reading and skipping have consumed so far.
    fn bytes_read(&self) -> u64 {
        match self {
            Source::File(file) => file.bytes_read(),
            Source::Bytes { read, .. } => *read as u64,
        }
    }

    /// Read into `buf`: how many bytes came (fewer only at the end), or a
    /// negative libuv error code.
    fn read(&mut self, buf: &mut [u8]) -> isize {
        match self {
            Source::File(file) => file.read(buf),
            Source::Bytes { data, read } => {
                let n = buf.len().min(data.len() - *read);
                buf[..n].copy_from_slice(&data[*read..*read + n]);
                *read += n;
                n as isize
            }
        }
    }

    /// Step over `size` bytes: how many went, or a negative libuv error.
    fn skip(&mut self, size: usize) -> isize {
        match self {
            Source::File(file) => file.skip(size),
            Source::Bytes { data, read } => {
                let n = size.min(data.len() - *read);
                *read += n;
                n as isize
            }
        }
    }

    /// How many bytes the source holds without reading.
    fn held(&self) -> usize {
        match self {
            Source::File(file) => file.buffered(),
            Source::Bytes { data, read } => data.len() - *read,
        }
    }

    /// The bytes of one entry: borrowed when the source holds them all, read
    /// into a buffer of their own when it does not.
    fn body(&mut self, len: usize) -> Result<Cow<'_, [u8]>, ReadStatus> {
        if self.held() >= len {
            let held = match self {
                Source::File(file) => file.try_read_buffered(len),
                Source::Bytes { data, read } => {
                    let held = &data[*read..*read + len];
                    *read += len;
                    Some(held)
                }
            };
            return Ok(Cow::Borrowed(
                held.expect("the source holds the whole entry"),
            ));
        }
        let mut owned = vec![0u8; len];
        self.read_exact(&mut owned)?;
        Ok(Cow::Owned(owned))
    }

    /// Fill `buf`, complaining if the file is shorter.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ReadStatus> {
        let read_bytes = self.read(buf);
        if read_bytes < 0 {
            shada_read_error_why(
                c"E886: System error while reading ShaDa file: %s",
                os_strerror(read_bytes as c_int),
            );
            return Err(ReadStatus::ReadError);
        }
        if read_bytes as usize != buf.len() {
            shada_read_error(c"E576: Error while reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier", buf.len() as u64);
            return Err(ReadStatus::NotShada);
        }
        Ok(())
    }

    /// Step over `size` bytes, complaining if the file is shorter.
    fn skip_exact(&mut self, size: usize) -> Result<(), ReadStatus> {
        let skip_bytes = self.skip(size);
        if skip_bytes < 0 {
            shada_read_error_why(
                c"E886: System error while skipping in ShaDa file: %s",
                os_strerror(skip_bytes as c_int),
            );
            return Err(ReadStatus::ReadError);
        }
        if skip_bytes as usize != size {
            assert!((skip_bytes as usize) < size, "skipped past end of file");
            if self.at_eof() {
                shada_read_error(c"E576: Reading ShaDa file: last entry specified that it occupies %lu bytes, but file ended earlier", size as u64);
            } else {
                shada_read_error_why(
                    c"E886: System error while skipping in ShaDa file: %s",
                    gettext(c"too few bytes read"),
                );
            }
            return Err(ReadStatus::NotShada);
        }
        Ok(())
    }

    /// Read one unsigned integer, one byte at a time.
    ///
    /// Unlike msgpack's own reader this consumes *exactly* the bytes the
    /// value needs, which is what lets the entry header be read without a
    /// buffer or a seek. One byte is always consumed, even when it turns out
    /// to be wrong.
    ///
    /// `Err(Finished)` means end of file, and is only produced when
    /// `allow_eof` -- i.e. for the first of the three header integers.
    fn read_uint64(&mut self, allow_eof: bool) -> Result<u64, ReadStatus> {
        let fpos = self.bytes_read();
        let mut first = [0u8; 1];
        let read_bytes = self.read(&mut first);
        if read_bytes < 0 {
            shada_read_error_why(
                c"E886: System error while reading integer from ShaDa file: %s",
                os_strerror(read_bytes as c_int),
            );
            return Err(ReadStatus::ReadError);
        }
        if read_bytes == 0 {
            if allow_eof && self.at_eof() {
                return Err(ReadStatus::Finished);
            }
            shada_read_error(c"E576: Error while reading ShaDa file: expected positive integer at position %lu, but got nothing", fpos);
            return Err(ReadStatus::NotShada);
        }

        // A positive fixint is its own value; the four uint formats say how
        // many big-endian bytes follow.
        let first = first[0];
        if first & 0x80 == 0 {
            return Ok(u64::from(first));
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
                return Err(ReadStatus::NotShada);
            }
        };
        let mut bytes = [0u8; 8];
        self.read_exact(&mut bytes[8 - length..])?;
        Ok(u64::from_be_bytes(bytes))
    }

    /// The type, timestamp and length that head every entry.
    fn read_header(&mut self) -> Result<Header, ReadStatus> {
        let fpos = self.bytes_read();
        let kind = self.read_uint64(true)?;
        let timestamp = self.read_uint64(false)?;
        let length = self.read_uint64(false)?;

        if length > isize::MAX as u64 {
            shada_read_error(c"E576: Error while reading ShaDa file: there is an item at position %lu that is stated to be too long", fpos);
            return Err(ReadStatus::NotShada);
        }
        if kind == 0 {
            // The internal "missing" kind would otherwise be skipped
            // silently, because `1 << 0` is never in the flags.
            shada_read_error(c"E576: Error while reading ShaDa file: there is an item at position %lu that must not be there: Missing items are for internal uses only", fpos);
            return Err(ReadStatus::NotShada);
        }
        Ok(Header {
            kind,
            timestamp,
            length: length as usize,
            fpos,
        })
    }

    /// Close a file source, saying so if that fails. A string has nothing
    /// to close.
    pub(crate) fn close(self) {
        if let Source::File(file) = self
            && let Err(error) = file.close()
        {
            shada_read_error_why(
                c"E886: System error while closing ShaDa file: %s",
                os_strerror(error),
            );
        }
    }

    /// Read the next entry.
    ///
    /// `kinds` says which kinds the caller wants (see [`super::want`]);
    /// anything else is skipped. `max_kbyte`, if non-zero, skips entries
    /// longer than that many kilobytes.
    pub(crate) fn next_entry(
        &mut self,
        kinds: ReadKinds,
        max_kbyte: usize,
    ) -> Result<ShadaEntry, ReadStatus> {
        loop {
            if self.at_eof() {
                return Err(ReadStatus::Finished);
            }
            let header = self.read_header()?;

            let disposition = disposition(&header, kinds, max_kbyte);
            if let Disposition::Skip = disposition {
                self.skip_exact(header.length)?;
                continue;
            }

            let parse_pos = self.bytes_read();
            let body = self.body(header.length)?;
            let mut cursor = Cursor::new(&body);

            if let Disposition::VerifyOnly = disposition {
                let status = cursor.skip();
                check_status(parse_pos, status, cursor.left())?;
                continue;
            }

            if header.kind > kind::LAST {
                // A strange *first* entry has to be proved to be msgpack
                // before the file is believed to be ShaDa at all.
                if header.fpos == 0 {
                    let status = cursor.skip();
                    check_status(parse_pos, status, cursor.left())?;
                }
                let item = ShadaUnknownItem {
                    kind: header.kind,
                    contents: body.into_owned(),
                };
                return Ok(ShadaEntry::new(
                    header.timestamp,
                    ShadaEntryData::Unknown(item),
                ));
            }

            return parse_known(&header, &mut cursor).map_err(|_| ReadStatus::Malformed);
        }
    }
}

/// What the msgpack parser made of an entry's payload.
pub(crate) fn check_status(
    initial_fpos: u64,
    status: Parsed,
    remaining: usize,
) -> Result<(), ReadStatus> {
    let at = initial_fpos;
    match status {
        Parsed::Ok if remaining != 0 => {
            shada_read_error(
                c"E576: Failed to parse ShaDa file: extra bytes in msgpack string at position %lu",
                at,
            );
            Err(ReadStatus::NotShada)
        }
        Parsed::Ok => Ok(()),
        Parsed::Eof => {
            semsg!("E576: Failed to parse ShaDa file: incomplete msgpack string at position {at}");
            Err(ReadStatus::NotShada)
        }
        Parsed::Error => {
            semsg!(
                "E576: Failed to parse ShaDa file due to a msgpack parser error at position {at}"
            );
            Err(ReadStatus::NotShada)
        }
    }
}

/// The type, timestamp and length that head every entry.
pub(crate) struct Header {
    /// The raw kind: not yet known to be one this Nvim reads.
    pub(crate) kind: u64,
    pub(crate) timestamp: u64,
    length: usize,
    /// Where in the file the entry started, for the error messages.
    pub(crate) fpos: u64,
}

/// Whether the caller asked for this entry type at this size.
fn wanted(header: &Header, kinds: ReadKinds, max_kbyte: usize) -> bool {
    let by_type = if header.kind > kind::LAST {
        kinds & WANT_UNKNOWN != 0
    } else {
        (1u32 << header.kind) & kinds != 0
    };
    by_type && (max_kbyte == 0 || header.length <= max_kbyte.wrapping_mul(1024))
}

/// What to do with an entry once its header has been read.
enum Disposition {
    /// Parse it into an entry.
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

fn disposition(header: &Header, kinds: ReadKinds, max_kbyte: usize) -> Disposition {
    if wanted(header, kinds, max_kbyte) {
        Disposition::Parse
    } else if header.fpos == 0 && (header.kind == u64::from(b'\n') || header.kind > kind::LAST) {
        Disposition::VerifyOnly
    } else {
        Disposition::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip(bytes: &[u8]) -> (Parsed, usize) {
        let mut cursor = Cursor::new(bytes);
        let status = cursor.skip();
        (status, cursor.left())
    }

    #[test]
    fn skip_steps_over_one_whole_value() {
        assert_eq!(skip(&[0x01, 0x02]), (Parsed::Ok, 1));
        assert_eq!(skip(&[0x92, 0x01, 0xc4, 0x01, b'u', 0x07]), (Parsed::Ok, 1));
        assert_eq!(skip(&[0x81, 0xa1, b'k', 0x90]), (Parsed::Ok, 0));
        assert_eq!(skip(&[0xc4, 0x00]), (Parsed::Ok, 0));
    }

    #[test]
    fn skip_reports_short_and_foreign_input() {
        assert_eq!(skip(&[]), (Parsed::Eof, 0));
        assert_eq!(skip(&[0x92, 0x01]), (Parsed::Eof, 0));
        assert_eq!(skip(&[0xc4, 0x05, b'a']), (Parsed::Eof, 0));
        assert_eq!(skip(&[0xcd, 0x01]), (Parsed::Eof, 0));
        // The cursor stops before the byte that is not msgpack.
        assert_eq!(skip(&[0x92, 0x01, 0xc1]), (Parsed::Error, 1));
    }

    #[test]
    fn skip_gives_up_past_thirty_two_frames() {
        let mut deep = vec![0x91; 31];
        deep.push(0x01);
        assert_eq!(skip(&deep), (Parsed::Ok, 0));
        let mut deeper = vec![0x91; 32];
        deeper.push(0x01);
        assert_eq!(skip(&deeper), (Parsed::Error, 1));
        // A string's chunk takes a frame of its own.
        let mut chunked = vec![0x91; 31];
        chunked.extend([0xa1, b'x']);
        assert_eq!(skip(&chunked), (Parsed::Error, 1));
    }

    #[test]
    fn a_string_whose_bytes_are_short_is_not_one() {
        let mut cursor = Cursor::new(&[0xd9, 0x05, b'a', b'b', b'c']);
        assert_eq!(cursor.string(), None);
        assert_eq!(cursor.left(), 5);
        let mut cursor = Cursor::new(&[0xa3, b'a', b'b', b'c', 0x01]);
        assert_eq!(cursor.string(), Some(&b"abc"[..]));
        assert_eq!(cursor.integer(), Some(1));
    }

    #[test]
    fn complaints_name_the_key_up_to_its_first_nul() {
        assert_eq!(
            complaint(b"has %.*s key value which is not a boolean", b"sm\0x"),
            c"has sm key value which is not a boolean"
        );
        assert_eq!(complaint(b"has empty key", b""), c"has empty key");
    }
}
