//! The public entry points: compiling a pattern and running it over a
//! string or a buffer.
//!
//! Two things happen here that the engines below do not see. One is
//! engine selection: `'regexpengine'` and a leading `\%#=` pick one, and
//! when the NFA engine gives up on a pattern (`NFA_TOO_EXPENSIVE`) the
//! pattern is recompiled for the backtracking engine and rerun. The other
//! is the match's context: each run builds its own [`RegExec`] on this
//! layer's stack — so a `\=` expression that starts a match of its own gets
//! a context of its own — and reads back what the engine left in it.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::global_cell::GlobalCell;
use crate::regexp::RegCompiler;
use crate::regexp::bt_regcomp;
use crate::regexp::nfa_regcomp;
use crate::regexp::{ExtMatch, ExtMatchIo, ExtMatchRef};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::cell::Cell;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::CString;

use super::{
    AUTOMATIC_ENGINE, BACKTRACKING_ENGINE, BtRegProg, GroupSlots, MatchSetup, NFA_ENGINE,
    NFA_TOO_EXPENSIVE, NfaRegProg, RE_NOBREAK, REGMAGIC, REX_ALL, RF_HAD_EOL, RF_HASNL, RegExec,
    Rex, bt_regexec, nfa_regexec,
};
use crate::message::state::called_emsg;
use crate::message::{emsg, msg_str, verbose_enter, verbose_leave};
use crate::option::vars::{P_RE, p_re, p_verbose};
use crate::os::cshim::gettext;
use crate::regexp::{RE_AUTO, REX_SET};
use crate::types::{ColNr, LPos, LineNr, OptInt, ProfTime, RE_GROUPS, RegMMatch, RegMatch};

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
#[inline(always)]
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

/// One engine's program.
#[derive(Clone)]
pub(crate) enum Program {
    Bt(BtRegProg),
    Nfa(NfaRegProg),
}

/// A compiled pattern: the engine's program, and what [`vim_regcomp`]
/// recorded about how it was asked for.
#[derive(Clone)]
struct Compiled {
    /// The engine 'regexpengine' or a `\%#=` named: automatic, backtracking
    /// or NFA. An automatic one may hand the pattern over mid-match.
    engine: c_uint,
    /// The `RE_*` flags the pattern was compiled with, which a recompile for
    /// the other engine repeats.
    re_flags: c_int,
    program: Program,
}

/// A compiled pattern, owned: dropped when its holder lets go, and replaced
/// in place when the NFA engine hands a pattern to the backtracking one.
///
/// Upstream shares a `regprog_T *` between whoever compiled it and whoever
/// matches with it, and a match may free it and put a recompiled program in
/// the caller's `regmatch_T`. Here the program is a box with one owner: a
/// match borrows it `&mut` from wherever it lives — a [`RegMatch`], a
/// [`RegMMatch`], a field of its own — and a recompile replaces it there,
/// so the owner gets the new program back through the same place.
///
/// `Clone` is a deep copy, for the one holder that needs a private program
/// per redraw (a window's match list).
#[derive(Clone)]
pub struct OwnedProg(Box<Compiled>);

impl OwnedProg {
    /// [`vim_regcomp`]: `None` when the pattern does not compile.
    pub fn compile(expr: &CStr, re_flags: c_int) -> Option<OwnedProg> {
        vim_regcomp(expr, re_flags)
    }

    /// The `RF_*` flags the compiler found.
    fn regflags(&self) -> c_uint {
        match &self.0.program {
            Program::Bt(bt) => bt.regflags,
            Program::Nfa(nfa) => nfa.regflags,
        }
    }

    /// Can the pattern match across a line break?
    pub fn is_multiline(&self) -> bool {
        self.regflags() & RF_HASNL as c_uint != 0
    }

    /// Did the pattern end with a `\n` that a search should treat as
    /// "match at end of line"? Read by the syntax code right after a compile.
    pub(crate) fn had_eol(&self) -> bool {
        self.regflags() & RF_HAD_EOL != 0
    }

    /// Does the program belong to the backtracking engine, and has its magic
    /// number not survived? Only that engine's programs carry one. Reports so
    /// when it has not.
    pub(crate) fn magic_wrong(&self) -> bool {
        let wrong = matches!(&self.0.program, Program::Bt(bt) if bt.program[0] != REGMAGIC as u8);
        if wrong {
            emsg(gettext(crate::message::e_re_corr));
        }
        wrong
    }

