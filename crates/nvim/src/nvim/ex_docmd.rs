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
    Callback, Callback_data as C2Rust_Unnamed_20, CdCause, ChannelPart, Direction, LineGetter,
    LuaRetMode, MarkGet, MotionType, OptValType, RemapValues, TriState, cmd_addr_T,
    dobuf_action_values, dobuf_start_values, estack_arg_T, etype_T, exarg_T, except_T, garray_T,
    handle_T, kNone, linenr_T, optmagic_T, uint8_t, uint16_t, uint32_t,
};
use crate::src::nvim::undo::{ex_undojoin, ex_undolist};
use crate::src::nvim::usercmd::{ex_comclear, ex_command, ex_delcommand};
use crate::src::nvim::version::{ex_intro, ex_version};
use core::ffi::{c_char, c_int, c_uint, c_void};

// Generated from `ex_cmds.lua`; see `tools/apigen` and `just apigen`.
mod cmdtable;
pub(crate) use self::cmdtable::*;

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
pub const kDirectionNotSet: Direction = 0;
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
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kMTLineWise: MotionType = 1;
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
pub const OPT_LOCAL: C2Rust_Unnamed_59 = 2;
pub const FIND_ANY: C2Rust_Unnamed_61 = 1;
pub const FIND_DEFINE: C2Rust_Unnamed_61 = 2;
pub const ACTION_SPLIT: C2Rust_Unnamed_62 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_62 = 2;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_62 = 4;
pub const ACTION_SHOW: C2Rust_Unnamed_62 = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const CHECK_PATH: C2Rust_Unnamed_61 = 3;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const FNAME_HYP: C2Rust_Unnamed_56 = 4;
pub const FIND_STRING: C2Rust_Unnamed_58 = 2;
pub const FIND_EVAL: C2Rust_Unnamed_58 = 4;
pub const FIND_IDENT: C2Rust_Unnamed_58 = 1;
pub const OPT_GLOBAL: C2Rust_Unnamed_59 = 1;
pub type C2Rust_Unnamed_56 = c_uint;
pub type C2Rust_Unnamed_58 = c_uint;
pub type C2Rust_Unnamed_59 = c_uint;
pub type C2Rust_Unnamed_61 = c_uint;
pub type C2Rust_Unnamed_62 = c_uint;
pub type C2Rust_Unnamed_65 = c_uint;
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
pub const OK: c_int = 1 as c_int;
pub const FAIL: c_int = 0 as c_int;
pub const CPO_ALTREAD: c_int = 'a' as c_int;
pub const CPO_BAR: c_int = 'b' as c_int;
pub const CPO_EXECBUF: c_int = 'e' as c_int;
pub const CPO_NOSYMLINKS: c_int = '~' as c_int;
static e_ambiguous_use_of_user_defined_command: [c_char; 44] =
    c_bytes(b"E464: Ambiguous use of user-defined command\0");
static e_no_call_stack_to_substitute_for_stack: [c_char; 48] =
    c_bytes(b"E489: No call stack to substitute for \"<stack>\"\0");
static e_not_an_editor_command: [c_char; 28] = c_bytes(b"E492: Not an editor command\0");
static e_no_autocommand_file_name_to_substitute_for_afile: [c_char; 59] =
    c_bytes(b"E495: No autocommand file name to substitute for \"<afile>\"\0");
static e_no_autocommand_buffer_number_to_substitute_for_abuf: [c_char; 62] =
    c_bytes(b"E496: No autocommand buffer number to substitute for \"<abuf>\"\0");
static e_no_autocommand_match_name_to_substitute_for_amatch: [c_char; 61] =
    c_bytes(b"E497: No autocommand match name to substitute for \"<amatch>\"\0");
static e_no_source_file_name_to_substitute_for_sfile: [c_char; 55] =
    c_bytes(b"E498: No :source file name to substitute for \"<sfile>\"\0");
static e_no_line_number_to_use_for_slnum: [c_char; 42] =
    c_bytes(b"E842: No line number to use for \"<slnum>\"\0");
static e_no_line_number_to_use_for_sflnum: [c_char; 43] =
    c_bytes(b"E961: No line number to use for \"<sflnum>\"\0");
static e_no_script_file_name_to_substitute_for_script: [c_char; 56] =
    c_bytes(b"E1274: No script file name to substitute for \"<script>\"\0");
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
pub const false_0: c_int = 0 as c_int;
