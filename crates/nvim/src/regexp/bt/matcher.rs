//! The backtracking match loop.
//!
//! `regmatch` walks the compiled program from a node forwards, and whenever a
//! node has more than one way to continue it pushes a frame onto `regstack`
//! describing what to try next. When the walk fails or reaches `END`, control
//! goes to [`super::resume`], which pops frames until one of them has another
//! alternative to offer.
//!
//! It is a loop rather than a recursion on purpose: a pattern like `\(a*\)*`
//! against a long line nests as deeply as the line is long, and the C this was
//! transpiled from had already been rewritten away from recursion for exactly
//! that reason. The only bound on how far it can go is 'maxmempattern', which
//! `regstack_push` charges against the stack's byte size.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::compile::regnext;
use super::resume::resume;
use super::single::match_one;
use super::state::{RegStack, capture_slot, regstack};
use crate::garray::ga_append_via_ptr;
use crate::main::{e_re_corr, got_int};
use crate::mbyte::{mb_isupper, mb_tolower, mb_toupper};
use crate::message::{iemsg, internal_error};
use crate::os::cshim::gettext;
use crate::profile::profile_passed_limit;
use crate::regexp::{
    ADD_NL, BACK, BEHIND, BRACE_COMPLEX, BRACE_LIMITS, BRACE_SIMPLE, BRANCH, EXACTLY, FIRST_NL,
    LAST_NL, MATCH, MAX_LIMIT, MCLOSE, MOPEN, NCLOSE, NOBEHIND, NOMATCH, NOPEN, PLUS, RA_BREAK,
    RA_CONT, RA_FAIL, RA_MATCH, RA_NOMATCH, RS_BEHIND1, RS_BRANCH, RS_BRCPLX_LONG, RS_BRCPLX_MORE,
    RS_BRCPLX_SHORT, RS_MCLOSE, RS_MOPEN, RS_NOMATCH, RS_NOPEN, RS_STAR_LONG, RS_STAR_SHORT,
    RS_ZCLOSE, RS_ZOPEN, Rex, STAR, SUBPAT, ZCLOSE, ZOPEN, backpos, backpos_T, bl_maxval,
    bl_minval, brace_count, brace_max, brace_min, cleanup_subexpr, cleanup_zsubexpr,
    reg_breakcheck, reg_nextline, reg_save, regstar_T, regstate_T, save_capture, save_subexpr,
};
use crate::types::{NUL, int16_t, int64_t, proftime_T, uint8_t};

/// The four bytes of a node operand at `off`, big-endian, as the compiler
/// wrote them.
///
/// # Safety
/// `p` must be a node whose operand runs at least `off + 4` bytes.
pub(crate) unsafe fn operand_u32(p: *const uint8_t, off: usize) -> int64_t {
    // SAFETY: the caller promises the four bytes.
    unsafe {
        let b = |i: usize| *p.add(off + i) as int64_t;
        (b(0) << 24) + (b(1) << 16) + (b(2) << 8) + b(3)
    }
}

const MOPEN_9: c_int = MOPEN + 9;
const MCLOSE_9: c_int = MCLOSE + 9;
const ZOPEN_1: c_int = ZOPEN + 1;
const ZOPEN_9: c_int = ZOPEN + 9;
const ZCLOSE_1: c_int = ZCLOSE + 1;
const ZCLOSE_9: c_int = ZCLOSE + 9;
const BRACE_COMPLEX_9: c_int = BRACE_COMPLEX + 9;

/// Does `op` belong to the `\_x` band, i.e. is it a node that also matches a
/// line break?
fn crosses_lines(op: c_int) -> bool {
    (FIRST_NL..=LAST_NL).contains(&op)
}

