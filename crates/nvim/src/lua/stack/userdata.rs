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
use crate::lua::ffi::{lua_getfenv, lua_newuserdata, lua_setfenv, luaL_testudata};
use crate::types::lua_State;

/// A Rust value Lua holds as full userdata, under a metatable of its own.
///
/// `Default` is what a collected value leaves behind (see the module
/// documentation).
pub trait Userdata: Default + 'static {
    /// The name the metatable is registered under (`luaL_newmetatable`), and
    /// the type name `luaL_checkudata`'s message gives.
    const METATABLE: &'static CStr;
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
        debug_assert!(self.depth() >= 1, "a view sets an environment it pushed");
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
            assert!(align_of::<T>() <= 8, "LuaJIT cannot align this userdata");
        }
        claimed::<T>(true);
        let block = unsafe { lua_newuserdata(self.l(), size_of::<T>()) }.cast::<T>();
        // SAFETY: a fresh block of `size_of::<T>()` bytes, aligned for `T`
        // (checked above), that nothing else has seen.
        unsafe { ptr::write(block, value) };
        self.get_field(At::REGISTRY, T::METATABLE);
        self.set_metatable(At::from_top(2));
    }

    /// The `T` at `at` when the slot holds one of `T`'s userdata.
    pub fn to_userdata<T: Userdata>(&mut self, at: At) -> Option<&mut T> {
        let block = self.userdata_block::<T>(at)?;
        // SAFETY: `userdata_block` answers only a block `new_userdata::<T>`
        // wrote; the borrow of `self` keeps the slot, and so the block,
        // alive.
        Some(unsafe { &mut *block })
    }

    /// `luaL_checkudata`: the `T` at `at`, or the error LuaJIT raises for it.
    pub fn check_userdata<T: Userdata>(&mut self, at: At) -> Result<&mut T, Raise> {
        match self.userdata_block::<T>(at) {
            // SAFETY: as in `to_userdata`.
            Some(block) => Ok(unsafe { &mut *block }),
            None => Err(Raise::check(at, Check::Userdata(T::METATABLE))),
        }
    }

    /// The block of the `T` at `at`, when the slot holds one of `T`'s
    /// userdata: `luaL_checkudata`'s test, after the claim.
    fn userdata_block<T: Userdata>(&mut self, at: At) -> Option<*mut T> {
        // A full userdata whose metatable is the registry's `T::METATABLE`
        // (a light userdata has none of its own).
        let block = unsafe { luaL_testudata(self.l(), at.0, T::METATABLE.as_ptr()) };
        if block.is_null() || !claimed::<T>(false) {
            return None;
        }
        // `T` holds the claim on its metatable, so only `new_userdata::<T>`
        // made a userdata carrying it, and the block holds a `T`.
        Some(block.cast::<T>())
    }
}

/// `__gc` for `T`'s metatable: drop the value, leaving `T::default()`.
///
/// # Safety
/// LuaJIT's contract for a `lua_CFunction`.
unsafe extern "C-unwind" fn collect<T: Userdata>(state: *mut lua_State) -> c_int {
    // SAFETY: the caller's state; the body owns its frame's only `Stack`.
    unsafe {
        Stack::entry(state, |stack| {
            let value = stack.to_userdata::<T>(At::arg(1)).map(core::mem::take);
            drop(value);
            Ok(0)
        })
    }
}
