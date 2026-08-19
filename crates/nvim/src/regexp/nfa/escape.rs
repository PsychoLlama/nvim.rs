//! The two escapes that open a family of their own: `\z`, which is the
//! syntax highlighter's private capture group, and `\%`, which is everything
//! from `\%(` through the `\%23l` position assertions.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::atom::nfa_regatom;
use super::parse::nfa_reg;
use super::postfix;
use crate::ascii::ascii_isdigit;
use crate::main::{curwin, rc_did_emsg, reg_do_extmatch};
use crate::plines::getvvcol;
use crate::regexp::{
    INT32_MAX, NFA_ANY_COMPOSING, NFA_BOF, NFA_COL, NFA_COL_GT, NFA_COL_LT, NFA_CURSOR, NFA_EOF,
    NFA_LNUM, NFA_LNUM_GT, NFA_LNUM_LT, NFA_MARK, NFA_MARK_GT, NFA_MARK_LT, NFA_NOPEN,
    NFA_OPT_CHARS, NFA_VCOL, NFA_VCOL_GT, NFA_VCOL_LT, NFA_VISUAL, NFA_ZEND, NFA_ZREF1, NFA_ZSTART,
    REG_NPAREN, REG_ZPAREN, REX_SET, REX_USE, Rex, at_start, getchr, getdecchrs, gethexchrs,
    getoctchrs, magic_prefix, pat_byte, peekchr, re_has_z, re_mult_next, unmagic,
};
use crate::semsg;
use crate::types::{FAIL, MB_MAXBYTES, NUL, OK, colnr_T};

/// `\z`: the highlighter's own captures, plus `\zs`/`\ze`.
pub(crate) fn z_atom(rex: Rex) -> c_int {
    let c = unmagic(getchr());
    // `u8::try_from` rather than `as u8`: a multibyte character after `\z`
    // must reach the default arm rather than alias one of these bytes.
    match u8::try_from(c) {
        Ok(b's') => {
            postfix::emit(NFA_ZSTART);
            if !re_mult_next("\\zs") {
                return FAIL;
            }
        }
        Ok(b'e') => {
            postfix::emit(NFA_ZEND);
            // The match end moves, so the matcher has to keep group 0's
            // end position rather than take it from where it stopped.
            rex.set_nfa_has_zend(1);
            if !re_mult_next("\\ze") {
                return FAIL;
            }
        }
        Ok(b'1'..=b'9') => {
            // A `\z1` back-reference only means something while a syntax
            // item's contained pattern is being matched.
            if reg_do_extmatch.get() & REX_USE == 0 {
                semsg!("E67: \\z1 - \\z9 not allowed here");
                rc_did_emsg.set(true);
                return FAIL;
            }
            postfix::emit(NFA_ZREF1 + (c - b'1' as c_int));
            re_has_z.set(REX_USE);
        }
        Ok(b'(') => {
            // And `\z(` only in the item that defines them.
            if reg_do_extmatch.get() != REX_SET {
                semsg!("E66: \\z( not allowed here");
                rc_did_emsg.set(true);
                return FAIL;
            }
            if nfa_reg(rex, REG_ZPAREN) == FAIL {
                return FAIL;
            }
            re_has_z.set(REX_SET);
        }
        _ => {
            let c = unmagic(c) as u8 as char;
            semsg!("E867: (NFA) Unknown operator '\\z{c}'");
            return FAIL;
        }
    }
    OK
}

/// `\%`: the non-capturing group, the character escapes, the boundary
/// assertions and the position family.
///
/// `save_prev_at_start` is the "still at the start of the pattern" flag from
/// before this atom was read; `\%23l` restores it, because a position
/// assertion consumes nothing and so does not move the start.
pub(crate) fn percent_atom(rex: Rex, save_prev_at_start: c_int) -> c_int {
    let c = unmagic(getchr());
    // `u8::try_from` rather than `as u8`: see `z_atom`.
    match u8::try_from(c) {
        Ok(b'(') => {
            if nfa_reg(rex, REG_NPAREN) == FAIL {
                return FAIL;
            }
            postfix::emit(NFA_NOPEN);
        }
        Ok(escape @ (b'd' | b'o' | b'x' | b'u' | b'U')) => return character_escape(escape),
        Ok(b'^') => postfix::emit(NFA_BOF),
        Ok(b'$') => postfix::emit(NFA_EOF),
        Ok(b'#') => {
            // `\%#=1` selects an engine and is only legal at the very start
            // of the pattern, where `vim_regcomp` strips it; getting here
            // means it was somewhere else.
            if pat_byte(0) == b'=' && matches!(pat_byte(1), b'0'..=b'2') {
                let which = pat_byte(1) as char;
                semsg!("E1281: Atom '\\%#={which}' must be at the start of the pattern");
                return FAIL;
            }
            postfix::emit(NFA_CURSOR);
        }
        Ok(b'V') => postfix::emit(NFA_VISUAL),
        Ok(b'C') => postfix::emit(NFA_ANY_COMPOSING),
        Ok(b'[') => return optional_sequence(rex),
        _ => return position_atom(c, save_prev_at_start),
    }
    OK
}

