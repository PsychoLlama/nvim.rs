//! Finding the identifier a command is about to act on: the word under
//! the cursor, the one at a given position, and the local declaration
//! `gd`/`gD` jumps to.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

/// The `mb_get_class` classes this file cares about: 0 is white space, 1 is
/// punctuation, 2 and up are the word classes (one per script).
const CLASS_WHITE: c_int = 0;
const CLASS_PUNCT: c_int = 1;
const CLASS_WORD: c_int = 2;

use crate::ops::Op;
use crate::winlayer::{Buf, Win};

use crate::ascii::ascii_iswhite;
use crate::change::get_leader_len;
use crate::charset::is_word_at;
use crate::cstr;
use crate::drawscreen::state::clear_cmdline;
use crate::fold::fold_open_cursor;
use crate::getchar::state::KeyTyped;
use crate::mark::setpcmark;
use crate::mbyte::{class_in, cluster_len, head_off};
use crate::memline::Lines;
use crate::message::e_noident;
use crate::message::state::msg_silent;
use crate::message::{emsg, messaging};
use crate::normal::{FIND_EVAL, FIND_IDENT, FIND_STRING, FM_FORWARD, clear_op_beep};
use crate::option::shortmess;
use crate::option::vars::{P_SCS, P_WS, fdo_flags, p_scs, p_ws};
use crate::options::kOptFdoFlagSearch;
use crate::os::cshim::gettext;
use crate::pos::clearpos;
use crate::regexp::RE_LAST;
use crate::search::{BACKWARD, FORWARD, SEARCH_START, findmatchlimit, reset_search_dir, searchit};
use crate::textobject::findpar;
use crate::types::{ColNr, LineNr, NUL, OpType, Pos, ShmFlag, int64_t};
use core::ffi::{CStr, c_int, c_uint};

/// Whether the character at `col` of `line` continues a `FIND_EVAL`
/// expression rather than ending it -- `.`, `->` and a balanced `[...]`
/// subscript.
///
/// `col` is advanced past the second byte of `->`, and `bn` is the depth of
/// unclosed brackets, counted from whichever end the walk started at. A
/// backwards walk is only asked past column 0.
fn find_is_eval_item(line: &[u8], col: &mut c_int, bn: &mut c_int, dir: c_int) -> bool {
    let backward = dir == BACKWARD as c_int;
    // Walking backwards, a `]` opens a subscript and a `[` closes it.
    let opener = if backward { b']' } else { b'[' };
    let closer = if backward { b'[' } else { b']' };
    let at = usize::try_from(*col).unwrap_or(0);
    let c = cstr::byte_at(line, at);
    if c == opener {
        *bn += 1;
    }
    if *bn > 0 {
        if c == closer {
            *bn -= 1;
        }
        return true;
    }
    if c == b'.' {
        return true;
    }
    // `->` read from either end: backwards the cursor is on the `>`,
    // forwards on the `-`.
    let (head, tail) = if backward {
        (c, cstr::byte_at(line, at - 1))
    } else {
        (cstr::byte_at(line, at + 1), c)
    };
    if head == b'>' && tail == b'-' {
        *col += dir;
        return true;
    }
    false
}

/// A copy of the identifier or string under the cursor -- empty when there
/// is none -- as [`find_ident_under_cursor`] finds it.
pub(crate) fn ident_under_cursor(find_type: c_int) -> Vec<u8> {
    let Some((col, len)) = find_ident_under_cursor(find_type) else {
        return Vec::new();
    };
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let from = usize::try_from(col).unwrap_or(0);
    line[from..from + len].to_vec()
}

/// The identifier or string under the cursor: the column it starts at in
/// the cursor's line and its length in bytes, or `None` (with the error
/// given) when there is none.
pub(crate) fn find_ident_under_cursor(find_type: c_int) -> Option<(ColNr, usize)> {
    let win = Win::current();
    let pos = win.w_cursor;
    find_ident_at_pos(win, pos.lnum, pos.col, find_type)
}

