// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagname_T {
    pub tn_tags: *mut ::core::ffi::c_char,
    pub tn_np: *mut ::core::ffi::c_char,
    pub tn_did_filefind_init: ::core::ffi::c_int,
    pub tn_hf_idx: ::core::ffi::c_int,
    pub tn_search_ctx: *mut ::core::ffi::c_void,
}
