//! What the match list refuses. The list operations answer a
//! [`MatchError`]; `:match` and the `match*()` builtins say it, once, with
//! upstream's text.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::eval::typval::Unconvertible;
use crate::memory::ThinCString;
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::tr;
use crate::types::Failed;

/// Why a match was not added or removed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum MatchError {
    /// An empty group or pattern: nothing to add, and nothing said, as
    /// upstream's `-1`.
    #[error("")]
    Empty,
    /// An id below 1 that is not `-1` ("allocate one").
    #[error("{}", tr!("E799: Invalid ID: {} (must be greater than or equal to 1)", i64::from(*.0)))]
    InvalidId(c_int),
    /// The id is in use.
    #[error("{}", tr!("E801: ID already taken: {}", i64::from(*.0)))]
    IdTaken(c_int),
    /// The pattern did not compile, which the compiler has said; this
    /// follows it.
    #[error("{}", tr!("E475: Invalid argument: {}", msg_cstr(.0.as_cstr())))]
    Pattern(ThinCString),
    /// A position entry that is an empty list.
    #[error("{}", tr!("E5030: Empty list at position {}", .0))]
    EmptyPosition(c_int),
    /// A position entry that is neither a list nor a number.
    #[error("{}", tr!("E5031: List or number required at position {}", .0))]
    NotAPosition(c_int),
    /// Refused by code that has said why: the group name, a position that
    /// is not a number.
    #[error("")]
    Reported(#[from] Failed),
    /// Deleting an id below 1.
    #[error("{}", tr!("E802: Invalid ID: {} (must be greater than or equal to 1)", i64::from(*.0)))]
    DeleteInvalidId(c_int),
    /// Deleting an id that is not in the list.
    #[error("{}", tr!("E803: ID not found: {}", i64::from(*.0)))]
    NoSuchId(c_int),
}

const _: () = assert!(size_of::<MatchError>() <= 16);

impl From<Unconvertible> for MatchError {
    fn from(Unconvertible: Unconvertible) -> Self {
        Self::Reported(Failed)
    }
}

impl MatchError {
    /// Put the message out, if the refusal has one left to say.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Empty | Self::Reported(_)) {
            emsg_text(self.to_string());
        }
    }
}
