//! Searching a file and everything it includes.
//!
//! [`find_pattern_in_path`] is `[i`, `[d`, `:ilist`, `:isearch`,
//! `:checkpath` and insert-mode `CTRL-X CTRL-I`/`CTRL-X CTRL-D` — one
//! walk with six things it may do when it finds a match ([`Action`]) and
//! three things it may be looking for ([`Kind`]). It reads the current
//! buffer line by line; every line matching `'include'` names a file that
//! is pushed onto a stack and read the same way, `'define'` narrows what
//! counts as a match, and `'path'` is where the names are resolved.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::is_word_at;
use crate::cstr;
use crate::file_search::FileNameOpts;
use crate::fileio::Fgets;
use crate::highlight_group::{HLF_D, HLF_R};
use crate::insexpand::{word_end, word_start};
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::message::msg_display_bytes;
use crate::message_fmt::msg_cstr;
use crate::option::vars::P_DEF;
use crate::option::vars::P_INC;
use crate::optionstr::{OptString, local_or_global};
use crate::regexp::RE_MAGIC;
use crate::smsg;
use crate::strings::has_bytes;
use crate::types::{FAIL, IOSIZE, NUL, OK, ShmFlag};
use crate::window::valid_win;
use crate::winlayer::WinId;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

const CHECK_PATH: c_int = super::CHECK_PATH as c_int;
const FIND_DEFINE: c_int = super::FIND_DEFINE as c_int;
const ACTION_SHOW: c_int = super::ACTION_SHOW as c_int;
const ACTION_SPLIT: c_int = super::ACTION_SPLIT as c_int;
const ACTION_SHOW_ALL: c_int = super::ACTION_SHOW_ALL as c_int;
const ACTION_EXPAND: c_int = super::ACTION_EXPAND as c_int;
const LSIZE: usize = super::LSIZE as usize;

/// One file on the include stack.
struct SearchedFile {
    /// Open while the walk is reading the file; `None` once it has been
    /// read to the end and moved to the "already searched" list.
    reader: Option<Fgets>,
    name: XString,
    /// Which file this is: upstream compared the name's address.
    id: u32,
    /// The line last read from it, for the listing.
    lnum: LineNr,
    /// Whether a match has been shown in this file or in one it includes.
    matched: bool,
}

/// Where a line came from: the current buffer, or an included file by its
/// [`SearchedFile::id`].
///
/// Upstream compared file *name pointers*, and the buffer's could be null
/// for a buffer with no name; [`Walk::curr_fname`] and its kin are `None`
/// exactly where that pointer was.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FileRef {
    Buffer,
    Included(u32),
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
    /// The id the next file pushed gets.
    next_id: u32,
}

impl FileStack {
    const fn new() -> Self {
        FileStack {
            open: Vec::new(),
            done: Vec::new(),
            next_id: 0,
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
    fn already_searched(&self, name: &CStr) -> Option<bool> {
        for file in self.open.iter().chain(self.done.iter().rev()) {
            let same = path_full_compare(name, file.name.as_cstr(), true, true) as c_uint
                & kEqualFiles as c_uint
                != 0;
            if same {
                return Some(file.matched);
            }
        }
        None
    }

    /// Close the innermost file and remember it as searched.
    fn close_innermost(&mut self) {
        if let Some(mut file) = self.open.pop() {
            file.reader = None;
            self.done.push(file);
        }
    }

    /// The innermost open file, which the walk is reading.
    fn innermost(&mut self) -> &mut SearchedFile {
        self.open.last_mut().expect("depth >= 0")
    }

    /// The name of the open file `id`.
    fn name_of(&self, id: u32) -> Option<&XString> {
        self.open
            .iter()
            .find(|file| file.id == id)
            .map(|file| &file.name)
    }
}

/// The three patterns the walk compiles: what to look for, `'include'`
/// and `'define'`. Any of them may be absent.
pub(crate) struct Patterns {
    pub(crate) pat: RegMatch,
    pub(crate) incl: RegMatch,
    pub(crate) def: RegMatch,
}

/// Compile `pat` with the current `'magic'`, answering false when it did
/// not compile.
fn compile(into: &mut RegMatch, pat: &CStr, ignore_case: bool) -> bool {
    into.regprog = vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 });
    into.rm_ic = ignore_case;
    into.regprog.is_some()
}

