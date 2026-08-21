//! The operating-system layer: filesystem, environment, processes, signals,
//! terminals and clocks, mostly over libuv.
//!
//! Every file in this subtree spells its `unsafe` operations out, which is
//! what the deny below asserts -- it propagates into all of them. There is no
//! `forbid(unsafe_code)` here and never will be: the layer's whole job is to
//! call libc and libuv.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod cshim;
pub mod dl;
pub mod env;
pub mod fileio;
pub mod fs;
pub mod input;
pub mod lang;
pub mod proc;
pub mod pty_proc_unix;
pub mod shell;
pub mod signal;
pub mod stdpaths;
pub mod time;
pub mod users;
pub mod uv_error;
