//! The built-in terminal UI: it renders the grid the editor publishes and
//! feeds the keys and mouse events it reads back in.
//!
//! Every file in this subtree spells its `unsafe` operations out, which is
//! what the deny below asserts -- it propagates into all of them. There is
//! no `forbid(unsafe_code)` here: the TUI owns raw grids, libuv streams and
//! the terminfo database's C strings.

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
pub mod terminfo;
pub mod termkey;
pub mod tui;
pub mod ugrid;
pub mod unibi;
