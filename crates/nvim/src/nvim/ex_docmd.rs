//! The Ex command dispatcher: the command table, and the shared
//! vocabulary its twenty-one children are written against.
//!
//! `cmdnames` is the whole of `:` — 557 rows, in the order `ex_cmds.lua`
//! lists them, with `CMD_*` as indices into it. Nothing else lives here:
//! the parsing is under `scan`, `address`, `modifier` and `lookup`, the
//! driving under `onecmd`, `cmdline` and `source`, and one file per family
//! of `ex_*` handler.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::arglist::{
    ex_all, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind,
};
use crate::src::nvim::autocmd::ex_doautoall;
use crate::src::nvim::buffer::{buflist_list, ex_buffer_all};
use crate::src::nvim::cmdhist::ex_history;
use crate::src::nvim::debugger::{
    ex_breakadd, ex_breakdel, ex_breaklist, ex_debug, ex_debuggreedy,
};
use crate::src::nvim::diff::{
    ex_diffgetput, ex_diffoff, ex_diffpatch, ex_diffsplit, ex_diffthis, ex_diffupdate,
};
use crate::src::nvim::digraph::ex_loadkeymap;
use crate::src::nvim::eval::userfunc::{ex_call, ex_delfunction, ex_function, ex_return};
use crate::src::nvim::eval::vars::{ex_let, ex_lockvar, ex_unlet};
use crate::src::nvim::eval::{ex_echo, ex_echohl, ex_execute};
use crate::src::nvim::ex_cmds::{
    do_ascii, do_wqall, ex_align, ex_append, ex_change, ex_file, ex_global, ex_oldfiles, ex_sort,
    ex_substitute, ex_substitute_preview, ex_uniq, ex_update, ex_wnext, ex_write, ex_z,
};
use crate::src::nvim::ex_cmds2::{
    ex_checktime, ex_compiler, ex_drop, ex_listdo, ex_perl, ex_perldo, ex_perlfile, ex_py3file,
    ex_pydo3, ex_python3, ex_ruby, ex_rubydo, ex_rubyfile,
};
use crate::src::nvim::ex_eval::{
    ex_break, ex_catch, ex_continue, ex_else, ex_endfunction, ex_endif, ex_endtry, ex_endwhile,
    ex_eval, ex_finally, ex_if, ex_throw, ex_try, ex_while,
};
use crate::src::nvim::ex_getln::getexline;
use crate::src::nvim::ex_session::{ex_loadview, ex_mkrc};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::help::{ex_exusage, ex_help, ex_helpclose, ex_helptags, ex_viusage};
use crate::src::nvim::indent::ex_retab;
use crate::src::nvim::lua::executor::{ex_lua, ex_luado, ex_luafile};
use crate::src::nvim::lua::secure::ex_trust;
use crate::src::nvim::main::c_bytes;
use crate::src::nvim::main::{
    e_backslash, e_invrange, e_line_number_out_of_range, e_no_errors, e_norange, e_zerocount,
    searchcmdlen,
};
use crate::src::nvim::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::src::nvim::mark::{ex_changes, ex_clearjumps, ex_delmarks, ex_jumps, ex_marks};
use crate::src::nvim::r#match::ex_match;
use crate::src::nvim::menu::{ex_emenu, ex_menu, ex_menutranslate};
use crate::src::nvim::message::ex_messages;
use crate::src::nvim::option::ex_set;
use crate::src::nvim::os::lang::ex_language;
use crate::src::nvim::profile::ex_profile;
use crate::src::nvim::quickfix::{
    ex_cbelow, ex_cbottom, ex_cbuffer, ex_cc, ex_cclose, ex_cexpr, ex_cfile, ex_cnext, ex_copen,
    ex_cwindow, ex_helpgrep, ex_make, ex_vimgrep, qf_age, qf_history, qf_list,
};
use crate::src::nvim::register::ex_display;
use crate::src::nvim::runtime::{
    ex_finish, ex_options, ex_packadd, ex_packloadall, ex_runtime, ex_scriptencoding,
    ex_scriptnames, ex_source,
};
use crate::src::nvim::sign::ex_sign;
use crate::src::nvim::spell::{ex_spelldump, ex_spellinfo, ex_spellrepall};
use crate::src::nvim::spellfile::{ex_mkspell, ex_spell};
use crate::src::nvim::syntax::{ex_ownsyntax, ex_syntax, ex_syntime};
use crate::src::nvim::tag::do_tags;
use crate::src::nvim::types::{
    BoolVarValue, CMD_index, Callback, Callback_data as C2Rust_Unnamed_20, CdCause, CdScope,
    ChannelPart, Direction, LineGetter, LuaRetMode, MarkGet, MotionType, OptValType, RemapValues,
    TriState, VarLockStatus, VarType, VimVarIndex, cmd_addr_T, dobuf_action_values,
    dobuf_start_values, estack_arg_T, etype_T, exarg_T, except_T, garray_T, handle_T, key_extra,
    linenr_T, optmagic_T, uint8_t, uint16_t, uint32_t,
};
use crate::src::nvim::undo::{ex_undojoin, ex_undolist};
use crate::src::nvim::usercmd::{ex_comclear, ex_command, ex_delcommand};
use crate::src::nvim::version::{ex_intro, ex_version};
use core::ffi::{c_char, c_int, c_uint, c_void};

