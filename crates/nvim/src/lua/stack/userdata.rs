//! Rust values Lua holds as full userdata.
//!
//! A [`Userdata`] type names the registry metatable its values carry. Lua
//! moves a userdata's bytes nowhere but never runs `Drop` either, so the
//! metatable [`Stack::new_userdata_metatable`] builds carries a `__gc` that
//! takes the value out (leaving `T::default()` behind) and drops it: a
//! userdata a finalizer resurrects after its own `__gc` has run still holds a
//! valid `T`.
//!
//! What makes reading one back sound is that a metatable name belongs to one
//! Rust type: the first `new_userdata::<T>` claims `T::METATABLE` for `T`,
//! a second type that names the same metatable panics there, and
//! [`Stack::to_userdata`] answers only for the type that holds the claim. So
//! every userdata carrying the metatable `to_userdata::<T>` matches was
//! written by `new_userdata::<T>`.

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

use super::{At, Check, Kind, Raise, Stack};
use crate::global_cell::GlobalCell;
use crate::lua::ffi::{lua_newuserdata, lua_touserdata};
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

/// Which Rust type each metatable name belongs to.
static CLAIMS: GlobalCell<Vec<(&'static CStr, TypeId)>> = GlobalCell::new(Vec::new());

/// Whether `T` holds the claim on its metatable's name; with `stake`, claim
/// it when nobody does.
///
/// # Panics
/// With `stake`, when another type holds the name.
fn claimed<T: Userdata>(stake: bool) -> bool {
    let ours = TypeId::of::<T>();
    CLAIMS.with_mut(
        |claims| match claims.iter().find(|(name, _)| *name == T::METATABLE) {
            Some(&(_, owner)) => {
                assert!(
                    !stake || owner == ours,
                    "two Rust types share the userdata metatable {:?}",
                    T::METATABLE
                );
                owner == ours
            }
            None if stake => {
                claims.push((T::METATABLE, ours));
                true
            }
            None => false,
        },
    )
}

impl Stack {
    /// Push `T`'s registry metatable, creating it -- with the `__gc` that
    /// drops a collected `T` -- when it does not exist yet; `true` when it
    /// was created. The caller adds the methods.
    pub fn new_userdata_metatable<T: Userdata>(&mut self) -> bool {
        let created = self.new_metatable(T::METATABLE);
        if created {
            self.push_function(collect::<T>);
            self.set_field(At::from_top(2), c"__gc");
        }
        created
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
        // A light userdata answers `lua_touserdata` too, but is not one of
        // ours: `luaL_checkudata` asks for a full userdata first.
        if self.kind(at) != Kind::Userdata || !claimed::<T>(false) {
            return None;
        }
        let block = unsafe { lua_touserdata(self.l(), at.0) };
        if block.is_null() || !self.get_metatable(at) {
            return None;
        }
        self.get_field(At::REGISTRY, T::METATABLE);
        let ours = self.raw_equal(At::from_top(1), At::from_top(2));
        self.pop(2);
        // SAFETY: `T` holds the claim on its metatable, so only
        // `new_userdata::<T>` made a userdata carrying it, and the block
        // holds a `T`; the borrow of `self` keeps the slot, and so the
        // block, alive.
        ours.then(|| unsafe { &mut *block.cast::<T>() })
    }

    /// `luaL_checkudata`: the `T` at `at`, or the error LuaJIT raises for it.
    pub fn check_userdata<T: Userdata>(&mut self, at: At) -> Result<&mut T, Raise> {
        if self.to_userdata::<T>(at).is_none() {
            return Err(Raise::check(at, Check::Userdata(T::METATABLE)));
        }
        Ok(self.to_userdata::<T>(at).expect("checked above"))
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
