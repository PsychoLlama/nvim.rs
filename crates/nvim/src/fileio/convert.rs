//! Turning a file's bytes into the buffer's UTF-8.
//!
//! `'fileencodings'` is a list of guesses; [`FencList`] walks it and
//! [`get_fio_flags`] turns each name into the [`FioFlags`] that say whether
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
//! All three work on a [`ReadBuf`], in place.
//!
//! This is the read side of `bufwrite::convert`.

#![forbid(unsafe_code)]

use std::fs::File;

use crate::bufwrite::translate;
use crate::cstr;
use crate::mbyte::{
    ENC_2BYTE, ENC_2WORD, ENC_4BYTE, ENC_ENDIAN_L, ENC_LATIN1, ENC_UNICODE, ICONV_EINVAL, char_at,
    encode_char, head_off, open_iconv, promised_char_len,
};
use crate::memory::XString;
use crate::option::vars::P_CCV;
use crate::os::iconv::Iconv;
use crate::strings::has_char;

use super::*;
use crate::types::FAIL;

crate::flag_set! {
    /// What Nvim knows how to convert itself, from an encoding's name:
    /// upstream's `FIO_*` bits. The empty set means "only iconv can".
    pub struct FioFlags;

    /// Latin-1, one byte per character.
    const LATIN1 = 0x01;
    const UTF8 = 0x02;
    const UCS2 = 0x04;
    const UCS4 = 0x08;
    const UTF16 = 0x10;
    /// Little-endian, for the UCS and UTF-16 ones.
    const ENDIAN_L = 0x80;
    /// "ucs-bom": the first bytes of the file decide.
    const UCSBOM = 0x4000;
    /// Every byte-order mark is acceptable, for `check_for_bom`.
    const ALL = -1;
}

/// How many bytes of a character split across two reads are carried over.
pub(crate) const CONV_RESTLEN: usize = 30;

/// How much room an iconv conversion is given: the read is cut to this
/// fraction of the buffer, because there is no knowing how much a character
/// grows.
pub(crate) const ICONV_MULT: isize = 8;

/// The guesses `'fileencodings'` lists, handed out one at a time.
///
/// A copy of the option, taken when the read starts, so that an autocommand
/// changing it mid-read does not move the walk.
pub(crate) struct FencList {
    list: Vec<u8>,
    /// Where the next entry starts, or `None` once the list is exhausted.
    at: Option<usize>,
}

impl FencList {
    pub(crate) fn new(list: &[u8]) -> Self {
        FencList {
            list: list.to_vec(),
            at: Some(0),
        }
    }

    /// An empty list: there is nothing to fall back to.
    pub(crate) fn none() -> Self {
        FencList {
            list: Vec::new(),
            at: None,
        }
    }

    /// The next entry, canonical. Once the list runs out this answers the
    /// empty name -- once -- and the list is exhausted after it.
    pub(crate) fn next(&mut self) -> XString {
        let Some(at) = self.at else {
            return XString::new();
        };
        let rest = &self.list[at..];
        if rest.is_empty() {
            self.at = None;
            return XString::new();
        }
        let (entry, next) = match cstr::find_byte(rest, b',') {
            Some(comma) => (&rest[..comma], at + comma + 1),
            None => (rest, self.list.len()),
        };
        let name = canonize(entry);
        self.at = Some(next);
        name
    }
}

