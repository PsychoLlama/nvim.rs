//! The public entry points: compiling a pattern and running it over a
//! string or a buffer.
//!
//! Two things happen here that the engines below do not see. One is
//! engine selection: `'regexpengine'` and a leading `\%#=` pick one, and
//! when the NFA engine gives up on a pattern (`NFA_TOO_EXPENSIVE`) the
//! pattern is recompiled for the backtracking engine and rerun. The other
//! is the `rex` handover — a match may run a `\=` expression that starts
//! a match of its own, so the context is saved and restored around every
//! run rather than assumed to be free.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::memory::XString;
use crate::regexp::ExtMatchIo;
use crate::regexp::ExtMatchRef;
use crate::regexp::RegCompiler;
use crate::regexp::bt_regcomp;
use crate::regexp::nfa_regcomp;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use super::{
    AUTOMATIC_ENGINE, BACKTRACKING_ENGINE, E_RECURSIVE, NFA_ENGINE, NFA_TOO_EXPENSIVE, NfaRegProg,
    REX_ALL, Rex, rex_in_use,
};
use crate::message::state::called_emsg;
use crate::message::{emsg, msg_str, verbose_enter, verbose_leave};
use crate::option::vars::{P_RE, p_re, p_verbose};
use crate::os::cshim::gettext;
use crate::regexp::RE_AUTO;
use crate::types::{
    ColNr, LineNr, OptInt, ProfTime, RE_GROUPS, RegMMatch, RegMatch, RegProg, uint8_t,
};

/// Reserve `rex` for `run`, restoring an outer match's context after. The
/// nesting is real: `:s/…/\=…/` can evaluate an expression that searches.
pub(crate) fn with_rex<R>(run: impl FnOnce() -> R) -> R {
    let outer = rex_in_use.get();
    let saved = outer.then(|| super::rex.get());
    rex_in_use.set(true);
    let result = run();
    rex_in_use.set(outer);
    if let Some(saved) = saved {
        super::rex.set(saved);
    }
    result
}

/// Compile `expr` into a program. A leading `\%#=0`, `\%#=1` or `\%#=2`
/// overrides 'regexpengine' for this pattern; with the automatic setting
/// the NFA engine is tried first and a failure that reported no error
/// falls back to the backtracking one.
///
/// Answers null when the pattern does not parse, having reported why;
/// otherwise the caller owns the program and frees it with [`vim_regfree`].
pub fn vim_regcomp(expr_arg: &CStr, re_flags: c_int) -> *mut RegProg {
    vim_regcomp_ext(expr_arg, re_flags, 0)
}

/// [`vim_regcomp`] for a syntax pattern: `extmatch` says which `\z`
/// specials it may use -- `REX_SET` for a region's start pattern, which
/// defines `\z(` groups, `REX_USE` for its skip and end patterns, which
/// match them with `\z1`..`\z9`.
pub(crate) fn vim_regcomp_ext(expr_arg: &CStr, re_flags: c_int, extmatch: c_int) -> *mut RegProg {
    let mut expr = expr_arg;
    let mut engine = p_re() as c_int;
    if expr.to_bytes().starts_with(b"\\%#=") {
        // The prefix may be the whole pattern, and then the byte after it
        // is the terminator -- which is not a digit, so the error below is
        // what upstream reports for it too.
        let chosen = c_int::from(cstr::byte_at(expr.to_bytes(), 4)) - '0' as c_int;
        if chosen == AUTOMATIC_ENGINE as c_int
            || chosen == BACKTRACKING_ENGINE as c_int
            || chosen == NFA_ENGINE as c_int
        {
            engine = chosen;
            expr = cstr::suffix(expr, 5);
        } else {
            emsg(gettext(
                c"E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used ",
            ));
            engine = AUTOMATIC_ENGINE as c_int;
        }
    }
    // The pattern can name a buffer-local thing (`\k`, say) while it is
    // being compiled, so the compiler reads the current buffer's.
    let buf = Buf::current();

    let called_emsg_before = called_emsg.get();
    let mut prog = if engine != BACKTRACKING_ENGINE as c_int {
        let auto = if engine == AUTOMATIC_ENGINE as c_int {
            RE_AUTO
        } else {
            0
        };
        nfa_regcomp(&mut RegCompiler::new(expr, re_flags + auto, buf, extmatch))
    } else {
        bt_regcomp(&mut RegCompiler::new(expr, re_flags, buf, extmatch))
    };
    // Only retry when the NFA engine declined quietly: an error means the
    // pattern is bad, not merely too much for that engine.
    if prog.is_null()
        && engine == AUTOMATIC_ENGINE as c_int
        && called_emsg.get() == called_emsg_before
    {
        engine = BACKTRACKING_ENGINE as c_int;
        if p_verbose() > 0 as OptInt {
            verbose_enter();
            msg_str(gettext(
                c"Switching to backtracking RE engine for pattern: ",
            ));
            msg_str(expr);
            verbose_leave();
        }
        prog = bt_regcomp(&mut RegCompiler::new(expr, re_flags, buf, extmatch));
    }
    if !prog.is_null() {
        // SAFETY: a program one of the engines just built.
        unsafe { (*prog).re_engine = engine as u32 };
        unsafe { (*prog).re_flags = re_flags as u32 };
    }
    prog
}

