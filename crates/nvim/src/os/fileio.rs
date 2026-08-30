//! Buffered reading from and writing to a file: an `fopen`/`fread`/
//! `fwrite` replacement over libuv, with no buffers, autocommands or any
//! other editor state involved.
//!
//! A [`FileDescriptor`] is either a read side or a write side, never both —
//! the flags have to pick one. Its `buffer` is an arena block of
//! [`ARENA_BLOCK_SIZE`] bytes; `read_pos..write_pos` delimits the live
//! region inside it (pending output when writing, unconsumed input when
//! reading).
//!
//! # Boundary
//!
//! `crates/nvim/tests/unit/fileio.rs` builds a `FileDescriptor` from
//! outside the crate and reads `wr` back out of it, which is why the type
//! and the `file_*` entry points are `pub`. The block comes from
//! `alloc_block`, so the three positions stay raw pointers rather than
//! becoming a slice and an index.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::event::libuv::uv_strerror;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::memory::{alloc_block, free_block};
use crate::message_fmt::c_str;
use crate::os::fs::{
    os_close, os_file_mkdir, os_fsync, os_open, os_open_stdin_fd, os_read, os_readv, os_write,
};
use crate::os::uv_error::{UV_EINVAL, UV_EIO, UV_ENOTSUP, UV_EROFS};
use crate::types::{FileDescriptor, iovec, ptrdiff_t, size_t};
use core::ffi::{c_char, c_int, c_uint};
use core::{ptr, slice};

/// Size of the arena block a `FileDescriptor` buffers through.
const ARENA_BLOCK_SIZE: usize = 4096;

/// `open(2)` flags, spelled out rather than pulled from libc so the numbers
/// stay the ones `os_open` hands to libuv.
const O_RDONLY: c_int = 0;
const O_WRONLY: c_int = 0o1;
const O_CREAT: c_int = 0o100;
const O_EXCL: c_int = 0o200;
const O_TRUNC: c_int = 0o1000;
const O_APPEND: c_int = 0o2000;
const O_NOFOLLOW: c_int = 0o400000;

/// The `flags` argument of [`file_open`] and [`file_open_fd`].
pub type FileOpenFlags = c_uint;
pub const kFileReadOnly: FileOpenFlags = 1;
pub const kFileCreate: FileOpenFlags = 2;
pub const kFileWriteOnly: FileOpenFlags = 4;
pub const kFileNoSymlink: FileOpenFlags = 8;
pub const kFileCreateOnly: FileOpenFlags = 16;
pub const kFileTruncate: FileOpenFlags = 32;
pub const kFileAppend: FileOpenFlags = 64;
pub const kFileNonBlocking: FileOpenFlags = 128;
pub const kFileMkDir: FileOpenFlags = 256;

/// Every flag that opens the file for writing.
const WRITING: FileOpenFlags =
    kFileCreate | kFileCreateOnly | kFileTruncate | kFileAppend | kFileWriteOnly;

fn has(flags: c_int, bits: FileOpenFlags) -> bool {
    flags & bits.cast_signed() != 0
}

/// A byte count as the `ssize_t` the `file_*` entry points answer with.
/// Every one of them is bounded by a caller-supplied `size`, which came from
/// the same type.
fn as_signed(n: usize) -> ptrdiff_t {
    ptrdiff_t::try_from(n).expect("a buffered read or write fits an ssize_t")
}

/// A byte count as the running total the descriptor keeps.
fn as_u64(n: usize) -> u64 {
    u64::try_from(n).expect("a byte count fits 64 bits")
}

/// Unconsumed input, or pending output: the live `read_pos..write_pos`
/// region of the block.
fn buffered_len(fp: &FileDescriptor) -> usize {
    fp.write_pos.addr() - fp.read_pos.addr()
}

/// Room left in the block after `write_pos`.
fn free_space(fp: &FileDescriptor) -> usize {
    fp.buffer.addr() + ARENA_BLOCK_SIZE - fp.write_pos.addr()
}