    /// Was it compiled with `RE_NOBREAK`?
    fn nobreak(&self) -> bool {
        self.0.re_flags & RE_NOBREAK != 0
    }

    /// May the NFA engine give the pattern up for the backtracking one?
    fn automatic(&self) -> bool {
        self.0.engine == AUTOMATIC_ENGINE
    }

    /// Does the program have `\z(` groups, and so need slots for them?
    #[inline(always)]
    fn has_z(&self) -> bool {
        match &self.0.program {
            Program::Bt(bt) => c_int::from(bt.reghasz) == REX_SET,
            Program::Nfa(nfa) => nfa.reghasz == REX_SET,
        }
    }

    /// Run the program over the match `rex` is set up for, from column
    /// `col`: the engine the program came from runs it.
    #[inline(always)]
    fn run(&self, rex: Rex, col: ColNr) -> c_int {
        match &self.0.program {
            Program::Bt(bt) => bt_regexec(rex, bt, col),
            Program::Nfa(nfa) => nfa_regexec(rex, nfa, col),
        }
    }

    /// [`vim_regexec`] of this pattern over `line` from byte `col`: the
    /// match (holding no program), or `None`.
    pub(crate) fn exec(&mut self, line: &CStr, col: usize, ignore_case: bool) -> Option<RegMatch> {
        let mut matches = RegMatch::new(None, ignore_case);
        regexec_string(self, &mut matches, line, col, false).then_some(matches)
    }

    /// [`vim_regexec_nl`] of this pattern over `line` from byte `col`: as
    /// [`OwnedProg::exec`], with a newline in `line` matched as one.
    pub(crate) fn exec_nl(
        &mut self,
        line: &CStr,
        col: usize,
        ignore_case: bool,
    ) -> Option<RegMatch> {
        let mut matches = RegMatch::new(None, ignore_case);
        regexec_string(self, &mut matches, line, col, true).then_some(matches)
    }

    /// [`vim_regexec_multi`] of this pattern over `buffer` from line `lnum`,
    /// column `col`, with `rmp`'s other settings (its own program, if it has
    /// one, is not the one run). Answers the number of lines the match spans
    /// plus one, or 0.
    pub(crate) fn exec_multi(
        &mut self,
        rmp: &mut RegMMatch,
        win: Option<Win>,
        buffer: Buf,
        lnum: LineNr,
        col: ColNr,
    ) -> c_int {
        let run = MultiRun {
            buffer: &buffer,
            win,
            lnum,
            col,
            deadline: None,
            ext_in: None,
        };
        run.exec(self, rmp, None, &mut None)
    }
}

/// Compile `expr` into a program. A leading `\%#=0`, `\%#=1` or `\%#=2`
/// overrides 'regexpengine' for this pattern; with the automatic setting
/// the NFA engine is tried first and a failure that reported no error
/// falls back to the backtracking one.
///
/// Answers `None` when the pattern does not parse, having reported why.
pub fn vim_regcomp(expr_arg: &CStr, re_flags: c_int) -> Option<OwnedProg> {
    vim_regcomp_ext(expr_arg, re_flags, 0)
}

/// [`vim_regcomp`] for a syntax pattern: `extmatch` says which `\z`
/// specials it may use -- `REX_SET` for a region's start pattern, which
/// defines `\z(` groups, `REX_USE` for its skip and end patterns, which
/// match them with `\z1`..`\z9`.
pub(crate) fn vim_regcomp_ext(
    expr_arg: &CStr,
    re_flags: c_int,
    extmatch: c_int,
) -> Option<OwnedProg> {
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
    let mut program = if engine != BACKTRACKING_ENGINE as c_int {
        let auto = if engine == AUTOMATIC_ENGINE as c_int {
            RE_AUTO
        } else {
            0
        };
        nfa_regcomp(&mut RegCompiler::new(expr, re_flags + auto, buf, extmatch)).map(|mut nfa| {
            nfa.automatic = engine == AUTOMATIC_ENGINE as c_int;
            Program::Nfa(nfa)
        })
    } else {
        bt_regcomp(&mut RegCompiler::new(expr, re_flags, buf, extmatch)).map(Program::Bt)
    };
    // Only retry when the NFA engine declined quietly: an error means the
    // pattern is bad, not merely too much for that engine.
    if program.is_none()
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
        program = bt_regcomp(&mut RegCompiler::new(expr, re_flags, buf, extmatch)).map(Program::Bt);
    }
    let engine = c_uint::try_from(engine).unwrap_or(AUTOMATIC_ENGINE);
    program.map(|program| {
        OwnedProg(Box::new(Compiled {
            engine,
            re_flags,
            program,
        }))
    })
}