/// Convert a file with the `'charconvert'` expression.
///
/// Closes the file being read (`input`), converts it, and opens the result
/// for reading instead. If anything fails, the original file is re-opened
/// (the caller has to check for an error) and `None` returned.
///
/// The expression is user code: it is handed copies of both names and runs
/// with nothing of the read borrowed.
///
/// @return  the name of the converted file, which the caller deletes after
///          reading it.
pub(crate) fn readfile_charconvert(
    fname: &CStr,
    fenc: &CStr,
    input: &mut Option<File>,
) -> Option<XString> {
    let mut errmsg: Option<&CStr> = None;
    let mut tmpname = temp_name();
    match &tmpname {
        None => errmsg = Some(translate(c"Can't find temp file for conversion")),
        Some(name) => {
            // Close the input file; errors are ignored.
            *input = None;
            if eval_charconvert(fenc, c"utf-8", fname, name.as_cstr()) == FAIL {
                errmsg = Some(translate(c"Conversion with 'charconvert' failed"));
            }
            if errmsg.is_none() {
                *input = open_for_read(name.as_cstr()).ok();
                if input.is_none() {
                    errmsg = Some(translate(c"can't read output of 'charconvert'"));
                }
            }
        }
    }

    if let Some(errmsg) = errmsg {
        // Not emsg(): that breaks mappings, and the retry with another type
        // of conversion might still work.
        msg(errmsg, 0);
        if let Some(name) = tmpname.take() {
            os_remove(name.as_cstr()); // delete the converted file
        }
    }

    // If the input file is closed, open it; the caller checks for an error.
    if input.is_none() {
        *input = open_for_read(fname).ok();
    }
    tmpname
}

/// Open `name` for reading, as `os_open(name, O_RDONLY, 0)` does: the file,
/// or the negative `errno` libuv would have answered.
pub(crate) fn open_for_read(name: &CStr) -> Result<File, c_int> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::OsStr::from_bytes(name.to_bytes());
    File::open(path).map_err(|err| -err.raw_os_error().unwrap_or(::libc::EIO))
}

/// Does file encoding `fenc` need converting from or to `'encoding'`?
pub fn need_conversion(fenc: &[u8]) -> bool {
    let fenc_flags;
    let same_encoding = if fenc.is_empty() || p_enc(|value| value.to_bytes() == fenc) {
        fenc_flags = FioFlags::NONE;
        true
    } else {
        // Ignore the difference between "ansi" and "latin1", "ucs-4" and
        // "ucs-4be", and so on.
        let enc_flags = p_enc(|value| get_fio_flags(value.to_bytes()));
        fenc_flags = get_fio_flags(fenc);
        !enc_flags.is_empty() && fenc_flags == enc_flags
    };
    if same_encoding {
        // The file encoding matches 'encoding'.
        return false;
    }
    // The encodings differ. Conversion is still not needed when 'encoding'
    // is any Unicode encoding and the file is UTF-8.
    fenc_flags != FioFlags::UTF8
}

/// The flags for converting `name` internally, or none when only iconv can
/// do it. An empty name means `'encoding'`.
pub fn get_fio_flags(name: &[u8]) -> FioFlags {
    let prop = if name.is_empty() {
        p_enc(|enc| canon_props(enc.to_bytes()))
    } else {
        canon_props(name)
    };
    let little = FioFlags::ENDIAN_L.when(prop & ENC_ENDIAN_L != 0);
    if prop & ENC_UNICODE != 0 {
        if prop & ENC_2BYTE != 0 {
            return FioFlags::UCS2 | little;
        }
        if prop & ENC_4BYTE != 0 {
            return FioFlags::UCS4 | little;
        }
        if prop & ENC_2WORD != 0 {
            return FioFlags::UTF16 | little;
        }
        return FioFlags::UTF8;
    }
    if prop & ENC_LATIN1 != 0 {
        return FioFlags::LATIN1;
    }
    // Must be ENC_DBCS, which requires iconv().
    FioFlags::NONE
}

