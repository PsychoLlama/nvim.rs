#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use core::mem::ManuallyDrop;

pub struct AdditionalDataBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ::core::ffi::c_char,
}
/// An arena allocator: a bump pointer into `cur_blk`, which heads a chain of
/// blocks the arena owns and `arena_mem_free` releases.
///
/// Not `Copy`. Two arenas over one block chain would each believe they may
/// bump it and each free it, so every hand-off is a move -- see
/// `unpack_object`, which lends its caller's arena to a scratch unpacker and
/// leaves `ARENA_EMPTY` behind until it is handed back.
#[derive(Clone)]
pub struct Arena {
    pub cur_blk: *mut ::core::ffi::c_char,
    pub pos: size_t,
    pub size: size_t,
}
pub type ArenaMem = *mut ConsumedBlk;
/// The API's array: an owned, growable sequence of [`Object`]s.
///
/// A newtype over `Vec<Object>` rather than the type alias, so that the
/// conversions that have to be deliberate (`From<Vec<Object>>` in, and
/// nothing implicit out) are, and so `ffigen` has a name to render. Every
/// reader goes through [`Deref`](core::ops::Deref) to the vector.
#[derive(Clone, Default)]
pub struct Array(Vec<Object>);

impl Array {
    /// No elements and nothing allocated: C's `ARRAY_DICT_INIT`.
    pub const EMPTY: Self = Self(Vec::new());

    /// An empty array with room for `capacity` elements.
    ///
    /// Every producer knows its element count up front -- the arena arrays
    /// this replaces had to, since a bump allocation cannot grow -- so this
    /// is the usual constructor and `push` never reallocates.
    pub fn with_capacity(capacity: size_t) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// The elements, which the array owns.
    pub fn into_vec(self) -> Vec<Object> {
        self.0
    }
}

impl From<Vec<Object>> for Array {
    fn from(items: Vec<Object>) -> Self {
        Self(items)
    }
}

impl FromIterator<Object> for Array {
    fn from_iter<I: IntoIterator<Item = Object>>(items: I) -> Self {
        Self(items.into_iter().collect())
    }
}

impl IntoIterator for Array {
    type Item = Object;
    type IntoIter = ::std::vec::IntoIter<Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = &'a Object;
    type IntoIter = ::core::slice::Iter<'a, Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl ::core::ops::Deref for Array {
    type Target = Vec<Object>;

    fn deref(&self) -> &Vec<Object> {
        &self.0
    }
}

impl ::core::ops::DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Vec<Object> {
        &mut self.0
    }
}

impl ::core::fmt::Debug for Array {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}
pub type Boolean = bool;
pub type BufferHandle = Handle;
/// The API's dictionary: an owned, ordered sequence of key/value pairs.
///
/// Ordered, not hashed: the API's dictionaries are small and are built once
/// and read once, and the order a caller sent is the order the answer
/// carries. [`Array`]'s shape, over [`KeyValuePair`].
#[derive(Clone, Default)]
pub struct ApiDict(Vec<KeyValuePair>);

impl ApiDict {
    /// No entries and nothing allocated: C's `ARRAY_DICT_INIT`.
    pub const EMPTY: Self = Self(Vec::new());

    /// An empty dictionary with room for `capacity` entries. See
    /// [`Array::with_capacity`].
    pub fn with_capacity(capacity: size_t) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Append `key: value`. Nothing checks for a duplicate key: the API's
    /// dictionaries are built from sources that cannot produce one.
    pub fn insert(&mut self, key: impl Into<DictKey>, value: Object) {
        self.0.push(KeyValuePair {
            key: key.into(),
            value,
        });
    }

    /// The value under `key`, by a linear scan -- these dictionaries are
    /// tens of entries at most.
    pub fn get(&self, key: &[u8]) -> Option<&Object> {
        self.iter()
            .find(|pair| pair.key.bytes() == key)
            .map(|pair| &pair.value)
    }

    /// The entries, which the dictionary owns.
    pub fn into_vec(self) -> Vec<KeyValuePair> {
        self.0
    }
}

impl From<Vec<KeyValuePair>> for ApiDict {
    fn from(items: Vec<KeyValuePair>) -> Self {
        Self(items)
    }
}

