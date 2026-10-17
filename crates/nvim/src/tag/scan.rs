//! Scanning the tags files for matches.
//!
//! [`find_tags`] is the entry point every tag lookup goes through: it
//! walks the tags files that apply ([`TagFiles`](super::TagFiles)), reads
//! each one, and hands back the matching lines grouped by how good a match
//! they are. [`FindTags`] is the state it threads through the readers in
//! [`parse`](super::parse) and [`collect`](super::collect).
//! [`find_tag_names`] and [`find_help_tag_matches`] are the same search for
//! the two callers that want something other than whole lines.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::file_search::Name;
use crate::guard::Suppress;
use crate::message_fmt::msg_cstr;
use crate::option::vars::P_HLG;
use crate::options::{
    kOptTcFlagFollowic, kOptTcFlagFollowscs, kOptTcFlagIgnore, kOptTcFlagMatch, kOptTcFlagSmart,
};
use crate::optionstr::OptString;
use crate::pos::MAXCOL;
use crate::regexp::RE_MAGIC;
use crate::search::ignorecase_opt;
use crate::semsg;
use crate::smsg;
use crate::types::RegMatch;
use crate::types::{CONV_NONE, CONV_NONE_INIT, FAIL, Failed, OK};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};
use core::hash::{Hash, Hasher};
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

/// The tag pattern, as it is searched for.
pub(crate) struct Pattern {
    /// What to look for: the caller's pattern, less the `@xx` help
    /// language [`find_tags`] stripped off the end.
    pub(crate) pat: XString,
    /// How much of `pat` counts, after `'taglength'` has had its say.
    pub(crate) len: usize,
    /// Where the leading part of `pat` that holds no regexp metacharacter
    /// starts — the prefix a sorted tags file can be bisected on.
    head_at: usize,
    /// How much of the head counts; zero when there is no usable prefix, in
    /// which case the file has to be read line by line.
    pub(crate) headlen: usize,
    /// The compiled pattern when the caller asked for a regexp. Its
    /// `rm_ic` is the search's "ignore case" whether or not there is one.
    pub(crate) regmatch: RegMatch,
}

impl Pattern {
    /// The pattern from its head on.
    pub(crate) fn head(&self) -> &[u8] {
        &self.pat[self.head_at..]
    }

    /// Take the pattern apart: what a bisection can compare on, and the
    /// compiled regexp when one was asked for.
    fn prepare(&mut self, has_re: bool) {
        self.head_at = 0;
        self.headlen = self.len;
        if !has_re {
            return;
        }
        // A pattern anchored with `^` or `\<` still has a plain head
        // after the anchor, which is what keeps bisection possible.
        let pat: &[u8] = &self.pat;
        if pat.first() == Some(&b'^') {
            self.head_at = 1;
        } else if pat.starts_with(b"\\<") {
            self.head_at = 2;
        }
        self.headlen = if self.head_at == 0 {
            0
        } else {
            // The head ends at the first metacharacter.
            let meta: &[u8] = if magic_isset() { b".[~*\\$" } else { b"\\$" };
            pat[self.head_at..]
                .iter()
                .position(|b| meta.contains(b))
                .unwrap_or(pat.len() - self.head_at)
        };
        if let Ok(limit) = usize::try_from(p_tl())
            && limit != 0
            && self.headlen > limit
        {
            self.headlen = limit;
        }
        let flags = if magic_isset() { RE_MAGIC } else { 0 };
        self.regmatch = RegMatch::compile(self.pat.as_cstr(), flags, false).unwrap_or_default();
    }
}

