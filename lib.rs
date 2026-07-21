#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(raw_ref_op)]
#![feature(strict_provenance)]
#![feature(thread_local)]

extern crate c2rust_bitfields;
extern crate libc;

pub mod src {
    pub mod cjson {
        pub mod fpconv;
        pub mod lua_cjson;
        pub mod strbuf;
    } // mod cjson
    pub mod mpack {
        pub mod conv;
        pub mod lmpack;
        pub mod mpack_core;
        pub mod object;
        pub mod rpc;
    } // mod mpack
    pub mod nvim {
        #[path = "eval.rs"]
        pub mod eval_1;
        pub mod api {
            pub mod autocmd;
            pub mod buffer;
            pub mod command;
            pub mod deprecated;
            pub mod events;
            pub mod extmark;
            pub mod options;
            pub mod private {
                pub mod converter;
                pub mod dispatch;
                pub mod helpers;
                pub mod validate;
            } // mod private
            pub mod tabpage;
            pub mod ui;
            pub mod vim;
            pub mod vimscript;
            pub mod win_config;
            pub mod window;
        } // mod api
        pub mod allocator;
        pub mod arabic;
        pub mod arglist;
        pub mod autocmd;
        pub mod base64;
        pub mod buffer;
        pub mod buffer_updates;
        pub mod bufwrite;
        pub mod change;
        pub mod channel;
        pub mod charset;
        pub mod clipboard;
        pub mod cmdexpand;
        pub mod cmdhist;
        pub mod context;
        pub mod cursor;
        pub mod cursor_shape;
        pub mod debugger;
        pub mod decoration;
        pub mod decoration_provider;
        pub mod diff;
        pub mod digraph;
        pub mod drawline;
        pub mod drawscreen;
        pub mod edit;
        pub mod eval {
            pub mod buffer;
            pub mod decode;
            pub mod deprecated;
            pub mod encode;
            pub mod executor;
            pub mod fs;
            pub mod funcs;
            pub mod gc;
            pub mod list;
            pub mod typval;
            pub mod userfunc;
            pub mod vars;
            pub mod window;
        } // mod eval
        pub mod event {
            pub mod libuv_proc;
            pub mod r#loop;
            pub mod multiqueue;
            pub mod proc;
            pub mod rstream;
            pub mod signal;
            pub mod socket;
            pub mod stream;
            pub mod time;
            pub mod wstream;
        } // mod event
        pub mod ex_cmds;
        pub mod ex_cmds2;
        pub mod ex_docmd;
        pub mod ex_eval;
        pub mod ex_getln;
        pub mod ex_session;
        pub mod extmark;
        pub mod file_search;
        pub mod fileio;
        pub mod fold;
        pub mod fuzzy;
        pub mod garray;
        pub mod getchar;
        pub mod global_cell;
        pub mod grid;
        pub mod hashtab;
        pub mod help;
        pub mod highlight;
        pub mod highlight_group;
        pub mod indent;
        pub mod indent_c;
        pub mod input;
        pub mod insexpand;
        pub mod keycodes;
        pub mod linematch;
        pub mod log;
        pub mod lua {
            pub mod api_wrappers;
            pub mod base64;
            pub mod converter;
            pub mod executor;
            pub mod secure;
            pub mod spell;
            pub mod stdlib;
            pub mod treesitter;
            pub mod xdiff;
        } // mod lua
        pub mod main;
        pub mod map;
        pub mod map_glyph_cache;
        pub mod mapping;
        pub mod mark;
        pub mod marktree;
        pub mod math;
        pub mod mbyte;
        pub mod memfile;
        pub mod memline;
        pub mod memory;
        pub mod menu;
        pub mod message;
        pub mod mouse;
        pub mod msgpack_rpc {
            pub mod channel;
            pub mod packer;
            pub mod server;
            pub mod unpacker;
        } // mod msgpack_rpc
        pub mod normal;
        pub mod ops;
        pub mod option;
        pub mod optionstr;
        pub mod os {
            pub mod dl;
            pub mod env;
            pub mod fileio;
            pub mod fs;
            pub mod input;
            pub mod lang;
            pub mod mem;
            pub mod proc;
            pub mod pty_proc_unix;
            pub mod shell;
            pub mod signal;
            pub mod stdpaths;
            pub mod time;
            pub mod users;
        } // mod os
        pub mod r#match;
        pub mod r#move;
        pub mod path;
        pub mod plines;
        pub mod popupmenu;
        pub mod profile;
        pub mod quickfix;
        pub mod regexp;
        pub mod register;
        pub mod runtime;
        pub mod search;
        pub mod sha256;
        pub mod shada;
        pub mod sign;
        pub mod spell;
        pub mod spellfile;
        pub mod spellsuggest;
        pub mod state;
        pub mod statusline;
        pub mod strings;
        pub mod syntax;
        pub mod tag;
        pub mod terminal;
        pub mod testing;
        pub mod textformat;
        pub mod textobject;
        pub mod tui {
            pub mod input;
            pub mod terminfo;
            pub mod termkey {
                pub mod driver_csi;
                pub mod driver_ti;
                pub mod termkey;
            } // mod termkey
            pub mod tui;
            pub mod ugrid;
        } // mod tui
        pub mod types;
        pub mod ui;
        pub mod ui_client;
        pub mod ui_compositor;
        pub mod undo;
        pub mod usercmd;
        pub mod version;
        pub mod viml {
            pub mod parser {
                pub mod expressions;
                pub mod parser;
            } // mod parser
        } // mod viml
        pub mod vterm {
            pub mod encoding;
            pub mod keyboard;
            pub mod mouse;
            pub mod parser;
            pub mod pen;
            pub mod screen;
            pub mod state;
            pub mod vterm;
        } // mod vterm
        pub mod window;
        pub mod winfloat;
    } // mod nvim
    pub mod xdiff {
        pub mod xdiffi;
        pub mod xemit;
        pub mod xhistogram;
        pub mod xpatience;
        pub mod xprepare;
        pub mod xutils;
    } // mod xdiff
} // mod src
