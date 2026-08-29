//! The engine's entry points: compiling a pattern, freeing a program, and
//! the two `regexec` shapes the engine table names — one over a string, one
//! over a buffer.
//!
//! Between them and the match loop sits `nfa_regexec_both`, which is where
//! the shortcuts live: an anchored pattern only tries column 0, a pattern
//! with a known first character skips to it, and a pattern that is one
//! literal run never runs the machine at all.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::build::{Pass, nfa_postprocess, post2nfa};
use super::compile::{nfa_get_match_text, nfa_get_reganch, nfa_get_regstart, nfa_regcomp_start};
use super::matcher::nfa_regmatch;
use super::parse::re2post;
use super::postfix;
use super::run::{find_match_text, skip_to_start};
use super::sub::{clear_sub, slots};
use crate::main::{e_null, re_extmatch_out};
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    NFA_TOO_EXPENSIVE, NSUBEXP, REX_SET, RF_ICASE, RF_ICOMBINE, RF_NOICASE, Rex, cleanup_subexpr,
    cleanup_zsubexpr, init_regexec, init_regexec_multi, make_extmatch, nfa_re_flags, nfa_regengine,
    nfa_regprog_T, nfa_state_T, nfa_time_count, nfa_time_limit, nfa_timed_out, nstate, re_has_z,
    reg_getline, regflags, regnpar, regsubs_T, state_ptr, unref_extmatch,
};
use crate::strings::xstrnsave;
use crate::types::{
    FAIL, NUL, buf_T, colnr_T, linenr_T, lpos_T, proftime_T, reg_extmatch_T, regmatch_T,
    regmmatch_T, regprog_T, uint8_t, win_T,
};

/// Try to match at column `col` of the current line.
///
/// Returns 0 for no match, `NFA_TOO_EXPENSIVE`, or one more than the line
/// the match ended on.
fn nfa_regtry(
    rex: Rex,
    prog: *mut nfa_regprog_T,
    col: colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: `prog` is a live program and `rex` the match context set up by
    // `nfa_regexec_both`; the capture arrays below are the caller's, sized
    // `NSUBEXP`.
    let mut subs: regsubs_T = unsafe { core::mem::zeroed() };
    let mut m: regsubs_T = unsafe { core::mem::zeroed() };
    rex.set_input(unsafe { rex.line().offset(col as isize) });
    nfa_time_limit.set(tm);
    nfa_timed_out.set(timed_out);
    nfa_time_count.set(0);

    clear_sub(rex, &mut subs.norm);
    clear_sub(rex, &mut m.norm);
    clear_sub(rex, &mut subs.synt);
    clear_sub(rex, &mut m.synt);

    let result = nfa_regmatch(
        rex,
        prog,
        unsafe { (*prog).start },
        &raw mut subs,
        &raw mut m,
    );
    if result == 0 || result == NFA_TOO_EXPENSIVE {
        return result;
    }

    cleanup_subexpr(rex);
    if rex.multi() {
        report_buffer_match(rex, &subs, col);
    } else {
        report_string_match(rex, &subs, col);
    }

    // The `\z(` captures go to the syntax highlighter as fresh copies,
    // because it keeps them past the end of this match.
    unsafe { unref_extmatch(re_extmatch_out.get()) };
    re_extmatch_out.set(core::ptr::null_mut::<reg_extmatch_T>());
    if unsafe { (*prog).reghasz } == REX_SET {
        cleanup_zsubexpr(rex);
        re_extmatch_out.set(make_extmatch());
        save_z_captures(rex, &subs);
    }
    1 + rex.lnum() as c_int
}

