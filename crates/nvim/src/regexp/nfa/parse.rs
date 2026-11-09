//! The recursive descent above the atom: an atom with its repeat, a
//! concatenation, a branch, and the whole pattern.
//!
//! Each level appends to the postfix program rather than returning a tree,
//! so "a then b" is emitted as `a b NFA_CONCAT` and the operators the
//! grammar recognises land after their operands.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::narrow::number_as_int;
use crate::regexp::chars::{
    M_AMP, M_AT, M_BAR, M_BRACE, M_C_LOWER, M_C_UPPER, M_EQUAL, M_M_LOWER, M_M_UPPER,
    M_PAREN_CLOSE, M_PLUS, M_QUESTION, M_STAR, M_V_LOWER, M_V_UPPER, M_Z_UPPER,
};
use crate::regexp::{NfaOp, RegCompiler, low_byte, set_magic};
use core::ffi::c_int;

use super::Parsed;
use super::atom::nfa_regatom as regatom;
use crate::ascii::NUL;
use crate::regexp::{
    CompileError, MAGIC_ALL, MAGIC_NONE, MAGIC_OFF, MAGIC_ON, MAX_LIMIT, NOT_MULTI, NSUBEXP,
    ParseState, RE_AUTO, REG_NOPAREN, REG_NPAREN, REG_PAREN, REG_ZPAREN, RF_ICASE, RF_ICOMBINE,
    RF_NOICASE, getchr, getdecchrs, magic, peekchr, re_multi_type, read_limits,
    restore_parse_state, save_parse_state, skipchr, skipchr_keepstart, spelled, unmagic,
};

/// Above this many repetitions the automatic engine gives up and lets the
/// backtracking engine have the pattern: `\{n,m}` is expanded by re-parsing
/// its atom `m` times, so a large bound costs states linearly.
const AUTO_MAX_REPEAT: c_int = 500;
const AUTO_MAX_SPAN: c_int = 200;

/// The lookaround `\@=`, `\@!`, `\@<=`, `\@<!` and `\@>` operators.
///
/// The two "just before" forms take the optional width `\@123<=` gives,
/// which caps how far back the match may start.
fn lookaround(rc: &mut RegCompiler) -> Parsed {
    // Read before the operator: `\@123<=` puts the width in front of it.
    let width = getdecchrs(rc);
    let mut op = unmagic(getchr(rc));
    let code = match low_byte(op) {
        b'=' => Some(NfaOp::PrevAtomNoWidth.code()),
        b'!' => Some(NfaOp::PrevAtomNoWidthNeg.code()),
        b'>' => Some(NfaOp::PrevAtomLikePattern.code()),
        b'<' => {
            // The message below names whatever followed the `<`, not the
            // `<` itself.
            op = unmagic(getchr(rc));
            match low_byte(op) {
                b'=' => Some(NfaOp::PrevAtomJustBefore.code()),
                b'!' => Some(NfaOp::PrevAtomJustBeforeNeg.code()),
                _ => None,
            }
        }
        _ => None,
    };
    let Some(code) = code else {
        return Err(CompileError::UnknownAtOperator(low_byte(op)));
    };
    rc.post.emit(code);
    if matches!(
        NfaOp::try_from(code),
        Ok(NfaOp::PrevAtomJustBefore | NfaOp::PrevAtomJustBeforeNeg)
    ) {
        // Upstream's `(int)` of the `long` the digits were read into.
        rc.post.emit(number_as_int(width));
    }
    Ok(())
}

/// What a repeat left for [`nfa_regpiece`] to do.
#[derive(Debug)]
enum Repeat {
    /// Emitted; the caller still rejects a second repeat after it.
    Emitted,
    /// `\{0}`: the piece is finished and a repeat after it is *not*
    /// rejected — `a\{0}*` is accepted, as upstream's early return does.
    Erased,
}

