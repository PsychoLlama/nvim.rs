//! The opcodes that settle here and now: a position assertion, one character
//! of input, or a back-reference. None of them leaves anything behind for the
//! matcher to reconsider, so each one just says whether the match continues.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::exec::re_num_cmp;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::main::{curwin, re_extmatch_in};
use crate::mark::mark_get;
use crate::mbyte::{
    mb_get_class_tab, utf_composinglike, utf_iscomposing_legacy, utf_ptr2char, utf_ptr2len,
    utfc_ptr2len,
};
use crate::plines::win_linetabsize;
use crate::pos::MAXCOL;
use crate::regexp::{
    ALPHA, ANY, ANYBUT, ANYOF, BACKREF, BHPOS, BOL, BOW, CURSOR, DIGIT, END, EOL, EOW, EXACTLY,
    FNAME, HEAD, HEX, IDENT, KWORD, LOWER, MULTIBYTECODE, NALPHA, NDIGIT, NEWL, NHEAD, NHEX,
    NLOWER, NOCTAL, NOTHING, NUPPER, NWHITE, NWORD, OCTAL, PRINT, RA_CONT, RA_MATCH, RA_NOMATCH,
    RE_BOF, RE_COL, RE_COMPOSING, RE_EOF, RE_LNUM, RE_MARK, RE_VCOL, RE_VISUAL, RI_ALPHA, RI_DIGIT,
    RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER, RI_WORD, Rex, SFNAME, SIDENT, SKWORD,
    SPRINT, UPPER, WHITE, WORD, ZREF, behind_pos, cleanup_subexpr, cleanup_zsubexpr, cstrchr,
    cstrncmp, kMarkBufLocal, match_with_backref, reg_getline, reg_getline_len, reg_match_visual,
    reg_nextline, reg_prev_class,
};
use crate::types::{GraphemeState, NUL, fmark_T, linenr_T, pos_T, uint8_t, uint32_t, uint64_t};
use ::libc::strlen;

use crate::winlayer::Win;
const BACKREF_1: c_int = BACKREF + 1;
const BACKREF_9: c_int = BACKREF + 9;
const ZREF_1: c_int = ZREF + 1;
const ZREF_9: c_int = ZREF + 9;

