#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::types::AutoEvent;
use core::ffi::{CStr, c_char, c_int};
use core::mem::offset_of;
use core::{ptr, slice};

use crate::ascii::{ascii_isident, ascii_iswhite, ascii_iswhite_nl_or_nul};
use crate::autocmd::apply_autocmds;
use crate::charset::{getdigits, skiptowhite, skipwhite, vim_strsize};
use crate::debugger::state::{debug_backtrace_level, debug_tick};
use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint, has_profiling};
use crate::drawscreen::state::cmdline_row;
use crate::eval::EVALARG_EVALUATE;
pub(crate) use crate::eval::Tv;
use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::funcs::{
    call_internal_func, call_internal_method, check_internal_func, find_internal_func,
};
use crate::eval::gc::want_garbage_collect;
use crate::eval::typval::{
    GARRAY_EMPTY, TV_INITIAL_VALUE, list_init_static, list_iter, list_iter_mut, list_set_lock,
    tv_clear, tv_copy, tv_dict_add, tv_dict_hi2di, tv_dict_item_alloc, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_iter, tv_dict_unref, tv_get_number_chk, value_check_lock,
};
use crate::eval::vars::{
    LVAL_INITIAL_VALUE, find_var, find_var_ht, find_var_in_ht, get_vim_var_nr, init_var_dict,
    list_hashtable_vars, skip_var_list, vars_clear, vars_clear_ext,
};
use crate::eval::{
    callback_call, check_luafunc_name, clear_evalarg, clear_lval, eval_isnamec, eval_isnamec1,
    eval_lavars_used, eval0, eval1, fill_evalarg_from_eap, find_name_end, garbage_collect,
    get_id_len, get_lval, handle_subscript, is_luafunc, last_set_msg, partial_name, partial_unref,
    set_ref_in_ht, set_ref_in_item, set_ref_in_list_items, skip_expr,
};
use crate::ex_docmd::state::ex_nesting_level;
use crate::ex_docmd::{check_nextcmd, checkforcmd, do_cmdline, ends_excmd, skip_range};
use crate::ex_eval::state::{did_throw, trylevel};
use crate::ex_eval::{
    aborted_in_try, aborting, cleanup_conditionals, exception_state_clear, exception_state_restore,
    exception_state_save, report_make_pending, update_force_abort,
};
use crate::ex_getln::{getcmdline, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave};
use crate::garray::{ga_append_via_ptr, ga_clear, ga_clear_strings, ga_grow, ga_init};
use crate::getchar::state::{KeyTyped, got_int};
use crate::getchar::{restore_redobuff, save_redobuff};
use crate::global_cell::GlobalCell;
use crate::guard::sandbox;
use crate::hashtab::{
    Slot, hash_add, hash_find, hash_find_len, hash_init, hash_remove, hash_set_key,
};
use crate::insexpand::ins_compl_active;
use crate::keycodes::K_SPECIAL;
use crate::lua::executor::{
    api_free_luaref, nlua_set_sctx, nlua_typval_call, typval_exec_lua_callable,
};
use crate::mbyte::mb_strnicmp;
use crate::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz, xmemrchr, xstrdup, xstrlcpy,
};
use crate::message::state::{
    did_emsg, emsg_severe, lines_left, msg_row, msg_scroll, need_wait_return,
};
use crate::message::{
    e_invarg2, e_invrange, e_toofewarg, e_toomanyarg, e_unknown_function_str, e_usingsid,
};
use crate::message::{
    emsg, iemsg, internal_error, message_filtered, msg_clr_eos, msg_ext_set_kind, msg_outnum,
    msg_prt_line, msg_putchar, msg_start, msg_str, trunc_string, verbose_enter_scroll,
    verbose_leave_scroll,
};
use crate::option::vars::{p_ic, p_mfd, p_verbose};
use crate::os::cshim::{gettext, snprintf};
use crate::os::input::line_breakcheck;
use crate::path::path_fnamecmp;
use crate::profile::do_profiling;
use crate::profile::{
    func_do_profile, func_line_end, func_line_start, prof_def_func, profile_add, profile_end,
    profile_self, profile_start, profile_sub_wait, profile_zero, script_prof_restore,
    script_prof_save,
};
use crate::regexp::{RE_MAGIC, skip_regexp, vim_regcomp, vim_regexec, vim_regfree};
use crate::runtime::state::current_sctx;
use crate::runtime::{
    autoload_name, estack_pop, estack_push_ufunc, get_sourced_lnum, script_autoload,
    script_id_valid,
};
use crate::search::{restore_search_patterns, save_search_patterns};
use crate::strings::{concat_str, xstrnsave};
use crate::types::ui::kUICmdline;
use crate::types::{
    Callback, Dict, DictItem, EStack, EvalArg, ExArg, ExceptionState, Expand, FuncCall,
    FuncCallEntry, FuncDict, FuncExe, GArray, HashTab, LVal, LineNr, ListItem, LuaRef, OptInt,
    Partial, RegMatch, SaveRedo, String_0, TypVal, UserFunc, VAR_DEF_SCOPE, VAR_DICT, VAR_FUNC,
    VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SHORT_LEN, VAR_STRING, VAR_UNKNOWN, VarLock,
    VarNumber, Vv, size_t,
};
use crate::ui::state::Rows;
use crate::ui::ui_has;
pub(crate) use crate::winlayer::{Ea, Live};
use ::libc::{abort, strcpy};

