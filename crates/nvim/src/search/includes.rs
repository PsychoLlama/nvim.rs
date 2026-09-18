//! Searching a file and everything it includes.
//!
//! [`find_pattern_in_path`] is `[i`, `[d`, `:ilist`, `:isearch`,
//! `:checkpath` and insert-mode `CTRL-X CTRL-I`/`CTRL-X CTRL-D` — one
//! walk with six things it may do when it finds a match ([`Action`]) and
//! three things it may be looking for ([`Kind`]). It reads the current
//! buffer line by line; every line matching `'include'` names a file that
//! is pushed onto a stack and read the same way, `'define'` narrows what
//! counts as a match, and `'path'` is where the names are resolved.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::file_search::FileNameOpts;
use crate::highlight_group::{HLF_D, HLF_R};
use crate::memory::XString;
use crate::message_fmt::c_str;
use crate::option::local_or_global;
use crate::option::vars::P_DEF;
use crate::option::vars::P_INC;
use crate::regexp::RE_MAGIC;
use crate::smsg;
use crate::strings::has_bytes;
use crate::types::{FAIL, IOSIZE, NUL, OK, ShmFlag};
use crate::window::valid_win;
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::ptr;

const CHECK_PATH: c_int = super::CHECK_PATH as c_int;
const FIND_DEFINE: c_int = super::FIND_DEFINE as c_int;
const ACTION_SHOW: c_int = super::ACTION_SHOW as c_int;
const ACTION_SPLIT: c_int = super::ACTION_SPLIT as c_int;
const ACTION_SHOW_ALL: c_int = super::ACTION_SHOW_ALL as c_int;
const ACTION_EXPAND: c_int = super::ACTION_EXPAND as c_int;
const LSIZE: usize = super::LSIZE as usize;

/// An `xmalloc`ed NUL-terminated name that frees itself.
struct Name(*mut c_char);

impl Name {
    fn as_ptr(&self) -> *mut c_char {
        self.0
    }

    fn as_cstr(&self) -> &CStr {
        // SAFETY: the allocation is live and NUL-terminated for as long as
        // this owns it.
        unsafe { cstr::at(self.0) }
    }
}

impl Drop for Name {
    fn drop(&mut self) {
        // SAFETY: always an xmalloc'ed allocation.
        unsafe { xfree(self.0 as *mut c_void) };
    }
}

/// One file on the include stack.
struct SearchedFile {
    /// Open while the walk is reading the file; null once it has been
    /// read to the end and moved to the "already searched" list.
    fp: *mut FILE,
    name: Name,
    /// The line last read from it, for the listing.
    lnum: LineNr,
    /// Whether a match has been shown in this file or in one it includes.
    matched: bool,
}

impl Drop for SearchedFile {
    fn drop(&mut self) {
        if !self.fp.is_null() {
            // SAFETY: an open stdio stream this struct owns.
            unsafe { fclose(self.fp) };
        }
    }
}

/// The include stack.
///
/// Upstream keeps both halves in one array, growing from the bottom for
/// the files it has open and down from the top for the ones it has
/// finished with — the second half is what makes "have I already been
/// here?" answerable after a file has been closed.
struct FileStack {
    /// Currently open, outermost first.
    open: Vec<SearchedFile>,
    /// Read to the end already, oldest first.
    done: Vec<SearchedFile>,
}

impl FileStack {
    const fn new() -> Self {
        FileStack {
            open: Vec::new(),
            done: Vec::new(),
        }
    }

    /// How deep the walk is inside included files; -1 in the buffer
    /// itself.
    fn depth(&self) -> c_int {
        self.open.len() as c_int - 1
    }

    /// Whether `name` has been searched already, and if so whether a
    /// match was found in it.
    ///
    /// Upstream scans the open files outermost-first and then the
    /// finished ones newest-first, and stops at the first equal one.
    ///
    /// # Safety
    /// `name` must be a NUL-terminated path.
    unsafe fn already_searched(&self, name: *mut c_char) -> Option<bool> {
        for file in self.open.iter().chain(self.done.iter().rev()) {
            if unsafe { path_full_compare(name, file.name.as_ptr(), true, true) } as c_uint
                & kEqualFiles as c_uint
                != 0
            {
                return Some(file.matched);
            }
        }
        None
    }

