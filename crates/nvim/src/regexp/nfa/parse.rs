//! The recursive descent above the atom: an atom with its repeat, a
//! concatenation, a branch, and the whole pattern.
//!
//! Each level appends to the postfix program rather than returning a tree,
//! so "a then b" is emitted as `a b NFA_CONCAT` and the operators the
//! grammar recognises land after their operands.

#![forbid(unsafe_code)]

use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::atom::nfa_regatom as regatom;
use super::{Parsed, Rejected, postfix};
use crate::main::rc_did_emsg;
use crate::regexp::{
    MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON, MAX_LIMIT, NOT_MULTI, NSUBEXP, RE_AUTO,
    REG_NOPAREN, REG_NPAREN, REG_PAREN, REG_ZPAREN, RF_ICASE, RF_ICOMBINE, RF_NOICASE, Rex, curchr,
    getchr, getdecchrs, had_endbrace, magic, magic_prefix, nfa_re_flags, parse_state_T, peekchr,
    re_multi_type, read_limits, reg_magic, regflags, regnpar, regnzpar, restore_parse_state,
    save_parse_state, skipchr, skipchr_keepstart, unmagic, wants_nfa,
};
use crate::semsg;
use crate::types::NUL;

const M_AMP: c_int = magic(b'&');
const M_AT: c_int = magic(b'@');
const M_BAR: c_int = magic(b'|');
const M_BRACE: c_int = magic(b'{');
const M_C_LOWER: c_int = magic(b'c');
const M_C_UPPER: c_int = magic(b'C');
const M_EQUAL: c_int = magic(b'=');
const M_M_LOWER: c_int = magic(b'm');
const M_M_UPPER: c_int = magic(b'M');
const M_PAREN_CLOSE: c_int = magic(b')');
const M_PLUS: c_int = magic(b'+');
const M_QUESTION: c_int = magic(b'?');
const M_STAR: c_int = magic(b'*');
const M_V_LOWER: c_int = magic(b'v');
const M_V_UPPER: c_int = magic(b'V');
const M_Z_UPPER: c_int = magic(b'Z');

/// Above this many repetitions the automatic engine gives up and lets the
/// backtracking engine have the pattern: `\{n,m}` is expanded by re-parsing
/// its atom `m` times, so a large bound costs states linearly.
const AUTO_MAX_REPEAT: c_int = 500;
const AUTO_MAX_SPAN: c_int = 200;

/// A blank parse-cursor snapshot for [`save_parse_state`] to fill in.
fn no_state() -> parse_state_T {
    parse_state_T {
        regparse: core::ptr::null_mut(),
        prevchr_len: 0,
        curchr: 0,
        prevchr: 0,
        prevprevchr: 0,
        nextchr: 0,
        at_start: 0,
        prev_at_start: 0,
        regnpar: 0,
    }
}

/// The lookaround `\@=`, `\@!`, `\@<=`, `\@<!` and `\@>` operators.
///
/// The two "just before" forms take the optional width `\@123<=` gives,
/// which caps how far back the match may start.
fn lookaround() -> Parsed {
    // Read before the operator: `\@123<=` puts the width in front of it.
    let width = getdecchrs();
    let mut op = unmagic(getchr());
    let code = match op as u8 {
        b'=' => Some(NfaOp::PrevAtomNoWidth.code()),
        b'!' => Some(NfaOp::PrevAtomNoWidthNeg.code()),
        b'>' => Some(NfaOp::PrevAtomLikePattern.code()),
        b'<' => {
            // The message below names whatever followed the `<`, not the
            // `<` itself.
            op = unmagic(getchr());
            match op as u8 {
                b'=' => Some(NfaOp::PrevAtomJustBefore.code()),
                b'!' => Some(NfaOp::PrevAtomJustBeforeNeg.code()),
                _ => None,
            }
        }
        _ => None,
    };
    let Some(code) = code else {
        let op = op as u8 as char;
        semsg!("E869: (NFA) Unknown operator '\\@{op}'");
        return Err(Rejected);
    };
    postfix::emit(code);
    if matches!(
        NfaOp::try_from(code),
        Ok(NfaOp::PrevAtomJustBefore | NfaOp::PrevAtomJustBeforeNeg)
    ) {
        postfix::emit(width as c_int);
    }
    Ok(())
}

/// What a repeat left for [`nfa_regpiece`] to do.
enum Repeat {
    /// Emitted; the caller still rejects a second repeat after it.
    Emitted,
    /// `\{0}`: the piece is finished and a repeat after it is *not*
    /// rejected — `a\{0}*` is accepted, as upstream's early return does.
    Erased,
    Failed,
}

