//! The garbage collector's roots registry: every list and dictionary the
//! allocator has handed out and nothing has freed.
//!
//! The collector's job is to find the containers nothing references any more
//! -- a cycle keeps its own reference counts up for ever -- so it needs the
//! whole live population, not just what is reachable. Upstream threaded an
//! intrusive doubly-linked chain through every `List` and `Dict`
//! (`lv_used_next`/`lv_used_prev`, `dv_used_next`/`dv_used_prev`) with
//! `gc_first_list`/`gc_first_dict` as the heads, four pointers a container.
//!
//! This is that population as a **slot table**: a `Vec` of live addresses
//! with holes, and a free list of the holes. A container carries the one
//! [`RootId`] it was given, four bytes instead of sixteen, and nothing ever
//! moves -- which is what lets registering and unregistering both be O(1)
//! without any container having to write into another one.
//!
//! Two consequences worth writing down:
//!
//! - **`RootId::NONE` is zero**, so a container in `xcalloc`'d storage that
//!   never reached the allocator reads as unregistered and unregistering it
//!   is a no-op. Upstream's unlink had no such guard: a `Dict` that was
//!   never on the chain has two null links, and unlinking it set the head
//!   to null and dropped every other dictionary out of the collector's
//!   view.
//! - **The walk is in registration order, oldest first**, where the chain
//!   was newest first, and a freed slot is reused by the next allocation.
//!   Nothing depends on it: [`crate::eval::collect::free_unref_items`]
//!   empties every unreachable container before it frees any of them, and
//!   the marking pass reaches a container through its holders, not through
//!   this table.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ptr::NonNull;

use crate::global_cell::GlobalCell;
use crate::types::{Dict, List};

pub(crate) static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);

/// Where a container sits in the registry.
///
/// Stored as the index plus one, so that [`RootId::NONE`] is all-zero: a
/// container that never reached the allocator -- a scope dictionary
/// initialised in place, a `FuncCall`'s `a:000` -- is not registered, and
/// says so without anybody having to write the field.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RootId(u32);

impl RootId {
    /// Not in the registry.
    pub const NONE: RootId = RootId(0);

    fn index(self) -> Option<usize> {
        (self != RootId::NONE).then(|| self.0 as usize - 1)
    }
}

impl Default for RootId {
    fn default() -> Self {
        RootId::NONE
    }
}

/// The live population of one container kind.
struct Registry<T> {
    /// The registered addresses; a `None` is a hole.
    slots: Vec<Option<NonNull<T>>>,
    /// Which slots are holes, so that registering is O(1).
    holes: Vec<u32>,
}

impl<T> Registry<T> {
    const fn new() -> Self {
        Registry {
            slots: Vec::new(),
            holes: Vec::new(),
        }
    }

    fn insert(&mut self, at: NonNull<T>) -> RootId {
        let idx = match self.holes.pop() {
            Some(idx) => {
                self.slots[idx as usize] = Some(at);
                idx
            }
            None => {
                self.slots.push(Some(at));
                u32::try_from(self.slots.len() - 1)
                    .expect("four billion live containers is not a working editor")
            }
        };
        RootId(idx + 1)
    }

    fn remove(&mut self, id: RootId) {
        let Some(idx) = id.index() else { return };
        debug_assert!(self.slots[idx].is_some(), "a root removed twice");
        self.slots[idx] = None;
        self.holes.push(
            u32::try_from(idx).expect("the index came back from `insert`, which range-checked it"),
        );
    }
}

static live_lists: GlobalCell<Registry<List>> = GlobalCell::new(Registry::new());
static live_dicts: GlobalCell<Registry<Dict>> = GlobalCell::new(Registry::new());

/// Register `l` as a live list, answering the id it must carry.
pub(crate) fn root_list(l: NonNull<List>) -> RootId {
    live_lists.with_mut(|reg| reg.insert(l))
}

/// Take `id` out of the list registry. A no-op for [`RootId::NONE`].
pub(crate) fn unroot_list(id: RootId) {
    live_lists.with_mut(|reg| reg.remove(id));
}

/// Register `d` as a live dictionary, answering the id it must carry.
pub(crate) fn root_dict(d: NonNull<Dict>) -> RootId {
    live_dicts.with_mut(|reg| reg.insert(d))
}

/// Take `id` out of the dictionary registry. A no-op for [`RootId::NONE`].
pub(crate) fn unroot_dict(id: RootId) {
    live_dicts.with_mut(|reg| reg.remove(id));
}

/// How many slots the list registry has: the bound of a walk's index.
pub(crate) fn list_slots() -> usize {
    live_lists.with(|reg| reg.slots.len())
}

/// How many slots the dictionary registry has.
pub(crate) fn dict_slots() -> usize {
    live_dicts.with(|reg| reg.slots.len())
}

/// The list in slot `idx`, or `None` for a hole or a slot past the end.
///
/// A walk reads one slot at a time rather than borrowing the table, because
/// what it does with each container re-enters: emptying a dictionary runs
/// its watchers' teardown, and freeing one can allocate.
pub(crate) fn list_at(idx: usize) -> Option<NonNull<List>> {
    live_lists.with(|reg| reg.slots.get(idx).copied().flatten())
}

/// The dictionary in slot `idx`, or `None` for a hole or a slot past the end.
pub(crate) fn dict_at(idx: usize) -> Option<NonNull<Dict>> {
    live_dicts.with(|reg| reg.slots.get(idx).copied().flatten())
}

/// Every live list, in registration order.
///
/// `pub` rather than `pub(crate)` because the registry *is* the assertion in
/// `crates/nvim/tests/unit/channel_reader.rs`: a list handed to a job
/// callback and not stored is freed again, which nothing reachable can see
/// -- `garbagecollect()` would collect a leaked one and no Vimscript, Lua or
/// API call names it.
pub fn rooted_lists() -> Vec<*mut List> {
    live_lists.with(|reg| reg.slots.iter().flatten().map(|l| l.as_ptr()).collect())
}
