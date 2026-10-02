//! What a `:try` is in the middle of.
//!
//! The exceptions themselves (`EXCEPTIONS`, their one owner), the one being
//! propagated (`current_exception`, `did_throw`, `need_rethrow`), how deep
//! the `:try` nesting goes (`trylevel`), the errors turned into exceptions
//! along the way (`msg_lists`, `suppress_errthrow`) and the `:catch` history
//! a `:finally` may resume (`caught_stack`, `force_abort`). The condition
//! stack's own two flags (`did_endif`, `check_cstack`) sit with them: they
//! are what tells `do_cmdline` that a `:endif` closed something it did not
//! open.
//!
//! One record behind one cell, reached a field at a time through selectors
//! that keep upstream's names. Nothing holds a borrow of it across a call:
//! a thrown exception's report runs messages, and messages reach Lua.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The selectors keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::state_record;
use crate::id_table::IdTable;
use crate::types::{ErrorMsgs, ExcId, Exception};
use core::ffi::c_int;

state_record! {
    /// The exception machinery's state: upstream's `ex_eval.c` globals.
    pub(crate) struct ExEvalState in EX_EVAL as ExEvalField;

    pub(crate) did_endif: bool = false;
    /// Every exception alive: being thrown, pending in a `:finally`,
    /// caught, or set aside by a nested command line. Everything else names
    /// one by id.
    pub(crate) EXCEPTIONS: IdTable<Exception> = IdTable::new();
    /// The exception being thrown.
    pub(crate) current_exception: Option<ExcId> = None;
    pub(crate) did_throw: bool = false;
    pub(crate) need_rethrow: bool = false;
    pub(crate) check_cstack: bool = false;
    pub(crate) trylevel: c_int = 0;
    pub(crate) force_abort: bool = false;
    /// The error messages of each running `do_cmdline` and API try bracket;
    /// errors go to the innermost. Upstream's `msg_list`, a pointer to the
    /// innermost one's list head in its own frame, null when there is none.
    pub(crate) msg_lists: MsgLists = MsgLists::new();
    pub(crate) suppress_errthrow: bool = false;
    /// The exceptions caught by the active catch clauses, innermost last.
    pub(crate) caught_stack: Vec<ExcId> = Vec::new();
}

/// One error-message list per running command line or API try bracket,
/// innermost last.
///
/// Opening and closing one is a counter: every API call and command line
/// does both, and nearly all of them collect nothing, so a list only comes
/// into being when an error is filed in it.
pub(crate) struct MsgLists {
    /// How many are open.
    depth: usize,
    /// The ones that have collected something, outermost first; never more
    /// than `depth`, and the innermost open one is at `depth - 1` when it is
    /// here at all.
    lists: Vec<ErrorMsgs>,
}

impl MsgLists {
    const fn new() -> MsgLists {
        MsgLists {
            depth: 0,
            lists: Vec::new(),
        }
    }

    /// Whether no list is open.
    pub(crate) fn is_empty(&self) -> bool {
        self.depth == 0
    }

    /// Open a list.
    pub(crate) fn push(&mut self) {
        self.depth += 1;
    }

    /// Close the innermost list, dropping what it held: messages only, so
    /// nothing a drop runs can reach back here.
    pub(crate) fn pop(&mut self) {
        assert!(self.depth > 0, "a message list to end");
        self.depth -= 1;
        if self.lists.len() > self.depth {
            self.drop_closed();
        }
    }

    /// Drop the lists past the open ones -- out of line, so that closing a
    /// list that collected nothing, which is nearly every close, stays a
    /// counter.
    #[cold]
    #[inline(never)]
    fn drop_closed(&mut self) {
        self.lists.truncate(self.depth);
    }

    /// The innermost open list, if it has collected anything.
    pub(crate) fn innermost(&self) -> Option<&ErrorMsgs> {
        self.depth.checked_sub(1).and_then(|at| self.lists.get(at))
    }

    /// The innermost open list, brought into being; `None` when none is open.
    pub(crate) fn innermost_mut(&mut self) -> Option<&mut ErrorMsgs> {
        let at = self.depth.checked_sub(1)?;
        if self.lists.len() <= at {
            self.lists.resize_with(at + 1, ErrorMsgs::default);
        }
        self.lists.get_mut(at)
    }
}
