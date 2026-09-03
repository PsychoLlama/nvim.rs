#![allow(clippy::missing_safety_doc)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
// `src/main/` is the transpiled `main.c`, not a binary entry point (the real
// one is `src/bin/nvim.rs`). Rust flags any `mod main;` as a likely mistake;
// the lint only listens at the crate root, not on the `mod` item itself.
#![allow(special_module_name)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
// The crate root cannot carry `forbid(unsafe_code)` — `forbid` reaches the
// whole subtree and cannot be lifted by a module, and the tree is still tens
// of thousands of unchecked lines deep. `deny(unsafe_op_in_unsafe_fn)` is the
// strongest marker it can carry, and it is now a *no-op*: every other source
// file in the crate already carries one of the two markers, so nothing is left
// for this to switch on. That is the whole point — it overrides the crate's
// Cargo.toml allow (which still governs the sibling bin/test/bench roots) and
// makes the finished state compiler-enforced rather than a grep result.
#![deny(unsafe_op_in_unsafe_fn)]

pub(crate) mod allocator;
pub mod api;
pub(crate) mod arabic;
pub mod arglist;
pub(crate) mod ascii;
pub mod autocmd;
pub(crate) mod base64;
pub(crate) mod bitfield;
pub mod buffer;
pub mod buffer_updates;
pub mod bufwrite;
pub mod change;
pub mod channel;
pub mod charset;
pub mod cjson;
pub(crate) mod clipboard;
pub mod cmdexpand;
pub mod cmdhist;
pub mod context;
pub(crate) mod cstr;
pub mod cursor;
pub(crate) mod cursor_shape;
pub mod debugger;
pub mod decoration;
pub(crate) mod decoration_provider;
pub mod diff;
pub mod digraph;
pub mod drawline;
pub mod drawscreen;
pub(crate) mod edit;
pub mod eval;
pub mod event;
pub mod ex_cmds;
pub(crate) mod ex_cmds2;
pub mod ex_docmd;
pub(crate) mod ex_eval;
pub mod ex_getln;
pub(crate) mod ex_session;
pub mod extmark;
pub(crate) mod file_search;
pub mod fileio;
pub(crate) mod flags;
pub mod fold;
pub mod fuzzy;
pub mod garray;
pub mod getchar;
pub mod global_cell;
pub mod grid;
pub mod guard;
pub mod hashtab;
pub(crate) mod help;
pub mod highlight;
pub(crate) mod highlight_group;
pub mod indent;
pub mod indent_c;
pub(crate) mod input;
pub mod insexpand;
pub mod keycodes;
pub(crate) mod kvec;
pub mod linematch;
pub mod log;
pub mod lua;
pub mod main;
pub mod map;
pub(crate) mod map_glyph_cache;
pub mod mapping;
pub mod mark;
pub mod marktree;
pub(crate) mod r#match;
pub(crate) mod math;
pub mod mbyte;
pub(crate) mod memfile;
pub mod memline;
pub mod memory;
pub(crate) mod menu;
pub mod message;
pub(crate) mod message_fmt;
pub(crate) mod mouse;
pub mod r#move;
pub mod mpack;
pub mod msgpack_rpc;
pub(crate) mod narrow;
pub(crate) mod normal;
pub mod ops;
pub(crate) mod option;
pub mod options;
pub mod optionstr;
pub mod os;
pub mod path;
pub(crate) mod plines;
pub mod popupmenu;
pub mod pos;
pub mod profile;
pub mod quickfix;
pub mod regexp;
pub mod register;
pub(crate) mod registry;
pub mod runtime;
pub mod search;
pub mod sha256;
pub mod shada;
pub(crate) mod sign;
pub mod spell;
pub mod spellfile;
pub(crate) mod spellsuggest;
pub mod state;
pub mod statusline;
pub mod strings;
pub(crate) mod syntax;
pub mod tag;
pub(crate) mod terminal;
pub(crate) mod testing;
pub mod textformat;
pub mod textobject;
pub mod tui;
pub mod types;
pub mod ui;
pub(crate) mod ui_client;
pub mod ui_compositor;
pub mod undo;
pub(crate) mod usercmd;
pub mod utf8proc;
pub(crate) mod version;
pub mod viml;
pub mod vterm;
pub mod window;
pub(crate) mod winfloat;
pub mod winlayer;
pub(crate) mod xdiff;
