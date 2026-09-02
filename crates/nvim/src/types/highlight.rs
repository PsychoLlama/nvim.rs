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
use crate::highlight::HlAttrFlags;

#[derive(Copy, Clone)]
pub struct ColorItem {
    pub attr_id: ::core::ffi::c_int,
    pub link_id: ::core::ffi::c_int,
    pub version: ::core::ffi::c_int,
    pub is_default: bool,
    pub link_global: bool,
}
#[derive(PartialEq, Eq, Hash)]
pub struct ColorKey {
    pub ns_id: ::core::ffi::c_int,
    pub syn_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct HlAttrs {
    pub rgb_ae_attr: HlAttrFlags,
    pub cterm_ae_attr: HlAttrFlags,
    pub rgb_fg_color: RgbValue,
    pub rgb_bg_color: RgbValue,
    pub rgb_sp_color: RgbValue,
    pub cterm_fg_color: int16_t,
    pub cterm_bg_color: int16_t,
    pub hl_blend: int32_t,
    pub url: int32_t,
}
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct HlEntry {
    pub attr: HlAttrs,
    pub kind: HlKind,
    pub id1: ::core::ffi::c_int,
    pub id2: ::core::ffi::c_int,
}
pub type HlKind = ::core::ffi::c_uint;
pub type RgbValue = int32_t;
pub type hlf_T = ::core::ffi::c_int;
