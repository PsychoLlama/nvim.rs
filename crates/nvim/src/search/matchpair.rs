//! Matching brackets.
//!
//! [`findmatchlimit`] is the walk `%` and its neighbours share: from a
//! position it looks for the other half of a `'matchpairs'` pair, a
//! `#if`/`#endif` partner, or the end of a C comment or raw string. What
//! it has to get right is everything that makes a bracket *not* count —
//! a `//` or Lisp `;` comment ([`check_linecomment`]), a string, an
//! escaping backslash, a raw-string delimiter ([`find_rawstring_end`]) —
//! and the `'cpoptions'` flags `%` and `M` that switch those rules off.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::charset::skip;
use crate::cstr::byte_at;
use crate::mbyte::char_at;
use crate::option::cpo_has;
use crate::optionstr::LocalOptStr;
use crate::pos::MAXCOL;
use crate::strings::has_char;
use crate::types::{CpoFlag, NUL};
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

const FM_BACKWARD: c_int = super::FM_BACKWARD as c_int;
const FM_FORWARD: c_int = super::FM_FORWARD as c_int;
const FM_BLOCKSTOP: c_int = super::FM_BLOCKSTOP as c_int;
const FORWARD: c_int = super::FORWARD as c_int;
const BACKWARD: c_int = super::BACKWARD as c_int;

/// Find the match for the bracket under the cursor.
pub fn findmatch(op: Option<&mut OpArg>, initc: c_int) -> Option<Pos> {
    findmatchlimit(op, initc, 0, 0)
}

/// Find the matching paren or brace, if it is within `maxtravel` lines of
/// the cursor. A `maxtravel` of 0 means "search until falling off the
/// edge of the file".
///
/// `initc` is the character to find a match for; NUL means the character
/// at or after the cursor. Four values are special: `'*'` looks for the
/// other end of a `/* */` comment, `'/'` does the same but ignores a
/// comment end, `'#'` looks for a preprocessor directive, and `'R'` looks
/// for the start of a raw string `R"delim(text)delim"` (backwards only).
///
/// `flags` is `FM_BACKWARD`/`FM_FORWARD` (which way to look, for the
/// `'/'`, `'*'` and `'#'` forms) and `FM_BLOCKSTOP` (stop at a `{` or `}`
/// in column 0).
///
/// `op` is only ever written to — `op.motion_type`, for the linewise
/// `#if` case — and all but four of the callers have no operator to
/// offer, which is why it is an `Option` and not a pointer.
pub fn findmatchlimit(
    op: Option<&mut OpArg>,
    initc: c_int,
    flags: c_int,
    maxtravel: int64_t,
) -> Option<Pos> {
    find_match(op, initc, flags, maxtravel)
}

// ---------------------------------------------------------------------
// Deciding what to look for.
// ---------------------------------------------------------------------

/// The byte `off` columns away from `col`, as the pointer walk read it.
///
/// Past the end of the line is its terminator, which is what stopped every
/// forward test. A *negative* index is the one place this is not the
/// pointer's answer: the C read the byte before the line, and every caller
/// guards against reaching that, so answering NUL keeps the guards honest
/// and takes the read out of the language.
fn at_col(line: &[u8], col: c_int, off: c_int) -> u8 {
    usize::try_from(col + off).map_or(0, |i| byte_at(line, i))
}

/// Whether the character before `line[col]` is `ch`, reporting the
/// column of that previous character through `prev`.
///
/// False when `col` is zero. Handles multi-byte characters.
fn check_prevcol(line: &[u8], col: c_int, ch: u8, prev: Option<&mut c_int>) -> bool {
    let mut col = col - 1;
    if col > 0 {
        col -= head_off(line, col as usize) as c_int;
    }
    if let Some(prev) = prev {
        *prev = col;
    }
    col >= 0 && byte_at(line, col as usize) == ch
}

/// How many backslashes immediately precede `line[col]`.
///
/// An odd number means the character there is escaped. `'cpoptions'` "M"
/// switches the whole idea off, and both callers check that first.
fn backslash_count(line: &[u8], col: c_int) -> c_int {
    let mut count = 0;
    let mut col = col;
    while check_prevcol(line, col, b'\\', Some(&mut col)) {
        count += 1;
    }
    count
}

