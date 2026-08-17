//! The identifier under the cursor, and the commands that look it up:
//! tags, `:help`, 'keywordprg', a declaration, a file name.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ptr;

use crate::buffer::buf_hide;
use crate::change::get_leader_len;
use crate::charset::{skipwhite, vim_iswordp};
use crate::cmdhist::{add_to_history, init_history};
use crate::cursor::{check_cursor_lnum, get_cursor_line_ptr};
use crate::edit::beginline;
use crate::ex_cmds::do_ecmd;
use crate::ex_cmds2::autowrite;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::vim_strsave_fnameescape;
use crate::file_search::grab_file_name;
use crate::fold::foldOpenCursor;
use crate::keycodes::Ctrl_RSB;
use crate::main::{
    KeyTyped, VIsual_active, clear_cmdline, curbuf, curwin, e_noident, fdo_flags, g_tag_at_cursor,
    msg_silent, no_smartcase, p_kp, p_scs, p_ws, restart_edit,
};
use crate::mapping::add_map;
use crate::mark::setpcmark;
use crate::mbyte::{mb_get_class, mb_prevptr, utf_head_off, utfc_ptr2len};
use crate::memline::ml_get_buf;
use crate::memory::{strequal, xfree, xmalloc, xrealloc};
use crate::message::{emsg, messaging};
use crate::normal::{
    BL_FIX, BL_SOL, DT_POP, ECMD_HIDE, ECMD_LAST, FIND_EVAL, FIND_IDENT, FIND_STRING, FM_FORWARD,
    HIST_SEARCH, NUL, OK, POUND, SHM_SEARCHCOUNT, VSE_NONE, check_text_or_curbuf_locked,
    checkclearopq, clearop, clearopbeep, false_0, get_visual_text, normal_search, true_0,
};
use crate::ops::clear_oparg;
use crate::option::{magic_isset, shortmess};
use crate::options::kOptFdoFlagSearch;
use crate::os::libc::{gettext, snprintf, strcmp, strcpy, strlen};
use crate::pos::clearpos;
use crate::regexp::RE_LAST;
use crate::search::{BACKWARD, FORWARD, SEARCH_START, findmatchlimit, reset_search_dir, searchit};
use crate::state::MODE_TERMINAL;
use crate::strings::{vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::tag::do_tag;
use crate::textobject::findpar;
use crate::types::{
    OP_NOP, cmdarg_T, colnr_T, int64_t, linenr_T, oparg_T, pos_T, size_t, uint8_t, win_T,
};
use crate::undo::curbufIsChanged;
use crate::window::check_can_set_curbuf_disabled;
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

/// The character classes `mb_get_class` answers with that this file cares
/// about: 0 is white space, 1 is punctuation, 2 and up are the word classes
/// (one per script).
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
    // SAFETY: `p` points into the line being scanned, and `colp`/`bnp` are
    // the caller's own counters.
    unsafe {
        let backward = dir == BACKWARD as c_int;
        // Walking backwards, a `]` opens a subscript and a `[` closes it.
        let opener = if backward { ']' } else { '[' } as c_int;
        let closer = if backward { '[' } else { ']' } as c_int;
        if *p as c_int == opener {
            *bnp += 1;
        }
        if *bnp > 0 {
            if *p as c_int == closer {
                *bnp -= 1;
            }
            return true;
        }
        if *p as c_int == '.' as c_int {
            return true;
        }
        // `->` read from either end: backwards the cursor is on the `>`,
        // forwards on the `-`.
        let (arrow_head, arrow_tail) = if backward { (0, -1) } else { (1, 0) };
        if *p.offset(arrow_head) as c_int == '>' as c_int
            && *p.offset(arrow_tail) as c_int == '-' as c_int
        {
            *colp += dir;
            return true;
        }
        false
    }
}

/// The identifier or string under the cursor. Answers its length and, through
/// `text`, a pointer into the buffer line; `offset` receives how far into it
/// the cursor was.
pub unsafe fn find_ident_under_cursor(
    text: *mut *mut c_char,
    find_type: c_int,
    offset: *mut c_int,
) -> size_t {
    // SAFETY: `text` and `offset` are the caller's own out-parameters.
    unsafe {
        let mut textcol: c_int = 0;
        let len = find_ident_at_pos(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.col,
            text,
            if offset.is_null() {
                ptr::null_mut()
            } else {
                &raw mut textcol
            },
            find_type,
        );
        if !offset.is_null() {
            *offset = (*curwin.get()).w_cursor.col - textcol;
        }
        len
    }
}