    /// Close the innermost file and remember it as searched.
    fn close_innermost(&mut self) {
        if let Some(mut file) = self.open.pop() {
            if !file.fp.is_null() {
                // SAFETY: an open stream this stack owns.
                unsafe { fclose(file.fp) };
                file.fp = ptr::null_mut();
            }
            self.done.push(file);
        }
    }

    /// The innermost open file, which the walk is reading.
    fn innermost(&mut self) -> &mut SearchedFile {
        self.open.last_mut().expect("depth >= 0")
    }
}

/// The three patterns the walk compiles: what to look for, `'include'`
/// and `'define'`. Any of them may be absent.
pub(crate) struct Patterns {
    pub(crate) pat: RegMatch,
    pub(crate) incl: RegMatch,
    pub(crate) def: RegMatch,
}

impl Drop for Patterns {
    fn drop(&mut self) {
        // SAFETY: each `regprog` is either null or a compiled program
        // this struct owns.
        unsafe { vim_regfree(self.pat.regprog) };
        unsafe { vim_regfree(self.incl.regprog) };
        unsafe { vim_regfree(self.def.regprog) };
    }
}

/// Compile `pat` with the current `'magic'`, answering false when it did
/// not compile.
///
/// # Safety
/// `pat` must be NUL-terminated; `into` must be writable.
unsafe fn compile(into: &mut RegMatch, pat: *const c_char, ignore_case: bool) -> bool {
    into.regprog = unsafe { vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 }) };
    into.rm_ic = ignore_case;
    !into.regprog.is_null()
}

/// Compile the three patterns.
///
/// Answers `None` when one of them failed, which abandons the search.
///
/// # Safety
/// `pattern` must point at `len` readable bytes.
unsafe fn compile_patterns(
    pattern: *mut c_char,
    len: size_t,
    whole: bool,
    kind: c_int,
) -> Option<Patterns> {
    let mut pats = Patterns {
        pat: RegMatch::default(),
        incl: RegMatch::default(),
        def: RegMatch::default(),
    };

    if kind != CHECK_PATH && kind != FIND_DEFINE && !compl_status_sol() {
        // When CONT_SOL is set, comparing "ptr" with the beginning of
        // the line is faster than quote_meta/regcomp/regexec of it
        // -- Acevedo.
        let mut pat = Vec::with_capacity(len + 5);
        if whole {
            pat.extend_from_slice(b"\\<");
        }
        // Upstream builds this with "%.*s", which stops at a NUL
        // inside the first `len` bytes; copying them all would put
        // the closing "\>" past the terminator.
        let bytes = unsafe { core::slice::from_raw_parts(pattern as *const u8, len) };
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(len);
        pat.extend_from_slice(&bytes[..end]);
        if whole {
            pat.extend_from_slice(b"\\>");
        }
        pat.push(0);
        // Ignore case according to 'ignorecase', 'smartcase' and the
        // pattern itself.
        let ic = unsafe { ignorecase(pat.as_ptr() as *mut c_char) };
        if !unsafe { compile(&mut pats.pat, pat.as_ptr() as *const c_char, ic) } {
            return None;
        }
    }

    let inc_opt = include_option();
    if !inc_opt.is_empty() {
        // Don't ignore case in the 'include' pattern.
        if !unsafe { compile(&mut pats.incl, inc_opt.as_ptr(), false) } {
            return None;
        }
    }

    if kind == FIND_DEFINE {
        let def = local_or_global(&Buf::current().b_p_def, P_DEF);
        // Don't ignore case in the 'define' pattern.
        if !def.is_empty() && !unsafe { compile(&mut pats.def, def.as_ptr().cast_mut(), false) } {
            return None;
        }
    }

    Some(pats)
}

/// The effective `'include'`: the buffer-local one, or the global one
/// when it is empty.
fn include_option() -> XString {
    local_or_global(&Buf::current().b_p_inc, P_INC)
}

/// Whether the `'include'` pattern uses `\zs`, which moves the file name
/// from "after the match" to "the match itself".
///
/// # Safety
/// `inc_opt` must be NUL-terminated.
unsafe fn include_uses_zs(inc_opt: *mut c_char) -> bool {
    has_bytes(unsafe { cstr::at(inc_opt) }, b"\\zs")
}

