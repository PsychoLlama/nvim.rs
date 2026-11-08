#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The shape of a request handler: what a msgpack-RPC method name resolves to.

use crate::os::libc::uint64_t;
use crate::types::{Array, Error, Object};

/// One generated msgpack-RPC dispatch wrapper: it decodes an argument array
/// against a method's signature, calls the `nvim_*` function and encodes the
/// answer, or answers `Err` with the [`Error`] the client is told about.
/// The arguments are the wrapper's own.
pub type ApiDispatchFn = fn(uint64_t, Array) -> Result<Object, Error>;

/// One row of the API dispatch table.
///
/// `Copy`: a static name and a function pointer.
#[derive(Copy, Clone)]
pub struct MsgpackRpcRequestHandler {
    pub name: Option<&'static ::core::ffi::CStr>,
    pub fn_0: Option<ApiDispatchFn>,
    pub fast: bool,
}
