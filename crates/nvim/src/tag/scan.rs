//! Scanning the tags files for matches.
//!
//! [`find_tags`] is the entry point every tag lookup goes through: it
//! walks the tags files that apply ([`TagFiles`](super::TagFiles)), reads
//! each one, and hands back the matching lines grouped by how good a match
//! they are. [`FindTags`] is the state it threads through the readers in
//! [`parse`](super::parse) and [`collect`](super::collect).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::file_search::Name;
use crate::options::{
    kOptTcFlagFollowic, kOptTcFlagFollowscs, kOptTcFlagIgnore, kOptTcFlagMatch, kOptTcFlagSmart,
};
use crate::pos::MAXCOL;
use crate::regexp::RE_MAGIC;
use crate::types::CONV_NONE;
#[allow(unused_imports)]
use crate::{semsg_c, smsg_c};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;
use std::collections::HashSet;

/// How many priority buckets a match can land in: four kinds of match,
/// doubled for an ignore-case match and again for a regexp match.
const MT_COUNT: usize = super::MT_COUNT as usize;
/// Static match in the current file.
const MT_ST_CUR: usize = super::MT_ST_CUR as usize;
/// Global match in the current file.
const MT_GL_CUR: usize = super::MT_GL_CUR as usize;
/// Global match in another file.
const MT_GL_OTH: usize = super::MT_GL_OTH as usize;
/// Static match in another file.
const MT_ST_OTH: usize = super::MT_ST_OTH as usize;
/// Added when the match only holds with case ignored.
const MT_IC_OFF: usize = super::MT_IC_OFF as usize;
/// Added when the match came from the regexp rather than from comparing
/// the pattern literally.
const MT_RE_OFF: usize = super::MT_RE_OFF as usize;

/// The line buffer starts this big and doubles whenever a line does not
/// fit in it.
const LSIZE: usize = super::LSIZE as usize;

/// How the tags file being read is being read.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reading {
    /// At the start of the file, where the header is.
    Start,
    /// Forwards to the end of the file, comparing every line.
    Linear,
    /// Bisecting a sorted file.
    Binary,
    /// Backwards from a bisection hit, to the first line that matches.
    SkipBack,
    /// Forwards from there, to the last one.
    StepForward,
}

/// The tag pattern, as the caller gave it and as it is searched for.
pub(crate) struct Pattern {
    /// What to look for. Points either at the caller's pattern or at the
    /// copy [`find_tags`] made when it stripped an `@xx` help language off
    /// the end.
    pub(crate) pat: *mut c_char,
    /// How much of `pat` counts, after `'taglength'` has had its say.
    pub(crate) len: c_int,
    /// The leading part of `pat` that holds no regexp metacharacter — the
    /// prefix a sorted tags file can be bisected on.
    pub(crate) head: *mut c_char,
    /// How much of `head` counts; zero when there is no usable prefix, in
    /// which case the file has to be read line by line.
    pub(crate) headlen: c_int,
    /// The compiled pattern, when the caller asked for a regexp.
    pub(crate) regmatch: regmatch_T,
}

impl Drop for Pattern {
    fn drop(&mut self) {
        // SAFETY: `regprog` came from `vim_regcomp`, or is NULL.
        unsafe { vim_regfree(self.regmatch.regprog) };
    }
}

impl Pattern {
    /// Take the pattern apart: what a bisection can compare on, and the
    /// compiled regexp when one was asked for.
    fn prepare(&mut self, has_re: bool) {
        self.head = self.pat;
        self.headlen = self.len;
        if !has_re {
            self.regmatch.regprog = ptr::null_mut();
            return;
        }
        // SAFETY: `pat` is the caller's NUL-terminated pattern.
        unsafe {
            // A pattern anchored with `^` or `\<` still has a plain head
            // after the anchor, which is what keeps bisection possible.
            if *self.pat == b'^' as c_char {
                self.head = self.pat.add(1);
            } else if *self.pat == b'\\' as c_char && *self.pat.add(1) == b'<' as c_char {
                self.head = self.pat.add(2);
            }
            if self.head == self.pat {
                self.headlen = 0;
            } else {
                // The head ends at the first metacharacter.
                let meta = if magic_isset() { c".[~*\\$" } else { c"\\$" };
                self.headlen = 0;
                loop {
                    let at = *self.head.offset(self.headlen as isize) as u8;
                    if at == 0 || !vim_strchr(meta.as_ptr(), at as c_int).is_null() {
                        break;
                    }
                    self.headlen += 1;
                }
            }
            if p_tl.get() != 0 && self.headlen as OptInt > p_tl.get() {
                self.headlen = p_tl.get() as c_int;
            }
            self.regmatch.regprog = vim_regcomp(self.pat, if magic_isset() { RE_MAGIC } else { 0 });
        }
    }
}

