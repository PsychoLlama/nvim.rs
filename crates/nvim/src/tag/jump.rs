//! Going to a tag.
//!
//! [`jumpto_tag`] takes one stored match, opens the file it names and runs
//! the search command (or line number) that follows, then puts the cursor
//! on the tag. [`parse_match`] and [`parse_tag_line`] are the readers for
//! the two line formats, and [`find_extra`] finds the optional
//! `;"<Tab>field:value` fields at the end of a line.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::search::SEARCH_KEEP;
use crate::window::WSP_VERT;
use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

/// The buffer a tag's command is copied into, in bytes.
///
/// A command longer than this is truncated, as upstream truncates it.
const LSIZE: usize = super::LSIZE as usize;

/// Split a tags-file line into name, file name and command.
///
/// The three are separated by TABs and the command runs to the end of the
/// line. Answers `false` for a line that is not of that shape, leaving
/// `tagp` part-filled.
///
/// # Safety
/// `lbuf` must be NUL-terminated, and must outlive `tagp`.
#[inline]
pub(crate) unsafe fn parse_tag_line(lbuf: *mut c_char, tagp: &mut TagParts) -> bool {
    // SAFETY: the caller's promise; `vim_strchr` stops at the terminator,
    // and every pointer stored stays inside the line.
    unsafe {
        // Through locals rather than back through `tagp`: this runs once
        // per line of every tags file read without a usable head.
        tagp.tagname = lbuf;
        let Some(name_end) = field_end(lbuf) else {
            return false;
        };
        tagp.tagname_end = name_end;

        let fname = skip_tab(name_end);
        tagp.fname = fname;
        let Some(fname_end) = field_end(fname) else {
            return false;
        };
        tagp.fname_end = fname_end;

        let command = skip_tab(fname_end);
        tagp.command = command;
        // A line that stops after the file name has no command at all.
        *command != 0
    }
}

/// Where the TAB after a field is, or `None` when there is no TAB left.
///
/// # Safety
/// `p` must be NUL-terminated.
#[inline]
unsafe fn field_end(p: *mut c_char) -> Option<*mut c_char> {
    // SAFETY: the caller's promise.
    let end = unsafe { vim_strchr(p, TAB) };
    (!end.is_null()).then_some(end)
}

/// Step over a field separator, unless the line ended there.
///
/// # Safety
/// `p` must be readable and NUL-terminated.
#[inline]
unsafe fn skip_tab(p: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's promise.
    unsafe { if *p != 0 { p.add(1) } else { p } }
}

/// Whether the line marks the tag as local to its file — a `file:` field.
///
/// # Safety
/// `tagp.command` must be NUL-terminated.
pub(crate) unsafe fn test_for_static(tagp: &TagParts) -> bool {
    // SAFETY: the caller's promise; every step stays before the
    // terminator.
    unsafe {
        let mut p = tagp.command;
        loop {
            p = vim_strchr(p, TAB);
            if p.is_null() {
                return false;
            }
            p = p.add(1);
            if strncmp(p, c"file:".as_ptr(), 5) == 0 {
                return true;
            }
        }
    }
}

/// How long a stored match is, not counting its terminator.
///
/// A match is `<bucket byte><tags file name>NUL<line>NUL`, so its length is
/// not one `strlen`.
///
/// # Safety
/// `lbuf` must point at a match of that shape.
pub(crate) unsafe fn matching_line_len(lbuf: *const c_char) -> size_t {
    // SAFETY: the caller's promise.
    unsafe {
        let line = lbuf.add(1).add(strlen(lbuf.add(1)) + 1);
        (line.offset_from(lbuf) as size_t) + strlen(line)
    }
}

