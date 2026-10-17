//! The opcodes that settle here and now: a position assertion, one character
//! of input, or a back-reference. None of them leaves anything behind for the
//! matcher to reconsider, so each one just says whether the match continues.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::compile::BtProg;
use super::exec::re_num_cmp;
use super::op::BtOp;
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::mark::mark_lookup;
use crate::mbyte::{class_in, utf_iscomposing_legacy};
use crate::plines::win_linetabsize_bytes;
use crate::pos::MAXCOL;
use crate::regexp::{
    ByteClass, RA_CONT, RA_MATCH, RA_NOMATCH, RI_FLAGS, Rex, char_at_nul, char_len_at,
    cleanup_subexpr, cleanup_zsubexpr, cluster_len_at, composes_at, cstrchr, cstrncmp,
    kMarkBufLocal, match_with_backref, nul_terminated_len, reg_getline, reg_getline_len,
    reg_match_visual, reg_nextline, reg_prev_class,
};
use crate::types::{LineNr, NUL, Pos, uint8_t, uint32_t};

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
    prog: BtProg<'_>,
    op: BtOp,
    scan: usize,
    next: usize,
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
    let ri = |mask: ByteClass| c < 0x100 && RI_FLAGS[c as usize].has(mask);
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
        BtOp::ReMark => at_mark(rex, prog, scan),
        BtOp::ReVisual => nomatch_unless(reg_match_visual(rex)),

        // `\%23l`, `\%23c`, `\%23v` and their `<`/`>` forms. A line
        // number only means something in a multi-line match.
        BtOp::ReLnum => {
            nomatch_unless(rex.multi() && re_num_cmp(prog, rex.buf_lnum() as uint32_t, scan))
        }
        BtOp::ReCol => nomatch_unless(re_num_cmp(
            prog,
            (rex.col() as uint32_t).wrapping_add(1),
            scan,
        )),
        BtOp::ReVcol => nomatch_unless(re_num_cmp(prog, virtual_column(rex).wrapping_add(1), scan)),

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
        BtOp::Digit => return take(ri(ByteClass::DIGIT)),
        BtOp::Ndigit => return take(c != NUL && !ri(ByteClass::DIGIT)),
        BtOp::Hex => return take(ri(ByteClass::HEX)),
        BtOp::Nhex => return take(c != NUL && !ri(ByteClass::HEX)),
        BtOp::Octal => return take(ri(ByteClass::OCTAL)),
        BtOp::Noctal => return take(c != NUL && !ri(ByteClass::OCTAL)),
        BtOp::Word => return take(ri(ByteClass::WORD)),
        BtOp::Nword => return take(c != NUL && !ri(ByteClass::WORD)),
        BtOp::Head => return take(ri(ByteClass::HEAD)),
        BtOp::Nhead => return take(c != NUL && !ri(ByteClass::HEAD)),
        BtOp::Alpha => return take(ri(ByteClass::ALPHA)),
        BtOp::Nalpha => return take(c != NUL && !ri(ByteClass::ALPHA)),
        BtOp::Lower => return take(ri(ByteClass::LOWER)),
        BtOp::Nlower => return take(c != NUL && !ri(ByteClass::LOWER)),
        BtOp::Upper => return take(ri(ByteClass::UPPER)),
        BtOp::Nupper => return take(c != NUL && !ri(ByteClass::UPPER)),

        BtOp::Exactly => exactly(rex, prog, scan, next),
        BtOp::Anyof | BtOp::Anybut => collection(rex, prog.operand(scan), c, op == BtOp::Anyof),
        BtOp::Multibytecode => multibyte(rex, prog.operand(scan)),

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
        // `behind_pos` is this engine's own saved position.
        BtOp::Bhpos => nomatch_unless(rex.is_at(rex.behind_pos().pos)),

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
fn cursor_of(rex: Rex) -> Option<Pos> {
    rex.reg_win().map(|win| win.w_cursor)
}