/// Compile the three patterns.
///
/// Answers `None` when one of them failed, which abandons the search.
fn compile_patterns(pattern: &[u8], whole: bool, kind: c_int) -> Option<Patterns> {
    let mut pats = Patterns {
        pat: RegMatch::default(),
        incl: RegMatch::default(),
        def: RegMatch::default(),
    };

    if kind != CHECK_PATH && kind != FIND_DEFINE && !compl_status_sol() {
        // When CONT_SOL is set, comparing "ptr" with the beginning of
        // the line is faster than quote_meta/regcomp/regexec of it
        // -- Acevedo.
        let mut pat = Vec::with_capacity(pattern.len() + 5);
        if whole {
            pat.extend_from_slice(b"\\<");
        }
        // Upstream builds this with "%.*s", which stops at a NUL inside
        // the pattern; copying it all would put the closing "\>" past the
        // terminator.
        pat.extend_from_slice(cstr::until_nul(pattern));
        if whole {
            pat.extend_from_slice(b"\\>");
        }
        // Ignore case according to 'ignorecase', 'smartcase' and the
        // pattern itself.
        let ic = ignorecase(&pat);
        if !compile(&mut pats.pat, &cstr::owned(&pat), ic) {
            return None;
        }
    }

    let inc_opt = include_option();
    // Don't ignore case in the 'include' pattern.
    if !inc_opt.is_empty() && !compile(&mut pats.incl, inc_opt.as_cstr(), false) {
        return None;
    }

    if kind == FIND_DEFINE {
        let def = local_or_global(&Buf::current().b_p_def, P_DEF).get();
        // Don't ignore case in the 'define' pattern.
        if !def.is_empty() && !compile(&mut pats.def, def.as_cstr(), false) {
            return None;
        }
    }

    Some(pats)
}

/// The effective `'include'`: the buffer-local one, or the global one
/// when it is empty.
fn include_option() -> XString {
    local_or_global(&Buf::current().b_p_inc, P_INC).get()
}

/// Line `lnum` of the current buffer, cut to what upstream's `LSIZE` buffer
/// held, terminated.
///
/// The copy is made because the regexp may invalidate the line when a
/// mark is used.
fn get_line_and_copy(lnum: LineNr) -> Vec<u8> {
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let mut copy = cstr::until_nul(&line[..line.len().min(LSIZE - 1)]).to_vec();
    copy.push(NUL as u8);
    copy
}

/// A line read from an included file, cut at its first NUL and terminated.
fn read_line_copy(mut read: Vec<u8>) -> Vec<u8> {
    let end = cstr::until_nul(&read).len();
    read.truncate(end);
    read.push(NUL as u8);
    read
}

/// The walk's position and the state that survives from one line to the
/// next.
struct Walk {
    /// The line being looked at, terminated: a copy of a buffer line or a
    /// line read from an included file -- except that the ACTION_EXPAND
    /// path reads the *next* line into it before this one is finished with.
    line: Vec<u8>,
    /// The buffer line the walk is on, while `files.depth() == -1`.
    lnum: LineNr,
    end_lnum: LineNr,
    /// A line has already been read into `line`; don't read another.
    already: bool,
    files: FileStack,
    /// The file the current line came from, compared by identity against
    /// the current buffer and against the previous line's file.
    curr_fname: Option<FileRef>,
    prev_fname: Option<FileRef>,
    /// How much of the include tree `:checkpath` has printed.
    depth_displayed: c_int,
    did_show: bool,
    found: bool,
    /// The number `:ilist` prints against the next match.
    match_count: c_int,
}