/// Look `*initc` up in `'matchpairs'`.
///
/// `'matchpairs'` is `"x:y,x:y"`. On a hit, `*findc` becomes the opposite
/// character and `*backwards` the direction to look in; with `switchit`
/// the roles are swapped, which is how `%` on a closing bracket comes to
/// look backwards for the opening one. Everything is left alone when the
/// character is not in the option.
///
/// Safe because the only pointer it walks is `'matchpairs'` itself, a live
/// NUL-terminated option value of the current buffer.
fn find_mps_values(target: &mut Target, switchit: bool) {
    let (initc, findc) = (&mut target.initc, &mut target.findc);
    let backwards = &mut target.backwards;
    // The walk reads 'matchpairs' in place, as upstream's does.
    let mut ptr = Buf::current().b_p_mps.value_ptr();
    while unsafe { *ptr } as c_int != NUL {
        // The opening half of this pair.
        if unsafe { utf_ptr2char(ptr) } == *initc {
            let other = unsafe { utf_ptr2char(ptr.offset(utfc_ptr2len(ptr) as isize + 1)) };
            if switchit {
                *findc = *initc;
                *initc = other;
                *backwards = true;
            } else {
                *findc = other;
                *backwards = false;
            }
            return;
        }
        // The closing half.
        let prev = ptr;
        ptr = unsafe { ptr.offset((utfc_ptr2len(ptr) + 1) as isize) };
        if unsafe { utf_ptr2char(ptr) } == *initc {
            if switchit {
                *findc = *initc;
                *initc = unsafe { utf_ptr2char(prev) };
                *backwards = false;
            } else {
                *findc = unsafe { utf_ptr2char(prev) };
                *backwards = true;
            }
            return;
        }
        ptr = unsafe { ptr.offset(utfc_ptr2len(ptr) as isize) };
        if unsafe { *ptr } as c_int == ',' as c_int {
            ptr = unsafe { ptr.offset(1) };
        }
    }
}

/// What the walk is looking for, once `initc` and the text under the
/// cursor have been interpreted.
struct Target {
    /// The character that opens; NUL when looking for a comment end.
    initc: c_int,
    /// The character that closes.
    findc: c_int,
    backwards: bool,
    /// Which way a comment end is being looked for; 0 when the target is
    /// a bracket pair.
    comment_dir: c_int,
    /// `[/` ignores running backwards into a `*/`: for that command any
    /// comment will do.
    ignore_cend: bool,
    /// `'R'`: the target is a raw-string start.
    raw_string: bool,
    /// Whether the bracket the walk started on was itself escaped; only a
    /// match with the same escaping counts.
    match_escaped: c_int,
}

/// What interpreting `initc` decided to do.
enum Plan {
    /// Walk the buffer looking for this target.
    Walk(Target),
    /// Walk lines looking for a `#if`/`#else`/`#endif` partner, in this
    /// direction.
    Hash(c_int),
    /// There is nothing to look for.
    Nothing,
}

