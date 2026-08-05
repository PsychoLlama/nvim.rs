#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{cstr_as_string, cstr_to_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::src::nvim::charset::{getdigits_int, skiptowhite, skipwhite};
use crate::src::nvim::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::funcs::{tv_get_buf, tv_get_buf_from_arg};
use crate::src::nvim::eval::typval::QUEUE_INIT;
use crate::src::nvim::eval::typval::{
    tv_check_str_or_nr, tv_clear, tv_copy, tv_dict_add, tv_dict_alloc, tv_dict_alloc_lock,
    tv_dict_item_alloc, tv_dict_item_remove, tv_dict_set_keys_readonly, tv_dict_unref,
    tv_dict_watcher_notify, tv_free, tv_get_bool_chk, tv_get_number, tv_get_number_chk,
    tv_get_string, tv_get_string_buf_chk, tv_get_string_chk, tv_item_lock, tv_list_alloc,
    tv_list_append_allocated_string, tv_list_append_string, tv_list_append_tv, tv_list_find_nr,
    tv_list_find_str, tv_list_free, tv_list_item_remove, tv_list_remove_items, value_check_lock,
};
use crate::src::nvim::eval::typval::{
    tv_dict_is_watched, tv_dict_set_ret, tv_is_func, tv_list_first, tv_list_len, tv_list_locked,
    tv_list_ref, tv_list_set_lock, tv_list_set_ret,
};
use crate::src::nvim::eval::userfunc::{
    find_hi_in_scoped_ht, find_var_in_scoped_ht, function_exists, get_current_funccal_dict,
    get_funccal_args_dict, get_funccal_args_ht, get_funccal_args_var, get_funccal_local_dict,
    get_funccal_local_ht, get_funccal_local_var, list_func_vars,
};
use crate::src::nvim::eval::window::{find_win_by_nr, restore_win, switch_win};
use crate::src::nvim::eval::{
    clear_evalarg, clear_lval, eval_expr_ext, eval_isnamec1, eval_lavars_used, eval_option,
    eval_to_bool, eval_to_string, eval0, eval1, fill_evalarg_from_eap, find_name_end,
    find_option_var_end, get_env_len, get_lval, get_name_len, handle_subscript,
    may_call_simple_func, num_divide, num_modulus, set_ref_in_ht, set_var_lval, skip_expr,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{check_nextcmd, ends_excmd};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{
    hash_add, hash_clear, hash_find, hash_find_len, hash_init, hash_lock, hash_remove, hash_unlock,
};
use crate::src::nvim::lua::executor::nlua_set_sctx;
use crate::src::nvim::main::{
    EVALARG_EVALUATE, called_emsg, curbuf, current_sctx, curtab, curwin, did_emsg,
    e_cannot_change_readonly_variable_str, e_cannot_delete_variable_str, e_cannot_mod,
    e_cannot_set_variable_in_sandbox_str, e_illvar, e_invarg, e_invarg2, e_letwrong, e_listreq,
    e_missing_close_curly_str, e_stray_closing_curly_str, e_string_required, e_trailing_arg,
    e_unknown_option2, emsg_off, emsg_severe, emsg_skip, firstwin, got_int, lastused_tabpage,
    no_hlsearch, p_ccv, p_dex, p_pex, p_verbose, sandbox, sc_col,
};
use crate::src::nvim::mbyte::utf_char2bytes;
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xstrdup, xstrlcat, xstrlcpy, xstrndup,
};
use crate::src::nvim::message::{
    emsg, internal_error, message_filtered, msg_advance, msg_clr_eos, msg_ext_set_kind,
    msg_outtrans, msg_putchar, msg_puts, msg_puts_len, msg_start, semsg,
};
use crate::src::nvim::option::{
    find_option, get_option, get_option_sctx, get_option_value, get_tty_option, get_winbuf_options,
    is_option_hidden, is_tty_option, kOptFlagFunc, option_has_type, optval_free,
    set_option_value_handle_tty,
};
use crate::src::nvim::options::{
    kOptAleph, kOptCharconvert, kOptDiffexpr, kOptInvalid, kOptPatchexpr, kOptSpellsuggest,
};
use crate::src::nvim::os::env::{vim_getenv, vim_setenv_ext, vim_unsetenv_ext};
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memchr, memcpy, memmove, memset, snprintf,
    strcmp, strcpy, strlen, strncmp,
};
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::register::{get_reg_contents, write_reg_contents};
use crate::src::nvim::runtime::{new_script_item, script_autoload, script_items};
use crate::src::nvim::search::set_search_direction;
use crate::src::nvim::strings::{concat_str, vim_strchr};
use crate::src::nvim::types::{
    BoolVarValue, CMD_const, CMD_lockvar, EvalFuncData, GRegFlags, OptIndex, OptInt, OptVal,
    OptValData, OptValType, QUEUE, ScopeDictDictItem, ScopeType, SpecialVarValue, TriState,
    VAR_BLOB, VAR_BOOL, VAR_DEF_SCOPE, VAR_DICT, VAR_FIXED, VAR_FLOAT, VAR_FUNC, VAR_LIST,
    VAR_NO_SCOPE, VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SPECIAL, VAR_STRING, VAR_TYPE_BLOB,
    VAR_TYPE_BOOL, VAR_TYPE_DICT, VAR_TYPE_FLOAT, VAR_TYPE_FUNC, VAR_TYPE_LIST, VAR_TYPE_NUMBER,
    VAR_TYPE_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_CC_FROM, VV_CC_TO, VV_CHAR, VV_CMDARG,
    VV_COMPAT, VV_COMPLETED_ITEM, VV_COUNT, VV_COUNT1, VV_ECHOSPACE, VV_ERRORS, VV_EVENT,
    VV_EXCEPTION, VV_EXITING, VV_FALSE, VV_FNAME_DIFF, VV_FNAME_IN, VV_FNAME_NEW, VV_FNAME_OUT,
    VV_HLSEARCH, VV_LUA, VV_MAXCOL, VV_MSGPACK_TYPES, VV_NULL, VV_NUMBERMAX, VV_NUMBERMIN,
    VV_NUMBERSIZE, VV_OPTION_COMMAND, VV_OPTION_NEW, VV_OPTION_OLD, VV_OPTION_OLDGLOBAL,
    VV_OPTION_OLDLOCAL, VV_OPTION_TYPE, VV_PREVCOUNT, VV_REG, VV_RO, VV_RO_SBX, VV_SEARCHFORWARD,
    VV_STDERR, VV_THROWPOINT, VV_TRUE, VV_TYPE_BLOB, VV_TYPE_BOOL, VV_TYPE_DICT, VV_TYPE_FLOAT,
    VV_TYPE_FUNC, VV_TYPE_LIST, VV_TYPE_NUMBER, VV_TYPE_STRING, VV_VAL, VV_VERSION, VV_VERSIONLONG,
    VarType, VimVarIndex, aco_save_T, blob_T, buf_T, dict_T, dictitem_T, evalarg_T, exarg_T,
    expand_T, garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarFalse, kBoolVarTrue, kFalse,
    kListLenUnknown, kNone, kSpecialVarNull, kTrue, list_T, list_stack_T, listitem_T, lval_T,
    partial_T, ptrdiff_t, queue, scid_T, scriptitem_T, scriptvar_T, sctx_T, size_t, ssize_t,
    switchwin_T, tabpage_T, typval_T, typval_vval_union, uint8_t, uint32_t, varnumber_T, win_T,
};
use crate::src::nvim::version::{highest_patch, min_vim_version};
use crate::src::nvim::window::{find_tabpage, goto_tabpage_tp, prevwin_curwin, valid_tabpage};

