//! Turning a file's bytes into the buffer's UTF-8.
//!
//! `'fileencodings'` is a list of guesses; [`next_fenc`] walks it and
//! [`get_fio_flags`] turns each name into the `FIO_*` bits that say whether
//! Nvim can do the conversion itself. [`need_conversion`] is the cheap test
//! for "this is already UTF-8, leave it alone", [`check_for_bom`] recognises
//! a byte-order mark and lets it override the guess, and
//! [`readfile_charconvert`] hands the whole file to the user's
//! `'charconvert'` program when nothing else can read it.
//!
//! [`Conv`] carries what the conversion knows across reads — the flags, the
//! iconv descriptor, and the bytes of a character split across two reads —
//! and its three methods are the three ways bytes become UTF-8:
//! [`with_iconv`](Conv::with_iconv), [`units_to_utf8`](Conv::units_to_utf8)
//! for the encodings Nvim knows itself, and [`check_utf8`](Conv::check_utf8)
//! for when there is nothing to convert but the bytes still have to be valid.
//!
//! This is the read side of `bufwrite::convert`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::ffi::CStr;

use crate::bufwrite::translate;

use super::*;

/// The iconv descriptor value that means "no iconv".
pub(crate) fn no_iconv() -> iconv_t {
    core::ptr::with_exposed_provenance_mut::<c_void>(-1i32 as usize)
}

/// Find the next `'fileencodings'` entry to try.
///
/// `pp` is advanced past the entry it returns, and set to NULL once the list
/// is exhausted — which is reported as an empty name. `alloced` says whether
/// the result has to be freed.
pub(crate) unsafe fn next_fenc(pp: &mut *mut c_char, alloced: &mut bool) -> *mut c_char {
    unsafe {
        *alloced = false;
        if **pp == 0 {
            *pp = core::ptr::null_mut();
            return c"".as_ptr().cast_mut();
        }
        let comma = vim_strchr(*pp, b',' as c_int);
        let r = if comma.is_null() {
            let r = enc_canonize(*pp);
            *pp = (*pp).add(strlen(*pp));
            r
        } else {
            let one = xmemdupz(pp.cast(), comma.offset_from(*pp) as size_t).cast::<c_char>();
            *pp = comma.add(1);
            let r = enc_canonize(one);
            xfree(one.cast());
            r
        };
        *alloced = true;
        r
    }
}

/// Convert a file with the `'charconvert'` expression.
///
/// Closes the file being read, converts it, and opens the result for reading
/// instead. If anything fails, the original file is re-opened (the caller has
/// to check for an error) and NULL returned.
///
/// @return  the name of the converted file, which the caller deletes after
///          reading it.
pub(crate) unsafe fn readfile_charconvert(
    fname: *mut c_char,
    fenc: *mut c_char,
    fdp: &mut c_int,
) -> *mut c_char {
    unsafe {
        let mut errmsg: Option<&CStr> = None;
        let mut tmpname = vim_tempname();
        if tmpname.is_null() {
            errmsg = Some(translate(c"Can't find temp file for conversion"));
        } else {
            close(*fdp); // close the input file, ignore errors
            *fdp = -1;
            if eval_charconvert(fenc, c"utf-8".as_ptr(), fname, tmpname) == FAIL {
                errmsg = Some(translate(c"Conversion with 'charconvert' failed"));
            }
            if errmsg.is_none() {
                *fdp = os_open(tmpname, O_RDONLY, 0);
                if *fdp < 0 {
                    errmsg = Some(translate(c"can't read output of 'charconvert'"));
                }
            }
        }

        if let Some(errmsg) = errmsg {
            // Not emsg(): that breaks mappings, and the retry with another
            // type of conversion might still work.
            msg(errmsg.as_ptr(), 0);
            if !tmpname.is_null() {
                os_remove(tmpname); // delete converted file
                xfree(tmpname.cast());
                tmpname = core::ptr::null_mut();
            }
        }

        // If the input file is closed, open it; the caller checks for an
        // error.
        if *fdp < 0 {
            *fdp = os_open(fname, O_RDONLY, 0);
        }
        tmpname
    }
}

