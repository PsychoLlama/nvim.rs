//! The built-in terminal UI: it renders the grid the editor publishes and
//! feeds the keys and mouse events it reads back in.
//!
//! Four files here touch libuv -- `tui.rs`, `input.rs`, `output.rs` and
//! `events.rs`, the `tui/` rows of docs/perimeter.md -- and spell their
//! `unsafe` operations out, which is what the deny below asserts. The rest
//! forbid unsafe code. This file cannot: `allow` under a `forbid` is
//! E0453, so it stays bare while those four allow.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod attrs;
pub mod cursor;
pub mod events;
pub mod input;
pub mod keys;
pub mod negotiate;
pub mod output;
pub mod paint;
pub mod quirks;
pub mod state;
pub mod terminfo;
pub mod termkey;
pub mod tui;
pub mod ugrid;
pub mod unibi;
