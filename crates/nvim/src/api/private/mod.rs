//! Plumbing behind the API: argument conversion, validation, and the
//! generated dispatch tables that route a method name to its handler.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod converter;
pub mod dispatch;
pub mod dispatch_wrappers;
pub mod helpers;
pub(crate) mod keyset;
pub mod metadata;
pub mod validate;