/// The current buffer as a line's source: `None` when it has no name, as
/// its null name pointer was upstream.
fn buffer_ref() -> Option<FileRef> {
    Buf::current().name.shown().map(|_| FileRef::Buffer)
}

impl Walk {
    /// The line, its terminator not included.
    fn text(&self) -> &[u8] {
        &self.line[..self.line.len() - 1]
    }

    /// The line, as the C string the matchers read.
    fn cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.line).expect("the walk's line is terminated")
    }

    /// The name of the file `fname` names, as it is shown.
    fn name_of(&self, fname: Option<FileRef>) -> Option<XString> {
        match fname? {
            FileRef::Buffer => Buf::current().name.shown().map(XString::from_cstr),
            FileRef::Included(id) => self.files.name_of(id).cloned(),
        }
    }

    /// Advance to the next line, closing included files that have been
    /// read to the end. Answers false when there is nothing left.
    fn next_line(&mut self) -> bool {
        // When reading an included file and hitting end-of-file,
        // close it and continue in the file that included it.
        let mut read = None;
        while self.files.depth() >= 0 && !self.already {
            let mut line = Vec::new();
            let reader = self.files.innermost().reader.as_mut();
            if reader.is_some_and(|reader| reader.line_capped(&mut line, LSIZE)) {
                read = Some(line);
                break;
            }
            self.files.close_innermost();
            self.curr_fname = if self.files.depth() == -1 {
                buffer_ref()
            } else {
                Some(FileRef::Included(self.files.innermost().id))
            };
            self.depth_displayed = self.depth_displayed.min(self.files.depth());
        }
        if self.files.depth() >= 0 {
            // We could read the line.
            if let Some(line) = read {
                self.line = read_line_copy(line);
            }
            self.files.innermost().lnum += 1;
            // Remove any CR and LF from it.
            for strip in *b"\n\r" {
                let len = self.line.len() - 1;
                if len > 0 && self.line[len - 1] == strip {
                    self.line.truncate(len - 1);
                    self.line.push(NUL as u8);
                }
            }
        } else if !self.already {
            self.lnum += 1;
            if self.lnum > self.end_lnum {
                return false;
            }
            self.line = get_line_and_copy(self.lnum);
        }
        self.already = false;
        true
    }

    /// Read the line after the current one, for the "add the next word
    /// too" half of insert-mode completion. Answers false at the end, with
    /// the line left as it was.
    fn read_following_line(&mut self) -> bool {
        if self.files.depth() < 0 {
            if self.lnum >= self.end_lnum {
                return false;
            }
            self.lnum += 1;
            self.line = get_line_and_copy(self.lnum);
            return true;
        }
        let mut line = Vec::new();
        let reader = self.files.innermost().reader.as_mut();
        if !reader.is_some_and(|reader| reader.line_capped(&mut line, LSIZE)) {
            return false;
        }
        self.line = read_line_copy(line);
        true
    }

    /// Show the current line with `show_pat_in_path`, continuation lines
    /// read from where it came from.
    fn show(&mut self, kind: c_int, did_show: bool, action: c_int, count: c_int) {
        let Walk {
            line, files, lnum, ..
        } = self;
        let line = &line[..line.len() - 1];
        if files.depth() == -1 {
            show_pat_in_path(line, kind, did_show, action, None, lnum, count);
        } else {
            let file = files.innermost();
            let stream = file.reader.as_mut();
            show_pat_in_path(line, kind, did_show, action, stream, &mut file.lnum, count);
        }
    }
}