/// Release a program [`vim_regcomp`] answered. Null is a no-op.
///
/// # Safety
/// `prog` must be null, or a program from [`vim_regcomp`] that has not been
/// freed and that nothing still points at — a match may have *replaced* the
/// program in its [`RegMatch`], so free what the match left behind rather
/// than what was put in.
pub unsafe fn vim_regfree(prog: *mut RegProg) {
    // SAFETY: `prog` is null or a program one of the engines produced.
    if !prog.is_null() {
        unsafe {
            (*(*prog).engine)
                .regfree
                .expect("non-null function pointer")(prog)
        };
    }
}

/// Recompile the NFA program `prog` for the backtracking engine, which is
/// what a `NFA_TOO_EXPENSIVE` result asks for. The pattern text is copied
/// out first because compiling frees the program that holds it.
///
/// # Safety
///
/// `prog` must point at a live `RegProg`, unaliased for the call.
unsafe fn recompile_backtracking(prog: *mut RegProg, extmatch: bool) -> *mut RegProg {
    // SAFETY: `prog` is a live NFA program, so it carries a pattern.
    let re_flags = unsafe { (*prog).re_flags } as c_int;
    let pat = XString::from_cstr(unsafe { cstr::at((*(prog as *mut NfaRegProg)).pattern) });
    let save_p_re = p_re();
    P_RE.set(BACKTRACKING_ENGINE as c_int as OptInt);
    if p_verbose() > 0 as OptInt {
        verbose_enter();
        msg_str(gettext(
            c"Switching to backtracking RE engine for pattern: ",
        ));
        msg_str(pat.as_cstr());
        verbose_leave();
    }
    // A buffer match may be a syntax match, whose `\z(` groups have to
    // survive the recompile.
    let new = vim_regcomp_ext(pat.as_cstr(), re_flags, if extmatch { REX_ALL } else { 0 });
    P_RE.set(save_p_re);
    new
}

