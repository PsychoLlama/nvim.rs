//! What a search refuses, and when it says so.
//!
//! [`searchit`](super::searchit) is the entry point that reports: the walk
//! under it answers a [`SearchError`], and `searchit` says what the
//! caller's `SEARCH_MSG` bits ask for, once, with upstream's text.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{SEARCH_MSG, get_search_pat};
use crate::message::{E_INTERR, emsg};
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::os::cshim::gettext;
use crate::regexp::Refused;
use crate::{semsg, tr};

/// Where a search that found nothing gave up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    /// It went round the whole buffer ('wrapscan').
    Wrapped,
    /// It reached the top of the buffer.
    Top,
    /// It reached the bottom of the buffer.
    Bottom,
}

/// Why a search found nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchError {
    /// The pattern did not compile, or there was no remembered one: the
    /// compiler or [`search_regcomp`](super::search_regcomp) has said so,
    /// and whether it blamed the pattern.
    Pattern(Refused),
    /// CTRL-C.
    Interrupted,
    /// No match.
    NotFound(Edge),
}

const _: () = assert!(size_of::<SearchError>() <= 16);

impl SearchError {
    /// Say what `options`' `SEARCH_MSG` bits ask for: E383 for a pattern
    /// the compiler did not blame under any of them, "not found" only under
    /// both. An interrupt is always said.
    pub(crate) fn report(self, options: c_int) {
        match self {
            Self::Pattern(Refused::Unexplained) if options & SEARCH_MSG != 0 => {
                let pattern = get_search_pat();
                let arg0 = msg_bytes(&pattern);
                semsg!("E383: Invalid search string: {arg0}");
            }
            Self::Pattern(_) => {}
            Self::Interrupted => {
                emsg(gettext(E_INTERR));
            }
            Self::NotFound(edge) if options & SEARCH_MSG == SEARCH_MSG => {
                let pattern = get_search_pat();
                let pat = msg_bytes(&pattern);
                emsg_text(match edge {
                    Edge::Wrapped => tr!("E486: Pattern not found: {pat}"),
                    Edge::Top => tr!("E384: Search hit TOP without match for: {pat}"),
                    Edge::Bottom => tr!("E385: Search hit BOTTOM without match for: {pat}"),
                });
            }
            Self::NotFound(_) => {}
        }
    }
}

/// Nothing matched: no line for line completion, no match for `gn` to
/// select. Nothing is said about it -- only a pattern refused on the way
/// was, by its compiler; the caller beeps or moves on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoMatch;
