//! Going to a tag.
//!
//! [`jumpto_tag`] takes one stored match, opens the file it names and runs
//! the search command (or line number) that follows, then puts the cursor
//! on the tag. [`parse_match`] and [`parse_tag_line`] are the readers for
//! the two line formats, and [`find_extra`] finds the optional
//! `;"<Tab>field:value` fields at the end of a line.

#![forbid(unsafe_code)]

use super::Jumped;
use super::*;
use crate::buffer::find_buffer_by_name;
use crate::cstr;
use crate::debugger::atoi;
use crate::ex_docmd::{cmdmod_add_split, cmdmod_set_tab, cmdmod_tab};
use crate::guard::{Lock, Suppress};
use crate::mbyte::c_cluster_len;
use crate::memory::XString;
use crate::option::cpo_has;
use crate::os::fs::path_exists;
use crate::regexp::skip_regexp_at;
use crate::search::{SEARCH_KEEP, do_search_in};
use crate::strings::{FmtArg, find_bytes, format_into};
use crate::types::{CpoFlag, FAIL, Failed, OK};
use crate::window::WSP_VERT;
use crate::window::tab_index;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win, WinId};
use core::ffi::{CStr, c_int, c_uint};

/// The buffer a tag's command is copied into, in bytes.
///
/// A command longer than this is truncated, as upstream truncates it.
const LSIZE: usize = super::LSIZE as usize;

/// The byte `at` of a C string's bytes, NUL past the end.
fn byte(s: &[u8], at: usize) -> u8 {
    cstr::byte_at(s, at)
}

/// Split a tags-file line into name, file name and command.
///
/// The three are separated by TABs and the command runs to the end of the
/// line. Answers `None` for a line that is not of that shape.
#[inline]
pub(crate) fn parse_tag_line(line: &[u8]) -> Option<TagParts> {
    let (parts, complete) = split_fields(line)?;
    complete.then_some(parts)
}

/// The name, file name and command of `line`, and whether there is a
/// command at all; `None` when the line lacks either TAB.
#[inline]
fn split_fields(line: &[u8]) -> Option<(TagParts, bool)> {
    let name_end = find_tab(line, 0)?;
    let fname = name_end + 1;
    let fname_end = find_tab(line, fname)?;
    let command = fname_end + 1;
    let parts = TagParts {
        tagname: Span {
            start: 0,
            end: name_end,
        },
        fname: Span {
            start: fname,
            end: fname_end,
        },
        command,
        ..TagParts::default()
    };
    // A line that stops after the file name has no command at all.
    Some((parts, command < line.len()))
}

/// Whether the line marks the tag as local to its file — a `file:` field.
pub(crate) fn test_for_static(line: &[u8], tagp: &TagParts) -> bool {
    let mut at = tagp.command;
    while let Some(tab) = find_tab(line, at) {
        at = tab + 1;
        if line[at..].starts_with(b"file:") {
            return true;
        }
    }
    false
}

/// Split a stored match's line into its parts.
///
/// On top of what [`parse_tag_line`] finds, this picks the optional
/// `kind:`, `user_data:` and `line:` fields out of the extra fields at the
/// end. `line` is the match's line *with* its terminator.
///
/// The flag is false for a line that is not a whole tag line. The parts
/// are still the ones the line has — a line with an empty command still
/// has its name and file name, which the listings show — and the default
/// for one without its two TABs.
pub(crate) fn parse_match(line: &[u8]) -> (TagParts, bool) {
    let text = &line[..line.len() - 1];
    let Some((mut tagp, complete)) = split_fields(text) else {
        return (TagParts::default(), false);
    };
    if !complete {
        return (tagp, false);
    }

    let mut kind_at = None;
    let mut user_data_at = None;
    if let Some(extra) = find_extra(line, tagp.command) {
        // The command ends where the extra fields begin, less the '|'
        // an ex-command address is terminated with.
        tagp.command_end = Some(if extra > tagp.command && line[extra - 1] == b'|' {
            extra - 1
        } else {
            extra
        });
        // Past the `;"`; the fields themselves follow a TAB.
        let after = extra + 2;
        if byte(line, after) == TAB as u8 {
            read_extra_fields(text, after + 1, &mut tagp, &mut kind_at, &mut user_data_at);
        }
    }

    tagp.tagkind = kind_at.map(|start| Span {
        start,
        end: field_text_end(text, start),
    });
    tagp.user_data = user_data_at.map(|start| Span {
        start,
        end: field_text_end(text, start),
    });
    (tagp, true)
}

