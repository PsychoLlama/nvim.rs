//! The engine's entry points: compiling a pattern and running a program.
//!
//! Between them and the match loop sits `nfa_regexec_both`, which is where
//! the shortcuts live: an anchored pattern only tries column 0, a pattern
//! with a known first character skips to it, and a pattern that is one
//! literal run never runs the machine at all.

#![forbid(unsafe_code)]

use crate::regexp::ExtMatch;
use crate::regexp::ExtMatchRef;
use crate::regexp::RF_HAD_EOL;
use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::build::{Pass, nfa_postprocess, post2nfa};
use super::compile::{nfa_get_match_text, nfa_get_reganch, nfa_get_regstart};
use super::list::{BLANK_SUBS, SpareLists};
use super::matcher::nfa_regmatch;
use super::parse::re2post;
use super::run::{find_match_text, skip_to_start};
use super::sub::{clear_sub, slots};
use crate::message::e_null;
use crate::message::iemsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    MatchPos, NFA_TOO_EXPENSIVE, NfaRegProg, REX_SET, RF_ICASE, RF_ICOMBINE, RF_NOICASE, RegSubs,
    Rex, TimeBudget, cleanup_subexpr, cleanup_zsubexpr, reg_getline,
};
use crate::types::{ColNr, LPos, NUL};

/// Try to match at column `col` of the current line.
///
/// Returns 0 for no match, `NFA_TOO_EXPENSIVE`, or one more than the line
/// the match ended on.
fn nfa_regtry(rex: Rex, prog: &NfaRegProg, col: ColNr) -> c_int {
    let mut subs = BLANK_SUBS;
    let mut m = BLANK_SUBS;
    rex.set_col(col);
    rex.set_time_budget(TimeBudget::NONE);

    clear_sub(rex, &mut subs.norm);
    clear_sub(rex, &mut m.norm);
    clear_sub(rex, &mut subs.synt);
    clear_sub(rex, &mut m.synt);

    let mut listids = Vec::new();
    let result = nfa_regmatch(rex, prog, prog.start, &mut subs, &mut m, &mut listids);
    if result == 0 || result == NFA_TOO_EXPENSIVE {
        return result;
    }

    cleanup_subexpr(rex);
    report_match(rex, &subs, col);

    // The `\z(` captures go to the syntax highlighter as fresh copies,
    // because it keeps them past the end of this match.
    let captures = (prog.reghasz == REX_SET).then(|| {
        cleanup_zsubexpr(rex);
        ExtMatchRef::new(z_captures(rex, &subs))
    });
    rex.put_ext_out(captures);
    1 + rex.lnum() as c_int
}

/// Copy the capture positions into the match's slots.
fn report_match(rex: Rex, subs: &RegSubs, col: ColNr) {
    let (starts, ends) = (
        rex.starts().as_array_of_cells(),
        rex.ends().as_array_of_cells(),
    );
    for i in 0..slots(subs.norm.in_use) {
        starts[i].set(subs.norm.list[i].start.as_pos());
        ends[i].set(subs.norm.list[i].end.as_pos());
    }
    if rex.multi() {
        rex.set_matchcol(subs.norm.orig_start_col);
    }
    // A `\zs` before the start, or a `\ze` before the end, can leave group 0
    // unset; it then covers what the machine actually walked.
    let (start, end) = (rex.start(0), rex.end(0));
    let kind = rex.pos_kind();
    if !start.get().is_set(kind) {
        start.set(MatchPos::from_pos(LPos { lnum: 0, col }));
    }
    if !end.get().is_set(kind) {
        end.set(rex.here());
    } else if rex.multi() {
        rex.set_lnum(end.get().as_pos().lnum);
    }
}

/// Copy what the `\z(` groups matched into the set the highlighter reads.
fn z_captures(rex: Rex, subs: &RegSubs) -> ExtMatch {
    let mut captures = ExtMatch::default();
    // Group 0 is the whole match, which the highlighter does not want.
    for i in 1..slots(subs.synt.in_use) {
        let capture = subs.synt.list[i];
        let (start, end) = (capture.start.as_pos(), capture.end.as_pos());
        let line = if rex.multi() {
            // A capture that spans lines cannot be handed over as one
            // string, so it is dropped.
            if start.lnum < 0 || start.lnum != end.lnum || end.col < start.col {
                continue;
            }
            let Some(line) = reg_getline(rex, start.lnum) else {
                continue;
            };
            line
        } else {
            if start.lnum < 0 || end.lnum < 0 {
                continue;
            }
            rex.line()
        };
        let from = usize::try_from(start.col).unwrap_or(0).min(line.len() - 1);
        let text = &line[from..];
        let text = match usize::try_from(end.col - start.col) {
            Ok(len) => &text[..len.min(text.len())],
            Err(_) => text,
        };
        captures.set(i, text);
    }
    captures
}

