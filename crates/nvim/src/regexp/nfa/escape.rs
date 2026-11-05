//! The two escapes that open a family of their own: `\z`, which is the
//! syntax highlighter's private capture group, and `\%`, which is everything
//! from `\%(` through the `\%23l` position assertions.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::narrow::const_len_as_int;
use crate::regexp::NfaOp;
use crate::regexp::RegCompiler;
use crate::regexp::low_byte;
use core::ffi::c_int;

use super::Parsed;
use super::atom::nfa_regatom;
use super::parse::nfa_reg;
use crate::ascii::ascii_isdigit;
use crate::regexp::{
    CompileError, INT32_MAX, REG_NPAREN, REG_ZPAREN, REX_SET, REX_USE, getchr, getdecchrs,
    gethexchrs, getoctchrs, pat_byte, peekchr, re_mult_next, spelled, unmagic,
};
use crate::types::{MB_MAXBYTES, NUL};

use crate::winlayer::Win;
/// `\z`: the highlighter's own captures, plus `\zs`/`\ze`.
pub(crate) fn z_atom(rc: &mut RegCompiler) -> Parsed {
    let c = unmagic(getchr(rc));
    // `u8::try_from` rather than `as u8`: a multibyte character after `\z`
    // must reach the default arm rather than alias one of these bytes.
    match u8::try_from(c) {
        Ok(b's') => {
            rc.post.emit_op(NfaOp::Zstart);
            re_mult_next(rc, false)?;
        }
        Ok(b'e') => {
            rc.post.emit_op(NfaOp::Zend);
            // The match end moves, so the matcher has to keep group 0's
            // end position rather than take it from where it stopped.
            rc.has_zend = true;
            re_mult_next(rc, true)?;
        }
        Ok(digit @ b'1'..=b'9') => {
            // A `\z1` back-reference only means something while a syntax
            // item's contained pattern is being matched.
            if rc.extmatch & REX_USE == 0 {
                return Err(CompileError::ZRefNotAllowed);
            }
            rc.post.emit_op(NfaOp::zref(usize::from(digit - b'0')));
            rc.has_z = REX_USE;
        }
        Ok(b'(') => {
            // And `\z(` only in the item that defines them.
            if rc.extmatch != REX_SET {
                return Err(CompileError::ZGroupNotAllowed);
            }
            nfa_reg(rc, REG_ZPAREN)?;
            rc.has_z = REX_SET;
        }
        _ => return Err(CompileError::UnknownZOperator(low_byte(unmagic(c)))),
    }
    Ok(())
}

/// `\%`: the non-capturing group, the character escapes, the boundary
/// assertions and the position family.
///
/// `save_prev_at_start` is the "still at the start of the pattern" flag from
/// before this atom was read; `\%23l` restores it, because a position
/// assertion consumes nothing and so does not move the start.
pub(crate) fn percent_atom(rc: &mut RegCompiler, save_prev_at_start: c_int) -> Parsed {
    let c = unmagic(getchr(rc));
    // `u8::try_from` rather than `as u8`: see `z_atom`.
    match u8::try_from(c) {
        Ok(b'(') => {
            nfa_reg(rc, REG_NPAREN)?;
            rc.post.emit_op(NfaOp::Nopen);
        }
        Ok(escape @ (b'd' | b'o' | b'x' | b'u' | b'U')) => return character_escape(rc, escape),
        Ok(b'^') => rc.post.emit_op(NfaOp::Bof),
        Ok(b'$') => rc.post.emit_op(NfaOp::Eof),
        Ok(b'#') => {
            // `\%#=1` selects an engine and is only legal at the very start
            // of the pattern, where `vim_regcomp` strips it; getting here
            // means it was somewhere else.
            if pat_byte(rc, 0) == b'=' && matches!(pat_byte(rc, 1), b'0'..=b'2') {
                return Err(CompileError::EngineNotAtStart(pat_byte(rc, 1)));
            }
            rc.post.emit_op(NfaOp::Cursor);
        }
        Ok(b'V') => rc.post.emit_op(NfaOp::Visual),
        Ok(b'C') => rc.post.emit_op(NfaOp::AnyComposing),
        Ok(b'[') => return optional_sequence(rc),
        _ => return position_atom(rc, c, save_prev_at_start),
    }
    Ok(())
}

/// `\%d123`, `\%o17`, `\%x2a`, `\%u20ac`, `\%U0001f600`: a character by its
/// code point.
fn character_escape(rc: &mut RegCompiler, escape: u8) -> Parsed {
    let nr = match escape {
        b'd' => getdecchrs(rc),
        b'o' => getoctchrs(rc),
        b'x' => gethexchrs(rc, 2),
        b'u' => gethexchrs(rc, 4),
        _ => gethexchrs(rc, 8),
    };
    if !(0..=i64::from(INT32_MAX)).contains(&nr) {
        return Err(CompileError::InvalidNumber(spelled(rc)));
    }
    // A NUL cannot be matched as itself; in a pattern it stands for a line
    // break, as it does everywhere else in the engine.
    rc.post.emit(if nr == 0 {
        0xa
    } else {
        c_int::try_from(nr).expect("checked against INT32_MAX")
    });
    Ok(())
}