/// Copy line `lnum` into `buf`.
///
/// The copy is made because the regexp may invalidate the line when a
/// mark is used.
///
/// # Safety
/// `lnum` must be a line of the current buffer and `buf` must hold
/// `LSIZE` bytes.
unsafe fn get_line_and_copy(lnum: LineNr, buf: *mut c_char) -> *mut c_char {
    unsafe { xstrlcpy(buf, ml_get(lnum), LSIZE as size_t) };
    buf
}

/// The walk's position and the state that survives from one line to the
/// next.
struct Walk {
    /// The line being looked at. Always points into the walk's own
    /// `LSIZE` buffer — [`get_line_and_copy`] and `vim_fgets` both fill
    /// it — except that the ACTION_EXPAND path reads the *next* line into
    /// it before this one is finished with.
    line: *mut c_char,
    buf: *mut c_char,
    /// The buffer line the walk is on, while `files.depth() == -1`.
    lnum: LineNr,
    end_lnum: LineNr,
    /// A line has already been read into `line`; don't read another.
    /// Upstream keeps a pointer here and only ever tests it.
    already: bool,
    files: FileStack,
    /// The file the current line came from. Compared by *identity*
    /// against `curbuf->b_fname` and against the previous line's file,
    /// which is why it stays a pointer.
    curr_fname: *mut c_char,
    prev_fname: *mut c_char,
    /// How much of the include tree `:checkpath` has printed.
    depth_displayed: c_int,
    did_show: bool,
    found: bool,
    /// The number `:ilist` prints against the next match.
    match_count: c_int,
}

impl Walk {
    /// Advance to the next line, closing included files that have been
    /// read to the end. Answers false when there is nothing left.
    fn next_line(&mut self) -> bool {
        // When reading an included file and hitting end-of-file,
        // close it and continue in the file that included it.
        while self.files.depth() >= 0
            && !self.already
            && unsafe { vim_fgets(self.buf, LSIZE as c_int, self.files.innermost().fp) }
        {
            self.line = self.buf;
            self.files.close_innermost();
            self.curr_fname = if self.files.depth() == -1 {
                Buf::current().name.shown_ptr()
            } else {
                self.files.innermost().name.as_ptr()
            };
            self.depth_displayed = self.depth_displayed.min(self.files.depth());
        }
        if self.files.depth() >= 0 {
            // We could read the line.
            self.line = self.buf;
            self.files.innermost().lnum += 1;
            // Remove any CR and LF from it.
            let mut i = unsafe { cstr::bytes_at(self.line) }.len() as isize;
            if i > 0 && unsafe { *self.line.offset(i - 1) } as c_int == '\n' as c_int {
                i -= 1;
                unsafe { *self.line.offset(i) = NUL as c_char };
            }
            if i > 0 && unsafe { *self.line.offset(i - 1) } as c_int == '\r' as c_int {
                i -= 1;
                unsafe { *self.line.offset(i) = NUL as c_char };
            }
        } else if !self.already {
            self.lnum += 1;
            if self.lnum > self.end_lnum {
                return false;
            }
            self.line = unsafe { get_line_and_copy(self.lnum, self.buf) };
        }
        self.already = false;
        true
    }

    /// Read the line after the current one, for the "add the next word
    /// too" half of insert-mode completion. Answers false at the end.
    fn read_following_line(&mut self) -> bool {
        if self.files.depth() < 0 {
            if self.lnum >= self.end_lnum {
                return false;
            }
            self.lnum += 1;
            self.line = unsafe { get_line_and_copy(self.lnum, self.buf) };
            return true;
        }
        self.line = self.buf;
        !unsafe { vim_fgets(self.buf, LSIZE as c_int, self.files.innermost().fp) }
    }

    /// Where `show_pat_in_path` should read continuation lines from, and
    /// which line number it should report.
    fn source(&mut self) -> (*mut FILE, *mut LineNr) {
        if self.files.depth() == -1 {
            (ptr::null_mut(), &raw mut self.lnum)
        } else {
            let file = self.files.innermost();
            (file.fp, &raw mut file.lnum)
        }
    }
}

