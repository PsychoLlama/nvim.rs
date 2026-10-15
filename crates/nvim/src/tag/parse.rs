//! Reading a tags file, one line at a time.
//!
//! A sorted tags file is searched by bisection and an unsorted one
//! linearly; [`FindTags::start_state`] chooses between them once the
//! header has been read, and [`FindTags::next_line`] is what reads, seeks
//! and re-seeks. [`FindTags::parse_line`] splits a line into its tag name,
//! file name and search command — and, during a bisection, says which way
//! to go next — while [`FindTags::match_tag`] decides whether the name it
//! found is the one being looked for.
//!
//! [`TagReader`] is the file under all of it: `fgets`, `ftello` and
//! `fseeko` as the bisection used them, over a [`File`].

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::mbyte::strnicmp_in;
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Seek, SeekFrom};
use std::os::unix::ffi::OsStrExt;

/// What reading the next line produced.
pub(crate) enum Line {
    /// A line is in the buffer and should be looked at.
    Read,
    /// The end of the file, or the end of a bisection with no match.
    Eof,
    /// Nothing usable; go round again.
    Ignore,
}

/// What a line means for the search.
pub(crate) enum Verdict {
    /// The line was parsed and may be the tag wanted.
    Success,
    /// The line is malformed.
    Fail,
    /// Every line that could match has been looked at.
    Stop,
    /// Look at the next line.
    Next,
}

/// How many bytes `vim_fgets` throws a too-long line's rest away in.
const DISCARD_CHUNK: usize = 200;

/// Eight copies of `byte`.
const fn splat(byte: u8) -> u64 {
    0x0101_0101_0101_0101 * byte as u64
}

/// The high bit of every byte of `word` that is zero.
///
/// A byte above a zero one can come out set as well (the subtraction
/// borrows through it), but never one below it: the lowest bit set is the
/// first zero byte, which is all [`find_either`] asks.
const fn zero_bytes(word: u64) -> u64 {
    // Integer arithmetic, spelled as a call: the ratchet's pointer-arithmetic
    // count reads `.wrapping_sub(` as a pointer step.
    u64::wrapping_sub(word, splat(1)) & !word & splat(0x80)
}

/// Where the first byte of `hay` that is `a` or `b` is.
///
/// Eight bytes at a time: the readers below run this over every byte of a
/// tags file, and a byte-at-a-time walk costs several times what C's
/// `memchr` did there.
fn find_either(hay: &[u8], a: u8, b: u8) -> Option<usize> {
    let (pa, pb) = (splat(a), splat(b));
    let (words, rest) = hay.as_chunks::<8>();
    let mut at = 0;
    for word in words {
        let word = u64::from_le_bytes(*word);
        let found = zero_bytes(word ^ pa) | zero_bytes(word ^ pb);
        if found != 0 {
            // Little-endian: the lowest set bit is the first byte.
            return Some(at + (found.trailing_zeros() / 8) as usize);
        }
        at += 8;
    }
    rest.iter()
        .position(|&byte| byte == a || byte == b)
        .map(|found| at + found)
}

/// Where the first `needle` in `hay` is; [`find_either`] for one byte.
pub(crate) fn find_byte(hay: &[u8], needle: u8) -> Option<usize> {
    find_either(hay, needle, needle)
}

/// A tags file open for reading.
///
/// The bisection's probing is defined by what C's `fgets`, `ftello` and
/// `fseeko` did — where a read stops, which offset it reports, what a
/// line too long for the buffer leaves behind — so this reads with exactly
/// those answers. Where the next byte comes from is tracked here rather
/// than asked of the file.
pub(crate) struct TagReader {
    file: BufReader<File>,
    /// The offset of the next byte a read answers.
    pos: u64,
}

impl TagReader {
    /// Open the tags file `name`, or `None` when it cannot be opened.
    pub(crate) fn open(name: &CStr) -> Option<TagReader> {
        let file = File::open(OsStr::from_bytes(name.to_bytes())).ok()?;
        Some(TagReader {
            file: BufReader::new(file),
            pos: 0,
        })
    }

