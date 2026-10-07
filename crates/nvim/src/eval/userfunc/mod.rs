// No `forbid(unsafe_code)` here: it would reach `call` and `ret`, which
// still need it.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::types::AutoEvent;
use core::ffi::{CStr, c_int};

use crate::ascii::{ascii_isident, ascii_iswhite, ascii_iswhite_nl_or_nul};
use crate::charset::skip;
use crate::debugger::state::{debug_backtrace_level, debug_tick};
use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint_named, has_profiling_named};
use crate::drawscreen::state::cmdline_row;
use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::funcs::{check_builtin_argcount, find_builtin};
use crate::eval::gc::want_garbage_collect;
use crate::eval::typval::{TV_INITIAL_VALUE, list_iter, tv_clear, tv_copy, tv_get_number_chk};
use crate::eval::vars::{skip_var_list, testing_enabled};
use crate::eval::{
    Cursor, LAMBDA_USES_LOCALS, callback_call, check_luafunc_name, eval_isnamec, eval_isnamec1,
    eval0_in_cmd, eval1, garbage_collect, get_lval, handle_subscript, id_len, is_luafunc,
    last_set_msg, mark_root, name_end, set_ref_in_dict_items, set_ref_in_list_items,
};
use crate::ex_docmd::state::ex_nesting_level;
use crate::ex_docmd::{ends_excmd, skip_range};
use crate::ex_eval::state::{did_throw, trylevel};
use crate::ex_eval::{
    PendingAction, aborted_in_try, aborting, cleanup_conditionals, exception_state_clear,
    exception_state_restore, exception_state_save, report_pending, update_force_abort,
};
use crate::ex_getln::{ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave};
use crate::getchar::state::{KeyTyped, got_int};
use crate::getchar::{restore_redobuff, save_redobuff};
use crate::global_cell::GlobalCell;
use crate::guard::sandbox;
use crate::insexpand::ins_compl_active;
use crate::keycodes::K_SPECIAL;
use crate::lua::executor::{release_luaref, typval_exec_lua_callable};
use crate::message::state::{
    did_emsg, emsg_severe, lines_left, msg_row, msg_scroll, need_wait_return,
};
use crate::message::{
    e_invarg2, e_invrange, e_toofewarg, e_toomanyarg, e_unknown_function_str, e_usingsid,
};
use crate::message::{
    emsg, internal_error, message_filtered, msg_clr_eos, msg_ext_set_kind, msg_outnum,
    msg_prt_line, msg_putchar, msg_start, msg_str, verbose_enter_scroll, verbose_leave_scroll,
};
use crate::message_fmt::msg_bytes;
use crate::option::vars::{p_ic, p_mfd, p_verbose};
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::path::path_fnamecmp;
use crate::profile::do_profiling;
use crate::profile::{
    func_do_profile, func_line_end, func_line_start, prof_def_func, profile_add, profile_end,
    profile_self, profile_start, profile_sub_wait, profile_zero, script_prof_restore,
    script_prof_save,
};
use crate::regexp::{RE_MAGIC, skip_regexp_at};
use crate::runtime::state::current_sctx;
use crate::runtime::{estack_pop, estack_push_ufunc, script_id_valid};
use crate::search::{restore_search_patterns, save_search_patterns};
use crate::types::ui::kUICmdline;
use crate::types::{
    Callback, Dict, DictItem, EStack, ExArg, Expand, FcId, FuncBody, FuncCall, LineNr, ListItem,
    LuaRef, OptInt, Partial, SaveRedo, TypVal, UserFunc, VAR_DEF_SCOPE, VAR_FUNC, VAR_NUMBER,
    VAR_SCOPE, VAR_STRING, VAR_UNKNOWN, VarLock, VarNumber, size_t,
};
use crate::ui::state::Rows;
use crate::ui::ui_has;

// The carve of the transpiled module; see each child's docs.
mod args;
mod body;
mod call;
mod define;
mod dispatch;
mod frames;
mod funccall;
mod lambda;
mod listing;
mod name;
mod ret;
mod table;
#[cfg(test)]
mod tests;

pub(crate) use self::args::*;
pub use self::body::*;
pub use self::call::*;
pub use self::define::*;
pub use self::dispatch::*;
pub(crate) use self::frames::*;
pub use self::funccall::*;
pub use self::lambda::*;
pub use self::listing::*;
pub(crate) use self::name::*;
pub use self::ret::*;
pub(crate) use self::table::*;
/// The refcount an item that must never be freed carries.
pub const DO_NOT_FREE_CNT: c_int = 1073741823;

/// `DictItem::di_flags`: fixed (the item lives inside its owner), and
/// read-only always or only inside the sandbox.
pub const DI_FLAGS_FIX: u8 = 4;
pub const DI_FLAGS_RO_SBX: u8 = 2;
pub const DI_FLAGS_RO: u8 = 1;

