use std::os::raw::c_int;

use crate::{ctx_stack, ffi, rs_ctx_free, stack, Context};

const KCTX_REGS: c_int = 1;
const KCTX_JUMPS: c_int = 2;
const KCTX_BUFS: c_int = 4;
const KCTX_GVARS: c_int = 8;
const KCTX_FUNCS: c_int = 32;

// kShaDaWantInfo (1) | kShaDaForceit (4)
const SHADA_READ_FLAGS: c_int = 5;

#[export_name = "ctx_restore"]
pub unsafe extern "C" fn rs_ctx_restore(ctx: *mut Context, flags: c_int) -> bool {
    let mut free_ctx = false;
    let ctx = if ctx.is_null() {
        if ctx_stack.size == 0 {
            return false;
        }
        free_ctx = true;
        stack::ctx_stack_pop_slot()
    } else {
        ctx
    };

    let c = &mut *ctx;

    ffi::nvim_ctx_save_shada_opt();
    ffi::nvim_ctx_set_shada_restore();

    if flags & KCTX_REGS != 0 {
        ffi::rs_shada_read_string(c.regs, SHADA_READ_FLAGS);
    }
    if flags & KCTX_JUMPS != 0 {
        ffi::rs_shada_read_string(c.jumps, SHADA_READ_FLAGS);
    }
    if flags & KCTX_BUFS != 0 {
        ffi::rs_shada_read_string(c.bufs, SHADA_READ_FLAGS);
    }
    if flags & KCTX_GVARS != 0 {
        ffi::rs_shada_read_string(c.gvars, SHADA_READ_FLAGS);
    }
    if flags & KCTX_FUNCS != 0 {
        ffi::nvim_ctx_restore_funcs(ctx);
    }

    if free_ctx {
        rs_ctx_free(ctx);
    }

    ffi::nvim_ctx_restore_shada_opt();
    true
}
