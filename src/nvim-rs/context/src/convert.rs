use std::ffi::c_char;
use std::os::raw::c_int;

use crate::{
    ffi, Context, Dict, KeyValuePair, NvimString, Object, ObjectData, K_OBJECT_TYPE_ARRAY,
};

type ArenaHandle = *mut std::ffi::c_void;

/// Minimal opaque Error struct matching C's `Error { ErrorType type; char *msg; }`.
/// `kErrorTypeNone = -1`; any other value means an error is set.
#[repr(C)]
struct ApiError {
    err_type: c_int,
    msg: *mut c_char,
}

impl ApiError {
    #[inline]
    const fn is_set(&self) -> bool {
        self.err_type != -1 // kErrorTypeNone = -1
    }
}

/// Create an Object wrapping an Array (mirrors C's `ARRAY_OBJ(array)` macro).
#[inline]
const fn array_obj(array: crate::Array) -> Object {
    Object {
        obj_type: K_OBJECT_TYPE_ARRAY,
        data: ObjectData { array },
    }
}

/// Create a static-string `NvimString` (mirrors C's `cstr_as_string(literal)`).
/// The pointer must point to a NUL-terminated static string literal.
#[inline]
const unsafe fn static_key(s: &'static [u8]) -> NvimString {
    NvimString {
        // Exclude the trailing NUL from the size
        data: s.as_ptr().cast::<c_char>().cast_mut(),
        size: s.len() - 1,
    }
}

/// Append a key-value pair to a pre-allocated dict (mirrors C's `PUT_C` macro).
///
/// # Safety
/// `dict` must have sufficient pre-allocated capacity (from `arena_dict`).
#[inline]
unsafe fn dict_put(dict: &mut Dict, key: NvimString, value: Object) {
    debug_assert!(dict.size < dict.capacity);
    *dict.items.add(dict.size) = KeyValuePair { key, value };
    dict.size += 1;
}

/// Compare a `NvimString` key against a static byte-string literal.
///
/// Equivalent to C's `strequal(item.key.data, "literal")` for fixed keys.
#[inline]
unsafe fn key_eq(key: NvimString, s: &[u8]) -> bool {
    if key.size != s.len() {
        return false;
    }
    if key.size == 0 {
        return true;
    }
    // Safety: key.data points to at least key.size valid bytes.
    std::slice::from_raw_parts(key.data.cast::<u8>(), key.size) == s
}

/// Context type flags (matching `ContextTypeFlags` in context.h)
const KCTX_REGS: c_int = 1;
const KCTX_JUMPS: c_int = 2;
const KCTX_BUFS: c_int = 4;
const KCTX_GVARS: c_int = 8;
const KCTX_FUNCS: c_int = 32;

/// Converts a `Context` to its `Dict` representation.
/// Replaces C `nvim_ctx_to_dict_impl`.
///
/// # Safety
/// `ctx` must be a valid non-null pointer. `arena` must be NULL or a valid Arena.
unsafe fn ctx_to_dict_impl(ctx: *mut Context, arena: ArenaHandle) -> Dict {
    debug_assert!(!ctx.is_null());
    let c = &*ctx;

    let mut rv = ffi::arena_dict(arena, 5);

    dict_put(
        &mut rv,
        static_key(b"regs\0"),
        array_obj(ffi::string_to_array(c.regs, false, arena)),
    );
    dict_put(
        &mut rv,
        static_key(b"jumps\0"),
        array_obj(ffi::string_to_array(c.jumps, false, arena)),
    );
    dict_put(
        &mut rv,
        static_key(b"bufs\0"),
        array_obj(ffi::string_to_array(c.bufs, false, arena)),
    );
    dict_put(
        &mut rv,
        static_key(b"gvars\0"),
        array_obj(ffi::string_to_array(c.gvars, false, arena)),
    );
    dict_put(
        &mut rv,
        static_key(b"funcs\0"),
        array_obj(ffi::copy_array(c.funcs, arena)),
    );

    rv
}

/// Converts a `Dict` back to a `Context`.
/// Replaces C `nvim_ctx_from_dict_impl`.
///
/// # Safety
/// `ctx` must be a valid non-null pointer. `err` must be a valid `Error *`.
unsafe fn ctx_from_dict_impl(dict: Dict, ctx: *mut Context, err: *mut ApiError) -> c_int {
    debug_assert!(!ctx.is_null());
    let c = &mut *ctx;

    let mut types: c_int = 0;

    for i in 0..dict.size {
        if (*err).is_set() {
            break;
        }
        let item = &*dict.items.add(i);
        // Only process Array-typed values
        if item.value.obj_type != K_OBJECT_TYPE_ARRAY {
            continue;
        }
        let array = item.value.data.array;

        if key_eq(item.key, b"regs") {
            types |= KCTX_REGS;
            c.regs = ffi::nvim_ctx_array_to_string(array, err.cast());
        } else if key_eq(item.key, b"jumps") {
            types |= KCTX_JUMPS;
            c.jumps = ffi::nvim_ctx_array_to_string(array, err.cast());
        } else if key_eq(item.key, b"bufs") {
            types |= KCTX_BUFS;
            c.bufs = ffi::nvim_ctx_array_to_string(array, err.cast());
        } else if key_eq(item.key, b"gvars") {
            types |= KCTX_GVARS;
            c.gvars = ffi::nvim_ctx_array_to_string(array, err.cast());
        } else if key_eq(item.key, b"funcs") {
            types |= KCTX_FUNCS;
            // copy_object(item.value, NULL).data.array
            // Use ptr::read to copy the Object out since Object is not Copy
            let obj_copy = std::ptr::read(&item.value);
            c.funcs = ffi::copy_object(obj_copy, std::ptr::null_mut()).data.array;
        }
    }

    types
}

#[export_name = "ctx_to_dict"]
pub unsafe extern "C" fn rs_ctx_to_dict(ctx: *mut Context, arena: ArenaHandle) -> Dict {
    ctx_to_dict_impl(ctx, arena)
}

#[export_name = "ctx_from_dict"]
pub unsafe extern "C" fn rs_ctx_from_dict(
    dict: Dict,
    ctx: *mut Context,
    err: *mut std::ffi::c_void,
) -> c_int {
    ctx_from_dict_impl(dict, ctx, err.cast::<ApiError>())
}
