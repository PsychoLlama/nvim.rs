//! Rust values Lua holds as full userdata.
//!
//! A [`Userdata`] type names the registry metatable its values carry. Lua
//! moves a userdata's bytes nowhere but never runs `Drop` either, so the
//! metatable [`Stack::new_userdata_metatable`] builds carries a `__gc` that
//! takes the value out (leaving `T::default()` behind) and drops it: a
//! userdata a finalizer resurrects after its own `__gc` has run still holds a
//! valid `T`. A `T` with nothing to drop (a `Copy` handle onto a C object
//! something else owns) gets no `__gc`, so the collector has no finalizer
//! to queue for it.
//!
//! # Collected
//!
//! `__gc` is an ordinary field of a metatable Lua can read, so plain Lua can
//! call it on a live value: `getmetatable(x).__gc(x)`, then go on using `x`
//! (upstream then used freed memory, and a second call freed it again). So
//! the block remembers that it was collected, beside the value: the first
//! `__gc` drops the value and marks the block, a second does nothing, and
//! from then on [`Stack::to_userdata`] answers `None` and
//! [`Stack::check_userdata`] raises "`<name>` has been collected" -- every
//! method of every type gets the refusal from the one accessor it already
//! calls. Two consequences for the types:
//!
//! - A method that runs Lua while it uses the value (a decoder, a log
//!   callback) takes what it needs out of the value for the call and puts it
//!   back through `to_userdata` afterwards: a `__gc` inside the call then
//!   finds the value empty, and the put-back finds the block collected and
//!   drops what it holds instead.
//! - A value that points into another userdatum's resource (a tree-sitter
//!   node into its tree) keeps that userdatum alive through its environment,
//!   which stops the collector but not a hand-called `__gc`; its accessor
//!   checks the owner with [`Stack::is_live_userdata`].
//!
//! What makes reading one back sound is that a metatable name belongs to one
//! Rust type: the first `new_userdata::<T>` claims `T::METATABLE` for `T`,
//! a second type that names the same metatable panics there, and
//! [`Stack::to_userdata`] answers only for the type that holds the claim. So
//! every userdata carrying the metatable `to_userdata::<T>` matches was
//! written by `new_userdata::<T>`.
//!
//! The claims are process-wide, behind a lock: a luv thread's state builds
//! `vim.mpack`'s userdata too.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::any::TypeId;
use core::ffi::{CStr, c_int};
use core::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use super::{At, Check, Raise, Stack};
use crate::lua::abi::lua_State;
use crate::lua::ffi::{lua_getfenv, lua_newuserdata, lua_setfenv, luaL_testudata};

/// A Rust value Lua holds as full userdata, under a metatable of its own.
///
/// `Default` is what a collected value leaves behind (see the module
/// documentation).
pub trait Userdata: Default + 'static {
    /// The name the metatable is registered under (`luaL_newmetatable`), and
    /// the type name `luaL_checkudata`'s message gives.
    const METATABLE: &'static CStr;

    /// What [`Stack::check_userdata`] says of a value whose `__gc` already
    /// ran; `None` for "`<METATABLE>` has been collected".
    const COLLECTED: Option<&'static str> = None;
}

/// The bytes of a userdata: the value, and whether its `__gc` has run (see
/// the module documentation).
struct Slot<T> {
    collected: bool,
    value: T,
}

impl<T: Default> Slot<T> {
    /// What `__gc` does to the slot: the value, leaving `T::default()`, and
    /// the slot marked collected; `None` the second time.
    fn collect(&mut self) -> Option<T> {
        let first = !core::mem::replace(&mut self.collected, true);
        first.then(|| core::mem::take(&mut self.value))
    }
}

/// Where a userdatum records that its `__gc` ran, for a value that keeps
/// that userdatum alive through its environment and checks the record on
/// every use without looking the userdatum up again: a tree-sitter node,
/// once per method, where [`Stack::is_live_userdata`]'s registry lookup
/// would cost more than most methods themselves.
#[derive(Clone, Copy, Debug)]
pub struct CollectedMark(ptr::NonNull<bool>);

