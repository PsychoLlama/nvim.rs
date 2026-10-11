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
use crate::memory::XString;

/// One argument: the name it was given, owned, and the buffer it was listed
/// as (0 when none).
#[derive(Clone)]
pub struct ArgEntry {
    pub name: XString,
    pub fnum: ::core::ffi::c_int,
}

/// Which argument list a window uses: the key the registry in
/// `arglist::state` finds the list by, and what `arglistid()` answers. Zero is
/// the global list, so a window allocated zeroed starts out on it.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
#[repr(transparent)]
pub struct ArgListId(pub ::core::ffi::c_int);

impl ArgListId {
    /// The global argument list, the one nvim starts with.
    pub const GLOBAL: ArgListId = ArgListId(0);
}

/// An argument list: the global one, or a window-local one shared by
/// reference count between the windows that use it.
pub struct ArgList {
    pub entries: Vec<ArgEntry>,
    /// The windows (and `:all` walks) holding the list. The global list's
    /// count is kept but never consulted: it is never freed.
    pub refcount: Refcount,
}