/// Translate [`FileOpenFlags`] into `open(2)` flags.
///
/// Every creating or truncating flag also implies `O_WRONLY`: this
/// interface never opens a file for reading and writing at once. The
/// assertions are upstream's — `kFileCreateOnly` excludes the other
/// creating flags, and `kFileReadOnly` excludes all of them.
fn open_flags(flags: c_int) -> c_int {
    let mut oflags = 0;
    if has(flags, kFileWriteOnly) {
        oflags |= O_WRONLY;
    }
    if has(flags, kFileCreateOnly) {
        oflags |= O_CREAT | O_EXCL | O_WRONLY;
    }
    if has(flags, kFileCreate) {
        debug_assert!(!has(flags, kFileCreateOnly));
        oflags |= O_CREAT | O_WRONLY;
    }
    if has(flags, kFileTruncate) {
        debug_assert!(!has(flags, kFileCreateOnly));
        oflags |= O_TRUNC | O_WRONLY;
    }
    if has(flags, kFileAppend) {
        debug_assert!(!has(flags, kFileCreateOnly));
        oflags |= O_APPEND | O_WRONLY;
    }
    if has(flags, kFileReadOnly) {
        debug_assert!(!has(flags, WRITING));
        oflags |= O_RDONLY;
    }
    if has(flags, kFileNoSymlink) {
        oflags |= O_NOFOLLOW;
    }
    if has(flags, kFileMkDir) {
        debug_assert!(!has(flags, kFileCreateOnly));
        oflags |= O_CREAT | O_WRONLY;
    }
    oflags
}

/// Open `fname` and wrap it in `ret_fp`. Returns 0, or a libuv error code
/// (see `os_strerror`).
///
/// # Safety
///
/// `fname` is NUL-terminated and `ret_fp` points to writable, possibly
/// uninitialized `FileDescriptor` storage.
pub unsafe fn file_open(
    ret_fp: *mut FileDescriptor,
    fname: *const c_char,
    flags: c_int,
    mode: c_int,
) -> c_int {
    // SAFETY: the caller's path and storage.
    unsafe {
        if has(flags, kFileMkDir) {
            let mkdir_ret = os_file_mkdir(fname.cast_mut(), 0o755);
            if mkdir_ret < 0 {
                return mkdir_ret;
            }
        }
        let fd = os_open(fname, open_flags(flags), mode);
        if fd < 0 {
            return fd;
        }
        file_open_fd(ret_fp, fd, flags)
    }
}

/// Wrap an already-open descriptor in `ret_fp`.
///
/// The descriptor must not be touched by any other means afterwards.
/// Returns 0; the result is kept for signature compatibility.
///
/// # Safety
///
/// `ret_fp` points to writable, possibly uninitialized `FileDescriptor`
/// storage, and `fd` is an open descriptor this call takes over.
pub unsafe fn file_open_fd(ret_fp: *mut FileDescriptor, fd: c_int, flags: c_int) -> c_int {
    // SAFETY: the caller's storage, written before anything reads it.
    let fp = unsafe { &mut *ret_fp };
    fp.wr = has(flags, WRITING);
    fp.non_blocking = has(flags, kFileNonBlocking);
    // Non-blocking writes are not supported.
    debug_assert!(!fp.wr || !fp.non_blocking);
    fp.fd = fd;
    fp.eof = false;
    // SAFETY: an arena block is `ARENA_BLOCK_SIZE` writable bytes, which is
    // exactly what the three positions below are bounded by.
    fp.buffer = unsafe { alloc_block() }.cast::<c_char>();
    fp.read_pos = fp.buffer;
    fp.write_pos = fp.buffer;
    fp.bytes_read = 0;
    0
}

/// Open standard input as a `FileDescriptor`.
///
/// # Safety
///
/// As [`file_open_fd`].
pub unsafe fn file_open_stdin(fp: *mut FileDescriptor) -> c_int {
    // SAFETY: the caller's storage; `os_open_stdin_fd` answers a descriptor
    // nothing else owns, and `uv_strerror` a static string.
    unsafe {
        let flags = (kFileReadOnly | kFileNonBlocking).cast_signed();
        let error = file_open_fd(fp, os_open_stdin_fd(), flags);
        if error != 0 {
            let (at, why) = (c"file_open_stdin", c_str(uv_strerror(error)));
            logmsg!(LOGLVL_ERR, at, 129, "failed to open stdin: {why}");
        }
        error
    }
}