/// Check for a Unicode byte-order mark at the start of `bytes`.
///
/// `flags` is what the encoding guess allows; [`FioFlags::ALL`] accepts any
/// BOM.
///
/// @return  the encoding the BOM names and how many bytes it takes, or None.
pub(crate) fn check_for_bom(bytes: &[u8], flags: FioFlags) -> Option<(&'static CStr, usize)> {
    let at = |i: usize| cstr::byte_at(bytes, i);
    let size = bytes.len();
    let all = flags == FioFlags::ALL;
    let le = FioFlags::ENDIAN_L;

    if at(0) == 0xef
        && at(1) == 0xbb
        && size >= 3
        && at(2) == 0xbf
        && (all || flags == FioFlags::UTF8 || flags.is_empty())
    {
        return Some((c"utf-8", 3)); // EF BB BF
    }
    if at(0) == 0xff && at(1) == 0xfe {
        if size >= 4 && at(2) == 0 && at(3) == 0 && (all || flags == FioFlags::UCS4 | le) {
            return Some((c"ucs-4le", 4)); // FF FE 00 00
        }
        if flags == FioFlags::UCS2 | le {
            return Some((c"ucs-2le", 2)); // FF FE
        }
        if all || flags == FioFlags::UTF16 | le {
            // utf-16le is preferred, it also works for ucs-2le text.
            return Some((c"utf-16le", 2)); // FF FE
        }
        return None;
    }
    if at(0) == 0xfe
        && at(1) == 0xff
        && (all || flags == FioFlags::UCS2 || flags == FioFlags::UTF16)
    {
        // Default to utf-16, it works for ucs-2 text as well.
        let name = if flags == FioFlags::UCS2 {
            c"ucs-2"
        } else {
            c"utf-16"
        };
        return Some((name, 2)); // FE FF
    }
    if size >= 4
        && at(0) == 0
        && at(1) == 0
        && at(2) == 0xfe
        && at(3) == 0xff
        && (all || flags == FioFlags::UCS4)
    {
        return Some((c"ucs-4", 4)); // 00 00 FE FF
    }
    None
}

/// The read buffer, and where in it the bytes being worked on are.
///
/// `bytes` is reused from one read to the next. `linerest` bytes of the line
/// the previous read ended in sit at its start; `ptr`/`size` are the bytes
/// just read, which conversion rewrites in place or moves further into the
/// buffer (up to `ptr + real_size`), moving the line's earlier bytes to just
/// before them -- which is where `line_start` ends up.
pub(crate) struct ReadBuf {
    pub bytes: Vec<u8>,
    pub ptr: usize,
    pub line_start: usize,
    /// How many bytes at `ptr` are to be worked on. Negative once the line
    /// splitting has used them all up.
    pub size: isize,
    /// The room after `ptr` the read was given; conversion may grow the
    /// bytes into it.
    pub real_size: usize,
    pub linerest: usize,
}

impl ReadBuf {
    pub(crate) fn new() -> Self {
        ReadBuf {
            bytes: Vec::new(),
            ptr: 0,
            line_start: 0,
            size: 0,
            real_size: 0,
            linerest: 0,
        }
    }

    /// `size` as a count, for a window that is known to be in use.
    pub(crate) fn len(&self) -> usize {
        usize::try_from(self.size).unwrap_or(0)
    }

    /// The bytes being worked on.
    pub(crate) fn window(&self) -> &[u8] {
        &self.bytes[self.ptr..self.ptr + self.len()]
    }

    /// Make room for `size` new bytes after the `linerest` already held,
    /// moving those to the front. False when even the smallest buffer could
    /// not be had.
    pub(crate) fn make_room(&mut self) -> bool {
        let from = self.ptr - self.linerest;
        while self.size >= 10 {
            let want = self.len() + self.linerest + 1;
            if want <= self.bytes.len() || try_grow_zeroed(&mut self.bytes, want) {
                // Copy the characters from the previous read.
                self.bytes.copy_within(from..from + self.linerest, 0);
                self.ptr = self.linerest;
                self.line_start = 0;
                return true;
            }
            self.size /= 2;
        }
        false
    }

    /// Move the previous line's tail to just before `at`, and make that the
    /// start of the current line.
    fn move_linerest(&mut self, at: usize) {
        self.line_start = at - self.linerest;
        self.bytes.copy_within(0..self.linerest, self.line_start);
    }
}

/// What the conversion knows, carried from one read to the next.
pub(crate) struct Conv {
    /// What Nvim converts itself, or nothing.
    pub flags: FioFlags,
    /// The iconv descriptor, when iconv does the converting.
    pub iconv: Option<Iconv>,
    /// Bytes of a character split across two reads.
    rest: [u8; CONV_RESTLEN],
    pub restlen: usize,
    /// `BAD_REPLACE`, `BAD_KEEP`, `BAD_DROP`, or the replacement character.
    pub bad_char: c_int,
    /// Line number of the first conversion error, or 0.
    pub conv_error: LineNr,
    /// Buffer line count before this read, for turning an offset into a line
    /// number.
    pub linecnt: LineNr,
    /// Is rewinding and trying another `'fileencoding'` still an option?
    pub can_retry: bool,
}

