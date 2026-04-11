use std::ffi::c_char;
use std::os::raw::c_int;

use nvim_option::storage::{OptVal, OptValData, String_};
use nvim_option::OptValType;

use crate::{ctx_stack, ffi, rs_ctx_free, stack, Context};

const KCTX_REGS: c_int = 1;
const KCTX_JUMPS: c_int = 2;
const KCTX_BUFS: c_int = 4;
const KCTX_GVARS: c_int = 8;
const KCTX_FUNCS: c_int = 32;

// kShaDaWantInfo (1) | kShaDaForceit (4)
const SHADA_READ_FLAGS: c_int = 5;

/// `kOptShada` = 254 (from `options_enum.generated.h`)
const K_OPT_SHADA: c_int = 254;

/// `OPT_GLOBAL` = 0x01 (from option.h)
const OPT_GLOBAL: c_int = 0x01;

/// Mirrors `"!,'100,%"` as a static string for `STATIC_CSTR_AS_OPTVAL`.
static SHADA_RESTORE_STR: &[u8] = b"!,'100,%\0";

/// Restore functions from a context by executing each function body via `do_cmdline_cmd`.
/// Replaces C `nvim_ctx_restore_funcs`.
unsafe fn ctx_restore_funcs(ctx: &Context) {
    for i in 0..ctx.funcs.size {
        let item = &*ctx.funcs.items.add(i);
        // item is an Object; funcs contains STRING_OBJ values
        // ObjectData.string is the NvimString
        ffi::do_cmdline_cmd(item.data.string.data);
    }
}

/// Saved shada option value (mirrors `saved_shada_opt` static in context.c).
static mut SAVED_SHADA_OPT: OptVal = OptVal {
    type_: OptValType::Nil,
    data: OptValData { number: 0 },
};

/// Save the current 'shada' option value globally.
/// Replaces C `nvim_ctx_save_shada_opt`.
unsafe fn ctx_save_shada_opt() {
    SAVED_SHADA_OPT = ffi::rs_get_option_value(K_OPT_SHADA, OPT_GLOBAL);
}

/// Set 'shada' to the restore value `"!,'100,%"`.
/// Replaces C `nvim_ctx_set_shada_restore`.
unsafe fn ctx_set_shada_restore() {
    // STATIC_CSTR_AS_OPTVAL("!,'100,%") expanded:
    //   OptVal { type: kOptValTypeString, data.string: { data: "!,'100,%", size: 8 } }
    let restore_val = OptVal {
        type_: OptValType::String,
        data: OptValData {
            string: String_ {
                data: SHADA_RESTORE_STR.as_ptr().cast::<c_char>().cast_mut(),
                size: SHADA_RESTORE_STR.len() - 1, // exclude NUL
            },
        },
    };
    ffi::rs_set_option_value(K_OPT_SHADA, restore_val, OPT_GLOBAL);
}

/// Restore 'shada' to the previously saved value and free it.
/// Replaces C `nvim_ctx_restore_shada_opt`.
unsafe fn ctx_restore_shada_opt() {
    ffi::rs_set_option_value(K_OPT_SHADA, SAVED_SHADA_OPT, OPT_GLOBAL);
    ffi::rs_optval_free(SAVED_SHADA_OPT);
}

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

    ctx_save_shada_opt();
    ctx_set_shada_restore();

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
        ctx_restore_funcs(c);
    }

    if free_ctx {
        rs_ctx_free(ctx);
    }

    ctx_restore_shada_opt();
    true
}