/// Run the opcode `op` if it is one of ours, and say how the match went.
/// `None` means the opcode belongs to the matcher's own dispatch.
///
/// `c` is the character at the cursor, decoded once by the caller; `next` is
/// the node after `scan`, which only `EXACTLY` looks at.
pub(crate) fn match_one(
    rex: Rex,
    op: c_int,
    scan: *mut uint8_t,
    next: *mut uint8_t,
    c: c_int,
) -> Option<c_int> {
    // Step over the character at the cursor when `ok`, else fail.
    let take = |ok: bool| {
        if !ok {
            return Some(RA_NOMATCH);
        }
        rex.advance_char();
        Some(RA_CONT)
    };
    // The `RI_*` byte classes only classify Latin-1.
    let ri = |mask: c_int| c < 0x100 && RI_FLAGS[c as usize] as c_int & mask != 0;
    // The `S`-prefixed classes reject a leading digit.
    let digit_here = || ascii_isdigit(rex.byte() as c_int);

    let status = match op {
        BOL => nomatch_unless(rex.at_bol()),
        EOL => nomatch_unless(c == NUL),
        RE_BOF => nomatch_unless(
            rex.lnum() == 0 && rex.at_bol() && (!rex.multi() || rex.reg_firstlnum() <= 1),
        ),
        RE_EOF => nomatch_unless(rex.lnum() == rex.reg_maxline() && c == NUL),
        CURSOR => nomatch_unless(match cursor_of(rex) {
            Some(cursor) => rex.buf_lnum() == cursor.lnum && rex.col() == cursor.col,
            None => false,
        }),
        RE_MARK => at_mark(rex, scan),
        RE_VISUAL => nomatch_unless(reg_match_visual(rex)),

        // `\%23l`, `\%23c`, `\%23v` and their `<`/`>` forms. A line
        // number only means something in a multi-line match.
        RE_LNUM => nomatch_unless(rex.multi() && re_num_cmp(rex.buf_lnum() as uint32_t, scan)),
        RE_COL => nomatch_unless(re_num_cmp((rex.col() as uint32_t).wrapping_add(1), scan)),
        RE_VCOL => nomatch_unless(re_num_cmp(virtual_column(rex).wrapping_add(1), scan)),

        // `\<` and `\>`: a change of character class, with the classes
        // below 2 (whitespace and punctuation) never starting a word.
        BOW => {
            if c == NUL {
                RA_NOMATCH
            } else {
                let this = char_class(rex);
                nomatch_unless(this > 1 && reg_prev_class(rex) != this)
            }
        }
        EOW => {
            if rex.at_bol() {
                RA_NOMATCH
            } else {
                let this = char_class(rex);
                let prev = reg_prev_class(rex);
                nomatch_unless(this != prev && prev != 0 && prev != 1)
            }
        }

        ANY => return take(c != NUL),
        IDENT => return take(is_ident_char(c)),
        SIDENT => return take(!digit_here() && is_ident_char(c)),
        KWORD => return take(rex.iswordp()),
        SKWORD => return take(!digit_here() && rex.iswordp()),
        FNAME => return take(is_file_char(c)),
        SFNAME => return take(!digit_here() && is_file_char(c)),
        PRINT => return take(is_printable(rex.char_here())),
        SPRINT => return take(!digit_here() && is_printable(rex.char_here())),
        WHITE => return take(ascii_iswhite(c)),
        NWHITE => return take(c != NUL && !ascii_iswhite(c)),
        DIGIT => return take(ri(RI_DIGIT)),
        NDIGIT => return take(c != NUL && !ri(RI_DIGIT)),
        HEX => return take(ri(RI_HEX)),
        NHEX => return take(c != NUL && !ri(RI_HEX)),
        OCTAL => return take(ri(RI_OCTAL)),
        NOCTAL => return take(c != NUL && !ri(RI_OCTAL)),
        WORD => return take(ri(RI_WORD)),
        NWORD => return take(c != NUL && !ri(RI_WORD)),
        HEAD => return take(ri(RI_HEAD)),
        NHEAD => return take(c != NUL && !ri(RI_HEAD)),
        ALPHA => return take(ri(RI_ALPHA)),
        NALPHA => return take(c != NUL && !ri(RI_ALPHA)),
        LOWER => return take(ri(RI_LOWER)),
        NLOWER => return take(c != NUL && !ri(RI_LOWER)),
        UPPER => return take(ri(RI_UPPER)),
        NUPPER => return take(c != NUL && !ri(RI_UPPER)),

        EXACTLY => exactly(rex, scan, next),
        ANYOF | ANYBUT => collection(rex, scan, c, op == ANYOF),
        MULTIBYTECODE => multibyte(rex, scan),

        // `\%C`: swallow any combining characters, matching nothing else.
        RE_COMPOSING => {
            while utf_iscomposing_legacy(rex.char_here()) {
                rex.advance(rex.base_len());
            }
            RA_CONT
        }
        NOTHING => RA_CONT,

        BACKREF_1..=BACKREF_9 => back_reference(rex, op - BACKREF),
        ZREF_1..=ZREF_9 => external_reference(rex, op - ZREF),

        // The position a `\@<=` look-behind has to end at.
        // SAFETY: `behind_pos` is this engine's own saved position.
        BHPOS => nomatch_unless(rex.is_at(behind_pos.get().pos)),

        NEWL => {
            let lbr = rex.reg_line_lbr();
            let at_break = c == NUL && rex.multi() && rex.lnum() <= rex.reg_maxline() && !lbr;
            if !at_break && !(c == '\n' as c_int && lbr) {
                RA_NOMATCH
            } else {
                if lbr {
                    rex.advance_char();
                } else {
                    reg_nextline(rex);
                }
                RA_CONT
            }
        }

        END => RA_MATCH,
        _ => return None,
    };
    Some(status)
}

