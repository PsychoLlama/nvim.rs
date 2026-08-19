//! The editor context: a snapshot of the session's state.
//!
//! `nvim_get_context` builds the msgpack dictionary `:mksession`-style
//! state is carried in (registers, jumplist, buffer list, global and
//! script-local variables and functions) and `nvim_load_context` applies
//! one back.  `nvim_get_mode` is the small sibling that reports only the
//! current mode and whether input is blocked.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, dict_put, has_key};

pub unsafe fn nvim_get_context(
    opts: *mut KeyDict_context,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut types: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        if has_key((*opts).is_set__context_, KEYSET_OPTIDX_context__types) {
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
                if (*types.items.add(i)).type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let s: *const ::core::ffi::c_char = (*types.items.add(i)).data.string.data;
                    if strequal(s, c"regs".as_ptr()) {
                        int_types |= kCtxRegs as ::core::ffi::c_int;
                    } else if strequal(s, c"jumps".as_ptr()) {
                        int_types |= kCtxJumps as ::core::ffi::c_int;
                    } else if strequal(s, c"bufs".as_ptr()) {
                        int_types |= kCtxBufs as ::core::ffi::c_int;
                    } else if strequal(s, c"gvars".as_ptr()) {
                        int_types |= kCtxGVars as ::core::ffi::c_int;
                    } else if strequal(s, c"sfuncs".as_ptr()) {
                        int_types |= kCtxSFuncs as ::core::ffi::c_int;
                    } else if strequal(s, c"funcs".as_ptr()) {
                        int_types |= kCtxFuncs as ::core::ffi::c_int;
                    } else if true {
                        api_err_invalid(err, c"type".as_ptr(), s, 0 as int64_t, true);
                        return Dict {
                            size: 0 as size_t,
                            capacity: 0 as size_t,
                            items: ::core::ptr::null_mut::<KeyValuePair>(),
                        }
                        .reported(error);
                    }
                }
                i = i.wrapping_add(1);
            }
        }
        let mut ctx: Context = CONTEXT_INIT;
        ctx_save(&raw mut ctx, int_types);
        let mut dict: Dict = ctx_to_dict(&raw mut ctx, arena);
        ctx_free(&raw mut ctx);
        return dict.reported(error);
    }
}

pub unsafe fn nvim_load_context(dict: Dict) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
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
        return NIL.reported(error);
    }
}

pub unsafe fn nvim_get_mode(arena: *mut Arena) -> Dict {
    unsafe {
        let mut rv: Dict = arena_dict(arena, 2 as size_t);
        let mut modestr: *mut ::core::ffi::c_char =
            arena_alloc(arena, MODE_MAX_LENGTH as size_t, false) as *mut ::core::ffi::c_char;
        get_mode(modestr);
        let mut blocked: bool = input_blocking();
        dict_put(&mut rv, c"mode", Object::string(cstr_as_string(modestr)));
        dict_put(&mut rv, c"blocking", Object::boolean(blocked));
        return rv;
    }
}
