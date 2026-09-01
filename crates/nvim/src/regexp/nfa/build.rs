//! Postfix form to state machine.
//!
//! Thompson's construction: the postfix program is run on a stack of
//! *fragments*, each an entry state plus the list of `out` pointers that are
//! still dangling. An operand pushes a one-state fragment; an operator pops
//! its operands, wires them together and pushes the result. The last
//! fragment left is the machine.
//!
//! It runs twice per compile. The first pass only counts states, because the
//! program is allocated as one block with the states inline; the second
//! builds into that block.

#![deny(unsafe_op_in_unsafe_fn)]

use super::list::{op, out_of, out1_of};
use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::run::failure_chance;
use super::sub::match_follows;
use crate::main::rc_did_emsg;
use crate::mbyte::utf_char2len;
use crate::regexp::{istate, nfa_regprog_T, nfa_state_T, nstate, state_ptr};
use crate::semsg;
use crate::types::MB_MAXBYTES;

/// How far the width walk follows a `NFA_SPLIT` before giving up.
const MAX_DEPTH: c_int = 4;

/// One unset edge slot of a half-built machine, seen as a link in the list
/// of slots still waiting to be patched.
///
/// The list has no storage of its own: each unset `out`/`out1` field holds
/// the address of the *next* unset slot until [`patch`] overwrites it with
/// the state the edge goes to. So a `Ptrlist` is an edge slot, and the two
/// things it can hold are the two things a `*mut nfa_state_T` can be made
/// to hold — which is why it is a newtype over that field's own type rather
/// than a `Vec`.
#[repr(transparent)]
struct Ptrlist(*mut nfa_state_T);

impl Ptrlist {
    /// The next slot in the list, or null at its end.
    fn next(&self) -> *mut Ptrlist {
        self.0.cast::<Ptrlist>()
    }

    /// Link this slot to `next`, keeping it unpatched.
    fn set_next(&mut self, next: *mut Ptrlist) {
        self.0 = next.cast::<nfa_state_T>();
    }

    /// Patch this slot: the edge goes to `state` from now on.
    fn set_state(&mut self, state: *mut nfa_state_T) {
        self.0 = state;
    }
}

/// A partly built machine: where it starts, and every edge out of it that
/// still has to be told where to go.
#[derive(Clone, Copy)]
struct Frag {
    start: *mut nfa_state_T,
    out: *mut Ptrlist,
}

/// Take the next state out of the program's inline array.
///
/// `None` once the counting pass's estimate is used up, which the callers
/// treat as "give up on this pattern" — silently, as upstream does.
fn state(c: c_int, out: *mut nfa_state_T, out1: *mut nfa_state_T) -> Option<*mut nfa_state_T> {
    if istate.get() >= nstate.get() {
        return None;
    }
    // SAFETY: `state_ptr` is the program's state array and `istate` is
    // below `nstate`, its length.
    let s = unsafe { state_ptr.get().offset(istate.get() as isize) };
    istate.set(istate.get() + 1);
    unsafe { (*s).c = c };
    unsafe { (*s).out = out };
    unsafe { (*s).out1 = out1 };
    unsafe { (*s).val = 0 };
    unsafe { (*s).id = istate.get() };
    unsafe { (*s).lastlist = [0, 0] };
    Some(s)
}

/// A one-element patch list over the edge slot `slot`.
fn list1(slot: *mut *mut nfa_state_T) -> *mut Ptrlist {
    // SAFETY: `slot` is an `out`/`out1` field of a live state; writing a
    // list link into it is what the list is for, and `patch` overwrites it
    // with a state before the machine runs.
    let list = slot.cast::<Ptrlist>();
    unsafe { (*list).set_next(core::ptr::null_mut()) };
    list
}

/// Point every edge in `list` at `state`.
fn patch(list: *mut Ptrlist, state: *mut nfa_state_T) {
    // SAFETY: every node in the chain is an edge slot of a live state.
    let mut node = list;
    while !node.is_null() {
        let next = unsafe { (*node).next() };
        unsafe { (*node).set_state(state) };
        node = next;
    }
}