/// Match the program starting at `scan` against `rex.input`.
///
/// `tm` and `timed_out`, when given, cap how long the search may run; the
/// clock is only read every hundredth node.
pub(crate) fn regmatch(
    rex: Rex,
    start: *mut uint8_t,
    tm: *const proftime_T,
    timed_out: *mut c_int,
) -> bool {
    // SAFETY: `start` is a node in the compiled program and `rex` describes a
    // match that is set up and running; every pointer below either comes from
    // the program, from `rex`, or from the two garrays this function owns for
    // the duration of the call.
    unsafe {
        let mut scan = start;
        let mut tm_count = 0;
        let mut status;
        // The stack is reserved for this call: nothing the walk below runs
        // re-enters `regmatch`.
        let stack = &mut *regstack.ptr();
        stack.begin();
        (*backpos.ptr()).ga_len = 0;

        loop {
            reg_breakcheck(rex);

            // Walk forward until something needs deciding.
            loop {
                if got_int.get() || scan.is_null() {
                    status = RA_FAIL;
                    break;
                }
                if !tm.is_null() && {
                    tm_count += 1;
                    tm_count == 100
                } {
                    tm_count = 0;
                    if profile_passed_limit(*tm) {
                        if !timed_out.is_null() {
                            *timed_out = 1;
                        }
                        status = RA_FAIL;
                        break;
                    }
                }

                status = RA_CONT;
                let mut next = regnext(scan);
                let mut op = *scan as c_int;

                // A `\_x` sitting at the end of the line consumes the line
                // break and stays on the same node.
                let at_line_end = rex.byte() as c_int == NUL;
                if !rex.reg_line_lbr()
                    && crosses_lines(op)
                    && rex.multi()
                    && at_line_end
                    && rex.lnum() <= rex.reg_maxline()
                {
                    reg_nextline(rex);
                } else if rex.reg_line_lbr()
                    && crosses_lines(op)
                    && rex.byte() as c_int == '\n' as c_int
                {
                    // With 'reg_line_lbr' the break is a real newline byte.
                    rex.advance_char();
                } else {
                    // Past the line break, a `\_x` behaves as its plain form.
                    if crosses_lines(op) {
                        op -= ADD_NL;
                    }
                    let c = rex.char_here();
                    status = match match_one(rex, op, scan, next, c) {
                        Some(status) => status,
                        None => push_frame(rex, stack, op, scan, &mut next),
                    };
                }

                if status != RA_CONT {
                    break;
                }
                scan = next;
            }

            resume(rex, stack, &mut scan, &mut status);

            if status == RA_CONT {
                continue;
            }
            if stack.depth() == 0 || status == RA_FAIL {
                if scan.is_null() {
                    // Should not happen. Providing a message and failing is
                    // better than a crash.
                    iemsg(gettext(&raw const e_re_corr as *const c_char));
                }
                return status == RA_MATCH;
            }
        }
    }
}