/// Where a bisection of a sorted tags file has got to.
#[derive(Default)]
pub(crate) struct SearchInfo {
    /// Offset of the first line that could still match.
    pub(crate) low_offset: FileOffset,
    /// Offset just past the last line that could still match.
    pub(crate) high_offset: FileOffset,
    /// Where in that range the file is being read.
    pub(crate) curr_offset: FileOffset,
    /// The `curr_offset` the current skip-back round started from; a long
    /// line would otherwise leave the walk stuck on it.
    pub(crate) curr_offset_used: FileOffset,
    /// Where the bisection found its match.
    pub(crate) match_offset: FileOffset,
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

/// One match, as the search keeps it until it hands the list over.
///
/// Two matches are the same match when their keys are equal; the score
/// and the order found take no part in that.
pub(crate) struct Found {
    /// When it was found, which orders a bucket.
    seq: usize,
    /// The match itself: `<tags file>NUL<line>` for a whole line, the name
    /// (or a `'showfulltag'` pattern) alone, or `<name>@<lang>` for help.
    pub(crate) key: Vec<u8>,
    /// A help match's heuristic score, which orders the help listing.
    pub(crate) score: Vec<u8>,
    /// The `MT_*` priority the caller is told: the bucket, except for a
    /// `'tagfunc'` answer, which carries its own.
    pub(crate) priority: u8,
}

impl Found {
    pub(crate) fn new(key: Vec<u8>, priority: usize) -> Self {
        Found {
            seq: 0,
            key,
            score: Vec::new(),
            priority: u8::try_from(priority).expect("an MT_* bucket"),
        }
    }
}

impl Hash for Found {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for Found {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Found {}

/// A match [`find_tags`] answered: one line of a tags file — or a
/// `'tagfunc'` answer in that shape — and the file it came from.
#[derive(Clone)]
pub(crate) struct TagMatch {
    /// The `MT_*` priority it was filed under.
    pub(crate) priority: u8,
    /// `<tags file name>NUL<line>NUL`.
    text: Vec<u8>,
    /// Where the line starts in `text`.
    line_at: usize,
}

impl TagMatch {
    fn new(priority: u8, mut text: Vec<u8>) -> Self {
        let line_at = cstr::until_nul(&text).len() + 1;
        // The list upstream hands back separates the two parts with 0x02
        // and turns *every* 0x02 back into a NUL, so a line reads only up
        // to the first one it holds.
        if let Some(sep) = text[line_at..].iter().position(|&b| b == TAG_SEP as u8) {
            text.truncate(line_at + sep);
        }
        text.push(0);
        TagMatch {
            priority,
            text,
            line_at,
        }
    }

    /// The name of the tags file the line came from; empty for a
    /// `'tagfunc'` answer.
    pub(crate) fn tag_fname(&self) -> &[u8] {
        &self.text[..self.line_at - 1]
    }

    /// The line, without its terminator.
    pub(crate) fn line(&self) -> &[u8] {
        &self.text[self.line_at..self.text.len() - 1]
    }

    /// The line and its terminator, for the readers that walk to it.
    pub(crate) fn line_nul(&self) -> &[u8] {
        &self.text[self.line_at..]
    }