/// Copy the capture positions of a buffer match into the caller's arrays.
fn report_buffer_match(rex: Rex, subs: &regsubs_T, col: colnr_T) {
    // SAFETY: a buffer match's context holds the caller's position arrays,
    // `NSUBEXP` slots each.
    let (starts, ends) = unsafe {
        (
            core::slice::from_raw_parts_mut(rex.reg_startpos(), NSUBEXP as usize),
            core::slice::from_raw_parts_mut(rex.reg_endpos(), NSUBEXP as usize),
        )
    };
    for i in 0..slots(subs.norm.in_use) {
        starts[i] = subs.norm.list[i].start.as_pos();
        ends[i] = subs.norm.list[i].end.as_pos();
    }
    if !rex.reg_mmatch().is_null() {
        // SAFETY: a non-null `reg_mmatch` is the caller's match structure.
        unsafe { (*rex.reg_mmatch()).rmm_matchcol = subs.norm.orig_start_col };
    }
    // A `\zs` before the start, or a `\ze` before the end, can leave group 0
    // unset; it then covers what the machine actually walked.
    if starts[0].lnum < 0 {
        starts[0] = lpos_T { lnum: 0, col };
    }
    if ends[0].lnum < 0 {
        ends[0] = lpos_T {
            lnum: rex.lnum(),
            col: rex.col(),
        };
    } else {
        rex.set_lnum(ends[0].lnum);
    }
}

/// As [`report_buffer_match`], for a match over a plain string.
fn report_string_match(rex: Rex, subs: &regsubs_T, col: colnr_T) {
    // SAFETY: as `report_buffer_match`; a string match's slots are pointers.
    let (starts, ends) = unsafe {
        (
            core::slice::from_raw_parts_mut(rex.reg_startp(), NSUBEXP as usize),
            core::slice::from_raw_parts_mut(rex.reg_endp(), NSUBEXP as usize),
        )
    };
    for i in 0..slots(subs.norm.in_use) {
        starts[i] = subs.norm.list[i].start.as_ptr();
        ends[i] = subs.norm.list[i].end.as_ptr();
    }
    if starts[0].is_null() {
        // SAFETY: `col` is a byte offset into the line being matched.
        starts[0] = unsafe { rex.line().offset(col as isize) };
    }
    if ends[0].is_null() {
        ends[0] = rex.input();
    }
}

/// Copy what the `\z(` groups matched into the set the highlighter reads.
///
/// SAFETY: `re_extmatch_out` holds a fresh capture set.
fn save_z_captures(rex: Rex, subs: &regsubs_T) {
    // Group 0 is the whole match, which the highlighter does not want.
    for i in 1..slots(subs.synt.in_use) {
        let capture = subs.synt.list[i];
        let text = if rex.multi() {
            let (start, end) = (capture.start.as_pos(), capture.end.as_pos());
            // A capture that spans lines cannot be handed over as one
            // string, so it is dropped.
            if start.lnum < 0 || start.lnum != end.lnum || end.col < start.col {
                continue;
            }
            unsafe {
                xstrnsave(
                    reg_getline(rex, start.lnum).offset(start.col as isize),
                    (end.col - start.col) as usize,
                )
            }
        } else {
            let (start, end) = (capture.start.as_ptr(), capture.end.as_ptr());
            if start.is_null() || end.is_null() {
                continue;
            }
            unsafe { xstrnsave(start as *mut c_char, end.offset_from(start) as usize) }
        };
        unsafe { (*re_extmatch_out.get()).matches[i] = text as *mut uint8_t };
    }
}

