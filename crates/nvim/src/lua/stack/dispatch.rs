//! C functions written as safe bodies, reached through one trampoline.
//!
//! LuaJIT calls a C function as `unsafe extern "C-unwind" fn(*mut
//! lua_State) -> c_int`, so a module that hands Lua many functions would
//! otherwise write that signature, and the one unsafe step that turns its
//! pointer into a [`Stack`], once per function. A [`Dispatch`] is a set of
//! such functions written as safe bodies over a `Stack` and told apart by an
//! index: [`Stack::push_dispatch`] pushes entry `n` as a C closure over
//! [`trampoline`] with `n` as its one upvalue, and the trampoline reads the
//! index back and runs the entry inside [`Stack::entry`], so the entry's
//! error is raised only after its frame is gone.

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

use super::{At, Raise, Stack};
use crate::types::{lua_Integer, lua_State};

/// A set of C functions written as safe bodies, numbered from 0.
pub trait Dispatch: 'static {
    /// Run entry `index` over the frame Lua called it with, answering how
    /// many results it left on top. An `index` that names no entry is the
    /// implementation's to refuse: the upvalue that carries it is reachable
    /// from Lua (`debug.setupvalue`).
    fn run(stack: &mut Stack, index: usize) -> Result<c_int, Raise>;
}

impl Stack {
    /// Push entry `index` of `D` as a Lua function.
    pub fn push_dispatch<D: Dispatch>(&mut self, index: usize) {
        let index = lua_Integer::try_from(index).expect("a dispatch index fits a Lua integer");
        self.push_integer(index);
        self.push_closure(trampoline::<D>, 1);
    }
}

/// The one C function behind every [`Dispatch`] entry: which entry is the
/// closure's upvalue.
///
/// # Safety
/// LuaJIT's contract for a `lua_CFunction`: `state` is the running state,
/// calling this function with its arguments on top of the stack.
unsafe extern "C-unwind" fn trampoline<D: Dispatch>(state: *mut lua_State) -> c_int {
    // SAFETY: the caller's state; the body owns its frame's only `Stack`.
    unsafe { Stack::entry(state, run::<D>) }
}

/// Read the entry's index back from the closure and run it.
fn run<D: Dispatch>(stack: &mut Stack) -> Result<c_int, Raise> {
    let index = stack.to_integer(At::upvalue(1));
    let index = usize::try_from(index).map_err(|_| Raise::message("no such function"))?;
    D::run(stack, index)
}
