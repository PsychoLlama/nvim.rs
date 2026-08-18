//! Reading a file into a value -- `readfile()` and `readblob()`.
//!
//! Both builtins share [`read_file_or_blob`], which reads the flags, opens the
//! path and then hands the stream to one of two fillers: [`read_blob`], which
//! copies a slice of the file -- given by an offset that may count back from
//! the end and a size that may be capped by it -- into a Blob, or
//! [`read_lines`], which splits the bytes into a List.
//!
//! # The splitter
//!
//! [`read_lines`] reads into a fixed buffer and walks it byte by byte, which
//! is where the `b` flag's "no trailing newline", the
//! embedded-NUL-becomes-NL convention, CRLF stripping, BOM removal and a
//! maximum line count that may be counted from the end of the file all live.
//! Three of those can straddle two reads, so the walk carries the tail of an
//! unfinished line in a [`Carry`] -- upstream's `prev`/`prevlen`/`prevsize` --
//! and every position in the buffer is a signed index rather than a pointer,
//! because closing the gap a BOM leaves behind steps one *before* the front.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{__S_IFMT, Args, READBIN, SEEK_END, SEEK_SET, frame, no_fileinfo, str_arg};
use crate::eval::typval::{
    tv_blob_alloc_ret, tv_blob_free, tv_get_number, tv_list_alloc_ret, tv_list_append_owned_tv,
    tv_list_first, tv_list_item_remove, tv_list_len,
};
use crate::garray::ga_grow;
use crate::main::{e_cant_read_file_str, e_isadir2, e_notopen};
use crate::memory::{xfree, xmemdupz, xrealloc};
use crate::os::fs::{os_fileinfo_fd, os_fileinfo_size, os_fopen, os_isdir};
use crate::os::libc::{fclose, fileno, fread, fseeko, gettext, memcpy, memmove};
use crate::pos::MAXLNUM;
use crate::semsg_c;
use crate::types::{
    __off_t, EvalFuncData, FILE, FileInfo, VAR_STRING, VAR_UNLOCKED, blob_T, int64_t,
    kListLenUnknown, list_T, off_T, ptrdiff_t, size_t, typval_T, typval_vval_union, uint64_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

// ---------------------------------------------------------------------
// The handles
// ---------------------------------------------------------------------

/// An open stream, closed when it goes out of scope.
struct File(*mut FILE);

impl Drop for File {
    fn drop(&mut self) {
        // SAFETY: opened by [`File::open`], which is the only constructor,
        // and closed exactly once.
        unsafe { fclose(self.0) };
    }
}

impl File {
    /// Open `fname` for reading, or None when it cannot be opened.
    ///
    /// Always in binary mode: the library functions have a mind of their own
    /// about CR-LF conversion.
    fn open(fname: &CStr) -> Option<Self> {
        // SAFETY: both arguments are NUL-terminated.
        let fd = unsafe { os_fopen(fname.as_ptr(), READBIN.as_ptr()) };
        // `then`, not `then_some`: the latter would build -- and drop, and
        // so `fclose` -- a `File` around the null.
        (!fd.is_null()).then(|| Self(fd))
    }

    /// The `stat` of the open file, or None when it cannot be taken.
    fn info(&self) -> Option<FileInfo> {
        let mut info = no_fileinfo();
        // SAFETY: a live stream, and a `FileInfo` for the callee to fill.
        let taken = unsafe { os_fileinfo_fd(fileno(self.0), &raw mut info) };
        taken.then_some(info)
    }

    /// Seek to `offset` relative to `whence`; false when the seek failed.
    fn seek(&self, offset: off_T, whence: c_int) -> bool {
        // SAFETY: a live stream.
        unsafe { fseeko(self.0, offset as __off_t, whence) == 0 }
    }

    /// Fill `buf` from the stream, answering how many bytes arrived.
    fn read(&self, buf: &mut [c_char]) -> usize {
        let (p, len) = (buf.as_mut_ptr().cast::<c_void>(), buf.len() as size_t);
        // SAFETY: `p` is writable for `len` bytes, which is what an element
        // size of one and a count of `len` ask for.
        unsafe { fread(p, 1 as size_t, len, self.0) as usize }
    }

    /// Read `len` bytes into `p`; false on a short read.
    ///
    /// # Safety
    /// `p` is writable for `len` bytes.
    unsafe fn read_into(&self, p: *mut c_void, len: usize) -> bool {
        // SAFETY: the caller's contract.
        unsafe { fread(p, 1 as size_t, len as size_t, self.0) as usize >= len }
    }
}

/// The Blob `readblob()` is filling.
#[derive(Clone, Copy)]
struct Blob(*mut blob_T);

impl Blob {
    /// Make `rettv` a fresh, empty Blob.
    fn alloc(rettv: &mut typval_T) -> Self {
        // SAFETY: `rettv` is the builtin's own cleared result slot.
        Self(unsafe { tv_blob_alloc_ret(rettv) })
    }

    /// Grow to `len` bytes and fill them from `fd`; false on a short read.
    ///
    /// The read is asked for the count *as the garray holds it*: upstream
    /// stores the length in `ga_len`, an `int`, and then reads it back, so a
    /// size that does not fit in one asks `fread` for a nonsense count
    /// against a buffer `ga_grow` never allocated -- an inherited overrun,
    /// reproduced rather than fixed, which is what the round trip through
    /// `c_int` below is.
    fn fill(self, fd: &File, len: usize) -> bool {
        let want = len as c_int as size_t;
        // SAFETY: a live blob; `ga_grow` makes room for `len` items past the
        // `ga_len` of zero a fresh blob has, so `ga_data` is writable for as
        // many bytes as `want` asks for -- whenever `len` fits in an `int`.
        unsafe {
            ga_grow(&raw mut (*self.0).bv_ga, len as c_int);
            (*self.0).bv_ga.ga_len = len as c_int;
            fd.read_into((*self.0).bv_ga.ga_data, want)
        }
    }

    /// Give the Blob back, which is what an error answers instead.
    fn free(self) {
        // SAFETY: a live blob nothing else refers to yet.
        unsafe { tv_blob_free(self.0) };
    }
}

/// The List `readfile()` is filling.
#[derive(Clone, Copy)]
struct Lines(*mut list_T);

impl Lines {
    /// Make `rettv` a fresh List whose length is not known in advance.
    fn alloc(rettv: &mut typval_T) -> Self {
        let unknown = kListLenUnknown as c_int as ptrdiff_t;
        // SAFETY: `rettv` is the builtin's own cleared result slot.
        Self(unsafe { tv_list_alloc_ret(rettv, unknown) })
    }

    fn len(self) -> int64_t {
        // SAFETY: a live list.
        unsafe { tv_list_len(self.0) as int64_t }
    }

    /// Append `s`, a NUL-terminated string in nvim's heap that the list owns
    /// from here on.
    fn push(self, s: *mut c_char) {
        let tv = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_string: s },
        };
        // SAFETY: a live list, and `tv` an owned String the list takes over.
        unsafe { tv_list_append_owned_tv(self.0, tv) };
    }

    /// Drop the oldest line, which is how a negative `{max}` keeps only the
    /// last few.
    fn drop_first(self) {
        // SAFETY: a live list, reached only with at least one item in it.
        unsafe { tv_list_item_remove(self.0, tv_list_first(self.0)) };
    }
}