/// Where a bisection of a sorted tags file has got to.
#[derive(Clone, Copy, Default)]
pub(crate) struct SearchInfo {
    /// Offset of the first line that could still match.
    pub(crate) low_offset: off_T,
    /// Offset just past the last line that could still match.
    pub(crate) high_offset: off_T,
    /// Where in that range the file is being read.
    pub(crate) curr_offset: off_T,
    /// The `curr_offset` the current skip-back round started from; a long
    /// line would otherwise leave the walk stuck on it.
    pub(crate) curr_offset_used: off_T,
    /// Where the bisection found its match.
    pub(crate) match_offset: off_T,
    /// The first byte of the line at `low_offset`.
    pub(crate) low_char: c_int,
    /// The first byte of the line at `high_offset`. A line whose first
    /// byte falls outside that range means the file is not sorted after
    /// all.
    pub(crate) high_char: c_int,
}

/// What comparing one line against the pattern established.
pub(crate) struct MatchArgs {
    /// Where in the tag name the regexp matched.
    pub(crate) matchoff: c_int,
    /// The match came from the regexp, not from comparing the pattern
    /// literally.
    pub(crate) match_re: bool,
    /// The match holds even with case taken into account.
    pub(crate) match_no_ic: bool,
    /// A regexp is in use.
    pub(crate) has_re: bool,
    /// The tags file says it is sorted with case folded.
    pub(crate) sortic: bool,
    /// A line turned up that the file's claimed sort order forbids.
    pub(crate) sort_error: bool,
}

impl MatchArgs {
    fn new(flags: c_int) -> Self {
        MatchArgs {
            matchoff: 0,
            match_re: false,
            match_no_ic: false,
            has_re: flags & TAG_REGEXP as c_int != 0,
            sortic: false,
            sort_error: false,
        }
    }
}

/// One match, in the form [`find_tags`] hands it to its caller: an
/// allocation the caller will `xfree`.
///
/// Everything up to the first NUL is the key duplicates are found by; a
/// help match carries its sort heuristic after that NUL, and every kind
/// leaves a byte or two of slack at the end, as upstream does — the
/// consumers write into it.
pub(crate) struct Match {
    at: *mut u8,
    len: usize,
}

impl Match {
    /// Room for `len` bytes, all zero.
    #[inline]
    pub(crate) fn zeroed(len: usize) -> Self {
        // SAFETY: `len` bytes are allocated and immediately zeroed.
        unsafe {
            let at = xmalloc(len).cast::<u8>();
            at.write_bytes(0, len);
            Match { at, len }
        }
    }

    /// The whole allocation, slack included.
    #[inline]
    pub(crate) fn bytes(&mut self) -> &mut [u8] {
        // SAFETY: `len` bytes were allocated and zeroed at construction.
        unsafe { core::slice::from_raw_parts_mut(self.at, self.len) }
    }

    /// Where the match's own bytes are, for a [`Key`] into it.
    fn at(&self) -> *const u8 {
        self.at
    }

    /// Hand the allocation over to the caller of [`find_tags`].
    fn into_raw(self) -> *mut c_char {
        let at = self.at;
        core::mem::forget(self);
        at.cast()
    }
}

impl Drop for Match {
    fn drop(&mut self) {
        // SAFETY: the allocation came from `xmalloc` and is dropped once.
        unsafe { xfree(self.at.cast()) };
    }
}

/// What two matches are compared on: the bytes of a [`Match`] before its
/// first NUL.
///
/// This borrows rather than copies. The buffer belongs to a `Match` in the
/// same bucket and is a heap allocation, so it does not move when the
/// vector holding that `Match` grows; the set and the vector are emptied
/// together.
#[derive(Clone, Copy)]
struct Key(*const u8);