/// Match `prog` against a line, from `startcol`.
///
/// SAFETY: `rex` has been pointed at the caller's match structure.
fn nfa_regexec_both(
    rex: Rex,
    mut line: *mut uint8_t,
    startcol: colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    let mut col = startcol;
    let prog: *mut nfa_regprog_T = if rex.multi() {
        line = reg_getline(rex, 0) as *mut uint8_t;
        rex.set_reg_startpos((unsafe { &raw mut (*rex.reg_mmatch()).startpos }).cast());
        rex.set_reg_endpos((unsafe { &raw mut (*rex.reg_mmatch()).endpos }).cast());
        unsafe { (*rex.reg_mmatch()).regprog.cast() }
    } else {
        rex.set_reg_startp((unsafe { &raw mut (*rex.reg_match()).startp }).cast());
        rex.set_reg_endp((unsafe { &raw mut (*rex.reg_match()).endp }).cast());
        unsafe { (*rex.reg_match()).regprog.cast() }
    };

    let mut retval = 0;
    if prog.is_null() || line.is_null() {
        iemsg(gettext(e_null));
    } else {
        // The pattern's own `\c`/`\C`/`\Z` override what the caller asked
        // for.
        if unsafe { (*prog).regflags } & RF_ICASE as u32 != 0 {
            rex.set_reg_ic(true);
        } else if unsafe { (*prog).regflags } & RF_NOICASE as u32 != 0 {
            rex.set_reg_ic(false);
        }
        if unsafe { (*prog).regflags } & RF_ICOMBINE as u32 != 0 {
            rex.set_reg_icombine(true);
        }
        rex.set_line(line);
        rex.set_lnum(0);
        rex.set_nfa_has_zend(unsafe { (*prog).has_zend });
        rex.set_nfa_has_backref(unsafe { (*prog).has_backref });
        rex.set_nfa_nsubexpr(unsafe { (*prog).nsubexp });
        rex.set_nfa_listid(1);
        rex.set_nfa_alt_listid(2);

        retval = match try_match(rex, prog, &mut col, tm, timed_out) {
            Attempt::Ran(retval) => retval,
            // The literal-text shortcut and the two "cannot match
            // here" answers report straight back, without the
            // start/end tidy-up below.
            Attempt::Done(retval) => return retval,
        };
    }

    if retval > 0 {
        // A `\ze` can put the end before the start; report an empty
        // match rather than a backwards one.
        if rex.multi() {
            let rmm = rex.reg_mmatch();
            let (start, end) = (unsafe { (*rmm).startpos[0] }, unsafe { (*rmm).endpos[0] });
            if end.lnum < start.lnum || (end.lnum == start.lnum && end.col < start.col) {
                unsafe { (*rmm).endpos[0] = start };
            }
        } else {
            let rm = rex.reg_match();
            if unsafe { (*rm).endp[0] } < unsafe { (*rm).startp[0] } {
                unsafe { (*rm).endp[0] = (*rm).startp[0] };
            }
            unsafe { (*rm).rm_matchcol = col };
        }
    }
    retval
}

/// What `try_match` settled on.
enum Attempt {
    /// The machine ran; the caller still tidies the reported positions.
    Ran(c_int),
    /// Report this and nothing more.
    Done(c_int),
}

