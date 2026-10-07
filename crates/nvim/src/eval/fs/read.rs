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

#![forbid(unsafe_code)]

use super::{__S_IFMT, str_arg};
use crate::eval::typval::NumBuf;
use crate::eval::typval::{tv_blob_alloc_ret, tv_get_number, tv_list_alloc_ret};
use crate::memory::ThinCString;
use crate::message::{e_cant_read_file_str, e_isadir2, e_notopen};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::os::fs::os_isdir_of;
use crate::pos::MAXLNUM;
use crate::tr_c;
use crate::types::{
    EvalFuncData, FileOffset, List, TypVal, int64_t, kListLenUnknown, ptrdiff_t, uint64_t,
};
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;

// ---------------------------------------------------------------------
// The handles
// ---------------------------------------------------------------------

/// An open file, read the way upstream's binary-mode `FILE *` reads it.
struct File(std::fs::File);

/// Where a [`File::seek`] counts from: upstream's `fseeko` `whence`.
#[derive(Clone, Copy)]
enum Whence {
    Start,
    End,
}

impl File {
    /// Open `fname` for reading, or None when it cannot be opened.
    fn open(fname: &CStr) -> Option<Self> {
        std::fs::File::open(OsStr::from_bytes(fname.to_bytes()))
            .ok()
            .map(Self)
    }

    /// The `stat` of the open file: its size and its mode, or None when it
    /// cannot be taken.
    fn info(&self) -> Option<(FileOffset, uint64_t)> {
        let info = self.0.metadata().ok()?;
        Some((info.size() as FileOffset, uint64_t::from(info.mode())))
    }

    /// Seek to `offset` relative to `whence`; false when the seek failed.
    fn seek(&mut self, offset: FileOffset, whence: Whence) -> bool {
        let to = match whence {
            Whence::Start => SeekFrom::Start(offset as u64),
            Whence::End => SeekFrom::End(offset),
        };
        self.0.seek(to).is_ok()
    }

    /// Fill `buf` from the file, answering how many bytes arrived: fewer
    /// only at the end of the file or on an error, as `fread` answers.
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut filled = 0;
        while filled < buf.len() {
            match self.0.read(&mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        filled
    }
}

/// How many lines the list holds so far.
fn count(lines: &List) -> int64_t {
    lines.len() as int64_t
}

/// Append `s` to the list, which owns it from here on.
fn push_line(lines: &mut List, s: ThinCString) {
    lines.push(TypVal::string(Some(s)));
}

/// The bytes of a line that a read ended in the middle of.
///
/// Upstream's `prev`/`prevlen`/`prevsize` triple, read back when a CR run
/// or a BOM straddles two reads, and finally handed to the list as the head
/// of the finished line.
struct Carry(Vec<u8>);

impl Carry {
    const fn new() -> Self {
        Self(Vec::new())
    }

    fn len(&self) -> isize {
        self.0.len() as isize
    }

    /// Byte `i` of the carried bytes.
    fn byte(&self, i: isize) -> u8 {
        self.0[i as usize]
    }

    /// Drop the last `n` bytes, which a BOM straddling two reads leaves
    /// behind.
    fn shorten(&mut self, n: isize) {
        self.0.truncate(self.0.len() - n as usize);
    }

    /// Drop the trailing CRs, which is what a CRLF split across two reads
    /// leaves behind.
    fn trim_cr(&mut self) {
        while self.0.last() == Some(&b'\r') {
            self.0.pop();
        }
    }