/// Split a stored match into its parts.
///
/// On top of what [`parse_tag_line`] finds, this picks the optional
/// `kind:`, `user_data:` and `line:` fields out of the extra fields at the
/// end. Answers `false` for a line that is not a tag line.
///
/// # Safety
/// `lbuf` must point at a match as [`matching_line_len`] describes, and
/// must outlive `tagp`.
pub(crate) unsafe fn parse_match(lbuf: *mut c_char, tagp: &mut TagParts) -> bool {
    // SAFETY: the caller's promise. Every pointer written into `tagp`
    // points into the match, which the caller keeps alive.
    unsafe {
        tagp.tag_fname = lbuf.add(1);
        let line = lbuf.add(strlen(tagp.tag_fname) + 2);

        let parsed = parse_tag_line(line, tagp);
        tagp.tagkind = ptr::null_mut();
        tagp.user_data = ptr::null_mut();
        tagp.tagline = 0;
        tagp.command_end = ptr::null_mut();
        if !parsed {
            return false;
        }

        if let Some(extra) = find_extra(tagp.command) {
            // The command ends where the extra fields begin, less the '|'
            // an ex-command address is terminated with.
            tagp.command_end = if extra > tagp.command && *extra.sub(1) == b'|' as c_char {
                extra.sub(1)
            } else {
                extra
            };
            // Past the `;"`; the fields themselves follow a TAB.
            let after = extra.add(2);
            if *after == TAB as c_char {
                read_extra_fields(tagp, after.add(1));
            }
        }

        if !tagp.tagkind.is_null() {
            tagp.tagkind_end = field_text_end(tagp.tagkind);
        }
        if !tagp.user_data.is_null() {
            tagp.user_data_end = field_text_end(tagp.user_data);
        }
        true
    }
}

/// Read the `field:value` pairs after a tag's command.
///
/// Only `kind:`, `user_data:` and `line:` are wanted. A field with no
/// colon before the next TAB is the kind written in its short form.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn read_extra_fields(tagp: &mut TagParts, mut p: *mut c_char) {
    // SAFETY: the caller's promise; every step stops at the terminator.
    unsafe {
        // Field names are ASCII letters, and any multibyte character is
        // taken for one too: the loop ends at the first byte that is
        // neither.
        while (*p as c_uint).wrapping_sub('A' as c_uint) < 26
            || (*p as c_uint).wrapping_sub('a' as c_uint) < 26
            || utfc_ptr2len(p) > 1
        {
            if strncmp(p, c"kind:".as_ptr(), 5) == 0 {
                tagp.tagkind = p.add(5);
            } else if strncmp(p, c"user_data:".as_ptr(), 10) == 0 {
                tagp.user_data = p.add(10);
            } else if strncmp(p, c"line:".as_ptr(), 5) == 0 {
                tagp.tagline = atoi(p.add(5)) as linenr_T;
            }
            if !tagp.tagkind.is_null() && !tagp.user_data.is_null() {
                // Nothing else is read from here.
                return;
            }

            let colon = vim_strchr(p, ':' as c_int);
            let tab = vim_strchr(p, TAB);
            if colon.is_null() || (!tab.is_null() && colon > tab) {
                // No colon in this field: it is the kind on its own.
                tagp.tagkind = p;
            }
            if tab.is_null() {
                return;
            }
            p = tab.add(utfc_ptr2len(tab) as usize);
        }
    }
}

/// Where a field's value ends: at a TAB, a line ending or the terminator.
///
/// # Safety
/// `p` must be NUL-terminated.
unsafe fn field_text_end(mut p: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's promise.
    unsafe {
        while !matches!(*p as u8, 0 | b'\t' | b'\r' | b'\n') {
            p = p.add(utfc_ptr2len(p) as usize);
        }
        p
    }
}

/// The name of the file a match points at, as the editor can open it.
///
/// The answer is allocated; the caller frees it.
///
/// # Safety
/// `tagp` must describe a live match.
pub(crate) unsafe fn tag_full_fname(tagp: &TagParts) -> *mut c_char {
    // SAFETY: the caller's promise. The byte written over the file name's
    // end, so that it reads as a string, is put back before returning.
    unsafe {
        let saved = *tagp.fname_end;
        *tagp.fname_end = 0;
        let fullname = expand_tag_fname(tagp.fname, tagp.tag_fname, false);
        *tagp.fname_end = saved;
        fullname
    }
}

