//! The function table: every user function that has a name, by that name.
//!
//! Upstream's `func_hashtab` keyed each function by a pointer into its own
//! allocation -- the name was the struct's trailing flexible member -- so a
//! table slot was where the function lived. Here the table *owns* a
//! reference to each function and keys it by the function's owned name.
//!
//! What is kept exactly is the slot every name lands in, because the slot
//! order is what Vim shows: `:function` lists the table in slot order, and so
//! do name completion and the profile report. The table is the same
//! open-addressed table as [`crate::types::HashTab`] -- the same hash, the
//! same probe sequence, the same resize thresholds, the same rehash order --
//! over slots that hold an owned [`UserFunc`] rather than a borrowed key.
//!
//! A walk over it reads by slot index and asks afresh at every step, since a
//! callback the walk runs (a listing's redraw, a completion) may define or
//! delete functions; [`FuncTable::changed`] is how it notices.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use std::rc::Rc;

use crate::global_cell::GlobalCell;
use crate::hashtab::{HT_INIT_SIZE, Probe, hash_bytes_len, resize_decision};
use crate::types::{HashValue, UserFunc};

/// One slot.
enum Slot {
    /// Never held a function: ends a probe.
    Empty,
    /// Held one that was removed: a probe walks past it, an insertion may
    /// reuse it.
    Removed,
    /// A function, with its name's hash.
    Kept(HashValue, Rc<UserFunc>),
}

/// The table itself.
pub(crate) struct FuncTable {
    slots: Vec<Slot>,
    /// Live entries.
    used: usize,
    /// Entries plus tombstones: what the load factor is measured against.
    filled: usize,
    /// Bumped by every add, remove and resize.
    changed: c_int,
}

impl FuncTable {
    const fn new() -> FuncTable {
        FuncTable {
            slots: Vec::new(),
            used: 0,
            filled: 0,
            changed: 0,
        }
    }

    /// The slot `name` occupies, or -- when it is absent -- the slot it
    /// belongs in: the first tombstone the probe crossed, else the empty slot
    /// that ended it. `None` before the table has slots.
    fn lookup(&self, name: &[u8], hash: HashValue) -> Option<usize> {
        if self.slots.is_empty() {
            return None;
        }
        let mut free = None;
        // The table is never full, so some slot is empty and ends the walk.
        for idx in Probe::new(hash, self.slots.len() - 1) {
            match &self.slots[idx] {
                Slot::Empty => return Some(free.unwrap_or(idx)),
                Slot::Removed => {
                    free.get_or_insert(idx);
                }
                Slot::Kept(kept, func) => {
                    if *kept == hash && func.name().as_bytes() == name {
                        return Some(idx);
                    }
                }
            }
        }
        unreachable!("a probe always ends at an empty slot")
    }

    fn find(&self, name: &[u8]) -> Option<&Rc<UserFunc>> {
        let idx = self.lookup(name, hash_bytes_len(name))?;
        match &self.slots[idx] {
            Slot::Kept(_, func) => Some(func),
            _ => None,
        }
    }

    /// Add `func` under its own name. A name already there is refused, and
    /// the function handed back.
    fn add(&mut self, func: Rc<UserFunc>) -> Result<(), Rc<UserFunc>> {
        if self.slots.is_empty() {
            self.slots = (0..HT_INIT_SIZE).map(|_| Slot::Empty).collect();
        }
        let hash = hash_bytes_len(func.name().as_bytes());
        let idx = self
            .lookup(func.name().as_bytes(), hash)
            .expect("the table has slots");
        let was_empty = match &self.slots[idx] {
            Slot::Kept(..) => return Err(func),
            Slot::Empty => true,
            Slot::Removed => false,
        };
        self.slots[idx] = Slot::Kept(hash, func);
        self.used += 1;
        self.changed += 1;
        if was_empty {
            self.filled += 1;
        }
        self.may_resize();
        Ok(())
    }

    /// Put `func` in the slot its name already holds, counting nothing as
    /// added or removed: a redefinition that keeps the entry. Answers the
    /// function back when its name holds no slot.
    fn replace(&mut self, func: Rc<UserFunc>) -> Result<Rc<UserFunc>, Rc<UserFunc>> {
        let hash = hash_bytes_len(func.name().as_bytes());
        if let Some(idx) = self.lookup(func.name().as_bytes(), hash)
            && let Slot::Kept(_, kept) = &mut self.slots[idx]
        {
            return Ok(core::mem::replace(kept, func));
        }
        Err(func)
    }

