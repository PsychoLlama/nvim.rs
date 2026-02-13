use std::os::raw::c_int;

use crate::{Array, Context, Dict, NvimString};

extern "C" {
    // Stack access
    pub fn nvim_get_ctx_stack_size() -> usize;
    pub fn nvim_get_ctx_at_index(index: usize) -> *mut Context;
    pub fn nvim_ctx_stack_at_forward(index: usize) -> *mut Context;

    // Stack mutation
    pub fn nvim_ctx_stack_push_init();
    pub fn nvim_ctx_stack_last() -> *mut Context;
    pub fn nvim_ctx_stack_pop() -> *mut Context;
    pub fn nvim_ctx_stack_destroy();

    // Memory
    pub fn rs_api_free_string(value: NvimString);
    pub fn rs_api_free_array(value: Array);

    // ShaDa encoding (already in shada.c)
    pub fn nvim_shada_encode_regs() -> NvimString;
    pub fn nvim_shada_encode_jumps() -> NvimString;
    pub fn nvim_shada_encode_buflist() -> NvimString;
    pub fn nvim_shada_encode_gvars() -> NvimString;

    // ShaDa reading (already in shada.c)
    pub fn nvim_shada_read_string(s: NvimString, flags: c_int);

    // Function save/restore (kept in C due to HASHTAB_ITER, exec_impl coupling)
    pub fn nvim_ctx_save_funcs(ctx: *mut Context, scriptonly: bool);
    pub fn nvim_ctx_restore_funcs(ctx: *mut Context);

    // Option save/restore for shada
    pub fn nvim_ctx_save_shada_opt();
    pub fn nvim_ctx_set_shada_restore();
    pub fn nvim_ctx_restore_shada_opt();

    // Dict conversion (kept in C due to Arena/API coupling)
    pub fn nvim_ctx_to_dict_impl(ctx: *mut Context, arena: *mut std::ffi::c_void) -> Dict;
    pub fn nvim_ctx_from_dict_impl(
        dict: Dict,
        ctx: *mut Context,
        err: *mut std::ffi::c_void,
    ) -> c_int;
}