    /// `fgets(3)` into `out`: up to and including a newline, or `out.len()`
    /// bytes, or what is left of the file. Answers how many bytes it read,
    /// and where the first NUL among them is.
    fn read_line_into(&mut self, out: &mut [u8]) -> (usize, Option<usize>) {
        let mut got = 0;
        let mut nul = None;
        while got < out.len() {
            let available = match self.file.fill_buf() {
                Ok(available) => available,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                // A read error ends the line where it struck, as the end of
                // the file would.
                Err(_) => break,
            };
            let room = (out.len() - got).min(available.len());
            if room == 0 {
                break;
            }
            let chunk = &available[..room];
            // The newline that ends the line, noting the first NUL on the
            // way: that is where a C reader of the line stops.
            let mut from = 0;
            let end = loop {
                match find_either(&chunk[from..], b'\n', 0) {
                    None => break None,
                    Some(at) if chunk[from + at] == b'\n' => break Some(from + at),
                    Some(at) => {
                        nul.get_or_insert(got + from + at);
                        from += at + 1;
                    }
                }
            };
            let take = end.map_or(room, |end| end + 1);
            out[got..got + take].copy_from_slice(&chunk[..take]);
            self.file.consume(take);
            got += take;
            if end.is_some() {
                break;
            }
        }
        self.pos += got as u64;
        (got, nul)
    }

    /// `vim_fgets`: read one line into `buf`, whose length is the buffer
    /// size. Answers the length of the line as a C string — up to its first
    /// NUL — or `None` at the end of the file.
    ///
    /// A line that does not fit is cut at `buf.len() - 1` bytes and the rest
    /// of it thrown away. What tells the caller so is the last-but-one
    /// byte: `fgets` leaves it NUL unless the line reached it.
    pub(crate) fn fgets(&mut self, buf: &mut [u8]) -> Option<usize> {
        let size = buf.len();
        buf[size - 2] = 0;
        let (got, nul) = self.read_line_into(&mut buf[..size - 1]);
        if got == 0 {
            // `fgets` leaves the buffer alone at the end of the file.
            return None;
        }
        buf[got] = 0;

        let filled = |byte: u8| byte != 0 && byte != b'\n';
        if filled(buf[size - 2]) {
            buf[size - 1] = 0;
            // Throw the rest of the line away, a chunk at a time. A chunk
            // that ends the file answers "end of file", as `fgets`
            // returning NULL there did.
            let mut chunk = [0; DISCARD_CHUNK - 1];
            loop {
                let (got, _) = self.read_line_into(&mut chunk);
                if got == 0 {
                    return None;
                }
                if got < chunk.len() || !filled(chunk[got - 1]) {
                    break;
                }
            }
        }
        Some(nul.unwrap_or(got))
    }

    /// `ftello`: the offset of the next byte a read answers.
    pub(crate) fn tell(&self) -> FileOffset {
        FileOffset::try_from(self.pos).unwrap_or(FileOffset::MAX)
    }

    /// `fseeko(SEEK_SET)`, answering whether it moved. A negative offset
    /// does not.
    pub(crate) fn seek(&mut self, offset: FileOffset) -> bool {
        let Ok(target) = u64::try_from(offset) else {
            return false;
        };
        // Relative, so a target still inside the buffer keeps it — as
        // `fseeko` does on a stream opened for reading.
        let delta =
            i64::try_from(target).unwrap_or(i64::MAX) - i64::try_from(self.pos).unwrap_or(i64::MAX);
        if self.file.seek_relative(delta).is_err() {
            return false;
        }
        self.pos = target;
        true
    }

    /// `fseeko(SEEK_END)` then `ftello`: the size of the file, or `None`
    /// when it cannot be sought. The position is left at the end.
    pub(crate) fn seek_end(&mut self) -> Option<FileOffset> {
        let end = self.file.seek(SeekFrom::End(0)).ok()?;
        self.pos = end;
        Some(self.tell())
    }
}