/// The bytes of a line that a read ended in the middle of.
///
/// Upstream's `prev`/`prevlen`/`prevsize` triple: a heap buffer, grown by
/// halves, read back when a CR run or a BOM straddles two reads, and finally
/// handed to the list as the head of the finished line.
struct Carry {
    buf: *mut c_char,
    len: isize,
    size: isize,
}

impl Drop for Carry {
    fn drop(&mut self) {
        // SAFETY: null, or nvim's own; [`Carry::take`] is the only other way
        // for the buffer to leave.
        unsafe { xfree(self.buf.cast::<c_void>()) };
    }
}

impl Carry {
    const fn new() -> Self {
        Self {
            buf: ptr::null_mut(),
            len: 0,
            size: 0,
        }
    }

    /// Byte `i` of the carried bytes.
    fn byte(&self, i: isize) -> u8 {
        debug_assert!(i >= 0 && i < self.len);
        // SAFETY: `i` indexes the bytes already written into the buffer.
        unsafe { *self.buf.offset(i) as u8 }
    }

    /// Drop the trailing CRs, which is what a CRLF split across two reads
    /// leaves behind.
    fn trim_cr(&mut self) {
        while self.len > 0 && self.byte(self.len - 1) == b'\r' {
            self.len -= 1;
        }
    }

