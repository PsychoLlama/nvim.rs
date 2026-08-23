//! Binary entry point for the c2rust-transpiled neovim.
//!
//! c2rust already emitted a `pub fn main()` in `src/main.rs` that
//! marshals `std::env::args()` into the `argc`/`argv` the transpiled C
//! `main` expects and calls it. All this shim does is invoke it, turning
//! the library crate into a runnable `nvim` executable without touching
//! the generated sources.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
fn main() {
    // Arm the GlobalCell debug main-thread assertion before any editor code
    // touches a global.
    neovim::global_cell::init_main_thread();
    neovim::main::main();
}