/// Whether a match names the file the search started in.
///
/// # Safety
/// `fname` and `tag_fname` must be NUL-terminated, `buf_ffname` NULL or
/// NUL-terminated, and `fname_end` must point into `fname`'s buffer.
pub(crate) unsafe fn test_for_current(
    fname: *mut c_char,
    fname_end: *mut c_char,
    tag_fname: *mut c_char,
    buf_ffname: *mut c_char,
) -> bool {
    if buf_ffname.is_null() {
        // Nothing to compare against.
        return false;
    }
    // SAFETY: the caller's promise. The byte written over the file name's
    // end is put back before returning.
    unsafe {
        let saved = *fname_end;
        *fname_end = 0;
        let fullname = expand_tag_fname(fname, tag_fname, true);
        let same =
            path_full_compare(fullname, buf_ffname, true, true) as c_uint & kEqualFiles as c_uint;
        xfree(fullname.cast());
        *fname_end = saved;
        same != 0
    }
}

/// Find the `;"` that ends a tag's command, skipping over the command.
///
/// A command is a line number, a `/pattern/` or a `?pattern?`, optionally
/// chained with `;`. Answers where the `;"` starts, or `None` when the
/// line has no extra fields.
///
/// # Safety
/// `start` must be NUL-terminated.
pub(crate) unsafe fn find_extra(start: *mut c_char) -> Option<*mut c_char> {
    // SAFETY: the caller's promise; every scan stops at the terminator.
    unsafe {
        let mut p = start;
        let mut first_char = *start;
        loop {
            if ascii_isdigit(*p as c_int) {
                p = skipdigits(p.add(1));
            } else if matches!(*p as u8, b'/' | b'?') {
                p = skip_regexp(p.add(1), *p as c_int, false_0);
                if *p != first_char {
                    // The pattern was never closed.
                    return None;
                }
                p = p.add(1);
            } else {
                // Not a command we know: look for the fields directly.
                // The '|' is what an ex-command address ends with.
                p = strstr(p, c"|;\"".as_ptr());
                if p.is_null() {
                    return None;
                }
                p = p.add(1);
                break;
            }
            // A ';' chains a second command onto the first.
            if *p != b';' as c_char
                || !(ascii_isdigit(*p.add(1) as c_int) || matches!(*p.add(1) as u8, b'/' | b'?'))
            {
                break;
            }
            p = p.add(1);
            first_char = *p;
        }
        (strncmp(p, c";\"".as_ptr(), 2) == 0).then_some(p)
    }
}

/// Jump to the tag one stored match describes.
///
/// Answers `OK`, `FAIL`, or `NOTAGFILE` when the file the match names does
/// not exist — the caller reports that one, reading [`nofile_fname`].
///
/// With `keep_help` the destination keeps the help-buffer flag of wherever
/// the jump came from; `forceit` is the `!` of `:tag!`.
///
/// # Safety
/// `lbuf_arg` must point at a stored match.
pub(crate) unsafe fn jumpto_tag(lbuf_arg: *const c_char, forceit: c_int, keep_help: bool) -> c_int {
    // SAFETY: the caller's promise. `lbuf` is our own copy of the match and
    // outlives every pointer `parse_match` takes into it.
    unsafe {
        if postponed_split.get() == 0 && !check_can_set_curbuf_forceit(forceit) {
            return FAIL;
        }

        // Our own copy: the jump writes terminators into it, and opening
        // the file may free the caller's.
        let len = matching_line_len(lbuf_arg) + 1;
        let mut lbuf = vec![0 as c_char; len];
        ptr::copy_nonoverlapping(lbuf_arg, lbuf.as_mut_ptr(), len);

        let mut tagp = TagParts::default();
        let retval = if parse_match(lbuf.as_mut_ptr(), &mut tagp) {
            // Truncate the file name, so that it reads as a string.
            *tagp.fname_end = 0;
            Jump {
                pattern: Pattern::of_command(&tagp),
                expanded: expand_tag_fname(tagp.fname, tagp.tag_fname, true),
                full_fname: ptr::null_mut(),
                preview: g_do_tagpreview.get() != 0,
                saved_win: ptr::null_mut(),
                reused_window: false,
                // Opening the file may reset it.
                key_typed: KeyTyped.get(),
            }
            .run(&tagp, forceit, keep_help)
        } else {
            FAIL
        };

        // For next time.
        g_do_tagpreview.set(0);
        retval
    }
}