impl CollectedMark {
    /// Whether the userdatum the mark was taken from has been collected.
    ///
    /// # Safety
    /// That userdatum's block is still allocated: whatever holds the mark
    /// keeps the userdatum reachable (its environment refers to it), so the
    /// collector has not freed it. A `__gc`, by hand or by the collector,
    /// only sets the record.
    pub unsafe fn is_collected(self) -> bool {
        // SAFETY: the caller's; nothing holds a reference to the block
        // while this reads (each access re-derives its own from the block
        // pointer).
        unsafe { self.0.as_ptr().read() }
    }
}

/// Which Rust type each metatable name belongs to. A lock rather than a
/// `GlobalCell`: luv threads' states make and read userdata too.
static CLAIMS: Mutex<Vec<(&'static CStr, TypeId)>> = Mutex::new(Vec::new());

/// The types holding a claim, as an immutable list the read path scans
/// without the lock: [`claimed_slow`] publishes a fresh copy each time a
/// claim is staked. The copies it replaces are leaked on purpose -- a reader
/// on another thread may still be scanning one -- and there is one per
/// `Userdata` type ever staked, a dozen small lists for the process.
static HELD: AtomicPtr<Vec<TypeId>> = AtomicPtr::new(ptr::null_mut());

/// Whether `T` holds the claim on its metatable's name; with `stake`, claim
/// it when nobody does.
///
/// # Panics
/// With `stake`, when another type holds the name.
#[inline]
fn claimed<T: Userdata>(stake: bool) -> bool {
    let ours = TypeId::of::<T>();
    let held = HELD.load(Ordering::Acquire);
    // SAFETY: null, or a list `claimed_slow` published (Release) and never
    // frees or writes again.
    if let Some(held) = unsafe { held.as_ref() }
        && held.contains(&ours)
    {
        return true;
    }
    claimed_slow(T::METATABLE, ours, stake)
}

/// [`claimed`]'s miss: ask the lock, and publish the claimants again when a
/// claim was staked.
#[cold]
#[inline(never)]
fn claimed_slow(name: &'static CStr, ours: TypeId, stake: bool) -> bool {
    // A panic below leaves the list as it was; a poisoned lock is still good.
    let mut claims = CLAIMS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let held = claim(&mut claims, name, ours, stake);
    if held {
        let claimants: Vec<TypeId> = claims.iter().map(|&(_, owner)| owner).collect();
        HELD.store(Box::into_raw(Box::new(claimants)), Ordering::Release);
    }
    drop(claims);
    held
}

/// Whether `ours` holds the claim on `name` in `claims`; with `stake`,
/// claim it when nobody does.
fn claim(
    claims: &mut Vec<(&'static CStr, TypeId)>,
    name: &'static CStr,
    ours: TypeId,
    stake: bool,
) -> bool {
    match claims.iter().find(|(claimed, _)| *claimed == name) {
        Some(&(_, owner)) => {
            assert!(
                !stake || owner == ours,
                "two Rust types share the userdata metatable {name:?}"
            );
            owner == ours
        }
        None if stake => {
            claims.push((name, ours));
            true
        }
        None => false,
    }
}

impl Stack {
    /// Push `T`'s registry metatable, creating it -- with the `__gc` that
    /// drops a collected `T`, when `T` has anything to drop -- when it does
    /// not exist yet; `true` when it was created. The caller adds the
    /// methods.
    pub fn new_userdata_metatable<T: Userdata>(&mut self) -> bool {
        let created = self.new_metatable(T::METATABLE);
        if created && core::mem::needs_drop::<T>() {
            self.push_function(collect::<T>);
            self.set_field(At::from_top(2), c"__gc");
        }
        created
    }

    /// Push the environment table of the userdata (or function, or thread)
    /// at `at`: `lua_getfenv`.
    pub fn push_environment(&mut self, at: At) {
        unsafe { lua_getfenv(self.l(), at.0) };
    }

    /// Pop the table on top and make it the environment of the userdata (or
    /// function, or thread) at `at`: `lua_setfenv`. `false` when the value
    /// there can have none (the table is popped all the same).
    pub fn set_environment(&mut self, at: At) -> bool {
        soundness_check!(self.depth() >= 1, "a view sets an environment it pushed");
        unsafe { lua_setfenv(self.l(), at.0) != 0 }
    }

    /// Push a full userdata of no bytes and no metatable: a value that is
    /// equal only to itself, for a sentinel such as `vim.NIL`.
    pub fn new_empty_userdata(&mut self) {
        unsafe { lua_newuserdata(self.l(), 0) };
    }

    /// Push `value` as a full userdata carrying `T`'s metatable, which must
    /// already be registered (see [`Stack::new_userdata_metatable`]).
    ///
    /// # Panics
    /// When another type has claimed `T::METATABLE`.
    pub fn new_userdata<T: Userdata>(&mut self, value: T) {
        const {
            // LuaJIT aligns a userdata's payload to 8 bytes.
            assert!(
                align_of::<Slot<T>>() <= 8,
                "LuaJIT cannot align this userdata"
            );
        }
        claimed::<T>(true);
        let block = unsafe { lua_newuserdata(self.l(), size_of::<Slot<T>>()) }.cast::<Slot<T>>();
        let slot = Slot {
            collected: false,
            value,
        };
        // SAFETY: a fresh block of `size_of::<Slot<T>>()` bytes, aligned for
        // it (checked above), that nothing else has seen.
        unsafe { ptr::write(block, slot) };
        self.get_field(At::REGISTRY, T::METATABLE);
        self.set_metatable(At::from_top(2));
    }

    /// The `T` at `at` when the slot holds one of `T`'s userdata, not yet
    /// collected.
    pub fn to_userdata<T: Userdata>(&mut self, at: At) -> Option<&mut T> {
        let slot = self.userdata_slot::<T>(at)?;
        (!slot.collected).then_some(&mut slot.value)
    }

    /// The [`CollectedMark`] of the `T` at `at`, collected or not.
    pub fn collected_mark<T: Userdata>(&mut self, at: At) -> Option<CollectedMark> {
        let block = self.userdata_block::<T>(at)?;
        // SAFETY: a block `new_userdata::<T>` wrote (see `userdata_slot`);
        // the field's address is taken without making a reference, so no
        // borrow of the slot is disturbed (the tests below hold Miri to it).
        let mark = unsafe { &raw mut (*block).collected };
        ptr::NonNull::new(mark).map(CollectedMark)
    }

    /// Whether the slot holds one of `T`'s userdata, not yet collected.
    pub fn is_live_userdata<T: Userdata>(&mut self, at: At) -> bool {
        self.to_userdata::<T>(at).is_some()
    }

    /// `luaL_checkudata`: the `T` at `at`, or the error LuaJIT raises for it
    /// -- and an argument error for one already collected.
    pub fn check_userdata<T: Userdata>(&mut self, at: At) -> Result<&mut T, Raise> {
        match self.userdata_slot::<T>(at) {
            Some(slot) if !slot.collected => Ok(&mut slot.value),
            Some(_) => Err(Raise::argument(
                at.0,
                match T::COLLECTED {
                    Some(message) => message.to_owned(),
                    None => format!("{} has been collected", T::METATABLE.to_string_lossy()),
                },
            )),
            None => Err(Raise::check(at, Check::Userdata(T::METATABLE))),
        }
    }

    /// The slot of the `T` at `at`, collected or not.
    fn userdata_slot<T: Userdata>(&mut self, at: At) -> Option<&mut Slot<T>> {
        let block = self.userdata_block::<T>(at)?;
        // SAFETY: `userdata_block` answers only a block `new_userdata::<T>`
        // wrote; the borrow of `self` keeps the stack slot, and so the
        // block, alive.
        Some(unsafe { &mut *block })
    }

    /// The block of the `T` at `at`, when the slot holds one of `T`'s
    /// userdata: `luaL_checkudata`'s test, after the claim.
    fn userdata_block<T: Userdata>(&mut self, at: At) -> Option<*mut Slot<T>> {
        // A full userdata whose metatable is the registry's `T::METATABLE`
        // (a light userdata has none of its own).
        let block = unsafe { luaL_testudata(self.l(), at.0, T::METATABLE.as_ptr()) };
        if block.is_null() || !claimed::<T>(false) {
            return None;
        }
        // `T` holds the claim on its metatable, so only `new_userdata::<T>`
        // made a userdata carrying it, and the block holds a `T`.
        Some(block.cast::<Slot<T>>())
    }
}

/// `__gc` for `T`'s metatable: drop the value, leaving `T::default()`, and
/// mark the block collected; nothing the second time.
///
/// # Safety
/// LuaJIT's contract for a `lua_CFunction`.
unsafe extern "C-unwind" fn collect<T: Userdata>(state: *mut lua_State) -> c_int {
    // SAFETY: the caller's state; the body owns its frame's only `Stack`.
    unsafe {
        Stack::entry(state, |stack| {
            let value = stack.userdata_slot::<T>(At::arg(1)).and_then(Slot::collect);
            drop(value);
            Ok(0)
        })
    }
}

#[cfg(test)]
mod tests {
    //! The block's raw-memory protocol, with a Rust allocation standing in
    //! for LuaJIT's block, so that Miri can follow it: the accessors above
    //! reach the block through `luaL_testudata`, which Miri cannot call.

    use core::cell::Cell;
    use core::ptr::NonNull;
    use std::rc::Rc;

    use super::{CollectedMark, Slot};

    /// Counts its drops, so a test can say when `__gc` released the value.
    #[derive(Debug, Default)]
    struct Tracked(Option<Rc<Cell<u32>>>);

    impl Drop for Tracked {
        fn drop(&mut self) {
            if let Some(drops) = &self.0 {
                drops.set(drops.get() + 1);
            }
        }
    }

    /// A mark taken from the block, then the value used through a fresh
    /// `&mut Slot` (what every method does), then `__gc` twice through
    /// further fresh ones, the mark read between each: the shape a
    /// tree-sitter node has against its tree, including a hand-called
    /// `__gc` while the node lives.
    #[test]
    fn the_mark_sees_a_collect_through_any_later_borrow() {
        let drops = Rc::new(Cell::new(0));
        let block = Box::into_raw(Box::new(Slot {
            collected: false,
            value: Tracked(Some(Rc::clone(&drops))),
        }));
        // As `Stack::collected_mark` takes it: from the block pointer, with
        // no reference made.
        // SAFETY: `block` is the live slot just made, until the `Box` below
        // takes it back.
        let mark = unsafe { CollectedMark(NonNull::new(&raw mut (*block).collected).unwrap()) };
        // SAFETY: as above; the mark's block is live.
        let collected = || unsafe { mark.is_collected() };
        // A fresh `&mut Slot` per access, as every method takes one.
        // SAFETY: as above; each borrow ends before the next access.
        let slot = || unsafe { &mut *block };

        assert!(!collected());
        assert!(!slot().collected);
        assert!(slot().value.0.is_some());
        assert!(!collected());

        let first = slot().collect();
        assert!(first.is_some());
        drop(first);
        assert_eq!(drops.get(), 1);
        assert!(collected());

        // A second `__gc`, by hand or by the collector after a
        // resurrection, finds nothing to drop.
        assert!(slot().collect().is_none());
        assert!(collected());
        assert!(slot().value.0.is_none());

        // SAFETY: the `Box` the block came from, taken back once.
        drop(unsafe { Box::from_raw(block) });
        assert_eq!(drops.get(), 1);
    }
}