    /// Append `bytes`.
    fn push(&mut self, bytes: &[c_char]) {
        let n = bytes.len() as isize;
        if n + self.len >= self.size {
            // A common use case is an ordinary text file, where the carry is
            // a fragment of a line: the first allocation is made small, to
            // avoid repeatedly allocating large and reallocating small.
            self.size = if self.size == 0 {
                n
            } else {
                (self.size * 3 / 2).max(n * 2 + self.len)
            };
            let size = self.size as size_t;
            // SAFETY: the pointer is null or nvim's own, and the new size
            // covers the bytes already written as well as `bytes`.
            self.buf = unsafe { xrealloc(self.buf.cast::<c_void>(), size).cast::<c_char>() };
        }
        let (dst, src) = (self.buf.wrapping_offset(self.len), bytes.as_ptr());
        // SAFETY: `len + n` bytes fit, by the growth above, and `bytes` is a
        // live slice of that length.
        unsafe { memmove(dst.cast::<c_void>(), src.cast::<c_void>(), n as size_t) };
        self.len += n;
    }

    /// Give the carry up as the head of a finished line, with `tail` and a
    /// terminator after it.
    ///
    /// Resizing rather than allocating afresh is what copies the bytes only
    /// once, so that a very long line is allocated only once too.
    fn take(&mut self, tail: &[c_char]) -> *mut c_char {
        let (len, n) = (self.len as usize, tail.len());
        // SAFETY: the pointer is nvim's own, and the new size covers the
        // bytes already written, `tail`, and the terminator after them.
        let s = unsafe {
            let s = xrealloc(self.buf.cast::<c_void>(), (len + n + 1) as size_t).cast::<c_char>();
            memcpy(
                s.add(len).cast::<c_void>(),
                tail.as_ptr().cast::<c_void>(),
                n,
            );
            *s.add(len + n) = 0;
            s
        };
        // Field by field: assigning through `self` would drop the buffer
        // that has just been handed out.
        self.buf = ptr::null_mut();
        self.len = 0;
        self.size = 0;
        s
    }
}

// ---------------------------------------------------------------------
// The two fillers
// ---------------------------------------------------------------------

/// `readblob()`'s body: `size_arg` bytes from `offset` into the Blob `rettv`
/// holds, where a negative offset counts back from the end of the file and a
/// size of -1 asks for everything from `offset` on.
///
/// False -- upstream's `FAIL` -- when the file could not be measured or the
/// read came up short; the Blob is then given back and `rettv` left empty.
fn read_blob(fd: &File, rettv: &mut typval_T, blob: Blob, offset: off_T, size_arg: off_T) -> bool {
    let Some(info) = fd.info() else {
        // Can't read the file, error.
        return false;
    };
    // SAFETY: a `FileInfo` this frame owns.
    let file_size = unsafe { os_fileinfo_size(&raw const info) } as off_T;
    // `S_ISCHR`: a character device, whose size a `stat` does not answer,
    // which is why the two clamps below skip it.
    const S_IFCHR: uint64_t = 0o20000;
    let chardev = info.stat.st_mode & __S_IFMT as uint64_t == S_IFCHR;

    let mut offset = offset;
    let mut size = size_arg;
    let whence = if offset >= 0 {
        // The size defaults to the whole file.  If a size is given it is
        // limited to not go past the end -- and may become negative, which
        // is what the test below catches.
        if size == -1 || (size > file_size - offset && !chardev) {
            size = file_size - offset;
        }
        SEEK_SET
    } else {
        // Limit the offset to not go before the start of the file.
        if -offset > file_size && !chardev {
            offset = -file_size;
        }
        // The size defaults to reading until the end of the file.
        if size == -1 || size > -offset {
            size = -offset;
        }
        SEEK_END
    };
    if size <= 0 {
        return true;
    }
    if offset != 0 && !fd.seek(offset, whence) {
        return true;
    }
    if blob.fill(fd, size as usize) {
        return true;
    }
    // An empty blob is returned on error.
    blob.free();
    rettv.vval.v_blob = ptr::null_mut();
    false
}