/// The cursor of the window the match runs in, if it runs in one. `\%#`
/// needs a window and a string match has none.
fn cursor_of(rex: Rex) -> Option<pos_T> {
    let win = rex.reg_win();
    // SAFETY: a non-null `reg_win` is the live window the match runs in.
    (!win.is_null()).then(|| unsafe { (*win).w_cursor })
}

/// `vim_is_ident_char`, `vim_isfilec` and `vim_isprintc` are pure tests on a code
/// point that read only option state.
fn is_ident_char(c: c_int) -> bool {
    unsafe { vim_is_ident_char(c) }
}

fn is_file_char(c: c_int) -> bool {
    unsafe { vim_isfilec(c) }
}

fn is_printable(c: c_int) -> bool {
    unsafe { vim_isprintc(c) }
}

fn nomatch_unless(ok: bool) -> c_int {
    if ok { RA_CONT } else { RA_NOMATCH }
}

/// The character class of the character at the cursor, per the buffer's
/// 'iskeyword'.
fn char_class(rex: Rex) -> c_int {
    // SAFETY: `rex.input` points into the current line and `reg_buf` is the
    // buffer being matched, so it has a 'iskeyword' table.
    unsafe {
        let chartab = (&raw mut (*rex.reg_buf()).b_chartab).cast::<uint64_t>();
        mb_get_class_tab(rex.input_str(), chartab)
    }
}

/// `\%23v` compares against this: the screen column the cursor sits in.
fn virtual_column(rex: Rex) -> uint32_t {
    let wp = if rex.reg_win().is_null() {
        curwin.get()
    } else {
        rex.reg_win()
    };
    let mut lnum: linenr_T = if rex.multi() { rex.buf_lnum() } else { 1 };
    // A string match has no line numbers, and a multi-line match may be
    // running over a line that has since been deleted.
    // SAFETY: `wp` is a live window, so it has a buffer.
    if rex.multi() && (lnum <= 0 || lnum > unsafe { (*(*wp).w_buffer).b_ml.ml_line_count }) {
        lnum = 1;
    }
    // SAFETY: `rex.line` is the line being matched, NUL-terminated, and the
    // cursor is a byte offset into it.
    unsafe { win_linetabsize(Win::new(wp), lnum, rex.line().cast(), rex.col()) as uint32_t }
}

/// `\%'m`, `\%<'m`, `\%>'m`: is the cursor at, before or after mark `m`?
fn at_mark(rex: Rex, scan: *mut uint8_t) -> c_int {
    // SAFETY: `scan` is an `RE_MARK` node, whose operand is the mark name and
    // the comparison character.
    let (mark, cmp) = unsafe { (*scan.add(3) as c_int, *scan.add(4) as c_int) };
    let col = if rex.multi() { rex.col() } else { 0 };
    // SAFETY: `reg_buf` is the buffer being matched and `curwin` the current
    // window; a null `fmark_T` asks for the buffer's own mark list.
    let fm = unsafe {
        mark_get(
            rex.reg_buf(),
            curwin.get(),
            core::ptr::null_mut::<fmark_T>(),
            kMarkBufLocal,
            mark,
        )
    };
    // `mark_get` can move the buffer's line pointers, so re-anchor.
    if rex.multi() {
        rex.seek(reg_getline(rex, rex.lnum()).cast(), col);
    }
    if fm.is_null() {
        return RA_NOMATCH;
    }
    // SAFETY: a non-null `mark_get` result is a live mark.
    let pos = unsafe { (*fm).mark };
    if pos.lnum <= 0 {
        return RA_NOMATCH;
    }
    let here_lnum = rex.buf_lnum();
    // A mark at MAXCOL sits at the end of its line.
    let pos_col = if pos.lnum == here_lnum && pos.col == MAXCOL as c_int {
        reg_getline_len(rex, pos.lnum - rex.reg_firstlnum())
    } else {
        pos.col
    };
    let here_col = rex.col();
    // Upstream's condition is the *failure* condition, kept as such.
    let fails = if pos.lnum == here_lnum {
        if pos_col == here_col {
            cmp == '<' as c_int || cmp == '>' as c_int
        } else if pos_col < here_col {
            cmp != '>' as c_int
        } else {
            cmp != '<' as c_int
        }
    } else if pos.lnum < here_lnum {
        cmp != '>' as c_int
    } else {
        cmp != '<' as c_int
    };
    if fails { RA_NOMATCH } else { RA_CONT }
}