/// Run the NFA program `prog` over the match `rex` is set up for, starting
/// at column `col`. Returns 0 for no match, `NFA_TOO_EXPENSIVE` when the
/// pattern has to go to the backtracking engine, or one more than the line
/// the match ended on.
pub(crate) fn nfa_regexec(rex: Rex, prog: &NfaRegProg, startcol: ColNr) -> c_int {
    let mut col = startcol;
    let line = if rex.multi() {
        reg_getline(rex, 0)
    } else {
        Some(rex.line())
    };

    let mut retval = 0;
    match line {
        Some(line) => {
            // The pattern's own `\c`/`\C`/`\Z` override what the caller
            // asked for.
            if prog.regflags & RF_ICASE as u32 != 0 {
                rex.set_reg_ic(true);
            } else if prog.regflags & RF_NOICASE as u32 != 0 {
                rex.set_reg_ic(false);
            }
            if prog.regflags & RF_ICOMBINE as u32 != 0 {
                rex.set_reg_icombine(true);
            }
            rex.set_line(line);
            rex.set_lnum(0);
            rex.set_nfa_has_zend(prog.has_zend);
            rex.set_nfa_has_backref(prog.has_backref);
            rex.set_nfa_nsubexpr(prog.nsubexp);
            rex.set_nfa_listid(1);
            rex.set_nfa_alt_listid(2);

            retval = match try_match(rex, prog, &mut col) {
                Attempt::Ran(retval) => retval,
                // The literal-text shortcut and the two "cannot match
                // here" answers report straight back, without the
                // start/end tidy-up below.
                Attempt::Done(retval) => return retval,
            };
        }
        None => iemsg(gettext(e_null)),
    }

    if retval > 0 {
        // A `\ze` can put the end before the start; report an empty
        // match rather than a backwards one.
        let kind = rex.pos_kind();
        let (start, end) = (rex.start(0).get(), rex.end(0));
        if end.get().is_before(start, kind) {
            end.set(start);
        }
        if !rex.multi() {
            rex.set_matchcol(col);
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
fn try_match(rex: Rex, prog: &NfaRegProg, col: &mut ColNr) -> Attempt {
    // An anchored pattern can only match at the start of the line.
    if prog.reganch && *col > 0 {
        return Attempt::Done(0);
    }
    rex.set_need_clear_subexpr(true);
    let has_z = prog.reghasz == REX_SET;
    rex.set_nfa_has_zsubexpr(has_z as c_int);
    rex.set_need_clear_zsubexpr(has_z);

    if prog.regstart != NUL {
        // The first character is known: there is no point trying any
        // column before the next one that holds it.
        if skip_to_start(rex, prog.regstart, col).is_err() {
            return Attempt::Done(0);
        }
        // And when the whole pattern is that literal run, the machine
        // has nothing to add. Not with 'regexpengine' combining
        // insensitivity, which the text comparison does not implement.
        if let Some(text) = prog.match_text.as_deref()
            && text[0] != 0
            && !rex.reg_icombine()
        {
            let retval = find_match_text(rex, col, prog.regstart, text);
            rex.set_matchcol(*col);
            return Attempt::Done(retval);
        }
    }
    if rex.reg_maxcol() > 0 && *col >= rex.reg_maxcol() {
        return Attempt::Ran(0);
    }

    // Every state starts out on no list.
    for s in &prog.states {
        s.lastlist[0].set(0);
        s.lastlist[1].set(0);
    }
    Attempt::Ran(nfa_regtry(rex, prog, *col))
}

/// Compile `rc`'s pattern into a program, or answer `None` after reporting
/// why not -- or, under 'regexpengine' 0, having declined it quietly.
pub(crate) fn nfa_regcomp(rc: &mut RegCompiler) -> Option<NfaRegProg> {
    rc.nfa.count = 0;
    rc.nfa.states = Vec::new();
    rc.post.start(rc.pattern_len);
    rc.wants_nfa = false;
    rc.has_zend = 0;
    rc.has_backref = 0;
    rc.restart();

    if re2post(rc).is_err() {
        return None;
    }
    // The first pass counts the states, which sizes the thread lists.
    post2nfa(&mut rc.nfa, rc.post.items(), Pass::Count);
    rc.nfa.states = Vec::with_capacity(usize::try_from(rc.nfa.count).unwrap_or(0));
    let start = post2nfa(&mut rc.nfa, rc.post.items(), Pass::Build);
    if start.is_none() {
        return None;
    }
    let mut flags = rc.flags;
    if rc.had_eol != 0 {
        flags |= RF_HAD_EOL;
    }
    let mut prog = NfaRegProg {
        regflags: flags,
        automatic: false,
        start,
        reganch: false,
        regstart: 0,
        match_text: None,
        has_zend: rc.has_zend,
        has_backref: rc.has_backref,
        reghasz: rc.has_z,
        pattern: rc.pattern.to_owned(),
        nsubexp: rc.next_group,
        nstate: rc.nfa.count,
        states: core::mem::take(&mut rc.nfa.states),
        spare: SpareLists::new(),
    };
    nfa_postprocess(&mut prog);
    prog.reganch = nfa_get_reganch(&prog.states, start, 0);
    prog.regstart = nfa_get_regstart(&prog.states, start, 0);
    prog.match_text = nfa_get_match_text(&prog.states, start);
    Some(prog)
}