/// One jump in progress, and the two names it has to give back.
struct Jump {
    /// The tag's command, as a search pattern or an ex command.
    pattern: Pattern,
    /// The expanded name of the file the tag is in. Owned.
    expanded: *mut c_char,
    /// Its absolute form, made when a preview window is reused, because
    /// entering that window may change directory. Owned; once it exists it
    /// is what gets opened.
    full_fname: *mut c_char,
    /// Whether this is a `:ptag`, landing in the preview window.
    preview: bool,
    /// The window to go back to afterwards, for a preview jump.
    saved_win: *mut win_T,
    /// Whether `'switchbuf'` already put us in a window holding the file,
    /// so that the usual loading must be skipped.
    reused_window: bool,
    /// Whether the key that started the jump was typed. Remembered before
    /// opening the file, which resets it.
    key_typed: bool,
}

impl Drop for Jump {
    fn drop(&mut self) {
        // SAFETY: both names are ours to free, or NULL.
        unsafe {
            xfree(self.expanded.cast());
            xfree(self.full_fname.cast());
        }
    }
}

impl Jump {
    /// The name to open: the absolute form once a preview window made one.
    fn fname(&self) -> *mut c_char {
        if self.full_fname.is_null() {
            self.expanded
        } else {
            self.full_fname
        }
    }

    /// Open the file and run the tag's command in it.
    ///
    /// # Safety
    /// `tagp` must describe the match, in a buffer that outlives the call.
    unsafe fn run(&mut self, tagp: &TagParts, forceit: c_int, keep_help: bool) -> c_int {
        // SAFETY: the caller's promise; the globals are live, and the file
        // name is NUL-terminated.
        unsafe {
            // Check the file exists before abandoning the current one. A
            // name a BufReadCmd autocommand claims (say "http://sys/file")
            // counts as existing.
            if !os_path_exists(self.fname())
                && !has_autocmd(EVENT_BUFREADCMD, self.fname(), ptr::null_mut())
            {
                xfree(nofile_fname.get().cast());
                nofile_fname.set(xstrdup(self.fname()));
                return NOTAGFILE;
            }

            *RedrawingDisabled.ptr() += 1;
            self.open_preview();
            if !self.open_window() {
                *RedrawingDisabled.ptr() -= 1;
                return FAIL;
            }

            if keep_help {
                // A `:ta` from a help file keeps the help flag set. For
                // `:ptag` it is the flag of the window we came from.
                keep_help_flag.set(if self.preview {
                    bt_help((*self.saved_win).w_buffer)
                } else {
                    (*curbuf.get()).b_help
                });
            }
            let opened = if self.reused_window {
                GETFILE_SAME_FILE as c_int
            } else {
                // Careful: this may trigger autocommands, which can call
                // `jumpto_tag` recursively.
                getfile(0, self.fname(), ptr::null_mut(), true, 0, forceit != 0)
            };
            keep_help_flag.set(false);

            // Anything above zero means the file could not be opened.
            if opened > 0 {
                *RedrawingDisabled.ptr() -= 1;
                if postponed_split.get() != 0 {
                    win_close(curwin.get(), false, false);
                    postponed_split.set(0);
                }
                return FAIL;
            }

            (*curwin.get()).w_set_curswant = true_0;
            postponed_split.set(0);
            let mut retval = self.run_command(tagp);
            // Jumping to another file counts as success: at least the file
            // was found.
            if opened == GETFILE_OPEN_OTHER as c_int {
                retval = OK;
            }
            if retval == OK {
                // In a help buffer put the cursor line at the top of the
                // window: the help subject is below it.
                if (*curbuf.get()).b_help {
                    set_topline(curwin.get(), (*curwin.get()).w_cursor.lnum);
                }
                if fdo_flags.get() & kOptFdoFlagTag as c_uint != 0 && self.key_typed {
                    foldOpenCursor();
                }
            }
            if self.preview && curwin.get() != self.saved_win && win_valid(self.saved_win) {
                // Put the cursor back where it was.
                validate_cursor(curwin.get());
                redraw_later(curwin.get(), UPD_VALID);
                win_enter(self.saved_win, true);
            }
            *RedrawingDisabled.ptr() -= 1;
            retval
        }
    }