/// A literal string. The operand is NUL-terminated, so an empty one always
/// matches.
fn exactly(rex: Rex, scan: *mut uint8_t, next: *mut uint8_t) -> c_int {
    // SAFETY: `scan` is an `EXACTLY` node, whose operand is a NUL-terminated
    // string; `rex.input` is NUL-terminated too, so `cstrncmp` stops.
    unsafe {
        let opnd = scan.add(3);
        if *opnd as c_int != rex.byte() as c_int && !rex.reg_ic() {
            return RA_NOMATCH;
        }
        if *opnd as c_int == NUL {
            return RA_CONT;
        }
        let mut len;
        // A one-byte operand with no case folding needs no compare: the
        // first-byte test above already settled it.
        if *opnd.add(1) as c_int == NUL && !rex.reg_ic() {
            len = 1;
        } else {
            len = strlen(opnd.cast()) as c_int;
            if cstrncmp(rex, opnd.cast(), rex.input().cast(), &mut len) != 0 {
                return RA_NOMATCH;
            }
        }
        // A combining character right after the match means the input has a
        // different grapheme here, unless 'reg_icombine' or a following `\%C`
        // says to ignore combining characters.
        if utf_composinglike(
            rex.input().cast(),
            rex.input_str().add(len as usize),
            core::ptr::null_mut::<GraphemeState>(),
        ) && !rex.reg_icombine()
            && *next as c_int != RE_COMPOSING
        {
            return RA_NOMATCH;
        }
        rex.advance(len);
        RA_CONT
    }
}

/// `[abc]` / `[^abc]`. The base character has to be in (or out of) the set,
/// and any combining characters the set's own entry carries must follow it.
fn collection(rex: Rex, scan: *mut uint8_t, c: c_int, positive: bool) -> c_int {
    // SAFETY: `scan` is an `ANYOF`/`ANYBUT` node with a NUL-terminated
    // operand; `rex.input` is NUL-terminated.
    unsafe {
        let mut q = scan.add(3);
        if c == NUL {
            return RA_NOMATCH;
        }
        if cstrchr(rex, q.cast(), c).is_null() == positive {
            return RA_NOMATCH;
        }
        // The set entry may itself be a grapheme; its combining part has to
        // match the input's byte for byte.
        let combining = utfc_ptr2len(q.cast()) - utf_ptr2len(q.cast());
        rex.advance(rex.base_len());
        q = q.add(utf_ptr2len(q.cast()) as usize);
        let mut status = RA_CONT;
        if combining != 0 {
            for i in 0..combining as usize {
                if *q.add(i) != *rex.input().add(i) {
                    status = RA_NOMATCH;
                    break;
                }
            }
            // Upstream advances even when the tail did not match; the status
            // is what decides the outcome.
            rex.advance(combining);
        }
        status
    }
}

