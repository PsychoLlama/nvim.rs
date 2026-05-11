//! API conversion helpers
//!
//! This module provides FFI helpers for API-specific type conversions
//! between Lua and Neovim's API types.

use std::ffi::{c_char, c_int, c_void};

use crate::state::LuaState;
use nvim_api::{Arena, Error, LuaRef, NvimString, Object};

/// Handle type (buffer, window, tabpage)
pub type HandleT = c_int;

/// Integer type (i64)
type Integer = i64;
/// Float type (f64)
type Float = f64;
/// Boolean type
type Boolean = bool;
/// LuaRef is typedef int in C
type LocalLuaRef = c_int;

/// Field hash function type for keydict.
///
/// Matches C's `FieldHashfn`: `KeySetLink *(*)(const char *str, size_t len)`.
pub type FieldHashfn = unsafe extern "C" fn(*const c_char, usize) -> *mut KeySetLink;

// kObjectType* constants (matching C enum values)
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

// kErrorTypeValidation = 1
const K_ERROR_VALIDATION: c_int = 1;

/// Mirrors C's `KeySetLink` in `nvim/api/private/defs.h`.
///
/// Field offsets are ABI-locked (part of generated keysets); see layout
/// assertions in the `tests` module below.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KeySetLink {
    /// Null-terminated key string.
    pub str_: *mut c_char,
    /// Byte offset of this field within the keyset struct.
    pub ptr_off: usize,
    /// Field type: one of the kObjectType* constants, or UnpackType.
    /// `kObjectTypeNil` means untyped (accept any Object).
    pub type_: c_int,
    /// Index into OptionalKeys bitmask, or -1 if always present.
    pub opt_index: c_int,
    /// If true, the Integer field accepts a highlight group name string.
    pub is_hlgroup: bool,
}

/// Mirrors C's `OptKeySet` in `nvim/api/private/defs.h`.
///
/// The prefix struct for all keysets with optional keys.
#[repr(C)]
pub struct OptKeySet {
    pub is_set_: u64,
}

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Typval conversion
    fn nlua_push_typval(lstate: *mut LuaState, tv: *const c_void, flags: c_int) -> bool;

    // Lua C API needed for keydict operations
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_pushnil(lstate: *mut LuaState);
    fn lua_next(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_createtable(lstate: *mut LuaState, narr: c_int, nrec: c_int);
    fn lua_pushstring(lstate: *mut LuaState, s: *const c_char) -> *const c_char;
    fn lua_pushnumber(lstate: *mut LuaState, n: f64);
    fn lua_pushboolean(lstate: *mut LuaState, b: c_int);
    fn lua_rawset(lstate: *mut LuaState, idx: c_int);

    // Highlight group name → id
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;
}

// api_set_error is also declared in from_lua.rs with a different local Error type;
// the C ABI is identical. Allow the clash to avoid having two separate Error newtypes.
#[allow(clashing_extern_declarations)]
extern "C" {
    fn api_set_error(err: *mut Error, err_type: c_int, fmt: *const c_char, ...);
}

// =============================================================================
// Rust FFI exports
// =============================================================================

/// Push a typval (VimL value) onto the Lua stack.
///
/// Converts a typval_T to its Lua equivalent.
///
/// # Arguments
/// * `tv` - Pointer to a typval_T
/// * `flags` - Conversion flags (kNluaPushSpecial, kNluaPushFreeRefs)
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `tv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_push_typval(
    lstate: *mut LuaState,
    tv: *const c_void,
    flags: c_int,
) -> bool {
    nlua_push_typval(lstate, tv, flags)
}

/// Pop a LuaRef from the Lua stack.
///
/// Creates a reference to the Lua value at the top of the stack.
/// Always pops one value from the stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` can be NULL or a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_luaref(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> LuaRef {
    // Delegate to from_lua real implementation (Phase 2).
    // from_lua returns c_int (the actual C LuaRef type); nvim_api::LuaRef is i64
    // but that is a pre-existing widening; the value fits.
    #[allow(clippy::cast_lossless)]
    let rv = crate::from_lua::rs_nlua_pop_LuaRef(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    ) as LuaRef;
    rv
}

/// Pop a handle (Buffer/Window/Tabpage) from the Lua stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `err` must be a valid Error pointer.
/// - `arena` must be a valid Arena pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_handle(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    // Delegate to from_lua real implementation (Phase 2)
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Initialize Lua type tables.
///
/// Sets up the special type markers used for typed tables.
///
/// # Safety
///
/// `lstate` must be a valid Lua state pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_init_types(lstate: *mut LuaState) {
    crate::to_lua::rs_nlua_init_types(lstate);
}

// lua_pop is a C macro: #define lua_pop(L,n) lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

// lua_istable is a C macro: lua_type(L, n) == LUA_TTABLE (5)
const LUA_TTABLE: c_int = 5;
// lua_type(L, n) == LUA_TSTRING (4)
const LUA_TSTRING: c_int = 4;