    /// For `:ptag`, make the preview window the current one.
    ///
    /// # Safety
    /// The globals must be live and the file name NUL-terminated.
    unsafe fn open_preview(&mut self) {
        if !self.preview {
            return;
        }
        // SAFETY: the caller's promise.
        unsafe {
            // Don't split again below.
            postponed_split.set(0);
            self.saved_win = curwin.get();
            if (*curwin.get()).w_onebuf_opt.wo_pvw == 0 {
                // Entering a reused window may change directory
                // (autocommands), so make the name absolute first.
                self.full_fname = FullName_save(self.fname(), false);
                prepare_tagpreview(true);
            }
        }
    }

    /// Get to the window the tag should land in.
    ///
    /// A `CTRL-W CTRL-]` or a `:tab tag` opens a new window or tab page,
    /// unless `'switchbuf'` says to go to one that already holds the file.
    /// Answers `false` when a split was wanted and failed.
    ///
    /// # Safety
    /// The globals must be live and the file name NUL-terminated.
    unsafe fn open_window(&mut self) -> bool {
        // SAFETY: the caller's promise.
        unsafe {
            let switchbuf = swb_flags.get();
            if postponed_split.get() != 0
                && switchbuf & (kOptSwbFlagUseopen | kOptSwbFlagUsetab) as c_uint != 0
            {
                let existing = buflist_findname_exp(self.fname());
                if !existing.is_null() && !swbuf_goto_win_with_buf(existing).is_null() {
                    self.reused_window = true;
                }
            }
            if self.reused_window || (postponed_split.get() == 0 && (*cmdmod.ptr()).cmod_tab == 0) {
                return true;
            }

            // 'switchbuf' may ask for the new window to be a vertical
            // split, or for a whole new tab page.
            if switchbuf & kOptSwbFlagVsplit as c_uint != 0 {
                (*cmdmod.ptr()).cmod_split |= WSP_VERT as c_int;
            }
            if switchbuf & kOptSwbFlagNewtab as c_uint != 0 && (*cmdmod.ptr()).cmod_tab == 0 {
                (*cmdmod.ptr()).cmod_tab = tabpage_index(curtab.get()) + 1;
            }
            if win_split(postponed_split.get().max(0), postponed_split_flags.get()) == FAIL {
                return false;
            }
            // A fresh window does not inherit the scroll and cursor binding.
            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
            true
        }
    }

    /// Run the tag's command: a search, or an ex command in the sandbox.
    ///
    /// # Safety
    /// `tagp` must describe the match, and the cursor must be in the
    /// buffer the jump landed in.
    unsafe fn run_command(&mut self, tagp: &TagParts) -> c_int {
        // SAFETY: the caller's promise; the globals are live.
        unsafe {
            let save_magic = magic_overruled.get();
            // Tag commands always run with 'nomagic'.
            magic_overruled.set(OPTION_MAGIC_OFF);
            // Jumping to a tag is not a real search, so 'hlsearch' must
            // not light up because of it.
            let save_no_hlsearch = no_hlsearch.get();
            // With 't' in 'cpoptions' the tag's pattern becomes the one
            // "n" repeats; without it, the pattern is not stored.
            let search_options = if vim_strchr(p_cpo.get(), CPO_TAGPAT).is_null() {
                SEARCH_KEEP as c_int
            } else {
                0
            };

            let retval = if self.pattern.is_whole_search() {
                self.search(tagp, search_options)
            } else {
                self.pattern.execute();
                OK
            };

            magic_overruled.set(save_magic);
            if search_options != 0 {
                set_no_hlsearch(save_no_hlsearch);
            }
            retval
        }
    }