/// What the walk does with a line that matches `'include'`.
///
/// # Safety
/// The current buffer and window must be valid.
unsafe fn handle_include(
    walk: &mut Walk,
    pats: &Patterns,
    inc_opt: *mut c_char,
    kind: c_int,
    action: c_int,
    silent: bool,
) {
    let mut progress = [0 as c_char; IOSIZE as usize];
    // A relative name is resolved against the file the line is in.
    let p_fname = if walk.curr_fname == Buf::current().name.shown_ptr() {
        Buf::current().name.full_ptr()
    } else {
        walk.curr_fname
    };
    // SAFETY: the 'include' match's spans are offsets into `walk.line`.
    let start = unsafe { walk.line.add(pats.incl.starts[0].unwrap_or(0)) };
    let end = unsafe { walk.line.add(pats.incl.ends[0].unwrap_or(0)) };
    let flags = FileNameOpts::EXP | FileNameOpts::INCL | FileNameOpts::REL;
    let raw = if unsafe { include_uses_zs(inc_opt) } {
        // Use the text from '\zs' to '\ze' (or the end) of 'include'.
        unsafe {
            find_file_name_in_path(start, end.offset_from(start) as size_t, flags, 1, p_fname)
        }
    } else {
        // Use the text after the match with 'include'.
        unsafe { file_name_in_line(end, 0, flags, 1, p_fname, ptr::null_mut()) }
    };
    let mut new_fname = if raw.is_null() { None } else { Some(Name(raw)) };

    let mut already_searched = false;
    if let Some(name) = &new_fname
        && let Some(matched) = unsafe { walk.files.already_searched(name.as_ptr()) }
    {
        if kind != CHECK_PATH && action == ACTION_SHOW_ALL && matched {
            msg_putchar('\n' as c_int); // cursor below the last one
            if !got_int.get() {
                // Don't display if 'q' was typed at the
                // "--more--" message.
                msg_home_replace(name.as_cstr());
                msg_str(gettext(c" (includes previously listed match)"));
                walk.prev_fname = ptr::null_mut();
            }
        }
        new_fname = None;
        already_searched = true;
    }

    if kind == CHECK_PATH
        && (action == ACTION_SHOW_ALL || (new_fname.is_none() && !already_searched))
    {
        unsafe { show_include_name(walk, pats, inc_opt, action, &new_fname, already_searched) };
    }

    let Some(new_fname) = new_fname else { return };

    // Push the new file onto the stack.
    let fp = unsafe { os_fopen(new_fname.as_ptr(), c"r".as_ptr()) };
    if fp.is_null() {
        return;
    }
    walk.curr_fname = new_fname.as_ptr();
    walk.files.open.push(SearchedFile {
        fp,
        name: new_fname,
        lnum: 0,
        matched: false,
    });
    let name = walk.files.innermost().name.as_ptr();
    if action == ACTION_EXPAND && !shortmess(ShmFlag::COMPLETIONSCAN) && !silent {
        msg_hist_off.set(true); // reset in msg_trunc()
        let (out, room) = (progress.as_mut_ptr(), IOSIZE as size_t);
        // SAFETY: formatting a NUL-terminated file name into a buffer of
        // `IOSIZE` bytes, whose size is passed with it.
        unsafe {
            let fmt = gettext(c"Scanning included file: %s");
            vim_snprintf(out, room, fmt.as_ptr(), name)
        };
        unsafe { msg_trunc(progress.as_mut_ptr(), true, HLF_R) };
    } else if p_verbose() >= 5 {
        verbose_enter();
        // SAFETY: a static, translated message and a NUL-terminated name.
        let name = unsafe { c_str(name) };
        smsg!(0, "Searching included file {name}");
        verbose_leave();
    }
}

