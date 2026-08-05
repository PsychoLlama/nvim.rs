#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};
use core::{ptr, slice};

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_nl_or_nul};
use crate::src::nvim::autocmd::{EVENT_FUNCUNDEFINED, apply_autocmds};
use crate::src::nvim::charset::{getdigits, skiptowhite, skipwhite, vim_strsize};
use crate::src::nvim::debugger::{dbg_breakpoint, dbg_find_breakpoint, has_profiling};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::funcs::{
    call_internal_func, call_internal_method, check_internal_func, find_internal_func,
};
use crate::src::nvim::eval::typval::{GARRAY_EMPTY, tv_is_func, tv_list_set_lock};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_item_alloc, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_unref, tv_get_number_chk, tv_list_append, tv_list_init_static,
    value_check_lock,
};
use crate::src::nvim::eval::vars::{
    find_var, find_var_ht, find_var_in_ht, get_vim_var_nr, init_var_dict, list_hashtable_vars,
    skip_var_list, vars_clear, vars_clear_ext,
};
use crate::src::nvim::eval::{
    callback_call, check_luafunc_name, clear_evalarg, clear_lval, eval_isnamec, eval_isnamec1,
    eval_lavars_used, eval0, eval1, fill_evalarg_from_eap, find_name_end, garbage_collect,
    get_id_len, get_lval, handle_subscript, is_luafunc, last_set_msg, partial_name, partial_unref,
    set_ref_in_ht, set_ref_in_item, set_ref_in_list_items, skip_expr,
};
use crate::src::nvim::ex_docmd::{check_nextcmd, checkforcmd, do_cmdline, ends_excmd, skip_range};
use crate::src::nvim::ex_eval::{
    aborted_in_try, aborting, cleanup_conditionals, exception_state_clear, exception_state_restore,
    exception_state_save, report_make_pending, update_force_abort,
};
use crate::src::nvim::ex_getln::{
    getcmdline, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave,
};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::getchar::{restoreRedobuff, saveRedobuff};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_add, hash_find, hash_find_len, hash_init, hash_remove};
use crate::src::nvim::insexpand::ins_compl_active;
use crate::src::nvim::keycodes::K_SPECIAL;
use crate::src::nvim::lua::executor::{
    api_free_luaref, nlua_set_sctx, nlua_typval_call, typval_exec_lua_callable,
};
use crate::src::nvim::main::{
    EVALARG_EVALUATE, IObuff, KeyTyped, RedrawingDisabled, Rows, cmdline_row, curbuf, current_sctx,
    curwin, debug_backtrace_level, debug_tick, did_emsg, did_throw, do_profiling, e_dictkey,
    e_invarg2, e_invexpr2, e_invrange, e_missingparen, e_not_callable_type_str,
    e_str_not_inside_function, e_toofewarg, e_toomanyarg, e_trailing_arg, e_unknown_function_str,
    e_usingsid, emsg_off, emsg_severe, emsg_skip, ex_nesting_level, got_int, lines_left, msg_row,
    msg_scroll, need_wait_return, no_wait_return, p_ic, p_mfd, p_verbose, sandbox, trylevel,
    want_garbage_collect,
};
use crate::src::nvim::mbyte::mb_strnicmp;
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz, xmemrchr, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, iemsg, internal_error, message_filtered, msg_clr_eos, msg_ext_set_kind, msg_outnum,
    msg_prt_line, msg_putchar, msg_puts, msg_start, semsg, smsg, swmsg, trunc_string,
    verbose_enter_scroll, verbose_leave_scroll,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memchr, memcmp, memcpy, memmove, memset,
    snprintf, strchr, strcmp, strcpy, strlen, strncmp, strstr,
};
use crate::src::nvim::path::path_fnamecmp;
use crate::src::nvim::profile::{
    func_do_profile, func_line_end, func_line_start, prof_def_func, profile_add, profile_end,
    profile_self, profile_start, profile_sub_wait, profile_zero, script_prof_restore,
    script_prof_save,
};
use crate::src::nvim::regexp::{RE_MAGIC, skip_regexp, vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::runtime::{
    autoload_name, estack_pop, estack_push_ufunc, exestack, get_sourced_lnum, script_autoload,
    script_items,
};
use crate::src::nvim::search::{restore_search_patterns, save_search_patterns};
use crate::src::nvim::strings::{concat_str, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUICmdline;
use crate::src::nvim::types::{
    CMD_defer, Callback, EvalFuncDef, LuaRef, OptInt, String_0, VAR_DEF_SCOPE, VAR_DICT, VAR_FIXED,
    VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SHORT_LEN, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, VV_TESTING, blob_T, buf_T, buffblock, buffblock_T, buffheader_T, colnr_T,
    cstack_T, dict_T, dictitem_T, estack_T, evalarg_T, exarg_T, except_T, exception_state_T,
    expand_T, funccal_entry_T, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T, funcdict_T,
    funcexe_T, garray_T, hashitem_T, hashtab_T, ht_stack_T, intmax_t, key_extra, linenr_T, list_T,
    list_stack_T, listitem_T, lval_T, partial_T, proftime_T, regmatch_T, regprog_T, save_redo_T,
    sctx_T, size_t, typval_T, typval_vval_union, ufunc_T, uint8_t, varnumber_T,
};
use crate::src::nvim::ui::ui_has;

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
/// The refcount an item that must never be freed carries.
pub const DO_NOT_FREE_CNT: c_int = 1073741823;

/// `dictitem_T::di_flags`: fixed (the item lives inside its owner), and
/// read-only always or only inside the sandbox.
pub const DI_FLAGS_FIX: u8 = 4;
pub const DI_FLAGS_RO_SBX: u8 = 2;
pub const DI_FLAGS_RO: u8 = 1;

/// How many arguments a call may carry, and how many locals live in the
/// funccall's own `fc_fixvar` array before one has to be allocated.
pub const MAX_FUNC_ARGS: c_int = 20;
pub const FIXVAR_CNT: c_int = 12;

pub const EXPAND_USER_FUNC: c_int = 19;
pub const CSTP_RETURN: c_int = 24;

/// `trans_function_name` flags.
pub const TFN_NO_DEREF: c_int = 8;
pub const TFN_NO_AUTOLOAD: c_int = 4;
pub const TFN_QUIET: c_int = 2;
pub const TFN_INT: c_int = 1;

pub const GLV_READ_ONLY: c_int = 16;
pub const EVAL_EVALUATE: c_int = 1;

/// Why a call could not be made; `user_func_error` turns one into a message.
pub const FCERR_DELETED: c_int = 7;
pub const FCERR_NONE: c_int = 5;
pub const FCERR_DICT: c_int = 4;
pub const FCERR_SCRIPT: c_int = 3;
pub const FCERR_TOOFEW: c_int = 2;
pub const FCERR_TOOMANY: c_int = 1;
pub const FCERR_UNKNOWN: c_int = 0;

pub const DOCMD_REPEAT: c_int = 4;
pub const DOCMD_VERBOSE: c_int = 1;
pub const DOCMD_NOWAIT: c_int = 2;
pub const KE_SNR: key_extra = 82;
pub const KS_EXTRA: c_int = 253;
pub const LUA_NOREF: c_int = -2;
pub const NUL: c_int = 0;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NOTDONE: c_int = 2;
pub const FNE_INCL_BR: c_int = 1;
pub const FNE_CHECK_START: c_int = 2;
pub const AUTOLOAD_CHAR: c_int = '#' as c_int;
pub const TV_CSTRING: size_t = size_t::MAX - 1;
pub const IOSIZE: c_int = 1024 + 1;
pub const MSG_BUF_LEN: c_int = 480;
pub const MSG_BUF_CLEN: c_int = MSG_BUF_LEN / 6;
pub const PROF_YES: c_int = 1;

// Transitional: the transpile's spellings of `true`, `false`, `NULL`,
// `isdigit()`'s ctype bit and an empty `garray_T`.  Each child drops its uses
// as it is rewritten; the last one to go deletes this block.
pub const true_0: c_int = 1;
pub const false_0: c_int = 0;
pub const NULL: *mut c_void = ptr::null_mut();
pub const _ISdigit: c_uint = 2048;
pub const GA_EMPTY_INIT_VALUE: garray_T = GARRAY_EMPTY;

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
static func_hashtab: GlobalCell<hashtab_T> = GlobalCell::new(hashtab_T {
    ht_mask: 0,
    ht_used: 0,
    ht_filled: 0,
    ht_changed: 0,
    ht_locked: 0,
    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
    ht_smallarray: [hashitem_T {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }; 16],
});
static funcargs: GlobalCell<garray_T> = GlobalCell::new(GARRAY_EMPTY);
static current_funccal: GlobalCell<*mut funccall_T> = GlobalCell::new(ptr::null_mut());
static previous_funccal: GlobalCell<*mut funccall_T> = GlobalCell::new(ptr::null_mut());

/// `ufunc_T::uf_flags`.
pub const FC_ABORT: c_int = 0x1;
pub const FC_RANGE: c_int = 0x2;
pub const FC_DICT: c_int = 0x4;
pub const FC_CLOSURE: c_int = 0x8;
pub const FC_DELETED: c_int = 0x10;
pub const FC_REMOVED: c_int = 0x20;
pub const FC_SANDBOX: c_int = 0x40;
pub const FC_NOARGS: c_int = 0x200;
pub const FC_LUAREF: c_int = 0x800;

pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
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

/// The `char *` items a string `garray_T` holds, as a slice.
///
/// Every `uf_args`/`uf_def_args`/`uf_lines` walk in this family is a read of
/// exactly this array, and c2rust spelled each one as a cast plus an index.
/// Safe, because the array belongs to the `garray_T` the borrow names.
pub(crate) fn ga_strings(gap: &garray_T) -> &[*mut c_char] {
    if gap.ga_data.is_null() {
        return &[];
    }
    // SAFETY: a `char *` garray's data is `ga_len` initialised pointers.
    unsafe { slice::from_raw_parts(gap.ga_data as *const *mut c_char, gap.ga_len as usize) }
}