/// `\{n,m}`: emitted by re-parsing the atom up to `maxval` times, each copy
/// after the `minval`th made optional. There is no counted-repeat state in
/// the machine, so this is the only way to say it.
///
/// `atom_start` is where the atom's own items begin; the first pass is
/// thrown away and re-emitted from there.
fn counted_repeat(
    rc: &mut RegCompiler,
    before_atom: &ParseState,
    atom_start: usize,
) -> Parsed<Repeat> {
    // `\{-n,m}` asks for the shortest match.
    let mut greedy = true;
    let c = peekchr(rc);
    if c == c_int::from(b'-') || c == magic(b'-') {
        skipchr(rc);
        greedy = false;
    }
    let (mut minval, mut maxval) = (0, 0);
    // `read_limits` is shared with the backtracking engine, whose refusal
    // this engine follows with its own.
    if let Err(err) = read_limits(rc, &mut minval, &mut maxval) {
        return Err(match err {
            CompileError::BraceSyntax(spelled) => CompileError::RepetitionLimits(spelled),
            other => other,
        });
    }

    // `\{}` and `\{,}` are plain stars.
    if minval == 0 && maxval == MAX_LIMIT {
        rc.post.emit(if greedy {
            NfaOp::Star.code()
        } else {
            NfaOp::StarNongreedy.code()
        });
        return Ok(Repeat::Emitted);
    }
    // `\{0}` matches nothing at all, so the atom's items go too.
    if maxval == 0 {
        rc.post.truncate(atom_start);
        rc.post.emit_op(NfaOp::Empty);
        return Ok(Repeat::Erased);
    }
    // Under 'regexpengine' = 0 a wide bound is not worth the states; fail
    // out and let the backtracking engine, which counts, take the pattern.
    // Unless something in it only this engine can do (`wants_nfa`).
    if rc.re_flags & RE_AUTO != 0
        && (maxval > AUTO_MAX_REPEAT || maxval > minval + AUTO_MAX_SPAN)
        && (maxval != MAX_LIMIT && minval < AUTO_MAX_SPAN)
        && !rc.wants_nfa
    {
        return Err(CompileError::Declined);
    }

    rc.post.truncate(atom_start);
    // Where the pattern continues, to be restored once the copies are out.
    let mut after_atom = ParseState::default();
    save_parse_state(rc, &mut after_atom);
    let quest = if greedy {
        NfaOp::Quest.code()
    } else {
        NfaOp::QuestNongreedy.code()
    };
    let mut i = 0;
    while i < maxval {
        restore_parse_state(rc, before_atom);
        let copy_start = rc.post.len();
        regatom(rc)?;
        if i + 1 > minval {
            if maxval == MAX_LIMIT {
                // An open-ended bound: the last copy stands for all of them.
                rc.post.emit(if greedy {
                    NfaOp::Star.code()
                } else {
                    NfaOp::StarNongreedy.code()
                });
            } else {
                rc.post.emit(quest);
            }
        }
        // Nothing to join to for the first copy — and an atom that emitted
        // no items at all leaves nothing to join either.
        if copy_start != atom_start {
            rc.post.emit_op(NfaOp::Concat);
        }
        if i + 1 > minval && maxval == MAX_LIMIT {
            break;
        }
        i += 1;
    }
    restore_parse_state(rc, &after_atom);
    rc.token = -1;
    Ok(Repeat::Emitted)
}

/// One atom and the repeat that follows it, if any.
pub(crate) fn nfa_regpiece(rc: &mut RegCompiler) -> Parsed {
    // `\+` and `\{n,m}` re-parse the atom, so the cursor as it stood before
    // it has to be recoverable.
    let mut before_atom = ParseState::default();
    save_parse_state(rc, &mut before_atom);
    let atom_start = rc.post.len();

    regatom(rc)?;
    let op = peekchr(rc);
    if re_multi_type(op) == NOT_MULTI {
        return Ok(());
    }
    skipchr(rc);

    match op {
        M_STAR => rc.post.emit_op(NfaOp::Star),
        // `\+` is "the atom, then the atom starred", which means parsing it
        // a second time.
        M_PLUS => {
            restore_parse_state(rc, &before_atom);
            rc.token = -1;
            regatom(rc)?;
            rc.post.emit_op(NfaOp::Star);
            rc.post.emit_op(NfaOp::Concat);
            skipchr(rc);
        }
        M_AT => lookaround(rc)?,
        M_QUESTION | M_EQUAL => rc.post.emit_op(NfaOp::Quest),
        M_BRACE => match counted_repeat(rc, &before_atom, atom_start)? {
            Repeat::Emitted => {}
            Repeat::Erased => return Ok(()),
        },
        _ => {}
    }

    if re_multi_type(peekchr(rc)) != NOT_MULTI {
        return Err(CompileError::MultiAfterMulti);
    }
    Ok(())
}

/// A run of pieces, and the flag escapes that can appear between them.
///
/// `\c`, `\v` and friends match nothing; they change how the rest of the
/// pattern is read, which is why they are handled here rather than in the
/// atom parser.
pub(crate) fn nfa_regconcat(rc: &mut RegCompiler) -> Parsed {
    let mut first = true;
    loop {
        match peekchr(rc) {
            // Anything that ends a concatenation is left for the caller.
            NUL | M_BAR | M_AMP | M_PAREN_CLOSE => return Ok(()),
            M_Z_UPPER => {
                rc.flags |= RF_ICOMBINE as u32;
                skipchr_keepstart(rc);
            }
            M_C_LOWER => {
                rc.flags |= RF_ICASE as u32;
                skipchr_keepstart(rc);
            }
            M_C_UPPER => {
                rc.flags |= RF_NOICASE as u32;
                skipchr_keepstart(rc);
            }
            // A 'magic' change alters what the *next* byte means, so the
            // lookahead has to be dropped along with it.
            M_V_LOWER => set_magic(rc, MAGIC_ALL),
            M_M_LOWER => set_magic(rc, MAGIC_ON),
            M_M_UPPER => set_magic(rc, MAGIC_OFF),
            M_V_UPPER => set_magic(rc, MAGIC_NONE),
            _ => {
                nfa_regpiece(rc)?;
                if first {
                    first = false;
                } else {
                    rc.post.emit_op(NfaOp::Concat);
                }
            }
        }
    }
}

