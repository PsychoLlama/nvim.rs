use std::ffi::c_char;
use std::os::raw::c_int;

use crate::{
    ffi, Context, Dict, KeyValuePair, NvimString, Object, ObjectData, K_OBJECT_TYPE_ARRAY,
};

type ArenaHandle = *mut std::ffi::c_void;
type ErrorHandle = *mut std::ffi::c_void;

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

#[export_name = "ctx_to_dict"]
pub unsafe extern "C" fn rs_ctx_to_dict(ctx: *mut Context, arena: ArenaHandle) -> Dict {
    ctx_to_dict_impl(ctx, arena)
}

#[export_name = "ctx_from_dict"]
pub unsafe extern "C" fn rs_ctx_from_dict(
    dict: Dict,
    ctx: *mut Context,
    err: ErrorHandle,
) -> c_int {
    ffi::nvim_ctx_from_dict_impl(dict, ctx, err)
}