/// The identifier or string at a given position.
///
/// Runs in at most two passes. The first accepts only a word character; the
/// second, which `FIND_STRING` asks for and which `FIND_IDENT` alone skips,
/// accepts anything that is not white space. Each pass scans forward from the
/// position for a character it will take, then backs up to that run's start.
pub unsafe fn find_ident_at_pos(
    wp: *mut win_T,
    lnum: linenr_T,
    mut startcol: colnr_T,
    text: *mut *mut c_char,
    textcol: *mut c_int,
    find_type: c_int,
) -> size_t {
    // SAFETY: `wp` is a live window and `text`/`textcol` are the caller's own
    // out-parameters.
    unsafe {
        let eval = find_type & FIND_EVAL as c_int != 0;
        let mut line = ml_get_buf((*wp).w_buffer, lnum);
        let mut col: c_int = 0;
        let mut this_class: c_int = 0;
        // Pass 0 wants a word character; pass 1 will take punctuation too.
        let mut pass = if find_type & FIND_IDENT as c_int != 0 {
            0
        } else {
            1
        };
        while pass < 2 {
            col = startcol;
            while *line.offset(col as isize) as c_int != NUL {
                // A `]` ends an expression, and is where the backwards walk
                // has to start from.
                if eval && *line.offset(col as isize) as c_int == ']' as c_int {
                    break;
                }
                this_class = mb_get_class(line.offset(col as isize));
                if this_class != CLASS_WHITE && (pass == 1 || this_class != CLASS_PUNCT) {
                    break;
                }
                col += utfc_ptr2len(line.offset(col as isize));
            }
            // The bracket depth the backwards walk starts with.
            let mut bn = (*line.offset(col as isize) as c_int == ']' as c_int) as c_int;
            if eval && *line.offset(col as isize) as c_int == ']' as c_int {
                // A subscript belongs to the name in front of it, so pretend
                // the `]` is a word character.
                this_class = mb_get_class(c"a".as_ptr());
            } else {
                this_class = mb_get_class(line.offset(col as isize));
            }
            while col > 0 && this_class != CLASS_WHITE {
                let mut prevcol =
                    col - 1 - utf_head_off(line, line.offset(col as isize).offset(-1));
                let prev_class = mb_get_class(line.offset(prevcol as isize));
                if this_class != prev_class
                    && (pass == 0
                        || prev_class == CLASS_WHITE
                        || find_type & FIND_IDENT as c_int != 0)
                    && (!eval
                        || prevcol == 0
                        || !find_is_eval_item(
                            line.offset(prevcol as isize),
                            &raw mut prevcol,
                            &raw mut bn,
                            BACKWARD as c_int,
                        ))
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

        if *line.offset(col as isize) as c_int == NUL || (pass == 0 && this_class != CLASS_WORD) {
            if find_type & FIND_STRING as c_int != 0 {
                emsg(gettext(c"E348: No string under cursor".as_ptr()));
            } else {
                emsg(gettext(&raw const e_noident as *const c_char));
            }
            return 0;
        }

        line = line.offset(col as isize);
        *text = line;
        if !textcol.is_null() {
            *textcol = col;
        }
        // Now walk forward to the run's end. `startcol` becomes the cursor's
        // offset within the run, which is how far the `FIND_EVAL` walk is
        // allowed to keep taking subscripts.
        let mut bn = 0;
        startcol -= col;
        col = 0;
        this_class = mb_get_class(line);
        while *line.offset(col as isize) as c_int != NUL
            && (if pass == 0 {
                mb_get_class(line.offset(col as isize)) == this_class
            } else {
                mb_get_class(line.offset(col as isize)) != CLASS_WHITE
            } || (eval
                && col <= startcol
                && find_is_eval_item(
                    line.offset(col as isize),
                    &raw mut col,
                    &raw mut bn,
                    FORWARD as c_int,
                )))
        {
            col += utfc_ptr2len(line.offset(col as isize));
        }
        debug_assert!(col >= 0);
        col as size_t
    }
}

/// `gd` and `gD`: jump to the local or global declaration of the identifier
/// under the cursor.
pub(crate) unsafe fn nv_gd(oap: *mut oparg_T, nchar: c_int, thisblock: c_int) {
    // SAFETY: `oap` is the caller's live operator.
    unsafe {
        let mut word: *mut c_char = ptr::null_mut();
        let len = find_ident_under_cursor(&raw mut word, FIND_IDENT as c_int, ptr::null_mut());
        if len == 0
            || !find_decl(
                word,
                len,
                nchar == 'd' as c_int,
                thisblock != 0,
                SEARCH_START as c_int,
            )
        {
            clearopbeep(oap);
            return;
        }
        if fdo_flags.get() & kOptFdoFlagSearch as c_uint != 0
            && KeyTyped.get()
            && (*oap).op_type == OP_NOP
        {
            foldOpenCursor();
        }
        // The search left a "search hit" message that has nothing to say
        // here, unless 'shortmess' has already suppressed it.
        if messaging() && msg_silent.get() == 0 && !shortmess(SHM_SEARCHCOUNT as c_int) {
            clear_cmdline.set(true);
        }
    }
}

/// Whether the byte at `offset` is ordinary code rather than inside a string
/// or a comment.
///
/// A very rough C-shaped scan: it knows `"`, `'`, `/* */` and `//`, and it
/// only ever looks at the one line.
pub(crate) unsafe fn is_ident(line: *const c_char, offset: c_int) -> bool {
    // SAFETY: `line` is a NUL-terminated buffer line.
    unsafe {
        let mut incomment = false;
        // The quote that opened the string we are inside, or 0.
        let mut instring: c_int = 0;
        let mut prev: c_int = 0;
        let mut i = 0;
        while i < offset && *line.offset(i as isize) as c_int != NUL {
            let c = *line.offset(i as isize) as uint8_t as c_int;
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
}

/// Search backwards for where `ptr` is declared: the first occurrence that is
/// not inside a comment or a string, above the cursor.
///
/// `locally` limits the search to the current `{}` block (`gd`); `thisblock`
/// further refuses a match whose block closes before the cursor.
pub unsafe fn find_decl(
    word: *mut c_char,
    len: size_t,
    locally: bool,
    thisblock: bool,
    flags_arg: c_int,
) -> bool {
    // SAFETY: `word` is `len` bytes of a buffer line.
    unsafe {
        let mut searchflags = flags_arg;
        // `\V` plus the word plus `\<`, `\>` and a terminator.
        let patsize = len.wrapping_add(7);
        let pat = xmalloc(patsize) as *mut c_char;
        debug_assert!(patsize <= c_int::MAX as size_t);
        let patlen = snprintf(
            pat,
            patsize,
            if vim_iswordp(word) {
                c"\\V\\<%.*s\\>".as_ptr()
            } else {
                c"\\V%.*s".as_ptr()
            },
            len as c_int,
            word,
        ) as size_t;

        let old_pos = (*curwin.get()).w_cursor;
        let save_p_ws = p_ws.get();
        let save_p_scs = p_scs.get();
        // The search must not wrap round the file or guess at case.
        p_ws.set(false_0);
        p_scs.set(false_0);

        // Where the enclosing block starts, which is as far back as a local
        // declaration may be.
        let mut par_pos;
        let mut incll = false;
        if !locally || !findpar(&raw mut incll, BACKWARD as c_int, 1, '{' as c_int, false) {
            setpcmark();
            (*curwin.get()).w_cursor.lnum = 1;
            par_pos = (*curwin.get()).w_cursor;
        } else {
            par_pos = (*curwin.get()).w_cursor;
            // Back up over the function's own header lines.
            while (*curwin.get()).w_cursor.lnum > 1
                && *skipwhite(get_cursor_line_ptr()) as c_int != NUL
            {
                (*curwin.get()).w_cursor.lnum -= 1;
            }
        }
        (*curwin.get()).w_cursor.col = 0;

        // The last match that was inside a comment or a string, kept as the
        // answer of last resort.
        let mut found_pos = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut found;
        loop {
            found = searchit(
                curwin.get(),
                curbuf.get(),
                &raw mut (*curwin.get()).w_cursor,
                ptr::null_mut(),
                FORWARD,
                pat,
                patlen,
                1,
                searchflags,
                RE_LAST as c_int,
                ptr::null_mut(),
            ) != 0;
            if (*curwin.get()).w_cursor.lnum >= old_pos.lnum {
                // Found it below the cursor, which is not a declaration of
                // what is under the cursor.
                found = false;
            }
            if thisblock && found {
                // Refuse a match whose enclosing block closes before the
                // cursor: it is a different scope.
                let maxtravel = (old_pos.lnum - (*curwin.get()).w_cursor.lnum + 1) as int64_t;
                let close = findmatchlimit(
                    ptr::null_mut(),
                    '}' as c_int,
                    FM_FORWARD as c_int,
                    maxtravel,
                );
                if !close.is_null() && (*close).lnum < old_pos.lnum {
                    (*curwin.get()).w_cursor = *close;
                    continue;
                }
            }
            if !found {
                if found_pos.lnum != 0 {
                    (*curwin.get()).w_cursor = found_pos;
                    found = true;
                }
                break;
            }
            if get_leader_len(get_cursor_line_ptr(), ptr::null_mut(), false, true) > 0 {
                // The whole line is a comment; skip past it.
                (*curwin.get()).w_cursor.lnum += 1;
                (*curwin.get()).w_cursor.col = 0;
                continue;
            }
            let valid = is_ident(get_cursor_line_ptr(), (*curwin.get()).w_cursor.col);
            if !valid && found_pos.lnum != 0 {
                // Nothing better than what was already found.
                (*curwin.get()).w_cursor = found_pos;
                break;
            }
            if valid && !locally {
                break;
            }
            if valid && (*curwin.get()).w_cursor.lnum >= par_pos.lnum {
                // Past the start of the block: a local search is done, and
                // the earlier match wins if there was one.
                if found_pos.lnum != 0 {
                    (*curwin.get()).w_cursor = found_pos;
                }
                break;
            }
            if valid {
                found_pos = (*curwin.get()).w_cursor;
            } else {
                clearpos(&mut found_pos);
            }
            // Having found one match, the next search must move.
            searchflags &= !(SEARCH_START as c_int);
        }

        if !found {
            (*curwin.get()).w_cursor = old_pos;
        } else {
            (*curwin.get()).w_set_curswant = true_0;
            reset_search_dir();
        }
        xfree(pat as *mut c_void);
        p_ws.set(save_p_ws);
        p_scs.set(save_p_scs);
        found
    }
}

/// Run one of the identifier commands from outside the command loop, with a
/// command argument built for the occasion. `CTRL-W ]` and friends use this.
pub unsafe fn do_nv_ident(c1: c_int, c2: c_int) {
    // SAFETY: both structures are plain data and are filled before use.
    unsafe {
        let mut oa: oparg_T = core::mem::zeroed();
        let mut ca: cmdarg_T = core::mem::zeroed();
        clear_oparg(&raw mut oa);
        ca.oap = &raw mut oa;
        ca.cmdchar = c1;
        ca.nchar = c2;
        nv_ident(&raw mut ca);
    }
}

/// Build the command `K` should run into `buf`.
///
/// Answers the length of the identifier still to be appended, or 0 when there
/// is nothing to look up -- in which case `buf` has already been freed.
#[allow(clippy::too_many_arguments)]
unsafe fn nv_K_getcmd(
    cap: *mut cmdarg_T,
    kp: *mut c_char,
    kp_help: bool,
    kp_ex: bool,
    ptr_arg: *mut *mut c_char,
    mut n: size_t,
    buf: *mut c_char,
    bufsize: size_t,
    buflen: *mut size_t,
) -> size_t {
    // SAFETY: `buf` is `bufsize` writable bytes and the rest are the caller's
    // own values.
    unsafe {
        if kp_help {
            strcpy(buf, c"help! ".as_ptr() as *mut c_char);
            *buflen = c"help! ".count_bytes() as size_t;
            return n;
        }
        if kp_ex {
            // An Ex 'keywordprg' takes the word as its argument, after an
            // optional count.
            *buflen = snprintf(buf, bufsize, c"%s ".as_ptr(), kp) as size_t;
            if (*cap).count0 != 0 {
                *buflen += snprintf(
                    buf.add(*buflen),
                    bufsize - *buflen,
                    c"%ld ".as_ptr(),
                    (*cap).count0 as int64_t,
                ) as size_t;
            }
            return n;
        }

        // A shell 'keywordprg' runs in a terminal in a new tab. Leading
        // dashes would look like options to it.
        let mut word = *ptr_arg;
        while *word as c_int == '-' as c_int && n > 0 {
            word = word.offset(1);
            n -= 1;
        }
        if n == 0 {
            emsg(gettext(&raw const e_noident as *const c_char));
            xfree(buf as *mut c_void);
            *ptr_arg = word;
            return 0;
        }
        // `man` and `man -s` take the count as a section number, which goes
        // in front of the word rather than becoming a line range.
        let isman = strcmp(kp, c"man".as_ptr()) == 0;
        let isman_s = strcmp(kp, c"man -s".as_ptr()) == 0;
        if (*cap).count0 != 0 && !(isman || isman_s) {
            *buflen = snprintf(
                buf,
                bufsize,
                c".,.+%ld".as_ptr(),
                ((*cap).count0 - 1) as int64_t,
            ) as size_t;
        }
        do_cmdline_cmd(c"tabnew".as_ptr());
        *buflen += snprintf(buf.add(*buflen), bufsize - *buflen, c"terminal ".as_ptr()) as size_t;
        if (*cap).count0 == 0 && isman_s {
            // `man -s` with no section is just `man`.
            *buflen += snprintf(buf.add(*buflen), bufsize - *buflen, c"man ".as_ptr()) as size_t;
        } else {
            *buflen += snprintf(buf.add(*buflen), bufsize - *buflen, c"%s ".as_ptr(), kp) as size_t;
        }
        if (*cap).count0 != 0 && (isman || isman_s) {
            *buflen += snprintf(
                buf.add(*buflen),
                bufsize - *buflen,
                c"%ld ".as_ptr(),
                (*cap).count0 as int64_t,
            ) as size_t;
        }
        *ptr_arg = word;
        n
    }
}

/// The characters that have to be backslash-escaped for the command being
/// built, given what it is.
unsafe fn ident_escapes(cmdchar: c_int, tag_cmd: bool) -> &'static CStr {
    // SAFETY: reads 'magic' and the current buffer's 'filetype'.
    unsafe {
        if cmdchar == '*' as c_int {
            if magic_isset() {
                c"/.*~[^$\\"
            } else {
                c"/^$\\"
            }
        } else if cmdchar == '#' as c_int {
            if magic_isset() {
                c"/?.*~[^$\\"
            } else {
                c"/?^$\\"
            }
        } else if tag_cmd {
            // A help tag may contain any of these, so nothing is escaped.
            if strcmp((*curbuf.get()).b_p_ft, c"help".as_ptr()) == 0 {
                c""
            } else {
                c"\\|\"\n["
            }
        } else {
            c"\\|\"\n*?["
        }
    }
}

/// Copy `n` bytes of the identifier into the command being built, escaping
/// whatever the command cannot take literally.
unsafe fn append_escaped(
    dest: *mut c_char,
    src: &mut *mut c_char,
    mut n: size_t,
    escapes: &CStr,
) -> *mut c_char {
    // SAFETY: `dest` has room for twice `n` plus a terminator, which is what
    // `nv_ident` sized the buffer for.
    unsafe {
        let mut out = dest;
        // The caller reads the source pointer back: `*` and `#` ask whether
        // the character *before* where this stopped was a word character, to
        // decide whether the search anchors at the end too.
        let p = src;
        while n > 0 {
            n -= 1;
            if !vim_strchr(escapes.as_ptr(), **p as uint8_t as c_int).is_null() {
                *out = '\\' as c_char;
                out = out.offset(1);
            }
            // `utfc_ptr2len` answers 0 at a NUL, so this is `(size_t)-1`
            // there and the inner loop then runs until `n` is spent --
            // upstream's behaviour, and what stops a NUL ending the copy
            // early.
            let trailing = (utfc_ptr2len(*p) - 1) as size_t;
            let mut i: size_t = 0;
            while i < trailing && n > 0 {
                *out = **p;
                out = out.offset(1);
                *p = p.offset(1);
                i += 1;
                n -= 1;
            }
            *out = **p;
            out = out.offset(1);
            *p = p.offset(1);
        }
        *out = NUL as c_char;
        out
    }
}

/// `*`, `#`, `K`, `]`, `CTRL-]` and their `g` forms: look up the identifier
/// under the cursor.
pub(crate) unsafe fn nv_ident(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        // The `g` forms carry the real command in `nchar`.
        let (mut cmdchar, g_cmd) = if (*cap).cmdchar == 'g' as c_int {
            ((*cap).nchar, true)
        } else {
            ((*cap).cmdchar, false)
        };
        if cmdchar == POUND {
            cmdchar = '#' as c_int;
        }

        let mut word: *mut c_char = ptr::null_mut();
        let mut n: size_t = 0;
        // Three of the commands take a Visual selection instead of the word
        // under the cursor.
        let mut visual_sel = false;
        if cmdchar == ']' as c_int || cmdchar == Ctrl_RSB || cmdchar == 'K' as c_int {
            if VIsual_active.get() && !get_visual_text(cap, &raw mut word, &raw mut n) {
                return;
            }
            visual_sel = !word.is_null();
            if checkclearopq((*cap).oap) {
                return;
            }
        }
        if word.is_null() {
            let mut ident_offset: c_int = 0;
            // `*` and `#` fall back to the string under the cursor when there
            // is no identifier there.
            let find_type = if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
                FIND_IDENT as c_int | FIND_STRING as c_int
            } else {
                FIND_IDENT as c_int
            };
            n = find_ident_under_cursor(&raw mut word, find_type, &raw mut ident_offset);
            if n == 0 {
                clearop((*cap).oap);
                return;
            }
        }

        // 'keywordprg', which decides what `K` does.
        let kp = if *(*curbuf.get()).b_p_kp as c_int == NUL {
            p_kp.get()
        } else {
            (*curbuf.get()).b_p_kp
        };
        let kp_helpbang = strequal(kp, c":help!".as_ptr());
        let kp_help = kp_helpbang
            || *kp as c_int == NUL
            || strequal(kp, c":he".as_ptr())
            || strequal(kp, c":help".as_ptr());
        if kp_help && !kp_helpbang && *skipwhite(word) as c_int == NUL {
            emsg(gettext(&raw const e_noident as *const c_char));
            return;
        }
        let kp_ex = *kp as c_int == ':' as c_int;

        // Room for the command, the word with every byte escaped, and a
        // terminator.
        let bufsize = n.wrapping_mul(2).wrapping_add(30).wrapping_add(strlen(kp));
        let mut buf = xmalloc(bufsize) as *mut c_char;
        *buf = NUL as c_char;
        let mut buflen: size_t = 0;

        // Whether the command being built is a tag lookup, which decides the
        // escaping below.
        let mut tag_cmd = false;
        match u8::try_from(cmdchar) {
            Ok(b'*' | b'#') => {
                // These become a search, so the cursor moves to the start of
                // the word first.
                setpcmark();
                (*curwin.get()).w_cursor.col = word.offset_from(get_cursor_line_ptr()) as colnr_T;
                if !g_cmd && vim_iswordp(word) {
                    // The plain forms anchor at a word boundary.
                    strcpy(buf, c"\\<".as_ptr() as *mut c_char);
                    buflen = c"\\<".count_bytes() as size_t;
                }
                no_smartcase.set(true);
            }
            Ok(b'K') => {
                n = nv_K_getcmd(
                    cap,
                    kp,
                    kp_help,
                    kp_ex,
                    &raw mut word,
                    n,
                    buf,
                    bufsize,
                    &raw mut buflen,
                );
                if n == 0 {
                    return;
                }
            }
            Ok(b']') => {
                tag_cmd = true;
                strcpy(buf, c"tselect ".as_ptr() as *mut c_char);
                buflen = c"tselect ".count_bytes() as size_t;
            }
            // CTRL-] and everything else: a plain tag jump.
            _ => {
                tag_cmd = true;
                let cmd: &CStr = if (*curbuf.get()).b_help {
                    c"help! "
                } else if g_cmd {
                    c"tjump "
                } else if (*cap).count0 == 0 {
                    c"tag "
                } else {
                    // A count picks which of the matching tags to jump to.
                    buflen = snprintf(buf, bufsize, c":%ldtag ".as_ptr(), (*cap).count0 as int64_t)
                        as size_t;
                    c""
                };
                if !cmd.is_empty() {
                    strcpy(buf, cmd.as_ptr() as *mut c_char);
                    buflen = cmd.count_bytes() as size_t;
                }
            }
        }

        if cmdchar == 'K' as c_int && kp_helpbang && !visual_sel {
            // `:help!` with no selection opens the help index rather than
            // looking anything up.
            strcpy(buf, c"help!".as_ptr() as *mut c_char);
            buflen = c"help!".count_bytes() as size_t;
        } else if cmdchar == 'K' as c_int && !kp_help {
            // A shell or Ex command takes the word quoted, not escaped
            // character by character.
            let owned = xstrnsave(word, n);
            let quoted = if kp_ex {
                vim_strsave_fnameescape(owned, VSE_NONE as c_int)
            } else {
                vim_strsave_shellescape(owned, true, true)
            };
            xfree(owned as *mut c_void);
            let plen = strlen(quoted);
            buf = xrealloc(buf as *mut c_void, buflen + plen + 1) as *mut c_char;
            strcpy(buf.add(buflen), quoted);
            buflen += plen;
            xfree(quoted as *mut c_void);
        } else {
            let escapes = ident_escapes(cmdchar, tag_cmd);
            let end = append_escaped(buf.add(buflen), &mut word, n, escapes);
            buflen = end.offset_from(buf) as size_t;
        }

        if cmdchar == '*' as c_int || cmdchar == '#' as c_int {
            if !g_cmd && vim_iswordp(mb_prevptr(get_cursor_line_ptr(), word)) {
                strcpy(buf.add(buflen), c"\\>".as_ptr() as *mut c_char);
                buflen += c"\\>".count_bytes() as size_t;
            }
            // The search goes into the history as if it had been typed.
            init_history();
            add_to_history(
                HIST_SEARCH as c_int,
                core::slice::from_raw_parts(buf as *const u8, buflen as usize),
                true,
                NUL as u8,
            );
            normal_search(
                cap,
                if cmdchar == '*' as c_int {
                    '/' as c_int
                } else {
                    '?' as c_int
                },
                buf,
                buflen,
                0,
                ptr::null_mut(),
            );
        } else {
            // `taglist()` and friends need to know the tag came from under
            // the cursor rather than from a command line.
            g_tag_at_cursor.set(true);
            do_cmdline_cmd(buf);
            g_tag_at_cursor.set(false);
            if cmdchar == 'K' as c_int && !kp_ex && !kp_help {
                // The terminal 'keywordprg' opened above: let <Esc> close it.
                restart_edit.set('i' as c_int);
                add_map(
                    c"<esc>".as_ptr() as *mut c_char,
                    c"<Cmd>bdelete!<CR>".as_ptr() as *mut c_char,
                    MODE_TERMINAL,
                    true,
                );
            }
        }
        xfree(buf as *mut c_void);
    }
}

/// `CTRL-T`: back up the tag stack.
pub(crate) unsafe fn nv_tagpop(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if !checkclearopq((*cap).oap) {
            do_tag(
                c"".as_ptr() as *mut c_char,
                DT_POP as c_int,
                (*cap).count1,
                false_0,
                true,
            );
        }
    }
}

