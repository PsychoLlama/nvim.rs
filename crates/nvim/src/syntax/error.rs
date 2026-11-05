//! What the `:syntax` parsers refuse.
//!
//! The parsers below the subcommands — the item options, the group lists,
//! the patterns, the keyword notation, `linecont=` — answer a
//! [`SyntaxError`]; the subcommand says it, once, and then its own E475 or
//! E404 as upstream did. The texts are upstream's, byte for byte.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::fmt;

use crate::memory::ThinCString;
use crate::message_fmt::{BytesDisplay, emsg_text, msg_bytes};
use crate::tr;
use crate::types::Failed;

/// Why a `:syntax` argument was refused.
#[derive(Debug)]
pub(crate) enum SyntaxError {
    /// `contains=` on an item that takes no list (`:syntax keyword`).
    ContainsNotAccepted,
    /// A `cchar=` that is not a printable character.
    InvalidCchar,
    /// `grouphere`/`groupthere` outside `:syntax sync match`.
    SyncGroupNotAccepted,
    /// `grouphere {group}` names a group without a region start.
    NoRegionItem(ThinCString),
    /// A group list keyword without its `=`; the text from the keyword.
    MissingEqualSign(ThinCString),
    /// A group list keyword with nothing after its `=`.
    EmptyArgument(ThinCString),
    /// `ALL`/`ALLBUT`/`TOP`/`CONTAINED` where only `contains=` and
    /// `containedin=` take them.
    NotAllowedHere(ThinCString),
    /// `ALL`/... after another name of the same list.
    MustBeFirst(ThinCString),
    /// A group list name that resolves to no group; the text from the name.
    UnknownGroup(ThinCString),
    /// A pattern whose closing delimiter is missing.
    NoDelimiter(ThinCString),
    /// Something other than an offset or white space after a pattern.
    GarbageAfterPattern(ThinCString),
    /// `ab[cd` in a `:syntax keyword`; the keyword as far as it was read.
    MissingBracket(ThinCString),
    /// `ab[cd]e`: the keyword, and what follows its `]`.
    TrailingAfterBracket(Box<(ThinCString, ThinCString)>),
    /// A second `linecont=` pattern.
    LineContTwice,
    /// The cluster table is full.
    TooManyClusters,
    /// Refused with nothing said, as upstream's silent `NULL`: a pattern
    /// too short to have two delimiters, an empty `grouphere` name. The
    /// subcommand's own E475 is all the user sees.
    Unexplained,
    /// Refused by code that has said why: the pattern compiler, the group
    /// name table.
    Reported(Failed),
}

const _: () = assert!(size_of::<SyntaxError>() <= 16);

impl From<Failed> for SyntaxError {
    fn from(failed: Failed) -> Self {
        Self::Reported(failed)
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The literals are upstream's msgids, named arguments and all.
        let text = match self {
            Self::ContainsNotAccepted => tr!("E395: Contains argument not accepted here"),
            Self::InvalidCchar => tr!("E844: Invalid cchar value"),
            Self::SyncGroupNotAccepted => tr!("E393: group[t]here not accepted here"),
            Self::NoRegionItem(name) => {
                let shown = shown(name);
                tr!("E394: Didn't find region item for {shown}")
            }
            Self::MissingEqualSign(text) => {
                let shown = shown(text);
                tr!("E405: Missing equal sign: {shown}")
            }
            Self::EmptyArgument(text) => {
                let shown = shown(text);
                tr!("E406: Empty argument: {shown}")
            }
            Self::NotAllowedHere(name) => {
                let shown = shown(name);
                tr!("E407: {shown} not allowed here")
            }
            Self::MustBeFirst(name) => {
                let shown = shown(name);
                tr!("E408: {shown} must be first in contains list")
            }
            Self::UnknownGroup(name) => {
                let shown = shown(name);
                tr!("E409: Unknown group name: {shown}")
            }
            Self::NoDelimiter(text) => {
                let shown = shown(text);
                tr!("E401: Pattern delimiter not found: {shown}")
            }
            Self::GarbageAfterPattern(text) => {
                let shown = shown(text);
                tr!("E402: Garbage after pattern: {shown}")
            }
            Self::MissingBracket(kw) => {
                let kw = shown(kw);
                tr!("E789: Missing ']': {kw}")
            }
            Self::TrailingAfterBracket(parts) => {
                let (kw, arg1) = (shown(&parts.0), shown(&parts.1));
                tr!("E890: Trailing char after ']': {kw}]{arg1}")
            }
            Self::LineContTwice => {
                tr!("E403: syntax sync: line continuations pattern specified twice")
            }
            Self::TooManyClusters => tr!("E848: Too many syntax clusters"),
            Self::Unexplained | Self::Reported(_) => String::new(),
        };
        f.write_str(&text)
    }
}

impl core::error::Error for SyntaxError {}

/// A payload as the message shows it.
fn shown(text: &ThinCString) -> BytesDisplay<'_> {
    msg_bytes(text.as_bytes())
}

impl SyntaxError {
    /// Put the message out, if the refusal has one left to say.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Unexplained | Self::Reported(_)) {
            emsg_text(self.to_string());
        }
    }
}
