//! Plumbing behind the API: argument conversion, validation, and the
//! generated dispatch tables that route a method name to its handler.

pub mod converter;
pub mod dispatch;
pub mod dispatch_wrappers;
pub mod helpers;
pub mod metadata;
pub mod validate;
