//! A value's string payload: the text a String holds and the name a
//! funcref holds, both a [`ThinCString`] the value owns.
//!
//! The readers borrow it ([`TypVal::string_ref`], [`TypVal::func_name`],
//! and the measured forms beside them); the writers hand a value a string
//! to own ([`TypVal::string`], [`TypVal::write_string`]); the takers move it
//! back out and leave the null string behind.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::ThinCString;
use crate::types::TypVal;

impl TypVal {
    /// A String value over `text`, which the value takes over; `None` is
    /// `v:_null_string`.
    #[inline(always)]
    pub(crate) const fn string(text: Option<ThinCString>) -> TypVal {
        TypVal::String(::core::mem::ManuallyDrop::new(text))
    }

    /// A funcref naming `name`, which the value takes over. The reference
    /// to the function is the caller's to have taken.
    #[inline(always)]
    pub(crate) const fn func(name: Option<ThinCString>) -> TypVal {
        TypVal::Func(::core::mem::ManuallyDrop::new(name))
    }

    /// A String value holding a copy of `bytes`.
    #[inline]
    pub(crate) fn string_from(bytes: &[u8]) -> TypVal {
        TypVal::string(Some(ThinCString::from_bytes(bytes)))
    }

    /// The text this String holds, borrowed -- `None` for every other kind
    /// and for `v:_null_string`, which most readers take for the empty
    /// string.
    #[inline(always)]
    pub(crate) fn string_ref(&self) -> Option<&ThinCString> {
        match self {
            TypVal::String(text) => text.as_ref(),
            _ => None,
        }
    }

    /// The text this String holds, as bytes: empty for the null string and
    /// for every other kind.
    #[inline]
    pub(crate) fn string_bytes(&self) -> &[u8] {
        self.string_ref().map_or(b"", |text| text.as_bytes())
    }

    /// The text this String holds, as a C string: `None` for every other
    /// kind and for the null string.
    #[inline]
    pub(crate) fn string_cstr(&self) -> Option<&::core::ffi::CStr> {
        self.string_ref().map(ThinCString::as_cstr)
    }

    /// The text this String holds, writable in place.
    #[inline(always)]
    pub(crate) fn string_mut(&mut self) -> Option<&mut ThinCString> {
        match self {
            TypVal::String(text) => text.as_mut(),
            _ => None,
        }
    }

    /// Whether this is a String -- the null one included.
    #[inline(always)]
    pub(crate) fn is_string(&self) -> bool {
        matches!(self, TypVal::String(_))
    }

    /// Move the text out of this String, leaving `v:_null_string` behind.
    /// `None` for every other kind, which is left alone.
    #[inline(always)]
    pub(crate) fn take_string(&mut self) -> Option<ThinCString> {
        match self {
            TypVal::String(text) => text.take(),
            _ => None,
        }
    }

    /// The function name this funcref holds, borrowed; `None` for every
    /// other kind and for a funcref that names nothing.
    #[inline(always)]
    pub(crate) fn func_name(&self) -> Option<&ThinCString> {
        match self {
            TypVal::Func(name) => name.as_ref(),
            _ => None,
        }
    }

    /// Move the name out of this funcref, leaving one that names nothing.
    /// The reference to the function goes with it.
    #[inline(always)]
    pub(crate) fn take_func_name(&mut self) -> Option<ThinCString> {
        match self {
            TypVal::Func(name) => name.take(),
            _ => None,
        }
    }

    /// The text under either variant that holds one -- a String's text or
    /// a funcref's name -- and `None` under any other.
    ///
    /// The arms that treat the two alike (`tv2bool`, `tv_copy`, the
    /// encoders) are the reason this exists; a site that means only one of
    /// them wants [`TypVal::string_ref`] or [`TypVal::func_name`].
    #[inline(always)]
    pub(crate) fn text_or_name(&self) -> Option<&ThinCString> {
        match self {
            TypVal::String(text) | TypVal::Func(text) => text.as_ref(),
            _ => None,
        }
    }

    /// Overwrite this slot with a String, **releasing nothing**: see
    /// [`TypVal::overwrite`].
    #[inline(always)]
    pub(crate) fn write_string(&mut self, text: Option<ThinCString>) {
        self.overwrite(TypVal::string(text));
    }

