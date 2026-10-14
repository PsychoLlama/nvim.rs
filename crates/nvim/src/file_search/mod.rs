//! Searching for a file along `'path'`, `'tags'` and `'cdpath'`.
//!
//! A caller builds a search context with [`FindContext::start`], then calls
//! [`FindContext::next_match`] until it answers `None`. Re-initialising an
//! existing context keeps its list of directories already visited, which is what lets a `'tags'`
//! setting like `"./**/tags,./**/TAGS,**/tags"` share work between its three
//! searches.
//!
//! The walk is depth first over [`FindContext::stack`]. The `'path'` grammar
//! it understands beyond plain wildcards lives only here: a `;` asks for the
//! search to be restarted one directory higher until a stop directory is
//! reached, and `**N` limits how far down `**` descends.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::autocmd::{fire_autocmds_for, has_event};
use crate::charset::vim_isfilec;
use crate::cstr;
use crate::eval::vars::set_vim_var_string;
use crate::eval::{eval_to_string_safe, get_v_event, restore_v_event};
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mbyte::mb_tolower;
use crate::memory::XString;
use crate::message::emsg;
use crate::message::state::line_msg;
use crate::message::{
    e_cant_find_directory_str_in_cdpath, e_cant_find_file_str_in_path,
    e_no_more_directory_str_found_in_cdpath, e_no_more_file_str_found_in_path,
};
use crate::normal::get_visual_text;
use crate::option::vars::p_fic;
use crate::option::{option_parts, was_set_insecurely};
use crate::options::kOptIncludeexpr;
use crate::optionstr::OptStringRef;
use crate::os::cshim::gettext;
use crate::os::fs::{dir_exists, dirname_within, os_chdir, path_exists};
use crate::os::input::os_breakcheck;
use crate::path::{
    ExpandFlags, after_pathsep, expand_wildcards, full_name_save, path_fnamecmp, path_fnamencmp,
    path_has_drive_letter, path_with_url, pathcmp, shorten_index, simplify_filename, tail_index,
    tail_with_sep_index, vim_is_abs_name, vim_ispathsep,
};
use crate::types::AutoEvent;
use crate::types::{BoolVarValue, CdCause, CdScope, FileID, LineNr, MAXPATHL, SaveVEvent};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

mod chdir;
mod cursor;
mod init;
mod resolve;
mod visited;

pub(crate) use self::chdir::*;
pub(crate) use self::cursor::*;
pub(crate) use self::init::*;
pub(crate) use self::resolve::*;
pub(crate) use self::visited::*;

pub(crate) const kCdCauseAuto: CdCause = 2;
pub(crate) const kCdCauseWindow: CdCause = 1;
pub(crate) const kCdCauseOther: CdCause = -1;
pub(crate) const kBufOptIncludeexpr: c_int = 46;

/// What a search should accept as a match.
pub(crate) const FINDFILE_DIR: c_int = 1;
pub(crate) const FINDFILE_BOTH: c_int = 2;

crate::flag_set! {
    /// How to read the file name under the cursor -- upstream's `FNAME_*`,
    /// the `options` argument [`file_name_at_cursor`] and
    /// [`find_file_name_in_path`] thread.
    pub(crate) struct FileNameOpts;

    /// Say why, when no name could be read or no file found.
    const MESS = 1;
    /// Expand the name: environment variables, `~`, and a `'path'` search.
    const EXP = 2;
    /// Recognise a hypertext link (`http://…`) as a name.
    const HYP = 4;
    /// Run the name through `'includeexpr'` first.
    const INCL = 8;
    /// Answer the name relative to the current directory when that is
    /// shorter than the absolute one.
    const REL = 16;
    /// Undo the command line's backslash escaping in the name.
    const UNESC = 32;
}

/// The directories `ff_get_next` walks into. `NOTWILD` is there — a bit of a
/// hack, upstream says — because the already-expanded path may hold wildcard
/// characters that are to be taken literally from here on.
const LITERAL_DIR: ExpandFlags = ExpandFlags::DIR
    .or(ExpandFlags::ADDSLASH)
    .or(ExpandFlags::SILENT)
    .or(ExpandFlags::NOTWILD);

