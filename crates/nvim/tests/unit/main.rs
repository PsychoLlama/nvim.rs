//! Ports of the pure-logic `test/unit` specs. Like the LuaJIT FFI harness
//! they replaced, these call the crate's `extern "C"` surface directly —
//! no editor state, no child process. Specs that need a live editor
//! (`early_init`) stay in `test/unit`.

// The harness calls the editor's `extern "C"` surface through raw pointers --
// that is what a port of the LuaJIT FFI specs is -- so the whole crate needs
// what `crates/nvim/Cargo.toml`'s `[lints.rust]` denies. Taken once at the
// root: `tests/` is not source the ratchet measures, and there is no per-file
// story to tell about it.
#![allow(unsafe_code)]

mod support;

mod api_converter;
mod arena;
mod arglist;
mod buffer;
mod bufwrite;
mod channel_reader;
mod charset;
mod charset_oracle;
mod cmdhist;
mod cursor;
mod digraph;
mod encode_sinks;

mod env;
mod eval_decode;
mod eval_encode;
mod ex_cmds2;
mod ex_docmd;
mod ex_session;
mod expressions;
mod file_search;
mod fileio;
mod fileio_names;
mod filename_oracle;
mod fold;
mod fpconv;
mod fs;
mod fuzzy;
mod garray;
mod getchar_oracle;
mod hashtab;
mod help;
mod help_oracle;
mod indent;
mod keycodes;
mod keycodes_oracle;
mod linematch;
mod log;
mod map;
mod mapping;
mod mapping_oracle;
mod marktree;
mod mbyte;
mod mbyte_oracle;
mod memline;
mod memory;
mod message;
mod message_oracle;
mod r#move;
mod msgpack;
mod multiqueue;
mod namespace;
mod normal;
mod options;
mod options_oracle;
mod optionstr;
mod optionstr_oracle;
mod packer;
mod parser;
mod path;
mod printf_oracle;
mod profile;
mod readfile;
mod regexp;
mod runtime;
mod search;
mod sha256;
mod shada;
mod shell;
mod sign;
mod sign_oracle;
mod spellfile;
mod startup;
mod statusline;
mod strings;
mod strings_oracle;
mod tag;
mod tempfile;
mod terminfo;
mod termkey;
mod typval;
mod typval_dict;
mod typval_list;
mod typval_value;
mod undo;
mod unpacker;
mod usercmd;
mod usercmd_oracle;
mod users;
