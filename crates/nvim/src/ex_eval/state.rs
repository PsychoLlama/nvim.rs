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
use crate::slot_table::SlotTable;
use crate::types::{ErrorMsgs, ExcId, Exception};
use core::ffi::c_int;

state_record! {
    /// The exception machinery's state: upstream's `ex_eval.c` globals.
    pub(crate) struct ExEvalState in EX_EVAL as ExEvalField;

    pub(crate) did_endif: bool = false;
    /// Every exception alive: being thrown, pending in a `:finally`,
    /// caught, or set aside by a nested command line. Everything else names
    /// one by id.
    pub(crate) EXCEPTIONS: SlotTable<Exception> = SlotTable::new();
    /// The exception being thrown.
    pub(crate) current_exception: Option<ExcId> = None;
    pub(crate) did_throw: bool = false;
    pub(crate) need_rethrow: bool = false;
    pub(crate) check_cstack: bool = false;
    pub(crate) trylevel: c_int = 0;
    pub(crate) force_abort: bool = false;
    /// The error messages of each running `do_cmdline` and API try bracket,
    /// innermost last; errors go to the innermost. Upstream's `msg_list`, a
    /// pointer to the innermost one's list head in its own frame, null when
    /// the stack is empty.
    pub(crate) msg_lists: Vec<ErrorMsgs> = Vec::new();
    pub(crate) suppress_errthrow: bool = false;
    /// The exceptions caught by the active catch clauses, innermost last.
    pub(crate) caught_stack: Vec<ExcId> = Vec::new();
}