impl Key {
    fn bytes(&self) -> &[u8] {
        // SAFETY: every `Match` is NUL-terminated within what it wrote,
        // and outlives the key pointing at it.
        unsafe { CStr::from_ptr(self.0.cast()) }.to_bytes()
    }
}

impl core::hash::Hash for Key {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.bytes().hash(state);
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for Key {}

/// The keys of the matches in one bucket.
type Seen = HashSet<Key>;

/// The state one [`find_tags`] call threads through the readers.
pub(crate) struct FindTags {
    /// How the file being read is being read.
    pub(crate) state: Reading,
    /// Stop as soon as the current file is done — enough matches were
    /// found, the file was malformed, or the user interrupted.
    pub(crate) stop_searching: bool,
    /// What is being looked for.
    pub(crate) orgpat: Pattern,
    /// The line last read from the tags file. Its length is the buffer
    /// size `vim_fgets` is given, and it doubles whenever a line does not
    /// fit.
    pub(crate) lbuf: Vec<c_char>,
    /// The name of the tags file being read.
    pub(crate) tag_fname: Name,
    /// The tags file being read.
    pub(crate) fp: *mut FILE,
    /// The `TAG_*` flags the caller passed.
    pub(crate) flags: c_int,
    /// The `!_TAG_FILE_SORTED` value, NUL when the file had no header.
    pub(crate) tag_file_sorted: c_int,
    /// `'showfulltag'`: read the line already in the buffer again, this
    /// time for its search command rather than its name.
    pub(crate) get_searchpat: bool,
    /// Only help tags are wanted.
    pub(crate) help_only: bool,
    /// At least one tags file was opened; otherwise E433.
    did_open: bool,
    /// `MAXCOL` to find every match, otherwise how many are enough.
    mincount: c_int,
    /// Read the file line by line rather than bisecting it.
    pub(crate) linear: bool,
    /// The conversion the file's `!_TAG_FILE_ENCODING` asked for.
    pub(crate) vimconv: vimconv_T,
    /// The two-letter language of the tags file being read.
    pub(crate) help_lang: [u8; 2],
    /// How far down `'helplang'` that language is, which is what orders
    /// help matches.
    pub(crate) help_pri: c_int,
    /// The language the pattern's `@xx` suffix asked for, if any.
    help_lang_find: *const c_char,
    /// The current buffer is a `.txt` help file, whose language is "en".
    is_txt: bool,
    /// How many matches have been found, over every bucket.
    pub(crate) match_count: c_int,
    /// The matches, one bucket per priority, in the order found.
    found: [Vec<Match>; MT_COUNT],
    /// The keys already in `found`, so a duplicate can be dropped.
    seen: [Seen; MT_COUNT],
}

impl FindTags {
    fn new(pat: *mut c_char, flags: c_int, mincount: c_int) -> Self {
        FindTags {
            state: Reading::Start,
            stop_searching: false,
            orgpat: Pattern {
                pat,
                // SAFETY: the caller's pattern is NUL-terminated.
                len: unsafe { CStr::from_ptr(pat) }.count_bytes() as c_int,
                head: ptr::null_mut(),
                headlen: 0,
                regmatch: regmatch_T::default(),
            },
            lbuf: vec![0; LSIZE],
            tag_fname: Name::default(),
            fp: ptr::null_mut(),
            flags,
            tag_file_sorted: NUL,
            get_searchpat: false,
            help_only: flags & TAG_HELP as c_int != 0,
            did_open: false,
            mincount,
            linear: false,
            vimconv: vimconv_T {
                vc_type: CONV_NONE,
                vc_factor: 0,
                vc_fd: ptr::null_mut(),
                vc_fail: false,
            },
            help_lang: *b"\0\0",
            help_pri: 0,
            help_lang_find: ptr::null(),
            is_txt: false,
            match_count: 0,
            found: [const { Vec::new() }; MT_COUNT],
            seen: core::array::from_fn(|_| Seen::default()),
        }
    }

    /// File `mfp` under `bucket`, unless one with the same key is already
    /// there.
    #[inline]
    pub(crate) fn record(&mut self, bucket: usize, mfp: Match) {
        if self.seen[bucket].insert(Key(mfp.at())) {
            self.found[bucket].push(mfp);
            self.match_count += 1;
        }
    }

