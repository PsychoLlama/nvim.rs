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

#[derive(Default)]
pub struct pumitem_T {
    pub pum_text: *mut ::core::ffi::c_char,
    pub pum_kind: *mut ::core::ffi::c_char,
    pub pum_extra: *mut ::core::ffi::c_char,
    pub pum_info: *mut ::core::ffi::c_char,
    pub pum_cpt_source_idx: ::core::ffi::c_int,
    pub pum_user_abbr_hlattr: ::core::ffi::c_int,
    pub pum_user_kind_hlattr: ::core::ffi::c_int,
}
