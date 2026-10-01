//! The two escape families whose second character decides everything: `\z`
//! (the external submatches a syntax item shares with its region) and `\%`
//! (position assertions, `\%(` groups, the `\%[...]` optional sequence and
//! the `\%d123` character escapes).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::atom::{denied_in_optional_sequence, regatom};
use super::compile::{chain_next, regc, regmbc, regnode, regnr, regtail, use_multibytecode};
use super::op::BtOp;
use super::piece::reg;
use crate::ascii::ascii_isdigit;
use crate::plines::getvvcol;
use crate::regexp::state::{rc_did_emsg, reg_do_extmatch};
use crate::regexp::{
    HASLOOKBH, HASNL, HASWIDTH, INT_MAX, JUST_CALC_SIZE, REG_NPAREN, REG_ZPAREN, REX_SET, REX_USE,
    SIMPLE, SPSTART, getchr, getdecchrs, gethexchrs, getoctchrs, magic_prefix, pat_byte,
    re_mult_next, ungetchr, unmagic,
};
use crate::semsg;
use crate::types::{ColNr, NUL, int64_t, uint8_t, uint32_t};

use crate::winlayer::Win;
/// `\z(`, `\z1`..`\z9`, `\zs` and `\ze`.
pub(crate) fn z_atom(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    match unmagic(getchr(rc)) as u8 {
        b'(' => {
            // Only a syntax pattern may *define* an external submatch.
            if reg_do_extmatch.get() & REX_SET == 0 {
                semsg!("E66: \\z( not allowed here");
                rc_did_emsg.set(true);
                return core::ptr::null_mut();
            }
            if denied_in_optional_sequence(rc) {
                return core::ptr::null_mut();
            }
            let mut flags = 0;
            let ret = reg(rc, REG_ZPAREN, &mut flags);
            if ret.is_null() {
                return ret;
            }
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            rc.has_z = REX_SET;
            ret
        }
        c @ b'1'..=b'9' => {
            // ...and only a pattern run inside such a region may use one.
            if reg_do_extmatch.get() & REX_USE == 0 {
                semsg!("E67: \\z1 - \\z9 not allowed here");
                rc_did_emsg.set(true);
                return core::ptr::null_mut();
            }
            let ret = regnode(rc, BtOp::ZREF[usize::from(c - b'0')]);
            rc.has_z = REX_USE;
            ret
        }
        // `\zs`/`\ze` move the reported match start/end without consuming
        // anything, so they are the group-0 open and close nodes.
        b's' => {
            let ret = regnode(rc, BtOp::Mopen);
            if re_mult_next(rc, "\\zs") {
                ret
            } else {
                core::ptr::null_mut()
            }
        }
        b'e' => {
            let ret = regnode(rc, BtOp::Mclose);
            if re_mult_next(rc, "\\ze") {
                ret
            } else {
                core::ptr::null_mut()
            }
        }
        _ => {
            semsg!("E68: Invalid character after \\z");
            rc_did_emsg.set(true);
            core::ptr::null_mut()
        }
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
) -> *mut uint8_t {
    let c = unmagic(getchr(rc));
    match c as u8 {
        b'(' => {
            if denied_in_optional_sequence(rc) {
                return core::ptr::null_mut();
            }
            let mut flags = 0;
            let ret = reg(rc, REG_NPAREN, &mut flags);
            if !ret.is_null() {
                *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            }
            ret
        }
        b'^' => regnode(rc, BtOp::ReBof),
        b'$' => regnode(rc, BtOp::ReEof),
        b'#' => {
            // `\%#=1` selects an engine and is only legal at the very start
            // of the pattern, where `vim_regcomp` strips it; getting here
            // means it was somewhere else.
            if pat_byte(rc, 0) == b'=' && matches!(pat_byte(rc, 1), b'0'..=b'2') {
                let which = pat_byte(rc, 1) as char;
                semsg!("E1281: Atom '\\%#={which}' must be at the start of the pattern");
                return core::ptr::null_mut();
            }
            regnode(rc, BtOp::Cursor)
        }
        b'V' => regnode(rc, BtOp::ReVisual),
        b'C' => regnode(rc, BtOp::ReComposing),
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
fn optional_sequence(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    if denied_in_optional_sequence(rc) {
        return core::ptr::null_mut();
    }
    let mut ret: *mut uint8_t = core::ptr::null_mut();
    let mut lastnode: *mut uint8_t = core::ptr::null_mut();

    loop {
        let c = getchr(rc);
        if c == ']' as c_int {
            break;
        }
        if c == NUL {
            let prefix = magic_prefix(rc);
            semsg!("E69: Missing ] after {prefix}%[");
            rc_did_emsg.set(true);
            return core::ptr::null_mut();
        }
        let br = regnode(rc, BtOp::Branch);
        if ret.is_null() {
            ret = br;
        } else {
            regtail(rc, lastnode, br);
            if rc.code.too_long != 0 {
                return core::ptr::null_mut();
            }
        }
        ungetchr(rc);
        // Each member is exactly one atom; `one_exactly` is what stops a
        // literal run from swallowing the rest of the sequence.
        rc.one_exactly = 1;
        lastnode = regatom(rc, flagp);
        rc.one_exactly = 0;
        if lastnode.is_null() {
            return core::ptr::null_mut();
        }
    }

    if ret.is_null() {
        let prefix = magic_prefix(rc);
        semsg!("E70: Empty {prefix}%[]");
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }

    let lastbranch = regnode(rc, BtOp::Branch);
    let mut br = regnode(rc, BtOp::Nothing);
    if ret != JUST_CALC_SIZE {
        regtail(rc, lastnode, br);
        regtail(rc, lastbranch, br);
        // Point every member's branch at the empty alternative that follows
        // the whole sequence.
        br = ret;
        // SAFETY: `br` walks nodes of the program just written; stepping
        // three bytes past a branch lands on its operand.
        while br != lastnode {
            if unsafe { *br } == BtOp::Branch.code() as uint8_t {
                regtail(rc, br, lastbranch);
                if rc.code.too_long != 0 {
                    return core::ptr::null_mut();
                }
                br = unsafe { br.add(3) };
            } else {
                br = chain_next(rc, br);
            }
        }
    }
    *flagp &= !(HASWIDTH | SIMPLE);
    ret
}

/// `\%d123`, `\%o40`, `\%x2f`, `\%u1234`, `\%U1234abcd`: one character named
/// by its code point.
fn character_escape(rc: &mut RegCompiler, flagp: &mut c_int, c: c_int) -> *mut uint8_t {
    let i = match c as u8 {
        b'd' => getdecchrs(rc),
        b'o' => getoctchrs(rc),
        b'x' => gethexchrs(rc, 2),
        b'u' => gethexchrs(rc, 4),
        b'U' => gethexchrs(rc, 8),
        _ => -1,
    };
    if !(0..=INT_MAX as int64_t).contains(&i) {
        let prefix = magic_prefix(rc);
        semsg!("E678: Invalid character after {prefix}%[dxouU]");
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }
    let i = i as c_int;
    let ret = if use_multibytecode(rc, i) {
        regnode(rc, BtOp::Multibytecode)
    } else {
        regnode(rc, BtOp::Exactly)
    };
    // A NUL in the pattern stands for a newline: the program is a C string,
    // so it cannot hold a NUL byte.
    if i == 0 {
        regc(rc, 0xa);
    } else {
        regmbc(rc, i);
    }
    regc(rc, NUL);
    *flagp |= HASWIDTH;
    ret
}

/// The position assertions: `\%23l`, `\%<23c`, `\%>23v`, `\%.l` (the cursor's
/// own line/column) and `\%'m` (a mark).
fn position_atom(rc: &mut RegCompiler, first: c_int, save_prev_at_start: c_int) -> *mut uint8_t {
    if (ascii_isdigit(first) || matches!(first as u8, b'<' | b'>' | b'\'' | b'.'))
        && let Some(node) = compare_atom(rc, first, save_prev_at_start)
    {
        return node;
    }
    let prefix = magic_prefix(rc);
    semsg!("E71: Invalid character after {prefix}%");
    rc_did_emsg.set(true);
    core::ptr::null_mut()
}

/// The body of [`position_atom`]: `None` means the escape did not turn out to
/// be a position assertion after all, and E71 is the answer.
fn compare_atom(
    rc: &mut RegCompiler,
    first: c_int,
    save_prev_at_start: c_int,
) -> Option<*mut uint8_t> {
    // `<` and `>` make the test "before" and "after"; the node stores the
    // character itself as the comparison.
    let cmp = first;
    let mut c = first;
    if matches!(cmp as u8, b'<' | b'>') {
        c = getchr(rc);
    }
    // `\%.l` is "the cursor's line", read now rather than at match time.
    let mut cur = false;
    if unmagic(c) == b'.' as c_int {
        cur = true;
        c = getchr(rc);
    }
    let mut n: uint32_t = 0;
    let mut got_digit = false;
    while ascii_isdigit(c) {
        got_digit = true;
        n = n
            .wrapping_mul(10)
            .wrapping_add((c - b'0' as c_int) as uint32_t);
        c = getchr(rc);
    }

    if unmagic(c) == b'\'' as c_int && n == 0 {
        // `\%'m`: the position of mark m.
        let c = getchr(rc);
        let ret = regnode(rc, BtOp::ReMark);
        regc(rc, c);
        regc(rc, cmp);
        return Some(ret);
    }
    if !matches!(c as u8, b'l' | b'c' | b'v') || !(cur || got_digit) {
        return None;
    }
    if cur && n != 0 {
        let c = unmagic(c) as u8 as char;
        semsg!("E1204: No Number allowed after .: '\\%{c}'");
        rc_did_emsg.set(true);
        return Some(core::ptr::null_mut());
    }

    let ret = match c as u8 {
        b'l' => {
            if cur {
                n = cursor_value(b'l');
            }
            let ret = regnode(rc, BtOp::ReLnum);
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
            regnode(rc, BtOp::ReCol)
        }
        _ => {
            if cur {
                n = cursor_value(b'v');
            }
            regnode(rc, BtOp::ReVcol)
        }
    };
    regnr(rc, n);
    regc(rc, cmp);
    Some(ret)
}

/// The cursor's own line, column or virtual column, as `\%.l`, `\%.c` and
/// `\%.v` read it at compile time. Columns are one-based in a pattern.
fn cursor_value(kind: u8) -> uint32_t {
    // SAFETY: `curwin` is the current window, and `getvvcol` writes only
    // through the out-parameters it is given.
    match kind {
        b'l' => (Win::current().w_cursor.lnum) as uint32_t,
        b'c' => (Win::current().w_cursor.col) as uint32_t + 1,
        _ => {
            let mut vcol: ColNr = 0;
            unsafe {
                getvvcol(
                    Win::current(),
                    &raw mut (*Win::current_raw()).w_cursor,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &raw mut vcol,
                )
            };
            vcol as uint32_t + 1
        }
    }
}