    /// Which of the sixteen buckets a match belongs in.
    #[inline]
    pub(crate) fn bucket(&self, is_static: bool, is_current: bool, margs: &MatchArgs) -> usize {
        let mut mtt = match (is_static, is_current) {
            (true, true) => MT_ST_CUR,
            (true, false) => MT_ST_OTH,
            (false, true) => MT_GL_CUR,
            (false, false) => MT_GL_OTH,
        };
        if self.orgpat.regmatch.rm_ic && !margs.match_no_ic {
            mtt += MT_IC_OFF;
        }
        if margs.match_re {
            mtt += MT_RE_OFF;
        }
        mtt
    }

    /// Work out the language and priority of the help tags file about to
    /// be read, answering false to skip the file entirely.
    fn in_help_init(&mut self) -> bool {
        // SAFETY: `tag_fname`, the current buffer's name and `'helplang'`
        // are all NUL-terminated.
        unsafe {
            // "doc/tags-xx" names its language; anything else, and a
            // ".txt" help file, is English.
            self.help_lang = match self.tag_fname.bytes() {
                _ if self.is_txt => *b"en",
                [_, .., b'-', a, b] => [*a, *b],
                _ => *b"en",
            };

            // When a language was asked for, skip every other one.
            if !self.help_lang_find.is_null() && !lang_is(self.help_lang_find, self.help_lang, true)
            {
                return false;
            }

            // For CTRL-] in a help file prefer a match in the same
            // language: a help file for language xx is named "*.xxx".
            let fname = (*curbuf.get()).b_fname;
            let flen = if fname.is_null() {
                0
            } else {
                CStr::from_ptr(fname).count_bytes()
            };
            if self.flags & TAG_KEEP_LANG as c_int != 0
                && self.help_lang_find.is_null()
                && flen > 4
                && *fname.add(flen - 1) == b'x' as c_char
                && *fname.add(flen - 4) == b'.' as c_char
                && lang_is(fname.add(flen - 3), self.help_lang, false)
            {
                self.help_pri = 0;
                return true;
            }

            // Otherwise the position in 'helplang' is the priority.
            self.help_pri = 1;
            let mut s = p_hlg.get();
            while *s != 0 {
                if lang_is(s, self.help_lang, false) {
                    break;
                }
                self.help_pri += 1;
                s = vim_strchr(s, b',' as c_int);
                if s.is_null() {
                    break;
                }
                s = s.add(1);
            }
            if s.is_null() || *s == 0 {
                // Not in 'helplang': sort last, but prefer English.
                self.help_pri += 1;
                if !self.help_lang.eq_ignore_ascii_case(b"en") {
                    self.help_pri += 1;
                }
            }
            true
        }
    }

    /// Let `'tagfunc'` answer instead of reading any tags file.
    ///
    /// Answers `NOTDONE` when there is no usable `'tagfunc'`, `OK` when it
    /// found at least one tag and `FAIL` otherwise.
    fn apply_tagfunc(&mut self, pat: *mut c_char, buf_ffname: *mut c_char) -> c_int {
        // SAFETY: `'tagfunc'` is a NUL-terminated buffer-local option, and
        // the growarray holds the allocated matches the callback made.
        unsafe {
            if self.flags & TAG_NO_TAGFUNC as c_int != 0
                || tfu_in_use.get()
                || *(*curbuf.get()).b_p_tfu == 0
            {
                return NOTDONE;
            }
            tfu_in_use.set(true);
            // `'tagfunc'` does its own filtering, so every answer is kept
            // and none of them is looked at for duplicates. Which bucket
            // they land in never reaches the caller: each one carries its
            // own priority in its first byte.
            let retval = find_tagfunc_tags(
                pat,
                &mut self.found[MT_ST_CUR],
                &mut self.match_count,
                self.flags,
                buf_ffname,
            );
            tfu_in_use.set(false);
            retval
        }
    }