// The carve of the transpiled module; see each child's docs.
mod assign;
mod external;
mod heredoc;
mod lifecycle;
mod listing;
mod lookup;
mod redir;
mod scoped;
mod store;
mod unlet;
mod vvar;

pub use self::assign::*;
pub use self::external::*;
pub use self::heredoc::*;
pub use self::lifecycle::*;
pub use self::listing::*;
pub use self::lookup::*;
pub use self::redir::*;
pub use self::scoped::*;
pub use self::store::*;
pub use self::unlet::*;
pub use self::vvar::*;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISlower: C2Rust_Unnamed = 512;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_15 = 1073741823;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DI_FLAGS_ALLOC: C2Rust_Unnamed_17 = 16;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_17 = 8;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_17 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_17 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_17 = 1;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const GLV_QUIET: C2Rust_Unnamed_21 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_22 {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [::core::ffi::c_char; 17],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimvar {
    pub vv_name: *mut ::core::ffi::c_char,
    pub vv_di: C2Rust_Unnamed_22,
    pub vv_flags: ::core::ffi::c_char,
}
pub const kGRegExprSrc: GRegFlags = 2;
pub type ex_unletlock_callback = Option<
    unsafe extern "C" fn(
        *mut lval_T,
        *mut ::core::ffi::c_char,
        *mut exarg_T,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub const OPT_LOCAL: C2Rust_Unnamed_23 = 2;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const BAD_KEEP: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const BAD_DROP: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FORCE_NOBIN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CHAN_STDERR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FNE_INCL_BR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn tv_init(tv: *mut typval_T) {
    unsafe {
        if !tv.is_null() {
            memset(
                tv as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<typval_T>(),
            );
        }
    }
}
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
pub const DICT_MAXNEST: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
static e_letunexp: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E18: Unexpected characters in :let\0".as_ptr() as *const ::core::ffi::c_char);
static e_double_semicolon_in_list_of_variables: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E452: Double ; in list of variables\0",
        )
    });
