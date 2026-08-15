#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct Context {
    pub regs: String_0,
    pub jumps: String_0,
    pub bufs: String_0,
    pub gvars: String_0,
    pub funcs: Array,
}
