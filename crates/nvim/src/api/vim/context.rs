//! The editor context: a snapshot of the session's state.
//!
//! `nvim_get_context` builds the msgpack dictionary `:mksession`-style
//! state is carried in (registers, jumplist, buffer list, global and
//! script-local variables and functions) and `nvim_load_context` applies
//! one back.  `nvim_get_mode` is the small sibling that reports only the
//! current mode and whether input is blocked.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::err_bad_value;
use crate::cstr;

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

pub fn nvim_get_context(opts: &mut KeyDict_context) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    let types: Array = opts.types.as_ref().unwrap_or(&Array::EMPTY).clone();
    let mut int_types: ::core::ffi::c_int = if types.len() > 0 as size_t {
        0 as ::core::ffi::c_int
    } else {
        CTX_ALL
    };
    if types.len() > 0 as size_t {
        let mut i: size_t = 0 as size_t;
        while i < types.len() {
            let item = &types[i];
            let named = item.as_string().map(|s| s.data());
            if let Some(s) = named {
                // SAFETY: the keyset's strings are NUL-terminated.
                let which = unsafe { NAMES.iter().position(|n| strequal(s, n.as_ptr())) };
                if let Some(which) = which {
                    int_types |= FLAGS[which];
                } else {
                    // SAFETY: the keyset's strings are NUL-terminated.
                    error = err_bad_value(c"type", unsafe { cstr::at(s) });
                    return ApiDict::EMPTY.reported(error);
                }
            }
            i = i.wrapping_add(1);
        }
    }
    let mut ctx: Context = CONTEXT_INIT;
    ctx_save(Some(&mut ctx), int_types);
    let dict: ApiDict = ctx_to_dict(&ctx);
    ctx_free(&mut ctx);
    dict.reported(error)
}

pub fn nvim_load_context(dict: ApiDict) -> Result<Object, Error> {
    let mut ctx: Context = CONTEXT_INIT;
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(0);
    let read = ctx_from_dict(dict, &mut ctx);
    if read.is_ok() {
        ctx_restore(Some(&ctx), CTX_ALL);
    }
    ctx_free(&mut ctx);
    did_emsg.set(save_did_emsg);
    read.map(|_| Object::Nil)
}

/// The current mode's short name, and whether the editor is blocked waiting
/// for input.
pub fn nvim_get_mode() -> ApiDict {
    let mut rv: ApiDict = ApiDict::with_capacity(2 as size_t);
    // `get_mode` answers the name NUL-padded to `MODE_MAX_LENGTH` bytes.
    let modestr = get_mode().map(|c| c.cast_unsigned());
    let len = modestr
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(modestr.len());
    let blocked: bool = input_blocking();
    let mode = String_0::from_bytes(&modestr[..len]);
    rv.insert(c"mode", Object::string(mode));
    rv.insert(c"blocking", Object::boolean(blocked));
    rv
}
