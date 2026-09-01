//! The opcodes that settle here and now: a position assertion, one character
//! of input, or a back-reference. None of them leaves anything behind for the
//! matcher to reconsider, so each one just says whether the match continues.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::c_int;

use super::exec::re_num_cmp;
use super::op::BtOp;
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
    RA_CONT, RA_MATCH, RA_NOMATCH, RI_ALPHA, RI_DIGIT, RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER,
    RI_OCTAL, RI_UPPER, RI_WORD, Rex, behind_pos, cleanup_subexpr, cleanup_zsubexpr, cstrchr,
    cstrncmp, kMarkBufLocal, match_with_backref, reg_getline, reg_getline_len, reg_match_visual,
    reg_nextline, reg_prev_class,
};
use crate::types::{GraphemeState, NUL, fmark_T, linenr_T, pos_T, uint8_t, uint32_t, uint64_t};

use crate::winlayer::Win;

/// The code each reference run is measured from. `const` items, because
/// `BtOp::Backref.code()` is a call at opt-level 0.
const BACKREF: c_int = BtOp::Backref.code();
const ZREF: c_int = BtOp::Zref.code();

/// Run the opcode `op` if it is one of ours, and say how the match went.
/// `None` means the opcode belongs to the matcher's own dispatch.
///
/// `c` is the character at the cursor, decoded once by the caller; `next` is
/// the node after `scan`, which only `EXACTLY` looks at.
pub(crate) fn match_one(
    rex: Rex,
    op: BtOp,
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
        BtOp::Bol => nomatch_unless(rex.at_bol()),
        BtOp::Eol => nomatch_unless(c == NUL),
        BtOp::ReBof => nomatch_unless(
            rex.lnum() == 0 && rex.at_bol() && (!rex.multi() || rex.reg_firstlnum() <= 1),
        ),
        BtOp::ReEof => nomatch_unless(rex.lnum() == rex.reg_maxline() && c == NUL),
        BtOp::Cursor => nomatch_unless(match cursor_of(rex) {
            Some(cursor) => rex.buf_lnum() == cursor.lnum && rex.col() == cursor.col,
            None => false,
        }),
        BtOp::ReMark => at_mark(rex, scan),
        BtOp::ReVisual => nomatch_unless(reg_match_visual(rex)),

        // `\%23l`, `\%23c`, `\%23v` and their `<`/`>` forms. A line
        // number only means something in a multi-line match.
        BtOp::ReLnum => nomatch_unless(rex.multi() && re_num_cmp(rex.buf_lnum() as uint32_t, scan)),
        BtOp::ReCol => nomatch_unless(re_num_cmp((rex.col() as uint32_t).wrapping_add(1), scan)),
        BtOp::ReVcol => nomatch_unless(re_num_cmp(virtual_column(rex).wrapping_add(1), scan)),

        // `\<` and `\>`: a change of character class, with the classes
        // below 2 (whitespace and punctuation) never starting a word.
        BtOp::Bow => {
            if c == NUL {
                RA_NOMATCH
            } else {
                let this = char_class(rex);
                nomatch_unless(this > 1 && reg_prev_class(rex) != this)
            }
        }
        BtOp::Eow => {
            if rex.at_bol() {
                RA_NOMATCH
            } else {
                let this = char_class(rex);
                let prev = reg_prev_class(rex);
                nomatch_unless(this != prev && prev != 0 && prev != 1)
            }
        }

        BtOp::Any => return take(c != NUL),
        BtOp::Ident => return take(is_ident_char(c)),
        BtOp::Sident => return take(!digit_here() && is_ident_char(c)),
        BtOp::Kword => return take(rex.iswordp()),
        BtOp::Skword => return take(!digit_here() && rex.iswordp()),
        BtOp::Fname => return take(is_file_char(c)),
        BtOp::Sfname => return take(!digit_here() && is_file_char(c)),
        BtOp::Print => return take(is_printable(rex.char_here())),
        BtOp::Sprint => return take(!digit_here() && is_printable(rex.char_here())),
        BtOp::White => return take(ascii_iswhite(c)),
        BtOp::Nwhite => return take(c != NUL && !ascii_iswhite(c)),
        BtOp::Digit => return take(ri(RI_DIGIT)),
        BtOp::Ndigit => return take(c != NUL && !ri(RI_DIGIT)),
        BtOp::Hex => return take(ri(RI_HEX)),
        BtOp::Nhex => return take(c != NUL && !ri(RI_HEX)),
        BtOp::Octal => return take(ri(RI_OCTAL)),
        BtOp::Noctal => return take(c != NUL && !ri(RI_OCTAL)),
        BtOp::Word => return take(ri(RI_WORD)),
        BtOp::Nword => return take(c != NUL && !ri(RI_WORD)),
        BtOp::Head => return take(ri(RI_HEAD)),
        BtOp::Nhead => return take(c != NUL && !ri(RI_HEAD)),
        BtOp::Alpha => return take(ri(RI_ALPHA)),
        BtOp::Nalpha => return take(c != NUL && !ri(RI_ALPHA)),
        BtOp::Lower => return take(ri(RI_LOWER)),
        BtOp::Nlower => return take(c != NUL && !ri(RI_LOWER)),
        BtOp::Upper => return take(ri(RI_UPPER)),
        BtOp::Nupper => return take(c != NUL && !ri(RI_UPPER)),

        BtOp::Exactly => exactly(rex, scan, next),
        BtOp::Anyof | BtOp::Anybut => collection(rex, scan, c, op == BtOp::Anyof),
        BtOp::Multibytecode => multibyte(rex, scan),

        // `\%C`: swallow any combining characters, matching nothing else.
        BtOp::ReComposing => {
            while utf_iscomposing_legacy(rex.char_here()) {
                rex.advance(rex.base_len());
            }
            RA_CONT
        }
        BtOp::Nothing => RA_CONT,

        BtOp::Backref1
        | BtOp::Backref2
        | BtOp::Backref3
        | BtOp::Backref4
        | BtOp::Backref5
        | BtOp::Backref6
        | BtOp::Backref7
        | BtOp::Backref8
        | BtOp::Backref9 => back_reference(rex, op.code() - BACKREF),
        BtOp::Zref1
        | BtOp::Zref2
        | BtOp::Zref3
        | BtOp::Zref4
        | BtOp::Zref5
        | BtOp::Zref6
        | BtOp::Zref7
        | BtOp::Zref8
        | BtOp::Zref9 => external_reference(rex, op.code() - ZREF),

        // The position a `\@<=` look-behind has to end at.
        // SAFETY: `behind_pos` is this engine's own saved position.
        BtOp::Bhpos => nomatch_unless(rex.is_at(behind_pos.get().pos)),

        BtOp::Newl => {
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

        BtOp::End => RA_MATCH,
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
    let chartab = (unsafe { &raw mut (*rex.reg_buf()).b_chartab }).cast::<uint64_t>();
    unsafe { mb_get_class_tab(rex.input_str(), chartab) }
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
    // The record `mark_get` answers into: a motion mark (`'{`, `'(`) has no
    // store of its own, so it is computed straight into this frame's slot.
    let mut slot = fmark_T::UNSET;
    // SAFETY: `reg_buf` is the buffer being matched and `curwin` the current
    // window; `slot` is this frame's and outlives every use of `fm`.
    let buf = rex.reg_buf();
    let win = curwin.get();
    let fm = unsafe { mark_get(buf, win, &raw mut slot, kMarkBufLocal, mark) };
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
    let opnd = unsafe { scan.add(3) };
    if unsafe { *opnd } as c_int != rex.byte() as c_int && !rex.reg_ic() {
        return RA_NOMATCH;
    }
    if unsafe { *opnd } as c_int == NUL {
        return RA_CONT;
    }
    let mut len;
    // A one-byte operand with no case folding needs no compare: the
    // first-byte test above already settled it.
    if unsafe { *opnd.add(1) } as c_int == NUL && !rex.reg_ic() {
        len = 1;
    } else {
        len = unsafe { cstr::bytes_at(opnd.cast()) }.len() as c_int;
        if unsafe { cstrncmp(rex, opnd.cast(), rex.input().cast(), &mut len) } != 0 {
            return RA_NOMATCH;
        }
    }
    // A combining character right after the match means the input has a
    // different grapheme here, unless 'reg_icombine' or a following `\%C`
    // says to ignore combining characters.
    if unsafe {
        utf_composinglike(
            rex.input().cast(),
            rex.input_str().add(len as usize),
            core::ptr::null_mut::<GraphemeState>(),
        )
    } && !rex.reg_icombine()
        && unsafe { *next } != BtOp::ReComposing.code() as uint8_t
    {
        return RA_NOMATCH;
    }
    rex.advance(len);
    RA_CONT
}

/// `[abc]` / `[^abc]`. The base character has to be in (or out of) the set,
/// and any combining characters the set's own entry carries must follow it.
fn collection(rex: Rex, scan: *mut uint8_t, c: c_int, positive: bool) -> c_int {
    // SAFETY: `scan` is an `ANYOF`/`ANYBUT` node with a NUL-terminated
    // operand; `rex.input` is NUL-terminated.
    let mut q = unsafe { scan.add(3) };
    if c == NUL {
        return RA_NOMATCH;
    }
    if unsafe { cstrchr(rex, q.cast(), c) }.is_null() == positive {
        return RA_NOMATCH;
    }
    // The set entry may itself be a grapheme; its combining part has to
    // match the input's byte for byte.
    let combining = unsafe { utfc_ptr2len(q.cast()) } - unsafe { utf_ptr2len(q.cast()) };
    rex.advance(rex.base_len());
    q = unsafe { q.add(utf_ptr2len(q.cast()) as usize) };
    let mut status = RA_CONT;
    if combining != 0 {
        for i in 0..combining as usize {
            if unsafe { *q.add(i) } != unsafe { *rex.input().add(i) } {
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

/// One multibyte character, possibly a bare combining character — which
/// matches anywhere in the grapheme at the cursor.
fn multibyte(rex: Rex, scan: *mut uint8_t) -> c_int {
    // SAFETY: `scan` is a `MULTIBYTECODE` node; both operand and input are
    // NUL-terminated.
    let opnd = unsafe { scan.add(3) };
    let mut len = unsafe { utfc_ptr2len(opnd.cast()) };
    if len < 2 {
        return RA_NOMATCH;
    }
    let opndc = unsafe { utf_ptr2char(opnd.cast()) };
    if utf_iscomposing_legacy(opndc) {
        // A bare combining character matches wherever it appears in the
        // grapheme at the cursor.
        let mut status = RA_NOMATCH;
        let mut i = 0;
        while unsafe { *rex.input().add(i as usize) } as c_int != NUL {
            let at = unsafe { rex.input_str().add(i as usize) };
            let inpc = unsafe { utf_ptr2char(at) };
            if !utf_iscomposing_legacy(inpc) {
                // The base character is allowed to be the first thing
                // looked at; anything past it ends the grapheme.
                if i > 0 {
                    break;
                }
            } else if opndc == inpc {
                // Include the combining characters that follow.
                len = i + unsafe { utfc_ptr2len(at) };
                // RA_MATCH, not RA_CONT: upstream ends the whole pattern
                // here. Preserved rather than "fixed".
                status = RA_MATCH;
                break;
            }
            i += unsafe { utf_ptr2len(at) };
        }
        // Upstream advances whether or not it found the character.
        rex.advance(len);
        return status;
    }
    if unsafe { cstrncmp(rex, opnd.cast(), rex.input().cast(), &mut len) } != 0 {
        return RA_NOMATCH;
    }
    rex.advance(len);
    RA_CONT
}

/// `\1`..`\9`: the text group `no` captured, again.
fn back_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_subexpr(rex);
    // SAFETY: the capture slots hold either null/negative-line markers or
    // positions inside the text being matched.
    let mut len = 0;
    let mut status = RA_CONT;
    if !rex.multi() {
        // A string match: the capture is a pair of pointers.
        let start = unsafe { *rex.reg_startp().add(no as usize) };
        let end = unsafe { *rex.reg_endp().add(no as usize) };
        if !start.is_null() && !end.is_null() {
            len = unsafe { end.offset_from(start) } as c_int;
            if unsafe { cstrncmp(rex, start.cast(), rex.input().cast(), &mut len) } != 0 {
                status = RA_NOMATCH;
            }
        }
    } else {
        let start = unsafe { *rex.reg_startpos().add(no as usize) };
        let end = unsafe { *rex.reg_endpos().add(no as usize) };
        if start.lnum < 0 || end.lnum < 0 {
            // Never captured: an empty match.
        } else if start.lnum == rex.lnum() && end.lnum == rex.lnum() {
            len = end.col - start.col;
            let at = unsafe {
                rex.line()
                    .cast::<core::ffi::c_char>()
                    .add(start.col as usize)
            };
            if unsafe { cstrncmp(rex, at, rex.input().cast(), &mut len) } != 0 {
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

/// `\z1`..`\z9`: the text the enclosing syntax region captured. Missing
/// captures match the empty string rather than failing.
fn external_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_zsubexpr(rex);
    let captures = re_extmatch_in.get();
    if captures.is_null() {
        return RA_CONT;
    }
    // SAFETY: `re_extmatch_in`'s entries are NUL-terminated copies.
    let text = unsafe { (*captures).matches[no as usize] };
    if text.is_null() {
        return RA_CONT;
    }
    let mut len = unsafe { cstr::bytes_at(text.cast()) }.len() as c_int;
    if unsafe { cstrncmp(rex, text.cast(), rex.input().cast(), &mut len) } != 0 {
        return RA_NOMATCH;
    }
    rex.advance(len);
    RA_CONT
}
