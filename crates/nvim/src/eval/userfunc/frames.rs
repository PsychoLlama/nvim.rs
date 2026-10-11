//! Who owns a [`FuncCall`], and which one is running.
//!
//! Upstream keeps three raw globals: `current_funccal` (the call in
//! progress, whose `fc_caller` chain is the call stack), `previous_funccal`
//! (the funccalls a closure, an escaped `l:` or a returned `a:000` kept alive
//! after their call returned, threaded through the same `fc_caller` link) and
//! `funccal_stack` (a list of `funccal_entry_T`s in the frames of whoever set
//! the call stack aside to run an autocommand or a callback).
//!
//! Here every funccall, running or parked, is **shared** (`Rc`) and filed
//! in one [`RcTable`], and the three globals are [`FcId`]s into it. The
//! table records beside each funccall the call it was made from, which is
//! upstream's `fc_caller` chain; the parked funccalls are a `Vec` of ids,
//! newest last, and the set-aside call stacks a `Vec` of the ids that were
//! current, innermost last. Whoever is using a funccall across a call that
//! can run user code -- the call running it, a closure body looking up a
//! captured variable -- holds an `Rc` of its own, so a funccall the table
//! lets go of lives until they are done.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::rc::Rc;

use crate::global_cell::GlobalCell;
use crate::id_table::RcTable;
use crate::types::{FcId, FuncCall};

/// The funccalls, each beside its caller, and the three ids upstream kept as
/// raw globals.
struct FuncCalls {
    table: RcTable<FuncCall, Option<FcId>>,
    /// The call in progress: upstream's `current_funccal`.
    current: Option<FcId>,
    /// `current`'s funccall: every variable lookup asks for it, and a table
    /// lookup per ask showed in `evalbench`. Kept in step with `current` by
    /// the only two writers, [`set_current_fc`] and [`CallStackAside`].
    current_frame: Option<Rc<FuncCall>>,
    /// Funccalls kept beyond their call, newest last: upstream's
    /// `previous_funccal` list.
    parked: Vec<FcId>,
    /// The call in progress at each [`CallStackAside`], innermost last:
    /// upstream's `funccal_stack`.
    aside: Vec<Option<FcId>>,
    /// Funccalls that are done, their scopes emptied, for the next calls to
    /// be built in: a call then allocates nothing.
    spare: Vec<Rc<FuncCall>>,
}

impl FuncCalls {
    /// Make `id` the call in progress, answering the reference to the one
    /// before -- for the caller to drop once the table is no longer
    /// borrowed, since a funccall's last reference frees its scopes, and
    /// that can reach back in here.
    #[must_use = "the old reference is dropped outside the borrow"]
    fn make_current(&mut self, id: Option<FcId>) -> Option<Rc<FuncCall>> {
        self.current = id;
        core::mem::replace(
            &mut self.current_frame,
            id.map(|id| self.table.get(id).clone()),
        )
    }
}

static FUNC_CALLS: GlobalCell<FuncCalls> = GlobalCell::new(FuncCalls {
    table: RcTable::new(),
    current: None,
    current_frame: None,
    parked: Vec::new(),
    aside: Vec::new(),
    spare: Vec::new(),
});

/// How many done funccalls are kept for reuse: enough for the recursion
/// a script commonly reaches, little enough not to matter after a deep one.
const SPARE_SCOPES: usize = 32;

impl FcId {
    /// The funccall `self` names.
    ///
    /// # Panics
    /// When that funccall has left the table.
    pub(crate) fn funccall(self) -> Rc<FuncCall> {
        FUNC_CALLS.with(|calls| calls.table.get(self).clone())
    }

    /// The funccall `self` names, or `None` once it has left the table.
    pub(crate) fn try_funccall(self) -> Option<Rc<FuncCall>> {
        FUNC_CALLS.with(|calls| calls.table.try_get(self).cloned())
    }

    /// The call `self` was made from: upstream's `fc_caller`.
    pub(crate) fn caller(self) -> Option<FcId> {
        FUNC_CALLS.with(|calls| *calls.table.meta(self))
    }
}

/// `top` and every call below it, innermost first: one call stack.
pub(crate) fn call_chain(top: Option<FcId>) -> Vec<FcId> {
    FUNC_CALLS.with(|calls| ::core::iter::successors(top, |&id| *calls.table.meta(id)).collect())
}

