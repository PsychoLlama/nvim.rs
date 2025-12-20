//! `MessagePack` unpacker for Neovim API
//!
//! This module provides a Rust implementation of the msgpack unpacker
//! that produces Object values compatible with the C API types.

#![allow(clippy::missing_safety_doc)] // FFI functions need unsafe but docs come later
#![allow(unsafe_code)] // FFI requires unsafe
#![allow(deprecated)] // rmp read_data_* functions are deprecated but still work
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::match_same_arms)]

use std::ffi::c_char;
use std::io::Cursor;
use std::os::raw::c_int;
use std::ptr;

use rmp::decode::{self, DecodeStringError, MarkerReadError, NumValueReadError, ValueReadError};
use rmp::Marker;

// ============================================================================
// Arena FFI - Arena is an opaque pointer passed from C
// ============================================================================

/// Opaque handle to C Arena struct
pub type ArenaHandle = *mut std::ffi::c_void;

extern "C" {
    /// Allocate memory from an arena
    /// void *`arena_alloc(Arena` *arena, `size_t` size, bool align);
    fn arena_alloc(arena: ArenaHandle, size: usize, align: bool) -> *mut std::ffi::c_void;
}

// ============================================================================
// FFI Type Definitions - must match C layout exactly
// ============================================================================

/// `ObjectType` enum - must match C enum in api/private/defs.h
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Nil = 0,
    Boolean = 1,
    Integer = 2,
    Float = 3,
    String = 4,
    Array = 5,
    Dict = 6,
    LuaRef = 7,
    Buffer = 8,
    Window = 9,
    Tabpage = 10,
}

/// `ErrorType` enum - must match C enum
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    None = -1,
    Exception = 0,
    Validation = 1,
}

/// String type - must match C String struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

impl Default for NvimString {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            size: 0,
        }
    }
}

/// `kvec_t` layout for Array/Dict
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KVec<T> {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut T,
}

impl<T> Default for KVec<T> {
    fn default() -> Self {
        Self {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        }
    }
}

/// Array type - `kvec_t(Object)`
pub type Array = KVec<Object>;

/// `KeyValuePair` for Dict
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KeyValuePair {
    pub key: NvimString,
    pub value: Object,
}

/// Dict type - `kvec_t(KeyValuePair)`
pub type Dict = KVec<KeyValuePair>;

/// Object union data - must match C union layout
#[repr(C)]
#[derive(Clone, Copy)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub floating: f64,
    pub string: NvimString,
    pub array: Array,
    pub dict: Dict,
    pub luaref: c_int,
}

impl Default for ObjectData {
    fn default() -> Self {
        Self { integer: 0 }
    }
}

/// Object struct - must match C struct object
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    pub r#type: ObjectType,
    pub data: ObjectData,
}

impl Default for Object {
    fn default() -> Self {
        Self::nil()
    }
}

impl Object {
    #[must_use]
    pub const fn nil() -> Self {
        Self {
            r#type: ObjectType::Nil,
            data: ObjectData { integer: 0 },
        }
    }

    #[must_use]
    pub const fn boolean(b: bool) -> Self {
        Self {
            r#type: ObjectType::Boolean,
            data: ObjectData { boolean: b },
        }
    }

    #[must_use]
    pub const fn integer(i: i64) -> Self {
        Self {
            r#type: ObjectType::Integer,
            data: ObjectData { integer: i },
        }
    }

    #[must_use]
    pub const fn float(f: f64) -> Self {
        Self {
            r#type: ObjectType::Float,
            data: ObjectData { floating: f },
        }
    }

    #[must_use]
    pub const fn string(s: NvimString) -> Self {
        Self {
            r#type: ObjectType::String,
            data: ObjectData { string: s },
        }
    }

    #[must_use]
    pub const fn array(a: Array) -> Self {
        Self {
            r#type: ObjectType::Array,
            data: ObjectData { array: a },
        }
    }

    #[must_use]
    pub const fn dict(d: Dict) -> Self {
        Self {
            r#type: ObjectType::Dict,
            data: ObjectData { dict: d },
        }
    }

    #[must_use]
    pub const fn buffer(handle: i64) -> Self {
        Self {
            r#type: ObjectType::Buffer,
            data: ObjectData { integer: handle },
        }
    }

    #[must_use]
    pub const fn window(handle: i64) -> Self {
        Self {
            r#type: ObjectType::Window,
            data: ObjectData { integer: handle },
        }
    }

    #[must_use]
    pub const fn tabpage(handle: i64) -> Self {
        Self {
            r#type: ObjectType::Tabpage,
            data: ObjectData { integer: handle },
        }
    }
}

