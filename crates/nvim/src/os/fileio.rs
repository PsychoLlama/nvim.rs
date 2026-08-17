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
//! The `FileDescriptor` layout and the C signatures of the `file_*` entry
//! points are frozen: `test/unit/os/fileio_spec.lua` builds the struct
//! through LuaJIT's FFI and reads `fp.wr` back out of it. The block comes
//! from `alloc_block`, so the three positions stay raw pointers rather
//! than becoming a slice and an index.

use crate::event::libuv::uv_strerror;
use crate::log::{LOGLVL_ERR, logmsg_c};
use crate::memory::{alloc_block, free_block};
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
    flags & bits as c_int != 0
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_open(
    ret_fp: *mut FileDescriptor,
    fname: *const c_char,
    flags: c_int,
    mode: c_int,
) -> c_int {
    if has(flags, kFileMkDir) {
        let mkdir_ret = os_file_mkdir(fname as *mut c_char, 0o755);
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

/// Wrap an already-open descriptor in `ret_fp`.
///
/// The descriptor must not be touched by any other means afterwards.
/// Returns 0; the result is kept for signature compatibility.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_open_fd(
    ret_fp: *mut FileDescriptor,
    fd: c_int,
    flags: c_int,
) -> c_int {
    (*ret_fp).wr = has(flags, WRITING);
    (*ret_fp).non_blocking = has(flags, kFileNonBlocking);
    // Non-blocking writes are not supported.
    debug_assert!(!(*ret_fp).wr || !(*ret_fp).non_blocking);
    (*ret_fp).fd = fd;
    (*ret_fp).eof = false;
    (*ret_fp).buffer = alloc_block() as *mut c_char;
    (*ret_fp).read_pos = (*ret_fp).buffer;
    (*ret_fp).write_pos = (*ret_fp).buffer;
    (*ret_fp).bytes_read = 0;
    0
}

/// Open standard input as a `FileDescriptor`.
pub unsafe extern "C" fn file_open_stdin(fp: *mut FileDescriptor) -> c_int {
    let error = file_open_fd(
        fp,
        os_open_stdin_fd(),
        (kFileReadOnly | kFileNonBlocking) as c_int,
    );
    if error != 0 {
        logmsg_c!(
            LOGLVL_ERR,
            ptr::null(),
            c"file_open_stdin".as_ptr(),
            129,
            true,
            c"failed to open stdin: %s".as_ptr(),
            uv_strerror(error),
        );
    }
    error
}

/// Wrap an in-memory buffer for reading. `data` is borrowed, not owned:
/// there is no block to free, and `file_close` is a no-op on the result
/// (`fd` is -1).
pub unsafe extern "C" fn file_open_buffer(
    ret_fp: *mut FileDescriptor,
    data: *mut c_char,
    len: size_t,
) {
    (*ret_fp).wr = false;
    (*ret_fp).non_blocking = false;
    (*ret_fp).fd = -1;
    (*ret_fp).eof = true;
    (*ret_fp).buffer = ptr::null_mut();
    (*ret_fp).read_pos = data;
    (*ret_fp).write_pos = data.add(len);
    (*ret_fp).bytes_read = 0;
}

/// Flush, close, and release the block. A close error outranks a flush
/// error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_close(fp: *mut FileDescriptor, do_fsync: bool) -> c_int {
    if (*fp).fd < 0 {
        return 0;
    }
    let flush_error = if do_fsync {
        file_fsync(fp)
    } else {
        file_flush(fp)
    };
    let close_error = os_close((*fp).fd);
    free_block((*fp).buffer as *mut core::ffi::c_void);
    if close_error != 0 {
        return close_error;
    }
    flush_error
}

/// Flush pending output and `fsync` it.
///
/// A file that cannot be synced at all (EINVAL, a read-only filesystem, or
/// storage without fsync support) is reported as success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_fsync(fp: *mut FileDescriptor) -> c_int {
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
    0
}

/// Write out whatever is pending in the block.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_flush(fp: *mut FileDescriptor) -> c_int {
    if !(*fp).wr {
        return 0;
    }
    let to_write = buffered_len(&*fp);
    if to_write == 0 {
        return 0;
    }
    let wres = os_write((*fp).fd, (*fp).read_pos, to_write, (*fp).non_blocking);
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).buffer;
    if wres == to_write as ptrdiff_t {
        return 0;
    }
    // A short write with no error of its own is an I/O error here.
    if wres >= 0 { UV_EIO } else { wres as c_int }
}