/// Recompile the NFA program `prog` for the backtracking engine, which is
/// what a `NFA_TOO_EXPENSIVE` result asks for.
fn recompile_backtracking(prog: &OwnedProg, extmatch: bool) -> Option<OwnedProg> {
    let Program::Nfa(nfa) = &prog.0.program else {
        return None;
    };
    let pat: CString = nfa.pattern.clone();
    let save_p_re = p_re();
    P_RE.set(BACKTRACKING_ENGINE as c_int as OptInt);
    if p_verbose() > 0 as OptInt {
        verbose_enter();
        msg_str(gettext(
            c"Switching to backtracking RE engine for pattern: ",
        ));
        msg_str(&pat);
        verbose_leave();
    }
    // A buffer match may be a syntax match, whose `\z(` groups have to
    // survive the recompile.
    let new = vim_regcomp_ext(&pat, prog.0.re_flags, if extmatch { REX_ALL } else { 0 });
    P_RE.set(save_p_re);
    new
}

/// Run `prog` over the whole of `line`, starting at byte `col`, leaving the
/// groups in `matches` (whose own program is not the one run). `nl` allows
/// a `$` to match at the end of the string.
///
/// The line is a `&CStr` and not a `&[u8]` on purpose. Both engines treat
/// the terminator as a *position* rather than a bound — `$` matches there,
/// `\n` is tested for there, and every character step reads it before
/// deciding it has run out — so a length alone would not tell them where
/// the line ends. Taking the terminated string says that, and hands the
/// callers a type they can hold.
fn regexec_string(
    prog: &mut OwnedProg,
    matches: &mut RegMatch,
    line: &CStr,
    col: usize,
    nl: bool,
) -> bool {
    debug_assert!(col <= line.count_bytes(), "col past the end of the line");
    let col = ColNr::try_from(col).unwrap_or(ColNr::MAX);
    let text = line.to_bytes_with_nul();
    // A string match has no buffer of its own, but `\k` and friends still
    // need an 'iskeyword' to read.
    let buf = Buf::current();
    let run = StringRun {
        buffer: &buf,
        text,
        col,
        nl,
    };
    let result = in_match_scope(None, |firstlnum| {
        let result = run.once(prog, matches, firstlnum);
        if result == NFA_TOO_EXPENSIVE && prog.automatic() {
            run.again_backtracking(prog, matches, firstlnum)
        } else {
            result
        }
    });
    result > 0
}

/// What one string match runs with, whichever program runs it.
struct StringRun<'a> {
    /// The current buffer, whose 'iskeyword' `\k` and friends read.
    buffer: &'a Buf,
    /// The string, its NUL included.
    text: &'a [u8],
    col: ColNr,
    nl: bool,
}

impl StringRun<'_> {
    /// Run `prog` once, leaving its groups in `matches` (as offsets into the
    /// string) on a hit and clearing them on a miss.
    #[inline(always)]
    fn once(&self, prog: &OwnedProg, matches: &mut RegMatch, firstlnum: LineNr) -> LineNr {
        let zslots;
        let setup = MatchSetup {
            buf: self.buffer,
            win: None,
            ic: matches.rm_ic,
            nobreak: prog.nobreak(),
            deadline: None,
            ext_in: None,
            ext_out: None,
            zslots: if prog.has_z() {
                zslots = GroupSlots::new();
                Some(&zslots)
            } else {
                None
            },
        };
        let slots = GroupSlots::new();
        let ctx = RegExec::for_string(setup, self.text, self.nl, firstlnum, &slots);
        let result = prog.run(ctx.rex(), self.col);
        if let Some(col) = ctx.matchcol() {
            matches.rm_matchcol = col;
        }
        if result > 0 {
            record_groups(matches, &slots.starts.get(), &slots.ends.get());
        } else {
            matches.clear_groups();
        }
        result
    }

    /// The NFA engine gave up on the pattern: recompile it for the
    /// backtracking one, in place, and run that. A failed recompile keeps
    /// the old program and reports no match.
    #[cold]
    #[inline(never)]
    fn again_backtracking(
        &self,
        prog: &mut OwnedProg,
        matches: &mut RegMatch,
        firstlnum: LineNr,
    ) -> LineNr {
        let Some(new) = recompile_backtracking(prog, false) else {
            return NFA_TOO_EXPENSIVE;
        };
        *prog = new;
        self.once(prog, matches, firstlnum)
    }
}