/// Print one line of the `:checkpath` tree.
///
/// # Safety
/// As [`handle_include`].
unsafe fn show_include_name(
    walk: &mut Walk,
    pats: &Patterns,
    inc_opt: *mut c_char,
    action: c_int,
    new_fname: &Option<Name>,
    already_searched: bool,
) {
    if walk.did_show {
        msg_putchar('\n' as c_int); // cursor below the last one
    } else {
        gotocmdline(true); // cursor at the status line
        msg_title(gettext(c"--- Included files "));
        if action != ACTION_SHOW_ALL {
            msg_title(gettext(c"not found "));
        }
        msg_title(gettext(c"in path ---\n"));
    }
    walk.did_show = true;

    // Catch the tree display up with how deep the walk has gone.
    while walk.depth_displayed < walk.files.depth() && !got_int.get() {
        walk.depth_displayed += 1;
        for _ in 0..walk.depth_displayed {
            msg_str(c"  ");
        }
        msg_home_replace(
            walk.files.open[walk.depth_displayed as usize]
                .name
                .as_cstr(),
        );
        msg_str(c" -->\n");
    }
    if got_int.get() {
        // Don't display if 'q' was typed at the "--more--" message.
        return;
    }
    for _ in 0..=walk.depth_displayed {
        msg_str(c"  ");
    }

    match new_fname {
        // Using the resolved name is more reliable, e.g. when
        // 'includeexpr' is set.
        Some(name) => {
            msg_display(name.as_cstr(), HLF_D, false);
        }
        None => {
            // Isolate the file name off the line, including the
            // surrounding "" or <> if they are there.
            let line = walk.line;
            let (mut p, mut i) = if unsafe { include_uses_zs(inc_opt) } {
                // The pattern contains \zs: use the match.
                let span = pats.incl.group(0).unwrap_or(0..0);
                // SAFETY: an offset into the line just matched.
                let start = unsafe { line.add(span.start) };
                (start, (span.end - span.start) as c_int)
            } else {
                // Find the file name after the end of the match.
                // SAFETY: as above.
                let mut p = unsafe { line.add(pats.incl.ends[0].unwrap_or(0)) };
                while unsafe { *p } as c_int != NUL && !unsafe { vim_isfilec(*p as u8 as c_int) } {
                    p = unsafe { p.offset(1) };
                }
                let mut i = 0;
                while unsafe { vim_isfilec(*p.offset(i as isize) as u8 as c_int) } {
                    i += 1;
                }
                (p, i)
            };
            if i == 0 {
                // Nothing found; use the rest of the line.
                // SAFETY: an offset into the line just matched.
                p = unsafe { line.add(pats.incl.ends[0].unwrap_or(0)) };
                i = unsafe { cstr::bytes_at(p) }.len() as c_int;
            } else if p > line {
                // Avoid looking before the start of the line, which
                // can happen when \zs appears in the pattern.
                if unsafe { *p.offset(-1) } as c_int == '"' as c_int
                    || unsafe { *p.offset(-1) } as c_int == '<' as c_int
                {
                    p = unsafe { p.offset(-1) };
                    i += 1;
                }
                if unsafe { *p.offset(i as isize) } as c_int == '"' as c_int
                    || unsafe { *p.offset(i as isize) } as c_int == '>' as c_int
                {
                    i += 1;
                }
            }
            let save = unsafe { *p.offset(i as isize) };
            unsafe { *p.offset(i as isize) = NUL as c_char };
            msg_display(unsafe { cstr::at(p) }, HLF_D, false);
            unsafe { *p.offset(i as isize) = save };
        }
    }

    if new_fname.is_none() && action == ACTION_SHOW_ALL {
        if already_searched {
            msg_str(gettext(c"  (Already listed)"));
        } else {
            msg_str(gettext(c"  NOT FOUND"));
        }
    }
}

/// What the walk should do after handling a match.
enum After {
    /// Done with this match; the rest of the line may still be searched,
    /// resuming the pattern search from here. Upstream leaves this in
    /// `p`, which the ACTION_EXPAND arm has walked to the end of the word
    /// it just offered -- or, when it read the following line, into that.
    Resume(*mut c_char),
    /// Go on to the next line.
    NextLine,
    /// Stop the whole search.
    Stop,
}