/// The opcodes that have to remember something before the walk goes on.
///
/// Each pushes a frame describing what [`super::resume`] should do if the
/// rest of the pattern fails from here. `next` is the node the walk continues
/// at; several of these redirect it into their own operand.
fn push_frame(
    rex: Rex,
    stack: &mut RegStack,
    op: c_int,
    scan: *mut uint8_t,
    next: &mut *mut uint8_t,
) -> c_int {
    // SAFETY: as `regmatch`; `scan` is a node in the program and the frames
    // pushed here are read back by `resume` while this call is still on the
    // stack.
    unsafe {
        match op {
            // A loop's back edge. Coming back to the same node at the same
            // input position means the loop is not making progress.
            BACK => {
                let seen = || (*backpos.ptr()).ga_data.cast::<backpos_T>();
                let count = (*backpos.ptr()).ga_len;
                let mut i = 0;
                while i < count && (*seen().add(i as usize)).bp_scan != scan {
                    i += 1;
                }
                let mut status = RA_CONT;
                if i == count {
                    // Appending can move the array, so `seen` is re-read
                    // rather than held across the call.
                    let fresh = ga_append_via_ptr(backpos.ptr(), size_of::<backpos_T>())
                        .cast::<backpos_T>();
                    (*fresh).bp_scan = scan;
                } else if rex.is_at((*seen().add(i as usize)).bp_pos.pos) {
                    status = RA_NOMATCH;
                }
                if status != RA_NOMATCH {
                    reg_save(rex, &mut (*seen().add(i as usize)).bp_pos, backpos.ptr());
                }
                status
            }

            // The capture-group boundaries. Each remembers the slot it is
            // about to overwrite so a failure can put it back.
            MOPEN..=MOPEN_9 => {
                cleanup_subexpr(rex);
                push_capture(rex, stack, RS_MOPEN, scan, op - MOPEN)
            }
            MCLOSE..=MCLOSE_9 => {
                cleanup_subexpr(rex);
                push_capture(rex, stack, RS_MCLOSE, scan, op - MCLOSE)
            }
            ZOPEN_1..=ZOPEN_9 => {
                cleanup_zsubexpr(rex);
                push_capture(rex, stack, RS_ZOPEN, scan, op - ZOPEN)
            }
            ZCLOSE_1..=ZCLOSE_9 => {
                cleanup_zsubexpr(rex);
                push_capture(rex, stack, RS_ZCLOSE, scan, op - ZCLOSE)
            }

            // `\%(` captures nothing, but still needs a frame so that the
            // unwinder has something to step over.
            NOPEN | NCLOSE => {
                if stack.push(RS_NOPEN, scan).is_none() {
                    RA_FAIL
                } else {
                    RA_CONT
                }
            }

            // The last branch of an alternation needs no frame: there is
            // nothing left to fall back to.
            BRANCH => {
                if *(*next) as c_int != BRANCH {
                    *next = scan.add(3);
                    RA_CONT
                } else if stack.push(RS_BRANCH, scan).is_none() {
                    RA_FAIL
                } else {
                    RA_BREAK
                }
            }

            // The bounds of the `{n,m}` that follows, stashed where the node
            // they belong to can find them.
            BRACE_LIMITS => {
                let next_op = *(*next) as c_int;
                if next_op == BRACE_SIMPLE {
                    bl_minval.set(operand_u32(scan, 3));
                    bl_maxval.set(operand_u32(scan, 7));
                    RA_CONT
                } else if (BRACE_COMPLEX..BRACE_COMPLEX + 10).contains(&next_op) {
                    let no = (next_op - BRACE_COMPLEX) as usize;
                    (*brace_min.ptr())[no] = operand_u32(scan, 3);
                    (*brace_max.ptr())[no] = operand_u32(scan, 7);
                    (*brace_count.ptr())[no] = 0;
                    RA_CONT
                } else {
                    internal_error(c"BRACE_LIMITS".as_ptr());
                    RA_FAIL
                }
            }

            BRACE_COMPLEX..=BRACE_COMPLEX_9 => {
                brace_complex(rex, stack, op - BRACE_COMPLEX, scan, next)
            }

            BRACE_SIMPLE | STAR | PLUS => counted_repeat(rex, stack, op, scan),

            // `\@=`, `\@!` and `\@>`: run the operand, then decide what its
            // outcome means.
            NOMATCH | MATCH | SUBPAT => {
                let Some(rp) = stack.push(RS_NOMATCH, scan) else {
                    return RA_FAIL;
                };
                rp.rs_no = op as int16_t;
                reg_save(rex, &mut rp.rs_saved, backpos.ptr());
                *next = scan.add(3);
                RA_CONT
            }

            // `\@<=` and `\@<!`: the operand has to match ending here, so the
            // unwinder walks the start position backwards until it does.
            BEHIND | NOBEHIND => {
                // The capture snapshot rides in front of the frame.
                if !stack.push_behind(RS_BEHIND1, scan) {
                    return RA_FAIL;
                }
                let (rp, bp) = stack.top_behind();
                save_subexpr(rex, bp);
                rp.rs_no = op as int16_t;
                reg_save(rex, &mut rp.rs_saved, backpos.ptr());
                RA_CONT
            }

            _ => {
                // Should not happen: the compiler emitted something the
                // matcher does not know.
                iemsg(gettext(&raw const e_re_corr as *const c_char));
                RA_FAIL
            }
        }
    }
}

/// Push a frame that restores capture group `no`'s slot if the rest of the
/// pattern fails.
///
/// # Safety
/// `state` must be a capture state and `no` a group of the running match —
/// see [`capture_slot`].
unsafe fn push_capture(
    rex: Rex,
    stack: &mut RegStack,
    state: regstate_T,
    scan: *mut uint8_t,
    no: c_int,
) -> c_int {
    // SAFETY: as `push_frame`.
    unsafe {
        let slot = capture_slot(rex, state, no as usize);
        let Some(rp) = stack.push(state, scan) else {
            return RA_FAIL;
        };
        rp.rs_no = no as int16_t;
        save_capture(rex, &mut rp.rs_saved.pos, slot);
        RA_CONT
    }
}