impl Conv {
    pub(crate) fn new(bad_char: c_int) -> Self {
        Conv {
            flags: FioFlags::NONE,
            iconv: None,
            rest: [0; CONV_RESTLEN],
            restlen: 0,
            bad_char,
            conv_error: 0,
            linecnt: 0,
            can_retry: false,
        }
    }

    pub(crate) fn has_iconv(&self) -> bool {
        self.iconv.is_some()
    }

    /// Close the iconv descriptor, if there is one.
    pub(crate) fn close_iconv(&mut self) {
        self.iconv = None;
    }

    /// Ask iconv to convert `fenc` to UTF-8. False when it cannot.
    pub(crate) fn open_iconv(&mut self, fenc: &CStr) -> bool {
        self.iconv = open_iconv(c"utf-8", fenc);
        self.has_iconv()
    }

    /// Is anything being converted?
    pub(crate) fn active(&self) -> bool {
        !self.flags.is_empty() || self.has_iconv()
    }

    /// Note a conversion error on the line `text` ends on, if none was noted
    /// yet.
    fn note_error(&mut self, text: &[u8]) {
        if self.conv_error == 0 {
            self.conv_error = readfile_linenr(self.linecnt, text);
        }
    }

    /// Keep `bytes` for the next read.
    fn stash(&mut self, bytes: &[u8]) {
        self.rest[..bytes.len()].copy_from_slice(bytes);
        self.restlen = bytes.len();
    }

    /// Put the held-over bytes at `at`.
    ///
    /// `restlen` deliberately stays set: the bytes are laid down before the
    /// read so that the read appends to them, and only counted back in once
    /// the read is done.
    pub(crate) fn restore(&self, into: &mut [u8]) {
        into[..self.restlen].copy_from_slice(&self.rest[..self.restlen]);
    }

    /// Convert the bytes in `w` with iconv, into the room after them.
    ///
    /// False means the encoding is wrong and the file should be read again
    /// with the next one.
    pub(crate) fn with_iconv(&mut self, w: &mut ReadBuf) -> bool {
        let Some(iconv) = self.iconv.take() else {
            return true;
        };
        let converted = self.iconv_into(&iconv, w);
        self.iconv = Some(iconv);
        converted
    }

    fn iconv_into(&mut self, iconv: &Iconv, w: &mut ReadBuf) -> bool {
        let (mut from, from_end) = (w.ptr, w.ptr + w.len());
        // The converted bytes go after the ones being converted.
        let out_start = from_end;
        let (mut to, to_end) = (out_start, w.ptr + w.real_size);

        // On a conversion error, or when there is not enough room, try
        // another conversion -- unless there is no alternative, as for help
        // files.
        loop {
            let (head, tail) = w.bytes.split_at_mut(out_start);
            let mut input = &head[from..from_end];
            let mut output = &mut tail[to - out_start..to_end - out_start];
            let status = iconv.step(&mut input, &mut output);
            from = from_end - input.len();
            to = to_end - output.len();
            let stopped = status.is_err_and(|errno| errno != ICONV_EINVAL);
            if (!stopped && from_end - from <= CONV_RESTLEN) || from == from_end {
                break;
            }
            if self.can_retry {
                return false;
            }
            self.note_error(&w.bytes[out_start..to]);

            // Deal with a bad byte and continue with the next.
            let bad = w.bytes[from];
            from += 1;
            let put = if self.bad_char == BAD_KEEP {
                Some(bad)
            } else if self.bad_char == BAD_DROP {
                None
            } else {
                Some(self.bad_char as u8)
            };
            // Upstream counts the room down without looking; there is
            // always some, the read having left seven eighths of the buffer
            // for the output.
            if let Some(byte) = put
                && to < to_end
            {
                w.bytes[to] = byte;
                to += 1;
            }
        }

        if from < from_end {
            // Some characters are left over; keep them for the next round.
            self.stash(&w.bytes[from..from_end]);
        }

        w.move_linerest(out_start);
        w.ptr = out_start;
        w.size = (to - out_start) as isize;
        true
    }

