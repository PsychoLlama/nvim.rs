//! Binary entry point for the c2rust-transpiled neovim.
//!
//! c2rust already emitted a `pub fn main()` in `src/nvim/main.rs` that
//! marshals `std::env::args()` into the `argc`/`argv` the transpiled C
//! `main` expects and calls it. All this shim does is invoke it, turning
//! the library crate into a runnable `nvim` executable without touching
//! the generated sources.
fn main() {
    c2rust_neovim::src::nvim::main::main();
}