    /// Take the function named `name` out of the table.
    fn remove(&mut self, name: &[u8]) -> Option<Rc<UserFunc>> {
        let idx = self.lookup(name, hash_bytes_len(name))?;
        if !matches!(self.slots[idx], Slot::Kept(..)) {
            return None;
        }
        let Slot::Kept(_, func) = core::mem::replace(&mut self.slots[idx], Slot::Removed) else {
            unreachable!("checked just above")
        };
        self.used -= 1;
        self.changed += 1;
        self.may_resize();
        Some(func)
    }

    /// Grow, shrink or compact the slots when the load factors say so: the
    /// hashtab's rule, and its rehash order.
    fn may_resize(&mut self) {
        let Some(size) = resize_decision(self.filled, self.used, self.slots.len(), 0) else {
            return;
        };
        let old = core::mem::replace(&mut self.slots, (0..size).map(|_| Slot::Empty).collect());
        let mask = size - 1;
        for slot in old {
            if let Slot::Kept(hash, func) = slot {
                let idx = Probe::new(hash, mask)
                    .find(|&idx| matches!(self.slots[idx], Slot::Empty))
                    .expect("a probe always ends at an empty slot");
                self.slots[idx] = Slot::Kept(hash, func);
            }
        }
        self.filled = self.used;
        self.changed += 1;
    }
}

static FUNCTIONS: GlobalCell<FuncTable> = GlobalCell::new(FuncTable::new());

/// Give the table its first slots, once, at startup.
pub(crate) fn func_init() {
    FUNCTIONS.with_mut(|table| {
        if table.slots.is_empty() {
            table.slots = (0..HT_INIT_SIZE).map(|_| Slot::Empty).collect();
        }
    });
}

/// The function named `name`, already translated (`<SNR>` mangled, no `g:`).
pub(crate) fn find_func(name: &[u8]) -> Option<Rc<UserFunc>> {
    FUNCTIONS.with(|table| table.find(name).cloned())
}

/// Whether a function is named `name`.
pub(crate) fn func_exists(name: &[u8]) -> bool {
    FUNCTIONS.with(|table| table.find(name).is_some())
}

/// Add `func` to the table under its name; a name already there is an
/// internal error (E685), and the function is handed back unadded.
pub(crate) fn add_func(func: Rc<UserFunc>) -> Result<(), Rc<UserFunc>> {
    FUNCTIONS
        .with_mut(|table| table.add(func))
        .inspect_err(|func| {
            let name = crate::message_fmt::msg_bytes(func.name().as_bytes());
            crate::siemsg!("E685: Internal error: hash_add(): duplicate key \"{name}\"");
        })
}

/// Put `func` in the place its name holds, which keeps the name's slot. A
/// name that has left the table meanwhile is added afresh.
pub(crate) fn replace_func(func: Rc<UserFunc>) {
    // The old function is dropped outside the table's borrow.
    let old = FUNCTIONS.with_mut(|table| table.replace(func));
    match old {
        Ok(old) => drop(old),
        Err(func) => {
            let _ = add_func(func);
        }
    }
}

/// Take the function named `name` out of the table; answers whether it was
/// there.
pub(crate) fn remove_func(name: &[u8]) -> bool {
    let removed = FUNCTIONS.with_mut(|table| table.remove(name));
    removed.is_some()
}

/// Bumped by every add, remove and resize: a walk compares it to tell that
/// a callback rearranged the table under it.
pub(crate) fn func_table_changed() -> c_int {
    FUNCTIONS.with(|table| table.changed)
}

/// How many functions the table holds.
pub(crate) fn func_table_used() -> usize {
    FUNCTIONS.with(|table| table.used)
}

/// The function in slot `idx`, if that slot holds one.
pub(crate) fn func_at_slot(idx: usize) -> Option<Rc<UserFunc>> {
    FUNCTIONS.with(|table| match table.slots.get(idx) {
        Some(Slot::Kept(_, func)) => Some(func.clone()),
        _ => None,
    })
}

/// Every function in the table, in slot order: a snapshot, for a walk whose
/// body cannot change the table or need not see it change.
pub(crate) fn all_funcs() -> Vec<Rc<UserFunc>> {
    FUNCTIONS.with(|table| {
        table
            .slots
            .iter()
            .filter_map(|slot| match slot {
                Slot::Kept(_, func) => Some(func.clone()),
                _ => None,
            })
            .collect()
    })
}