    /// Overwrite this slot with a funcref naming `name`, **releasing
    /// nothing**: see [`TypVal::overwrite`].
    #[inline(always)]
    pub(crate) fn write_func_name(&mut self, name: Option<ThinCString>) {
        self.overwrite(TypVal::func(name));
    }

    // TRANSIENT: the pointer forms below go once their callers convert.

    /// TRANSIENT.
    #[inline(always)]
    pub(crate) fn string_or_null(&self) -> *mut ::core::ffi::c_char {
        self.string_ref()
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut())
    }

    /// TRANSIENT.
    #[inline(always)]
    pub(crate) fn func_name_or_null(&self) -> *mut ::core::ffi::c_char {
        self.func_name()
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut())
    }

    /// TRANSIENT.
    #[inline(always)]
    pub(crate) fn write_string_raw(&mut self, raw: *mut ::core::ffi::c_char) {
        // SAFETY: TRANSIENT -- the callers hand over an owned block.
        self.write_string(unsafe { ThinCString::from_raw(raw) });
    }

    /// TRANSIENT.
    #[inline(always)]
    pub(crate) fn string_raw(raw: *mut ::core::ffi::c_char) -> TypVal {
        // SAFETY: TRANSIENT -- the callers hand over an owned block.
        TypVal::string(unsafe { ThinCString::from_raw(raw) })
    }

    /// TRANSIENT.
    #[inline(always)]
    pub(crate) fn func_raw(raw: *mut ::core::ffi::c_char) -> TypVal {
        // SAFETY: TRANSIENT -- the callers hand over an owned block.
        TypVal::func(unsafe { ThinCString::from_raw(raw) })
    }

    /// Append `tail` to this String in place, growing its allocation. False,
    /// leaving the value alone, for anything that is not a String with an
    /// allocation to grow.
    pub(crate) fn append_to_string(&mut self, tail: &[u8]) -> bool {
        match self.string_mut() {
            Some(text) => {
                text.push_bytes(tail);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_kinds_that_both_hold_a_string_stay_apart() {
        let string = TypVal::string_from(b"x");
        assert_eq!(string.string_ref().map(|s| s.as_bytes()), Some(&b"x"[..]));
        assert!(string.func_name().is_none());
        assert_eq!(string.text_or_name().map(|s| s.as_bytes()), Some(&b"x"[..]));

        // A funcref without the function reference a real one carries: its
        // name is taken back out below rather than released by a clear.
        let mut func = TypVal::func(Some(ThinCString::from_bytes(b"x")));
        assert!(func.string_ref().is_none());
        assert_eq!(func.func_name().map(|s| s.as_bytes()), Some(&b"x"[..]));
        assert_eq!(func.text_or_name().map(|s| s.as_bytes()), Some(&b"x"[..]));
        drop(func.take_func_name());
        assert!(func.is_empty());
    }

    /// The null string is a String holding nothing, and it is empty in
    /// every sense the readers ask about; the empty string is a block.
    #[test]
    fn the_null_string_and_the_empty_string_read_alike_but_differ() {
        let null = TypVal::string(None);
        let empty = TypVal::string_from(b"");
        assert!(null.is_string() && empty.is_string());
        assert!(null.string_ref().is_none());
        assert!(empty.string_ref().is_some_and(ThinCString::is_empty));
        assert_eq!(null.string_bytes(), b"");
        assert_eq!(empty.string_bytes(), b"");
        assert!(null.is_empty());
        assert!(!empty.is_empty());
        assert!(null.payload_address().is_null());
        assert!(!empty.payload_address().is_null());
    }

    #[test]
    fn taking_the_text_leaves_the_null_string() {
        let mut tv = TypVal::string_from(b"moved");
        let text = tv.take_string();
        assert_eq!(text.as_ref().map(|s| s.as_bytes()), Some(&b"moved"[..]));
        assert!(tv.is_string() && tv.string_ref().is_none());
        assert!(TypVal::Number(1).take_string().is_none());
    }

    #[test]
    fn appending_grows_a_string_and_refuses_the_null_one() {
        let mut tv = TypVal::string_from(b"ab");
        assert!(tv.append_to_string(b"cd"));
        assert_eq!(tv.string_bytes(), b"abcd");
        let mut null = TypVal::string(None);
        assert!(!null.append_to_string(b"x"));
        assert!(null.string_ref().is_none());
    }
}