/// `\%d123`, `\%o17`, `\%x2a`, `\%u20ac`, `\%U0001f600`: a character by its
/// code point.
fn character_escape(escape: u8) -> c_int {
    let nr = match escape {
        b'd' => getdecchrs(),
        b'o' => getoctchrs(),
        b'x' => gethexchrs(2),
        b'u' => gethexchrs(4),
        _ => gethexchrs(8),
    };
    if !(0..=INT32_MAX as i64).contains(&nr) {
        let prefix = magic_prefix();
        semsg!("E678: Invalid character after {prefix}%[dxouU]");
        rc_did_emsg.set(true);
        return FAIL;
    }
    // A NUL cannot be matched as itself; in a pattern it stands for a line
    // break, as it does everywhere else in the engine.
    postfix::emit(if nr == 0 { 0xa } else { nr as c_int });
    OK
}

/// `\%[abc]`: a sequence in which every trailing part is optional.
///
/// Each member is one atom, and `NFA_OPT_CHARS` carries how many of them
/// there were.
fn optional_sequence(rex: Rex) -> c_int {
    let mut n = 0;
    loop {
        let c = peekchr();
        if c == b']' as c_int {
            break;
        }
        if c == NUL {
            let prefix = magic_prefix();
            semsg!("E69: Missing ] after {prefix}%[");
            rc_did_emsg.set(true);
            return FAIL;
        }
        if nfa_regatom(rex) == FAIL {
            return FAIL;
        }
        n += 1;
    }
    getchr();
    if n == 0 {
        let prefix = magic_prefix();
        semsg!("E70: Empty {prefix}%[]");
        rc_did_emsg.set(true);
        return FAIL;
    }
    postfix::emit(NFA_OPT_CHARS);
    postfix::emit(n);
    postfix::emit(NFA_NOPEN);
    OK
}

/// Which side of the given position a match has to be on.
fn compare(cmp: c_int, lt: c_int, gt: c_int, at: c_int) -> c_int {
    if cmp == b'<' as c_int {
        lt
    } else if cmp == b'>' as c_int {
        gt
    } else {
        at
    }
}

/// The `\%23l`, `\%23c`, `\%23v` and `\%'m` assertions, in their bare,
/// `\%<` and `\%>` forms, and with `.` standing for the cursor's own line,
/// column or virtual column.
fn position_atom(cmp: c_int, save_prev_at_start: c_int) -> c_int {
    let mut c = cmp;
    if c == b'<' as c_int || c == b'>' as c_int {
        c = getchr();
    }
    let cur = unmagic(c) == b'.' as c_int;
    if cur {
        c = getchr();
    }

    let mut n: i64 = 0;
    let mut got_digit = false;
    while ascii_isdigit(c) {
        if cur {
            let c = unmagic(c) as u8 as char;
            semsg!("E1204: No Number allowed after .: '\\%{c}'");
            return FAIL;
        }
        if n > ((INT32_MAX - (c - b'0' as c_int)) / 10) as i64 {
            semsg!("E951: \\% value too large");
            return FAIL;
        }
        n = n * 10 + (c - b'0' as c_int) as i64;
        c = getchr();
        got_digit = true;
    }

    if let Ok(unit @ (b'l' | b'c' | b'v')) = u8::try_from(c) {
        if !cur && !got_digit {
            let c = unmagic(c) as u8 as char;
            semsg!("E1273: (NFA regexp) missing value in '\\%{c}'");
            return FAIL;
        }
        let mut limit = INT32_MAX;
        match unit {
            b'l' => {
                if cur {
                    n = cursor_lnum();
                }
                postfix::emit(compare(cmp, NFA_LNUM_LT, NFA_LNUM_GT, NFA_LNUM));
                // A line assertion matches nothing, so a `^` after it is
                // still at the start of the pattern.
                if save_prev_at_start != 0 {
                    at_start.set(1);
                }
            }
            b'c' => {
                if cur {
                    n = cursor_col() + 1;
                }
                postfix::emit(compare(cmp, NFA_COL_LT, NFA_COL_GT, NFA_COL));
            }
            _ => {
                if cur {
                    n = cursor_vcol() + 1;
                }
                postfix::emit(compare(cmp, NFA_VCOL_LT, NFA_VCOL_GT, NFA_VCOL));
                // A virtual column is bounded by what a byte column can
                // expand to.
                limit = INT32_MAX / MB_MAXBYTES as c_int;
            }
        }
        if n >= limit as i64 {
            semsg!("E951: \\% value too large");
            return FAIL;
        }
        postfix::emit(n as c_int);
        return OK;
    }

    if unmagic(c) == b'\'' as c_int && n == 0 {
        postfix::emit(compare(cmp, NFA_MARK_LT, NFA_MARK_GT, NFA_MARK));
        postfix::emit(getchr());
        return OK;
    }

    let c = unmagic(c) as u8 as char;
    semsg!("E867: (NFA) Unknown operator '\\%{c}'");
    FAIL
}

fn cursor_lnum() -> i64 {
    // SAFETY: `curwin` is the current window.
    unsafe { (*curwin.get()).w_cursor.lnum as i64 }
}

fn cursor_col() -> i64 {
    // SAFETY: as `cursor_lnum`.
    unsafe { (*curwin.get()).w_cursor.col as i64 }
}

fn cursor_vcol() -> i64 {
    // SAFETY: as `cursor_lnum`; `getvvcol` writes only through the pointers
    // it is given.
    unsafe {
        let mut vcol: colnr_T = 0;
        getvvcol(
            curwin.get(),
            &raw mut (*curwin.get()).w_cursor,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut vcol,
        );
        vcol as i64
    }
}