mod cmdline;
pub use self::cmdline::*;
mod source;
pub use self::source::*;
mod onecmd;
pub use self::onecmd::*;
mod api;
pub use self::api::*;
mod modifier;
pub use self::modifier::*;
mod address;
pub use self::address::*;
mod scan;
pub use self::scan::*;
mod lookup;
pub use self::lookup::*;
mod verify;
pub use self::verify::*;
mod filename;
pub use self::filename::*;
mod argopt;
pub use self::argopt::*;
mod quit;
pub use self::quit::*;
mod restart;
pub(crate) use self::restart::*;
mod window;
pub use self::window::*;
mod file;
pub use self::file::*;
mod path;
pub use self::path::*;
mod edit;
pub use self::edit::*;
mod display;
pub use self::display::*;
mod tags;
pub(crate) use self::tags::*;
mod filetype;
pub use self::filetype::*;
use crate::src::nvim::eval::typval::kCallbackNone;
mod childproc;
pub(crate) use self::childproc::*;
pub const kTrue: TriState = 1;
pub const kFalse: TriState = 0;
pub const kNone: TriState = -1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BOOL: VarType = 7;
pub const VAR_LIST: VarType = 4;
pub const VAR_STRING: VarType = 2;
pub const VAR_UNKNOWN: VarType = 0;
pub type C2Rust_Unnamed_31 = c_uint;
pub const MAXCOL: C2Rust_Unnamed_31 = 2147483647;
pub const kDirectionNotSet: Direction = 0;
pub const kCdScopeGlobal: CdScope = 2;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdCauseManual: CdCause = 0;
pub type C2Rust_Unnamed_33 = c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_33 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_33 = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub type C2Rust_Unnamed_35 = c_uint;
pub const CSF_CAUGHT: C2Rust_Unnamed_35 = 4096;
pub const CSF_THROWN: C2Rust_Unnamed_35 = 2048;
pub const CSF_FINALLY: C2Rust_Unnamed_35 = 512;
pub const CSF_TRY: C2Rust_Unnamed_35 = 256;
pub const CSF_FOR: C2Rust_Unnamed_35 = 16;
pub const CSF_WHILE: C2Rust_Unnamed_35 = 8;
pub const CSF_ACTIVE: C2Rust_Unnamed_35 = 2;
pub const CSF_TRUE: C2Rust_Unnamed_35 = 1;
pub type C2Rust_Unnamed_36 = c_uint;
pub const CSTP_THROW: C2Rust_Unnamed_36 = 4;
pub const CSTP_INTERRUPT: C2Rust_Unnamed_36 = 2;
pub const CSTP_ERROR: C2Rust_Unnamed_36 = 1;
pub type C2Rust_Unnamed_37 = c_uint;
pub const CSL_HAD_FINA: C2Rust_Unnamed_37 = 8;
pub const CSL_HAD_CONT: C2Rust_Unnamed_37 = 4;
pub const CSL_HAD_ENDLOOP: C2Rust_Unnamed_37 = 2;
pub const CSL_HAD_LOOP: C2Rust_Unnamed_37 = 1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_yank: CMD_index = 546;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_while: CMD_index = 525;
pub const CMD_write: CMD_index = 522;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_try: CMD_index = 488;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_throw: CMD_index = 473;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_sview: CMD_index = 442;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_split: CMD_index = 420;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_return: CMD_index = 371;
pub const CMD_redir: CMD_index = 363;
pub const CMD_read: CMD_index = 360;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3: CMD_index = 349;
pub const CMD_python: CMD_index = 346;
pub const CMD_put: CMD_index = 344;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_perl: CMD_index = 323;
pub const CMD_print: CMD_index = 318;
pub const CMD_only: CMD_index = 311;
pub const CMD_number: CMD_index = 304;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_new: CMD_index = 291;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_match: CMD_index = 278;
pub const CMD_make: CMD_index = 274;
pub const CMD_move: CMD_index = 272;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_hide: CMD_index = 181;
pub const CMD_help: CMD_index = 176;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_global: CMD_index = 170;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_finally: CMD_index = 159;
pub const CMD_filter: CMD_index = 157;
pub const CMD_file: CMD_index = 154;
pub const CMD_execute: CMD_index = 151;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endif: CMD_index = 143;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delete: CMD_index = 109;
pub const CMD_const: CMD_index = 99;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_close: CMD_index = 79;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_cc: CMD_index = 59;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_browse: CMD_index = 38;
pub const CMD_botright: CMD_index = 31;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_append: CMD_index = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
/// A command handler. Plain `unsafe fn`, not `extern "C"`: nothing
/// outside this crate calls the table.
pub type ex_func_T = Option<unsafe fn(*mut exarg_T)>;
/// An 'inccommand' preview callback, likewise.
pub type ex_preview_func_T = Option<unsafe fn(*mut exarg_T, c_int, handle_T) -> c_int>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandDefinition {
    pub cmd_name: *mut c_char,
    pub cmd_func: ex_func_T,
    pub cmd_preview_func: ex_preview_func_T,
    pub cmd_argt: uint32_t,
    pub cmd_addr_type: cmd_addr_T,
}
pub type C2Rust_Unnamed_38 = c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_38 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_38 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_38 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_38 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_38 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_38 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_38 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_38 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_38 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_38 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_38 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_38 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_38 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_38 = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kMTLineWise: MotionType = 1;
pub type C2Rust_Unnamed_42 = c_uint;
pub const PUT_LINE: C2Rust_Unnamed_42 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_42 = 4;
pub const PUT_FIXINDENT: C2Rust_Unnamed_42 = 1;
pub type C2Rust_Unnamed_43 = c_uint;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_43 = 2;
pub type C2Rust_Unnamed_44 = c_uint;
pub const WILD_NOERROR: C2Rust_Unnamed_44 = 2048;
pub const WILD_ICASE: C2Rust_Unnamed_44 = 256;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_44 = 16;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_46 = c_uint;
pub const BL_FIX: C2Rust_Unnamed_46 = 4;
pub const BL_SOL: C2Rust_Unnamed_46 = 2;
pub const BL_WHITE: C2Rust_Unnamed_46 = 1;
pub type C2Rust_Unnamed_47 = c_uint;
pub const VIM_QUESTION: C2Rust_Unnamed_47 = 4;
pub type C2Rust_Unnamed_48 = c_uint;
pub const VIM_YES: C2Rust_Unnamed_48 = 2;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub type C2Rust_Unnamed_49 = c_uint;
pub const ECMD_ALTBUF: C2Rust_Unnamed_49 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_49 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_49 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_49 = 4;
pub const ECMD_HIDE: C2Rust_Unnamed_49 = 1;
pub type C2Rust_Unnamed_50 = c_int;
pub const ECMD_ONE: C2Rust_Unnamed_50 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_50 = -1;
pub type C2Rust_Unnamed_51 = c_uint;
pub const CCGD_EXCMD: C2Rust_Unnamed_51 = 16;
pub const CCGD_FORCEIT: C2Rust_Unnamed_51 = 4;
pub const CCGD_MULTWIN: C2Rust_Unnamed_51 = 2;
pub const CCGD_AW: C2Rust_Unnamed_51 = 1;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_52 = c_uint;
pub const DOCMD_KEEPLINE: C2Rust_Unnamed_52 = 32;
pub const DOCMD_EXCRESET: C2Rust_Unnamed_52 = 16;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_52 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_52 = 4;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_52 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_52 = 1;
pub type C2Rust_Unnamed_53 = c_uint;
pub const VALID_HEAD: C2Rust_Unnamed_53 = 2;
pub const VALID_PATH: C2Rust_Unnamed_53 = 1;
pub type C2Rust_Unnamed_54 = c_uint;
pub const DIALOG_MSG_SIZE: C2Rust_Unnamed_54 = 1000;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dbg_stuff {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub caught_stack: *mut except_T,
    pub vv_exception: *mut c_char,
    pub vv_throwpoint: *mut c_char,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: bool,
    pub need_rethrow: c_int,
    pub check_cstack: c_int,
    pub current_exception: *mut except_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_cookie {
    pub lines_gap: *mut garray_T,
    pub current_line: c_int,
    pub repeating: c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wcmd_T {
    pub line: *mut c_char,
    pub lnum: linenr_T,
}
pub const ETYPE_EXCEPT: etype_T = 5;
pub const OP_LSHIFT: C2Rust_Unnamed_67 = 4;
pub const OP_RSHIFT: C2Rust_Unnamed_67 = 5;
pub const OP_YANK: C2Rust_Unnamed_67 = 2;
pub const OP_DELETE: C2Rust_Unnamed_67 = 1;
pub const WSP_VERT: C2Rust_Unnamed_66 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_56 = 1;
pub const DT_LTAG: C2Rust_Unnamed_65 = 11;
pub const DT_TAG: C2Rust_Unnamed_65 = 1;
pub const DT_LAST: C2Rust_Unnamed_65 = 6;
pub const DT_FIRST: C2Rust_Unnamed_65 = 5;
pub const DT_POP: C2Rust_Unnamed_65 = 2;
pub const DT_NEXT: C2Rust_Unnamed_65 = 3;
pub const DT_PREV: C2Rust_Unnamed_65 = 4;
pub const DT_SELECT: C2Rust_Unnamed_65 = 7;
pub const DT_JUMP: C2Rust_Unnamed_65 = 9;
pub const KE_IGNORE: key_extra = 53;
pub const OPT_LOCAL: C2Rust_Unnamed_59 = 2;
pub const KE_XF2: key_extra = 58;
pub const KE_XF1: key_extra = 57;
pub const FIND_ANY: C2Rust_Unnamed_61 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_61 = 2;
pub const ACTION_SPLIT: C2Rust_Unnamed_62 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_62 = 2;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_62 = 4;
pub const ACTION_SHOW: C2Rust_Unnamed_62 = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const DIP_ALL: C2Rust_Unnamed_60 = 1;
pub const CHECK_PATH: C2Rust_Unnamed_61 = 3;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const FNAME_HYP: C2Rust_Unnamed_56 = 4;
pub const FIND_STRING: C2Rust_Unnamed_58 = 2;
pub const FIND_EVAL: C2Rust_Unnamed_58 = 4;
pub const FIND_IDENT: C2Rust_Unnamed_58 = 1;
pub const WSP_TOP: C2Rust_Unnamed_66 = 8;
pub const WSP_BELOW: C2Rust_Unnamed_66 = 64;
pub const WSP_ABOVE: C2Rust_Unnamed_66 = 128;
pub const WSP_HOR: C2Rust_Unnamed_66 = 4;
pub const WSP_BOT: C2Rust_Unnamed_66 = 16;
pub const OPT_GLOBAL: C2Rust_Unnamed_59 = 1;
pub type C2Rust_Unnamed_56 = c_uint;
pub type C2Rust_Unnamed_58 = c_uint;
pub type C2Rust_Unnamed_59 = c_uint;
pub type C2Rust_Unnamed_60 = c_uint;
pub type C2Rust_Unnamed_61 = c_uint;
pub type C2Rust_Unnamed_62 = c_uint;
pub type C2Rust_Unnamed_65 = c_uint;
pub type C2Rust_Unnamed_66 = c_uint;
pub type C2Rust_Unnamed_67 = c_uint;
pub const INT32_MAX: c_int = 2147483647 as c_int;
pub const NULL_1: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const EXIT_FAILURE: c_int = 1 as c_int;
pub const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub const BF_DUMMY: c_int = 0x80 as c_int;
pub const ML_EMPTY: c_int = 0x1 as c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as c_int,
    ga_maxlen: 0 as c_int,
    ga_itemsize: 0 as c_int,
    ga_growsize: 1 as c_int,
    ga_data: NULL_1,
};
pub const EX_RANGE: c_uint = 0x1 as c_uint;
pub const EX_BANG: c_uint = 0x2 as c_uint;
pub const EX_EXTRA: c_uint = 0x4 as c_uint;
pub const EX_XFILE: c_uint = 0x8 as c_uint;
pub const EX_NOSPC: c_uint = 0x10 as c_uint;
pub const EX_DFLALL: c_uint = 0x20 as c_uint;
pub const EX_WHOLEFOLD: c_uint = 0x40 as c_uint;
pub const EX_NEEDARG: c_uint = 0x80 as c_uint;
pub const EX_TRLBAR: c_uint = 0x100 as c_uint;
pub const EX_REGSTR: c_uint = 0x200 as c_uint;
pub const EX_COUNT: c_uint = 0x400 as c_uint;
pub const EX_NOTRLCOM: c_uint = 0x800 as c_uint;
pub const EX_ZEROR: c_uint = 0x1000 as c_uint;
pub const EX_CTRLV: c_uint = 0x2000 as c_uint;
pub const EX_CMDARG: c_uint = 0x4000 as c_uint;
pub const EX_BUFNAME: c_uint = 0x8000 as c_uint;
pub const EX_BUFUNL: c_uint = 0x10000 as c_uint;
pub const EX_ARGOPT: c_uint = 0x20000 as c_uint;
pub const EX_SBOXOK: c_uint = 0x40000 as c_uint;
pub const EX_CMDWIN: c_uint = 0x80000 as c_uint;
pub const EX_MODIFY: c_uint = 0x100000 as c_uint;
pub const EX_FLAGS: c_uint = 0x200000 as c_uint;
pub const EX_LOCK_OK: c_uint = 0x1000000 as c_uint;
pub const EX_PREVIEW: c_uint = 0x8000000 as c_uint;
pub const BAD_KEEP: c_int = -1 as c_int;
pub const BAD_DROP: c_int = -2 as c_int;
pub const FORCE_BIN: c_int = 1 as c_int;
pub const FORCE_NOBIN: c_int = 2 as c_int;
pub const EXFLAG_LIST: c_int = 0x1 as c_int;
pub const EXFLAG_NR: c_int = 0x2 as c_int;
pub const EXFLAG_PRINT: c_int = 0x4 as c_int;
pub const NUL: c_int = '\0' as c_int;
pub const Ctrl_C: c_int = 3 as c_int;
pub const Ctrl_G: c_int = 7 as c_int;
pub const Ctrl_O: c_int = 15 as c_int;
pub const Ctrl_V: c_int = 22 as c_int;
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const CPO_ALTREAD: c_int = 'a' as c_int;
pub const CPO_BAR: c_int = 'b' as c_int;
pub const CPO_EXECBUF: c_int = 'e' as c_int;
pub const CPO_NOSYMLINKS: c_int = '~' as c_int;
static e_ambiguous_use_of_user_defined_command: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E464: Ambiguous use of user-defined command\0"));
static e_no_call_stack_to_substitute_for_stack: GlobalCell<[c_char; 48]> = GlobalCell::new(
    c_bytes(b"E489: No call stack to substitute for \"<stack>\"\0"),
);
static e_not_an_editor_command: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E492: Not an editor command\0"));
static e_no_autocommand_file_name_to_substitute_for_afile: GlobalCell<[c_char; 59]> =
    GlobalCell::new(c_bytes(
        b"E495: No autocommand file name to substitute for \"<afile>\"\0",
    ));
static e_no_autocommand_buffer_number_to_substitute_for_abuf: GlobalCell<[c_char; 62]> =
    GlobalCell::new(c_bytes(
        b"E496: No autocommand buffer number to substitute for \"<abuf>\"\0",
    ));
static e_no_autocommand_match_name_to_substitute_for_amatch: GlobalCell<[c_char; 61]> =
    GlobalCell::new(c_bytes(
        b"E497: No autocommand match name to substitute for \"<amatch>\"\0",
    ));
static e_no_source_file_name_to_substitute_for_sfile: GlobalCell<[c_char; 55]> = GlobalCell::new(
    c_bytes(b"E498: No :source file name to substitute for \"<sfile>\"\0"),
);
static e_no_line_number_to_use_for_slnum: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E842: No line number to use for \"<slnum>\"\0"));
static e_no_line_number_to_use_for_sflnum: GlobalCell<[c_char; 43]> =
    GlobalCell::new(c_bytes(b"E961: No line number to use for \"<sflnum>\"\0"));
