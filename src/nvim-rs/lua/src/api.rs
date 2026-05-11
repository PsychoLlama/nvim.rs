//! API conversion helpers
//!
//! This module provides FFI helpers for API-specific type conversions
//! between Lua and Neovim's API types.

use std::ffi::{c_char, c_int, c_void};

use crate::state::LuaState;
use nvim_api::{Arena, Error, LuaRef, NvimString, Object};

// VarType constants (matching C's VarType enum)
const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
// VAR_STRING = 2 (not needed here)
// VAR_FUNC = 3
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_UNLOCKED: c_int = 0;

// SpecialVarValue / BoolVarValue constants
const K_SPECIAL_VAR_NULL: c_int = 0;
const K_BOOL_VAR_TRUE: c_int = 1;
const K_BOOL_VAR_FALSE: c_int = 0;

// VARNUMBER limits
const VARNUMBER_MAX: f64 = 9_223_372_036_854_775_807.0_f64; // INT64_MAX
const VARNUMBER_MIN: f64 = -9_223_372_036_854_775_808.0_f64; // INT64_MIN

// LUA_NOREF = -2
const LUA_NOREF: c_int = -2;

// TypevalHandle = opaque pointer (same as *mut c_void in typval crate)
type TypevalHandle = *mut c_void;
// ListHandle = opaque pointer
type ListHandle = *mut c_void;
// DictHandle = opaque pointer
type DictHandle = *mut c_void;
// DictItemHandle = opaque pointer
type DictItemHandle = *mut c_void;

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
    // Lua C API needed for keydict and push_typval operations
    fn lua_gettop(lstate: *mut LuaState) -> c_int;
    fn lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int;
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

    // Additional Lua C API functions needed for Phase 10
    fn lua_tonumber(lstate: *mut LuaState, idx: c_int) -> f64;
    fn lua_toboolean(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_rawequal(lstate: *mut LuaState, idx1: c_int, idx2: c_int) -> c_int;
    fn lua_rawgeti(lstate: *mut LuaState, idx: c_int, n: c_int);
    fn lua_getmetatable(lstate: *mut LuaState, idx: c_int) -> c_int;

    // Highlight group name → id
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;

    // Message output
    fn semsg(fmt: *const c_char, ...);

    // Lua encoder (provided by nvim-eval-codec Rust crate)
    fn encode_vim_to_lua(
        lstate: *mut LuaState,
        tv: *const c_void,
        objname: *const c_char,
        allow_special: bool,
    ) -> c_int;
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

// kNluaPushSpecial flag: use typed tables for nil/special values.
const K_NLUA_PUSH_SPECIAL: c_int = 0x01;
// FAIL constant: matches encode.c's FAIL = 0.
const ENCODE_FAIL: c_int = 0;

/// Push a typval (VimL value) onto the Lua stack.
///
/// Converts a typval_T to its Lua equivalent. Should leave exactly one
/// value on the Lua stack. May only fail if Lua failed to grow its stack.
///
/// # Safety
///
/// - `lstate` must be a valid Lua state pointer.
/// - `tv` must be a valid typval_T pointer.
#[unsafe(export_name = "nlua_push_typval")]
pub unsafe extern "C" fn rs_nlua_push_typval(
    lstate: *mut LuaState,
    tv: *const c_void,
    flags: c_int,
) -> bool {
    let allow_special = (flags & K_NLUA_PUSH_SPECIAL) != 0;
    let initial_size = lua_gettop(lstate);
    if lua_checkstack(lstate, initial_size + 2) == 0 {
        semsg(
            c"E1502: Lua failed to grow stack to %i".as_ptr(),
            initial_size + 4,
        );
        return false;
    }
    if encode_vim_to_lua(
        lstate,
        tv,
        c"nlua_push_typval argument".as_ptr(),
        allow_special,
    ) == ENCODE_FAIL
    {
        return false;
    }
    debug_assert_eq!(lua_gettop(lstate), initial_size + 1);
    true
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

// =============================================================================
// Phase 10: nlua_pop_typval
// =============================================================================

extern "C" {
    // typval write/read accessors for Phase 10
    fn nvim_tv_set_type(tv: TypevalHandle, v_type: c_int);
    fn nvim_tv_set_lock(tv: TypevalHandle, v_lock: c_int);
    fn nvim_tv_set_number(tv: TypevalHandle, n: i64);
    fn nvim_tv_set_float(tv: TypevalHandle, f: f64);
    fn nvim_tv_set_bool(tv: TypevalHandle, val: c_int);
    fn nvim_tv_set_special(tv: TypevalHandle, val: c_int);
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);
    fn nvim_tv_set_list(tv: TypevalHandle, l: ListHandle);
    fn nvim_tv_set_dict(tv: TypevalHandle, d: DictHandle);
    fn nvim_tv_get_list(tv: TypevalHandle) -> ListHandle;
    fn nvim_tv_get_dict(tv: TypevalHandle) -> DictHandle;
    fn tv_clear(tv: TypevalHandle);
    fn tv_copy(from: TypevalHandle, to: TypevalHandle);

    // list accessors
    fn tv_list_alloc(len: isize) -> ListHandle;
    #[link_name = "nvim_list_ref"]
    fn tv_list_ref(l: ListHandle);
    #[link_name = "nvim_list_get_len"]
    fn tv_list_len(l: ListHandle) -> c_int;
    fn nvim_tv_list_append_unknown_and_get(l: ListHandle) -> TypevalHandle;
    fn tv_list_append_list(l: ListHandle, itemlist: ListHandle);
    fn nvim_list_set_lua_table_ref(l: ListHandle, ref_: c_int);

    // dict accessors
    fn tv_dict_alloc() -> DictHandle;
    fn tv_dict_item_alloc_len(key: *const c_char, len: usize) -> DictItemHandle;
    fn tv_dict_add(d: DictHandle, item: DictItemHandle) -> c_int;
    fn tv_dict_find(d: DictHandle, key: *const c_char, keylen: c_int) -> DictItemHandle;
    fn nvim_dictitem_di_tv(di: DictItemHandle) -> TypevalHandle;
    fn nvim_dict_set_lua_table_ref(d: DictHandle, ref_: c_int);
    fn nvim_dict_inc_refcount(d: DictHandle);

    // Lua function registration (from userfunc crate)
    fn register_luafunc(func: c_int) -> *mut c_char;

    // Vimscript string helpers
    fn xstrdup(s: *const c_char) -> *mut c_char;
    // message functions
    fn emsg(fmt: *const c_char) -> c_int;

    // decode_string via the Rust pointer-out form.
    // Parameters: (s, len, force_blob, s_allocated, rettv)
    // force_blob=true → create blob even without NUL.
    // s_allocated=false → copy the string data (don't take ownership).
    fn rs_decode_string_into(
        s: *const c_char,
        len: usize,
        force_blob: bool,
        s_allocated: bool,
        rettv: TypevalHandle,
    );

    // decode_create_map_special_dict: creates {_TYPE: map, _VAL: list} and returns the _VAL list.
    fn decode_create_map_special_dict(ret_tv: TypevalHandle, len: isize) -> ListHandle;
}

// Lua type constants (duplicated locally to avoid cross-module confusion)
const LUA_TNIL_PH10: c_int = 0;
const LUA_TBOOLEAN_PH10: c_int = 1;
const LUA_TNUMBER_PH10: c_int = 3;
const LUA_TSTRING_PH10: c_int = 4;
const LUA_TTABLE_PH10: c_int = 5;
const LUA_TFUNCTION_PH10: c_int = 6;
const LUA_TUSERDATA_PH10: c_int = 7;

/// Helper: lua_pop for Phase 10 (avoid name collision with the api.rs lua_pop defined above)
#[inline]
unsafe fn lua_pop_ph10(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

/// Internal: get the inline LuaRef from leaf.rs (nil ref)
#[inline]
unsafe fn get_nil_ref(lstate: *mut LuaState) -> c_int {
    crate::leaf::rs_nlua_get_nil_ref(lstate)
}

/// Helper item for the iterative stack in nlua_pop_typval.
///
/// Mirrors the C `TVPopStackItem` struct. The `is_dict` field replaces reading
/// `cur.tv->v_type == VAR_DICT` at iteration time (since we know at push time).
#[derive(Copy, Clone)]
struct TVPopStackItem {
    /// Pointer to the typval slot being filled.
    tv: TypevalHandle,
    /// Maximum length when tv is a list (only valid when container=true, is_dict=false).
    list_len: usize,
    /// True if tv is a container (list or dict) that needs further iteration.
    container: bool,
    /// True if tv is the _VAL part of a special dict (NUL-key mapping).
    special: bool,
    /// True if the container is a dict (vs a list). Used instead of reading tv->v_type.
    is_dict: bool,
    /// Lua stack index of this container table (for self-reference detection).
    idx: c_int,
}

/// Convert a Lua value to a Vimscript typval_T.
///
/// Pops exactly one value from the Lua stack. Returns `true` on success,
/// `false` on error (error is reported and `*ret_tv` is cleared to VAR_NUMBER(0)).
///
/// Mirrors C's `nlua_pop_typval` in `src/nvim/lua/converter.c`.
///
/// # Safety
/// - `lstate` must be a valid Lua state.
/// - `ret_tv` must be a valid pointer to a typval_T.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "nlua_pop_typval")]
pub unsafe extern "C" fn rs_nlua_pop_typval(lstate: *mut LuaState, ret_tv: TypevalHandle) -> bool {
    let mut ret = true;
    let initial_size = lua_gettop(lstate);

    // Pre-initialize ret_tv so tv_clear is safe on error.
    nvim_tv_set_type(ret_tv, VAR_NUMBER);
    nvim_tv_set_lock(ret_tv, VAR_UNLOCKED);
    nvim_tv_set_number(ret_tv, 0);

    let mut stack: Vec<TVPopStackItem> = Vec::with_capacity(2);
    stack.push(TVPopStackItem {
        tv: ret_tv,
        list_len: 0,
        container: false,
        special: false,
        is_dict: false,
        idx: 0,
    });

    while ret && !stack.is_empty() {
        if lua_checkstack(lstate, lua_gettop(lstate) + 3) == 0 {
            semsg(
                c"E1502: Lua failed to grow stack to %i".as_ptr(),
                lua_gettop(lstate) + 3,
            );
            ret = false;
            break;
        }

        let mut cur = stack.pop().unwrap();

        if cur.container {
            // Container iteration: cur.special means iterating string keys into a list (_VAL),
            // cur.is_dict means iterating string keys into a VAR_DICT.
            if cur.special || cur.is_dict {
                // Dict-like iteration: find the next string key via lua_next.
                // Stack: [table, key] (key = nil on first call, previous key otherwise)
                let mut next_key_found = false;
                while lua_next(lstate, -2) != 0 {
                    // Stack: [table, new_key, value]
                    if lua_type(lstate, -2) == LUA_TSTRING_PH10 {
                        next_key_found = true;
                        break;
                    }
                    lua_pop_ph10(lstate, 1); // pop value, keep key for lua_next
                }
                if next_key_found {
                    // Stack: [table, key, value]
                    let mut len: usize = 0;
                    let s = lua_tolstring(lstate, -2, &raw mut len);
                    if cur.special {
                        // Append key/value pair to the special _VAL list.
                        // cur.tv is a VAR_LIST typval pointing to the _VAL list.
                        let outer_val_list = nvim_tv_get_list(cur.tv);
                        let kv_pair_list = tv_list_alloc(2);
                        tv_list_ref(kv_pair_list);

                        // Append key as blob typval (force_blob=true because key may have NUL).
                        let key_tv = nvim_tv_list_append_unknown_and_get(kv_pair_list);
                        rs_decode_string_into(s, len, true, false, key_tv);

                        // Append placeholder for value.
                        let val_tv = nvim_tv_list_append_unknown_and_get(kv_pair_list);
                        nvim_tv_set_type(val_tv, VAR_UNKNOWN);

                        // Append kv_pair_list to the _VAL outer list.
                        tv_list_append_list(outer_val_list, kv_pair_list);

                        // Re-push cur to continue iterating string keys.
                        stack.push(cur);

                        // The value slot is val_tv; push a new item for it.
                        cur = TVPopStackItem {
                            tv: val_tv,
                            list_len: 0,
                            container: false,
                            special: false,
                            is_dict: false,
                            idx: 0,
                        };
                    } else {
                        // Normal dict: allocate a new dict item and push.
                        let di = tv_dict_item_alloc_len(s, len);
                        let dict = nvim_tv_get_dict(cur.tv);
                        let add_rc = tv_dict_add(dict, di);
                        if add_rc == 0 {
                            // tv_dict_add returned FAIL — key already exists or internal error.
                            // Treat as conversion error so callers see a clean failure.
                            ret = false;
                            break;
                        }
                        stack.push(cur);
                        let di_tv = nvim_dictitem_di_tv(di);
                        cur = TVPopStackItem {
                            tv: di_tv,
                            list_len: 0,
                            container: false,
                            special: false,
                            is_dict: false,
                            idx: 0,
                        };
                    }
                } else {
                    // No more string keys: done with this container.
                    // Stack: [table] (lua_next consumed the nil, or we exhausted all keys).
                    lua_pop_ph10(lstate, 1); // pop table
                    continue;
                }
            } else {
                // List iteration: fetch the next element by index.
                // cur.tv is a VAR_LIST typval.
                let list = nvim_tv_get_list(cur.tv);
                #[allow(clippy::cast_sign_loss)]
                let cur_len = tv_list_len(list) as usize;
                if cur_len == cur.list_len {
                    // All elements filled.
                    lua_pop_ph10(lstate, 1); // pop table
                    continue;
                }
                #[allow(clippy::cast_possible_truncation)]
                lua_rawgeti(lstate, -1, cur_len as c_int + 1);
                // Append a placeholder and get its tv pointer.
                let item_tv = nvim_tv_list_append_unknown_and_get(list);
                nvim_tv_set_type(item_tv, VAR_UNKNOWN);

                stack.push(cur);
                cur = TVPopStackItem {
                    tv: item_tv,
                    list_len: 0,
                    container: false,
                    special: false,
                    is_dict: false,
                    idx: 0,
                };
            }
        }

        // ----------------------------------------------------------------
        // Leaf: fill cur.tv from the Lua value on top of the stack.
        // ----------------------------------------------------------------
        debug_assert!(!cur.container);

        // Pre-initialize cur.tv
        nvim_tv_set_type(cur.tv, VAR_NUMBER);
        nvim_tv_set_lock(cur.tv, VAR_UNLOCKED);
        nvim_tv_set_number(cur.tv, 0);

        // Use a labeled block to model the C `goto nlua_pop_typval_table_processing_end`.
        'leaf: {
            match lua_type(lstate, -1) {
                t if t == LUA_TNIL_PH10 => {
                    nvim_tv_set_type(cur.tv, VAR_SPECIAL);
                    nvim_tv_set_special(cur.tv, K_SPECIAL_VAR_NULL);
                }
                t if t == LUA_TBOOLEAN_PH10 => {
                    nvim_tv_set_type(cur.tv, VAR_BOOL);
                    let bval = if lua_toboolean(lstate, -1) != 0 {
                        K_BOOL_VAR_TRUE
                    } else {
                        K_BOOL_VAR_FALSE
                    };
                    nvim_tv_set_bool(cur.tv, bval);
                }
                t if t == LUA_TSTRING_PH10 => {
                    let mut len: usize = 0;
                    let s = lua_tolstring(lstate, -1, &raw mut len);
                    rs_decode_string_into(s, len, false, false, cur.tv);
                }
                t if t == LUA_TNUMBER_PH10 => {
                    let n = lua_tonumber(lstate, -1);
                    if !(VARNUMBER_MIN..=VARNUMBER_MAX).contains(&n) || n.fract() != 0.0 {
                        nvim_tv_set_type(cur.tv, VAR_FLOAT);
                        nvim_tv_set_float(cur.tv, n);
                    } else {
                        nvim_tv_set_type(cur.tv, VAR_NUMBER);
                        #[allow(clippy::cast_possible_truncation)]
                        nvim_tv_set_number(cur.tv, n as i64);
                    }
                }
                t if t == LUA_TTABLE_PH10 => {
                    // Grab a lua_table_ref if there is a metatable.
                    let table_ref: c_int = if lua_getmetatable(lstate, -1) != 0 {
                        lua_pop_ph10(lstate, 1); // pop the metatable
                                                 // rs_nlua_ref_global returns LuaRef (i64); lua_table_ref is c_int.
                        #[allow(clippy::cast_possible_truncation)]
                        let r = crate::refs::rs_nlua_ref_global(lstate, -1) as c_int;
                        r
                    } else {
                        LUA_NOREF
                    };

                    let props: crate::from_lua::LuaTableProps =
                        crate::from_lua::traverse_table_pub(lstate);

                    // Self-reference detection: check if any container on our stack
                    // has the same Lua table (same stack index value, compared with lua_rawequal).
                    for item in &stack {
                        if item.container && lua_rawequal(lstate, -1, item.idx) != 0 {
                            // Re-use the already-allocated typval.
                            tv_copy(item.tv, cur.tv);
                            cur.container = false;
                            break 'leaf; // goto nlua_pop_typval_table_processing_end
                        }
                    }

                    match props.obj_type {
                        k if k == K_OBJECT_TYPE_ARRAY => {
                            let list = tv_list_alloc(props.maxidx as isize);
                            tv_list_ref(list);
                            nvim_list_set_lua_table_ref(list, table_ref);
                            nvim_tv_set_type(cur.tv, VAR_LIST);
                            nvim_tv_set_list(cur.tv, list);
                            cur.list_len = props.maxidx;
                            if props.maxidx != 0 {
                                cur.container = true;
                                cur.is_dict = false;
                                cur.idx = lua_gettop(lstate);
                                stack.push(cur);
                            }
                        }
                        k if k == K_OBJECT_TYPE_DICT => {
                            if props.string_keys_num == 0 {
                                // Empty dict.
                                let dict = tv_dict_alloc();
                                nvim_dict_inc_refcount(dict);
                                nvim_dict_set_lua_table_ref(dict, table_ref);
                                nvim_tv_set_type(cur.tv, VAR_DICT);
                                nvim_tv_set_dict(cur.tv, dict);
                            } else if props.has_string_with_nul {
                                // Special dict for NUL-keyed mappings.
                                let val_list = decode_create_map_special_dict(
                                    cur.tv,
                                    props.string_keys_num as isize,
                                );
                                // _VAL list: set lua_table_ref
                                nvim_list_set_lua_table_ref(val_list, table_ref);
                                // Navigate to the _VAL list typval.
                                let val_di = tv_dict_find(cur.tv, c"_VAL".as_ptr(), 4);
                                debug_assert!(!val_di.is_null());
                                let val_di_tv = nvim_dictitem_di_tv(val_di);
                                cur.tv = val_di_tv;
                                cur.special = true;
                                cur.is_dict = false;
                                cur.list_len = props.string_keys_num;
                                cur.container = true;
                                cur.idx = lua_gettop(lstate);
                                stack.push(cur);
                                lua_pushnil(lstate); // push nil sentinel for lua_next
                            } else {
                                // Normal dict with string keys.
                                let dict = tv_dict_alloc();
                                nvim_dict_inc_refcount(dict);
                                nvim_dict_set_lua_table_ref(dict, table_ref);
                                nvim_tv_set_type(cur.tv, VAR_DICT);
                                nvim_tv_set_dict(cur.tv, dict);
                                cur.special = false;
                                cur.is_dict = true;
                                cur.container = true;
                                cur.idx = lua_gettop(lstate);
                                stack.push(cur);
                                lua_pushnil(lstate); // push nil sentinel for lua_next
                            }
                        }
                        k if k == K_OBJECT_TYPE_FLOAT => {
                            nvim_tv_set_type(cur.tv, VAR_FLOAT);
                            nvim_tv_set_float(cur.tv, props.val);
                        }
                        _ => {
                            // kObjectTypeNil: mixed or invalid table
                            emsg(
                                c"E5100: Cannot convert given Lua table: table should contain either only integer keys or only string keys".as_ptr(),
                            );
                            ret = false;
                        }
                    }
                    // goto nlua_pop_typval_table_processing_end (break the labeled block)
                    break 'leaf;
                }
                t if t == LUA_TFUNCTION_PH10 => {
                    let func = crate::refs::rs_nlua_ref_global(lstate, -1);
                    #[allow(clippy::cast_possible_truncation)]
                    let name = register_luafunc(func as c_int);
                    nvim_tv_set_type(cur.tv, VAR_FUNC);
                    nvim_tv_set_string(cur.tv, xstrdup(name));
                }
                t if t == LUA_TUSERDATA_PH10 => {
                    let nil_ref = get_nil_ref(lstate);
                    crate::refs::rs_nlua_pushref(lstate, nil_ref);
                    let is_nil = lua_rawequal(lstate, -2, -1) != 0;
                    lua_pop_ph10(lstate, 1);
                    if is_nil {
                        nvim_tv_set_type(cur.tv, VAR_SPECIAL);
                        nvim_tv_set_special(cur.tv, K_SPECIAL_VAR_NULL);
                    } else {
                        emsg(c"E5101: Cannot convert given Lua type".as_ptr());
                        ret = false;
                    }
                }
                _ => {
                    emsg(c"E5101: Cannot convert given Lua type".as_ptr());
                    ret = false;
                }
            }
        } // end 'leaf

        if !cur.container {
            lua_pop_ph10(lstate, 1);
        }
    }

    if !ret {
        tv_clear(ret_tv);
        nvim_tv_set_type(ret_tv, VAR_NUMBER);
        nvim_tv_set_lock(ret_tv, VAR_UNLOCKED);
        nvim_tv_set_number(ret_tv, 0);
        let excess = lua_gettop(lstate) - initial_size + 1;
        if excess > 0 {
            lua_pop_ph10(lstate, excess);
        }
    }

    debug_assert_eq!(lua_gettop(lstate), initial_size - 1);
    ret
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