/// How deep a search descends under one entry of `'path'`, `'cdpath'` or
/// `'tags'`, whatever its `**` allows.
pub(crate) const FIND_LEVEL: c_int = 100;

/// How far `**` descends when the pattern does not say.
pub(crate) const FF_MAX_STAR_STAR_EXPAND: u8 = 30;

/// An owned NUL-terminated byte string.
///
/// The searcher hands nearly every name it builds to the file system as a
/// `&CStr`, so the terminator is part of the value; [`len`](Name::len) and
/// [`bytes`](Name::bytes) count only what comes before it.
#[derive(Clone, Default)]
pub(crate) struct Name(Vec<u8>);

impl Name {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(0);
        Name(owned)
    }

    /// The name as it is written, without the terminator.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0[..self.len()]
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len().saturating_sub(1)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The byte at `at`, which may be the terminator.
    pub(crate) fn at(&self, at: usize) -> u8 {
        self.0[at]
    }

    pub(crate) fn set(&mut self, at: usize, byte: u8) {
        self.0[at] = byte;
    }

    /// The name as the NUL-terminated string it is stored as.
    pub(crate) fn as_cstr(&self) -> &CStr {
        cstr::in_bytes(&self.0)
    }

    /// For the callers outside the searcher (tags) that still hand a name to
    /// a C reader.
    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }

    /// For the C functions that rewrite a name in place. They only ever
    /// shorten it, so [`truncate`](Name::truncate) puts the length back.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr().cast()
    }

    /// Append `bytes`.
    pub(crate) fn extend(&mut self, bytes: &[u8]) {
        self.0.pop();
        self.0.extend_from_slice(bytes);
        self.0.push(0);
    }

    /// Forget everything from `at` on.
    pub(crate) fn truncate(&mut self, at: usize) {
        self.0.truncate(at);
        self.0.push(0);
    }

    pub(crate) fn clear(&mut self) {
        self.truncate(0);
    }

    /// [`simplify_filename`] in place: the name it simplifies to, at the
    /// length upstream's answer gives it.
    pub(crate) fn simplify(&mut self) {
        let len = simplify_filename(&mut self.0);
        self.truncate(len);
    }

    /// Forget the first `n` bytes, keeping the terminator.
    pub(crate) fn drain_front(&mut self, n: usize) {
        self.0.drain(..n);
    }
}

/// The state of one file search: what is left to look at, and where it has
/// already been.
///
pub(crate) struct FindContext {
    /// Directories still to search, deepest last.
    pub(crate) stack: Vec<StackFrame>,
    /// Files already answered, one list per name searched for.
    pub(crate) visited: VisitedLists,
    /// Directories already searched, one list per name searched for.
    pub(crate) dir_visited: VisitedLists,
    /// The file to search for.
    pub(crate) file_to_search: Name,
    /// The directory the search starts from, when the search path was
    /// relative. The upward search shortens it one component at a time.
    pub(crate) start_dir: Option<Name>,
    /// The fixed leading part of the given path. Kept for the upward
    /// search, which rebuilds the first stack frame from it.
    pub(crate) fix_path: Name,
    /// The part of the given path holding wildcards; `None` when it had
    /// none at all.
    pub(crate) wc_path: Option<Name>,
    /// How many levels of directories to search downwards.
    pub(crate) level: c_int,
    /// Where to stop the upward search. `None` asks for no upward search;
    /// an empty entry means "ascend to the top of the tree".
    pub(crate) stopdirs: Option<Vec<Name>>,
    /// `FINDFILE_BOTH`, `FINDFILE_DIR` or `FINDFILE_FILE`.
    pub(crate) find_what: c_int,
    /// Searching for a tags file: do not use `'suffixesadd'`.
    pub(crate) tagfile: bool,
}

