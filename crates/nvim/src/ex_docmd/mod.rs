//! The Ex command dispatcher: the command table, and the shared
//! vocabulary its twenty-one children are written against.
//!
//! `cmdnames` is the whole of `:` — 557 rows, in the order `ex_cmds.lua`
//! lists them, with `CMD_*` as indices into it. Nothing else lives here:
//! the parsing is under `scan`, `address`, `modifier` and `lookup`, the
//! driving under `onecmd`, `cmdline` and `source`, and one file per family
//! of `ex_*` handler.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

pub mod state;
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
use crate::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::mark::{ex_changes, ex_clearjumps, ex_delmarks, ex_jumps, ex_marks};
use crate::r#match::ex_match;
use crate::menu::{ex_emenu, ex_menu, ex_menutranslate};
use crate::message::ex_messages;
use crate::message::{
    e_backslash, e_invrange, e_line_number_out_of_range, e_no_errors, e_norange, e_zerocount,
};
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
use crate::search::state::searchcmdlen;
use crate::sign::ex_sign;
use crate::spell::{ex_spelldump, ex_spellinfo, ex_spellrepall};
use crate::spellfile::{ex_mkspell, ex_spell};
use crate::syntax::{ex_ownsyntax, ex_syntax, ex_syntime};
use crate::tag::do_tags;
use crate::types::{
    Callback, CdCause, ChannelPart, CmdAddr, Direction, DoBufAction, DoBufStart, EStackArg,
    EStackType, ExArg, ExArgt, Exception, GArray, Handle, LineGetter, LineNr, LuaRetMode, MarkGet,
    MotionType, OptMagic, RemapValues, uint8_t, uint16_t,
};
use crate::undo::{ex_undojoin, ex_undolist};
use crate::usercmd::{ex_comclear, ex_command, ex_delcommand};
use crate::version::{ex_intro, ex_version};
use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

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
mod addrtype;
pub use self::addrtype::*;
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
pub(crate) use self::file::*;
mod path;
pub use self::path::*;
mod edit;
pub use self::edit::*;
mod exnormal;
pub use self::exnormal::*;
mod display;
pub use self::display::*;
mod tags;
pub(crate) use self::tags::*;
mod filetype;
pub use self::filetype::*;
mod childproc;
pub(crate) use self::childproc::*;
pub const kDirectionNotSet: Direction = 0;
pub const kCdCauseManual: CdCause = 0;
pub const OPTION_MAGIC_OFF: OptMagic = 2;
pub const OPTION_MAGIC_ON: OptMagic = 1;
pub const kMarkAll: MarkGet = 1;
pub const kMarkBufLocal: MarkGet = 0;
pub const CSTP_THROW: c_uint = 4;
pub const CSTP_INTERRUPT: c_uint = 2;
pub const CSTP_ERROR: c_uint = 1;
/// A command handler. Plain `unsafe fn`, not `extern "C"`: nothing
/// outside this crate calls the table.
pub type ExFunc = Option<fn(&mut ExArg)>;
/// An 'inccommand' preview callback, likewise.
pub type ExPreviewFunc = Option<fn(&mut ExArg, c_int, Handle) -> c_int>;
pub struct CommandDefinition {
    pub cmd_name: *mut c_char,
    pub cmd_func: ExFunc,
    pub cmd_preview_func: ExPreviewFunc,
    pub cmd_argt: ExArgt,
    pub cmd_addr_type: CmdAddr,
}
pub const DOBUF_WIPE: DoBufAction = 4;
pub const DOBUF_DEL: DoBufAction = 3;
pub const DOBUF_UNLOAD: DoBufAction = 2;
pub const DOBUF_MOD: DoBufStart = 3;
pub const DOBUF_LAST: DoBufStart = 2;
pub const DOBUF_FIRST: DoBufStart = 1;
pub const DOBUF_CURRENT: DoBufStart = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kMTLineWise: MotionType = 1;
pub const VIM_QUESTION: c_uint = 4;
pub const VIM_YES: c_uint = 2;
pub const CCGD_EXCMD: c_uint = 16;
pub const CCGD_FORCEIT: c_uint = 4;
pub const CCGD_MULTWIN: c_uint = 2;
pub const CCGD_AW: c_uint = 1;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const VALID_HEAD: c_uint = 2;
pub const VALID_PATH: c_uint = 1;
pub const DIALOG_MSG_SIZE: c_uint = 1000;
pub struct SavedDebugState {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub caught_stack: *mut Exception,
    pub vv_exception: *mut c_char,
    pub vv_throwpoint: *mut c_char,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: bool,
    pub need_rethrow: c_int,
    pub check_cstack: c_int,
    pub current_exception: *mut Exception,
}
pub struct LoopCookie {
    pub lines_gap: *mut GArray,
    pub current_line: c_int,
    pub repeating: c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut c_void,
}
pub struct WhileCmd {
    pub line: *mut c_char,
    pub lnum: LineNr,
}
pub const ETYPE_EXCEPT: EStackType = 5;
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
pub const ESTACK_SCRIPT: EStackArg = 3;
pub const ESTACK_STACK: EStackArg = 2;
pub const ESTACK_SFILE: EStackArg = 1;
pub const FIND_STRING: c_uint = 2;
pub const FIND_EVAL: c_uint = 4;
pub const FIND_IDENT: c_uint = 1;
pub const INT32_MAX: c_int = 2147483647 as c_int;
pub const NULL_1: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const EXIT_FAILURE: c_int = 1 as c_int;
pub const GA_EMPTY_INIT_VALUE: GArray = GArray {
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
static e_ambiguous_use_of_user_defined_command: &CStr =
    c"E464: Ambiguous use of user-defined command";
static e_no_call_stack_to_substitute_for_stack: &CStr =
    c"E489: No call stack to substitute for \"<stack>\"";
static e_not_an_editor_command: &CStr = c"E492: Not an editor command";
static e_no_autocommand_file_name_to_substitute_for_afile: &CStr =
    c"E495: No autocommand file name to substitute for \"<afile>\"";
static e_no_autocommand_buffer_number_to_substitute_for_abuf: &CStr =
    c"E496: No autocommand buffer number to substitute for \"<abuf>\"";
static e_no_autocommand_match_name_to_substitute_for_amatch: &CStr =
    c"E497: No autocommand match name to substitute for \"<amatch>\"";
static e_no_source_file_name_to_substitute_for_sfile: &CStr =
    c"E498: No :source file name to substitute for \"<sfile>\"";
static e_no_line_number_to_use_for_slnum: &CStr = c"E842: No line number to use for \"<slnum>\"";
static e_no_line_number_to_use_for_sflnum: &CStr = c"E961: No line number to use for \"<sflnum>\"";
static e_no_script_file_name_to_substitute_for_script: &CStr =
    c"E1274: No script file name to substitute for \"<script>\"";
static quitmore: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static ex_pressedreturn: GlobalCell<bool> = GlobalCell::new(false);
/// The `+cmd` argument a bare `+` stands for. Never written, and
/// recognised by *address* in `expand_filename`, which is why it is one
/// static rather than a literal at each of its two uses.
static dollar_command: &CStr = c"$";
static cmdline_call_depth: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
/// The command Ex mode substitutes for a bare newline. Never written, and
/// recognised by address in `ex_range_without_command`.
static exmode_plus: &CStr = c"+";
static ffu_cb: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static prev_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static filetype_detect: GlobalCell<Option<bool>> = GlobalCell::new(None);
static filetype_plugin: GlobalCell<Option<bool>> = GlobalCell::new(None);
static filetype_indent: GlobalCell<Option<bool>> = GlobalCell::new(None);
pub const MSG_BUF_LEN: c_int = 480 as c_int;
pub const FILETYPE_FILE: &CStr = c"filetype.lua filetype.vim";
pub const FTPLUGIN_FILE: &CStr = c"ftplugin.vim";
pub const INDENT_FILE: &CStr = c"indent.vim";
pub const FTOFF_FILE: &CStr = c"ftoff.vim";
pub const FTPLUGOF_FILE: &CStr = c"ftplugof.vim";
pub const INDOFF_FILE: &CStr = c"indoff.vim";
pub const PROF_YES: c_int = 1 as c_int;
pub const SID_NONE: c_int = -6 as c_int;
pub const KS_SPECIAL: c_int = 254 as c_int;

/// `xfree()` as checked code.
///
/// Nine of the family's files free an `xmalloc`ed buffer on their way out
/// of a command, and each had carried its own copy of this shim. One here
/// pays the promise once for all of them.
pub(crate) fn xfree(ptr: *mut c_void) {
    // SAFETY: `xmalloc`ed, or null.
    unsafe { crate::memory::xfree(ptr) }
}
