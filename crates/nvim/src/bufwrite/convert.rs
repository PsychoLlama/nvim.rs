//! Turning buffer text into file bytes.
//!
//! The buffer is always UTF-8; the file is whatever `'fileencoding'` says.
//! The conversions Nvim knows itself — UCS-2, UCS-4, UTF-16 and Latin1, in
//! either endianness — go through [`ucs2bytes`] one character at a time;
//! everything else is handed to iconv. [`ByteWriter`] is the funnel both
//! sides come out of: text is staged into it, and each
//! [`flush`](ByteWriter::flush) converts what is staged, writes it, and
//! keeps the trailing partial character for next time.
//!
//! [`make_bom`] writes the byte-order mark, which is the same encoder
//! applied to U+FEFF.

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::Write as _;

use core::ffi::{CStr, c_uint};

use super::*;
use crate::fileio::FioFlags;
use crate::mbyte::ICONV_EINVAL;
use crate::mbyte::{char_at, promised_char_len};
use crate::memory::try_grow_zeroed;
use crate::os::iconv::Iconv;

/// The staging buffer's size, and the one it falls back to when that much
/// memory cannot be had, so that writing still works when out of memory.
const WRITEBUFSIZE: usize = 8192;
const SMALLBUFSIZE: usize = 256;

/// How much room an iconv conversion is given per staged byte.
const ICONV_MULT: usize = 8;

/// Encode one character for the file.
///
/// Returns the bytes, how many of them there are, and whether the character
/// could not be represented — in which case a replacement was written
/// anyway, because upstream keeps writing and reports the error at the end.
fn ucs2bytes(c: c_uint, flags: FioFlags) -> ([u8; 4], usize, bool) {
    let little = flags.has(FioFlags::ENDIAN_L);
    if flags.has(FioFlags::UCS4) {
        return (
            if little {
                c.to_le_bytes()
            } else {
                c.to_be_bytes()
            },
            4,
            false,
        );
    }

    let mut out = [0u8; 4];
    let mut put = |at: usize, word: u16| {
        out[at..at + 2].copy_from_slice(&if little {
            word.to_le_bytes()
        } else {
            word.to_be_bytes()
        });
    };

    if flags.has(FioFlags::UCS2 | FioFlags::UTF16) {
        let mut c = c;
        let mut at = 0;
        let mut error = false;
        if c >= 0x10000 {
            if flags.has(FioFlags::UTF16) {
                // Make two words, ten bits of the character in each. The
                // first is 0xd800-0xdbff, the second 0xdc00-0xdfff.
                c -= 0x10000;
                error = c >= 0x100000;
                put(0, (((c >> 10) & 0x3ff) + 0xd800) as u16);
                at = 2;
                c = (c & 0x3ff) + 0xdc00;
            } else {
                error = true;
            }
        }
        put(at, c as u16);
        return (out, at + 2, error);
    }

    // Latin1.
    if c >= 0x100 {
        out[0] = 0xbf;
        (out, 1, true)
    } else {
        out[0] = c as u8;
        (out, 1, false)
    }
}

/// Generate the byte-order mark for encoding `name` into `buf`.
///
/// Returns its length, zero when the encoding has no BOM.
pub(crate) fn make_bom(buf: &mut [u8], name: &[u8]) -> usize {
    let flags = get_fio_flags(name);
    // Can't put a BOM in a non-Unicode file.
    if flags == FioFlags::LATIN1 || flags.is_empty() {
        return 0;
    }
    if flags == FioFlags::UTF8 {
        buf[..3].copy_from_slice(b"\xef\xbb\xbf");
        return 3;
    }
    let (bytes, len, _) = ucs2bytes(0xfeff, flags);
    buf[..len].copy_from_slice(&bytes[..len]);
    len
}