static e_lock_unlock: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E940: Cannot lock or unlock variable %s\0".as_ptr() as *const ::core::ffi::c_char,
);
static e_setting_v_str_to_value_with_wrong_type: GlobalCell<[::core::ffi::c_char; 44]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 44], [::core::ffi::c_char; 44]>(
            *b"E963: Setting v:%s to value with wrong type\0",
        )
    });
static e_missing_end_marker_str: GlobalCell<[::core::ffi::c_char; 30]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 30], [::core::ffi::c_char; 30]>(
        *b"E990: Missing end marker '%s'\0",
    )
});
static e_cannot_use_heredoc_here: GlobalCell<[::core::ffi::c_char; 26]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(*b"E991: Cannot use =<< here\0")
});
static globvars_var: GlobalCell<ScopeDictDictItem> = GlobalCell::new(ScopeDictDictItem {
    di_tv: typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    },
    di_flags: 0,
    di_key: [0; 1],
});
static globvardict: GlobalCell<dict_T> = GlobalCell::new(dict_T {
    dv_lock: VAR_UNLOCKED,
    dv_scope: VAR_NO_SCOPE,
    dv_refcount: 0,
    dv_copyID: 0,
    dv_hashtab: hashtab_T {
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
    },
    dv_copydict: ::core::ptr::null_mut::<dict_T>(),
    dv_used_next: ::core::ptr::null_mut::<dict_T>(),
    dv_used_prev: ::core::ptr::null_mut::<dict_T>(),
    watchers: QUEUE {
        next: ::core::ptr::null_mut::<queue>(),
        prev: ::core::ptr::null_mut::<queue>(),
    },
    lua_table_ref: 0,
});
static compat_hashtab: GlobalCell<hashtab_T> = GlobalCell::new(hashtab_T {
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
/// One row of the `v:` table: everything about it that is not the same
/// for all 106 rows.
///
/// c2rust expanded the C's designated initialiser field by field at
/// every row -- 31 lines each, 3,311 in total, and a single item, so no
/// carve could bring the file under the line cap while it stood.  Every
/// row was identical but for these three fields: `v_lock` is always
/// `VAR_UNLOCKED`, `vval` always the zero `v_number`, `di_flags` always
/// 0 and `di_key` always the 17 zero bytes `evalvars_init` copies the
/// name into.
const fn vv(
    name: &'static ::core::ffi::CStr,
    v_type: VarType,
    vv_flags: ::core::ffi::c_int,
) -> vimvar {
    vimvar {
        vv_name: name.as_ptr() as *mut ::core::ffi::c_char,
        vv_di: C2Rust_Unnamed_22 {
            di_tv: typval_T {
                v_type,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0,
            di_key: [0; 17],
        },
        vv_flags: vv_flags as ::core::ffi::c_char,
    }
}

static vimvars: GlobalCell<[vimvar; 106]> = GlobalCell::new([
    vv(c"count", VAR_NUMBER, VV_RO),
    vv(c"count1", VAR_NUMBER, VV_RO),
    vv(c"prevcount", VAR_NUMBER, VV_RO),
    vv(c"errmsg", VAR_STRING, 0),
    vv(c"warningmsg", VAR_STRING, 0),
    vv(c"statusmsg", VAR_STRING, 0),
    vv(c"shell_error", VAR_NUMBER, VV_RO),
    vv(c"this_session", VAR_STRING, 0),
    vv(c"version", VAR_NUMBER, VV_COMPAT | VV_RO),
    vv(c"lnum", VAR_NUMBER, VV_RO_SBX),
    vv(c"termrequest", VAR_STRING, VV_RO),
    vv(c"termresponse", VAR_STRING, VV_RO),
    vv(c"fname", VAR_STRING, VV_RO),
    vv(c"lang", VAR_STRING, VV_RO),
    vv(c"lc_time", VAR_STRING, VV_RO),
    vv(c"ctype", VAR_STRING, VV_RO),
    vv(c"charconvert_from", VAR_STRING, VV_RO),
    vv(c"charconvert_to", VAR_STRING, VV_RO),
    vv(c"fname_in", VAR_STRING, VV_RO),
    vv(c"fname_out", VAR_STRING, VV_RO),
    vv(c"fname_new", VAR_STRING, VV_RO),
    vv(c"fname_diff", VAR_STRING, VV_RO),
    vv(c"cmdarg", VAR_STRING, VV_RO),
    vv(c"foldstart", VAR_NUMBER, VV_RO_SBX),
    vv(c"foldend", VAR_NUMBER, VV_RO_SBX),
    vv(c"folddashes", VAR_STRING, VV_RO_SBX),
    vv(c"foldlevel", VAR_NUMBER, VV_RO_SBX),
    vv(c"progname", VAR_STRING, VV_RO),
    vv(c"servername", VAR_STRING, VV_RO),
    vv(c"dying", VAR_NUMBER, VV_RO),
    vv(c"exception", VAR_STRING, VV_RO),
    vv(c"throwpoint", VAR_STRING, VV_RO),
    vv(c"register", VAR_STRING, VV_RO),
    vv(c"cmdbang", VAR_NUMBER, VV_RO),
    vv(c"insertmode", VAR_STRING, VV_RO),
    vv(c"val", VAR_UNKNOWN, VV_RO),
    vv(c"key", VAR_UNKNOWN, VV_RO),
    vv(c"profiling", VAR_NUMBER, VV_RO),
    vv(c"fcs_reason", VAR_STRING, VV_RO),
    vv(c"fcs_choice", VAR_STRING, 0),
    vv(c"beval_bufnr", VAR_NUMBER, VV_RO),
    vv(c"beval_winnr", VAR_NUMBER, VV_RO),
    vv(c"beval_winid", VAR_NUMBER, VV_RO),
    vv(c"beval_lnum", VAR_NUMBER, VV_RO),
    vv(c"beval_col", VAR_NUMBER, VV_RO),
    vv(c"beval_text", VAR_STRING, VV_RO),
    vv(c"scrollstart", VAR_STRING, 0),
    vv(c"swapname", VAR_STRING, VV_RO),
    vv(c"swapchoice", VAR_STRING, 0),
    vv(c"swapcommand", VAR_STRING, VV_RO),
    vv(c"char", VAR_STRING, 0),
    vv(c"mouse_win", VAR_NUMBER, 0),
    vv(c"mouse_winid", VAR_NUMBER, 0),
    vv(c"mouse_lnum", VAR_NUMBER, 0),
    vv(c"mouse_col", VAR_NUMBER, 0),
    vv(c"operator", VAR_STRING, VV_RO),
    vv(c"searchforward", VAR_NUMBER, 0),
    vv(c"hlsearch", VAR_NUMBER, 0),
    vv(c"oldfiles", VAR_LIST, 0),
    vv(c"windowid", VAR_NUMBER, VV_RO_SBX),
    vv(c"progpath", VAR_STRING, VV_RO),
    vv(c"completed_item", VAR_DICT, 0),
    vv(c"option_new", VAR_STRING, VV_RO),
    vv(c"option_old", VAR_STRING, VV_RO),
    vv(c"option_oldlocal", VAR_STRING, VV_RO),
    vv(c"option_oldglobal", VAR_STRING, VV_RO),
    vv(c"option_command", VAR_STRING, VV_RO),
    vv(c"option_type", VAR_STRING, VV_RO),
    vv(c"errors", VAR_LIST, 0),
    vv(c"false", VAR_BOOL, VV_RO),
    vv(c"true", VAR_BOOL, VV_RO),
    vv(c"null", VAR_SPECIAL, VV_RO),
    vv(c"numbermax", VAR_NUMBER, VV_RO),
    vv(c"numbermin", VAR_NUMBER, VV_RO),
    vv(c"numbersize", VAR_NUMBER, VV_RO),
    vv(c"vim_did_enter", VAR_NUMBER, VV_RO),
    vv(c"testing", VAR_NUMBER, 0),
    vv(c"t_number", VAR_NUMBER, VV_RO),
    vv(c"t_string", VAR_NUMBER, VV_RO),
    vv(c"t_func", VAR_NUMBER, VV_RO),
    vv(c"t_list", VAR_NUMBER, VV_RO),
    vv(c"t_dict", VAR_NUMBER, VV_RO),
    vv(c"t_float", VAR_NUMBER, VV_RO),
    vv(c"t_bool", VAR_NUMBER, VV_RO),
    vv(c"t_blob", VAR_NUMBER, VV_RO),
    vv(c"event", VAR_DICT, VV_RO),
    vv(c"versionlong", VAR_NUMBER, VV_RO),
    vv(c"echospace", VAR_NUMBER, VV_RO),
    vv(c"argf", VAR_LIST, VV_RO),
    vv(c"argv", VAR_LIST, VV_RO),
    vv(c"collate", VAR_STRING, VV_RO),
    vv(c"exiting", VAR_NUMBER, VV_RO),
    vv(c"maxcol", VAR_NUMBER, VV_RO),
    vv(c"stacktrace", VAR_LIST, VV_RO),
    vv(c"vim_did_init", VAR_NUMBER, VV_RO),
    vv(c"stderr", VAR_NUMBER, VV_RO),
    vv(c"msgpack_types", VAR_DICT, VV_RO),
    vv(c"_null_string", VAR_STRING, VV_RO),
    vv(c"_null_list", VAR_LIST, VV_RO),
    vv(c"_null_dict", VAR_DICT, VV_RO),
    vv(c"_null_blob", VAR_BLOB, VV_RO),
    vv(c"lua", VAR_PARTIAL, VV_RO),
    vv(c"relnum", VAR_NUMBER, VV_RO),
    vv(c"virtnum", VAR_NUMBER, VV_RO),
    vv(c"starttime", VAR_NUMBER, VV_RO),
    vv(c"exitreason", VAR_STRING, VV_RO),
]);
static vimvars_var: GlobalCell<ScopeDictDictItem> = GlobalCell::new(ScopeDictDictItem {
    di_tv: typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    },
    di_flags: 0,
    di_key: [0; 1],
});
static vimvardict: GlobalCell<dict_T> = GlobalCell::new(dict_T {
    dv_lock: VAR_UNLOCKED,
    dv_scope: VAR_NO_SCOPE,
    dv_refcount: 0,
    dv_copyID: 0,
    dv_hashtab: hashtab_T {
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
    },
    dv_copydict: ::core::ptr::null_mut::<dict_T>(),
    dv_used_next: ::core::ptr::null_mut::<dict_T>(),
    dv_used_prev: ::core::ptr::null_mut::<dict_T>(),
    watchers: QUEUE {
        next: ::core::ptr::null_mut::<queue>(),
        prev: ::core::ptr::null_mut::<queue>(),
    },
    lua_table_ref: 0,
});
static msgpack_type_names: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"nil\0".as_ptr() as *const ::core::ffi::c_char,
    b"boolean\0".as_ptr() as *const ::core::ffi::c_char,
    b"integer\0".as_ptr() as *const ::core::ffi::c_char,
    b"float\0".as_ptr() as *const ::core::ffi::c_char,
    b"string\0".as_ptr() as *const ::core::ffi::c_char,
    b"array\0".as_ptr() as *const ::core::ffi::c_char,
    b"map\0".as_ptr() as *const ::core::ffi::c_char,
    b"ext\0".as_ptr() as *const ::core::ffi::c_char,
]);
pub static eval_msgpack_type_lists: GlobalCell<[*const list_T; 8]> = GlobalCell::new([
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
    ::core::ptr::null::<list_T>(),
]);
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SID_LUA: ::core::ffi::c_int = -8 as ::core::ffi::c_int;
pub const SID_STR: ::core::ffi::c_int = -10 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
