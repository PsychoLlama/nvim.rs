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

/// One row of the popup menu, owning its text.
///
/// Upstream's rows point into the strings of whoever built them -- the
/// completion's matches, the command line's expansion, a menu's entries --
/// and the menu has to come down before any of those is freed. Here a row
/// owns copies, so the menu's array (an `Rc<[PumItem]>` shared with the
/// producer) stays good whatever the producer does next. `pum_text` is the
/// one column every row has; the other three are absent rather than empty.
#[derive(Default, Clone)]
pub struct PumItem {
    pub pum_text: crate::memory::XString,
    pub pum_kind: Option<crate::memory::XString>,
    pub pum_extra: Option<crate::memory::XString>,
    pub pum_info: Option<crate::memory::XString>,
    pub pum_cpt_source_idx: ::core::ffi::c_int,
    pub pum_user_abbr_hlattr: ::core::ffi::c_int,
    pub pum_user_kind_hlattr: ::core::ffi::c_int,
}
