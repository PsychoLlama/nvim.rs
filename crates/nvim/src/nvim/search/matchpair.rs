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

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::types::{kFalse, kNone, kTrue};
use core::ffi::{c_char, c_int};
use core::ptr;

const FM_BACKWARD: c_int = super::FM_BACKWARD as c_int;
const FM_FORWARD: c_int = super::FM_FORWARD as c_int;
const FM_BLOCKSTOP: c_int = super::FM_BLOCKSTOP as c_int;
const FORWARD: c_int = super::FORWARD as c_int;
const BACKWARD: c_int = super::BACKWARD as c_int;

/// The position [`findmatchlimit`] answers. It is a static because every
/// caller takes the pointer rather than the value; the next call
/// overwrites it.
static match_at: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});

/// Find the match for the bracket under the cursor.
///
/// # Safety
/// `oap` must be null or valid; the current window and buffer must be
/// valid.
pub unsafe extern "C" fn findmatch(oap: *mut oparg_T, initc: c_int) -> *mut pos_T {
    unsafe { findmatchlimit(oap, initc, 0, 0) }
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
/// `oap` is used only to set `oap->motion_type` for the linewise `#if`
/// case; it may be null.
///
/// # Safety
/// `oap` must be null or valid; the current window and buffer must be
/// valid. The answer points at a static and is invalidated by the next
/// call.
pub unsafe extern "C" fn findmatchlimit(
    oap: *mut oparg_T,
    initc: c_int,
    flags: c_int,
    maxtravel: int64_t,
) -> *mut pos_T {
    unsafe {
        match find_match(oap, initc, flags, maxtravel) {
            Some(pos) => {
                match_at.set(pos);
                match_at.ptr()
            }
            None => ptr::null_mut(),
        }
    }
}

// ---------------------------------------------------------------------
// Deciding what to look for.
// ---------------------------------------------------------------------

/// Whether the character before `linep[col]` is `ch`, reporting the
/// column of that previous character through `prev`.
///
/// False when `col` is zero. Handles multi-byte characters.
///
/// # Safety
/// `linep` must be a NUL-terminated line and `col` a column in it.
unsafe fn check_prevcol(
    linep: *mut c_char,
    col: c_int,
    ch: c_int,
    prev: Option<&mut c_int>,
) -> bool {
    unsafe {
        let mut col = col - 1;
        if col > 0 {
            col -= utf_head_off(linep, linep.offset(col as isize));
        }
        if let Some(prev) = prev {
            *prev = col;
        }
        col >= 0 && *linep.offset(col as isize) as u8 as c_int == ch
    }
}

/// How many backslashes immediately precede `linep[col]`.
///
/// An odd number means the character there is escaped. `'cpoptions'` "M"
/// switches the whole idea off, and both callers check that first.
///
/// # Safety
/// As [`check_prevcol`].
unsafe fn backslash_count(linep: *mut c_char, col: c_int) -> c_int {
    unsafe {
        let mut count = 0;
        let mut col = col;
        while check_prevcol(linep, col, '\\' as c_int, Some(&mut col)) {
            count += 1;
        }
        count
    }
}

/// Look `*initc` up in `'matchpairs'`.
///
/// `'matchpairs'` is `"x:y,x:y"`. On a hit, `*findc` becomes the opposite
/// character and `*backwards` the direction to look in; with `switchit`
/// the roles are swapped, which is how `%` on a closing bracket comes to
/// look backwards for the opening one. Everything is left alone when the
/// character is not in the option.
///
/// # Safety
/// The current buffer must be valid.
unsafe fn find_mps_values(
    initc: &mut c_int,
    findc: &mut c_int,
    backwards: &mut bool,
    switchit: bool,
) {
    unsafe {
        let mut ptr = (*curbuf.get()).b_p_mps;
        while *ptr as c_int != NUL {
            // The opening half of this pair.
            if utf_ptr2char(ptr) == *initc {
                let other = utf_ptr2char(ptr.offset(utfc_ptr2len(ptr) as isize + 1));
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
            ptr = ptr.offset((utfc_ptr2len(ptr) + 1) as isize);
            if utf_ptr2char(ptr) == *initc {
                if switchit {
                    *findc = *initc;
                    *initc = utf_ptr2char(prev);
                    *backwards = false;
                } else {
                    *findc = utf_ptr2char(prev);
                    *backwards = true;
                }
                return;
            }
            ptr = ptr.offset(utfc_ptr2len(ptr) as isize);
            if *ptr as c_int == ',' as c_int {
                ptr = ptr.offset(1);
            }
        }
    }
}

/// Whether `ptr` begins with `word`.
///
/// # Safety
/// `ptr` must be NUL-terminated.
unsafe fn starts_with(ptr: *const c_char, word: &str) -> bool {
    unsafe { strncmp(ptr, word.as_ptr() as *const c_char, word.len() as size_t) == 0 }
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
/// # Safety
/// `pos` and `linep` must address the current buffer; `oap` must be null
/// or valid.
unsafe fn make_plan(
    oap: *mut oparg_T,
    initc: c_int,
    dir: c_int,
    pos: &mut pos_T,
    linep: *mut c_char,
    cpo_match: bool,
    cpo_bsl: bool,
) -> Plan {
    unsafe {
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
            find_mps_values(
                &mut target.initc,
                &mut target.findc,
                &mut target.backwards,
                true,
            );
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
                let ptr = skipwhite(linep);
                let col = pos.col as isize;
                let at = |off: isize| *linep.offset(col + off) as c_int;
                if *ptr as c_int == '#' as c_int && pos.col <= ptr.offset_from(linep) as colnr_T {
                    // Are we before or at #if, #else etc.?
                    let ptr = skipwhite(ptr.offset(1));
                    if starts_with(ptr, "if") || starts_with(ptr, "endif") || starts_with(ptr, "el")
                    {
                        hash_dir = 1;
                    }
                } else if at(0) == '/' as c_int {
                    // Are we on a comment?
                    if at(1) == '*' as c_int {
                        target.comment_dir = FORWARD;
                        target.backwards = false;
                        pos.col += 1;
                    } else if pos.col > 0 && at(-1) == '*' as c_int {
                        target.comment_dir = BACKWARD;
                        target.backwards = true;
                        pos.col -= 1;
                    }
                } else if at(0) == '*' as c_int {
                    if at(1) == '/' as c_int {
                        target.comment_dir = BACKWARD;
                        target.backwards = true;
                    } else if pos.col > 0 && at(-1) == '/' as c_int {
                        target.comment_dir = FORWARD;
                        target.backwards = false;
                    }
                }
            }

            // Not on a comment or on the # at the start of a line: look
            // for a brace anywhere on this line at or after the cursor.
            if hash_dir == 0 && target.comment_dir == 0 {
                // Beyond the end of the line, use its last character.
                if *linep.offset(pos.col as isize) as c_int == NUL && pos.col != 0 {
                    pos.col -= 1;
                }
                loop {
                    target.initc = utf_ptr2char(linep.offset(pos.col as isize));
                    if target.initc == NUL {
                        break;
                    }
                    find_mps_values(
                        &mut target.initc,
                        &mut target.findc,
                        &mut target.backwards,
                        false,
                    );
                    if target.findc != 0 {
                        break;
                    }
                    pos.col += utfc_ptr2len(linep.offset(pos.col as isize));
                }
                if target.findc == 0 {
                    // No brace in the line; maybe use "  #if" then.
                    if !cpo_match && *skipwhite(linep) as c_int == '#' as c_int {
                        hash_dir = 1;
                    } else {
                        return Plan::Nothing;
                    }
                } else if !cpo_bsl {
                    target.match_escaped = backslash_count(linep, pos.col) & 1;
                }
            }
        }

        if hash_dir == 0 {
            return Plan::Walk(target);
        }

        // Look for a matching #if, #else, #elif or #endif.
        if !oap.is_null() {
            (*oap).motion_type = kMTLineWise; // linewise for this case only
        }
        if initc != '#' as c_int {
            let ptr = skipwhite(skipwhite(linep).offset(1));
            hash_dir = if starts_with(ptr, "if") || starts_with(ptr, "el") {
                1
            } else if starts_with(ptr, "endif") {
                -1
            } else {
                return Plan::Nothing;
            };
        }
        Plan::Hash(hash_dir)
    }
}

/// Walk lines looking for the `#if`/`#else`/`#endif` that partners the
/// one the cursor is on.
///
/// # Safety
/// `pos` must address the current buffer.
unsafe fn find_hash_match(mut pos: pos_T, hash_dir: c_int, initc: c_int) -> Option<pos_T> {
    unsafe {
        let mut count = 0;
        pos.col = 0;
        while !got_int.get() {
            if hash_dir > 0 {
                if pos.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
            } else if pos.lnum == 1 {
                break;
            }
            pos.lnum += hash_dir;
            let linep = ml_get(pos.lnum);
            line_breakcheck(); // check for CTRL-C typed
            let ptr = skipwhite(linep);
            if *ptr as c_int != '#' as c_int {
                continue;
            }
            pos.col = ptr.offset_from(linep) as colnr_T;
            let ptr = skipwhite(ptr.offset(1));
            if hash_dir > 0 {
                if starts_with(ptr, "if") {
                    count += 1;
                } else if starts_with(ptr, "el") {
                    if count == 0 {
                        return Some(pos);
                    }
                } else if starts_with(ptr, "endif") {
                    if count == 0 {
                        return Some(pos);
                    }
                    count -= 1;
                }
            } else if starts_with(ptr, "if") {
                if count == 0 {
                    return Some(pos);
                }
                count -= 1;
            } else if initc == '#' as c_int && starts_with(ptr, "el") {
                if count == 0 {
                    return Some(pos);
                }
            } else if starts_with(ptr, "endif") {
                count += 1;
            }
        }
        None
    }
}

// ---------------------------------------------------------------------
// The walk itself.
// ---------------------------------------------------------------------

/// What looking at one position decided.
enum Step {
    /// Look at the next position.
    Next,
    /// The match is here.
    Found(pos_T),
    /// Give up: there is no match at all.
    Nothing,
}

/// Everything the walk carries from one position to the next.
struct Walk {
    pos: pos_T,
    /// The line `pos` is on. `ml_get` keeps only one line, so this is
    /// re-derived at every line boundary and after anything that may
    /// have released it.
    linep: *mut c_char,
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
    /// Whether the *start* position was inside quotes; `kNone` until the
    /// first line has been counted.
    start_in_quotes: TriState,
    /// Nesting depth, and where the innermost `/*` was found.
    count: c_int,
    match_pos: pos_T,
}

impl Walk {
    /// Step one character backwards, answering false at the start of the
    /// buffer or when the travel limit is reached.
    ///
    /// # Safety
    /// The current buffer must be the one `self.pos` addresses.
    unsafe fn step_back(&mut self, comment_dir: c_int) -> bool {
        unsafe {
            // The character to match is inside a comment; don't look
            // outside it.
            if self.lispcomm && self.pos.col < self.comment_col {
                return false;
            }
            if self.pos.col != 0 {
                self.pos.col -= 1;
                self.pos.col -= utf_head_off(self.linep, self.linep.offset(self.pos.col as isize));
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
            self.linep = ml_get(self.pos.lnum);
            self.pos.col = ml_get_len(self.pos.lnum); // pos.col on the trailing NUL
            self.do_quotes = -1;
            line_breakcheck();
            // Does this line hold a single-line comment?
            if comment_dir != 0 || self.lisp {
                self.comment_col = check_linecomment(self.linep);
            }
            if self.lisp && self.comment_col != MAXCOL {
                self.pos.col = self.comment_col; // skip the comment
            }
            true
        }
    }

    /// Step one character forwards, answering false at the end of the
    /// buffer or when the travel limit is reached.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    unsafe fn step_forward(&mut self) -> bool {
        unsafe {
            let at_end = *self.linep.offset(self.pos.col as isize) as c_int == NUL
                // For Lisp don't look for a match inside a comment.
                || (self.lisp && self.comment_col != MAXCOL && self.pos.col == self.comment_col);
            if !at_end {
                self.pos.col += utfc_ptr2len(self.linep.offset(self.pos.col as isize));
                return true;
            }
            // End of file, or the line is exhausted and the comment with
            // it — then don't look for a match out in the code.
            if self.pos.lnum == (*curbuf.get()).b_ml.ml_line_count || self.lispcomm {
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
            self.linep = ml_get(self.pos.lnum);
            self.pos.col = 0;
            self.do_quotes = -1;
            line_breakcheck();
            if self.lisp {
                self.comment_col = check_linecomment(self.linep); // in the new line
            }
            true
        }
    }

    /// Look at one position while hunting for the other end of a comment
    /// or of a raw string.
    ///
    /// Comments do not nest, and quotes inside them are ignored.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    unsafe fn comment_step(&mut self, target: &Target) -> Step {
        unsafe {
            let linep = self.linep;
            let col = self.pos.col as isize;
            let at = |off: isize| *linep.offset(col + off) as c_int;

            if target.comment_dir == FORWARD {
                if at(0) == '*' as c_int && at(1) == '/' as c_int {
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
                if at(-1) == 'R' as c_int
                    && at(0) == '"' as c_int
                    && !vim_strchr(linep.offset(col + 1), '(' as c_int).is_null()
                {
                    // A possible start of a raw string. Now that the
                    // delimiter is known, check whether it ends before
                    // where the search started, or before the previously
                    // found raw-string start.
                    let mut end = if self.count > 0 {
                        self.match_pos
                    } else {
                        (*curwin.get()).w_cursor
                    };
                    if !find_rawstring_end(linep, &raw mut self.pos, &raw mut end) {
                        self.count += 1;
                        self.match_pos = self.pos;
                        self.match_pos.col -= 1;
                    }
                    self.linep = ml_get(self.pos.lnum); // may have been released
                }
                return Step::Next;
            }
            if at(-1) == '/' as c_int
                && at(0) == '*' as c_int
                && (self.pos.col == 1 || at(-2) != '*' as c_int)
                && self.pos.col < self.comment_col
            {
                self.count += 1;
                self.match_pos = self.pos;
                self.match_pos.col -= 1;
            } else if at(-1) == '*' as c_int && at(0) == '/' as c_int {
                if self.count > 0 {
                    self.pos = self.match_pos;
                } else if self.pos.col > 1
                    && at(-2) == '/' as c_int
                    && self.pos.col <= self.comment_col
                {
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
    unsafe fn count_quotes(&mut self) {
        unsafe {
            // A walk that never reaches the start position leaves
            // `at_start` at -1, i.e. *true*. Upstream.
            let mut at_start = self.do_quotes;
            let stop = self
                .linep
                .offset(self.pos.col as isize + self.backwards as isize);
            // Count the quotes, skipping \" and '"'. Watch out for "\\".
            let mut ptr = self.linep;
            while *ptr as c_int != NUL {
                if ptr == stop {
                    at_start = self.do_quotes & 1;
                }
                if *ptr as c_int == '"' as c_int
                    && (ptr == self.linep
                        || *ptr.offset(-1) as c_int != '\'' as c_int
                        || *ptr.offset(1) as c_int != '\'' as c_int)
                {
                    self.do_quotes += 1;
                }
                if *ptr as c_int == '\\' as c_int && *ptr.offset(1) as c_int != NUL {
                    ptr = ptr.offset(1);
                }
                ptr = ptr.offset(1);
            }
            self.do_quotes &= 1; // 1 with an even number of quotes

            if self.do_quotes != 0 {
                return;
            }
            // An uneven count: check this line and the previous one for a
            // trailing '\'.
            self.inquote = false;
            if *ptr.offset(-1) as c_int == '\\' as c_int {
                self.do_quotes = 1;
                if self.start_in_quotes == kNone {
                    // Do we need to use at_start here?
                    self.inquote = true;
                    self.start_in_quotes = kTrue;
                } else if self.backwards {
                    self.inquote = true;
                }
            }
            if self.pos.lnum <= 1 {
                return;
            }
            let prev = ml_get(self.pos.lnum - 1);
            if *prev as c_int != NUL
                && *prev.offset(ml_get_len(self.pos.lnum - 1) as isize - 1) as c_int
                    == '\\' as c_int
            {
                self.do_quotes = 1;
                if self.start_in_quotes == kNone {
                    self.inquote = at_start != 0;
                    if self.inquote {
                        self.start_in_quotes = kTrue;
                    }
                } else if !self.backwards {
                    self.inquote = true;
                }
            }
            // ml_get() keeps only one line; get linep back.
            self.linep = ml_get(self.pos.lnum);
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
    unsafe fn skip_char_constant(&mut self) -> bool {
        unsafe {
            let linep = self.linep;
            let col = self.pos.col as isize;
            let at = |off: isize| *linep.offset(col + off) as c_int;
            if self.backwards {
                if self.pos.col > 1 {
                    if at(-2) == '\'' as c_int {
                        self.pos.col -= 2;
                        return true;
                    }
                    if at(-2) == '\\' as c_int && self.pos.col > 2 && at(-3) == '\'' as c_int {
                        self.pos.col -= 3;
                        return true;
                    }
                }
            } else if at(1) != NUL {
                // Forward search.
                if at(1) == '\\' as c_int && at(2) != NUL && at(3) == '\'' as c_int {
                    self.pos.col += 3;
                    return true;
                }
                if at(2) == '\'' as c_int {
                    self.pos.col += 2;
                    return true;
                }
            }
            false
        }
    }

    /// Look at the character under `pos` and decide whether it is the
    /// match, keeping the "am I inside a string?" state up to date.
    ///
    /// # Safety
    /// As [`Walk::step_back`].
    unsafe fn match_char(&mut self, target: &Target, cpo_match: bool, cpo_bsl: bool) -> Step {
        unsafe {
            let c = utf_ptr2char(self.linep.offset(self.pos.col as isize));
            if c == NUL {
                // At the end of a line without a trailing backslash,
                // reset inquote.
                if self.pos.col == 0
                    || *self.linep.offset(self.pos.col as isize - 1) as c_int != '\\' as c_int
                {
                    self.inquote = false;
                    self.start_in_quotes = kFalse;
                }
                return Step::Next;
            }
            if c == '"' as c_int {
                // A quote preceded by an odd number of backslashes is
                // ignored.
                if self.do_quotes != 0 {
                    let mut col = self.pos.col - 1;
                    while col >= 0 && *self.linep.offset(col as isize) as c_int == '\\' as c_int {
                        col -= 1;
                    }
                    if ((self.pos.col - 1 - col) & 1) == 0 {
                        self.inquote = !self.inquote;
                        self.start_in_quotes = kFalse;
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
            if (*curbuf.get()).b_p_lisp != 0
                && !vim_strchr(c"(){}[]".as_ptr(), c).is_null()
                && self.pos.col > 1
                && check_prevcol(self.linep, self.pos.col, '\\' as c_int, None)
                && check_prevcol(self.linep, self.pos.col - 1, '#' as c_int, None)
            {
                return Step::Next;
            }

            // Check for a match outside of quotes, and inside of quotes
            // when the start position is inside quotes too.
            if (!self.inquote || self.start_in_quotes == kTrue)
                && (c == target.initc || c == target.findc)
            {
                let bslcnt = if cpo_bsl {
                    0
                } else {
                    backslash_count(self.linep, self.pos.col)
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
}

/// The body of [`findmatchlimit`], answering the position by value.
///
/// # Safety
/// As [`findmatchlimit`].
unsafe fn find_match(
    oap: *mut oparg_T,
    initc: c_int,
    flags: c_int,
    maxtravel: int64_t,
) -> Option<pos_T> {
    unsafe {
        let mut pos = (*curwin.get()).w_cursor;
        pos.coladd = 0;
        let linep = ml_get(pos.lnum);
        let lisp = (*curbuf.get()).b_p_lisp != 0; // engage Lisp-specific hacks ;)

        // vi compatible matching, and "don't recognise backslashes".
        let cpo_match = !vim_strchr(p_cpo.get(), CPO_MATCH).is_null();
        let cpo_bsl = !vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null();

        // Direction to search when initc is '/', '*' or '#'.
        let dir = if flags & FM_BACKWARD != 0 {
            BACKWARD
        } else if flags & FM_FORWARD != 0 {
            FORWARD
        } else {
            0
        };

        let mut target = match make_plan(oap, initc, dir, &mut pos, linep, cpo_match, cpo_bsl) {
            Plan::Nothing => return None,
            Plan::Hash(hash_dir) => return find_hash_match(pos, hash_dir, initc),
            Plan::Walk(target) => target,
        };

        // This is just guessing: with 'rightleft' set, look for the
        // matching paren or brace in the other direction.
        if (*curwin.get()).w_onebuf_opt.wo_rl != 0
            && !vim_strchr(c"()[]{}<>".as_ptr(), target.initc).is_null()
        {
            target.backwards = !target.backwards;
        }

        let mut walk = Walk {
            pos,
            linep,
            backwards: target.backwards,
            lisp,
            comment_col: MAXCOL,
            lispcomm: false,
            traveled: 0,
            maxtravel,
            do_quotes: -1,
            inquote: false,
            start_in_quotes: kNone,
            count: 0,
            match_pos: pos_T::default(),
        };

        // Backward search: does this line hold a single-line comment?
        if (walk.backwards && target.comment_dir != 0) || lisp {
            walk.comment_col = check_linecomment(walk.linep);
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

            // With FM_BLOCKSTOP, stop at a '{' or '}' in column 0.
            if walk.pos.col == 0
                && flags & FM_BLOCKSTOP != 0
                && (*walk.linep as c_int == '{' as c_int || *walk.linep as c_int == '}' as c_int)
            {
                if *walk.linep as c_int == target.findc && walk.count == 0 {
                    return Some(walk.pos); // match!
                }
                break; // out of scope
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
            if walk.start_in_quotes == kNone {
                walk.start_in_quotes = kFalse;
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
}
