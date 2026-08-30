#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

/// The `int` return convention almost every editor function still speaks:
/// `OK` on success, `FAIL` on failure. `vim_defs.h`'s, one copy for the tree.
pub const OK: ::core::ffi::c_int = 1;
/// The failing half of the [`OK`] convention.
pub const FAIL: ::core::ffi::c_int = 0;

/// What `FAIL` said, as an error type: *it did not work*, and nothing more.
///
/// The overwhelming majority of the editor's status-code functions carry no
/// error information at all -- the message, if there is one, was already
/// shown by whichever call actually failed, and the caller's only decision is
/// whether to keep going. `Result<(), Failed>` says exactly that, and says it
/// in a type the `?` operator can thread, which `c_int` cannot.
///
/// `Result<(), ()>` would be the same shape and a worse one: the error half
/// would have no name, no `Display`, and no place to hang a variant when a
/// caller does eventually need to tell two failures apart. When a domain
/// *does* have something to say -- `quickfix::QfError`, `indent::ParseError`,
/// `channel::CloseError` -- it gets its own enum with a `From<Failed>` where
/// the two meet, rather than another unit type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the operation failed")]
pub struct Failed;

pub type CdCause = ::core::ffi::c_int;
pub type CdScope = ::core::ffi::c_int;
/// Which scope a `:cd` applies to (`getcwd()`/`haslocaldir()` report it).
pub const kCdScopeInvalid: CdScope = -1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeGlobal: CdScope = 2;
pub type Direction = ::core::ffi::c_int;
