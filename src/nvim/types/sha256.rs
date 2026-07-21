// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct context_sha256_T {
    pub total: [uint32_t; 2],
    pub state: [uint32_t; 8],
    pub buffer: [uint8_t; 64],
}