impl FromIterator<KeyValuePair> for ApiDict {
    fn from_iter<I: IntoIterator<Item = KeyValuePair>>(items: I) -> Self {
        Self(items.into_iter().collect())
    }
}

impl IntoIterator for ApiDict {
    type Item = KeyValuePair;
    type IntoIter = ::std::vec::IntoIter<KeyValuePair>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ApiDict {
    type Item = &'a KeyValuePair;
    type IntoIter = ::core::slice::Iter<'a, KeyValuePair>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl ::core::ops::Deref for ApiDict {
    type Target = Vec<KeyValuePair>;

    fn deref(&self) -> &Vec<KeyValuePair> {
        &self.0
    }
}

impl ::core::ops::DerefMut for ApiDict {
    fn deref_mut(&mut self) -> &mut Vec<KeyValuePair> {
        &mut self.0
    }
}

impl ::core::fmt::Debug for ApiDict {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|pair| (&pair.key, &pair.value)))
            .finish()
    }
}
pub type ErrorType = ::core::ffi::c_int;
/// What an [`Error`] carries, and the one value that means it carries
/// nothing. Every module that reports an API error needs these.
pub const kErrorTypeNone: ErrorType = -1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeValidation: ErrorType = 1;
/// A kvec of extmark pairs.
///
/// Not `Copy`: `items` is the array's own allocation.
#[derive(Clone)]
pub struct ExtmarkInfoArray {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTPair,
}
pub type FieldHashfn = Option<unsafe fn(*const ::core::ffi::c_char, size_t) -> *const KeySetLink>;
pub type HLGroupID = Integer;
/// Not `Copy`: a kvec of chunks, each owning its text. A `clone` aliases
/// the same array — which several message paths do deliberately, handing
/// ownership on exactly once.
#[derive(Clone)]
pub struct HlMessage {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut HlMessageChunk,
}
pub type Integer = int64_t;
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub is_hlgroup: bool,
}
pub type KeyValuePair = key_value_pair;
pub type LuaRef = ::core::ffi::c_int;
pub type MessageType = ::core::ffi::c_int;
pub type ObjectType = ::core::ffi::c_uint;
/// The numbers [`Object`]'s variants carry, which are visible outside the
/// editor in two places: the Lua binding recognises three of them in a
/// table's `_TYPE` key, and the generated keyset tables store one per field
/// as a plain `int`. [`Object::kind`] answers them.
pub const kObjectTypeNil: ObjectType = 0;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeLuaRef: ObjectType = 7;
/// The three the API uses for handles, which never appear in a value the
/// msgpack layer serialises.
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeTabpage: ObjectType = 10;
pub type TabpageHandle = Handle;
pub type WindowHandle = Handle;
/// One entry of an [`ApiDict`]: an owned key and an owned value.
///
/// The key is a [`DictKey`], not a [`String_0`]: an API key is a short
/// identifier almost without exception, and a `String_0` is an `xmalloc` and
/// an `xfree` for every one of them. The same inline-or-boxed shape a
/// Vimscript dictionary item's key has, for the same reason, and the type is
/// free to differ because `key_value_pair` has no C image -- `tools/ffigen`
/// declares it opaque.
// The type keeps upstream's spelling; `KeyValuePair` is the alias the tree
// uses and `tools/ffigen` reads this name off the C header it mirrors.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct key_value_pair {
    pub key: DictKey,
    pub value: Object,
}
/// An API value: one of eleven kinds, each carrying its own payload.
///
/// **The value owns its payload.** A `String`, `Array` or `Dict` arm holds
/// its own storage and an unreferenced Lua reference is this object's, so
/// dropping an object releases the whole tree below it. That is what the
/// arena used to do wholesale, and what `api_free_object` used to do by
/// hand at 42 call sites.
///
/// `#[repr(u32)]` pins the discriminants, which [`ObjectType`] says are read
/// outside the editor: the Lua binding recognises three of them in a table's
/// `_TYPE` key, and the generated keyset tables store one per field as a
/// plain `int`.
///
/// The three owning payloads sit in a [`ManuallyDrop`]. The value is
/// released by [`Object`]'s own `Drop`, which walks the tree with an
/// explicit stack; giving the compiler a field to drop after that would
/// hand `drop_in_place::<Object>` a landing pad, and a shim with one is
/// neither a tail call nor inlinable at the hundreds of sites that drop an
/// object.
#[derive(Debug)]
#[repr(u32)]
pub enum Object {
    Nil = 0,
    Boolean(Boolean) = 1,
    Integer(Integer) = 2,
    Float(Float) = 3,
    String(ManuallyDrop<String_0>) = 4,
    Array(ManuallyDrop<Array>) = 5,
    Dict(ManuallyDrop<ApiDict>) = 6,
    /// A reference to a Lua value, held in that state's registry. Owned:
    /// dropping the object releases it.
    LuaRef(LuaRef) = 7,
    /// A buffer handle. Distinct from [`Object::Integer`] only in the tag:
    /// the wire encoding gives handles their own msgpack extension type, so
    /// a handle sent as a plain integer arrives as a plain integer.
    Buffer(Integer) = 8,
    /// A window handle. See [`Object::Buffer`].
    Window(Integer) = 9,
    /// A tabpage handle. See [`Object::Buffer`].
    Tabpage(Integer) = 10,
}