/// Offer the match to insert-mode completion (ACTION_EXPAND).
///
/// # Safety
/// As [`handle_include`]; `startp` must be inside `walk.line`.
unsafe fn expand_match(walk: &mut Walk, startp: *mut c_char, dir: &mut Direction) -> After {
    // Where a joined `CTRL-X CTRL-I` line is assembled; upstream shares
    // `IObuff`, which the completion it feeds writes again.
    let mut joined = [0 as c_char; IOSIZE as usize];
    let mut cont_s_ipos = false;
    if walk.files.depth() == -1 && walk.lnum == Win::current().w_cursor.lnum {
        return After::Stop;
    }
    walk.found = true;
    let mut p = startp;
    let mut aux = p;
    if compl_status_adding() && unsafe { cstr::bytes_at(p) }.len() as c_int >= ins_compl_len() {
        p = unsafe { p.offset(ins_compl_len() as isize) };
        if unsafe { vim_iswordp(p) } {
            return After::Resume(p);
        }
        p = unsafe { find_word_start(p) };
    }
    p = unsafe { find_word_end(p) };
    let mut i = unsafe { p.offset_from(aux) } as c_int;

    if compl_status_adding() && i == ins_compl_len() {
        // IOSIZE > compl_length, so the copy fits.
        let iobuff = joined.as_mut_ptr();
        unsafe { ptr::copy_nonoverlapping(aux, iobuff, i as usize) };

        // Get the next line: from the current buffer below depth 0,
        // otherwise from the included file. Give up when past the
        // last line.
        if !walk.read_following_line() {
            return After::Resume(p);
        }

        // A line was read; remember that, so that it is looked at
        // rather than skipped. When depth >= 0 the file's line number
        // is bumped further below -- Acevedo.
        walk.already = true;
        p = unsafe { skipwhite(walk.line) };
        aux = p;
        p = unsafe { find_word_start(p) };
        p = unsafe { find_word_end(p) };
        if p > aux {
            if unsafe { *aux } as c_int != ')' as c_int
                && unsafe { *iobuff.offset(i as isize - 1) } as c_int != TAB
            {
                if unsafe { *iobuff.offset(i as isize - 1) } as c_int != ' ' as c_int {
                    unsafe { *iobuff.offset(i as isize) = ' ' as c_char };
                    i += 1;
                }
                // `joined` =~ "\(\k\|\i\).* ", so i >= 2.
                if p_js()
                    && (unsafe { *iobuff.offset(i as isize - 2) } as c_int == '.' as c_int
                        || unsafe { *iobuff.offset(i as isize - 2) } as c_int == '?' as c_int
                        || unsafe { *iobuff.offset(i as isize - 2) } as c_int == '!' as c_int)
                {
                    unsafe { *iobuff.offset(i as isize) = ' ' as c_char };
                    i += 1;
                }
            }
            // Copy as much of the new word as fits.
            if unsafe { p.offset_from(aux) } >= (IOSIZE - i) as isize {
                p = unsafe { aux.offset((IOSIZE - i - 1) as isize) };
            }
            // SAFETY: `aux..p` is inside the word just scanned, and the
            // clamp above keeps the copy inside `iobuff`'s `IOSIZE` bytes.
            unsafe {
                let n = p.offset_from(aux) as usize;
                ptr::copy_nonoverlapping(aux, iobuff.offset(i as isize), n)
            };
            i += unsafe { p.offset_from(aux) } as c_int;
            cont_s_ipos = true;
        }
        unsafe { *iobuff.offset(i as isize) = NUL as c_char };
        aux = iobuff;

        if i == ins_compl_len() {
            return After::Resume(p);
        }
    }

    let from_file = if walk.curr_fname == Buf::current().name.shown_ptr() {
        ptr::null_mut()
    } else {
        walk.curr_fname
    };
    match unsafe { ins_compl_add_infercase(aux, i, p_ic(), from_file, *dir, cont_s_ipos, 0) } {
        // If dir was BACKWARD, honour it just once.
        r if r == OK => *dir = FORWARD,
        r if r == FAIL => return After::Stop,
        _ => {}
    }
    After::Resume(p)
}

/// List the match (ACTION_SHOW_ALL).
fn list_match(walk: &mut Walk, kind: c_int, action: c_int) {
    walk.found = true;
    if !walk.did_show {
        gotocmdline(true); // cursor at the status line
    }
    if walk.curr_fname != walk.prev_fname {
        if walk.did_show {
            msg_putchar('\n' as c_int); // cursor below the last one
        }
        if !got_int.get() {
            // Don't display if 'q' was typed at the "--more--"
            // message.
            msg_home_replace(unsafe { cstr::at(walk.curr_fname) });
        }
        walk.prev_fname = walk.curr_fname;
    }
    walk.did_show = true;
    if !got_int.get() {
        let count = walk.match_count;
        walk.match_count += 1;
        let line = walk.line;
        let (fp, lnum) = walk.source();
        unsafe { show_pat_in_path(line, kind, true, action, fp, lnum, count) };
    }
    // Set the matched flag for this file and for every one that
    // includes it.
    for file in &mut walk.files.open {
        file.matched = true;
    }
}