/// The buffer line `find_ident_at_pos` walks, addressed by byte column; a
/// column at or past its end reads as the terminator.
#[derive(Clone, Copy)]
struct ScanLine<'a>(&'a [u8]);

impl<'a> ScanLine<'a> {
    /// The byte at `col`, or NUL at and past the end of the line.
    fn at(self, col: c_int) -> c_int {
        c_int::from(cstr::byte_at(self.0, Self::index(col)))
    }
    /// The rest of the line from `col` on.
    fn from(self, col: c_int) -> Self {
        ScanLine(&self.0[Self::index(col)..])
    }
    /// The character class of the character starting at `col`.
    fn class(self, col: c_int) -> c_int {
        class_in(
            self.0.get(Self::index(col)..).unwrap_or_default(),
            Buf::current(),
        )
    }
    /// The length of the character at `col`, its combining marks included.
    fn char_len(self, col: c_int) -> c_int {
        c_int::try_from(cluster_len(&self.0[Self::index(col)..])).unwrap_or(0)
    }
    /// The column the character in front of `col` starts at.
    fn prev_col(self, col: c_int) -> c_int {
        // Only asked past column 0, so `col - 1` is a byte of this line.
        col - 1 - c_int::try_from(head_off(self.0, Self::index(col - 1))).unwrap_or(0)
    }
    /// `find_is_eval_item` at `col`, advancing `col` and the bracket depth.
    fn is_eval_item(self, col: &mut c_int, bn: &mut c_int, dir: c_int) -> bool {
        find_is_eval_item(self.0, col, bn, dir)
    }
    /// A column as an index, clamped: the walks ask about the terminator.
    fn index(col: c_int) -> usize {
        usize::try_from(col).unwrap_or(0)
    }
}

/// The identifier or string at a given position.
///
/// Runs in at most two passes. The first accepts only a word character; the
/// second, which `FIND_STRING` asks for and which `FIND_IDENT` alone skips,
/// accepts anything that is not white space. Each pass scans forward from the
/// position for a character it will take, then backs up to that run's start.
///
/// Answers the column the run starts at and its length, or `None` (with the
/// error given) when there is nothing there.
pub(crate) fn find_ident_at_pos(
    window: Win,
    lnum: LineNr,
    mut startcol: ColNr,
    find_type: c_int,
) -> Option<(ColNr, usize)> {
    let eval = find_type & FIND_EVAL.cast_signed() != 0;
    let mut lines = window.buffer().lines();
    let mut line = ScanLine(lines.line(lnum));
    let mut col: c_int = 0;
    let mut this_class: c_int = 0;
    // Pass 0 wants a word character; pass 1 will take punctuation too.
    let mut pass = c_int::from(find_type & FIND_IDENT.cast_signed() == 0);
    while pass < 2 {
        col = startcol;
        while line.at(col) != NUL {
            // A `]` ends an expression, and is where the backwards walk
            // has to start from.
            if eval && line.at(col) == ']' as c_int {
                break;
            }
            this_class = line.class(col);
            if this_class != CLASS_WHITE && (pass == 1 || this_class != CLASS_PUNCT) {
                break;
            }
            col += line.char_len(col);
        }
        // The bracket depth the backwards walk starts with.
        let mut bn = c_int::from(line.at(col) == ']' as c_int);
        if eval && line.at(col) == ']' as c_int {
            // A subscript belongs to the name in front of it, so pretend
            // the `]` is a word character.
            this_class = class_in(b"a", Buf::current());
        } else {
            this_class = line.class(col);
        }
        while col > 0 && this_class != CLASS_WHITE {
            let mut prevcol = line.prev_col(col);
            let prev_class = line.class(prevcol);
            if this_class != prev_class
                && (pass == 0
                    || prev_class == CLASS_WHITE
                    || find_type & FIND_IDENT.cast_signed() != 0)
                && (!eval
                    || prevcol == 0
                    || !line.is_eval_item(&mut prevcol, &mut bn, BACKWARD as c_int))
            {
                break;
            }
            col = prevcol;
        }
        // Every word script counts as the same class from here on.
        this_class = this_class.min(CLASS_WORD);
        if find_type & FIND_STRING.cast_signed() == 0 || this_class == CLASS_WORD {
            break;
        }
        pass += 1;
    }

    if line.at(col) == NUL || (pass == 0 && this_class != CLASS_WORD) {
        if find_type & FIND_STRING.cast_signed() != 0 {
            emsg(gettext(c"E348: No string under cursor"));
        } else {
            emsg(gettext(e_noident));
        }
        return None;
    }

    line = line.from(col);
    let textcol = col;
    // Now walk forward to the run's end. `startcol` becomes the cursor's
    // offset within the run, which is how far the `FIND_EVAL` walk is
    // allowed to keep taking subscripts.
    let mut bn = 0;
    startcol -= col;
    col = 0;
    this_class = line.class(0);
    while line.at(col) != NUL
        && (if pass == 0 {
            line.class(col) == this_class
        } else {
            line.class(col) != CLASS_WHITE
        } || (eval
            && col <= startcol
            && line.is_eval_item(&mut col, &mut bn, FORWARD as c_int)))
    {
        col += line.char_len(col);
    }
    debug_assert!(col >= 0);
    Some((textcol, usize::try_from(col).unwrap_or(0)))
}

