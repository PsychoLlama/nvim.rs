// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ucmd_T {
    pub uc_name: *mut ::core::ffi::c_char,
    pub uc_argt: uint32_t,
    pub uc_rep: *mut ::core::ffi::c_char,
    pub uc_def: int64_t,
    pub uc_compl: ::core::ffi::c_int,
    pub uc_addr_type: cmd_addr_T,
    pub uc_script_ctx: sctx_T,
    pub uc_compl_arg: *mut ::core::ffi::c_char,
    pub uc_compl_luaref: LuaRef,
    pub uc_preview_luaref: LuaRef,
    pub uc_luaref: LuaRef,
}