/// Read the `field:value` pairs after a tag's command, from `at`.
///
/// Only `kind:`, `user_data:` and `line:` are wanted. A field with no
/// colon before the next TAB is the kind written in its short form.
fn read_extra_fields(
    line: &[u8],
    mut at: usize,
    tagp: &mut TagParts,
    kind: &mut Option<usize>,
    user_data: &mut Option<usize>,
) {
    // Field names are ASCII letters, and any multibyte character is
    // taken for one too: the loop ends at the first byte that is
    // neither.
    while byte(line, at).is_ascii_alphabetic() || c_cluster_len(&line[at..]) > 1 {
        let field = &line[at..];
        if field.starts_with(b"kind:") {
            *kind = Some(at + 5);
        } else if field.starts_with(b"user_data:") {
            *user_data = Some(at + 10);
        } else if field.starts_with(b"line:") {
            tagp.tagline = atoi(&field[5..]);
        }
        if kind.is_some() && user_data.is_some() {
            // Nothing else is read from here.
            return;
        }

        let colon = field.iter().position(|&b| b == b':');
        let tab = find_tab(field, 0);
        if colon.is_none() || tab.is_some_and(|tab| colon > Some(tab)) {
            // No colon in this field: it is the kind on its own.
            *kind = Some(at);
        }
        let Some(tab) = tab else {
            return;
        };
        at += tab + c_cluster_len(&field[tab..]);
    }
}

/// Where a field's value ends: at a TAB, a line ending or the end.
fn field_text_end(line: &[u8], mut at: usize) -> usize {
    while !matches!(byte(line, at), 0 | b'\t' | b'\r' | b'\n') {
        at += c_cluster_len(&line[at..]);
    }
    at
}

/// The name of the file a match points at, as the editor can open it.
pub(crate) fn tag_full_fname(found: &TagMatch, tagp: &TagParts) -> XString {
    expand_tag_fname(tagp.fname.of(found.line()), found.tag_fname(), false)
}

/// Whether a match's file name, `fname` in the tags file `tag_fname`, names
/// the file the search started in.
pub(crate) fn test_for_current(fname: &[u8], tag_fname: &[u8], buf_ffname: Option<&CStr>) -> bool {
    let Some(buf_ffname) = buf_ffname else {
        // Nothing to compare against.
        return false;
    };
    let fullname = expand_tag_fname(fname, tag_fname, true);
    path_full_compare(fullname.as_cstr(), buf_ffname, true, true) as c_uint
        & crate::path::kEqualFiles as c_uint
        != 0
}

/// Find the `;"` that ends a tag's command, skipping over the command.
///
/// A command is a line number, a `/pattern/` or a `?pattern?`, optionally
/// chained with `;`. `text` is NUL-terminated and the command starts at
/// `start`. Answers where the `;"` starts, or `None` when the line has no
/// extra fields.
pub(crate) fn find_extra(text: &[u8], start: usize) -> Option<usize> {
    let mut at = start;
    let mut first_char = byte(text, start);
    loop {
        let here = byte(text, at);
        if here.is_ascii_digit() {
            at += 1;
            while byte(text, at).is_ascii_digit() {
                at += 1;
            }
        } else if matches!(here, b'/' | b'?') {
            at += 1 + skip_regexp_at(&text[at + 1..], c_int::from(here), 0);
            if byte(text, at) != first_char {
                // The pattern was never closed.
                return None;
            }
            at += 1;
        } else {
            // Not a command we know: look for the fields directly.
            // The '|' is what an ex-command address ends with.
            let rest = cstr::until_nul(&text[at..]);
            at += find_bytes(rest, b"|;\"")? + 1;
            break;
        }
        // A ';' chains a second command onto the first.
        let next = byte(text, at + 1);
        if byte(text, at) != b';' || !(next.is_ascii_digit() || matches!(next, b'/' | b'?')) {
            break;
        }
        at += 1;
        first_char = byte(text, at);
    }
    text[at..].starts_with(b";\"").then_some(at)
}

