#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_uint, c_void};

use crate::api::private::helpers::{cstr_as_string, cstr_to_string};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::charset::{getdigits_int, skiptowhite, skipwhite};
use crate::drawscreen::{UPD_SOME_VALID, redraw_all_later};
use crate::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::eval::entry::tv_init;
use crate::eval::executor::eexe_mod_op;
use crate::eval::funcs::{tv_get_buf, tv_get_buf_from_arg};
use crate::eval::typval::{
    TV_INITIAL_VALUE, queue_init, tv_check_str_or_nr, tv_clear, tv_copy, tv_dict_add,
    tv_dict_alloc, tv_dict_alloc_lock, tv_dict_hi2di, tv_dict_is_watched, tv_dict_item_alloc,
    tv_dict_item_alloc_len, tv_dict_item_key, tv_dict_item_remove, tv_dict_set_keys_readonly,
    tv_dict_set_ret, tv_dict_unref, tv_dict_watcher_notify, tv_free, tv_get_bool_chk,
    tv_get_number, tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_get_string_chk,
    tv_ht_iter, tv_is_func, tv_item_lock, tv_list_alloc, tv_list_append_allocated_string,
    tv_list_append_string, tv_list_append_tv, tv_list_find_nr, tv_list_find_str, tv_list_first,
    tv_list_free, tv_list_item_remove, tv_list_len, tv_list_locked, tv_list_ref,
    tv_list_remove_items, tv_list_set_lock, tv_list_set_ret, value_check_lock,
};
use crate::eval::userfunc::{
    find_hi_in_scoped_ht, find_var_in_scoped_ht, function_exists, get_current_funccal_dict,
    get_funccal_args_dict, get_funccal_args_ht, get_funccal_args_var, get_funccal_local_dict,
    get_funccal_local_ht, get_funccal_local_var, list_func_vars,
};
use crate::eval::window::{find_win_by_nr, restore_win, switch_win};
use crate::eval::{
    clear_evalarg, clear_lval, eval_expr_ext, eval_isnamec1, eval_lavars_used, eval_option,
    eval_to_bool, eval_to_string, eval0, eval1, fill_evalarg_from_eap, find_name_end,
    find_option_var_end, get_env_len, get_lval, get_name_len, handle_subscript,
    may_call_simple_func, num_divide, num_modulus, set_ref_in_ht, set_var_lval, skip_expr,
};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{check_nextcmd, ends_excmd};
use crate::ex_eval::aborting;
use crate::garray::{ga_append, ga_clear, ga_concat, ga_concat_len, ga_init};
use crate::global_cell::GlobalCell;
use crate::hashtab::{
    hash_add, hash_clear, hash_find, hash_find_len, hash_init, hash_lock, hash_remove, hash_unlock,
};
use crate::lua::executor::nlua_set_sctx;
use crate::main::{
    EVALARG_EVALUATE, called_emsg, curbuf, current_sctx, curtab, curwin, did_emsg,
    e_cannot_change_readonly_variable_str, e_cannot_delete_variable_str, e_cannot_mod,
    e_cannot_set_variable_in_sandbox_str, e_illvar, e_invarg, e_invarg2, e_letwrong, e_listreq,
    e_missing_close_curly_str, e_stray_closing_curly_str, e_string_required, e_trailing_arg,
    e_unknown_option2, emsg_severe, firstwin, got_int, lastused_tabpage, no_hlsearch, p_ccv, p_dex,
    p_pex, p_verbose, sandbox, sc_col,
};
use crate::mbyte::utf_char2bytes;
use crate::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xstrdup, xstrlcat, xstrlcpy, xstrndup,
};
use crate::message::{
    emsg, internal_error, message_filtered, msg_advance, msg_clr_eos, msg_ext_set_kind,
    msg_outtrans, msg_putchar, msg_puts, msg_puts_len, msg_start,
};
use crate::option::{
    find_option, get_option, get_option_sctx, get_option_value, get_tty_option, get_winbuf_options,
    is_option_hidden, is_tty_option, kOptFlagFunc, option_has_type, optval_free,
    set_option_value_handle_tty,
};
use crate::options::{
    kOptAleph, kOptCharconvert, kOptDiffexpr, kOptInvalid, kOptPatchexpr, kOptSpellsuggest,
};
use crate::os::cshim::{__ctype_b_loc, gettext, snprintf, strncmp};
use crate::os::env::{vim_getenv, vim_setenv_ext, vim_unsetenv_ext};
use crate::pos::MAXCOL;
use crate::register::{get_reg_contents, write_reg_contents};
use crate::runtime::{new_script_item, script_autoload, script_items};
use crate::search::set_search_direction;
use crate::strings::{concat_str, vim_strchr};
use crate::types::{
    BoolVarValue, CMD_const, CMD_lockvar, EvalFuncData, GRegFlags, OptIndex, OptInt, OptVal,
    OptValData, OptValType, QUEUE, ScopeDictDictItem, ScopeType, SpecialVarValue, VAR_BLOB,
    VAR_BOOL, VAR_DEF_SCOPE, VAR_DICT, VAR_FIXED, VAR_FLOAT, VAR_FUNC, VAR_LIST, VAR_NO_SCOPE,
    VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SPECIAL, VAR_STRING, VAR_TYPE_BLOB, VAR_TYPE_BOOL,
    VAR_TYPE_DICT, VAR_TYPE_FLOAT, VAR_TYPE_FUNC, VAR_TYPE_LIST, VAR_TYPE_NUMBER, VAR_TYPE_STRING,
    VAR_UNKNOWN, VAR_UNLOCKED, VarType, VimVarFlags, Vv, aco_save_T, buf_T, dict_T, dictitem_T,
    evalarg_T, exarg_T, expand_T, garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarFalse,
    kBoolVarTrue, kListLenUnknown, kSpecialVarNull, list_T, listitem_T, lval_T, partial_T,
    ptrdiff_t, scid_T, scriptitem_T, scriptvar_T, size_t, ssize_t, switchwin_T, tabpage_T,
    typval_T, typval_vval_union, uint8_t, uint32_t, varnumber_T, win_T,
};
use crate::version::{highest_patch, min_vim_version};
use crate::window::{find_tabpage, goto_tabpage_tp, prevwin_curwin, valid_tabpage};
use ::libc::{abort, memchr, memcpy, strcmp, strcpy, strlen};

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
/// `__ctype_b_loc()`'s lower-case bit, the one `islower()` reads.
pub const _ISlower: c_uint = 512;

