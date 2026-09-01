//! What the match loop calls out to: the character classes, the back
//! references, the recursive sub-match a lookaround runs, and the shortcuts
//! that let the loop skip work.

#![deny(unsafe_op_in_unsafe_fn)]

use super::list::{op, out_of, out1_of};
use crate::cstr;
use crate::guard::Depth;
use crate::regexp::NfaOp;
use crate::siemsg;
use core::ffi::{c_char, c_int, c_ushort};

use super::matcher::nfa_regmatch;
use crate::ascii::{ascii_isdigit, ascii_isxdigit};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::main::re_extmatch_in;
use crate::mbyte::{
    mb_islower, mb_isupper, utf_char2len, utf_fold, utf_head_off, utf_iscomposing_legacy,
    utf_ptr2char, utf_ptr2len,
};
use crate::os::cshim::__ctype_b_loc;
use crate::profile::profile_passed_limit;
use crate::regexp::{
    _ISalnum, _ISalpha, _IScntrl, _ISgraph, _ISpunct, ESC, MatchPos, NFA_TOO_EXPENSIVE, RA_MATCH,
    Rex, cleanup_subexpr, cleanup_zsubexpr, cstrchr, cstrncmp, match_with_backref, nfa_endp,
    nfa_ll_index, nfa_match, nfa_pim_T, nfa_regprog_T, nfa_state_T, nfa_time_limit, nfa_timed_out,
    reg_getline, reg_getline_len, reg_iswordc, regsub_T, regsubs_T,
};
use crate::types::{Failed, colnr_T, uint8_t};

/// Is `c` a member of the `[:name:]` class `cls` stands for?
///
/// The ranges are upstream's and are not uniform: some classes only ever
/// answer for ASCII, others run the whole Latin-1 range or defer to a
/// multibyte predicate.
pub(crate) fn check_char_class(rex: Rex, cls: c_int, c: c_int) -> Result<(), Failed> {
    // SAFETY: the ctype table is indexed only inside the guards below.
    let ctype = |mask: c_int| unsafe {
        *(*__ctype_b_loc()).offset(c as isize) as c_int & mask as c_ushort as c_int != 0
    };
    let member = match NfaOp::try_from(cls) {
        Ok(NfaOp::ClassAlnum) => (1..128).contains(&c) && ctype(_ISalnum as c_int),
        Ok(NfaOp::ClassAlpha) => (1..128).contains(&c) && ctype(_ISalpha as c_int),
        Ok(NfaOp::ClassBlank) => c == b' ' as c_int || c == b'\t' as c_int,
        Ok(NfaOp::ClassCntrl) => (1..=127).contains(&c) && ctype(_IScntrl as c_int),
        Ok(NfaOp::ClassDigit) => ascii_isdigit(c),
        Ok(NfaOp::ClassGraph) => (1..=127).contains(&c) && ctype(_ISgraph as c_int),
        // U+00AA and U+00BA are the ordinal indicators: lowercase
        // letters, but not the lower half of a case pair.
        Ok(NfaOp::ClassLower) => mb_islower(c) && c != 170 && c != 186,
        Ok(NfaOp::ClassPrint) => unsafe { vim_isprintc(c) },
        Ok(NfaOp::ClassPunct) => (1..128).contains(&c) && ctype(_ISpunct as c_int),
        Ok(NfaOp::ClassSpace) => (9..=13).contains(&c) || c == b' ' as c_int,
        Ok(NfaOp::ClassUpper) => mb_isupper(c),
        Ok(NfaOp::ClassXdigit) => ascii_isxdigit(c),
        Ok(NfaOp::ClassTab) => c == b'\t' as c_int,
        Ok(NfaOp::ClassReturn) => c == b'\r' as c_int,
        Ok(NfaOp::ClassBackspace) => c == 0x08,
        Ok(NfaOp::ClassEscape) => c == ESC,
        Ok(NfaOp::ClassIdent) => unsafe { vim_is_ident_char(c) },
        Ok(NfaOp::ClassKeyword) => reg_iswordc(rex, c),
        Ok(NfaOp::ClassFname) => unsafe { vim_isfilec(c) },
        _ => {
            siemsg!("E877: (NFA regexp) Invalid character class: {}", cls as i64);
            return Err(Failed);
        }
    };
    if member { Ok(()) } else { Err(Failed) }
}