/// Run `matches`'s program over the whole of `line`, starting at byte
/// `col`. `nl` allows a `$` to match at the end of the string.
///
/// The line is a `&CStr` and not a `&[u8]` on purpose. Both engines treat
/// the terminator as a *position* rather than a bound — `$` matches there,
/// `\n` is tested for there, and every character step reads it before
/// deciding it has run out — so a length alone would not tell them where
/// the line ends. Taking the terminated string says that, and hands the
/// callers a type they can hold.
fn vim_regexec_string(matches: &mut RegMatch, line: &CStr, col: usize, nl: bool) -> bool {
    let text = line.as_ptr();
    debug_assert!(col <= line.count_bytes(), "col past the end of the line");
    let col = ColNr::try_from(col).unwrap_or(ColNr::MAX);
    let rmp: *mut RegMatch = matches;
    // SAFETY: `rmp` is the caller's match structure, borrowed exclusively
    // for the call, and `text` its NUL-terminated line.
    // A program cannot match against itself: `\=` calling back into the
    // same pattern would reuse the program's own state.
    if unsafe { (*(*rmp).regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return false;
    }
    let result = with_rex(|| {
        unsafe { (*(*rmp).regprog).re_in_use = true };
        // A string match has no position slots, only the pointer ones the
        // context holds itself.
        // SAFETY: `with_rex` reserved the context for this match.
        let rex = unsafe { Rex::acquire() };
        rex.set_reg_startpos(core::ptr::null_mut());
        rex.set_reg_endpos(core::ptr::null_mut());
        let exec = |rmp: *mut RegMatch| unsafe {
            (*(*(*rmp).regprog).engine)
                .regexec_nl
                .expect("non-null function pointer")(rmp, text as *mut uint8_t, col, nl)
        };
        let mut result = exec(rmp);
        unsafe { (*(*rmp).regprog).re_in_use = false };
        if unsafe { (*(*rmp).regprog).re_engine } == AUTOMATIC_ENGINE as c_int as u32
            && result == NFA_TOO_EXPENSIVE as c_int
        {
            let prev = unsafe { (*rmp).regprog };
            unsafe { (*rmp).regprog = recompile_backtracking(prev, false) };
            unsafe { vim_regfree(prev) };
            if !unsafe { (*rmp).regprog.is_null() } {
                unsafe { (*(*rmp).regprog).re_in_use = true };
                result = exec(rmp);
                unsafe { (*(*rmp).regprog).re_in_use = false };
            }
        }
        // The slots have to be read before `with_rex` puts an outer
        // match's context back, and `matches` must not be touched while
        // `rmp` is live, so what comes out is a copy.
        // SAFETY: the context is still this match's.
        let rex = unsafe { Rex::acquire() };
        (result, rex.str_starts(), rex.str_ends())
    });
    let (result, starts, ends) = result;
    if result > 0 {
        record_groups(matches, &starts, &ends, line);
    } else {
        matches.clear_groups();
    }
    result > 0
}

/// Copy the slots the engine filled into `matches`, as offsets into `line`.
///
/// A slot is null for a group the pattern never reached, and otherwise
/// points into the line the match ran over — which for a string match is
/// `line` itself from the first byte to the last: nothing in either engine
/// moves a string match onto another line.
fn record_groups(
    matches: &mut RegMatch,
    starts: &[*mut uint8_t; RE_GROUPS],
    ends: &[*mut uint8_t; RE_GROUPS],
    line: &CStr,
) {
    let first = line.as_ptr().cast::<uint8_t>();
    let len = line.count_bytes();
    let offset = |slot: *mut uint8_t| {
        if slot.is_null() {
            return None;
        }
        // SAFETY: the slot is a position inside the line just matched.
        let at = unsafe { slot.offset_from(first) };
        debug_assert!(
            (0..=len as isize).contains(&at),
            "capture outside the matched line"
        );
        Some(at as usize)
    };
    for no in 0..RE_GROUPS {
        matches.starts[no] = offset(starts[no]);
        matches.ends[no] = offset(ends[no]);
    }
}

/// [`vim_regexec`] against a program the caller owns, so that the fall back
/// to the backtracking engine can replace it.
pub fn vim_regexec_prog(
    prog: &mut *mut RegProg,
    ignore_case: bool,
    line: &CStr,
    col: usize,
) -> bool {
    let mut regex_match = RegMatch::new(*prog, ignore_case);
    let matched = vim_regexec_string(&mut regex_match, line, col, false);
    *prog = regex_match.regprog;
    matched
}

/// Run `matches`'s program over `line`, starting at byte `col`.
///
/// On a hit `matches`'s groups are byte offsets into `line`; on a miss they
/// are all unset. This re-enters the editor — a `\=` expression can start a
/// match of its own — so nothing may be held across it.
pub fn vim_regexec(matches: &mut RegMatch, line: &CStr, col: usize) -> bool {
    vim_regexec_string(matches, line, col, false)
}

/// [`vim_regexec`] with `$` allowed to match at the end of the string.
pub fn vim_regexec_nl(matches: &mut RegMatch, line: &CStr, col: usize) -> bool {
    vim_regexec_string(matches, line, col, true)
}

/// Run `rmp`'s program over `buffer` starting at line `lnum`, column `col`.
/// Returns the number of lines the match spans plus one, or 0 for no
/// match; `tm`/`timed_out` bound how long the NFA engine may spend.
///
/// # Safety
///
/// `rmp` must point at a live `RegMMatch`, unaliased for the call. `tm` must
/// point at a live `ProfTime`, unaliased for the call. `timed_out` must point
/// at a writable `int` the caller owns.
pub unsafe fn vim_regexec_multi(
    rmp: *mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: `rmp` holds a live program; `win`/`buffer`/`tm`/`timed_out` are
    // the caller's and may be null where the engines allow it.
    if unsafe { (*(*rmp).regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return 0;
    }
    let result = with_rex(|| {
        unsafe { (*(*rmp).regprog).re_in_use = true };
        let exec = |rmp: *mut RegMMatch| unsafe {
            (*(*(*rmp).regprog).engine)
                .regexec_multi
                .expect("non-null function pointer")(
                rmp, win, buffer, lnum, col, tm, timed_out
            )
        };
        let mut result = exec(rmp);
        unsafe { (*(*rmp).regprog).re_in_use = false };
        if unsafe { (*(*rmp).regprog).re_engine } == AUTOMATIC_ENGINE as c_int as u32
            && result == NFA_TOO_EXPENSIVE as c_int
        {
            let prev = unsafe { (*rmp).regprog };
            let new = unsafe { recompile_backtracking(prev, true) };
            // Unlike the string case, a failed recompile keeps the old
            // program rather than leaving the caller without one.
            if new.is_null() {
                unsafe { (*rmp).regprog = prev };
            } else {
                unsafe { (*rmp).regprog = new };
                unsafe { vim_regfree(prev) };
                unsafe { (*(*rmp).regprog).re_in_use = true };
                result = exec(rmp);
                unsafe { (*(*rmp).regprog).re_in_use = false };
            }
        }
        result
    });
    result.max(0)
}

#[allow(
    clippy::too_many_arguments,
    reason = "vim_regexec_multi's, and the `\\z` traffic"
)]
/// [`vim_regexec_multi`] for a syntax pattern: `io.input` is what `\z1`..
/// `\z9` match, and a successful match leaves its own `\z(` captures in
/// `io.output` (or `None` when it has none).
///
/// The two are aimed at for the match and cleared after, outside the context
/// reservation: a syntax match never runs inside another match (the matcher
/// runs no user code), and every other match finds them cleared, so the plain
/// entry point pays nothing for them.
///
/// # Safety
///
/// As for [`vim_regexec_multi`].
pub(crate) unsafe fn vim_regexec_syntax(
    rmp: *mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
    io: ExtMatchIo<'_>,
) -> LineNr {
    debug_assert!(!rex_in_use.get(), "a syntax match inside another match");
    let ext_in = io.input.map_or(core::ptr::null(), core::ptr::from_ref);
    let ext_out: *mut Option<ExtMatchRef> = io.output.map_or(core::ptr::null_mut(), |out| out);
    super::rex.with_mut(|rex| (rex.ext_in, rex.ext_out) = (ext_in, ext_out));
    // SAFETY: the caller's promises; the two sets are its borrows, live
    // until the fields are cleared below.
    let result = unsafe { vim_regexec_multi(rmp, win, buffer, lnum, col, tm, timed_out) };
    super::rex.with_mut(|rex| {
        (rex.ext_in, rex.ext_out) = (core::ptr::null(), core::ptr::null_mut());
    });
    result
}
