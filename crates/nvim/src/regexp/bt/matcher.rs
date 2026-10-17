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

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::compile::BtProg;
use super::op::BtOp;
use super::resume::resume;
use super::single::match_one;
use super::state::{BackPos, Braces, BtState, RegStack, capture_slot};
use crate::getchar::state::got_int;
use crate::mbyte::{mb_isupper, mb_tolower, mb_toupper};
use crate::message::e_re_corr;
use crate::message::{iemsg, internal_error};
use crate::os::cshim::gettext;
use crate::profile::profile_passed_limit;
use crate::regexp::{
    MAX_LIMIT, RA_BREAK, RA_CONT, RA_FAIL, RA_MATCH, RA_NOMATCH, RS_BEHIND1, RS_BRANCH,
    RS_BRCPLX_LONG, RS_BRCPLX_MORE, RS_BRCPLX_SHORT, RS_MCLOSE, RS_MOPEN, RS_NOMATCH, RS_NOPEN,
    RS_STAR_LONG, RS_STAR_SHORT, RS_ZCLOSE, RS_ZOPEN, RegStar, RegState, Rex, cleanup_subexpr,
    cleanup_zsubexpr, reg_breakcheck, reg_nextline, reg_save, save_capture, save_subexpr,
};
use crate::types::{NUL, int16_t, int64_t, uint8_t};

/// The first code of each run a group number is measured from. These are
/// `const` items because `BtOp::Mopen.code()` is a *call* in a runtime
/// expression that nothing folds at opt-level 0.
const MOPEN: c_int = BtOp::Mopen.code();
const MCLOSE: c_int = BtOp::Mclose.code();
const ZOPEN: c_int = BtOp::Zopen.code();
const ZCLOSE: c_int = BtOp::Zclose.code();
const BRACE_COMPLEX: c_int = BtOp::BraceComplex.code();

/// Match the program starting at `scan` against `rex.input`.
///
/// The caller's deadline, when it gave one, caps how long the search may
/// run; the clock is only read every hundredth node.
pub(crate) fn regmatch(rex: Rex, prog: BtProg<'_>, state: &mut BtState, start: usize) -> bool {
    // The working state is threaded down by `&mut`: `reg_save` and
    // `reg_restore` are called far too often to pay for a borrow each, and
    // the three fields are handed over separately so that saving *into* the
    // back-edge record does not borrow the whole of it.
    state.begin();
    let BtState {
        stack,
        backpos,
        braces,
    } = state;
    walk(rex, prog, stack, backpos, braces, start)
}

/// [`regmatch`]'s loop, with the working state already borrowed.
fn walk(
    rex: Rex,
    prog: BtProg<'_>,
    stack: &mut RegStack,
    backpos: &mut BackPos,
    braces: &mut Braces,
    start: usize,
) -> bool {
    let mut scan = start;
    let mut tm_count = 0;
    let mut status;
    let deadline = rex.deadline();

    loop {
        reg_breakcheck(rex);

        // Walk forward until something needs deciding.
        loop {
            if got_int.get() || scan == 0 {
                status = RA_FAIL;
                break;
            }
            if let Some(deadline) = deadline
                && {
                    tm_count += 1;
                    tm_count == 100
                }
            {
                tm_count = 0;
                if profile_passed_limit(deadline) {
                    rex.set_timed_out();
                    status = RA_FAIL;
                    break;
                }
            }

            status = RA_CONT;
            let (op_byte, mut next) = prog.op_and_next(scan);
            // The `\_x` band is decoded away here, once per node: past the
            // line break such a node behaves as its plain form.
            let Ok((op, crosses_lines)) = BtOp::decode(op_byte) else {
                status = RA_FAIL;
                break;
            };

            // A `\_x` sitting at the end of the line consumes the line
            // break and stays on the same node. Only such a node looks: the
            // byte is read for no other.
            if crosses_lines
                && !rex.reg_line_lbr()
                && rex.multi()
                && rex.byte() as c_int == NUL
                && rex.lnum() <= rex.reg_maxline()
            {
                reg_nextline(rex);
            } else if crosses_lines && rex.reg_line_lbr() && rex.byte() as c_int == '\n' as c_int {
                // With 'reg_line_lbr' the break is a real newline byte.
                rex.advance_char();
            } else {
                let c = rex.char_here();
                status = match match_one(rex, prog, op, scan, next, c) {
                    Some(status) => status,
                    None => push_frame(rex, prog, stack, backpos, braces, op, scan, &mut next),
                };
            }

            if status != RA_CONT {
                break;
            }
            scan = next;
        }

        resume(rex, prog, stack, backpos, braces, &mut scan, &mut status);

        if status == RA_CONT {
            continue;
        }
        if stack.depth() == 0 || status == RA_FAIL {
            if scan == 0 {
                // Should not happen. Providing a message and failing is
                // better than a crash.
                iemsg(gettext(e_re_corr));
            }
            return status == RA_MATCH;
        }
    }
}