/// The staging buffer between the buffer's lines and the file's bytes.
///
/// Nvim does its own buffering here because `fwrite()` is so slow. Bytes are
/// [pushed](Self::push) in and [flushed](Self::flush) whenever the buffer
/// fills up; the flush is where conversion happens, so a multi-byte character
/// split across two flushes is carried over rather than mangled.
pub(crate) struct ByteWriter {
    /// Where the bytes go; `None` while conversion is only being checked.
    pub file: Option<File>,
    /// The staging buffer, all of it in use as room.
    buf: Vec<u8>,
    /// How much of `buf` is staged.
    len: usize,
    /// What the conversion is.
    pub flags: FioFlags,
    /// Scratch for a conversion that cannot be done in place; empty until
    /// [`reserve_conv_buf`](Self::reserve_conv_buf).
    conv_buf: Vec<u8>,
    /// iconv, when it is doing the conversion.
    iconv: Option<Iconv>,
    /// The next iconv call is the first, and must emit the initial shift
    /// state sequence.
    first: bool,
    /// Some character could not be represented in the target encoding.
    pub conv_error: bool,
    /// The line the first such character was on, or zero if not known.
    pub conv_error_lnum: LineNr,
    /// The line the staged bytes start on, for `conv_error_lnum`.
    pub start_lnum: LineNr,
}

impl ByteWriter {
    /// A writer with an 8 KiB staging buffer, or a 256-byte one when that
    /// much memory cannot be had (which has been reported then).
    pub(crate) fn new() -> Self {
        let mut buf = Vec::new();
        if !try_grow_zeroed(&mut buf, WRITEBUFSIZE) {
            buf = vec![0; SMALLBUFSIZE];
        }
        ByteWriter {
            file: None,
            buf,
            len: 0,
            flags: FioFlags::NONE,
            conv_buf: Vec::new(),
            iconv: None,
            first: false,
            conv_error: false,
            conv_error_lnum: 0,
            start_lnum: 0,
        }
    }

    /// How much fits before a flush is due.
    pub(crate) fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// How much is staged and not yet written.
    pub(crate) fn staged(&self) -> usize {
        self.len
    }

    /// Reserve the conversion scratch buffer, `mult` bytes per staged byte.
    ///
    /// False when there was not enough memory, which aborts the write.
    pub(crate) fn reserve_conv_buf(&mut self, mult: usize) -> bool {
        try_grow_zeroed(&mut self.conv_buf, self.buf.len() * mult)
    }

    /// Reserve the scratch buffer an iconv conversion needs.
    pub(crate) fn reserve_iconv_buf(&mut self) -> bool {
        self.reserve_conv_buf(ICONV_MULT)
    }

    /// Is iconv doing the conversion?
    pub(crate) fn has_iconv(&self) -> bool {
        self.iconv.is_some()
    }

    /// Stage the byte-order mark for encoding `fenc`, and return its length.
    pub(crate) fn stage_bom(&mut self, fenc: &[u8]) -> usize {
        self.len = make_bom(&mut self.buf, fenc);
        self.len
    }

    /// Set up iconv to convert to `fenc`. False when iconv cannot do it.
    pub(crate) fn open_iconv(&mut self, fenc: &CStr) -> bool {
        self.iconv = crate::mbyte::open_iconv(fenc, c"utf-8");
        self.first = self.iconv.is_some();
        self.first
    }

    /// Drop anything staged, without writing it.
    pub(crate) fn clear(&mut self) {
        self.len = 0;
    }

    /// Stage one byte. True when the buffer is now full and must be flushed
    /// before anything else is staged.
    pub(crate) fn push(&mut self, byte: u8) -> bool {
        self.buf[self.len] = byte;
        self.len += 1;
        self.len == self.buf.len()
    }

    /// Stage as much of `bytes` as fits, translated for the file: a NL (a
    /// NUL in the buffer's text) becomes a NUL, and with `mac` a CR becomes
    /// a NL. Answers how many bytes were taken; the buffer is full when
    /// that is fewer than were offered.
    pub(crate) fn push_text(&mut self, bytes: &[u8], mac: bool) -> usize {
        let room = self.buf.len() - self.len;
        let n = bytes.len().min(room);
        let staged = &mut self.buf[self.len..self.len + n];
        staged.copy_from_slice(&bytes[..n]);
        for byte in staged.iter_mut() {
            if *byte == NL as u8 {
                *byte = 0; // replace newlines with NULs
            } else if mac && *byte == CAR as u8 {
                *byte = NL as u8; // Mac: replace CRs with NLs
            }
        }
        self.len += n;
        n
    }

    /// Whether the staging buffer is full.
    pub(crate) fn is_full(&self) -> bool {
        self.len == self.buf.len()
    }

    /// Write `bytes` to the file, when there is one. False on a write error.
    pub(crate) fn write_raw(&mut self, bytes: &[u8]) -> bool {
        match &mut self.file {
            Some(file) => file.write_all(bytes).is_ok(),
            None => true,
        }
    }

