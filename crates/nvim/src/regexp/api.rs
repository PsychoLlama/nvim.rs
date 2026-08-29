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

use core::ffi::{c_char, c_int};

use super::{
    AUTOMATIC_ENGINE, BACKTRACKING_ENGINE, E_RECURSIVE, NFA_ENGINE, NFA_TOO_EXPENSIVE, REX_ALL,
    Rex, bt_regengine, nfa_regengine, nfa_regprog_T, regexp_engine, rex_in_use,
};
use crate::main::{called_emsg, curbuf, p_re, p_verbose, reg_do_extmatch};
use crate::memory::{xfree, xstrdup};
use crate::message::{emsg, msg_puts, verbose_enter, verbose_leave};
use crate::os::cshim::{gettext, gettext_ptr, strncmp};
use crate::regexp::RE_AUTO;
use crate::types::{
    OptInt, buf_T, colnr_T, linenr_T, proftime_T, regmatch_T, regmmatch_T, regprog_T, uint8_t,
    win_T,
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
///
/// # Safety
/// `expr_arg` must be a NUL-terminated pattern, borrowed for the call only.
pub unsafe fn vim_regcomp(expr_arg: *const c_char, re_flags: c_int) -> *mut regprog_T {
    // SAFETY: `expr_arg` is the caller's NUL-terminated pattern, and the
    // engine table's entries are set at compile time.
    let mut expr = expr_arg;
    regexp_engine.set(p_re.get() as c_int);
    if unsafe { strncmp(expr, c"\\%#=".as_ptr(), 4) } == 0 {
        let chosen = unsafe { *expr.offset(4) } as c_int - '0' as c_int;
        if chosen == AUTOMATIC_ENGINE as c_int
            || chosen == BACKTRACKING_ENGINE as c_int
            || chosen == NFA_ENGINE as c_int
        {
            regexp_engine.set(chosen);
            expr = unsafe { expr.offset(5) };
        } else {
            let fmt = c"E864: \\%#= can only be followed by 0, 1, or 2. The automatic engine will be used "
                .as_ptr();
            // SAFETY: a static, NUL-terminated message.
            unsafe { emsg(gettext_ptr(fmt)) };
            regexp_engine.set(AUTOMATIC_ENGINE as c_int);
        }
    }
    // The pattern can name a buffer-local thing (`\k`, say) while it is
    // being compiled, so point the context at a buffer.
    // SAFETY: nothing is matching, so nothing else holds the context;
    // only `reg_buf` is touched, which needs no line set up.
    unsafe { Rex::acquire() }.set_reg_buf(curbuf.get());

    let called_emsg_before = called_emsg.get();
    let mut prog = if regexp_engine.get() != BACKTRACKING_ENGINE as c_int {
        let auto = if regexp_engine.get() == AUTOMATIC_ENGINE as c_int {
            RE_AUTO
        } else {
            0
        };
        let regcomp = nfa_regengine.regcomp.expect("non-null function pointer");
        unsafe { regcomp(expr as *mut uint8_t, re_flags + auto) }
    } else {
        let regcomp = bt_regengine.regcomp.expect("non-null function pointer");
        unsafe { regcomp(expr as *mut uint8_t, re_flags) }
    };
    // Only retry when the NFA engine declined quietly: an error means the
    // pattern is bad, not merely too much for that engine.
    if prog.is_null()
        && regexp_engine.get() == AUTOMATIC_ENGINE as c_int
        && called_emsg.get() == called_emsg_before
    {
        regexp_engine.set(BACKTRACKING_ENGINE as c_int);
        if p_verbose.get() > 0 as OptInt {
            unsafe { verbose_enter() };
            let note = c"Switching to backtracking RE engine for pattern: ".as_ptr();
            unsafe { msg_puts(gettext_ptr(note).as_ptr()) };
            unsafe { msg_puts(expr) };
            unsafe { verbose_leave() };
        }
        let regcomp = bt_regengine.regcomp.expect("non-null function pointer");
        prog = unsafe { regcomp(expr as *mut uint8_t, re_flags) };
    }
    if !prog.is_null() {
        unsafe { (*prog).re_engine = regexp_engine.get() as u32 };
        unsafe { (*prog).re_flags = re_flags as u32 };
    }
    prog
}

/// Release a program [`vim_regcomp`] answered. Null is a no-op.
///
/// # Safety
/// `prog` must be null, or a program from [`vim_regcomp`] that has not been
/// freed and that nothing still points at — a match may have *replaced* the
/// program in its [`regmatch_T`], so free what the match left behind rather
/// than what was put in.
pub unsafe fn vim_regfree(prog: *mut regprog_T) {
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
unsafe fn recompile_backtracking(prog: *mut regprog_T, extmatch: bool) -> *mut regprog_T {
    // SAFETY: `prog` is a live NFA program, so it carries a pattern.
    let re_flags = unsafe { (*prog).re_flags } as c_int;
    let pat = unsafe { xstrdup((*(prog as *mut nfa_regprog_T)).pattern) };
    let save_p_re = p_re.get();
    p_re.set(BACKTRACKING_ENGINE as c_int as OptInt);
    if p_verbose.get() > 0 as OptInt {
        unsafe { verbose_enter() };
        unsafe { msg_puts(gettext(c"Switching to backtracking RE engine for pattern: ").as_ptr()) };
        unsafe { msg_puts(pat) };
        unsafe { verbose_leave() };
    }
    if extmatch {
        // A buffer match may be a syntax match, whose `\z(` groups have
        // to survive the recompile.
        reg_do_extmatch.set(REX_ALL);
    }
    let new = unsafe { vim_regcomp(pat, re_flags) };
    if extmatch {
        reg_do_extmatch.set(0);
    }
    unsafe { xfree(pat.cast()) };
    p_re.set(save_p_re);
    new
}

/// Run `rmp`'s program over the single line `line`, starting at `col`.
/// `nl` allows a `$` to match at the end of the string.
unsafe fn vim_regexec_string(
    rmp: *mut regmatch_T,
    line: *const c_char,
    col: colnr_T,
    nl: bool,
) -> bool {
    // SAFETY: `rmp` holds a live program and `line` is the caller's text.
    // A program cannot match against itself: `\=` calling back into the
    // same pattern would reuse the program's own state.
    if unsafe { (*(*rmp).regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return false;
    }
    let result = with_rex(|| {
        unsafe { (*(*rmp).regprog).re_in_use = true };
        // A string match has no position slots, only pointer ones.
        // SAFETY: `with_rex` reserved the context for this match.
        let rex = unsafe { Rex::acquire() };
        rex.set_reg_startp(core::ptr::null_mut());
        rex.set_reg_endp(core::ptr::null_mut());
        rex.set_reg_startpos(core::ptr::null_mut());
        rex.set_reg_endpos(core::ptr::null_mut());
        let exec = |rmp: *mut regmatch_T| unsafe {
            (*(*(*rmp).regprog).engine)
                .regexec_nl
                .expect("non-null function pointer")(rmp, line as *mut uint8_t, col, nl)
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
        result
    });
    result > 0
}

/// [`vim_regexec`] against a program the caller owns by pointer, so that
/// the fall back to the backtracking engine can replace it.
pub unsafe fn vim_regexec_prog(
    prog: *mut *mut regprog_T,
    ignore_case: bool,
    line: *const c_char,
    col: colnr_T,
) -> bool {
    // SAFETY: `prog` points at the caller's program handle.
    let mut regmatch = regmatch_T {
        regprog: unsafe { *prog },
        startp: [core::ptr::null_mut(); 10],
        endp: [core::ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: ignore_case,
    };
    let matched = unsafe { vim_regexec_string(&raw mut regmatch, line, col, false) };
    unsafe { *prog = regmatch.regprog };
    matched
}

/// Run `rmp`'s program over the string `line`, starting at byte `col`.
///
/// On a hit `rmp`'s `startp`/`endp` point into `line`, so `line` has to
/// outlive every read of them.
///
/// # Safety
/// `rmp` must be writable and hold a live program, `line` must be
/// NUL-terminated, and `col` must be within it. This re-enters the editor —
/// a `\=` expression can start a match of its own — so nothing may be held
/// across it.
pub unsafe fn vim_regexec(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool {
    // SAFETY: as `vim_regexec_string`.
    unsafe { vim_regexec_string(rmp, line, col, false) }
}

/// [`vim_regexec`] with `$` allowed to match at the end of the string.
pub unsafe fn vim_regexec_nl(rmp: *mut regmatch_T, line: *const c_char, col: colnr_T) -> bool {
    // SAFETY: as `vim_regexec_string`.
    unsafe { vim_regexec_string(rmp, line, col, true) }
}

/// Run `rmp`'s program over `buf` starting at line `lnum`, column `col`.
/// Returns the number of lines the match spans plus one, or 0 for no
/// match; `tm`/`timed_out` bound how long the NFA engine may spend.
pub unsafe fn vim_regexec_multi(
    rmp: *mut regmmatch_T,
    win: *mut win_T,
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    tm: *mut proftime_T,
    timed_out: *mut c_int,
) -> c_int {
    // SAFETY: `rmp` holds a live program; `win`/`buf`/`tm`/`timed_out` are
    // the caller's and may be null where the engines allow it.
    if unsafe { (*(*rmp).regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return 0;
    }
    let result = with_rex(|| {
        unsafe { (*(*rmp).regprog).re_in_use = true };
        let exec = |rmp: *mut regmmatch_T| unsafe {
            (*(*(*rmp).regprog).engine)
                .regexec_multi
                .expect("non-null function pointer")(
                rmp, win, buf, lnum, col, tm, timed_out
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