    /// Append `bytes`.
    fn push(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    /// Give the carry up as the head of a finished line, with `tail` after
    /// it.
    fn take(&mut self, tail: &[u8]) -> ThinCString {
        let mut line = core::mem::take(&mut self.0);
        line.extend_from_slice(tail);
        ThinCString::from_vec(line)
    }
}

// ---------------------------------------------------------------------
// The two fillers
// ---------------------------------------------------------------------

/// `readblob()`'s body: `size_arg` bytes from `offset` into the Blob `result`
/// holds, where a negative offset counts back from the end of the file and a
/// size of -1 asks for everything from `offset` on.
///
/// False -- upstream's `FAIL` -- when the file could not be measured or the
/// read came up short; the Blob is then given back and `result` left empty.
fn read_blob(fd: &mut File, result: &mut TypVal, offset: FileOffset, size_arg: FileOffset) -> bool {
    let Some((file_size, mode)) = fd.info() else {
        // Can't read the file, error.
        return false;
    };
    // `S_ISCHR`: a character device, whose size a `stat` does not answer,
    // which is why the two clamps below skip it.
    const S_IFCHR: uint64_t = 0o20000;
    let chardev = mode & __S_IFMT as uint64_t == S_IFCHR;

    let mut offset = offset;
    let mut size = size_arg;
    let whence = if offset >= 0 {
        // The size defaults to the whole file.  If a size is given it is
        // limited to not go past the end -- and may become negative, which
        // is what the test below catches.
        if size == -1 || (size > file_size - offset && !chardev) {
            size = file_size - offset;
        }
        Whence::Start
    } else {
        // Limit the offset to not go before the start of the file.
        if -offset > file_size && !chardev {
            offset = -file_size;
        }
        // The size defaults to reading until the end of the file.
        if size == -1 || size > -offset {
            size = -offset;
        }
        Whence::End
    };
    if size <= 0 {
        return true;
    }
    if offset != 0 && !fd.seek(offset, whence) {
        return true;
    }
    // Upstream stored the length in `ga_len`, an `int`, and read it back,
    // so a size past `INT_MAX` asked `fread` for a nonsense count against a
    // buffer `ga_grow` never allocated. The blob is a `Vec` now and is asked
    // for exactly `size`.
    let len = size as usize;
    let filled = result
        .blob_mut()
        .is_some_and(|blob| fd.read(blob.claim(len)) >= len);
    if filled {
        return true;
    }
    // An empty blob is returned on error: the slot gives up the one it
    // holds, which frees it.
    result.write_blob(None);
    false
}

/// `readfile()`'s body: the file split into lines, at most `maxline` of them
/// -- kept from the end of the file when that is negative.
fn read_lines(fd: &mut File, lines: &mut List, binary: bool, maxline: int64_t) {
    // `IOSIZE` rounded down to a multiple of 256, to avoid the odd + 1.
    let mut buf = [0_u8; (1025 / 256) * 256];
    let mut carry = Carry::new();

    while maxline < 0 || count(lines) < maxline {
        let mut readlen = fd.read(&mut buf) as isize;
        let (mut p, mut start) = (0_isize, 0_isize);

        // This loop processes what was read, but is also entered at end of
        // file so that either an incomplete line gets written, or a "binary"
        // file gets an empty line at the end if it ends in a newline.
        while p < readlen || (readlen <= 0 && (carry.len() > 0 || binary)) {
            if readlen <= 0 || buf[p as usize] == b'\n' {
                // Finished a line.  Remove the CRs before the NL.
                let mut len = (p - start) as usize;
                if readlen > 0 && !binary {
                    while len > 0 && buf[start as usize + len - 1] == b'\r' {
                        len -= 1;
                    }
                    // The removal may cross back into the carry.
                    if len == 0 {
                        carry.trim_cr();
                    }
                }
                let line = &buf[start as usize..start as usize + len];
                push_line(
                    lines,
                    if carry.len() == 0 {
                        dupz(line)
                    } else {
                        carry.take(line)
                    },
                );

                start = p + 1; // Step over the newline.
                if maxline < 0 {
                    if count(lines) > -maxline {
                        debug_assert!(count(lines) == 1 + -maxline, "list_len(l) == 1 + -maxline");
                        drop(lines.take_range(0, 0));
                    }
                } else if count(lines) >= maxline {
                    debug_assert!(count(lines) == maxline, "list_len(l) == maxline");
                    break;
                }
                if readlen <= 0 {
                    break;
                }
            } else if buf[p as usize] == 0 {
                buf[p as usize] = b'\n';
            } else if buf[p as usize] == 0xbf && !binary {
                // Check for a UTF-8 "bom"; U+FEFF is encoded as EF BB BF.
                // This is done on finding the BF, by looking at the two
                // bytes before it -- which, when `p` is at the front of the
                // buffer or just after it, may be in the carry.
                let back1 = if p >= 1 {
                    buf[(p - 1) as usize]
                } else if carry.len() >= 1 {
                    carry.byte(carry.len() - 1)
                } else {
                    0
                };
                let back2 = if p >= 2 {
                    buf[(p - 2) as usize]
                } else if p == 1 && carry.len() >= 1 {
                    carry.byte(carry.len() - 1)
                } else if carry.len() >= 2 {
                    carry.byte(carry.len() - 2)
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
                        carry.shorten(adjust_carry);
                        p = dest - 1;
                    }
                }
            }
            p += 1;
        }

        if (maxline >= 0 && count(lines) >= maxline) || readlen <= 0 {
            break;
        }
        if start < p {
            // There is part of a line in the buffer: carry it over.
            carry.push(&buf[start as usize..p as usize]);
        }
    }
}

/// A fresh NUL-terminated copy of `line`.
fn dupz(line: &[u8]) -> ThinCString {
    debug_assert!(line.len() < c_int::MAX as usize, "len < INT_MAX");
    ThinCString::from_bytes(line)
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// Report the one-`%s` message `fmt`, translated, about the path `p`.
fn err_path(fmt: &'static CStr, p: &CStr) {
    emsg_text(tr_c!(fmt, msg_cstr(p)));
}

/// Argument `i` as a Number, which is how `readblob()` reads its offset and
/// size and `readfile()` its maximum line count.
fn nr(args: &[TypVal], i: usize) -> int64_t {
    tv_get_number(&args[i])
}

/// The body both builtins share.
fn read_file_or_blob(args: &[TypVal], result: &mut TypVal, always_blob: bool) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut numbuf3 = NumBuf::new();
    let mut binary = false;
    let mut blob = always_blob;
    let mut maxline = MAXLNUM as int64_t;
    let mut offset: FileOffset = 0;
    let mut size: FileOffset = -1;

    if args.len() > 1 {
        if always_blob {
            offset = nr(args, 1) as FileOffset;
            if args.len() > 2 {
                size = nr(args, 2) as FileOffset;
            }
        } else {
            // The flag is coerced once per comparison, as upstream does, so
            // a type with no string form reports its error twice.
            if str_arg(args, 1, &mut numbuf).to_bytes() == b"b" {
                binary = true;
            } else if str_arg(args, 1, &mut numbuf2).to_bytes() == b"B" {
                blob = true;
            }
            if args.len() > 2 {
                maxline = nr(args, 2);
            }
        }
    }

    if blob {
        tv_blob_alloc_ret(result);
    } else {
        tv_list_alloc_ret(result, kListLenUnknown as c_int as ptrdiff_t);
    }

    let fname = str_arg(args, 0, &mut numbuf3);
    if os_isdir_of(fname) {
        err_path(e_isadir2, fname);
        return;
    }
    let empty = fname.to_bytes().is_empty();
    let Some(mut fd) = (if empty { None } else { File::open(fname) }) else {
        err_path(e_notopen, if empty { gettext(c"<empty>") } else { fname });
        return;
    };

    if blob {
        if !read_blob(&mut fd, result, offset, size) {
            err_path(e_cant_read_file_str, fname);
        }
    } else if let Some(lines) = result.list_mut() {
        read_lines(&mut fd, lines, binary, maxline);
    }
}

/// `readblob({fname} [, {offset} [, {size}]])`: the file's bytes as a Blob.
pub fn f_readblob(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    read_file_or_blob(args, result, true);
}

/// `readfile({fname} [, {type} [, {max}]])`: the file's lines as a List.
pub fn f_readfile(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    read_file_or_blob(args, result, false);
}
