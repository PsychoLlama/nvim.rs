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
//! [`Stack::entry`] and [`Raise`] live here too: how any C function's body
//! starts, and the one way it raises.
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

use core::ffi::{CStr, c_char, c_int};
use core::ptr::{self, NonNull};

use super::{At, CFunction, Kind, Stack};
use crate::lua::ffi::{
    lua_call, lua_concat, lua_error, lua_pushlstring, lua_tolstring, luaL_argerror,
    luaL_checkinteger, luaL_checklstring, luaL_checknumber, luaL_checktype, luaL_checkudata,
    luaL_where,
};
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

// ---------------------------------------------------------------------------
// Errors

/// A Lua error a C function's body answers instead of raising it.
///
/// [`Stack::entry`] raises it once the body has returned. Building one owns
/// nothing Lua can strand: the message is copied onto the Lua stack and the
/// `Raise` dropped before the jump.
#[derive(Debug)]
#[must_use]
pub struct Raise(Thrown);

#[derive(Debug)]
enum Thrown {
    /// `luaL_error`'s shape: where the running function is, then the message.
    Message(Vec<u8>),
    /// `luaL_argerror`'s: `bad argument #n to 'f' (message)`.
    Argument { arg: c_int, message: Vec<u8> },
    /// A `luaL_check*` that refused, raised by the C check itself.
    Check { arg: c_int, check: Check },
    /// The value already on top of the stack, raised as it is.
    Value,
    /// Not an error: `lua_call(L, args, results)` in tail position.
    TailCall { args: c_int, results: c_int },
}

/// Which `luaL_check*` refused.
#[derive(Debug, Clone, Copy)]
pub(super) enum Check {
    Integer,
    Number,
    String,
    Kind(Kind),
    Userdata(&'static CStr),
}

impl Raise {
    /// `luaL_error(L, "%s", message)`: the message, after the position of the
    /// running function when it is a Lua one (a C function has none, so from
    /// a binding this is the message alone).
    ///
    /// Unlike `luaL_error`'s `%s`, a NUL inside the message is kept.
    pub fn message(message: impl Into<Vec<u8>>) -> Raise {
        Raise(Thrown::Message(message.into()))
    }

    /// `luaL_argerror(L, arg, message)`: `bad argument #arg to 'f' (message)`.
    ///
    /// The message is cut at its first NUL, as `luaL_argerror`'s `%s` cuts it.
    pub fn argument(arg: c_int, message: impl Into<Vec<u8>>) -> Raise {
        Raise(Thrown::Argument {
            arg,
            message: message.into(),
        })
    }

    /// Raise the value on top of the stack as the error, as `lua_error` does:
    /// what a failed [`Stack::pcall`] leaves there, re-raised.
    pub fn top_value() -> Raise {
        Raise(Thrown::Value)
    }

    /// Not an error but the body's last step: call the function below the top
    /// `args` values, unprotected, and answer its `results` values -- what
    /// `return (lua_call(L, args, results), results)` was in C. The call runs
    /// once the body's frame is gone, so an error in it unwinds as Lua raised
    /// it (a message handler further out sees the frames where it happened)
    /// and finds nothing of the body's to strand.
    pub fn tail_call(args: c_int, results: c_int) -> Raise {
        debug_assert!(results >= 0, "a tail call answers a fixed count");
        Raise(Thrown::TailCall { args, results })
    }

    pub(super) fn check(arg: At, check: Check) -> Raise {
        Raise(Thrown::Check { arg: arg.0, check })
    }
}

impl Stack {
    /// Run a C function's `body` over the frame Lua called it with, and raise
    /// the error it answers once it has returned.
    ///
    /// This is the only way a `Stack` raises a Lua error, and why the error
    /// never strands an owned value: by the time it is raised, every local of
    /// `body` has been dropped. Call it as the C function's whole body, so its
    /// own frame owns nothing either.
    ///
    /// # Safety
    /// As [`Stack::from_raw`]: `state` is the state Lua called the C function
    /// on, which is LuaJIT's contract for a `lua_CFunction`.
    pub unsafe fn entry(
        state: *mut lua_State,
        body: impl FnOnce(&mut Stack) -> Result<c_int, Raise>,
    ) -> c_int {
        let outcome = {
            // SAFETY: the caller's promise; this is the frame's only `Stack`.
            let mut stack = Stack {
                state: unsafe { NonNull::new_unchecked(state) },
                base: 0,
            };
            body(&mut stack)
        };
        match outcome {
            Ok(results) => results,
            Err(Raise(Thrown::TailCall { args, results })) => {
                // SAFETY: the state Lua called this function on, with the
                // callee and its arguments on top (the body's promise), and
                // nothing of this frame's left to release.
                unsafe { lua_call(state, args, results) };
                results
            }
            // SAFETY: the state Lua called this function on, with nothing of
            // this frame's left to release.
            Err(error) => unsafe { raise(state, error) },
        }
    }
}

/// Raise `error` on `state` and never come back.
///
/// Every owned part of `error` is copied onto the Lua stack and dropped
/// before the jump.
///
/// # Safety
/// `state` is the live state of the running C function.
#[cold]
#[inline(never)]
unsafe fn raise(state: *mut lua_State, error: Raise) -> ! {
    // SAFETY: the caller's state, throughout.
    unsafe {
        match error.0 {
            Thrown::Message(message) => {
                luaL_where(state, 1);
                lua_pushlstring(state, message.as_ptr().cast::<c_char>(), message.len());
                drop(message);
                lua_concat(state, 2);
            }
            Thrown::Argument { arg, message } => {
                // The copy on the stack is NUL-terminated (every Lua string
                // is) and stays alive for `luaL_argerror` to format.
                lua_pushlstring(state, message.as_ptr().cast::<c_char>(), message.len());
                drop(message);
                let message = lua_tolstring(state, -1, ptr::null_mut());
                luaL_argerror(state, arg, message);
            }
            Thrown::Check { arg, check } => {
                // The C check fails on the same argument the Rust half did,
                // and throws LuaJIT's own message.
                match check {
                    Check::Integer => _ = luaL_checkinteger(state, arg),
                    Check::Number => _ = luaL_checknumber(state, arg),
                    Check::String => _ = luaL_checklstring(state, arg, ptr::null_mut()),
                    Check::Kind(kind) => luaL_checktype(state, arg, kind.to_raw()),
                    Check::Userdata(name) => _ = luaL_checkudata(state, arg, name.as_ptr()),
                }
                // A check that passes after its Rust half refused is a bug
                // in that half; there is no message to raise.
                std::process::abort();
            }
            Thrown::Value => {}
            Thrown::TailCall { .. } => unreachable!("`entry` makes the call"),
        }
        lua_error(state);
    }
    // `lua_error` does not return.
    std::process::abort()
}