    /// Read every matching tag out of the file named by `tag_fname`.
    fn in_file(&mut self, buf_ffname: *mut c_char) {
        self.vimconv.vc_type = CONV_NONE;
        self.tag_file_sorted = NUL;
        self.fp = ptr::null_mut();
        let mut margs = MatchArgs::new(self.flags);

        // SAFETY: `tag_fname` is NUL-terminated, and the file this opens
        // is closed before the block ends.
        unsafe {
            // A help tags file for another language is skipped entirely.
            if (*curbuf.get()).b_help && !self.in_help_init() {
                return;
            }

            // A file that does not exist is silently ignored; E433 is
            // given further on, and only when not one was found.
            self.fp = os_fopen(self.tag_fname.as_ptr(), c"r".as_ptr());
            if self.fp.is_null() {
                return;
            }
            if p_verbose.get() >= 5 {
                verbose_enter();
                smsg_c!(
                    0,
                    gettext(c"Searching tags file %s".as_ptr()),
                    self.tag_fname.as_ptr(),
                );
                verbose_leave();
            }
            self.did_open = true;
            self.state = Reading::Start;

            self.get_all_tags(&mut margs, buf_ffname);

            fclose(self.fp);
            self.fp = ptr::null_mut();
            if self.vimconv.vc_type != CONV_NONE {
                convert_setup(&raw mut self.vimconv, ptr::null_mut(), ptr::null_mut());
            }
            if margs.sort_error {
                semsg_c!(
                    gettext(c"E432: Tags file not sorted: %s".as_ptr()),
                    self.tag_fname.as_ptr(),
                );
            }
        }

        // Stop searching once enough tags have been found.
        if self.match_count >= self.mincount {
            self.stop_searching = true;
        }
    }

    /// Read and parse the lines of the open tags file, one by one.
    fn get_all_tags(&mut self, margs: &mut MatchArgs, buf_ffname: *mut c_char) {
        // SAFETY: `fp` is open for the whole loop. Every pointer `tagp`
        // holds points into `lbuf`, which is only replaced at the two
        // points below where nothing holds one.
        unsafe {
            let mut tagp = TagParts::default();
            let mut sinfo = SearchInfo::default();

            loop {
                // Check for CTRL-C, more often when jumping around.
                if matches!(self.state, Reading::Binary | Reading::SkipBack) {
                    line_breakcheck();
                } else {
                    fast_breakcheck();
                }
                if self.flags & TAG_INS_COMP as c_int != 0 {
                    ins_compl_check_keys(30, false);
                }
                if got_int.get() || ins_compl_interrupted() {
                    self.stop_searching = true;
                    break;
                }
                // For completion, stop once there are plenty.
                if self.mincount == TAG_MANY as c_int && self.match_count >= TAG_MANY as c_int {
                    self.stop_searching = true;
                    break;
                }

                // `'showfulltag'` re-reads the line already in the buffer.
                if !self.get_searchpat {
                    match self.next_line(&mut sinfo) {
                        Line::Ignore => continue,
                        Line::Eof => break,
                        Line::Read => {}
                    }
                }

                if self.vimconv.vc_type != CONV_NONE {
                    self.convert_line();
                }

                // While still at the start of the file, read the header.
                if self.state == Reading::Start && !self.start_state(margs, &mut sinfo) {
                    continue;
                }

                // A line that did not fit leaves the NUL somewhere other
                // than the last-but-one byte (see `vim_fgets`). Reported
                // for Mozilla JS, which has extremely long names.
                if self.lbuf[self.lbuf.len() - 2] != NUL as c_char {
                    self.lbuf = vec![0; self.lbuf.len() * 2];
                    if matches!(self.state, Reading::StepForward | Reading::Linear) {
                        // Seek back to read the same line again.
                        vim_ignored.set(fseeko(self.fp, sinfo.curr_offset, SEEK_SET));
                    }
                    // The offset has to differ, or the retry reads the
                    // same line into the same too-small buffer.
                    sinfo.curr_offset = 0;
                    continue;
                }

                match self.parse_line(&mut tagp, margs, &mut sinfo) {
                    TagMatch::Next => continue,
                    TagMatch::Stop => break,
                    TagMatch::Fail => {
                        semsg_c!(
                            gettext(c"E431: Format error in tags file \"%s\"".as_ptr()),
                            self.tag_fname.as_ptr(),
                        );
                        semsg_c!(
                            gettext(c"Before byte %ld".as_ptr()),
                            ftello(self.fp) as int64_t,
                        );
                        self.stop_searching = true;
                        return;
                    }
                    TagMatch::Success => {}
                }

                if self.match_tag(&tagp, margs) {
                    self.add_match(&tagp, margs, buf_ffname);
                }
            }
        }
    }