/// Interpret `initc` and, when it is NUL, the text under the cursor.
///
/// May move `pos` — onto the other half of a `/*` or `*/`, or forward
/// along the line to the first bracket after the cursor.
///
/// `line` has to be the line `pos` is on for the answer to mean anything,
/// but that is a correctness claim and not a memory one: a mismatched
/// pair reads the wrong bytes of a slice it still owns, and every read
/// here is bounds-checked.
fn make_plan(
    op: Option<&mut OpArg>,
    initc: c_int,
    dir: c_int,
    pos: &mut Pos,
    line: &[u8],
    cpo_match: bool,
    cpo_bsl: bool,
) -> Plan {
    let mut target = Target {
        initc,
        findc: 0,
        backwards: false,
        comment_dir: 0,
        ignore_cend: false,
        raw_string: false,
        match_escaped: 0,
    };

    // '/' and '*' are special cases: look for the start or end of a
    // comment. When '/' is used, running backwards into a "*/" is
    // ignored, because for the "[*" command any comment will do.
    if initc == '/' as c_int || initc == '*' as c_int || initc == 'R' as c_int {
        target.comment_dir = dir;
        target.ignore_cend = initc == '/' as c_int;
        target.backwards = dir != FORWARD;
        target.raw_string = initc == 'R' as c_int;
        target.initc = NUL;
        return Plan::Walk(target);
    }

    if initc != '#' as c_int && initc != NUL {
        // A given character: look it up in the table.
        find_mps_values(&mut target, true);
        if dir != 0 {
            target.backwards = dir != FORWARD;
        }
        if target.findc == NUL {
            return Plan::Nothing;
        }
        return Plan::Walk(target);
    }

    // Either initc is '#', or no initc was given and something under
    // or near the cursor has to be matched.
    let mut hash_dir = if initc == '#' as c_int { dir } else { 0 };
    if initc != '#' as c_int {
        // Only check for the special things when 'cpo' has no '%'.
        if !cpo_match {
            let white = skip::white(line);
            let col = pos.col;
            let at = |off: c_int| at_col(line, col, off);
            if byte_at(line, white) == b'#' && pos.col <= white as ColNr {
                // Are we before or at #if, #else etc.?
                let word = after_hash(line, white);
                if word.starts_with(b"if") || word.starts_with(b"endif") || word.starts_with(b"el")
                {
                    hash_dir = 1;
                }
            } else if at(0) == b'/' {
                // Are we on a comment?
                if at(1) == b'*' {
                    target.comment_dir = FORWARD;
                    target.backwards = false;
                    pos.col += 1;
                } else if pos.col > 0 && at(-1) == b'*' {
                    target.comment_dir = BACKWARD;
                    target.backwards = true;
                    pos.col -= 1;
                }
            } else if at(0) == b'*' {
                if at(1) == b'/' {
                    target.comment_dir = BACKWARD;
                    target.backwards = true;
                } else if pos.col > 0 && at(-1) == b'/' {
                    target.comment_dir = FORWARD;
                    target.backwards = false;
                }
            }
        }

        // Not on a comment or on the # at the start of a line: look
        // for a brace anywhere on this line at or after the cursor.
        if hash_dir == 0 && target.comment_dir == 0 {
            // Beyond the end of the line, use its last character.
            if pos.col as usize >= line.len() && pos.col != 0 {
                pos.col -= 1;
            }
            loop {
                let rest = &line[(pos.col as usize).min(line.len())..];
                target.initc = char_at(rest);
                if target.initc == NUL {
                    break;
                }
                find_mps_values(&mut target, false);
                if target.findc != 0 {
                    break;
                }
                pos.col += cluster_len(rest) as ColNr;
            }
            if target.findc == 0 {
                // No brace in the line; maybe use "  #if" then.
                if !cpo_match && byte_at(line, skip::white(line)) == b'#' {
                    hash_dir = 1;
                } else {
                    return Plan::Nothing;
                }
            } else if !cpo_bsl {
                target.match_escaped = backslash_count(line, pos.col) & 1;
            }
        }
    }

    if hash_dir == 0 {
        return Plan::Walk(target);
    }

    // Look for a matching #if, #else, #elif or #endif.
    if let Some(op) = op {
        op.motion_type = kMTLineWise; // linewise for this case only
    }
    if initc != '#' as c_int {
        let word = after_hash(line, skip::white(line));
        hash_dir = if word.starts_with(b"if") || word.starts_with(b"el") {
            1
        } else if word.starts_with(b"endif") {
            -1
        } else {
            return Plan::Nothing;
        };
    }
    Plan::Hash(hash_dir)
}

/// The directive name after the `#` at `hash`: one byte on, then white
/// space skipped.
fn after_hash(line: &[u8], hash: usize) -> &[u8] {
    let rest = &line[(hash + 1).min(line.len())..];
    &rest[skip::white(rest)..]
}

/// Walk lines looking for the `#if`/`#else`/`#endif` that partners the
/// one the cursor is on.
///
/// # Safety
/// `pos` must address the current buffer.
fn find_hash_match(mut pos: Pos, hash_dir: c_int, initc: c_int) -> Option<Pos> {
    let mut count = 0;
    pos.col = 0;
    let mut lines = Lines::current();
    while !got_int.get() {
        if hash_dir > 0 {
            if pos.lnum == Buf::current().b_ml.ml_line_count {
                break;
            }
        } else if pos.lnum == 1 {
            break;
        }
        pos.lnum += hash_dir;
        line_breakcheck(); // check for CTRL-C typed
        let line = lines.line(pos.lnum);
        let white = skip::white(line);
        if byte_at(line, white) != b'#' {
            continue;
        }
        pos.col = white as ColNr;
        let word = after_hash(line, white);
        if hash_dir > 0 {
            if word.starts_with(b"if") {
                count += 1;
            } else if word.starts_with(b"el") {
                if count == 0 {
                    return Some(pos);
                }
            } else if word.starts_with(b"endif") {
                if count == 0 {
                    return Some(pos);
                }
                count -= 1;
            }
        } else if word.starts_with(b"if") {
            if count == 0 {
                return Some(pos);
            }
            count -= 1;
        } else if initc == '#' as c_int && word.starts_with(b"el") {
            if count == 0 {
                return Some(pos);
            }
        } else if word.starts_with(b"endif") {
            count += 1;
        }
    }
    None
}

