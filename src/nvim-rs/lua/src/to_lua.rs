//! Typval to Lua conversion helpers
//!
//! This module provides the real Rust implementations of nlua_push_* functions,
//! replacing the C bodies from converter.c (Phases 1 and 5).

#![allow(non_snake_case)]

use std::ffi::{c_char, c_int};

use crate::state::LuaState;
use crate::types::push_flags::{NLUA_PUSH_FREE_REFS, NLUA_PUSH_SPECIAL};
use nvim_api::{Array, Dict, NvimString, Object};

// =============================================================================
// Type aliases
// =============================================================================

/// Integer type (i64)
pub type Integer = i64;
/// Float type (f64)
pub type Float = f64;
/// Boolean type
pub type Boolean = bool;
/// Handle type for Buffer, Window, Tabpage
pub type Handle = c_int;

/// LuaRef is int in C (typedef int LuaRef)
type LuaRef = c_int;

// LUA_NOREF constant
const LUA_NOREF: LuaRef = -2;

// kObjectType constants (matching C enum values)
const K_OBJECT_TYPE_NIL: c_int = 0;
const K_OBJECT_TYPE_BOOLEAN: c_int = 1;
const K_OBJECT_TYPE_INTEGER: c_int = 2;
const K_OBJECT_TYPE_FLOAT: c_int = 3;
const K_OBJECT_TYPE_STRING: c_int = 4;
const K_OBJECT_TYPE_ARRAY: c_int = 5;
const K_OBJECT_TYPE_DICT: c_int = 6;
const K_OBJECT_TYPE_LUAREF: c_int = 7;
const K_OBJECT_TYPE_BUFFER: c_int = 8;
const K_OBJECT_TYPE_WINDOW: c_int = 9;
const K_OBJECT_TYPE_TABPAGE: c_int = 10;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Lua C API
    fn lua_pushnil(lstate: *mut LuaState);
    fn lua_pushnumber(lstate: *mut LuaState, n: f64);
    fn lua_pushboolean(lstate: *mut LuaState, b: c_int);
    fn lua_pushlstring(lstate: *mut LuaState, s: *const c_char, len: usize);
    fn lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_rawset(lstate: *mut LuaState, idx: c_int);
    fn lua_rawseti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_setmetatable(lstate: *mut LuaState, objindex: c_int) -> c_int;
}

// Call into the Rust implementations directly to avoid redeclaring externs
// with conflicting types. refs.rs uses LuaRef=c_int and leaf.rs also uses c_int.
#[inline]
unsafe fn pushref(lstate: *mut LuaState, ref_: LuaRef) {
    crate::refs::rs_nlua_pushref(lstate, ref_);
}

#[inline]
unsafe fn free_luaref(ref_: LuaRef) {
    crate::refs::rs_api_free_luaref(ref_);
}

#[inline]
unsafe fn get_nil_ref(lstate: *mut LuaState) -> LuaRef {
    crate::leaf::rs_nlua_get_nil_ref(lstate)
}

#[inline]
unsafe fn get_empty_dict_ref(lstate: *mut LuaState) -> LuaRef {
    crate::leaf::rs_nlua_get_empty_dict_ref(lstate)
}

// =============================================================================
// Private inline helpers (were static inline in C, Rust-only here)
// =============================================================================

/// Push the type index key (true boolean) onto the stack.
///
/// Used as the key for all "typed" Lua tables representing Vimscript values.
#[inline]
unsafe fn nlua_push_type_idx(lstate: *mut LuaState) {
    lua_pushboolean(lstate, 1); // TYPE_IDX_VALUE = true
}

/// Push the value index key (false boolean) onto the stack.
///
/// Used as the key for tables representing scalar values like float.
#[inline]
unsafe fn nlua_push_val_idx(lstate: *mut LuaState) {
    lua_pushboolean(lstate, 0); // VAL_IDX_VALUE = false
}