/// Wrap an in-memory buffer for reading. `data` is borrowed, not owned:
/// there is no block to free, and `file_close` is a no-op on the result
/// (`fd` is -1).
///
/// # Safety
///
/// `ret_fp` points to writable `FileDescriptor` storage, and `data` is
/// readable for `len` bytes and outlives the descriptor.
pub unsafe fn file_open_buffer(ret_fp: *mut FileDescriptor, data: *mut c_char, len: size_t) {
    // SAFETY: the caller's storage, written before anything reads it.
    let fp = unsafe { &mut *ret_fp };
    fp.wr = false;
    fp.non_blocking = false;
    fp.fd = -1;
    fp.eof = true;
    fp.buffer = ptr::null_mut();
    fp.read_pos = data;
    // SAFETY: the caller's `len` readable bytes end here.
    fp.write_pos = unsafe { data.add(len) };
    fp.bytes_read = 0;
}

/// Flush, close, and release the block. A close error outranks a flush
/// error.
///
/// # Safety
///
/// `fp` points to a live `FileDescriptor`, which this call spends.
pub unsafe fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> c_int {
    // SAFETY: the caller's descriptor, and the arena block it owns.
    unsafe {
        if (*fp).fd < 0 {
            return 0;
        }
        let flush_error = if do_fsync {
            file_fsync(fp)
        } else {
            file_flush(fp)
        };
        let close_error = os_close((*fp).fd);
        free_block((*fp).buffer.cast::<core::ffi::c_void>());
        if close_error != 0 {
            return close_error;
        }
        flush_error
    }
}

/// Flush pending output and `fsync` it.
///
/// A file that cannot be synced at all (EINVAL, a read-only filesystem, or
/// storage without fsync support) is reported as success.
///
/// # Safety
///
/// `fp` points to a live `FileDescriptor`.
pub unsafe fn file_fsync(fp: *mut FileDescriptor) -> c_int {
    // SAFETY: the caller's descriptor.
    unsafe {
        if !(*fp).wr {
            return 0;
        }
        let flush_error = file_flush(fp);
        if flush_error != 0 {
            return flush_error;
        }
        let fsync_error = os_fsync((*fp).fd);
        if fsync_error != UV_EINVAL && fsync_error != UV_EROFS && fsync_error != UV_ENOTSUP {
            return fsync_error;
        }
    }
    0
}

/// Write out whatever is pending in the block.
///
/// # Safety
///
/// `fp` points to a live `FileDescriptor`.
pub unsafe fn file_flush(fp: *mut FileDescriptor) -> c_int {
    // SAFETY: the caller's descriptor; `read_pos..write_pos` is the pending
    // output inside its own block.
    let fp = unsafe { &mut *fp };
    if !fp.wr {
        return 0;
    }
    let to_write = buffered_len(fp);
    if to_write == 0 {
        return 0;
    }
    let wres = unsafe { os_write(fp.fd, fp.read_pos, to_write, fp.non_blocking) };
    fp.write_pos = fp.buffer;
    fp.read_pos = fp.buffer;
    if wres == as_signed(to_write) {
        return 0;
    }
    // A short write with no error of its own is an I/O error here.
    if wres >= 0 {
        UV_EIO
    } else {
        c_int::try_from(wres).expect("a libuv error code is an int")
    }
}

/// A count of bytes a syscall reported reading, which the callers below have
/// already tested for negativity.
fn read_count(n: ptrdiff_t) -> usize {
    usize::try_from(n).expect("a successful read is not negative")
}