/// Concatenate two patch lists, returning the first.
fn append(first: *mut Ptrlist, second: *mut Ptrlist) -> *mut Ptrlist {
    // SAFETY: as `patch`; `first` is never empty where this is called.
    let mut last = first;
    while !unsafe { (*last).next() }.is_null() {
        last = unsafe { (*last).next() };
    }
    unsafe { (*last).set_next(second) };
    first
}

fn out_edge(s: *mut nfa_state_T) -> *mut *mut nfa_state_T {
    // SAFETY: `s` is a live state; this only takes the field's address.
    unsafe { &raw mut (*s).out }
}

fn out1_edge(s: *mut nfa_state_T) -> *mut *mut nfa_state_T {
    // SAFETY: as `out_edge`.
    unsafe { &raw mut (*s).out1 }
}

/// The fragment stack.
///
/// Capped at the state count the counting pass produced, as upstream's is: a
/// push past the cap is silently dropped rather than growing the stack.
struct Stack {
    frags: Vec<Frag>,
    cap: usize,
}

impl Stack {
    fn push(&mut self, start: *mut nfa_state_T, out: *mut Ptrlist) {
        if self.frags.len() < self.cap {
            self.frags.push(Frag { start, out });
        }
    }

    /// Pop, reporting E874 when the program asks for an operand that is not
    /// there.
    fn pop(&mut self) -> Option<Frag> {
        let frag = self.frags.pop();
        if frag.is_none() {
            semsg!("E874: (NFA) Could not pop the stack!");
        }
        frag
    }
}

/// Which of the two passes is running.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Pass {
    /// Only add up the states the program needs, into `nstate`.
    Count,
    /// Build into the program's state array.
    Build,
}

/// The widest match the machine from `startstate` can make, or -1 when that
/// cannot be bounded. Backs `\@<=`, which has to know how far back to start.
fn nfa_max_width(startstate: *mut nfa_state_T, depth: c_int) -> c_int {
    if depth > MAX_DEPTH {
        return -1;
    }
    let mut len = 0;
    let mut state = startstate;
    // SAFETY: the walk stays inside the program `startstate` belongs to.

    while !state.is_null() {
        let code = op(state);
        match NfaOp::try_from(code) {
            // The end of the lookbehind's own pattern.
            Ok(NfaOp::EndInvisible | NfaOp::EndInvisibleNeg) => return len,
            Ok(NfaOp::Split) => {
                let l = nfa_max_width(out_of(state), depth + 1);
                let r = nfa_max_width(out1_of(state), depth + 1);
                if l < 0 || r < 0 {
                    return -1;
                }
                return len + l.max(r);
            }
            // Any character, so as wide as a character gets. A
            // collection continues past its `NFA_END_COLL`, which is
            // where `out1` points.
            Ok(c @ (NfaOp::Any | NfaOp::StartColl | NfaOp::StartNegColl)) => {
                len += MB_MAXBYTES as c_int;
                if c != NfaOp::Any {
                    if out1_of(state).is_null() || out_of(out1_of(state)).is_null() {
                        return -1;
                    }
                    state = out_of(out1_of(state));
                    continue;
                }
            }
            // The ASCII-only classes are one byte.
            Ok(NfaOp::Digit | NfaOp::White | NfaOp::Hex | NfaOp::Octal) => len += 1,
            // The rest can match a multibyte character, which upstream
            // bounds at three bytes rather than `MB_MAXBYTES`.
            Ok(NfaOp::AnyComposing) => len += 3,
            Ok(class) if class.is_class() => len += 3,
            // A nested lookaround matches no input of its own; step
            // over it to what follows.
            Ok(
                NfaOp::StartInvisible
                | NfaOp::StartInvisibleNeg
                | NfaOp::StartInvisibleBefore
                | NfaOp::StartInvisibleBeforeNeg,
            ) => {
                state = out_of(out1_of(state));
                continue;
            }
            // A back-reference is as wide as whatever it captured, and a
            // line break or a skip is unbounded.
            Ok(NfaOp::Newl | NfaOp::Skip) => return -1,
            Ok(reference) if reference.is_reference() => return -1,
            // Zero width.
            Ok(marker) if marker.is_capture_marker() => {}
            Ok(
                NfaOp::Bol
                | NfaOp::Eol
                | NfaOp::Bow
                | NfaOp::Eow
                | NfaOp::Bof
                | NfaOp::Eof
                | NfaOp::Nopen
                | NfaOp::Nclose
                | NfaOp::Cursor
                | NfaOp::Lnum
                | NfaOp::LnumGt
                | NfaOp::LnumLt
                | NfaOp::Col
                | NfaOp::ColGt
                | NfaOp::ColLt
                | NfaOp::Vcol
                | NfaOp::VcolGt
                | NfaOp::VcolLt
                | NfaOp::Mark
                | NfaOp::MarkGt
                | NfaOp::MarkLt
                | NfaOp::Visual
                | NfaOp::Zstart
                | NfaOp::Zend
                | NfaOp::OptChars
                | NfaOp::Empty
                | NfaOp::StartPattern
                | NfaOp::EndPattern
                | NfaOp::Composing
                | NfaOp::EndComposing,
            ) => {}
            // A literal character; any opcode not named above is one
            // this walk cannot reason about.
            _ => {
                if code < 0 {
                    return -1;
                }
                len += utf_char2len(code);
            }
        }
        state = out_of(state);
    }
    -1
}

