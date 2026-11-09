//! What the highlight group table and `:highlight` refuse.
//!
//! The group table ([`super::syn_check_group`]) refuses a name it cannot
//! intern and says why itself, answering 0 — the id no group has, which its
//! many callers already read as "no highlight". `:highlight`'s key loop
//! answers a [`HighlightError`] and [`super::do_highlight`] says it, once.
//! The texts are upstream's, byte for byte.

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
use crate::message::{
    E_HIGHLIGHT_GROUP_NAME_INVALID_CHAR, E_HIGHLIGHT_GROUP_NAME_TOO_LONG, msg_source,
};
use crate::message_fmt::{CDisplay, emsg_text, msg_bytes, msg_cstr};
use crate::os::cshim::gettext;
use crate::tr;

use super::HLF_W;

/// Why a group name was not interned, or a `:highlight` argument refused.
#[derive(Debug)]
pub(crate) enum HighlightError {
    /// A group name over `MAX_SYN_NAME` bytes.
    NameTooLong,
    /// A group name with a byte that is not printable.
    UnprintableName,
    /// A group name with a byte that is not a letter, digit, `_`, `.`, `@`
    /// or `-`.
    InvalidNameChar,
    /// The group table is full.
    TooManyGroups,
    /// The attribute table is full even after a rebuild.
    TooManyAttributes,
    /// A key that starts with `=`; the rest of the line from it.
    UnexpectedEqualSign(ThinCString),
    /// A key without its `=`.
    MissingEqualSign(ThinCString),
    /// A `'quoted` value without its closing quote.
    InvalidArgument(ThinCString),
    /// A key with nothing after its `=`.
    MissingArgument(ThinCString),
    /// A key or a value too long to be one.
    TooLong,
    /// A key `:highlight` does not know.
    IllegalKey(ThinCString),
    /// An attribute list naming an attribute that does not exist; the list.
    IllegalValue(ThinCString),
    /// `ctermfg=fg` while `Normal` has no foreground colour.
    FgUnknown,
    /// `ctermbg=bg` while `Normal` has no background colour.
    BgUnknown,
    /// A `cterm` colour that is neither a number nor a colour name.
    ColorNotRecognized(ThinCString),
}

const _: () = assert!(size_of::<HighlightError>() <= 16);

/// A payload as the message shows it, from the C string it was cut from.
fn shown(text: &ThinCString) -> CDisplay<'_> {
    msg_cstr(text.as_cstr())
}

impl fmt::Display for HighlightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The literals are upstream's msgids, named arguments and all.
        let text = match self {
            Self::NameTooLong => {
                return write!(f, "{}", msg_cstr(gettext(E_HIGHLIGHT_GROUP_NAME_TOO_LONG)));
            }
            Self::UnprintableName => tr!("E669: Unprintable character in group name"),
            Self::InvalidNameChar => {
                return write!(
                    f,
                    "{}",
                    msg_cstr(gettext(E_HIGHLIGHT_GROUP_NAME_INVALID_CHAR))
                );
            }
            Self::TooManyGroups => tr!("E849: Too many highlight and syntax groups"),
            Self::TooManyAttributes => {
                tr!("E424: Too many different highlighting attributes in use")
            }
            Self::UnexpectedEqualSign(rest) => {
                let arg0 = shown(rest);
                tr!("E415: Unexpected equal sign: {arg0}")
            }
            Self::MissingEqualSign(rest) => {
                let arg0 = shown(rest);
                tr!("E416: Missing equal sign: {arg0}")
            }
            Self::InvalidArgument(rest) => {
                let arg0 = shown(rest);
                tr!("E475: Invalid argument: {arg0}")
            }
            Self::MissingArgument(rest) => {
                let arg0 = shown(rest);
                tr!("E417: Missing argument: {arg0}")
            }
            Self::TooLong => tr!("E423: Illegal argument"),
            Self::IllegalKey(rest) => {
                let key_start = shown(rest);
                tr!("E423: Illegal argument: {key_start}")
            }
            Self::IllegalValue(arg) => tr!("E418: Illegal value: {}", msg_bytes(arg.as_bytes())),
            Self::FgUnknown => tr!("E419: FG color unknown"),
            Self::BgUnknown => tr!("E420: BG color unknown"),
            Self::ColorNotRecognized(rest) => {
                let key_start = shown(rest);
                tr!("E421: Color name or number not recognized: {key_start}")
            }
        };
        f.write_str(&text)
    }
}

impl core::error::Error for HighlightError {}

impl HighlightError {
    /// Put the message out.
    #[cold]
    #[inline(never)]
    pub(crate) fn report(&self) {
        if matches!(self, Self::InvalidNameChar) {
            // Upstream names the script and line first, as a warning.
            msg_source(HLF_W);
        }
        emsg_text(self.to_string());
    }
}