/// Does file encoding `fenc` need converting from or to `'encoding'`?
pub unsafe extern "C" fn need_conversion(fenc: *const c_char) -> bool {
    unsafe {
        let fenc_flags;
        let same_encoding = if *fenc == 0 || strcmp(p_enc.get(), fenc) == 0 {
            fenc_flags = 0;
            true
        } else {
            // Ignore the difference between "ansi" and "latin1", "ucs-4" and
            // "ucs-4be", and so on.
            let enc_flags = get_fio_flags(p_enc.get());
            fenc_flags = get_fio_flags(fenc);
            enc_flags != 0 && fenc_flags == enc_flags
        };
        if same_encoding {
            // The file encoding matches 'encoding'.
            return false;
        }
        // The encodings differ. Conversion is still not needed when
        // 'encoding' is any Unicode encoding and the file is UTF-8.
        fenc_flags != FIO_UTF8
    }
}

/// The `FIO_*` flags for converting `name` internally, or 0 when only iconv
/// can do it. An empty name means `'encoding'`.
pub unsafe extern "C" fn get_fio_flags(name: *const c_char) -> c_int {
    unsafe {
        let name = if *name == 0 { p_enc.get() } else { name };
        let prop = enc_canon_props(name);
        let little = if prop & ENC_ENDIAN_L as c_int != 0 {
            FIO_ENDIAN_L
        } else {
            0
        };
        if prop & ENC_UNICODE as c_int != 0 {
            if prop & ENC_2BYTE as c_int != 0 {
                return FIO_UCS2 | little;
            }
            if prop & ENC_4BYTE as c_int != 0 {
                return FIO_UCS4 | little;
            }
            if prop & ENC_2WORD as c_int != 0 {
                return FIO_UTF16 | little;
            }
            return FIO_UTF8;
        }
        if prop & ENC_LATIN1 as c_int != 0 {
            return FIO_LATIN1;
        }
        // Must be ENC_DBCS, which requires iconv().
        0
    }
}

/// Check for a Unicode byte-order mark at the start of `bytes`.
///
/// `flags` is what the encoding guess allows; `FIO_ALL` accepts any BOM.
///
/// @return  the encoding the BOM names and how many bytes it takes, or None.
pub(crate) fn check_for_bom(bytes: &[u8], flags: c_int) -> Option<(&'static CStr, usize)> {
    let at = |i: usize| bytes.get(i).copied().unwrap_or(0);
    let size = bytes.len();

    if at(0) == 0xef
        && at(1) == 0xbb
        && size >= 3
        && at(2) == 0xbf
        && (flags == FIO_ALL || flags == FIO_UTF8 || flags == 0)
    {
        return Some((c"utf-8", 3)); // EF BB BF
    }
    if at(0) == 0xff && at(1) == 0xfe {
        if size >= 4
            && at(2) == 0
            && at(3) == 0
            && (flags == FIO_ALL || flags == FIO_UCS4 | FIO_ENDIAN_L)
        {
            return Some((c"ucs-4le", 4)); // FF FE 00 00
        }
        if flags == FIO_UCS2 | FIO_ENDIAN_L {
            return Some((c"ucs-2le", 2)); // FF FE
        }
        if flags == FIO_ALL || flags == FIO_UTF16 | FIO_ENDIAN_L {
            // utf-16le is preferred, it also works for ucs-2le text.
            return Some((c"utf-16le", 2)); // FF FE
        }
        return None;
    }
    if at(0) == 0xfe
        && at(1) == 0xff
        && (flags == FIO_ALL || flags == FIO_UCS2 || flags == FIO_UTF16)
    {
        // Default to utf-16, it works for ucs-2 text as well.
        return Some((
            if flags == FIO_UCS2 {
                c"ucs-2"
            } else {
                c"utf-16"
            },
            2,
        )); // FE FF
    }
    if size >= 4
        && at(0) == 0
        && at(1) == 0
        && at(2) == 0xfe
        && at(3) == 0xff
        && (flags == FIO_ALL || flags == FIO_UCS4)
    {
        return Some((c"ucs-4", 4)); // 00 00 FE FF
    }
    None
}

/// Where the read buffer currently is, as the conversion steps see it.
///
/// `buffer` is the whole allocation. `ptr`/`size` are the bytes just read,
/// which conversion rewrites in place or moves within the allocation;
/// `linerest` bytes of the previous line sit at `buffer` and are moved to
/// just before the converted bytes, which is where `line_start` ends up.
pub(crate) struct Window {
    pub buffer: *mut c_char,
    pub ptr: *mut c_char,
    pub line_start: *mut c_char,
    pub size: ptrdiff_t,
    /// The allocation's size; conversion may grow the bytes into it.
    pub real_size: c_int,
    pub linerest: ptrdiff_t,
}

