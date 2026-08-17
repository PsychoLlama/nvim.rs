#![forbid(unsafe_code)]

// Canonical definitions, hoisted out of the per-module copies c2rust emitted.
// One definition per logical name; every module imports from here.

/// `insertchar()` flags — upstream's anonymous enum in `edit.h`.
///
/// c2rust typed this `c_uint`, so every use site is `INSCHAR_X as c_int`;
/// retyping belongs to the slice that deletes those casts.
pub type InscharFlags = ::core::ffi::c_uint;

/// force formatting
pub const INSCHAR_FORMAT: InscharFlags = 1;
/// format comments
pub const INSCHAR_DO_COM: InscharFlags = 2;
/// the character was typed just after CTRL-V
pub const INSCHAR_CTRLV: InscharFlags = 4;
/// do not use `'formatexpr'`
pub const INSCHAR_NO_FEX: InscharFlags = 8;
/// format comments with a list or second-line indent
pub const INSCHAR_COM_LIST: InscharFlags = 16;
