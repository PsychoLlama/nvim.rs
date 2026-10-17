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
use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::regexp::ExtMatchIo;
use crate::regexp::RegCompiler;
use crate::regexp::bt_regcomp;
use crate::regexp::nfa_regcomp;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};
use core::ptr;

use super::{
    AUTOMATIC_ENGINE, BACKTRACKING_ENGINE, BtRegProg, E_RECURSIVE, MatchPos, MatchSetup,
    NFA_ENGINE, NFA_TOO_EXPENSIVE, NfaRegProg, RE_NOBREAK, REX_ALL, RegExec, Rex, bt_regexec,
    nfa_regexec,
};
use crate::message::state::called_emsg;
use crate::message::{emsg, msg_str, verbose_enter, verbose_leave};
use crate::option::vars::{P_RE, p_re, p_verbose};
use crate::os::cshim::gettext;
use crate::regexp::RE_AUTO;
use crate::types::{ColNr, LineNr, OptInt, ProfTime, RE_GROUPS, RegMMatch, RegMatch, RegProg};

/// The first line of the last buffer match, which upstream's `rex` kept
/// after the match was over: a string match never set it, so `\%'m` in one
/// compares the mark against that line. Kept here so that it still does.
static LAST_FIRSTLNUM: GlobalCell<LineNr> = GlobalCell::new(0);

/// How many matches (or expansions) are running, one inside another.
static MATCH_DEPTH: GlobalCell<c_int> = GlobalCell::new(0);

/// Run `f` as one of upstream's `rex` reservations: a buffer match or a
/// buffer expansion records its first line (`first`), and `f` is handed the
/// line in force. A nested reservation puts the outer one's line back when it
/// is done, as restoring the whole of `rex` around a nested match did.
pub(crate) fn in_match_scope<R>(first: Option<LineNr>, f: impl FnOnce(LineNr) -> R) -> R {
    let depth = MATCH_DEPTH.get();
    let saved = LAST_FIRSTLNUM.get();
    if let Some(first) = first {
        LAST_FIRSTLNUM.set(first);
    }
    MATCH_DEPTH.set(depth + 1);
    let result = f(LAST_FIRSTLNUM.get());
    MATCH_DEPTH.set(depth);
    if depth > 0 {
        LAST_FIRSTLNUM.set(saved);
    }
    result
}

/// Run `prog` over the match `rex` is set up for, from column `col`: the
/// engine the program came from decides which engine runs it.
///
/// # Safety
///
/// `prog` must be a live program [`vim_regcomp`] answered.
unsafe fn run_engine(prog: *mut RegProg, rex: Rex, col: ColNr) -> c_int {
    // SAFETY: the caller's live program, whose engine field says which shape
    // it is; a backtracking program is a `BtRegProg` (see `bt_regcomp`).
    if unsafe { (*prog).engine.cast_const() } == &raw const super::bt_regengine {
        bt_regexec(rex, unsafe { &*prog.cast::<BtRegProg>() }, col)
    } else {
        unsafe { nfa_regexec(rex, prog.cast::<NfaRegProg>(), col) }
    }
}

