#![forbid(unsafe_code)]

//! libvterm's key and modifier codes.

// These are libvterm's types, Copyright (c) 2008 Paul Evans, under the MIT
// license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

pub type VTermKey = ::core::ffi::c_uint;
pub type VTermModifier = ::core::ffi::c_uint;
