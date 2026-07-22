// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

pub type IndentGetter = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
pub type Indenter = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