impl Window {
    /// Move the previous line's tail to just before `at`, and make that the
    /// start of the current line.
    unsafe fn move_linerest(&mut self, at: *mut c_char) {
        unsafe {
            self.line_start = at.offset(-self.linerest);
            core::ptr::copy(self.buffer, self.line_start, self.linerest as usize);
        }
    }
}

/// What the conversion knows, carried from one read to the next.
pub(crate) struct Conv {
    /// `FIO_*` bits for a conversion Nvim does itself, or 0.
    pub flags: c_int,
    /// The iconv descriptor, or [`no_iconv`]; owned.
    pub iconv: iconv_t,
    /// Bytes of a character split across two reads.
    rest: [c_char; CONV_RESTLEN as usize],
    pub restlen: c_int,
    /// `BAD_REPLACE`, `BAD_KEEP`, `BAD_DROP`, or the replacement character.
    pub bad_char: c_int,
    /// Line number of the first conversion error, or 0.
    pub conv_error: linenr_T,
    /// Buffer line count before this read, for turning an offset into a line
    /// number.
    pub linecnt: linenr_T,
    /// Is rewinding and trying another `'fileencoding'` still an option?
    pub can_retry: bool,
}

impl Drop for Conv {
    fn drop(&mut self) {
        self.close_iconv();
    }
}

impl Conv {
    pub(crate) fn new(bad_char: c_int) -> Self {
        Conv {
            flags: 0,
            iconv: no_iconv(),
            rest: [0; CONV_RESTLEN as usize],
            restlen: 0,
            bad_char,
            conv_error: 0,
            linecnt: 0,
            can_retry: false,
        }
    }

    pub(crate) fn has_iconv(&self) -> bool {
        self.iconv != no_iconv()
    }

    /// Close the iconv descriptor, if there is one.
    pub(crate) fn close_iconv(&mut self) {
        if self.has_iconv() {
            // SAFETY: the descriptor is ours and is open.
            unsafe { iconv_close(self.iconv) };
            self.iconv = no_iconv();
        }
    }

    /// Ask iconv to convert `fenc` to UTF-8. False when it cannot.
    pub(crate) unsafe fn open_iconv(&mut self, fenc: *mut c_char) -> bool {
        self.iconv = unsafe { my_iconv_open(c"utf-8".as_ptr().cast_mut(), fenc) };
        self.has_iconv()
    }

    /// Is anything being converted?
    pub(crate) fn active(&self) -> bool {
        self.flags != 0 || self.has_iconv()
    }

    /// Note a conversion error at the line `at` falls on, if none was noted
    /// yet.
    unsafe fn note_error(&mut self, start: *const c_char, at: *const c_char) {
        if self.conv_error == 0 {
            self.conv_error = unsafe { readfile_linenr(self.linecnt, start, at) };
        }
    }

    /// Keep `len` bytes at `from` for the next read.
    unsafe fn stash(&mut self, from: *const c_char, len: usize) {
        unsafe { core::ptr::copy(from, self.rest.as_mut_ptr(), len) };
        self.restlen = len as c_int;
    }

    /// Put the held-over bytes at `ptr`.
    ///
    /// `restlen` deliberately stays set: the bytes are laid down before the
    /// read so that the read appends to them, and only counted back in once
    /// the read is done.
    pub(crate) unsafe fn restore(&self, ptr: *mut c_char) {
        unsafe { core::ptr::copy(self.rest.as_ptr(), ptr, self.restlen as usize) };
    }