    /// Convert Unicode or Latin1 code units in `w` to UTF-8.
    ///
    /// Works from the end of the buffer towards the start, because the number
    /// of bytes may grow. False means "read the file again with the next
    /// encoding".
    pub(crate) fn units_to_utf8(&mut self, w: &mut ReadBuf) -> bool {
        let flags = self.flags;
        let utf16_pair = FioFlags::UCS2 | FioFlags::UTF16;
        let start = w.ptr;
        let data_end = start + w.len();
        // Where the UTF-8 bytes go, filling the room from its end.
        let room_end = start + w.real_size;
        let mut dest = room_end;
        let mut p = data_end;
        // An incomplete sequence at the end, to be kept for next time.
        let mut tail: Option<usize> = None;

        if flags == FioFlags::LATIN1 || flags == FioFlags::UTF8 {
            if flags == FioFlags::UTF8 {
                // Check for a trailing incomplete UTF-8 sequence.
                let mut t = data_end - 1;
                while t > start && w.bytes[t] & 0xc0 == 0x80 {
                    t -= 1;
                }
                if t + utf_byte2len(c_int::from(w.bytes[t])) as usize > data_end {
                    tail = Some(t);
                    p = t;
                }
            }
        } else if flags.has(utf16_pair) {
            // Check for a trailing odd byte.
            p = start + (w.len() & !1);
            if w.len() & 1 != 0 {
                tail = Some(p);
            }
            if flags.has(FioFlags::UTF16) && p > start {
                // Check for a trailing leading word.
                let u8c = read_word(&w.bytes, &mut p, flags);
                if (0xd800..=0xdbff).contains(&u8c) {
                    tail = Some(p);
                } else {
                    p += 2;
                }
            }
        } else {
            // UCS-4: check for 1, 2 or 3 trailing bytes.
            p = start + (w.len() & !3);
            if w.len() & 3 != 0 {
                tail = Some(p);
            }
        }

        if let Some(tail) = tail {
            self.stash(&w.bytes[tail..data_end]);
            w.size -= (data_end - tail) as isize;
        }

        if flags == FioFlags::LATIN1 {
            // The common case, and a simple one: each byte is a character,
            // one or two bytes long in UTF-8.
            let bytes = &mut w.bytes[start..room_end];
            let mut dest = room_end - start;
            for p in (0..p - start).rev() {
                let b = bytes[p];
                if b < 0x80 {
                    dest -= 1;
                    bytes[dest] = b;
                } else {
                    dest -= 2;
                    bytes[dest] = 0xc0 | (b >> 6);
                    bytes[dest + 1] = 0x80 | (b & 0x3f);
                }
            }
            let dest = start + dest;
            w.move_linerest(dest);
            w.size = (room_end - dest) as isize;
            w.ptr = dest;
            return true;
        }

        while p > start {
            let mut u8c;
            if flags.has(FioFlags::LATIN1) {
                p -= 1;
                u8c = u32::from(w.bytes[p]);
            } else if flags.has(utf16_pair) {
                u8c = read_word(&w.bytes, &mut p, flags);
                if flags.has(FioFlags::UTF16) && (0xdc00..=0xdfff).contains(&u8c) {
                    if p == start {
                        // Missing leading word.
                        if self.can_retry {
                            return false;
                        }
                        self.note_error(&w.bytes[start..p]);
                        if self.bad_char == BAD_DROP {
                            continue;
                        }
                        if self.bad_char != BAD_KEEP {
                            u8c = self.bad_char as u32;
                        }
                    }
                    // The second word of a pair; take the first and put them
                    // together.
                    let u16c = read_word(&w.bytes, &mut p, flags);
                    u8c = 0x10000 + ((u16c & 0x3ff) << 10) + (u8c & 0x3ff);

                    // Check that the first word really was a leading word.
                    if !(0xd800..=0xdbff).contains(&u16c) {
                        if self.can_retry {
                            return false;
                        }
                        self.note_error(&w.bytes[start..p]);
                        if self.bad_char == BAD_DROP {
                            continue;
                        }
                        if self.bad_char != BAD_KEEP {
                            u8c = self.bad_char as u32;
                        }
                    }
                }
            } else if flags.has(FioFlags::UCS4) {
                let mut word = 0u32;
                for shift in 0..4 {
                    p -= 1;
                    let at = if flags.has(FioFlags::ENDIAN_L) {
                        24 - 8 * shift
                    } else {
                        8 * shift
                    };
                    word |= u32::from(w.bytes[p]) << at;
                }
                u8c = word;
                // Replace characters over INT_MAX with the Unicode
                // replacement character.
                if u8c > i32::MAX as u32 {
                    u8c = 0xfffd;
                }
            } else {
                // UTF-8.
                p -= 1;
                if w.bytes[p] < 0x80 {
                    u8c = u32::from(w.bytes[p]);
                } else {
                    let len = head_off(&w.bytes[start..data_end], p - start);
                    p -= len;
                    u8c = char_at(&w.bytes[p..data_end]) as u32;
                    if len == 0 {
                        // Not a valid UTF-8 character: retry with another
                        // fenc when possible, otherwise report the error.
                        if self.can_retry {
                            return false;
                        }
                        self.note_error(&w.bytes[start..p]);
                        if self.bad_char == BAD_DROP {
                            continue;
                        }
                        if self.bad_char != BAD_KEEP {
                            u8c = self.bad_char as u32;
                        }
                    }
                }
            }
            debug_assert!(u8c <= i32::MAX as u32);
            let c = u8c as c_int;
            dest -= utf_char2len(c) as usize;
            encode_char(c, &mut w.bytes[dest..room_end]);
        }

        w.move_linerest(dest);
        w.size = (room_end - dest) as isize;
        w.ptr = dest;
        true
    }

