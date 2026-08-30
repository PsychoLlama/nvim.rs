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

/// The `types` key's spellings, and the `kCtx*` bit each one names.
const NAMES: [&::core::ffi::CStr; 6] = [c"regs", c"jumps", c"bufs", c"gvars", c"sfuncs", c"funcs"];

/// [`NAMES`]' bits, in the same order.
const FLAGS: [::core::ffi::c_int; 6] = [
    kCtxRegs as ::core::ffi::c_int,
    kCtxJumps as ::core::ffi::c_int,
    kCtxBufs as ::core::ffi::c_int,
    kCtxGVars as ::core::ffi::c_int,
    kCtxSFuncs as ::core::ffi::c_int,
    kCtxFuncs as ::core::ffi::c_int,
];

pub unsafe fn nvim_get_context(
    opts: *mut KeyDict_context,
    arena: *mut Arena,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut types: Array = Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    };
    if has_key(
        unsafe { (*opts).is_set__context_ },
        KEYSET_OPTIDX_context__types,
    ) {
        types = unsafe { (*opts).types };
    }
    let mut int_types: ::core::ffi::c_int = if types.size > 0 as size_t {
        0 as ::core::ffi::c_int
    } else {
        kCtxAll.get()
    };
    if types.size > 0 as size_t {
        let mut i: size_t = 0 as size_t;
        while i < types.size {
            // SAFETY: `types` names its own `size` items, and the tag says
            // whether the string arm is the live one.
            let named = unsafe {
                let item = *types.items.add(i);
                (item.type_0 == kObjectTypeString).then(|| item.data.string.data())
            };
            if let Some(s) = named {
                // SAFETY: the keyset's strings are NUL-terminated.
                let which = unsafe { NAMES.iter().position(|n| strequal(s, n.as_ptr())) };
                if let Some(which) = which {
                    int_types |= FLAGS[which];
                } else {
                    // SAFETY: `err` is this frame's own slot.
                    unsafe { api_err_invalid(err, c"type".as_ptr(), s, 0, true) };
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
    unsafe { ctx_save(&raw mut ctx, int_types) };
    let mut dict: Dict = unsafe { ctx_to_dict(&raw mut ctx, arena) };
    unsafe { ctx_free(&raw mut ctx) };
    dict.reported(error)
}

pub unsafe fn nvim_load_context(dict: Dict) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut ctx: Context = CONTEXT_INIT;
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(0);
    unsafe { ctx_from_dict(dict, &raw mut ctx, err) };
    if !error.is_set() {
        // SAFETY: `ctx` is this frame's own, filled in above.
        unsafe { ctx_restore(&raw mut ctx, kCtxAll.get()) };
    }
    unsafe { ctx_free(&raw mut ctx) };
    did_emsg.set(save_did_emsg);
    NIL.reported(error)
}

pub unsafe fn nvim_get_mode(arena: *mut Arena) -> Dict {
    let mut rv: Dict = arena_dict(arena, 2 as size_t);
    let mut modestr: *mut ::core::ffi::c_char =
        unsafe { arena_alloc(arena, MODE_MAX_LENGTH as size_t, false) } as *mut ::core::ffi::c_char;
    // The name is copied into the arena because the `Dict` borrows it;
    // `get_mode` answers exactly `MODE_MAX_LENGTH` NUL-padded bytes.
    unsafe { modestr.copy_from_nonoverlapping(get_mode().as_ptr(), MODE_MAX_LENGTH as size_t) };
    let mut blocked: bool = input_blocking();
    unsafe { dict_put(&mut rv, c"mode", Object::string(cstr_as_string(modestr))) };
    unsafe { dict_put(&mut rv, c"blocking", Object::boolean(blocked)) };
    rv
}