/// `readfile()`'s body: the file split into lines, at most `maxline` of them
/// -- kept from the end of the file when that is negative.
fn read_lines(fd: &File, lines: Lines, binary: bool, maxline: int64_t) {
    // `IOSIZE` rounded down to a multiple of 256, to avoid the odd + 1.
    let mut buf = [0 as c_char; (1025 / 256) * 256];
    let mut carry = Carry::new();

    while maxline < 0 || lines.len() < maxline {
        let mut readlen = fd.read(&mut buf) as isize;
        let (mut p, mut start) = (0_isize, 0_isize);

        // This loop processes what was read, but is also entered at end of
        // file so that either an incomplete line gets written, or a "binary"
        // file gets an empty line at the end if it ends in a newline.
        while p < readlen || (readlen <= 0 && (carry.len > 0 || binary)) {
            if readlen <= 0 || buf[p as usize] == b'\n' as c_char {
                // Finished a line.  Remove the CRs before the NL.
                let mut len = (p - start) as usize;
                if readlen > 0 && !binary {
                    while len > 0 && buf[start as usize + len - 1] == b'\r' as c_char {
                        len -= 1;
                    }
                    // The removal may cross back into the carry.
                    if len == 0 {
                        carry.trim_cr();
                    }
                }
                let line = &buf[start as usize..start as usize + len];
                lines.push(if carry.len == 0 {
                    dupz(line)
                } else {
                    carry.take(line)
                });

                start = p + 1; // Step over the newline.
                if maxline < 0 {
                    if lines.len() > -maxline {
                        debug_assert!(
                            lines.len() == 1 + -maxline,
                            "tv_list_len(l) == 1 + -maxline"
                        );
                        lines.drop_first();
                    }
                } else if lines.len() >= maxline {
                    debug_assert!(lines.len() == maxline, "tv_list_len(l) == maxline");
                    break;
                }
                if readlen <= 0 {
                    break;
                }
            } else if buf[p as usize] == 0 {
                buf[p as usize] = b'\n' as c_char;
            } else if buf[p as usize] as u8 == 0xbf && !binary {
                // Check for a UTF-8 "bom"; U+FEFF is encoded as EF BB BF.
                // This is done on finding the BF, by looking at the two
                // bytes before it -- which, when `p` is at the front of the
                // buffer or just after it, may be in the carry.
                let back1 = if p >= 1 {
                    buf[(p - 1) as usize] as u8
                } else if carry.len >= 1 {
                    carry.byte(carry.len - 1)
                } else {
                    0
                };
                let back2 = if p >= 2 {
                    buf[(p - 2) as usize] as u8
                } else if p == 1 && carry.len >= 1 {
                    carry.byte(carry.len - 1)
                } else if carry.len >= 2 {
                    carry.byte(carry.len - 2)
                } else {
                    0
                };
                if back2 == 0xef && back1 == 0xbb {
                    let mut dest = p - 2;
                    // Usually a BOM is at the beginning of a file, and so at
                    // the beginning of a line; then it can just be stepped
                    // over.
                    if start == dest {
                        start = p + 1;
                    } else {
                        // Otherwise the buffer has to be shuffled to close
                        // the gap.
                        let mut adjust_carry = 0;
                        if dest < 0 {
                            // Which is 1 or 2 bytes back into the carry.
                            adjust_carry = -dest;
                            dest = 0;
                        }
                        if readlen > p + 1 {
                            buf.copy_within((p + 1) as usize..readlen as usize, dest as usize);
                        }
                        readlen -= 3 - adjust_carry;
                        carry.len -= adjust_carry;
                        p = dest - 1;
                    }
                }
            }
            p += 1;
        }

        if (maxline >= 0 && lines.len() >= maxline) || readlen <= 0 {
            break;
        }
        if start < p {
            // There is part of a line in the buffer: carry it over.
            carry.push(&buf[start as usize..p as usize]);
        }
    }
}

