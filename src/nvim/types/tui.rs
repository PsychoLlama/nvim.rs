// No forbid(unsafe_code) until the `extern type` below becomes an opaque
// struct: edition 2024 trips the unsafe_code lint on extern blocks.

// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

unsafe extern "C" {
    pub type TUIData;
}

pub type TermMode = ::core::ffi::c_uint;
pub type TermModeState = ::core::ffi::c_uint;
