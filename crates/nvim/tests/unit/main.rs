//! Ports of the pure-logic `test/unit` specs. Like the LuaJIT FFI harness
//! they replaced, these call the crate's `extern "C"` surface directly —
//! no editor state, no child process. Specs that need a live editor
//! (`early_init`) stay in `test/unit`.

mod support;

mod api_converter;
mod arena;
mod arglist;
mod buffer;
mod channel_reader;
mod charset;
mod cmdhist;
mod cursor;
mod digraph;
mod env;
mod eval_decode;
mod eval_encode;
mod expressions;
mod fileio;
mod fileio_names;
mod fold;
mod fpconv;
mod fs;
mod fuzzy;
mod garray;
mod hashtab;
mod indent;
mod keycodes;
mod linematch;
mod log;
mod map;
mod marktree;
mod mbyte;
mod memline;
mod memory;
mod message;
mod r#move;
mod msgpack;
mod multiqueue;
mod namespace;
mod optionstr;
mod packer;
mod parser;
mod path;
mod profile;
mod regexp;
mod search;
mod sha256;
mod shell;
mod spellfile;
mod statusline;
mod strings;
mod tempfile;
mod terminfo;
mod termkey;
mod typval;
mod typval_dict;
mod typval_list;
mod typval_value;
mod undo;
mod unpacker;
mod users;
mod window;
