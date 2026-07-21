// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Context {
    pub regs: String_0,
    pub jumps: String_0,
    pub bufs: String_0,
    pub gvars: String_0,
    pub funcs: Array,
}
