//! The user-facing search commands.
//!
//! [`do_search`] is `/` and `?` in full: it parses the offset off the end
//! of the pattern, echoes the command, and drives [`searchit`](super::searchit)
//! once per count — possibly several times over, because a search command
//! may be a `;`-separated chain (`/foo/;?bar`). [`showmatch`] is the
//! `'showmatch'` blink, which sits here because it is the other thing a
//! typed character can set going. The `f`/`t` character search is in
//! [`charsearch`](super::charsearch) and `gn`/`gN` in
//! [`select`](super::select).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::autocmd::fire_autocmds;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memory::XString;
use crate::message::msg_display_bytes;
use crate::narrow::number_as_lnum;
use crate::narrow::{len_as_int, number_as_int};
use crate::option::defs::{CpoFlag, ShmFlag};
use crate::option::{ScrollMargin, ScrollOff, cpo_has};
use crate::optionstr::{OptString, OptStringRef};
use crate::plines::getvcol;
use crate::pos::MAXCOL;
use crate::regexp::RE_LAST;
use crate::regexp::low_byte;
use crate::search::{
    SEARCH_ECHO, SEARCH_END, SEARCH_HIS, SEARCH_KEEP, SEARCH_MARK, SEARCH_MSG, SEARCH_NOOF,
    SEARCH_OPT, SEARCH_PEEK, SEARCH_REV, SEARCH_START, SEARCH_STAT_BUF_LEN,
    SEARCH_STAT_DEF_TIMEOUT, SearchError,
};
use crate::types::CmdModFlags;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int};

// ---------------------------------------------------------------------
// `/` and `?`.
// ---------------------------------------------------------------------

/// The echoed search command, terminated.
///
/// The buffer is deliberately over-long and space-filled: the search
/// statistics are written into its tail, right-aligned, by
/// [`cmdline_search_stat`](super::cmdline_search_stat).
type Echo = Vec<u8>;

/// What a search command found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hit {
    /// A match.
    Match,
    /// A match, with a line offset added: the motion is linewise.
    Linewise,
}

/// Why a search command found nothing. Each is said where it is made, as
/// the command's `options` ask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandError {
    /// The search itself failed; `searchit` has said so.
    Search(SearchError),
    /// There is no previous pattern to reuse (E35).
    NoPattern,
    /// A `;` was followed by something other than `/` or `?` (E386).
    BadLeg,
}

impl From<SearchError> for CommandError {
    fn from(err: SearchError) -> Self {
        CommandError::Search(err)
    }
}

/// Where the pattern to search for comes from.
#[derive(Debug)]
enum Source {
    /// The text the command holds.
    Typed,
    /// The remembered search pattern, which an empty pattern stands for
    /// and [`search_regcomp`](super::search_regcomp) fills in itself.
    Remembered,
    /// The `:substitute` pattern, named explicitly because there is no
    /// search pattern yet.
    Substitute(XString),
}

/// Choose the pattern to search for, given the first byte of what is left
/// of the command.
///
/// An empty pattern means "the last one": normally that is left to
/// [`search_regcomp`](super::search_regcomp), which is told to use
/// `RE_LAST`. Only when there is no search pattern at all does the
/// `:substitute` pattern get named explicitly.
///
/// Answers `None` when neither remembered pattern exists, which the caller
/// says (E35).
fn pattern_to_search(first: u8, delim: c_int) -> Option<Source> {
    if !(first == b'\0' || as_char(first) == delim) {
        return Some(Source::Typed);
    }
    if has_last_search_pattern() {
        return Some(Source::Remembered);
    }
    substitute_pattern().map(Source::Substitute)
}

/// `byte` as upstream compares it with a delimiter: a `char`, which is
/// signed here, so a byte at or above 0x80 never equals an unsigned one.
fn as_char(byte: u8) -> c_int {
    c_int::from(c_char::from_ne_bytes([byte]))
}