/// Jump to the tag one stored match describes.
///
/// Answers [`Jumped::NoSuchFile`] when the file the match names does
/// not exist — the caller reports that one, reading [`MISSING_FILE`].
///
/// With `keep_help` the destination keeps the help-buffer flag of wherever
/// the jump came from; `forceit` is the `!` of `:tag!`. The match is a copy
/// of its own: opening the file may replace the list it came from.
pub(crate) fn jumpto_tag(
    found: TagMatch,
    forceit: c_int,
    keep_help: bool,
) -> Result<Jumped, Failed> {
    if postponed_split.get() == 0 && !check_can_set_curbuf_forceit(forceit) {
        return Err(Failed);
    }

    let retval = match found.parse() {
        (tagp, true) => {
            let line = found.line_nul();
            Jump {
                pattern: Pattern::of_command(line, &tagp),
                expanded: expand_tag_fname(tagp.fname.of(line), found.tag_fname(), true),
                full_fname: None,
                preview: g_do_tagpreview.get() != 0,
                saved_win: None,
                reused_window: false,
                // Opening the file may reset it.
                key_typed: KeyTyped.get(),
            }
            .run(line, &tagp, forceit, keep_help)
        }
        (_, false) => Err(Failed),
    };

    // For next time.
    g_do_tagpreview.set(0);
    retval
}

/// One jump in progress.
struct Jump {
    /// The tag's command, as a search pattern or an ex command.
    pattern: Pattern,
    /// The expanded name of the file the tag is in.
    expanded: XString,
    /// Its absolute form, made when a preview window is reused, because
    /// entering that window may change directory. Once it exists it is
    /// what gets opened.
    full_fname: Option<XString>,
    /// Whether this is a `:ptag`, landing in the preview window.
    preview: bool,
    /// The window to go back to afterwards, for a preview jump. By
    /// identity, not by address: `getfile` below runs autocommands that may
    /// close it, and a freed `Window`'s address can be handed back out.
    saved_win: Option<WinId>,
    /// Whether `'switchbuf'` already put us in a window holding the file,
    /// so that the usual loading must be skipped.
    reused_window: bool,
    /// Whether the key that started the jump was typed. Remembered before
    /// opening the file, which resets it.
    key_typed: bool,
}

impl Jump {
    /// The name to open: the absolute form once a preview window made one.
    fn fname(&self) -> &CStr {
        self.full_fname.as_ref().unwrap_or(&self.expanded).as_cstr()
    }

    /// Open the file and run the tag's command in it.
    fn run(
        &mut self,
        line: &[u8],
        tagp: &TagParts,
        forceit: c_int,
        keep_help: bool,
    ) -> Result<Jumped, Failed> {
        // Check the file exists before abandoning the current one. A
        // name a BufReadCmd autocommand claims (say "http://sys/file")
        // counts as existing.
        if !path_exists(self.fname()) && !has_autocmd(AutoEvent::BufReadCmd, self.fname(), None) {
            MISSING_FILE.set(Some(XString::from_cstr(self.fname())));
            return Ok(Jumped::NoSuchFile);
        }

        let redraw_off = Suppress::redraw();
        self.open_preview();
        if !self.open_window() {
            drop(redraw_off);
            return Err(Failed);
        }

        if keep_help {
            // A `:ta` from a help file keeps the help flag set. For
            // `:ptag` it is the flag of the window we came from.
            keep_help_flag.set(if self.preview {
                // `open_preview` took the id a moment ago and nothing since
                // can have closed the window, but ask rather than assume.
                buf_is_help(
                    self.saved_win
                        .and_then(WinId::get)
                        .and_then(|w| w.buffer_or_none()),
                )
            } else {
                Buf::current().b_help
            });
        }
        let opened = if self.reused_window {
            GETFILE_SAME_FILE as c_int
        } else {
            // Careful: this may trigger autocommands, which can call
            // `jumpto_tag` recursively. The name is this jump's own.
            getfile(0, Some(self.fname()), None, true, 0, forceit != 0)
        };
        keep_help_flag.set(false);

        // Anything above zero means the file could not be opened.
        if opened > 0 {
            drop(redraw_off);
            if postponed_split.get() != 0 {
                win_close(Win::current(), false, false);
                postponed_split.set(0);
            }
            return Err(Failed);
        }

        Win::current().w_set_curswant = true;
        postponed_split.set(0);
        let mut retval = if self.run_command(line, tagp) == OK {
            Ok(Jumped::Done)
        } else {
            Err(Failed)
        };
        // Jumping to another file counts as success: at least the file
        // was found.
        if opened == GETFILE_OPEN_OTHER as c_int {
            retval = Ok(Jumped::Done);
        }
        if retval.is_ok() {
            // In a help buffer put the cursor line at the top of the
            // window: the help subject is below it.
            if Buf::current().b_help {
                set_topline(Win::current(), Win::current().w_cursor.lnum);
            }
            if fdo_flags.get() & kOptFdoFlagTag as c_uint != 0 && self.key_typed {
                fold_open_cursor();
            }
        }
        // `getfile` above runs autocommands, so the window the jump started
        // from may be gone; ask the registry for it rather than the address.
        // `win_valid` is still the second question — the object may exist and
        // yet be off this tab page's list.
        let saved = self.saved_win.and_then(WinId::get);
        if self.preview
            && let Some(saved) = saved
            && !saved.is_current()
            && win_valid(saved.id())
        {
            // Put the cursor back where it was.
            validate_cursor(Win::current());
            redraw_later(Win::current(), UPD_VALID);
            win_enter(saved, true);
        }
        drop(redraw_off);
        retval
    }