/// File the funccall `make` builds -- told its id first -- as a call made
/// from `caller`.
pub(crate) fn adopt_funccal(
    caller: Option<FcId>,
    make: impl FnOnce(FcId) -> Rc<FuncCall>,
) -> Rc<FuncCall> {
    let mut made = None;
    FUNC_CALLS.with_mut(|calls| {
        calls.table.insert_with(caller, |id| {
            let frame = make(id);
            made = Some(frame.clone());
            frame
        });
    });
    made.expect("the table built the funccall")
}

/// Take `id` out of the table, answering the table's reference.
pub(crate) fn release_funccal(id: FcId) -> Rc<FuncCall> {
    FUNC_CALLS.with_mut(|calls| calls.table.remove(id))
}

/// A done funccall to build the next call in, if one is spare.
pub(crate) fn take_spare_funccal() -> Option<Rc<FuncCall>> {
    FUNC_CALLS.with_mut(|calls| calls.spare.pop())
}

/// Keep `frame`, done and emptied, for a later call; dropped when enough
/// are kept.
pub(crate) fn give_spare_funccal(frame: Rc<FuncCall>) {
    let extra = FUNC_CALLS.with_mut(|calls| {
        if calls.spare.len() < SPARE_SCOPES {
            calls.spare.push(frame);
            None
        } else {
            Some(frame)
        }
    });
    drop(extra);
}

/// The call in progress.
pub(crate) fn current_fc() -> Option<Rc<FuncCall>> {
    FUNC_CALLS.with(|calls| calls.current_frame.clone())
}

/// Run `f` on the call in progress, without taking a reference: for the
/// variable lookups, which ask on every name and run no user code.
pub(crate) fn with_current_fc<R>(f: impl FnOnce(Option<&FuncCall>) -> R) -> R {
    FUNC_CALLS.with(|calls| f(calls.current_frame.as_deref()))
}

/// The id of the call in progress.
pub(crate) fn current_fc_id() -> Option<FcId> {
    FUNC_CALLS.with(|calls| calls.current)
}

/// Make `id` the call in progress.
pub(crate) fn set_current_fc(id: Option<FcId>) {
    let old = FUNC_CALLS.with_mut(|calls| calls.make_current(id));
    drop(old);
}

/// Keep `id` beyond its call, for the garbage collector to free later.
pub(crate) fn park_funccal(id: FcId) {
    FUNC_CALLS.with_mut(|calls| calls.parked.push(id));
}

/// The parked funccall after `prev` in upstream's newest-first order, or
/// the newest when `prev` is `None` or no longer parked.
///
/// A walk asks afresh at every step rather than holding an iterator,
/// because freeing a parked funccall can re-enter and unpark others.
pub(crate) fn parked_after(prev: Option<FcId>) -> Option<FcId> {
    FUNC_CALLS.with(|calls| {
        let mut newest_first = calls.parked.iter().rev().copied();
        if let Some(prev) = prev
            && calls.parked.contains(&prev)
        {
            newest_first.find(|&id| id == prev);
        }
        newest_first.next()
    })
}

/// Every parked funccall, newest first.
pub(crate) fn parked_funccals() -> Vec<FcId> {
    FUNC_CALLS.with(|calls| calls.parked.iter().rev().copied().collect())
}

/// Take `id` off the parked list; answers whether it was there.
pub(crate) fn unpark_funccal(id: FcId) -> bool {
    FUNC_CALLS.with_mut(|calls| {
        let at = calls.parked.iter().rposition(|&parked| parked == id);
        at.map(|at| calls.parked.remove(at)).is_some()
    })
}

/// The call in progress at each set-aside stack, innermost first.
pub(crate) fn set_aside_call_stacks() -> Vec<Option<FcId>> {
    FUNC_CALLS.with(|calls| calls.aside.iter().rev().copied().collect())
}

/// The call stack put aside while something -- an autocommand, a callback,
/// a provider -- runs from no function at all; dropping it puts the stack
/// back. Upstream's `save_funccal`/`restore_funccal` pair.
#[must_use = "dropping it puts the call stack straight back"]
pub(crate) struct CallStackAside(());

impl CallStackAside {
    /// Put the call stack aside.
    pub(crate) fn new() -> CallStackAside {
        let old = FUNC_CALLS.with_mut(|calls| {
            let current = calls.current;
            calls.aside.push(current);
            calls.make_current(None)
        });
        drop(old);
        CallStackAside(())
    }
}

impl Drop for CallStackAside {
    fn drop(&mut self) {
        let old = FUNC_CALLS.with_mut(|calls| {
            let saved = calls
                .aside
                .pop()
                .expect("a set-aside call stack to restore");
            calls.make_current(saved)
        });
        drop(old);
    }
}