/// What the walk does with a line that matches `'include'`.
fn handle_include(
    walk: &mut Walk,
    pats: &Patterns,
    uses_zs: bool,
    kind: c_int,
    action: c_int,
    silent: bool,
) {
    // A relative name is resolved against the file the line is in. A copy:
    // the lookup can run 'includeexpr', which may rename the buffer.
    let p_fname = match walk.curr_fname {
        None | Some(FileRef::Buffer) => Buf::current().name.full().map(XString::from_cstr),
        included => walk.name_of(included),
    };
    let p_fname = p_fname.as_ref().map(XString::as_cstr);
    let line = walk.text();
    // The 'include' match's spans are offsets into the line.
    let start = pats.incl.starts[0].unwrap_or(0);
    let end = pats.incl.ends[0].unwrap_or(0);
    let flags = FileNameOpts::EXP | FileNameOpts::INCL | FileNameOpts::REL;
    let found = if uses_zs {
        // Use the text from '\zs' to '\ze' (or the end) of 'include'.
        find_file_name_in_path(&line[start..end], flags, 1, p_fname)
    } else {
        // Use the text after the match with 'include'.
        file_name_in_line(&line[end..], 0, flags, 1, p_fname, None)
    };
    let mut new_fname = found;

    let mut already_searched = false;
    if let Some(name) = &new_fname
        && let Some(matched) = walk.files.already_searched(name.as_cstr())
    {
        if kind != CHECK_PATH && action == ACTION_SHOW_ALL && matched {
            msg_putchar('\n' as c_int); // cursor below the last one
            if !got_int.get() {
                // Don't display if 'q' was typed at the
                // "--more--" message.
                msg_home_replace(name.as_cstr());
                msg_str(gettext(c" (includes previously listed match)"));
                walk.prev_fname = None;
            }
        }
        new_fname = None;
        already_searched = true;
    }

    if kind == CHECK_PATH
        && (action == ACTION_SHOW_ALL || (new_fname.is_none() && !already_searched))
    {
        show_include_name(
            walk,
            pats,
            uses_zs,
            action,
            new_fname.as_ref(),
            already_searched,
        );
    }

    let Some(new_fname) = new_fname else { return };

    // Push the new file onto the stack.
    let Ok(file) = File::open(OsStr::from_bytes(&new_fname)) else {
        return;
    };
    let id = walk.files.next_id;
    walk.files.next_id += 1;
    walk.curr_fname = Some(FileRef::Included(id));
    walk.files.open.push(SearchedFile {
        reader: Some(Fgets::new(file)),
        name: new_fname,
        id,
        lnum: 0,
        matched: false,
    });
    let name = walk.files.innermost().name.as_cstr();
    if action == ACTION_EXPAND && !shortmess(ShmFlag::COMPLETIONSCAN) && !silent {
        msg_hist_off.set(true); // reset in msg_fit_head()
        // Upstream formats into `IObuff`, and the name is cut there.
        let fmt = gettext(c"Scanning included file: %s");
        let progress =
            XString::from_bytes(&crate::formatted!(IOSIZE as usize, fmt.to_bytes(), name));
        msg_fit_head(progress.as_cstr(), true, HLF_R);
    } else if p_verbose() >= 5 {
        verbose_enter();
        let name = msg_cstr(name);
        smsg!(0, "Searching included file {name}");
        verbose_leave();
    }
}

/// Print one line of the `:checkpath` tree.
fn show_include_name(
    walk: &mut Walk,
    pats: &Patterns,
    uses_zs: bool,
    action: c_int,
    new_fname: Option<&XString>,
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
        let shown = &walk.files.open[walk.depth_displayed as usize].name;
        msg_home_replace(shown.as_cstr());
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
            let line = walk.text();
            let at = |i: usize| cstr::byte_at(line, i);
            let match_end = pats.incl.ends[0].unwrap_or(0);
            let (mut p, mut i) = if uses_zs {
                // The pattern contains \zs: use the match.
                let span = pats.incl.group(0).unwrap_or(0..0);
                (span.start, span.end - span.start)
            } else {
                // Find the file name after the end of the match.
                let mut p = match_end;
                while at(p) != NUL as u8 && !vim_isfilec(c_int::from(at(p))) {
                    p += 1;
                }
                let mut i = 0;
                while vim_isfilec(c_int::from(at(p + i))) {
                    i += 1;
                }
                (p, i)
            };
            if i == 0 {
                // Nothing found; use the rest of the line.
                p = match_end;
                i = line.len().saturating_sub(p);
            } else if p > 0 {
                // Avoid looking before the start of the line, which
                // can happen when \zs appears in the pattern.
                if matches!(at(p - 1), b'"' | b'<') {
                    p -= 1;
                    i += 1;
                }
                if matches!(at(p + i), b'"' | b'>') {
                    i += 1;
                }
            }
            let end = (p + i).min(line.len());
            msg_display_bytes(&line[p.min(end)..end], HLF_D, false);
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
    /// resuming the pattern search from this offset. Upstream leaves this
    /// in `p`, which the ACTION_EXPAND arm has walked to the end of the
    /// word it just offered -- or, when it read the following line, into
    /// that.
    Resume(usize),
    /// Go on to the next line.
    NextLine,
    /// Stop the whole search.
    Stop,
}