// ---------------------------------------------------------------------
// The walk itself.
// ---------------------------------------------------------------------

/// What looking at one position decided.
enum Step {
    /// Look at the next position.
    Next,
    /// The match is here.
    Found(Pos),
    /// Give up: there is no match at all.
    Nothing,
}

/// Everything the walk carries from one position to the next.
struct Walk {
    pos: Pos,
    /// The buffer's line cache. The walk reads `pos`'s line out of it at
    /// every step; upstream keeps a pointer instead and re-derives it at
    /// each line boundary and after anything that may have released it,
    /// which is the bookkeeping this replaces.
    lines: Lines,
    backwards: bool,
    lisp: bool,
    /// Where a `//` (or Lisp `;`) comment starts on this line, or MAXCOL.
    comment_col: c_int,
    /// The start position is inside a Lisp comment, so the match has to
    /// be inside it too.
    lispcomm: bool,
    /// Lines stepped over so far, against `maxtravel`.
    traveled: c_int,
    maxtravel: int64_t,
    /// Whether quoted text on this line can be skipped: -1 = not counted
    /// yet, 0 = no (an odd number of quotes, or `'cpo'` has `%`),
    /// 1 = yes.
    do_quotes: c_int,
    inquote: bool,
    /// Whether the *start* position was inside quotes; `None` until the
    /// first line has been counted.
    start_in_quotes: Option<bool>,
    /// Nesting depth, and where the innermost `/*` was found.
    count: c_int,
    match_pos: Pos,
}

impl Walk {
    /// Step one character backwards, answering false at the start of the
    /// buffer or when the travel limit is reached.
    ///
    /// # Safety
    /// The current buffer must be the one `self.pos` addresses.
    fn step_back(&mut self, comment_dir: c_int) -> bool {
        // The character to match is inside a comment; don't look
        // outside it.
        if self.lispcomm && self.pos.col < self.comment_col {
            return false;
        }
        if self.pos.col != 0 {
            self.pos.col -= 1;
            let back = head_off(self.lines.line(self.pos.lnum), self.pos.col as usize);
            self.pos.col -= back as ColNr;
            return true;
        }
        // At the start of the line, go to the previous one.
        if self.pos.lnum == 1 {
            return false; // start of file
        }
        self.pos.lnum -= 1;
        self.traveled += 1;
        if self.maxtravel > 0 && self.traveled as int64_t > self.maxtravel {
            return false;
        }
        self.pos.col = self.lines.line_len(self.pos.lnum); // pos.col on the trailing NUL
        self.do_quotes = -1;
        line_breakcheck();
        // Does this line hold a single-line comment?
        if comment_dir != 0 || self.lisp {
            self.comment_col = check_linecomment(self.lines.line(self.pos.lnum));
        }
        if self.lisp && self.comment_col != MAXCOL {
            self.pos.col = self.comment_col; // skip the comment
        }
        true
    }

