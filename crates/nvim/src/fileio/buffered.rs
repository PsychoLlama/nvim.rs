//! Buffered reading from and writing to a file: an `fopen`/`fread`/
//! `fwrite` replacement with no buffers, autocommands or any other editor
//! state involved.
//!
//! A [`BufferedFile`] is either a read side or a write side, never both --
//! the flags have to pick one. It buffers through a block of
//! [`BLOCK_BYTES`] bytes; `read_pos..write_pos` delimits the live region
//! inside it (pending output when writing, unconsumed input when reading).
//! A read that the block cannot serve fills the caller's buffer and the
//! block in one `readv`, so a read that overshoots the request is served
//! from the block next time.
//!
//! Errors are libuv codes -- `-errno` on every platform nvim builds for,
//! which is what an [`io::Error`](std::io::Error)'s OS code turns into here -- so the
//! messages callers build from them read as they did when the calls went
//! through libuv.
//!
//! [`ReadFile`] and [`WriteFile`] are the two sides as types, for the
//! editor; the unit suite drives [`BufferedFile`] itself.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The flag names keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::event::uv::{uv_gid_t, uv_uid_t};
use crate::log::{LOGLVL_ERR, logmsg};
use crate::os::fs::{close_file, file_mkdir, fsync_file, os_strerror, stdin_file};
use crate::os::libc::ptrdiff_t;
use crate::os::uv_error::{
    UV_EAGAIN, UV_EBADF, UV_EINTR, UV_EINVAL, UV_EIO, UV_ENOTSUP, UV_EROFS, uv_code,
};
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{IoSliceMut, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;

/// Size of the block a [`BufferedFile`] buffers through.
pub(crate) const BLOCK_BYTES: usize = 4096;

/// `open(2)` flags.
const O_RDONLY: c_int = 0;
const O_WRONLY: c_int = 0o1;
const O_ACCMODE: c_int = 0o3;
const O_CREAT: c_int = 0o100;
const O_EXCL: c_int = 0o200;
const O_TRUNC: c_int = 0o1000;
const O_APPEND: c_int = 0o2000;
const O_NOFOLLOW: c_int = 0o400000;

/// The `flags` argument of [`BufferedFile::open`].
pub type FileOpenFlags = c_uint;
pub const FILE_READ_ONLY: FileOpenFlags = 1;
pub const FILE_CREATE: FileOpenFlags = 2;
pub const FILE_WRITE_ONLY: FileOpenFlags = 4;
pub const FILE_NO_SYMLINK: FileOpenFlags = 8;
pub const FILE_CREATE_ONLY: FileOpenFlags = 16;
pub const FILE_TRUNCATE: FileOpenFlags = 32;
pub(crate) const FILE_APPEND: FileOpenFlags = 64;
pub(crate) const kFileNonBlocking: FileOpenFlags = 128;
pub(crate) const FILE_MK_DIR: FileOpenFlags = 256;

/// Every flag that opens the file for writing.
const WRITING: FileOpenFlags =
    FILE_CREATE | FILE_CREATE_ONLY | FILE_TRUNCATE | FILE_APPEND | FILE_WRITE_ONLY;

fn has(flags: FileOpenFlags, bits: FileOpenFlags) -> bool {
    flags & bits != 0
}

/// A byte count as the `ssize_t` the entry points answer with. Every one
/// of them is bounded by a slice's length.
fn as_signed(n: usize) -> ptrdiff_t {
    ptrdiff_t::try_from(n).expect("a buffered read or write fits an ssize_t")
}

/// A byte count as the running total the file keeps.
fn as_u64(n: usize) -> u64 {
    u64::try_from(n).expect("a byte count fits 64 bits")
}

/// A libuv code as the `ssize_t` the read and write entry points answer.
fn code_as_signed(code: c_int) -> ptrdiff_t {
    ptrdiff_t::try_from(code).expect("a libuv error code fits an ssize_t")
}

/// Translate [`FileOpenFlags`] into `open(2)` flags.
///
/// Every creating or truncating flag also implies `O_WRONLY`: this
/// interface never opens a file for reading and writing at once. The
/// assertions are upstream's -- `FILE_CREATE_ONLY` excludes the other
/// creating flags, and `FILE_READ_ONLY` excludes all of them.
fn open_flags(flags: FileOpenFlags) -> c_int {
    let mut oflags = 0;
    if has(flags, FILE_WRITE_ONLY) {
        oflags |= O_WRONLY;
    }
    if has(flags, FILE_CREATE_ONLY) {
        oflags |= O_CREAT | O_EXCL | O_WRONLY;
    }
    if has(flags, FILE_CREATE) {
        debug_assert!(!has(flags, FILE_CREATE_ONLY));
        oflags |= O_CREAT | O_WRONLY;
    }
    if has(flags, FILE_TRUNCATE) {
        debug_assert!(!has(flags, FILE_CREATE_ONLY));
        oflags |= O_TRUNC | O_WRONLY;
    }
    if has(flags, FILE_APPEND) {
        debug_assert!(!has(flags, FILE_CREATE_ONLY));
        oflags |= O_APPEND | O_WRONLY;
    }
    if has(flags, FILE_READ_ONLY) {
        debug_assert!(!has(flags, WRITING));
        oflags |= O_RDONLY;
    }
    if has(flags, FILE_NO_SYMLINK) {
        oflags |= O_NOFOLLOW;
    }
    if has(flags, FILE_MK_DIR) {
        debug_assert!(!has(flags, FILE_CREATE_ONLY));
        oflags |= O_CREAT | O_WRONLY;
    }
    oflags
}

/// Fill `bufs` from `file`, looping over short reads: how many bytes came,
/// or a negative libuv code. `eof` says whether the file ended. On a
/// `non_blocking` file a read that would block ends the loop.
fn read_into(
    file: &File,
    eof: &mut bool,
    mut bufs: &mut [IoSliceMut<'_>],
    non_blocking: bool,
) -> ptrdiff_t {
    *eof = false;
    let toread: usize = bufs.iter().map(|buf| buf.len()).sum();
    let mut read_bytes = 0;
    while read_bytes < toread && !bufs.is_empty() && !*eof {
        match (&*file).read_vectored(bufs) {
            Ok(0) => *eof = true,
            Ok(count) => {
                read_bytes += count;
                IoSliceMut::advance_slices(&mut bufs, count);
            }
            Err(error) => {
                let error = uv_code(&error);
                if non_blocking && error == UV_EAGAIN {
                    break;
                }
                if error != UV_EINTR && error != UV_EAGAIN {
                    return code_as_signed(error);
                }
            }
        }
    }
    as_signed(read_bytes)
}

/// Write all of `bytes` to `file`, looping over short writes: how many
/// bytes went, or a negative libuv code.
fn write_from(file: &File, bytes: &[u8], non_blocking: bool) -> ptrdiff_t {
    let mut written = 0;
    while written != bytes.len() {
        match (&*file).write(&bytes[written..]) {
            // A zero-length write of a non-empty buffer has no explanation.
            Ok(0) => return code_as_signed(crate::os::uv_error::UV_UNKNOWN),
            Ok(count) => written += count,
            Err(error) => {
                let error = uv_code(&error);
                if non_blocking && error == UV_EAGAIN {
                    break;
                }
                if error != UV_EINTR && error != UV_EAGAIN {
                    return code_as_signed(error);
                }
            }
        }
    }
    as_signed(written)
}

/// An open file and the block it buffers through.
#[derive(Debug)]
pub struct BufferedFile {
    file: File,
    block: Box<[u8]>,
    read_pos: usize,
    write_pos: usize,
    wr: bool,
    eof: bool,
    non_blocking: bool,
    bytes_read: u64,
}

impl BufferedFile {
    /// Open `fname` with `flags` and, for a file it creates, `mode`; or the
    /// libuv error code saying why not.
    pub fn open(fname: &CStr, flags: FileOpenFlags, mode: c_int) -> Result<Self, c_int> {
        if has(flags, FILE_MK_DIR) {
            file_mkdir(fname, 0o755)?;
        }
        let oflags = open_flags(flags);
        let writing = oflags & O_ACCMODE != O_RDONLY;
        // The access mode is the `read`/`write` pair; every other flag goes
        // through as it is, `O_CLOEXEC` added as libuv adds it.
        let file = OpenOptions::new()
            .read(!writing)
            .write(writing)
            .custom_flags(oflags & !O_ACCMODE)
            .mode(mode.cast_unsigned())
            .open(OsStr::from_bytes(fname.to_bytes()))
            .map_err(|error| uv_code(&error))?;
        Ok(Self::from_file(file, flags))
    }

    /// Buffer an already-open `file`, which nothing else may touch
    /// afterwards. `flags` says which side it is.
    pub fn from_file(file: File, flags: FileOpenFlags) -> Self {
        let wr = has(flags, WRITING);
        let non_blocking = has(flags, kFileNonBlocking);
        // Non-blocking writes are not supported.
        debug_assert!(!wr || !non_blocking);
        BufferedFile {
            file,
            block: vec![0; BLOCK_BYTES].into_boxed_slice(),
            read_pos: 0,
            write_pos: 0,
            wr,
            eof: false,
            non_blocking,
            bytes_read: 0,
        }
    }

    /// Whether this is the write side.
    pub fn writing(&self) -> bool {
        self.wr
    }

    /// Unconsumed input, or pending output.
    fn buffered(&self) -> usize {
        self.write_pos - self.read_pos
    }

    /// Read into `buf`: how many bytes came (fewer than asked only at EOF,
    /// or on a non-blocking file), or a negative libuv code.
    pub fn read(&mut self, buf: &mut [u8]) -> ptrdiff_t {
        debug_assert!(!self.wr);
        let size = buf.len();

        // Serve what the block already holds.
        let from_buffer = self.buffered().min(size);
        buf[..from_buffer].copy_from_slice(&self.block[self.read_pos..self.read_pos + from_buffer]);
        let mut read_remaining = size - from_buffer;
        if read_remaining == 0 {
            self.bytes_read += as_u64(from_buffer);
            self.read_pos += from_buffer;
            return as_signed(from_buffer);
        }

        // The block is spent; restart it from the beginning.
        self.write_pos = 0;
        self.read_pos = 0;

        let mut filled = from_buffer;
        let mut called_read = false;
        while read_remaining != 0 {
            // At most one read call on a non-blocking file.
            if self.eof || (called_read && self.non_blocking) {
                break;
            }
            // Fill the caller's buffer and the block in the same syscall.
            let mut bufs = [
                IoSliceMut::new(&mut buf[filled..]),
                IoSliceMut::new(&mut self.block[self.write_pos..]),
            ];
            let r_ret = read_into(&self.file, &mut self.eof, &mut bufs, self.non_blocking);
            let Ok(read) = usize::try_from(r_ret) else {
                return r_ret;
            };
            if read > read_remaining {
                self.write_pos += read - read_remaining;
                read_remaining = 0;
            } else {
                filled += read;
                read_remaining -= read;
            }
            called_read = true;
        }

        self.bytes_read += as_u64(size - read_remaining);
        as_signed(size - read_remaining)
    }

    /// Discard `size` bytes, like `fseek(fp, size, SEEK_CUR)` -- but really
    /// by reading into the block and throwing the result away. How many
    /// went (fewer only at EOF, or on a non-blocking file), or a negative
    /// libuv code.
    pub fn skip(&mut self, size: usize) -> ptrdiff_t {
        debug_assert!(!self.wr);
        let from_buffer = self.buffered().min(size);
        let mut skip_remaining = size - from_buffer;
        if skip_remaining == 0 {
            self.read_pos += from_buffer;
            self.bytes_read += as_u64(from_buffer);
            return as_signed(from_buffer);
        }

        // The block is spent; restart it from the beginning.
        self.write_pos = 0;
        self.read_pos = 0;

        let mut called_read = false;
        while skip_remaining > 0 {
            // At most one read call on a non-blocking file.
            if self.eof || (called_read && self.non_blocking) {
                break;
            }
            let mut bufs = [IoSliceMut::new(&mut self.block[..])];
            let r_ret = read_into(&self.file, &mut self.eof, &mut bufs, self.non_blocking);
            let Ok(read) = usize::try_from(r_ret) else {
                return r_ret;
            };
            if read > skip_remaining {
                // Overshot: keep the excess buffered for the next read.
                self.read_pos = skip_remaining;
                self.write_pos = read;
                self.bytes_read += as_u64(size);
                return as_signed(size);
            }
            skip_remaining -= read;
            called_read = true;
        }

        self.bytes_read += as_u64(size - skip_remaining);
        as_signed(size - skip_remaining)
    }

    /// Write `data`, buffering it when it fits: how many bytes were
    /// accepted, or a negative libuv code.
    pub fn write(&mut self, data: &[u8]) -> ptrdiff_t {
        debug_assert!(self.wr);
        let size = data.len();
        // The `>=` (rather than `>`) is upstream's: a write that exactly
        // fills the block flushes instead of filling it.
        if size >= BLOCK_BYTES - self.write_pos {
            if let Err(code) = self.flush() {
                return code_as_signed(code);
            }
            if size >= BLOCK_BYTES {
                // Too big to buffer; hand it straight to the file.
                let wres = write_from(&self.file, data, self.non_blocking);
                if wres != as_signed(size) && wres >= 0 {
                    return code_as_signed(UV_EIO);
                }
                return wres;
            }
        }
        self.block[self.write_pos..self.write_pos + size].copy_from_slice(data);
        self.write_pos += size;
        as_signed(size)
    }

    /// Write out whatever is pending, or answer the libuv code.
    pub fn flush(&mut self) -> Result<(), c_int> {
        if !self.wr {
            return Ok(());
        }
        let pending = &self.block[self.read_pos..self.write_pos];
        if pending.is_empty() {
            return Ok(());
        }
        let wres = write_from(&self.file, pending, self.non_blocking);
        let to_write = as_signed(pending.len());
        self.write_pos = 0;
        self.read_pos = 0;
        if wres == to_write {
            return Ok(());
        }
        // A short write with no error of its own is an I/O error here.
        Err(if wres >= 0 {
            UV_EIO
        } else {
            c_int::try_from(wres).expect("a libuv error code is an int")
        })
    }

    /// Flush pending output and `fsync` it, or answer the libuv code.
    ///
    /// A file that cannot be synced at all (EINVAL, a read-only filesystem,
    /// or storage without fsync support) is reported as success.
    pub fn fsync(&mut self) -> Result<(), c_int> {
        if !self.wr {
            return Ok(());
        }
        self.flush()?;
        match fsync_file(&self.file) {
            Err(error) if error != UV_EINVAL && error != UV_EROFS && error != UV_ENOTSUP => {
                Err(error)
            }
            _ => Ok(()),
        }
    }

    /// Flush (and `fsync` when asked), then close, or answer the libuv
    /// code. A close error outranks a flush error.
    pub fn close(mut self, do_fsync: bool) -> Result<(), c_int> {
        let flushed = if do_fsync { self.fsync() } else { self.flush() };
        close_file(self.file).and(flushed)
    }
}

/// A [`BufferedFile`] opened for writing.
///
/// The only way to make one is [`WriteFile::open`]. Closing is not a
/// `Drop`: closing answers an error code the caller reports, so
/// [`WriteFile::close`] spends the value instead.
#[derive(Debug)]
pub(crate) struct WriteFile(BufferedFile);

impl WriteFile {
    /// Open `fname` with the [`FileOpenFlags`] `flags` (which must ask for
    /// writing) and `mode`, or the libuv error code saying why not.
    pub(crate) fn open(fname: &CStr, flags: FileOpenFlags, mode: c_int) -> Result<Self, c_int> {
        debug_assert!(has(flags, WRITING));
        BufferedFile::open(fname, flags, mode).map(Self)
    }

    /// Write `data`, answering how many bytes were accepted or a negative
    /// libuv error code.
    pub(crate) fn write(&mut self, data: &[u8]) -> ptrdiff_t {
        self.0.write(data)
    }

    /// Give the open file to `owner` and `group`; zero or a libuv error
    /// code.
    pub(crate) fn fchown(&self, owner: uv_uid_t, group: uv_gid_t) -> c_int {
        match std::os::unix::fs::fchown(&self.0.file, Some(owner), Some(group)) {
            Ok(()) => 0,
            Err(error) => uv_code(&error),
        }
    }

    /// Write out whatever is pending, or answer the libuv error code.
    pub(crate) fn flush(&mut self) -> Result<(), c_int> {
        self.0.flush()
    }

    /// Flush, `fsync` when asked, and close, or answer the libuv error
    /// code.
    pub(crate) fn close(self, do_fsync: bool) -> Result<(), c_int> {
        self.0.close(do_fsync)
    }
}

/// A [`BufferedFile`] opened for reading: the read half of [`WriteFile`].
/// `-s` and `:source!` read their keys through one. Closing is not a
/// `Drop`, as for `WriteFile`.
#[derive(Debug)]
pub(crate) struct ReadFile(BufferedFile);

impl ReadFile {
    /// Open `fname` with the [`FileOpenFlags`] `flags` (which must not ask
    /// for writing), or the libuv error code saying why not.
    pub(crate) fn open(fname: &CStr, flags: FileOpenFlags) -> Result<Self, c_int> {
        debug_assert!(!has(flags, WRITING));
        BufferedFile::open(fname, flags, 0).map(Self)
    }

    /// Standard input, read without blocking, or the libuv error code
    /// saying why not.
    pub(crate) fn stdin() -> Result<Self, c_int> {
        let Some(file) = stdin_file() else {
            let why = os_strerror(UV_EBADF);
            logmsg!(
                LOGLVL_ERR,
                c"file_open_stdin",
                129,
                "failed to open stdin: {}",
                crate::message_fmt::msg_cstr(why)
            );
            return Err(UV_EBADF);
        };
        Ok(Self(BufferedFile::from_file(
            file,
            FILE_READ_ONLY | kFileNonBlocking,
        )))
    }

    /// Read into `buf`: how many bytes came (fewer than asked only at EOF,
    /// or on a non-blocking file), or a negative libuv error code.
    pub(crate) fn read(&mut self, buf: &mut [u8]) -> ptrdiff_t {
        self.0.read(buf)
    }

    /// Discard `size` bytes: how many went (fewer only at EOF, or on a
    /// non-blocking file), or a negative libuv error code.
    pub(crate) fn skip(&mut self, size: usize) -> ptrdiff_t {
        self.0.skip(size)
    }

    /// `size` bytes the block already holds, handed out in place and
    /// consumed, or `None` when it holds fewer. The borrow ends before the
    /// next read can refill the block.
    pub(crate) fn try_read_buffered(&mut self, size: usize) -> Option<&[u8]> {
        let file = &mut self.0;
        if file.buffered() < size {
            return None;
        }
        let at = file.read_pos;
        file.read_pos += size;
        file.bytes_read += as_u64(size);
        Some(&file.block[at..at + size])
    }

    /// How many bytes the block holds that have not been read yet.
    pub(crate) fn buffered(&self) -> usize {
        self.0.buffered()
    }

    /// Whether everything the file holds has been read.
    pub(crate) fn at_eof(&self) -> bool {
        self.0.eof && self.0.read_pos == self.0.write_pos
    }

    /// How many bytes reading and skipping have consumed so far.
    pub(crate) fn bytes_read(&self) -> u64 {
        self.0.bytes_read
    }

    /// Close, or the libuv error code saying what went wrong.
    pub(crate) fn close(self) -> Result<(), c_int> {
        self.0.close(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_flags_are_the_default() {
        assert_eq!(open_flags(0), O_RDONLY);
        assert_eq!(open_flags(FILE_READ_ONLY), O_RDONLY);
    }

    #[test]
    fn every_writing_flag_implies_write_only() {
        for bit in [
            FILE_WRITE_ONLY,
            FILE_CREATE,
            FILE_CREATE_ONLY,
            FILE_TRUNCATE,
            FILE_APPEND,
            FILE_MK_DIR,
        ] {
            assert_ne!(open_flags(bit) & O_WRONLY, 0, "{bit:#x}");
        }
    }

    #[test]
    fn create_only_is_exclusive_create() {
        assert_eq!(open_flags(FILE_CREATE_ONLY), O_CREAT | O_EXCL | O_WRONLY);
        // Plain kFileCreate opens an existing file instead of failing.
        assert_eq!(open_flags(FILE_CREATE), O_CREAT | O_WRONLY);
    }

    #[test]
    fn flags_accumulate() {
        assert_eq!(
            open_flags(FILE_CREATE | FILE_TRUNCATE | FILE_NO_SYMLINK),
            O_CREAT | O_TRUNC | O_WRONLY | O_NOFOLLOW
        );
        // Non-blocking is a property of the buffered file, not an open flag.
        assert_eq!(open_flags(FILE_READ_ONLY | kFileNonBlocking), O_RDONLY);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn create_only_cannot_be_combined_with_create() {
        open_flags(FILE_CREATE_ONLY | FILE_CREATE);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn read_only_cannot_be_combined_with_a_writing_flag() {
        open_flags(FILE_READ_ONLY | FILE_APPEND);
    }
}
