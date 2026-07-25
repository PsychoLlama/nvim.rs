//! Ports of the pure-logic `test/unit` specs. Like the LuaJIT FFI harness
//! they replaced, these call the crate's `extern "C"` surface directly —
//! no editor state, no child process. Specs that need a live editor
//! (`early_init`) stay in `test/unit`.

mod support;

mod charset;
mod cmdhist;
mod digraph;
mod garray;
mod keycodes;
mod memory;
mod profile;
mod sha256;
mod strings;