/// Parse the `e`/`s`/`b` and `+n`/`-n` suffix that starts `text[at..]`,
/// leaving it in [`search_offset`](super::search_offset).
///
/// Answers the offset of the first byte after it. For `get_address` (echo
/// off) the character offsets are not looked for, because they are
/// meaningless there and the `s` could be a `:substitute`.
fn parse_offset(text: &[u8], mut at: usize, options: c_int) -> usize {
    let mut off = search_offset();
    off.line = false;
    off.end = false;
    off.off = 0;

    let byte = |i: usize| cstr::byte_at(text, i);
    if matches!(byte(at), b'+' | b'-') || byte(at).is_ascii_digit() {
        off.line = true;
    } else if options & SEARCH_OPT != 0 && matches!(byte(at), b'e' | b's' | b'b') {
        off.end = byte(at) == b'e';
        at += 1;
    }

    if byte(at).is_ascii_digit() || matches!(byte(at), b'+' | b'-') {
        // 'nr' or '+nr' or '-nr'
        off.off = if byte(at).is_ascii_digit() || byte(at + 1).is_ascii_digit() {
            atol_bytes(&text[at..])
        } else if byte(at) == b'-' {
            -1 // a single '-'
        } else {
            1 // a single '+'
        };
        at += 1;
        while byte(at).is_ascii_digit() {
            at += 1;
        }
    }

    set_search_offset(off);
    at
}

/// C's `atol` over the start of `text`: leading white space, a sign and
/// digits, saturating where `strtol` would.
fn atol_bytes(text: &[u8]) -> int64_t {
    let mut i = text
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r'))
        .unwrap_or(text.len());
    let negative = cstr::byte_at(text, i) == b'-';
    if matches!(cstr::byte_at(text, i), b'+' | b'-') {
        i += 1;
    }
    let mut value: int64_t = 0;
    while let Some(digit) = text.get(i).filter(|b| b.is_ascii_digit()) {
        let digit = int64_t::from(digit - b'0');
        value = if negative {
            value.saturating_mul(10).saturating_sub(digit)
        } else {
            value.saturating_mul(10).saturating_add(digit)
        };
        i += 1;
    }
    value
}

/// Spell the offset back out, for the echoed command line.
fn write_offset(dirc: c_int, off: SearchOffset) -> Vec<u8> {
    let mut out = Vec::new();
    if !(off.line || off.end || off.off != 0) {
        return out;
    }
    out.push(low_byte(dirc));
    if off.end {
        out.push(b'e');
    } else if !off.line {
        out.push(b's');
    }
    if off.off != 0 || off.line {
        out.extend_from_slice(format!("{:+}", off.off).as_bytes());
    }
    out
}

/// How much room to reserve for the echoed command.
///
/// With the search statistics switched on the buffer is deliberately as
/// wide as the space available, so that the statistics land right-aligned
/// in its tail; `msg_strtrunc` shortens it in the middle if it still does
/// not fit.
fn echo_size(plen: size_t, off_len: size_t, with_stats: bool) -> size_t {
    if !with_stats {
        return plen + off_len + 3;
    }
    let available = {
        if ui_has(UI_MESSAGES) {
            0 // adjusted below
        } else if MSG_SCROLLED.get() != 0 && !CMD_SILENT.get() {
            // Use all the columns.
            usize::try_from((ROWS.get() - MSG_ROW.get()) * COLUMNS.get() - 1).unwrap_or(0)
        } else {
            // Use up to the 'showcmd' column.
            let cells = (ROWS.get() - MSG_ROW.get() - 1) * COLUMNS.get() + SC_COL.get() - 1;
            usize::try_from(cells).unwrap_or(0)
        }
    };
    available.max(plen + off_len + SEARCH_STAT_BUF_LEN as usize + 3)
}

/// Reverse the echoed command (its terminator not included) in place, for
/// `'rightleft'` with `'rlc'` naming the search command.
///
/// The pattern could be shown on the right in rightleft mode, but the
/// `'ruler'` and `'showcmd'` areas use that space too and would blank it
/// out again very soon; it is shown on the left with the text reversed
/// instead.
fn reverse_echo(echo: &[u8]) -> Vec<u8> {
    let mut text = reversed_text(echo);
    // Move the reversed text to the beginning of the buffer, and blank
    // what is left after it.
    let lead = text.iter().take_while(|&&b| b == b' ').count();
    let pat_len = text.len() - lead;
    text.copy_within(lead.., 0);
    text[pat_len..].fill(b' ');
    text
}