/// Read `size` bytes into `ret_buf`. Returns the number of bytes read
/// (less than `size` only at EOF, or on a non-blocking file) or a libuv
/// error code.
///
/// # Safety
///
/// `fp` points to a live read-side `FileDescriptor` and `ret_buf` is
/// writable for `size` bytes.
pub unsafe fn file_read(fp: *mut FileDescriptor, ret_buf: *mut c_char, size: size_t) -> ptrdiff_t {
    // SAFETY: the caller's descriptor and output buffer. The block's three
    // positions stay inside the block, and `os_readv` only writes what the
    // iovecs describe.
    let fp = unsafe { &mut *fp };
    debug_assert!(!fp.wr);
    let out: &mut [u8] = if size == 0 {
        &mut []
    } else {
        unsafe { slice::from_raw_parts_mut(ret_buf.cast::<u8>(), size) }
    };

    // Serve what the block already holds.
    let from_buffer = buffered_len(fp).min(size);
    if from_buffer != 0 {
        let held = unsafe { slice::from_raw_parts(fp.read_pos.cast::<u8>(), from_buffer) };
        out[..from_buffer].copy_from_slice(held);
    }
    let mut read_remaining = size - from_buffer;
    if read_remaining == 0 {
        fp.bytes_read += as_u64(from_buffer);
        fp.read_pos = unsafe { fp.read_pos.add(from_buffer) };
        return as_signed(from_buffer);
    }

    // The block is spent; restart it from the beginning.
    fp.write_pos = fp.buffer;
    fp.read_pos = fp.buffer;

    let mut filled = from_buffer;
    let mut called_read = false;
    while read_remaining != 0 {
        // At most one os_readv call on a non-blocking file.
        if fp.eof || (called_read && fp.non_blocking) {
            break;
        }
        // Fill the caller's buffer and the block in the same syscall; a
        // read that overshoots the request lands in the block and is
        // served from there next time.
        let mut iov = [
            iovec {
                iov_base: out[filled..].as_mut_ptr().cast::<core::ffi::c_void>(),
                iov_len: read_remaining,
            },
            iovec {
                iov_base: fp.write_pos.cast::<core::ffi::c_void>(),
                iov_len: ARENA_BLOCK_SIZE,
            },
        ];
        let r_ret = unsafe {
            os_readv(
                fp.fd,
                &raw mut fp.eof,
                iov.as_mut_ptr(),
                iov.len(),
                fp.non_blocking,
            )
        };
        if r_ret < 0 {
            return r_ret;
        }
        let read = read_count(r_ret);
        if read > read_remaining {
            fp.write_pos = unsafe { fp.write_pos.add(read - read_remaining) };
            read_remaining = 0;
        } else {
            filled += read;
            read_remaining -= read;
        }
        called_read = true;
    }

    fp.bytes_read += as_u64(size - read_remaining);
    as_signed(size - read_remaining)
}

/// Hand out `size` already-buffered bytes in place, or NULL when the block
/// does not hold that many. The pointer dies at the next [`file_read`].
///
/// # Safety
///
/// `fp` points to a live read-side `FileDescriptor`.
pub unsafe fn file_try_read_buffered(fp: *mut FileDescriptor, size: size_t) -> *mut c_char {
    // SAFETY: the caller's descriptor; the advance stays inside the live
    // region the test above just measured.
    let fp = unsafe { &mut *fp };
    if buffered_len(fp) < size {
        return ptr::null_mut();
    }
    let ret = fp.read_pos;
    fp.read_pos = unsafe { fp.read_pos.add(size) };
    fp.bytes_read += as_u64(size);
    ret
}

/// Write `size` bytes of `buf`, buffering them when they fit. Returns the
/// number of bytes accepted or a libuv error code.
///
/// # Safety
///
/// `fp` points to a live write-side `FileDescriptor`, and `buf` is readable
/// for `size` bytes and does not point into the descriptor's own block.
pub unsafe fn file_write(fp: *mut FileDescriptor, buf: *const c_char, size: size_t) -> ptrdiff_t {
    // SAFETY: the caller's descriptor and input. The copy below only runs
    // once `size` is known to fit the space left after `write_pos`.
    let fp = unsafe { &mut *fp };
    debug_assert!(fp.wr);
    // The `<` (rather than `<=`) is upstream's: a write that exactly fills
    // the block flushes instead of filling it.
    if size >= free_space(fp) {
        let status = unsafe { file_flush(&raw mut *fp) };
        if status < 0 {
            return ptrdiff_t::try_from(status).expect("a libuv error code fits a pointer");
        }
        if size >= ARENA_BLOCK_SIZE {
            // Too big to buffer; hand it straight to the file.
            let wres = unsafe { os_write(fp.fd, buf, size, fp.non_blocking) };
            if wres != as_signed(size) && wres >= 0 {
                return ptrdiff_t::try_from(UV_EIO).expect("a libuv error code fits a pointer");
            }
            return wres;
        }
    }
    if size != 0 {
        unsafe { ptr::copy_nonoverlapping(buf.cast::<u8>(), fp.write_pos.cast::<u8>(), size) };
    }
    fp.write_pos = unsafe { fp.write_pos.add(size) };
    as_signed(size)
}

