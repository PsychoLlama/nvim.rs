//! Reading a tags file, one line at a time.
//!
//! A sorted tags file is searched by bisection and an unsorted one
//! linearly; [`FindTags::start_state`] chooses between them once the
//! header has been read, and [`FindTags::next_line`] is what reads, seeks
//! and re-seeks. [`FindTags::parse_line`] splits a line into its tag name,
//! file name and search command — and, during a bisection, says which way
//! to go next — while [`FindTags::match_tag`] decides whether the name it
//! found is the one being looked for.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{c_char, c_int};

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
pub(crate) enum TagMatch {
    /// The line was parsed and may be the tag wanted.
    Success,
    /// The line is malformed.
    Fail,
    /// Every line that could match has been looked at.
    Stop,
    /// Look at the next line.
    Next,
}

/// Compare two tag names for at most `len` bytes with case folded to
/// upper, the way `sort -f` orders them.
///
/// # Safety
/// Both names must be NUL-terminated.
unsafe fn tag_strnicmp(s1: *const c_char, s2: *const c_char, len: usize) -> c_int {
    // SAFETY: neither walk passes the terminator of `s1`; `s2` is the
    // pattern head, which is at least as long as the comparison runs.
    for at in 0..len {
        let a = (unsafe { *s1.add(at) } as u8).to_ascii_uppercase();
        let b = (unsafe { *s2.add(at) } as u8).to_ascii_uppercase();
        if a != b {
            return a as c_int - b as c_int;
        }
        if a == 0 {
            break;
        }
    }
    0
}

impl FindTags {
    /// Read a line into the buffer, answering true at end of file.
    #[inline(always)]
    fn fgets(&mut self) -> bool {
        let size = self.lbuf.len() as c_int;
        let fp = self.fp;
        let buf = self.lbuf.as_mut_ptr();
        // SAFETY: `buf` holds `size` writable bytes and `fp` is open.
        unsafe { vim_fgets(buf, size, fp) }
    }

    /// Whether the line just read holds nothing but white space.
    #[inline(always)]
    fn blank_line(&self) -> bool {
        // SAFETY: `vim_fgets` NUL-terminated the buffer.
        unsafe { vim_isblankline(self.lbuf.as_ptr().cast_mut()) }
    }

    /// Where the file is being read from now.
    #[inline(always)]
    fn tell(&self) -> off_T {
        // SAFETY: `fp` is open.
        unsafe { ftello(self.fp) }
    }