/// The reference count `init_var_dict` gives a scope dictionary: high enough
/// that nothing ever frees one.
pub const DO_NOT_FREE_CNT: c_int = 1073741823;

/// `dictitem_T::di_flags`.
pub const DI_FLAGS_ALLOC: uint8_t = 16;
pub const DI_FLAGS_LOCK: uint8_t = 8;
pub const DI_FLAGS_FIX: uint8_t = 4;
pub const DI_FLAGS_RO_SBX: uint8_t = 2;
pub const DI_FLAGS_RO: uint8_t = 1;

pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const kOptValTypeNil: OptValType = -1;

/// `get_lval`'s "do not report" flag.
pub const GLV_QUIET: c_int = 2;

/// One `v:` variable: a `dictitem_T` whose flexible key member is spelled
/// out at the longest name the table holds (`VIMVAR_KEY_LEN`, 16, plus the
/// NUL), so that the whole table can be a `static`.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VimVarItem {
    pub di_tv: typval_T,
    pub di_flags: uint8_t,
    pub di_key: [c_char; 17],
}

/// One row of the `v:` table.
#[derive(Copy, Clone)]
pub struct VimVar {
    pub vv_name: *mut c_char,
    pub vv_di: VimVarItem,
    pub vv_flags: c_char,
}

pub const kGRegExprSrc: GRegFlags = 2;

/// What `ex_unletlock` does to each argument it resolves: `do_unlet_var` or
/// `do_lock_var`.  The two are written together because the walk that finds
/// the arguments is what makes `:unlet` and `:lockvar` agree.
pub type ex_unletlock_callback = unsafe fn(*mut lval_T, *mut c_char, *mut exarg_T, c_int) -> c_int;

pub const NULL: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub const INT64_MIN: ::core::ffi::c_long = -9223372036854775807 - 1;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615;

pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const BAD_KEEP: c_int = -1;
pub const BAD_DROP: c_int = -2;
pub const FORCE_BIN: c_int = 1;
pub const FORCE_NOBIN: c_int = 2;
pub const NOTDONE: c_int = 2;
pub const CHAN_STDERR: c_int = 2;
pub const FNE_INCL_BR: c_int = 1;
pub const FNE_CHECK_START: c_int = 2;
pub const AUTOLOAD_CHAR: c_char = b'#' as c_char;

/// The two `name_len` sentinels the `var_check_*` family accepts in place of
/// a real length: translate the name and measure it, or just measure it.
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX - 1;

/// A zeroed `lval_T`, which is what `get_lval` expects to be handed.
pub(crate) const LVAL_INITIAL_VALUE: lval_T = lval_T {
    ll_name: ::core::ptr::null(),
    ll_name_len: 0,
    ll_exp_name: ::core::ptr::null_mut(),
    ll_tv: ::core::ptr::null_mut(),
    ll_li: ::core::ptr::null_mut(),
    ll_list: ::core::ptr::null_mut(),
    ll_range: false,
    ll_empty2: false,
    ll_n1: 0,
    ll_n2: 0,
    ll_dict: ::core::ptr::null_mut(),
    ll_di: ::core::ptr::null_mut(),
    ll_newkey: ::core::ptr::null_mut(),
    ll_blob: ::core::ptr::null_mut(),
};

/// How deep `:const` locks the value it stores.
pub const DICT_MAXNEST: c_int = 100;

pub const SID_LUA: c_int = -8;
pub const SID_STR: c_int = -10;

// The error texts this family owns.  They are `%`-format strings handed to
// the variadic `semsg`/`emsg`, so they stay C strings rather than becoming
// `semsg!` arguments.
pub const e_letunexp: &CStr = c"E18: Unexpected characters in :let";
pub const e_double_semicolon_in_list_of_variables: &CStr = c"E452: Double ; in list of variables";
pub const e_lock_unlock: &CStr = c"E940: Cannot lock or unlock variable %s";
pub const e_setting_v_str_to_value_with_wrong_type: &CStr =
    c"E963: Setting v:%s to value with wrong type";
pub const e_missing_end_marker_str: &CStr = c"E990: Missing end marker '%s'";
pub const e_cannot_use_heredoc_here: &CStr = c"E991: Cannot use =<< here";

/// The `scriptvar_T` of script `sid`: upstream's `SCRIPT_SV`.
///
/// # Safety
/// `sid` is a live script id -- `1 ..= script_items.ga_len`.
pub(crate) unsafe fn script_sv(sid: c_int) -> *mut scriptvar_T {
    unsafe {
        (**((*script_items.ptr()).ga_data as *mut *mut scriptitem_T).offset((sid - 1) as isize))
            .sn_vars
    }
}

/// A `hashtab_T` before `hash_init`: the shape every zeroed table has, and
/// what the three `static` ones below start as.
const EMPTY_HASHTAB: hashtab_T = hashtab_T {
    ht_mask: 0,
    ht_used: 0,
    ht_filled: 0,
    ht_changed: 0,
    ht_locked: 0,
    ht_array: ::core::ptr::null_mut(),
    ht_smallarray: [hashitem_T {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut(),
    }; 16],
};

/// A scope dictionary before `init_var_dict`.
const EMPTY_SCOPE_DICT: dict_T = dict_T {
    dv_lock: VAR_UNLOCKED,
    dv_scope: VAR_NO_SCOPE,
    dv_refcount: 0,
    dv_copyID: 0,
    dv_hashtab: EMPTY_HASHTAB,
    dv_copydict: ::core::ptr::null_mut(),
    dv_used_next: ::core::ptr::null_mut(),
    dv_used_prev: ::core::ptr::null_mut(),
    watchers: QUEUE {
        next: ::core::ptr::null_mut(),
        prev: ::core::ptr::null_mut(),
    },
    lua_table_ref: 0,
};

/// The `dictitem_T` a scope dictionary is reached through -- what
/// `find_var_in_ht` answers for a bare `g:` or `v:`.  Its key is the empty
/// string; `init_var_dict` fills the rest in.
const EMPTY_SCOPE_VAR: ScopeDictDictItem = ScopeDictDictItem {
    di_tv: TV_INITIAL_VALUE,
    di_flags: 0,
    di_key: [0; 1],
};

