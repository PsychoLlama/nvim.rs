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

/// # Safety
///
/// `opts` must point at the `KeyDict_context` the dispatcher filled in, live
/// for the call. `arena` must point at a live arena, which the memory this
/// answers with is taken from and must outlive.
pub unsafe fn nvim_get_context(opts: *mut KeyDict_context) -> Result<ApiDict, Error> {
    let mut error = Error::none();
    let types: Array = unsafe { (*opts).types.as_ref() }
        .unwrap_or(&Array::EMPTY)
        .clone();
    let mut int_types: ::core::ffi::c_int = if types.len() > 0 as size_t {
        0 as ::core::ffi::c_int
    } else {
        kCtxAll.get()
    };
    if types.len() > 0 as size_t {
        let mut i: size_t = 0 as size_t;
        while i < types.len() {
            // SAFETY: `types` names its own `size` items.
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
    unsafe { ctx_save(&raw mut ctx, int_types) };
    let dict: ApiDict = unsafe { ctx_to_dict(&raw mut ctx) };
    unsafe { ctx_free(&raw mut ctx) };
    dict.reported(error)
}

/// # Safety
///
/// `dict` must be a well-formed API dictionary, its `size` entries
/// initialized.
pub unsafe fn nvim_load_context(dict: ApiDict) -> Result<Object, Error> {
    let mut ctx: Context = CONTEXT_INIT;
    let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(0);
    let read = unsafe { ctx_from_dict(dict, &raw mut ctx) };
    if read.is_ok() {
        // SAFETY: `ctx` is this frame's own, filled in above.
        unsafe { ctx_restore(&raw mut ctx, kCtxAll.get()) };
    }
    unsafe { ctx_free(&raw mut ctx) };
    did_emsg.set(save_did_emsg);
    read.map(|_| Object::Nil)
}

/// # Safety
///
/// `arena` must point at a live arena, which the memory this answers with is
/// taken from and must outlive.
pub unsafe fn nvim_get_mode(arena: *mut Arena) -> ApiDict {
    let mut rv: ApiDict = ApiDict::with_capacity(2 as size_t);
    let modestr: *mut ::core::ffi::c_char =
        unsafe { arena_alloc(arena, MODE_MAX_LENGTH as size_t, false) } as *mut ::core::ffi::c_char;
    // The name is copied into the arena because the `ApiDict` borrows it;
    // `get_mode` answers exactly `MODE_MAX_LENGTH` NUL-padded bytes.
    unsafe { modestr.copy_from_nonoverlapping(get_mode().as_ptr(), MODE_MAX_LENGTH as size_t) };
    let blocked: bool = input_blocking();
    // SAFETY: `modestr` is the NUL-padded buffer filled just above.
    let mode = unsafe { cstr_to_string(modestr) };
    rv.insert(String_0::from_cstr(c"mode"), Object::string(mode));
    rv.insert(String_0::from_cstr(c"blocking"), Object::boolean(blocked));
    rv
}
