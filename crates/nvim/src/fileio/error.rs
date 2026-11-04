//! What reading a file, matching a file pattern and the file-system helpers
//! refuse.
//!
//! [`ReadError`] is what `open_source` answers when the read cannot go
//! ahead; `readfile` reports it, once. [`PatternError`] is a file pattern
//! whose braces do not balance; each caller of `file_pat_to_reg_pat`
//! reports it where it decides what to do instead. [`IoFailed`] is the
//! copy, rename and delete helpers' failure, which upstream never reports:
//! the caller answers its own sentinel (`filecopy()`'s 0, `rename()`'s -1)
//! or says why itself.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::message_fmt::{emsg_text, msg_cstr};
use crate::os::cshim::gettext;
use crate::tr;
use crate::types::Failed;

/// Why `readfile` did not read the file.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadError {
    /// The swap-file autocommands (`SwapExists`) changed the buffer or its
    /// name.
    #[error("{}", tr!("E812: Autocommands changed buffer or buffer name"))]
    AutocmdsChangedBuffer,
    /// The `*ReadPre` autocommands removed the file, or made it unreadable.
    #[error("{}", tr!("E200: *ReadPre autocommands made the file unreadable"))]
    ReadPreUnreadable,
    /// The `*ReadPre` autocommands changed the current buffer.
    #[error("{}", tr!("E201: *ReadPre autocommands must not change current buffer"))]
    ReadPreChangedBuffer,
    /// Refused with only a note on the screen (a name too long, not a
    /// file, permission denied), or by an autocommand that aborted.
    #[error(transparent)]
    Reported(#[from] Failed),
}

const _: () = assert!(size_of::<ReadError>() <= 16);

impl ReadError {
    /// Puts the message out, unless it is already out.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Reported(_)) {
            emsg_text(self.to_string());
        }
    }
}

/// A file pattern whose braces do not balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    // Through `gettext`: the braces would have to be doubled in a `tr!`
    // literal, which is then another msgid.
    /// More `}` than `{`.
    #[error("{}", msg_cstr(gettext(c"E219: Missing {.")))]
    MissingOpen,
    /// More `{` than `}`.
    #[error("{}", msg_cstr(gettext(c"E220: Missing }.")))]
    MissingClose,
}

impl PatternError {
    /// Puts the message out.
    pub(crate) fn report(self) {
        emsg_text(self.to_string());
    }
}

/// A copy, rename or delete that did not happen. Nothing has been said:
/// upstream reports none of these, and the caller decides what to say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the file operation failed")]
pub(crate) struct IoFailed;
