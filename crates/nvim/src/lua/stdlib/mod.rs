//! `vim.*`: the C half of Neovim's Lua standard library.
//!
//! Everything here is a `lua_CFunction` registered onto the `vim` table by
//! [`nlua_state_add_stdlib`], plus the two pieces of plumbing they share: the
//! zero values for an [`Error`] and a [`TryState`], and
//! [`nlua_push_errstr`](register::nlua_push_errstr), which is `luaL_error`
//! split in half so a caller can clean up between formatting the message and
//! throwing it.
//!
//! The four jobs are one child each — [`regex`] for `vim.regex()`, [`strings`]
//! for the UTF-8 and `iconv` helpers, [`vars`] for the `vim.g`/`b`/`w`/`t`/`v`
//! accessors, [`with`] for `vim._with()` — and [`register`] installs them.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::{Error, TryState};

mod regex;
mod register;
mod strings;
mod vars;
mod with;

pub use self::regex::*;
pub use self::register::*;
pub use self::strings::*;
pub use self::vars::*;
pub(crate) use self::with::*;

/// An all-zero [`Error`], C's `ERROR_INIT`.
pub(crate) const ERROR_INIT: Error = Error::none();

/// An all-zero [`TryState`], which [`try_enter`] fills.
///
/// [`try_enter`]: crate::api::private::helpers::try_enter
pub(crate) const TRY_STATE_INIT: TryState = TryState {
    current_exception: ::core::ptr::null_mut(),
    private_msg_list: ::core::ptr::null_mut(),
    msg_list: ::core::ptr::null(),
    got_int: 0,
    did_throw: false,
    need_rethrow: 0,
    did_emsg: 0,
};

/// Whether an API call left an error behind: C's `ERROR_SET`.
pub(crate) fn error_set(err: &Error) -> bool {
    err.is_set()
}
