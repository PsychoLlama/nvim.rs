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

use super::{ErrorType, kErrorTypeNone};
use core::ffi::CStr;
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
}

impl Default for Error {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{kErrorTypeException, kErrorTypeValidation};

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
    fn display_is_lossy_where_the_message_is_not() {
        let err = Error::from_message(kErrorTypeException, c"\xff");
        assert_eq!(err.message_or_empty().to_bytes(), b"\xff");
        assert_eq!(err.to_string(), "\u{fffd}");
    }
}
