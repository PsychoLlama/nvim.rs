//! `vim.*`: the C half of Neovim's Lua standard library.
//!
//! Every function here is a safe body over a [`Stack`](crate::lua::stack::Stack),
//! set on the `vim` table by [`add_editor_stdlib`], plus the zero value for a
//! [`TryState`] they share.
//!
//! The four jobs are one child each — [`regex`](mod@regex) for `vim.regex()`, [`strings`]
//! for the UTF-8 and `iconv` helpers, [`vars`] for the `vim.g`/`b`/`w`/`t`/`v`
//! accessors, [`with`](mod@with) for `vim._with()` — and [`register`] installs them.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::TryState;

mod regex;
mod register;
mod strings;
mod vars;
mod with;

pub(crate) use self::regex::*;
pub use self::register::*;
pub(crate) use self::strings::*;
pub(crate) use self::vars::*;
pub(crate) use self::with::*;

/// An all-zero [`TryState`], which [`try_enter`] fills.
///
/// [`try_enter`]: crate::api::private::helpers::try_enter
pub(crate) const TRY_STATE_INIT: TryState = TryState::INIT;
