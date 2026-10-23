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

/// One sign on a row, as the sign column sorts it: the sign item's index in
/// the decoration store, its mark's id, and the two fields of the item the
/// order reads.
#[derive(Copy, Clone)]
pub struct SignItem {
    pub idx: uint32_t,
    pub id: uint32_t,
    pub priority: DecorPriority,
    pub sign_add_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct SignTextAttrs {
    pub text: [ScreenChar; 2],
    pub hl_id: ::core::ffi::c_int,
}
/// One `:sign define` entry: what a placement copies when it is made.
///
/// Owns its strings; the highlight groups are the ids `syn_check_group`
/// answered for them, as upstream keeps them.
#[derive(Clone, Default)]
pub struct Sign {
    pub name: ::std::ffi::CString,
    /// The `icon=` path, backslashes already halved; never drawn.
    pub icon: Option<Vec<u8>>,
    pub text: [ScreenChar; 2],
    pub line_hl: ::core::ffi::c_int,
    pub text_hl: ::core::ffi::c_int,
    pub cul_hl: ::core::ffi::c_int,
    pub num_hl: ::core::ffi::c_int,
    /// `-1` when the definition names none -- or still 0, when a fresh
    /// definition's `text=` failed before the priority was reached.
    pub priority: ::core::ffi::c_int,
}
