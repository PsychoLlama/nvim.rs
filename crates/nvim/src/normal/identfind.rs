//! Finding the identifier a command is about to act on: the word under
//! the cursor, the one at a given position, and the local declaration
//! `gd`/`gD` jumps to.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

/// The `mb_get_class` classes this file cares about: 0 is white space, 1 is
/// punctuation, 2 and up are the word classes (one per script).
const CLASS_WHITE: c_int = 0;
const CLASS_PUNCT: c_int = 1;
const CLASS_WORD: c_int = 2;

use crate::ops::Op;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::change::get_leader_len;
use crate::charset::{skipwhite, vim_iswordp};
use crate::cursor::get_cursor_line_ptr;
use crate::drawscreen::state::clear_cmdline;
use crate::fold::fold_open_cursor;
use crate::getchar::state::KeyTyped;
use crate::mark::setpcmark;
use crate::mbyte::{mb_get_class, utf_head_off, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{xfree, xmalloc};
use crate::message::e_noident;
use crate::message::state::msg_silent;
use crate::message::{emsg, messaging};
use crate::normal::{FIND_EVAL, FIND_IDENT, FIND_STRING, FM_FORWARD, clear_op_beep};
use crate::option::shortmess;
use crate::option::vars::{P_SCS, P_WS, fdo_flags, p_scs, p_ws};
use crate::options::kOptFdoFlagSearch;
use crate::os::cshim::{gettext, snprintf};
use crate::pos::clearpos;
use crate::regexp::RE_LAST;
use crate::search::{BACKWARD, FORWARD, SEARCH_START, findmatchlimit, reset_search_dir, searchit};
use crate::textobject::findpar;
use crate::types::{ColNr, LineNr, NUL, OpArg, OpType, Pos, ShmFlag, int64_t, size_t, uint8_t};
use core::ffi::{c_char, c_int, c_uint, c_void};

/// Whether this character continues a `FIND_EVAL` expression rather than
/// ending it -- `.`, `->` and a balanced `[...]` subscript.
///
/// `colp` is advanced past the second byte of `->`, and `bnp` is the depth of
/// unclosed brackets, counted from whichever end the walk started at.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string. `colp` must point at a writable
/// `int` the caller owns. `bnp` must point at a writable `int` the caller
/// owns.
pub(crate) unsafe fn find_is_eval_item(
    p: *const c_char,
    colp: *mut c_int,
    bnp: *mut c_int,
    dir: c_int,
) -> bool {
    let backward = dir == BACKWARD as c_int;
    // Walking backwards, a `]` opens a subscript and a `[` closes it.
    let opener = if backward { ']' } else { '[' } as c_int;
    let closer = if backward { '[' } else { ']' } as c_int;
    // SAFETY: `p` is in the line being scanned; `colp`/`bnp` are the caller's.
    let (c, col, bn) = unsafe { (*p as c_int, &mut *colp, &mut *bnp) };
    if c == opener {
        *bn += 1;
    }
    if *bn > 0 {
        if c == closer {
            *bn -= 1;
        }
        return true;
    }
    if c == '.' as c_int {
        return true;
    }
    // `->` read from either end: backwards the cursor is on the `>`,
    // forwards on the `-`.
    let (arrow_head, arrow_tail) = if backward { (0, -1) } else { (1, 0) };
    // SAFETY: both arrow bytes are in `p`'s line -- forwards `p` is not the
    // terminator, backwards the caller stopped short of column 0.
    let (head, tail) = unsafe { (*p.offset(arrow_head), *p.offset(arrow_tail)) };
    if head as c_int == '>' as c_int && tail as c_int == '-' as c_int {
        *col += dir;
        return true;
    }
    false
}

/// The identifier or string under the cursor. Answers its length and, through
/// `text`, a pointer into the line; `offset` gets the cursor's offset in it.
///
/// # Safety
///
/// `text` must point at a writable `*mut c_char` slot the caller owns for the
/// call. `offset` must point at a writable `int` the caller owns.
pub(crate) unsafe fn find_ident_under_cursor(
    text: *mut *mut c_char,
    find_type: c_int,
    offset: *mut c_int,
) -> size_t {
    let mut textcol: c_int = 0;
    let textcolp = if offset.is_null() {
        ptr::null_mut()
    } else {
        &raw mut textcol
    };
    let win = Win::current();
    let pos = Win::current().w_cursor;
    // SAFETY: `win` is live; `text`/`textcolp` have room for one value each.
    let len = unsafe { find_ident_at_pos(win, pos.lnum, pos.col, text, textcolp, find_type) };
    if !offset.is_null() {
        // SAFETY: `offset` is the caller's own out-parameter.
        unsafe { *offset = Win::current().w_cursor.col - textcol };
    }
    len
}

/// The buffer line `find_ident_at_pos` walks, addressed by byte column. Every
/// method's precondition is the constructor's: the pointer addresses a
/// NUL-terminated line, and the walk never steps past its terminator.
#[derive(Clone, Copy)]
struct ScanLine(*mut c_char);

impl ScanLine {
    /// The byte at `col`.
    fn at(self, col: c_int) -> c_int {
        // SAFETY: `col` is at most the terminator's own column.
        unsafe { *self.0.offset(col as isize) as c_int }
    }
    /// The address of the byte at `col`.
    fn ptr(self, col: c_int) -> *mut c_char {
        unsafe { self.0.offset(col as isize) }
    }
    /// The character class of the character starting at `col`.
    fn class(self, col: c_int) -> c_int {
        unsafe { mb_get_class(self.0.offset(col as isize)) }
    }
    /// The length of the character at `col`, its combining marks included.
    fn char_len(self, col: c_int) -> c_int {
        unsafe { utfc_ptr2len(self.0.offset(col as isize)) }
    }
    /// The column the character in front of `col` starts at.
    fn prev_col(self, col: c_int) -> c_int {
        // SAFETY: only asked past column 0, so `col - 1` is a byte of this line.
        unsafe { col - 1 - utf_head_off(self.0, self.0.offset(col as isize).offset(-1)) }
    }
    /// `find_is_eval_item` at `col`, advancing `col` and the bracket depth.
    fn is_eval_item(self, col: &mut c_int, bn: &mut c_int, dir: c_int) -> bool {
        // SAFETY: as `at`; `col`/`bn` are the walk's own, and a backwards walk
        // is only asked past column 0.
        unsafe { find_is_eval_item(self.0.offset(*col as isize), col, bn, dir) }
    }
}

/// The identifier or string at a given position.
///
/// Runs in at most two passes. The first accepts only a word character; the
/// second, which `FIND_STRING` asks for and which `FIND_IDENT` alone skips,
/// accepts anything that is not white space. Each pass scans forward from the
/// position for a character it will take, then backs up to that run's start.
///
/// # Safety
///
/// `text` must point at a writable `*mut c_char` slot the caller owns for the
/// call. `textcol` must point at a writable `int` the caller owns.
pub(crate) unsafe fn find_ident_at_pos(
    window: Win,
    lnum: LineNr,
    mut startcol: ColNr,
    text: *mut *mut c_char,
    textcol: *mut c_int,
    find_type: c_int,
) -> size_t {
    let eval = find_type & FIND_EVAL as c_int != 0;
    // SAFETY: `window` is a live window, so its buffer is live too.
    let mut line = ScanLine(unsafe { ml_get_buf(window.buffer(), lnum) });
    let mut col: c_int = 0;
    let mut this_class: c_int = 0;
    // Pass 0 wants a word character; pass 1 will take punctuation too.
    let mut pass = c_int::from(find_type & FIND_IDENT as c_int == 0);
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
        let mut bn = (line.at(col) == ']' as c_int) as c_int;
        if eval && line.at(col) == ']' as c_int {
            // A subscript belongs to the name in front of it, so pretend
            // the `]` is a word character.
            // SAFETY: a NUL-terminated literal.
            this_class = unsafe { mb_get_class(c"a".as_ptr()) };
        } else {
            this_class = line.class(col);
        }
        while col > 0 && this_class != CLASS_WHITE {
            let mut prevcol = line.prev_col(col);
            let prev_class = line.class(prevcol);
            if this_class != prev_class
                && (pass == 0 || prev_class == CLASS_WHITE || find_type & FIND_IDENT as c_int != 0)
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
        if find_type & FIND_STRING as c_int == 0 || this_class == CLASS_WORD {
            break;
        }
        pass += 1;
    }

    if line.at(col) == NUL || (pass == 0 && this_class != CLASS_WORD) {
        if find_type & FIND_STRING as c_int != 0 {
            emsg(gettext(c"E348: No string under cursor"));
        } else {
            emsg(gettext(e_noident));
        }
        return 0;
    }

    line = ScanLine(line.ptr(col));
    // SAFETY: `text` is the caller's own out-parameter.
    unsafe { *text = line.0 };
    if !textcol.is_null() {
        // SAFETY: `textcol` is the caller's own out-parameter.
        unsafe { *textcol = col };
    }
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
    col as size_t
}

/// `gd` and `gD`: jump to the local or global declaration of the identifier
/// under the cursor.
///
/// # Safety
///
/// `op` must point at a live `OpArg`, unaliased for the call.
pub(crate) unsafe fn nv_gd(op: *mut OpArg, nchar: c_int, thisblock: c_int) {
    let mut word: *mut c_char = ptr::null_mut();
    let out = &raw mut word;
    // SAFETY: `out` points at this frame's own `word`.
    let len = unsafe { find_ident_under_cursor(out, FIND_IDENT as c_int, ptr::null_mut()) };
    let locally = nchar == 'd' as c_int;
    // SAFETY: `word` is `len` bytes of the cursor's line.
    let found =
        len != 0 && unsafe { find_decl(word, len, locally, thisblock != 0, SEARCH_START as c_int) };
    if !found {
        // SAFETY: `op` is the caller's live operator.
        clear_op_beep(unsafe { Op::new(op) });
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as c_uint != 0
        && KeyTyped.get()
        // SAFETY: `op` is the caller's live operator.
        && unsafe { (*op).op_type } == OpType::Nop
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
///
/// # Safety
///
/// `line` must point at a NUL-terminated string.
pub(crate) unsafe fn is_ident(line: *const c_char, offset: c_int) -> bool {
    let mut incomment = false;
    // The quote that opened the string we are inside, or 0.
    let mut instring: c_int = 0;
    let mut prev: c_int = 0;
    let mut i = 0;
    while i < offset {
        // SAFETY: `line` is NUL-terminated and the walk stops at its NUL.
        let c = unsafe { *line.offset(i as isize) } as uint8_t as c_int;
        if c == NUL {
            break;
        }
        if instring != 0 {
            if prev != '\\' as c_int && c == instring {
                instring = 0;
            }
        } else if (c == '"' as c_int || c == '\'' as c_int) && !incomment {
            instring = c;
        } else if incomment {
            if prev == '*' as c_int && c == '/' as c_int {
                incomment = false;
            }
        } else if prev == '/' as c_int && c == '*' as c_int {
            incomment = true;
        } else if prev == '/' as c_int && c == '/' as c_int {
            // The rest of the line is a comment, so the offset is too.
            return false;
        }
        prev = c;
        i += 1;
    }
    !incomment && instring == 0
}

/// Search backwards for where `ptr` is declared: the first occurrence that is
/// not inside a comment or a string, above the cursor. `locally` limits the
/// search to the current `{}` block (`gd`); `thisblock` further refuses a
/// match whose block closes before the cursor.
///
/// # Safety
///
/// `word` must point at `len` bytes the caller owns, readable and writable,
/// unaliased for the call.
pub(crate) unsafe fn find_decl(
    word: *mut c_char,
    len: size_t,
    locally: bool,
    thisblock: bool,
    flags_arg: c_int,
) -> bool {
    let mut searchflags = flags_arg;
    // `\V` plus the word plus `\<`, `\>` and a terminator.
    let patsize = len.wrapping_add(7);
    // SAFETY: `xmalloc` answers `patsize` writable bytes and never null.
    let pat = unsafe { xmalloc(patsize) } as *mut c_char;
    debug_assert!(patsize <= c_int::MAX as size_t);
    // SAFETY: `word` is `len` bytes of a NUL-terminated line, and `pat` is
    // `patsize` writable bytes that `fmt`'s conversion fits in.
    let fmt = if unsafe { vim_iswordp(word) } {
        c"\\V\\<%.*s\\>".as_ptr()
    } else {
        c"\\V%.*s".as_ptr()
    };
    let patlen = unsafe { snprintf(pat, patsize, fmt, len as c_int, word) } as size_t;

    let old_pos = Win::current().w_cursor;
    let save_p_ws = p_ws();
    let save_p_scs = p_scs();
    // The search must not wrap round the file or guess at case.
    P_WS.set(false);
    P_SCS.set(false);

    // Where the enclosing block starts, which is as far back as a local
    // declaration may be.
    let par_pos;
    let mut incll = false;
    // SAFETY: `incll` is this frame's own out-parameter.
    let in_block =
        locally && unsafe { findpar(&raw mut incll, BACKWARD as c_int, 1, '{' as c_int, false) };
    if !in_block {
        setpcmark();
        Win::current().w_cursor.lnum = 1;
        par_pos = Win::current().w_cursor;
    } else {
        par_pos = Win::current().w_cursor;
        // Back up over the function's own header lines.
        while Win::current().w_cursor.lnum > 1 {
            // SAFETY: the cursor line is a NUL-terminated buffer line.
            if unsafe { *skipwhite(get_cursor_line_ptr()) } as c_int == NUL {
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
        let mut wp = Win::current();
        let pos = &raw mut wp.w_cursor;
        let (win, buf) = (Some(wp), Buf::current());
        let end = ptr::null_mut();
        let arg = ptr::null_mut();
        let opts = searchflags;
        let re = RE_LAST as c_int;
        // SAFETY: window and buffer are live; `pat` is `patlen` bytes.
        let hit = unsafe { searchit(win, buf, pos, end, FORWARD, pat, patlen, 1, opts, re, arg) };
        found = hit != 0;
        if Win::current().w_cursor.lnum >= old_pos.lnum {
            // Found it below the cursor, which is not a declaration of
            // what is under the cursor.
            found = false;
        }
        if thisblock && found {
            // Refuse a match whose enclosing block closes before the
            // cursor: it is a different scope.
            let travel = (old_pos.lnum - Win::current().w_cursor.lnum + 1) as int64_t;
            let brace = '}' as c_int;
            // SAFETY: the current window and buffer are live.
            let close = findmatchlimit(None, brace, FM_FORWARD as c_int, travel);
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
        // SAFETY: the cursor line is a NUL-terminated buffer line.
        let leader = unsafe { get_leader_len(get_cursor_line_ptr(), ptr::null_mut(), false, true) };
        if leader > 0 {
            // The whole line is a comment; skip past it.
            Win::current().w_cursor.lnum += 1;
            Win::current().w_cursor.col = 0;
            continue;
        }
        // SAFETY: as above.
        let valid = unsafe { is_ident(get_cursor_line_ptr(), Win::current().w_cursor.col) };
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
    // SAFETY: `pat` came from `xmalloc` above and is not used again.
    unsafe { xfree(pat as *mut c_void) };
    P_WS.set(save_p_ws);
    P_SCS.set(save_p_scs);
    found
}