/// Echo the search command on the command line and answer the buffer it
/// was drawn from, which the search statistics are later written into.
fn echo_search_cmd(dirc: c_int, searchstr: &[u8], options: c_int) -> Option<(Echo, bool)> {
    if !(options & SEARCH_ECHO != 0
        && messaging()
        && MSG_SILENT.get() == 0
        && (!CMD_SILENT.get() || !shortmess(ShmFlag::SEARCHCOUNT)))
    {
        return None;
    }

    // Compute msg_row early.
    msg_start();
    msg_ext_set_kind(c"search_cmd");

    let off_text = if CMD_SILENT.get() {
        Vec::new()
    } else {
        write_offset(dirc, search_offset())
    };

    let remembered;
    let p: &[u8] = if searchstr.is_empty() {
        remembered = last_search_pattern().unwrap_or_default();
        &remembered
    } else {
        searchstr
    };
    let plen = p.len();

    let with_stats = !shortmess(ShmFlag::SEARCHCOUNT) || CMD_SILENT.get();
    let size = echo_size(plen, off_text.len(), with_stats);
    let mut echo = vec![b' '; size];
    echo[size - 1] = b'\0';

    // Do not fill the buffer when cmd_silent is set: it is left empty
    // for the search-statistics feature.
    if !CMD_SILENT.get() {
        ui_busy_start();
        echo[0] = low_byte(dirc);
        let at = if utf_iscomposing_first(char_at(p)) {
            // Use a space to draw the composing character on.
            echo[1] = b' ';
            2
        } else {
            1
        };
        echo[at..at + plen].copy_from_slice(p);
        echo[plen + 1..plen + 1 + off_text.len()].copy_from_slice(&off_text);

        let text = cstr::until_nul(&echo);
        if let Some(truncated) = fit_middle(text, true) {
            echo = truncated.as_bytes_with_nul().to_vec();
        }

        if Win::current().w_onebuf_opt.wo_rl != 0
            && Win::current().w_onebuf_opt.wo_rlc.first_byte() == b's'
        {
            let mut reversed = reverse_echo(cstr::until_nul(&echo));
            reversed.push(b'\0');
            echo = reversed;
        }

        msg_display_bytes(cstr::until_nul(&echo), 0, false);
        msg_clr_eos();
        msg_check();
        gotocmdline(false);
        ui_flush();
        ui_busy_stop();
        MSG_NOWAIT.set(true);
    }

    Some((echo, !shortmess(ShmFlag::SEARCHCOUNT)))
}

/// Step the start position back over a character offset before searching,
/// so that `?pat?e+2` and `/pat/s-2` do not get stuck on the same match.
///
/// Not done for a line offset, because then this would not be vi
/// compatible; skipped when `pos.col` is near `MAXCOL` (a closed fold).
fn back_off_start(pos: &mut Pos, off: i64) {
    let mut c = off;
    if off > 0 {
        while c != 0 {
            if decl(pos).is_stuck() {
                break;
            }
            c -= 1;
        }
        if c != 0 {
            // At the start of the buffer; lnum == 0 is allowed here.
            pos.lnum = LineNr::ZERO;
            pos.col = ColNr::MAX;
        }
    } else {
        while c != 0 {
            if incl(pos).is_stuck() {
                break;
            }
            c += 1;
        }
        if c != 0 {
            // At the end of the buffer.
            pos.lnum = Buf::current().b_ml.ml_line_count + 1;
            pos.col = ColNr::ZERO;
        }
    }
}

/// Apply the offset to the position a search found.
///
/// Answers [`Hit::Linewise`] when a *line* offset was added.
fn add_offset(pos: &mut Pos, off: SearchOffset) -> Hit {
    if off.line {
        // Add the offset to the line number.
        let lnum = i64::from(pos.lnum) + off.off;
        let last = Buf::current().b_ml.ml_line_count;
        pos.lnum = if lnum < 1 {
            LineNr::ONE
        } else if lnum > i64::from(last) {
            last
        } else {
            number_as_lnum(lnum)
        };
        pos.col = ColNr::ZERO;
        return Hit::Linewise;
    }
    if pos.col < ColNr::new(MAXCOL - 2) {
        // Just in case.
        let mut c = off.off;
        if c > 0 {
            // To the right, checking for the end of the file.
            while c > 0 {
                c -= 1;
                if incl(pos).is_stuck() {
                    break;
                }
            }
        } else {
            // To the left, checking for the start of the file.
            while c < 0 {
                c += 1;
                if decl(pos).is_stuck() {
                    break;
                }
            }
        }
    }
    Hit::Match
}