    /// Read the next line of the tags file into the buffer.
    ///
    /// While bisecting, this is also what moves: it picks the next offset,
    /// seeks there and reads past whatever partial line it landed in.
    #[inline]
    pub(crate) fn next_line(&mut self, sinfo: &mut SearchInfo) -> Line {
        // SAFETY: `fp` is open and the buffer is what `fgets` fills.
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
                sinfo.curr_offset -= (self.lbuf.len() * 2) as off_T;
                if sinfo.curr_offset < 0 {
                    sinfo.curr_offset = 0;
                    vim_ignored.set(unsafe { fseeko(self.fp, 0, SEEK_SET) });
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
        vim_ignored.set(unsafe { fseeko(self.fp, sinfo.curr_offset, SEEK_SET) });
        let mut eof = self.fgets();
        if !eof && sinfo.curr_offset != 0 {
            sinfo.curr_offset = self.tell();
            if sinfo.curr_offset == sinfo.high_offset {
                // Went a bit too far; try from the low offset.
                vim_ignored.set(unsafe { fseeko(self.fp, sinfo.low_offset, SEEK_SET) });
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
        // SAFETY: the buffer is NUL-terminated, and every prefix compared
        // below is checked before anything past it is read.
        let line = self.lbuf.as_mut_ptr();
        // A header line starts with "!_TAG_"; anything else here is a
        // non-header item before the header, e.g. "!" on its own.
        if unsafe { strncmp(line, c"!_TAG_".as_ptr().cast_mut(), 6) } != 0 {
            return true;
        }
        if unsafe { strncmp(line, c"!_TAG_FILE_SORTED\t".as_ptr().cast_mut(), 18) } == 0 {
            self.tag_file_sorted = unsafe { *line.add(18) } as u8 as c_int;
        }
        if unsafe { strncmp(line, c"!_TAG_FILE_ENCODING\t".as_ptr().cast_mut(), 20) } == 0 {
            // Prepare to convert every line from that encoding to
            // 'encoding'. The name ends at the first byte that is not
            // printable ASCII, which is cut off in place.
            let mut end = 20;
            while unsafe { *line.add(end) } as u8 > b' ' && (unsafe { *line.add(end) } as u8) < 127
            {
                end += 1;
            }
            unsafe { *line.add(end) = 0 };
            unsafe { convert_setup(&raw mut self.vimconv, line.add(20), p_enc.get()) };
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

        // SAFETY: the buffer is NUL-terminated and `fp` is open.
        // The header ends at the first line that sorts below "!_TAG_".
        // With case folded, a lower-case letter sorts before "_".
        if unsafe { strncmp(self.lbuf.as_ptr(), c"!_TAG_".as_ptr(), 6) } <= 0
            || (self.lbuf[0] == b'!' as c_char && (self.lbuf[1] as u8).is_ascii_lowercase())
        {
            return self.hdr_parse();
        }

        // With no usable pattern head, or with case ignored, the file
        // has to be read line by line. Without a `!_TAG_FILE_SORTED`
        // header, assume it is sorted: if it is not, the second round
        // reads it linearly anyway.
        self.state = match self.tag_file_sorted as u8 {
            _ if self.linear => Reading::Linear,
            0 | b'1' => Reading::Binary,
            b'2' => {
                // Sorted with case folded.
                margs.sortic = true;
                self.orgpat.regmatch.rm_ic = p_ic.get() != 0 || !noic;
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
        if unsafe { fseeko(self.fp, 0, SEEK_END) } != 0 {
            // Cannot seek, so cannot bisect.
            self.state = Reading::Linear;
            return true;
        }
        // Don't use lseek(); it does not work properly on macOS
        // Catalina.
        let filesize = self.tell();
        vim_ignored.set(unsafe { fseeko(self.fp, 0, SEEK_SET) });
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
    ) -> TagMatch {
        // SAFETY: the buffer is NUL-terminated, `fp` is open, and every
        // pointer written into `tagp` points into the buffer.
        if self.orgpat.headlen == 0 {
            // No head to compare: take the line apart the slow way.
            return match unsafe { parse_tag_line(self.lbuf.as_mut_ptr(), tagp) } {
                true => TagMatch::Success,
                false => TagMatch::Fail,
            };
        }

        *tagp = TagParts::default();
        tagp.tagname = self.lbuf.as_mut_ptr();
        tagp.tagname_end = unsafe { vim_strchr(tagp.tagname, TAB) };
        if tagp.tagname_end.is_null() {
            // Corrupted tag line.
            return TagMatch::Fail;
        }

        // How much of the two names to compare.
        let mut cmplen = unsafe { tagp.tagname_end.offset_from(tagp.tagname) } as c_int;
        if p_tl.get() != 0 && cmplen as OptInt > p_tl.get() {
            cmplen = p_tl.get() as c_int;
        }
        if margs.has_re && self.orgpat.headlen < cmplen {
            cmplen = self.orgpat.headlen;
        } else if self.state == Reading::Linear && self.orgpat.headlen != cmplen {
            // A different length and no regexp: it cannot match.
            return TagMatch::Next;
        }
        let head = self.orgpat.head;
        debug_assert!(cmplen >= 0);
        let cmplen = cmplen as usize;

        match self.state {
            Reading::Binary => {
                // A first byte outside the range the bisection has
                // narrowed to means the file is not sorted after all.
                // Upstream folds case through a *signed* `char`, so a
                // byte above 0x7f reads negative here and unsigned in
                // the branch below it. Preserved.
                let signed = unsafe { *tagp.tagname } as c_int;
                let sort_key = if !margs.sortic {
                    unsafe { *tagp.tagname as u8 as c_int }
                } else if !(b'a' as c_int..=b'z' as c_int).contains(&signed) {
                    signed
                } else {
                    signed - (b'a' - b'A') as c_int
                };
                if sort_key < sinfo.low_char || sort_key > sinfo.high_char {
                    margs.sort_error = true;
                }

                let mut tagcmp = if margs.sortic {
                    unsafe { tag_strnicmp(tagp.tagname, head, cmplen) }
                } else {
                    unsafe { strncmp(tagp.tagname, head, cmplen) }
                };
                // A match on a shorter tag means to search forward, on
                // a longer one to search backward.
                if tagcmp == 0 {
                    tagcmp = (cmplen as c_int).cmp(&self.orgpat.headlen) as c_int;
                }

                if tagcmp == 0 {
                    // Found it. Skip back, then forward again, to land
                    // on the first tag that matches.
                    self.state = Reading::SkipBack;
                    sinfo.match_offset = sinfo.curr_offset;
                    return TagMatch::Next;
                }
                if tagcmp < 0 {
                    sinfo.curr_offset = self.tell();
                    if sinfo.curr_offset < sinfo.high_offset {
                        sinfo.low_offset = sinfo.curr_offset;
                        sinfo.low_char = sort_key;
                        return TagMatch::Next;
                    }
                }
                if tagcmp > 0 && sinfo.curr_offset != sinfo.high_offset {
                    sinfo.high_offset = sinfo.curr_offset;
                    sinfo.high_char = sort_key;
                    return TagMatch::Next;
                }
                // No match, and the range is exhausted.
                return TagMatch::Stop;
            }
            Reading::SkipBack => {
                if unsafe { mb_strnicmp(tagp.tagname, head, cmplen) } != 0 {
                    self.state = Reading::StepForward;
                } else {
                    // Have to skip back further. Put the offset the
                    // round started from back, or a long line leaves
                    // the walk stuck on it.
                    sinfo.curr_offset = sinfo.curr_offset_used;
                }
                return TagMatch::Next;
            }
            Reading::StepForward => {
                if unsafe { mb_strnicmp(tagp.tagname, head, cmplen) } != 0 {
                    return if self.tell() > sinfo.match_offset {
                        // Past the last match.
                        TagMatch::Stop
                    } else {
                        // Not yet at the first one.
                        TagMatch::Next
                    };
                }
            }
            _ => {
                if unsafe { mb_strnicmp(tagp.tagname, head, cmplen) } != 0 {
                    return TagMatch::Next;
                }
            }
        }

        // Could be the tag wanted: isolate the file name and command.
        tagp.fname = unsafe { tagp.tagname_end.add(1) };
        tagp.fname_end = unsafe { vim_strchr(tagp.fname, TAB) };
        if tagp.fname_end.is_null() {
            return TagMatch::Fail;
        }
        tagp.command = unsafe { tagp.fname_end.add(1) };
        TagMatch::Success
    }

    /// Whether the tag name the line holds is the one being looked for.
    ///
    /// The pattern is tried literally first — even when it is a regexp —
    /// and the regexp only afterwards, which is what tells the collector
    /// how good the match was.
    #[inline]
    pub(crate) fn match_tag(&mut self, tagp: &TagParts, margs: &mut MatchArgs) -> bool {
        // SAFETY: `tagname` and `tagname_end` bracket the name inside the
        // line buffer; the terminator written over `tagname_end` for the
        // regexp is put back before returning.
        let mut cmplen = unsafe { tagp.tagname_end.offset_from(tagp.tagname) } as c_int;
        if p_tl.get() != 0 && cmplen as OptInt > p_tl.get() {
            cmplen = p_tl.get() as c_int;
        }
        debug_assert!(cmplen >= 0);
        let len = cmplen as usize;
        let pat = self.orgpat.pat;

        // A name of a different length cannot match literally.
        let mut matched = self.orgpat.len == cmplen && {
            if self.orgpat.regmatch.rm_ic {
                let same = unsafe { mb_strnicmp(tagp.tagname, pat, len) } == 0;
                if same {
                    margs.match_no_ic = unsafe { strncmp(tagp.tagname, pat, len) } == 0;
                }
                same
            } else {
                unsafe { strncmp(tagp.tagname, pat, len) == 0 }
            }
        };

        // With a regexp, also find the tags it matches.
        margs.match_re = false;
        if !matched && !self.orgpat.regmatch.regprog.is_null() {
            let saved = unsafe { *tagp.tagname_end };
            unsafe { *tagp.tagname_end = 0 };
            matched = unsafe { vim_regexec(&raw mut self.orgpat.regmatch, tagp.tagname, 0) };
            if matched {
                margs.matchoff =
                    unsafe { self.orgpat.regmatch.startp[0].offset_from(tagp.tagname) } as c_int;
                if self.orgpat.regmatch.rm_ic {
                    // Ask again with case, to find out how good the
                    // match is.
                    self.orgpat.regmatch.rm_ic = false;
                    margs.match_no_ic =
                        unsafe { vim_regexec(&raw mut self.orgpat.regmatch, tagp.tagname, 0) };
                    self.orgpat.regmatch.rm_ic = true;
                }
            }
            unsafe { *tagp.tagname_end = saved };
            margs.match_re = true;
        }

        matched
    }
}