    /// Step one character forwards, answering false at the end of the
    /// buffer or when the travel limit is reached.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    fn step_forward(&mut self) -> bool {
        let line = self.lines.line(self.pos.lnum);
        let col = self.pos.col as usize;
        let at_end = col >= line.len()
            // For Lisp don't look for a match inside a comment.
            || (self.lisp && self.comment_col != MAXCOL && self.pos.col == self.comment_col);
        if !at_end {
            self.pos.col += cluster_len(&line[col..]) as ColNr;
            return true;
        }
        // End of file, or the line is exhausted and the comment with
        // it — then don't look for a match out in the code.
        if self.pos.lnum == Buf::current().b_ml.ml_line_count || self.lispcomm {
            return false;
        }
        self.pos.lnum += 1;
        // Upstream compares the count *before* the increment here and
        // *after* it when going backwards; preserved.
        let before = self.traveled;
        self.traveled += 1;
        if self.maxtravel != 0 && before as int64_t > self.maxtravel {
            return false;
        }
        self.pos.col = 0;
        self.do_quotes = -1;
        line_breakcheck();
        if self.lisp {
            // In the new line.
            self.comment_col = check_linecomment(self.lines.line(self.pos.lnum));
        }
        true
    }

    /// Look at one position while hunting for the other end of a comment
    /// or of a raw string.
    ///
    /// Comments do not nest, and quotes inside them are ignored.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    fn comment_step(&mut self, target: &Target) -> Step {
        let col = self.pos.col;
        let line = self.lines.line(self.pos.lnum);
        let at = |off: c_int| at_col(line, col, off);

        if target.comment_dir == FORWARD {
            if at(0) == b'*' && at(1) == b'/' {
                self.pos.col += 1;
                return Step::Found(self.pos);
            }
            return Step::Next;
        }

        // Searching backwards. A comment may contain "/*" or "//",
        // and may start or end with "/*/". Ignore a "/*" after "//"
        // and after "*".
        if self.pos.col == 0 {
            return Step::Next;
        }
        if target.raw_string {
            let opens = at(-1) == b'R'
                && at(0) == b'"'
                && line[(col as usize + 1).min(line.len())..].contains(&b'(');
            if opens {
                // A possible start of a raw string. Now that the
                // delimiter is known, check whether it ends before
                // where the search started, or before the previously
                // found raw-string start.
                let end = if self.count > 0 {
                    self.match_pos
                } else {
                    Win::current().w_cursor
                };
                // The borrow of `line` ends here: the scan below reads
                // every line between the two positions out of the same
                // cache, which is what used to release `linep`.
                if !find_rawstring_end(&mut self.lines, &self.pos, &end) {
                    self.count += 1;
                    self.match_pos = self.pos;
                    self.match_pos.col -= 1;
                }
            }
            return Step::Next;
        }
        if at(-1) == b'/'
            && at(0) == b'*'
            && (self.pos.col == 1 || at(-2) != b'*')
            && self.pos.col < self.comment_col
        {
            self.count += 1;
            self.match_pos = self.pos;
            self.match_pos.col -= 1;
        } else if at(-1) == b'*' && at(0) == b'/' {
            if self.count > 0 {
                self.pos = self.match_pos;
            } else if self.pos.col > 1 && at(-2) == b'/' && self.pos.col <= self.comment_col {
                self.pos.col -= 2;
            } else if target.ignore_cend {
                return Step::Next;
            } else {
                return Step::Nothing;
            }
            return Step::Found(self.pos);
        }
        Step::Next
    }

    /// Count the quotes on the current line, deciding whether quoted text
    /// on it can be skipped at all.
    ///
    /// Braces inside quotes are ignored, but only when the line holds an
    /// even number of quotes — with an odd count there is no telling
    /// which half to ignore. A line ending in a backslash continues the
    /// string onto the next one, which is what rescues the odd case.
    /// Complicated, isn't it?
    ///
    /// # Safety
    /// As [`Walk::step_back`]. Only called with `do_quotes == -1`, which
    /// is also where the count starts: after N quotes it holds `N - 1`,
    /// so masking with 1 answers "the count was even".
    fn count_quotes(&mut self) {
        // A walk that never reaches the start position leaves
        // `at_start` at -1, i.e. *true*. Upstream.
        let mut at_start = self.do_quotes;
        let stop = self.pos.col as usize + usize::from(self.backwards);
        {
            let line = self.lines.line(self.pos.lnum);
            // Count the quotes, skipping \" and '"'. Watch out for "\\".
            let mut at = 0usize;
            while at < line.len() {
                if at == stop {
                    at_start = self.do_quotes & 1;
                }
                if line[at] == b'"'
                    && (at == 0 || line[at - 1] != b'\'' || byte_at(line, at + 1) != b'\'')
                {
                    self.do_quotes += 1;
                }
                if line[at] == b'\\' && byte_at(line, at + 1) != 0 {
                    at += 1;
                }
                at += 1;
            }
            self.do_quotes &= 1; // 1 with an even number of quotes

            if self.do_quotes != 0 {
                return;
            }
            // An uneven count: check this line and the previous one for a
            // trailing '\'.
            self.inquote = false;
            if line.last() == Some(&b'\\') {
                self.do_quotes = 1;
                if self.start_in_quotes.is_none() {
                    // Do we need to use at_start here?
                    self.inquote = true;
                    self.start_in_quotes = Some(true);
                } else if self.backwards {
                    self.inquote = true;
                }
            }
        }
        if self.pos.lnum <= 1 {
            return;
        }
        // The line above is a second read of the same cache, so the borrow
        // above has to be over -- which is the whole of what upstream's
        // "ml_get() keeps only one line; get linep back" was about.
        let continued = self.lines.line(self.pos.lnum - 1).last() == Some(&b'\\');
        if continued {
            self.do_quotes = 1;
            if self.start_in_quotes.is_none() {
                self.inquote = at_start != 0;
                if self.inquote {
                    self.start_in_quotes = Some(true);
                }
            } else if !self.backwards {
                self.inquote = true;
            }
        }
    }

    /// Skip over a single-quoted character constant: `'x'` or `'\x'`.
    ///
    /// Careful with a lone single quote, as in "jon's". Things like
    /// `'\233'` and `'\x3f'` are not skipped — there is never a brace in
    /// them. Answers whether the position moved.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    fn skip_char_constant(&mut self) -> bool {
        let col = self.pos.col;
        let line = self.lines.line(self.pos.lnum);
        let at = |off: c_int| at_col(line, col, off);
        if self.backwards {
            if self.pos.col > 1 {
                if at(-2) == b'\'' {
                    self.pos.col -= 2;
                    return true;
                }
                if at(-2) == b'\\' && self.pos.col > 2 && at(-3) == b'\'' {
                    self.pos.col -= 3;
                    return true;
                }
            }
        } else if at(1) != 0 {
            // Forward search.
            if at(1) == b'\\' && at(2) != 0 && at(3) == b'\'' {
                self.pos.col += 3;
                return true;
            }
            if at(2) == b'\'' {
                self.pos.col += 2;
                return true;
            }
        }
        false
    }

    /// Look at the character under `pos` and decide whether it is the
    /// match, keeping the "am I inside a string?" state up to date.
    fn match_char(&mut self, target: &Target, cpo_match: bool, cpo_bsl: bool) -> Step {
        let col = self.pos.col;
        let c = {
            let line = self.lines.line(self.pos.lnum);
            char_at(&line[(col as usize).min(line.len())..])
        };
        if c == NUL {
            // At the end of a line without a trailing backslash,
            // reset inquote.
            if col == 0 || at_col(self.lines.line(self.pos.lnum), col, -1) != b'\\' {
                self.inquote = false;
                self.start_in_quotes = Some(false);
            }
            return Step::Next;
        }
        if c == '"' as c_int {
            // A quote preceded by an odd number of backslashes is
            // ignored.
            if self.do_quotes != 0 {
                let line = self.lines.line(self.pos.lnum);
                let mut back = col - 1;
                while back >= 0 && byte_at(line, back as usize) == b'\\' {
                    back -= 1;
                }
                if ((col - 1 - back) & 1) == 0 {
                    self.inquote = !self.inquote;
                    self.start_in_quotes = Some(false);
                }
            }
            return Step::Next;
        }
        // Skipping a character constant does not apply when the quote
        // itself is what is being matched.
        if c == '\'' as c_int
            && !cpo_match
            && target.initc != '\'' as c_int
            && target.findc != '\'' as c_int
            && self.skip_char_constant()
        {
            return Step::Next;
        }

        // For Lisp skip over backslashed (), {} and [] — actually
        // over "#\(" and friends.
        if Buf::current().b_p_lisp != 0
            && has_char(c"(){}[]", c)
            && col > 1
            && check_prevcol(self.lines.line(self.pos.lnum), col, b'\\', None)
            && check_prevcol(self.lines.line(self.pos.lnum), col - 1, b'#', None)
        {
            return Step::Next;
        }

        // Check for a match outside of quotes, and inside of quotes
        // when the start position is inside quotes too.
        if (!self.inquote || self.start_in_quotes == Some(true))
            && (c == target.initc || c == target.findc)
        {
            let bslcnt = if cpo_bsl {
                0
            } else {
                backslash_count(self.lines.line(self.pos.lnum), col)
            };
            // Only accept a match when 'M' is in 'cpo', or when the
            // escaping is what it was at the start.
            if cpo_bsl || (bslcnt & 1) == target.match_escaped {
                if c == target.initc {
                    self.count += 1;
                } else {
                    if self.count == 0 {
                        return Step::Found(self.pos);
                    }
                    self.count -= 1;
                }
            }
        }
        Step::Next
    }
}

