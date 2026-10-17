//! The compiled machine: its states, addressed by index, and what the
//! compiler worked out about the pattern.
//!
//! Upstream allocates the program as one block — the head, then the states
//! inline — and links the states by pointer. Here the states are a `Vec`
//! the program owns and a link is a [`StateId`], the index of the state it
//! goes to. Nothing else changes shape: a state is still its opcode, its two
//! continuations and its operand, and the matcher still marks a state with
//! the list generation it was last added to so that it is added once.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::Cell;
use core::ffi::{c_int, c_uint};
use std::ffi::CString;

use super::list::SpareLists;
use crate::types::RegEngine;

/// A state of the program, by its index in [`NfaRegProg::states`].
///
/// [`StateId::NONE`] is upstream's null `out`: an edge that goes nowhere,
/// which only the end states and the half-built machine have.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct StateId(u32);

impl StateId {
    /// No state.
    pub(crate) const NONE: StateId = StateId(u32::MAX);

    /// The state at `index`.
    #[inline(always)]
    pub(crate) fn new(index: usize) -> StateId {
        StateId(u32::try_from(index).expect("a program has fewer than 2^32 states"))
    }

    /// Does this name no state?
    #[inline(always)]
    pub(crate) fn is_none(self) -> bool {
        self == StateId::NONE
    }

    /// Its index into the program's states.
    #[inline(always)]
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }

    /// The raw value, for the builder's patch list, which keeps its links
    /// in edges that are not patched yet.
    #[inline(always)]
    pub(crate) fn raw(self) -> u32 {
        self.0
    }

    /// [`StateId::raw`]'s inverse.
    #[inline(always)]
    pub(crate) fn from_raw(raw: u32) -> StateId {
        StateId(raw)
    }
}

/// One state of the machine.
pub(crate) struct NfaState {
    /// The opcode (negative), or the literal character the state matches.
    pub(crate) c: c_int,
    /// Where a thread goes on from here.
    pub(crate) out: StateId,
    /// The second continuation, which the branching opcodes have — and the
    /// collections and lookarounds, whose end state it names.
    pub(crate) out1: StateId,
    /// The operand: a range end, a position assertion's number, a
    /// lookbehind's width.
    pub(crate) val: c_int,
    /// The last list generation the state was put on, one per nesting
    /// level of lookaround. A cell: the matcher writes it through the shared
    /// program it runs.
    pub(crate) lastlist: [Cell<c_int>; 2],
}

impl NfaState {
    /// A state with the opcode or character `c`, on no list.
    pub(crate) fn new(c: c_int, out: StateId, out1: StateId) -> NfaState {
        NfaState {
            c,
            out,
            out1,
            val: 0,
            lastlist: [Cell::new(0), Cell::new(0)],
        }
    }
}

/// An NFA program: the [`RegProg`](crate::types::RegProg) head every program
/// opens with, the states, and the shortcuts the compiler found.
#[repr(C)]
pub(crate) struct NfaRegProg {
    pub(crate) engine: *mut RegEngine,
    pub(crate) regflags: c_uint,
    pub(crate) re_engine: c_uint,
    pub(crate) re_flags: c_uint,
    pub(crate) re_in_use: bool,
    /// The machine's entry.
    pub(crate) start: StateId,
    /// Every match starts at the start of a line.
    pub(crate) reganch: bool,
    /// The character every match starts with, or `NUL`.
    pub(crate) regstart: c_int,
    /// The literal text after `regstart` the whole pattern is, NUL
    /// included, when it is nothing else.
    pub(crate) match_text: Option<Vec<u8>>,
    pub(crate) has_zend: c_int,
    pub(crate) has_backref: c_int,
    pub(crate) reghasz: c_int,
    /// The pattern, for the recompile a `NFA_TOO_EXPENSIVE` asks for.
    pub(crate) pattern: CString,
    pub(crate) nsubexp: c_int,
    /// How many states the counting pass asked for, which sizes the thread
    /// lists — and so decides when 'maxmempattern' is reached.
    pub(crate) nstate: c_int,
    pub(crate) states: Vec<NfaState>,
    /// Thread-list storage between matches.
    pub(crate) spare: SpareLists,
}

impl NfaRegProg {
    /// The states, as the matcher reads them.
    #[inline(always)]
    pub(crate) fn nfa(&self) -> States<'_> {
        States::new(&self.states)
    }
}

/// A program's states, by shared borrow: `Copy`, and the slice it holds
/// travels in registers rather than being reloaded from the program at
/// every state the walk visits.
#[derive(Clone, Copy)]
pub(crate) struct States<'p>(&'p [NfaState]);

impl<'p> States<'p> {
    /// The states `states` holds.
    #[inline(always)]
    pub(crate) fn new(states: &'p [NfaState]) -> States<'p> {
        States(states)
    }

    /// The state `id` names.
    #[inline(always)]
    pub(crate) fn state(self, id: StateId) -> &'p NfaState {
        &self.0[id.index()]
    }

    /// Its opcode or character.
    #[inline(always)]
    pub(crate) fn op(self, id: StateId) -> c_int {
        self.state(id).c
    }

    /// Where it continues.
    #[inline(always)]
    pub(crate) fn out(self, id: StateId) -> StateId {
        self.state(id).out
    }

    /// Its operand.
    #[inline(always)]
    pub(crate) fn val(self, id: StateId) -> c_int {
        self.state(id).val
    }
}
