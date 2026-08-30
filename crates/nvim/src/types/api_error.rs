//! `api/private/defs.h`'s `Error`: what an `nvim_*` function refuses with.
//!
//! Upstream's is a `{ ErrorType type; char *msg; }` pair whose message is an
//! `xmalloc`'d C string the caller frees by hand, passed as a `*mut Error`
//! out-parameter into every API function and half the editor besides. This
//! one owns its message, is not `Copy`, and implements [`std::error::Error`],
//! so an API function answers `Result<T, Error>` and `?` composes.
//!
//! **The layout is free.** Nothing in `metrics/abi-ledger.jsonl` names
//! `Error`, no unit spec constructs or reads one, and the RPC codec
//! serialises the *message text*, never the struct. The one place the type
//! crosses a language boundary is the `Error *` parameter of
//! [`ApiDispatchWrapper`](super::ApiDispatchWrapper), which is a pointer the
//! other side only ever passes back. So there is no `#[repr(C)]` here and
//! `tools/ffigen` emits the name as an opaque forward declaration.
//!
//! # The message's bytes
//!
//! An API error quotes file names, patterns and buffer text, none of which is
//! guaranteed to be UTF-8, so the message is held as a [`CString`] and read
//! back as a [`CStr`]: byte for byte, the way `vsnprintf` left it.
//! [`api_error!`](crate::api_error) renders through
//! [`message_fmt`](crate::message_fmt), whose writer escapes a non-UTF-8 byte
//! on the way in and restores it on the way out.
//!
//! [`Display`] is *lossy*, and is there for [`std::error::Error`]'s sake.
//! Anything that puts the message on the screen or on the wire reads
//! [`message_or_empty`](Error::message_or_empty) instead.
//!
//! [`Display`]: core::fmt::Display
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{ErrorType, kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::message_fmt::to_message;
use core::ffi::CStr;
use core::fmt;
use std::ffi::CString;

/// The longest message an API error carries, terminator included -- the cap
/// upstream's `api_set_error` put on its `xmalloc`.
const MAXLEN: usize = 1024 * 1024;

/// Why an `nvim_*` function refused, and what to tell the client.
///
/// A value with [`kErrorTypeNone`] carries no failure: that is what
/// [`none`](Error::none) builds, and what the out-parameter slots the
/// migration has not reached yet start out holding. An `Error` inside an
/// `Err` always carries one.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{}", .msg.as_deref().unwrap_or(c"").to_string_lossy())]
pub struct Error {
    kind: ErrorType,
    msg: Option<CString>,
}

impl Error {
    /// An error that is not set: upstream's `ERROR_INIT`.
    pub(crate) const fn none() -> Self {
        Self {
            kind: kErrorTypeNone,
            msg: None,
        }
    }

    /// A failure of kind `kind`, with the message `args` renders.
    ///
    /// Spelled through [`api_error!`](crate::api_error), which checks the
    /// literal for a leftover C conversion first. The message is truncated to
    /// [`MAXLEN`] and ends at an interior NUL, exactly where
    /// `api_set_error`'s `vsnprintf` ended it.
    pub(crate) fn new(kind: ErrorType, args: fmt::Arguments<'_>) -> Self {
        debug_assert!(kind != kErrorTypeNone);
        Self {
            kind,
            msg: Some(to_message(args.to_string(), MAXLEN)),
        }
    }

    /// A failure of kind `kind`, carrying `msg` as its message.
    pub(crate) fn from_message(kind: ErrorType, msg: &CStr) -> Self {
        debug_assert!(kind != kErrorTypeNone);
        let mut bytes = msg.to_bytes();
        bytes = &bytes[..bytes.len().min(MAXLEN - 1)];
        Self {
            kind,
            msg: Some(CString::new(bytes).unwrap_or_default()),
        }
    }

    /// `kErrorTypeException`: the call failed. Vim's own errors, and
    /// whatever a Lua callback threw, arrive as this.
    pub(crate) fn exception(msg: &CStr) -> Self {
        Self::from_message(kErrorTypeException, msg)
    }

    /// `kErrorTypeValidation`: the call's arguments were wrong.
    pub(crate) fn validation(msg: &CStr) -> Self {
        Self::from_message(kErrorTypeValidation, msg)
    }

    /// Which kind of failure this is, or [`kErrorTypeNone`] for none.
    pub(crate) fn kind(&self) -> ErrorType {
        self.kind
    }

    /// Whether this carries a failure at all: C's `ERROR_SET`.
    pub(crate) fn is_set(&self) -> bool {
        self.kind != kErrorTypeNone
    }

    /// The message, or the empty string, for the reporting paths that had a
    /// null-tolerant `%s` in C.
    pub(crate) fn message_or_empty(&self) -> &CStr {
        self.msg.as_deref().unwrap_or(c"")
    }

    /// Forget whatever is set: upstream's `api_clear_error`, which no longer
    /// has to free anything.
    pub(crate) fn clear(&mut self) {
        *self = Self::none();
    }

    /// Move the failure out of a slot, leaving it unset. `None` when the slot
    /// carried none.
    ///
    /// This is how a function that still lends an out-parameter to a callee
    /// turns what the callee left there into its own `Err`.
    pub(crate) fn take(&mut self) -> Option<Self> {
        self.is_set().then(|| core::mem::take(self))
    }
}

/// Build an [`Error`] from a checked format literal.
///
/// `api_error!(kErrorTypeValidation, "Invalid buffer id: {id}")` replaces
/// upstream's `api_set_error(err, kErrorTypeValidation, "Invalid buffer id:
/// %d", id)`: `format_args!` checks the placeholders against the arguments,
/// and the `const` block rejects a literal that still holds a C conversion,
/// so a half-finished conversion fails the build rather than printing
/// itself.
///
/// Unlike the message macros this does *not* consult the catalogue. Upstream
/// never translated an API error -- a client reads it, not a user -- and the
/// text is asserted verbatim by the functional suite.
///
/// A `%s` argument that may not be UTF-8 goes through
/// [`msg_cstr`](crate::message_fmt::msg_cstr) or its siblings, which is what
/// keeps the bytes intact across the render.
#[macro_export]
macro_rules! api_error {
    ($kind:expr, $lit:literal $(, $arg:expr)* $(,)?) => {{
        const { $crate::message_fmt::check_template($lit) };
        $crate::types::Error::new($kind, ::core::format_args!($lit $(, $arg)*))
    }};
}

impl Default for Error {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_not_set() {
        let err = Error::none();
        assert!(!err.is_set());
        assert_eq!(err.message_or_empty(), c"");
        assert_eq!(err.kind(), kErrorTypeNone);
    }

    #[test]
    fn a_message_keeps_its_bytes() {
        let err = Error::from_message(kErrorTypeValidation, c"caf\xe9");
        assert_eq!(err.kind(), kErrorTypeValidation);
        assert_eq!(err.message_or_empty().to_bytes(), b"caf\xe9");
    }

    #[test]
    fn a_formatted_message_keeps_its_bytes() {
        let name = crate::message_fmt::msg_bytes(b"caf\xe9");
        let err = api_error!(kErrorTypeValidation, "Invalid file: {name}");
        assert_eq!(err.kind(), kErrorTypeValidation);
        assert_eq!(err.message_or_empty().to_bytes(), b"Invalid file: caf\xe9");
    }

    #[test]
    fn take_empties_the_slot() {
        let mut slot = api_error!(kErrorTypeException, "boom");
        let taken = slot.take().expect("set");
        assert_eq!(taken.message_or_empty(), c"boom");
        assert!(!slot.is_set());
        assert!(slot.take().is_none());
    }

    #[test]
    fn a_formatted_message_ends_at_an_interior_nul() {
        let err = api_error!(kErrorTypeException, "a{}b", '\0');
        assert_eq!(err.message_or_empty(), c"a");
    }

    #[test]
    fn display_is_lossy_where_the_message_is_not() {
        let err = Error::from_message(kErrorTypeException, c"\xff");
        assert_eq!(err.message_or_empty().to_bytes(), b"\xff");
        assert_eq!(err.to_string(), "\u{fffd}");
    }
}