/// Run the postfix program.
///
/// [`Pass::Count`] only adds up `nstate` and returns null; [`Pass::Build`]
/// returns the machine's entry state, or null once it has said why not.
pub(crate) fn post2nfa(items: &[c_int], pass: Pass) -> *mut nfa_state_T {
    let counting = pass == Pass::Count;
    let cap = if counting {
        0
    } else {
        nstate.get() as usize + 1
    };
    let mut stack = Stack {
        frags: Vec::with_capacity(cap),
        cap,
    };

    let mut i = 0;
    while i < items.len() {
        let item = items[i];
        // The two operators that carry an inline operand read it here, so
        // that both passes step over it.
        let operand = if NfaOp::try_from(item).is_ok_and(NfaOp::has_inline_operand) {
            i += 1;
            items[i]
        } else {
            0
        };

        if counting {
            nstate.set(nstate.get() + count_for(item, operand));
            i += 1;
            continue;
        }

        match NfaOp::try_from(item) {
            Ok(NfaOp::Concat) => {
                // Two fragments run one after the other: the first's loose
                // ends go to the second's start.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return core::ptr::null_mut();
                };
                patch(e1.out, e2.start);
                stack.push(e1.start, e2.out);
            }
            Ok(NfaOp::Or) => {
                // A choice between two fragments.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return core::ptr::null_mut();
                };
                let Some(s) = state(NfaOp::Split.code(), e1.start, e2.start) else {
                    return core::ptr::null_mut();
                };
                stack.push(s, append(e1.out, e2.out));
            }
            // A repeat is a split whose taken branch loops back to it; `\?`
            // is the same split without the loop. Which of the two edges is
            // the body and which the exit decides whether it is greedy.
            Ok(
                repeat
                @ (NfaOp::Star | NfaOp::StarNongreedy | NfaOp::Quest | NfaOp::QuestNongreedy),
            ) => {
                let Some(e) = stack.pop() else {
                    return core::ptr::null_mut();
                };
                let greedy = matches!(repeat, NfaOp::Star | NfaOp::Quest);
                let (out, out1) = if greedy {
                    (e.start, core::ptr::null_mut())
                } else {
                    (core::ptr::null_mut(), e.start)
                };
                let Some(s) = state(NfaOp::Split.code(), out, out1) else {
                    return core::ptr::null_mut();
                };
                let exit = list1(if greedy { out1_edge(s) } else { out_edge(s) });
                if matches!(repeat, NfaOp::Star | NfaOp::StarNongreedy) {
                    patch(e.out, s);
                    stack.push(s, exit);
                } else {
                    stack.push(s, append(e.out, exit));
                }
            }
            Ok(NfaOp::EndColl | NfaOp::EndNegColl) => {
                // The collection's members are already chained; close them
                // off with the state the matcher stops at, and point the
                // opening state's `out1` at it so a walk can skip over the
                // whole collection.
                let Some(e) = stack.pop() else {
                    return core::ptr::null_mut();
                };
                let Some(s) = state(
                    NfaOp::EndColl.code(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) else {
                    return core::ptr::null_mut();
                };
                patch(e.out, s);
                // SAFETY: `e.start` is the collection's opening state.
                unsafe { (*e.start).out1 = s };
                stack.push(e.start, list1(out_edge(s)));
            }
            Ok(NfaOp::Range) => {
                // Two members become a range: their character values move
                // into `val` and the opcodes say which end each one is.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return core::ptr::null_mut();
                };
                // SAFETY: both fragments are single member states.
                unsafe { (*e2.start).val = (*e2.start).c };
                unsafe { (*e2.start).c = NfaOp::RangeMax.code() };
                unsafe { (*e1.start).val = (*e1.start).c };
                unsafe { (*e1.start).c = NfaOp::RangeMin.code() };
                patch(e1.out, e2.start);
                stack.push(e1.start, e2.out);
            }
            Ok(NfaOp::OptChars) => {
                // `\%[abc]`: `operand` members, each of which becomes a
                // split that can leave the sequence, so every prefix
                // matches.
                let mut n = operand;
                let mut s = core::ptr::null_mut();
                let mut s1 = core::ptr::null_mut();
                let mut first_out = core::ptr::null_mut::<Ptrlist>();
                while n > 0 {
                    n -= 1;
                    let Some(e) = stack.pop() else {
                        return core::ptr::null_mut();
                    };
                    let Some(new) = state(NfaOp::Split.code(), e.start, core::ptr::null_mut())
                    else {
                        return core::ptr::null_mut();
                    };
                    if first_out.is_null() {
                        first_out = e.out;
                    }
                    patch(e.out, s1);
                    append(first_out, list1(out1_edge(new)));
                    s1 = new;
                    s = new;
                }
                stack.push(s, first_out);
            }
            // The lookarounds: the operand becomes a machine of its own,
            // entered from an `NFA_START_*` state and ended by the matching
            // `NFA_END_*`.
            Ok(
                look @ (NfaOp::PrevAtomNoWidth
                | NfaOp::PrevAtomNoWidthNeg
                | NfaOp::PrevAtomJustBefore
                | NfaOp::PrevAtomJustBeforeNeg
                | NfaOp::PrevAtomLikePattern),
            ) => {
                let (start_state, end_state) = lookaround_states(look);
                let before = matches!(
                    look,
                    NfaOp::PrevAtomJustBefore | NfaOp::PrevAtomJustBeforeNeg
                );
                let pattern = look == NfaOp::PrevAtomLikePattern;
                let Some(e) = stack.pop() else {
                    return core::ptr::null_mut();
                };
                let Some(end) = state(
                    end_state.code(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) else {
                    return core::ptr::null_mut();
                };
                let Some(s) = state(start_state.code(), e.start, end) else {
                    return core::ptr::null_mut();
                };
                if pattern {
                    // `\@>` keeps what it matched, so the sub-match's end is
                    // recorded and the outer match resumes past it.
                    let Some(skip) = state(
                        NfaOp::Skip.code(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) else {
                        return core::ptr::null_mut();
                    };
                    let Some(zend) = state(NfaOp::Zend.code(), end, core::ptr::null_mut()) else {
                        return core::ptr::null_mut();
                    };
                    // SAFETY: `end` is a state allocated just above.
                    unsafe { (*end).out = skip };
                    patch(e.out, zend);
                    stack.push(s, list1(out_edge(skip)));
                } else {
                    patch(e.out, end);
                    stack.push(s, list1(out_edge(end)));
                    if before {
                        // With no explicit `\@123<=` width, work out how far
                        // back the pattern could possibly start.
                        let width = if operand <= 0 {
                            nfa_max_width(e.start, 0)
                        } else {
                            operand
                        };
                        // SAFETY: `s` is a state allocated just above.
                        unsafe { (*s).val = width };
                    }
                }
            }
            // A bracket: an opening state, the operand, and the closing
            // state that pairs with it.
            Ok(open)
                if open == NfaOp::Composing || open == NfaOp::Nopen || open.opens_capture() =>
            {
                let mclose = closing_bracket(open);
                // An empty stack means an empty group, `\(\)`.
                let inner = if stack.frags.is_empty() {
                    None
                } else {
                    match stack.pop() {
                        Some(e) => Some(e),
                        None => return core::ptr::null_mut(),
                    }
                };
                let start = inner.map_or(core::ptr::null_mut(), |e| e.start);
                let Some(s) = state(item, start, core::ptr::null_mut()) else {
                    return core::ptr::null_mut();
                };
                let Some(s1) = state(mclose.code(), core::ptr::null_mut(), core::ptr::null_mut())
                else {
                    return core::ptr::null_mut();
                };
                match inner {
                    None => patch(list1(out_edge(s)), s1),
                    Some(e) => {
                        patch(e.out, s1);
                        if open == NfaOp::Composing {
                            // The matcher reaches the group's end through
                            // `out1`, so that edge has to be wired too.
                            patch(list1(out1_edge(s)), s1);
                        }
                    }
                }
                stack.push(s, list1(out_edge(s1)));
            }
            // A back-reference matches a run whose length is only known at
            // match time, so an `NFA_SKIP` follows it to consume the rest.
            Ok(reference) if reference.is_reference() => {
                let Some(s) = state(item, core::ptr::null_mut(), core::ptr::null_mut()) else {
                    return core::ptr::null_mut();
                };
                let Some(s1) = state(
                    NfaOp::Skip.code(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) else {
                    return core::ptr::null_mut();
                };
                patch(list1(out_edge(s)), s1);
                stack.push(s, list1(out_edge(s1)));
            }
            // Everything else — a literal character, a class, a boundary, a
            // position assertion — is one state that stands alone. The
            // assertions keep their operand in `val`.
            _ => {
                let Some(s) = state(item, core::ptr::null_mut(), core::ptr::null_mut()) else {
                    return core::ptr::null_mut();
                };
                if operand != 0 {
                    // SAFETY: `s` is a state allocated just above.
                    unsafe { (*s).val = operand };
                }
                stack.push(s, list1(out_edge(s)));
            }
        }
        i += 1;
    }

    if counting {
        // One more for the accepting state added below.
        nstate.set(nstate.get() + 1);
        return core::ptr::null_mut();
    }

    let Some(e) = stack.pop() else {
        return core::ptr::null_mut();
    };
    if !stack.frags.is_empty() {
        semsg!(
            "E875: (NFA regexp) (While converting from postfix to NFA),too many states left on stack"
        );
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }
    if istate.get() >= nstate.get() {
        semsg!("E876: (NFA regexp) Not enough space to store the whole NFA ");
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }
    // The accepting state, taken by hand rather than through `state`: it
    // must have id 0, which is how the matcher recognises it.
    // SAFETY: `istate` is below `nstate`, checked just above.
    let matchstate = unsafe { state_ptr.get().offset(istate.get() as isize) };
    istate.set(istate.get() + 1);
    unsafe { (*matchstate).c = NfaOp::Match.code() };
    unsafe { (*matchstate).out = core::ptr::null_mut() };
    unsafe { (*matchstate).out1 = core::ptr::null_mut() };
    unsafe { (*matchstate).id = 0 };
    patch(e.out, matchstate);
    e.start
}

/// How many states one program item needs.
fn count_for(item: c_int, operand: c_int) -> c_int {
    match NfaOp::try_from(item) {
        // Wire-ups: no state of their own.
        Ok(NfaOp::Concat | NfaOp::Range) => 0,
        Ok(NfaOp::OptChars) => operand,
        // Two states for a bracket pair, a back-reference plus its skip, and
        // a lookaround's start and end — four for `\@>`, which also needs
        // the skip and the `\ze` marker.
        Ok(NfaOp::PrevAtomLikePattern) => 4,
        Ok(
            NfaOp::PrevAtomNoWidth
            | NfaOp::PrevAtomNoWidthNeg
            | NfaOp::PrevAtomJustBefore
            | NfaOp::PrevAtomJustBeforeNeg
            | NfaOp::Composing
            | NfaOp::Nopen,
        ) => 2,
        Ok(op) if op.opens_capture() || op.is_reference() => 2,
        _ => 1,
    }
}

/// The `NFA_START_*`/`NFA_END_*` pair one lookaround operator compiles to.
fn lookaround_states(op: NfaOp) -> (NfaOp, NfaOp) {
    match op {
        NfaOp::PrevAtomNoWidth => (NfaOp::StartInvisible, NfaOp::EndInvisible),
        NfaOp::PrevAtomNoWidthNeg => (NfaOp::StartInvisibleNeg, NfaOp::EndInvisibleNeg),
        NfaOp::PrevAtomJustBefore => (NfaOp::StartInvisibleBefore, NfaOp::EndInvisible),
        NfaOp::PrevAtomJustBeforeNeg => (NfaOp::StartInvisibleBeforeNeg, NfaOp::EndInvisibleNeg),
        _ => (NfaOp::StartPattern, NfaOp::EndPattern),
    }
}

/// The closing opcode that pairs with an opening bracket.
fn closing_bracket(open: NfaOp) -> NfaOp {
    match open {
        NfaOp::Nopen => NfaOp::Nclose,
        NfaOp::Composing => NfaOp::EndComposing,
        // Both bracket families close with the marker at the same position
        // in the closing run -- which for the numbered groups is upstream's
        // "add `NSUBEXP`".
        _ => match open.index_in(&NfaOp::ZOPEN) {
            Some(i) => NfaOp::ZCLOSE[i],
            None => NfaOp::MCLOSE[open.index_in(&NfaOp::MOPEN).expect("an opening bracket")],
        },
    }
}

/// Decide, for each lookaround in the finished machine, whether to run it as
/// soon as it is reached or to postpone it until the rest of the pattern has
/// been tried — the "postponed invisible match" the matcher carries around.
///
/// Postponing pays when the lookaround is expensive and what follows it is
/// cheap, because the cheap test rejects most positions first.
pub(crate) fn nfa_postprocess(prog: *mut nfa_regprog_T) {
    // SAFETY: `prog` is a program this module just built, with `nstate`
    // states inline.
    let states = unsafe { &raw mut (*prog).state } as *mut nfa_state_T;
    for i in 0..unsafe { (*prog).nstate } {
        let s = unsafe { states.offset(i as isize) };
        let c = NfaOp::try_from(unsafe { (*s).c });
        if !matches!(
            c,
            Ok(NfaOp::StartInvisible
                | NfaOp::StartInvisibleNeg
                | NfaOp::StartInvisibleBefore
                | NfaOp::StartInvisibleBeforeNeg)
        ) {
            continue;
        }
        let follows = unsafe { (*(*s).out1).out };
        let directly = if match_follows(follows, 0) {
            // The pattern ends right after it, so there is nothing
            // cheaper to try first.
            true
        } else {
            let ch_invisible = failure_chance(unsafe { (*s).out }, 0);
            let ch_follows = failure_chance(follows, 0);
            if matches!(
                c,
                Ok(NfaOp::StartInvisibleBefore | NfaOp::StartInvisibleBeforeNeg)
            ) {
                // A lookbehind of unknown width has to be retried from
                // every start position, so postpone it unless what
                // follows is very much cheaper.
                if unsafe { (*s).val } <= 0 && ch_follows > 0 {
                    false
                } else {
                    ch_follows * 10 < ch_invisible
                }
            } else {
                ch_follows < ch_invisible
            }
        };
        if directly {
            // The `_FIRST` variant of each opcode is the next one along.
            unsafe { (*s).c += 1 };
        }
    }
}