/// `\{n,m}`: emitted by re-parsing the atom up to `maxval` times, each copy
/// after the `minval`th made optional. There is no counted-repeat state in
/// the machine, so this is the only way to say it.
///
/// `atom_start` is where the atom's own items begin; the first pass is
/// thrown away and re-emitted from there.
fn counted_repeat(rex: Rex, before_atom: &parse_state_T, atom_start: usize) -> Repeat {
    // `\{-n,m}` asks for the shortest match.
    let mut greedy = true;
    let c = peekchr();
    if c == b'-' as c_int || c == magic(b'-') {
        skipchr();
        greedy = false;
    }
    let (mut minval, mut maxval) = (0, 0);
    // `read_limits` is shared with the backtracking engine and still
    // answers OK/FAIL.
    if read_limits(&mut minval, &mut maxval).is_err() {
        semsg!("E870: (NFA regexp) Error reading repetition limits");
        rc_did_emsg.set(true);
        return Repeat::Failed;
    }

    // `\{}` and `\{,}` are plain stars.
    if minval == 0 && maxval == MAX_LIMIT {
        postfix::emit(if greedy {
            NfaOp::Star.code()
        } else {
            NfaOp::StarNongreedy.code()
        });
        return Repeat::Emitted;
    }
    // `\{0}` matches nothing at all, so the atom's items go too.
    if maxval == 0 {
        postfix::truncate(atom_start);
        postfix::emit_op(NfaOp::Empty);
        return Repeat::Erased;
    }
    // Under 'regexpengine' = 0 a wide bound is not worth the states; fail
    // out and let the backtracking engine, which counts, take the pattern.
    // Unless something in it only this engine can do (`wants_nfa`).
    if nfa_re_flags.get() & RE_AUTO != 0
        && (maxval > AUTO_MAX_REPEAT || maxval > minval + AUTO_MAX_SPAN)
        && (maxval != MAX_LIMIT && minval < AUTO_MAX_SPAN)
        && !wants_nfa.get()
    {
        return Repeat::Failed;
    }

    postfix::truncate(atom_start);
    // Where the pattern continues, to be restored once the copies are out.
    let mut after_atom = no_state();
    save_parse_state(&mut after_atom);
    let quest = if greedy {
        NfaOp::Quest.code()
    } else {
        NfaOp::QuestNongreedy.code()
    };
    let mut i = 0;
    while i < maxval {
        restore_parse_state(before_atom);
        let copy_start = postfix::len();
        if regatom(rex).is_err() {
            return Repeat::Failed;
        }
        if i + 1 > minval {
            if maxval == MAX_LIMIT {
                // An open-ended bound: the last copy stands for all of them.
                postfix::emit(if greedy {
                    NfaOp::Star.code()
                } else {
                    NfaOp::StarNongreedy.code()
                });
            } else {
                postfix::emit(quest);
            }
        }
        // Nothing to join to for the first copy — and an atom that emitted
        // no items at all leaves nothing to join either.
        if copy_start != atom_start {
            postfix::emit_op(NfaOp::Concat);
        }
        if i + 1 > minval && maxval == MAX_LIMIT {
            break;
        }
        i += 1;
    }
    restore_parse_state(&after_atom);
    curchr.set(-1);
    Repeat::Emitted
}

/// One atom and the repeat that follows it, if any.
pub(crate) fn nfa_regpiece(rex: Rex) -> Parsed {
    // `\+` and `\{n,m}` re-parse the atom, so the cursor as it stood before
    // it has to be recoverable.
    let mut before_atom = no_state();
    save_parse_state(&mut before_atom);
    let atom_start = postfix::len();

    regatom(rex)?;
    let op = peekchr();
    if re_multi_type(op) == NOT_MULTI {
        return Ok(());
    }
    skipchr();

    match op {
        M_STAR => postfix::emit_op(NfaOp::Star),
        // `\+` is "the atom, then the atom starred", which means parsing it
        // a second time.
        M_PLUS => {
            restore_parse_state(&before_atom);
            curchr.set(-1);
            regatom(rex)?;
            postfix::emit_op(NfaOp::Star);
            postfix::emit_op(NfaOp::Concat);
            skipchr();
        }
        M_AT => lookaround()?,
        M_QUESTION | M_EQUAL => postfix::emit_op(NfaOp::Quest),
        M_BRACE => match counted_repeat(rex, &before_atom, atom_start) {
            Repeat::Emitted => {}
            Repeat::Erased => return Ok(()),
            Repeat::Failed => return Err(Rejected),
        },
        _ => {}
    }

    if re_multi_type(peekchr()) != NOT_MULTI {
        semsg!("E871: (NFA regexp) Can't have a multi follow a multi");
        rc_did_emsg.set(true);
        return Err(Rejected);
    }
    Ok(())
}

/// A run of pieces, and the flag escapes that can appear between them.
///
/// `\c`, `\v` and friends match nothing; they change how the rest of the
/// pattern is read, which is why they are handled here rather than in the
/// atom parser.
pub(crate) fn nfa_regconcat(rex: Rex) -> Parsed {
    let mut first = true;
    loop {
        match peekchr() {
            // Anything that ends a concatenation is left for the caller.
            NUL | M_BAR | M_AMP | M_PAREN_CLOSE => return Ok(()),
            M_Z_UPPER => {
                regflags.set(regflags.get() | RF_ICOMBINE as u32);
                skipchr_keepstart();
            }
            M_C_LOWER => {
                regflags.set(regflags.get() | RF_ICASE as u32);
                skipchr_keepstart();
            }
            M_C_UPPER => {
                regflags.set(regflags.get() | RF_NOICASE as u32);
                skipchr_keepstart();
            }
            // A 'magic' change alters what the *next* byte means, so the
            // lookahead has to be dropped along with it.
            M_V_LOWER => set_magic(MAGIC_ALL),
            M_M_LOWER => set_magic(MAGIC_ON),
            M_M_UPPER => set_magic(MAGIC_OFF),
            M_V_UPPER => set_magic(MAGIC_NONE),
            _ => {
                nfa_regpiece(rex)?;
                if first {
                    first = false;
                } else {
                    postfix::emit_op(NfaOp::Concat);
                }
            }
        }
    }
}