/// Show or jump to the match — the last one of `count`.
///
/// # Safety
/// As [`handle_include`].
unsafe fn goto_match(
    walk: &mut Walk,
    startp: *mut c_char,
    kind: c_int,
    action: c_int,
    forceit: bool,
    tagpreview: c_int,
) -> After {
    walk.found = true;
    let mut curwin_save: Option<WinId> = None;
    if walk.files.depth() == -1 && walk.lnum == Win::current().w_cursor.lnum && tagpreview == 0 {
        emsg(gettext(c"E387: Match is on current line"));
    } else if action == ACTION_SHOW {
        let did_show = walk.did_show;
        let line = walk.line;
        let (fp, lnum) = walk.source();
        unsafe { show_pat_in_path(line, kind, did_show, action, fp, lnum, 1) };
        walk.did_show = true;
    } else {
        // ":psearch" uses the preview window.
        if tagpreview != 0 {
            curwin_save = Some(Win::current().id());
            prepare_tagpreview(true);
        }
        if action == ACTION_SPLIT {
            if win_split(0, 0).is_err() {
                return After::Stop;
            }
            // RESET_BINDING: a new window does not inherit
            // 'scrollbind'/'cursorbind'.
            Win::current().w_onebuf_opt.wo_scb = 0;
            Win::current().w_onebuf_opt.wo_crb = 0;
        }
        if walk.files.depth() == -1 {
            // The match is in the current file.
            if tagpreview != 0 {
                let Some(saved) = curwin_save.and_then(valid_win) else {
                    return After::Stop;
                };
                // GETFILE_SUCCESS: anything but a positive answer.
                let handle = saved.buffer().handle as c_int;
                let (no_name, no_alt) = (ptr::null_mut(), ptr::null_mut());
                // SAFETY: jumping to a buffer named by its own handle.
                let failed = unsafe { getfile(handle, no_name, no_alt, true, walk.lnum, forceit) };
                if failed > 0 {
                    return After::Stop; // failed to jump to the file
                }
            } else {
                setpcmark();
            }
            Win::current().w_cursor.lnum = walk.lnum;
            check_cursor(Win::current());
        } else {
            let file = walk.files.innermost();
            let (name, flnum) = (file.name.as_ptr(), file.lnum);
            if unsafe { getfile(0, name, ptr::null_mut(), true, flnum, forceit) } > 0 {
                return After::Stop; // failed to jump to the file
            }
            // Autocommands may have changed the line number; that is
            // not wanted here.
            Win::current().w_cursor.lnum = flnum;
        }
    }
    if action != ACTION_SHOW {
        Win::current().w_cursor.col = unsafe { startp.offset_from(walk.line) } as ColNr;
        Win::current().w_set_curswant = true;
    }

    if let Some(saved) = curwin_save.and_then(valid_win).filter(|_| tagpreview != 0)
        && !saved.is_current()
    {
        // Return the cursor to where it was.
        validate_cursor(Win::current());
        redraw_later(Win::current(), UPD_VALID);
        win_enter(saved, true);
    }
    After::Stop
}