/// The body of [`findmatchlimit`], answering the position by value.
fn find_match(
    op: Option<&mut OpArg>,
    initc: c_int,
    flags: c_int,
    maxtravel: int64_t,
) -> Option<Pos> {
    let mut pos = Win::current().w_cursor;
    pos.coladd = 0;
    let mut lines = Lines::current();
    let lisp = Buf::current().b_p_lisp != 0; // engage Lisp-specific hacks ;)

    // vi compatible matching, and "don't recognise backslashes".
    let cpo_match = cpo_has(CpoFlag::MATCH);
    let cpo_bsl = cpo_has(CpoFlag::MATCHBSL);

    // Direction to search when initc is '/', '*' or '#'.
    let dir = if flags & FM_BACKWARD != 0 {
        BACKWARD
    } else if flags & FM_FORWARD != 0 {
        FORWARD
    } else {
        0
    };

    let plan = {
        let line = lines.line(pos.lnum);
        make_plan(op, initc, dir, &mut pos, line, cpo_match, cpo_bsl)
    };
    let mut target = match plan {
        Plan::Nothing => return None,
        Plan::Hash(hash_dir) => return find_hash_match(pos, hash_dir, initc),
        Plan::Walk(target) => target,
    };

    // This is just guessing: with 'rightleft' set, look for the
    // matching paren or brace in the other direction.
    if Win::current().w_onebuf_opt.wo_rl != 0 && has_char(c"()[]{}<>", target.initc) {
        target.backwards = !target.backwards;
    }

    let mut walk = Walk {
        pos,
        lines,
        backwards: target.backwards,
        lisp,
        comment_col: MAXCOL,
        lispcomm: false,
        traveled: 0,
        maxtravel,
        do_quotes: -1,
        inquote: false,
        start_in_quotes: None,
        count: 0,
        match_pos: Pos::default(),
    };

    // Backward search: does this line hold a single-line comment?
    if (walk.backwards && target.comment_dir != 0) || lisp {
        walk.comment_col = check_linecomment(walk.lines.line(walk.pos.lnum));
    }
    if lisp && walk.comment_col != MAXCOL && walk.pos.col > walk.comment_col {
        walk.lispcomm = true; // find the match inside this comment
    }

    while !got_int.get() {
        // Go to the next position. inc() and dec() would do, but they
        // are much slower.
        let moved = if walk.backwards {
            walk.step_back(target.comment_dir)
        } else {
            walk.step_forward()
        };
        if !moved {
            break;
        }

        // With FM_BLOCKSTOP, stop at a '{' or '}' in column 0. The line is
        // read only once the column and the flag both say so; the walk
        // steps a character at a time and this is per step.
        if walk.pos.col == 0 && flags & FM_BLOCKSTOP != 0 {
            let first = c_int::from(byte_at(walk.lines.line(walk.pos.lnum), 0));
            if first == '{' as c_int || first == '}' as c_int {
                if first == target.findc && walk.count == 0 {
                    return Some(walk.pos); // match!
                }
                break; // out of scope
            }
        }

        if target.comment_dir != 0 {
            match walk.comment_step(&target) {
                Step::Next => continue,
                Step::Found(pos) => return Some(pos),
                Step::Nothing => return None,
            }
        }

        // With smart matching ('cpoptions' without '%'), braces
        // inside quotes are ignored.
        if cpo_match {
            walk.do_quotes = 0;
        } else if walk.do_quotes == -1 {
            walk.count_quotes();
        }
        if walk.start_in_quotes.is_none() {
            walk.start_in_quotes = Some(false);
        }

        match walk.match_char(&target, cpo_match, cpo_bsl) {
            Step::Next => continue,
            Step::Found(pos) => return Some(pos),
            Step::Nothing => return None,
        }
    }

    if target.comment_dir == BACKWARD && walk.count > 0 {
        return Some(walk.match_pos);
    }
    None // never found it
}