/// Error struct - must match C Error struct
#[repr(C)]
pub struct Error {
    pub r#type: ErrorType,
    pub msg: *mut c_char,
}

impl Default for Error {
    fn default() -> Self {
        Self {
            r#type: ErrorType::None,
            msg: ptr::null_mut(),
        }
    }
}

// ============================================================================
// Unpacker Implementation
// ============================================================================

/// Maximum EXT type value
const EXT_OBJECT_TYPE_MAX: i8 = 2; // kObjectTypeTabpage - kObjectTypeBuffer

/// Maximum recursion depth for unpacking
const MAX_DEPTH: usize = 32;

/// Unpacker state
struct Unpacker<'a> {
    cursor: Cursor<&'a [u8]>,
    arena: ArenaHandle,
    depth: usize,
}

impl<'a> Unpacker<'a> {
    const fn new(data: &'a [u8], arena: ArenaHandle) -> Self {
        Self {
            cursor: Cursor::new(data),
            arena,
            depth: 0,
        }
    }

    /// Allocate memory from the arena for a string
    fn alloc_string(&mut self, len: usize) -> *mut c_char {
        // Allocate len + 1 for NUL terminator
        unsafe { arena_alloc(self.arena, len + 1, false).cast::<c_char>() }
    }

    /// Allocate memory from the arena for an array of Objects
    fn alloc_array(&mut self, len: usize) -> *mut Object {
        let size = len * std::mem::size_of::<Object>();
        unsafe { arena_alloc(self.arena, size, false).cast::<Object>() }
    }

    /// Allocate memory from the arena for an array of `KeyValuePairs`
    fn alloc_dict(&mut self, len: usize) -> *mut KeyValuePair {
        let size = len * std::mem::size_of::<KeyValuePair>();
        unsafe { arena_alloc(self.arena, size, false).cast::<KeyValuePair>() }
    }

    /// Unpack a single Object
    fn unpack_object(&mut self) -> Result<Object, UnpackError> {
        if self.depth >= MAX_DEPTH {
            return Err(UnpackError::TooDeep);
        }
        self.depth += 1;
        let result = self.unpack_object_inner();
        self.depth -= 1;
        result
    }