/// Match what capture group `subidx` captured. On success `*bytelen` is how
/// far to advance — on the *last* line, for a capture that spans lines.
pub(crate) fn match_backref(rex: Rex, sub: &regsub_T, subidx: c_int, bytelen: &mut c_int) -> bool {
    // An unset group matches the empty string rather than failing.
    if sub.in_use <= subidx {
        *bytelen = 0;
        return true;
    }
    // SAFETY: the capture slots belong to this match, and `rex.input` points
    // into the line being matched.
    let capture = sub.list[subidx as usize];
    if rex.multi() {
        let (start, end) = (capture.start.as_pos(), capture.end.as_pos());
        if start.lnum < 0 || end.lnum < 0 {
            *bytelen = 0;
            return true;
        }
        if start.lnum == rex.lnum() && end.lnum == rex.lnum() {
            // Wholly on this line: a plain comparison.
            let mut len = end.col - start.col;
            let captured = unsafe { (rex.line() as *mut c_char).offset(start.col as isize) };
            if unsafe { cstrncmp(rex, captured, rex.input_str(), &mut len) } == 0 {
                *bytelen = len;
                return true;
            }
        } else if match_with_backref(rex, start.lnum, start.col, end.lnum, end.col, Some(bytelen))
            == RA_MATCH
        {
            return true;
        }
    } else {
        let (start, end) = (capture.start.as_ptr(), capture.end.as_ptr());
        if start.is_null() || end.is_null() {
            *bytelen = 0;
            return true;
        }
        let mut len = unsafe { end.offset_from(start) } as c_int;
        if unsafe { cstrncmp(rex, start as *mut c_char, rex.input_str(), &mut len) } == 0 {
            *bytelen = len;
            return true;
        }
    }
    false
}

/// Match what the enclosing syntax item's `\z(` group captured.
pub(crate) fn match_zref(rex: Rex, subidx: c_int, bytelen: &mut c_int) -> bool {
    cleanup_zsubexpr(rex);
    // SAFETY: `re_extmatch_in` is the capture set the syntax item handed in,
    // whose members are NUL-terminated copies.
    let captures = re_extmatch_in.get();
    if captures.is_null() || unsafe { (*captures).matches[subidx as usize].is_null() } {
        *bytelen = 0;
        return true;
    }
    let captured = unsafe { (*captures).matches[subidx as usize] } as *mut c_char;
    let mut len = unsafe { cstr::bytes_at(captured) }.len() as c_int;
    if unsafe { cstrncmp(rex, captured, rex.input_str(), &mut len) } == 0 {
        *bytelen = len;
        return true;
    }
    false
}

/// Set every state's second-generation list id aside, and clear it.
///
/// A lookaround runs a whole match of its own over the same program, so it
/// needs the generation counters to itself; these two put them back.
fn nfa_save_listids(prog: *mut nfa_regprog_T, list: &mut [c_int]) {
    // SAFETY: `prog` is the running program and `list` is `prog.nstate` long.
    let states = unsafe { &raw mut (*prog).state } as *mut nfa_state_T;
    for (i, slot) in list.iter_mut().enumerate() {
        let s = unsafe { states.add(i) };
        *slot = unsafe { (*s).lastlist[1] };
        unsafe { (*s).lastlist[1] = 0 };
    }
}

fn nfa_restore_listids(prog: *mut nfa_regprog_T, list: &[c_int]) {
    // SAFETY: as `nfa_save_listids`.
    let states = unsafe { &raw mut (*prog).state } as *mut nfa_state_T;
    for (i, &saved) in list.iter().enumerate() {
        unsafe { (*states.add(i)).lastlist[1] = saved };
    }
}

/// The `\%23l`-family comparison: `op` 1 is `>`, 2 is `<`, anything else is
/// equality.
pub(crate) fn nfa_re_num_cmp(val: u64, op: c_int, pos: u64) -> bool {
    match op {
        1 => pos > val,
        2 => pos < val,
        _ => val == pos,
    }
}

