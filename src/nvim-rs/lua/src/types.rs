//! Lua type conversion constants and utilities
//!
//! This module defines type constants for Lua-Neovim value conversion,
//! matching the Lua C API type definitions.

use std::ffi::{c_char, c_int, CStr};

// =============================================================================
// Lua type constants (matching lua.h)
// =============================================================================

/// Lua type: none (for invalid stack positions)
pub const LUA_TNONE: c_int = -1;
/// Lua type: nil
pub const LUA_TNIL: c_int = 0;
/// Lua type: boolean
pub const LUA_TBOOLEAN: c_int = 1;
/// Lua type: light userdata (pointer)
pub const LUA_TLIGHTUSERDATA: c_int = 2;
/// Lua type: number
pub const LUA_TNUMBER: c_int = 3;
/// Lua type: string
pub const LUA_TSTRING: c_int = 4;
/// Lua type: table
pub const LUA_TTABLE: c_int = 5;
/// Lua type: function
pub const LUA_TFUNCTION: c_int = 6;
/// Lua type: userdata
pub const LUA_TUSERDATA: c_int = 7;
/// Lua type: thread (coroutine)
pub const LUA_TTHREAD: c_int = 8;

/// Number of valid Lua types (excluding TNONE)
pub const LUA_NUMTYPES: c_int = 9;

// =============================================================================
// Lua type enum for Rust code
// =============================================================================

/// Lua value type enumeration
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LuaType {
    /// Invalid stack position
    None = -1,
    /// Nil value
    Nil = 0,
    /// Boolean value
    Boolean = 1,
    /// Light userdata (raw pointer)
    LightUserdata = 2,
    /// Number (floating point in LuaJIT)
    Number = 3,
    /// String (byte array)
    String = 4,
    /// Table
    Table = 5,
    /// Function (Lua or C)
    Function = 6,
    /// Full userdata
    Userdata = 7,
    /// Thread (coroutine)
    Thread = 8,
}

impl LuaType {
    /// Create a LuaType from a raw integer type code.
    ///
    /// Returns `None` for invalid type codes.
    #[must_use]
    pub const fn from_raw(type_code: c_int) -> Option<Self> {
        match type_code {
            -1 => Some(Self::None),
            0 => Some(Self::Nil),
            1 => Some(Self::Boolean),
            2 => Some(Self::LightUserdata),
            3 => Some(Self::Number),
            4 => Some(Self::String),
            5 => Some(Self::Table),
            6 => Some(Self::Function),
            7 => Some(Self::Userdata),
            8 => Some(Self::Thread),
            _ => Option::None,
        }
    }

    /// Check if this type is a "falsy" value in Lua.
    ///
    /// In Lua, only nil and false are falsy.
    #[must_use]
    pub const fn is_falsy(self) -> bool {
        matches!(self, Self::Nil | Self::None)
    }

    /// Check if this type is a valid value (not None).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Push/pop flags (matching converter.h)
// =============================================================================

/// Flags for nlua_push_* functions
pub mod push_flags {
    use std::ffi::c_int;

    /// Use lua-special-tbl when necessary
    pub const NLUA_PUSH_SPECIAL: c_int = 0x01;
    /// Free luarefs to elide an api_luarefs_free_*() later
    pub const NLUA_PUSH_FREE_REFS: c_int = 0x02;
}

// =============================================================================
// Type name strings
// =============================================================================

// Static type name C strings
static NONE_CSTR: &CStr = c"no value";
static NIL_CSTR: &CStr = c"nil";
static BOOLEAN_CSTR: &CStr = c"boolean";
static LIGHTUSERDATA_CSTR: &CStr = c"userdata";
static NUMBER_CSTR: &CStr = c"number";
static STRING_CSTR: &CStr = c"string";
static TABLE_CSTR: &CStr = c"table";
static FUNCTION_CSTR: &CStr = c"function";
static USERDATA_CSTR: &CStr = c"userdata";
static THREAD_CSTR: &CStr = c"thread";
static UNKNOWN_CSTR: &CStr = c"unknown";

/// Get the name of a Lua type as a C string.
///
/// Returns a pointer to a static null-terminated string.
///
/// # Arguments
/// * `lua_type` - The Lua type code (LUA_T* constants)
///
/// # Safety
///
/// Always returns a valid pointer to a static string.
#[no_mangle]
pub extern "C" fn rs_lua_type_name(lua_type: c_int) -> *const c_char {
    let cstr = match lua_type {
        LUA_TNONE => NONE_CSTR,
        LUA_TNIL => NIL_CSTR,
        LUA_TBOOLEAN => BOOLEAN_CSTR,
        LUA_TLIGHTUSERDATA => LIGHTUSERDATA_CSTR,
        LUA_TNUMBER => NUMBER_CSTR,
        LUA_TSTRING => STRING_CSTR,
        LUA_TTABLE => TABLE_CSTR,
        LUA_TFUNCTION => FUNCTION_CSTR,
        LUA_TUSERDATA => USERDATA_CSTR,
        LUA_TTHREAD => THREAD_CSTR,
        _ => UNKNOWN_CSTR,
    };
    cstr.as_ptr()
}

// Type name lookup table for index-based lookup
static TYPE_CSTRS: [&CStr; 9] = [
    NIL_CSTR,
    BOOLEAN_CSTR,
    LIGHTUSERDATA_CSTR,
    NUMBER_CSTR,
    STRING_CSTR,
    TABLE_CSTR,
    FUNCTION_CSTR,
    USERDATA_CSTR,
    THREAD_CSTR,
];

/// Convert a type name string to its Lua type index.
///
/// Returns the type index (0-8) on match, or -1 if not found.
///
/// # Arguments
/// * `name` - Null-terminated type name string
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_typename_to_idx(name: *const c_char) -> c_int {
    if name.is_null() {
        return LUA_TNONE;
    }

    let name_cstr = CStr::from_ptr(name);

    for (idx, type_cstr) in TYPE_CSTRS.iter().enumerate() {
        if name_cstr == *type_cstr {
            return c_int::try_from(idx).unwrap_or(LUA_TNONE);
        }
    }

    LUA_TNONE
}

// =============================================================================
// Special table keys (for vim.types)
// =============================================================================

/// Type index key value (true in Lua boolean)
pub const TYPE_IDX_VALUE: bool = true;
/// Value index key value (false in Lua boolean)
pub const VAL_IDX_VALUE: bool = false;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_type_constants() {
        // Verify constants match Lua C API
        assert_eq!(LUA_TNONE, -1);
        assert_eq!(LUA_TNIL, 0);
        assert_eq!(LUA_TBOOLEAN, 1);
        assert_eq!(LUA_TLIGHTUSERDATA, 2);
        assert_eq!(LUA_TNUMBER, 3);
        assert_eq!(LUA_TSTRING, 4);
        assert_eq!(LUA_TTABLE, 5);
        assert_eq!(LUA_TFUNCTION, 6);
        assert_eq!(LUA_TUSERDATA, 7);
        assert_eq!(LUA_TTHREAD, 8);
    }