/// One branch: concatenations joined by `\&`, all of which must match at the
/// same position, and the last of which is the one that counts.
///
/// `a\&b` compiles as "b, with a as a zero-width lookahead in front of it",
/// which is why each concatenation but the last is wrapped in
/// `NFA_NOPEN` + `NFA_PREV_ATOM_NO_WIDTH`.
pub(crate) fn nfa_regbranch(rc: &mut RegCompiler) -> Parsed {
    let mut concat_start = rc.post.len();
    nfa_regconcat(rc)?;
    while peekchr(rc) == M_AMP {
        skipchr(rc);
        // An empty concatenation still has to leave an item behind for the
        // operator that follows to apply to.
        if concat_start == rc.post.len() {
            rc.post.emit_op(NfaOp::Empty);
        }
        rc.post.emit_op(NfaOp::Nopen);
        rc.post.emit_op(NfaOp::PrevAtomNoWidth);
        concat_start = rc.post.len();
        nfa_regconcat(rc)?;
        if concat_start == rc.post.len() {
            rc.post.emit_op(NfaOp::Empty);
        }
        rc.post.emit_op(NfaOp::Concat);
    }
    if concat_start == rc.post.len() {
        rc.post.emit_op(NfaOp::Empty);
    }
    Ok(())
}

/// What kind of bracket the pattern being parsed sits inside, if any.
fn open_bracket(rc: &mut RegCompiler, paren: c_int) -> Parsed<usize> {
    match paren {
        REG_PAREN => {
            if rc.next_group >= NSUBEXP as usize {
                return Err(CompileError::NfaTooManyGroups);
            }
            let parno = rc.next_group;
            rc.next_group = parno + 1;
            Ok(parno)
        }
        REG_ZPAREN => {
            if rc.next_zgroup >= NSUBEXP as usize {
                return Err(CompileError::NfaTooManyZGroups);
            }
            let parno = rc.next_zgroup;
            rc.next_zgroup = parno + 1;
            Ok(parno)
        }
        _ => Ok(0),
    }
}

/// The bracket the pattern failed to close.
fn unbalanced(rc: &RegCompiler, paren: c_int) -> CompileError {
    if paren == REG_NPAREN {
        CompileError::UnmatchedNoCapture(spelled(rc))
    } else {
        CompileError::UnmatchedGroup(spelled(rc))
    }
}

/// A whole pattern, or the contents of one bracket: branches joined by `\|`.
///
/// `paren` says which bracket the caller opened, and hence what has to close
/// it and which capture group the result becomes.
pub(crate) fn nfa_reg(rc: &mut RegCompiler, paren: c_int) -> Parsed {
    let parno = open_bracket(rc, paren)?;

    nfa_regbranch(rc)?;
    while peekchr(rc) == magic(b'|') {
        skipchr(rc);
        nfa_regbranch(rc)?;
        rc.post.emit_op(NfaOp::Or);
    }

    if paren != REG_NOPAREN {
        if getchr(rc) != M_PAREN_CLOSE {
            return Err(unbalanced(rc, paren));
        }
    } else if peekchr(rc) != NUL {
        // The whole pattern was parsed but there is more text: either a
        // stray `\)` or something the grammar never reached.
        if peekchr(rc) == M_PAREN_CLOSE {
            return Err(CompileError::UnmatchedClose(spelled(rc)));
        }
        return Err(CompileError::Termination);
    }

    // The bracket's own marker goes last, as the operator over everything
    // the branches emitted.
    if paren == REG_PAREN {
        rc.closed_groups[parno] = 1;
        rc.post.emit_op(NfaOp::mopen(parno));
    } else if paren == REG_ZPAREN {
        rc.post.emit_op(NfaOp::zopen(parno));
    }
    Ok(())
}

/// Compile the pattern at the parse cursor into the postfix program.
///
/// The trailing `NFA_MOPEN` is capture group 0 — the whole match — which
/// `post2nfa` turns into the machine's entry and exit states.
pub(crate) fn re2post(rc: &mut RegCompiler) -> Parsed {
    nfa_reg(rc, REG_NOPAREN)?;
    rc.post.emit_op(NfaOp::Mopen);
    Ok(())
}
