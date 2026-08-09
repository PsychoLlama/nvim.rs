//! Searching for a file along `'path'`, `'tags'` and `'cdpath'`.
//!
//! A caller builds a search context with [`vim_findfile_init`], then calls
//! [`vim_findfile`] until it answers NULL, then hands the context to
//! [`vim_findfile_cleanup`]. Re-initialising an existing context keeps its
//! list of directories already visited, which is what lets a `'tags'`
//! setting like `"./**/tags,./**/TAGS,**/tags"` share work between its three
//! searches.
//!
//! The walk is depth first over [`FindContext::stack`]. The `'path'` grammar
//! it understands beyond plain wildcards lives only here: a `;` asks for the
//! search to be restarted one directory higher until a stop directory is
//! reached, and `**N` limits how far down `**` descends.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::autocmd::{EVENT_DIRCHANGED, EVENT_DIRCHANGEDPRE, apply_autocmds, has_event};
use crate::src::nvim::charset::{getdigits_int32, getdigits_long, skipwhite, vim_isfilec};
use crate::src::nvim::cursor::get_cursor_line_ptr;
use crate::src::nvim::eval::typval::{
    tv_dict_add_bool, tv_dict_add_str, tv_dict_set_keys_readonly,
};
use crate::src::nvim::eval::vars::set_vim_var_string;
use crate::src::nvim::eval::{eval_to_string_safe, get_v_event, restore_v_event};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    NameBuff, VIsual_active, curbuf, current_sctx, curwin, e_cant_find_directory_str_in_cdpath,
    e_cant_find_file_str_in_path, e_no_more_directory_str_found_in_cdpath,
    e_no_more_file_str_found_in_path, got_int, line_msg, p_cdpath, p_cpo, p_fic,
};
use crate::src::nvim::mbyte::{mb_tolower, utf_head_off, utf_ptr2char, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmemdupz, xstrlcpy};
use crate::src::nvim::message::emsg;
use crate::src::nvim::normal::get_visual_text;
use crate::src::nvim::option::{copy_option_part, was_set_insecurely};
use crate::src::nvim::options::kOptIncludeexpr;
use crate::src::nvim::os::env::expand_env_esc;
use crate::src::nvim::os::fs::{
    os_chdir, os_dirname, os_fileid, os_fileid_equal, os_isdir, os_path_exists,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{abort, gettext, strcpy, strlen, strncmp};
use crate::src::nvim::path::{
    FreeWild, FullName_save, after_pathsep, expand_wildcards, path_fnamecmp, path_fnamencmp,
    path_has_drive_letter, path_is_url, path_shorten_fname, path_tail, path_tail_with_sep,
    path_with_url, pathcmp, simplify_filename, vim_isAbsName, vim_ispathsep,
};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, xstrnsave};
use crate::src::nvim::types::{
    BoolVarValue, CdCause, CdScope, FileID, cmdarg_T, event_T, linenr_T, ptrdiff_t, save_v_event_T,
    size_t,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::ffi::CStr;

mod chdir;
mod cursor;
mod init;
mod resolve;
mod visited;

pub use self::chdir::*;
pub use self::cursor::*;
pub use self::init::*;
pub use self::resolve::*;
pub(crate) use self::visited::*;

pub const kCdCauseAuto: CdCause = 2;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseOther: CdCause = -1;
pub const kBufOptIncludeexpr: c_int = 46;

/// What a search should accept as a match.
pub const FINDFILE_DIR: c_int = 1;
pub const FINDFILE_BOTH: c_int = 2;

pub const FNAME_MESS: c_int = 1;
pub const FNAME_EXP: c_int = 2;
pub const FNAME_HYP: c_int = 4;
pub const FNAME_INCL: c_int = 8;
pub const FNAME_REL: c_int = 16;
pub const FNAME_UNESC: c_int = 32;

pub const EW_DIR: c_int = 1;
pub const EW_ADDSLASH: c_int = 8;
pub const EW_SILENT: c_int = 32;
pub const EW_NOTWILD: c_int = 1024;
pub const OPT_LOCAL: c_int = 2;

pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
/// `'cpoptions'` flag: a `'tags'` entry starting with `"./"` is relative to
/// the current directory rather than to the current file.
pub const CPO_DOTTAG: c_int = 'd' as c_int;

/// The longest path name the searcher will build, buffers included.
pub const MAXPATHL: usize = 4096;

/// How far `**` descends when the pattern does not say.
pub const FF_MAX_STAR_STAR_EXPAND: u8 = 30;

/// An owned NUL-terminated byte string.
///
/// The searcher hands nearly every name it builds to a C function, so the
/// terminator is part of the value; [`len`](Name::len) and
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

    /// # Safety
    /// `p` must point at a NUL-terminated string.
    pub(crate) unsafe fn from_ptr(p: *const c_char) -> Self {
        // SAFETY: the caller's promise.
        Self::from_bytes(unsafe { CStr::from_ptr(p) }.to_bytes())
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

    pub(crate) fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }

    /// For the C functions that rewrite a name in place. They only ever
    /// shorten it, so [`truncate`](Name::truncate) puts the length back.
    pub(crate) fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr().cast()
    }

    /// Forget everything from `at` on.
    pub(crate) fn truncate(&mut self, at: usize) {
        self.0.truncate(at);
        self.0.push(0);
    }

    pub(crate) fn clear(&mut self) {
        self.truncate(0);
    }

    /// Forget the first `n` bytes, keeping the terminator.
    pub(crate) fn drain_front(&mut self, n: usize) {
        self.0.drain(..n);
    }
}

