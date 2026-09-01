//! The identifier under the cursor, and the commands that look it up:
//! tags, `:help`, 'keywordprg', a declaration, a file name.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::ops::Op;
use crate::winlayer::{Buf, Win};
use core::ptr;

use crate::buffer::buf_hide;
use crate::change::get_leader_len;
use crate::charset::{skipwhite, vim_iswordp};
use crate::cmdhist::{add_to_history, init_history};
use crate::cursor::{check_cursor_lnum, get_cursor_line_ptr};
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::vim_strsave_fnameescape;
use crate::file_search::grab_file_name;
use crate::fold::fold_open_cursor;
use crate::keycodes::Ctrl_RSB;
use crate::main::{
    KeyTyped, clear_cmdline, curbuf, curwin, e_noident, fdo_flags, g_tag_at_cursor, msg_silent,
    no_smartcase, p_kp, p_scs, p_ws, restart_edit,
};
use crate::mapping::add_map;
use crate::mark::setpcmark;
use crate::mbyte::{mb_get_class, mb_prevptr, utf_head_off, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::message::{emsg, messaging};
use crate::normal::{
    CmdArg, DT_POP, ECMD_HIDE, ECMD_LAST, FIND_EVAL, FIND_IDENT, FIND_STRING, FM_FORWARD,
    HIST_SEARCH, POUND, VSE_NONE, check_clear_op_quit, check_text_or_curbuf_locked, clear_op,
    clear_op_beep, get_visual_text, normal_search, visual_active,
};
use crate::ops::clear_oparg;
use crate::option::{magic_isset, shortmess};
use crate::options::kOptFdoFlagSearch;
use crate::os::cshim::{gettext, snprintf};
use crate::pos::clearpos;
use crate::regexp::RE_LAST;
use crate::search::{BACKWARD, FORWARD, SEARCH_START, findmatchlimit, reset_search_dir, searchit};
use crate::state::MODE_TERMINAL;
use crate::strings::{vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::tag::do_tag;
use crate::textobject::findpar;
use crate::types::{
    NUL, OpType, ShmFlag, cmdarg_T, colnr_T, int64_t, linenr_T, oparg_T, pos_T, size_t, uint8_t,
};
use crate::undo::curbuf_is_changed;
use crate::window::check_can_set_curbuf_disabled;
use ::libc::strcpy;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

/// The `mb_get_class` classes this file cares about: 0 is white space, 1 is
/// punctuation, 2 and up are the word classes (one per script).
const CLASS_WHITE: c_int = 0;
const CLASS_PUNCT: c_int = 1;
const CLASS_WORD: c_int = 2;

/// Whether this character continues a `FIND_EVAL` expression rather than
/// ending it -- `.`, `->` and a balanced `[...]` subscript.
///
/// `colp` is advanced past the second byte of `->`, and `bnp` is the depth of
/// unclosed brackets, counted from whichever end the walk started at.
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
    let win = curwin.get();
    let pos = cur_win().w_cursor;
    // SAFETY: `win` is live; `text`/`textcolp` have room for one value each.
    let len =
        unsafe { find_ident_at_pos(Win::new(win), pos.lnum, pos.col, text, textcolp, find_type) };
    if !offset.is_null() {
        // SAFETY: `offset` is the caller's own out-parameter.
        unsafe { *offset = cur_win().w_cursor.col - textcol };
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
pub(crate) unsafe fn find_ident_at_pos(
    wp: Win,
    lnum: linenr_T,
    mut startcol: colnr_T,
    text: *mut *mut c_char,
    textcol: *mut c_int,
    find_type: c_int,
) -> size_t {
    let eval = find_type & FIND_EVAL as c_int != 0;
    // SAFETY: `wp` is a live window, so its buffer is live too.
    let mut line = ScanLine(unsafe { ml_get_buf(wp.w_buffer, lnum) });
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
pub(crate) unsafe fn nv_gd(oap: *mut oparg_T, nchar: c_int, thisblock: c_int) {
    let mut word: *mut c_char = ptr::null_mut();
    let out = &raw mut word;
    // SAFETY: `out` points at this frame's own `word`.
    let len = unsafe { find_ident_under_cursor(out, FIND_IDENT as c_int, ptr::null_mut()) };
    let locally = nchar == 'd' as c_int;
    // SAFETY: `word` is `len` bytes of the cursor's line.
    let found =
        len != 0 && unsafe { find_decl(word, len, locally, thisblock != 0, SEARCH_START as c_int) };
    if !found {
        // SAFETY: `oap` is the caller's live operator.
        clear_op_beep(unsafe { Op::new(oap) });
        return;
    }
    if fdo_flags.get() & kOptFdoFlagSearch as c_uint != 0
        && KeyTyped.get()
        // SAFETY: `oap` is the caller's live operator.
        && unsafe { (*oap).op_type } == OpType::Nop
    {
        // SAFETY: the editor's fold state is live.
        unsafe { fold_open_cursor() };
    }
    // The search left a "search hit" message that has nothing to say
    // here, unless 'shortmess' has already suppressed it.
    // SAFETY: reads the message state, which is live.
    if unsafe { messaging() } && msg_silent.get() == 0 && !shortmess(ShmFlag::SEARCHCOUNT) {
        clear_cmdline.set(true);
    }
}

/// Whether the byte at `offset` is ordinary code rather than inside a string
/// or a comment. A very rough C-shaped scan: it knows `"`, `'`, `/* */` and
/// `//`, and it only ever looks at the one line.
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

    let old_pos = cur_win().w_cursor;
    let save_p_ws = p_ws.get();
    let save_p_scs = p_scs.get();
    // The search must not wrap round the file or guess at case.
    p_ws.set(0);
    p_scs.set(0);

    // Where the enclosing block starts, which is as far back as a local
    // declaration may be.
    let mut par_pos;
    let mut incll = false;
    // SAFETY: `incll` is this frame's own out-parameter.
    let in_block =
        locally && unsafe { findpar(&raw mut incll, BACKWARD as c_int, 1, '{' as c_int, false) };
    if !in_block {
        setpcmark();
        cur_win().w_cursor.lnum = 1;
        par_pos = cur_win().w_cursor;
    } else {
        par_pos = cur_win().w_cursor;
        // Back up over the function's own header lines.
        while cur_win().w_cursor.lnum > 1 {
            // SAFETY: the cursor line is a NUL-terminated buffer line.
            if unsafe { *skipwhite(get_cursor_line_ptr()) } as c_int == NUL {
                break;
            }
            cur_win().w_cursor.lnum -= 1;
        }
    }
    cur_win().w_cursor.col = 0;

    // The last match that was inside a comment or a string, kept as the
    // answer of last resort.
    let mut found_pos = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut found;
    loop {
        let wp = curwin.get();
        // SAFETY: `wp` is the live window, so its cursor is live too.
        let pos = unsafe { &raw mut (*wp).w_cursor };
        // SAFETY: `curwin` and `curbuf` are the live window and buffer.
        let (win, buf) = unsafe { (Some(Win::new(wp)), Buf::current()) };
        let end = ptr::null_mut();
        let arg = ptr::null_mut();
        let opts = searchflags;
        let re = RE_LAST as c_int;
        // SAFETY: window and buffer are live; `pat` is `patlen` bytes.
        let hit = unsafe { searchit(win, buf, pos, end, FORWARD, pat, patlen, 1, opts, re, arg) };
        found = hit != 0;
        if cur_win().w_cursor.lnum >= old_pos.lnum {
            // Found it below the cursor, which is not a declaration of
            // what is under the cursor.
            found = false;
        }
        if thisblock && found {
            // Refuse a match whose enclosing block closes before the
            // cursor: it is a different scope.
            let travel = (old_pos.lnum - cur_win().w_cursor.lnum + 1) as int64_t;
            let no_oap = ptr::null_mut();
            let brace = '}' as c_int;
            // SAFETY: the current window and buffer are live.
            let close = unsafe { findmatchlimit(no_oap, brace, FM_FORWARD as c_int, travel) };
            if let Some(close) = close
                && close.lnum < old_pos.lnum
            {
                cur_win().w_cursor = close;
                continue;
            }
        }
        if !found {
            if found_pos.lnum != 0 {
                cur_win().w_cursor = found_pos;
                found = true;
            }
            break;
        }
        // SAFETY: the cursor line is a NUL-terminated buffer line.
        let leader = unsafe { get_leader_len(get_cursor_line_ptr(), ptr::null_mut(), false, true) };
        if leader > 0 {
            // The whole line is a comment; skip past it.
            cur_win().w_cursor.lnum += 1;
            cur_win().w_cursor.col = 0;
            continue;
        }
        // SAFETY: as above.
        let valid = unsafe { is_ident(get_cursor_line_ptr(), cur_win().w_cursor.col) };
        if !valid && found_pos.lnum != 0 {
            // Nothing better than what was already found.
            cur_win().w_cursor = found_pos;
            break;
        }
        if valid && !locally {
            break;
        }
        if valid && cur_win().w_cursor.lnum >= par_pos.lnum {
            // Past the start of the block: a local search is done, and
            // the earlier match wins if there was one.
            if found_pos.lnum != 0 {
                cur_win().w_cursor = found_pos;
            }
            break;
        }
        if valid {
            found_pos = cur_win().w_cursor;
        } else {
            clearpos(&mut found_pos);
        }
        // Having found one match, the next search must move.
        searchflags &= !(SEARCH_START as c_int);
    }

    if !found {
        cur_win().w_cursor = old_pos;
    } else {
        cur_win().w_set_curswant = true;
        reset_search_dir();
    }
    // SAFETY: `pat` came from `xmalloc` above and is not used again.
    unsafe { xfree(pat as *mut c_void) };
    p_ws.set(save_p_ws);
    p_scs.set(save_p_scs);
    found
}

/// Run one of the identifier commands from outside the command loop, with a
/// command argument built for it. `CTRL-W ]` and friends use this.
pub(crate) unsafe fn do_nv_ident(c1: c_int, c2: c_int) {
    // SAFETY: both structures are plain data, and both are filled in before
    // `nv_ident` reads them.
    let mut oa: oparg_T = unsafe { core::mem::zeroed() };
    let mut ca: cmdarg_T = unsafe { core::mem::zeroed() };
    unsafe { clear_oparg(&raw mut oa) };
    ca.oap = &raw mut oa;
    ca.cmdchar = c1;
    ca.nchar = c2;
    unsafe { nv_ident(&raw mut ca) };
}

/// The command line `nv_ident` builds: `size` writable bytes with the first
/// `len` in use, always NUL-terminated. Freed by hand -- `build_keywordprg_cmd`
/// hands it back already freed when there is nothing to look up.
struct CmdBuf {
    ptr: *mut c_char,
    size: size_t,
    len: size_t,
}

impl CmdBuf {
    /// An empty command of `size` bytes.
    fn new(size: size_t) -> Self {
        // SAFETY: `xmalloc` answers `size` writable bytes and never null.
        let ptr = unsafe { xmalloc(size) } as *mut c_char;
        // SAFETY: `size` is well over the one byte a terminator needs.
        unsafe { *ptr = NUL as c_char };
        Self { ptr, size, len: 0 }
    }
    /// The command so far, NUL-terminated.
    fn as_ptr(&self) -> *mut c_char {
        self.ptr
    }
    /// How many of its bytes are in use.
    fn used(&self) -> size_t {
        self.len
    }
    /// Where the next byte goes.
    fn tail(&self) -> *mut c_char {
        // SAFETY: `len` of the `size` bytes are in use, so this is in bounds.
        unsafe { self.ptr.add(self.len) }
    }
    /// Replace the command with `s`.
    fn set(&mut self, s: &CStr) {
        self.len = 0;
        self.append(s);
    }
    /// Append `s` as it stands.
    fn append(&mut self, s: &CStr) {
        // SAFETY: the buffer was sized for the whole command, `s` included.
        unsafe { strcpy(self.tail(), s.as_ptr()) };
        self.len += s.count_bytes() as size_t;
    }
    /// Append `s` through `snprintf`, which truncates rather than overruns.
    fn push(&mut self, s: &CStr) {
        self.push_str(c"%s", s.as_ptr());
    }
    /// Append `fmt` filled with one NUL-terminated string.
    fn push_str(&mut self, fmt: &CStr, arg: *const c_char) {
        // SAFETY: the tail has the `size - len` bytes `snprintf` is told of,
        // and `arg` matches the one `%s` in `fmt`.
        let wrote = unsafe { snprintf(self.tail(), self.size - self.len, fmt.as_ptr(), arg) };
        self.len += wrote as size_t;
    }
    /// Append `fmt` filled with one number.
    fn push_num(&mut self, fmt: &CStr, arg: int64_t) {
        // SAFETY: as `push_str`; `arg` matches the one `%ld` in `fmt`.
        let wrote = unsafe { snprintf(self.tail(), self.size - self.len, fmt.as_ptr(), arg) };
        self.len += wrote as size_t;
    }
    /// Resize around what is in the buffer plus `n` more bytes, and append
    /// them from `s`.
    fn append_grown(&mut self, s: *const c_char, n: size_t) {
        let size = self.len + n + 1;
        // SAFETY: it came from `xmalloc`, and `size` holds what is in it now
        // plus `s` and a terminator.
        self.ptr = unsafe { xrealloc(self.ptr as *mut c_void, size) } as *mut c_char;
        self.size = size;
        // SAFETY: the tail now has room for `s`'s `n` bytes and its NUL.
        unsafe { strcpy(self.tail(), s) };
        self.len += n;
    }
    /// Take the length from where an in-place append stopped.
    fn ends_at(&mut self, end: *mut c_char) {
        // SAFETY: `end` is inside the same allocation `self.ptr` addresses.
        self.len = unsafe { end.offset_from(self.ptr) } as size_t;
    }
    /// Release the buffer. The command must not be used again.
    fn free(&mut self) {
        // SAFETY: the buffer came from `xmalloc`/`xrealloc` above.
        unsafe { xfree(self.ptr as *mut c_void) };
        self.ptr = ptr::null_mut();
        self.size = 0;
        self.len = 0;
    }
}

/// Build the command `K` should run into `out`. Answers the length of the
/// identifier still to be appended, or 0 when there is nothing to look up --
/// in which case `out` has already been freed.
#[allow(clippy::too_many_arguments)]
unsafe fn build_keywordprg_cmd(
    cap: *mut cmdarg_T,
    kp: *mut c_char,
    kp_help: bool,
    kp_ex: bool,
    ptr_arg: &mut *mut c_char,
    mut n: size_t,
    out: &mut CmdBuf,
) -> size_t {
    // SAFETY (throughout): `cap` is live, and nothing below reaches back into it.
    let count0 = unsafe { (*cap).count0 };
    if kp_help {
        out.set(c"help! ");
        return n;
    }
    if kp_ex {
        // An Ex 'keywordprg' takes the word as its argument, after an
        // optional count.
        out.push_str(c"%s ", kp);
        if count0 != 0 {
            out.push_num(c"%ld ", count0 as int64_t);
        }
        return n;
    }

    // A shell 'keywordprg' runs in a terminal in a new tab. Leading
    // dashes would look like options to it.
    let mut word = *ptr_arg;
    // SAFETY: `word` walks the identifier, which is NUL-terminated.
    while unsafe { *word } as c_int == '-' as c_int && n > 0 {
        // SAFETY: the byte at `word` is a `-`, so the next one is in the line.
        word = unsafe { word.offset(1) };
        n -= 1;
    }
    if n == 0 {
        emsg(gettext(e_noident));
        out.free();
        *ptr_arg = word;
        return 0;
    }
    // `man` and `man -s` take the count as a section number, which goes
    // in front of the word rather than becoming a line range.
    // SAFETY: 'keywordprg' is a NUL-terminated option string.
    let isman = unsafe { cstr::bytes_at(kp) == b"man" };
    let isman_s = unsafe { cstr::bytes_at(kp) == b"man -s" };
    if count0 != 0 && !(isman || isman_s) {
        out.push_num(c".,.+%ld", (count0 - 1) as int64_t);
    }
    // SAFETY: a NUL-terminated literal command.
    let _ = unsafe { do_cmdline_cmd(c"tabnew".as_ptr()) };
    out.push(c"terminal ");
    if count0 == 0 && isman_s {
        // `man -s` with no section is just `man`.
        out.push(c"man ");
    } else {
        out.push_str(c"%s ", kp);
    }
    if count0 != 0 && (isman || isman_s) {
        out.push_num(c"%ld ", count0 as int64_t);
    }
    *ptr_arg = word;
    n
}

/// The characters that have to be backslash-escaped for the command being
/// built, given what it is.
fn ident_escapes(cmdchar: c_int, tag_cmd: bool) -> &'static CStr {
    if cmdchar == '*' as c_int {
        return if magic_isset() {
            c"/.*~[^$\\"
        } else {
            c"/^$\\"
        };
    }
    if cmdchar == '#' as c_int {
        return if magic_isset() {
            c"/?.*~[^$\\"
        } else {
            c"/?^$\\"
        };
    }
    if !tag_cmd {
        return c"\\|\"\n*?[";
    }
    // A help tag may contain any of these, so nothing is escaped.
    // SAFETY: 'filetype' is a NUL-terminated option string.
    if unsafe { cstr::bytes_at(cur_buf().b_p_ft) == b"help" } {
        c""
    } else {
        c"\\|\"\n["
    }
}

/// Copy the byte `src` points at to where `dst` points, advancing both.
/// # Safety
/// `*src` addresses a readable byte and `*dst` a writable one.
unsafe fn copy_byte(dst: &mut *mut c_char, src: &mut *mut c_char) {
    // SAFETY: the caller promises both bytes, and one past each is an
    // address a pointer may hold.
    unsafe { **dst = **src };
    (*dst, *src) = unsafe { (dst.offset(1), src.offset(1)) };
}

/// Copy `n` bytes of the identifier into the command being built, escaping
/// whatever the command cannot take literally.
/// # Safety
/// `dest` has room for twice `n` plus a terminator, which is what `nv_ident`
/// sized the buffer for, and `*src` is `n` bytes of a NUL-terminated line.
unsafe fn append_escaped(
    dest: *mut c_char,
    src: &mut *mut c_char,
    mut n: size_t,
    escapes: &CStr,
) -> *mut c_char {
    let mut out = dest;
    // The caller reads the source pointer back: `*` and `#` ask whether
    // the character *before* where this stopped was a word character, to
    // decide whether the search anchors at the end too.
    let p = src;
    while n > 0 {
        n -= 1;
        // SAFETY: `p` walks the identifier, which is NUL-terminated.
        let c = unsafe { **p } as uint8_t as c_int;
        // SAFETY: `escapes` is a NUL-terminated literal.
        if !unsafe { vim_strchr(escapes.as_ptr(), c) }.is_null() {
            // SAFETY: the caller promises the room.
            unsafe { *out = '\\' as c_char };
            out = unsafe { out.offset(1) };
        }
        // `utfc_ptr2len` answers 0 at a NUL, so this is `(size_t)-1`
        // there and the inner loop then runs until `n` is spent --
        // upstream's behaviour, and what stops a NUL ending the copy
        // early.
        // SAFETY: `p` points at a character of a NUL-terminated line.
        let trailing = (unsafe { utfc_ptr2len(*p) } - 1) as size_t;
        let mut i: size_t = 0;
        while i < trailing && n > 0 {
            // SAFETY: as above, and `out` still has the caller's room.
            unsafe { copy_byte(&mut out, p) };
            i += 1;
            n -= 1;
        }
        unsafe { copy_byte(&mut out, p) };
    }
    unsafe { *out = NUL as c_char };
    out
}

/// `*`, `#`, `K`, `]`, `CTRL-]` and their `g` forms: look up the identifier
/// under the cursor.
pub(crate) unsafe fn nv_ident(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY (throughout): `cap` is the caller's live command argument.
    let (typed, nchar) = unsafe { ((*cap).cmdchar, (*cap).nchar) };
    // The `g` forms carry the real command in `nchar`.
    let g_cmd = typed == 'g' as c_int;
    let mut cmdchar = if g_cmd { nchar } else { typed };
    if cmdchar == POUND {
        cmdchar = '#' as c_int;
    }

    let mut word: *mut c_char = ptr::null_mut();
    let mut n: size_t = 0;
    // Three of the commands take a Visual selection instead of the word
    // under the cursor.
    let mut visual_sel = false;
    if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
        // SAFETY: `cap` is live and `word`/`n` are this frame's own.
        if visual_active() && !unsafe { get_visual_text(cap, &raw mut word, &raw mut n) } {
            return;
        }
        visual_sel = !word.is_null();
        if check_clear_op_quit(ca.op()) {
            return;
        }
    }
    if word.is_null() {
        let mut ident_offset: c_int = 0;
        // `*` and `#` fall back to the string under the cursor when there
        // is no identifier there.
        let searchy = cmdchar == '*' as c_int || cmdchar == '#' as c_int;
        let find_type = FIND_IDENT as c_int | if searchy { FIND_STRING as c_int } else { 0 };
        // SAFETY: both out-parameters are this frame's own.
        n = unsafe { find_ident_under_cursor(&raw mut word, find_type, &raw mut ident_offset) };
        if n == 0 {
            clear_op(ca.op());
            return;
        }
    }

    // 'keywordprg', which decides what `K` does.
    // SAFETY: 'keywordprg' is a NUL-terminated option string.
    let kp = if unsafe { *cur_buf().b_p_kp } as c_int == NUL {
        p_kp.get()
    } else {
        cur_buf().b_p_kp
    };
    // SAFETY: `kp` is NUL-terminated, as are the literals.
    let kp_helpbang = unsafe { strequal(kp, c":help!".as_ptr()) };
    let kp_help = kp_helpbang
        || unsafe { *kp } as c_int == NUL
        || unsafe { strequal(kp, c":he".as_ptr()) }
        || unsafe { strequal(kp, c":help".as_ptr()) };
    if kp_help && !kp_helpbang {
        // SAFETY: `word` points into a NUL-terminated buffer line.
        if unsafe { *skipwhite(word) } as c_int == NUL {
            emsg(gettext(e_noident));
            return;
        }
    }
    // SAFETY: `kp` is NUL-terminated.
    let kp_ex = unsafe { *kp } as c_int == ':' as c_int;

    // Room for the command, the word with every byte escaped, and a
    // terminator.
    let kplen = unsafe { cstr::bytes_at(kp) }.len();
    let mut out = CmdBuf::new(n.wrapping_mul(2).wrapping_add(30).wrapping_add(kplen));

    // Whether the command being built is a tag lookup, which decides the
    // escaping below.
    let mut tag_cmd = false;
    match u8::try_from(cmdchar) {
        Ok(b'*' | b'#') => {
            // These become a search, so the cursor moves to the start of
            // the word first.
            // SAFETY: `word` points into the cursor's own line.
            setpcmark();
            let col = unsafe { word.offset_from(get_cursor_line_ptr()) } as colnr_T;
            cur_win().w_cursor.col = col;
            if !g_cmd && unsafe { vim_iswordp(word) } {
                // The plain forms anchor at a word boundary.
                out.set(c"\\<");
            }
            no_smartcase.set(true);
        }
        Ok(b'K') => {
            // SAFETY: all of these are live, and `word` is this frame's own.
            n = unsafe { build_keywordprg_cmd(cap, kp, kp_help, kp_ex, &mut word, n, &mut out) };
            if n == 0 {
                return;
            }
        }
        Ok(b']') => {
            tag_cmd = true;
            out.set(c"tselect ");
        }
        // CTRL-] and everything else: a plain tag jump.
        _ => {
            tag_cmd = true;
            let count0 = unsafe { (*cap).count0 };
            let cmd: &CStr = if cur_buf().b_help {
                c"help! "
            } else if g_cmd {
                c"tjump "
            } else if count0 == 0 {
                c"tag "
            } else {
                // A count picks which of the matching tags to jump to.
                out.push_num(c":%ldtag ", count0 as int64_t);
                c""
            };
            if !cmd.is_empty() {
                out.set(cmd);
            }
        }
    }

    if cmdchar == 'K' as c_int && kp_helpbang && !visual_sel {
        // `:help!` with no selection opens the help index rather than
        // looking anything up.
        out.set(c"help!");
    } else if cmdchar == 'K' as c_int && !kp_help {
        // A shell or Ex command takes the word quoted, not escaped
        // character by character.
        // SAFETY: `word` is `n` bytes of a buffer line.
        let owned = unsafe { xstrnsave(word, n) };
        // SAFETY: `owned` is a NUL-terminated copy of the word.
        let quoted = if kp_ex {
            unsafe { vim_strsave_fnameescape(owned, VSE_NONE as c_int) }
        } else {
            unsafe { vim_strsave_shellescape(owned, true, true) }
        };
        // SAFETY: `owned` and `quoted` came from the allocations above, and
        // `quoted` is NUL-terminated.
        unsafe { xfree(owned as *mut c_void) };
        let plen = unsafe { cstr::bytes_at(quoted) }.len();
        out.append_grown(quoted, plen);
        unsafe { xfree(quoted as *mut c_void) };
    } else {
        let escapes = ident_escapes(cmdchar, tag_cmd);
        // SAFETY: the buffer holds twice the word; `word` is `n` line bytes.
        let end = unsafe { append_escaped(out.tail(), &mut word, n, escapes) };
        out.ends_at(end);
    }

    if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
        // SAFETY: `word` points into the cursor's own line.
        if !g_cmd && unsafe { vim_iswordp(mb_prevptr(get_cursor_line_ptr(), word)) } {
            out.append(c"\\>");
        }
        // The search goes into the history as if it had been typed.
        init_history();
        // SAFETY: the command holds `used()` bytes.
        let entry = unsafe { core::slice::from_raw_parts(out.as_ptr() as *const u8, out.used()) };
        add_to_history(HIST_SEARCH as c_int, entry, true, NUL as u8);
        let star = cmdchar == '*' as c_int;
        let dir = if star { '/' as c_int } else { '?' as c_int };
        let cmd = out.as_ptr();
        let used = out.used();
        unsafe { normal_search(cap, dir, cmd, used, 0, ptr::null_mut()) };
    } else {
        // `taglist()` and friends need to know the tag came from under
        // the cursor rather than from a command line.
        g_tag_at_cursor.set(true);
        let _ = unsafe { do_cmdline_cmd(out.as_ptr()) };
        g_tag_at_cursor.set(false);
        if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
            // The terminal 'keywordprg' opened above: let <Esc> close it.
            restart_edit.set('i' as c_int);
            let lhs = c"<esc>".as_ptr() as *mut c_char;
            let rhs = c"<Cmd>bdelete!<CR>".as_ptr() as *mut c_char;
            // SAFETY: both sides are NUL-terminated literals.
            unsafe { add_map(lhs, rhs, MODE_TERMINAL, true) };
        }
    }
    out.free();
}