/// Push a type number onto the stack.
#[inline]
unsafe fn nlua_push_type(lstate: *mut LuaState, type_val: c_int) {
    lua_pushnumber(lstate, f64::from(type_val));
}

/// Create a Lua table tagged with its Vimscript type.
///
/// Leaves a table with a `type_idx => type_val` entry already inserted.
pub unsafe fn nlua_create_typed_table(
    lstate: *mut LuaState,
    narr: usize,
    nrec: usize,
    type_val: c_int,
) {
    // narr and nrec are table size hints; silently truncating large tables is safe here.
    #[allow(clippy::cast_possible_truncation)]
    let narr_i = narr as c_int;
    #[allow(clippy::cast_possible_truncation)]
    let nrec_i = (1 + nrec) as c_int;
    lua_createtable(lstate, narr_i, nrec_i);
    nlua_push_type_idx(lstate);
    nlua_push_type(lstate, type_val);
    lua_rawset(lstate, -3);
}

// =============================================================================
// Phase 1: Trivial scalar pushers
// These replace the C bodies in converter.c.
// =============================================================================

/// Convert a String to a Lua string and push it onto the stack.
#[unsafe(export_name = "nlua_push_String")]
pub unsafe extern "C" fn rs_nlua_push_String(lstate: *mut LuaState, s: NvimString, _flags: c_int) {
    let ptr = if s.size != 0 && !s.data.is_null() {
        s.data.cast_const()
    } else {
        c"".as_ptr()
    };
    lua_pushlstring(lstate, ptr, s.size);
}

/// Convert an Integer to a Lua number and push it onto the stack.
#[unsafe(export_name = "nlua_push_Integer")]
pub unsafe extern "C" fn rs_nlua_push_Integer(lstate: *mut LuaState, n: Integer, _flags: c_int) {
    #[allow(clippy::cast_precision_loss)]
    lua_pushnumber(lstate, n as f64);
}

/// Convert a Float to a Lua value and push it onto the stack.
///
/// If `kNluaPushSpecial` is set in flags, pushes a typed float table; otherwise a plain number.
#[unsafe(export_name = "nlua_push_Float")]
pub unsafe extern "C" fn rs_nlua_push_Float(lstate: *mut LuaState, f: Float, flags: c_int) {
    if flags & NLUA_PUSH_SPECIAL != 0 {
        nlua_create_typed_table(lstate, 0, 1, K_OBJECT_TYPE_FLOAT);
        nlua_push_val_idx(lstate);
        lua_pushnumber(lstate, f);
        lua_rawset(lstate, -3);
    } else {
        lua_pushnumber(lstate, f);
    }
}

/// Convert a Boolean to a Lua boolean and push it onto the stack.
#[unsafe(export_name = "nlua_push_Boolean")]
pub unsafe extern "C" fn rs_nlua_push_Boolean(lstate: *mut LuaState, b: Boolean, _flags: c_int) {
    lua_pushboolean(lstate, c_int::from(b));
}

/// Push a handle (buffer/window/tabpage integer) onto the stack.
#[unsafe(export_name = "nlua_push_handle")]
pub unsafe extern "C" fn rs_nlua_push_handle(lstate: *mut LuaState, item: Handle, _flags: c_int) {
    lua_pushnumber(lstate, f64::from(item));
}

// =============================================================================
// Phase 5: Recursive/composite pushers
// These replace the C bodies for push_Array, push_Dict, push_Object.
// =============================================================================

/// Convert an Array to a Lua table (1-based integer keys) and push it onto the stack.
#[unsafe(export_name = "nlua_push_Array")]
pub unsafe extern "C" fn rs_nlua_push_Array(lstate: *mut LuaState, array: Array, flags: c_int) {
    #[allow(clippy::cast_possible_truncation)]
    lua_createtable(lstate, array.size as c_int, 0);
    for i in 0..array.size {
        let item = &mut *array.items.add(i);
        rs_nlua_push_Object(lstate, item, flags);
        #[allow(clippy::cast_possible_truncation)]
        lua_rawseti(lstate, -2, (i + 1) as c_int);
    }
}

