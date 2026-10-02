//! Who owns a [`FuncCall`], and which one is running.
//!
//! Upstream keeps three raw globals: `current_funccal` (the call in
//! progress, whose `fc_caller` chain is the call stack), `previous_funccal`
//! (the funccalls a closure, an escaped `l:` or a returned `a:000` kept alive
//! after their call returned, threaded through the same `fc_caller` link) and
//! `funccal_stack` (a list of `funccal_entry_T`s in the frames of whoever set
//! the call stack aside to run an autocommand or a callback).
//!
//! Here every funccall, running or parked, is **owned by one slot table**,
//! and the three globals are ids into it:
//!
//! - a [`FcId`] is a slot index plus the slot's generation, so an id kept
//!   past its funccall's end resolves to a panic, not to freed memory or to
//!   whatever reused the slot;
//! - a slot holds its funccall in a `Box` that never moves once filled, and
//!   every pointer into the funccall -- the `l:` dictionary a value refers
//!   to, a closure's `uf_scoped` -- is derived through the `UnsafeCell`, so
//!   none of them is a borrow of the box;
//! - the box holds the funccall in a [`ManuallyDrop`]: a funccall's contents
//!   are given back by hand ([`super::free_funccal`]), as upstream's `xfree`
//!   gives back only the block;
//! - each slot records the call its funccall was made from, which is
//!   upstream's `fc_caller` chain;
//! - the parked funccalls are a `Vec` of ids, newest last, and the set-aside
//!   call stacks a `Vec` of the ids that were current, innermost last.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::num::NonZeroU32;
use core::ptr;

use crate::global_cell::GlobalCell;
use crate::types::{FcId, FuncCall};

/// One funccall's storage. It is filled once and emptied once; the box
/// itself is never moved in between, so the funccall's address is fixed.
type Frame = Box<UnsafeCell<ManuallyDrop<FuncCall>>>;

/// A slot of the table.
struct Slot {
    /// Bumped every time the slot is emptied, so an id of an earlier tenant
    /// does not match.
    generation: NonZeroU32,
    frame: Option<Frame>,
    /// The call this one was made from: upstream's `fc_caller`.
    caller: Option<FcId>,
}

/// The funccalls and the three ids upstream kept as raw globals.
struct FuncCalls {
    slots: Vec<Slot>,
    /// Empty slots, reused last-freed first.
    vacant: Vec<u32>,
    /// The call in progress: upstream's `current_funccal`.
    current: Option<FcId>,
    /// Funccalls kept beyond their call, newest last: upstream's
    /// `previous_funccal` list.
    parked: Vec<FcId>,
    /// The call in progress at each [`CallStackAside`], innermost last:
    /// upstream's `funccal_stack`.
    aside: Vec<Option<FcId>>,
}

static FUNC_CALLS: GlobalCell<FuncCalls> = GlobalCell::new(FuncCalls {
    slots: Vec::new(),
    vacant: Vec::new(),
    current: None,
    parked: Vec::new(),
    aside: Vec::new(),
});

impl FuncCalls {
    fn slot(&self, id: FcId) -> &Slot {
        let slot = &self.slots[id.0 as usize];
        assert!(
            slot.generation == id.1 && slot.frame.is_some(),
            "a funccall id outlived its funccall"
        );
        slot
    }

    fn funccall(&self, id: FcId) -> *mut FuncCall {
        match &self.slot(id).frame {
            Some(frame) => frame.get().cast(),
            None => unreachable!(),
        }
    }
}

impl FcId {
    /// The funccall `self` names.
    ///
    /// # Panics
    /// When that funccall has been freed.
    pub(crate) fn funccall(self) -> *mut FuncCall {
        FUNC_CALLS.with(|calls| calls.funccall(self))
    }

    /// The call `self` was made from: upstream's `fc_caller`.
    pub(crate) fn caller(self) -> Option<FcId> {
        FUNC_CALLS.with(|calls| calls.slot(self).caller)
    }
}

/// `top` and every call below it, innermost first: one call stack.
pub(crate) fn call_chain(top: Option<FcId>) -> Vec<FcId> {
    FUNC_CALLS.with(|calls| ::core::iter::successors(top, |&id| calls.slot(id).caller).collect())
}

/// Take ownership of `fc`, a call made from `caller`, and answer its id and
/// its fixed address. Its `fc_id` is set here.
pub(crate) fn adopt_funccal(mut fc: FuncCall, caller: Option<FcId>) -> (FcId, *mut FuncCall) {
    FUNC_CALLS.with_mut(|calls| {
        let index = calls.vacant.pop().unwrap_or_else(|| {
            let index = u32::try_from(calls.slots.len()).expect("fewer than 2^32 funccalls");
            calls.slots.push(Slot {
                generation: NonZeroU32::MIN,
                frame: None,
                caller: None,
            });
            index
        });
        let slot = &mut calls.slots[index as usize];
        let id = FcId(index, slot.generation);
        fc.fc_id = Some(id);
        slot.caller = caller;
        let frame = slot
            .frame
            .insert(Box::new(UnsafeCell::new(ManuallyDrop::new(fc))));
        (id, frame.get().cast())
    })
}

/// Give back `id`'s storage. What the funccall held must have been given
/// back already; nothing of it is dropped here.
pub(crate) fn release_funccal(id: FcId) {
    let frame = FUNC_CALLS.with_mut(|calls| {
        let slot = &mut calls.slots[id.0 as usize];
        assert!(slot.generation == id.1, "a funccall freed twice");
        slot.generation = slot.generation.checked_add(1).unwrap_or(NonZeroU32::MIN);
        calls.vacant.push(id.0);
        slot.caller = None;
        slot.frame.take()
    });
    // Freed outside the borrow; the `ManuallyDrop` keeps it to the block.
    drop(frame);
}

/// The call in progress, or null.
pub(crate) fn current_fc() -> *mut FuncCall {
    FUNC_CALLS.with(|calls| {
        calls
            .current
            .map_or(ptr::null_mut(), |id| calls.funccall(id))
    })
}

/// The id of the call in progress.
pub(crate) fn current_fc_id() -> Option<FcId> {
    FUNC_CALLS.with(|calls| calls.current)
}

/// Make `id` the call in progress.
pub(crate) fn set_current_fc(id: Option<FcId>) {
    FUNC_CALLS.with_mut(|calls| calls.current = id);
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
        FUNC_CALLS.with_mut(|calls| {
            let current = calls.current.take();
            calls.aside.push(current);
        });
        CallStackAside(())
    }
}

impl Drop for CallStackAside {
    fn drop(&mut self) {
        FUNC_CALLS.with_mut(|calls| {
            let saved = calls
                .aside
                .pop()
                .expect("a set-aside call stack to restore");
            calls.current = saved;
        });
    }
}
