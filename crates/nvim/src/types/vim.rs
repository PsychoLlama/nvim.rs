#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

/// A failure whose message is already out: the call that failed reported it
/// through Vim's `emsg` channel, and the caller's only decision is whether to
/// keep going.
///
/// `emsg` is the editor's reporting boundary, not a stage on the way to one.
/// It sets `v:errmsg`, raises the exception `:try` catches, and honours
/// `EMSG_SILENT` and `'debug'` *where the failure happened*, so carrying the
/// message up to a caller and reporting it there would change when that
/// state is set. `Err(Failed)` is therefore the honest answer after an
/// `emsg`, where a `false`, `-1` or `None` would be indistinguishable from a
/// real result (the ratchet's `emit_sentinel` counts those).
///
/// It is not a place to put a message that has *not* been shown. A family
/// whose callers need to tell failures apart, or that has one reporting
/// boundary of its own -- `quickfix::QfError`, `indent::ParseError`,
/// `channel::CloseError` -- gets its own error enum with a `From<Failed>`
/// where the two meet, rather than another unit type.
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