/// Search forward for `pattern` from the cursor, as `/pattern/` would but
/// keeping the last search pattern and its direction (`SEARCH_KEEP`).
/// Answers whether it was found; the cursor is on the match when it was.
pub(crate) fn search_forward_keep(pattern: &[u8]) -> bool {
    let slash = c_int::from(b'/');
    do_search(None, slash, slash, pattern, 1, SEARCH_KEEP, None).is_some()
}

/// [`do_search`] once for the pattern that starts `text` and ends at its
/// first NUL (or the end), with no operator and no extra arguments.
pub(crate) fn do_search_in(
    text: &[u8],
    dirc: c_int,
    search_delim: c_int,
    options: c_int,
) -> Option<Hit> {
    let pat = cstr::until_nul(text);
    do_search(None, dirc, search_delim, pat, 1, options, None)
}

/// Search for `pat`, `count` times, from the cursor.
///
/// `dirc` is `'/'` or `'?'`, or 0 to reuse the direction of the previous
/// search. An empty `pat` reuses the previous pattern. `options` is a mask
/// of the `SEARCH_*` flags.
///
/// Careful: with a line offset of 0 (`SPATS[0].off.line` set and
/// `off.off == 0`) this makes the motion linewise without moving the
/// match position.
///
/// Answers what was found, or `None` -- having said why, as `options`
/// asks.
pub(crate) fn do_search(
    op: Option<&mut OpArg>,
    dirc: c_int,
    search_delim: c_int,
    pat: &[u8],
    count: c_int,
    options: c_int,
    sia: Option<&mut SearchItArg>,
) -> Option<Hit> {
    SEARCHCMDLEN.set(0);
    // A line offset is not remembered; this is vi compatible.
    let mut off = search_offset();
    if off.line && cpo_has(CpoFlag::LINEOFF) {
        off.line = false;
        off.off = 0;
        set_search_offset(off);
    }
    // Saved for when SEARCH_KEEP is used.
    let old_off = search_offset();

    let mut command = Command::new(pat, dirc, off.dir, search_delim, options);
    let mut pos = start_position(command.dirc);

    // Turn 'hlsearch' highlighting back on.
    if NO_HLSEARCH.get() && options & SEARCH_KEEP == 0 {
        redraw_all_later(UPD_SOME_VALID);
        set_no_hlsearch(false);
    }

    let found = search_legs(&mut command, &mut pos, op, count, options, sia);
    if found.is_ok() {
        if options & SEARCH_MARK != 0 {
            setpcmark();
        }
        Win::current().w_cursor = pos;
        Win::current().w_set_curswant = true;
    }

    if options & SEARCH_KEEP != 0 || cmdmod_has(CmdModFlags::KEEPPATTERNS) {
        set_search_offset(old_off);
    }
    found.ok()
}

/// Where a search in direction `dirc` starts: the cursor, or past the end
/// (or before the start) of the closed fold it is in, so that the same fold
/// is not found again.
fn start_position(dirc: c_int) -> Pos {
    let mut pos = Win::current().w_cursor;
    if dirc == '/' as c_int {
        if Win::current().has_folding(pos.lnum, None, Some(&mut pos.lnum)) {
            pos.col = ColNr::new(MAXCOL - 2); // avoid overflow when adding 1
        }
    } else if Win::current().has_folding(pos.lnum, Some(&mut pos.lnum), None) {
        pos.col = ColNr::ZERO;
    }
    pos
}

/// A search command being walked leg by leg.
#[derive(Debug)]
struct Command {
    /// The command, terminated: what is left of it is `text[at..]` -- the
    /// pattern, then the offset, then either nothing or `;` and the next
    /// leg.
    text: Vec<u8>,
    at: usize,
    /// How long the caller (and, after a leg, the walk) says what is left
    /// is, kept with upstream's wrapping arithmetic.
    patlen: usize,
    /// Whether `text` is the copy `skip_regexp_copy` made when it rewrote
    /// "\?" to "?"; a later leg rewrites that copy in place.
    copied: bool,
    /// This leg's direction.
    dirc: c_int,
    /// The character that ends this leg's pattern -- usually the direction,
    /// but `:s%pat%rep` passes its own.
    delim: c_int,
}

