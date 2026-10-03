//! The open undo file and the buffer it belongs to.
//!
//! Upstream's `bufinfo_T` paired the buffer with a `FILE *` and every field
//! went through `fwrite`/`fread`/`getc`. Here the stream is a buffered
//! [`File`] the record owns, so a reader or writer is a `&mut BufInfo` and the
//! file closes when the record drops. The byte format is unchanged: every
//! multi-byte field is big-endian, and a field read past the end of the file
//! answers -1, as `get2c`/`get4c`/`get8ctime` did.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

use super::format::encode_be;
use crate::types::{UndoLink, time_t};
use crate::winlayer::Buf;

/// Which way the undo file is open.
enum Stream {
    Read(BufReader<File>),
    Write(BufWriter<File>),
}

/// An undo file being read or written, and the buffer it describes.
pub(crate) struct BufInfo {
    pub(crate) bi_buf: Buf,
    stream: Stream,
}

impl BufInfo {
    /// `file`, open for reading the undo tree of `buffer`.
    pub(crate) fn reader(buffer: Buf, file: File) -> Self {
        BufInfo {
            bi_buf: buffer,
            stream: Stream::Read(BufReader::new(file)),
        }
    }

    /// `file`, open for writing the undo tree of `buffer`.
    pub(crate) fn writer(buffer: Buf, file: File) -> Self {
        BufInfo {
            bi_buf: buffer,
            stream: Stream::Write(BufWriter::new(file)),
        }
    }

    // -- writing ---------------------------------------------------------

    /// Writes `data`. False when the file refused it, or is open for reading.
    pub(crate) fn write(&mut self, data: &[u8]) -> bool {
        match &mut self.stream {
            Stream::Write(w) => w.write_all(data).is_ok(),
            Stream::Read(_) => false,
        }
    }

    /// Writes `nr` as a `len`-byte big-endian field. See [`encode_be`].
    pub(crate) fn write_bytes(&mut self, nr: u64, len: usize) -> bool {
        self.write(&encode_be(nr, len)[..len])
    }

    /// Writes one link as the four big-endian bytes of the sequence number
    /// it already holds -- 0 for a link to nothing, which is what the reader
    /// takes it back as.
    pub(crate) fn put_header_link(&mut self, link: UndoLink) {
        debug_assert!(link.seq() >= 0, "an undo link is 0 or a sequence number");
        self.write_bytes(u64::from(link.seq().cast_unsigned()), 4);
    }

    /// Writes a timestamp as eight bytes. The one clock-dependent field in
    /// the whole file.
    pub(crate) fn put_time(&mut self, when: time_t) {
        self.write(&when.cast_unsigned().to_be_bytes());
    }

    /// Writes the optional-field trailer that both the file header and every
    /// undo header end with: one tagged four-byte field, then the
    /// terminator.
    ///
    /// The reader skips a tag it does not know by the length written here,
    /// which is what makes this an extension point.
    pub(crate) fn put_optional_field(&mut self, what: c_int, value: c_int) {
        // The field's length, then its tag.
        self.write_bytes(4, 1);
        self.write_bytes(u64::from(what.cast_unsigned()), 1);
        self.write_bytes(u64::from(value.cast_unsigned()), 4);
        self.write_bytes(0, 1);
    }

    /// Flushes what is buffered and asks the system to put it on the disk.
    /// False only when the sync fails: a flush that fails skips it, as
    /// upstream's `fflush(fp) == 0 && os_fsync(fd) != 0` did.
    pub(crate) fn sync(&mut self) -> bool {
        match &mut self.stream {
            Stream::Write(w) => w.flush().is_err() || w.get_ref().sync_all().is_ok(),
            Stream::Read(_) => false,
        }
    }

    // -- reading ---------------------------------------------------------

    /// Fills `buf`, zeroing it if the file ran out first.
    pub(crate) fn read(&mut self, buf: &mut [u8]) -> bool {
        let read = match &mut self.stream {
            Stream::Read(r) => r.read_exact(buf).is_ok(),
            Stream::Write(_) => false,
        };
        if !read {
            buf.fill(0);
        }
        read
    }

    /// Reads an `N`-byte field, or `None` at the end of the file.
    fn read_array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let mut bytes = [0u8; N];
        self.read(&mut bytes).then_some(bytes)
    }

    /// Reads one byte, or -1 at the end of the file.
    pub(crate) fn read_byte(&mut self) -> c_int {
        self.read_array::<1>().map_or(-1, |[b]| c_int::from(b))
    }

    /// Reads a two-byte big-endian field, or -1 at the end of the file.
    pub(crate) fn read_2c(&mut self) -> c_int {
        self.read_array()
            .map_or(-1, |b| c_int::from(u16::from_be_bytes(b)))
    }

    /// Reads a four-byte big-endian field, or -1 at the end of the file. The
    /// result wraps rather than saturating when the top bit is set.
    pub(crate) fn read_4c(&mut self) -> c_int {
        self.read_array()
            .map_or(-1, |b| u32::from_be_bytes(b).cast_signed())
    }

    /// Reads an eight-byte timestamp, or -1 at the end of the file.
    pub(crate) fn read_time(&mut self) -> time_t {
        self.read_array()
            .map_or(-1, |b| u64::from_be_bytes(b).cast_signed())
    }
}
