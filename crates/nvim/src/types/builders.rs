//! Constructing API values.
//!
//! [`Object`] is a tag plus a payload, and [`Array`]/[`ApiDict`] are owned
//! boxed slices. Building one literally takes a dozen lines of struct syntax per
//! element, which is why the transpiled call sites run to hundreds of lines
//! for a single `nvim_echo`.
//!
//! Two pieces here. [`Object`]'s constructors tag the payload correctly by
//! construction and put the owning ones in the [`ManuallyDrop`] the type
//! wants them in. [`ArrayBuf`] and [`DictBuf`] are fixed-capacity builders
//! -- the safe spelling of C's `MAXSIZE_TEMP_ARRAY`, for the call sites
//! that know their element count from the shape of the code and would
//! rather say it once than at every `push`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{
    ApiDict, ApiString, Array, BufferHandle, DictKey, Float, Integer, KeyValuePair, LuaRef, Object,
    TabpageHandle, WindowHandle,
};
use core::ffi::CStr;
use core::mem::ManuallyDrop;

impl Object {
    pub const fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    pub const fn integer(value: Integer) -> Self {
        Self::Integer(value)
    }

    pub const fn float(value: Float) -> Self {
        Self::Float(value)
    }

    /// An API string, whose bytes the object takes over.
    pub const fn string(value: ApiString) -> Self {
        Self::String(ManuallyDrop::new(value))
    }

    /// [`Object::string`] for a string literal, whose bytes are copied: an
    /// object owns its string, and the binary's read-only data is not
    /// something it can own.
    pub fn literal(text: &'static str) -> Self {
        Self::string(ApiString::from(text))
    }

    /// An array, whose elements the object takes over: an [`Array`], or
    /// the `Vec<Object>` it was built in, frozen here.
    pub fn array(value: impl Into<Array>) -> Self {
        Self::Array(ManuallyDrop::new(value.into()))
    }

    /// A dictionary, whose entries the object takes over: an [`ApiDict`],
    /// or the [`DictBuilder`] it was built in, frozen here.
    pub fn dict(value: impl Into<ApiDict>) -> Self {
        Self::Dict(ManuallyDrop::new(value.into()))
    }

    /// A reference to a Lua value, held in that state's registry. The
    /// reference is owned: dropping the object releases it.
    pub const fn luaref(value: LuaRef) -> Self {
        Self::LuaRef(value)
    }

    /// A window handle. Handles are `Handle`; the variant carries the
    /// widened [`Integer`] the wire and the payload always did.
    pub const fn window(value: WindowHandle) -> Self {
        Self::Window(value.to_integer())
    }

    /// A buffer handle. See [`Object::window`].
    pub const fn buffer(value: BufferHandle) -> Self {
        Self::Buffer(value.to_integer())
    }

    /// A tabpage handle. See [`Object::window`].
    pub const fn tabpage(value: TabpageHandle) -> Self {
        Self::Tabpage(value.to_integer())
    }
}

/// A builder for an [`Array`] of at most `N` elements.
///
/// The capacity is a property of the call site -- how many times the code
/// below can push -- so it is stated once, in the type, and [`Self::push`]
/// panics past it rather than growing. The elements wait in the builder
/// itself, so the array is one allocation of exactly the size pushed: a
/// heap reservation of `N` would be a second, and a shrink when fewer came.
pub struct ArrayBuf<const N: usize> {
    items: [Object; N],
    len: usize,
}

impl<const N: usize> ArrayBuf<N> {
    pub fn new() -> Self {
        Self {
            items: [const { Object::Nil }; N],
            len: 0,
        }
    }

    /// Appends `value`. Panics past `N` elements -- the capacity is a
    /// property of the call site, not of anything a user can influence.
    pub fn push(&mut self, value: Object) -> &mut Self {
        assert!(self.len < N, "ArrayBuf overflow");
        self.items[self.len] = value;
        self.len += 1;
        self
    }

    /// The elements pushed so far, leaving the builder empty.
    pub fn array(&mut self) -> Array {
        let len = ::core::mem::take(&mut self.len);
        self.items[..len].iter_mut().map(Object::take).collect()
    }

    /// [`Self::array`], wrapped for nesting inside another builder.
    pub fn object(&mut self) -> Object {
        Object::array(self.array())
    }
}

impl<const N: usize> Default for ArrayBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for an [`ApiDict`] of at most `N` entries. [`ArrayBuf`]'s rules
/// apply unchanged.
pub struct DictBuf<const N: usize> {
    entries: [KeyValuePair; N],
    len: usize,
}

impl<const N: usize> DictBuf<N> {
    pub fn new() -> Self {
        Self {
            entries: [const {
                KeyValuePair {
                    key: DictKey::EMPTY,
                    value: Object::Nil,
                }
            }; N],
            len: 0,
        }
    }

    /// Appends `key: value`, with `key` a literal. Dict keys in generated
    /// calls always are; a computed key wants [`Self::insert_string`].
    pub fn insert(&mut self, key: &'static CStr, value: Object) -> &mut Self {
        self.insert_string(key, value)
    }

    /// [`Self::insert`] for a key the caller built, whose bytes the
    /// dictionary copies.
    pub fn insert_string(&mut self, key: impl Into<DictKey>, value: Object) -> &mut Self {
        assert!(self.len < N, "DictBuf overflow");
        self.entries[self.len] = KeyValuePair {
            key: key.into(),
            value,
        };
        self.len += 1;
        self
    }

    /// The entries inserted so far, leaving the builder empty.
    pub fn dict(&mut self) -> ApiDict {
        let len = ::core::mem::take(&mut self.len);
        self.entries[..len]
            .iter_mut()
            .map(|pair| KeyValuePair {
                key: ::core::mem::replace(&mut pair.key, DictKey::EMPTY),
                value: pair.value.take(),
            })
            .collect()
    }

    /// [`Self::dict`], wrapped for nesting inside another builder.
    pub fn object(&mut self) -> Object {
        Object::dict(self.dict())
    }
}

impl<const N: usize> Default for DictBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_reports_what_was_pushed() {
        let mut buf = ArrayBuf::<4>::new();
        buf.push(Object::integer(7));
        buf.push(Object::boolean(true));
        let array = buf.array();
        assert_eq!(array.len(), 2);
        assert_eq!(array[0].as_integer(), Some(7));
        assert_eq!(array[1].as_boolean(), Some(true));
    }

    #[test]
    fn dict_nests_in_an_array() {
        let mut opts = DictBuf::<1>::new();
        opts.insert(c"verbose", Object::boolean(true));
        let entry = opts.object();
        assert_eq!(
            entry.as_dict().map(|d| d[0].key.bytes()),
            Some(&b"verbose"[..])
        );

        let mut args = ArrayBuf::<2>::new();
        args.push(Object::literal("hello"));
        args.push(entry);
        let array = args.array();
        assert_eq!(
            array[0].as_string().map(ApiString::as_bytes),
            Some(&b"hello"[..])
        );
        assert_eq!(
            array[1]
                .as_dict()
                .and_then(|d| d.get(b"verbose"))
                .and_then(Object::as_boolean),
            Some(true)
        );
    }

    #[test]
    fn a_dropped_array_releases_its_strings() {
        let mut buf = ArrayBuf::<2>::new();
        buf.push(Object::literal("one"));
        buf.push(Object::literal("two"));
        drop(buf.array());
    }
}
