//! Lua to Neovim API value conversion helpers
//!
//! This module provides real Rust implementations of nlua_pop_* functions,
//! replacing C bodies from converter.c. Phase 2: scalar poppers.

#![allow(non_snake_case)]

use std::ffi::{c_char, c_int};

use crate::state::LuaState;
use crate::types::{
    LUA_TBOOLEAN, LUA_TFUNCTION, LUA_TNIL, LUA_TNUMBER, LUA_TSTRING, LUA_TTABLE, LUA_TUSERDATA,
};
use nvim_api::{Array, Dict, KeyValuePair, NvimString, Object};

/// Float type (f64)
pub type Float = f64;
/// Integer type (i64)
pub type Integer = i64;
/// Boolean type
pub type Boolean = bool;

/// LuaRef is typedef int in C
type LuaRef = c_int;

// API_INTEGER_MIN corresponds to INT64_MIN; MAX not needed (literal used instead)
const API_INTEGER_MIN: Integer = i64::MIN;

// kErrorTypeValidation = 1
const K_ERROR_VALIDATION: c_int = 1;
// kErrorTypeException = 0
const K_ERROR_EXCEPTION: c_int = 0;

// =============================================================================
// Error type (matches Neovim's Error struct)
// =============================================================================

/// Error type enum
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    None = -1,
    Exception = 0,
    Validation = 1,
}

/// Error struct matching Neovim's Error
#[repr(C)]
pub struct Error {
    pub r#type: ErrorType,
    pub msg: *mut c_char,
}

impl Error {
    /// Check if an error is set
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.r#type != ErrorType::None
    }
}