/// `vim_is_ident_char`, `vim_isfilec` and `vim_isprintc` are pure tests on a code
/// point that read only option state.
fn is_ident_char(c: c_int) -> bool {
    vim_is_ident_char(c)
}

fn is_file_char(c: c_int) -> bool {
    vim_isfilec(c)
}

fn is_printable(c: c_int) -> bool {
    vim_isprintc(c)
}

fn nomatch_unless(ok: bool) -> c_int {
    if ok { RA_CONT } else { RA_NOMATCH }
}

/// The character class of the character at the cursor, per the buffer's
/// 'iskeyword'.
fn char_class(rex: Rex) -> c_int {
    class_in(rex.rest(), rex.reg_buf())
}

/// `\%23v` compares against this: the screen column the cursor sits in.
fn virtual_column(rex: Rex) -> uint32_t {
    let wp = rex.reg_win().unwrap_or_else(Win::current);
    let mut lnum: LineNr = if rex.multi() { rex.buf_lnum() } else { 1 };
    // A string match has no line numbers, and a multi-line match may be
    // running over a line that has since been deleted.
    if rex.multi() && (lnum <= 0 || lnum > wp.buffer().b_ml.ml_line_count) {
        lnum = 1;
    }
    win_linetabsize_bytes(wp, lnum, rex.line(), rex.col()) as uint32_t
}