impl Command {
    /// The command over `pat`, searching in the direction `dirc` (`/` or
    /// `?`), which is remembered as the search direction and
    /// `v:searchforward`; or `previous`, the remembered one, when `dirc` is
    /// 0. `SEARCH_REV` turns it round.
    fn new(pat: &[u8], dirc: c_int, previous: c_char, delim: c_int, options: c_int) -> Self {
        let mut dirc = dirc;
        if dirc == 0 {
            dirc = c_int::from(previous.to_ne_bytes()[0]);
        } else {
            let mut off = search_offset();
            off.dir = low_byte(dirc).cast_signed();
            set_search_offset(off);
            set_vv_searchforward();
        }
        if options & SEARCH_REV != 0 {
            dirc = if dirc == '/' as c_int {
                '?' as c_int
            } else {
                '/' as c_int
            };
        }
        let mut text = pat.to_vec();
        text.push(b'\0');
        Command {
            text,
            at: 0,
            patlen: pat.len(),
            copied: false,
            dirc,
            delim,
        }
    }

    /// Whether a `;` follows the offset, i.e. whether another leg comes
    /// next.
    fn chained(&self) -> bool {
        cstr::byte_at(&self.text, self.at) == b';'
    }

    /// Takes this leg's pattern, delimiter and offset off the front of the
    /// command, counting them into `SEARCHCMDLEN` for `get_address`, and
    /// answers the pattern to search for.
    fn take_pattern(&mut self, source: &Source, options: c_int) -> Vec<u8> {
        // The pattern searched for, as a span of `text` or as the
        // substitute pattern.
        let mut span = (self.at, self.patlen);

        if cstr::byte_at(&self.text, self.at) != b'\0' {
            let at = self.at;
            // Find the end of the regular expression. If there is a
            // matching '/' or '?', toss it.
            let is_magic = c_int::from(magic_isset());
            let end = if self.copied {
                skip_regexp_in_place(&mut self.text, at, self.delim, is_magic) - at
            } else {
                let (end, copy) = skip_regexp_copy(&self.text[at..], self.delim, is_magic);
                if let Some(copy) = copy {
                    span = self.adopt_copy(copy);
                }
                end
            };
            let at = self.at;
            let mut p = at + end;
            if as_char(cstr::byte_at(&self.text, p)) == self.delim {
                // The pattern searched for ends at the delimiter -- when it
                // is the typed one.
                if matches!(source, Source::Typed) {
                    span = (at, p - at);
                }
                p += 1;
            }

            p = parse_offset(&self.text, p, options);

            // Compute the length of the search command, for get_address().
            let consumed = p - at;
            SEARCHCMDLEN.set(SEARCHCMDLEN.get() + len_as_int(consumed));
            self.patlen = self.patlen.wrapping_sub(consumed);
            self.at = p; // put pat after the search command
        }

        let searchstr: &[u8] = match source {
            Source::Typed => {
                let (start, len) = span;
                let end = start.saturating_add(len).min(self.text.len());
                cstr::until_nul(&self.text[start.min(end)..end])
            }
            Source::Remembered => b"",
            Source::Substitute(subst) => subst,
        };
        searchstr.to_vec()
    }

    /// Walks on in the copy of the pattern that changed "\?" to "?",
    /// answering the span of the pattern in it.
    fn adopt_copy(&mut self, mut copy: Vec<u8>) -> (usize, usize) {
        let len = cstr::until_nul(&copy).len();
        copy.truncate(len);
        copy.push(b'\0');
        // Wrapping, as upstream: `patlen` is what the caller claimed, which
        // for `get_address` is a prefix of a longer command line — the copy
        // can be longer, and C's size_t difference is then the negative
        // number this wants.
        let extra = number_as_int(self.patlen.wrapping_sub(len).cast_signed() as i64);
        SEARCHCMDLEN.set(SEARCHCMDLEN.get() + extra);
        self.text = copy;
        self.at = 0;
        self.patlen = len;
        self.copied = true;
        (0, len)
    }

    /// Steps over the `;` and the next leg's direction, which must be `/`
    /// or `?` (E386, said, otherwise).
    fn start_next_leg(&mut self) -> Result<(), CommandError> {
        self.at += 1;
        self.dirc = c_int::from(cstr::byte_at(&self.text, self.at));
        self.delim = self.dirc;
        if self.dirc != '?' as c_int && self.dirc != '/' as c_int {
            emsg(gettext(c"E386: Expected '?' or '/'  after ';'"));
            return Err(CommandError::BadLeg);
        }
        self.at += 1;
        self.patlen = self.patlen.wrapping_sub(1);
        Ok(())
    }
}