/// The byte `at` of a C string's bytes, NUL past the end.
fn byte(s: &[u8], at: usize) -> u8 {
    cstr::byte_at(s, at)
}

/// `strncmp(s1, s2, len)` over the bytes of two C strings.
fn prefix_cmp(s1: &[u8], s2: &[u8], len: usize) -> c_int {
    for at in 0..len {
        let (a, b) = (byte(s1, at), byte(s2, at));
        if a != b {
            return c_int::from(a) - c_int::from(b);
        }
        if a == 0 {
            break;
        }
    }
    0
}

/// Compare two tag names for at most `len` bytes with case folded to
/// upper, the way `sort -f` orders them.
fn tag_strnicmp(s1: &[u8], s2: &[u8], len: usize) -> c_int {
    for at in 0..len {
        let a = byte(s1, at).to_ascii_uppercase();
        let b = byte(s2, at).to_ascii_uppercase();
        if a != b {
            return c_int::from(a) - c_int::from(b);
        }
        if a == 0 {
            break;
        }
    }
    0
}

/// `mb_strnicmp(s1, s2, len)` over the bytes of two C strings.
pub(crate) fn strnicmp_prefix(s1: &[u8], s2: &[u8], len: usize) -> c_int {
    strnicmp_in(&s1[..len.min(s1.len())], &s2[..len.min(s2.len())])
}

/// Where the first TAB at or after `from` is in `line`.
pub(crate) fn find_tab(line: &[u8], from: usize) -> Option<usize> {
    find_byte(line.get(from..)?, TAB as u8).map(|at| from + at)
}

impl FindTags {
    /// Read a line into the buffer, answering true at end of file.
    #[inline(always)]
    fn fgets(&mut self) -> bool {
        let Some(reader) = self.reader.as_mut() else {
            return true;
        };
        match reader.fgets(&mut self.lbuf) {
            Some(len) => {
                self.line_len = len;
                false
            }
            None => true,
        }
    }

    /// The line in the buffer, up to its terminator.
    #[inline(always)]
    pub(crate) fn line(&self) -> &[u8] {
        &self.lbuf[..self.line_len]
    }

    /// Whether the line just read holds nothing but white space.
    #[inline(always)]
    fn blank_line(&self) -> bool {
        is_blank_line(self.line())
    }

    /// Where the file is being read from now.
    #[inline(always)]
    pub(crate) fn tell(&self) -> FileOffset {
        self.reader.as_ref().map_or(0, TagReader::tell)
    }

    /// Move to `offset`; where a seek fails, the reading goes on from
    /// where it was, as an `fseeko` whose answer was ignored.
    #[inline(always)]
    pub(crate) fn seek(&mut self, offset: FileOffset) {
        if let Some(reader) = self.reader.as_mut() {
            reader.seek(offset);
        }
    }

    /// Read the next line of the tags file into the buffer.
    ///
    /// While bisecting, this is also what moves: it picks the next offset,
    /// seeks there and reads past whatever partial line it landed in.
    #[inline]
    pub(crate) fn next_line(&mut self, sinfo: &mut SearchInfo) -> Line {
        match self.state {
            Reading::Binary => {
                // Halve the range that is left.
                let offset = sinfo.low_offset + (sinfo.high_offset - sinfo.low_offset) / 2;
                if offset == sinfo.curr_offset {
                    // The range is down to nothing: no match.
                    return Line::Eof;
                }
                sinfo.curr_offset = offset;
            }
            Reading::SkipBack => {
                // Step back over roughly two lines' worth of file.
                sinfo.curr_offset -= FileOffset::try_from(self.lbuf.len() * 2).unwrap_or(0);
                if sinfo.curr_offset < 0 {
                    sinfo.curr_offset = 0;
                    self.seek(0);
                    self.state = Reading::StepForward;
                }
            }
            _ => {}
        }

        if !matches!(self.state, Reading::Binary | Reading::SkipBack) {
            // Not jumping around: just read the next line.
            let mut eof = self.fgets();
            while !eof && self.blank_line() {
                eof = self.fgets();
            }
            return if eof { Line::Eof } else { Line::Read };
        }

        // Landing at an offset lands in the middle of a line, so the
        // first read is thrown away and the one after it is the line.
        sinfo.curr_offset_used = sinfo.curr_offset;
        self.seek(sinfo.curr_offset);
        let mut eof = self.fgets();
        if !eof && sinfo.curr_offset != 0 {
            sinfo.curr_offset = self.tell();
            if sinfo.curr_offset == sinfo.high_offset {
                // Went a bit too far; try from the low offset.
                self.seek(sinfo.low_offset);
                sinfo.curr_offset = sinfo.low_offset;
            }
            eof = self.fgets();
        }
        while !eof && self.blank_line() {
            sinfo.curr_offset = self.tell();
            eof = self.fgets();
        }
        if eof {
            // Hit the end of the file; skip backwards instead.
            self.state = Reading::SkipBack;
            sinfo.match_offset = self.tell();
            sinfo.curr_offset = sinfo.curr_offset_used;
            return Line::Ignore;
        }
        Line::Read
    }

