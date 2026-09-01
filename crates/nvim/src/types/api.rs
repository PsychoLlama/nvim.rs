#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
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
pub type ArenaMem = *mut consumed_blk;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Array {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
}
#[derive(Copy, Clone)]
pub struct ArrayBuilder {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut Object,
    pub init_array: [Object; 16],
}
pub type Boolean = bool;
pub type Buffer = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ChangedtickDictItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 12],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Dict {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut KeyValuePair,
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
pub type Float = ::core::ffi::c_double;
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
#[derive(Copy, Clone)]
pub struct KeySetLink {
    pub str: *mut ::core::ffi::c_char,
    pub ptr_off: size_t,
    pub type_0: ::core::ffi::c_int,
    pub opt_index: ::core::ffi::c_int,
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
#[derive(Copy, Clone)]
pub struct OptKeySet {
    pub is_set_: OptionalKeys,
}
pub type OptionalKeys = uint64_t;
pub type Tabpage = handle_T;
pub type Window = handle_T;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_value_pair {
    pub key: String_0,
    pub value: Object,
}
/// An API value: one of eleven kinds, each carrying its own payload.
///
/// `#[repr(C, u32)]` is deliberate on three counts. It keeps the C image --
/// a `u32` tag followed by the union of the payloads -- that the generated
/// keyset decoder writes fields into by byte offset and that the unit
/// suite's cdefs describe. It pins the discriminants, which
/// [`ObjectType`] says are read outside the editor. And it makes
/// `mem::zeroed()` of an aggregate holding one produce [`Object::Nil`]
/// rather than an invalid discriminant, which is what lets the keyset
/// decoder start from a zeroed struct.
///
/// `Copy`, as the C struct was: every payload is a scalar or a
/// pointer/length pair. Copying one copies neither what it points at nor
/// the obligation to free it -- ownership travels with the value, exactly
/// once, as it did before.
#[derive(Copy, Clone)]
#[repr(C, u32)]
pub enum Object {
    Nil = 0,
    Boolean(Boolean) = 1,
    Integer(Integer) = 2,
    Float(Float) = 3,
    String(String_0) = 4,
    Array(Array) = 5,
    Dict(Dict) = 6,
    /// A reference to a Lua value, held in that state's registry. Owned:
    /// whoever frees the object releases it.
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

    pub const fn as_boolean(self) -> Option<Boolean> {
        match self {
            Object::Boolean(v) => Some(v),
            _ => None,
        }
    }

    /// The `Integer` arm only. A handle is [`Object::as_handle`].
    pub const fn as_integer(self) -> Option<Integer> {
        match self {
            Object::Integer(v) => Some(v),
            _ => None,
        }
    }

    pub const fn as_float(self) -> Option<Float> {
        match self {
            Object::Float(v) => Some(v),
            _ => None,
        }
    }

    pub const fn as_string(self) -> Option<String_0> {
        match self {
            Object::String(v) => Some(v),
            _ => None,
        }
    }

    pub const fn as_array(self) -> Option<Array> {
        match self {
            Object::Array(v) => Some(v),
            _ => None,
        }
    }

    pub const fn as_dict(self) -> Option<Dict> {
        match self {
            Object::Dict(v) => Some(v),
            _ => None,
        }
    }

    pub const fn as_luaref(self) -> Option<LuaRef> {
        match self {
            Object::LuaRef(v) => Some(v),
            _ => None,
        }
    }

    /// The number a handle carries, whichever of the three tags it wears.
    /// A plain integer is *not* one: the callers that accept one say so.
    pub const fn as_handle(self) -> Option<Integer> {
        match self {
            Object::Buffer(v) | Object::Window(v) | Object::Tabpage(v) => Some(v),
            _ => None,
        }
    }
}