/// Every leg of the command, as in "/foo/;?bar", each starting where the
/// last one matched. Answers the last leg's hit; a leg that fails has
/// said why.
fn search_legs(
    command: &mut Command,
    pos: &mut Pos,
    mut op: Option<&mut OpArg>,
    count: c_int,
    options: c_int,
    mut sia: Option<&mut SearchItArg>,
) -> Result<Hit, CommandError> {
    loop {
        let hit = search_leg(
            command,
            pos,
            op.as_deref_mut(),
            count,
            options,
            sia.as_deref_mut(),
        )?;
        // The search command can be followed by a ';' to do another
        // search, as in "/pat/;/foo/+3;?bar". That is like another search
        // command, except that the remembered direction is the first
        // search's and that an error leaves the cursor where it was. Not
        // done when called by get_address(), which handles ';' itself.
        if options & SEARCH_OPT == 0 || !command.chained() {
            return Ok(hit);
        }
        command.start_next_leg()?;
    }
}

/// One leg: its pattern taken off the command and echoed, searched for
/// `count` times from `pos`, and its offset applied. `Err` when there is
/// no pattern (E35, said) or no match (`searchit` says so, as `options`
/// asks).
fn search_leg(
    command: &mut Command,
    pos: &mut Pos,
    op: Option<&mut OpArg>,
    count: c_int,
    options: c_int,
    mut sia: Option<&mut SearchItArg>,
) -> Result<Hit, CommandError> {
    let Some(source) = pattern_to_search(cstr::byte_at(&command.text, command.at), command.delim)
    else {
        emsg(gettext(E_NOPREVRE));
        return Err(CommandError::NoPattern);
    };
    let searchstr = command.take_pattern(&source, options);
    let dirc = command.dirc;

    let echoed = echo_search_cmd(dirc, &searchstr, options);

    let off = search_offset();
    if !off.line && off.off != 0 && pos.col < ColNr::new(MAXCOL - 2) {
        back_off_start(pos, off.off);
    }

    // A ';'-chained leg always applies its offset, whatever the caller
    // asked for.
    let noof = if command.chained() { 0 } else { SEARCH_NOOF };
    let (w, b) = (Some(Win::current()), Buf::current());
    let dir = if dirc == '/' as c_int {
        FORWARD
    } else {
        BACKWARD
    };
    let kept = SEARCH_KEEP + SEARCH_PEEK + SEARCH_HIS + SEARCH_MSG + SEARCH_START + noof;
    let opts = c_int::from(off.end) * SEARCH_END + (options & kept);
    let c = searchit(
        w,
        b,
        pos,
        None,
        dir,
        &searchstr,
        count,
        opts,
        RE_LAST,
        sia.as_deref_mut(),
    );

    let wrapped = sia.as_ref().is_some_and(|sia| sia.sa_wrapped);
    let show_top_bot_msg = !shortmess(ShmFlag::SEARCH) && wrapped;
    c?;

    let off = search_offset();
    if off.end
        && let Some(op) = op
    {
        op.inclusive = true; // 'e' includes the last character
    }

    if wrapped {
        fire_autocmds(AutoEvent::SearchWrapped, false, None);
    }

    let mut hit = Hit::Match;
    let mut has_offset = false;
    if options & SEARCH_NOOF == 0 || command.chained() {
        let org_pos = *pos;
        hit = add_offset(pos, search_offset());
        has_offset = !equalpos(*pos, org_pos);
    }

    // Show [1/15] if 'S' is not in 'shortmess'.
    if let Some((mut echo, true)) = echoed {
        let inexact = count != 1
            || has_offset
            || (FDO_FLAGS.get() & OPT_FDO_FLAG_SEARCH == 0 && {
                let (w, lnum) = (Win::current(), Win::current().w_cursor.lnum);
                w.has_folding(lnum, None, None)
            });
        let cursor = Win::current().w_cursor;
        let (top_bot, maxcount) = (show_top_bot_msg, number_as_int(p_msc()));
        let tm = SEARCH_STAT_DEF_TIMEOUT;
        cmdline_search_stat(
            dirc, *pos, cursor, top_bot, &mut echo, inexact, maxcount, tm,
        );
    }
    Ok(hit)
}

