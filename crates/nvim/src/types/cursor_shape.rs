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

pub type CursorShape = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct CursorEntry {
    pub full_name: &'static ::core::ffi::CStr,
    pub shape: CursorShape,
    pub mshape: ::core::ffi::c_int,
    pub percentage: ::core::ffi::c_int,
    pub blinkwait: ::core::ffi::c_int,
    pub blinkon: ::core::ffi::c_int,
    pub blinkoff: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
    pub id_lm: ::core::ffi::c_int,
    pub name: &'static ::core::ffi::CStr,
    pub used_for: ::core::ffi::c_char,
}

impl CursorEntry {
    /// An entry with every number zero and both names empty: what a
    /// terminal UI holds for a mode until the server describes it.
    pub const BLANK: Self = Self {
        full_name: c"",
        shape: 0,
        mshape: 0,
        percentage: 0,
        blinkwait: 0,
        blinkon: 0,
        blinkoff: 0,
        id: 0,
        id_lm: 0,
        name: c"",
        used_for: 0,
    };

    /// A terminal UI's table before the server has described any mode.
    pub const BLANK_TABLE: [Self; 18] = [Self::BLANK; 18];
}