static e_no_script_file_name_to_substitute_for_script: GlobalCell<[c_char; 56]> = GlobalCell::new(
    c_bytes(b"E1274: No script file name to substitute for \"<script>\"\0"),
);
static quitmore: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static ex_pressedreturn: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static dollar_command: GlobalCell<[c_char; 2]> = GlobalCell::new(['$' as c_char, 0 as c_char]);
static cmdline_call_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static ex_error_buf: GlobalCell<[c_char; 480]> = GlobalCell::new([0; 480]);
static exmode_plus: GlobalCell<[c_char; 2]> = GlobalCell::new(c_bytes(b"+\0"));
static ffu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: C2Rust_Unnamed_20 {
        funcref: ::core::ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
});
static prev_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static filetype_detect: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_plugin: GlobalCell<TriState> = GlobalCell::new(kNone);
static filetype_indent: GlobalCell<TriState> = GlobalCell::new(kNone);
pub const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub const MSG_BUF_LEN: c_int = 480 as c_int;
pub const FILETYPE_FILE: [c_char; 26] = c_bytes(b"filetype.lua filetype.vim\0");
pub const FTPLUGIN_FILE: [c_char; 13] = c_bytes(b"ftplugin.vim\0");
pub const INDENT_FILE: [c_char; 11] = c_bytes(b"indent.vim\0");
pub const FTOFF_FILE: [c_char; 10] = c_bytes(b"ftoff.vim\0");
pub const FTPLUGOF_FILE: [c_char; 13] = c_bytes(b"ftplugof.vim\0");
pub const INDOFF_FILE: [c_char; 11] = c_bytes(b"indoff.vim\0");
pub const PROF_YES: c_int = 1 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const KS_SPECIAL: c_int = 254 as c_int;
pub const KE_FILLER: c_int = 'X' as c_int;
static command_count: GlobalCell<c_int> = GlobalCell::new(557 as c_int);
/// One row of the Ex command table, spelled the way `ex_cmds.lua` spells it.
///
/// c2rust wrote each of these as a twelve-line struct literal whose
/// `cmd_func` went through a transmute from `ex_func_T` to `ex_func_T` --
/// a no-op that cost three `unsafe ` tokens a row.
const fn cmd<const N: usize>(
    name: &'static [u8; N],
    func: unsafe fn(*mut exarg_T),
    argt: c_uint,
    addr: cmd_addr_T,
) -> CommandDefinition {
    CommandDefinition {
        cmd_name: name.as_ptr() as *mut c_char,
        cmd_func: Some(func),
        cmd_preview_func: None,
        cmd_argt: argt as uint32_t,
        cmd_addr_type: addr,
    }
}