    /// Convert and write what is staged.
    ///
    /// Bytes of an incomplete character at the end are moved to the front of
    /// the buffer and stay staged. False on a conversion or write error.
    pub(crate) fn flush(&mut self) -> bool {
        let staged = self.len;
        // Skip conversion when writing the BOM.
        let converted = if self.flags.has(FioFlags::NOCONVERT) {
            Some((Out::Staged, staged, staged))
        } else {
            self.convert()
        };
        let Some((out, outlen, consumed)) = converted else {
            return false;
        };

        let remaining = staged - consumed;
        self.len = remaining;

        // Skip writing while only checking the conversion.
        if let Some(file) = &mut self.file {
            let bytes = match out {
                Out::Staged => &self.buf[..outlen],
                Out::Scratch => &self.conv_buf[..outlen],
            };
            if file.write_all(bytes).is_err() {
                return false;
            }
        }
        if remaining > 0 {
            self.buf.copy_within(consumed..staged, 0);
        }
        true
    }

    /// Convert the staged bytes.
    ///
    /// Returns where the converted bytes are, how many there are, and how
    /// many staged bytes went into them.
    fn convert(&mut self) -> Option<(Out, usize, usize)> {
        let flags = self.flags;
        let staged = self.len;
        let mut out = Out::Staged;
        let mut outlen = staged;
        let mut consumed = staged;

        if flags.has(FioFlags::UCS4 | FioFlags::UTF16 | FioFlags::UCS2 | FioFlags::LATIN1) {
            // Convert the UTF-8 in the buffer to UCS-2, UCS-4, UTF-16 or
            // Latin1. Latin1 can only get shorter, so it translates in
            // place; the rest go to the conversion buffer.
            let latin1 = flags.has(FioFlags::LATIN1);
            let mut at = 0;
            let mut wlen = 0;
            while wlen < staged {
                let rest = &self.buf[wlen..staged];
                let n = promised_char_len(rest);
                if n > rest.len() {
                    // An incomplete byte sequence at the end. It cannot
                    // be converted without the rest of it, so keep the
                    // bytes for the next call.
                    break;
                }
                let c = if n > 1 {
                    char_at(rest) as c_uint
                } else {
                    c_uint::from(rest[0])
                };
                let (bytes, need, bad) = ucs2bytes(c, flags);
                if latin1 {
                    // `need` is one, and `at` never passes `wlen`.
                    self.buf[at] = bytes[0];
                } else {
                    // Check that there is enough space.
                    let dest = self.conv_buf.get_mut(at..at + need)?;
                    dest.copy_from_slice(&bytes[..need]);
                }
                at += need;
                if bad && !self.conv_error {
                    self.conv_error = true;
                    self.conv_error_lnum = self.start_lnum;
                }
                if c == NL as c_uint {
                    self.start_lnum += 1;
                }
                wlen += n;
            }
            consumed = wlen;
            outlen = at;
            if !latin1 {
                out = Out::Scratch;
            }
        }

        if self.iconv.is_some() {
            return self.convert_with_iconv(out, outlen);
        }
        Some((out, outlen, consumed))
    }

    /// Hand the `inlen` bytes `input` names to iconv, into the scratch
    /// buffer.
    ///
    /// Only reached with iconv set up and nothing converted before it, so
    /// the input is always the staged bytes.
    fn convert_with_iconv(&mut self, input: Out, inlen: usize) -> Option<(Out, usize, usize)> {
        debug_assert!(matches!(input, Out::Staged));
        let iconv = self.iconv.as_ref()?;
        let total = self.conv_buf.len();
        let mut to: &mut [u8] = &mut self.conv_buf;
        if self.first {
            // Output the initial shift state sequence.
            iconv.shift_initial(&mut to);
            self.first = false;
        }
        let mut from = &self.buf[..inlen];
        if let Err(errno) = iconv.step(&mut from, &mut to)
            && errno != ICONV_EINVAL
        {
            self.conv_error = true;
            return None;
        }
        let produced = total - to.len();
        Some((Out::Scratch, produced, inlen - from.len()))
    }
}

/// Which buffer a conversion left its bytes in.
#[derive(Clone, Copy)]
enum Out {
    /// The staging buffer itself: no conversion, or Latin1 in place.
    Staged,
    /// The conversion scratch buffer.
    Scratch,
}
