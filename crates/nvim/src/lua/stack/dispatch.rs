//! C functions written as safe bodies, reached through one trampoline.
//!
//! LuaJIT calls a C function as `unsafe extern "C-unwind" fn(*mut
//! lua_State) -> c_int`, so a module that hands Lua many functions would
//! otherwise write that signature, and the one unsafe step that turns its
//! pointer into a [`Stack`], once per function. A [`Dispatch`] is a set of
//! such functions written as safe bodies over a `Stack` and told apart by an
//! index: [`dispatch_entry::<D, N>`](dispatch_entry) is entry `N` as a C
//! function -- [`trampoline`] instantiated for that index -- which runs the
//! entry inside [`Stack::entry`], so the entry's error is raised only after
//! its frame is gone.
//!
//! The index is a const parameter rather than a closure upvalue: reading an
//! upvalue back costs LuaJIT's pseudo-index walk on every call (about 37
//! instructions, 1 % of a small `vim.api` call), and an upvalue is writable
//! from Lua (`debug.setupvalue`).

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

use core::ffi::c_int;

use super::{CFunction, Raise, Stack};
use crate::types::lua_State;

/// A set of C functions written as safe bodies, numbered from 0.
pub trait Dispatch: 'static {
    /// Run entry `index` over the frame Lua called it with, answering how
    /// many results it left on top.
    fn run(stack: &mut Stack, index: usize) -> Result<c_int, Raise>;
}

/// Entry `N` of `D`, as the C function Lua calls.
pub(crate) const fn dispatch_entry<D: Dispatch, const N: usize>() -> CFunction {
    trampoline::<D, N>
}

/// The one C function behind every [`Dispatch`] entry, instantiated per
/// entry.
///
/// # Safety
/// LuaJIT's contract for a `lua_CFunction`: `state` is the running state,
/// calling this function with its arguments on top of the stack.
unsafe extern "C-unwind" fn trampoline<D: Dispatch, const N: usize>(
    state: *mut lua_State,
) -> c_int {
    // SAFETY: the caller's state; the body owns its frame's only `Stack`.
    unsafe { Stack::entry(state, run::<D, N>) }
}

/// Entry `N`'s body.
fn run<D: Dispatch, const N: usize>(stack: &mut Stack) -> Result<c_int, Raise> {
    D::run(stack, N)
}