/// Convert a Dict to a Lua table (string keys) and push it onto the stack.
///
/// An empty dict gets the empty_dict metatable to distinguish it from an empty array.
#[unsafe(export_name = "nlua_push_Dict")]
pub unsafe extern "C" fn rs_nlua_push_Dict(lstate: *mut LuaState, dict: Dict, flags: c_int) {
    #[allow(clippy::cast_possible_truncation)]
    lua_createtable(lstate, 0, dict.size as c_int);
    if dict.size == 0 {
        let empty_ref = get_empty_dict_ref(lstate);
        pushref(lstate, empty_ref);
        lua_setmetatable(lstate, -2);
    }
    for i in 0..dict.size {
        let kv = &*dict.items.add(i);
        rs_nlua_push_String(lstate, kv.key, flags);
        // kv.value is conceptually const here; cast to *mut is safe because
        // push_Object only reads through the pointer when dealing with value types.
        let val_ptr: *mut Object = std::ptr::from_ref(&kv.value).cast_mut();
        rs_nlua_push_Object(lstate, val_ptr, flags);
        lua_rawset(lstate, -3);
    }
}

/// Convert an Object to a Lua value and push it onto the stack.
///
/// Dispatches to the appropriate typed push function based on `obj.type`.
#[unsafe(export_name = "nlua_push_Object")]
pub unsafe extern "C" fn rs_nlua_push_Object(
    lstate: *mut LuaState,
    obj: *mut Object,
    flags: c_int,
) {
    let obj_ref = &mut *obj;
    match obj_ref.obj_type {
        K_OBJECT_TYPE_NIL => {
            if flags & NLUA_PUSH_SPECIAL != 0 {
                lua_pushnil(lstate);
            } else {
                let nil_ref = get_nil_ref(lstate);
                pushref(lstate, nil_ref);
            }
        }
        K_OBJECT_TYPE_LUAREF => {
            // data.luaref is stored as i64 in ObjectData but real LuaRef is c_int.
            // On little-endian the low 32 bits are the actual int value.
            #[allow(clippy::cast_possible_truncation)]
            let luaref = obj_ref.data.luaref as c_int;
            pushref(lstate, luaref);
            if flags & NLUA_PUSH_FREE_REFS != 0 {
                free_luaref(luaref);
                // Write LUA_NOREF back into the field
                obj_ref.data.luaref = i64::from(LUA_NOREF);
            }
        }
        K_OBJECT_TYPE_BOOLEAN => {
            rs_nlua_push_Boolean(lstate, obj_ref.data.boolean, flags);
        }
        K_OBJECT_TYPE_INTEGER => {
            rs_nlua_push_Integer(lstate, obj_ref.data.integer, flags);
        }
        K_OBJECT_TYPE_FLOAT => {
            rs_nlua_push_Float(lstate, obj_ref.data.floating, flags);
        }
        K_OBJECT_TYPE_STRING => {
            rs_nlua_push_String(lstate, obj_ref.data.string, flags);
        }
        K_OBJECT_TYPE_ARRAY => {
            rs_nlua_push_Array(lstate, obj_ref.data.array, flags);
        }
        K_OBJECT_TYPE_DICT => {
            rs_nlua_push_Dict(lstate, obj_ref.data.dict, flags);
        }
        // Remote handle types: Buffer=8, Window=9, Tabpage=10
        // data.integer stores the handle value
        K_OBJECT_TYPE_BUFFER | K_OBJECT_TYPE_WINDOW | K_OBJECT_TYPE_TABPAGE => {
            #[allow(clippy::cast_precision_loss)]
            lua_pushnumber(lstate, obj_ref.data.integer as f64);
        }
        _ => {
            // Unreachable in practice; match C behavior of falling off switch
        }
    }
}

