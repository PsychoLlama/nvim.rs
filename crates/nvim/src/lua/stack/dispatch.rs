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

use core::ffi::{CStr, c_int};

use super::{At, CFunction, Raise, Stack};
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

/// One named C function of a [`Dispatch`] table: the name Lua knows it by,
/// and entry `N` of the table as the C function Lua calls.
///
/// [`lua_functions!`](crate::lua_functions) builds these; a `static` table of
/// them is what [`Stack::register_functions`] sets on a Lua table, and its
/// entries are the very pointers Lua holds, so one can be compared with what
/// Lua hands back.
pub struct Function {
    name: &'static CStr,
    index: usize,
    entry: CFunction,
}

impl Function {
    /// Entry `N` of `D`, under `name`.
    pub const fn new<D: Dispatch, const N: usize>(name: &'static CStr) -> Function {
        Function {
            name,
            index: N,
            entry: dispatch_entry::<D, N>(),
        }
    }

    /// The name Lua knows the function by.
    pub fn name(&self) -> &'static CStr {
        self.name
    }

    /// The C function Lua calls.
    pub fn entry(&self) -> CFunction {
        self.entry
    }
}

impl Stack {
    /// Set each of `functions` as a field of the table on top, under its
    /// name, as `luaL_register(L, NULL, ..)` does.
    ///
    /// # Panics
    /// If an entry is not in the slot its index names, which would make it
    /// run another entry's body.
    pub fn register_functions(&mut self, functions: &[Function]) {
        for (index, function) in functions.iter().enumerate() {
            assert_eq!(
                function.index, index,
                "{:?} is not where its entry says",
                function.name
            );
            self.push_function(function.entry);
            self.set_field(At::from_top(2), function.name);
        }
    }
}

/// A table of C functions written as safe bodies: an uninhabited type that
/// is their [`Dispatch`], and a `static` slice of [`Function`]s, one per
/// body, numbered as written.
///
/// ```ignore
/// lua_functions! {
///     /// `vim.base64`.
///     enum Base64Functions in BASE64 {
///         0 => c"encode": encode,
///         1 => c"decode": decode,
///     }
/// }
/// ```
///
/// Each body is a `fn(&mut Stack) -> Result<c_int, Raise>`, run inside
/// [`Stack::entry`] by the trampoline instantiated for its number. The
/// numbers are written out because a const parameter needs one; a number
/// out of place fails [`Stack::register_functions`].
#[macro_export]
macro_rules! lua_functions {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident in $table:ident {
            $($index:literal => $lua_name:literal : $body:path),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {}

        impl $crate::lua::stack::Dispatch for $name {
            fn run(
                stack: &mut $crate::lua::stack::Stack,
                index: usize,
            ) -> ::core::result::Result<::core::ffi::c_int, $crate::lua::stack::Raise> {
                match index {
                    $($index => $body(stack),)*
                    _ => ::core::unreachable!("only the table names an entry"),
                }
            }
        }

        $vis static $table: &[$crate::lua::stack::Function] = &[
            $($crate::lua::stack::Function::new::<$name, $index>($lua_name),)*
        ];
    };
}