    /// Check that the bytes in `w` really are valid UTF-8, replacing,
    /// dropping or keeping the ones that are not.
    ///
    /// False means "read the file again with another conversion".
    pub(crate) fn check_utf8(
        &mut self,
        w: &mut ReadBuf,
        filesize: FileOffset,
        illegal_byte: &mut LineNr,
    ) -> bool {
        let start = w.ptr;
        let mut p = start;
        let mut incomplete_tail = false;
        loop {
            let end = start + w.len();
            p = skip_valid_utf8(&w.bytes[..end], p);
            if p >= end {
                break;
            }
            let todo = end - p;

            // A length of 1 means it is an illegal byte. An incomplete
            // character at the end is accepted, though: the next read gets
            // the rest of it and we check it then.
            let l = promised_char_len(&w.bytes[p..end]);
            if l > todo && !incomplete_tail {
                // Avoid retrying with a different encoding when a truncated
                // file is more likely, or reading the rest of an incomplete
                // sequence when we already did so.
                if p > start || filesize > 0 {
                    incomplete_tail = true;
                }
                if p > start {
                    self.stash(&w.bytes[p..end]);
                    w.size -= todo as isize;
                    break;
                }
            }
            if l != 1 && l <= todo {
                p += l;
                continue;
            }

            // An illegal byte. Try another encoding if we can, unless we are
            // at end of file, where a truncated file is more likely than a
            // conversion error.
            if self.can_retry && !incomplete_tail {
                break;
            }
            // When we did a conversion, report an error.
            if self.has_iconv() {
                self.note_error(&w.bytes[start..p]);
            }
            // Remember the first line with an illegal byte.
            if self.conv_error == 0 && *illegal_byte == 0 {
                *illegal_byte = readfile_linenr(self.linecnt, &w.bytes[start..p]);
            }

            // Drop, keep or replace the bad byte.
            if self.bad_char == BAD_DROP {
                w.bytes.copy_within(p + 1..end, p);
                w.size -= 1;
            } else {
                if self.bad_char != BAD_KEEP {
                    w.bytes[p] = self.bad_char as u8;
                }
                p += 1;
            }
        }
        // A UTF-8 error was detected if we stopped short of the end.
        p >= start + w.len() || incomplete_tail
    }
}

