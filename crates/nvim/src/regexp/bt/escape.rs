//! The two escape families whose second character decides everything: `\z`
//! (the external submatches a syntax item shares with its region) and `\%`
//! (position assertions, `\%(` groups, the `\%[...]` optional sequence and
//! the `\%d123` character escapes).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::compile::Node;
use crate::regexp::RegCompiler;
use crate::regexp::low_byte;
use core::ffi::c_int;

use super::atom::{denied_in_optional_sequence, regatom};
use super::compile::use_multibytecode;
use super::op::BtOp;
use super::piece::reg;
use crate::ascii::ascii_isdigit;
use crate::os::libc::uint32_t;
use crate::regexp::{
    CompileError, Compiled, HASLOOKBH, HASNL, HASWIDTH, REG_NPAREN, REG_ZPAREN, REX_SET, REX_USE,
    SIMPLE, SPSTART, getchr, getdecchrs, gethexchrs, getoctchrs, pat_byte, re_mult_next, spelled,
    ungetchr, unmagic,
};
use crate::types::NUL;

use crate::winlayer::Win;
/// `\z(`, `\z1`..`\z9`, `\zs` and `\ze`.
pub(crate) fn z_atom(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    match low_byte(unmagic(getchr(rc))) {
        b'(' => {
            // Only a syntax pattern may *define* an external submatch.
            if rc.extmatch & REX_SET == 0 {
                return Err(CompileError::ZGroupNotAllowed);
            }
            denied_in_optional_sequence(rc)?;
            let mut flags = 0;
            let ret = reg(rc, REG_ZPAREN, &mut flags)?;
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            rc.has_z = REX_SET;
            Ok(ret)
        }
        c @ b'1'..=b'9' => {
            // ...and only a pattern run inside such a region may use one.
            if rc.extmatch & REX_USE == 0 {
                return Err(CompileError::ZRefNotAllowed);
            }
            let ret = rc.code.node(BtOp::ZREF[usize::from(c - b'0')]);
            rc.has_z = REX_USE;
            Ok(ret)
        }
        // `\zs`/`\ze` move the reported match start/end without consuming
        // anything, so they are the group-0 open and close nodes.
        b's' => {
            let ret = rc.code.node(BtOp::Mopen);
            re_mult_next(rc, false)?;
            Ok(ret)
        }
        b'e' => {
            let ret = rc.code.node(BtOp::Mclose);
            re_mult_next(rc, true)?;
            Ok(ret)
        }
        _ => Err(CompileError::InvalidAfterZ),
    }
}

/// The `\%` family.
///
/// `save_prev_at_start` is `prev_at_start` from before this atom was read:
/// `\%23l` and friends consume no input, so a `^` after one is still at the
/// start of the pattern.
pub(crate) fn percent_atom(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    save_prev_at_start: c_int,
) -> Compiled<Node> {
    let c = unmagic(getchr(rc));
    match low_byte(c) {
        b'(' => {
            denied_in_optional_sequence(rc)?;
            let mut flags = 0;
            let ret = reg(rc, REG_NPAREN, &mut flags)?;
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            Ok(ret)
        }
        b'^' => Ok(rc.code.node(BtOp::ReBof)),
        b'$' => Ok(rc.code.node(BtOp::ReEof)),
        b'#' => {
            // `\%#=1` selects an engine and is only legal at the very start
            // of the pattern, where `vim_regcomp` strips it; getting here
            // means it was somewhere else.
            if pat_byte(rc, 0) == b'=' && matches!(pat_byte(rc, 1), b'0'..=b'2') {
                return Err(CompileError::EngineNotAtStart(pat_byte(rc, 1)));
            }
            Ok(rc.code.node(BtOp::Cursor))
        }
        b'V' => Ok(rc.code.node(BtOp::ReVisual)),
        b'C' => Ok(rc.code.node(BtOp::ReComposing)),
        b'[' => optional_sequence(rc, flagp),
        b'd' | b'o' | b'x' | b'u' | b'U' => character_escape(rc, flagp, c),
        _ => position_atom(rc, c, save_prev_at_start),
    }
}

/// `\%[abc]`: match as many of the members as are there, in order, and
/// succeed on any prefix — including the empty one.
///
/// Built as a chain of branches: each member's branch falls through to the
/// next, and every branch's tail lands on the same trailing `NOTHING`, so
/// stopping early is always an option.
fn optional_sequence(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    denied_in_optional_sequence(rc)?;
    let mut first: Option<Node> = None;
    let mut last: Option<Node> = None;

    loop {
        let c = getchr(rc);
        if c == ']' as c_int {
            break;
        }
        if c == NUL {
            return Err(CompileError::MissingOptionalEnd(spelled(rc)));
        }
        let br = rc.code.node(BtOp::Branch);
        match last {
            None => first = Some(br),
            Some(prev) => {
                rc.code.tail(prev, br);
                if rc.code.too_long {
                    return Err(CompileError::TooLong);
                }
            }
        }
        ungetchr(rc);
        // Each member is exactly one atom; `one_exactly` is what stops a
        // literal run from swallowing the rest of the sequence.
        rc.one_exactly = 1;
        let member = regatom(rc, flagp);
        rc.one_exactly = 0;
        last = Some(member?);
    }

    let (Some(ret), Some(lastnode)) = (first, last) else {
        return Err(CompileError::EmptyOptional(spelled(rc)));
    };

    let lastbranch = rc.code.node(BtOp::Branch);
    let nothing = rc.code.node(BtOp::Nothing);
    rc.code.tail(lastnode, nothing);
    rc.code.tail(lastbranch, nothing);
    // Point every member's branch at the empty alternative that follows
    // the whole sequence; a branch's operand is the member itself.
    let mut br = ret;
    while br != lastnode {
        if rc.code.opcode_at(br) == BtOp::Branch.byte() {
            rc.code.tail(br, lastbranch);
            if rc.code.too_long {
                return Err(CompileError::TooLong);
            }
            br = br.operand();
        } else {
            let Some(next) = rc.code.next(br) else {
                break;
            };
            br = next;
        }
    }
    *flagp &= !(HASWIDTH | SIMPLE);
    Ok(ret)
}