/// Offer the match at offset `startp` to insert-mode completion
/// (ACTION_EXPAND).
fn expand_match(walk: &mut Walk, startp: usize, dir: &mut Direction) -> After {
    if walk.files.depth() == -1 && walk.lnum == Win::current().w_cursor.lnum {
        return After::Stop;
    }
    walk.found = true;
    let compl_len = usize::try_from(ins_compl_len()).unwrap_or(0);
    let mut p = startp;
    let mut aux = p;
    if compl_status_adding() && walk.text().len().saturating_sub(p) >= compl_len {
        p += compl_len;
        if vim_iswordp_in(walk.text(), p) {
            return After::Resume(p);
        }
        p = word_start(walk.text(), p);
    }
    p = word_end(walk.text(), p);
    let mut i = p - aux;

    // Where a joined `CTRL-X CTRL-I` line is assembled; upstream shares
    // `IObuff`, which the completion it feeds writes again.
    let mut joined = None;
    let mut cont_s_ipos = false;
    if compl_status_adding() && i == compl_len {
        // IOSIZE > compl_length, so the copy fits.
        let mut iobuff = walk.text()[aux..aux + i].to_vec();

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
        let text = walk.text();
        p = skip::white(text);
        aux = p;
        p = word_start(text, p);
        p = word_end(text, p);
        if p > aux {
            let last = |buf: &[u8], back: usize| buf.len().checked_sub(back).map_or(0, |i| buf[i]);
            if text[aux] != b')' && c_int::from(last(&iobuff, 1)) != TAB {
                if last(&iobuff, 1) != b' ' {
                    iobuff.push(b' ');
                }
                // `joined` =~ "\(\k\|\i\).* ", so it holds at least two.
                if p_js() && matches!(last(&iobuff, 2), b'.' | b'?' | b'!') {
                    iobuff.push(b' ');
                }
            }
            // Copy as much of the new word as fits.
            if p - aux >= IOSIZE as usize - iobuff.len() {
                p = aux + IOSIZE as usize - iobuff.len() - 1;
            }
            iobuff.extend_from_slice(&text[aux..p]);
            cont_s_ipos = true;
        }
        i = iobuff.len();
        joined = Some(iobuff);

        if i == compl_len {
            return After::Resume(p);
        }
    }

    let from_file = match walk.curr_fname {
        None | Some(FileRef::Buffer) => None,
        included => walk.name_of(included),
    };
    let rest = match &joined {
        Some(joined) => &joined[..],
        None => &walk.text()[aux..],
    };
    let from_file = from_file.as_ref().map(XString::as_cstr);
    match ins_compl_add_infercase(rest, i, p_ic(), from_file, *dir, cont_s_ipos, 0) {
        // If dir was BACKWARD, honour it just once.
        r if r == OK => *dir = FORWARD,
        r if r == FAIL => return After::Stop,
        _ => {}
    }
    After::Resume(p)
}

