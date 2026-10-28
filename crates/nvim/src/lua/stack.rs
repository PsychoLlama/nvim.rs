//! The typed `Stack` over a `lua_State`: the one module that pushes, pops,
//! reads and indexes LuaJIT's value stack.
//!
//! Empty today. It will hold the `Stack` handle (built once per entry from
//! the raw `lua_State` the interpreter hands a C function), `At` slot
//! indices with the balance arithmetic in the type, `push`/`pop`/`get`
//! over the editor's values, the `luaL_check*` family as `Result`s, table
//! and registry access, protected calls, and the single error path that
//! raises a Lua error once every Rust-owned value is dropped. Every raw
//! stack call elsewhere in the tree moves in here; `lua_raw_stack` counts
//! the ones still outside this file and `ffi.rs`.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