/// `gf`, `gF` and `[f`: edit the file named under the cursor.
pub(crate) unsafe fn nv_gotofile(cap: *mut cmdarg_T) {
    // SAFETY: `cap` is the caller's live command argument.
    unsafe {
        if check_text_or_curbuf_locked((*cap).oap) || !check_can_set_curbuf_disabled() {
            return;
        }
        // `gF` also takes a line number off the end of the name.
        let mut lnum: linenr_T = -1;
        let name = grab_file_name((*cap).count1, &raw mut lnum);
        if name.is_null() {
            clearop((*cap).oap);
            return;
        }
        // Leaving the only window on a changed buffer that cannot be hidden
        // means writing it first.
        if curbufIsChanged() && (*curbuf.get()).b_nwindows <= 1 && !buf_hide(curbuf.get()) {
            autowrite(curbuf.get(), false);
        }
        setpcmark();
        let opened = do_ecmd(
            0,
            name,
            ptr::null_mut(),
            ptr::null_mut(),
            ECMD_LAST as linenr_T,
            if buf_hide(curbuf.get()) {
                ECMD_HIDE as c_int
            } else {
                0
            },
            curwin.get(),
        ) == OK;
        if opened && (*cap).nchar == 'F' as c_int && lnum >= 0 {
            (*curwin.get()).w_cursor.lnum = lnum;
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        xfree(name as *mut c_void);
    }
}