// ---------------------------------------------------------------------
// `'showmatch'` and `gn`/`gN`.
// ---------------------------------------------------------------------

/// Whether `c` is one of the `'matchpairs'` characters that should blink,
/// given the effective right-to-left setting.
///
/// `'matchpairs'` is `"x:y,x:y"`: the opening character of each pair
/// blinks in left-to-right mode and the closing one in right-to-left.
fn mps_shows_match(c: c_int) -> bool {
    let rightleft = (Win::current().w_onebuf_opt.wo_rl != 0) ^ p_ri();
    let buf = Buf::current();
    let mps = buf.b_p_mps.bytes();
    // The walk reads the value as the C string it is: the end is a NUL.
    let char_from = |i: usize| char_at(mps.get(i..).unwrap_or_default());
    let len_from = |i: usize| c_cluster_len(mps.get(i..).unwrap_or_default());
    let mut i = 0;
    while i < mps.len() {
        if char_from(i).get() == c && rightleft {
            return true;
        }
        i += len_from(i) + 1;
        if char_from(i).get() == c && !rightleft {
            return true;
        }
        i += len_from(i);
        if i >= mps.len() {
            return false;
        }
        i += 1;
    }
    false
}

/// Briefly show the match for the just-typed `c`, for `'showmatch'`.
pub fn showmatch(c: c_int) {
    // Only show a match for characters in 'matchpairs'.
    if !mps_shows_match(c) {
        return;
    }

    let found = findmatch(None, 0);
    let Some(mut lpos) = found else {
        vim_beep(OPT_BO_FLAG_SHOWMATCH); // no match, so beep
        return;
    };
    if lpos.lnum < Win::current().w_topline || lpos.lnum >= Win::current().w_botline {
        return;
    }

    if Win::current().w_onebuf_opt.wo_wrap == 0 {
        let vcol = getvcol(Win::current(), &mut lpos).1;
        if !(vcol >= Win::current().w_leftcol
            && vcol < Win::current().w_leftcol + Win::current().w_view_width)
        {
            return;
        }
    }

    // 'scrolloff' and 'sidescrolloff' are window-local with a global
    // fallback; the blink writes through whichever is in effect.
    let win = Win::current();
    let so = ScrollOff::of(win, ScrollMargin::Lines);
    let siso = ScrollOff::of(win, ScrollMargin::Columns);

    let mpos = lpos; // save the pos, update_screen() may change it
    let save_cursor = Win::current().w_cursor;
    let save_so = so.get();
    let save_siso = siso.get();

    // Handle "$" in 'cpo': if the ')' is typed on top of the "$", stop
    // displaying the "$".
    if DOLLAR_VCOL.get() >= ColNr::ZERO && DOLLAR_VCOL.get() == Win::current().w_virtcol {
        DOLLAR_VCOL.set(ColNr::NONE);
    }
    Win::current().w_virtcol += 1; // do display ')' just before "$"

    let save_dollar_vcol = DOLLAR_VCOL.get();
    let save_state = STATE.get();
    STATE.set(MODE_SHOWMATCH);
    ui_cursor_shape(); // may show a different cursor shape
    Win::current().w_cursor = mpos; // move to the matching char
    so.set(0); // don't use 'scrolloff' here
    siso.set(0); // don't use 'sidescrolloff' here
    show_cursor_info_later(false);
    let _ = update_screen(); // show the new char
    setcursor();
    ui_flush();
    // Restore dollar_vcol: setcursor() may call curs_rows(), which
    // resets it when the matching position is on an earlier line and
    // has a higher column number.
    DOLLAR_VCOL.set(save_dollar_vcol);

    // Brief pause, unless 'm' is present in 'cpo' and a character is
    // available.
    if cpo_has(CpoFlag::SHOWMATCH) {
        os_delay(p_mat().cast_unsigned() * 100 + 8, true);
    } else if !char_avail() {
        os_delay(p_mat().cast_unsigned() * 100 + 9, false);
    }

    Win::current().w_cursor = save_cursor; // restore cursor position
    so.set(save_so);
    siso.set(save_siso);
    STATE.set(save_state);
    ui_cursor_shape(); // may show a different cursor shape
}
