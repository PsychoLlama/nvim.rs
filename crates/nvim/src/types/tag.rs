#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

#[derive(Copy, Clone)]
pub struct tagname_T {
    pub tn_tags: *mut ::core::ffi::c_char,
    pub tn_np: *mut ::core::ffi::c_char,
    pub tn_did_filefind_init: ::core::ffi::c_int,
    pub tn_hf_idx: ::core::ffi::c_int,
    pub tn_search_ctx: *mut ::core::ffi::c_void,
}