    /// The line's fields; see [`parse_match`].
    pub(crate) fn parse(&self) -> (TagParts, bool) {
        parse_match(self.line_nul())
    }
}

/// The matches in one bucket.
type Seen = HashSet<Found>;

/// The state one [`find_tags`] call threads through the readers.
pub(crate) struct FindTags {
    /// How the file being read is being read.
    pub(crate) state: Reading,
    /// Stop as soon as the current file is done — enough matches were
    /// found, the file was malformed, or the user interrupted.
    pub(crate) stop_searching: bool,
    /// What is being looked for.
    pub(crate) orgpat: Pattern,
    /// The buffer the last line was read into. Its length is the size the
    /// reader is given, and it doubles whenever a line does not fit.
    pub(crate) lbuf: Vec<u8>,
    /// How long the line in `lbuf` is, up to its terminator.
    pub(crate) line_len: usize,
    /// The name of the tags file being read.
    pub(crate) tag_fname: Name,
    /// The tags file being read.
    pub(crate) reader: Option<TagReader>,
    /// The `TAG_*` flags the caller passed.
    pub(crate) flags: c_int,
    /// The `!_TAG_FILE_SORTED` value, NUL when the file had no header.
    pub(crate) tag_file_sorted: u8,
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
    pub(crate) vimconv: VimConv,
    /// The two-letter language of the tags file being read.
    pub(crate) help_lang: [u8; 2],
    /// How far down `'helplang'` that language is, which is what orders
    /// help matches.
    pub(crate) help_pri: c_int,
    /// The language the pattern's `@xx` suffix asked for, if any.
    help_lang_find: Option<[u8; 2]>,
    /// The current buffer is a `.txt` help file, whose language is "en".
    is_txt: bool,
    /// How many matches have been found, over every bucket.
    pub(crate) match_count: c_int,
    /// The matches, one set per priority.
    found: [Seen; MT_COUNT],
    /// What `'tagfunc'` answered: every one kept, in order, none of them
    /// looked at for duplicates.
    from_tagfunc: Vec<Found>,
    /// How many matches have been offered so far, which orders a bucket.
    seq: usize,
}

impl FindTags {
    /// The state for one search for `pat`.
    fn new(pat: XString, flags: c_int, mincount: c_int) -> Self {
        FindTags {
            state: Reading::Start,
            stop_searching: false,
            orgpat: Pattern {
                len: pat.len(),
                pat,
                head_at: 0,
                headlen: 0,
                regmatch: RegMatch::default(),
            },
            lbuf: vec![0; LSIZE],
            line_len: 0,
            tag_fname: Name::default(),
            reader: None,
            flags,
            tag_file_sorted: 0,
            get_searchpat: false,
            help_only: flags & TAG_HELP as c_int != 0,
            did_open: false,
            mincount,
            linear: false,
            vimconv: VimConv {
                vc_type: CONV_NONE,
                vc_factor: 0,
                vc_fd: None,
                vc_fail: false,
            },
            help_lang: *b"\0\0",
            help_pri: 0,
            help_lang_find: None,
            is_txt: false,
            match_count: 0,
            found: core::array::from_fn(|_| Seen::default()),
            from_tagfunc: Vec::new(),
            seq: 0,
        }
    }