/// Pop a keydict (structured options table) from the Lua stack.
///
/// Parses a Lua table into a keyset struct using the provided hash function.
/// On success, the table is consumed from the stack. On error, `err` is set
/// and `*err_opt` is pointed at the offending field name (if any).
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `retval` must be a valid pointer to the keyset structure.
/// - `hashy` must be a valid field hash function for this keyset type.
/// - `err_opt` can be NULL or a valid pointer-to-pointer.
/// - `arena` must be a valid Arena pointer (or NULL for heap allocation).
/// - `err` must be a valid Error pointer.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_ptr_alignment)] // C keyset structs are always aligned correctly
#[allow(clippy::cast_possible_truncation)] // len (usize) → c_int: safe for key lengths
#[unsafe(export_name = "nlua_pop_keydict")]
pub unsafe extern "C" fn rs_nlua_pop_keydict(
    lstate: *mut LuaState,
    retval: *mut c_void,
    hashy: FieldHashfn,
    err_opt: *mut *mut c_char,
    arena: *mut Arena,
    err: *mut Error,
) {
    // Validate that the top of stack is a table.
    if lua_type(lstate, -1) != LUA_TTABLE {
        api_set_error(err, K_ERROR_VALIDATION, c"Expected Lua table".as_ptr());
        // NOTE: The original C had `lua_pop(L, -1)` here which is a no-op
        // (lua_settop(L, 0) clears the whole stack). We preserve the C bug
        // for ABI compatibility: the table is NOT popped on this error path.
        lua_pop(lstate, -1);
        return;
    }

    lua_pushnil(lstate); // [dict, nil]
    while lua_next(lstate, -2) != 0 {
        // Stack: [dict, key, value]
        let mut len: usize = 0;
        let s = lua_tolstring(lstate, -2, &raw mut len);
        let field = hashy(s, len);
        if field.is_null() {
            api_set_error(
                err,
                K_ERROR_VALIDATION,
                c"invalid key: %.*s".as_ptr(),
                len as c_int, // len is a key name length, truncation is safe
                s,
            );
            lua_pop(lstate, 3); // []
            return;
        }

        let field = &*field;

        if field.opt_index >= 0 {
            let ks = retval.cast::<OptKeySet>();
            (*ks).is_set_ |= 1u64 << field.opt_index;
        }
        let mem = retval.cast::<u8>().add(field.ptr_off);

        // Dispatch based on field type.
        if field.type_ == K_OBJECT_TYPE_NIL {
            // Untyped: accept any Object.
            *mem.cast::<Object>() = crate::from_lua::rs_nlua_pop_Object(
                lstate,
                true,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_INTEGER {
            if field.is_hlgroup && lua_type(lstate, -1) == LUA_TSTRING {
                let mut name_len: usize = 0;
                let name = lua_tolstring(lstate, -1, &raw mut name_len);
                lua_pop(lstate, 1);
                *mem.cast::<Integer>() = if name_len > 0 {
                    Integer::from(syn_check_group(name, name_len))
                } else {
                    0
                };
            } else {
                *mem.cast::<Integer>() = crate::from_lua::rs_nlua_pop_Integer(
                    lstate,
                    arena.cast::<crate::from_lua::Arena>(),
                    err.cast::<crate::from_lua::Error>(),
                );
            }
        } else if field.type_ == K_OBJECT_TYPE_BOOLEAN {
            *mem.cast::<Boolean>() = crate::from_lua::rs_nlua_pop_Boolean_strict(
                lstate,
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_STRING {
            *mem.cast::<NvimString>() = crate::from_lua::rs_nlua_pop_String(
                lstate,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_FLOAT {
            *mem.cast::<Float>() = crate::from_lua::rs_nlua_pop_Float(
                lstate,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_BUFFER
            || field.type_ == K_OBJECT_TYPE_WINDOW
            || field.type_ == K_OBJECT_TYPE_TABPAGE
        {
            *mem.cast::<HandleT>() = crate::from_lua::rs_nlua_pop_handle(
                lstate,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_ARRAY {
            *mem.cast::<nvim_api::Array>() = crate::from_lua::rs_nlua_pop_Array(
                lstate,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_DICT {
            *mem.cast::<nvim_api::Dict>() = crate::from_lua::rs_nlua_pop_Dict(
                lstate,
                false,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else if field.type_ == K_OBJECT_TYPE_LUAREF {
            *mem.cast::<LocalLuaRef>() = crate::from_lua::rs_nlua_pop_LuaRef(
                lstate,
                arena.cast::<crate::from_lua::Arena>(),
                err.cast::<crate::from_lua::Error>(),
            );
        } else {
            // Unknown type: should never happen with well-formed generated keysets.
            std::process::abort();
        }

        // ERROR_SET: err->type != kErrorTypeNone (-1)
        if (*err).err_type != -1 {
            if !err_opt.is_null() {
                *err_opt = field.str_;
            }
            break;
        }
    }
    // [dict] → pop the dict
    lua_pop(lstate, 1);
    // []
}

/// Push a keyset struct as a Lua table onto the stack.
///
/// Converts a C keyset struct to a Lua table with string keys.
/// Only fields that are set (according to the OptKeySet bitmask) are pushed.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `value` must be a valid pointer to the keyset structure.
/// - `table` must be a valid KeySetLink array, terminated by an entry with
///   a null `str_` pointer.
#[allow(clippy::cast_ptr_alignment)] // C keyset structs are always aligned correctly
#[allow(clippy::cast_precision_loss)] // Integer → f64: intentional (matches C behavior)
#[unsafe(export_name = "nlua_push_keydict")]
pub unsafe extern "C" fn rs_nlua_push_keydict(
    lstate: *mut LuaState,
    value: *mut c_void,
    table: *mut KeySetLink,
) {
    lua_createtable(lstate, 0, 0);
    let mut i: usize = 0;
    loop {
        let field = &*table.add(i);
        if field.str_.is_null() {
            break;
        }

        let is_set = if field.opt_index >= 0 {
            let ks = value.cast::<OptKeySet>();
            ((*ks).is_set_ & (1u64 << field.opt_index)) != 0
        } else {
            true
        };

        if !is_set {
            i += 1;
            continue;
        }

        let mem = value.cast::<u8>().add(field.ptr_off);

        lua_pushstring(lstate, field.str_);

        if field.type_ == K_OBJECT_TYPE_NIL {
            crate::to_lua::rs_nlua_push_Object(lstate, mem.cast::<Object>(), 0);
        } else if field.type_ == K_OBJECT_TYPE_INTEGER {
            #[allow(clippy::cast_precision_loss)]
            lua_pushnumber(lstate, *mem.cast::<Integer>() as f64);
        } else if field.type_ == K_OBJECT_TYPE_BUFFER
            || field.type_ == K_OBJECT_TYPE_WINDOW
            || field.type_ == K_OBJECT_TYPE_TABPAGE
        {
            lua_pushnumber(lstate, f64::from(*mem.cast::<HandleT>()));
        } else if field.type_ == K_OBJECT_TYPE_FLOAT {
            lua_pushnumber(lstate, *mem.cast::<Float>());
        } else if field.type_ == K_OBJECT_TYPE_BOOLEAN {
            lua_pushboolean(lstate, c_int::from(*mem.cast::<Boolean>()));
        } else if field.type_ == K_OBJECT_TYPE_STRING {
            crate::to_lua::rs_nlua_push_String(lstate, *mem.cast::<NvimString>(), 0);
        } else if field.type_ == K_OBJECT_TYPE_ARRAY {
            crate::to_lua::rs_nlua_push_Array(lstate, *mem.cast::<nvim_api::Array>(), 0);
        } else if field.type_ == K_OBJECT_TYPE_DICT {
            crate::to_lua::rs_nlua_push_Dict(lstate, *mem.cast::<nvim_api::Dict>(), 0);
        } else if field.type_ == K_OBJECT_TYPE_LUAREF {
            let ref_ = *mem.cast::<LocalLuaRef>();
            crate::refs::rs_nlua_pushref(lstate, ref_);
        } else {
            // Unknown type: should never happen with well-formed generated keysets.
            std::process::abort();
        }

        lua_rawset(lstate, -3);
        i += 1;
    }
}

// =============================================================================
// Buffer/Window/Tabpage type aliases
// =============================================================================

/// Pop a Buffer handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_buffer(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Pop a Window handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_window(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

/// Pop a Tabpage handle from the Lua stack.
///
/// Alias for `rs_nlua_pop_handle`.
///
/// # Safety
///
/// Same as `rs_nlua_pop_handle`.
#[no_mangle]
pub unsafe extern "C" fn rs_nlua_pop_tabpage(
    lstate: *mut LuaState,
    arena: *mut Arena,
    err: *mut Error,
) -> HandleT {
    crate::from_lua::rs_nlua_pop_handle(
        lstate,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    /// Verify KeySetLink field layout matches the C definition in defs.h.
    ///
    /// C layout (x86-64):
    ///   char *str;        offset 0
    ///   size_t ptr_off;   offset 8
    ///   int type;         offset 16
    ///   int opt_index;    offset 20
    ///   bool is_hlgroup;  offset 24
    ///   (padding to 32)
    #[test]
    fn test_keysetlink_layout() {
        assert_eq!(offset_of!(KeySetLink, str_), 0);
        assert_eq!(offset_of!(KeySetLink, ptr_off), 8);
        assert_eq!(offset_of!(KeySetLink, type_), 16);
        assert_eq!(offset_of!(KeySetLink, opt_index), 20);
        assert_eq!(offset_of!(KeySetLink, is_hlgroup), 24);
    }

    #[test]
    fn test_optkeyset_layout() {
        assert_eq!(std::mem::size_of::<OptKeySet>(), 8);
        assert_eq!(offset_of!(OptKeySet, is_set_), 0);
    }

    #[test]
    fn test_handle_type() {
        // HandleT should be c_int
        assert_eq!(std::mem::size_of::<HandleT>(), std::mem::size_of::<c_int>());
    }
}
