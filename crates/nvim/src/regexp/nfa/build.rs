//! Postfix form to state machine.
//!
//! Thompson's construction: the postfix program is run on a stack of
//! *fragments*, each an entry state plus the list of `out` pointers that are
//! still dangling. An operand pushes a one-state fragment; an operator pops
//! its operands, wires them together and pushes the result. The last
//! fragment left is the machine.
//!
//! It runs twice per compile. The first pass only counts states, which is
//! what the program's thread lists are sized by (and so where
//! 'maxmempattern' bites); the second builds them.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::program::{NfaState, StateId, States};
use crate::regexp::NfaOp;
use crate::regexp::NfaStates;
use core::ffi::c_int;

use super::run::failure_chance;
use super::sub::match_follows;
use crate::mbyte::utf_char2len;
use crate::regexp::NfaRegProg;
use crate::regexp::state::rc_did_emsg;
use crate::semsg;
use crate::types::MB_MAXBYTES;

/// How far the width walk follows a `NFA_SPLIT` before giving up.
const MAX_DEPTH: c_int = 4;

/// One unset edge of a half-built machine: a state's `out` or `out1`, seen
/// as a link in the list of edges still waiting to be patched.
///
/// The list has no storage of its own: each unset edge holds the [`Slot`]
/// of the *next* unset edge until [`patch`] overwrites it with the state
/// the edge goes to, as upstream's pointer-to-pointer list did. The end of
/// the list is [`Slot::END`], which is the raw value of
/// [`StateId::NONE`] — so an edge patched to "nowhere" also reads as the
/// end of a list, which the `\%[` construction below relies on exactly as
/// upstream's did on `NULL`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Slot(u32);

impl Slot {
    const END: Slot = Slot(u32::MAX);

    /// `state`'s `out` edge.
    fn out(state: StateId) -> Slot {
        Slot(state.raw() << 1)
    }

    /// `state`'s `out1` edge.
    fn out1(state: StateId) -> Slot {
        Slot((state.raw() << 1) | 1)
    }

    /// The edge itself.
    fn edge(self, states: &mut [NfaState]) -> &mut StateId {
        let state = &mut states[(self.0 >> 1) as usize];
        if self.0 & 1 == 0 {
            &mut state.out
        } else {
            &mut state.out1
        }
    }

    /// The next edge in the list this one is in.
    fn next(self, states: &mut [NfaState]) -> Slot {
        Slot(self.edge(states).raw())
    }

    /// Link this edge to `next`, keeping it unpatched.
    fn set_next(self, states: &mut [NfaState], next: Slot) {
        *self.edge(states) = StateId::from_raw(next.0);
    }
}

/// A partly built machine: where it starts, and every edge out of it that
/// still has to be told where to go.
#[derive(Clone, Copy)]
struct Frag {
    start: StateId,
    out: Slot,
}

/// Take the next state of the program.
///
/// `None` once the counting pass's estimate is used up, which the callers
/// treat as "give up on this pattern" — silently, as upstream does.
fn state(states: &mut NfaStates, c: c_int, out: StateId, out1: StateId) -> Option<StateId> {
    if states.states.len() >= usize::try_from(states.count).unwrap_or(0) {
        return None;
    }
    let id = StateId::new(states.states.len());
    states.states.push(NfaState::new(c, out, out1));
    Some(id)
}

/// A one-element patch list over the edge `slot`.
fn list1(states: &mut NfaStates, slot: Slot) -> Slot {
    slot.set_next(&mut states.states, Slot::END);
    slot
}

/// Point every edge in `list` at `state`.
fn patch(states: &mut NfaStates, list: Slot, state: StateId) {
    let mut node = list;
    while node != Slot::END {
        let next = node.next(&mut states.states);
        *node.edge(&mut states.states) = state;
        node = next;
    }
}

