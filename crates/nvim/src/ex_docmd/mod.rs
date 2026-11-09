//! The Ex command dispatcher: the command table, and the shared
//! vocabulary its twenty-one children are written against.
//!
//! `CMDNAMES` is the whole of `:` — 557 rows, in the order `ex_cmds.lua`
//! lists them, with `CMD_*` as indices into it. Nothing else lives here:
//! the parsing is under `scan`, `address`, `modifier` and `lookup`, the
//! driving under `onecmd`, `cmdline` and `source`, and one file per family
//! of `ex_*` handler.
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

pub mod state;

use crate::arglist::{
    ex_all, ex_argadd, ex_argdedupe, ex_argdelete, ex_argedit, ex_args, ex_argument, ex_last,
    ex_next, ex_previous, ex_rewind,
};
use crate::autocmd::ex_doautoall;
use crate::buffer::{buflist_list, ex_buffer_all};
use crate::channel::defs::ChannelPart;
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
use crate::ex_eval::defs::ExcId;
use crate::ex_eval::{
    ex_break, ex_catch, ex_continue, ex_else, ex_endfunction, ex_endif, ex_endtry, ex_endwhile,
    ex_eval, ex_finally, ex_if, ex_throw, ex_try, ex_while,
};
use crate::ex_session::{ex_loadview, ex_mkrc};
use crate::global_cell::GlobalCell;
use crate::help::{ex_exusage, ex_help, ex_helpclose, ex_helptags, ex_viusage};
use crate::indent::ex_retab;
use crate::keycodes::KS_SPECIAL;
use crate::lua::executor::{ex_lua, ex_luado, ex_luafile};
use crate::lua::secure::ex_trust;
use crate::mapping::{ex_abbreviate, ex_abclear, ex_map, ex_mapclear, ex_unmap};
use crate::mark::defs::MarkGet;
use crate::mark::{ex_changes, ex_clearjumps, ex_delmarks, ex_jumps, ex_marks};
use crate::r#match::ex_match;
use crate::memory::{ThinCString, XString};
use crate::menu::{ex_emenu, ex_menu, ex_menutranslate};
use crate::message::ex_messages;
use crate::message::{
    E_BACKSLASH, E_INVRANGE, E_LINE_NUMBER_OUT_OF_RANGE, E_NO_ERRORS, E_NORANGE, E_ZEROCOUNT,
};
use crate::option::ex_set;
use crate::os::lang::ex_language;
use crate::os::libc::{uint8_t, uint16_t};
use crate::profile::ex_profile;
use crate::quickfix::{
    ex_cbelow, ex_cbottom, ex_cbuffer, ex_cc, ex_cclose, ex_cexpr, ex_cfile, ex_cnext, ex_copen,
    ex_cwindow, ex_helpgrep, ex_make, ex_vimgrep, qf_age, qf_history, qf_list,
};
use crate::regexp::OptMagic;
use crate::register::ex_display;

use crate::runtime::{
    ex_finish, ex_options, ex_packadd, ex_packloadall, ex_runtime, ex_scriptencoding,
    ex_scriptnames, ex_source,
};
use crate::search::state::SEARCHCMDLEN;
use crate::sign::ex_sign;
use crate::spell::{ex_spelldump, ex_spellinfo, ex_spellrepall};
use crate::spellfile::{ex_mkspell, ex_spell};
use crate::syntax::{ex_ownsyntax, ex_syntax, ex_syntime};
use crate::tag::do_tags;
use crate::types::{
    Callback, CdCause, CmdAddr, Direction, DoBufStart, ExArg, ExArgt, Handle, LineNr,
};
use crate::undo::{ex_undojoin, ex_undolist};
use crate::usercmd::{ex_comclear, ex_command, ex_delcommand};
use crate::version::{ex_intro, ex_version};
use core::ffi::{CStr, c_int, c_uint, c_void};

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
#[cfg(test)]
mod tests;
pub(crate) use self::childproc::*;
pub const DIRECTION_NOT_SET: Direction = 0;
pub const CD_CAUSE_MANUAL: CdCause = 0;
pub const OPTION_MAGIC_OFF: OptMagic = 2;
pub const OPTION_MAGIC_ON: OptMagic = 1;
pub const MARK_ALL: MarkGet = 1;
crate::flag_set! {
    /// How [`do_cmdline`] should run the lines it is given -- upstream's
    /// `DOCMD_*`.
    pub struct DoCmdOpts;

    /// Include the command in any error message.
    const VERBOSE = 1;
    /// Skip `wait_return` and friends.
    const NOWAIT = 2;
    /// Keep asking the line source until it runs out.
    const REPEAT = 4;
    /// Leave `KEY_TYPED` alone.
    const KEYTYPED = 8;
    /// Save and restore the exception environment (debugging).
    const EXCRESET = 16;
    /// Remember the first typed line, for `.` to repeat.
    const KEEPLINE = 32;
}

