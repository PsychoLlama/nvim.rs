#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// libmpack's typedefs, named as the codec names them.
#![allow(non_camel_case_types)]

//! libmpack's integer typedefs: the widest integers a token carries. The
//! token's type tag is [`crate::mpack::token::Kind`].

pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
pub type mpack_uint32_t = ::core::ffi::c_uint;
