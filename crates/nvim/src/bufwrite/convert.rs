//! Turning buffer text into file bytes.
//!
//! The buffer is always UTF-8; the file is whatever `'fileencoding'` says.
//! The conversions Nvim knows itself — UCS-2, UCS-4, UTF-16 and Latin1, in
//! either endianness — go through [`ucs2bytes`] one character at a time;
//! everything else is handed to iconv. [`ByteWriter`] is the funnel both
//! sides come out of: text is staged into it a byte at a time, and each
//! [`flush`](ByteWriter::flush) converts what is staged, writes it, and
//! keeps the trailing partial character for next time.
//!
//! [`make_bom`] writes the byte-order mark, which is the same encoder
//! applied to U+FEFF.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

use super::*;
use crate::mbyte::ICONV_EINVAL;

/// The iconv descriptor value that means "no iconv".
fn no_iconv() -> iconv_t {
    core::ptr::with_exposed_provenance_mut::<core::ffi::c_void>(-1i32 as usize)
}

/// Encode one character for the file.
///
/// Returns the bytes, how many of them there are, and whether the character
/// could not be represented — in which case a replacement was written
/// anyway, because upstream keeps writing and reports the error at the end.
fn ucs2bytes(c: c_uint, flags: c_int) -> ([u8; 4], usize, bool) {
    let little = flags & FIO_ENDIAN_L != 0;
    if flags & FIO_UCS4 != 0 {
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

    if flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        let mut c = c;
        let mut at = 0;
        let mut error = false;
        if c >= 0x10000 {
            if flags & FIO_UTF16 != 0 {
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
pub(crate) unsafe fn make_bom(buf: &mut [c_char], name: *mut c_char) -> usize {
    unsafe {
        let flags = get_fio_flags(name);
        // Can't put a BOM in a non-Unicode file.
        if flags == FIO_LATIN1 || flags == 0 {
            return 0;
        }
        if flags == FIO_UTF8 {
            buf[..3].copy_from_slice(&[0xefu8 as c_char, 0xbbu8 as c_char, 0xbfu8 as c_char]);
            return 3;
        }
        let (bytes, len, _) = ucs2bytes(0xfeff, flags);
        for (at, byte) in bytes[..len].iter().enumerate() {
            buf[at] = *byte as c_char;
        }
        len
    }
}

/// The staging buffer between the buffer's lines and the file's bytes.
///
/// Nvim does its own buffering here because `fwrite()` is so slow. Bytes are
/// [pushed](Self::push) in one at a time and [flushed](Self::flush) whenever
/// the buffer fills up; the flush is where conversion happens, so a
/// multi-byte character split across two flushes is carried over rather than
/// mangled.
pub(crate) struct ByteWriter<'a> {
    /// Where the bytes go, or -1 while conversion is only being checked.
    pub fd: c_int,
    /// The staging buffer.
    buf: &'a mut [c_char],
    /// How much of `buf` is staged.
    len: usize,
    /// `FIO_*` flags describing the conversion.
    pub flags: c_int,
    /// Scratch for a conversion that cannot be done in place. Allocated with
    /// `verbose_try_malloc`, so it can be null; owned.
    conv_buf: *mut c_char,
    conv_buflen: usize,
    /// The iconv descriptor when iconv is doing the conversion; owned.
    iconv: iconv_t,
    /// The next iconv call is the first, and must emit the initial shift
    /// state sequence.
    first: bool,
    /// Some character could not be represented in the target encoding.
    pub conv_error: bool,
    /// The line the first such character was on, or zero if not known.
    pub conv_error_lnum: linenr_T,
    /// The line the staged bytes start on, for `conv_error_lnum`.
    pub start_lnum: linenr_T,
}

impl Drop for ByteWriter<'_> {
    fn drop(&mut self) {
        unsafe {
            xfree(self.conv_buf.cast());
            if self.iconv != no_iconv() {
                iconv_close(self.iconv);
            }
        }
    }
}

impl<'a> ByteWriter<'a> {
    pub(crate) fn new(buf: &'a mut [c_char]) -> Self {
        ByteWriter {
            fd: 0,
            buf,
            len: 0,
            flags: 0,
            conv_buf: core::ptr::null_mut(),
            conv_buflen: 0,
            iconv: no_iconv(),
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
    pub(crate) unsafe fn reserve_conv_buf(&mut self, mult: usize) -> bool {
        unsafe {
            self.conv_buflen = self.buf.len() * mult;
            self.conv_buf = verbose_try_malloc(self.conv_buflen).cast();
            !self.conv_buf.is_null()
        }
    }

    /// Is iconv doing the conversion?
    pub(crate) fn has_iconv(&self) -> bool {
        self.iconv != no_iconv()
    }

    /// Stage the byte-order mark for encoding `fenc`, and return its length.
    pub(crate) unsafe fn stage_bom(&mut self, fenc: *mut c_char) -> usize {
        self.len = unsafe { make_bom(self.buf, fenc) };
        self.len
    }

    /// Set up iconv to convert to `fenc`. False when iconv cannot do it.
    pub(crate) unsafe fn open_iconv(&mut self, fenc: *mut c_char) -> bool {
        unsafe {
            self.iconv = my_iconv_open(fenc, c"utf-8".as_ptr().cast_mut());
            if self.iconv == no_iconv() {
                return false;
            }
            self.first = true;
            true
        }
    }

    /// Drop anything staged, without writing it.
    pub(crate) fn clear(&mut self) {
        self.len = 0;
    }

    /// Stage one byte. True when the buffer is now full and must be flushed
    /// before anything else is staged.
    pub(crate) fn push(&mut self, byte: c_char) -> bool {
        self.buf[self.len] = byte;
        self.len += 1;
        self.len == self.buf.len()
    }

    /// Convert and write what is staged.
    ///
    /// Bytes of an incomplete character at the end are moved to the front of
    /// the buffer and stay staged. False on a conversion or write error.
    pub(crate) unsafe fn flush(&mut self) -> bool {
        unsafe {
            let staged = self.len;
            // Skip conversion when writing the BOM.
            let converted = if self.flags & FIO_NOCONVERT != 0 {
                Some((self.buf.as_ptr(), staged, staged))
            } else {
                self.convert()
            };
            let Some((out, outlen, consumed)) = converted else {
                return false;
            };

            let remaining = staged - consumed;
            self.len = remaining;

            // Skip writing while only checking the conversion.
            if self.fd >= 0
                && (write_eintr(self.fd, out.cast_mut().cast(), outlen as size_t) as c_int)
                    < outlen as c_int
            {
                return false;
            }
            if remaining > 0 {
                core::ptr::copy(
                    self.buf.as_ptr().add(consumed),
                    self.buf.as_mut_ptr(),
                    remaining,
                );
            }
            true
        }
    }

    /// Convert the staged bytes.
    ///
    /// Returns where the converted bytes are, how many there are, and how
    /// many staged bytes went into them.
    unsafe fn convert(&mut self) -> Option<(*const c_char, usize, usize)> {
        unsafe {
            let flags = self.flags;
            let staged = self.len;
            let mut out = self.buf.as_ptr();
            let mut outlen = staged;
            let mut consumed = staged;

            if flags & (FIO_UCS4 | FIO_UTF16 | FIO_UCS2 | FIO_LATIN1) != 0 {
                // Convert the UTF-8 in the buffer to UCS-2, UCS-4, UTF-16 or
                // Latin1. Latin1 can only get shorter, so it translates in
                // place; the rest go to the conversion buffer.
                let latin1 = flags & FIO_LATIN1 != 0;
                let dest: *mut u8 = if latin1 {
                    self.buf.as_mut_ptr().cast()
                } else {
                    self.conv_buf.cast()
                };
                let mut at = 0;
                let mut wlen = 0;
                while wlen < staged {
                    let n = utf_ptr2len_len(self.buf.as_ptr().add(wlen), (staged - wlen) as c_int)
                        as usize;
                    if n > staged - wlen {
                        // An incomplete byte sequence at the end. It cannot
                        // be converted without the rest of it, so keep the
                        // bytes for the next call.
                        break;
                    }
                    let c = if n > 1 {
                        utf_ptr2char(self.buf.as_ptr().add(wlen)) as c_uint
                    } else {
                        self.buf[wlen] as u8 as c_uint
                    };
                    let (bytes, need, bad) = ucs2bytes(c, flags);
                    // Check that there is enough space.
                    if !latin1 && at + need > self.conv_buflen {
                        return None;
                    }
                    core::ptr::copy_nonoverlapping(bytes.as_ptr(), dest.add(at), need);
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
                    out = self.conv_buf;
                }
            }

            if self.iconv != no_iconv() {
                return self.convert_with_iconv(out, outlen);
            }
            Some((out, outlen, consumed))
        }
    }

    /// Hand `inlen` bytes at `input` to iconv.
    unsafe fn convert_with_iconv(
        &mut self,
        input: *const c_char,
        inlen: usize,
    ) -> Option<(*const c_char, usize, usize)> {
        unsafe {
            let mut from = input;
            let mut fromlen = inlen as size_t;
            let mut tolen = self.conv_buflen as size_t;
            let mut to = self.conv_buf;

            if self.first {
                let save_len = tolen;
                // Output the initial shift state sequence.
                iconv(
                    self.iconv,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &raw mut to,
                    &raw mut tolen,
                );
                // There is a bug in iconv() on Linux (which appears to be
                // wide-spread) which sets "to" to NULL and messes up "tolen".
                if to.is_null() {
                    to = self.conv_buf;
                    tolen = save_len;
                }
                self.first = false;
            }

            if iconv(
                self.iconv,
                (&raw mut from).cast::<*mut c_char>(),
                &raw mut fromlen,
                &raw mut to,
                &raw mut tolen,
            ) == -1i32 as size_t
                && *__errno_location() != ICONV_EINVAL
            {
                self.conv_error = true;
                return None;
            }
            Some((
                self.conv_buf,
                to.offset_from(self.conv_buf) as usize,
                inlen - fromlen as usize,
            ))
        }
    }
}