/// Copy the slots the engine filled into `matches`, as offsets into the
/// string the match ran over — which for a string match holds every
/// position: nothing in either engine moves a string match onto another
/// line.
fn record_groups(matches: &mut RegMatch, starts: &[LPos; RE_GROUPS], ends: &[LPos; RE_GROUPS]) {
    let offset = |pos: &LPos| (pos.lnum >= 0).then(|| usize::try_from(pos.col).unwrap_or(0));
    for no in 0..RE_GROUPS {
        matches.starts[no] = offset(&starts[no]);
        matches.ends[no] = offset(&ends[no]);
    }
}

/// Run `matches`'s program over `line`, starting at byte `col`.
///
/// On a hit `matches`'s groups are byte offsets into `line`; on a miss they
/// are all unset, and so they are when `matches` holds no program. This
/// re-enters the editor — a `\=` expression can start a match of its own —
/// so nothing may be held across it.
pub fn vim_regexec(matches: &mut RegMatch, line: &CStr, col: usize) -> bool {
    regexec_holder(matches, line, col, false)
}

/// [`vim_regexec`] with `$` allowed to match at the end of the string.
pub fn vim_regexec_nl(matches: &mut RegMatch, line: &CStr, col: usize) -> bool {
    regexec_holder(matches, line, col, true)
}

/// [`regexec_string`] of the program `matches` holds, which is taken out for
/// the match and put back -- recompiled, it may be.
fn regexec_holder(matches: &mut RegMatch, line: &CStr, col: usize, nl: bool) -> bool {
    let Some(mut prog) = matches.regprog.take() else {
        matches.clear_groups();
        return false;
    };
    let matched = regexec_string(&mut prog, matches, line, col, nl);
    matches.regprog = Some(prog);
    matched
}

/// [`vim_regexec`] against a program the caller keeps in a slot of its own,
/// which the fall back to the backtracking engine replaces in place. No
/// program, no match.
pub fn vim_regexec_prog(
    prog: &mut Option<OwnedProg>,
    ignore_case: bool,
    line: &CStr,
    col: usize,
) -> bool {
    let Some(prog) = prog else {
        return false;
    };
    let mut matches = RegMatch::new(None, ignore_case);
    regexec_string(prog, &mut matches, line, col, false)
}

/// Run `rmp`'s program over `buffer` starting at line `lnum`, column `col`.
/// Returns the number of lines the match spans plus one, or 0 for no match
/// (and for no program). `tm` bounds how long the NFA engine may spend; a
/// match that runs out of it sets `timed_out`.
// Kept out of line: inlined into `searchit` and `next_search_hl` (as the
// optimizer chose on its own once unrelated code moved), the body below
// leaves its closure as a call of its own and costs those loops ~0.8 % of
// the memline benchmark's instructions and ~1.3 % of its time.
#[inline(never)]
pub fn vim_regexec_multi(
    rmp: &mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: Option<&ProfTime>,
    timed_out: Option<&mut bool>,
) -> c_int {
    let io = ExtMatchIo {
        input: None,
        output: None,
    };
    regexec_multi_io(rmp, win, buffer, lnum, col, tm, timed_out, io)
}

#[allow(
    clippy::too_many_arguments,
    reason = "vim_regexec_multi's, and the `\\z` traffic"
)]
/// [`vim_regexec_multi`] with a syntax match's `\z` traffic.
fn regexec_multi_io(
    rmp: &mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: Option<&ProfTime>,
    timed_out: Option<&mut bool>,
    io: ExtMatchIo<'_>,
) -> c_int {
    let Some(mut prog) = rmp.regprog.take() else {
        return 0;
    };
    let run = MultiRun {
        buffer: &buffer,
        win,
        lnum,
        col,
        deadline: tm.copied(),
        ext_in: io.input,
    };
    let mut ext_out = io.output;
    let result = run.exec(&mut prog, rmp, timed_out, &mut ext_out);
    rmp.regprog = Some(prog);
    result
}

/// What one buffer match runs with, whichever program runs it.
struct MultiRun<'a> {
    buffer: &'a Buf,
    win: Option<Win>,
    lnum: LineNr,
    col: ColNr,
    deadline: Option<ProfTime>,
    ext_in: Option<&'a ExtMatch>,
}

