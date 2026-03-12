use std::os::raw::c_int;

use crate::{Array, Context, Dict, NvimString};

extern "C" {
    // Memory
    #[link_name = "api_free_string"]
    pub fn rs_api_free_string(value: NvimString);
    #[link_name = "api_free_array"]
    pub fn rs_api_free_array(value: Array);
    pub fn xfree(ptr: *mut std::ffi::c_void);
    pub fn xrealloc(ptr: *mut std::ffi::c_void, size: usize) -> *mut std::ffi::c_void;

    // ShaDa encoding (implemented in nvim-shada Rust crate)
    pub fn rs_shada_encode_regs() -> NvimString;
    pub fn rs_shada_encode_jumps() -> NvimString;
    pub fn rs_shada_encode_buflist() -> NvimString;
    pub fn rs_shada_encode_gvars() -> NvimString;

    // ShaDa reading (implemented in nvim-shada Rust crate)
    pub fn rs_shada_read_string(s: NvimString, flags: c_int);

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