/// One pass of a `\{n,m}` around something that is not `SIMPLE`.
///
/// The count lives in `brace_count`; which frame goes on the stack depends on
/// whether the bound is greedy (`min <= max`) and on whether the minimum has
/// been reached yet.
///
/// # Safety
/// As `push_frame`.
unsafe fn brace_complex(
    rex: Rex,
    stack: &mut RegStack,
    no: c_int,
    scan: *mut uint8_t,
    next: &mut *mut uint8_t,
) -> c_int {
    // SAFETY: as `push_frame`.
    unsafe {
        let slot = no as usize;
        (*brace_count.ptr())[slot] += 1;
        let count = (*brace_count.ptr())[slot] as int64_t;
        let min = (*brace_min.ptr())[slot];
        let max = (*brace_max.ptr())[slot];
        let greedy = min <= max;

        // Still below the smaller bound: another pass is mandatory.
        if count <= min.min(max) {
            let Some(rp) = stack.push(RS_BRCPLX_MORE, scan) else {
                return RA_FAIL;
            };
            rp.rs_no = no as int16_t;
            reg_save(rex, &mut rp.rs_saved, backpos.ptr());
            *next = scan.add(3);
        } else if greedy {
            // Another pass is allowed; try it, and fall back to stopping.
            if count <= max {
                let Some(rp) = stack.push(RS_BRCPLX_LONG, scan) else {
                    return RA_FAIL;
                };
                rp.rs_no = no as int16_t;
                reg_save(rex, &mut rp.rs_saved, backpos.ptr());
                *next = scan.add(3);
            }
        } else if count <= min {
            // Non-greedy: try stopping first, and fall back to another pass.
            let Some(rp) = stack.push(RS_BRCPLX_SHORT, scan) else {
                return RA_FAIL;
            };
            reg_save(rex, &mut rp.rs_saved, backpos.ptr());
        }
        RA_CONT
    }
}

/// `*`, `+` or `\{n,m}` around a `SIMPLE` item: count the matches in one go
/// and push a frame that can give them back one at a time.
///
/// # Safety
/// As `push_frame`.
unsafe fn counted_repeat(rex: Rex, stack: &mut RegStack, op: c_int, scan: *mut uint8_t) -> c_int {
    // SAFETY: as `push_frame`.
    unsafe {
        let mut rst = regstar_T {
            nextb: NUL,
            nextb_ic: NUL,
            count: 0,
            minval: 0,
            maxval: 0,
        };
        // Knowing the byte the repeat has to stop before lets the unwinder
        // skip positions that cannot possibly continue.
        let next = regnext(scan);
        if !next.is_null() && *next as c_int == EXACTLY {
            rst.nextb = *next.add(3) as c_int;
            rst.nextb_ic = if !rex.reg_ic() {
                rst.nextb
            } else if mb_isupper(rst.nextb) {
                mb_tolower(rst.nextb)
            } else {
                mb_toupper(rst.nextb)
            };
        }
        if op != BRACE_SIMPLE {
            rst.minval = if op == STAR { 0 } else { 1 };
            rst.maxval = MAX_LIMIT as int64_t;
        } else {
            rst.minval = bl_minval.get();
            rst.maxval = bl_maxval.get();
        }

        rst.count = super::repeat::regrepeat(rex, scan.add(3), rst.maxval) as int64_t;
        if got_int.get() {
            return RA_FAIL;
        }
        // A reversed bound (`\{-n,m}`) is the non-greedy form, and then the
        // *larger* number is the one that has to be reached.
        let enough = if rst.minval <= rst.maxval {
            rst.count >= rst.minval
        } else {
            rst.count >= rst.maxval
        };
        if !enough {
            return RA_NOMATCH;
        }
        // The counter rides in front of the frame.
        let state = if rst.minval <= rst.maxval {
            RS_STAR_LONG
        } else {
            RS_STAR_SHORT
        };
        if !stack.push_star(state, scan, rst) {
            return RA_FAIL;
        }
        RA_BREAK
    }
}