// =============================================================================
// Direct Lua stack operations (kept for Rust consumers that import this module)
// =============================================================================

/// Push nil onto the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushnil(lstate: *mut LuaState) {
    lua_pushnil(lstate);
}

/// Push a number onto the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushnumber(lstate: *mut LuaState, n: f64) {
    lua_pushnumber(lstate, n);
}

/// Push a boolean onto the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushboolean(lstate: *mut LuaState, b: bool) {
    lua_pushboolean(lstate, c_int::from(b));
}

/// Push a string onto the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pushlstring(lstate: *mut LuaState, s: *const c_char, len: usize) {
    lua_pushlstring(lstate, s, len);
}

// =============================================================================
// Phase 3: nlua_init_types
// =============================================================================

/// Record auxiliary type values in the vim module table.
///
/// Assumes the vim module table is on top of the stack.
/// Populates `vim.type_idx`, `vim.val_idx`, and `vim.types`.
///
/// This is the Rust replacement for the C `nlua_init_types` in converter.c.
#[unsafe(export_name = "nlua_init_types")]
pub unsafe extern "C" fn rs_nlua_init_types(lstate: *mut LuaState) {
    // vim.type_idx = true (the type-index sentinel key)
    lua_pushlstring(lstate, c"type_idx".as_ptr(), 8);
    nlua_push_type_idx(lstate);
    lua_rawset(lstate, -3);

    // vim.val_idx = false (the value-index sentinel key)
    lua_pushlstring(lstate, c"val_idx".as_ptr(), 7);
    nlua_push_val_idx(lstate);
    lua_rawset(lstate, -3);

    // vim.types = { float=3, [3]="float", array=5, [5]="array", dictionary=6, [6]="dictionary" }
    lua_pushlstring(lstate, c"types".as_ptr(), 5);
    lua_createtable(lstate, 0, 3);

    lua_pushlstring(lstate, c"float".as_ptr(), 5);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_FLOAT));
    lua_rawset(lstate, -3);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_FLOAT));
    lua_pushlstring(lstate, c"float".as_ptr(), 5);
    lua_rawset(lstate, -3);

    lua_pushlstring(lstate, c"array".as_ptr(), 5);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_ARRAY));
    lua_rawset(lstate, -3);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_ARRAY));
    lua_pushlstring(lstate, c"array".as_ptr(), 5);
    lua_rawset(lstate, -3);

    lua_pushlstring(lstate, c"dictionary".as_ptr(), 10);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_DICT));
    lua_rawset(lstate, -3);
    lua_pushnumber(lstate, f64::from(K_OBJECT_TYPE_DICT));
    lua_pushlstring(lstate, c"dictionary".as_ptr(), 10);
    lua_rawset(lstate, -3);

    lua_rawset(lstate, -3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_constants() {
        assert_eq!(K_OBJECT_TYPE_NIL, 0);
        assert_eq!(K_OBJECT_TYPE_BOOLEAN, 1);
        assert_eq!(K_OBJECT_TYPE_INTEGER, 2);
        assert_eq!(K_OBJECT_TYPE_FLOAT, 3);
        assert_eq!(K_OBJECT_TYPE_STRING, 4);
        assert_eq!(K_OBJECT_TYPE_ARRAY, 5);
        assert_eq!(K_OBJECT_TYPE_DICT, 6);
        assert_eq!(K_OBJECT_TYPE_LUAREF, 7);
        assert_eq!(K_OBJECT_TYPE_BUFFER, 8);
        assert_eq!(K_OBJECT_TYPE_WINDOW, 9);
        assert_eq!(K_OBJECT_TYPE_TABPAGE, 10);
    }

    #[test]
    fn test_push_flags() {
        assert_eq!(NLUA_PUSH_SPECIAL, 0x01);
        assert_eq!(NLUA_PUSH_FREE_REFS, 0x02);
    }
}
