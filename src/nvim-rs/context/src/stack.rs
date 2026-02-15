use crate::{ffi, rs_ctx_free};

#[export_name = "ctx_free_all"]
pub unsafe extern "C" fn rs_ctx_free_all() {
    let size = ffi::nvim_get_ctx_stack_size();
    for i in 0..size {
        let ctx = ffi::nvim_ctx_stack_at_forward(i);
        if !ctx.is_null() {
            rs_ctx_free(ctx);
        }
    }
    ffi::nvim_ctx_stack_destroy();
}