    /// Search for the tag's pattern, guessing at it if it is not there.
    ///
    /// # Safety
    /// `tagp` must describe the match.
    unsafe fn search(&mut self, tagp: &TagParts, search_options: c_int) -> c_int {
        // SAFETY: the caller's promise; the globals are live.
        unsafe {
            let save_p_ws = p_ws.get() != 0;
            let save_p_ic = p_ic.get();
            let save_p_scs = p_scs.get();
            // 'wrapscan' is needed for a backward search, and the pattern
            // was not typed by the user, so case must not be folded.
            p_ws.set(true_0);
            p_ic.set(false_0);
            p_scs.set(false_0);

            let save_lnum = (*curwin.get()).w_cursor.lnum;
            // Start before the line the "line:" field named, or before the
            // first line.
            (*curwin.get()).w_cursor.lnum = (tagp.tagline - 1).max(0);

            let found = if self.pattern.search(search_options) {
                Found::Exactly
            } else {
                // Try again, ignoring case this time.
                p_ic.set(true_0);
                if self.pattern.search(search_options) {
                    Found::IgnoringCase
                } else {
                    self.guess(tagp, search_options)
                }
            };

            let retval = match found {
                // The tag's own pattern matched: nothing to report.
                Found::Exactly => OK,
                Found::Nowhere => {
                    emsg(gettext(c"E434: Can't find tag pattern".as_ptr()));
                    (*curwin.get()).w_cursor.lnum = save_lnum;
                    FAIL
                }
                Found::IgnoringCase | Found::Guessing => {
                    // Only say so when it really was a guess, not when
                    // 'ignorecase' was already set and the match turned up
                    // once case was folded.
                    if matches!(found, Found::Guessing) || save_p_ic == 0 {
                        msg(
                            gettext(c"E435: Couldn't find tag, just guessing!".as_ptr()),
                            0,
                        );
                        if msg_scrolled.get() == 0 && msg_silent.get() == 0 {
                            msg_delay(1010, true);
                        }
                    }
                    OK
                }
            };

            p_ws.set(c_int::from(save_p_ws));
            p_ic.set(save_p_ic);
            p_scs.set(save_p_scs);
            // A search command may have put the cursor beyond the end of
            // the line; correct that here.
            check_cursor(curwin.get());
            retval
        }
    }

    /// The pattern was not in the file: search for what a declaration of
    /// the tag would look like instead.
    ///
    /// # Safety
    /// `tagp.tagname` must lie in a writable buffer, ending at
    /// `tagname_end`.
    unsafe fn guess(&mut self, tagp: &TagParts, search_options: c_int) -> Found {
        // SAFETY: the caller's promise. The byte written over the tag
        // name's end is put back before returning.
        //
        // (Upstream calls `test_for_static` here and drops the answer; it
        // has no side effects, so it is not repeated.)
        unsafe {
            let saved = *tagp.tagname_end;
            *tagp.tagname_end = 0;
            // "^func  ("
            let mut found = self
                .pattern
                .search_for(c"^%s\\s\\*(", tagp.tagname, search_options);
            if !found {
                // "^char * \<func  ("
                found = self.pattern.search_for(
                    c"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(",
                    tagp.tagname,
                    search_options,
                );
            }
            *tagp.tagname_end = saved;
            if found {
                Found::Guessing
            } else {
                Found::Nowhere
            }
        }
    }
}

/// How well the jump found the tag, which decides what the user is told.
enum Found {
    /// The tag's own pattern matched.
    Exactly,
    /// It matched once case was folded.
    IgnoringCase,
    /// Only a guess at what the declaration looks like matched.
    Guessing,
    /// Not even a guess matched.
    Nowhere,
}

/// A tag's command, in the fixed buffer the guesses are written into.
struct Pattern {
    /// `LSIZE` bytes, whatever the command's length: the guesses are
    /// formatted straight into it.
    buf: Vec<c_char>,
    /// How much of it the command uses, not counting the terminator.
    len: usize,
}

