#![deny(unsafe_op_in_unsafe_fn)]

// No forbid(unsafe_code) until the `extern type` below becomes an opaque
// struct: edition 2024 trips the unsafe_code lint on extern blocks.

// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

unsafe extern "C" {
    pub type qf_info_S;
}
