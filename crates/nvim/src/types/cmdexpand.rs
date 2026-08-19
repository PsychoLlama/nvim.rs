#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

crate::flag_set! {
    /// How the completion machinery must escape a backslash in what it
    /// answers -- upstream's `XP_BS_*`, the bits [`expand_T::xp_backslash`]
    /// carries. `NONE` is upstream's `XP_BS_NONE`: the context takes its
    /// text literally and nothing is escaped.
    pub struct BackslashEscape;

    /// A space is escaped with one backslash.
    const ONE = 1;
    /// A space is escaped with three backslashes -- the `'*func'` options,
    /// where the value is read back through another layer.
    const THREE = 2;
    /// A comma is escaped as well as a space.
    const COMMA = 4;
}

pub type CompleteListItemGetter =
    Option<unsafe fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ::core::ffi::c_int,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: BackslashEscape,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type xp_prefix_T = ::core::ffi::c_uint;