    /// Convert the bytes in `w` with iconv.
    ///
    /// False means the encoding is wrong and the file should be read again
    /// with the next one.
    pub(crate) unsafe fn with_iconv(&mut self, w: &mut Window) -> bool {
        unsafe {
            let mut fromp: *const c_char = w.ptr;
            let mut from_size = w.size as size_t;
            // The converted bytes go after the ones being converted.
            w.ptr = w.ptr.offset(w.size);
            let mut top = w.ptr;
            let mut to_size = (w.real_size as ptrdiff_t - w.size) as size_t;

            // On a conversion error, or when there is not enough room, try
            // another conversion — unless there is no alternative, as for
            // help files.
            while iconv(
                self.iconv,
                (&raw mut fromp).cast::<*mut c_char>(),
                &raw mut from_size,
                &raw mut top,
                &raw mut to_size,
            ) == -1i32 as size_t
                && *__errno_location() != ICONV_EINVAL
                || from_size > CONV_RESTLEN as size_t
            {
                if self.can_retry {
                    return false;
                }
                self.note_error(w.ptr, top);

                // Deal with a bad byte and continue with the next.
                fromp = fromp.add(1);
                from_size -= 1;
                if self.bad_char == BAD_KEEP {
                    *top = *fromp.offset(-1);
                    top = top.add(1);
                    to_size -= 1;
                } else if self.bad_char != BAD_DROP {
                    *top = self.bad_char as c_char;
                    top = top.add(1);
                    to_size -= 1;
                }
            }

            if from_size > 0 {
                // Some characters are left over; keep them for the next round.
                self.stash(fromp, from_size as usize);
            }

            w.move_linerest(w.ptr);
            w.size = top.offset_from(w.ptr);
            true
        }
    }

    /// Convert Unicode or Latin1 code units in `w` to UTF-8.
    ///
    /// Works from the end of the buffer towards the start, because the number
    /// of bytes may grow. False means "read the file again with the next
    /// encoding".
    pub(crate) unsafe fn units_to_utf8(&mut self, w: &mut Window) -> bool {
        unsafe {
            let flags = self.flags;
            let start: *const u8 = w.ptr.cast();
            // Where the UTF-8 bytes go, filling the allocation from its end.
            let mut dest = w.ptr.offset(w.real_size as isize);
            let mut p: *const u8 = start.offset(w.size);
            // An incomplete sequence at the end, to be kept for next time.
            let mut tail: *const u8 = core::ptr::null();

            if flags == FIO_LATIN1 || flags == FIO_UTF8 {
                if flags == FIO_UTF8 {
                    // Check for a trailing incomplete UTF-8 sequence.
                    let mut t = start.offset(w.size - 1);
                    while t > start && *t & 0xc0 == 0x80 {
                        t = t.offset(-1);
                    }
                    if t.add(utf_byte2len(*t as c_int) as usize) > start.offset(w.size) {
                        tail = t;
                        p = t;
                    }
                }
            } else if flags & (FIO_UCS2 | FIO_UTF16) != 0 {
                // Check for a trailing odd byte.
                p = start.offset(w.size & !1);
                if w.size & 1 != 0 {
                    tail = p;
                }
                if flags & FIO_UTF16 != 0 && p > start {
                    // Check for a trailing leading word.
                    let u8c = read_word(&mut p, flags);
                    if (0xd800..=0xdbff).contains(&u8c) {
                        tail = p;
                    } else {
                        p = p.add(2);
                    }
                }
            } else {
                // FIO_UCS4: check for 1, 2 or 3 trailing bytes.
                p = start.offset(w.size & !3);
                if w.size & 3 != 0 {
                    tail = p;
                }
            }

            if !tail.is_null() {
                let len = start.offset(w.size).offset_from(tail) as usize;
                self.stash(tail.cast(), len);
                w.size -= len as ptrdiff_t;
            }

            while p > start {
                let mut u8c;
                if flags & FIO_LATIN1 != 0 {
                    p = p.offset(-1);
                    u8c = *p as c_uint;
                } else if flags & (FIO_UCS2 | FIO_UTF16) != 0 {
                    u8c = read_word(&mut p, flags);
                    if flags & FIO_UTF16 != 0 && (0xdc00..=0xdfff).contains(&u8c) {
                        if p == start {
                            // Missing leading word.
                            if self.can_retry {
                                return false;
                            }
                            self.note_error(start.cast(), p.cast());
                            if self.bad_char == BAD_DROP {
                                continue;
                            }
                            if self.bad_char != BAD_KEEP {
                                u8c = self.bad_char as c_uint;
                            }
                        }
                        // The second word of a pair; take the first and put
                        // them together.
                        let u16c = read_word(&mut p, flags);
                        u8c = 0x10000 + ((u16c & 0x3ff) << 10) + (u8c & 0x3ff);

                        // Check that the first word really was a leading word.
                        if !(0xd800..=0xdbff).contains(&u16c) {
                            if self.can_retry {
                                return false;
                            }
                            self.note_error(start.cast(), p.cast());
                            if self.bad_char == BAD_DROP {
                                continue;
                            }
                            if self.bad_char != BAD_KEEP {
                                u8c = self.bad_char as c_uint;
                            }
                        }
                    }
                } else if flags & FIO_UCS4 != 0 {
                    let mut word = 0u32;
                    for shift in 0..4 {
                        p = p.offset(-1);
                        word |= (*p as u32)
                            << (if flags & FIO_ENDIAN_L != 0 {
                                24 - 8 * shift
                            } else {
                                8 * shift
                            });
                    }
                    u8c = word;
                    // Replace characters over INT_MAX with the Unicode
                    // replacement character.
                    if u8c > INT_MAX as c_uint {
                        u8c = 0xfffd;
                    }
                } else {
                    // UTF-8.
                    p = p.offset(-1);
                    if *p < 0x80 {
                        u8c = *p as c_uint;
                    } else {
                        let len = utf_head_off(start.cast(), p.cast());
                        p = p.offset(-(len as isize));
                        u8c = utf_ptr2char(p.cast()) as c_uint;
                        if len == 0 {
                            // Not a valid UTF-8 character: retry with another
                            // fenc when possible, otherwise report the error.
                            if self.can_retry {
                                return false;
                            }
                            self.note_error(start.cast(), p.cast());
                            if self.bad_char == BAD_DROP {
                                continue;
                            }
                            if self.bad_char != BAD_KEEP {
                                u8c = self.bad_char as c_uint;
                            }
                        }
                    }
                }
                debug_assert!(u8c <= INT_MAX as c_uint);
                dest = dest.offset(-(utf_char2len(u8c as c_int) as isize));
                utf_char2bytes(u8c as c_int, dest);
            }

            w.move_linerest(dest);
            w.size = w.ptr.offset(w.real_size as isize).offset_from(dest);
            w.ptr = dest;
            true
        }
    }