    /// Hand the matches to the caller, best priority first.
    ///
    /// Answers how many there are; `matchesp` is left NULL when there are
    /// none.
    ///
    /// # Safety
    /// `matchesp` must be writable; the array and its entries become the
    /// caller's to free.
    unsafe fn into_matches(mut self, matchesp: *mut *mut *mut c_char) -> c_int {
        let name_only = self.flags & TAG_NAMES as c_int != 0;
        // The keys point into the matches, so they go first.
        self.seen = core::array::from_fn(|_| Seen::default());
        // SAFETY: `matches` is `match_count` pointers and every one of
        // them is filled before the array is handed over.
        unsafe {
            if self.match_count <= 0 {
                *matchesp = ptr::null_mut();
                return 0;
            }
            let matches =
                xmalloc(self.match_count as usize * size_of::<*mut c_char>()).cast::<*mut c_char>();
            let mut at = 0;
            for bucket in core::mem::take(&mut self.found) {
                for mut mfp in bucket {
                    if !name_only {
                        // Put the bucket number back the way the readers
                        // want it, and the field separators back to NUL.
                        // The walk stops at the match's own terminator,
                        // short of the slack after it.
                        let bytes = mfp.bytes();
                        bytes[0] -= 1;
                        for byte in &mut bytes[1..] {
                            match *byte {
                                0 => break,
                                b if b == TAG_SEP as u8 => *byte = NUL as u8,
                                _ => {}
                            }
                        }
                    }
                    *matches.add(at) = mfp.into_raw();
                    at += 1;
                }
            }
            debug_assert_eq!(at, self.match_count as usize);
            *matchesp = matches;
        }
        self.match_count
    }
}

/// Whether the NUL-terminated `at` names `lang`.
///
/// With `whole`, `at` must be exactly the two letters, which is how the
/// pattern's `@xx` suffix is compared; otherwise only the two letters have
/// to line up, which is how a file name's suffix and a `'helplang'` entry
/// are.
///
/// # Safety
/// `at` must be NUL-terminated.
unsafe fn lang_is(at: *const c_char, lang: [u8; 2], whole: bool) -> bool {
    // SAFETY: the second byte is only read once the first has proved not
    // to be the terminator — `lang` never holds one.
    unsafe {
        (*at as u8).eq_ignore_ascii_case(&lang[0])
            && (*at.add(1) as u8).eq_ignore_ascii_case(&lang[1])
            && (!whole || *at.add(2) == 0)
    }
}

/// A NUL-terminated string as an owned [`Match`], terminator included.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn match_of(p: *const c_char) -> Match {
    // SAFETY: the caller's promise.
    unsafe {
        let bytes = CStr::from_ptr(p).to_bytes_with_nul();
        let mut mfp = Match::zeroed(bytes.len());
        mfp.bytes().copy_from_slice(bytes);
        mfp
    }
}

