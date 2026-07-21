// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorItem {
    pub attr_id: ::core::ffi::c_int,
    pub link_id: ::core::ffi::c_int,
    pub version: ::core::ffi::c_int,
    pub is_default: bool,
    pub link_global: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ColorKey {
    pub ns_id: ::core::ffi::c_int,
    pub syn_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlAttrs {
    pub rgb_ae_attr: int32_t,
    pub cterm_ae_attr: int32_t,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlEntry {
    pub attr: HlAttrs,
    pub kind: HlKind,
    pub id1: ::core::ffi::c_int,
    pub id2: ::core::ffi::c_int,
    pub winid: ::core::ffi::c_int,
}
pub type HlKind = ::core::ffi::c_uint;
pub type RgbValue = int32_t;
pub type hlf_T = ::core::ffi::c_uint;