/// The opcodes that have to remember something before the walk goes on.
///
/// Each pushes a frame describing what [`super::resume`] should do if the
/// rest of the pattern fails from here. `next` is the node the walk continues
/// at; several of these redirect it into their own operand.
#[allow(
    clippy::too_many_arguments,
    reason = "the walk's working state, threaded down"
)]
fn push_frame(
    rex: Rex,
    prog: BtProg<'_>,
    stack: &mut RegStack,
    backpos: &mut BackPos,
    braces: &mut Braces,
    op: BtOp,
    scan: usize,
    next: &mut usize,
) -> c_int {
    match op {
        // A loop's back edge. Coming back to the same node at the same
        // input position means the loop is not making progress.
        BtOp::Back => {
            if backpos.stepped(rex, scan) {
                RA_CONT
            } else {
                RA_NOMATCH
            }
        }

        // The capture-group boundaries. Each remembers the slot it is
        // about to overwrite so a failure can put it back.
        BtOp::Mopen
        | BtOp::Mopen1
        | BtOp::Mopen2
        | BtOp::Mopen3
        | BtOp::Mopen4
        | BtOp::Mopen5
        | BtOp::Mopen6
        | BtOp::Mopen7
        | BtOp::Mopen8
        | BtOp::Mopen9 => {
            cleanup_subexpr(rex);
            push_capture(rex, stack, RS_MOPEN, scan, op.code() - MOPEN)
        }
        BtOp::Mclose
        | BtOp::Mclose1
        | BtOp::Mclose2
        | BtOp::Mclose3
        | BtOp::Mclose4
        | BtOp::Mclose5
        | BtOp::Mclose6
        | BtOp::Mclose7
        | BtOp::Mclose8
        | BtOp::Mclose9 => {
            cleanup_subexpr(rex);
            push_capture(rex, stack, RS_MCLOSE, scan, op.code() - MCLOSE)
        }
        BtOp::Zopen1
        | BtOp::Zopen2
        | BtOp::Zopen3
        | BtOp::Zopen4
        | BtOp::Zopen5
        | BtOp::Zopen6
        | BtOp::Zopen7
        | BtOp::Zopen8
        | BtOp::Zopen9 => {
            cleanup_zsubexpr(rex);
            push_capture(rex, stack, RS_ZOPEN, scan, op.code() - ZOPEN)
        }
        BtOp::Zclose1
        | BtOp::Zclose2
        | BtOp::Zclose3
        | BtOp::Zclose4
        | BtOp::Zclose5
        | BtOp::Zclose6
        | BtOp::Zclose7
        | BtOp::Zclose8
        | BtOp::Zclose9 => {
            cleanup_zsubexpr(rex);
            push_capture(rex, stack, RS_ZCLOSE, scan, op.code() - ZCLOSE)
        }

        // `\%(` captures nothing, but still needs a frame so that the
        // unwinder has something to step over.
        BtOp::Nopen | BtOp::Nclose => {
            if stack.push(RS_NOPEN, scan).is_none() {
                RA_FAIL
            } else {
                RA_CONT
            }
        }

        // The last branch of an alternation needs no frame: there is
        // nothing left to fall back to.
        BtOp::Branch => {
            if prog.op_byte(*next) != BtOp::Branch.code() as uint8_t {
                *next = scan + 3;
                RA_CONT
            } else if stack.push(RS_BRANCH, scan).is_none() {
                RA_FAIL
            } else {
                RA_BREAK
            }
        }

        // The bounds of the `{n,m}` that follows, stashed where the node
        // they belong to can find them.
        BtOp::BraceLimits => {
            let next_op = BtOp::decode(prog.op_byte(*next)).map(|(op, _)| op);
            if next_op == Ok(BtOp::BraceSimple) {
                rex.set_brace_limits(prog.operand_u32(scan, 3), prog.operand_u32(scan, 7));
                RA_CONT
            } else if let Ok(next_op) = next_op
                && let Some(slot) = next_op.index_in(BtOp::BraceComplex)
            {
                let brace = braces.slot(slot);
                brace.min = prog.operand_u32(scan, 3);
                brace.max = prog.operand_u32(scan, 7);
                brace.count = 0;
                RA_CONT
            } else {
                internal_error(c"BRACE_LIMITS");
                RA_FAIL
            }
        }

        BtOp::BraceComplex
        | BtOp::BraceComplex1
        | BtOp::BraceComplex2
        | BtOp::BraceComplex3
        | BtOp::BraceComplex4
        | BtOp::BraceComplex5
        | BtOp::BraceComplex6
        | BtOp::BraceComplex7
        | BtOp::BraceComplex8
        | BtOp::BraceComplex9 => {
            let slot = op.code() - BRACE_COMPLEX;
            brace_complex(rex, stack, backpos, braces, slot, scan, next)
        }

        BtOp::BraceSimple | BtOp::Star | BtOp::Plus => counted_repeat(rex, prog, stack, op, scan),

        // `\@=`, `\@!` and `\@>`: run the operand, then decide what its
        // outcome means.
        BtOp::Nomatch | BtOp::Match | BtOp::Subpat => {
            let Some(rp) = stack.push(RS_NOMATCH, scan) else {
                return RA_FAIL;
            };
            rp.rs_no = op.code() as int16_t;
            rp.rs_saved = reg_save(rex, backpos);
            *next = scan + 3;
            RA_CONT
        }

        // `\@<=` and `\@<!`: the operand has to match ending here, so the
        // unwinder walks the start position backwards until it does.
        BtOp::Behind | BtOp::Nobehind => {
            // The capture snapshot rides in front of the frame.
            if !stack.push_behind(RS_BEHIND1, scan) {
                return RA_FAIL;
            }
            let (rp, bp) = stack.top_behind();
            save_subexpr(rex, bp);
            rp.rs_no = op.code() as int16_t;
            rp.rs_saved = reg_save(rex, backpos);
            RA_CONT
        }

        _ => {
            // Should not happen: the compiler emitted something the
            // matcher does not know.
            iemsg(gettext(e_re_corr));
            RA_FAIL
        }
    }
}

