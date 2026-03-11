use std::os::raw::c_int;

use crate::{ffi, stack, Context};

const KCTX_REGS: c_int = 1;
const KCTX_JUMPS: c_int = 2;
const KCTX_BUFS: c_int = 4;
const KCTX_GVARS: c_int = 8;
const KCTX_SFUNCS: c_int = 16;
const KCTX_FUNCS: c_int = 32;

#[export_name = "ctx_save"]
pub unsafe extern "C" fn rs_ctx_save(ctx: *mut Context, flags: c_int) {
    let ctx = stack::ctx_stack_push_or_ptr(ctx);

    let c = &mut *ctx;

    if flags & KCTX_REGS != 0 {
        c.regs = ffi::rs_shada_encode_regs();
    }
    if flags & KCTX_JUMPS != 0 {
        c.jumps = ffi::rs_shada_encode_jumps();
    }
    if flags & KCTX_BUFS != 0 {
        c.bufs = ffi::rs_shada_encode_buflist();
    }
    if flags & KCTX_GVARS != 0 {
        c.gvars = ffi::rs_shada_encode_gvars();
    }

    if flags & KCTX_FUNCS != 0 {
        ffi::nvim_ctx_save_funcs(ctx, false);
    } else if flags & KCTX_SFUNCS != 0 {
        ffi::nvim_ctx_save_funcs(ctx, true);
    }
}