    /// Read one header line, answering true if it is not a header after
    /// all and should be parsed as a tag.
    fn hdr_parse(&mut self) -> bool {
        let line = &self.lbuf[..self.line_len];
        // A header line starts with "!_TAG_"; anything else here is a
        // non-header item before the header, e.g. "!" on its own.
        if !line.starts_with(b"!_TAG_") {
            return true;
        }
        if line.starts_with(b"!_TAG_FILE_SORTED\t") {
            self.tag_file_sorted = byte(line, 18);
        }
        if line.starts_with(b"!_TAG_FILE_ENCODING\t") {
            // Prepare to convert every line from that encoding to
            // 'encoding'. The name ends at the first byte that is not
            // printable ASCII.
            let rest = &line[20..];
            let end = rest
                .iter()
                .position(|&b| !(b > b' ' && b < 127))
                .unwrap_or(rest.len());
            let name = cstr::owned(&rest[..end]);
            let _ = p_enc(|value| self.vimconv.setup(Some(&name), Some(value)));
        }
        // Read the next line; an unrecognised flag is ignored.
        false
    }

    /// Look at the first lines of a tags file: read the header, then
    /// decide how the rest of the file will be read.
    ///
    /// Answers true if the line in the buffer should be parsed as a tag.
    pub(crate) fn start_state(&mut self, margs: &mut MatchArgs, sinfo: &mut SearchInfo) -> bool {
        let noic = self.flags & TAG_NOIC as c_int != 0;

        // The header ends at the first line that sorts below "!_TAG_".
        // With case folded, a lower-case letter sorts before "_".
        let line = self.line();
        if &line[..line.len().min(6)] <= b"!_TAG_".as_slice()
            || (byte(line, 0) == b'!' && byte(line, 1).is_ascii_lowercase())
        {
            return self.hdr_parse();
        }

        // With no usable pattern head, or with case ignored, the file
        // has to be read line by line. Without a `!_TAG_FILE_SORTED`
        // header, assume it is sorted: if it is not, the second round
        // reads it linearly anyway.
        self.state = match self.tag_file_sorted {
            _ if self.linear => Reading::Linear,
            0 | b'1' => Reading::Binary,
            b'2' => {
                // Sorted with case folded.
                margs.sortic = true;
                self.orgpat.regmatch.rm_ic = p_ic() || !noic;
                Reading::Binary
            }
            _ => Reading::Linear,
        };
        if self.state == Reading::Binary && self.orgpat.regmatch.rm_ic && !margs.sortic {
            // Bisection cannot find a match that only holds with case
            // ignored.
            self.linear = true;
            self.state = Reading::Linear;
        }

        if self.state != Reading::Binary {
            return true;
        }

        // Starting a bisection: the range is the whole file, and the
        // first read is from the middle of it.
        let Some(filesize) = self.reader.as_mut().and_then(TagReader::seek_end) else {
            // Cannot seek, so cannot bisect.
            self.state = Reading::Linear;
            return true;
        };
        self.seek(0);
        *sinfo = SearchInfo {
            low_offset: 0,
            low_char: 0,
            high_offset: filesize,
            curr_offset: 0,
            high_char: 0xff,
            ..*sinfo
        };
        false
    }