    /// For `:ptag`, make the preview window the current one.
    fn open_preview(&mut self) {
        if !self.preview {
            return;
        }
        // Don't split again below.
        postponed_split.set(0);
        self.saved_win = Some(Win::current().id());
        if Win::current().w_onebuf_opt.wo_pvw == 0 {
            // Entering a reused window may change directory
            // (autocommands), so make the name absolute first.
            self.full_fname = Some(full_name_save(self.fname(), false));
            prepare_tagpreview(true);
        }
    }

    /// Get to the window the tag should land in.
    ///
    /// A `CTRL-W CTRL-]` or a `:tab tag` opens a new window or tab page,
    /// unless `'switchbuf'` says to go to one that already holds the file.
    /// Answers `false` when a split was wanted and failed.
    fn open_window(&mut self) -> bool {
        let switchbuf = swb_flags.get();
        if postponed_split.get() != 0
            && switchbuf & (kOptSwbFlagUseopen | kOptSwbFlagUsetab) as c_uint != 0
            && let Some(existing) = find_buffer_by_name(self.fname())
            && !swbuf_goto_win_with_buf(Some(existing)).is_none()
        {
            self.reused_window = true;
        }
        if self.reused_window || (postponed_split.get() == 0 && cmdmod_tab() == 0) {
            return true;
        }

        // 'switchbuf' may ask for the new window to be a vertical
        // split, or for a whole new tab page.
        if switchbuf & kOptSwbFlagVsplit as c_uint != 0 {
            cmdmod_add_split(WSP_VERT as c_int);
        }
        if switchbuf & kOptSwbFlagNewtab as c_uint != 0 && cmdmod_tab() == 0 {
            cmdmod_set_tab(tab_index(TabPage::current()) + 1);
        }
        if win_split(postponed_split.get().max(0), postponed_split_flags.get()).is_err() {
            return false;
        }
        // A fresh window does not inherit the scroll and cursor binding.
        Win::current().w_onebuf_opt.wo_scb = 0;
        Win::current().w_onebuf_opt.wo_crb = 0;
        true
    }