/// Where the first byte from `p` on that is not part of a whole, valid UTF-8
/// character is: an illegal byte, a sequence `bytes` cuts short, or the end.
///
/// ASCII goes a machine word at a time. This is every byte of every file
/// read without a conversion, so it keeps the walk to itself.
fn skip_valid_utf8(bytes: &[u8], mut p: usize) -> usize {
    const WORD: usize = size_of::<u64>();
    loop {
        let (words, _) = bytes[p..].as_chunks::<WORD>();
        let ascii = words
            .iter()
            .position(|word| u64::from_ne_bytes(*word) & NONASCII_MASK != 0)
            .unwrap_or(words.len());
        p += ascii * WORD;
        p += bytes[p..]
            .iter()
            .position(|&b| b >= 0x80)
            .unwrap_or(bytes.len() - p);
        if p >= bytes.len() {
            return p;
        }
        let l = promised_char_len(&bytes[p..]);
        if l == 1 || l > bytes.len() - p {
            return p;
        }
        p += l;
    }
}

/// Read one 16-bit code unit backwards from `p`, in the flags' byte order.
fn read_word(bytes: &[u8], p: &mut usize, flags: FioFlags) -> u32 {
    let first = u32::from(bytes[*p - 1]);
    let second = u32::from(bytes[*p - 2]);
    *p -= 2;
    if flags.has(FioFlags::ENDIAN_L) {
        (first << 8) + second
    } else {
        first + (second << 8)
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
    pub(crate) fn from_ffs() -> Self {
        FormatGuess {
            try_dos: p_ffs(|value| has_char(value, c_int::from(b'd'))),
            try_unix: c_int::from(p_ffs(|value| has_char(value, c_int::from(b'x')))),
            try_mac: c_int::from(p_ffs(|value| has_char(value, c_int::from(b'm')))),
        }
    }

    /// Guess the end-of-line format from the first bytes of the file.
    pub(crate) fn guess(&mut self, data: &[u8]) -> c_int {
        let (nl, cr) = (NL as u8, CAR as u8);
        let mut fileformat = EOL_UNKNOWN;

        // First try finding a NL, for Dos and Unix.
        if self.try_dos || self.try_unix != 0 {
            // Reset the carriage return counter.
            if self.try_mac != 0 {
                self.try_mac = 1;
            }

            let mut p = 0;
            while p < data.len() {
                if data[p] == nl {
                    fileformat =
                        if self.try_unix == 0 || (self.try_dos && p > 0 && data[p - 1] == cr) {
                            EOL_DOS
                        } else {
                            EOL_UNIX
                        };
                    break;
                }
                if data[p] == cr && self.try_mac != 0 {
                    self.try_mac += 1;
                }
                p += 1;
            }

            // Don't give in to EOL_UNIX if EOL_MAC is more likely.
            if fileformat == EOL_UNIX && self.try_mac != 0 {
                // The counters have to be reset when retrying the fenc.
                self.try_mac = 1;
                self.try_unix = 1;
                // Is there a CR at or before the NL?
                if data[..=p].contains(&cr) {
                    for &b in data {
                        if b == nl {
                            self.try_unix += 1;
                        } else if b == cr {
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

/// Set up for reading the file again with another conversion.
pub(crate) fn rewind_retry(
    did_iconv: &mut bool,
    advance_fenc: &mut bool,
    file_rewind: &mut bool,
    had_iconv: bool,
) {
    if P_CCV.first_byte() != 0 && had_iconv {
        // iconv() failed; try 'charconvert'.
        *did_iconv = true;
    } else {
        // Use the next item from 'fileencodings'.
        *advance_fenc = true;
    }
    *file_rewind = true;
}