    /// Split the line in the buffer into its fields, and — while
    /// bisecting — work out where to look next.
    ///
    /// The tag name is compared against the pattern's plain head first, as
    /// a quick way of rejecting a line: that is most of what makes tag
    /// searching fast.
    #[inline]
    pub(crate) fn parse_line(
        &mut self,
        tagp: &mut TagParts,
        margs: &mut MatchArgs,
        sinfo: &mut SearchInfo,
    ) -> Verdict {
        let line = &self.lbuf[..self.line_len];
        if self.orgpat.headlen == 0 {
            // No head to compare: take the line apart the slow way.
            return match parse_tag_line(line) {
                Some(parts) => {
                    *tagp = parts;
                    Verdict::Success
                }
                None => Verdict::Fail,
            };
        }

        *tagp = TagParts::default();
        let Some(name_end) = find_tab(line, 0) else {
            // Corrupted tag line.
            return Verdict::Fail;
        };
        tagp.tagname = Span {
            start: 0,
            end: name_end,
        };

        // How much of the two names to compare.
        let mut cmplen = name_end;
        if let Ok(limit) = usize::try_from(p_tl())
            && limit != 0
            && cmplen > limit
        {
            cmplen = limit;
        }
        if margs.has_re && self.orgpat.headlen < cmplen {
            cmplen = self.orgpat.headlen;
        } else if self.state == Reading::Linear && self.orgpat.headlen != cmplen {
            // A different length and no regexp: it cannot match.
            return Verdict::Next;
        }
        let head = self.orgpat.head();

        match self.state {
            Reading::Binary => {
                // A first byte outside the range the bisection has
                // narrowed to means the file is not sorted after all.
                // Upstream folds case through a *signed* `char`, so a
                // byte above 0x7f reads negative here and unsigned in
                // the branch below it. Preserved.
                let first = byte(line, 0);
                let signed = c_int::from(first.cast_signed());
                let sort_key = if !margs.sortic {
                    c_int::from(first)
                } else if first.is_ascii_lowercase() {
                    signed - c_int::from(b'a' - b'A')
                } else {
                    signed
                };
                if sort_key < sinfo.low_char || sort_key > sinfo.high_char {
                    margs.sort_error = true;
                }

                let mut tagcmp = if margs.sortic {
                    tag_strnicmp(line, head, cmplen)
                } else {
                    prefix_cmp(line, head, cmplen)
                };
                // A match on a shorter tag means to search forward, on
                // a longer one to search backward.
                if tagcmp == 0 {
                    tagcmp = cmplen.cmp(&self.orgpat.headlen) as c_int;
                }

                if tagcmp == 0 {
                    // Found it. Skip back, then forward again, to land
                    // on the first tag that matches.
                    self.state = Reading::SkipBack;
                    sinfo.match_offset = sinfo.curr_offset;
                    return Verdict::Next;
                }
                if tagcmp < 0 {
                    sinfo.curr_offset = self.tell();
                    if sinfo.curr_offset < sinfo.high_offset {
                        sinfo.low_offset = sinfo.curr_offset;
                        sinfo.low_char = sort_key;
                        return Verdict::Next;
                    }
                }
                if tagcmp > 0 && sinfo.curr_offset != sinfo.high_offset {
                    sinfo.high_offset = sinfo.curr_offset;
                    sinfo.high_char = sort_key;
                    return Verdict::Next;
                }
                // No match, and the range is exhausted.
                return Verdict::Stop;
            }
            Reading::SkipBack => {
                if strnicmp_prefix(line, head, cmplen) != 0 {
                    self.state = Reading::StepForward;
                } else {
                    // Have to skip back further. Put the offset the
                    // round started from back, or a long line leaves
                    // the walk stuck on it.
                    sinfo.curr_offset = sinfo.curr_offset_used;
                }
                return Verdict::Next;
            }
            Reading::StepForward => {
                if strnicmp_prefix(line, head, cmplen) != 0 {
                    return if self.tell() > sinfo.match_offset {
                        // Past the last match.
                        Verdict::Stop
                    } else {
                        // Not yet at the first one.
                        Verdict::Next
                    };
                }
            }
            _ => {
                if strnicmp_prefix(line, head, cmplen) != 0 {
                    return Verdict::Next;
                }
            }
        }

        // Could be the tag wanted: isolate the file name and command.
        let fname = name_end + 1;
        let Some(fname_end) = find_tab(line, fname) else {
            return Verdict::Fail;
        };
        tagp.fname = Span {
            start: fname,
            end: fname_end,
        };
        tagp.command = fname_end + 1;
        Verdict::Success
    }