impl MultiRun<'_> {
    /// Run `prog` with `rmm`'s settings, into `rmm`'s slots, falling back to
    /// the backtracking engine when the NFA one gives up.
    fn exec(
        &self,
        prog: &mut OwnedProg,
        rmm: &mut RegMMatch,
        mut timed_out: Option<&mut bool>,
        ext_out: &mut Option<&mut Option<ExtMatchRef>>,
    ) -> c_int {
        let result = in_match_scope(Some(self.lnum), |_| {
            let result = self.once(prog, rmm, &mut timed_out, ext_out);
            if result == NFA_TOO_EXPENSIVE && prog.automatic() {
                self.again_backtracking(prog, rmm, &mut timed_out, ext_out)
            } else {
                result
            }
        });
        result.max(0)
    }

    /// Run `prog` once.
    #[inline(always)]
    fn once(
        &self,
        prog: &OwnedProg,
        rmm: &mut RegMMatch,
        timed_out: &mut Option<&mut bool>,
        ext_out: &mut Option<&mut Option<ExtMatchRef>>,
    ) -> LineNr {
        let zslots;
        let setup = MatchSetup {
            buf: self.buffer,
            win: self.win,
            ic: rmm.rmm_ic != 0,
            nobreak: prog.nobreak(),
            deadline: self.deadline,
            ext_in: self.ext_in,
            ext_out: ext_out.as_deref_mut().map(Cell::from_mut),
            zslots: if prog.has_z() {
                zslots = GroupSlots::new();
                Some(&zslots)
            } else {
                None
            },
        };
        // The match writes the caller's slots in place, as upstream's did.
        let maxcol = rmm.rmm_maxcol;
        let (starts, ends) = (
            Cell::from_mut(&mut rmm.startpos),
            Cell::from_mut(&mut rmm.endpos),
        );
        let ctx = RegExec::for_buffer(setup, self.lnum, maxcol, starts, ends);
        let result = prog.run(ctx.rex(), self.col);
        let (matchcol, did_time_out) = (ctx.matchcol(), ctx.timed_out());
        if let Some(col) = matchcol {
            rmm.rmm_matchcol = col;
        }
        if did_time_out && let Some(flag) = timed_out.as_deref_mut() {
            *flag = true;
        }
        result
    }

    /// The NFA engine gave up on the pattern: recompile it for the
    /// backtracking one, in place, and run that. A failed recompile keeps
    /// the old program rather than leaving the caller without one.
    #[cold]
    #[inline(never)]
    fn again_backtracking(
        &self,
        prog: &mut OwnedProg,
        rmm: &mut RegMMatch,
        timed_out: &mut Option<&mut bool>,
        ext_out: &mut Option<&mut Option<ExtMatchRef>>,
    ) -> LineNr {
        let Some(new) = recompile_backtracking(prog, true) else {
            return NFA_TOO_EXPENSIVE;
        };
        *prog = new;
        self.once(prog, rmm, timed_out, ext_out)
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "vim_regexec_multi's, and the `\\z` traffic"
)]
/// [`vim_regexec_multi`] for a syntax pattern: `io.input` is what `\z1`..
/// `\z9` match, and a successful match leaves its own `\z(` captures in
/// `io.output` (or `None` when it has none).
pub(crate) fn vim_regexec_syntax(
    rmp: &mut RegMMatch,
    win: Option<Win>,
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    tm: Option<&ProfTime>,
    timed_out: Option<&mut bool>,
    io: ExtMatchIo<'_>,
) -> LineNr {
    regexec_multi_io(rmp, win, buffer, lnum, col, tm, timed_out, io)
}

impl RegMatch {
    /// [`vim_regcomp`] into a match structure. `None` when the pattern does
    /// not compile.
    pub(crate) fn compile(expr: &CStr, re_flags: c_int, ignore_case: bool) -> Option<RegMatch> {
        vim_regcomp(expr, re_flags).map(|prog| RegMatch::new(Some(prog), ignore_case))
    }
}

impl RegMMatch {
    /// Override the case rule the pattern was compiled with: `:s///i`.
    pub(crate) fn set_ignore_case(&mut self, ignore_case: bool) {
        self.rmm_ic = c_int::from(ignore_case);
    }

    /// Whether there is a program.
    pub(crate) fn has_program(&self) -> bool {
        self.regprog.is_some()
    }

    /// Can the pattern match a line break?
    pub(crate) fn is_multiline(&self) -> bool {
        self.regprog.as_ref().is_some_and(OwnedProg::is_multiline)
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
        vim_regexec_multi(self, win, buffer, lnum, col, None, None)
    }
}
