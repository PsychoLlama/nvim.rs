#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct ucmd_T {
    pub uc_name: *mut ::core::ffi::c_char,
    pub uc_argt: ExArgt,
    pub uc_rep: *mut ::core::ffi::c_char,
    pub uc_def: int64_t,
    pub uc_compl: ::core::ffi::c_int,
    pub uc_addr_type: CmdAddr,
    pub uc_script_ctx: sctx_T,
    pub uc_compl_arg: *mut ::core::ffi::c_char,
    pub uc_compl_luaref: LuaRef,
    pub uc_preview_luaref: LuaRef,
    pub uc_luaref: LuaRef,
}