/// Read `size` bytes into `ret_buf`. Returns the number of bytes read
/// (less than `size` only at EOF, or on a non-blocking file) or a libuv
/// error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_read(
    fp: *mut FileDescriptor,
    ret_buf: *mut c_char,
    size: size_t,
) -> ptrdiff_t {
    debug_assert!(!(*fp).wr);
    let out: &mut [u8] = if size == 0 {
        &mut []
    } else {
        slice::from_raw_parts_mut(ret_buf as *mut u8, size)
    };

    // Serve what the block already holds.
    let from_buffer = buffered_len(&*fp).min(size);
    if from_buffer != 0 {
        out[..from_buffer].copy_from_slice(slice::from_raw_parts(
            (*fp).read_pos as *const u8,
            from_buffer,
        ));
    }
    let mut read_remaining = size - from_buffer;
    if read_remaining == 0 {
        (*fp).bytes_read += from_buffer as u64;
        (*fp).read_pos = (*fp).read_pos.add(from_buffer);
        return from_buffer as ptrdiff_t;
    }

    // The block is spent; restart it from the beginning.
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).buffer;

    let mut filled = from_buffer;
    let mut called_read = false;
    while read_remaining != 0 {
        // At most one os_readv call on a non-blocking file.
        if (*fp).eof || (called_read && (*fp).non_blocking) {
            break;
        }
        // Fill the caller's buffer and the block in the same syscall; a
        // read that overshoots the request lands in the block and is
        // served from there next time.
        let mut iov = [
            iovec {
                iov_base: out[filled..].as_mut_ptr() as *mut core::ffi::c_void,
                iov_len: read_remaining,
            },
            iovec {
                iov_base: (*fp).write_pos as *mut core::ffi::c_void,
                iov_len: ARENA_BLOCK_SIZE,
            },
        ];
        let r_ret = os_readv(
            (*fp).fd,
            &raw mut (*fp).eof,
            iov.as_mut_ptr(),
            iov.len(),
            (*fp).non_blocking,
        );
        if r_ret < 0 {
            return r_ret;
        }
        let read = r_ret as usize;
        if read > read_remaining {
            (*fp).write_pos = (*fp).write_pos.add(read - read_remaining);
            read_remaining = 0;
        } else {
            filled += read;
            read_remaining -= read;
        }
        called_read = true;
    }

    (*fp).bytes_read += (size - read_remaining) as u64;
    (size - read_remaining) as ptrdiff_t
}

/// Hand out `size` already-buffered bytes in place, or NULL when the block
/// does not hold that many. The pointer dies at the next [`file_read`].
pub unsafe extern "C" fn file_try_read_buffered(
    fp: *mut FileDescriptor,
    size: size_t,
) -> *mut c_char {
    if buffered_len(&*fp) < size {
        return ptr::null_mut();
    }
    let ret = (*fp).read_pos;
    (*fp).read_pos = (*fp).read_pos.add(size);
    (*fp).bytes_read += size as u64;
    ret
}

/// Write `size` bytes of `buf`, buffering them when they fit. Returns the
/// number of bytes accepted or a libuv error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_write(
    fp: *mut FileDescriptor,
    buf: *const c_char,
    size: size_t,
) -> ptrdiff_t {
    debug_assert!((*fp).wr);
    // The `<` (rather than `<=`) is upstream's: a write that exactly fills
    // the block flushes instead of filling it.
    if size >= free_space(&*fp) {
        let status = file_flush(fp);
        if status < 0 {
            return status as ptrdiff_t;
        }
        if size >= ARENA_BLOCK_SIZE {
            // Too big to buffer; hand it straight to the file.
            let wres = os_write((*fp).fd, buf, size, (*fp).non_blocking);
            if wres != size as ptrdiff_t && wres >= 0 {
                return UV_EIO as ptrdiff_t;
            }
            return wres;
        }
    }
    if size != 0 {
        slice::from_raw_parts_mut((*fp).write_pos as *mut u8, size)
            .copy_from_slice(slice::from_raw_parts(buf as *const u8, size));
    }
    (*fp).write_pos = (*fp).write_pos.add(size);
    size as ptrdiff_t
}

/// Discard `size` bytes, like `fseek(fp, size, SEEK_CUR)` — but really by
/// reading into the block and throwing the result away.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_skip(fp: *mut FileDescriptor, size: size_t) -> ptrdiff_t {
    debug_assert!(!(*fp).wr);
    let from_buffer = buffered_len(&*fp).min(size);
    let mut skip_remaining = size - from_buffer;
    if skip_remaining == 0 {
        (*fp).read_pos = (*fp).read_pos.add(from_buffer);
        (*fp).bytes_read += from_buffer as u64;
        return from_buffer as ptrdiff_t;
    }

    // The block is spent; restart it from the beginning.
    (*fp).write_pos = (*fp).buffer;
    (*fp).read_pos = (*fp).buffer;

    let mut called_read = false;
    while skip_remaining > 0 {
        // At most one os_read call on a non-blocking file.
        if (*fp).eof || (called_read && (*fp).non_blocking) {
            break;
        }
        let r_ret = os_read(
            (*fp).fd,
            &raw mut (*fp).eof,
            (*fp).buffer,
            ARENA_BLOCK_SIZE,
            (*fp).non_blocking,
        );
        if r_ret < 0 {
            return r_ret;
        }
        if r_ret as usize > skip_remaining {
            // Overshot: keep the excess buffered for the next read.
            (*fp).read_pos = (*fp).buffer.add(skip_remaining);
            (*fp).write_pos = (*fp).buffer.add(r_ret as usize);
            (*fp).bytes_read += size as u64;
            return size as ptrdiff_t;
        }
        skip_remaining -= r_ret as usize;
        called_read = true;
    }

    (*fp).bytes_read += (size - skip_remaining) as u64;
    (size - skip_remaining) as ptrdiff_t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags_of(bits: FileOpenFlags) -> c_int {
        open_flags(bits as c_int)
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