/// Whether the character at `text[at..]` is a word character of the
/// current buffer; the end of `text` is a NUL, which is not.
fn vim_iswordp_in(text: &[u8], at: usize) -> bool {
    match text.get(at..) {
        Some(rest) if !rest.is_empty() => is_word_at(rest, Buf::current()),
        _ => false,
    }
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
            // message. A buffer with no name shows nothing, as
            // upstream's `home_replace` of a null name.
            let name = walk.name_of(walk.curr_fname).unwrap_or_default();
            msg_home_replace(name.as_cstr());
        }
        walk.prev_fname = walk.curr_fname;
    }
    walk.did_show = true;
    if !got_int.get() {
        let count = walk.match_count;
        walk.match_count += 1;
        walk.show(kind, true, action, count);
    }
    // Set the matched flag for this file and for every one that
    // includes it.
    for file in &mut walk.files.open {
        file.matched = true;
    }
}

/// Show or jump to the match at offset `startp` — the last one of `count`.
fn goto_match(
    walk: &mut Walk,
    startp: usize,
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
        walk.show(kind, did_show, action, 1);
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
                // Jumping to a buffer named by its own handle.
                let failed = getfile(handle, None, None, true, walk.lnum, forceit);
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
            // A copy: the jump runs autocommands, and the walk's name is
            // lent for no longer than the call.
            let (name, flnum) = (file.name.as_cstr().to_owned(), file.lnum);
            if getfile(0, Some(&name), None, true, flnum, forceit) > 0 {
                return After::Stop; // failed to jump to the file
            }
            // Autocommands may have changed the line number; that is
            // not wanted here.
            Win::current().w_cursor.lnum = flnum;
        }
    }
    if action != ACTION_SHOW {
        Win::current().w_cursor.col = startp as ColNr;
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
    let Some(mut pats) = compile_patterns(pattern, whole, kind) else {
        return;
    };
    // Whether the 'include' pattern uses `\zs`, which moves the file name
    // from "after the match" to "the match itself".
    let uses_zs = has_bytes(include_option().as_cstr(), b"\\zs");

    let end_lnum = end_lnum.min(Buf::current().b_ml.ml_line_count);
    // Do at least one line.
    let lnum = start_lnum.min(end_lnum);
    let mut walk = Walk {
        line: get_line_and_copy(lnum),
        lnum,
        end_lnum,
        already: false,
        files: FileStack::new(),
        curr_fname: buffer_ref(),
        prev_fname: None,
        depth_displayed: -1,
        did_show: false,
        found: false,
        match_count: 1,
    };

    loop {
        if pats.incl.regprog.is_some() && vim_regexec(&mut pats.incl, walk.cstr(), 0) {
            handle_include(&mut walk, &pats, uses_zs, kind, action, silent);
        } else {
            // Look for a match, possibly several times in one line.
            let mut from = 0;
            let mut stop = false;
            while let Some(startp) =
                match_on_line(walk.cstr(), &mut pats, from, pattern, whole, skip_comments)
            {
                let after = if action == ACTION_EXPAND {
                    expand_match(&mut walk, startp, &mut dir)
                } else if action == ACTION_SHOW_ALL {
                    list_match(&mut walk, kind, action);
                    After::NextLine
                } else {
                    count -= 1;
                    if count > 0 {
                        After::NextLine
                    } else {
                        goto_match(&mut walk, startp, kind, action, forceit, tagpreview)
                    }
                };
                let After::Resume(resume) = after else {
                    if matches!(after, After::Stop) {
                        stop = true;
                    }
                    break;
                };
                // Look for other matches in the rest of the line,
                // when there is any of it left. The offset is read in the
                // line the walk holds now, which the match may have
                // replaced with the following one, as upstream's pointer
                // into the one buffer did.
                let text = walk.text();
                let after_char = startp + c_cluster_len(text.get(startp..).unwrap_or_default());
                if pats.def.regprog.is_none()
                    && action == ACTION_EXPAND
                    && !compl_status_sol()
                    && cstr::byte_at(text, startp) != NUL as u8
                    && cstr::byte_at(text, after_char) != NUL as u8
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