/// `\%[abc]`: a sequence in which every trailing part is optional.
///
/// Each member is one atom, and `NFA_OPT_CHARS` carries how many of them
/// there were.
fn optional_sequence(rc: &mut RegCompiler) -> Parsed {
    let mut n = 0;
    loop {
        let c = peekchr(rc);
        if c == c_int::from(b']') {
            break;
        }
        if c == NUL {
            return Err(CompileError::MissingOptionalEnd(spelled(rc)));
        }
        nfa_regatom(rc)?;
        n += 1;
    }
    getchr(rc);
    if n == 0 {
        return Err(CompileError::EmptyOptional(spelled(rc)));
    }
    rc.post.emit_op(NfaOp::OptChars);
    rc.post.emit(n);
    rc.post.emit_op(NfaOp::Nopen);
    Ok(())
}

/// Which side of the given position a match has to be on.
fn compare(cmp: c_int, lt: c_int, gt: c_int, at: c_int) -> c_int {
    if cmp == c_int::from(b'<') {
        lt
    } else if cmp == c_int::from(b'>') {
        gt
    } else {
        at
    }
}

/// The `\%23l`, `\%23c`, `\%23v` and `\%'m` assertions, in their bare,
/// `\%<` and `\%>` forms, and with `.` standing for the cursor's own line,
/// column or virtual column.
fn position_atom(rc: &mut RegCompiler, cmp: c_int, save_prev_at_start: c_int) -> Parsed {
    let mut c = cmp;
    if c == c_int::from(b'<') || c == c_int::from(b'>') {
        c = getchr(rc);
    }
    let cur = unmagic(c) == c_int::from(b'.');
    if cur {
        c = getchr(rc);
    }

    let mut n: i64 = 0;
    let mut got_digit = false;
    while ascii_isdigit(c) {
        if cur {
            return Err(CompileError::NumberAfterDot(low_byte(unmagic(c))));
        }
        if n > i64::from((INT32_MAX - (c - c_int::from(b'0'))) / 10) {
            return Err(CompileError::PositionTooLarge);
        }
        n = n * 10 + i64::from(c - c_int::from(b'0'));
        c = getchr(rc);
        got_digit = true;
    }

    if let Ok(unit @ (b'l' | b'c' | b'v')) = u8::try_from(c) {
        if !cur && !got_digit {
            return Err(CompileError::MissingValue(low_byte(unmagic(c))));
        }
        let mut limit = INT32_MAX;
        match unit {
            b'l' => {
                if cur {
                    n = cursor_lnum();
                }
                rc.post.emit(compare(
                    cmp,
                    NfaOp::LnumLt.code(),
                    NfaOp::LnumGt.code(),
                    NfaOp::Lnum.code(),
                ));
                // A line assertion matches nothing, so a `^` after it is
                // still at the start of the pattern.
                if save_prev_at_start != 0 {
                    rc.at_start = 1;
                }
            }
            b'c' => {
                if cur {
                    n = cursor_col() + 1;
                }
                rc.post.emit(compare(
                    cmp,
                    NfaOp::ColLt.code(),
                    NfaOp::ColGt.code(),
                    NfaOp::Col.code(),
                ));
            }
            _ => {
                if cur {
                    n = cursor_vcol() + 1;
                }
                rc.post.emit(compare(
                    cmp,
                    NfaOp::VcolLt.code(),
                    NfaOp::VcolGt.code(),
                    NfaOp::Vcol.code(),
                ));
                // A virtual column is bounded by what a byte column can
                // expand to.
                limit = INT32_MAX / const_len_as_int(MB_MAXBYTES);
            }
        }
        if n >= i64::from(limit) {
            return Err(CompileError::PositionTooLarge);
        }
        rc.post
            .emit(c_int::try_from(n).expect("checked against the limit"));
        return Ok(());
    }

    if unmagic(c) == c_int::from(b'\'') && n == 0 {
        rc.post.emit(compare(
            cmp,
            NfaOp::MarkLt.code(),
            NfaOp::MarkGt.code(),
            NfaOp::Mark.code(),
        ));
        let c = getchr(rc);
        rc.post.emit(c);
        return Ok(());
    }

    Err(CompileError::UnknownPercentOperator(low_byte(unmagic(c))))
}

fn cursor_lnum() -> i64 {
    i64::from(Win::current().w_cursor.lnum)
}

fn cursor_col() -> i64 {
    i64::from(Win::current().w_cursor.col)
}

fn cursor_vcol() -> i64 {
    // The last column the character under the cursor covers.
    let win = Win::current();
    i64::from(win.virtual_vcol_span(win.cursor()).1)
}
