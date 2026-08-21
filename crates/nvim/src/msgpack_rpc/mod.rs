#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
//! The msgpack-RPC transport: channels, the server that accepts them, and
//! the packer/unpacker pair that moves API values over the wire.

pub mod channel;
pub mod packer;
pub mod server;
pub mod unpacker;