static globvars_var: GlobalCell<ScopeDictDictItem> = GlobalCell::new(EMPTY_SCOPE_VAR);
static globvardict: GlobalCell<dict_T> = GlobalCell::new(EMPTY_SCOPE_DICT);

/// The names that mean `v:version` in every scope: upstream's
/// `compat_hashtab`, which `evalvars_init` fills from the `VimVarFlags::COMPAT` rows.
static compat_hashtab: GlobalCell<hashtab_T> = GlobalCell::new(EMPTY_HASHTAB);

const fn vv(name: &'static CStr, v_type: VarType, vv_flags: VimVarFlags) -> VimVar {
    VimVar {
        vv_name: name.as_ptr().cast_mut(),
        vv_di: VimVarItem {
            di_tv: typval_T {
                v_type,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
            di_flags: 0,
            di_key: [0; 17],
        },
        vv_flags: vv_flags.bits() as c_char,
    }
}

static vimvars: GlobalCell<[VimVar; 106]> = GlobalCell::new([
    vv(c"count", VAR_NUMBER, VimVarFlags::RO),
    vv(c"count1", VAR_NUMBER, VimVarFlags::RO),
    vv(c"prevcount", VAR_NUMBER, VimVarFlags::RO),
    vv(c"errmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"warningmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"statusmsg", VAR_STRING, VimVarFlags::NONE),
    vv(c"shell_error", VAR_NUMBER, VimVarFlags::RO),
    vv(c"this_session", VAR_STRING, VimVarFlags::NONE),
    vv(
        c"version",
        VAR_NUMBER,
        VimVarFlags::COMPAT.or(VimVarFlags::RO),
    ),
    vv(c"lnum", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"termrequest", VAR_STRING, VimVarFlags::RO),
    vv(c"termresponse", VAR_STRING, VimVarFlags::RO),
    vv(c"fname", VAR_STRING, VimVarFlags::RO),
    vv(c"lang", VAR_STRING, VimVarFlags::RO),
    vv(c"lc_time", VAR_STRING, VimVarFlags::RO),
    vv(c"ctype", VAR_STRING, VimVarFlags::RO),
    vv(c"charconvert_from", VAR_STRING, VimVarFlags::RO),
    vv(c"charconvert_to", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_in", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_out", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_new", VAR_STRING, VimVarFlags::RO),
    vv(c"fname_diff", VAR_STRING, VimVarFlags::RO),
    vv(c"cmdarg", VAR_STRING, VimVarFlags::RO),
    vv(c"foldstart", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"foldend", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"folddashes", VAR_STRING, VimVarFlags::RO_SBX),
    vv(c"foldlevel", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"progname", VAR_STRING, VimVarFlags::RO),
    vv(c"servername", VAR_STRING, VimVarFlags::RO),
    vv(c"dying", VAR_NUMBER, VimVarFlags::RO),
    vv(c"exception", VAR_STRING, VimVarFlags::RO),
    vv(c"throwpoint", VAR_STRING, VimVarFlags::RO),
    vv(c"register", VAR_STRING, VimVarFlags::RO),
    vv(c"cmdbang", VAR_NUMBER, VimVarFlags::RO),
    vv(c"insertmode", VAR_STRING, VimVarFlags::RO),
    vv(c"val", VAR_UNKNOWN, VimVarFlags::RO),
    vv(c"key", VAR_UNKNOWN, VimVarFlags::RO),
    vv(c"profiling", VAR_NUMBER, VimVarFlags::RO),
    vv(c"fcs_reason", VAR_STRING, VimVarFlags::RO),
    vv(c"fcs_choice", VAR_STRING, VimVarFlags::NONE),
    vv(c"beval_bufnr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_winnr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_winid", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_lnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_col", VAR_NUMBER, VimVarFlags::RO),
    vv(c"beval_text", VAR_STRING, VimVarFlags::RO),
    vv(c"scrollstart", VAR_STRING, VimVarFlags::NONE),
    vv(c"swapname", VAR_STRING, VimVarFlags::RO),
    vv(c"swapchoice", VAR_STRING, VimVarFlags::NONE),
    vv(c"swapcommand", VAR_STRING, VimVarFlags::RO),
    vv(c"char", VAR_STRING, VimVarFlags::NONE),
    vv(c"mouse_win", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_winid", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_lnum", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"mouse_col", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"operator", VAR_STRING, VimVarFlags::RO),
    vv(c"searchforward", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"hlsearch", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"oldfiles", VAR_LIST, VimVarFlags::NONE),
    vv(c"windowid", VAR_NUMBER, VimVarFlags::RO_SBX),
    vv(c"progpath", VAR_STRING, VimVarFlags::RO),
    vv(c"completed_item", VAR_DICT, VimVarFlags::NONE),
    vv(c"option_new", VAR_STRING, VimVarFlags::RO),
    vv(c"option_old", VAR_STRING, VimVarFlags::RO),
    vv(c"option_oldlocal", VAR_STRING, VimVarFlags::RO),
    vv(c"option_oldglobal", VAR_STRING, VimVarFlags::RO),
    vv(c"option_command", VAR_STRING, VimVarFlags::RO),
    vv(c"option_type", VAR_STRING, VimVarFlags::RO),
    vv(c"errors", VAR_LIST, VimVarFlags::NONE),
    vv(c"false", VAR_BOOL, VimVarFlags::RO),
    vv(c"true", VAR_BOOL, VimVarFlags::RO),
    vv(c"null", VAR_SPECIAL, VimVarFlags::RO),
    vv(c"numbermax", VAR_NUMBER, VimVarFlags::RO),
    vv(c"numbermin", VAR_NUMBER, VimVarFlags::RO),
    vv(c"numbersize", VAR_NUMBER, VimVarFlags::RO),
    vv(c"vim_did_enter", VAR_NUMBER, VimVarFlags::RO),
    vv(c"testing", VAR_NUMBER, VimVarFlags::NONE),
    vv(c"t_number", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_string", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_func", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_list", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_dict", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_float", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_bool", VAR_NUMBER, VimVarFlags::RO),
    vv(c"t_blob", VAR_NUMBER, VimVarFlags::RO),
    vv(c"event", VAR_DICT, VimVarFlags::RO),
    vv(c"versionlong", VAR_NUMBER, VimVarFlags::RO),
    vv(c"echospace", VAR_NUMBER, VimVarFlags::RO),
    vv(c"argf", VAR_LIST, VimVarFlags::RO),
    vv(c"argv", VAR_LIST, VimVarFlags::RO),
    vv(c"collate", VAR_STRING, VimVarFlags::RO),
    vv(c"exiting", VAR_NUMBER, VimVarFlags::RO),
    vv(c"maxcol", VAR_NUMBER, VimVarFlags::RO),
    vv(c"stacktrace", VAR_LIST, VimVarFlags::RO),
    vv(c"vim_did_init", VAR_NUMBER, VimVarFlags::RO),
    vv(c"stderr", VAR_NUMBER, VimVarFlags::RO),
    vv(c"msgpack_types", VAR_DICT, VimVarFlags::RO),
    vv(c"_null_string", VAR_STRING, VimVarFlags::RO),
    vv(c"_null_list", VAR_LIST, VimVarFlags::RO),
    vv(c"_null_dict", VAR_DICT, VimVarFlags::RO),
    vv(c"_null_blob", VAR_BLOB, VimVarFlags::RO),
    vv(c"lua", VAR_PARTIAL, VimVarFlags::RO),
    vv(c"relnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"virtnum", VAR_NUMBER, VimVarFlags::RO),
    vv(c"starttime", VAR_NUMBER, VimVarFlags::RO),
    vv(c"exitreason", VAR_STRING, VimVarFlags::RO),
]);
static vimvars_var: GlobalCell<ScopeDictDictItem> = GlobalCell::new(EMPTY_SCOPE_VAR);
static vimvardict: GlobalCell<dict_T> = GlobalCell::new(EMPTY_SCOPE_DICT);

/// The eight `v:msgpack_types` keys, in `MessagePackType` order.
const msgpack_type_names: [&CStr; 8] = [
    c"nil", c"boolean", c"integer", c"float", c"string", c"array", c"map", c"ext",
];

/// The eight `v:msgpack_types` lists themselves, which the msgpack encoder
/// and decoder compare against by identity.
pub static eval_msgpack_type_lists: GlobalCell<[*const list_T; 8]> =
    GlobalCell::new([::core::ptr::null(); 8]);