    fn unpack_object_inner(&mut self) -> Result<Object, UnpackError> {
        let marker = decode::read_marker(&mut self.cursor)?;

        match marker {
            Marker::Null => Ok(Object::nil()),

            Marker::True => Ok(Object::boolean(true)),
            Marker::False => Ok(Object::boolean(false)),

            // Positive fixint
            Marker::FixPos(n) => Ok(Object::integer(i64::from(n))),

            // Negative fixint
            Marker::FixNeg(n) => Ok(Object::integer(i64::from(n))),

            // Unsigned integers
            Marker::U8 => {
                let n = decode::read_data_u8(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::U16 => {
                let n = decode::read_data_u16(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::U32 => {
                let n = decode::read_data_u32(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::U64 => {
                let n = decode::read_data_u64(&mut self.cursor)?;
                // Note: This may overflow for very large u64 values
                Ok(Object::integer(n as i64))
            }

            // Signed integers
            Marker::I8 => {
                let n = decode::read_data_i8(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::I16 => {
                let n = decode::read_data_i16(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::I32 => {
                let n = decode::read_data_i32(&mut self.cursor)?;
                Ok(Object::integer(i64::from(n)))
            }
            Marker::I64 => {
                let n = decode::read_data_i64(&mut self.cursor)?;
                Ok(Object::integer(n))
            }

            // Floats
            Marker::F32 => {
                let n = decode::read_data_f32(&mut self.cursor)?;
                Ok(Object::float(f64::from(n)))
            }
            Marker::F64 => {
                let n = decode::read_data_f64(&mut self.cursor)?;
                Ok(Object::float(n))
            }

            // Strings (fixstr)
            Marker::FixStr(len) => self.unpack_string(len as usize),

            // Strings (str8, str16, str32)
            Marker::Str8 => {
                let len = decode::read_data_u8(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }
            Marker::Str16 => {
                let len = decode::read_data_u16(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }
            Marker::Str32 => {
                let len = decode::read_data_u32(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }

            // Binary (treat as string)
            Marker::Bin8 => {
                let len = decode::read_data_u8(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }
            Marker::Bin16 => {
                let len = decode::read_data_u16(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }
            Marker::Bin32 => {
                let len = decode::read_data_u32(&mut self.cursor)? as usize;
                self.unpack_string(len)
            }

            // Arrays (fixarray)
            Marker::FixArray(len) => self.unpack_array(len as usize),

            // Arrays (array16, array32)
            Marker::Array16 => {
                let len = decode::read_data_u16(&mut self.cursor)? as usize;
                self.unpack_array(len)
            }
            Marker::Array32 => {
                let len = decode::read_data_u32(&mut self.cursor)? as usize;
                self.unpack_array(len)
            }

            // Maps (fixmap)
            Marker::FixMap(len) => self.unpack_map(len as usize),

            // Maps (map16, map32)
            Marker::Map16 => {
                let len = decode::read_data_u16(&mut self.cursor)? as usize;
                self.unpack_map(len)
            }
            Marker::Map32 => {
                let len = decode::read_data_u32(&mut self.cursor)? as usize;
                self.unpack_map(len)
            }

            // EXT types (fixext1 through fixext16, ext8, ext16, ext32)
            Marker::FixExt1 => self.unpack_ext(1),
            Marker::FixExt2 => self.unpack_ext(2),
            Marker::FixExt4 => self.unpack_ext(4),
            Marker::FixExt8 => self.unpack_ext(8),
            Marker::FixExt16 => self.unpack_ext(16),
            Marker::Ext8 => {
                let len = decode::read_data_u8(&mut self.cursor)? as usize;
                self.unpack_ext(len)
            }
            Marker::Ext16 => {
                let len = decode::read_data_u16(&mut self.cursor)? as usize;
                self.unpack_ext(len)
            }
            Marker::Ext32 => {
                let len = decode::read_data_u32(&mut self.cursor)? as usize;
                self.unpack_ext(len)
            }

            Marker::Reserved => Err(UnpackError::Invalid),
        }
    }

    fn unpack_string(&mut self, len: usize) -> Result<Object, UnpackError> {
        let ptr = self.alloc_string(len);
        if ptr.is_null() && len > 0 {
            return Err(UnpackError::OutOfMemory);
        }

        // Read string data directly into arena-allocated buffer
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr.cast::<u8>(), len) };
        std::io::Read::read_exact(&mut self.cursor, slice)?;

        // NUL-terminate
        unsafe {
            *ptr.add(len) = 0;
        }

        Ok(Object::string(NvimString {
            data: ptr,
            size: len,
        }))
    }

    fn unpack_array(&mut self, len: usize) -> Result<Object, UnpackError> {
        let items = self.alloc_array(len);
        if items.is_null() && len > 0 {
            return Err(UnpackError::OutOfMemory);
        }

        for i in 0..len {
            let obj = self.unpack_object()?;
            unsafe {
                *items.add(i) = obj;
            }
        }

        Ok(Object::array(Array {
            size: len,
            capacity: len,
            items,
        }))
    }

    fn unpack_map(&mut self, len: usize) -> Result<Object, UnpackError> {
        let items = self.alloc_dict(len);
        if items.is_null() && len > 0 {
            return Err(UnpackError::OutOfMemory);
        }

        for i in 0..len {
            // Unpack key (must be string)
            let key_obj = self.unpack_object()?;
            let key = match key_obj.r#type {
                ObjectType::String => unsafe { key_obj.data.string },
                _ => NvimString::default(), // Non-string keys become empty strings
            };

            // Unpack value
            let value = self.unpack_object()?;

            unsafe {
                *items.add(i) = KeyValuePair { key, value };
            }
        }

        Ok(Object::dict(Dict {
            size: len,
            capacity: len,
            items,
        }))
    }

    fn unpack_ext(&mut self, len: usize) -> Result<Object, UnpackError> {
        // Read ext type
        let ext_type = decode::read_data_i8(&mut self.cursor)?;

        // Check if this is a known ext type (Buffer, Window, Tabpage)
        if (0..=EXT_OBJECT_TYPE_MAX).contains(&ext_type) {
            // Read the integer value from the ext data
            if len == 0 {
                return Ok(Object::nil());
            }

            // The ext data should contain a msgpack integer
            let marker = decode::read_marker(&mut self.cursor)?;
            let handle = match marker {
                Marker::FixPos(n) => i64::from(n),
                Marker::FixNeg(n) => i64::from(n),
                Marker::U8 => i64::from(decode::read_data_u8(&mut self.cursor)?),
                Marker::U16 => i64::from(decode::read_data_u16(&mut self.cursor)?),
                Marker::U32 => i64::from(decode::read_data_u32(&mut self.cursor)?),
                Marker::U64 => decode::read_data_u64(&mut self.cursor)? as i64,
                Marker::I8 => i64::from(decode::read_data_i8(&mut self.cursor)?),
                Marker::I16 => i64::from(decode::read_data_i16(&mut self.cursor)?),
                Marker::I32 => i64::from(decode::read_data_i32(&mut self.cursor)?),
                Marker::I64 => decode::read_data_i64(&mut self.cursor)?,
                _ => return Ok(Object::nil()),
            };

            // Map ext type to ObjectType
            match ext_type {
                0 => Ok(Object::buffer(handle)),
                1 => Ok(Object::window(handle)),
                2 => Ok(Object::tabpage(handle)),
                _ => Ok(Object::nil()),
            }
        } else {
            // Unknown ext type, skip the data and return nil
            for _ in 0..len {
                decode::read_data_u8(&mut self.cursor)?;
            }
            Ok(Object::nil())
        }
    }
}

/// Unpacker error types
#[derive(Debug)]
enum UnpackError {
    #[allow(dead_code)]
    Io(std::io::Error),
    Invalid,
    TooDeep,
    OutOfMemory,
    Incomplete,
}

impl From<std::io::Error> for UnpackError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Self::Incomplete
        } else {
            Self::Io(e)
        }
    }
}

impl From<MarkerReadError> for UnpackError {
    fn from(e: MarkerReadError) -> Self {
        Self::from(e.0)
    }
}

impl From<ValueReadError> for UnpackError {
    fn from(e: ValueReadError) -> Self {
        match e {
            ValueReadError::InvalidMarkerRead(io) => Self::from(io),
            ValueReadError::InvalidDataRead(io) => Self::from(io),
            ValueReadError::TypeMismatch(_) => Self::Invalid,
        }
    }
}

impl From<NumValueReadError> for UnpackError {
    fn from(e: NumValueReadError) -> Self {
        match e {
            NumValueReadError::InvalidMarkerRead(io) => Self::from(io),
            NumValueReadError::InvalidDataRead(io) => Self::from(io),
            NumValueReadError::TypeMismatch(_) => Self::Invalid,
            NumValueReadError::OutOfRange => Self::Invalid,
        }
    }
}

impl From<DecodeStringError<'_>> for UnpackError {
    fn from(e: DecodeStringError) -> Self {
        match e {
            DecodeStringError::InvalidMarkerRead(io) => Self::from(io),
            DecodeStringError::InvalidDataRead(io) => Self::from(io),
            DecodeStringError::TypeMismatch(_) => Self::Invalid,
            DecodeStringError::InvalidUtf8(..) => Self::Invalid,
            DecodeStringError::BufferSizeTooSmall(_) => Self::OutOfMemory,
        }
    }
}

// ============================================================================
// FFI Exports
// ============================================================================

extern "C" {
    fn api_set_error(err: *mut Error, r#type: ErrorType, msg: *const c_char, ...);
}

/// Unpack msgpack data into an Object
///
/// # Safety
/// - `data` must be a valid pointer to `size` bytes
/// - `arena` must be a valid arena handle
/// - `err` must be a valid pointer to an Error struct
#[no_mangle]
pub unsafe extern "C" fn rs_unpack(
    data: *const c_char,
    size: usize,
    arena: ArenaHandle,
    err: *mut Error,
) -> Object {
    if data.is_null() || size == 0 {
        return Object::nil();
    }

    let slice = std::slice::from_raw_parts(data.cast::<u8>(), size);
    let mut unpacker = Unpacker::new(slice, arena);

    match unpacker.unpack_object() {
        Ok(obj) => {
            // Check for trailing data
            let pos = unpacker.cursor.position() as usize;
            if pos < size {
                api_set_error(
                    err,
                    ErrorType::Exception,
                    b"trailing data in msgpack string\0"
                        .as_ptr()
                        .cast::<c_char>(),
                );
            }
            obj
        }
        Err(UnpackError::TooDeep) => {
            api_set_error(
                err,
                ErrorType::Exception,
                b"object was too deep to unpack\0".as_ptr().cast::<c_char>(),
            );
            Object::nil()
        }
        Err(UnpackError::Incomplete) => {
            api_set_error(
                err,
                ErrorType::Exception,
                b"incomplete msgpack string\0".as_ptr().cast::<c_char>(),
            );
            Object::nil()
        }
        Err(UnpackError::Invalid) => {
            api_set_error(
                err,
                ErrorType::Exception,
                b"invalid msgpack string\0".as_ptr().cast::<c_char>(),
            );
            Object::nil()
        }
        Err(UnpackError::OutOfMemory) => {
            api_set_error(
                err,
                ErrorType::Exception,
                b"out of memory during unpack\0".as_ptr().cast::<c_char>(),
            );
            Object::nil()
        }
        Err(UnpackError::Io(_)) => {
            api_set_error(
                err,
                ErrorType::Exception,
                b"io error during unpack\0".as_ptr().cast::<c_char>(),
            );
            Object::nil()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_sizes() {
        // Verify our types match expected C sizes
        assert_eq!(std::mem::size_of::<NvimString>(), 16); // ptr + size_t
        assert_eq!(std::mem::size_of::<KVec<Object>>(), 24); // 3 * size_t
    }
}
