// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type DoInRuntimepathCB = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        *mut *mut ::core::ffi::c_char,
        bool,
        *mut ::core::ffi::c_void,
    ) -> bool,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct estack_T {
    pub es_lnum: linenr_T,
    pub es_name: *mut ::core::ffi::c_char,
    pub es_type: etype_T,
    pub es_info: estack_T_es_info,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union estack_T_es_info {
    pub sctx: *mut sctx_T,
    pub ufunc: *mut ufunc_T,
    pub aucmd: *mut AutoPatCmd,
    pub except: *mut except_T,
}
pub type estack_arg_T = ::core::ffi::c_uint;
pub type etype_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct scriptitem_T {
    pub sn_vars: *mut scriptvar_T,
    pub sn_name: *mut ::core::ffi::c_char,
    pub sn_lua: bool,
    pub sn_prof_on: bool,
    pub sn_pr_force: bool,
    pub sn_pr_child: proftime_T,
    pub sn_pr_nest: ::core::ffi::c_int,
    pub sn_pr_count: ::core::ffi::c_int,
    pub sn_pr_total: proftime_T,
    pub sn_pr_self: proftime_T,
    pub sn_pr_start: proftime_T,
    pub sn_pr_children: proftime_T,
    pub sn_prl_ga: garray_T,
    pub sn_prl_start: proftime_T,
    pub sn_prl_children: proftime_T,
    pub sn_prl_wait: proftime_T,
    pub sn_prl_idx: linenr_T,
    pub sn_prl_execed: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct scriptvar_T {
    pub sv_var: ScopeDictDictItem,
    pub sv_dict: dict_T,
}
