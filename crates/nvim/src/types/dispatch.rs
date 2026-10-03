#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// One generated msgpack-RPC dispatch wrapper: it decodes an argument array
/// against a method's signature, calls the `nvim_*` function and encodes the
/// answer, or answers `Err` with the [`Error`] the client is told about.
/// The arguments are the wrapper's own; the arena is scratch the handler may
/// allocate in, which the caller releases once the answer is encoded.
pub type ApiDispatchFn = fn(uint64_t, Array, &mut Arena) -> Result<Object, Error>;
