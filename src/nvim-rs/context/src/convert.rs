use std::os::raw::c_int;

use crate::{ffi, Context, Dict};

type ArenaHandle = *mut std::ffi::c_void;
type ErrorHandle = *mut std::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn rs_ctx_to_dict(ctx: *mut Context, arena: ArenaHandle) -> Dict {
    ffi::nvim_ctx_to_dict_impl(ctx, arena)
}

#[no_mangle]
pub unsafe extern "C" fn rs_ctx_from_dict(
    dict: Dict,
    ctx: *mut Context,
    err: ErrorHandle,
) -> c_int {
    ffi::nvim_ctx_from_dict_impl(dict, ctx, err)
}