    /// Check that the bytes in `w` really are valid UTF-8, replacing,
    /// dropping or keeping the ones that are not.
    ///
    /// False means "read the file again with another conversion".
    pub(crate) unsafe fn check_utf8(
        &mut self,
        w: &mut Window,
        filesize: off_T,
        illegal_byte: &mut linenr_T,
    ) -> bool {
        unsafe {
            let start: *mut u8 = w.ptr.cast();
            let mut p = start;
            let mut incomplete_tail = false;
            loop {
                let end = start.offset(w.size);
                // Skip ASCII bytes quickly, a machine word at a time.
                while end.offset_from(p) >= size_of::<u64>() as isize {
                    let mut word = 0u64;
                    memcpy((&raw mut word).cast(), p.cast(), size_of::<u64>() as size_t);
                    if word & NONASCII_MASK != 0 {
                        break;
                    }
                    p = p.add(size_of::<u64>());
                }
                while p < end && *p < 0x80 {
                    p = p.add(1);
                }

                let todo = end.offset_from(p) as c_int;
                if todo <= 0 {
                    break;
                }
                if *p < 0x80 {
                    continue;
                }

                // A length of 1 means it is an illegal byte. An incomplete
                // character at the end is accepted, though: the next read
                // gets the rest of it and we check it then.
                let l = utf_ptr2len_len(p.cast(), todo);
                if l > todo && !incomplete_tail {
                    // Avoid retrying with a different encoding when a
                    // truncated file is more likely, or reading the rest of
                    // an incomplete sequence when we already did so.
                    if p > start || filesize > 0 {
                        incomplete_tail = true;
                    }
                    if p > start {
                        self.stash(p.cast(), todo as usize);
                        w.size -= todo as ptrdiff_t;
                        break;
                    }
                }
                if l != 1 && l <= todo {
                    p = p.offset(l as isize);
                    continue;
                }

                // An illegal byte. Try another encoding if we can, unless we
                // are at end of file, where a truncated file is more likely
                // than a conversion error.
                if self.can_retry && !incomplete_tail {
                    break;
                }
                // When we did a conversion, report an error.
                if self.has_iconv() {
                    self.note_error(start.cast(), p.cast());
                }
                // Remember the first line with an illegal byte.
                if self.conv_error == 0 && *illegal_byte == 0 {
                    *illegal_byte = readfile_linenr(self.linecnt, start.cast(), p.cast());
                }

                // Drop, keep or replace the bad byte.
                if self.bad_char == BAD_DROP {
                    core::ptr::copy(p.add(1), p, todo as usize - 1);
                    w.size -= 1;
                } else {
                    if self.bad_char != BAD_KEEP {
                        *p = self.bad_char as u8;
                    }
                    p = p.add(1);
                }
            }
            // A UTF-8 error was detected if we stopped short of the end.
            p >= start.offset(w.size) || incomplete_tail
        }
    }
}