// The carve of the transpiled module; see each child's docs.
mod args;
mod body;
mod call;
mod define;
mod dispatch;
mod funccall;
mod lambda;
mod listing;
mod name;
mod ret;

pub use self::args::*;
pub use self::body::*;
pub use self::call::*;
pub use self::define::*;
pub use self::dispatch::*;
pub use self::funccall::*;
pub use self::lambda::*;
pub use self::listing::*;
pub use self::name::*;
pub use self::ret::*;
/// The two pointees this family passes around, as `Copy` newtypes.
///
/// Each is a [`Live<T>`](crate::winlayer::Live): a record that whoever built
/// it promised the pointee outlives the value. Construction is the one
/// unsafe step; every `(*p).field` after it is ordinary checked code.
///
/// Emphatically **not** `&mut *p`. A user function re-enters the evaluator,
/// autocommands and Lua while the same `FuncCall` is still reachable
/// through `current_funccal` and the same `UserFunc` through the function
/// table, and a `&mut` is `noalias` to LLVM.
///
/// A user function and its body.
pub(crate) type Uf = Live<UserFunc>;

/// One call of one: its `a:`/`l:` scopes, its caller and its return value.
pub(crate) type Fc = Live<FuncCall>;

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
pub const EVAL_EVALUATE: c_int = 1;

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
static func_hashtab: GlobalCell<HashTab> = GlobalCell::new(HashTab::new());

/// The arguments of the calls currently in progress, innermost last.
///
/// Only kept while `v:testing` is set: `test_garbagecollect_now()` marks
/// through it so that a value living only in a caller's argument array is not
/// collected. The entries are borrowed -- each points into a caller's own
/// `argvars` -- which is why this is a `Vec` of pointers and not of values.
static funcargs: GlobalCell<Vec<*mut TypVal>> = GlobalCell::new(Vec::new());
static current_funccal: GlobalCell<*mut FuncCall> = GlobalCell::new(ptr::null_mut());
static previous_funccal: GlobalCell<*mut FuncCall> = GlobalCell::new(ptr::null_mut());

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

pub const FUNCEXE_INIT: FuncExe = FuncExe {
    fe_argv_func: None,
    fe_firstline: 0,
    fe_lastline: 0,
    fe_doesrange: ptr::null_mut(),
    fe_evaluate: false,
    fe_partial: ptr::null_mut(),
    fe_selfdict: ptr::null_mut(),
    fe_basetv: ptr::null_mut(),
    fe_found_var: false,
};

/// A zeroed `RegMatch`, for the two places that compile a pattern here.
pub(crate) const REGMATCH_INIT: RegMatch = RegMatch {
    regprog: ptr::null_mut(),
    startp: [ptr::null_mut(); 10],
    endp: [ptr::null_mut(); 10],
    rm_matchcol: 0,
    rm_ic: false,
};

/// A zeroed `FuncDict`: no dictionary, no key, no item.
pub(crate) const FUNCDICT_INIT: FuncDict = FuncDict {
    fd_dict: ptr::null_mut(),
    fd_newkey: ptr::null_mut(),
    fd_di: ptr::null_mut(),
};

/// The name a `UserFunc` carries in the flexible member at its end -- C's
/// `UF2HIKEY`, and the key the function hashtable is indexed by.
///
/// Safe: a field's address is the object's plus a constant, so saying where
/// the name is reads nothing. Whether there is a name *there* is the
/// caller's business, as it is for every other pointer it holds.
pub(crate) fn uf_name_ptr(func: *mut UserFunc) -> *mut c_char {
    func.wrapping_byte_add(offset_of!(UserFunc, uf_name)).cast()
}

/// The innermost entry of the `:source`/function call stack: what C's
/// `SOURCING_LNUM` and `SOURCING_NAME` macros read.
///
/// # Safety
/// The exec stack is non-empty, which it is whenever anything is running.
pub(crate) fn sourcing_entry() -> EStack {
    crate::runtime::innermost_frame()
}

/// The line number the innermost exec-stack entry is on.
///
/// # Safety
/// As [`sourcing_entry`].
pub(crate) fn sourcing_lnum() -> LineNr {
    sourcing_entry().es_lnum
}

/// Append `s`, already owned, to a `char *` garray that has room for it.
///
/// # Safety
/// `gap` is a `char *` garray with at least one free slot (the caller has
/// just called `ga_grow`), and `s` is an allocation `ga_clear_strings` may
/// free.
pub(crate) unsafe fn ga_push_string(gap: *mut GArray, s: *mut c_char) {
    // SAFETY: the contract's garray, borrowed for the push.
    let gap = unsafe { &mut *gap };
    let slots: *mut *mut c_char = gap.ga_data.cast();
    // SAFETY: the contract says the slot at `ga_len` is free.
    unsafe { *slots.offset(gap.ga_len as isize) = s };
    gap.ga_len += 1;
}

/// The `char *` items a string `GArray` holds, as a slice.
///
/// Every `uf_args`/`uf_def_args`/`uf_lines` walk in this family is a read of
/// exactly this array, and c2rust spelled each one as a cast plus an index.
/// Safe, because the array belongs to the `GArray` the borrow names.
pub(crate) fn ga_strings(gap: &GArray) -> &[*mut c_char] {
    if gap.ga_data.is_null() {
        return &[];
    }
    let len = usize::try_from(gap.ga_len).expect("a garray length is never negative");
    // SAFETY: a `char *` garray's data is `ga_len` initialised pointers.
    unsafe { slice::from_raw_parts(gap.ga_data as *const *mut c_char, len) }
}