/// One multibyte character, possibly a bare combining character — which
/// matches anywhere in the grapheme at the cursor.
fn multibyte(rex: Rex, scan: *mut uint8_t) -> c_int {
    // SAFETY: `scan` is a `MULTIBYTECODE` node; both operand and input are
    // NUL-terminated.
    unsafe {
        let opnd = scan.add(3);
        let mut len = utfc_ptr2len(opnd.cast());
        if len < 2 {
            return RA_NOMATCH;
        }
        let opndc = utf_ptr2char(opnd.cast());
        if utf_iscomposing_legacy(opndc) {
            // A bare combining character matches wherever it appears in the
            // grapheme at the cursor.
            let mut status = RA_NOMATCH;
            let mut i = 0;
            while *rex.input().add(i as usize) as c_int != NUL {
                let at = rex.input_str().add(i as usize);
                let inpc = utf_ptr2char(at);
                if !utf_iscomposing_legacy(inpc) {
                    // The base character is allowed to be the first thing
                    // looked at; anything past it ends the grapheme.
                    if i > 0 {
                        break;
                    }
                } else if opndc == inpc {
                    // Include the combining characters that follow.
                    len = i + utfc_ptr2len(at);
                    // RA_MATCH, not RA_CONT: upstream ends the whole pattern
                    // here. Preserved rather than "fixed".
                    status = RA_MATCH;
                    break;
                }
                i += utf_ptr2len(at);
            }
            // Upstream advances whether or not it found the character.
            rex.advance(len);
            return status;
        }
        if cstrncmp(rex, opnd.cast(), rex.input().cast(), &mut len) != 0 {
            return RA_NOMATCH;
        }
        rex.advance(len);
        RA_CONT
    }
}

/// `\1`..`\9`: the text group `no` captured, again.
fn back_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_subexpr(rex);
    // SAFETY: the capture slots hold either null/negative-line markers or
    // positions inside the text being matched.
    unsafe {
        let mut len = 0;
        let mut status = RA_CONT;
        if !rex.multi() {
            // A string match: the capture is a pair of pointers.
            let start = *rex.reg_startp().add(no as usize);
            let end = *rex.reg_endp().add(no as usize);
            if !start.is_null() && !end.is_null() {
                len = end.offset_from(start) as c_int;
                if cstrncmp(rex, start.cast(), rex.input().cast(), &mut len) != 0 {
                    status = RA_NOMATCH;
                }
            }
        } else {
            let start = *rex.reg_startpos().add(no as usize);
            let end = *rex.reg_endpos().add(no as usize);
            if start.lnum < 0 || end.lnum < 0 {
                // Never captured: an empty match.
            } else if start.lnum == rex.lnum() && end.lnum == rex.lnum() {
                len = end.col - start.col;
                if cstrncmp(
                    rex,
                    rex.line()
                        .cast::<core::ffi::c_char>()
                        .add(start.col as usize),
                    rex.input().cast(),
                    &mut len,
                ) != 0
                {
                    status = RA_NOMATCH;
                }
            } else {
                let r = match_with_backref(
                    rex,
                    start.lnum,
                    start.col,
                    end.lnum,
                    end.col,
                    Some(&mut len),
                );
                if r != RA_MATCH {
                    status = r;
                }
            }
        }
        // Upstream advances by whatever `len` ended up as, match or not.
        rex.advance(len);
        status
    }
}

/// `\z1`..`\z9`: the text the enclosing syntax region captured. Missing
/// captures match the empty string rather than failing.
fn external_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_zsubexpr(rex);
    let captures = re_extmatch_in.get();
    if captures.is_null() {
        return RA_CONT;
    }
    // SAFETY: `re_extmatch_in`'s entries are NUL-terminated copies.
    unsafe {
        let text = (*captures).matches[no as usize];
        if text.is_null() {
            return RA_CONT;
        }
        let mut len = strlen(text.cast()) as c_int;
        if cstrncmp(rex, text.cast(), rex.input().cast(), &mut len) != 0 {
            return RA_NOMATCH;
        }
        rex.advance(len);
        RA_CONT
    }
}