/// `\%d123`, `\%o40`, `\%x2f`, `\%u1234`, `\%U1234abcd`: one character named
/// by its code point.
fn character_escape(rc: &mut RegCompiler, flagp: &mut c_int, c: c_int) -> Compiled<Node> {
    let i = match low_byte(c) {
        b'd' => getdecchrs(rc),
        b'o' => getoctchrs(rc),
        b'x' => gethexchrs(rc, 2),
        b'u' => gethexchrs(rc, 4),
        b'U' => gethexchrs(rc, 8),
        _ => -1,
    };
    let Ok(i) = c_int::try_from(i) else {
        return Err(CompileError::InvalidNumber(spelled(rc)));
    };
    if i < 0 {
        return Err(CompileError::InvalidNumber(spelled(rc)));
    }
    let ret = if use_multibytecode(rc, i) {
        rc.code.node(BtOp::Multibytecode)
    } else {
        rc.code.node(BtOp::Exactly)
    };
    // A NUL in the pattern stands for a newline: the program is a C string,
    // so it cannot hold a NUL byte.
    if i == 0 {
        rc.code.byte(0xa);
    } else {
        rc.code.char(i);
    }
    rc.code.byte(NUL);
    *flagp |= HASWIDTH;
    Ok(ret)
}

/// The position assertions: `\%23l`, `\%<23c`, `\%>23v`, `\%.l` (the cursor's
/// own line/column) and `\%'m` (a mark).
fn position_atom(rc: &mut RegCompiler, first: c_int, save_prev_at_start: c_int) -> Compiled<Node> {
    if (ascii_isdigit(first) || matches!(low_byte(first), b'<' | b'>' | b'\'' | b'.'))
        && let Some(node) = compare_atom(rc, first, save_prev_at_start)
    {
        return node;
    }
    Err(CompileError::InvalidAfterPercent(spelled(rc)))
}

/// The body of [`position_atom`]: `None` means the escape did not turn out to
/// be a position assertion after all, and E71 is the answer.
fn compare_atom(
    rc: &mut RegCompiler,
    first: c_int,
    save_prev_at_start: c_int,
) -> Option<Compiled<Node>> {
    // `<` and `>` make the test "before" and "after"; the node stores the
    // character itself as the comparison.
    let cmp = first;
    let mut c = first;
    if matches!(low_byte(cmp), b'<' | b'>') {
        c = getchr(rc);
    }
    // `\%.l` is "the cursor's line", read now rather than at match time.
    let mut cur = false;
    if unmagic(c) == c_int::from(b'.') {
        cur = true;
        c = getchr(rc);
    }
    let mut n: uint32_t = 0;
    let mut got_digit = false;
    while ascii_isdigit(c) {
        got_digit = true;
        n = n
            .wrapping_mul(10)
            .wrapping_add((c - c_int::from(b'0')).cast_unsigned());
        c = getchr(rc);
    }

    if unmagic(c) == c_int::from(b'\'') && n == 0 {
        // `\%'m`: the position of mark m.
        let c = getchr(rc);
        let ret = rc.code.node(BtOp::ReMark);
        rc.code.byte(c);
        rc.code.byte(cmp);
        return Some(Ok(ret));
    }
    if !matches!(low_byte(c), b'l' | b'c' | b'v') || !(cur || got_digit) {
        return None;
    }
    if cur && n != 0 {
        return Some(Err(CompileError::NumberAfterDot(low_byte(unmagic(c)))));
    }

    let ret = match low_byte(c) {
        b'l' => {
            if cur {
                n = cursor_value(b'l');
            }
            let ret = rc.code.node(BtOp::ReLnum);
            // A line assertion matches an empty string, so a `^` after it is
            // still at the start of the pattern.
            if save_prev_at_start != 0 {
                rc.at_start = 1;
            }
            ret
        }
        b'c' => {
            if cur {
                n = cursor_value(b'c');
            }
            rc.code.node(BtOp::ReCol)
        }
        _ => {
            if cur {
                n = cursor_value(b'v');
            }
            rc.code.node(BtOp::ReVcol)
        }
    };
    rc.code.number(n);
    rc.code.byte(cmp);
    Some(Ok(ret))
}

/// The cursor's own line, column or virtual column, as `\%.l`, `\%.c` and
/// `\%.v` read it at compile time. Columns are one-based in a pattern.
fn cursor_value(kind: u8) -> uint32_t {
    let win = Win::current();
    match kind {
        b'l' => win.w_cursor.lnum.get().cast_unsigned(),
        b'c' => win.w_cursor.col.get().cast_unsigned() + 1,
        // The last screen column of the character under the cursor.
        _ => {
            win.virtual_vcol_span_at(win.w_cursor)
                .1
                .get()
                .cast_unsigned()
                + 1
        }
    }
}
