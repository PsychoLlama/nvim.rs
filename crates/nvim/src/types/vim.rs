#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type CdCause = ::core::ffi::c_int;
pub type CdScope = ::core::ffi::c_int;
/// Which scope a `:cd` applies to (`getcwd()`/`haslocaldir()` report it).
pub const kCdScopeInvalid: CdScope = -1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeGlobal: CdScope = 2;
pub type Direction = ::core::ffi::c_int;