/// Arena type (opaque handle)
#[repr(C)]
pub struct Arena {
    _private: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Lua C API functions for reading values
    fn lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64;
    fn lua_toboolean(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_gettop(lstate: *mut LuaState) -> c_int;

    // API error function (variadic; only used with literal format strings below)
    fn api_set_error(err: *mut Error, err_type: c_int, fmt: *const c_char, ...);

    // arena_memdupz: used by rs_nlua_pop_String (Phase 3)
    fn arena_memdupz(arena: *mut Arena, data: *const c_char, len: usize) -> *mut c_char;

    // Table traversal helpers (Phase 4+6)
    fn lua_pushnil(lstate: *mut LuaState);
    fn lua_next(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_rawgeti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_getmetatable(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_rawequal(lstate: *mut LuaState, idx1: c_int, idx2: c_int) -> c_int;
    fn lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int;
    fn lua_pushvalue(lstate: *mut LuaState, idx: c_int);
    fn semsg(fmt: *const c_char, ...);
    fn api_typename(obj_type: c_int) -> *const c_char;

    // Arena allocation (Phase 6)
    fn arena_array(arena: *mut Arena, max_size: usize) -> Array;
    fn arena_dict(arena: *mut Arena, max_size: usize) -> Dict;
    fn api_free_array(value: Array);
    fn api_free_dict(value: Dict);
    fn api_free_object(value: Object);

    // Still-C pop functions (Phase 10 will replace)
    fn nlua_pop_typval(lstate: *mut LuaState, ret_tv: *mut std::ffi::c_void) -> bool;
}

// lua_pop is a macro: #define lua_pop(L,n) lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// =============================================================================
// Phase 4: Internal table-traversal helpers (Rust-private, no C symbol export)
// =============================================================================

// kObjectType* constants needed for traverse_table / check_type / pop_Object
const K_OBJECT_TYPE_NIL: c_int = 0;
const K_OBJECT_TYPE_BOOLEAN: c_int = 1;
const K_OBJECT_TYPE_INTEGER: c_int = 2;
const K_OBJECT_TYPE_FLOAT: c_int = 3;
const K_OBJECT_TYPE_STRING: c_int = 4;
const K_OBJECT_TYPE_ARRAY: c_int = 5;
const K_OBJECT_TYPE_DICT: c_int = 6;
const K_OBJECT_TYPE_LUAREF: c_int = 7;

// TYPE_IDX_VALUE is `true` (boolean true = 1 from lua_toboolean)
// VAL_IDX_VALUE is `false` (0)
const TYPE_IDX_VALUE: c_int = 1;

/// Properties of a Lua table as determined by traverse_table.
///
/// Mirrors the C `LuaTableProps` struct.
#[derive(Debug, Clone, Copy, Default)]
struct LuaTableProps {
    /// Maximum positive integral key found (= length if sequential).
    maxidx: usize,
    /// Number of string-typed keys.
    string_keys_num: usize,
    /// True if at least one string key contains a NUL byte.
    has_string_with_nul: bool,
    /// Inferred object type of the table.
    obj_type: c_int,
    /// If has_type_key && has_val_key && val is number: the float value.
    val: f64,
    /// True if a boolean-true key (type_idx) was found.
    has_type_key: bool,
}

/// Inspect a Lua table on top of the stack and classify its type.
///
/// Does not pop the table. Mirrors `nlua_traverse_table` from converter.c.
///
/// # Safety
/// `lstate` must be a valid Lua state with a table at the top.
#[allow(clippy::too_many_lines)]
unsafe fn traverse_table(lstate: *mut LuaState) -> LuaTableProps {
    let mut props = LuaTableProps {
        obj_type: K_OBJECT_TYPE_NIL,
        ..LuaTableProps::default()
    };

    if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
        semsg(
            c"E1502: Lua failed to grow stack to %i".as_ptr(),
            lua_gettop(lstate) + 2,
        );
        return props;
    }

    let mut tsize: usize = 0;
    let mut val_type: c_int = 0;
    let mut has_val_key = false;
    let mut other_keys_num: usize = 0;

    lua_pushnil(lstate);
    while lua_next(lstate, -2) != 0 {
        match lua_type(lstate, -2) {
            t if t == LUA_TSTRING => {
                let mut len: usize = 0;
                let s = lua_tolstring(lstate, -2, &raw mut len);
                // Check for embedded NUL bytes
                let slice = std::slice::from_raw_parts(s.cast::<u8>(), len);
                if slice.contains(&0) {
                    props.has_string_with_nul = true;
                }
                props.string_keys_num += 1;
            }
            t if t == LUA_TNUMBER => {
                let n = lua_tonumber(lstate, -2);
                // A valid array index is a positive integer in [1, usize::MAX].
                // usize::MAX as f64 rounds up on 64-bit, use 2^53 as a safe cap.
                let max_exact_f64: f64 = 9_007_199_254_740_992.0; // 2^53
                let n_trunc = n.trunc();
                if n_trunc < 1.0 || n_trunc > max_exact_f64 {
                    other_keys_num += 1;
                } else {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let idx = n_trunc as usize;
                    if idx > props.maxidx {
                        props.maxidx = idx;
                    }
                }
            }
            t if t == LUA_TBOOLEAN => {
                let b = lua_toboolean(lstate, -2);
                if b == TYPE_IDX_VALUE {
                    // This is the type_idx key (boolean true)
                    if lua_type(lstate, -1) == LUA_TNUMBER {
                        let n = lua_tonumber(lstate, -1);
                        #[allow(clippy::cast_possible_truncation)]
                        let n_int = n as c_int;
                        if n_int == K_OBJECT_TYPE_FLOAT
                            || n_int == K_OBJECT_TYPE_ARRAY
                            || n_int == K_OBJECT_TYPE_DICT
                        {
                            props.has_type_key = true;
                            props.obj_type = n_int;
                        } else {
                            other_keys_num += 1;
                        }
                    } else {
                        other_keys_num += 1;
                    }
                } else {
                    // This is the val_idx key (boolean false)
                    has_val_key = true;
                    val_type = lua_type(lstate, -1);
                    if val_type == LUA_TNUMBER {
                        props.val = lua_tonumber(lstate, -1);
                    }
                }
            }
            _ => {
                other_keys_num += 1;
            }
        }
        tsize += 1;
        lua_pop(lstate, 1);
    }

    if props.has_type_key {
        if props.obj_type == K_OBJECT_TYPE_FLOAT && (!has_val_key || val_type != LUA_TNUMBER) {
            props.obj_type = K_OBJECT_TYPE_NIL;
        } else if props.obj_type == K_OBJECT_TYPE_ARRAY {
            // Recompute maxidx to be the last index in the *sequential* prefix.
            // This guards against {[type_idx]=array, [SIZE_MAX]=1} crashing nvim.
            let expected = tsize
                - usize::from(props.has_type_key)
                - other_keys_num
                - usize::from(has_val_key)
                - props.string_keys_num;
            if props.maxidx != 0 && props.maxidx != expected {
                props.maxidx = 0;
                loop {
                    #[allow(clippy::cast_possible_truncation)]
                    lua_rawgeti(lstate, -1, props.maxidx as c_int + 1);
                    if lua_type(lstate, -1) == LUA_TNIL {
                        lua_pop(lstate, 1);
                        break;
                    }
                    lua_pop(lstate, 1);
                    props.maxidx += 1;
                }
            }
        }
    } else if tsize == 0
        || (tsize <= props.maxidx && other_keys_num == 0 && props.string_keys_num == 0)
    {
        props.obj_type = K_OBJECT_TYPE_ARRAY;
        // Check if table has the empty-dict metatable
        if tsize == 0 && lua_getmetatable(lstate, -1) != 0 {
            let empty_dict_ref = crate::leaf::rs_nlua_get_empty_dict_ref(lstate);
            crate::refs::rs_nlua_pushref(lstate, empty_dict_ref);
            if lua_rawequal(lstate, -2, -1) != 0 {
                props.obj_type = K_OBJECT_TYPE_DICT;
            }
            lua_pop(lstate, 2);
        }
    } else if props.string_keys_num == tsize {
        props.obj_type = K_OBJECT_TYPE_DICT;
    } else {
        props.obj_type = K_OBJECT_TYPE_NIL;
    }

    props
}

/// Classify the table on top of the stack and validate it has the expected type.
///
/// Mirrors `nlua_check_type` from converter.c.
/// Does NOT pop the table.
///
/// # Safety
/// `lstate` must be a valid Lua state with a table at the top.
unsafe fn check_type(lstate: *mut LuaState, err: *mut Error, expected: c_int) -> LuaTableProps {
    if lua_type(lstate, -1) != LUA_TTABLE {
        if !err.is_null() {
            if expected == K_OBJECT_TYPE_FLOAT {
                api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua number".as_ptr());
            } else {
                api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua table".as_ptr());
            }
        }
        return LuaTableProps::default();
    }
    let mut props = traverse_table(lstate);

    // Allow empty array to be treated as dict
    if expected == K_OBJECT_TYPE_DICT
        && props.obj_type == K_OBJECT_TYPE_ARRAY
        && props.maxidx == 0
        && !props.has_type_key
    {
        props.obj_type = K_OBJECT_TYPE_DICT;
    }

    if props.obj_type != expected && !err.is_null() {
        api_set_error(
            err,
            K_ERROR_VALIDATION,
            c"Expected %s-like Lua table".as_ptr(),
            api_typename(expected),
        );
    }

    props
}

// =============================================================================
// Phase 2: Trivial scalar poppers (real implementations)
// =============================================================================

/// Convert a Lua number to Integer.
///
/// Validates the top-of-stack value is a number and fits in Integer range.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Integer")]
pub unsafe extern "C" fn rs_nlua_pop_Integer(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    err: *mut Error,
) -> Integer {
    if lua_type(lstate, -1) != LUA_TNUMBER {
        lua_pop(lstate, 1);
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua number".as_ptr());
        return 0;
    }
    let n = lua_tonumber(lstate, -1);
    lua_pop(lstate, 1);
    // i64::MAX as f64 rounds up, so compare with next-lower exact power of two
    #[allow(clippy::cast_precision_loss)]
    let in_range = n >= (API_INTEGER_MIN as f64) && n < 9_223_372_036_854_775_808.0_f64;
    let is_integral = n.fract() == 0.0;
    if !in_range || !is_integral {
        api_set_error(err, K_ERROR_EXCEPTION, c"Number is not integral".as_ptr());
        return 0;
    }
    #[allow(clippy::cast_possible_truncation)]
    {
        n as Integer
    }
}

/// Convert a Lua value to Boolean using Lua semantics.
///
/// Any value can be coerced to boolean (nil and false are falsy).
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Boolean")]
pub unsafe extern "C" fn rs_nlua_pop_Boolean(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    _err: *mut Error,
) -> Boolean {
    let ret = lua_toboolean(lstate, -1) != 0;
    lua_pop(lstate, 1);
    ret
}

/// Convert a Lua value to Boolean with strict API validation.
///
/// Follows API conventions: only boolean, number, or nil are accepted.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Boolean_strict")]
pub unsafe extern "C" fn rs_nlua_pop_Boolean_strict(
    lstate: *mut LuaState,
    err: *mut Error,
) -> Boolean {
    let ret = match lua_type(lstate, -1) {
        t if t == LUA_TBOOLEAN => lua_toboolean(lstate, -1) != 0,
        t if t == LUA_TNUMBER => lua_tonumber(lstate, -1) != 0.0,
        t if t == LUA_TNIL => false,
        _ => {
            api_set_error(err, K_ERROR_VALIDATION, c"not a boolean".as_ptr());
            false
        }
    };
    lua_pop(lstate, 1);
    ret
}

/// Pop a LuaRef from the Lua stack.
///
/// Creates a global reference to the top-of-stack value and pops it.
#[unsafe(export_name = "nlua_pop_LuaRef")]
pub unsafe extern "C" fn rs_nlua_pop_LuaRef(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    _err: *mut Error,
) -> LuaRef {
    let rv = crate::refs::rs_nlua_ref_global(lstate, -1);
    lua_pop(lstate, 1);
    rv
}

/// Pop a handle (Buffer/Window/Tabpage integer) from the Lua stack.
///
/// Validates the top-of-stack value is a Lua number.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_handle")]
pub unsafe extern "C" fn rs_nlua_pop_handle(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    err: *mut Error,
) -> c_int {
    let ret = if lua_type(lstate, -1) == LUA_TNUMBER {
        #[allow(clippy::cast_possible_truncation)]
        {
            lua_tonumber(lstate, -1) as c_int
        }
    } else {
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua number".as_ptr());
        -1
    };
    lua_pop(lstate, 1);
    ret
}

// =============================================================================
// Still-forwarding stubs for Phases 3, 6, 7 (pop_String, pop_Float,
// pop_Array, pop_Dict, pop_Object, pop_typval). These will be replaced
// with real implementations in later phases.
// =============================================================================

/// Pop a String from the Lua stack.
///
/// Validates the top-of-stack value is a Lua string, copies it into the
/// arena, and pops it. Returns an empty String and sets `err` on failure.
#[unsafe(export_name = "nlua_pop_String")]
pub unsafe extern "C" fn rs_nlua_pop_String(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> NvimString {
    if lua_type(lstate, -1) != LUA_TSTRING {
        lua_pop(lstate, 1);
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua string".as_ptr());
        return NvimString {
            data: std::ptr::null_mut(),
            size: 0,
        };
    }
    let mut size: usize = 0;
    let data_ptr = lua_tolstring(lstate, -1, &raw mut size);
    // Safety: lua_tolstring returns non-null for string values.
    let data = arena_memdupz(arena, data_ptr, size);
    lua_pop(lstate, 1);
    NvimString { data, size }
}

/// Pop a Float from the Lua stack (Phase 6 real implementation).
///
/// Accepts either a plain Lua number or a typed float table.
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Float")]
pub unsafe extern "C" fn rs_nlua_pop_Float(
    lstate: *mut LuaState,
    _arena: *mut Arena,
    err: *mut Error,
) -> Float {
    if lua_type(lstate, -1) == LUA_TNUMBER {
        let ret = lua_tonumber(lstate, -1);
        lua_pop(lstate, 1);
        return ret;
    }
    let props = check_type(lstate, err, K_OBJECT_TYPE_FLOAT);
    lua_pop(lstate, 1);
    if props.obj_type != K_OBJECT_TYPE_FLOAT {
        return 0.0;
    }
    props.val
}

// =============================================================================
// Phase 6: pop_Array and pop_Dict (real implementations)
// =============================================================================

/// Convert a Lua table to an Array without pre-checking the type.
///
/// `table_props` must have been produced by `traverse_table`.
/// Pops the table from the stack.
unsafe fn pop_Array_unchecked(
    lstate: *mut LuaState,
    props: LuaTableProps,
    arena: *mut Arena,
    err: *mut Error,
) -> Array {
    let mut ret = arena_array(arena, props.maxidx);

    if props.maxidx == 0 {
        lua_pop(lstate, 1);
        return ret;
    }

    for i in 1..=props.maxidx {
        #[allow(clippy::cast_possible_truncation)]
        lua_rawgeti(lstate, -1, i as c_int);
        let val = rs_nlua_pop_Object(lstate, false, arena, err);
        if (*err).is_set() {
            lua_pop(lstate, 1);
            if arena.is_null() {
                api_free_array(ret);
            }
            return Array {
                size: 0,
                capacity: 0,
                items: std::ptr::null_mut(),
            };
        }
        // ADD_C: append to pre-allocated array (capacity guaranteed by arena_array)
        let idx = ret.size;
        ret.size += 1;
        ret.items.add(idx).write(val);
    }
    lua_pop(lstate, 1);
    ret
}

/// Convert a Lua table to an Array.
///
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Array")]
pub unsafe extern "C" fn rs_nlua_pop_Array(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> Array {
    let props = check_type(lstate, err, K_OBJECT_TYPE_ARRAY);
    if props.obj_type != K_OBJECT_TYPE_ARRAY {
        return Array {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
    }
    pop_Array_unchecked(lstate, props, arena, err)
}

/// Convert a Lua table to a Dict without pre-checking the type.
///
/// `table_props` must have been produced by `traverse_table`.
/// Pops the table from the stack.
unsafe fn pop_Dict_unchecked(
    lstate: *mut LuaState,
    props: LuaTableProps,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    let mut ret = arena_dict(arena, props.string_keys_num);

    if props.string_keys_num == 0 {
        lua_pop(lstate, 1);
        return ret;
    }

    lua_pushnil(lstate);
    let mut i: usize = 0;
    while lua_next(lstate, -2) != 0 && i < props.string_keys_num {
        // stack: dict, key, value
        if lua_type(lstate, -2) == LUA_TSTRING {
            // Duplicate the key so pop_String can consume it without disturbing lua_next
            lua_pushvalue(lstate, -2);
            // stack: dict, key, value, key_dup

            let key = rs_nlua_pop_String(lstate, arena, err);
            // stack: dict, key, value

            if (*err).is_set() {
                lua_pop(lstate, 1);
                // stack: dict, key
            } else {
                let value = rs_nlua_pop_Object(lstate, ref_, arena, err);
                // kv_push_c: append KeyValuePair to pre-allocated dict
                let idx = ret.size;
                ret.size += 1;
                ret.items.add(idx).write(KeyValuePair { key, value });
                // stack: dict, key
            }

            if (*err).is_set() {
                if arena.is_null() {
                    api_free_dict(ret);
                }
                lua_pop(lstate, 2); // pop key and dict
                return Dict {
                    size: 0,
                    capacity: 0,
                    items: std::ptr::null_mut(),
                };
            }
            i += 1;
        } else {
            lua_pop(lstate, 1); // pop value, keep key for lua_next
        }
    }
    lua_pop(lstate, 1); // pop the dict
    ret
}

/// Convert a Lua table to a Dict.
///
/// Always pops one value from the stack.
#[unsafe(export_name = "nlua_pop_Dict")]
pub unsafe extern "C" fn rs_nlua_pop_Dict(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Dict {
    let props = check_type(lstate, err, K_OBJECT_TYPE_DICT);
    if props.obj_type != K_OBJECT_TYPE_DICT {
        lua_pop(lstate, 1);
        return Dict {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
    }
    pop_Dict_unchecked(lstate, props, ref_, arena, err)
}

// =============================================================================
// Phase 7: nlua_pop_Object (real implementation)
// =============================================================================

/// Helper for the pop_Object iterative stack.
#[derive(Copy, Clone)]
struct ObjPopStackItem {
    /// Pointer to the Object slot being filled.
    obj: *mut Object,
    /// True if we are iterating into a container (array/dict).
    container: bool,
}

/// Convert the Lua value on top of the stack to an Object.
///
/// Always pops one value from the stack. Mirrors `nlua_pop_Object` from converter.c.
///
/// # Safety
/// `lstate` must be valid; `err` must be a valid Error pointer.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "nlua_pop_Object")]
pub unsafe extern "C" fn rs_nlua_pop_Object(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    use nvim_api::ObjectData;

    let mut ret = Object::nil();
    let initial_size = lua_gettop(lstate);

    // Small-vector: capacity 2 matches kvi_init default
    let mut stack: Vec<ObjPopStackItem> = Vec::with_capacity(2);
    stack.push(ObjPopStackItem {
        obj: &raw mut ret,
        container: false,
    });

    while !(*err).is_set() && !stack.is_empty() {
        let mut cur = stack.pop().unwrap();

        if cur.container {
            if !lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
                api_set_error(err, K_ERROR_EXCEPTION, c"Lua failed to grow stack".as_ptr());
                break;
            }
            if (*cur.obj).obj_type == K_OBJECT_TYPE_DICT {
                // stack: …, dict, key
                let dict = &mut (*cur.obj).data.dict;
                if dict.size == dict.capacity {
                    lua_pop(lstate, 2);
                    continue;
                }
                let mut next_key_found = false;
                while lua_next(lstate, -2) != 0 {
                    // stack: …, dict, new key, val
                    if lua_type(lstate, -2) == LUA_TSTRING {
                        next_key_found = true;
                        break;
                    }
                    lua_pop(lstate, 1);
                    // stack: …, dict, new key
                }
                if next_key_found {
                    // stack: …, dict, new key, val
                    let mut len: usize = 0;
                    let s = lua_tolstring(lstate, -2, &raw mut len);
                    let idx = dict.size;
                    dict.size += 1;
                    let key_data = arena_memdupz(arena, s, len);
                    let key = NvimString {
                        data: key_data,
                        size: len,
                    };
                    let val_ptr = &raw mut dict.items.add(idx).as_mut().unwrap().value;
                    dict.items.add(idx).as_mut().unwrap().key = key;
                    stack.push(cur);
                    cur = ObjPopStackItem {
                        obj: val_ptr,
                        container: false,
                    };
                } else {
                    // stack: …, dict
                    lua_pop(lstate, 1);
                    // stack: …
                    continue;
                }
            } else {
                // Array container
                let arr = &mut (*cur.obj).data.array;
                if arr.size == arr.capacity {
                    lua_pop(lstate, 1);
                    continue;
                }
                let idx = arr.size;
                arr.size += 1;
                #[allow(clippy::cast_possible_truncation)]
                lua_rawgeti(lstate, -1, idx as c_int + 1);
                let elem_ptr = arr.items.add(idx);
                stack.push(cur);
                cur = ObjPopStackItem {
                    obj: elem_ptr,
                    container: false,
                };
            }
        }

        // cur is now a leaf slot to fill
        *cur.obj = Object::nil();

        let lua_t = lua_type(lstate, -1);
        'type_dispatch: {
            match lua_t {
                t if t == LUA_TNIL => {
                    // leave as nil
                }
                t if t == LUA_TBOOLEAN => {
                    *cur.obj = Object {
                        obj_type: K_OBJECT_TYPE_BOOLEAN,
                        data: ObjectData {
                            boolean: lua_toboolean(lstate, -1) != 0,
                        },
                    };
                }
                t if t == LUA_TSTRING => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(lstate, -1, &raw mut len);
                    let data = arena_memdupz(arena, s, len);
                    *cur.obj = Object {
                        obj_type: K_OBJECT_TYPE_STRING,
                        data: ObjectData {
                            string: NvimString { data, size: len },
                        },
                    };
                }
                t if t == LUA_TNUMBER => {
                    let n = lua_tonumber(lstate, -1);
                    // Round-trip check: if it converts exactly to integer, use integer
                    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                    let as_int = n as Integer;
                    // float_cmp: intentional exact round-trip check to detect integer-valued floats
                    #[allow(clippy::cast_precision_loss, clippy::float_cmp)]
                    if n > (i64::MAX as f64) || n < (i64::MIN as f64) || (as_int as f64) != n {
                        *cur.obj = Object {
                            obj_type: K_OBJECT_TYPE_FLOAT,
                            data: ObjectData { floating: n },
                        };
                    } else {
                        *cur.obj = Object {
                            obj_type: K_OBJECT_TYPE_INTEGER,
                            data: ObjectData { integer: as_int },
                        };
                    }
                }
                t if t == LUA_TTABLE => {
                    let props = traverse_table(lstate);
                    match props.obj_type {
                        k if k == K_OBJECT_TYPE_ARRAY => {
                            *cur.obj = Object {
                                obj_type: K_OBJECT_TYPE_ARRAY,
                                data: ObjectData {
                                    array: Array {
                                        size: 0,
                                        capacity: 0,
                                        items: std::ptr::null_mut(),
                                    },
                                },
                            };
                            if props.maxidx != 0 {
                                (*cur.obj).data.array = arena_array(arena, props.maxidx);
                                cur.container = true;
                                stack.push(cur);
                            }
                        }
                        k if k == K_OBJECT_TYPE_DICT => {
                            *cur.obj = Object {
                                obj_type: K_OBJECT_TYPE_DICT,
                                data: ObjectData {
                                    dict: Dict {
                                        size: 0,
                                        capacity: 0,
                                        items: std::ptr::null_mut(),
                                    },
                                },
                            };
                            if props.string_keys_num != 0 {
                                (*cur.obj).data.dict = arena_dict(arena, props.string_keys_num);
                                cur.container = true;
                                stack.push(cur);
                                lua_pushnil(lstate);
                            }
                        }
                        k if k == K_OBJECT_TYPE_FLOAT => {
                            *cur.obj = Object {
                                obj_type: K_OBJECT_TYPE_FLOAT,
                                data: ObjectData {
                                    floating: props.val,
                                },
                            };
                        }
                        _ => {
                            api_set_error(
                                err,
                                K_ERROR_VALIDATION,
                                c"Cannot convert given Lua table".as_ptr(),
                            );
                        }
                    }
                }
                t if t == LUA_TFUNCTION => {
                    if ref_ {
                        let luaref = crate::refs::rs_nlua_ref_global(lstate, -1);
                        *cur.obj = Object {
                            obj_type: K_OBJECT_TYPE_LUAREF,
                            data: ObjectData {
                                luaref: i64::from(luaref),
                            },
                        };
                    } else {
                        // goto type_error equivalent
                        api_set_error(
                            err,
                            K_ERROR_VALIDATION,
                            c"Cannot convert given Lua type".as_ptr(),
                        );
                        break 'type_dispatch;
                    }
                }
                t if t == LUA_TUSERDATA => {
                    let nil_ref = crate::leaf::rs_nlua_get_nil_ref(lstate);
                    crate::refs::rs_nlua_pushref(lstate, nil_ref);
                    let is_nil = lua_rawequal(lstate, -2, -1) != 0;
                    lua_pop(lstate, 1);
                    if is_nil {
                        *cur.obj = Object::nil();
                    } else {
                        api_set_error(err, K_ERROR_VALIDATION, c"Cannot convert userdata".as_ptr());
                    }
                }
                _ => {
                    api_set_error(
                        err,
                        K_ERROR_VALIDATION,
                        c"Cannot convert given Lua type".as_ptr(),
                    );
                }
            }
        } // end 'type_dispatch

        if !cur.container {
            lua_pop(lstate, 1);
        }
    }

    if (*err).is_set() {
        if arena.is_null() {
            api_free_object(ret);
        }
        ret = Object::nil();
        let excess = lua_gettop(lstate) - initial_size + 1;
        if excess > 0 {
            lua_pop(lstate, excess);
        }
    }

    ret
}

/// Forwarding shim for Rust callers that used the old rs_ prefix.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_object(
    lstate: *mut LuaState,
    ref_: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    rs_nlua_pop_Object(lstate, ref_, arena, err)
}

/// Pop a typval from the Lua stack. (Phase 10 will replace with real impl)
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_typval(
    lstate: *mut LuaState,
    ret_tv: *mut std::ffi::c_void,
) -> bool {
    nlua_pop_typval(lstate, ret_tv)
}

// =============================================================================
// Direct Lua stack reading operations (kept for Rust consumers)
// =============================================================================

/// Get a number from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64 {
    lua_tonumber(lstate, idx)
}

/// Get a boolean from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_toboolean(lstate: *mut LuaState, idx: c_int) -> bool {
    lua_toboolean(lstate, idx) != 0
}

/// Get a string from the Lua stack without popping.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_tolstring(
    lstate: *mut LuaState,
    idx: c_int,
    len: *mut usize,
) -> *const c_char {
    lua_tolstring(lstate, idx, len)
}