/// Find identifiers or defines in the current file and everything it
/// includes.
///
/// `pattern`/`len` is the pattern; with `p_ic` and `compl_status_sol()` it
/// must be lowercase. `whole` matches whole words only, `skip_comments`
/// ignores matches inside comments, `kind` is what is being looked for
/// (`FIND_ANY`, `FIND_DEFINE` or `CHECK_PATH`), `action` is what to do
/// with a match, `start_lnum`/`end_lnum` bound the buffer lines to start
/// from, `forceit` always switches to the file found, and `silent`
/// suppresses the messages for `ACTION_EXPAND`.
///
/// `pattern` is empty when `kind` is `CHECK_PATH`.
#[allow(clippy::too_many_arguments)]
pub fn find_pattern_in_path(
    pattern: &[u8],
    dir: Direction,
    whole: bool,
    skip_comments: bool,
    kind: c_int,
    count: c_int,
    action: c_int,
    start_lnum: LineNr,
    end_lnum: LineNr,
    forceit: bool,
    silent: bool,
) {
    let mut dir = dir;
    let mut count = count;
    let tagpreview = g_do_tagpreview.get();
    // SAFETY: the caller's pattern, `pattern.len()` bytes of it, which the
    // compiler only reads.
    let pat = pattern.as_ptr().cast::<c_char>().cast_mut();
    let found = unsafe { compile_patterns(pat, pattern.len(), whole, kind) };
    let Some(mut pats) = found else {
        return;
    };
    let inc_opt = include_option();
    let inc_opt = inc_opt.as_ptr().cast_mut();

    let mut file_line = vec![0 as c_char; LSIZE];
    let end_lnum = end_lnum.min(Buf::current().b_ml.ml_line_count);
    // Do at least one line.
    let lnum = start_lnum.min(end_lnum);
    let mut walk = Walk {
        line: unsafe { get_line_and_copy(lnum, file_line.as_mut_ptr()) },
        buf: file_line.as_mut_ptr(),
        lnum,
        end_lnum,
        already: false,
        files: FileStack::new(),
        curr_fname: Buf::current().name.shown_ptr(),
        prev_fname: ptr::null_mut(),
        depth_displayed: -1,
        did_show: false,
        found: false,
        match_count: 1,
    };

    loop {
        // SAFETY: the walk's line is NUL-terminated.
        if !pats.incl.regprog.is_null()
            && vim_regexec(&mut pats.incl, unsafe { cstr::at(walk.line) }, 0)
        {
            unsafe { handle_include(&mut walk, &pats, inc_opt, kind, action, silent) };
        } else {
            // Look for a match, possibly several times in one line.
            let mut from = walk.line;
            let mut stop = false;
            while let Some(startp) = unsafe {
                match_on_line(
                    walk.line,
                    &mut pats,
                    from,
                    pat,
                    pattern.len(),
                    whole,
                    skip_comments,
                )
            } {
                let after = if action == ACTION_EXPAND {
                    unsafe { expand_match(&mut walk, startp, &mut dir) }
                } else if action == ACTION_SHOW_ALL {
                    list_match(&mut walk, kind, action);
                    After::NextLine
                } else {
                    count -= 1;
                    if count > 0 {
                        After::NextLine
                    } else {
                        unsafe { goto_match(&mut walk, startp, kind, action, forceit, tagpreview) }
                    }
                };
                let After::Resume(resume) = after else {
                    if matches!(after, After::Stop) {
                        stop = true;
                    }
                    break;
                };
                // Look for other matches in the rest of the line,
                // when there is any of it left.
                if pats.def.regprog.is_null()
                    && action == ACTION_EXPAND
                    && !compl_status_sol()
                    && unsafe { *startp } as c_int != NUL
                    && unsafe { *startp.offset(utfc_ptr2len(startp) as isize) } as c_int != NUL
                {
                    from = resume;
                    continue;
                }
                break;
            }
            if stop {
                break;
            }
        }

        line_breakcheck();
        if action == ACTION_EXPAND {
            ins_compl_check_keys(30, false);
        }
        if got_int.get() || ins_compl_interrupted() {
            break;
        }
        if !walk.next_line() {
            break;
        }
    }

    // Everything still open is closed by FileStack's Drop.
    drop(walk.files);

    if kind == CHECK_PATH {
        if !walk.did_show {
            if action != ACTION_SHOW_ALL {
                msg(gettext(c"All included files were found"), 0);
            } else {
                msg(gettext(c"No included files"), 0);
            }
        }
    } else if !walk.found && action != ACTION_EXPAND && !silent {
        if got_int.get() || ins_compl_interrupted() {
            emsg(gettext(e_interr));
        } else if kind == FIND_DEFINE {
            emsg(gettext(c"E388: Couldn't find definition"));
        } else {
            emsg(gettext(c"E389: Couldn't find pattern"));
        }
    }
    if action == ACTION_SHOW || action == ACTION_SHOW_ALL {
        msg_end();
    }
}