/// The state of one file search: what is left to look at, and where it has
/// already been.
///
/// Callers hold this as an opaque pointer.
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
/// A fixed `MAXPATHL` buffer that never moves: `expand_wildcards` is given a
/// pointer into it before the name is finished, and `copy_option_part`
/// writes `'suffixesadd'` parts straight into its tail.
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
            buf: vec![0; MAXPATHL],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.buf[0] = 0;
        self.len = 0;
    }

    fn as_ptr(&self) -> *const c_char {
        self.buf.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buf.as_mut_ptr().cast()
    }

    /// Write `parts` one after another at `at`, and answer where the whole
    /// name would have ended — which may be past `MAXPATHL`, and every
    /// caller tests it, because that is how the search gives up.
    ///
    /// # Safety
    /// Each part must be a NUL-terminated string, and `at` must be no
    /// greater than `MAXPATHL`.
    unsafe fn write_at(&mut self, at: usize, parts: [*const c_char; 3]) -> usize {
        // SAFETY: the caller's promise; `MAXPATHL - at` is the room left.
        at + unsafe {
            vim_snprintf(
                self.as_mut_ptr().add(at),
                MAXPATHL - at,
                c"%s%s%s".as_ptr(),
                parts[0],
                parts[1],
                parts[2],
            )
        } as usize
    }

    /// A path separator, unless `part` already ends in one.
    ///
    /// # Safety
    /// `part` must hold `partlen` readable bytes.
    unsafe fn separator(part: *const c_char, partlen: usize) -> *const c_char {
        // SAFETY: the caller's promise.
        if unsafe { after_pathsep(part, part.add(partlen)) } == 0 {
            c"/".as_ptr()
        } else {
            c"".as_ptr()
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), TooLong> {
        if self.len + 1 >= MAXPATHL {
            return Err(TooLong);
        }
        self.buf[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    fn terminate(&mut self) {
        self.buf[self.len] = 0;
    }

    /// A copy of the name, for the caller to own and free.
    fn take(&self) -> *mut c_char {
        // SAFETY: `self.len` bytes of `self.buf` are initialised.
        unsafe { xmemdupz(self.buf.as_ptr().cast(), self.len) }.cast()
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
    ///
    /// # Safety
    /// There must be a current buffer.
    unsafe fn expand(
        &self,
        frame: &mut StackFrame,
        file_path: &mut Candidate,
    ) -> Result<usize, TooLong> {
        unsafe {
            // Whether "**" should also expand to nothing here, which is
            // done by handing expand_wildcards a second pattern.
            let mut expand_empty = false;

            // If we have a start dir copy it in.
            if !vim_isAbsName(frame.fix_path.as_ptr())
                && let Some(start_dir) = &self.start_dir
            {
                if start_dir.len() + 1 >= MAXPATHL {
                    return Err(TooLong);
                }
                file_path.len = file_path.write_at(
                    0,
                    [
                        start_dir.as_ptr(),
                        Candidate::separator(start_dir.as_ptr(), start_dir.len()),
                        c"".as_ptr(),
                    ],
                );
                if file_path.len >= MAXPATHL {
                    return Err(TooLong);
                }
            }

            // Append the fixed part of the search path.
            let fix_path = &frame.fix_path;
            if file_path.len + fix_path.len() + 1 >= MAXPATHL {
                return Err(TooLong);
            }
            file_path.len = file_path.write_at(
                file_path.len,
                [
                    fix_path.as_ptr(),
                    Candidate::separator(fix_path.as_ptr(), fix_path.len()),
                    c"".as_ptr(),
                ],
            );
            if file_path.len >= MAXPATHL {
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
                while rest < frame.wc_path.len() && !vim_ispathsep(frame.wc_path.at(rest) as c_int)
                {
                    file_path.push(frame.wc_path.at(rest))?;
                    rest += 1;
                }
                file_path.terminate();
                if rest < frame.wc_path.len() {
                    rest += 1; // step over the separator
                }
            }

            // Expand wildcards like "*" and "$VAR". If the path is a URL
            // don't try this. The pointers are taken last: everything above
            // writes through `file_path`, which would invalidate one taken
            // earlier.
            let mut dirptrs: [*mut c_char; 2] = [
                file_path.as_mut_ptr(),
                if expand_empty {
                    frame.fix_path.as_ptr().cast_mut()
                } else {
                    ptr::null_mut()
                },
            ];
            let files = if path_with_url(dirptrs[0]) != 0 {
                FileList::of_one(dirptrs[0], file_path.len)
            } else {
                // Add EW_NOTWILD because the expanded path may contain
                // wildcard characters that are to be taken literally.
                // This is a bit of a hack.
                FileList::expand(
                    if dirptrs[1].is_null() { 1 } else { 2 },
                    dirptrs.as_mut_ptr(),
                    EW_DIR | EW_ADDSLASH | EW_SILENT | EW_NOTWILD,
                )
            };
            frame.files = Some(files);
            frame.files_cur = 0;
            frame.stage = 0;
            Ok(rest)
        }
    }

    /// Look for the wanted file in each directory this frame expanded to,
    /// answering the index of the one that had it.
    ///
    /// The name is left in `file_path`, shortened relative to the current
    /// directory when that is possible.
    ///
    /// # Safety
    /// There must be a current buffer.
    unsafe fn find_hit(
        &mut self,
        frame: &StackFrame,
        file_path: &mut Candidate,
    ) -> Result<Option<usize>, TooLong> {
        unsafe {
            let files = frame.files();
            for i in frame.files_cur..files.len() {
                let dir = files.get(i);
                if path_with_url(dir) == 0 && !os_isdir(dir) {
                    continue; // not a directory
                }
                // Prepare the filename to be checked for existence below.
                let dirlen = strlen(dir);
                let wanted = &self.file_to_search;
                if dirlen + 1 + wanted.len() >= MAXPATHL {
                    return Err(TooLong);
                }
                file_path.len = file_path
                    .write_at(0, [dir, Candidate::separator(dir, dirlen), wanted.as_ptr()]);
                if file_path.len >= MAXPATHL {
                    return Err(TooLong);
                }

                // Try without extra suffix and then with suffixes from
                // 'suffixesadd'.
                let stem = file_path.len;
                let mut suffix = if self.tagfile {
                    c"".as_ptr().cast_mut()
                } else {
                    (*curbuf.get()).b_p_sua
                };
                loop {
                    let exists = path_with_url(file_path.as_ptr()) != 0
                        || (os_path_exists(file_path.as_ptr())
                            && (self.find_what == FINDFILE_BOTH
                                || (self.find_what == FINDFILE_DIR)
                                    == os_isdir(file_path.as_ptr())));
                    // If the file exists and we didn't already find it.
                    if exists
                        && self
                            .visited
                            .current()
                            .add(&file_path.buf[..file_path.len], b"")
                    {
                        if path_with_url(file_path.as_ptr()) == 0 {
                            file_path.len = simplify_filename(file_path.as_mut_ptr());
                        }
                        self.shorten(file_path);
                        return Ok(Some(i));
                    }

                    // Not found or found already, try next suffix.
                    if *suffix == 0 {
                        break;
                    }
                    debug_assert!(stem <= MAXPATHL);
                    // `copy_option_part` answers what it wrote, which is at
                    // most MAXPATHL - stem - 1.
                    file_path.len = stem
                        + copy_option_part(
                            &raw mut suffix,
                            file_path.as_mut_ptr().add(stem),
                            MAXPATHL - stem,
                            c",".as_ptr().cast_mut(),
                        );
                }
            }
            Ok(None)
        }
    }

    /// Drop the current directory's prefix from the answer, so that a hit
    /// below the working directory reads as a relative name.
    ///
    /// # Safety
    /// `file_path` must hold a NUL-terminated name.
    unsafe fn shorten(&self, file_path: &mut Candidate) {
        unsafe {
            let mut curdir = [0 as c_char; MAXPATHL];
            if os_dirname(curdir.as_mut_ptr(), MAXPATHL) != OK {
                return;
            }
            let base = file_path.as_mut_ptr();
            let short = path_shorten_fname(base, curdir.as_mut_ptr());
            if short.is_null() {
                return;
            }
            // Measured against the pointer `path_shorten_fname` was handed,
            // so that neither is re-derived through `file_path` first.
            let at = short.offset_from(base) as usize;
            file_path.buf.copy_within(at..file_path.len + 1, 0);
            file_path.len -= at;
        }
    }

    /// Push every directory this frame expanded to, to be searched with the
    /// wildcards that are left.
    ///
    /// # Safety
    /// The frame must have been expanded.
    unsafe fn push_subdirs(&mut self, frame: &StackFrame, rest: usize) {
        unsafe {
            let files = frame.files();
            for i in frame.files_cur..files.len() {
                let dir = files.get(i);
                if !os_isdir(dir) {
                    continue; // not a directory
                }
                self.stack.push(StackFrame::new(
                    Name::from_ptr(dir),
                    Name::from_bytes(&frame.wc_path.bytes()[rest..]),
                    frame.level - 1,
                    false,
                ));
            }
        }
    }

    /// `**` descends to the leaves of the tree: push every subdirectory
    /// again with the same wildcards, one level shallower.
    ///
    /// # Safety
    /// The frame must have been expanded.
    unsafe fn push_descent(&mut self, frame: &StackFrame) {
        unsafe {
            let files = frame.files();
            for i in frame.files_cur..files.len() {
                let dir = files.get(i);
                if path_fnamecmp(dir, frame.fix_path.as_ptr()) == 0 {
                    continue; // don't repush the same directory
                }
                if !os_isdir(dir) {
                    continue; // not a directory
                }
                self.stack.push(StackFrame::new(
                    Name::from_ptr(dir),
                    frame.wc_path.clone(),
                    frame.level - 1,
                    true,
                ));
            }
        }
    }

    /// Work the stack until something is found, it runs out, or a name grows
    /// too long.
    ///
    /// # Safety
    /// There must be a current buffer.
    unsafe fn search_downwards(&mut self, file_path: &mut Candidate) -> Down {
        unsafe {
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
    }

    /// Shorten the starting directory by one component and push it as a
    /// fresh frame, so the walk starts again one level higher.
    ///
    /// Answers false when the top of the tree, or a stop directory, has been
    /// reached.
    ///
    /// # Safety
    /// There must be a start directory and a stop list.
    unsafe fn step_up(
        &mut self,
        path_end: &mut usize,
        file_path: &mut Candidate,
    ) -> Result<bool, TooLong> {
        unsafe {
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
            if start_dir.len() + 1 + self.fix_path.len() >= MAXPATHL {
                return Err(TooLong);
            }

            file_path.len = file_path.write_at(
                0,
                [
                    start_dir.as_ptr(),
                    Candidate::separator(start_dir.as_ptr(), start_dir.len()),
                    self.fix_path.as_ptr(),
                ],
            );
            if file_path.len >= MAXPATHL {
                return Err(TooLong);
            }

            // Create a new stack entry.
            let fix_path = Name::from_bytes(&file_path.buf[..file_path.len]);
            let wc_path = self.wc_path.clone().unwrap_or_default();
            self.stack
                .push(StackFrame::new(fix_path, wc_path, self.level, false));
            Ok(true)
        }
    }
}

/// Find a file in a search context, built by [`vim_findfile_init`].
///
/// To get all matching files call this until it answers NULL. If the passed
/// search context is NULL, NULL is answered.
///
/// The search algorithm is depth first. To change this replace the stack
/// with a list (don't forget to leave partly searched directories on the top
/// of the list).
///
/// @return  a pointer to an allocated file name, or NULL if nothing found.
pub unsafe extern "C" fn vim_findfile(search_ctx_arg: *mut c_void) -> *mut c_char {
    unsafe {
        if search_ctx_arg.is_null() {
            return ptr::null_mut();
        }
        let ctx = &mut *search_ctx_arg.cast::<FindContext>();
        let mut file_path = Candidate::new();

        // Where the start dir ends -- needed for the upward search.
        let mut path_end = ctx.start_dir.as_ref().map_or(0, Name::len);

        loop {
            match ctx.search_downwards(&mut file_path) {
                Down::Found => return file_path.take(),
                Down::TooLong => break,
                Down::Exhausted => {}
            }

            // We didn't find anything downwards. Should we search upwards?
            if ctx.start_dir.is_none() || ctx.stopdirs.is_none() || got_int.get() {
                break;
            }
            match ctx.step_up(&mut path_end, &mut file_path) {
                Ok(true) => {}
                Ok(false) | Err(TooLong) => break,
            }
        }
        ptr::null_mut()
    }
}
