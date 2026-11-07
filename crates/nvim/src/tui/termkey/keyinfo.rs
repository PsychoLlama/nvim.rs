#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// libtermkey's internal `keyinfo`.
#![allow(non_camel_case_types)]

//! libtermkey's internal `keyinfo`: what a key table maps a sequence to.

// These are libtermkey's types, Copyright (c) 2007-2011 Paul Evans, under
// the MIT license; the notice is reproduced in
// licenses/libtermkey-LICENSE.txt.

use crate::tui::termkey::types::{TermKeySym, TermKeyType};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct keyinfo {
    pub kind: TermKeyType,
    pub sym: TermKeySym,
    pub modifier_mask: ::core::ffi::c_int,
    pub modifier_set: ::core::ffi::c_int,
}