/// Get the type of a Lua value on the stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_type(lstate: *mut LuaState, idx: c_int) -> c_int {
    lua_type(lstate, idx)
}

/// Pop values from the Lua stack.
#[no_mangle]
pub unsafe extern "C" fn rs_lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_pop(lstate, n);
}

/// Get the top of the Lua stack (number of elements).
#[no_mangle]
pub unsafe extern "C" fn rs_lua_gettop(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

// =============================================================================
// Safe Rust wrappers
// =============================================================================

/// Get the type of a Lua value on the stack.
#[inline]
#[must_use]
pub unsafe fn get_type(lstate: *mut LuaState, idx: c_int) -> crate::types::LuaType {
    crate::types::LuaType::from_raw(lua_type(lstate, idx)).unwrap_or(crate::types::LuaType::None)
}

/// Get the stack height.
#[inline]
#[must_use]
pub unsafe fn stack_height(lstate: *mut LuaState) -> c_int {
    lua_gettop(lstate)
}

/// Pop n values from the stack.
#[inline]
pub unsafe fn pop(lstate: *mut LuaState, n: c_int) {
    lua_pop(lstate, n);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_values() {
        assert_eq!(ErrorType::None as i32, -1);
        assert_eq!(ErrorType::Exception as i32, 0);
        assert_eq!(ErrorType::Validation as i32, 1);
    }

    #[test]
    fn test_error_is_set() {
        let error_none = Error {
            r#type: ErrorType::None,
            msg: std::ptr::null_mut(),
        };
        assert!(!error_none.is_set());

        let error_set = Error {
            r#type: ErrorType::Exception,
            msg: std::ptr::null_mut(),
        };
        assert!(error_set.is_set());
    }

    #[test]
    fn test_arena_is_opaque() {
        assert_eq!(std::mem::size_of::<Arena>(), 0);
    }

    #[test]
    fn test_integer_bounds() {
        assert_eq!(API_INTEGER_MIN, i64::MIN);
    }
}