/// A fresh NUL-terminated copy of `line`.
fn dupz(line: &[c_char]) -> *mut c_char {
    debug_assert!(line.len() < c_int::MAX as usize, "len < INT_MAX");
    let (p, len) = (line.as_ptr().cast::<c_void>(), line.len());
    // SAFETY: `p` is readable for `len` bytes, which is what a live slice of
    // that length promises.
    unsafe { xmemdupz(p, len).cast::<c_char>() }
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// Report the one-`%s` message `fmt`, translated, about the path `p`.
fn err_path(fmt: &[c_char], p: *const c_char) {
    // SAFETY: `fmt` is a NUL-terminated format taking one string, and `p` is
    // a NUL-terminated string.
    unsafe { semsg_c!(gettext(fmt.as_ptr()), p) };
}

/// Argument `i` as a Number, which is how `readblob()` reads its offset and
/// size and `readfile()` its maximum line count.
fn nr(args: Args<'_>, i: usize) -> int64_t {
    // SAFETY: a live typval; `tv_get_number` reports its own error and reads
    // as 0 for a type that has no number form.
    unsafe { tv_get_number(args.ptr(i)) }
}

/// The body both builtins share.
fn read_file_or_blob(args: Args<'_>, rettv: &mut typval_T, always_blob: bool) {
    let mut binary = false;
    let mut blob = always_blob;
    let mut maxline = MAXLNUM as c_int as int64_t;
    let mut offset: off_T = 0;
    let mut size: off_T = -1;

    if args.has(1) {
        if always_blob {
            offset = nr(args, 1) as off_T;
            if args.has(2) {
                size = nr(args, 2) as off_T;
            }
        } else {
            // The flag is coerced once per comparison, as upstream does, so
            // a type with no string form reports its error twice.
            if str_arg(args, 1).to_bytes() == b"b" {
                binary = true;
            } else if str_arg(args, 1).to_bytes() == b"B" {
                blob = true;
            }
            if args.has(2) {
                maxline = nr(args, 2);
            }
        }
    }

    let filling = if blob {
        Ok(Blob::alloc(rettv))
    } else {
        Err(Lines::alloc(rettv))
    };

    let fname = str_arg(args, 0);
    // SAFETY: `fname` is NUL-terminated.
    if unsafe { os_isdir(fname.as_ptr()) } {
        err_path(&e_isadir2, fname.as_ptr());
        return;
    }
    let empty = fname.to_bytes().is_empty();
    let Some(fd) = (if empty { None } else { File::open(fname) }) else {
        // SAFETY: a NUL-terminated literal, which is all `gettext` reads.
        let what = unsafe { gettext(c"<empty>".as_ptr()) };
        err_path(&e_notopen, if empty { what } else { fname.as_ptr() });
        return;
    };

    match filling {
        Ok(blob) => {
            if !read_blob(&fd, rettv, blob, offset, size) {
                err_path(&e_cant_read_file_str, fname.as_ptr());
            }
        }
        Err(lines) => read_lines(&fd, lines, binary, maxline),
    }
}

/// `readblob({fname} [, {offset} [, {size}]])`: the file's bytes as a Blob.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 1..3, and `rettv`
/// a cleared result.
pub unsafe fn f_readblob(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    read_file_or_blob(args, rettv, true);
}

/// `readfile({fname} [, {type} [, {max}]])`: the file's lines as a List.
///
/// # Safety
/// As [`f_readblob`].
pub unsafe fn f_readfile(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    read_file_or_blob(args, rettv, false);
}