    /// Whether the tag name the line holds is the one being looked for.
    ///
    /// The pattern is tried literally first — even when it is a regexp —
    /// and the regexp only afterwards, which is what tells the collector
    /// how good the match was.
    #[inline]
    pub(crate) fn match_tag(&mut self, tagp: &TagParts, margs: &mut MatchArgs) -> bool {
        let mut cmplen = tagp.tagname.len();
        if let Ok(limit) = usize::try_from(p_tl())
            && limit != 0
            && cmplen > limit
        {
            cmplen = limit;
        }
        let name = &self.lbuf[tagp.tagname.start..self.line_len];
        let pat: &[u8] = &self.orgpat.pat;

        // A name of a different length cannot match literally.
        let mut matched = self.orgpat.len == cmplen && {
            if self.orgpat.regmatch.rm_ic {
                let same = strnicmp_prefix(name, pat, cmplen) == 0;
                if same {
                    margs.match_no_ic = prefix_cmp(name, pat, cmplen) == 0;
                }
                same
            } else {
                prefix_cmp(name, pat, cmplen) == 0
            }
        };

        // With a regexp, also find the tags it matches.
        margs.match_re = false;
        if !matched && !self.orgpat.regmatch.regprog.is_null() {
            // The name is terminated in the buffer for the regexp, and the
            // TAB put back after.
            let end = tagp.tagname.end;
            let saved = self.lbuf[end];
            self.lbuf[end] = 0;
            let name = cstr::in_bytes(&self.lbuf[tagp.tagname.start..=end]);
            let regmatch = &mut self.orgpat.regmatch;
            matched = vim_regexec(regmatch, name, 0);
            if matched {
                margs.matchoff =
                    regmatch.starts[0].map_or(0, |at| c_int::try_from(at).unwrap_or(0));
                if regmatch.rm_ic {
                    // Ask again with case, to find out how good the
                    // match is.
                    regmatch.rm_ic = false;
                    margs.match_no_ic = vim_regexec(regmatch, name, 0);
                    regmatch.rm_ic = true;
                }
            }
            self.lbuf[end] = saved;
            margs.match_re = true;
        }

        matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The word-at-a-time search answers what a byte walk does, wherever
    /// the bytes fall against the word boundaries — including a byte above
    /// 0x80 and a match right after one.
    #[test]
    fn find_either_is_a_byte_walk() {
        let alphabet = [b'a', b'\n', 0, 0x80, 0xff, 0x01, b'\t'];
        let mut seed: u32 = 36;
        for len in 0..40 {
            for _ in 0..50 {
                let hay: Vec<u8> = (0..len)
                    .map(|_| {
                        seed = u32::wrapping_add(seed.wrapping_mul(1_103_515_245), 12_345);
                        alphabet[(seed >> 16) as usize % alphabet.len()]
                    })
                    .collect();
                for (a, b) in [(b'\n', 0), (b'\t', b'\t'), (0x80, 0xff)] {
                    let walk = hay.iter().position(|&byte| byte == a || byte == b);
                    assert_eq!(find_either(&hay, a, b), walk, "{hay:?} {a} {b}");
                }
            }
        }
    }
}