/// Run the lookaround at `state` as a match of its own.
///
/// `pim` is set when the lookaround was postponed, in which case the match
/// runs from where it was postponed rather than from the current position.
/// `listids` is the caller's scratch buffer for the generation counters.
pub(crate) fn recursive_regmatch(
    rex: Rex,
    state: *mut nfa_state_T,
    pim: *mut nfa_pim_T,
    prog: *mut nfa_regprog_T,
    submatch: *mut regsubs_T,
    m: *mut regsubs_T,
    listids: &mut Vec<c_int>,
) -> c_int {
    // SAFETY: everything below reads and restores the match context, which
    // is live for the duration of the match.
    let save_reginput_col = unsafe { rex.input().offset_from(rex.line()) } as c_int;
    let save_reglnum = rex.lnum();
    let save_nfa_match = nfa_match.get();
    let save_nfa_listid = rex.nfa_listid();
    let save_nfa_endp = nfa_endp.get();

    // Where the lookaround runs from: where the thread stood when it was
    // postponed, or — with no pim — where the outer match stands now.
    let from = if pim.is_null() {
        rex.here()
    } else {
        unsafe { (*pim).end }
    };
    if !pim.is_null() {
        rex.seek_col_of(from);
    }

    // A lookbehind has to start earlier in the line and stop where the
    // outer match stands; `endpos` is that stopping point.
    let mut endpos = MatchPos::NOWHERE;
    let mut endposp = core::ptr::null_mut::<MatchPos>();
    if matches!(
        NfaOp::try_from(op(state)),
        Ok(NfaOp::StartInvisibleBefore
            | NfaOp::StartInvisibleBeforeFirst
            | NfaOp::StartInvisibleBeforeNeg
            | NfaOp::StartInvisibleBeforeNegFirst)
    ) {
        endpos = from;
        endposp = &raw mut endpos;
        unsafe { start_lookbehind(rex, state) };
    }

    // Two generations of list ids are available. The first nested match
    // takes the second; a deeper one has to save and restore instead.
    let need_restore = nfa_ll_index.get() == 1;
    let generation = if need_restore {
        listids.clear();
        listids.resize(unsafe { (*prog).nstate } as usize, 0);
        nfa_save_listids(prog, listids);
        None
    } else {
        let held = Depth::of(&nfa_ll_index);
        if rex.nfa_listid() <= rex.nfa_alt_listid() {
            rex.set_nfa_listid(rex.nfa_alt_listid());
        }
        Some(held)
    };

    nfa_endp.set(endposp);
    let result = nfa_regmatch(rex, prog, out_of(state), submatch, m);
    if need_restore {
        nfa_restore_listids(prog, listids);
    } else {
        drop(generation);
        rex.set_nfa_alt_listid(rex.nfa_listid());
    }

    rex.set_lnum(save_reglnum);
    if rex.multi() {
        rex.set_line(reg_getline(rex, rex.lnum()) as *mut uint8_t);
    }
    rex.set_input(unsafe { rex.line().offset(save_reginput_col as isize) });
    // A match that ran out of budget keeps its verdict; anything else
    // hands the outer match its own state back.
    if result != NFA_TOO_EXPENSIVE {
        nfa_match.set(save_nfa_match);
        rex.set_nfa_listid(save_nfa_listid);
    }
    nfa_endp.set(save_nfa_endp);
    result
}