impl Default for FindContext {
    fn default() -> Self {
        FindContext {
            stack: Vec::new(),
            visited: VisitedLists::default(),
            dir_visited: VisitedLists::default(),
            file_to_search: Name::default(),
            start_dir: None,
            fix_path: Name::default(),
            wc_path: None,
            level: 0,
            stopdirs: None,
            find_what: FINDFILE_BOTH,
            tagfile: false,
        }
    }
}

impl FindContext {
    /// Clear the search context, but NOT the visited lists.
    pub(crate) fn reset(&mut self) {
        self.stack.clear();
        self.stopdirs = None;
        self.file_to_search.clear();
        self.start_dir = None;
        self.fix_path.clear();
        self.wc_path = None;
        self.level = 0;
    }
}

/// The path being built, and handed to the caller when it turns out to name
/// the file we want.
///
/// A fixed `MAXPATHL` buffer, as upstream's: a name that does not fit gives
/// the search up rather than being truncated.
pub(crate) struct Candidate {
    buf: Vec<u8>,
    len: usize,
}

/// A name did not fit in `MAXPATHL`. The search gives up rather than
/// truncating.
pub(crate) struct TooLong;

impl Candidate {
    fn new() -> Self {
        Candidate {
            buf: vec![0; MAXPATHL as usize],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.buf[0] = 0;
        self.len = 0;
    }

    /// The name so far, as the NUL-terminated string it is built as.
    fn as_cstr(&self) -> &CStr {
        cstr::in_bytes(&self.buf)
    }

    /// The name so far.
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Write `parts` one after another at `at` — as much of them as fits,
    /// terminated — and answer where the whole name would have ended, which
    /// may be past `MAXPATHL`: every caller tests that, because it is how
    /// the search gives up. Upstream's `snprintf("%s%s%s")`.
    fn write_at(&mut self, at: usize, parts: [&[u8]; 3]) -> usize {
        let mut end = at;
        for part in parts {
            let room = (MAXPATHL as usize - 1).saturating_sub(end);
            let fits = part.len().min(room);
            if fits > 0 {
                self.buf[end..end + fits].copy_from_slice(&part[..fits]);
            }
            end += part.len();
        }
        self.buf[end.min(MAXPATHL as usize - 1)] = 0;
        end
    }

    /// A path separator, unless `part` already ends in one.
    fn separator(part: &[u8]) -> &'static [u8] {
        if after_pathsep(part, part.len()) {
            b""
        } else {
            b"/"
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), TooLong> {
        if self.len + 1 >= MAXPATHL as usize {
            return Err(TooLong);
        }
        self.buf[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    fn terminate(&mut self) {
        self.buf[self.len] = 0;
    }

    /// A copy of the name, for the caller to own.
    fn take(&self) -> XString {
        XString::from_bytes(self.bytes())
    }
}

/// What one pass of the downward walk ended on.
enum Down {
    /// A file was found; it is in the caller's [`Candidate`].
    Found,
    /// The stack ran out, or the user interrupted.
    Exhausted,
    /// A name grew past `MAXPATHL`; the whole search gives up.
    TooLong,
}

impl FindContext {
    /// Build the directory name this frame stands for and expand its first
    /// wildcard, answering where in `wc_path` the rest of the wildcards
    /// start.
    fn expand(&self, frame: &mut StackFrame, file_path: &mut Candidate) -> Result<usize, TooLong> {
        // Whether "**" should also expand to nothing here, which is
        // done by handing expand_wildcards a second pattern.
        let mut expand_empty = false;

        // If we have a start dir copy it in.
        if !vim_is_abs_name(frame.fix_path.as_cstr())
            && let Some(start_dir) = &self.start_dir
        {
            if start_dir.len() + 1 >= MAXPATHL as usize {
                return Err(TooLong);
            }
            file_path.len = file_path.write_at(
                0,
                [
                    start_dir.bytes(),
                    Candidate::separator(start_dir.bytes()),
                    b"",
                ],
            );
            if file_path.len >= MAXPATHL as usize {
                return Err(TooLong);
            }
        }

        // Append the fixed part of the search path.
        let fix_path = &frame.fix_path;
        if file_path.len + fix_path.len() + 1 >= MAXPATHL as usize {
            return Err(TooLong);
        }
        file_path.len = file_path.write_at(
            file_path.len,
            [
                fix_path.bytes(),
                Candidate::separator(fix_path.bytes()),
                b"",
            ],
        );
        if file_path.len >= MAXPATHL as usize {
            return Err(TooLong);
        }

        let mut rest = 0;
        if !frame.wc_path.is_empty() {
            if frame.wc_path.bytes().starts_with(b"**") {
                // The byte after "**" is the descent counter, not a
                // character. Upstream reads it through a `char`, which
                // is signed here, so a limit of 128 or more tests as
                // negative: `**200` never descends and never counts
                // down. Preserved -- it is what those patterns do.
                let left = frame.wc_path.at(2) as i8;
                if left > 0 {
                    frame.wc_path.set(2, (left - 1) as u8);
                    file_path.push(b'*')?;
                }
                if frame.wc_path.at(2) == 0 {
                    // The limit is spent: drop "**<count>" for good.
                    frame.wc_path.drain_front(3);
                } else {
                    rest = 3;
                }
                if !frame.star_star_empty {
                    // If not done before, expand '**' to empty.
                    frame.star_star_empty = true;
                    expand_empty = true;
                }
            }

            // Copy until the next path separator or the end of the path.
            // Stopping at a separator means there is still something
            // left, which is handled below by pushing every directory
            // expand_wildcards() returned back onto the stack.
            while rest < frame.wc_path.len() && !vim_ispathsep(frame.wc_path.at(rest) as c_int) {
                file_path.push(frame.wc_path.at(rest))?;
                rest += 1;
            }
            file_path.terminate();
            if rest < frame.wc_path.len() {
                rest += 1; // step over the separator
            }
        }

        // Expand wildcards like "*" and "$VAR". If the path is a URL
        // don't try this.
        let files = if path_with_url(file_path.as_cstr()) != 0 {
            FileList::of_one(file_path.bytes())
        } else {
            let mut patterns = vec![file_path.as_cstr()];
            if expand_empty {
                patterns.push(frame.fix_path.as_cstr());
            }
            FileList::expand(&patterns, LITERAL_DIR)
        };
        frame.files = Some(files);
        frame.files_cur = 0;
        frame.stage = 0;
        Ok(rest)
    }

    /// Look for the wanted file in each directory this frame expanded to,
    /// answering the index of the one that had it.
    ///
    /// The name is left in `file_path`, shortened relative to the current
    /// directory when that is possible.
    fn find_hit(
        &mut self,
        frame: &StackFrame,
        file_path: &mut Candidate,
    ) -> Result<Option<usize>, TooLong> {
        // Borrowed for the call: nothing below runs user code that could
        // set the option.
        let buffer = Buf::current();
        let suffixes: &[u8] = if self.tagfile {
            b""
        } else {
            buffer.b_p_sua.bytes()
        };
        let files = frame.files();
        for i in frame.files_cur..files.len() {
            let dir = files.get(i);
            if path_with_url(dir.as_cstr()) == 0 && !dir_exists(dir.as_cstr()) {
                continue; // not a directory
            }
            // Prepare the filename to be checked for existence below.
            let wanted = &self.file_to_search;
            if dir.len() + 1 + wanted.len() >= MAXPATHL as usize {
                return Err(TooLong);
            }
            file_path.len = file_path.write_at(0, [dir, Candidate::separator(dir), wanted.bytes()]);
            if file_path.len >= MAXPATHL as usize {
                return Err(TooLong);
            }

            // Try without extra suffix and then with suffixes from
            // 'suffixesadd'.
            let stem = file_path.len;
            let mut suffixes = option_parts(suffixes);
            loop {
                let exists = path_with_url(file_path.as_cstr()) != 0
                    || (path_exists(file_path.as_cstr())
                        && (self.find_what == FINDFILE_BOTH
                            || (self.find_what == FINDFILE_DIR)
                                == dir_exists(file_path.as_cstr())));
                // If the file exists and we didn't already find it.
                if exists && self.visited.current().add(file_path.bytes(), b"") {
                    if path_with_url(file_path.as_cstr()) == 0 {
                        file_path.len = simplify_filename(&mut file_path.buf);
                    }
                    self.shorten(file_path);
                    return Ok(Some(i));
                }

                // Not found or found already, try next suffix.
                let Some(suffix) = suffixes.next() else {
                    break;
                };
                // Cut where upstream's `copy_option_part` into the rest of
                // the buffer cuts it.
                let room = MAXPATHL as usize - 1 - stem;
                file_path.len =
                    file_path.write_at(stem, [&suffix[..suffix.len().min(room)], b"", b""]);
            }
        }
        Ok(None)
    }

    /// Drop the current directory's prefix from the answer, so that a hit
    /// below the working directory reads as a relative name.
    fn shorten(&self, file_path: &mut Candidate) {
        let Ok(curdir) = dirname_within(MAXPATHL as usize) else {
            return;
        };
        let Some(at) = shorten_index(file_path.as_cstr().to_bytes(), &curdir) else {
            return;
        };
        file_path.buf.copy_within(at..file_path.len + 1, 0);
        file_path.len -= at;
    }

    /// Push every directory this frame expanded to, to be searched with the
    /// wildcards that are left.
    fn push_subdirs(&mut self, frame: &StackFrame, rest: usize) {
        let files = frame.files();
        for i in frame.files_cur..files.len() {
            let dir = files.get(i);
            if !dir_exists(dir.as_cstr()) {
                continue; // not a directory
            }
            self.stack.push(StackFrame::new(
                Name::from_bytes(dir),
                Name::from_bytes(&frame.wc_path.bytes()[rest..]),
                frame.level - 1,
                false,
            ));
        }
    }

    /// `**` descends to the leaves of the tree: push every subdirectory
    /// again with the same wildcards, one level shallower.
    fn push_descent(&mut self, frame: &StackFrame) {
        let files = frame.files();
        for i in frame.files_cur..files.len() {
            let dir = files.get(i);
            if path_fnamecmp(dir, frame.fix_path.bytes()) == 0 {
                continue; // don't repush the same directory
            }
            if !dir_exists(dir.as_cstr()) {
                continue; // not a directory
            }
            self.stack.push(StackFrame::new(
                Name::from_bytes(dir),
                frame.wc_path.clone(),
                frame.level - 1,
                true,
            ));
        }
    }

    /// Work the stack until something is found, it runs out, or a name grows
    /// too long.
    fn search_downwards(&mut self, file_path: &mut Candidate) -> Down {
        loop {
            // Check if the user wants to stop the search.
            os_breakcheck();
            if got_int.get() {
                return Down::Exhausted;
            }
            let Some(mut frame) = self.stack.pop() else {
                return Down::Exhausted;
            };

            // TODO(vim): decide if we leave this test in
            //
            // GOOD: don't search a directory(-tree) twice.
            // BAD:  - check linked list for every new directory entered.
            //       - check for double files also done below
            //
            // Good if you have links on the same directory via several
            // ways, or self-references in directories (e.g. SuSE Linux
            // 6.3: /etc/rc.d/init.d is linked to /etc/rc.d -> endless
            // loop). Only needed for directories worked on for the first
            // time, hence the test on `files`.
            if frame.files.is_none()
                && !self
                    .dir_visited
                    .current()
                    .add(frame.fix_path.bytes(), frame.wc_path.bytes())
            {
                continue;
            }

            // Check depth.
            if frame.level <= 0 {
                continue;
            }

            file_path.clear();

            // If no file list till now expand wildcards. expand_wildcards
            // handles an array of paths and returns every expansion in
            // one array, which is how '**' expands to an empty string.
            let rest = if frame.files.is_none() {
                match self.expand(&mut frame, file_path) {
                    Ok(rest) => rest,
                    Err(TooLong) => return Down::TooLong,
                }
            } else {
                frame.wc_path.len()
            };

            if frame.stage == 0 {
                // This is the first time we work on this directory.
                if rest == frame.wc_path.len() {
                    // No further wildcards to expand, so check for the
                    // final file now.
                    match self.find_hit(&frame, file_path) {
                        Err(TooLong) => return Down::TooLong,
                        Ok(Some(i)) => {
                            // Keep the dir, to examine the rest of its
                            // entries on the next call.
                            frame.files_cur = i + 1;
                            self.stack.push(frame);
                            return Down::Found;
                        }
                        Ok(None) => {}
                    }
                } else {
                    // Still wildcards left, push the directories for
                    // further search.
                    self.push_subdirs(&frame, rest);
                }
                frame.files_cur = 0;
                frame.stage = 1;
            }

            // If the wildcards contain '**' we have to descend till we
            // reach the leaves of the directory tree.
            if frame.wc_path.bytes().starts_with(b"**") {
                self.push_descent(&frame);
            }
            // We are done with the current directory; dropping the frame
            // frees its file list.
        }
    }

    /// Shorten the starting directory by one component and push it as a
    /// fresh frame, so the walk starts again one level higher.
    ///
    /// Answers false when the top of the tree, or a stop directory, has been
    /// reached.
    fn step_up(
        &mut self,
        path_end: &mut usize,
        file_path: &mut Candidate,
    ) -> Result<bool, TooLong> {
        let start_dir = self.start_dir.as_mut().expect("checked by the caller");
        let stopdirs = self.stopdirs.as_deref().expect("checked by the caller");

        // path_end sits on the terminator until the first step up, and
        // on the last character after that.
        let plen = *path_end + usize::from(start_dir.at(*path_end) != 0);
        // Is the last starting directory in the stop list?
        if ff_path_in_stoplist(start_dir, plen, stopdirs) {
            return Ok(false);
        }

        // Cut off the last directory.
        while *path_end > 0 && vim_ispathsep(start_dir.at(*path_end) as c_int) {
            *path_end -= 1;
        }
        while *path_end > 0 && !vim_ispathsep(start_dir.at(*path_end - 1) as c_int) {
            *path_end -= 1;
        }
        start_dir.truncate(*path_end);
        // Upstream steps one back even when the name is now empty,
        // forming a pointer before the string; the test below is what
        // keeps it from ever being read.
        *path_end = path_end.saturating_sub(1);

        if start_dir.is_empty() {
            return Ok(false);
        }
        if start_dir.len() + 1 + self.fix_path.len() >= MAXPATHL as usize {
            return Err(TooLong);
        }

        file_path.len = file_path.write_at(
            0,
            [
                start_dir.bytes(),
                Candidate::separator(start_dir.bytes()),
                self.fix_path.bytes(),
            ],
        );
        if file_path.len >= MAXPATHL as usize {
            return Err(TooLong);
        }

        // Create a new stack entry.
        let fix_path = Name::from_bytes(file_path.bytes());
        let wc_path = self.wc_path.clone().unwrap_or_default();
        self.stack
            .push(StackFrame::new(fix_path, wc_path, self.level, false));
        Ok(true)
    }
}

impl FindContext {
    /// The next file this search finds, or `None` when there are no more.
    ///
    /// The context is built by [`FindContext::start`]; to get all matching
    /// files call this until it answers `None`.
    ///
    /// The search algorithm is depth first. To change this replace the
    /// stack with a list (don't forget to leave partly searched directories
    /// on the top of the list).
    pub(crate) fn next_match(&mut self) -> Option<XString> {
        let mut file_path = Candidate::new();

        // Where the start dir ends -- needed for the upward search.
        let mut path_end = self.start_dir.as_ref().map_or(0, Name::len);

        loop {
            match self.search_downwards(&mut file_path) {
                Down::Found => return Some(file_path.take()),
                Down::TooLong => return None,
                Down::Exhausted => {}
            }

            // We didn't find anything downwards. Should we search upwards?
            if self.start_dir.is_none() || self.stopdirs.is_none() || got_int.get() {
                return None;
            }
            match self.step_up(&mut path_end, &mut file_path) {
                Ok(true) => {}
                Ok(false) | Err(TooLong) => return None,
            }
        }
    }
}