    /// File `found` under `bucket`, unless an equal one is already there.
    #[inline]
    pub(crate) fn record(&mut self, bucket: usize, mut found: Found) {
        found.seq = self.seq;
        self.seq += 1;
        if self.found[bucket].insert(found) {
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
        // "doc/tags-xx" names its language; anything else, and a
        // ".txt" help file, is English.
        self.help_lang = match self.tag_fname.bytes() {
            _ if self.is_txt => *b"en",
            [_, .., b'-', a, b] => [*a, *b],
            _ => *b"en",
        };

        // When a language was asked for, skip every other one.
        if let Some(wanted) = self.help_lang_find
            && !wanted.eq_ignore_ascii_case(&self.help_lang)
        {
            return false;
        }

        // For CTRL-] in a help file prefer a match in the same
        // language: a help file for language xx is named "*.xxx".
        let buffer = Buf::current();
        let fname = buffer.name.shown().map_or(&[][..], CStr::to_bytes);
        let flen = fname.len();
        if self.flags & TAG_KEEP_LANG as c_int != 0
            && self.help_lang_find.is_none()
            && flen > 4
            && fname[flen - 1] == b'x'
            && fname[flen - 4] == b'.'
            && lang_is(&fname[flen - 3..], self.help_lang)
        {
            self.help_pri = 0;
            return true;
        }

        // Otherwise the position in 'helplang' is the priority.
        self.help_pri = 1;
        let hlg = P_HLG.get();
        let hlg: &[u8] = &hlg;
        let mut at = Some(0);
        while let Some(entry) = at.and_then(|at| hlg.get(at..)).filter(|e| !e.is_empty()) {
            if lang_is(entry, self.help_lang) {
                break;
            }
            self.help_pri += 1;
            at = at
                .zip(entry.iter().position(|&b| b == b','))
                .map(|(at, comma)| at + comma + 1);
        }
        if at.is_none_or(|at| at >= hlg.len()) {
            // Not in 'helplang': sort last, but prefer English.
            self.help_pri += 1;
            if !self.help_lang.eq_ignore_ascii_case(b"en") {
                self.help_pri += 1;
            }
        }
        true
    }

    /// Let `'tagfunc'` answer instead of reading any tags file.
    ///
    /// Answers `NOTDONE` when there is no usable `'tagfunc'`, `OK` when it
    /// answered and `FAIL` otherwise.
    fn apply_tagfunc(&mut self, pat: &CStr, buf_ffname: Option<&CStr>) -> c_int {
        if self.flags & TAG_NO_TAGFUNC as c_int != 0
            || TAGFUNC_RUNNING.get()
            || Buf::current().b_p_tfu.first_byte() == 0
        {
            return NOTDONE;
        }
        TAGFUNC_RUNNING.set(true);
        // `'tagfunc'` does its own filtering, so every answer is kept
        // and none of them is looked at for duplicates.
        let answer = find_tagfunc_tags(pat, self.flags, buf_ffname);
        TAGFUNC_RUNNING.set(false);
        match answer {
            TagfuncAnswer::Declined => NOTDONE,
            TagfuncAnswer::Unusable => FAIL,
            TagfuncAnswer::Tags(tags) => {
                self.match_count = c_int::try_from(tags.len()).unwrap_or(c_int::MAX);
                self.from_tagfunc = tags;
                OK
            }
        }
    }

    /// Read every matching tag out of the file named by `tag_fname`.
    fn in_file(&mut self, buf_ffname: Option<&CStr>) {
        self.vimconv.vc_type = CONV_NONE;
        self.tag_file_sorted = 0;
        let mut margs = MatchArgs::new(self.flags);

        // A help tags file for another language is skipped entirely.
        if Buf::current().b_help && !self.in_help_init() {
            return;
        }

        // A file that does not exist is silently ignored; E433 is
        // given further on, and only when not one was found.
        self.reader = TagReader::open(self.tag_fname.as_cstr());
        if self.reader.is_none() {
            return;
        }
        if p_verbose() >= 5 {
            verbose_enter();
            let tag_fname = msg_cstr(self.tag_fname.as_cstr());
            smsg!(0, "Searching tags file {tag_fname}");
            verbose_leave();
        }
        self.did_open = true;
        self.state = Reading::Start;

        self.get_all_tags(&mut margs, buf_ffname);

        self.reader = None;
        if self.vimconv.vc_type != CONV_NONE {
            self.vimconv = CONV_NONE_INIT;
        }
        if margs.sort_error {
            let tag_fname = msg_cstr(self.tag_fname.as_cstr());
            semsg!("E432: Tags file not sorted: {tag_fname}");
        }

        // Stop searching once enough tags have been found.
        if self.match_count >= self.mincount {
            self.stop_searching = true;
        }
    }

    /// Read and parse the lines of the open tags file, one by one.
    fn get_all_tags(&mut self, margs: &mut MatchArgs, buf_ffname: Option<&CStr>) {
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
            if self.lbuf[self.lbuf.len() - 2] != 0 {
                self.lbuf = vec![0; self.lbuf.len() * 2];
                self.line_len = 0;
                if matches!(self.state, Reading::StepForward | Reading::Linear) {
                    // Seek back to read the same line again.
                    self.seek(sinfo.curr_offset);
                }
                // The offset has to differ, or the retry reads the
                // same line into the same too-small buffer.
                sinfo.curr_offset = 0;
                continue;
            }

            match self.parse_line(&mut tagp, margs, &mut sinfo) {
                Verdict::Next => continue,
                Verdict::Stop => break,
                Verdict::Fail => {
                    let tag_fname = msg_cstr(self.tag_fname.as_cstr());
                    semsg!("E431: Format error in tags file \"{tag_fname}\"");
                    let at = self.tell();
                    semsg!("Before byte {at}");
                    self.stop_searching = true;
                    return;
                }
                Verdict::Success => {}
            }

            if self.match_tag(&tagp, margs) {
                self.add_match(&tagp, margs, buf_ffname);
            }
        }
    }