/// Concatenate two patch lists, returning the first.
fn append(states: &mut NfaStates, first: Slot, second: Slot) -> Slot {
    // `first` is never empty where this is called.
    let mut last = first;
    loop {
        let next = last.next(&mut states.states);
        if next == Slot::END {
            break;
        }
        last = next;
    }
    last.set_next(&mut states.states, second);
    first
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
    fn push(&mut self, start: StateId, out: Slot) {
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
    /// Only add up the states the program needs, into `count`.
    Count,
    /// Build into the program's state array.
    Build,
}

/// The widest match the machine from `startstate` can make, or -1 when that
/// cannot be bounded. Backs `\@<=`, which has to know how far back to start.
fn nfa_max_width(states: &[NfaState], startstate: StateId, depth: c_int) -> c_int {
    if depth > MAX_DEPTH {
        return -1;
    }
    let mut len = 0;
    let mut state = startstate;
    let op = |s: StateId| states[s.index()].c;
    let out_of = |s: StateId| states[s.index()].out;
    let out1_of = |s: StateId| states[s.index()].out1;
    while !state.is_none() {
        let code = op(state);
        match NfaOp::try_from(code) {
            // The end of the lookbehind's own pattern.
            Ok(NfaOp::EndInvisible | NfaOp::EndInvisibleNeg) => return len,
            Ok(NfaOp::Split) => {
                let l = nfa_max_width(states, out_of(state), depth + 1);
                let r = nfa_max_width(states, out1_of(state), depth + 1);
                if l < 0 || r < 0 {
                    return -1;
                }
                return len + l.max(r);
            }
            // Any character, so as wide as a character gets. A
            // collection continues past its `NFA_END_COLL`, which is
            // where `out1` points.
            Ok(c @ (NfaOp::Any | NfaOp::StartColl | NfaOp::StartNegColl)) => {
                len += c_int::try_from(MB_MAXBYTES).expect("`MB_MAXBYTES` is 21");
                if c != NfaOp::Any {
                    if out1_of(state).is_none() || out_of(out1_of(state)).is_none() {
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
/// [`Pass::Count`] only adds up `count` and returns no state; [`Pass::Build`]
/// returns the machine's entry state, or none once it has said why not.
pub(crate) fn post2nfa(states: &mut NfaStates, items: &[c_int], pass: Pass) -> StateId {
    let counting = pass == Pass::Count;
    let cap = if counting {
        0
    } else {
        usize::try_from(states.count).expect("the state count is never negative") + 1
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
            states.count += count_for(item, operand);
            i += 1;
            continue;
        }

        match NfaOp::try_from(item) {
            Ok(NfaOp::Concat) => {
                // Two fragments run one after the other: the first's loose
                // ends go to the second's start.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return StateId::NONE;
                };
                patch(states, e1.out, e2.start);
                stack.push(e1.start, e2.out);
            }
            Ok(NfaOp::Or) => {
                // A choice between two fragments.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return StateId::NONE;
                };
                let Some(s) = state(states, NfaOp::Split.code(), e1.start, e2.start) else {
                    return StateId::NONE;
                };
                stack.push(s, append(states, e1.out, e2.out));
            }
            // A repeat is a split whose taken branch loops back to it; `\?`
            // is the same split without the loop. Which of the two edges is
            // the body and which the exit decides whether it is greedy.
            Ok(
                repeat
                @ (NfaOp::Star | NfaOp::StarNongreedy | NfaOp::Quest | NfaOp::QuestNongreedy),
            ) => {
                let Some(e) = stack.pop() else {
                    return StateId::NONE;
                };
                let greedy = matches!(repeat, NfaOp::Star | NfaOp::Quest);
                let (out, out1) = if greedy {
                    (e.start, StateId::NONE)
                } else {
                    (StateId::NONE, e.start)
                };
                let Some(s) = state(states, NfaOp::Split.code(), out, out1) else {
                    return StateId::NONE;
                };
                let exit = list1(states, if greedy { Slot::out1(s) } else { Slot::out(s) });
                if matches!(repeat, NfaOp::Star | NfaOp::StarNongreedy) {
                    patch(states, e.out, s);
                    stack.push(s, exit);
                } else {
                    stack.push(s, append(states, e.out, exit));
                }
            }
            Ok(NfaOp::EndColl | NfaOp::EndNegColl) => {
                // The collection's members are already chained; close them
                // off with the state the matcher stops at, and point the
                // opening state's `out1` at it so a walk can skip over the
                // whole collection.
                let Some(e) = stack.pop() else {
                    return StateId::NONE;
                };
                let Some(s) = state(states, NfaOp::EndColl.code(), StateId::NONE, StateId::NONE)
                else {
                    return StateId::NONE;
                };
                patch(states, e.out, s);
                states.states[e.start.index()].out1 = s;
                stack.push(e.start, list1(states, Slot::out(s)));
            }
            Ok(NfaOp::Range) => {
                // Two members become a range: their character values move
                // into `val` and the opcodes say which end each one is.
                let (Some(e2), Some(e1)) = (stack.pop(), stack.pop()) else {
                    return StateId::NONE;
                };
                // Both fragments are single member states.
                for (end, op) in [(e2.start, NfaOp::RangeMax), (e1.start, NfaOp::RangeMin)] {
                    let member = &mut states.states[end.index()];
                    member.val = member.c;
                    member.c = op.code();
                }
                patch(states, e1.out, e2.start);
                stack.push(e1.start, e2.out);
            }
            Ok(NfaOp::OptChars) => {
                // `\%[abc]`: `operand` members, each of which becomes a
                // split that can leave the sequence, so every prefix
                // matches.
                let mut n = operand;
                let mut s = StateId::NONE;
                let mut s1 = StateId::NONE;
                let mut first_out = Slot::END;
                while n > 0 {
                    n -= 1;
                    let Some(e) = stack.pop() else {
                        return StateId::NONE;
                    };
                    let Some(new) = state(states, NfaOp::Split.code(), e.start, StateId::NONE)
                    else {
                        return StateId::NONE;
                    };
                    if first_out == Slot::END {
                        first_out = e.out;
                    }
                    patch(states, e.out, s1);
                    let exit = list1(states, Slot::out1(new));
                    append(states, first_out, exit);
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
                    return StateId::NONE;
                };
                let Some(end) = state(states, end_state.code(), StateId::NONE, StateId::NONE)
                else {
                    return StateId::NONE;
                };
                let Some(s) = state(states, start_state.code(), e.start, end) else {
                    return StateId::NONE;
                };
                if pattern {
                    // `\@>` keeps what it matched, so the sub-match's end is
                    // recorded and the outer match resumes past it.
                    let Some(skip) =
                        state(states, NfaOp::Skip.code(), StateId::NONE, StateId::NONE)
                    else {
                        return StateId::NONE;
                    };
                    let Some(zend) = state(states, NfaOp::Zend.code(), end, StateId::NONE) else {
                        return StateId::NONE;
                    };
                    states.states[end.index()].out = skip;
                    patch(states, e.out, zend);
                    stack.push(s, list1(states, Slot::out(skip)));
                } else {
                    patch(states, e.out, end);
                    stack.push(s, list1(states, Slot::out(end)));
                    if before {
                        // With no explicit `\@123<=` width, work out how far
                        // back the pattern could possibly start.
                        let width = if operand <= 0 {
                            nfa_max_width(&states.states, e.start, 0)
                        } else {
                            operand
                        };
                        states.states[s.index()].val = width;
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
                        None => return StateId::NONE,
                    }
                };
                let start = inner.map_or(StateId::NONE, |e| e.start);
                let Some(s) = state(states, item, start, StateId::NONE) else {
                    return StateId::NONE;
                };
                let Some(s1) = state(states, mclose.code(), StateId::NONE, StateId::NONE) else {
                    return StateId::NONE;
                };
                match inner {
                    None => {
                        let edge = list1(states, Slot::out(s));
                        patch(states, edge, s1);
                    }
                    Some(e) => {
                        patch(states, e.out, s1);
                        if open == NfaOp::Composing {
                            // The matcher reaches the group's end through
                            // `out1`, so that edge has to be wired too.
                            let edge = list1(states, Slot::out1(s));
                            patch(states, edge, s1);
                        }
                    }
                }
                stack.push(s, list1(states, Slot::out(s1)));
            }
            // A back-reference matches a run whose length is only known at
            // match time, so an `NFA_SKIP` follows it to consume the rest.
            Ok(reference) if reference.is_reference() => {
                let Some(s) = state(states, item, StateId::NONE, StateId::NONE) else {
                    return StateId::NONE;
                };
                let Some(s1) = state(states, NfaOp::Skip.code(), StateId::NONE, StateId::NONE)
                else {
                    return StateId::NONE;
                };
                let edge = list1(states, Slot::out(s));
                patch(states, edge, s1);
                stack.push(s, list1(states, Slot::out(s1)));
            }
            // Everything else — a literal character, a class, a boundary, a
            // position assertion — is one state that stands alone. The
            // assertions keep their operand in `val`.
            _ => {
                let Some(s) = state(states, item, StateId::NONE, StateId::NONE) else {
                    return StateId::NONE;
                };
                if operand != 0 {
                    states.states[s.index()].val = operand;
                }
                stack.push(s, list1(states, Slot::out(s)));
            }
        }
        i += 1;
    }

    if counting {
        // One more for the accepting state added below.
        states.count += 1;
        return StateId::NONE;
    }

    let Some(e) = stack.pop() else {
        return StateId::NONE;
    };
    if !stack.frags.is_empty() {
        semsg!(
            "E875: (NFA regexp) (While converting from postfix to NFA),too many states left on stack"
        );
        rc_did_emsg.set(true);
        return StateId::NONE;
    }
    if states.states.len() >= usize::try_from(states.count).unwrap_or(0) {
        semsg!("E876: (NFA regexp) Not enough space to store the whole NFA ");
        rc_did_emsg.set(true);
        return StateId::NONE;
    }
    // The accepting state, taken by hand rather than through `state`, whose
    // refusal would have been silent.
    let matchstate = StateId::new(states.states.len());
    states.states.push(NfaState::new(
        NfaOp::Match.code(),
        StateId::NONE,
        StateId::NONE,
    ));
    patch(states, e.out, matchstate);
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
pub(crate) fn nfa_postprocess(prog: &mut NfaRegProg) {
    for i in 0..prog.states.len() {
        let states = &prog.states;
        let s = &states[i];
        let c = NfaOp::try_from(s.c);
        if !matches!(
            c,
            Ok(NfaOp::StartInvisible
                | NfaOp::StartInvisibleNeg
                | NfaOp::StartInvisibleBefore
                | NfaOp::StartInvisibleBeforeNeg)
        ) {
            continue;
        }
        let follows = states[s.out1.index()].out;
        let directly = if match_follows(States::new(states), follows, 0) {
            // The pattern ends right after it, so there is nothing
            // cheaper to try first.
            true
        } else {
            let ch_invisible = failure_chance(states, s.out, 0);
            let ch_follows = failure_chance(states, follows, 0);
            if matches!(
                c,
                Ok(NfaOp::StartInvisibleBefore | NfaOp::StartInvisibleBeforeNeg)
            ) {
                // A lookbehind of unknown width has to be retried from
                // every start position, so postpone it unless what
                // follows is very much cheaper.
                if s.val <= 0 && ch_follows > 0 {
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
            prog.states[i].c += 1;
        }
    }
}
