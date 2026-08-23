//! Ports of the pure-logic `test/unit` specs. Like the LuaJIT FFI harness
//! they replaced, these call the crate's `extern "C"` surface directly —
//! no editor state, no child process. Specs that need a live editor
//! (`early_init`) stay in `test/unit`.

mod support;

mod arena;
mod arglist;
mod charset;
mod cmdhist;
mod cursor;
mod digraph;
mod expressions;
mod fold;
mod fpconv;
mod garray;
mod hashtab;
mod keycodes;
mod linematch;
mod map;
mod marktree;
mod memory;
mod r#move;
mod multiqueue;
mod packer;
mod parser;
mod profile;
mod sha256;
mod shell;
mod strings;
mod terminfo;
mod termkey;
mod typval;
mod undo;
mod unpacker;
mod users;
mod window;
