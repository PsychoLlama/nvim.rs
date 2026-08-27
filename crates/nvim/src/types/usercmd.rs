#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// Neither `Copy` nor `Clone`. One entry owns its name, its replacement
/// text, its `-complete=` argument and its three Lua references; nothing may
/// duplicate one, and the absence of the derives is what says so.
pub struct ucmd_T {
    pub uc_name: *mut ::core::ffi::c_char,
    pub uc_argt: ExArgt,
    pub uc_rep: *mut ::core::ffi::c_char,
    pub uc_def: int64_t,
    pub uc_compl: ExpandContext,
    pub uc_addr_type: CmdAddr,
    pub uc_script_ctx: sctx_T,
    pub uc_compl_arg: *mut ::core::ffi::c_char,
    pub uc_compl_luaref: LuaRef,
    pub uc_preview_luaref: LuaRef,
    pub uc_luaref: LuaRef,
}
