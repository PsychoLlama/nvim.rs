//! Where each running command line keeps its condition stack.
//!
//! A command handler reaches the stack of the command line it runs in by the
//! id its `ExArg` carries, a borrow at a time: nothing holds the table
//! across a call that can run user code, which may start a command line of
//! its own and open a stack beside this one.
//!
//! A stack is about 1.5 KiB and a function call opens one, so an emptied one
//! is kept, already cleared, for the next.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::id_table::{Boxed, IdTable};
use crate::types::{CondId, CondStack};
use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;

/// The condition stack of every running command line, and the emptied ones
/// kept to build the next one in: a cell of its own, so that a step on a
/// stack may read the exception state.
static COND_STACKS: GlobalCell<CondStacks> = GlobalCell::new(CondStacks {
    live: IdTable::new(),
    spare: Vec::new(),
});

struct CondStacks {
    live: IdTable<CondStack>,
    spare: Vec<Boxed<CondStack>>,
}

/// How many emptied stacks are kept: more than a command line nests in
/// practice, few enough to be nothing in memory.
const SPARES: usize = 16;

impl CondId {
    /// Run `f` on the stack this id names. `f` must not run user code.
    ///
    /// # Panics
    /// When the stack is gone: its command line has finished.
    #[inline]
    pub(crate) fn with<R>(self, f: impl FnOnce(&mut CondStack) -> R) -> R {
        COND_STACKS.with_mut(|stacks| f(stacks.live.get_mut(self)))
    }
}

/// A condition stack a command line opened, closed when this goes.
pub(crate) struct OwnedCondStack(CondId);

impl OwnedCondStack {
    /// An empty stack, in a spare one when there is one.
    pub(crate) fn open() -> OwnedCondStack {
        let id = COND_STACKS.with_mut(|stacks| {
            let boxed = stacks
                .spare
                .pop()
                .unwrap_or_else(|| Box::new(UnsafeCell::new(ManuallyDrop::new(CondStack::new()))));
            stacks.live.insert_boxed((), boxed, |_, _| {}).0
        });
        OwnedCondStack(id)
    }

    /// Its id, for the commands run against it.
    pub(crate) fn id(&self) -> CondId {
        self.0
    }
}

impl Drop for OwnedCondStack {
    fn drop(&mut self) {
        let mut boxed = COND_STACKS.with_mut(|stacks| stacks.live.evict(self.0));
        // Whatever a level still holds -- a `:for`'s list, a pending
        // `:return`'s value -- is released outside the table's borrow.
        let left = boxed.get_mut().clear();
        let extra = COND_STACKS.with_mut(|stacks| {
            if stacks.spare.len() < SPARES {
                stacks.spare.push(boxed);
                None
            } else {
                Some(boxed)
            }
        });
        if let Some(boxed) = extra {
            drop(ManuallyDrop::into_inner(UnsafeCell::into_inner(*boxed)));
        }
        drop(left);
    }
}