impl Pattern {
    /// Copy a tag's command out of its match, dropping any trailing CR/NL
    /// and the `;"<Tab>field:value` stuff, which is of no use here.
    ///
    /// # Safety
    /// `tagp.command` must be NUL-terminated.
    unsafe fn of_command(tagp: &TagParts) -> Self {
        let mut buf = vec![0 as c_char; LSIZE];
        let mut len = 0;
        // SAFETY: the caller's promise, and the copy stops one short of
        // the buffer's end so that the terminator always fits.
        unsafe {
            let mut str = tagp.command;
            while !matches!(*str as u8, 0 | b'\n' | b'\r') {
                buf[len] = *str;
                len += 1;
                str = str.add(1);
                if len + 1 >= LSIZE {
                    break;
                }
            }
            buf[len] = 0;

            if let Some(extra) = find_extra(buf.as_mut_ptr()) {
                len = extra.offset_from(buf.as_ptr()) as usize;
                buf[len] = 0;
            }
        }
        Pattern { buf, len }
    }

    /// Whether the command is a whole search command with nothing after
    /// it, which is the only form worth handing to [`do_search`].
    ///
    /// # Safety
    /// The buffer must hold a NUL-terminated command.
    unsafe fn is_whole_search(&self) -> bool {
        // SAFETY: the caller's promise; `skip_regexp` stops at the
        // terminator.
        unsafe {
            let start = self.buf.as_ptr().cast_mut();
            let after = if matches!(*start as u8, b'/' | b'?') {
                skip_regexp(start.add(1), *start as c_int, false_0).add(1)
            } else {
                start
            };
            after.offset_from(start) as usize >= self.len
        }
    }

    /// Run the command as a search, its first byte the delimiter.
    ///
    /// # Safety
    /// The editor must be in a buffer a search may run in.
    unsafe fn search(&mut self, options: c_int) -> bool {
        // SAFETY: the caller's promise. The length may wrap for an empty
        // command, exactly as the `size_t` subtraction upstream does.
        unsafe {
            let delim = self.buf[0] as c_int;
            do_search(
                ptr::null_mut(),
                delim,
                delim,
                self.buf.as_mut_ptr().add(1),
                self.len.wrapping_sub(1),
                1,
                options,
                ptr::null_mut(),
            ) != 0
        }
    }

    /// Format a guess into the buffer and search for that instead.
    ///
    /// # Safety
    /// `name` must be NUL-terminated.
    unsafe fn search_for(&mut self, fmt: &CStr, name: *const c_char, options: c_int) -> bool {
        // SAFETY: the caller's promise; `snprintf` never writes past
        // `LSIZE`. Its answer is the length it *would* have needed, which
        // is what upstream hands to `do_search` — kept as it is.
        unsafe {
            self.len =
                snprintf(self.buf.as_mut_ptr(), LSIZE as size_t, fmt.as_ptr(), name) as usize;
            do_search(
                ptr::null_mut(),
                '/' as c_int,
                '/' as c_int,
                self.buf.as_mut_ptr(),
                self.len,
                1,
                options,
                ptr::null_mut(),
            ) != 0
        }
    }

    /// Run the command as an ex command, in the sandbox: it came out of a
    /// tags file, which is not to be trusted.
    ///
    /// # Safety
    /// The editor must be in a buffer a command may run in.
    unsafe fn execute(&mut self) {
        // SAFETY: the caller's promise; the buffer is NUL-terminated.
        unsafe {
            let save_secure = secure.get();
            secure.set(1);
            *sandbox.ptr() += 1;

            // Start the command in line 1.
            (*curwin.get()).w_cursor = pos_T {
                lnum: 1,
                col: 0,
                coladd: 0,
            };
            do_cmdline_cmd(self.buf.as_mut_ptr());

            // When the command did something that is not allowed, make
            // sure the error message can be seen.
            if secure.get() == 2 {
                wait_return(true_0);
            }
            secure.set(save_secure);
            *sandbox.ptr() -= 1;
        }
    }
}
