//! The editor context: a snapshot of the session's state.
//!
//! `nvim_get_context` builds the msgpack dictionary `:mksession`-style
//! state is carried in (registers, jumplist, buffer list, global and
//! script-local variables and functions) and `nvim_load_context` applies
//! one back.  `nvim_get_mode` is the small sibling that reports only the
//! current mode and whether input is blocked.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_get_context(
    mut opts: *mut KeyDict_context,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut types: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        if (*opts).is_set__context_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_context__types
            != 0 as ::core::ffi::c_ulonglong
        {
            types = (*opts).types;
        }
        let mut int_types: ::core::ffi::c_int = if types.size > 0 as size_t {
            0 as ::core::ffi::c_int
        } else {
            kCtxAll.get()
        };
        if types.size > 0 as size_t {
            let mut i: size_t = 0 as size_t;
            while i < types.size {
                if (*types.items.offset(i as isize)).type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let s: *const ::core::ffi::c_char =
                        (*types.items.offset(i as isize)).data.string.data;
                    if strequal(s, b"regs\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxRegs as ::core::ffi::c_int;
                    } else if strequal(s, b"jumps\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxJumps as ::core::ffi::c_int;
                    } else if strequal(s, b"bufs\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxBufs as ::core::ffi::c_int;
                    } else if strequal(s, b"gvars\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxGVars as ::core::ffi::c_int;
                    } else if strequal(s, b"sfuncs\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxSFuncs as ::core::ffi::c_int;
                    } else if strequal(s, b"funcs\0".as_ptr() as *const ::core::ffi::c_char) {
                        int_types |= kCtxFuncs as ::core::ffi::c_int;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"type\0".as_ptr() as *const ::core::ffi::c_char,
                            s,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        return Dict {
                            size: 0 as size_t,
                            capacity: 0 as size_t,
                            items: ::core::ptr::null_mut::<KeyValuePair>(),
                        };
                    }
                }
                i = i.wrapping_add(1);
            }
        }
        let mut ctx: Context = CONTEXT_INIT;
        ctx_save(&raw mut ctx, int_types);
        let mut dict: Dict = ctx_to_dict(&raw mut ctx, arena);
        ctx_free(&raw mut ctx);
        return dict;
    }
}

pub unsafe extern "C" fn nvim_load_context(mut dict: Dict, mut err: *mut Error) -> Object {
    unsafe {
        let mut ctx: Context = CONTEXT_INIT;
        let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
        did_emsg.set(false_0);
        ctx_from_dict(dict, &raw mut ctx, err);
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            ctx_restore(&raw mut ctx, kCtxAll.get());
        }
        ctx_free(&raw mut ctx);
        did_emsg.set(save_did_emsg);
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
}

pub unsafe extern "C" fn nvim_get_mode(mut arena: *mut Arena) -> Dict {
    unsafe {
        let mut rv: Dict = arena_dict(arena, 2 as size_t);
        let mut modestr: *mut ::core::ffi::c_char =
            arena_alloc(arena, MODE_MAX_LENGTH as size_t, false_0 != 0) as *mut ::core::ffi::c_char;
        get_mode(modestr);
        let mut blocked: bool = input_blocking();
        let c2rust_fresh10 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh10 as isize) = key_value_pair {
            key: cstr_as_string(b"mode\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(modestr),
                },
            },
        };
        let c2rust_fresh11 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh11 as isize) = key_value_pair {
            key: cstr_as_string(b"blocking\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed { boolean: blocked },
            },
        };
        return rv;
    }
}