/// Move the input back to where a lookbehind has to start trying: `val`
/// bytes before the current position, or the start of the line when the
/// width is unknown.
///
/// # Safety
///
/// The match context must be live, and `state` one of the lookbehind
/// opcodes.
unsafe fn start_lookbehind(rex: Rex, state: *mut nfa_state_T) {
    let val = unsafe { (*state).val };
    if val <= 0 {
        // Unknown width: try from the start of the previous line.
        if rex.multi() {
            rex.set_lnum(rex.lnum() - 1);
            rex.set_line(reg_getline(rex, rex.lnum()) as *mut uint8_t);
            if rex.line().is_null() {
                // Already on the first line.
                rex.set_lnum(rex.lnum() + 1);
                rex.set_line(reg_getline(rex, rex.lnum()) as *mut uint8_t);
            }
        }
        rex.set_input(rex.line());
        return;
    }
    // The width reaches back past the start of this line.
    if rex.multi() && (unsafe { rex.input().offset_from(rex.line()) } as c_int) < val {
        rex.set_lnum(rex.lnum() - 1);
        rex.set_line(reg_getline(rex, rex.lnum()) as *mut uint8_t);
        if rex.line().is_null() {
            rex.set_lnum(rex.lnum() + 1);
            rex.set_line(reg_getline(rex, rex.lnum()) as *mut uint8_t);
            rex.set_input(rex.line());
        } else {
            rex.set_input(unsafe { rex.line().offset(reg_getline_len(rex, rex.lnum()) as isize) });
        }
    }
    if unsafe { rex.input().offset_from(rex.line()) } as c_int >= val {
        rex.set_input(unsafe { rex.input().offset(-(val as isize)) });
        // Land on a character boundary, not inside one.
        rex.set_input(unsafe {
            rex.input()
                .offset(-(utf_head_off(rex.line() as *mut c_char, rex.input_str()) as isize))
        });
    } else {
        rex.set_input(rex.line());
    }
}

/// Roughly how unlikely the machine from `state` is to match, as a
/// percentage. Only the ordering matters: it decides whether a lookaround is
/// cheaper to run now or after the rest of the pattern.
pub(crate) fn failure_chance(state: *mut nfa_state_T, depth: c_int) -> c_int {
    if depth > 4 {
        return 1;
    }
    // SAFETY: the walk stays inside the program `state` belongs to.
    let code = op(state);
    match NfaOp::try_from(code) {
        Ok(NfaOp::Split) => {
            // A split of splits is a long alternation; do not try to
            // reason about it.
            if unsafe { (*out_of(state)).c } == NfaOp::Split.code()
                || unsafe { (*out1_of(state)).c } == NfaOp::Split.code()
            {
                return 1;
            }
            let l = failure_chance(out_of(state), depth + 1);
            let r = failure_chance(out1_of(state), depth + 1);
            l.min(r)
        }
        // `.` matches nearly anything.
        Ok(NfaOp::Any) => 1,
        // These match without looking at the input at all.
        Ok(NfaOp::Match | NfaOp::Mclose | NfaOp::AnyComposing) => 0,
        // A lookaround is cheap to *start*.
        Ok(
            NfaOp::StartInvisible
            | NfaOp::StartInvisibleFirst
            | NfaOp::StartInvisibleNeg
            | NfaOp::StartInvisibleNegFirst
            | NfaOp::StartInvisibleBefore
            | NfaOp::StartInvisibleBeforeFirst
            | NfaOp::StartInvisibleBeforeNeg
            | NfaOp::StartInvisibleBeforeNegFirst
            | NfaOp::StartPattern,
        ) => 5,
        // A boundary assertion holds in one position out of very many.
        Ok(NfaOp::Bol | NfaOp::Eol | NfaOp::Bof | NfaOp::Eof | NfaOp::Newl) => 99,
        Ok(NfaOp::Bow | NfaOp::Eow) => 90,
        // A bracket matches nothing itself: ask what is inside it.
        Ok(NfaOp::Nopen | NfaOp::Nclose) => failure_chance(out_of(state), depth + 1),
        Ok(marker) if marker.is_capture_marker() => failure_chance(out_of(state), depth + 1),
        Ok(reference) if reference.is_reference() => 94,
        Ok(NfaOp::Lnum) => 90,
        Ok(NfaOp::Cursor | NfaOp::Col | NfaOp::Vcol | NfaOp::Mark) => 98,
        // The remaining position assertions: an inequality holds more
        // often than an equality.
        Ok(
            NfaOp::LnumGt
            | NfaOp::LnumLt
            | NfaOp::ColGt
            | NfaOp::ColLt
            | NfaOp::VcolGt
            | NfaOp::VcolLt
            | NfaOp::MarkGt
            | NfaOp::MarkLt
            | NfaOp::Visual,
        ) => 85,
        Ok(NfaOp::Composing) => 95,
        // A literal character.
        _ if code > 0 => 95,
        _ => 50,
    }
}