/// How many arguments a call may carry, and how many locals live in the
/// funccall's own `fc_fixvar` array before one has to be allocated.
pub const MAX_FUNC_ARGS: c_int = 20;
pub const FIXVAR_CNT: c_int = 12;

pub const CSTP_RETURN: c_int = 24;

/// `trans_function_name` flags.
pub const TFN_NO_DEREF: c_int = 8;
pub const TFN_NO_AUTOLOAD: c_int = 4;
pub const TFN_QUIET: c_int = 2;
pub const TFN_INT: c_int = 1;

pub const GLV_READ_ONLY: c_int = 16;

/// Why a call could not be made; `user_func_error` turns one into a message.
pub const FCERR_NOTMETHOD: c_int = 8;
pub const FCERR_DELETED: c_int = 7;
pub const FCERR_OTHER: c_int = 6;
pub const FCERR_NONE: c_int = 5;
pub const FCERR_DICT: c_int = 4;
pub const FCERR_SCRIPT: c_int = 3;
pub const FCERR_TOOFEW: c_int = 2;
pub const FCERR_TOOMANY: c_int = 1;
pub const FCERR_UNKNOWN: c_int = 0;

pub const KS_EXTRA: c_int = 253;
pub const LUA_NOREF: c_int = -2;
pub const NOTDONE: c_int = 2;
pub const FNE_INCL_BR: c_int = 1;
pub const FNE_CHECK_START: c_int = 2;
pub const AUTOLOAD_CHAR: c_int = '#' as c_int;
pub const TV_CSTRING: size_t = size_t::MAX - 1;
pub const MSG_BUF_LEN: c_int = 480;
pub const MSG_BUF_CLEN: c_int = MSG_BUF_LEN / 6;
pub const PROF_YES: c_int = 1;

/// The error texts this file owns, which upstream keeps as file statics.
pub const E_FUNCEXTS: &CStr = c"E122: Function %s already exists, add ! to replace it";
pub const E_FUNCDICT: &CStr = c"E717: Dictionary entry already exists";
pub const E_FUNCREF: &CStr = c"E718: Funcref required";
pub const E_NOFUNC: &CStr = c"E130: Unknown function: %s";
pub const E_FUNCTION_LIST_WAS_MODIFIED: &CStr = c"E454: Function list was modified";
pub const E_FUNCTION_NESTING_TOO_DEEP: &CStr = c"E1058: Function nesting too deep";
pub const E_NO_WHITE_SPACE_ALLOWED_BEFORE_STR_STR: &CStr =
    c"E1068: No white space allowed before '%s': %s";
pub const E_MISSING_HEREDOC_END_MARKER_STR: &CStr = c"E1145: Missing heredoc end marker: %s";
pub const E_CANNOT_USE_PARTIAL_WITH_DICTIONARY_FOR_DEFER: &CStr =
    c"E1300: Cannot use a partial with dictionary for :defer";
/// The arguments of the calls currently in progress, innermost last.
///
/// Only kept while `v:testing` is set: `test_garbagecollect_now()` marks
/// through it so that a value living only in a caller's argument array is not
/// collected. Each frame *names* a caller's arguments, releasing none: the
/// caller holds them by shared borrow for the whole call, and marking only
/// reads them.
static funcargs: GlobalCell<Vec<crate::eval::typval::CallFrame<{ MAX_FUNC_ARGS as usize + 1 }>>> =
    GlobalCell::new(Vec::new());

crate::flag_set! {
    /// `UserFunc::uf_flags`: how a user function was defined and what has
    /// become of it.
    pub struct FuncFlags;

    /// `:function! foo() abort` -- an error inside aborts the function.
    const ABORT = 0x1;
    /// It takes a range and handles it itself.
    const RANGE = 0x2;
    /// It is a dictionary function and wants `self`.
    const DICT = 0x4;
    /// It captures its enclosing scope.
    const CLOSURE = 0x8;
    /// `:delfunction` ran while the function was executing; it goes when
    /// the last call returns.
    const DELETED = 0x10;
    /// A redefinition replaced it, likewise while it was executing.
    const REMOVED = 0x20;
    /// Defined inside `:sandbox`, so every call runs sandboxed.
    const SANDBOX = 0x40;
    /// A lambda with no `a:` arguments at all, which lets the call skip
    /// building the argument dictionary.
    const NOARGS = 0x200;
    /// Not Vimscript: the body is a Lua reference, and the funcref can go
    /// back to the API as a `LuaRef`.
    const LUAREF = 0x800;
}

/// The innermost entry of the `:source`/function call stack: what C's
/// `SOURCING_LNUM` and `SOURCING_NAME` macros read.
pub(crate) fn sourcing_entry() -> EStack {
    crate::runtime::innermost_frame()
}

/// The line number the innermost exec-stack entry is on.
pub(crate) fn sourcing_lnum() -> LineNr {
    sourcing_entry().es_lnum
}