/// `CTRL-T`: back up the tag stack.
pub(crate) unsafe fn nv_tagpop(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY (throughout): `cap` is the caller's live command argument.
    if check_clear_op_quit(ca.op()) {
        return;
    }
    let none = c"".as_ptr() as *mut c_char;
    // SAFETY: `cap` is live and `none` is an empty NUL-terminated literal.
    unsafe { do_tag(none, DT_POP as c_int, (*cap).count1, 0, true) };
}

/// `gf`, `gF` and `[f`: edit the file named under the cursor.
pub(crate) unsafe fn nv_gotofile(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    let mut ca = unsafe { CmdArg::new(cap) };
    // SAFETY (throughout): `cap` is the caller's live command argument, and
    // the current window and buffer are live.
    if unsafe { check_text_or_curbuf_locked((*cap).oap) } || !check_can_set_curbuf_disabled() {
        return;
    }
    // `gF` also takes a line number off the end of the name.
    let mut lnum: linenr_T = -1;
    // SAFETY: `lnum` is this frame's own out-parameter.
    let name = unsafe { grab_file_name((*cap).count1, &raw mut lnum) };
    if name.is_null() {
        clear_op(ca.op());
        return;
    }
    // Leaving the only window on a changed buffer that cannot be hidden
    // means writing it first.
    let must_write =
        curbuf_is_changed() && cur_buf().b_nwindows <= 1 && !unsafe { buf_hide(curbuf.get()) };
    if must_write {
        let _ = unsafe { autowrite(curbuf.get(), false) };
    }
    setpcmark();
    let hidden = unsafe { buf_hide(curbuf.get()) };
    let hide = if hidden { ECMD_HIDE as c_int } else { 0 };
    let last = ECMD_LAST as linenr_T;
    let win = curwin.get();
    // SAFETY: `name` is a NUL-terminated file name.
    let opened = unsafe { do_ecmd(0, name, ptr::null_mut(), ptr::null_mut(), last, hide, win) };
    if opened.is_ok() && unsafe { (*cap).nchar } == 'F' as c_int && lnum >= 0 {
        cur_win().w_cursor.lnum = lnum;
        check_cursor_lnum(unsafe { Win::current() });
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    }
    // SAFETY: `name` came from `grab_file_name`.
    unsafe { xfree(name as *mut c_void) };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