/// Discard `size` bytes, like `fseek(fp, size, SEEK_CUR)` — but really by
/// reading into the block and throwing the result away.
///
/// # Safety
///
/// `fp` points to a live read-side `FileDescriptor`.
pub unsafe fn file_skip(fp: *mut FileDescriptor, size: size_t) -> ptrdiff_t {
    // SAFETY: the caller's descriptor; every position below is bounded by
    // the block's own `ARENA_BLOCK_SIZE` bytes.
    let fp = unsafe { &mut *fp };
    debug_assert!(!fp.wr);
    let from_buffer = buffered_len(fp).min(size);
    let mut skip_remaining = size - from_buffer;
    if skip_remaining == 0 {
        fp.read_pos = unsafe { fp.read_pos.add(from_buffer) };
        fp.bytes_read += as_u64(from_buffer);
        return as_signed(from_buffer);
    }

    // The block is spent; restart it from the beginning.
    fp.write_pos = fp.buffer;
    fp.read_pos = fp.buffer;

    let mut called_read = false;
    while skip_remaining > 0 {
        // At most one os_read call on a non-blocking file.
        if fp.eof || (called_read && fp.non_blocking) {
            break;
        }
        let r_ret = unsafe {
            os_read(
                fp.fd,
                &raw mut fp.eof,
                fp.buffer,
                ARENA_BLOCK_SIZE,
                fp.non_blocking,
            )
        };
        if r_ret < 0 {
            return r_ret;
        }
        let read = read_count(r_ret);
        if read > skip_remaining {
            // Overshot: keep the excess buffered for the next read.
            fp.read_pos = unsafe { fp.buffer.add(skip_remaining) };
            fp.write_pos = unsafe { fp.buffer.add(read) };
            fp.bytes_read += as_u64(size);
            return as_signed(size);
        }
        skip_remaining -= read;
        called_read = true;
    }

    fp.bytes_read += as_u64(size - skip_remaining);
    as_signed(size - skip_remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags_of(bits: FileOpenFlags) -> c_int {
        open_flags(bits.cast_signed())
    }

    #[test]
    fn read_only_flags_are_the_default() {
        assert_eq!(flags_of(0), O_RDONLY);
        assert_eq!(flags_of(kFileReadOnly), O_RDONLY);
    }

    #[test]
    fn every_writing_flag_implies_write_only() {
        for bit in [
            kFileWriteOnly,
            kFileCreate,
            kFileCreateOnly,
            kFileTruncate,
            kFileAppend,
            kFileMkDir,
        ] {
            assert_ne!(flags_of(bit) & O_WRONLY, 0, "{bit:#x}");
        }
    }

    #[test]
    fn create_only_is_exclusive_create() {
        assert_eq!(flags_of(kFileCreateOnly), O_CREAT | O_EXCL | O_WRONLY);
        // Plain kFileCreate opens an existing file instead of failing.
        assert_eq!(flags_of(kFileCreate), O_CREAT | O_WRONLY);
    }

    #[test]
    fn flags_accumulate() {
        assert_eq!(
            flags_of(kFileCreate | kFileTruncate | kFileNoSymlink),
            O_CREAT | O_TRUNC | O_WRONLY | O_NOFOLLOW
        );
        // Non-blocking is a FileDescriptor property, not an open flag.
        assert_eq!(flags_of(kFileReadOnly | kFileNonBlocking), O_RDONLY);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn create_only_cannot_be_combined_with_create() {
        flags_of(kFileCreateOnly | kFileCreate);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn read_only_cannot_be_combined_with_a_writing_flag() {
        flags_of(kFileReadOnly | kFileAppend);
    }
}