    /// Run the tag's command: a search, or an ex command in the sandbox.
    fn run_command(&mut self, line: &[u8], tagp: &TagParts) -> c_int {
        // Tag commands always run with 'nomagic'.
        let save_magic = magic_overruled.get();
        magic_overruled.set(OPTION_MAGIC_OFF);
        // Jumping to a tag is not a real search, so 'hlsearch' must
        // not light up because of it.
        let save_no_hlsearch = no_hlsearch.get();
        // With 't' in 'cpoptions' the tag's pattern becomes the one
        // "n" repeats; without it, the pattern is not stored.
        let search_options = if !cpo_has(CpoFlag::TAGPAT) {
            SEARCH_KEEP as c_int
        } else {
            0
        };

        let retval = if self.pattern.is_whole_search() {
            self.search(line, tagp, search_options)
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

    /// Search for the tag's pattern, guessing at it if it is not there.
    fn search(&mut self, line: &[u8], tagp: &TagParts, search_options: c_int) -> c_int {
        let save_p_ws = p_ws();
        let save_p_ic = p_ic();
        let save_p_scs = p_scs();
        // 'wrapscan' is needed for a backward search, and the pattern
        // was not typed by the user, so case must not be folded.
        P_WS.set(true);
        P_IC.set(false);
        P_SCS.set(false);

        let save_lnum = Win::current().w_cursor.lnum;
        // Start before the line the "line:" field named, or before the
        // first line.
        Win::current().w_cursor.lnum = (tagp.tagline - 1).max(0);

        let found = if self.pattern.search(search_options) {
            Found::Exactly
        } else {
            // Try again, ignoring case this time.
            P_IC.set(true);
            if self.pattern.search(search_options) {
                Found::IgnoringCase
            } else {
                self.guess(tagp.tagname.of(line), search_options)
            }
        };

        let retval = match found {
            // The tag's own pattern matched: nothing to report.
            Found::Exactly => OK,
            Found::Nowhere => {
                tag_emsg(c"E434: Can't find tag pattern");
                Win::current().w_cursor.lnum = save_lnum;
                FAIL
            }
            Found::IgnoringCase | Found::Guessing => {
                // Only say so when it really was a guess, not when
                // 'ignorecase' was already set and the match turned up
                // once case was folded.
                if matches!(found, Found::Guessing) || !save_p_ic {
                    let s2 = gettext(c"E435: Couldn't find tag, just guessing!");
                    msg(s2, 0);
                    if msg_scrolled.get() == 0 && msg_silent.get() == 0 {
                        msg_delay(1010, true);
                    }
                }
                OK
            }
        };

        P_WS.set(save_p_ws);
        P_IC.set(save_p_ic);
        P_SCS.set(save_p_scs);
        // A search command may have put the cursor beyond the end of
        // the line; correct that here.
        check_cursor(Win::current());
        retval
    }

    /// The pattern was not in the file: search for what a declaration of
    /// the tag `name` would look like instead.
    ///
    /// (Upstream calls `test_for_static` here and drops the answer; it
    /// has no side effects, so it is not repeated.)
    fn guess(&mut self, name: &[u8], search_options: c_int) -> Found {
        // "^func  ("
        let found = self.pattern.search_for(b"^%s\\s\\*(", name, search_options)
            // "^char * \<func  ("
            || self.pattern.search_for(b"^\\[#a-zA-Z_]\\.\\*\\<%s\\s\\*(", name, search_options);
        if found {
            Found::Guessing
        } else {
            Found::Nowhere
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
    buf: Vec<u8>,
    /// How much of it the command uses, not counting the terminator.
    len: usize,
}

impl Pattern {
    /// Copy a tag's command out of its match, dropping any trailing CR/NL
    /// and the `;"<Tab>field:value` stuff, which is of no use here.
    fn of_command(line: &[u8], tagp: &TagParts) -> Self {
        let mut buf = vec![0; LSIZE];
        let command = &line[tagp.command..];
        // The copy stops one short of the buffer's end so that the
        // terminator always fits.
        let mut len = command
            .iter()
            .position(|b| matches!(b, 0 | b'\n' | b'\r'))
            .unwrap_or(command.len())
            .min(LSIZE - 1);
        buf[..len].copy_from_slice(&command[..len]);

        if let Some(extra) = find_extra(&buf, 0) {
            len = extra;
            buf[len] = 0;
        }
        Pattern { buf, len }
    }

    /// Whether the command is a whole search command with nothing after
    /// it, which is the only form worth handing to `do_search`.
    fn is_whole_search(&self) -> bool {
        let delim = self.buf[0];
        let past = if matches!(delim, b'/' | b'?') {
            skip_regexp_at(&self.buf[1..], c_int::from(delim), 0) + 2
        } else {
            0
        };
        past >= self.len
    }

    /// Run the command as a search, its first byte the delimiter.
    fn search(&mut self, options: c_int) -> bool {
        let delim = c_int::from(self.buf[0]);
        do_search_in(&mut self.buf[1..], delim, delim, options) != 0
    }

    /// Format a guess into the buffer and search for that instead.
    ///
    /// The guess is cut where the buffer ends, as upstream's `snprintf`
    /// cut it.
    fn search_for(&mut self, fmt: &[u8], name: &[u8], options: c_int) -> bool {
        self.len = format_into(&mut self.buf, fmt, &[FmtArg::Str(name)]);
        let slash = c_int::from(b'/');
        do_search_in(&mut self.buf, slash, slash, options) != 0
    }

    /// Run the command as an ex command, in the sandbox: it came out of a
    /// tags file, which is not to be trusted.
    fn execute(&mut self) {
        // The command came out of a tags file, so it runs with `secure` set
        // and inside the sandbox -- and `secure` is *restored*, not cleared:
        // the editor may already have been in secure mode when the tag was
        // followed, and clearing it would hand the tags file a way out.
        let save_secure = secure.get();
        secure.set(1);
        let _sandboxed = Lock::sandbox();

        // Start the command in line 1.
        Win::current().w_cursor = Pos {
            lnum: 1,
            col: 0,
            coladd: 0,
        };
        let _ = do_cmdline_cmd(cstr::in_bytes(&self.buf));

        // When the command did something that is not allowed, make
        // sure the error message can be seen.
        if secure.get() == 2 {
            wait_return(1);
        }
        secure.set(save_secure);
    }
}