/// Search the tags files that apply for tags matching `pat`.
///
/// Answers `FAIL` if the search failed completely — `num_matches` is then
/// zero and `matchesp` NULL — and `OK` otherwise.
///
/// There is a priority in which a kind of match is recognised:
///
/// 6. a static or global tag fully matching, in the current file;
/// 5. a global tag fully matching, in another file;
/// 4. a static tag fully matching, in another file;
/// 3. a static or global tag matching but for case, in the current file;
/// 2. a global tag matching but for case, in another file;
/// 1. a static tag matching but for case, in another file.
///
/// `flags` is a set of `TAG_HELP` (only help tags), `TAG_NAMES` (answer
/// only the names), `TAG_REGEXP` (`pat` is a regexp), `TAG_NOIC` (do not
/// always ignore case), `TAG_KEEP_LANG` (keep the help language) and
/// `TAG_NO_TAGFUNC` (do not call `'tagfunc'`).
///
/// `mincount` is `MAXCOL` to find every match, otherwise how many are
/// enough. `buf_ffname` is the buffer whose matches take priority.
///
/// # Safety
/// `pat` must be NUL-terminated and the two out-parameters writable.
pub unsafe fn find_tags(
    pat: *mut c_char,
    num_matches: *mut c_int,
    matchesp: *mut *mut *mut c_char,
    flags: c_int,
    mincount: c_int,
    buf_ffname: *mut c_char,
) -> c_int {
    // SAFETY: the caller's pattern outlives the search, and `saved_pat` is
    // declared before the state that points into it, so it is dropped
    // after it.
    unsafe {
        // Find every match when the caller wants them all, and also for
        // completion, where the count is only a cut-off.
        let findall = mincount == MAXCOL as c_int || mincount == TAG_MANY as c_int;
        let has_re = flags & TAG_REGEXP as c_int != 0;
        let noic = flags & TAG_NOIC as c_int != 0;

        // 'tagcase' decides how case is treated for this search.
        let save_p_ic = p_ic.get();
        let tagcase = match (*curbuf.get()).b_tc_flags {
            0 => tc_flags.get(),
            local => local,
        };
        match tagcase {
            kOptTcFlagFollowic => {}
            kOptTcFlagIgnore => p_ic.set(true_0),
            kOptTcFlagMatch => p_ic.set(false_0),
            kOptTcFlagFollowscs => p_ic.set(ignorecase(pat)),
            kOptTcFlagSmart => p_ic.set(ignorecase_opt(pat, true_0, true_0)),
            _ => abort(),
        }

        let help_save = (*curbuf.get()).b_help;
        let saved_pat: Option<Name>;
        let mut st = FindTags::new(pat, flags, mincount);
        if st.help_only {
            (*curbuf.get()).b_help = true;
        }

        // In a help buffer a trailing "@xx" names the language wanted.
        let bytes = CStr::from_ptr(pat).to_bytes();
        saved_pat = if (*curbuf.get()).b_help
            && let [.., b'@', a, b] = bytes
            && a.is_ascii_alphabetic()
            && b.is_ascii_alphabetic()
        {
            let stripped = Name::from_bytes(&bytes[..bytes.len() - 3]);
            st.help_lang_find = pat.add(bytes.len() - 2);
            st.orgpat.pat = stripped.as_ptr().cast_mut();
            st.orgpat.len -= 3;
            Some(stripped)
        } else {
            None
        };

        if p_tl.get() != 0 && st.orgpat.len as OptInt > p_tl.get() {
            st.orgpat.len = p_tl.get() as c_int;
        }

        // A pattern that does not compile is the caller's problem, not a
        // message from here.
        let save_emsg_off = emsg_off.get();
        emsg_off.set(true_0);
        st.orgpat.prepare(has_re);
        emsg_off.set(save_emsg_off);

        let mut retval = FAIL;
        if !(has_re && st.orgpat.regmatch.regprog.is_null()) {
            retval = st.apply_tagfunc(pat, buf_ffname);
            if retval == NOTDONE {
                retval = FAIL;
                // A ".txt" help file keeps "en" as its language.
                let fname = (*curbuf.get()).b_fname;
                if flags & TAG_KEEP_LANG as c_int != 0
                    && st.help_lang_find.is_null()
                    && !fname.is_null()
                {
                    let len = CStr::from_ptr(fname).count_bytes();
                    st.is_txt =
                        len > 4 && strcasecmp(fname.add(len - 4), c".txt".as_ptr().cast_mut()) == 0;
                }

                // Ignoring case rules out bisection, so a search that may
                // ignore case reads every file twice: once matching case,
                // and again ignoring it if nothing turned up.
                st.orgpat.regmatch.rm_ic = (p_ic.get() != 0 || !noic)
                    && (findall || st.orgpat.headlen == 0 || p_tbs.get() == 0);
                for round in 1..=2 {
                    st.linear = st.orgpat.headlen == 0 || p_tbs.get() == 0 || round == 2;

                    let mut files = TagFiles::new();
                    while let Some(name) = files.next() {
                        st.tag_fname = name;
                        st.in_file(buf_ffname);
                        if st.stop_searching {
                            retval = OK;
                            break;
                        }
                    }
                    drop(files);

                    if st.stop_searching
                        || st.linear
                        || (p_ic.get() == 0 && noic)
                        || st.orgpat.regmatch.rm_ic
                    {
                        break;
                    }
                    st.orgpat.regmatch.rm_ic = true;
                }

                if !st.stop_searching {
                    if !st.did_open && flags & TAG_VERBOSE as c_int != 0 {
                        emsg(gettext(c"E433: No tags file".as_ptr()));
                    }
                    retval = OK;
                }
            }
        }

        if retval == FAIL {
            st.match_count = 0;
        }
        *num_matches = st.into_matches(matchesp);

        (*curbuf.get()).b_help = help_save;
        p_ic.set(save_p_ic);
        drop(saved_pat);
        retval
    }
}