/// `\%'m`, `\%<'m`, `\%>'m`: is the cursor at, before or after mark `m`?
fn at_mark(rex: Rex, prog: BtProg<'_>, scan: usize) -> c_int {
    // The operand is the mark name and the comparison character.
    let (mark, cmp) = (
        c_int::from(prog.byte(scan, 3)),
        c_int::from(prog.byte(scan, 4)),
    );
    let col = if rex.multi() { rex.col() } else { 0 };
    // `mark_lookup` computes a motion mark (`'{`, `'(`) on the spot.
    let fm = mark_lookup(rex.reg_buf(), Win::current(), kMarkBufLocal, mark);
    // Looking the mark up can move the buffer's line, so fetch it again.
    if rex.multi() {
        rex.seek(reg_getline(rex, rex.lnum()).unwrap_or(b"\0"), col);
    }
    let Some(fm) = fm else {
        return RA_NOMATCH;
    };
    let pos = fm.mark;
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
fn exactly(rex: Rex, prog: BtProg<'_>, scan: usize, next: usize) -> c_int {
    let opnd = prog.operand(scan);
    if opnd[0] != rex.byte() && !rex.reg_ic() {
        return RA_NOMATCH;
    }
    if c_int::from(opnd[0]) == NUL {
        return RA_CONT;
    }
    let mut len;
    // A one-byte operand with no case folding needs no compare: the
    // first-byte test above already settled it.
    if c_int::from(opnd[1]) == NUL && !rex.reg_ic() {
        len = 1;
    } else {
        len = nul_terminated_len(opnd) as c_int;
        if cstrncmp(rex, opnd, rex.rest(), &mut len) != 0 {
            return RA_NOMATCH;
        }
    }
    // A combining character right after the match means the input has a
    // different grapheme here, unless 'reg_icombine' or a following `\%C`
    // says to ignore combining characters.
    let at = rex.input();
    if composes_at(rex.line(), at, at + len as usize, None)
        && !rex.reg_icombine()
        && prog.op_byte(next) != BtOp::ReComposing.code() as uint8_t
    {
        return RA_NOMATCH;
    }
    rex.advance(len);
    RA_CONT
}

/// `[abc]` / `[^abc]`. The base character has to be in (or out of) the set,
/// and any combining characters the set's own entry carries must follow it.
fn collection(rex: Rex, operand: &[u8], c: c_int, positive: bool) -> c_int {
    if c == NUL {
        return RA_NOMATCH;
    }
    let found = cstrchr(rex, operand, c);
    if found.is_none() == positive {
        return RA_NOMATCH;
    }
    // The set entry may itself be a grapheme; its combining part has to
    // match the input's byte for byte.
    let base = char_len_at(operand, 0);
    let combining = cluster_len_at(operand, 0) - base;
    rex.advance(rex.base_len());
    let mut status = RA_CONT;
    if combining != 0 {
        if (0..combining).any(|i| operand[base + i] != rex.byte_at(i)) {
            status = RA_NOMATCH;
        }
        // Upstream advances even when the tail did not match; the status
        // is what decides the outcome.
        rex.advance(combining as c_int);
    }
    status
}

/// One multibyte character, possibly a bare combining character — which
/// matches anywhere in the grapheme at the cursor.
fn multibyte(rex: Rex, opnd: &[u8]) -> c_int {
    let mut len = cluster_len_at(opnd, 0) as c_int;
    if len < 2 {
        return RA_NOMATCH;
    }
    let opndc = char_at_nul(opnd, 0);
    if utf_iscomposing_legacy(opndc) {
        // A bare combining character matches wherever it appears in the
        // grapheme at the cursor.
        let (line, input) = (rex.line(), rex.input());
        let mut status = RA_NOMATCH;
        let mut i = 0;
        while c_int::from(line[input + i]) != NUL {
            let inpc = char_at_nul(line, input + i);
            if !utf_iscomposing_legacy(inpc) {
                // The base character is allowed to be the first thing
                // looked at; anything past it ends the grapheme.
                if i > 0 {
                    break;
                }
            } else if opndc == inpc {
                // Include the combining characters that follow.
                len = (i + cluster_len_at(line, input + i)) as c_int;
                // RA_MATCH, not RA_CONT: upstream ends the whole pattern
                // here. Preserved rather than "fixed".
                status = RA_MATCH;
                break;
            }
            i += char_len_at(line, input + i);
        }
        // Upstream advances whether or not it found the character.
        rex.advance(len);
        return status;
    }
    if cstrncmp(rex, opnd, rex.rest(), &mut len) != 0 {
        return RA_NOMATCH;
    }
    rex.advance(len);
    RA_CONT
}

/// `\1`..`\9`: the text group `no` captured, again.
fn back_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_subexpr(rex);
    let kind = rex.pos_kind();
    let (start, end) = (rex.start(no as usize).get(), rex.end(no as usize).get());
    let mut len = 0;
    let mut status = RA_CONT;
    if !start.is_set(kind) || !end.is_set(kind) {
        // Never captured: an empty match.
    } else if !rex.multi() || (start.as_pos().lnum == rex.lnum() && end.as_pos().lnum == rex.lnum())
    {
        // Wholly on this line: a plain comparison.
        let (start, end) = (start.as_pos().col, end.as_pos().col);
        len = end - start;
        let captured = usize::try_from(start)
            .ok()
            .and_then(|at| rex.line().get(at..))
            .unwrap_or(b"\0");
        if cstrncmp(rex, captured, rex.rest(), &mut len) != 0 {
            status = RA_NOMATCH;
        }
    } else {
        let (start, end) = (start.as_pos(), end.as_pos());
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
    // Upstream advances by whatever `len` ended up as, match or not.
    rex.advance(len);
    status
}

/// `\z1`..`\z9`: the text the enclosing syntax region captured. Missing
/// captures match the empty string rather than failing.
fn external_reference(rex: Rex, no: c_int) -> c_int {
    cleanup_zsubexpr(rex);
    rex.with_ext_capture(no as usize, |text| {
        let Some(text) = text else {
            return RA_CONT;
        };
        let mut len = text.count_bytes() as c_int;
        if cstrncmp(rex, text.to_bytes_with_nul(), rex.rest(), &mut len) != 0 {
            return RA_NOMATCH;
        }
        rex.advance(len);
        RA_CONT
    })
}