/// A row whose command also has a 'inccommand' preview implementation.
const fn cmd_pv<const N: usize>(
    name: &'static [u8; N],
    func: unsafe fn(*mut exarg_T),
    preview: unsafe fn(*mut exarg_T, c_int, handle_T) -> c_int,
    argt: c_uint,
    addr: cmd_addr_T,
) -> CommandDefinition {
    CommandDefinition {
        cmd_name: name.as_ptr() as *mut c_char,
        cmd_func: Some(func),
        cmd_preview_func: Some(preview),
        cmd_argt: argt as uint32_t,
        cmd_addr_type: addr,
    }
}

/// Every Ex command, in the order `ex_cmds.lua` lists them.
///
/// **The order is load-bearing.** `CMD_*` are indices into this array, and
/// `cmdidxs1`/`cmdidxs2` are precomputed offsets into it that
/// `find_ex_command` uses to skip straight to a first/second letter. Adding,
/// removing or moving a row means regenerating all three.
#[rustfmt::skip]
static cmdnames: GlobalCell<[CommandDefinition; 557]> = GlobalCell::new([
    cmd(b"append\0", ex_append, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"abbreviate\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"abclear\0", ex_abclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"aboveleft\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"all\0", ex_all, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"amenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"anoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"args\0", ex_args, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"argadd\0", ex_argadd, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_ZEROR, ADDR_ARGUMENTS),
    cmd(b"argdelete\0", ex_argdelete, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR, ADDR_ARGUMENTS),
    cmd(b"argdo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_ARGUMENTS),
    cmd(b"argdedupe\0", ex_argdedupe, EX_TRLBAR, ADDR_NONE),
    cmd(b"argedit\0", ex_argedit, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_ZEROR | EX_CMDARG | EX_ARGOPT, ADDR_ARGUMENTS),
    cmd(b"argglobal\0", ex_args, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"arglocal\0", ex_args, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"argument\0", ex_argument, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_ARGUMENTS),
    cmd(b"ascii\0", do_ascii, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"autocmd\0", ex_autocmd, EX_BANG | EX_EXTRA | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"augroup\0", ex_autocmd, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"aunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"buffer\0", ex_buffer, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_BUFNAME | EX_BUFUNL, ADDR_BUFFERS),
    cmd(b"bNext\0", ex_bprevious, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"ball\0", ex_buffer_all, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"badd\0", ex_edit, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"balt\0", ex_edit, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"bdelete\0", ex_bunload, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_BUFNAME, ADDR_BUFFERS),
    cmd(b"belowright\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"bfirst\0", ex_brewind, EX_RANGE | EX_BANG | EX_TRLBAR | EX_CMDARG, ADDR_OTHER),
    cmd(b"blast\0", ex_blast, EX_RANGE | EX_BANG | EX_TRLBAR | EX_CMDARG, ADDR_OTHER),
    cmd(b"bmodified\0", ex_bmodified, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"bnext\0", ex_bnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"botright\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"bprevious\0", ex_bprevious, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"brewind\0", ex_brewind, EX_RANGE | EX_BANG | EX_TRLBAR | EX_CMDARG, ADDR_OTHER),
    cmd(b"break\0", ex_break, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"breakadd\0", ex_breakadd, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"breakdel\0", ex_breakdel, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"breaklist\0", ex_breaklist, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"browse\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"buffers\0", buflist_list, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"bufdo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_BUFFERS),
    cmd(b"bunload\0", ex_bunload, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_BUFNAME, ADDR_LOADED_BUFFERS),
    cmd(b"bwipeout\0", ex_bunload, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_BUFNAME | EX_BUFUNL, ADDR_BUFFERS),
    cmd(b"change\0", ex_change, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"cNext\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cNfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cabclear\0", ex_abclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cabove\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"caddbuffer\0", ex_cbuffer, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"caddexpr\0", ex_cexpr, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"caddfile\0", ex_cfile, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"cafter\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"call\0", ex_call, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"catch\0", ex_catch, EX_EXTRA | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cbuffer\0", ex_cbuffer, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"cbefore\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cbelow\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cbottom\0", ex_cbottom, EX_TRLBAR, ADDR_NONE),
    cmd(b"cc\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_QUICKFIX),
    cmd(b"cclose\0", ex_cclose, EX_TRLBAR, ADDR_NONE),
    cmd(b"cd\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cdo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_QUICKFIX_VALID),
    cmd(b"center\0", ex_align, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"cexpr\0", ex_cexpr, EX_BANG | EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"cfile\0", ex_cfile, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"cfdo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_QUICKFIX_VALID),
    cmd(b"cfirst\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cgetfile\0", ex_cfile, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"cgetbuffer\0", ex_cbuffer, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"cgetexpr\0", ex_cexpr, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"chdir\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"changes\0", ex_changes, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"checkhealth\0", ex_checkhealth, EX_EXTRA | EX_TRLBAR, ADDR_NONE),
    cmd(b"checkpath\0", ex_checkpath, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"checktime\0", ex_checktime, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_BUFNAME, ADDR_OTHER),
    cmd(b"chistory\0", qf_history, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"clist\0", qf_list, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"clast\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"close\0", ex_close, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_WINDOWS),
    cmd(b"clearjumps\0", ex_clearjumps, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"cnext\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cnewer\0", qf_age, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cnfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cnoreabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cnoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"copy\0", ex_copymove, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"colder\0", qf_age, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"colorscheme\0", ex_colorscheme, EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"command\0", ex_command, EX_BANG | EX_EXTRA | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"comclear\0", ex_comclear, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"compiler\0", ex_compiler, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"continue\0", ex_continue, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"confirm\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"connect\0", ex_connect, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"const\0", ex_let, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"copen\0", ex_copen, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"cprevious\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cpfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"cquit\0", ex_cquit, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_ZEROR, ADDR_UNSIGNED),
    cmd(b"crewind\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"cunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cunabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"cwindow\0", ex_cwindow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"delete\0", ex_operators, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_REGSTR | EX_COUNT | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"delmarks\0", ex_delmarks, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"debug\0", ex_debug, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"debuggreedy\0", ex_debuggreedy, EX_RANGE | EX_TRLBAR | EX_ZEROR | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"defer\0", ex_call, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"delcommand\0", ex_delcommand, EX_BANG | EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"delfunction\0", ex_delfunction, EX_BANG | EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"detach\0", ex_detach, EX_TRLBAR, ADDR_NONE),
    cmd(b"display\0", ex_display, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"diffupdate\0", ex_diffupdate, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"diffget\0", ex_diffgetput, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_ZEROR | EX_MODIFY, ADDR_LINES),
    cmd(b"diffoff\0", ex_diffoff, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"diffpatch\0", ex_diffpatch, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_MODIFY, ADDR_NONE),
    cmd(b"diffput\0", ex_diffgetput, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_ZEROR, ADDR_LINES),
    cmd(b"diffsplit\0", ex_diffsplit, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"diffthis\0", ex_diffthis, EX_TRLBAR, ADDR_NONE),
    cmd(b"digraphs\0", ex_digraphs, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"djump\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD, ADDR_LINES),
    cmd(b"dlist\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"doautocmd\0", ex_doautocmd, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"doautoall\0", ex_doautoall, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"drop\0", ex_drop, EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"dsearch\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"dsplit\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD, ADDR_LINES),
    cmd(b"edit\0", ex_edit, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"earlier\0", ex_later, EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"echo\0", ex_echo, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"echoerr\0", ex_execute, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"echohl\0", ex_echohl, EX_EXTRA | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"echomsg\0", ex_execute, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"echon\0", ex_echo, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"else\0", ex_else, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"elseif\0", ex_else, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"emenu\0", ex_emenu, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"endif\0", ex_endif, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"endfunction\0", ex_endfunction, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"endfor\0", ex_endwhile, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"endtry\0", ex_endtry, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"endwhile\0", ex_endwhile, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"enew\0", ex_edit, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"eval\0", ex_eval, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"ex\0", ex_edit, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"execute\0", ex_execute, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"exit\0", ex_exit, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"exusage\0", ex_exusage, EX_TRLBAR, ADDR_NONE),
    cmd(b"file\0", ex_file, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"files\0", buflist_list, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"filetype\0", ex_filetype, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"filter\0", ex_wrongmodifier, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"find\0", ex_find, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"finally\0", ex_finally, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"finish\0", ex_finish, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"first\0", ex_rewind, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"fold\0", ex_fold, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"foldclose\0", ex_foldopen, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"folddoopen\0", ex_folddo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_LINES),
    cmd(b"folddoclosed\0", ex_folddo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_LINES),
    cmd(b"foldopen\0", ex_foldopen, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"for\0", ex_while, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"function\0", ex_function, EX_BANG | EX_EXTRA | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"fclose\0", ex_fclose, EX_RANGE | EX_BANG | EX_TRLBAR, ADDR_OTHER),
    cmd(b"global\0", ex_global, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"goto\0", ex_goto, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"grep\0", ex_make, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM, ADDR_OTHER),
    cmd(b"grepadd\0", ex_make, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM, ADDR_OTHER),
    cmd(b"gui\0", ex_nogui, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"gvim\0", ex_nogui, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"help\0", ex_help, EX_BANG | EX_EXTRA | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"helpclose\0", ex_helpclose, EX_TRLBAR, ADDR_NONE),
    cmd(b"helpgrep\0", ex_helpgrep, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"helptags\0", ex_helptags, EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"highlight\0", ex_highlight, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"hide\0", ex_hide, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT, ADDR_WINDOWS),
    cmd(b"history\0", ex_history, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"horizontal\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"insert\0", ex_append, EX_RANGE | EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"iabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"iabclear\0", ex_abclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"if\0", ex_if, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"ijump\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD, ADDR_LINES),
    cmd(b"ilist\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"imap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"imapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"imenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"inoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"inoreabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"inoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"intro\0", ex_intro, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"iput\0", ex_iput, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_REGSTR | EX_ZEROR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"isearch\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"isplit\0", ex_findpat, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD, ADDR_LINES),
    cmd(b"iunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"iunabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"iunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"join\0", ex_join, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_MODIFY | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"jumps\0", ex_jumps, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"k\0", ex_mark, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"keepmarks\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"keepjumps\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"keeppatterns\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"keepalt\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"list\0", ex_print, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"lNext\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lNfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"last\0", ex_last, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"labove\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"language\0", ex_language, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"laddexpr\0", ex_cexpr, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"laddbuffer\0", ex_cbuffer, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"laddfile\0", ex_cfile, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"lafter\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"later\0", ex_later, EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lbuffer\0", ex_cbuffer, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"lbefore\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lbelow\0", ex_cbelow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lbottom\0", ex_cbottom, EX_TRLBAR, ADDR_NONE),
    cmd(b"lcd\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lchdir\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lclose\0", ex_cclose, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"ldo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_QUICKFIX_VALID),
    cmd(b"left\0", ex_align, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"leftabove\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"let\0", ex_let, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lexpr\0", ex_cexpr, EX_BANG | EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"lfile\0", ex_cfile, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"lfdo\0", ex_listdo, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_QUICKFIX_VALID),
    cmd(b"lfirst\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lgetfile\0", ex_cfile, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"lgetbuffer\0", ex_cbuffer, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_LINES),
    cmd(b"lgetexpr\0", ex_cexpr, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"lgrep\0", ex_make, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM, ADDR_OTHER),
    cmd(b"lgrepadd\0", ex_make, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM, ADDR_OTHER),
    cmd(b"lhelpgrep\0", ex_helpgrep, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"lhistory\0", qf_history, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"ll\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_QUICKFIX),
    cmd(b"llast\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"llist\0", qf_list, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lmake\0", ex_make, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"lnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lnext\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lnewer\0", qf_age, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lnfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"loadview\0", ex_loadview, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"loadkeymap\0", ex_loadkeymap, EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lockmarks\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"lockvar\0", ex_lockvar, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lolder\0", qf_age, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lopen\0", ex_copen, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"lprevious\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"lpfile\0", ex_cnext, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"lrewind\0", ex_cc, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_UNSIGNED),
    cmd(b"ltag\0", ex_tag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"lunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lua\0", ex_lua, EX_RANGE | EX_EXTRA | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"luado\0", ex_luado, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"luafile\0", ex_luafile, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"lvimgrep\0", ex_vimgrep, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"lvimgrepadd\0", ex_vimgrep, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"lwindow\0", ex_cwindow, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"ls\0", buflist_list, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"lsp\0", ex_lsp, EX_EXTRA | EX_NEEDARG, ADDR_NONE),
    cmd(b"move\0", ex_copymove, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"mark\0", ex_mark, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"make\0", ex_make, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"map\0", ex_map, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"mapclear\0", ex_mapclear, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"marks\0", ex_marks, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"match\0", ex_match, EX_RANGE | EX_EXTRA | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"menu\0", ex_menu, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"menutranslate\0", ex_menutranslate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"messages\0", ex_messages, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"mkexrc\0", ex_mkrc, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"mksession\0", ex_mkrc, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"mkspell\0", ex_mkspell, EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"mkvimrc\0", ex_mkrc, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"mkview\0", ex_mkrc, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"mode\0", ex_mode, EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"mzscheme\0", ex_script_ni, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"mzfile\0", ex_ni, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"next\0", ex_next, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"new\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"nmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"nmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"nmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"nnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"nnoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"noremap\0", ex_map, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"noautocmd\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"nohlsearch\0", ex_nohlsearch, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"noreabbrev\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"noremenu\0", ex_menu, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"noswapfile\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"normal\0", ex_normal, EX_RANGE | EX_BANG | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_CTRLV | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"number\0", ex_print, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"nunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"nunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"oldfiles\0", ex_oldfiles, EX_BANG | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"omap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"omapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"omenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"only\0", ex_only, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_WINDOWS),
    cmd(b"onoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"onoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"options\0", ex_options, EX_TRLBAR, ADDR_NONE),
    cmd(b"ounmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"ounmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"ownsyntax\0", ex_ownsyntax, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"print\0", ex_print, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_SBOXOK | EX_CMDWIN | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"packadd\0", ex_packadd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"packloadall\0", ex_packloadall, EX_BANG | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"pbuffer\0", ex_pbuffer, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_BUFNAME | EX_BUFUNL, ADDR_BUFFERS),
    cmd(b"pclose\0", ex_pclose, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"perl\0", ex_perl, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"perldo\0", ex_perldo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"perlfile\0", ex_perlfile, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pedit\0", ex_pedit, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"pop\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_ZEROR, ADDR_OTHER),
    cmd(b"popup\0", ex_popup, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"ppop\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_ZEROR, ADDR_OTHER),
    cmd(b"preserve\0", ex_preserve, EX_TRLBAR, ADDR_NONE),
    cmd(b"previous\0", ex_previous, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"profile\0", ex_profile, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"profdel\0", ex_breakdel, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"psearch\0", ex_psearch, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD, ADDR_LINES),
    cmd(b"ptag\0", ex_ptag, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptNext\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptfirst\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptjump\0", ex_ptag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"ptlast\0", ex_ptag, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"ptnext\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptprevious\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptrewind\0", ex_ptag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"ptselect\0", ex_ptag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"put\0", ex_put, EX_RANGE | EX_BANG | EX_WHOLEFOLD | EX_TRLBAR | EX_REGSTR | EX_ZEROR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pwd\0", ex_pwd, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"python\0", ex_python3, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pydo\0", ex_pydo3, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pyfile\0", ex_py3file, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"py3\0", ex_python3, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"py3do\0", ex_pydo3, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"python3\0", ex_python3, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"py3file\0", ex_py3file, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pyx\0", ex_python3, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pyxdo\0", ex_pydo3, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pythonx\0", ex_python3, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"pyxfile\0", ex_py3file, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"quit\0", ex_quit, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_WINDOWS),
    cmd(b"quitall\0", ex_quitall, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"qall\0", ex_quitall, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"read\0", ex_read, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_WHOLEFOLD | EX_TRLBAR | EX_ZEROR | EX_ARGOPT | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"recover\0", ex_recover, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"redo\0", ex_redo, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"redir\0", ex_redir, EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"redraw\0", ex_redraw, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"redrawstatus\0", ex_redrawstatus, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"redrawtabline\0", ex_redrawtabline, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"registers\0", ex_display, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"resize\0", ex_resize, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"restart\0", ex_restart, EX_EXTRA | EX_NOTRLCOM | EX_CMDARG, ADDR_NONE),
    cmd(b"retab\0", ex_retab, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"return\0", ex_return, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"rewind\0", ex_rewind, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"right\0", ex_align, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"rightbelow\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"rshada\0", ex_shada, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"runtime\0", ex_runtime, EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"rundo\0", ex_rundo, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG, ADDR_NONE),
    cmd(b"ruby\0", ex_ruby, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"rubydo\0", ex_rubydo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"rubyfile\0", ex_rubyfile, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"rviminfo\0", ex_shada, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd_pv(b"substitute\0", ex_substitute, ex_substitute_preview, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK | EX_PREVIEW, ADDR_LINES),
    cmd(b"sNext\0", ex_previous, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"sargument\0", ex_argument, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_ARGUMENTS),
    cmd(b"sall\0", ex_all, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"sandbox\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"saveas\0", ex_write, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sbuffer\0", ex_buffer, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_BUFNAME | EX_BUFUNL, ADDR_BUFFERS),
    cmd(b"sbNext\0", ex_bprevious, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"sball\0", ex_buffer_all, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"sbfirst\0", ex_brewind, EX_TRLBAR | EX_CMDARG, ADDR_NONE),
    cmd(b"sblast\0", ex_blast, EX_TRLBAR | EX_CMDARG, ADDR_NONE),
    cmd(b"sbmodified\0", ex_bmodified, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"sbnext\0", ex_bnext, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"sbprevious\0", ex_bprevious, EX_RANGE | EX_TRLBAR | EX_COUNT | EX_CMDARG, ADDR_OTHER),
    cmd(b"sbrewind\0", ex_brewind, EX_TRLBAR | EX_CMDARG, ADDR_NONE),
    cmd(b"scriptnames\0", ex_scriptnames, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"scriptencoding\0", ex_scriptencoding, EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"set\0", ex_set, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"setfiletype\0", ex_setfiletype, EX_EXTRA | EX_NEEDARG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"setglobal\0", ex_set, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"setlocal\0", ex_set, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sfind\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"sfirst\0", ex_rewind, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"simalt\0", ex_ni, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sign\0", ex_sign, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"silent\0", ex_wrongmodifier, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sleep\0", ex_sleep, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"slast\0", ex_last, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd_pv(b"smagic\0", ex_submagic, ex_submagic_preview, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK | EX_PREVIEW, ADDR_LINES),
    cmd(b"smap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"smapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"smenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"snext\0", ex_next, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd_pv(b"snomagic\0", ex_submagic, ex_submagic_preview, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK | EX_PREVIEW, ADDR_LINES),
    cmd(b"snoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"snoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"source\0", ex_source, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"sort\0", ex_sort, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_NOTRLCOM | EX_MODIFY, ADDR_LINES),
    cmd(b"split\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"spellgood\0", ex_spell, EX_RANGE | EX_BANG | EX_EXTRA | EX_NEEDARG | EX_TRLBAR, ADDR_OTHER),
    cmd(b"spelldump\0", ex_spelldump, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"spellinfo\0", ex_spellinfo, EX_TRLBAR, ADDR_NONE),
    cmd(b"spellrepall\0", ex_spellrepall, EX_TRLBAR, ADDR_NONE),
    cmd(b"spellrare\0", ex_spell, EX_RANGE | EX_BANG | EX_EXTRA | EX_NEEDARG | EX_TRLBAR, ADDR_OTHER),
    cmd(b"spellundo\0", ex_spell, EX_RANGE | EX_BANG | EX_EXTRA | EX_NEEDARG | EX_TRLBAR, ADDR_OTHER),
    cmd(b"spellwrong\0", ex_spell, EX_RANGE | EX_BANG | EX_EXTRA | EX_NEEDARG | EX_TRLBAR, ADDR_OTHER),
    cmd(b"sprevious\0", ex_previous, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"srewind\0", ex_rewind, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"stop\0", ex_stop, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"stag\0", ex_stag, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"startinsert\0", ex_startinsert, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"startgreplace\0", ex_startinsert, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"startreplace\0", ex_startinsert, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"stopinsert\0", ex_stopinsert, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"stjump\0", ex_stag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"stselect\0", ex_stag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"sunhide\0", ex_buffer_all, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"sunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"suspend\0", ex_stop, EX_BANG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"sview\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"swapname\0", ex_swapname, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"syntax\0", ex_syntax, EX_EXTRA | EX_NOTRLCOM | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"syntime\0", ex_syntime, EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"syncbind\0", ex_syncbind, EX_TRLBAR, ADDR_NONE),
    cmd(b"t\0", ex_copymove, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"tcd\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tchdir\0", ex_cd, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tNext\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"tag\0", ex_tag, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"tags\0", do_tags, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tab\0", ex_wrongmodifier, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_ZEROR, ADDR_TABS),
    cmd(b"tabclose\0", ex_tabclose, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR | EX_CMDWIN | EX_LOCK_OK, ADDR_TABS),
    cmd(b"tabdo\0", ex_listdo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_TABS),
    cmd(b"tabedit\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ZEROR | EX_CMDARG | EX_ARGOPT, ADDR_TABS),
    cmd(b"tabfind\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_TRLBAR | EX_ZEROR | EX_CMDARG | EX_ARGOPT, ADDR_TABS),
    cmd(b"tabfirst\0", ex_tabnext, EX_TRLBAR, ADDR_NONE),
    cmd(b"tabmove\0", ex_tabmove, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_TABS),
    cmd(b"tablast\0", ex_tabnext, EX_TRLBAR, ADDR_NONE),
    cmd(b"tabnext\0", ex_tabnext, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_TABS),
    cmd(b"tabnew\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ZEROR | EX_CMDARG | EX_ARGOPT, ADDR_TABS),
    cmd(b"tabonly\0", ex_tabonly, EX_RANGE | EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR | EX_CMDWIN | EX_LOCK_OK, ADDR_TABS),
    cmd(b"tabprevious\0", ex_tabnext, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_TABS_RELATIVE),
    cmd(b"tabNext\0", ex_tabnext, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_TRLBAR | EX_ZEROR, ADDR_TABS_RELATIVE),
    cmd(b"tabrewind\0", ex_tabnext, EX_TRLBAR, ADDR_NONE),
    cmd(b"tabs\0", ex_tabs, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tcl\0", ex_script_ni, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"tcldo\0", ex_ni, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"tclfile\0", ex_ni, EX_RANGE | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"terminal\0", ex_terminal, EX_BANG | EX_EXTRA | EX_XFILE | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tfirst\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"throw\0", ex_throw, EX_EXTRA | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tjump\0", ex_tag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"tlast\0", ex_tag, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"tlmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"tlnoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"tlunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"tmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tnext\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"tnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"topleft\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"tprevious\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"trewind\0", ex_tag, EX_RANGE | EX_BANG | EX_TRLBAR | EX_ZEROR, ADDR_OTHER),
    cmd(b"trust\0", ex_trust, EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_LOCK_OK, ADDR_NONE),
    cmd(b"try\0", ex_try, EX_TRLBAR | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tselect\0", ex_tag, EX_BANG | EX_EXTRA | EX_NOSPC | EX_TRLBAR, ADDR_NONE),
    cmd(b"tunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"tunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"undo\0", ex_undo, EX_RANGE | EX_BANG | EX_TRLBAR | EX_COUNT | EX_ZEROR | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"undojoin\0", ex_undojoin, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"undolist\0", ex_undolist, EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unabbreviate\0", ex_abbreviate, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unhide\0", ex_buffer_all, EX_RANGE | EX_TRLBAR | EX_COUNT, ADDR_OTHER),
    cmd(b"uniq\0", ex_uniq, EX_RANGE | EX_BANG | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_NOTRLCOM | EX_MODIFY, ADDR_LINES),
    cmd(b"unlet\0", ex_unlet, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unlockvar\0", ex_lockvar, EX_BANG | EX_EXTRA | EX_NEEDARG | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unmap\0", ex_unmap, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unmenu\0", ex_menu, EX_BANG | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"unsilent\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"update\0", ex_update, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT, ADDR_LINES),
    cmd(b"vglobal\0", ex_global, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"version\0", ex_version, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"verbose\0", ex_wrongmodifier, EX_RANGE | EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"vertical\0", ex_wrongmodifier, EX_EXTRA | EX_NEEDARG | EX_NOTRLCOM, ADDR_NONE),
    cmd(b"visual\0", ex_edit, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"view\0", ex_edit, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_NONE),
    cmd(b"vimgrep\0", ex_vimgrep, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"vimgrepadd\0", ex_vimgrep, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NEEDARG | EX_TRLBAR | EX_NOTRLCOM | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"viusage\0", ex_viusage, EX_TRLBAR, ADDR_NONE),
    cmd(b"vmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"vmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"vmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"vnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"vnew\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"vnoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"vsplit\0", ex_splitview, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
    cmd(b"vunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"vunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"write\0", ex_write, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"wNext\0", ex_wnext, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT, ADDR_OTHER),
    cmd(b"wall\0", do_wqall, EX_BANG | EX_TRLBAR | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"while\0", ex_while, EX_EXTRA | EX_NOTRLCOM | EX_SBOXOK | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"winsize\0", ex_winsize, EX_EXTRA | EX_NEEDARG | EX_TRLBAR, ADDR_NONE),
    cmd(b"wincmd\0", ex_wincmd, EX_RANGE | EX_EXTRA | EX_NOSPC | EX_NEEDARG | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"windo\0", ex_listdo, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_NEEDARG | EX_NOTRLCOM, ADDR_WINDOWS),
    cmd(b"winpos\0", ex_ni, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"wnext\0", ex_wnext, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ARGOPT, ADDR_OTHER),
    cmd(b"wprevious\0", ex_wnext, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ARGOPT, ADDR_OTHER),
    cmd(b"wq\0", ex_exit, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT, ADDR_LINES),
    cmd(b"wqall\0", do_wqall, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_ARGOPT, ADDR_NONE),
    cmd(b"wshada\0", ex_shada, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"wundo\0", ex_wundo, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_NEEDARG, ADDR_NONE),
    cmd(b"wviminfo\0", ex_shada, EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"xit\0", ex_exit, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_NOSPC | EX_DFLALL | EX_WHOLEFOLD | EX_TRLBAR | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"xall\0", do_wqall, EX_BANG | EX_TRLBAR, ADDR_NONE),
    cmd(b"xmap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"xmapclear\0", ex_mapclear, EX_EXTRA | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"xmenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"xnoremap\0", ex_map, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"xnoremenu\0", ex_menu, EX_RANGE | EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_ZEROR | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_OTHER),
    cmd(b"xunmap\0", ex_unmap, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"xunmenu\0", ex_menu, EX_EXTRA | EX_TRLBAR | EX_NOTRLCOM | EX_CTRLV | EX_CMDWIN | EX_LOCK_OK, ADDR_NONE),
    cmd(b"yank\0", ex_operators, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_REGSTR | EX_COUNT | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"z\0", ex_z, EX_RANGE | EX_BANG | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"!\0", ex_bang, EX_RANGE | EX_BANG | EX_EXTRA | EX_XFILE | EX_WHOLEFOLD | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"#\0", ex_print, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"&\0", ex_substitute, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"<\0", ex_operators, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_MODIFY | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"=\0", ex_equal, EX_RANGE | EX_EXTRA | EX_DFLALL | EX_ARGOPT | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b">\0", ex_operators, EX_RANGE | EX_WHOLEFOLD | EX_TRLBAR | EX_COUNT | EX_CMDWIN | EX_MODIFY | EX_FLAGS | EX_LOCK_OK, ADDR_LINES),
    cmd(b"@\0", ex_at, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_TRLBAR | EX_CMDWIN | EX_LOCK_OK, ADDR_LINES),
    cmd(b"~\0", ex_substitute, EX_RANGE | EX_EXTRA | EX_WHOLEFOLD | EX_CMDWIN | EX_MODIFY | EX_LOCK_OK, ADDR_LINES),
    cmd(b"Next\0", ex_previous, EX_RANGE | EX_BANG | EX_EXTRA | EX_TRLBAR | EX_COUNT | EX_CMDARG | EX_ARGOPT, ADDR_OTHER),
]);
/// For each letter a-z, the index of the first command in `cmdnames`
/// that starts with it. Generated; regenerate with the table.
static cmdidxs1: GlobalCell<[uint16_t; 26]> = GlobalCell::new([
    0, 20, 43, 109, 133, 154, 170, 176, 184, 203, 205, 210, 272, 290, 307, 318, 357, 360, 382, 447,
    492, 504, 522, 537, 546, 547,
]);
/// For each pair of letters, the offset from `cmdidxs1` of the first
/// command starting with them. Generated; regenerate with the table.
static cmdidxs2: GlobalCell<[[uint8_t; 26]; 26]> = GlobalCell::new([
    [
        0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 5, 6, 0, 0, 0, 7, 16, 0, 17, 0, 0, 0, 0, 0,
    ],
    [
        2, 0, 0, 5, 6, 7, 0, 0, 0, 0, 0, 8, 9, 10, 11, 12, 0, 13, 0, 0, 0, 0, 22, 0, 0, 0,
    ],
    [
        3, 12, 16, 18, 20, 22, 25, 0, 0, 0, 0, 34, 38, 41, 47, 58, 60, 61, 0, 0, 62, 0, 65, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 8, 17, 0, 18, 0, 0, 19, 0, 0, 21, 22, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 7, 9, 10, 0, 0, 0, 0, 0, 0, 0, 16, 0, 17, 0, 0,
    ],
    [
        0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 4, 5, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 5, 6, 0, 0, 13, 0, 0, 14, 0, 16, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        3, 11, 15, 18, 19, 23, 26, 31, 0, 0, 0, 33, 36, 39, 43, 49, 0, 51, 60, 52, 53, 57, 59, 0,
        0, 0,
    ],
    [
        1, 0, 0, 0, 7, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 5, 7, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 0, 7, 0, 0, 0, 0, 8, 0, 10, 0, 0, 0,
    ],
    [
        1, 3, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 11, 0, 0, 16, 17, 26, 0, 27, 0, 28, 0,
    ],
    [
        2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 0, 16, 21, 0, 0, 0, 0,
    ],
    [
        2, 6, 15, 0, 17, 21, 0, 0, 23, 0, 0, 26, 28, 32, 36, 38, 0, 47, 0, 48, 0, 60, 61, 0, 62, 0,
    ],
    [
        4, 0, 1, 0, 24, 25, 0, 26, 0, 27, 0, 28, 32, 35, 37, 38, 0, 39, 42, 0, 43, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 9, 12, 0, 0, 0, 0, 15, 0, 16, 0, 0, 0, 0, 0,
    ],
    [
        2, 0, 0, 0, 0, 0, 0, 3, 4, 0, 0, 0, 0, 8, 0, 9, 10, 0, 12, 0, 13, 14, 0, 0, 0, 0,
    ],
    [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 5, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
]);
pub const false_0: c_int = 0 as c_int;
pub const RE_MAGIC: c_int = 1 as c_int;
