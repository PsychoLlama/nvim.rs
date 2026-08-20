//! Plumbing behind the API: argument conversion, validation, and the
//! generated dispatch tables that route a method name to its handler.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod converter;
pub mod dispatch;
pub mod dispatch_wrappers;
pub mod helpers;
pub mod metadata;
pub mod validate;
