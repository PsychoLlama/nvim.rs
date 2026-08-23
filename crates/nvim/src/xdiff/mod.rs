//! The bundled `xdiff` diff engine.

#![deny(unsafe_op_in_unsafe_fn)]

pub(crate) mod ffi;
pub(crate) mod xdiffi;
pub(crate) mod xemit;
pub(crate) mod xhistogram;
pub(crate) mod xpatience;
pub(crate) mod xprepare;
pub(crate) mod xtypes;
pub(crate) mod xutils;