/// Push a frame that restores capture group `no`'s slot if the rest of the
/// pattern fails.
///
/// `state` is a capture state and `no` a group number decoded from one of
/// the capture opcodes: see [`capture_slot`].
fn push_capture(rex: Rex, stack: &mut RegStack, state: RegState, scan: usize, no: c_int) -> c_int {
    let slot = capture_slot(rex, state, no as usize);
    let Some(rp) = stack.push(state, scan) else {
        return RA_FAIL;
    };
    rp.rs_no = no as int16_t;
    save_capture(rex, &mut rp.rs_saved.pos, slot);
    RA_CONT
}

/// One pass of a `\{n,m}` around something that is not `SIMPLE`.
///
/// The count lives in `brace_count`; which frame goes on the stack depends on
/// whether the bound is greedy (`min <= max`) and on whether the minimum has
/// been reached yet.
///
fn brace_complex(
    rex: Rex,
    stack: &mut RegStack,
    backpos: &mut BackPos,
    braces: &mut Braces,
    no: c_int,
    scan: usize,
    next: &mut usize,
) -> c_int {
    let brace = braces.slot(no as usize);
    brace.count += 1;
    let (count, min, max) = (brace.count, brace.min, brace.max);
    let greedy = min <= max;

    // Still below the smaller bound: another pass is mandatory.
    if count <= min.min(max) {
        let Some(rp) = stack.push(RS_BRCPLX_MORE, scan) else {
            return RA_FAIL;
        };
        rp.rs_no = no as int16_t;
        rp.rs_saved = reg_save(rex, backpos);
        *next = scan + 3;
    } else if greedy {
        // Another pass is allowed; try it, and fall back to stopping.
        if count <= max {
            let Some(rp) = stack.push(RS_BRCPLX_LONG, scan) else {
                return RA_FAIL;
            };
            rp.rs_no = no as int16_t;
            rp.rs_saved = reg_save(rex, backpos);
            *next = scan + 3;
        }
    } else if count <= min {
        // Non-greedy: try stopping first, and fall back to another pass.
        let Some(rp) = stack.push(RS_BRCPLX_SHORT, scan) else {
            return RA_FAIL;
        };
        rp.rs_saved = reg_save(rex, backpos);
    }
    RA_CONT
}

/// `*`, `+` or `\{n,m}` around a `SIMPLE` item: count the matches in one go
/// and push a frame that can give them back one at a time.
///
fn counted_repeat(
    rex: Rex,
    prog: BtProg<'_>,
    stack: &mut RegStack,
    op: BtOp,
    scan: usize,
) -> c_int {
    let mut rst = RegStar {
        nextb: NUL,
        nextb_ic: NUL,
        count: 0,
        minval: 0,
        maxval: 0,
    };
    // Knowing the byte the repeat has to stop before lets the unwinder
    // skip positions that cannot possibly continue.
    let next = prog.next(scan);
    if next != 0 && prog.op_byte(next) == BtOp::Exactly.code() as uint8_t {
        rst.nextb = c_int::from(prog.byte(next, 3));
        rst.nextb_ic = if !rex.reg_ic() {
            rst.nextb
        } else if mb_isupper(rst.nextb) {
            mb_tolower(rst.nextb)
        } else {
            mb_toupper(rst.nextb)
        };
    }
    if op != BtOp::BraceSimple {
        rst.minval = if op == BtOp::Star { 0 } else { 1 };
        rst.maxval = MAX_LIMIT as int64_t;
    } else {
        (rst.minval, rst.maxval) = rex.brace_limits();
    }

    rst.count = super::repeat::regrepeat(rex, prog, scan + 3, rst.maxval) as int64_t;
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