/// `gd` and `gD`: jump to the local or global declaration of the identifier
/// under the cursor.
pub(crate) fn nv_gd(op: Op, nchar: c_int, thisblock: c_int) {
    let word = ident_under_cursor(FIND_IDENT.cast_signed());
    let locally = nchar == 'd' as c_int;
    let found =
        !word.is_empty() && find_decl(&word, locally, thisblock != 0, SEARCH_START as c_int);
    if !found {
        clear_op_beep(op);
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as c_uint != 0
        && KeyTyped.get()
        && op.op_type == OpType::Nop
    {
        fold_open_cursor();
    }
    // The search left a "search hit" message that has nothing to say
    // here, unless 'shortmess' has already suppressed it.
    if messaging() && msg_silent.get() == 0 && !shortmess(ShmFlag::SEARCHCOUNT) {
        clear_cmdline.set(true);
    }
}

/// Whether the byte at `offset` is ordinary code rather than inside a string
/// or a comment. A very rough C-shaped scan: it knows `"`, `'`, `/* */` and
/// `//`, and it only ever looks at the one line.
pub(crate) fn is_ident(line: &[u8], offset: c_int) -> bool {
    let mut incomment = false;
    // The quote that opened the string we are inside, or 0.
    let mut instring: u8 = 0;
    let mut prev: u8 = 0;
    let end = usize::try_from(offset).unwrap_or(0).min(line.len());
    for &c in &line[..end] {
        if instring != 0 {
            if prev != b'\\' && c == instring {
                instring = 0;
            }
        } else if (c == b'"' || c == b'\'') && !incomment {
            instring = c;
        } else if incomment {
            if prev == b'*' && c == b'/' {
                incomment = false;
            }
        } else if prev == b'/' && c == b'*' {
            incomment = true;
        } else if prev == b'/' && c == b'/' {
            // The rest of the line is a comment, so the offset is too.
            return false;
        }
        prev = c;
    }
    !incomment && instring == 0
}

/// [`find_decl`] of a word held as text: `searchdecl()`'s entry point.
pub(crate) fn find_decl_of(word: &CStr, locally: bool, thisblock: bool, flags: c_int) -> bool {
    find_decl(word.to_bytes(), locally, thisblock, flags)
}

/// Search backwards for where `word` is declared: the first occurrence that is
/// not inside a comment or a string, above the cursor. `locally` limits the
/// search to the current `{}` block (`gd`); `thisblock` further refuses a
/// match whose block closes before the cursor.
pub(crate) fn find_decl(word: &[u8], locally: bool, thisblock: bool, flags_arg: c_int) -> bool {
    let mut searchflags = flags_arg;
    // `\V` plus the word, between `\<` and `\>` when it starts with a word
    // character.
    let mut pat = Vec::with_capacity(word.len() + 6);
    let anchored = is_word_at(word, Buf::current());
    pat.extend_from_slice(if anchored { b"\\V\\<" } else { b"\\V" });
    pat.extend_from_slice(word);
    if anchored {
        pat.extend_from_slice(b"\\>");
    }

    let old_pos = Win::current().w_cursor;
    let save_p_ws = p_ws();
    let save_p_scs = p_scs();
    // The search must not wrap round the file or guess at case.
    P_WS.set(false);
    P_SCS.set(false);

    // Where the enclosing block starts, which is as far back as a local
    // declaration may be.
    let par_pos;
    let in_block = locally && findpar(BACKWARD as c_int, 1, '{' as c_int, false).is_some();
    if !in_block {
        setpcmark();
        Win::current().w_cursor.lnum = 1;
        par_pos = Win::current().w_cursor;
    } else {
        par_pos = Win::current().w_cursor;
        // Back up over the function's own header lines.
        while Win::current().w_cursor.lnum > 1 {
            let lnum = Win::current().w_cursor.lnum;
            if Lines::current()
                .line(lnum)
                .iter()
                .all(|&b| ascii_iswhite(c_int::from(b)))
            {
                break;
            }
            Win::current().w_cursor.lnum -= 1;
        }
    }
    Win::current().w_cursor.col = 0;

    // The last match that was inside a comment or a string, kept as the
    // answer of last resort.
    let mut found_pos = Pos {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found;
    loop {
        let wp = Win::current();
        let mut pos = wp.w_cursor;
        let (win, buf) = (Some(wp), Buf::current());
        let opts = searchflags;
        let re = RE_LAST as c_int;
        let hit = searchit(win, buf, &mut pos, None, FORWARD, &pat, 1, opts, re, None);
        Win::current().w_cursor = pos;
        found = hit != 0;
        if Win::current().w_cursor.lnum >= old_pos.lnum {
            // Found it below the cursor, which is not a declaration of
            // what is under the cursor.
            found = false;
        }
        if thisblock && found {
            // Refuse a match whose enclosing block closes before the
            // cursor: it is a different scope.
            let travel = int64_t::from(old_pos.lnum - Win::current().w_cursor.lnum + 1);
            let brace = '}' as c_int;
            // SAFETY: the current window and buffer are live.
            let close = findmatchlimit(None, brace, FM_FORWARD.cast_signed(), travel);
            if let Some(close) = close
                && close.lnum < old_pos.lnum
            {
                Win::current().w_cursor = close;
                continue;
            }
        }
        if !found {
            if found_pos.lnum != 0 {
                Win::current().w_cursor = found_pos;
                found = true;
            }
            break;
        }
        let lnum = Win::current().w_cursor.lnum;
        let leader = get_leader_len(Lines::current().line(lnum), false, true).0;
        if leader > 0 {
            // The whole line is a comment; skip past it.
            Win::current().w_cursor.lnum += 1;
            Win::current().w_cursor.col = 0;
            continue;
        }
        let cursor = Win::current().w_cursor;
        let valid = is_ident(Lines::current().line(cursor.lnum), cursor.col);
        if !valid && found_pos.lnum != 0 {
            // Nothing better than what was already found.
            Win::current().w_cursor = found_pos;
            break;
        }
        if valid && !locally {
            break;
        }
        if valid && Win::current().w_cursor.lnum >= par_pos.lnum {
            // Past the start of the block: a local search is done, and
            // the earlier match wins if there was one.
            if found_pos.lnum != 0 {
                Win::current().w_cursor = found_pos;
            }
            break;
        }
        if valid {
            found_pos = Win::current().w_cursor;
        } else {
            clearpos(&mut found_pos);
        }
        // Having found one match, the next search must move.
        searchflags &= !(SEARCH_START as c_int);
    }

    if !found {
        Win::current().w_cursor = old_pos;
    } else {
        Win::current().w_set_curswant = true;
        reset_search_dir();
    }
    P_WS.set(save_p_ws);
    P_SCS.set(save_p_scs);
    found
}