    /// Hand the matches over, best priority first and in the order found
    /// within a priority.
    fn into_found(self) -> Vec<Found> {
        let mut out = self.from_tagfunc;
        out.reserve(usize::try_from(self.match_count).unwrap_or(0));
        for bucket in self.found {
            let start = out.len();
            out.extend(bucket);
            out[start..].sort_unstable_by_key(|found| found.seq);
        }
        out
    }
}

/// Whether `at` starts with the two letters of `lang`, ignoring case.
fn lang_is(at: &[u8], lang: [u8; 2]) -> bool {
    cstr::byte_at(at, 0).eq_ignore_ascii_case(&lang[0])
        && cstr::byte_at(at, 1).eq_ignore_ascii_case(&lang[1])
}

/// Search the tags files that apply for tags matching `pat`, answering the
/// matches in the shape `flags` asks for.
///
/// Answers `Err` if the search failed completely, and `Ok` otherwise.
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
/// enough. `buf_ffname` is the buffer whose matches take priority; it is the
/// caller's own copy, because `'tagfunc'` may rename the buffer.
fn search(
    pat: &[u8],
    flags: c_int,
    mincount: c_int,
    buf_ffname: Option<&CStr>,
) -> Result<Vec<Found>, Failed> {
    // Find every match when the caller wants them all, and also for
    // completion, where the count is only a cut-off.
    let findall = mincount == MAXCOL as c_int || mincount == TAG_MANY as c_int;
    let has_re = flags & TAG_REGEXP as c_int != 0;
    let noic = flags & TAG_NOIC as c_int != 0;
    // The pattern as the caller gave it, for 'tagcase' and 'tagfunc'.
    let pattern = XString::from_bytes(cstr::until_nul(pat));

    let save_p_ic = p_ic();
    // 'tagcase' decides how case is treated for this search.
    let tagcase = match Buf::current().b_tc_flags {
        0 => tc_flags.get(),
        local => local,
    };
    if tagcase == kOptTcFlagIgnore {
        P_IC.set(true);
    } else if tagcase == kOptTcFlagMatch {
        P_IC.set(false);
    } else if tagcase == kOptTcFlagFollowscs {
        P_IC.set(ignorecase_opt(&pattern, p_ic(), p_scs()));
    } else if tagcase == kOptTcFlagSmart {
        P_IC.set(ignorecase_opt(&pattern, true, true));
    } else {
        assert_eq!(
            tagcase, kOptTcFlagFollowic,
            "'tagcase' is one of its five values"
        );
    }

    let help_save = Buf::current().b_help;
    if flags & TAG_HELP as c_int != 0 {
        Buf::current().b_help = true;
    }

    // In a help buffer a trailing "@xx" names the language wanted.
    let bytes: &[u8] = &pattern;
    let (searched, help_lang_find) = if Buf::current().b_help
        && let [.., b'@', a, b] = bytes
        && a.is_ascii_alphabetic()
        && b.is_ascii_alphabetic()
    {
        (
            XString::from_bytes(&bytes[..bytes.len() - 3]),
            Some([*a, *b]),
        )
    } else {
        (pattern.clone(), None)
    };
    let mut st = FindTags::new(searched, flags, mincount);
    st.help_lang_find = help_lang_find;

    if let Ok(limit) = usize::try_from(p_tl())
        && limit != 0
        && st.orgpat.len > limit
    {
        st.orgpat.len = limit;
    }

    // A pattern that does not compile is the caller's problem, not a
    // message from here.
    let no_emsg = Suppress::emsg_outright();
    st.orgpat.prepare(has_re);
    drop(no_emsg);

    let mut retval = Err(Failed);
    if !(has_re && st.orgpat.regmatch.regprog.is_none()) {
        // `apply_tagfunc` still answers the C's three statuses; `NOTDONE`
        // means there is no 'tagfunc' and the tag files are searched below.
        let tagfunc = st.apply_tagfunc(pattern.as_cstr(), buf_ffname);
        retval = if tagfunc == OK { Ok(()) } else { Err(Failed) };
        if tagfunc == NOTDONE {
            // A ".txt" help file keeps "en" as its language.
            if flags & TAG_KEEP_LANG as c_int != 0 && st.help_lang_find.is_none() {
                let buffer = Buf::current();
                let fname = buffer.name.shown().map_or(&[][..], CStr::to_bytes);
                st.is_txt =
                    fname.len() > 4 && fname[fname.len() - 4..].eq_ignore_ascii_case(b".txt");
            }

            // Ignoring case rules out bisection, so a search that may
            // ignore case reads every file twice: once matching case,
            // and again ignoring it if nothing turned up.
            st.orgpat.regmatch.rm_ic =
                (p_ic() || !noic) && (findall || st.orgpat.headlen == 0 || !p_tbs());
            for round in 1..=2 {
                st.linear = st.orgpat.headlen == 0 || !p_tbs() || round == 2;

                let mut files = TagFiles::new();
                while let Some(name) = files.next() {
                    st.tag_fname = name;
                    st.in_file(buf_ffname);
                    if st.stop_searching {
                        retval = Ok(());
                        break;
                    }
                }
                drop(files);

                if st.stop_searching || st.linear || (!p_ic() && noic) || st.orgpat.regmatch.rm_ic {
                    break;
                }
                st.orgpat.regmatch.rm_ic = true;
            }

            if !st.stop_searching {
                if !st.did_open && flags & TAG_VERBOSE as c_int != 0 {
                    tag_emsg(c"E433: No tags file");
                }
                retval = Ok(());
            }
        }
    }

    let found = if retval.is_ok() {
        st.into_found()
    } else {
        Vec::new()
    };

    Buf::current().b_help = help_save;
    P_IC.set(save_p_ic);
    retval.map(|()| found)
}

/// Search the tags files that apply for tags matching `pat`: the lines
/// that matched, best first. See [`search`] for the order and the flags;
/// `TAG_NAMES` and `TAG_HELP` have callers of their own.
pub(crate) fn find_tags(
    pat: &[u8],
    flags: c_int,
    mincount: c_int,
    buf_ffname: Option<&CStr>,
) -> Result<Vec<TagMatch>, Failed> {
    debug_assert_eq!(flags & (TAG_NAMES | TAG_HELP) as c_int, 0);
    let found = search(pat, flags, mincount, buf_ffname)?;
    Ok(found
        .into_iter()
        .map(|found| TagMatch::new(found.priority, found.key))
        .collect())
}

/// [`find_tags`] for the names alone — and, with `'showfulltag'` during
/// insert-mode completion, each name's search pattern after it.
pub(crate) fn find_tag_names(
    pat: &[u8],
    flags: c_int,
    mincount: c_int,
    buf_ffname: Option<&CStr>,
) -> Result<Vec<XString>, Failed> {
    let found = search(pat, flags | TAG_NAMES as c_int, mincount, buf_ffname)?;
    Ok(found
        .into_iter()
        .map(|found| XString::from_vec(found.key))
        .collect())
}

/// A help match's two fields: the tag as offered (`{name}@{lang}`) and the
/// heuristic score that orders the listing.
pub(crate) type HelpTagMatch = (Vec<u8>, Vec<u8>);

/// [`find_tags`] of a help pattern: each match as its two fields.
pub(crate) fn find_help_tag_matches(
    pat: &CStr,
    flags: c_int,
    mincount: c_int,
) -> Result<Vec<HelpTagMatch>, Failed> {
    debug_assert_ne!(flags & TAG_HELP as c_int, 0);
    let found = search(pat.to_bytes(), flags, mincount, None)?;
    Ok(found
        .into_iter()
        .map(|found| (found.key, found.score))
        .collect())
}