    #[test]
    fn test_lua_type_enum() {
        assert_eq!(LuaType::None as i32, -1);
        assert_eq!(LuaType::Nil as i32, 0);
        assert_eq!(LuaType::Boolean as i32, 1);
        assert_eq!(LuaType::Number as i32, 3);
        assert_eq!(LuaType::String as i32, 4);
        assert_eq!(LuaType::Table as i32, 5);
        assert_eq!(LuaType::Function as i32, 6);
    }

    #[test]
    fn test_lua_type_from_raw() {
        assert_eq!(LuaType::from_raw(-1), Some(LuaType::None));
        assert_eq!(LuaType::from_raw(0), Some(LuaType::Nil));
        assert_eq!(LuaType::from_raw(5), Some(LuaType::Table));
        assert_eq!(LuaType::from_raw(100), None);
        assert_eq!(LuaType::from_raw(-100), None);
    }

    #[test]
    fn test_lua_type_is_falsy() {
        assert!(LuaType::None.is_falsy());
        assert!(LuaType::Nil.is_falsy());
        assert!(!LuaType::Boolean.is_falsy());
        assert!(!LuaType::Number.is_falsy());
        assert!(!LuaType::Table.is_falsy());
    }

    #[test]
    fn test_lua_type_is_valid() {
        assert!(!LuaType::None.is_valid());
        assert!(LuaType::Nil.is_valid());
        assert!(LuaType::Boolean.is_valid());
    }

    #[test]
    fn test_type_name() {
        let nil_name = rs_lua_type_name(LUA_TNIL);
        assert!(!nil_name.is_null());

        let boolean_name = rs_lua_type_name(LUA_TBOOLEAN);
        assert!(!boolean_name.is_null());

        let unknown_name = rs_lua_type_name(100);
        assert!(!unknown_name.is_null());
    }

    #[test]
    fn test_typename_to_idx() {
        unsafe {
            assert_eq!(rs_lua_typename_to_idx(c"nil".as_ptr()), 0);
            assert_eq!(rs_lua_typename_to_idx(c"boolean".as_ptr()), 1);
            assert_eq!(rs_lua_typename_to_idx(c"number".as_ptr()), 3);
            assert_eq!(rs_lua_typename_to_idx(c"string".as_ptr()), 4);
            assert_eq!(rs_lua_typename_to_idx(c"table".as_ptr()), 5);
            assert_eq!(rs_lua_typename_to_idx(c"function".as_ptr()), 6);

            // Unknown type
            assert_eq!(rs_lua_typename_to_idx(c"invalid".as_ptr()), -1);

            // Null pointer
            assert_eq!(rs_lua_typename_to_idx(std::ptr::null()), -1);
        }
    }

    #[test]
    fn test_push_flags() {
        assert_eq!(push_flags::NLUA_PUSH_SPECIAL, 0x01);
        assert_eq!(push_flags::NLUA_PUSH_FREE_REFS, 0x02);
    }

    #[test]
    fn test_type_idx_values() {
        // Verify the values are distinct
        assert_ne!(TYPE_IDX_VALUE, VAL_IDX_VALUE);
    }
}