/// Move `*colp` forward to the next occurrence of `c` in the line, or fail
/// if there is none. The whole machine cannot match before the character
/// every match must start with.
pub(crate) fn skip_to_start(rex: Rex, c: c_int, colp: &mut colnr_T) -> Result<(), Failed> {
    // SAFETY: `rex.line` is the NUL-terminated line being matched and
    // `*colp` is a column inside it.
    let from = unsafe { (rex.line() as *mut c_char).offset(*colp as isize) };
    let found = unsafe { cstrchr(rex, from, c) };
    if found.is_null() {
        return Err(Failed);
    }
    *colp = unsafe { found.offset_from(rex.line() as *mut c_char) } as colnr_T;
    Ok(())
}

/// A pattern that is one literal run needs no machine at all: look for the
/// text directly. Returns 1 and fills in the capture slots on a match.
pub(crate) fn find_match_text(
    rex: Rex,
    startcol: &mut colnr_T,
    regstart: c_int,
    match_text: *mut uint8_t,
) -> c_int {
    let mut col = *startcol;
    let regstart_len = utf_char2len(regstart);
    // SAFETY: `match_text` and the line being matched are NUL-terminated,
    // and the walk below stops at either terminator.
    loop {
        let mut matched = true;
        let mut s1 = match_text;
        // The first character is `regstart`, which `skip_to_start`
        // already found — but a fold can encode to a different length,
        // so measure what is actually there.
        let mut regstart_len2 = regstart_len;
        if regstart_len2 > 1
            && unsafe { utf_ptr2len((rex.line() as *mut c_char).offset(col as isize)) }
                != regstart_len2
        {
            regstart_len2 = utf_char2len(utf_fold(regstart));
        }
        let mut s2 = unsafe {
            rex.line()
                .offset(col as isize)
                .offset(regstart_len2 as isize)
        };
        while unsafe { *s1 } != 0 {
            let c1 = unsafe { utf_ptr2char(s1 as *mut c_char) };
            let c2 = unsafe { utf_ptr2char(s2 as *mut c_char) };
            if c1 != c2 && (!rex.reg_ic() || utf_fold(c1) != utf_fold(c2)) {
                matched = false;
                break;
            }
            s1 = unsafe { s1.offset(utf_ptr2len(s1 as *mut c_char) as isize) };
            s2 = unsafe { s2.offset(utf_ptr2len(s2 as *mut c_char) as isize) };
        }
        // A combining character after the run makes it a different
        // grapheme, so it is not the text after all.
        if matched && !utf_iscomposing_legacy(unsafe { utf_ptr2char(s2 as *mut c_char) }) {
            cleanup_subexpr(rex);
            if rex.multi() {
                let start = rex.reg_startpos();
                let end = rex.reg_endpos();
                unsafe { (*start).lnum = rex.lnum() };
                unsafe { (*start).col = col };
                unsafe { (*end).lnum = rex.lnum() };
                unsafe { (*end).col = s2.offset_from(rex.line()) as colnr_T };
            } else {
                unsafe { *rex.reg_startp() = rex.line().offset(col as isize) };
                unsafe { *rex.reg_endp() = s2 };
            }
            *startcol = col;
            return 1;
        }
        col += regstart_len;
        if skip_to_start(rex, regstart, &mut col).is_err() {
            break;
        }
    }
    *startcol = col;
    0
}

/// Has the caller's time limit passed? Records the fact for the caller when
/// it has.
pub(crate) fn nfa_did_time_out() -> bool {
    // SAFETY: both are the caller's out-parameters, null when it gave none.
    let limit = nfa_time_limit.get();
    if limit.is_null() || !profile_passed_limit(unsafe { *limit }) {
        return false;
    }
    if !nfa_timed_out.get().is_null() {
        unsafe { *nfa_timed_out.get() = 1 };
    }
    true
}
