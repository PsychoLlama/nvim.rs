//! The libuv event loop and the streams, processes and timers on it.
//!
//! Every module here is written the same way: the `*mut T` libuv threads
//! through a handle's `data` field is wrapped once — `EventLoop`, `Conn`,
//! `Reader`, `Queue`, `Child`, `Watcher`, `Timer`, `Signal` — so the field
//! accesses beneath are ordinary Rust and the `unsafe` is paid at
//! construction. The `extern "C"` callbacks libuv is handed keep their ABI;
//! they unwrap the pointer and hand a handle to a safe function.
//!
//! Both denies are on this module and so reach every file below it, which is
//! why this file was the last of the batch to take them.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod channel;
pub mod libuv;
pub mod libuv_proc;
pub mod r#loop;
pub mod proc;
pub mod rstream;
pub mod signal;
pub mod socket;
pub mod stream;
pub mod time;
pub mod wstream;
