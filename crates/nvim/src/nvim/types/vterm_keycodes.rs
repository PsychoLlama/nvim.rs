#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
//
// These are libvterm's types, Copyright (c) 2008 Paul Evans, under the MIT
// license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

pub type VTermKey = ::core::ffi::c_uint;
pub type VTermModifier = ::core::ffi::c_uint;
