// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type OptScope = ::core::ffi::c_uint;
pub type OptScopeFlags = uint8_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OptVal {
    pub type_0: OptValType,
    pub data: OptValData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union OptValData {
    pub boolean: TriState,
    pub number: OptInt,
    pub string: String_0,
}
pub type OptValType = ::core::ffi::c_int;
pub type opt_did_set_cb_T =
    Option<unsafe extern "C" fn(*mut optset_T) -> *const ::core::ffi::c_char>;
pub type opt_expand_cb_T = Option<
    unsafe extern "C" fn(
        *mut optexpand_T,
        *mut ::core::ffi::c_int,
        *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct optexpand_T {
    pub oe_varp: *mut ::core::ffi::c_char,
    pub oe_idx: OptIndex,
    pub oe_opt_value: *mut ::core::ffi::c_char,
    pub oe_append: bool,
    pub oe_include_orig_val: bool,
    pub oe_regmatch: *mut regmatch_T,
    pub oe_xp: *mut expand_T,
    pub oe_set_arg: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct optset_T {
    pub os_varp: *mut ::core::ffi::c_void,
    pub os_idx: OptIndex,
    pub os_flags: ::core::ffi::c_int,
    pub os_oldval: OptValData,
    pub os_newval: OptValData,
    pub os_value_checked: bool,
    pub os_value_changed: bool,
    pub os_restore_chartab: bool,
    pub os_errbuf: *mut ::core::ffi::c_char,
    pub os_errbuflen: size_t,
    pub os_win: *mut ::core::ffi::c_void,
    pub os_buf: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimoption_T {
    pub fullname: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
    pub flags: uint32_t,
    pub type_0: OptValType,
    pub scope_flags: OptScopeFlags,
    pub var: *mut ::core::ffi::c_void,
    pub flags_var: *mut ::core::ffi::c_uint,
    pub scope_idx: [ssize_t; 3],
    pub immutable: bool,
    pub values: *mut *const ::core::ffi::c_char,
    pub values_len: size_t,
    pub opt_did_set_cb: opt_did_set_cb_T,
    pub opt_expand_cb: opt_expand_cb_T,
    pub def_val: OptVal,
    pub script_ctx: sctx_T,
}
