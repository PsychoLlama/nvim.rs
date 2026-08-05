#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_nl_or_nul};
use crate::src::nvim::autocmd::{EVENT_FUNCUNDEFINED, apply_autocmds};
use crate::src::nvim::charset::{getdigits, skiptowhite, skipwhite, vim_strsize};
use crate::src::nvim::debugger::{dbg_breakpoint, dbg_find_breakpoint, has_profiling};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::funcs::{
    call_internal_func, call_internal_method, check_internal_func, find_internal_func,
};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_item_alloc, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_unref, tv_get_number_chk, tv_list_append, tv_list_init_static,
    value_check_lock,
};
use crate::src::nvim::eval::typval::{tv_is_func, tv_list_set_lock};
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
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_14 = 1073741823;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_15 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_15 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_16 = 20;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const FIXVAR_CNT: C2Rust_Unnamed_18 = 12;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_19 = 19;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CSTP_RETURN: C2Rust_Unnamed_21 = 24;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const TFN_NO_DEREF: C2Rust_Unnamed_22 = 8;
pub const TFN_NO_AUTOLOAD: C2Rust_Unnamed_22 = 4;
pub const TFN_QUIET: C2Rust_Unnamed_22 = 2;
pub const TFN_INT: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const GLV_READ_ONLY: C2Rust_Unnamed_23 = 16;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FCERR_DELETED: C2Rust_Unnamed_25 = 7;
pub const FCERR_NONE: C2Rust_Unnamed_25 = 5;
pub const FCERR_DICT: C2Rust_Unnamed_25 = 4;
pub const FCERR_SCRIPT: C2Rust_Unnamed_25 = 3;
pub const FCERR_TOOFEW: C2Rust_Unnamed_25 = 2;
pub const FCERR_TOOMANY: C2Rust_Unnamed_25 = 1;
pub const FCERR_UNKNOWN: C2Rust_Unnamed_25 = 0;
pub const DOCMD_REPEAT: C2Rust_Unnamed_27 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_27 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_27 = 2;
pub const KE_SNR: key_extra = 82;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FNE_INCL_BR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
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
static funcargs: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static current_funccal: GlobalCell<*mut funccall_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccall_T>());
static previous_funccal: GlobalCell<*mut funccall_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccall_T>());
static e_funcexts: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E122: Function %s already exists, add ! to replace it\0".as_ptr()
        as *const ::core::ffi::c_char,
);
static e_funcdict: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E717: Dictionary entry already exists\0".as_ptr() as *const ::core::ffi::c_char,
);
static e_funcref: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E718: Funcref required\0".as_ptr() as *const ::core::ffi::c_char);
static e_nofunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E130: Unknown function: %s\0".as_ptr() as *const ::core::ffi::c_char);
static e_function_list_was_modified: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E454: Function list was modified\0",
        )
    });
static e_function_nesting_too_deep: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E1058: Function nesting too deep\0",
        )
    });
static e_no_white_space_allowed_before_str_str: GlobalCell<[::core::ffi::c_char; 46]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
            *b"E1068: No white space allowed before '%s': %s\0",
        )
    });
static e_missing_heredoc_end_marker_str: GlobalCell<[::core::ffi::c_char; 38]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
            *b"E1145: Missing heredoc end marker: %s\0",
        )
    });
static e_cannot_use_partial_with_dictionary_for_defer: GlobalCell<[::core::ffi::c_char; 55]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
            *b"E1300: Cannot use a partial with dictionary for :defer\0",
        )
    });
pub const FC_ABORT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FC_RANGE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const FC_DICT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FC_CLOSURE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FC_DELETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FC_REMOVED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const FC_SANDBOX: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const FC_NOARGS: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const MSG_BUF_CLEN: ::core::ffi::c_int = MSG_BUF_LEN / 6 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
