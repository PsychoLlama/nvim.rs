//! The C variadics, behind a type gate.
//!
//! A C variadic passes what it is handed, byte for byte: nothing checks an
//! argument against the format string, and Rust accepts any sized value in
//! the variadic position. An owned or fat value -- an `XString`, a `Vec<u8>`,
//! a `&CStr` -- lowers into the argument area as two or three words, and a
//! `%s` reads whichever word the ABI put first. The tree shipped that twice:
//! `spell_load_lang` handed `vim_snprintf` an `XString` (garbage in a release
//! build, the data pointer in a debug one, so every debug lane was green), and
//! `aucmd_next` handed `snprintf` a `&CStr` (a fault in a debug build). The
//! compiler said nothing either time.
//!
//! Every variadic the tree calls has a macro here of the same name, which
//! binds each variadic argument through [`c_arg`] and so through [`CArg`]:
//! the thin scalars and raw pointers a `printf` conversion actually consumes.
//! Anything else is a compile error at the call site, where the fix is
//! `.as_ptr()`:
//!
//! ```compile_fail,E0277
//! // `spell_load_lang`'s shape: the owner, not its bytes.
//! use core::ffi::c_char;
//! use neovim::{memory::XString, vim_snprintf};
//! fn spell_file(buf: *mut c_char, room: usize, lang: *const c_char, enc: &XString) {
//!     unsafe { vim_snprintf!(buf, room, c"spell/%s.%s.spl".as_ptr(), lang, enc) };
//! }
//! ```
//!
//! ```compile_fail,E0277
//! // `aucmd_next`'s shape: a `&CStr` is a fat reference.
//! use core::ffi::{CStr, c_char};
//! use neovim::snprintf;
//! fn sourcing_name(out: *mut c_char, len: usize, name: &CStr, pat: *const c_char) {
//!     unsafe { snprintf!(out, len, c"%s Autocommands for \"%s\"".as_ptr(), name, pat) };
//! }
//! ```
//!
//! The macros expand to the very call they replace -- [`c_arg`] is the
//! identity, `#[inline(always)]` -- so a call costs what it cost before. They
//! do not check the *format*: a `%s` handed a `c_int` is still the caller's
//! `unsafe` to get right, which is why every one of them still has to sit in
//! an `unsafe` block.
//!
//! The ratchet's `raw_variadic_calls` counts the direct calls left, which is
//! the perimeter's: [`direct`] is the one path the macros reach the callees
//! through, and the needle does not count it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_long, c_uint, c_ulong};

/// The argument types a C variadic can carry to a `printf` conversion: the
/// thin scalars and raw pointers, and deliberately not `&CStr`, `XString`,
/// `Vec<u8>`, `String`, `Option<_>` or any reference -- each of those either
/// lowers as more than one word or is not the pointer the callee reads.
pub trait CArg {}

macro_rules! c_arg_scalars {
    ($($t:ty),* $(,)?) => { $(impl CArg for $t {})* };
}

c_arg_scalars!(c_int, c_uint, c_long, c_ulong, usize, isize, f64);

impl<T> CArg for *const T {}
impl<T> CArg for *mut T {}

/// A variadic argument, checked: the identity, bounded by [`CArg`].
#[doc(hidden)]
#[inline(always)]
pub const fn c_arg<T: CArg>(arg: T) -> T {
    arg
}

/// The callees themselves, for the macros' expansions. Calling one of these
/// by hand is the unchecked call the macros exist to replace.
#[doc(hidden)]
pub mod direct {
    pub use crate::lua::ffi::{lua_pushfstring, luaL_error};
    pub use crate::lua::stdlib::nlua_push_errstr;
    pub use crate::os::cshim::snprintf;
    pub use crate::strings::{printf_string, vim_snprintf, vim_snprintf_add, vim_snprintf_safelen};
    pub use ::libc::{fprintf, printf, sscanf};
}

/// `vim_snprintf(buf, len, fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! vim_snprintf {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::vim_snprintf(
            $buf, $len, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// `vim_snprintf_add(buf, len, fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! vim_snprintf_add {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::vim_snprintf_add(
            $buf, $len, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// `vim_snprintf_safelen(buf, len, fmt, args...)`, each argument bound by
/// [`CArg`].
#[macro_export]
macro_rules! vim_snprintf_safelen {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::vim_snprintf_safelen(
            $buf, $len, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// libc's `snprintf(buf, len, fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! snprintf {
    ($buf:expr, $len:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::snprintf(
            $buf, $len, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// libc's `fprintf(file, fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! fprintf {
    ($file:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::fprintf($file, $fmt $(, $crate::variadic::c_arg($arg))*)
    };
}

/// libc's `printf(fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! printf {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::printf($fmt $(, $crate::variadic::c_arg($arg))*)
    };
}

/// libc's `sscanf(text, fmt, out...)`, each out-pointer bound by [`CArg`].
#[macro_export]
macro_rules! sscanf {
    ($text:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::sscanf($text, $fmt $(, $crate::variadic::c_arg($arg))*)
    };
}

/// `printf_string(fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! printf_string {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::printf_string($fmt $(, $crate::variadic::c_arg($arg))*)
    };
}

/// `nlua_push_errstr(lstate, fmt, args...)`, each argument bound by [`CArg`].
#[macro_export]
macro_rules! nlua_push_errstr {
    ($lstate:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::nlua_push_errstr(
            $lstate, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// LuaJIT's `lua_pushfstring(lstate, fmt, args...)`, each argument bound by
/// [`CArg`].
#[macro_export]
macro_rules! lua_pushfstring {
    ($lstate:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::lua_pushfstring(
            $lstate, $fmt $(, $crate::variadic::c_arg($arg))*
        )
    };
}

/// LuaJIT's `luaL_error(lstate, fmt, args...)`, each argument bound by
/// [`CArg`].
#[macro_export]
macro_rules! luaL_error {
    ($lstate:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::variadic::direct::luaL_error($lstate, $fmt $(, $crate::variadic::c_arg($arg))*)
    };
}