impl Object {
    /// The tag, as the number the Lua binding and the keyset tables speak.
    pub const fn kind(&self) -> ObjectType {
        match self {
            Object::Nil => kObjectTypeNil,
            Object::Boolean(_) => kObjectTypeBoolean,
            Object::Integer(_) => kObjectTypeInteger,
            Object::Float(_) => kObjectTypeFloat,
            Object::String(_) => kObjectTypeString,
            Object::Array(_) => kObjectTypeArray,
            Object::Dict(_) => kObjectTypeDict,
            Object::LuaRef(_) => kObjectTypeLuaRef,
            Object::Buffer(_) => kObjectTypeBuffer,
            Object::Window(_) => kObjectTypeWindow,
            Object::Tabpage(_) => kObjectTypeTabpage,
        }
    }

    pub const fn is_nil(&self) -> bool {
        matches!(self, Object::Nil)
    }

    pub const fn as_boolean(&self) -> Option<Boolean> {
        match self {
            Object::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    /// The `Integer` arm only. A handle is [`Object::as_handle`].
    pub const fn as_integer(&self) -> Option<Integer> {
        match self {
            Object::Integer(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_float(&self) -> Option<Float> {
        match self {
            Object::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// A borrow of the string the object owns.
    pub fn as_string(&self) -> Option<&String_0> {
        match self {
            Object::String(v) => Some(v),
            _ => None,
        }
    }

    /// A borrow of the array the object owns.
    pub fn as_array(&self) -> Option<&Array> {
        match self {
            Object::Array(v) => Some(v),
            _ => None,
        }
    }

    /// A mutable borrow of the array the object owns, for the callers that
    /// fill one in place.
    pub fn as_array_mut(&mut self) -> Option<&mut Array> {
        match self {
            Object::Array(v) => Some(v),
            _ => None,
        }
    }

    /// A borrow of the dictionary the object owns.
    pub fn as_dict(&self) -> Option<&ApiDict> {
        match self {
            Object::Dict(v) => Some(v),
            _ => None,
        }
    }

    /// A mutable borrow of the dictionary the object owns.
    pub fn as_dict_mut(&mut self) -> Option<&mut ApiDict> {
        match self {
            Object::Dict(v) => Some(v),
            _ => None,
        }
    }

    /// The Lua registry reference, which stays the object's: releasing it
    /// twice is a use-after-free. [`Object::into_luaref`] takes it.
    pub const fn as_luaref(&self) -> Option<LuaRef> {
        match self {
            Object::LuaRef(v) => Some(*v),
            _ => None,
        }
    }

    /// The number a handle carries, whichever of the three tags it wears.
    /// A plain integer is *not* one: the callers that accept one say so.
    pub const fn as_handle(&self) -> Option<Integer> {
        match self {
            Object::Buffer(v) | Object::Window(v) | Object::Tabpage(v) => Some(*v),
            _ => None,
        }
    }

    /// The value, leaving [`Object::Nil`] behind.
    ///
    /// The one way to move a payload out of a place: an `Object` has a
    /// destructor, so it cannot be destructured where it stands.
    pub fn take(&mut self) -> Self {
        ::core::mem::replace(self, Object::Nil)
    }
}