/// Read one 16-bit code unit backwards from `p`, in the flags' byte order.
unsafe fn read_word(p: &mut *const u8, flags: c_int) -> c_uint {
    unsafe {
        *p = p.offset(-1);
        let first = **p as c_uint;
        *p = p.offset(-1);
        let second = **p as c_uint;
        if flags & FIO_ENDIAN_L != 0 {
            (first << 8) + second
        } else {
            first + (second << 8)
        }
    }
}

/// Which end-of-line marker the file uses, or `EOL_UNKNOWN` while guessing.
///
/// `try_dos`/`try_unix`/`try_mac` say which ones `'fileformats'` allows;
/// `try_mac` doubles as the count of carriage returns seen, so it is a
/// counter rather than a flag.
pub(crate) struct FormatGuess {
    pub try_dos: bool,
    pub try_unix: c_int,
    pub try_mac: c_int,
}

impl FormatGuess {
    pub(crate) unsafe fn from_ffs() -> Self {
        unsafe {
            FormatGuess {
                try_dos: !vim_strchr(p_ffs.get(), b'd' as c_int).is_null(),
                try_unix: !vim_strchr(p_ffs.get(), b'x' as c_int).is_null() as c_int,
                try_mac: !vim_strchr(p_ffs.get(), b'm' as c_int).is_null() as c_int,
            }
        }
    }

    /// Guess the end-of-line format from the first bytes of the file.
    pub(crate) unsafe fn guess(&mut self, ptr: *const c_char, size: ptrdiff_t) -> c_int {
        unsafe {
            let start: *const u8 = ptr.cast();
            let end = start.offset(size);
            let mut fileformat = EOL_UNKNOWN;

            // First try finding a NL, for Dos and Unix.
            if self.try_dos || self.try_unix != 0 {
                // Reset the carriage return counter.
                if self.try_mac != 0 {
                    self.try_mac = 1;
                }

                let mut p = start;
                while p < end {
                    if *p == NL as u8 {
                        fileformat = if self.try_unix == 0
                            || (self.try_dos && p > start && *p.offset(-1) == CAR as u8)
                        {
                            EOL_DOS
                        } else {
                            EOL_UNIX
                        };
                        break;
                    }
                    if *p == CAR as u8 && self.try_mac != 0 {
                        self.try_mac += 1;
                    }
                    p = p.add(1);
                }

                // Don't give in to EOL_UNIX if EOL_MAC is more likely.
                if fileformat == EOL_UNIX && self.try_mac != 0 {
                    // The counters have to be reset when retrying the fenc.
                    self.try_mac = 1;
                    self.try_unix = 1;
                    while p >= start && *p != CAR as u8 {
                        p = p.offset(-1);
                    }
                    if p >= start {
                        for p in core::slice::from_raw_parts(start, size as usize) {
                            if *p == NL as u8 {
                                self.try_unix += 1;
                            } else if *p == CAR as u8 {
                                self.try_mac += 1;
                            }
                        }
                        if self.try_mac > self.try_unix {
                            fileformat = EOL_MAC;
                        }
                    }
                } else if fileformat == EOL_UNKNOWN && self.try_mac == 1 {
                    // Looking for CR but found no end-of-line markers at all:
                    // use the default format.
                    fileformat = default_fileformat();
                }
            }

            // No NL found: may use Mac format.
            if fileformat == EOL_UNKNOWN && self.try_mac != 0 {
                fileformat = EOL_MAC;
            }
            // Still nothing found? Use the first format in 'fileformats'.
            if fileformat == EOL_UNKNOWN {
                fileformat = default_fileformat();
            }
            fileformat
        }
    }
}

/// Set up for reading the file again with another conversion.
pub(crate) fn rewind_retry(
    did_iconv: &mut bool,
    advance_fenc: &mut bool,
    file_rewind: &mut bool,
    had_iconv: bool,
) {
    // SAFETY: reading an option string pointer.
    if unsafe { *p_ccv.get() } != 0 && had_iconv {
        // iconv() failed; try 'charconvert'.
        *did_iconv = true;
    } else {
        // Use the next item from 'fileencodings'.
        *advance_fenc = true;
    }
    *file_rewind = true;
}
