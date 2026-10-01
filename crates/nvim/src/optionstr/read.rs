//! [`OptString`]: the questions a string option's value answers, wherever
//! the value lives.
//!
//! A string option's value is in one of three places — the option record
//! (a global value, [`StrOpt`]), a window's, buffer's or syntax block's own
//! field (`Option<XString>`), or "the local copy where it is set, the
//! global one where not" ([`local_or_global`]) — plus [`StrVar`], which
//! names one of the first two by selector. The readers ask all of them the
//! same five things, so they ask through one trait rather than three
//! look-alike APIs.
//!
//! A field can do one thing the others cannot: lend out its bytes, because
//! the borrow is the caller's own. That is [`OptStringRef`]. The option
//! record's value is behind a cell and is only ever *projected*
//! (`p_xx(|value| ...)`), so it lends nothing.
//!
//! `None` is upstream's shared empty string on either side: the option owns
//! nothing and holds no value of its own, so a global-local option falls
//! back and a `:setlocal` reads as unset.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char};

use super::empty_option;
use crate::memory::XString;
use crate::options::vars::StrOpt;

/// The questions every string option's value answers, wherever it lives.
pub(crate) trait OptString {
    /// The value as the `char *` the option protocol and the C callees
    /// still speak, which is the shared empty string when the variable owns
    /// nothing.
    ///
    /// **The pointer is the variable's own buffer**, and lives until the
    /// variable is written — not until the end of the caller's statement. A
    /// reader that keeps it across anything that can set an option is
    /// reading freed bytes; take [`get`](Self::get) instead.
    fn value_ptr(&self) -> *mut c_char;

    /// The value's first byte, which is 0 for an empty value — upstream's
    /// `*p` on a variable that is never null. Constant time.
    fn first_byte(&self) -> u8;

    /// Whether the value holds `byte` — upstream's `vim_strchr(p, c) !=
    /// NULL`, which is how every *letter* option is queried.
    fn has_byte(&self, byte: u8) -> bool;

    /// Whether the variable owns no string of its own — upstream's
    /// `is_empty_option`. **Not** "the value is empty": an option explicitly
    /// set to `""` owns an empty string.
    fn is_unset(&self) -> bool;

    /// A copy of the value, for a caller that needs it to outlive the
    /// variable's next write.
    fn get(&self) -> XString;
}

/// A value whose bytes the caller can borrow outright: a window's, buffer's
/// or syntax block's own field, which the caller already holds a borrow of.
pub(crate) trait OptStringRef: OptString {
    /// The value's bytes, without the terminator.
    fn bytes(&self) -> &[u8];

    /// The value as a borrowed C string, which is the shared empty one when
    /// the field owns nothing -- [`OptString::value_ptr`] with the
    /// terminator's whereabouts written into the type.
    fn cstr(&self) -> &CStr;
}

impl OptString for Option<XString> {
    fn value_ptr(&self) -> *mut c_char {
        self.as_ref()
            .map_or_else(empty_option, |value| value.as_ptr().cast_mut())
    }

    fn first_byte(&self) -> u8 {
        self.bytes().first().copied().unwrap_or(0)
    }

    fn has_byte(&self, byte: u8) -> bool {
        self.bytes().contains(&byte)
    }

    fn is_unset(&self) -> bool {
        self.is_none()
    }

    fn get(&self) -> XString {
        self.clone().unwrap_or_default()
    }
}

impl OptStringRef for Option<XString> {
    fn bytes(&self) -> &[u8] {
        self.as_deref().unwrap_or_default()
    }

    fn cstr(&self) -> &CStr {
        self.as_ref().map_or(c"", |value| value.as_cstr())
    }
}

/// The global value, through the generated accessors on the selector.
impl OptString for StrOpt {
    fn value_ptr(&self) -> *mut c_char {
        StrOpt::value_ptr(*self)
    }

    fn first_byte(&self) -> u8 {
        StrOpt::first_byte(*self)
    }

    fn has_byte(&self, byte: u8) -> bool {
        StrOpt::has_byte(*self, byte)
    }

    fn is_unset(&self) -> bool {
        StrOpt::is_unset(*self)
    }

    fn get(&self) -> XString {
        StrOpt::get(*self)
    }
}

/// A global-local string option, read where it is in force: the local
/// copy unless that is empty, the global value otherwise.
///
/// An empty local copy is upstream's "not set here" whether the field owns
/// the empty string or owns nothing at all.
pub(crate) struct LocalOrGlobal<'a> {
    local: &'a Option<XString>,
    global: StrOpt,
}

/// The value of a global-local string option in force: `local` where it is
/// set, else `global`. See [`LocalOrGlobal`].
pub(crate) fn local_or_global(local: &Option<XString>, global: StrOpt) -> LocalOrGlobal<'_> {
    LocalOrGlobal { local, global }
}

impl LocalOrGlobal<'_> {
    /// The local copy, when it is the one in force.
    fn local(&self) -> Option<&XString> {
        self.local.as_ref().filter(|value| !value.is_empty())
    }
}

impl OptString for LocalOrGlobal<'_> {
    fn value_ptr(&self) -> *mut c_char {
        match self.local() {
            Some(value) => value.as_ptr().cast_mut(),
            None => self.global.value_ptr(),
        }
    }

    fn first_byte(&self) -> u8 {
        match self.local() {
            Some(value) => value[0],
            None => self.global.first_byte(),
        }
    }

    fn has_byte(&self, byte: u8) -> bool {
        match self.local() {
            Some(value) => value.contains(&byte),
            None => self.global.has_byte(byte),
        }
    }

    fn is_unset(&self) -> bool {
        self.local().is_none() && self.global.is_unset()
    }

    fn get(&self) -> XString {
        match self.local() {
            Some(value) => value.clone(),
            None => self.global.get(),
        }
    }
}