impl ExArg<'_> {
    /// The next line `self`'s source answers -- upstream's
    /// `eap->ea_getline(c, eap->cookie, indent, do_concat)` -- or `None` at the
    /// end of the input, or when the command was given no source.
    pub(crate) fn next_source_line(
        &mut self,
        c: Option<u8>,
        indent: c_int,
        do_concat: bool,
    ) -> Option<XString> {
        self.source.as_mut()?.next_line(c, indent, do_concat)
    }
}

/// The line number the message and breakpoint machinery reports.
pub(crate) fn sourcing_lnum() -> LineNr {
    crate::runtime::sourcing_lnum()
}

/// A command handler. A plain safe `fn`, not `extern "C"`: nothing outside
/// this crate calls the table, and a handler that still needed the keyword
/// would not coerce -- which is how the table checks its own rows.
pub type ExFunc = Option<fn(&mut ExArg)>;
/// An 'inccommand' preview callback, likewise.
pub type ExPreviewFunc = Option<fn(&mut ExArg, c_int, Handle) -> c_int>;
#[derive(Debug)]
pub struct CommandDefinition {
    pub cmd_name: &'static CStr,
    pub cmd_func: ExFunc,
    pub cmd_preview_func: ExPreviewFunc,
    pub cmd_argt: ExArgt,
    pub cmd_addr_type: CmdAddr,
}
pub const DOBUF_LAST: DoBufStart = 2;
pub const CHANNEL_PART_ALL: ChannelPart = 4;
pub const VALID_HEAD: c_uint = 2;
pub const VALID_PATH: c_uint = 1;
pub const DIALOG_MSG_SIZE: c_uint = 1000;
#[derive(Debug)]
pub struct SavedDebugState {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub(crate) caught_stack: Vec<ExcId>,
    /// Copies of `v:exception` and `v:throwpoint`, which stay set.
    pub vv_exception: Option<ThinCString>,
    pub vv_throwpoint: Option<ThinCString>,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: bool,
    pub need_rethrow: c_int,
    pub check_cstack: c_int,
    pub(crate) current_exception: Option<ExcId>,
}

pub const INT32_MAX: c_int = 2147483647 as c_int;
pub const NULL_1: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const EXIT_FAILURE: c_int = 1 as c_int;
pub const EXFLAG_LIST: c_int = 0x1 as c_int;
pub const EXFLAG_NR: c_int = 0x2 as c_int;
pub const EXFLAG_PRINT: c_int = 0x4 as c_int;
static E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND: &CStr =
    c"E464: Ambiguous use of user-defined command";
static E_NO_CALL_STACK_TO_SUBSTITUTE_FOR_STACK: &CStr =
    c"E489: No call stack to substitute for \"<stack>\"";
static E_NOT_AN_EDITOR_COMMAND: &CStr = c"E492: Not an editor command";
static E_NO_AUTOCOMMAND_FILE_NAME_TO_SUBSTITUTE_FOR_AFILE: &CStr =
    c"E495: No autocommand file name to substitute for \"<afile>\"";
static E_NO_AUTOCOMMAND_BUFFER_NUMBER_TO_SUBSTITUTE_FOR_ABUF: &CStr =
    c"E496: No autocommand buffer number to substitute for \"<abuf>\"";
static E_NO_AUTOCOMMAND_MATCH_NAME_TO_SUBSTITUTE_FOR_AMATCH: &CStr =
    c"E497: No autocommand match name to substitute for \"<amatch>\"";
static E_NO_SOURCE_FILE_NAME_TO_SUBSTITUTE_FOR_SFILE: &CStr =
    c"E498: No :source file name to substitute for \"<sfile>\"";
static E_NO_LINE_NUMBER_TO_USE_FOR_SLNUM: &CStr = c"E842: No line number to use for \"<slnum>\"";
static E_NO_LINE_NUMBER_TO_USE_FOR_SFLNUM: &CStr = c"E961: No line number to use for \"<sflnum>\"";
static E_NO_SCRIPT_FILE_NAME_TO_SUBSTITUTE_FOR_SCRIPT: &CStr =
    c"E1274: No script file name to substitute for \"<script>\"";
static QUITMORE: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static EX_PRESSEDRETURN: GlobalCell<bool> = GlobalCell::new(false);
static CMDLINE_CALL_DEPTH: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
static FFU_CB: GlobalCell<Callback> = GlobalCell::new(Callback::None);
static PREV_DIR: GlobalCell<Option<XString>> = GlobalCell::new(None);
static FILETYPE_DETECT: GlobalCell<Option<bool>> = GlobalCell::new(None);
static FILETYPE_PLUGIN: GlobalCell<Option<bool>> = GlobalCell::new(None);
static FILETYPE_INDENT: GlobalCell<Option<bool>> = GlobalCell::new(None);
pub const FILETYPE_FILE: &CStr = c"filetype.lua filetype.vim";
pub const FTPLUGIN_FILE: &CStr = c"ftplugin.vim";
pub const INDENT_FILE: &CStr = c"indent.vim";
pub const FTOFF_FILE: &CStr = c"ftoff.vim";
pub const FTPLUGOF_FILE: &CStr = c"ftplugof.vim";
pub const INDOFF_FILE: &CStr = c"indoff.vim";
