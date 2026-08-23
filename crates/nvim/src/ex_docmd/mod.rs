//! The Ex command dispatcher: the command table, and the shared
//! vocabulary its twenty-one children are written against.
//!
//! `cmdnames` is the whole of `:` — 557 rows, in the order `ex_cmds.lua`
//! lists them, with `CMD_*` as indices into it. Nothing else lives here:
//! the parsing is under `scan`, `address`, `modifier` and `lookup`, the
//! driving under `onecmd`, `cmdline` and `source`, and one file per family
//! of `ex_*` handler.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::arglist::{
    ex_all, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind,
};
use crate::autocmd::ex_doautoall;
use crate::buffer::{buflist_list, ex_buffer_all};
use crate::cmdhist::ex_history;
use crate::debugger::{ex_breakadd, ex_breakdel, ex_breaklist, ex_debug, ex_debuggreedy};
use crate::diff::{
    ex_diffgetput, ex_diffoff, ex_diffpatch, ex_diffsplit, ex_diffthis, ex_diffupdate,
};
use crate::digraph::ex_loadkeymap;
use crate::eval::userfunc::{ex_call, ex_delfunction, ex_function, ex_return};
use crate::eval::vars::{ex_let, ex_lockvar, ex_unlet};
use crate::eval::{ex_echo, ex_echohl, ex_execute};
use crate::ex_cmds::{
    do_ascii, do_wqall, ex_align, ex_append, ex_change, ex_file, ex_global, ex_oldfiles, ex_sort,
    ex_substitute, ex_substitute_preview, ex_uniq, ex_update, ex_wnext, ex_write, ex_z,
};
use crate::ex_cmds2::{
    ex_checktime, ex_compiler, ex_drop, ex_listdo, ex_perl, ex_perldo, ex_perlfile, ex_py3file,
    ex_pydo3, ex_python3, ex_ruby, ex_rubydo, ex_rubyfile,
};
use crate::ex_eval::{
    ex_break, ex_catch, ex_continue, ex_else, ex_endfunction, ex_endif, ex_endtry, ex_endwhile,
    ex_eval, ex_finally, ex_if, ex_throw, ex_try, ex_while,
};
use crate::ex_getln::getexline;
use crate::ex_session::{ex_loadview, ex_mkrc};
use crate::global_cell::GlobalCell;
use crate::help::{ex_exusage, ex_help, ex_helpclose, ex_helptags, ex_viusage};
use crate::indent::ex_retab;
use crate::lua::executor::{ex_lua, ex_luado, ex_luafile};
use crate::lua::secure::ex_trust;
use crate::main::{
    c_bytes, e_backslash, e_invrange, e_line_number_out_of_range, e_no_errors, e_norange,
    e_zerocount, searchcmdlen,
};
use crate::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::mark::{ex_changes, ex_clearjumps, ex_delmarks, ex_jumps, ex_marks};
use crate::r#match::ex_match;
use crate::menu::{ex_emenu, ex_menu, ex_menutranslate};
use crate::message::ex_messages;
use crate::option::ex_set;
use crate::os::lang::ex_language;
use crate::profile::ex_profile;
use crate::quickfix::{
    ex_cbelow, ex_cbottom, ex_cbuffer, ex_cc, ex_cclose, ex_cexpr, ex_cfile, ex_cnext, ex_copen,
    ex_cwindow, ex_helpgrep, ex_make, ex_vimgrep, qf_age, qf_history, qf_list,
};
use crate::register::ex_display;
use crate::runtime::{
    ex_finish, ex_options, ex_packadd, ex_packloadall, ex_runtime, ex_scriptencoding,
    ex_scriptnames, ex_source,
};
use crate::sign::ex_sign;
use crate::spell::{ex_spelldump, ex_spellinfo, ex_spellrepall};
use crate::spellfile::{ex_mkspell, ex_spell};
use crate::syntax::{ex_ownsyntax, ex_syntax, ex_syntime};
use crate::tag::do_tags;
use crate::types::{
    Callback, Callback_data, CdCause, ChannelPart, CmdAddr, Direction, ExArgt, LineGetter,
    LuaRetMode, MarkGet, MotionType, OptValType, RemapValues, dobuf_action_values,
    dobuf_start_values, estack_arg_T, etype_T, exarg_T, except_T, garray_T, handle_T, linenr_T,
    optmagic_T, uint8_t, uint16_t,
};
use crate::undo::{ex_undojoin, ex_undolist};
use crate::usercmd::{ex_comclear, ex_command, ex_delcommand};
use crate::version::{ex_intro, ex_version};
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
use crate::eval::typval::kCallbackNone;
mod childproc;
pub(crate) use self::childproc::*;
pub const kDirectionNotSet: Direction = 0;
pub const kCdCauseManual: CdCause = 0;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub const CSF_CAUGHT: c_uint = 4096;
pub const CSF_THROWN: c_uint = 2048;
pub const CSF_FINALLY: c_uint = 512;
pub const CSF_TRY: c_uint = 256;
pub const CSF_FOR: c_uint = 16;
pub const CSF_WHILE: c_uint = 8;
pub const CSF_ACTIVE: c_uint = 2;
pub const CSF_TRUE: c_uint = 1;
pub const CSTP_THROW: c_uint = 4;
pub const CSTP_INTERRUPT: c_uint = 2;
pub const CSTP_ERROR: c_uint = 1;
pub const CSL_HAD_FINA: c_uint = 8;
pub const CSL_HAD_CONT: c_uint = 4;
pub const CSL_HAD_ENDLOOP: c_uint = 2;
pub const CSL_HAD_LOOP: c_uint = 1;
/// A command handler. Plain `unsafe fn`, not `extern "C"`: nothing
/// outside this crate calls the table.
pub type ex_func_T = Option<unsafe fn(*mut exarg_T)>;
/// An 'inccommand' preview callback, likewise.
pub type ex_preview_func_T = Option<unsafe fn(*mut exarg_T, c_int, handle_T) -> c_int>;
#[derive(Copy, Clone)]
pub struct CommandDefinition {
    pub cmd_name: *mut c_char,
    pub cmd_func: ex_func_T,
    pub cmd_preview_func: ex_preview_func_T,
    pub cmd_argt: ExArgt,
    pub cmd_addr_type: CmdAddr,
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
pub const VIM_QUESTION: c_uint = 4;
pub const VIM_YES: c_uint = 2;
pub const ECMD_ALTBUF: c_uint = 32;
pub const ECMD_ADDBUF: c_uint = 16;
pub const ECMD_FORCEIT: c_uint = 8;
pub const ECMD_OLDBUF: c_uint = 4;
pub const ECMD_HIDE: c_uint = 1;
pub const ECMD_ONE: c_int = 1;
pub const ECMD_LAST: c_int = -1;
pub const CCGD_EXCMD: c_uint = 16;
pub const CCGD_FORCEIT: c_uint = 4;
pub const CCGD_MULTWIN: c_uint = 2;
pub const CCGD_AW: c_uint = 1;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const VALID_HEAD: c_uint = 2;
pub const VALID_PATH: c_uint = 1;
pub const DIALOG_MSG_SIZE: c_uint = 1000;
#[derive(Copy, Clone)]
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
pub struct loop_cookie {
    pub lines_gap: *mut garray_T,
    pub current_line: c_int,
    pub repeating: c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut c_void,
}
#[derive(Copy, Clone)]
pub struct wcmd_T {
    pub line: *mut c_char,
    pub lnum: linenr_T,
}
pub const ETYPE_EXCEPT: etype_T = 5;
pub const DT_LTAG: c_uint = 11;
pub const DT_TAG: c_uint = 1;
pub const DT_LAST: c_uint = 6;
pub const DT_FIRST: c_uint = 5;
pub const DT_POP: c_uint = 2;
pub const DT_NEXT: c_uint = 3;
pub const DT_PREV: c_uint = 4;
pub const DT_SELECT: c_uint = 7;
pub const DT_JUMP: c_uint = 9;
pub const FIND_ANY: c_uint = 1;
pub const FIND_DEFINE: c_uint = 2;
pub const ACTION_SPLIT: c_uint = 3;
pub const ACTION_GOTO: c_uint = 2;
pub const ACTION_SHOW_ALL: c_uint = 4;
pub const ACTION_SHOW: c_uint = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const CHECK_PATH: c_uint = 3;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const FIND_STRING: c_uint = 2;
pub const FIND_EVAL: c_uint = 4;
pub const FIND_IDENT: c_uint = 1;
pub const INT32_MAX: c_int = 2147483647 as c_int;
pub const NULL_1: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const EXIT_FAILURE: c_int = 1 as c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as c_int,
    ga_maxlen: 0 as c_int,
    ga_itemsize: 0 as c_int,
    ga_growsize: 1 as c_int,
    ga_data: NULL_1,
};
pub const BAD_KEEP: c_int = -1 as c_int;
pub const BAD_DROP: c_int = -2 as c_int;
pub const FORCE_BIN: c_int = 1 as c_int;
pub const FORCE_NOBIN: c_int = 2 as c_int;
pub const EXFLAG_LIST: c_int = 0x1 as c_int;
pub const EXFLAG_NR: c_int = 0x2 as c_int;
pub const EXFLAG_PRINT: c_int = 0x4 as c_int;
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
static ex_pressedreturn: GlobalCell<bool> = GlobalCell::new(false);
static dollar_command: GlobalCell<[c_char; 2]> = GlobalCell::new(['$' as c_char, 0 as c_char]);
static cmdline_call_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static ex_error_buf: GlobalCell<[c_char; 480]> = GlobalCell::new([0; 480]);
static exmode_plus: GlobalCell<[c_char; 2]> = GlobalCell::new(c_bytes(b"+\0"));
static ffu_cb: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
});
static prev_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static filetype_detect: GlobalCell<Option<bool>> = GlobalCell::new(None);
static filetype_plugin: GlobalCell<Option<bool>> = GlobalCell::new(None);
static filetype_indent: GlobalCell<Option<bool>> = GlobalCell::new(None);
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