/// Was `prog` compiled with `RE_NOBREAK`?
///
/// # Safety
///
/// `prog` must be a live program.
unsafe fn nobreak(prog: *mut RegProg) -> bool {
    (unsafe { (*prog).re_flags } & RE_NOBREAK as u32) != 0
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
    debug_assert!(col <= line.count_bytes(), "col past the end of the line");
    let col = ColNr::try_from(col).unwrap_or(ColNr::MAX);
    let text = line.to_bytes_with_nul();
    // A string match has no buffer of its own, but `\k` and friends still
    // need an 'iskeyword' to read.
    let buf = Buf::current();
    let ic = matches.rm_ic;
    // SAFETY: the caller's match structure holds a live program, or none.
    // A program cannot match against itself: `\=` calling back into the
    // same pattern would reuse the program's own state.
    if unsafe { (*matches.regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return false;
    }
    let run = |prog: *mut RegProg, firstlnum: LineNr| {
        // SAFETY: a live program, marked in use for the run.
        unsafe { (*prog).re_in_use = true };
        let setup = MatchSetup {
            buf: &buf,
            win: None,
            ic,
            nobreak: unsafe { nobreak(prog) },
            deadline: None,
            ext_in: None,
        };
        let ctx = RegExec::for_string(setup, text, nl, firstlnum);
        let result = unsafe { run_engine(prog, ctx.rex(), col) };
        unsafe { (*prog).re_in_use = false };
        (result, ctx.starts(), ctx.ends(), ctx.matchcol())
    };
    let ran = in_match_scope(None, |firstlnum| {
        let mut ran = run(matches.regprog, firstlnum);
        if unsafe { (*matches.regprog).re_engine } == AUTOMATIC_ENGINE && ran.0 == NFA_TOO_EXPENSIVE
        {
            let prev = matches.regprog;
            matches.regprog = unsafe { recompile_backtracking(prev, false) };
            unsafe { vim_regfree(prev) };
            if !matches.regprog.is_null() {
                ran = run(matches.regprog, firstlnum);
            }
        }
        ran
    });
    let (result, starts, ends, matchcol) = ran;
    if let Some(col) = matchcol {
        matches.rm_matchcol = col;
    }
    if result > 0 {
        record_groups(matches, &starts, &ends);
    } else {
        matches.clear_groups();
    }
    result > 0
}

/// Copy the slots the engine filled into `matches`, as offsets into the
/// string the match ran over — which for a string match holds every
/// position: nothing in either engine moves a string match onto another
/// line.
fn record_groups(
    matches: &mut RegMatch,
    starts: &[MatchPos; RE_GROUPS],
    ends: &[MatchPos; RE_GROUPS],
) {
    let offset = |slot: &MatchPos| {
        let pos = slot.as_pos();
        (pos.lnum >= 0).then(|| usize::try_from(pos.col).unwrap_or(0))
    };
    for no in 0..RE_GROUPS {
        matches.starts[no] = offset(&starts[no]);
        matches.ends[no] = offset(&ends[no]);
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
// Kept out of line: inlined into `searchit` and `next_search_hl` (as the
// optimizer chose on its own once unrelated code moved), the body below
// leaves its closure as a call of its own and costs those loops ~0.8 % of
// the memline benchmark's instructions and ~1.3 % of its time.
#[inline(never)]
pub unsafe fn vim_regexec_multi(
    rmp: *mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    let io = ExtMatchIo {
        input: None,
        output: None,
    };
    // SAFETY: the caller's promises.
    unsafe { regexec_multi_io(rmp, win, buffer, lnum, col, tm, timed_out, io) }
}

#[allow(
    clippy::too_many_arguments,
    reason = "vim_regexec_multi's, and the `\\z` traffic"
)]
/// [`vim_regexec_multi`] with a syntax match's `\z` traffic.
///
/// # Safety
///
/// As for [`vim_regexec_multi`].
unsafe fn regexec_multi_io(
    rmp: *mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: *mut ProfTime,
    timed_out: *mut c_int,
    io: ExtMatchIo<'_>,
) -> c_int {
    // SAFETY: the caller's match structure, borrowed for the call; `tm` and
    // `timed_out` are null or the caller's.
    let rmm = unsafe { &mut *rmp };
    if unsafe { (*rmm.regprog).re_in_use } {
        emsg(gettext(E_RECURSIVE));
        return 0;
    }
    let deadline = unsafe { tm.as_ref() }.copied();
    let ext_in = io.input;
    let mut ext_out = io.output;
    let mut run = |rmm: &mut RegMMatch| {
        let prog = rmm.regprog;
        // SAFETY: a live program, marked in use for the run.
        unsafe { (*prog).re_in_use = true };
        let setup = MatchSetup {
            buf: &buffer,
            win,
            ic: rmm.rmm_ic != 0,
            nobreak: unsafe { nobreak(prog) },
            deadline,
            ext_in,
        };
        let ctx = RegExec::for_buffer(setup, lnum, rmm.rmm_maxcol, &rmm.startpos, &rmm.endpos);
        let result = unsafe { run_engine(prog, ctx.rex(), col) };
        unsafe { (*prog).re_in_use = false };
        // Upstream matched into the caller's slots in place, so they hold
        // what the match left there whether or not it succeeded.
        let (starts, ends) = (ctx.starts(), ctx.ends());
        for no in 0..RE_GROUPS {
            rmm.startpos[no] = starts[no].as_pos();
            rmm.endpos[no] = ends[no].as_pos();
        }
        if let Some(col) = ctx.matchcol() {
            rmm.rmm_matchcol = col;
        }
        if ctx.timed_out() && !timed_out.is_null() {
            // SAFETY: the caller's flag.
            unsafe { *timed_out = 1 };
        }
        if let (Some(out), Some(captures)) = (ext_out.as_deref_mut(), ctx.take_ext_out()) {
            *out = captures;
        }
        result
    };
    let result = in_match_scope(Some(lnum), |_| {
        let mut result = run(rmm);
        if unsafe { (*rmm.regprog).re_engine } == AUTOMATIC_ENGINE && result == NFA_TOO_EXPENSIVE {
            let prev = rmm.regprog;
            let new = unsafe { recompile_backtracking(prev, true) };
            // Unlike the string case, a failed recompile keeps the old
            // program rather than leaving the caller without one.
            if !new.is_null() {
                rmm.regprog = new;
                unsafe { vim_regfree(prev) };
                result = run(rmm);
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
    // SAFETY: the caller's promises.
    unsafe { regexec_multi_io(rmp, win, buffer, lnum, col, tm, timed_out, io) }
}

/// Did `prog`'s pattern end with a `\n` that a search should treat as
/// "match at end of line"? False for a null program. Read by the syntax
/// code right after a compile.
pub(crate) fn vim_regcomp_had_eol(prog: *const RegProg) -> bool {
    // SAFETY: a program `vim_regcomp` answered, or null.
    !prog.is_null() && unsafe { (*prog).regflags } & super::RF_HAD_EOL != 0
}

/// Release a backtracking program: the box `bt_regcomp` handed over.
///
/// # Safety
///
/// `prog` must be a program `bt_regcomp` answered, not yet freed.
pub(crate) unsafe fn bt_regfree(prog: *mut RegProg) {
    // SAFETY: the caller's promise -- the box `bt_regcomp` leaked.
    drop(unsafe { Box::from_raw(prog.cast::<BtRegProg>()) });
}

/// Does `prog` belong to the backtracking engine, and has its magic number
/// not survived? Only that engine's programs carry one. Reports so and
/// answers 1 when it has not.
pub(crate) fn prog_magic_wrong(prog: *mut RegProg) -> c_int {
    // SAFETY: a live program `vim_regcomp` answered, whose engine field says
    // which shape it is.
    let bt = unsafe { (*prog).engine.cast_const() } == &raw const super::bt_regengine;
    if bt && unsafe { &*prog.cast::<BtRegProg>() }.program[0] != super::REGMAGIC as u8 {
        emsg(gettext(crate::message::e_re_corr));
        return 1;
    }
    0
}

/// Can `prog`'s pattern match across a line break?
///
/// # Safety
///
/// `prog` must point at a live `RegProg`.
pub unsafe fn re_multiline(prog: *const RegProg) -> bool {
    (unsafe { (*prog).regflags } & super::RF_HASNL as u32) != 0
}

/// Free the program a field owns and empty the field: upstream's
/// `vim_regfree(p); p = NULL` for a slot that holds its own program.
pub(crate) fn free_prog_in(slot: &mut *mut RegProg) {
    // SAFETY: the slot's own program, or null.
    unsafe { vim_regfree(*slot) };
    *slot = ptr::null_mut();
}

/// A compiled pattern its holder owns: freed when dropped, and replaced in
/// place when the engine falls back to the other one.
pub(crate) struct OwnedProg(*mut RegProg);

impl OwnedProg {
    /// [`vim_regcomp`], owned. `None` when the pattern does not compile.
    pub(crate) fn compile(expr: &CStr, re_flags: c_int) -> Option<OwnedProg> {
        let prog = vim_regcomp(expr, re_flags);
        (!prog.is_null()).then_some(OwnedProg(prog))
    }

    /// [`vim_regexec`] of this pattern over `line` from byte `col`: the
    /// match, or `None`.
    pub(crate) fn exec(&mut self, line: &CStr, col: usize, ignore_case: bool) -> Option<RegMatch> {
        if self.0.is_null() {
            return None;
        }
        let mut matches = RegMatch::new(self.0, ignore_case);
        let matched = vim_regexec_string(&mut matches, line, col, false);
        self.0 = matches.regprog;
        matched.then_some(matches)
    }

    /// [`vim_regexec_nl`] of this pattern over `line` from byte `col`: as
    /// [`OwnedProg::exec`], with a newline in `line` matched as one.
    pub(crate) fn exec_nl(
        &mut self,
        line: &CStr,
        col: usize,
        ignore_case: bool,
    ) -> Option<RegMatch> {
        if self.0.is_null() {
            return None;
        }
        let mut matches = RegMatch::new(self.0, ignore_case);
        let matched = vim_regexec_nl(&mut matches, line, col);
        self.0 = matches.regprog;
        matched.then_some(matches)
    }

    /// [`vim_regexec_multi`] of this pattern over `buffer` from line `lnum`,
    /// column `col`, with `rmp`'s other settings. Answers the number of
    /// lines the match spans plus one, or 0.
    pub(crate) fn exec_multi(
        &mut self,
        rmp: &mut RegMMatch,
        win: Option<Win>,
        buffer: Buf,
        lnum: LineNr,
        col: ColNr,
    ) -> c_int {
        if self.0.is_null() {
            return 0;
        }
        rmp.regprog = self.0;
        // SAFETY: a match record of the caller's own holding this live
        // program, and no time limit.
        let lines = unsafe {
            vim_regexec_multi(
                rmp,
                win,
                buffer,
                lnum,
                col,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        self.0 = rmp.regprog;
        lines
    }
}

impl Drop for OwnedProg {
    fn drop(&mut self) {
        // SAFETY: this holder's own program, or null.
        unsafe { vim_regfree(self.0) };
    }
}

/// A [`RegMatch`] that owns its program: compiled on the way in, freed when
/// it goes. For a caller that hands the match structure itself on, as the
/// completion generators take it.
pub(crate) struct OwnedMatch(RegMatch);

impl OwnedMatch {
    /// [`vim_regcomp`] into a match structure. `None` when the pattern does
    /// not compile.
    pub(crate) fn compile(expr: &CStr, re_flags: c_int, ignore_case: bool) -> Option<OwnedMatch> {
        let prog = vim_regcomp(expr, re_flags);
        (!prog.is_null()).then(|| OwnedMatch(RegMatch::new(prog, ignore_case)))
    }

    /// A match structure with no program, for a caller that matches some
    /// other way but has to hand one on.
    pub(crate) fn none() -> OwnedMatch {
        OwnedMatch(RegMatch::new(ptr::null_mut(), false))
    }
}

impl core::ops::Deref for OwnedMatch {
    type Target = RegMatch;

    fn deref(&self) -> &RegMatch {
        &self.0
    }
}

impl core::ops::DerefMut for OwnedMatch {
    fn deref_mut(&mut self) -> &mut RegMatch {
        &mut self.0
    }
}

impl Drop for OwnedMatch {
    fn drop(&mut self) {
        // SAFETY: this holder's program -- the one it compiled, or the one an
        // engine switch put in its place -- or null.
        unsafe { vim_regfree(self.0.regprog) };
    }
}

/// A [`RegMMatch`] that owns its program: the buffer-match counterpart of
/// [`OwnedMatch`], for a command that compiles once and runs the program
/// over line after line (`:substitute`, `:global`). Freed when dropped --
/// whatever program the last engine switch left in it.
///
/// Reads go through [`Deref`](core::ops::Deref); there is no `DerefMut`,
/// because a caller that replaced `regprog` would leak this one's program
/// or free another's.
pub(crate) struct OwnedMMatch(RegMMatch);

impl OwnedMMatch {
    /// Take over `prog`, matching with `ignore_case` (an `rmm_ic` value).
    ///
    /// # Safety
    /// `prog` must be a live program from [`vim_regcomp`] that nothing else
    /// will free.
    pub(crate) unsafe fn adopt(prog: *mut RegProg, ignore_case: c_int) -> OwnedMMatch {
        OwnedMMatch(RegMMatch {
            regprog: prog,
            rmm_ic: ignore_case,
            rmm_maxcol: 0,
            ..RegMMatch::default()
        })
    }

    /// Override the case rule the pattern was compiled with: `:s///i`.
    pub(crate) fn set_ignore_case(&mut self, ignore_case: bool) {
        self.0.rmm_ic = c_int::from(ignore_case);
    }

    /// Whether there is still a program -- a failed recompile for the
    /// backtracking engine can leave none.
    pub(crate) fn has_program(&self) -> bool {
        !self.0.regprog.is_null()
    }

    /// Can the pattern match a line break?
    pub(crate) fn is_multiline(&self) -> bool {
        // SAFETY: this holder's live program.
        self.has_program() && unsafe { re_multiline(self.0.regprog) }
    }

    /// [`vim_regexec_multi`] over `buffer` from line `lnum`, column `col`,
    /// with no time limit: how many lines the match spans (counting the
    /// one it starts in), or 0 for no match.
    pub(crate) fn exec(
        &mut self,
        win: Option<Win>,
        buffer: Buf,
        lnum: LineNr,
        col: ColNr,
    ) -> LineNr {
        if !self.has_program() {
            return 0;
        }
        // SAFETY: the match record is this holder's own, with its live
        // program; no time limit. An engine switch replaces the program in
        // the record, which is what `Drop` then frees.
        unsafe {
            vim_regexec_multi(
                &raw mut self.0,
                win,
                buffer,
                lnum,
                col,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        }
    }

    /// The match record, for the substitution's two passes.
    pub(super) fn record(&mut self) -> &mut RegMMatch {
        &mut self.0
    }
}

impl core::ops::Deref for OwnedMMatch {
    type Target = RegMMatch;

    fn deref(&self) -> &RegMMatch {
        &self.0
    }
}

impl Drop for OwnedMMatch {
    fn drop(&mut self) {
        // SAFETY: this holder's program, or null.
        unsafe { vim_regfree(self.0.regprog) };
    }
}