fn set_magic(level: crate::types::magic_T) {
    reg_magic.set(level);
    skipchr_keepstart();
    curchr.set(-1);
}

/// One branch: concatenations joined by `\&`, all of which must match at the
/// same position, and the last of which is the one that counts.
///
/// `a\&b` compiles as "b, with a as a zero-width lookahead in front of it",
/// which is why each concatenation but the last is wrapped in
/// `NFA_NOPEN` + `NFA_PREV_ATOM_NO_WIDTH`.
pub(crate) fn nfa_regbranch(rex: Rex) -> Parsed {
    let mut concat_start = postfix::len();
    nfa_regconcat(rex)?;
    while peekchr() == M_AMP {
        skipchr();
        // An empty concatenation still has to leave an item behind for the
        // operator that follows to apply to.
        if concat_start == postfix::len() {
            postfix::emit_op(NfaOp::Empty);
        }
        postfix::emit_op(NfaOp::Nopen);
        postfix::emit_op(NfaOp::PrevAtomNoWidth);
        concat_start = postfix::len();
        nfa_regconcat(rex)?;
        if concat_start == postfix::len() {
            postfix::emit_op(NfaOp::Empty);
        }
        postfix::emit_op(NfaOp::Concat);
    }
    if concat_start == postfix::len() {
        postfix::emit_op(NfaOp::Empty);
    }
    Ok(())
}

/// What kind of bracket the pattern being parsed sits inside, if any.
fn open_bracket(paren: c_int) -> Parsed<c_int> {
    match paren {
        REG_PAREN => {
            if regnpar.get() >= NSUBEXP as c_int {
                semsg!("E872: (NFA regexp) Too many '('");
                rc_did_emsg.set(true);
                return Err(Rejected);
            }
            let parno = regnpar.get();
            regnpar.set(parno + 1);
            Ok(parno)
        }
        REG_ZPAREN => {
            if regnzpar.get() >= NSUBEXP as c_int {
                semsg!("E879: (NFA regexp) Too many \\z(");
                rc_did_emsg.set(true);
                return Err(Rejected);
            }
            let parno = regnzpar.get();
            regnzpar.set(parno + 1);
            Ok(parno)
        }
        _ => Ok(0),
    }
}

/// Report the bracket the pattern failed to close or opened too many of.
fn unbalanced(paren: c_int) -> Rejected {
    let prefix = magic_prefix();
    if paren == REG_NPAREN {
        semsg!("E53: Unmatched {prefix}%(");
    } else {
        semsg!("E54: Unmatched {prefix}(");
    }
    rc_did_emsg.set(true);
    Rejected
}

/// A whole pattern, or the contents of one bracket: branches joined by `\|`.
///
/// `paren` says which bracket the caller opened, and hence what has to close
/// it and which capture group the result becomes.
pub(crate) fn nfa_reg(rex: Rex, paren: c_int) -> Parsed {
    let parno = open_bracket(paren)?;

    nfa_regbranch(rex)?;
    while peekchr() == magic(b'|') {
        skipchr();
        nfa_regbranch(rex)?;
        postfix::emit_op(NfaOp::Or);
    }

    if paren != REG_NOPAREN {
        if getchr() != M_PAREN_CLOSE {
            return Err(unbalanced(paren));
        }
    } else if peekchr() != NUL {
        // The whole pattern was parsed but there is more text: either a
        // stray `\)` or something the grammar never reached.
        if peekchr() == M_PAREN_CLOSE {
            let prefix = magic_prefix();
            semsg!("E55: Unmatched {prefix})");
        } else {
            semsg!("E873: (NFA regexp) proper termination error");
        }
        rc_did_emsg.set(true);
        return Err(Rejected);
    }

    // The bracket's own marker goes last, as the operator over everything
    // the branches emitted.
    if paren == REG_PAREN {
        had_endbrace.with_mut(|seen| seen[parno as usize] = 1);
        postfix::emit_op(NfaOp::mopen(parno));
    } else if paren == REG_ZPAREN {
        postfix::emit_op(NfaOp::zopen(parno));
    }
    Ok(())
}

/// Compile the pattern at the parse cursor into the postfix program.
///
/// The trailing `NFA_MOPEN` is capture group 0 — the whole match — which
/// `post2nfa` turns into the machine's entry and exit states.
pub(crate) fn re2post(rex: Rex) -> Parsed {
    nfa_reg(rex, REG_NOPAREN)?;
    postfix::emit_op(NfaOp::Mopen);
    Ok(())
}
