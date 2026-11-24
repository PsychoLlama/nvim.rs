#![forbid(unsafe_code)]
//! The msgpack-RPC transport: channels, the server that accepts them, and
//! the packer/unpacker pair that moves API values over the wire.

pub mod channel;
pub mod handler;
pub mod packer;
pub mod server;
pub mod unpacker;