/// The shortcuts, and the match itself.
///
/// SAFETY: As `nfa_regexec_both`.
fn try_match(
    rex: Rex,
    prog: *mut nfa_regprog_T,
    col: &mut colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> Attempt {
    // An anchored pattern can only match at the start of the line.
    if unsafe { (*prog).reganch } != 0 && *col > 0 {
        return Attempt::Done(0);
    }
    rex.set_need_clear_subexpr(1);
    let has_z = unsafe { (*prog).reghasz } == REX_SET;
    rex.set_nfa_has_zsubexpr(has_z as c_int);
    rex.set_need_clear_zsubexpr(has_z as c_int);

    if unsafe { (*prog).regstart } != NUL {
        // The first character is known: there is no point trying any
        // column before the next one that holds it.
        if skip_to_start(rex, unsafe { (*prog).regstart }, col) == FAIL {
            return Attempt::Done(0);
        }
        // And when the whole pattern is that literal run, the machine
        // has nothing to add. Not with 'regexpengine' combining
        // insensitivity, which the text comparison does not implement.
        if !unsafe { (*prog).match_text.is_null() }
            && unsafe { *(*prog).match_text } as c_int != NUL
            && !rex.reg_icombine()
        {
            let retval = find_match_text(rex, col, unsafe { (*prog).regstart }, unsafe {
                (*prog).match_text
            });
            rex.set_matchcol(*col);
            return Attempt::Done(retval);
        }
    }
    if rex.reg_maxcol() > 0 && *col >= rex.reg_maxcol() {
        return Attempt::Ran(0);
    }

    // Every state starts out on no list.
    nstate.set(0);
    let states = unsafe { &raw mut (*prog).state } as *mut nfa_state_T;
    for i in 0..unsafe { (*prog).nstate } {
        let s = unsafe { states.offset(i as isize) };
        unsafe { (*s).id = i };
        unsafe { (*s).lastlist = [0, 0] };
    }
    Attempt::Ran(nfa_regtry(rex, prog, *col, tm, timed_out))
}

/// Compile `expr` into a program, or null after reporting why not.
///
/// # Safety
///
/// `expr` must be null or a NUL-terminated pattern.
pub(crate) unsafe fn nfa_regcomp(expr: *mut uint8_t, re_flags: c_int) -> *mut regprog_T {
    if expr.is_null() {
        return core::ptr::null_mut();
    }
    nfa_re_flags.set(re_flags);
    // The compiler records what it found about the pattern (`\ze`, a
    // back-reference) in the same context a match later reads it from.
    // SAFETY: compiling, so no match is reading the context; only the
    // `nfa_*` fields are touched and they need no line set up.
    let rex = unsafe { Rex::acquire() };
    unsafe { nfa_regcomp_start(rex, expr, re_flags) };

    let mut prog: *mut nfa_regprog_T = core::ptr::null_mut();
    if re2post(rex).is_ok() {
        // The first pass counts the states, because the program is one
        // block with them inline.
        postfix::with_items(|items| post2nfa(items, Pass::Count));
        let size = 80 + size_of::<nfa_state_T>() * nstate.get() as usize;
        prog = unsafe { xmalloc(size) } as *mut nfa_regprog_T;
        state_ptr.set(unsafe { &raw mut (*prog).state } as *mut nfa_state_T);
        unsafe { (*prog).re_in_use = false };
        unsafe { (*prog).start = postfix::with_items(|items| post2nfa(items, Pass::Build)) };
        if unsafe { (*prog).start.is_null() } {
            unsafe { xfree(prog.cast()) };
            prog = core::ptr::null_mut();
        } else {
            unsafe { (*prog).regflags = regflags.get() };
            unsafe { (*prog).engine = (&raw const nfa_regengine).cast_mut() };
            unsafe { (*prog).nstate = nstate.get() };
            unsafe { (*prog).has_zend = rex.nfa_has_zend() };
            unsafe { (*prog).has_backref = rex.nfa_has_backref() };
            unsafe { (*prog).nsubexp = regnpar.get() };
            nfa_postprocess(prog);
            unsafe { (*prog).reganch = nfa_get_reganch((*prog).start, 0) };
            unsafe { (*prog).regstart = nfa_get_regstart((*prog).start, 0) };
            unsafe { (*prog).match_text = nfa_get_match_text((*prog).start) };
            unsafe { (*prog).reghasz = re_has_z.get() };
            unsafe { (*prog).pattern = xstrdup(expr as *mut c_char) };
        }
    }

    postfix::finish();
    state_ptr.set(core::ptr::null_mut::<nfa_state_T>());
    prog.cast()
}

/// Free a program this engine compiled.
///
/// # Safety
///
/// `prog` must be null or such a program.
pub(crate) unsafe fn nfa_regfree(prog: *mut regprog_T) {
    if prog.is_null() {
        return;
    }
    let prog = prog as *mut nfa_regprog_T;
    unsafe { xfree((*prog).match_text.cast()) };
    unsafe { xfree((*prog).pattern.cast()) };
    unsafe { xfree(prog.cast()) };
}

/// Match against a string, treating `\n` as an ordinary character when
/// `line_lbr` is set.
///
/// # Safety
///
/// `rmp` must hold a program this engine compiled, and `line` be a
/// NUL-terminated string.
pub(crate) unsafe fn nfa_regexec_nl(
    rmp: *mut regmatch_T,
    line: *mut uint8_t,
    col: colnr_T,
    line_lbr: bool,
) -> c_int {
    // SAFETY: the caller holds the context (`with_rex`) and hands us a
    // live match structure and a NUL-terminated line.
    let rex = unsafe { Rex::acquire() };
    init_regexec(rex, rmp, line_lbr);
    nfa_regexec_both(rex, line, col, core::ptr::null_mut(), core::ptr::null_mut())
}

/// Match against a buffer, starting at `lnum`.
///
/// # Safety
///
/// `rmp` must hold a program this engine compiled, and `buf`/`win` be the
/// buffer and window the match runs over.
pub(crate) unsafe fn nfa_regexec_multi(
    rmp: *mut regmmatch_T,
    win: *mut win_T,
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: the caller holds the context (`with_rex`) and hands us a live
    // match structure over a live buffer; `init_regexec_multi` points the
    // context at them, which is what `nfa_regexec_both` reads it out of.
    let rex = unsafe { Rex::acquire() };
    init_regexec_multi(rex, rmp, win, buf, lnum);
    nfa_regexec_both(rex, core::ptr::null_mut::<uint8_t>(), col, tm, timed_out)
}
